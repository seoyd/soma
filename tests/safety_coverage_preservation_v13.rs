mod support;

use soma_zero::SafetyCoveragePreservationReportV13Status;
use support::sprint69_support as sprint;

#[test]
fn safety_coverage_preservation_v13_stays_green() {
    let report = sprint::run_sprint97_bundle(
        "soma_sprint97_counterfactual_backfill_recover.toml",
        "safety-coverage-v13",
    )
    .safety_coverage_preservation_report_v13;
    assert_eq!(
        report.safety_status,
        SafetyCoveragePreservationReportV13Status::SafetyCoveragePreserved
    );
    assert!(report.queue_closure_separate_from_workspace_acceptance);
}
