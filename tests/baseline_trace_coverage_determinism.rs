mod common;
#[path = "support/sprint69_support.rs"]
mod sprint69_support;

#[test]
fn sprint69_baseline_trace_coverage_bundle_is_deterministic() {
    let left = sprint69_support::run_coverage(
        "soma_baseline_snapshot_coverage.toml",
        "coverage-determinism-a",
    );
    let right = sprint69_support::run_coverage(
        "soma_baseline_snapshot_coverage.toml",
        "coverage-determinism-b",
    );
    let left_json = serde_json::to_string_pretty(&left).expect("serialize left");
    let right_json = serde_json::to_string_pretty(&right).expect("serialize right");
    assert_eq!(left_json, right_json);
}
