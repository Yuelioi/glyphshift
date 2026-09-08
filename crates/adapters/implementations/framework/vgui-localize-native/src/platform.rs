use crate::{memory, query_abi, returned_text::ReturnedText};
use glyphshift_adapter_native_abi::*;
use std::cell::Cell;
use std::ffi::c_void;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Condvar, Mutex, OnceLock,
};
use windows_sys::Win32::{
    Foundation::{FreeLibrary, GetLastError, SetLastError},
    System::{
        LibraryLoader::*,
        Memory::{PAGE_READWRITE, PAGE_WRITECOPY},
    },
};

const MAX_SOURCE_UNITS: usize = 2048;
const MAX_LABEL_UNITS: usize = 256;
type Find = unsafe extern "thiscall" fn(*mut c_void, *const u8) -> *mut u16;
type Value = unsafe extern "thiscall" fn(*mut c_void, u32) -> *mut u16;

struct Installed {
    object: usize,
    original: usize,
    shadow: usize,
    find: Find,
    value: Value,
}
static INSTALLED: OnceLock<Installed> = OnceLock::new();

#[derive(Clone, Copy)]
struct Host {
    context: usize,
    decide: DecideUtf16V1,
    features: u64,
}
#[derive(Default)]
struct Session {
    host: Option<Host>,
    in_flight: usize,
    generation: u64,
}
static SESSION: Mutex<Session> = Mutex::new(Session {
    host: None,
    in_flight: 0,
    generation: 0,
});
static DRAINED: Condvar = Condvar::new();
static RETURNS: OnceLock<Mutex<ReturnedText>> = OnceLock::new();
thread_local! { static DEPTH: Cell<usize> = const { Cell::new(0) }; }

struct Depth(usize);
impl Depth {
    fn enter() -> Self {
        Self(DEPTH.with(|depth| {
            let previous = depth.get();
            depth.set(previous.saturating_add(1));
            previous
        }))
    }
}
impl Drop for Depth {
    fn drop(&mut self) {
        DEPTH.with(|depth| depth.set(self.0));
    }
}

struct Lease(Host);
impl Lease {
    fn begin() -> Option<Self> {
        let mut session = SESSION.lock().ok()?;
        let host = session.host?;
        session.in_flight += 1;
        Some(Self(host))
    }
}
impl Drop for Lease {
    fn drop(&mut self) {
        if let Ok(mut session) = SESSION.lock() {
            session.in_flight -= 1;
            if session.in_flight == 0 {
                DRAINED.notify_all();
            }
        }
    }
}

pub fn activate(host: NativeRuntimeHostV1, features: u64) -> Result<(), ()> {
    {
        let session = SESSION.lock().map_err(|_| ())?;
        if session.host.is_some() || session.in_flight != 0 || !crate::refresh::ready() {
            return Err(());
        }
    }
    if features == 0 {
        return Ok(());
    }
    install()?;
    let mut session = SESSION.lock().map_err(|_| ())?;
    session.generation = 0;
    session.host = Some(Host {
        context: host.context as usize,
        decide: host.decide_utf16,
        features,
    });
    drop(session);
    if features & FEATURE_TEXT_REPLACE != 0 {
        crate::refresh::activate();
    }
    Ok(())
}

pub fn deactivate() -> Result<(), ()> {
    let mut session = SESSION.lock().map_err(|_| ())?;
    session.host = None;
    let (session, _) = DRAINED
        .wait_timeout_while(session, std::time::Duration::from_secs(3), |session| {
            session.in_flight != 0
        })
        .map_err(|_| ())?;
    let drained = session.in_flight == 0;
    drop(session);
    crate::refresh::deactivate()?;
    drained.then_some(()).ok_or(())
}

pub(crate) fn pin(address: usize) -> Result<(), ()> {
    let mut module = std::ptr::null_mut();
    let ok = unsafe {
        GetModuleHandleExW(
            GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_PIN,
            address as *const u16,
            &mut module,
        )
    };
    (ok != 0).then_some(()).ok_or(())
}

fn install() -> Result<(), ()> {
    if let Some(installed) = INSTALLED.get() {
        return (memory::word(installed.object) == Some(installed.shadow))
            .then_some(())
            .ok_or(());
    }
    let module_name = "vgui2.dll"
        .encode_utf16()
        .chain(Some(0))
        .collect::<Vec<_>>();
    let mut module = std::ptr::null_mut();
    if unsafe { GetModuleHandleExW(0, module_name.as_ptr(), &mut module) } == 0 {
        return Err(());
    }
    struct Module(*mut c_void);
    impl Drop for Module {
        fn drop(&mut self) {
            unsafe {
                FreeLibrary(self.0);
            }
        }
    }
    let _held = Module(module);
    let factory = unsafe { GetProcAddress(module, c"CreateInterface".as_ptr().cast()) }.ok_or(())?;
    let factory: unsafe extern "C" fn(*const u8, *mut i32) -> *mut c_void =
        unsafe { std::mem::transmute(factory) };
    let object =
        unsafe { factory(c"VGUI_Localize005".as_ptr().cast(), std::ptr::null_mut()) } as usize;
    if object == 0 || !object.is_multiple_of(align_of::<usize>()) {
        return Err(());
    }
    let region = memory::region(object).ok_or(())?;
    if !matches!(region.Protect & 0xff, PAGE_READWRITE | PAGE_WRITECOPY)
        || object.checked_add(size_of::<usize>()).ok_or(())?
            > (region.BaseAddress as usize)
                .checked_add(region.RegionSize)
                .ok_or(())?
    {
        return Err(());
    }
    let original = memory::word(object).ok_or(())?;
    let mut entries = Vec::new();
    let mut methods = Vec::new();
    for slot in 0..64 {
        let Some(entry) = memory::word(original.checked_add(slot * size_of::<usize>()).ok_or(())?)
        else {
            break;
        };
        let Some(code) = memory::code(entry, module as usize, 4096) else {
            break;
        };
        entries.push(entry);
        methods.push(code);
    }
    if entries.len() == 64 {
        return Err(());
    }
    let slots = query_abi::identify(&methods).ok_or(())?;
    if entries[slots.find] == entries[slots.value]
        || entries[slots.find] == entries[slots.index]
        || entries[slots.value] == entries[slots.index]
    {
        return Err(());
    }
    for slot in [slots.find, slots.index, slots.value] {
        if memory::code(entries[slot], module as usize, 4096).as_ref() != Some(&methods[slot]) {
            return Err(());
        }
    }
    for (slot, entry) in entries.iter().enumerate() {
        if memory::word(original + slot * size_of::<usize>()) != Some(*entry) {
            return Err(());
        }
    }
    let mut shadow = Vec::with_capacity(entries.len() + 1);
    shadow.push(memory::word(original.checked_sub(size_of::<usize>()).ok_or(())?).ok_or(())?);
    shadow.extend_from_slice(&entries);
    shadow[slots.find + 1] = hook_find as *const () as usize;
    shadow[slots.value + 1] = hook_value as *const () as usize;
    // Both callbacks and original methods may be called after deactivation by
    // a thread that fetched the old vptr. Keep both modules and the table alive.
    pin(module as usize)?;
    pin(hook_find as *const () as usize)?;
    let shadow = Box::leak(shadow.into_boxed_slice()).as_mut_ptr();
    let installed = Installed {
        object,
        original,
        shadow: unsafe { shadow.add(1) } as usize,
        find: unsafe { std::mem::transmute::<usize, Find>(entries[slots.find]) },
        value: unsafe { std::mem::transmute::<usize, Value>(entries[slots.value]) },
    };
    INSTALLED.set(installed).map_err(|_| ())?;
    let installed = INSTALLED.get().ok_or(())?;
    // An aligned atomic replacement does not edit the original shared vtable.
    unsafe { &*(object as *const AtomicUsize) }
        .compare_exchange(
            installed.original,
            installed.shadow,
            Ordering::AcqRel,
            Ordering::Acquire,
        )
        .map_err(|_| ())?;
    Ok(())
}

pub(crate) fn plain_label(units: &[u16]) -> bool {
    !units.is_empty()
        && units.len() <= MAX_LABEL_UNITS
        && String::from_utf16(units).is_ok()
        && units
            .iter()
            .all(|unit| *unit >= 0x20 && *unit != 0x7f && *unit != b'%' as u16)
}

pub(crate) fn original_index(index: u32) -> Option<Vec<u16>> {
    let i = INSTALLED.get()?;
    let value = unsafe { (i.value)(i.object as _, index) };
    memory::utf16(value as usize, MAX_LABEL_UNITS)
}

pub(crate) fn returned_pair(source: &[u16], text: &[u16]) -> bool {
    RETURNS
        .get()
        .and_then(|r| r.lock().ok())
        .is_some_and(|r| r.contains(source, text))
}

pub(crate) fn refresh_decision(source: &[u16]) -> Option<Vec<u16>> {
    let lease = Lease::begin()?;
    let mut output = [0u16; MAX_LABEL_UNITS];
    let decision = (lease.0.decide)(
        lease.0.context as _,
        source.as_ptr(),
        source.len() as u32,
        output.as_mut_ptr(),
        output.len() as u32,
        std::ptr::null_mut(),
        0,
    );
    let mut session = SESSION.lock().ok()?;
    if session.host.is_none() || decision.generation < session.generation {
        return None;
    }
    session.generation = decision.generation;
    Some(refresh_result(source, &output, decision, lease.0.features))
}

fn refresh_result(
    source: &[u16],
    output: &[u16],
    decision: NativeDecisionV1,
    features: u64,
) -> Vec<u16> {
    if decision.status != STATUS_OK
        || features & FEATURE_TEXT_REPLACE == 0
        || decision.decision_bits & DECISION_TEXT_REPLACE == 0
    {
        return source.to_vec();
    }
    let Some(text) = output.get(..decision.text_len as usize) else {
        return source.to_vec();
    };
    if plain_label(text) {
        text.to_vec()
    } else {
        source.to_vec()
    }
}

fn decide(original: *mut u16) -> Option<*mut u16> {
    let lease = Lease::begin()?;
    let source = memory::utf16(original as usize, MAX_SOURCE_UNITS)?;
    if source.is_empty() || String::from_utf16(&source).is_err() {
        return None;
    }
    let mut output = [0u16; MAX_LABEL_UNITS];
    let decision = (lease.0.decide)(
        lease.0.context as *mut c_void,
        source.as_ptr(),
        source.len() as u32,
        output.as_mut_ptr(),
        output.len() as u32,
        std::ptr::null_mut(),
        0,
    );
    // Every observed decision advances the boundary, including a publication
    // that removed a translation or whose output did not fit the caller buffer.
    let mut session = SESSION.lock().ok()?;
    if session.host.is_none() || decision.generation < session.generation {
        return None;
    }
    session.generation = decision.generation;
    if lease.0.features & FEATURE_TEXT_REPLACE == 0
        || decision.status != STATUS_OK
        || decision.decision_bits & DECISION_TEXT_REPLACE == 0
        || decision.text_len as usize > output.len()
    {
        return None;
    }
    let text = &output[..decision.text_len as usize];
    if !plain_label(&source) || !plain_label(text) {
        return None;
    }
    // Serialize generation acceptance with storage publication. A slow decision
    // from an earlier generation must not publish after a newer decision.
    let mut returned = RETURNS
        .get_or_init(|| Mutex::new(ReturnedText::default()))
        .lock()
        .ok()?;
    returned
        .intern(&source, text)
        .map(|address| address as *mut u16)
}

unsafe extern "thiscall" fn hook_find(this: *mut c_void, token: *const u8) -> *mut u16 {
    let error = GetLastError();
    let depth = Depth::enter();
    SetLastError(error);
    let installed = INSTALLED.get().expect("published before the shadow vtable");
    let original = (installed.find)(this, token);
    finish(this, original, depth)
}
unsafe extern "thiscall" fn hook_value(this: *mut c_void, index: u32) -> *mut u16 {
    let error = GetLastError();
    let depth = Depth::enter();
    SetLastError(error);
    let installed = INSTALLED.get().expect("published before the shadow vtable");
    let original = (installed.value)(this, index);
    finish(this, original, depth)
}
unsafe fn finish(this: *mut c_void, original: *mut u16, depth: Depth) -> *mut u16 {
    let error = GetLastError();
    let result = if depth.0 == 0
        && INSTALLED
            .get()
            .is_some_and(|installed| installed.object == this as usize)
    {
        std::panic::catch_unwind(|| decide(original))
            .ok()
            .flatten()
            .unwrap_or(original)
    } else {
        original
    };
    drop(depth);
    SetLastError(error);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejected_new_translation_restores_source_instead_of_retaining_old_translation() {
        let source = [79, 112, 101, 110];
        let valid = NativeDecisionV1 {
            status: STATUS_OK,
            generation: 3,
            decision_bits: DECISION_TEXT_REPLACE,
            text_len: 1,
            font_len: 0,
        };
        assert_eq!(
            refresh_result(&source, &[65], valid, FEATURE_TEXT_REPLACE),
            [65]
        );
        for decision in [
            NativeDecisionV1 {
                status: STATUS_OUTPUT_TOO_SMALL,
                ..valid
            },
            NativeDecisionV1 {
                text_len: 257,
                ..valid
            },
            NativeDecisionV1 {
                decision_bits: 0,
                ..valid
            },
        ] {
            assert_eq!(
                refresh_result(&source, &[65], decision, FEATURE_TEXT_REPLACE),
                source
            );
        }
        for output in [[0xd800], [37], [0]] {
            assert_eq!(
                refresh_result(&source, &output, valid, FEATURE_TEXT_REPLACE),
                source
            );
        }
    }
}
