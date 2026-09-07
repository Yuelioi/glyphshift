//! Opt-in synthetic QML contract using a caller-supplied Qt installation.
#![cfg(all(windows, target_arch = "x86_64"))]
#[path = "../src/qt.rs"]
#[allow(dead_code)]
mod qt;
use glyphshift_adapter_native_abi::*;
use std::{
    ffi::{c_void, CString},
    path::PathBuf,
    sync::atomic::{AtomicU64, AtomicUsize, Ordering},
    time::{Duration, Instant},
};
use windows_sys::Win32::System::LibraryLoader::{
    GetProcAddress, LoadLibraryExW, LOAD_WITH_ALTERED_SEARCH_PATH,
};
type Object = *mut c_void;
static MODE: AtomicU64 = AtomicU64::new(1);
static OBSERVED: AtomicUsize = AtomicUsize::new(0);
static FORBIDDEN: AtomicUsize = AtomicUsize::new(0);
extern "C" fn decide(
    _: Object,
    source: *const u16,
    len: u32,
    out: *mut u16,
    capacity: u32,
    _: *mut u16,
    _: u32,
) -> NativeDecisionV1 {
    let source =
        unsafe { String::from_utf16_lossy(std::slice::from_raw_parts(source, len as usize)) };
    OBSERVED.fetch_add(1, Ordering::Relaxed);
    if source.contains("User content") || source.contains("Rich") {
        FORBIDDEN.fetch_add(1, Ordering::Relaxed);
    }
    let mode = MODE.load(Ordering::Acquire);
    let replacement = match source.as_str() {
        "File" => Some(if mode == 1 { "文件一" } else { "文件二" }),
        "Changed" => Some("新的标题"),
        "Dynamic" => Some("动态标签"),
        _ => None,
    };
    let mut response = NativeDecisionV1 {
        status: STATUS_OK,
        generation: mode,
        decision_bits: 0,
        text_len: 0,
        font_len: 0,
    };
    if let Some(text) = replacement {
        let units = text.encode_utf16().collect::<Vec<_>>();
        assert!(units.len() <= capacity as usize);
        unsafe {
            std::ptr::copy_nonoverlapping(units.as_ptr(), out, units.len());
        }
        response.decision_bits = DECISION_TEXT_REPLACE;
        response.text_len = units.len() as u32;
    }
    response
}
extern "C" fn characters(_: Object, _: *mut u16, _: u32) -> u32 {
    0
}
#[repr(C, align(16))]
struct Storage([u8; 2048]);
impl Storage {
    fn new() -> Self {
        Self([0; 2048])
    }
    fn ptr(&mut self) -> Object {
        self.0.as_mut_ptr().cast()
    }
}

#[test]
#[ignore = "requires GLYPHSHIFT_QT_BIN, GLYPHSHIFT_QT_PLUGIN_PATH and GLYPHSHIFT_QML_IMPORT_PATH for a Qt 6.8.3 installation"]
fn retained_labels_survive_control_thread_exit_updates_and_object_destruction() {
    unsafe {
        let root =
            PathBuf::from(std::env::var_os("GLYPHSHIFT_QT_BIN").expect("explicit Qt runtime"));
        let load = |name: &str| {
            let path = root
                .join(name)
                .as_os_str()
                .to_string_lossy()
                .encode_utf16()
                .chain(Some(0))
                .collect::<Vec<_>>();
            let module = LoadLibraryExW(
                path.as_ptr(),
                std::ptr::null_mut(),
                LOAD_WITH_ALTERED_SEARCH_PATH,
            );
            assert!(!module.is_null(), "required Qt module: {name}");
            module
        };
        let core = load("Qt6Core.dll");
        let gui = load("Qt6Gui.dll");
        let qml = load("Qt6Qml.dll");
        let _quick = load("Qt6Quick.dll");
        macro_rules! f {
            ($module:expr,$name:expr,$ty:ty) => {
                std::mem::transmute::<unsafe extern "system" fn() -> isize, $ty>(
                    GetProcAddress($module, concat!($name, "\0").as_ptr()).expect($name),
                )
            };
        }
        let app_ctor = f!(
            gui,
            "??0QGuiApplication@@QEAA@AEAHPEAPEADH@Z",
            unsafe extern "system" fn(Object, *mut i32, *mut *mut i8, i32) -> Object
        );
        let app_dtor = f!(
            gui,
            "??1QGuiApplication@@UEAA@XZ",
            unsafe extern "system" fn(Object)
        );
        let process = f!(
            core,
            "?processEvents@QCoreApplication@@SAXV?$QFlags@W4ProcessEventsFlag@QEventLoop@@@@@Z",
            unsafe extern "system" fn(i32)
        );
        let arguments = [
            CString::new("synthetic-qt-host").unwrap(),
            CString::new("-platformpluginpath").unwrap(),
            CString::new(
                std::env::var("GLYPHSHIFT_QT_PLUGIN_PATH").expect("explicit plugin directory"),
            )
            .unwrap(),
        ];
        let mut argv = arguments
            .iter()
            .map(|v| v.as_ptr() as *mut i8)
            .chain(Some(std::ptr::null_mut()))
            .collect::<Vec<_>>();
        let mut argc = 3;
        let mut app = Storage::new();
        app_ctor(app.ptr(), &mut argc, argv.as_mut_ptr(), 0);
        let engine_ctor = f!(
            qml,
            "??0QQmlApplicationEngine@@QEAA@PEAVQObject@@@Z",
            unsafe extern "system" fn(Object, Object) -> Object
        );
        let engine_dtor = f!(
            qml,
            "??1QQmlApplicationEngine@@UEAA@XZ",
            unsafe extern "system" fn(Object)
        );
        let add_import = f!(
            qml,
            "?addImportPath@QQmlEngine@@QEAAXAEBVQString@@@Z",
            unsafe extern "system" fn(Object, Object)
        );
        let str_ctor = f!(
            core,
            "??0QString@@QEAA@PEBVQChar@@_J@Z",
            unsafe extern "system" fn(Object, *const u16, i64) -> Object
        );
        let str_dtor = f!(
            core,
            "??1QString@@QEAA@XZ",
            unsafe extern "system" fn(Object)
        );
        let mut engine = Storage::new();
        engine_ctor(engine.ptr(), std::ptr::null_mut());
        let mut import = Storage::new();
        let import_units = std::env::var("GLYPHSHIFT_QML_IMPORT_PATH")
            .expect("explicit QML imports")
            .encode_utf16()
            .collect::<Vec<_>>();
        str_ctor(
            import.ptr(),
            import_units.as_ptr(),
            import_units.len() as i64,
        );
        add_import(engine.ptr(), import.ptr());
        str_dtor(import.ptr());
        let bytes_ctor = f!(
            core,
            "??0QByteArray@@QEAA@PEBD_J@Z",
            unsafe extern "system" fn(Object, *const u8, i64) -> Object
        );
        let bytes_dtor = f!(
            core,
            "??1QByteArray@@QEAA@XZ",
            unsafe extern "system" fn(Object)
        );
        let url_ctor = f!(
            core,
            "??0QUrl@@QEAA@XZ",
            unsafe extern "system" fn(Object) -> Object
        );
        let url_dtor = f!(core, "??1QUrl@@QEAA@XZ", unsafe extern "system" fn(Object));
        let load_data = f!(
            qml,
            "?loadData@QQmlApplicationEngine@@QEAAXAEBVQByteArray@@AEBVQUrl@@@Z",
            unsafe extern "system" fn(Object, Object, Object)
        );
        let source=br#"import QtQuick
Window {visible:true;width:480;height:240
 Text {id:label;objectName:"label";property string caption:"File";property bool dynamicActive:false;text:caption;x:20;y:20;font.pixelSize:24
  Loader {active:label.dynamicActive;y:40;sourceComponent:Text {text:"Dynamic";font.pixelSize:24}}
 }
 TextInput {text:"User content";y:110}
 Text {text:"<b>Rich</b>";textFormat:Text.RichText;y:150}
}"#;
        let mut bytes = Storage::new();
        let mut url = Storage::new();
        bytes_ctor(bytes.ptr(), source.as_ptr(), source.len() as i64);
        url_ctor(url.ptr());
        load_data(engine.ptr(), bytes.ptr(), url.ptr());
        bytes_dtor(bytes.ptr());
        url_dtor(url.ptr());
        let pump = |duration: Duration| {
            let until = Instant::now() + duration;
            while Instant::now() < until {
                process(0);
                std::thread::sleep(Duration::from_millis(5));
            }
        };
        pump(Duration::from_millis(150));
        let qt = qt::Qt::resolve().expect("supported Qt ABI");
        let label = qt
            .labels()
            .into_iter()
            .find(|r| r.source == "File")
            .expect("synthetic root label")
            .object as Object;
        let dll = std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("glyphshift_adapter_qt_quick_native.dll");
        let library = libloading::Library::new(dll).unwrap();
        let entry = library
            .get::<NativeAdapterEntryV1>(ENTRY_SYMBOL_V1)
            .unwrap();
        let api = entry();
        assert_eq!(
            api.descriptor.to_descriptor().unwrap(),
            glyphshift_adapter_qt_quick::descriptor()
        );
        assert_eq!(
            (api.negotiate_features)(3, 1).status,
            STATUS_UNAUTHORIZED_FEATURE
        );
        assert_eq!(
            (api.negotiate_features)(FEATURE_FONT_SUBSTITUTE, FEATURE_FONT_SUBSTITUTE).status,
            STATUS_UNSUPPORTED_FEATURE
        );
        let host = Box::leak(Box::new(NativeRuntimeHostV1 {
            struct_size: std::mem::size_of::<NativeRuntimeHostV1>() as u32,
            context: std::ptr::null_mut(),
            decide_utf16: decide,
            source_characters_utf16: characters,
        })) as *const _ as usize;
        let activate = api.activate;
        // The control thread exits before the GUI consumes its request. A thread-
        // owned SetWindowsHookEx registration cannot satisfy this contract.
        assert_eq!(
            std::thread::spawn(move || activate(host as *const NativeRuntimeHostV1, 3, 3))
                .join()
                .unwrap()
                .status,
            STATUS_OK
        );
        pump(Duration::from_millis(400));
        assert_eq!(qt.read(label).as_deref(), Some("文件一"));
        MODE.store(2, Ordering::Release);
        (api.request_refresh)();
        pump(Duration::from_millis(350));
        assert_eq!(qt.read(label).as_deref(), Some("文件二"));
        let variant_string = f!(
            core,
            "??0QVariant@@QEAA@AEBVQString@@@Z",
            unsafe extern "system" fn(Object, Object) -> Object
        );
        let variant_bool = f!(
            core,
            "??0QVariant@@QEAA@_N@Z",
            unsafe extern "system" fn(Object, bool) -> Object
        );
        let variant_dtor = f!(
            core,
            "??1QVariant@@QEAA@XZ",
            unsafe extern "system" fn(Object)
        );
        let set_property = f!(
            core,
            "?setProperty@QObject@@QEAA_NPEBDAEBVQVariant@@@Z",
            unsafe extern "system" fn(Object, *const u8, Object) -> bool
        );
        let mut value = Storage::new();
        let mut text = Storage::new();
        let changed = "Changed".encode_utf16().collect::<Vec<_>>();
        str_ctor(text.ptr(), changed.as_ptr(), changed.len() as i64);
        variant_string(value.ptr(), text.ptr());
        assert!(set_property(label, c"caption".as_ptr().cast(), value.ptr()));
        variant_dtor(value.ptr());
        str_dtor(text.ptr());
        pump(Duration::from_millis(350));
        assert_eq!(qt.read(label).as_deref(), Some("新的标题"));
        variant_bool(value.ptr(), true);
        assert!(set_property(
            label,
            c"dynamicActive".as_ptr().cast(),
            value.ptr()
        ));
        variant_dtor(value.ptr());
        pump(Duration::from_millis(350));
        assert!(qt.labels().iter().any(|r| r.source == "动态标签"));
        variant_bool(value.ptr(), false);
        assert!(set_property(
            label,
            c"dynamicActive".as_ptr().cast(),
            value.ptr()
        ));
        variant_dtor(value.ptr());
        pump(Duration::from_millis(350));
        assert!(!qt.labels().iter().any(|r| r.source == "动态标签"));
        let stop = api.deactivate;
        let control = std::thread::spawn(move || stop());
        while !control.is_finished() {
            process(0);
            std::thread::sleep(Duration::from_millis(5));
        }
        assert_eq!(control.join().unwrap(), STATUS_OK);
        assert_eq!(qt.read(label).as_deref(), Some("Changed"));
        assert_eq!(
            std::thread::spawn(move || activate(host as *const NativeRuntimeHostV1, 3, 3))
                .join()
                .unwrap()
                .status,
            STATUS_OK
        );
        pump(Duration::from_millis(350));
        assert_eq!(qt.read(label).as_deref(), Some("新的标题"));
        let stop = api.deactivate;
        let control = std::thread::spawn(move || stop());
        while !control.is_finished() {
            process(0);
            std::thread::sleep(Duration::from_millis(5));
        }
        assert_eq!(control.join().unwrap(), STATUS_OK);
        assert_eq!(qt.read(label).as_deref(), Some("Changed"));
        assert!(OBSERVED.load(Ordering::Relaxed) > 0);
        assert_eq!(FORBIDDEN.load(Ordering::Relaxed), 0);
        engine_dtor(engine.ptr());
        app_dtor(app.ptr());
    }
}
