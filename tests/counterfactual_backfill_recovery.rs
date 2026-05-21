mod support;

use soma_zero::{CounterfactualBackfillRecoveryStatus, Sprint88SevenBlockerRecoveryRunner};
use support::sprint69_support as sprint;

#[test]
fn counterfactual_backfill_recovery_preserves_no_lookahead_and_determinism() {
    let config = sprint::sprint88_config_from_example(
        "soma_counterfactual_backfill_recovery.toml",
        "counterfactual-recovery",
    );
    let report = Sprint88SevenBlockerRecoveryRunner::default()
        .run_counterfactual_backfill_recovery(&config)
        .expect("report");
    assert!(report.no_trade_counterfactual_covered);
    assert!(report.risk_denied_counterfactual_covered);
    assert!(report.defensive_value_covered);
    assert!(report.opportunity_cost_covered);
    assert!(report.no_fabricated_outcomes_covered);
    assert!(report.no_lookahead_preserved);
    assert!(report.deterministic_backfill_covered);
    assert_eq!(
        report.recovery_status,
        CounterfactualBackfillRecoveryStatus::CounterfactualBackfillReduced
    );
}
