//! Synthetic MV bridge publications; no installed-game paths or product catalog entry.
use glyphshift_adapter_registry::{AdapterBinding, AdapterHostBinding, ArtifactHash, PackageArtifactId};
use glyphshift_adapter_sdk::{AdapterDescriptor, AdapterVersion};
use glyphshift_capture::{CaptureProducerConfiguration, CaptureProducerId};
use glyphshift_domain::{AdapterId, ApplyModel, Feature, Generation, Placement, RouteProgram};
use glyphshift_runtime_contract::RuntimePublication;
use glyphshift_target_runtime_contract::{NativeAdapterDeployment, TargetRuntimeDeployment};
use glyphshift_translation::{FontPolicy, TranslationSnapshot};
use sha2::{Digest, Sha256};
use std::{fs, path::PathBuf};

fn main() {
    let mut args = std::env::args_os().skip(1);
    let native = PathBuf::from(args.next().expect("native fixture bridge"));
    let output = PathBuf::from(args.next().expect("local test evidence directory"));
    fs::create_dir_all(&output).unwrap();
    let descriptor = AdapterDescriptor::new(
        AdapterId::new("fixture.rpgmaker-mv.runtime-bridge"), AdapterVersion::new(1, 0, 0),
        ApplyModel::InlineRender, Placement::TargetProcess,
        [Feature::TextObserve, Feature::TextReplace],
    ).with_platforms(["windows"]).with_architectures(["x86"]);
    let binding = AdapterBinding {
        adapter_id: descriptor.adapter_id().clone(), version: descriptor.version(),
        apply_model: descriptor.apply_model(), descriptor,
        artifact_hash: ArtifactHash::sha256(Sha256::digest(fs::read(&native).unwrap()).into()),
        host: AdapterHostBinding::TargetProcess { library: PackageArtifactId::new("fixtures/mv-bridge") },
        features: vec![Feature::TextObserve, Feature::TextReplace],
    };
    for (generation, translation) in [
        (1, "今天，我们来探索新事物。\n准备好后，选择一条路。"),
        (2, "今天，一起开始新的探索。\n准备好了就选条路吧。"),
    ] {
        let snapshot = TranslationSnapshot::empty(Generation::new(generation)).with_entry(
            "text", "Today, we are going to explore something new.\nChoose a path when you are ready.", translation);
        let publication = RuntimePublication::new(RouteProgram::direct("text"), snapshot, FontPolicy::empty());
        fs::write(output.join(format!("publication-{generation}.json")), publication.encode_json().unwrap()).unwrap();
        let deployment = TargetRuntimeDeployment::new(publication,
            [NativeAdapterDeployment::new(&native, binding.clone()).unwrap()])
            .with_observation_producer(CaptureProducerConfiguration::new(
                CaptureProducerId::new("mv-runtime-contract").unwrap(), generation).unwrap());
        fs::write(output.join(format!("deployment-{generation}.json")), deployment.encode_json().unwrap()).unwrap();
    }
}
