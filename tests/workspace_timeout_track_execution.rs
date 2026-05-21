mod support;

use std::fs;

use soma_zero::WorkspaceTimeoutTrackExecutionConfig;
use support::sprint116_support::run_sprint116;

#[test]
fn workspace_timeout_track_execution_config_and_bundle_are_sane() {
    let config = WorkspaceTimeoutTrackExecutionConfig::default();
    assert!(config.require_backlog_burndown);
    assert!(config.require_timeout_cleanup_actual_counts);
    assert!(config.require_cargo_json_actual_parsing);
    assert!(config.require_acceptance_truth_gate);
    assert!(config.require_consolidation_still_paused);
    assert!(!config.allow_fifth_patch_application);
    assert!(!config.allow_assertion_movement);
    assert!(!config.allow_test_target_retirement);
    let toml = config.to_toml_string().expect("toml");
    for forbidden in [
        "training_enabled",
        "broker",
        "order",
        "account",
        "runtime_field",
    ] {
        assert!(!toml.contains(forbidden), "unexpected field {forbidden}");
    }

    let invalid = WorkspaceTimeoutTrackExecutionConfig {
        output_root: "https://example.invalid/out".to_string(),
        ..config.clone()
    };
    assert!(invalid.validate().is_err());

    let bundle = run_sprint116(
        "soma_sprint116_workspace_timeout_track.toml",
        "workspace-timeout-track-execution",
    );
    assert_eq!(bundle.storage_report.file_count, 41);
    assert_eq!(
        bundle.acceptance_truth_gate_v17.status,
        "AcceptanceTruthReadyWithWarnings"
    );
    assert!(!bundle.acceptance_truth_gate_v17.can_claim_full_acceptance);
    assert_eq!(
        bundle.real_no_run_observation_attempt_v17.attempt_status,
        "NoRunNotRun"
    );
    let out = std::path::PathBuf::from(&bundle.storage_report.output_dir);
    assert!(out.join("summary.txt").exists());
    assert!(out.join("storage_report.txt").exists());
    let summary = fs::read_to_string(out.join("summary.txt")).expect("summary");
    assert!(summary.contains("## 1. Sprint summary"));
    assert!(summary.contains("## 59. Next gstack sprint recommendation"));
    assert!(summary.contains("file_count=41"));
    assert_eq!(fs::read_dir(out).expect("list output").count(), 41);
}
