use crate::shape::{Image, Profile, Shape};
mod abi;
mod encoding;
use abi::{Convention, Function, Method};
use glyphshift_adapter_native_abi::*;
use retour::GenericDetour;
use std::{
    cell::Cell,
    collections::BTreeMap,
    ffi::c_void,
    sync::{
        atomic::{AtomicU64, Ordering},
        Condvar, Mutex, OnceLock,
    },
    time::Duration,
};
use windows_sys::Win32::{
    Foundation::{GetLastError, SetLastError},
    System::{
        Diagnostics::Debug::ReadProcessMemory,
        LibraryLoader::*,
        Memory::*,
        Threading::{GetCurrentProcess, GetCurrentThreadId},
    },
};

const LIMIT: usize = 8192;
const MAX_BYTES: usize = 4 * 1024 * 1024;
const MAX_OWNERS: usize = MAX_BYTES / (2 * LIMIT);
type Text = unsafe extern "thiscall" fn(*mut c_void, *const u8) -> usize;
type Destroy = unsafe extern "thiscall" fn(*mut c_void) -> usize;
type Tick = unsafe extern "thiscall" fn(*mut c_void, usize) -> usize;
type Show = unsafe extern "thiscall" fn(*mut c_void, i32) -> usize;
struct Hooks {
    shape: Shape,
    code_page: u32,
    finish: Function<Destroy>,
    setter: Method<Text>,
    append: Method<Text>,
    destroy: Method<Destroy>,
    tick: GenericDetour<Tick>,
    show: Method<Show>,
    history_tick: Option<GenericDetour<Tick>>,
}
static HOOKS: OnceLock<Hooks> = OnceLock::new();
static EPOCH: AtomicU64 = AtomicU64::new(1);
#[derive(Clone, Copy)]
struct Host {
    context: usize,
    decide: DecideUtf16V1,
    features: u64,
}
struct Owner {
    thread: u32,
    source: Vec<u8>,
    last: Vec<u8>,
    epoch: u64,
    checked: std::time::Instant,
    presented: bool,
}
struct Session {
    host: Option<Host>,
    owners: BTreeMap<usize, Owner>,
    restoring: bool,
    generation: u64,
}
static SESSION: Mutex<Session> = Mutex::new(Session {
    host: None,
    owners: BTreeMap::new(),
    restoring: false,
    generation: 0,
});
static RESTORED: Condvar = Condvar::new();
thread_local! { static BUSY: Cell<bool> = const { Cell::new(false) }; }
struct Guard;
impl Guard {
    fn enter() -> Option<Self> {
        BUSY.with(|b| if b.replace(true) { None } else { Some(Self) })
    }
}
impl Drop for Guard {
    fn drop(&mut self) {
        BUSY.with(|b| b.set(false));
    }
}

fn read(at: usize, length: usize) -> Option<Vec<u8>> {
    if at == 0 || length > 256 * 1024 * 1024 {
        return None;
    }
    at.checked_add(length)?;
    let mut result = vec![0; length];
    let mut count = 0;
    let ok = unsafe {
        ReadProcessMemory(
            GetCurrentProcess(),
            at as _,
            result.as_mut_ptr().cast(),
            length,
            &mut count,
        )
    };
    (ok != 0 && count == length).then_some(result)
}
fn word(at: usize) -> Option<usize> {
    Some(u32::from_le_bytes(read(at, 4)?.try_into().ok()?) as usize)
}
fn string(at: usize) -> Option<Vec<u8>> {
    let mut result = Vec::new();
    while result.len() <= LIMIT {
        let here = at.checked_add(result.len())?;
        let mut info: MEMORY_BASIC_INFORMATION = unsafe { std::mem::zeroed() };
        if unsafe { VirtualQuery(here as _, &mut info, size_of::<MEMORY_BASIC_INFORMATION>()) } == 0
            || info.State != MEM_COMMIT
            || info.Protect & (PAGE_GUARD | PAGE_NOACCESS) != 0
        {
            return None;
        }
        let length = (info.BaseAddress as usize + info.RegionSize - here)
            .min(256)
            .min(LIMIT + 1 - result.len());
        if length == 0 {
            return None;
        }
        let bytes = read(here, length)?;
        if let Some(end) = bytes.iter().position(|b| *b == 0) {
            result.extend_from_slice(&bytes[..end]);
            return Some(result);
        }
        result.extend_from_slice(&bytes);
    }
    None
}
fn current(hooks: &Hooks, object: usize) -> Option<Vec<u8>> {
    if word(object) != Some(hooks.shape.table) {
        return None;
    }
    let pointer = word(object.checked_add(hooks.shape.text_member)?)?;
    if pointer == 0 {
        return Some(Vec::new());
    }
    string(pointer)
}
fn image() -> Option<Image> {
    let base = unsafe { GetModuleHandleW(std::ptr::null()) } as usize;
    let header = read(base, 4096)?;
    if header.get(..2)? != b"MZ" {
        return None;
    }
    let pe = u32::from_le_bytes(header.get(60..64)?.try_into().ok()?) as usize;
    if header.get(pe..pe + 6)? != b"PE\0\0\x4c\x01" {
        return None;
    }
    let u16at = |at| Some(u16::from_le_bytes(header.get(at..at + 2)?.try_into().ok()?));
    let u32at = |at| Some(u32::from_le_bytes(header.get(at..at + 4)?.try_into().ok()?));
    if u16at(pe + 24)? != 0x10b {
        return None;
    }
    let size = u32at(pe + 80)? as usize;
    if !(4096..=256 * 1024 * 1024).contains(&size) {
        return None;
    }
    let count = u16at(pe + 6)? as usize;
    if count > 96 {
        return None;
    }
    let table = pe + 24 + u16at(pe + 20)? as usize;
    let mut executable = Vec::new();
    for index in 0..count {
        let section = table + index * 40;
        let start = u32at(section + 12)? as usize;
        let length = u32at(section + 8)? as usize;
        if start.checked_add(length)? > size {
            return None;
        }
        if u32at(section + 36)? & 0x20000000 != 0 {
            executable.push(base + start..base + start + length);
        }
    }
    Some(Image {
        base,
        bytes: read(base, size)?,
        executable,
    })
}
fn install() -> Result<(), ()> {
    if HOOKS.get().is_none() {
        let shape = image().and_then(|i| i.discover()).ok_or(())?;
        let convention = |classic| {
            if shape.profile == Profile::Classic {
                classic
            } else {
                Convention::Thiscall
            }
        };
        let hooks = unsafe {
            Hooks {
                shape,
                code_page: windows_sys::Win32::Globalization::GetACP(),
                finish: Function::new(shape.finish, convention(Convention::Finish))?,
                setter: Method::new(
                    shape.setter,
                    set_hook as Text,
                    convention(Convention::Setter),
                )
                .map_err(|_| ())?,
                append: Method::new(
                    shape.append,
                    append_hook as Text,
                    convention(Convention::Append),
                )
                .map_err(|_| ())?,
                destroy: Method::new(
                    shape.destroy,
                    destroy_hook as Destroy,
                    convention(Convention::Destroy),
                )
                .map_err(|_| ())?,
                tick: GenericDetour::new(
                    std::mem::transmute::<usize, Tick>(shape.tick),
                    tick_hook as Tick,
                )
                .map_err(|_| ())?,
                show: Method::new(shape.show, show_hook as Show, convention(Convention::Show))
                    .map_err(|_| ())?,
                history_tick: shape
                    .history
                    .map(|(_, tick)| {
                        GenericDetour::new(
                            std::mem::transmute::<usize, Tick>(tick),
                            history_tick_hook as Tick,
                        )
                    })
                    .transpose()
                    .map_err(|_| ())?,
            }
        };
        let mut module = std::ptr::null_mut();
        if unsafe {
            GetModuleHandleExW(
                GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_PIN,
                tick_hook as *const u16,
                &mut module,
            )
        } == 0
        {
            return Err(());
        }
        HOOKS.set(hooks).map_err(|_| ())?;
    }
    let h = HOOKS.get().ok_or(())?;
    unsafe {
        // Cleanup and external writes must be observed before tick can retain an object.
        let result = h
            .destroy
            .enable()
            .and_then(|_| h.setter.enable())
            .and_then(|_| h.append.enable())
            .and_then(|_| h.show.enable())
            .and_then(|_| h.history_tick.as_ref().map_or(Ok(()), |hook| hook.enable()))
            .and_then(|_| h.tick.enable());
        if result.is_err() {
            let _ = h.tick.disable();
            if let Some(hook) = &h.history_tick {
                let _ = hook.disable();
            }
            let _ = h.show.disable();
            let _ = h.append.disable();
            let _ = h.setter.disable();
            let _ = h.destroy.disable();
            return Err(());
        }
    }
    Ok(())
}
pub fn activate(host: NativeRuntimeHostV1, features: u64) -> Result<(), ()> {
    let mut state = SESSION.lock().map_err(|_| ())?;
    if state.host.is_some() || state.restoring {
        return Err(());
    }
    install()?;
    state.generation = 0;
    state.host = Some(Host {
        context: host.context as usize,
        decide: host.decide_utf16,
        features,
    });
    refresh();
    Ok(())
}
pub fn refresh() {
    EPOCH.fetch_add(1, Ordering::AcqRel);
}
pub fn deactivate() -> Result<(), ()> {
    let mut state = SESSION.lock().map_err(|_| ())?;
    state.host = None;
    state.owners.retain(|_, o| o.last != o.source);
    state.restoring = !state.owners.is_empty();
    refresh();
    let (state, _) = RESTORED
        .wait_timeout_while(state, Duration::from_secs(3), |s| s.restoring)
        .map_err(|_| ())?;
    if state.restoring {
        Err(())
    } else {
        Ok(())
    }
}
fn plain(bytes: &[u8]) -> bool {
    !bytes.is_empty()
        && bytes.len() <= LIMIT
        && !bytes
            .iter()
            .any(|b| matches!(*b, 0..=8 | 11..=31 | 127 | b'\\' | b'[' | b']'))
}
fn decide(host: Host, source: &[u8], generation: &mut u64) -> Vec<u8> {
    let h = HOOKS.get().unwrap();
    let (prefix, text, marker) = if h.shape.profile == Profile::Classic {
        let (prefix, body, marker) = crate::text::classic_parts(source);
        let Some(text) = encoding::decode(body, h.code_page) else {
            return source.to_vec();
        };
        (prefix, text, marker)
    } else {
        let Some((text, marker)) = crate::text::decode(source) else {
            return source.to_vec();
        };
        (&[][..], text.to_owned(), marker)
    };
    let input: Vec<_> = text.encode_utf16().collect();
    let mut output = vec![0u16; LIMIT];
    let result = (host.decide)(
        host.context as _,
        input.as_ptr(),
        input.len() as u32,
        output.as_mut_ptr(),
        output.len() as u32,
        std::ptr::null_mut(),
        0,
    );
    if result.generation < *generation {
        return source.to_vec();
    }
    *generation = result.generation;
    if host.features & FEATURE_TEXT_REPLACE == 0
        || result.status != STATUS_OK
        || result.decision_bits & DECISION_TEXT_REPLACE == 0
        || result.text_len as usize > output.len()
        || !plain(text.as_bytes())
    {
        return source.to_vec();
    }
    String::from_utf16(&output[..result.text_len as usize])
        .ok()
        .filter(|text| plain(text.as_bytes()))
        .and_then(|text| {
            if h.shape.profile == Profile::Classic {
                encoding::encode(&text, h.code_page)
            } else {
                Some(text.into_bytes())
            }
        })
        .and_then(|body| {
            let mut bytes = prefix.to_vec();
            bytes.extend_from_slice(&body);
            bytes.extend_from_slice(marker);
            (bytes.len() <= LIMIT).then_some(bytes)
        })
        .unwrap_or_else(|| source.to_vec())
}
fn completed(h: &Hooks, bytes: &[u8]) -> bool {
    bytes.ends_with(if h.shape.profile == Profile::Classic {
        &[255]
    } else {
        crate::text::MARKER
    })
}
unsafe fn set(h: &Hooks, object: usize, bytes: &[u8], presented: bool) {
    let mut terminated = bytes.to_vec();
    terminated.push(0);
    (h.setter.original.call)(object as _, terminated.as_ptr());
    // A completed dialogue is an engine string plus its terminal marker. The
    // setter invalidates glyphs but does not submit display/finish instructions.
    // Use the engine's own methods on the owner thread; never advance the script.
    if presented || completed(h, bytes) {
        let finish = h.finish.call;
        (h.show.original.call)(object as _, -1);
        finish(object as _);
    }
}
fn tick(object: usize, presented: bool) {
    let Some(_guard) = Guard::enter() else { return };
    let h = HOOKS.get().unwrap();
    let Ok(mut state) = SESSION.lock() else {
        return;
    };
    let thread = unsafe { GetCurrentThreadId() };
    if state.restoring {
        let keys: Vec<_> = state
            .owners
            .iter()
            .filter_map(|(key, o)| (o.thread == thread).then_some(*key))
            .collect();
        for key in keys {
            let owner = state.owners.remove(&key).unwrap();
            if current(h, key).as_ref() == Some(&owner.last) {
                unsafe {
                    set(h, key, &owner.source, owner.presented);
                }
            }
        }
        state.restoring = !state.owners.is_empty();
        if !state.restoring {
            RESTORED.notify_all();
        }
        return;
    }
    let Some(host) = state.host else { return };
    let Some(actual) = current(h, object) else {
        state.owners.remove(&object);
        return;
    };
    let epoch = EPOCH.load(Ordering::Acquire);
    if state
        .owners
        .get(&object)
        .is_some_and(|o| o.thread != thread)
    {
        return;
    }
    let previous = state.owners.remove(&object);
    let mut owner = match previous {
        Some(owner) if owner.last == actual => owner,
        _ => Owner {
            thread,
            source: actual.clone(),
            last: actual,
            epoch: 0,
            checked: std::time::Instant::now(),
            presented,
        },
    };
    owner.presented |= presented;
    if owner.source.is_empty() {
        return;
    }
    // Reserve the maximum original + translated storage at admission. An
    // existing translated owner can never be evicted by later budget pressure.
    if state.owners.len() >= MAX_OWNERS {
        return;
    }
    if owner.epoch != epoch || owner.checked.elapsed() >= Duration::from_secs(1) {
        let next = decide(host, &owner.source, &mut state.generation);
        if next != owner.last
            || (owner.epoch != epoch
                && next != owner.source
                && (owner.presented || completed(h, &next)))
        {
            unsafe {
                set(h, object, &next, owner.presented);
            }
            // This profile must copy the input. Retain only the value the engine accepted.
            if current(h, object).as_ref() != Some(&next) {
                return;
            }
            owner.last = next;
        }
        owner.epoch = epoch;
        owner.checked = std::time::Instant::now();
    }
    state.owners.insert(object, owner);
}
unsafe extern "thiscall" fn tick_hook(object: *mut c_void, arg: usize) -> usize {
    let error = GetLastError();
    let _ = std::panic::catch_unwind(|| tick(object as usize, false));
    SetLastError(error);
    HOOKS.get().unwrap().tick.call(object, arg)
}
unsafe extern "thiscall" fn show_hook(object: *mut c_void, limit: i32) -> usize {
    let error = GetLastError();
    // History rows submit display directly without running the string's tick.
    // Admit at this actual display boundary; do not guess visibility from UTF-8.
    let _ = std::panic::catch_unwind(|| tick(object as usize, true));
    SetLastError(error);
    (HOOKS.get().unwrap().show.original.call)(object, limit)
}
fn history_tick(object: usize) {
    let h = HOOKS.get().unwrap();
    if h.shape.history.map(|(table, _)| table) != word(object) {
        return;
    }
    let epoch = EPOCH.load(Ordering::Acquire);
    let thread = unsafe { GetCurrentThreadId() };
    let keys = {
        let Ok(state) = SESSION.lock() else { return };
        if state.restoring {
            drop(state);
            tick(object, false);
            return;
        }
        if state.host.is_none() {
            return;
        }
        state
            .owners
            .iter()
            .filter_map(|(key, owner)| {
                (owner.presented
                    && owner.thread == thread
                    && (owner.epoch != epoch || owner.checked.elapsed() >= Duration::from_secs(1)))
                .then_some(*key)
            })
            .collect::<Vec<_>>()
    };
    // Reuse the same conflict, lifetime, thread and budget checks as dialogue.
    for key in keys {
        tick(key, false);
    }
}
unsafe extern "thiscall" fn history_tick_hook(object: *mut c_void, arg: usize) -> usize {
    let error = GetLastError();
    // A Host may re-enter the engine while a decision holds SESSION.
    if !BUSY.with(Cell::get) {
        let _ = std::panic::catch_unwind(|| history_tick(object as usize));
    }
    SetLastError(error);
    HOOKS
        .get()
        .unwrap()
        .history_tick
        .as_ref()
        .unwrap()
        .call(object, arg)
}
unsafe extern "thiscall" fn set_hook(object: *mut c_void, text: *const u8) -> usize {
    let error = GetLastError();
    if let Some(_guard) = Guard::enter() {
        if let Ok(mut state) = SESSION.lock() {
            state.owners.remove(&(object as usize));
        }
    }
    SetLastError(error);
    (HOOKS.get().unwrap().setter.original.call)(object, text)
}
unsafe extern "thiscall" fn append_hook(object: *mut c_void, text: *const u8) -> usize {
    let error = GetLastError();
    let h = HOOKS.get().unwrap();
    let guard = Guard::enter();
    let mut copied = None;
    if guard.is_some() {
        if let Ok(mut state) = SESSION.lock() {
            if let Some(owner) = state.owners.remove(&(object as usize)) {
                if owner.thread == GetCurrentThreadId()
                    && owner.source != owner.last
                    && current(h, object as usize).as_ref() == Some(&owner.last)
                {
                    // Copy first: the caller may pass a substring of the current buffer.
                    copied = string(text as usize).map(|mut s| {
                        s.push(0);
                        s
                    });
                    if copied.is_some() || text.is_null() {
                        set(h, object as usize, &owner.source, false);
                    } else {
                        state.owners.insert(object as usize, owner);
                    }
                }
            }
        }
    }
    SetLastError(error);
    (h.append.original.call)(object, copied.as_ref().map_or(text, |s| s.as_ptr()))
}
unsafe extern "thiscall" fn destroy_hook(object: *mut c_void) -> usize {
    let error = GetLastError();
    if let Some(_guard) = Guard::enter() {
        if let Ok(mut state) = SESSION.lock() {
            state.owners.remove(&(object as usize));
            if state.restoring && state.owners.is_empty() {
                state.restoring = false;
                RESTORED.notify_all();
            }
        }
    }
    SetLastError(error);
    (HOOKS.get().unwrap().destroy.original.call)(object)
}
