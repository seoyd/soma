mod support;

use soma_zero::CounterfactualBackfillEntryConsumedStatus;
use support::sprint69_support as sprint;

#[test]
fn counterfactual_backfill_entry_is_consumed_once_reduction_runs() {
    let report = sprint::run_sprint97_bundle(
        "soma_sprint97_counterfactual_backfill_recover.toml",
        "counterfactual-entry-consumed",
    )
    .counterfactual_backfill_entry_consumed_report;
    assert_eq!(
        report.consumed_status,
        CounterfactualBackfillEntryConsumedStatus::EntryConsumedForCounterfactualBackfill
    );
}
