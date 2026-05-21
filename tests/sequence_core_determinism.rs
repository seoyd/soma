#[path = "support/sprint69_support.rs"]
mod support;

#[test]
fn sprint79_outputs_are_deterministic() {
    let first = support::run_sprint79_bundle(
        "soma_sequence_core_registry.toml",
        "soma_training_storage_materialize.toml",
        "determinism-a",
    );
    let second = support::run_sprint79_bundle(
        "soma_sequence_core_registry.toml",
        "soma_training_storage_materialize.toml",
        "determinism-b",
    );
    assert_eq!(first, second);
    assert!(first.final_summary.contains("registry_status"));
    assert!(first.final_summary.contains("storage_integrity_status"));
}
