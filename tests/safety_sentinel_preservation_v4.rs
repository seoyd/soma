mod support;

use support::sprint110_support::run_sprint110;

#[test]
fn safety_sentinels_and_safety_coverage_are_preserved() {
    let bundle = run_sprint110(
        "soma_safety_sentinel_preservation_v4.toml",
        "safety-sentinel-preservation-v4",
    );
    let report = bundle.safety_sentinel_preservation_report_v4;
    assert!(report.committee_cli_safety_preserved);
    assert!(report.workspace_cli_safety_preserved);
    assert!(report.workspace_determinism_preserved);
    assert!(report.paper_lifecycle_safety_preserved);
    assert!(report.runtime_deferred_guard_preserved);
    assert_eq!(report.sentinel_status, "SafetySentinelsPreserved");
}
