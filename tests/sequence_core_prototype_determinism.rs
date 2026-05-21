#[path = "support/sprint69_support.rs"]
mod support;

#[test]
fn sprint80_outputs_are_deterministic() {
    let first =
        support::run_sprint80_bundle("soma_sequence_core_prototype_compare.toml", "determinism-a");
    let second =
        support::run_sprint80_bundle("soma_sequence_core_prototype_compare.toml", "determinism-b");
    assert_eq!(first, second);
    assert!(first.final_summary.contains("prototype_comparison_status"));
    assert!(first.final_summary.contains("artifact_population_status"));
}
