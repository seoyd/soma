mod support;

use soma_zero::{WorkspaceTimeoutRootCauseConfig, WorkspaceTimeoutRootCauseReport};
use support::sprint111_support::{read_fixture, run_sprint111};

#[test]
fn workspace_timeout_root_cause_config_defaults_and_remote_rejection_hold() {
    let config = WorkspaceTimeoutRootCauseConfig::default();
    assert!(config.require_sprint110_truth_import);
    assert!(config.require_cumulative_ledger);
    assert!(config.require_timeout_cleanup_verification);
    assert!(config.require_fifth_patch_decision_gate);
    assert!(!config.allow_fifth_patch_application);
    assert!(!config.run_real_no_run_observation);
    assert!(!config.run_real_full_observation);
    assert!(!config.run_cargo_json_progress_capture);

    let mut invalid = config.clone();
    invalid.output_root = "https://example.invalid/out".to_string();
    assert!(invalid.validate().is_err());
}

#[test]
fn workspace_timeout_root_cause_matches_expected_fixture() {
    let bundle = run_sprint111(
        "soma_sprint111_workspace_timeout_root_cause.toml",
        "workspace-timeout-root-cause",
    );
    let expected: WorkspaceTimeoutRootCauseReport =
        read_fixture("sprint111_data/workspace_timeout_root_cause_expected.json");
    assert_eq!(bundle.workspace_timeout_root_cause_report, expected);
    assert!(
        bundle
            .workspace_timeout_root_cause_report
            .suspected_root_causes
            .contains(&"IntegrationTestBinaryFanout".to_string())
    );
    assert_eq!(
        bundle.workspace_timeout_root_cause_report.root_cause_status,
        "TimeoutRootCausePartiallyIsolated"
    );
}
