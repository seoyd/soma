mod support;

use soma_zero::CounterfactualBackfillRiskDeniedPreservationStatus;
use support::sprint69_support as sprint;

#[test]
fn counterfactual_backfill_riskdenied_preservation_stays_green() {
    let report = sprint::run_sprint97_bundle(
        "soma_sprint97_counterfactual_backfill_recover.toml",
        "counterfactual-riskdenied",
    )
    .counterfactual_backfill_risk_denied_preservation_report;
    assert_eq!(
        report.risk_denied_status,
        CounterfactualBackfillRiskDeniedPreservationStatus::RiskDeniedCounterfactualPreserved
    );
    assert!(report.risk_denied_not_overridden_by_opportunity_cost);
}
