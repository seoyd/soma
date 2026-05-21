mod support;

use support::sprint108_support::{read_fixture, run_sprint108};

#[test]
fn safety_sentinels_and_safety_coverage_are_preserved() {
    let bundle = run_sprint108(
        "soma_safety_sentinel_preservation_v2.toml",
        "safety-sentinel-preservation-v2",
    );
    let actual =
        serde_json::to_value(&bundle.safety_coverage_preservation_report_v24).expect("actual");
    let expected: serde_json::Value =
        read_fixture("sprint108_data/safety_coverage_v24_expected.json");
    assert_eq!(actual, expected);
    let report = bundle.safety_sentinel_preservation_report_v2;
    assert!(report.committee_cli_safety_preserved);
    assert!(report.workspace_cli_safety_preserved);
    assert!(report.workspace_determinism_preserved);
    assert!(report.paper_lifecycle_safety_preserved);
    assert!(report.runtime_deferred_guard_preserved);
    assert_eq!(report.sentinel_status, "SafetySentinelsPreserved");
}
