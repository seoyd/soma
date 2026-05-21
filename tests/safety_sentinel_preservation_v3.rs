mod support;

use support::sprint109_support::{read_fixture, run_sprint109};

#[test]
fn safety_sentinels_and_safety_coverage_are_preserved() {
    let bundle = run_sprint109(
        "soma_safety_sentinel_preservation_v3.toml",
        "safety-sentinel-preservation-v3",
    );
    let actual =
        serde_json::to_value(&bundle.safety_sentinel_preservation_report_v3).expect("actual");
    let expected: serde_json::Value =
        read_fixture("sprint109_data/safety_sentinel_preservation_v3_expected.json");
    assert_eq!(actual, expected);
    let report = bundle.safety_sentinel_preservation_report_v3;
    assert!(report.committee_cli_safety_preserved);
    assert!(report.workspace_cli_safety_preserved);
    assert!(report.workspace_determinism_preserved);
    assert!(report.paper_lifecycle_safety_preserved);
    assert!(report.runtime_deferred_guard_preserved);
    assert_eq!(report.sentinel_status, "SafetySentinelsPreserved");
}
