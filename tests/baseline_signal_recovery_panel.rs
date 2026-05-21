mod support;

use soma_zero::CounterfactualBackfillEntryGateStatus;
use support::sprint69_support as sprint;

#[test]
fn sprint96_control_tower_panel_stays_read_only_and_points_to_counterfactual() {
    let bundle = sprint::run_sprint96_bundle(
        "soma_sprint96_baseline_signal_recover.toml",
        "baseline-signal-recovery-panel",
    );
    let panel = bundle.control_tower_baseline_signal_recovery_panel;
    assert_eq!(panel.primary_next_family, "CounterfactualBackfill");
    assert_eq!(
        panel.counterfactual_backfill_entry_status,
        CounterfactualBackfillEntryGateStatus::CounterfactualBackfillEntryReady
    );
    assert_eq!(panel.runtime_deferred_summary, "ResearchOnly");
    assert_eq!(panel.next_actions.len(), 3);
}
