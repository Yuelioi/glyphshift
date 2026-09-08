//! Persistent Label updates are owned by the target UI thread. Control-plane
//! callbacks only publish epochs; generational VGUI handles guard later writes.
use crate::{memory, panel_abi, platform, refresh_abi};
use std::{
    collections::BTreeMap,
    ffi::c_void,
    sync::{
        atomic::{AtomicU32, AtomicU64, AtomicUsize, Ordering},
        Condvar, Mutex, OnceLock,
    },
};
use windows_sys::Win32::{
    Foundation::{GetLastError, SetLastError},
    System::{LibraryLoader::*, Threading::GetCurrentThreadId},
};

type Callback = unsafe extern "thiscall" fn(*mut c_void, usize, usize, usize, usize) -> usize;
type Setter = unsafe extern "thiscall" fn(*mut c_void, *const u16, u8);
struct Installed {
    object: usize,
    shadow: usize,
    original: Callback,
    module: usize,
    client_slot: usize,
    ivgui: usize,
    handles: refresh_abi::Handles,
}
static INSTALLED: OnceLock<Installed> = OnceLock::new();
static MODE: AtomicU32 = AtomicU32::new(0); // 0 inactive, 1 active, 2 restoring
static EPOCH: AtomicU64 = AtomicU64::new(1);
static THREAD: AtomicU32 = AtomicU32::new(0);
static STATE: Mutex<State> = Mutex::new(State {
    owners: BTreeMap::new(),
    layouts: BTreeMap::new(),
});
static RESTORED: Condvar = Condvar::new();
thread_local! {static BUSY:std::cell::Cell<bool>=const{std::cell::Cell::new(false)};}

#[derive(Clone, Copy)]
struct Layout {
    member: usize,
    image_table: usize,
    image: refresh_abi::ImageShape,
    setter: usize,
}
#[derive(Clone)]
struct Owner {
    vpanel: usize,
    object: usize,
    table: usize,
    image: usize,
    symbol: usize,
    layout: Layout,
    source: Vec<u16>,
    last: Vec<u16>,
    epoch: u64,
}
struct State {
    owners: BTreeMap<u32, Owner>,
    layouts: BTreeMap<usize, Option<Layout>>,
}

pub fn request() {
    EPOCH.fetch_add(1, Ordering::AcqRel);
}
pub fn activate() {
    // Query support remains available on framework builds outside these proofs.
    if install().is_ok() {
        MODE.store(1, Ordering::Release);
        request();
    }
}
pub fn ready() -> bool {
    MODE.load(Ordering::Acquire) != 2
}
pub fn deactivate() -> Result<(), ()> {
    MODE.store(2, Ordering::Release);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
    let state = loop {
        match STATE.try_lock() {
            Ok(state) => break state,
            Err(std::sync::TryLockError::Poisoned(_)) => return Err(()),
            Err(std::sync::TryLockError::WouldBlock) if std::time::Instant::now() < deadline => {
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
            Err(_) => return Err(()),
        }
    };
    if state.owners.is_empty() {
        MODE.store(0, Ordering::Release);
        return Ok(());
    }
    let (state, _) = RESTORED
        .wait_timeout_while(
            state,
            deadline.saturating_duration_since(std::time::Instant::now()),
            |s| !s.owners.is_empty(),
        )
        .map_err(|_| ())?;
    if !state.owners.is_empty() {
        return Err(());
    }
    MODE.store(0, Ordering::Release);
    Ok(())
}

fn module(address: usize) -> Option<usize> {
    Some(memory::region(address)?.AllocationBase as usize)
}
fn code(address: usize, n: usize) -> Option<Vec<u8>> {
    memory::code(address, module(address)?, n)
}
fn table(address: usize, limit: usize) -> Option<(Vec<usize>, Vec<Vec<u8>>)> {
    let mut entries = Vec::new();
    let mut methods = Vec::new();
    for n in 0..limit {
        let Some(entry) = memory::word(address.checked_add(n * 4)?) else {
            break;
        };
        // An adjacent RTTI/data word can happen to point into another module's
        // executable range. It is not another member of this framework table.
        if module(entry) != module(address) {
            break;
        }
        let Some(bytes) = code(entry, 2048) else {
            break;
        };
        entries.push(entry);
        methods.push(bytes);
    }
    (!entries.is_empty()).then_some((entries, methods))
}
fn factory(module: usize, name: &std::ffi::CStr) -> Option<usize> {
    let f = unsafe { GetProcAddress(module as _, c"CreateInterface".as_ptr().cast()) }?;
    let f: unsafe extern "C" fn(*const u8, *mut i32) -> *mut c_void =
        unsafe { std::mem::transmute(f) };
    let object = unsafe { f(name.as_ptr().cast(), std::ptr::null_mut()) } as usize;
    (object != 0 && object.is_multiple_of(4)).then_some(object)
}
fn install() -> Result<(), ()> {
    if let Some(i) = INSTALLED.get() {
        return (memory::word(i.object) == Some(i.shadow))
            .then_some(())
            .ok_or(());
    }
    let name = "vgui2.dll"
        .encode_utf16()
        .chain(Some(0))
        .collect::<Vec<_>>();
    let module = unsafe { GetModuleHandleW(name.as_ptr()) } as usize;
    if module == 0 {
        return Err(());
    }
    let object = factory(module, c"VGUI_Panel009").ok_or(())?;
    let region = memory::region(object).ok_or(())?;
    use windows_sys::Win32::System::Memory::{PAGE_READWRITE, PAGE_WRITECOPY};
    if !matches!(region.Protect & 0xff, PAGE_READWRITE | PAGE_WRITECOPY)
        || object.checked_add(4).ok_or(())?
            > (region.BaseAddress as usize)
                .checked_add(region.RegionSize)
                .ok_or(())?
    {
        return Err(());
    }
    let original = memory::word(object).ok_or(())?;
    let (entries, methods) = table(original, 96).ok_or(())?;
    if entries.len() == 96 || entries.iter().any(|e| self::module(*e) != Some(module)) {
        return Err(());
    }
    let boundary = panel_abi::identify(&methods).ok_or(())?;
    let ivgui = factory(module, c"VGUI_ivgui008").ok_or(())?;
    let (_, methods) = table(memory::word(ivgui).ok_or(())?, 64).ok_or(())?;
    let handles = refresh_abi::handles(&methods).ok_or(())?;
    let mut shadow = Vec::with_capacity(entries.len() + 1);
    shadow.push(memory::word(original.checked_sub(4).ok_or(())?).ok_or(())?);
    shadow.extend_from_slice(&entries);
    shadow[boundary.callback + 1] = callback as *const () as usize;
    platform::pin(module)?;
    platform::pin(callback as *const () as usize)?;
    let ptr = Box::leak(shadow.into_boxed_slice()).as_mut_ptr();
    let shadow = unsafe { ptr.add(1) } as usize;
    let i = Installed {
        object,
        shadow,
        module,
        original: unsafe { std::mem::transmute::<usize, Callback>(entries[boundary.callback]) },
        client_slot: boundary.vpanel_client_getter,
        ivgui,
        handles,
    };
    INSTALLED.set(i).map_err(|_| ())?;
    unsafe { &*(object as *const AtomicUsize) }
        .compare_exchange(original, shadow, Ordering::AcqRel, Ordering::Acquire)
        .map_err(|_| ())?;
    Ok(())
}

fn typename(descriptor: usize) -> Option<Vec<u8>> {
    let bytes = memory::read(descriptor.checked_add(8)?, 128)?;
    let n = bytes.iter().position(|b| *b == 0)?;
    Some(bytes[..n].to_vec())
}
fn locator(table: usize) -> Option<usize> {
    let col = memory::word(table.checked_sub(4)?)?;
    (memory::word(col) == Some(0) && memory::word(col + 8) == Some(0)).then_some(col)
}
fn is_image(table: usize) -> bool {
    (|| Some(typename(memory::word(locator(table)? + 12)?)? == b".?AVTextImage@vgui@@"))()
        == Some(true)
}
fn is_label(table: usize) -> Option<()> {
    let col = locator(table)?;
    if memory::word(col + 4) != Some(0) {
        return None;
    }
    let hierarchy = memory::word(col + 16)?;
    let count = memory::word(hierarchy + 8)?;
    let bases = memory::word(hierarchy + 12)?;
    if count > 128 {
        return None;
    }
    for n in 0..count {
        let b = memory::word(bases + n * 4)?;
        if typename(memory::word(b)?)? == b".?AVLabel@vgui@@"
            && memory::word(b + 8) == Some(0)
            && memory::word(b + 12) == Some(u32::MAX as usize)
        {
            return Some(());
        }
    }
    None
}
fn field(object: usize, slot: usize, owner: usize) -> Option<usize> {
    let vtable = memory::word(object)?;
    let entry = memory::word(vtable.checked_add(slot * 4)?)?;
    let bytes = memory::code(entry, owner, 16)?;
    let offset = panel_abi::member_getter(&bytes)?;
    memory::word(object.checked_add(offset)?)
}
fn resolve(i: &Installed, handle: u32) -> Option<usize> {
    if handle == u32::MAX {
        return None;
    }
    let index = (handle & 0xfffff) as usize;
    let count = memory::word(i.ivgui + i.handles.count)?;
    if count > 0x100000 || index >= count {
        return None;
    }
    let entries = memory::word(i.ivgui + i.handles.entries)?;
    let at = entries.checked_add(index.checked_mul(8)?)?;
    let serial = memory::word(at)?;
    if serial & 0x80000000 != 0 || serial & 0x7fffffff != (handle >> 20) as usize {
        return None;
    }
    let panel = memory::word(at + 4)?;
    (panel != 0).then_some(panel)
}
fn owner(i: &Installed, vpanel: usize) -> Option<(u32, usize, usize)> {
    let handle = field(vpanel, i.handles.panel_slot, i.module)? as u32;
    if resolve(i, handle) != Some(vpanel) {
        return None;
    }
    let client = field(vpanel, i.client_slot, i.module)?;
    let col = locator(memory::word(client)?)?;
    let adjustment = memory::word(col + 4)?;
    if adjustment > 4096 {
        return None;
    }
    let object = client.checked_sub(adjustment)?;
    let vtable = memory::word(object)?;
    Some((handle, object, vtable))
}
fn layout(object: usize, vtable: usize) -> Option<Layout> {
    is_label(vtable)?;
    let (entries, methods) = table(vtable, 256)?;
    let mut result = None;
    for getter in &methods {
        let Some(member) = panel_abi::member_getter(getter) else {
            continue;
        };
        let Some(image_object) = memory::word(object.checked_add(member)?) else {
            continue;
        };
        let Some(image_table) = memory::word(image_object) else {
            continue;
        };
        if !is_image(image_table) {
            continue;
        }
        let (_, image_methods) = table(image_table, 64)?;
        let image = refresh_abi::image(&image_methods)?;
        let slot = refresh_abi::label_setter(&methods, member, image)?;
        platform::pin(entries[slot]).ok()?;
        platform::pin(image_table).ok()?;
        if result.is_some() {
            return None;
        }
        result = Some(Layout {
            member,
            image_table,
            image,
            setter: entries[slot],
        });
    }
    result
}
fn text(owner: &Owner) -> Option<Vec<u16>> {
    if memory::word(owner.object) != Some(owner.table)
        || memory::word(owner.object + owner.layout.member) != Some(owner.image)
        || memory::word(owner.image) != Some(owner.layout.image_table)
        || memory::word(owner.image + owner.layout.image.symbol) != Some(owner.symbol)
    {
        return None;
    }
    memory::utf16(memory::word(owner.image + owner.layout.image.text)?, 256)
}
fn valid(i: &Installed, handle: u32, o: &Owner) -> bool {
    resolve(i, handle) == Some(o.vpanel) && owner(i, o.vpanel) == Some((handle, o.object, o.table))
}
fn write(o: &Owner, value: &[u16]) -> Option<()> {
    if text(o).as_deref() != Some(o.last.as_slice()) {
        return None;
    }
    let value = value.iter().copied().chain(Some(0)).collect::<Vec<_>>();
    let setter: Setter = unsafe { std::mem::transmute(o.layout.setter) };
    unsafe { setter(o.object as _, value.as_ptr(), 0) };
    (text(o).as_deref() == Some(&value[..value.len() - 1])).then_some(())
}
fn update(i: &Installed, handle: u32, o: &mut Owner, restore: bool, epoch: u64) -> bool {
    update_with(i, handle, o, restore, epoch, platform::refresh_decision)
}
fn update_with(
    i: &Installed,
    handle: u32,
    o: &mut Owner,
    restore: bool,
    epoch: u64,
    decide: impl FnOnce(&[u16]) -> Option<Vec<u16>>,
) -> bool {
    if !valid(i, handle, o) || text(o).as_deref() != Some(o.last.as_slice()) {
        return false;
    }
    let desired = if restore {
        o.source.clone()
    } else {
        let Some(value) = decide(&o.source) else {
            return true;
        };
        value
    };
    if !restore && (MODE.load(Ordering::Acquire) != 1 || EPOCH.load(Ordering::Acquire) != epoch) {
        return true;
    }
    if desired != o.last {
        if write(o, &desired).is_none() {
            return false;
        }
        o.last = desired;
    }
    o.epoch = epoch;
    !restore
}

fn tick(i: &Installed, vpanel: usize) {
    let mode = MODE.load(Ordering::Acquire);
    if mode == 0 {
        return;
    }
    let thread = unsafe { GetCurrentThreadId() };
    let previous = THREAD
        .compare_exchange(0, thread, Ordering::AcqRel, Ordering::Acquire)
        .unwrap_or_else(|v| v);
    if previous != 0 && previous != thread {
        return;
    }
    let Ok(mut s) = STATE.lock() else { return };
    let epoch = EPOCH.load(Ordering::Acquire);
    let restore = MODE.load(Ordering::Acquire) == 2;
    // This visits hidden owners too, through serial-checked framework handles.
    s.owners.retain(|h, o| {
        if restore || o.epoch != epoch {
            update(i, *h, o, restore, epoch)
        } else {
            true
        }
    });
    if restore {
        if s.owners.is_empty() {
            RESTORED.notify_all();
        }
        return;
    }
    if MODE.load(Ordering::Acquire) != 1 {
        return;
    }
    let Some((handle, object, vtable)) = owner(i, vpanel) else {
        return;
    };
    if let Some(o) = s.owners.get(&handle) {
        if text(o).as_deref() == Some(o.last.as_slice()) {
            return;
        }
        s.owners.remove(&handle);
    }
    if s.owners.len() >= 4096 || s.layouts.len() >= 256 {
        return;
    }
    let layout = *s
        .layouts
        .entry(vtable)
        .or_insert_with(|| layout(object, vtable));
    let Some(layout) = layout else { return };
    let Some(image) = memory::word(object + layout.member) else {
        return;
    };
    let Some(symbol) = memory::word(image + layout.image.symbol) else {
        return;
    };
    if symbol == u32::MAX as usize {
        return;
    }
    let Some(source) = platform::original_index(symbol as u32) else {
        return;
    };
    if !platform::plain_label(&source) {
        return;
    }
    let mut o = Owner {
        vpanel,
        object,
        table: vtable,
        image,
        symbol,
        layout,
        source,
        last: Vec::new(),
        epoch: 0,
    };
    let Some(current) = text(&o) else { return };
    if current != o.source && !platform::returned_pair(&o.source, &current) {
        return;
    }
    o.last = current;
    if update(i, handle, &mut o, false, epoch) {
        s.owners.insert(handle, o);
    }
}
unsafe extern "thiscall" fn callback(
    this: *mut c_void,
    panel: usize,
    a: usize,
    b: usize,
    c: usize,
) -> usize {
    let error = GetLastError();
    let i = INSTALLED.get().expect("published before callback");
    let nested = BUSY.with(|busy| busy.replace(true));
    if !nested && this as usize == i.object {
        let _ = std::panic::catch_unwind(|| tick(i, panel));
    }
    BUSY.with(|busy| busy.set(nested));
    SetLastError(error);
    (i.original)(this, panel, a, b, c)
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows_sys::Win32::System::Memory::*;

    unsafe extern "thiscall" fn setter(image_owner: *mut c_void, value: *const u16, _clear: u8) {
        let image = *((image_owner as *const usize).add(1));
        let buffer = *((image as *const usize).add(1)) as *mut u16;
        let text = memory::utf16(value as usize, 256).unwrap();
        std::ptr::copy_nonoverlapping(value, buffer, text.len() + 1);
    }

    #[test]
    fn retained_owner_generations_external_edits_reuse_hidden_restore_and_timeout() {
        // Synthetic ABI objects; the two getters are inspected, never executed.
        let code = unsafe {
            VirtualAlloc(
                std::ptr::null(),
                4096,
                MEM_COMMIT | MEM_RESERVE,
                PAGE_EXECUTE_READWRITE,
            )
        };
        assert!(!code.is_null());
        unsafe {
            std::ptr::copy_nonoverlapping(
                [0x8bu8, 0x41, 4, 0xc3, 0x8b, 0x41, 8, 0xc3].as_ptr(),
                code.cast(),
                8,
            );
        }
        let mut buffer = vec![0u16; 257];
        buffer[..5].copy_from_slice(&[79, 112, 101, 110, 0]);
        let image_table = [0usize];
        let image = [
            image_table.as_ptr() as usize,
            buffer.as_mut_ptr() as usize,
            7,
        ];
        let col = [0usize; 5];
        let owner_table = [col.as_ptr() as usize, 0];
        let object = [
            unsafe { owner_table.as_ptr().add(1) } as usize,
            image.as_ptr() as usize,
        ];
        let panel_table = [code as usize, code as usize + 4];
        let handle = 2u32 << 20;
        let panel = [
            panel_table.as_ptr() as usize,
            handle as usize,
            object.as_ptr() as usize,
        ];
        let mut entry = [2usize, panel.as_ptr() as usize];
        let ivgui = [entry.as_mut_ptr() as usize, 1];
        let i = Installed {
            object: 0,
            shadow: 0,
            original: callback,
            module: code as usize,
            client_slot: 1,
            ivgui: ivgui.as_ptr() as usize,
            handles: refresh_abi::Handles {
                entries: 0,
                count: 4,
                panel_slot: 0,
            },
        };
        let mut o = Owner {
            vpanel: panel.as_ptr() as usize,
            object: object.as_ptr() as usize,
            table: object[0],
            image: image.as_ptr() as usize,
            symbol: 7,
            layout: Layout {
                member: 4,
                image_table: image_table.as_ptr() as usize,
                image: refresh_abi::ImageShape {
                    text: 4,
                    symbol: 8,
                    setter: 0,
                },
                setter: setter as *const () as usize,
            },
            source: vec![79, 112, 101, 110],
            last: vec![79, 112, 101, 110],
            epoch: 0,
        };
        MODE.store(1, Ordering::Release);
        EPOCH.store(9, Ordering::Release);
        for translated in [vec![65, 108, 112, 104, 97], vec![66, 101, 116, 97]] {
            assert!(update_with(&i, handle, &mut o, false, 9, |source| {
                assert_eq!(source, [79, 112, 101, 110]);
                Some(translated.clone())
            }));
            assert_eq!(text(&o), Some(translated));
        }
        // A result computed across publication cannot overwrite the newer epoch.
        assert!(update_with(&i, handle, &mut o, false, 9, |_| {
            EPOCH.store(10, Ordering::Release);
            Some(vec![88])
        }));
        assert_eq!(text(&o), Some(vec![66, 101, 116, 97]));
        // Reusing the same address with another generation invalidates ownership.
        entry[0] = 3;
        assert!(!update_with(&i, handle, &mut o, true, 10, |_| panic!(
            "restore called host"
        )));
        assert_eq!(text(&o), Some(vec![66, 101, 116, 97]));
        entry[0] = 2;
        assert_eq!(entry[0], 2);
        // A game-owned edit is preserved, including when deactivating.
        buffer[0] = 90;
        assert!(!update_with(&i, handle, &mut o, true, 10, |_| panic!(
            "restore called host"
        )));
        assert_eq!(buffer[0], 90);
        buffer[0] = 66;
        STATE.lock().unwrap().owners.insert(handle, o.clone());
        // No UI callback means no success acknowledgement and no reactivation.
        assert!(deactivate().is_err());
        assert!(!ready());
        // A callback for a different/hidden panel still restores tracked owners.
        tick(&i, 0);
        assert_eq!(text(&o), Some(o.source.clone()));
        assert!(STATE.lock().unwrap().owners.is_empty());
        assert!(deactivate().is_ok());
        assert!(ready());
        unsafe { VirtualFree(code, 0, MEM_RELEASE) };
    }
}
