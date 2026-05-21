mod support;

use serde_json::to_value;
use soma_zero::{WorkspaceAcceptanceRecoveryV7Config, WorkspaceAcceptanceRecoveryV7Runner};
use support::sprint106_support::{run_sprint106, sprint106_config_from_example};

#[test]
fn sprint106_config_defaults_are_safe_and_bundle_builds() {
    let config = WorkspaceAcceptanceRecoveryV7Config::default();
    assert!(!config.run_real_no_run);
    assert!(!config.run_real_full);
    assert!(config.capture_cargo_json);
    assert!(config.require_truth_gate);
    assert!(config.require_safety_preservation);
    assert!(config.require_no_assertion_deletion);
    assert!(config.require_no_hidden_skips);
    assert!(config.preserve_runtime_deferred);
    assert!(config.preserve_dual_agent_separation);
    assert!(config.validate().is_ok());

    let bundle = run_sprint106(
        "soma_sprint106_workspace_acceptance_recover.toml",
        "workspace_acceptance_recovery_v7",
    );
    assert!(
        bundle
            .workspace_compile_cost_profile_v3
            .profile_status
            .contains("CompileCostProfileReady")
    );
    let json = to_value(&bundle).expect("bundle json");
    assert!(json.get("storage_report").is_some());
}

#[test]
fn sprint106_rejects_remote_paths() {
    let mut config = WorkspaceAcceptanceRecoveryV7Config::default();
    config.output_root = "https://example.com/out".to_string();
    assert!(config.validate().is_err());
}

#[test]
fn sprint106_rejects_missing_explicit_json_inputs() {
    let mut config = sprint106_config_from_example(
        "soma_sprint106_workspace_acceptance_recover.toml",
        "sprint106-missing-explicit-json",
    );
    config.compile_cost_paths = Some(vec![
        "target/sprint106-tests/missing/compile_cost_profile.json".to_string(),
    ]);
    let err = WorkspaceAcceptanceRecoveryV7Runner::default()
        .run(&config)
        .expect_err("missing explicit JSON input should fail");
    assert!(err.contains("failed to read sprint106 JSON input"));
}
