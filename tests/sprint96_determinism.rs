mod support;

use support::sprint69_support as sprint;

#[test]
fn sprint96_bundle_is_deterministic() {
    let first = sprint::run_sprint96_bundle(
        "soma_sprint96_baseline_signal_recover.toml",
        "sprint96-determinism-a",
    );
    let second = sprint::run_sprint96_bundle(
        "soma_sprint96_baseline_signal_recover.toml",
        "sprint96-determinism-b",
    );
    assert_eq!(
        first.baseline_signal_real_reduction_plan,
        second.baseline_signal_real_reduction_plan
    );
    assert_eq!(
        first.baseline_signal_real_reduction_report,
        second.baseline_signal_real_reduction_report
    );
    assert_eq!(
        first.counterfactual_backfill_entry_gate,
        second.counterfactual_backfill_entry_gate
    );
    assert_eq!(
        first.control_tower_baseline_signal_recovery_panel,
        second.control_tower_baseline_signal_recovery_panel
    );
    assert_eq!(first.final_summary, second.final_summary);
}
