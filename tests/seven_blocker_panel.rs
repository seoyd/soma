mod support;

use soma_zero::{SevenBlockerFamilyRecoveryStatus, Sprint88SevenBlockerRecoveryRunner};
use support::sprint69_support as sprint;

#[test]
fn seven_blocker_panel_surfaces_queue_probe_gate_and_safety_state() {
    let config = sprint::sprint88_config_from_example(
        "soma_control_tower_seven_blocker.toml",
        "seven-blocker-panel",
    );
    let first = Sprint88SevenBlockerRecoveryRunner::default()
        .run_control_tower_seven_blocker(&config)
        .expect("first");
    let second = Sprint88SevenBlockerRecoveryRunner::default()
        .run_control_tower_seven_blocker(&config)
        .expect("second");
    assert_eq!(
        first.seven_blocker_recovery_status,
        SevenBlockerFamilyRecoveryStatus::SevenBlockerRecoveryReadyWithWarnings
    );
    assert_eq!(first.primary_next_family, "CandleExpansionOps");
    assert!(first.family_statuses.contains_key("CommitteeCliSafety"));
    assert!(
        first
            .per_family_probe_status
            .contains_key("ExternalPrediction")
    );
    assert!(first.no_run_gate_status.contains("RealNoRunStillBlocked"));
    assert!(
        first
            .full_workspace_gate_status
            .contains("FullWorkspaceStillBlocked")
    );
    assert!(first.measured_delta_status.contains("SampleBackedOnly"));
    assert!(
        first
            .safety_coverage_status
            .contains("SafetyCoveragePreserved")
    );
    assert!(
        first
            .committee_cli_safety_isolation_status
            .contains("CommitteeCliSafetyKeptIsolated")
    );
    assert!(first.runtime_deferred_status.contains("research-only"));
    assert_eq!(first, second);
}
