//! The deliberately narrow, dynamically resolved Qt 6.8.3 / 6.11.1 MSVC x64 ABI.
use glyphshift_adapter_qt_quick::{eligible_text, MAX_TEXT_UNITS};
use std::{
    collections::HashSet,
    ffi::c_void,
    sync::atomic::{AtomicI32, Ordering},
};
use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};

pub type Object = *mut c_void;
pub type Setter = unsafe extern "system" fn(Object, Object);
pub type Destructor = unsafe extern "system" fn(Object);
type Getter = unsafe extern "system" fn(Object, Object) -> Object;
type List = unsafe extern "system" fn(Object) -> Object;
type Pointer = unsafe extern "system" fn(Object) -> Object;
type Predicate = unsafe extern "system" fn(Object) -> bool;

const SUPPORTED_QT_VERSIONS: &[&[u8]] = &[b"6.8.3", b"6.11.1"];

fn supported_qt_version(version: &[u8]) -> bool {
    SUPPORTED_QT_VERSIONS.contains(&version)
}

pub struct Qt {
    all_windows: List,
    content: Pointer,
    children: Getter,
    inherits: unsafe extern "system" fn(Object, *const u8) -> bool,
    text: Getter,
    pub setter: Setter,
    pub destructor: Destructor,
    visible: Predicate,
    window_visible: Predicate,
    opacity: unsafe extern "system" fn(Object) -> f64,
    format: unsafe extern "system" fn(Object) -> i32,
    ctor: unsafe extern "system" fn(Object, *const u16, i64) -> Object,
    dtor: Destructor,
    size: unsafe extern "system" fn(Object) -> i64,
    utf16: unsafe extern "system" fn(Object) -> *const u16,
    free_array: unsafe extern "system" fn(Object, i64, i64),
}
pub struct Label {
    pub object: usize,
    pub source: String,
    pub eligible: bool,
}
#[repr(C, align(8))]
struct Storage([usize; 4]);

impl Qt {
    pub unsafe fn resolve() -> Result<Self, ()> {
        let module = |name: &str| {
            let wide = name.encode_utf16().chain(Some(0)).collect::<Vec<_>>();
            GetModuleHandleW(wide.as_ptr())
        };
        let core = module("Qt6Core.dll");
        let gui = module("Qt6Gui.dll");
        let quick = module("Qt6Quick.dll");
        if core.is_null() || gui.is_null() || quick.is_null() {
            return Err(());
        }
        macro_rules! symbol {
            ($module:expr, $name:expr, $ty:ty) => {
                std::mem::transmute::<unsafe extern "system" fn() -> isize, $ty>(
                    GetProcAddress($module, concat!($name, "\0").as_ptr()).ok_or(())?,
                )
            };
        }
        let version = symbol!(core, "qVersion", unsafe extern "system" fn() -> *const i8);
        if !supported_qt_version(std::ffi::CStr::from_ptr(version()).to_bytes()) {
            return Err(());
        }
        Ok(Self {
            all_windows: symbol!(
                gui,
                "?allWindows@QGuiApplication@@SA?AV?$QList@PEAVQWindow@@@@XZ",
                List
            ),
            content: symbol!(
                quick,
                "?contentItem@QQuickWindow@@QEBAPEAVQQuickItem@@XZ",
                Pointer
            ),
            children: symbol!(
                quick,
                "?childItems@QQuickItem@@QEBA?AV?$QList@PEAVQQuickItem@@@@XZ",
                Getter
            ),
            inherits: symbol!(
                core,
                "?inherits@QObject@@QEBA_NPEBD@Z",
                unsafe extern "system" fn(Object, *const u8) -> bool
            ),
            text: symbol!(quick, "?text@QQuickText@@QEBA?AVQString@@XZ", Getter),
            setter: symbol!(quick, "?setText@QQuickText@@QEAAXAEBVQString@@@Z", Setter),
            destructor: symbol!(quick, "??1QQuickText@@UEAA@XZ", Destructor),
            visible: symbol!(quick, "?isVisible@QQuickItem@@QEBA_NXZ", Predicate),
            window_visible: symbol!(gui, "?isVisible@QWindow@@QEBA_NXZ", Predicate),
            opacity: symbol!(
                quick,
                "?opacity@QQuickItem@@QEBANXZ",
                unsafe extern "system" fn(Object) -> f64
            ),
            format: symbol!(
                quick,
                "?textFormat@QQuickText@@QEBA?AW4TextFormat@1@XZ",
                unsafe extern "system" fn(Object) -> i32
            ),
            ctor: symbol!(
                core,
                "??0QString@@QEAA@PEBVQChar@@_J@Z",
                unsafe extern "system" fn(Object, *const u16, i64) -> Object
            ),
            dtor: symbol!(core, "??1QString@@QEAA@XZ", Destructor),
            size: symbol!(
                core,
                "?size@QString@@QEBA_JXZ",
                unsafe extern "system" fn(Object) -> i64
            ),
            utf16: symbol!(
                core,
                "?utf16@QString@@QEBAPEBGXZ",
                unsafe extern "system" fn(Object) -> *const u16
            ),
            free_array: symbol!(
                core,
                "?deallocate@QArrayData@@SAXPEAU1@_J1@Z",
                unsafe extern "system" fn(Object, i64, i64)
            ),
        })
    }
    pub unsafe fn string(&self, object: Object) -> Option<String> {
        if object.is_null() {
            return None;
        }
        let length = usize::try_from((self.size)(object)).ok()?;
        if length > MAX_TEXT_UNITS {
            return None;
        }
        if length == 0 {
            return Some(String::new());
        }
        let units = (self.utf16)(object);
        if units.is_null() {
            return None;
        }
        String::from_utf16(std::slice::from_raw_parts(units, length)).ok()
    }
    pub unsafe fn read(&self, object: Object) -> Option<String> {
        let mut value = Storage([0; 4]);
        let ptr = std::ptr::from_mut(&mut value).cast();
        (self.text)(object, ptr);
        let result = self.string(ptr);
        (self.dtor)(ptr);
        result
    }
    pub unsafe fn write(&self, object: Object, source: &str, setter: impl FnOnce(Object, Object)) {
        let units = source.encode_utf16().collect::<Vec<_>>();
        let mut value = Storage([0; 4]);
        let ptr = std::ptr::from_mut(&mut value).cast();
        (self.ctor)(ptr, units.as_ptr(), units.len() as i64);
        setter(object, ptr);
        (self.dtor)(ptr);
    }
    unsafe fn list(&self, value: Storage) -> Vec<Object> {
        let count = value.0[2];
        let data = value.0[1] as *const Object;
        let result = if count <= 30_000 && (count == 0 || !data.is_null()) {
            (0..count).map(|i| *data.add(i)).collect()
        } else {
            Vec::new()
        };
        if value.0[0] != 0 {
            let refs = &*(value.0[0] as *const AtomicI32);
            if refs.load(Ordering::Relaxed) > 0 && refs.fetch_sub(1, Ordering::AcqRel) == 1 {
                (self.free_array)(value.0[0] as Object, 8, 8);
            }
        }
        result
    }
    pub unsafe fn labels(&self) -> Vec<Label> {
        let mut value = Storage([0; 4]);
        (self.all_windows)(std::ptr::from_mut(&mut value).cast());
        let mut pending = Vec::new();
        for window in self.list(value) {
            if !window.is_null() && (self.inherits)(window, c"QQuickWindow".as_ptr().cast()) {
                pending.push((
                    (self.content)(window),
                    if (self.window_visible)(window) {
                        1.0
                    } else {
                        0.0
                    },
                ));
            }
        }
        let mut seen = HashSet::new();
        let mut result = Vec::new();
        while let Some((item, parent_opacity)) = pending.pop() {
            if item.is_null() || !seen.insert(item as usize) {
                continue;
            }
            if seen.len() > 30_000 || result.len() >= 4096 {
                break;
            }
            let alpha = parent_opacity * (self.opacity)(item);
            if (self.inherits)(item, c"QQuickText".as_ptr().cast()) {
                if let Some(source) = self.read(item) {
                    let eligible = alpha > 0.01
                        && (self.visible)(item)
                        && eligible_text(&source, (self.format)(item));
                    result.push(Label {
                        object: item as usize,
                        source,
                        eligible,
                    });
                }
            }
            let mut children = Storage([0; 4]);
            (self.children)(item, std::ptr::from_mut(&mut children).cast());
            pending.extend(self.list(children).into_iter().map(|child| (child, alpha)));
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::supported_qt_version;

    #[test]
    fn only_verified_qt_versions_are_accepted() {
        assert!(supported_qt_version(b"6.8.3"));
        assert!(supported_qt_version(b"6.11.1"));
        assert!(!supported_qt_version(b"6.8.2"));
        assert!(!supported_qt_version(b"6.9.0"));
        assert!(!supported_qt_version(b"6.11.0"));
        assert!(!supported_qt_version(b"6.12.0"));
    }
}
