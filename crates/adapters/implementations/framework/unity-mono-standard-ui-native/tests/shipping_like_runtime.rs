use glyphshift_adapter_unity_mono_standard_ui::{
    ManagedObjectId, ManagedText, ObserverEvent, StandardUiKind,
};
use glyphshift_adapter_unity_mono_standard_ui_native::{
    ObserverDriver, ObserverDriverError, StandardUiObservationRuntime, StandardUiProfile,
};
use std::cell::Cell;
use std::collections::{BTreeMap, VecDeque};
use std::rc::Rc;

#[derive(Clone, Copy, PartialEq, Eq)]
enum FixtureBackend {
    Mono,
    Il2Cpp,
}

struct FixtureDomain {
    assemblies: Vec<FixtureAssembly>,
}

struct FixtureAssembly {
    image: FixtureImage,
}

struct FixtureImage {
    classes: Vec<FixtureClass>,
}

struct FixtureClass {
    namespace: &'static str,
    name: &'static str,
    methods: Vec<&'static str>,
}

struct FixtureObject {
    object_id: ManagedObjectId,
    namespace: &'static str,
    class_name: &'static str,
    text: FixtureString,
}

struct FixtureString(Vec<u16>);

enum FixtureCallback {
    MainThreadSnapshot,
    Setter {
        object_id: ManagedObjectId,
        text: FixtureString,
    },
    Collected(ManagedObjectId),
}

struct ShippingLikeMonoRuntime {
    backend: FixtureBackend,
    domain: FixtureDomain,
    objects: BTreeMap<ManagedObjectId, FixtureObject>,
    callbacks: VecDeque<FixtureCallback>,
    recognized: BTreeMap<(&'static str, &'static str), StandardUiKind>,
    stopped: Rc<Cell<bool>>,
}

impl ShippingLikeMonoRuntime {
    fn standard_ui() -> (Self, Rc<Cell<bool>>) {
        let stopped = Rc::new(Cell::new(false));
        let runtime = Self {
            backend: FixtureBackend::Mono,
            domain: FixtureDomain {
                assemblies: vec![FixtureAssembly {
                    image: FixtureImage {
                        classes: vec![
                            FixtureClass {
                                namespace: "UnityEngine.UI",
                                name: "CanvasUpdateRegistry",
                                methods: vec!["PerformUpdate"],
                            },
                            FixtureClass {
                                namespace: "UnityEngine",
                                name: "Object",
                                methods: vec!["FindObjectsOfType"],
                            },
                            FixtureClass {
                                namespace: "TMPro",
                                name: "TMP_Text",
                                methods: vec!["set_text", "SetText"],
                            },
                            FixtureClass {
                                namespace: "UnityEngine.UI",
                                name: "Text",
                                methods: vec!["set_text"],
                            },
                        ],
                    },
                }],
            },
            objects: [
                FixtureObject {
                    object_id: ManagedObjectId::new(1),
                    namespace: "TMPro",
                    class_name: "TMP_Text",
                    text: FixtureString("Score: 0".encode_utf16().collect()),
                },
                FixtureObject {
                    object_id: ManagedObjectId::new(2),
                    namespace: "UnityEngine.UI",
                    class_name: "Text",
                    text: FixtureString("Download Resources".encode_utf16().collect()),
                },
            ]
            .into_iter()
            .map(|object| (object.object_id, object))
            .collect(),
            callbacks: [
                FixtureCallback::MainThreadSnapshot,
                FixtureCallback::Setter {
                    object_id: ManagedObjectId::new(1),
                    text: FixtureString("Score: 0".encode_utf16().collect()),
                },
                FixtureCallback::Setter {
                    object_id: ManagedObjectId::new(1),
                    text: FixtureString("Score: 248".encode_utf16().collect()),
                },
                FixtureCallback::Setter {
                    object_id: ManagedObjectId::new(2),
                    text: FixtureString(Vec::new()),
                },
                FixtureCallback::Setter {
                    object_id: ManagedObjectId::new(2),
                    text: FixtureString(vec![0xd800]),
                },
                FixtureCallback::Collected(ManagedObjectId::new(1)),
            ]
            .into(),
            recognized: BTreeMap::new(),
            stopped: Rc::clone(&stopped),
        };
        (runtime, stopped)
    }

    fn ngui_only() -> Self {
        let (mut runtime, _) = Self::standard_ui();
        runtime.domain.assemblies[0].image.classes.truncate(2);
        runtime.domain.assemblies[0]
            .image
            .classes
            .push(FixtureClass {
                namespace: "NGUI",
                name: "UILabel",
                methods: vec!["set_text"],
            });
        runtime.objects.clear();
        runtime.callbacks.clear();
        runtime
    }

    fn unknown_ui() -> Self {
        let (mut runtime, _) = Self::standard_ui();
        runtime.domain.assemblies[0].image.classes.truncate(2);
        runtime.domain.assemblies[0]
            .image
            .classes
            .push(FixtureClass {
                namespace: "Game.Ui",
                name: "MeshLabel",
                methods: vec!["SetCaption"],
            });
        runtime.objects.clear();
        runtime.callbacks.clear();
        runtime
    }

    fn standard_ui_without_live_objects() -> (Self, Rc<Cell<bool>>) {
        let (mut runtime, stopped) = Self::standard_ui();
        runtime.objects.clear();
        runtime.callbacks = [FixtureCallback::MainThreadSnapshot].into();
        (runtime, stopped)
    }

    fn kind_for(class: &FixtureClass) -> Option<StandardUiKind> {
        match (class.namespace, class.name) {
            ("TMPro", "TMP_Text")
                if class
                    .methods
                    .iter()
                    .any(|method| matches!(*method, "set_text" | "SetText")) =>
            {
                Some(StandardUiKind::TextMeshPro)
            }
            ("UnityEngine.UI", "Text") if class.methods.contains(&"set_text") => {
                Some(StandardUiKind::UGui)
            }
            _ => None,
        }
    }

    fn managed_text(&self, object: &FixtureObject) -> Option<ManagedText> {
        let kind = *self
            .recognized
            .get(&(object.namespace, object.class_name))?;
        Some(ManagedText::utf16(
            object.object_id,
            kind,
            object.text.0.clone(),
        ))
    }
}

impl StandardUiObservationRuntime for ShippingLikeMonoRuntime {
    fn recognize_standard_ui(&mut self) -> Result<StandardUiProfile, ObserverDriverError> {
        if self.backend != FixtureBackend::Mono {
            return Err(ObserverDriverError::BackendMismatch);
        }

        self.recognized.clear();
        let mut has_main_thread_dispatch = false;
        let mut has_object_enumeration = false;
        for assembly in &self.domain.assemblies {
            for class in &assembly.image.classes {
                has_main_thread_dispatch |= class.namespace == "UnityEngine.UI"
                    && class.name == "CanvasUpdateRegistry"
                    && class.methods.contains(&"PerformUpdate");
                has_object_enumeration |= class.namespace == "UnityEngine"
                    && class.name == "Object"
                    && class.methods.contains(&"FindObjectsOfType");
                if let Some(kind) = Self::kind_for(class) {
                    self.recognized.insert((class.namespace, class.name), kind);
                }
            }
        }
        if !has_main_thread_dispatch || !has_object_enumeration {
            return Err(ObserverDriverError::MainThreadDispatchUnavailable);
        }
        let profile = StandardUiProfile::from_kinds(self.recognized.values().copied());
        if profile.is_empty() {
            Err(ObserverDriverError::StandardUiUnavailable)
        } else {
            Ok(profile)
        }
    }

    fn next_observer_event(&mut self) -> Result<Option<ObserverEvent>, ObserverDriverError> {
        let Some(callback) = self.callbacks.pop_front() else {
            return Ok(None);
        };
        let event = match callback {
            FixtureCallback::MainThreadSnapshot => ObserverEvent::AttachSnapshot(
                self.objects
                    .values()
                    .filter_map(|object| self.managed_text(object))
                    .collect(),
            ),
            FixtureCallback::Setter { object_id, text } => {
                let Some(object) = self.objects.get_mut(&object_id) else {
                    return Ok(None);
                };
                object.text = text;
                let kind = *self
                    .recognized
                    .get(&(object.namespace, object.class_name))
                    .ok_or(ObserverDriverError::RuntimeFailure)?;
                ObserverEvent::Setter(ManagedText::utf16(
                    object.object_id,
                    kind,
                    object.text.0.clone(),
                ))
            }
            FixtureCallback::Collected(object_id) => {
                self.objects.remove(&object_id);
                ObserverEvent::Collected(object_id)
            }
        };
        Ok(Some(event))
    }

    fn stop(&mut self) -> Result<(), ObserverDriverError> {
        self.callbacks.clear();
        self.recognized.clear();
        self.stopped.set(true);
        Ok(())
    }
}

#[test]
fn observes_main_thread_snapshot_and_only_changed_setter_values() {
    let (runtime, stopped) = ShippingLikeMonoRuntime::standard_ui();
    let mut driver = ObserverDriver::activate(runtime).expect("standard UI should activate");
    assert!(driver.profile().supports(StandardUiKind::TextMeshPro));
    assert!(driver.profile().supports(StandardUiKind::UGui));

    let initial = driver
        .poll()
        .expect("main-thread snapshot should be accepted");
    assert_eq!(
        initial.iter().map(|text| text.source()).collect::<Vec<_>>(),
        ["Score: 0", "Download Resources"]
    );
    assert!(driver
        .poll()
        .expect("duplicate setter should be valid")
        .is_empty());
    assert_eq!(
        driver.poll().expect("changed setter should be valid")[0].source(),
        "Score: 248"
    );
    assert!(driver
        .poll()
        .expect("empty setter should be valid")
        .is_empty());
    assert!(driver
        .poll()
        .expect("invalid UTF-16 should fail open")
        .is_empty());
    assert!(driver
        .poll()
        .expect("collection should be valid")
        .is_empty());

    driver.stop().expect("stop should clean runtime state");
    assert!(stopped.get());
    assert!(driver.poll().expect("stopped driver is inert").is_empty());
}

#[test]
fn rejects_ngui_only_unknown_ui_and_il2cpp_fixtures() {
    assert_eq!(
        ObserverDriver::activate(ShippingLikeMonoRuntime::ngui_only())
            .err()
            .expect("NGUI-only fixture must be rejected"),
        ObserverDriverError::StandardUiUnavailable
    );
    assert_eq!(
        ObserverDriver::activate(ShippingLikeMonoRuntime::unknown_ui())
            .err()
            .expect("unknown UI fixture must be rejected"),
        ObserverDriverError::StandardUiUnavailable
    );

    let (mut runtime, _) = ShippingLikeMonoRuntime::standard_ui();
    runtime.backend = FixtureBackend::Il2Cpp;
    assert_eq!(
        ObserverDriver::activate(runtime)
            .err()
            .expect("IL2CPP fixture must be rejected"),
        ObserverDriverError::BackendMismatch
    );
}

#[test]
fn setter_before_main_thread_snapshot_fails_closed_and_stops_runtime() {
    let (mut runtime, stopped) = ShippingLikeMonoRuntime::standard_ui();
    runtime.callbacks.pop_front();
    let mut driver = ObserverDriver::activate(runtime).expect("metadata should be recognized");

    assert_eq!(
        driver.poll().expect_err("initial snapshot is mandatory"),
        ObserverDriverError::InitialSnapshotUnavailable
    );
    assert!(stopped.get());
    assert!(driver.poll().expect("failed driver stays inert").is_empty());
}

#[test]
fn second_attach_snapshot_fails_closed_instead_of_resetting_observer_state() {
    let (mut runtime, stopped) = ShippingLikeMonoRuntime::standard_ui();
    runtime
        .callbacks
        .insert(1, FixtureCallback::MainThreadSnapshot);
    let mut driver = ObserverDriver::activate(runtime).expect("metadata should be recognized");
    assert_eq!(driver.poll().expect("first snapshot is required").len(), 2);

    assert_eq!(
        driver
            .poll()
            .expect_err("a second attach snapshot is invalid"),
        ObserverDriverError::UnexpectedSnapshot
    );
    assert!(stopped.get());
}

#[test]
fn rejects_standard_ui_when_no_verified_main_thread_dispatch_exists() {
    let (mut runtime, stopped) = ShippingLikeMonoRuntime::standard_ui();
    runtime.domain.assemblies[0].image.classes.remove(0);

    assert_eq!(
        ObserverDriver::activate(runtime)
            .err()
            .expect("missing main-thread dispatch must be rejected"),
        ObserverDriverError::MainThreadDispatchUnavailable
    );
    assert!(stopped.get());
}

#[test]
fn rejects_loaded_standard_ui_metadata_without_live_text_objects() {
    let (runtime, stopped) = ShippingLikeMonoRuntime::standard_ui_without_live_objects();
    let mut driver = ObserverDriver::activate(runtime).expect("metadata recognition");

    assert_eq!(
        driver.poll().expect_err("empty snapshot must reject"),
        ObserverDriverError::InitialSnapshotUnavailable
    );
    assert!(stopped.get());
}
