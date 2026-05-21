mod support;

use soma_zero::CounterfactualBackfillDefensiveValuePreservationStatus;
use support::sprint69_support as sprint;

#[test]
fn counterfactual_backfill_defensive_value_preservation_stays_green() {
    let report = sprint::run_sprint97_bundle(
        "soma_sprint97_counterfactual_backfill_recover.toml",
        "counterfactual-defensive-value",
    )
    .counterfactual_backfill_defensive_value_preservation_report;
    assert_eq!(
        report.defensive_status,
        CounterfactualBackfillDefensiveValuePreservationStatus::DefensiveValuePreserved
    );
    assert!(report.prevented_drawdown_preserved);
}
