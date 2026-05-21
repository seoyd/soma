mod support;

use soma_zero::{
    ExhaustiveCliSmokeManifestStatus, RepresentativeCliSmokeHarnessStatus,
    SafetyCliSmokeManifestStatus,
};
use support::sprint69_support as sprint;

#[test]
fn representative_exhaustive_and_safety_smoke_artifacts_cover_key_commands() {
    let bundle = sprint::run_sprint84_bundle(
        "soma_representative_smoke_harness.toml",
        "sprint84-smoke-harness",
    );
    assert_eq!(
        bundle.representative_cli_smoke_harness.harness_status,
        RepresentativeCliSmokeHarnessStatus::RepresentativeSmokeReady
    );
    assert!(
        bundle
            .representative_cli_smoke_harness
            .representative_commands
            .contains(&"official-evidence-depth-expand".to_string())
    );
    assert!(
        bundle
            .representative_cli_smoke_harness
            .representative_commands
            .contains(&"sprint83-acceptance-recovery".to_string())
    );
    assert_eq!(
        bundle.exhaustive_cli_smoke_manifest.manifest_status,
        ExhaustiveCliSmokeManifestStatus::ExhaustiveManifestReady
    );
    assert_eq!(
        bundle.safety_cli_smoke_manifest.manifest_status,
        SafetyCliSmokeManifestStatus::SafetySmokeReady
    );
    assert!(
        bundle
            .safety_cli_smoke_manifest
            .forbidden_command_checks
            .contains(&"train-model".to_string())
    );
}
