mod support;

use soma_zero::CounterfactualBackfillSourceBoundaryPreservationStatus;
use support::sprint69_support as sprint;

#[test]
fn counterfactual_backfill_source_boundary_preservation_stays_green() {
    let report = sprint::run_sprint97_bundle(
        "soma_sprint97_counterfactual_backfill_recover.toml",
        "counterfactual-source-boundary",
    )
    .counterfactual_backfill_source_boundary_preservation_report;
    assert_eq!(
        report.source_boundary_status,
        CounterfactualBackfillSourceBoundaryPreservationStatus::SourceBoundaryPreserved
    );
}
