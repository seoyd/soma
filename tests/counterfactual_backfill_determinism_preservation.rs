mod support;

use soma_zero::CounterfactualBackfillDeterminismPreservationStatus;
use support::sprint69_support as sprint;

#[test]
fn counterfactual_backfill_determinism_preservation_stays_green() {
    let report = sprint::run_sprint97_bundle(
        "soma_sprint97_counterfactual_backfill_recover.toml",
        "counterfactual-determinism",
    )
    .counterfactual_backfill_determinism_preservation_report;
    assert_eq!(
        report.determinism_status,
        CounterfactualBackfillDeterminismPreservationStatus::DeterminismPreserved
    );
}
