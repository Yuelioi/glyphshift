//! Generates deterministic publications for the real Runtime/MonoGame contract.
use glyphshift_adapter_registry::{AdapterBinding, AdapterHostBinding, ArtifactHash, PackageArtifactId};
use glyphshift_capture::{CaptureProducerConfiguration, CaptureProducerId};
use glyphshift_domain::{Feature, Generation, RouteProgram};
use glyphshift_runtime_contract::RuntimePublication;
use glyphshift_target_runtime_contract::{NativeAdapterDeployment, TargetRuntimeDeployment};
use glyphshift_translation::{FontPolicy, TranslationSnapshot};
use sha2::{Digest, Sha256};
use std::{fs, path::PathBuf};

fn main() {
    let mut args = std::env::args_os().skip(1);
    let native = PathBuf::from(args.next().expect("native adapter"));
    let output = PathBuf::from(args.next().expect("local test evidence directory"));
    fs::create_dir_all(&output).unwrap();
    let descriptor = glyphshift_adapter_monogame::descriptor();
    let binding = AdapterBinding {
        adapter_id: descriptor.adapter_id().clone(), version: descriptor.version(),
        apply_model: descriptor.apply_model(), descriptor,
        artifact_hash: ArtifactHash::sha256(Sha256::digest(fs::read(&native).unwrap()).into()),
        host: AdapterHostBinding::TargetProcess { library: PackageArtifactId::new("adapters/monogame") },
        features: vec![Feature::TextObserve, Feature::TextReplace],
    };
    for (generation, translation) in [(1, "中文"), (2, "中"), (3, "斧头"), (4, "中"), (5, "A中B"), (6, "中"), (7, "中")] {
        let mut snapshot = TranslationSnapshot::empty(Generation::new(generation)).with_entry("text", "AAA", translation);
        snapshot = snapshot.with_entry("text", "A B A B", "中文中文")
            .with_entry("text", "B \r\nA B", "中")
            .with_entry("text", "B \r\nB A", "中")
            .with_entry("text", "B B \r\nA", "文");
        if generation == 7 { snapshot = snapshot.with_entry("text", " AAA ", "B"); }
        let publication = RuntimePublication::new(RouteProgram::direct("text"), snapshot, FontPolicy::empty());
        fs::write(output.join(format!("publication-{generation}.json")), publication.encode_json().unwrap()).unwrap();
        let deployment = TargetRuntimeDeployment::new(publication,
            [NativeAdapterDeployment::new(&native, binding.clone()).unwrap()])
            .with_observation_producer(CaptureProducerConfiguration::new(CaptureProducerId::new("monogame-runtime-contract").unwrap(), generation).unwrap());
        fs::write(output.join(format!("deployment-{generation}.json")), deployment.encode_json().unwrap()).unwrap();
    }
}
