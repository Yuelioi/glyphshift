//! Adapts verified optimized register entry points to thiscall callbacks.
//! Pages become executable only after construction and live with the hooks.
use retour::RawDetour;
use std::ffi::c_void;
use windows_sys::Win32::System::{
    Diagnostics::Debug::FlushInstructionCache, Memory::*, Threading::GetCurrentProcess,
};

#[derive(Clone, Copy)]
pub enum Convention {
    Thiscall,
    Setter,
    Append,
    Destroy,
    Show,
    Finish,
}
struct Page(usize);
unsafe impl Send for Page {}
unsafe impl Sync for Page {}
impl Page {
    unsafe fn new(bytes: &[u8]) -> Result<Self, ()> {
        let p = VirtualAlloc(
            std::ptr::null(),
            bytes.len(),
            MEM_COMMIT | MEM_RESERVE,
            PAGE_READWRITE,
        );
        if p.is_null() {
            return Err(());
        }
        let page = Self(p as usize);
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), p.cast(), bytes.len());
        let mut old = 0;
        if VirtualProtect(p, bytes.len(), PAGE_EXECUTE_READ, &mut old) == 0
            || FlushInstructionCache(GetCurrentProcess(), p, bytes.len()) == 0
        {
            return Err(());
        }
        Ok(page)
    }
}
impl Drop for Page {
    fn drop(&mut self) {
        unsafe {
            VirtualFree(self.0 as *mut c_void, 0, MEM_RELEASE);
        }
    }
}
fn branch(code: &mut Vec<u8>, target: usize, jump: bool) {
    code.push(0xba); // mov edx, target; call/jmp edx
    code.extend_from_slice(&(target as u32).to_le_bytes());
    code.extend_from_slice(&[0xff, if jump { 0xe2 } else { 0xd2 }]);
}
fn bridge(target: usize, kind: Convention, inbound: bool) -> Vec<u8> {
    let (head, tail, jump): (&[u8], &[u8], bool) = match (kind, inbound) {
        (Convention::Setter, true) => (&[0x56, 0x8b, 0xcf], &[0xc3], false),
        (Convention::Append, true) => (&[0x8b, 0xcb], &[], true),
        (Convention::Destroy, true) => (&[0x8b, 0xcf], &[], true),
        (Convention::Show, true) => (&[0x51, 0x8b, 0xc8], &[0xc3], false),
        (Convention::Setter, false) => (
            &[0x57, 0x56, 0x8b, 0xf9, 0x8b, 0x74, 0x24, 0x0c],
            &[0x5e, 0x5f, 0xc2, 4, 0],
            false,
        ),
        (Convention::Append, false) => (
            &[0x53, 0x8b, 0xd9, 0xff, 0x74, 0x24, 8],
            &[0x5b, 0xc2, 4, 0],
            false,
        ),
        (Convention::Destroy, false) => (&[0x57, 0x8b, 0xf9], &[0x5f, 0xc3], false),
        (Convention::Show, false) => (&[0x8b, 0xc1, 0x8b, 0x4c, 0x24, 4], &[0xc2, 4, 0], false),
        (Convention::Finish, false) => (&[0x8b, 0xc1], &[], true),
        _ => (&[], &[], true),
    };
    let mut code = head.to_vec();
    branch(&mut code, target, jump);
    code.extend_from_slice(tail);
    code
}
pub struct Function<F> {
    pub call: F,
    _page: Option<Page>,
}
impl<F: Copy> Function<F> {
    pub unsafe fn new(target: usize, convention: Convention) -> Result<Self, ()> {
        assert_eq!(size_of::<F>(), size_of::<usize>());
        let page = if matches!(convention, Convention::Thiscall) {
            None
        } else {
            Some(Page::new(&bridge(target, convention, false))?)
        };
        let address = page.as_ref().map_or(target, |p| p.0);
        Ok(Self {
            call: std::mem::transmute_copy(&address),
            _page: page,
        })
    }
}
pub struct Method<F> {
    pub original: Function<F>,
    hook: RawDetour,
    _entry: Option<Page>,
}
impl<F: Copy> Method<F> {
    pub unsafe fn new(target: usize, callback: F, convention: Convention) -> Result<Self, ()> {
        assert_eq!(size_of::<F>(), size_of::<usize>());
        let callback: usize = std::mem::transmute_copy(&callback);
        let entry = if matches!(convention, Convention::Thiscall) {
            None
        } else {
            Some(Page::new(&bridge(callback, convention, true))?)
        };
        let hook = RawDetour::new(target as _, entry.as_ref().map_or(callback, |p| p.0) as _)
            .map_err(|_| ())?;
        let original = Function::new(hook.trampoline() as *const () as usize, convention)?;
        Ok(Self {
            original,
            hook,
            _entry: entry,
        })
    }
    pub unsafe fn enable(&self) -> retour::Result<()> {
        self.hook.enable()
    }
    pub unsafe fn disable(&self) -> retour::Result<()> {
        self.hook.disable()
    }
}
