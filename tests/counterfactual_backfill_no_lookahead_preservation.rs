mod support;

use soma_zero::CounterfactualBackfillNoLookaheadPreservationStatus;
use support::sprint69_support as sprint;

#[test]
fn counterfactual_backfill_no_lookahead_preservation_stays_green() {
    let report = sprint::run_sprint97_bundle(
        "soma_sprint97_counterfactual_backfill_recover.toml",
        "counterfactual-no-lookahead",
    )
    .counterfactual_backfill_no_lookahead_preservation_report;
    assert_eq!(
        report.no_lookahead_status,
        CounterfactualBackfillNoLookaheadPreservationStatus::NoLookaheadPreserved
    );
}
