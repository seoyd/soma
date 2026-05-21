mod support;

use soma_zero::CargoJsonTargetBlockerExtractionReportV1;
use support::sprint118_support::run_sprint118;

#[test]
fn cargo_json_target_blocker_extraction_is_deterministic() {
    let bundle = run_sprint118(
        "soma_cargo_json_target_blocker_extraction_v1.toml",
        "cargo-json-target-blocker-extraction-v1",
    );
    let report: CargoJsonTargetBlockerExtractionReportV1 =
        bundle.cargo_json_target_blocker_extraction_report_v1;
    assert!(
        report
            .target_blockers
            .iter()
            .any(|value| value.contains("workspace_cli_integration"))
    );
    assert!(
        report
            .suspect_targets
            .iter()
            .any(|value| value.contains("macro_link_heavy_suite"))
    );
    assert!(
        report
            .artifact_blockers
            .iter()
            .any(|value| value.contains("workspace_timeout_guard"))
    );
}
