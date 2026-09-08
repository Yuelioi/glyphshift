#![cfg(windows)]
use glyphshift_adapter_registry::{
    AdapterBinding, AdapterHostBinding, ArtifactHash, PackageArtifactId,
};
use glyphshift_adapter_sdk::AdapterDescriptor;
use glyphshift_capture::{CaptureProducerConfiguration, CaptureProducerId};
use glyphshift_domain::{Feature, Generation, RouteProgram};
use glyphshift_runtime_contract::RuntimePublication;
use glyphshift_target_runtime::{
    activate_deployment, deactivate_runtime, query_observations, update_publication,
};
use glyphshift_target_runtime_contract::{NativeAdapterDeployment, TargetRuntimeDeployment};
use glyphshift_translation::{FontPolicy, TranslationSnapshot};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

fn adapter(profile: &Path, file: &str, descriptor: AdapterDescriptor) -> NativeAdapterDeployment {
    let library = profile.join(file);
    let binding = AdapterBinding {
        adapter_id: descriptor.adapter_id().clone(),
        version: descriptor.version(),
        apply_model: descriptor.apply_model(),
        artifact_hash: ArtifactHash::sha256(
            Sha256::digest(std::fs::read(&library).unwrap()).into(),
        ),
        host: AdapterHostBinding::TargetProcess {
            library: PackageArtifactId::new(file),
        },
        descriptor,
        features: vec![Feature::TextObserve, Feature::TextReplace],
    };
    NativeAdapterDeployment::new(library, binding).unwrap()
}

fn publication(generation: u64, translation: &str) -> RuntimePublication {
    RuntimePublication::new(
        RouteProgram::direct("text"),
        TranslationSnapshot::empty(Generation::new(generation)).with_entry(
            "text",
            "Open",
            translation,
        ),
        FontPolicy::empty(),
    )
}

#[test]
#[ignore = "requires the independently compiled MSVC Qt/GDI drawing fixture"]
fn complete_framework_call_suppresses_nested_glyphs_but_keeps_independent_gdi() {
    let root =
        PathBuf::from(std::env::var_os("GLYPHSHIFT_FRAMEWORK_ABI_ROOT").expect("fixture root"));
    let profile = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    unsafe {
        let _core = Box::leak(Box::new(
            libloading::Library::new(root.join("Qt5Core.dll")).unwrap(),
        ));
        let gui = Box::leak(Box::new(
            libloading::Library::new(root.join("Qt5Gui.dll")).unwrap(),
        ));
        let draw = *gui
            .get::<unsafe extern "C" fn(i32, *const u16, i32) -> *const u16>(b"fixture_draw\0")
            .unwrap();
        let raster = *gui
            .get::<unsafe extern "C" fn()>(b"fixture_raster_only\0")
            .unwrap();
        gui.get::<unsafe extern "C" fn(i32)>(b"fixture_set_raster\0")
            .unwrap()(1);
        let render = |kind| {
            let input = "Open".encode_utf16().collect::<Vec<_>>();
            let result = draw(kind, input.as_ptr(), 4);
            let len = (0..64).find(|i| *result.add(*i) == 0).unwrap();
            String::from_utf16(std::slice::from_raw_parts(result, len)).unwrap()
        };
        let deployment = TargetRuntimeDeployment::new(
            publication(1, "打开"),
            [
                adapter(
                    &profile,
                    "glyphshift_adapter_qt_painter_native.dll",
                    glyphshift_adapter_qt_painter::descriptor(),
                ),
                adapter(
                    &profile,
                    "glyphshift_adapter_gdi_native.dll",
                    glyphshift_adapter_gdi::descriptor(),
                ),
            ],
        )
        .with_observation_producer(
            CaptureProducerConfiguration::new(CaptureProducerId::new("scope-fixture").unwrap(), 1)
                .unwrap(),
        );
        activate_deployment(deployment).unwrap();
        // Reproduce the fragment symptom through the actual active GDI adapter.
        raster();
        let isolated = query_observations().unwrap();
        assert_eq!(isolated.records().len(), 4);
        assert!(
            isolated
                .records()
                .iter()
                .all(|r| r.source().len() == 1
                    && r.adapter_id() == glyphshift_adapter_gdi::ADAPTER_ID)
        );
        for kind in 0..4 {
            assert_eq!(render(kind), "打开");
        }
        let complete = query_observations().unwrap();
        assert_eq!(complete.records().len(), 4);
        assert!(complete
            .records()
            .iter()
            .all(|r| r.source() == "Open"
                && r.adapter_id() == glyphshift_adapter_qt_painter::ADAPTER_ID));
        update_publication(publication(2, "第二代")).unwrap();
        assert_eq!(render(0), "第二代");
        let updated = query_observations().unwrap();
        assert!(updated.records().iter().all(|r| r.source() == "Open"));
        // Leaving the call scope must not globally disable low-level observation.
        raster();
        assert_eq!(query_observations().unwrap().records().len(), 4);
        deactivate_runtime().unwrap();
        assert_eq!(render(0), "Open");
    }
}
