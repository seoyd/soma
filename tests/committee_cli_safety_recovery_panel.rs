mod support;

use soma_zero::{
    BaselineSignalEntryGateStatus, CommitteeCliSafetyReductionStatus,
    Sprint95CommitteeCliSafetyRecoveryRunner,
};
use support::sprint69_support as sprint;

#[test]
fn control_tower_panel_threads_committee_statuses() {
    let report = Sprint95CommitteeCliSafetyRecoveryRunner::default()
        .run_control_tower_committee_cli_safety_recovery(&sprint::sprint95_config_from_example(
            "soma_control_tower_committee_cli_safety_recovery.toml",
            "control-tower-committee-cli-safety-recovery",
        ))
        .expect("report");
    assert_eq!(
        report.committee_cli_safety_reduction_status,
        CommitteeCliSafetyReductionStatus::CommitteeCliSafetyKeptIsolated
    );
    assert_eq!(
        report.baseline_signal_entry_status,
        BaselineSignalEntryGateStatus::BaselineSignalEntryReady
    );
    assert_eq!(report.runtime_deferred_summary, "RuntimeDeferred");
}
