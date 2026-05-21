mod support;

use soma_zero::CounterfactualBackfillNoTradePreservationStatus;
use support::sprint69_support as sprint;

#[test]
fn counterfactual_backfill_notrade_preservation_stays_green() {
    let report = sprint::run_sprint97_bundle(
        "soma_sprint97_counterfactual_backfill_recover.toml",
        "counterfactual-notrade",
    )
    .counterfactual_backfill_no_trade_preservation_report;
    assert_eq!(
        report.no_trade_status,
        CounterfactualBackfillNoTradePreservationStatus::NoTradeCounterfactualPreserved
    );
    assert!(report.no_trade_can_remain_best_outcome);
}
