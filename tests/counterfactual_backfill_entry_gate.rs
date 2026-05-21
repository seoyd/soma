mod support;

use soma_zero::CounterfactualBackfillEntryGateStatus;
use support::sprint69_support as sprint;

#[test]
fn counterfactual_backfill_entry_gate_is_ready_only_after_baseline_signal() {
    let bundle = sprint::run_sprint96_bundle(
        "soma_sprint96_baseline_signal_recover.toml",
        "counterfactual-backfill-entry-gate",
    );
    let gate = bundle.counterfactual_backfill_entry_gate;
    assert_eq!(
        gate.gate_status,
        CounterfactualBackfillEntryGateStatus::CounterfactualBackfillEntryReady
    );
    assert!(gate.counterfactual_backfill_next_allowed);
}
