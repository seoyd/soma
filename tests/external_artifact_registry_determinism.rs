mod common;
#[path = "support/sprint64_support.rs"]
mod sprint64_support;

use soma_zero::ExternalArtifactRegistryRunner;

#[test]
fn same_fixture_input_produces_same_registry_bundle() {
    let first = sprint64_support::registry_config_from_example(
        "soma_external_artifact_registry.toml",
        "determinism-first",
    );
    let second = sprint64_support::registry_config_from_example(
        "soma_external_artifact_registry.toml",
        "determinism-second",
    );
    let first_bundle = ExternalArtifactRegistryRunner::default()
        .run(&first)
        .expect("run first registry bundle");
    let second_bundle = ExternalArtifactRegistryRunner::default()
        .run(&second)
        .expect("run second registry bundle");
    assert_eq!(
        first_bundle.artifact_registry,
        second_bundle.artifact_registry
    );
    assert_eq!(
        first_bundle.evaluation_history_report,
        second_bundle.evaluation_history_report
    );
    assert_eq!(
        first_bundle.conservative_leaderboard,
        second_bundle.conservative_leaderboard
    );
    assert_eq!(
        first_bundle.previous_external_comparison_report,
        second_bundle.previous_external_comparison_report
    );
    assert_eq!(first_bundle.final_summary, second_bundle.final_summary);
}
