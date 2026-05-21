mod support;

use soma_zero::CounterfactualBackfillNoFabricatedOutcomeStatus;
use support::sprint69_support as sprint;

#[test]
fn counterfactual_backfill_no_fabricated_outcome_stays_green() {
    let report = sprint::run_sprint97_bundle(
        "soma_sprint97_counterfactual_backfill_recover.toml",
        "counterfactual-no-fabricated",
    )
    .counterfactual_backfill_no_fabricated_outcome_report;
    assert_eq!(
        report.fabrication_status,
        CounterfactualBackfillNoFabricatedOutcomeStatus::NoFabricatedOutcomesPreserved
    );
    assert!(report.missing_outcomes_not_fabricated);
}
