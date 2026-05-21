mod support;

use soma_zero::{SevenBlockerFamilyRecoveryStatus, Sprint88SevenBlockerRecoveryRunner};
use support::sprint69_support as sprint;

#[test]
fn seven_blocker_config_defaults_preserve_queue_and_safety() {
    let config = sprint::sprint88_config_from_example(
        "soma_seven_blocker_family_recovery.toml",
        "seven-blocker-config",
    );
    assert_eq!(
        config
            .ordered_family_queue
            .iter()
            .map(|family| format!("{family:?}"))
            .collect::<Vec<_>>(),
        vec![
            "CandleExpansionOps",
            "ExternalPrediction",
            "KrxEvidence",
            "DashboardRenderer",
            "CommitteeCliSafety",
            "BaselineSignal",
            "CounterfactualBackfill",
        ]
    );
    assert!(config.preserve_assertions);
    assert!(config.preserve_safety_guards);
    assert!(config.require_committee_cli_safety_isolation);
}

#[test]
fn seven_blocker_config_rejects_remote_paths() {
    let mut config = sprint::sprint88_config_from_example(
        "soma_seven_blocker_family_recovery.toml",
        "seven-blocker-remote",
    );
    config.blocker_drilldown_paths = vec!["https://example.com/queue.json".to_string()];
    assert!(config.validate().is_err());
}

#[test]
fn seven_blocker_report_is_deterministic_and_keeps_primary_next_family() {
    let config = sprint::sprint88_config_from_example(
        "soma_seven_blocker_family_recovery.toml",
        "seven-blocker-run",
    );
    let first = Sprint88SevenBlockerRecoveryRunner::default()
        .run_seven_blocker_family_recovery(&config)
        .expect("first");
    let second = Sprint88SevenBlockerRecoveryRunner::default()
        .run_seven_blocker_family_recovery(&config)
        .expect("second");
    assert_eq!(
        first.recovery_status,
        SevenBlockerFamilyRecoveryStatus::SevenBlockerRecoveryReadyWithWarnings
    );
    assert_eq!(first.primary_next_family, "CandleExpansionOps");
    assert_eq!(first, second);
}
