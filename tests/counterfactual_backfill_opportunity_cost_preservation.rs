mod support;

use soma_zero::CounterfactualBackfillOpportunityCostPreservationStatus;
use support::sprint69_support as sprint;

#[test]
fn counterfactual_backfill_opportunity_cost_preservation_stays_green() {
    let report = sprint::run_sprint97_bundle(
        "soma_sprint97_counterfactual_backfill_recover.toml",
        "counterfactual-opportunity-cost",
    )
    .counterfactual_backfill_opportunity_cost_preservation_report;
    assert_eq!(
        report.opportunity_status,
        CounterfactualBackfillOpportunityCostPreservationStatus::OpportunityCostPreserved
    );
    assert!(report.opportunity_cost_not_allowed_to_override_risk_veto);
}
