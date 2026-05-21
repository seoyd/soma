#[path = "support/sprint69_support.rs"]
mod support;

#[test]
fn sprint78_outputs_are_deterministic() {
    let first = support::run_sprint78_bundle("soma_core_completion_v2.toml", "sprint78-a");
    let second = support::run_sprint78_bundle("soma_core_completion_v2.toml", "sprint78-b");
    assert_eq!(first, second);
    assert!(first.final_summary.contains("training_storage_status"));
    assert!(first.final_summary.contains("roadmap_recommendation"));
}
