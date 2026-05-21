mod support;

use soma_zero::{Sprint87CompileGateRecoveryRunner, WorkspaceCompileGraphAuditStatus};
use support::sprint69_support as sprint;

#[test]
fn workspace_compile_graph_audit_config_defaults_are_conservative() {
    let config = sprint::sprint87_config_from_example(
        "soma_workspace_compile_graph_audit.toml",
        "workspace-compile-graph-audit-config",
    );
    assert!(config.include_future_window_family);
    assert!(config.include_official_diversity_family);
    assert!(config.include_trinity_operational_family);
    assert!(config.include_dataset_export_family);
    assert!(config.include_control_tower_v1_family);
    assert!(config.include_candle_expansion_family);
    assert!(config.include_external_prediction_family);
    assert!(config.include_krx_evidence_family);
    assert!(config.include_dashboard_renderer_family);
    assert!(config.include_baseline_signal_family);
    assert!(config.include_counterfactual_backfill_family);
    assert!(config.preserve_assertions);
    assert!(config.preserve_safety_guards);
}

#[test]
fn workspace_compile_graph_audit_rejects_remote_paths() {
    let mut config = sprint::sprint87_config_from_example(
        "soma_workspace_compile_graph_audit.toml",
        "workspace-compile-graph-audit-remote",
    );
    config.cargo_metadata_paths = vec!["https://example.com/meta.json".to_string()];
    assert!(config.validate().is_err());
}

#[test]
fn workspace_compile_graph_audit_is_deterministic() {
    let config = sprint::sprint87_config_from_example(
        "soma_workspace_compile_graph_audit.toml",
        "workspace-compile-graph-audit-run",
    );
    let first = Sprint87CompileGateRecoveryRunner::default()
        .run_workspace_compile_graph_audit(&config)
        .expect("first");
    let second = Sprint87CompileGateRecoveryRunner::default()
        .run_workspace_compile_graph_audit(&config)
        .expect("second");
    assert_eq!(
        first.audit_status,
        WorkspaceCompileGraphAuditStatus::CompileGraphAuditReadyWithWarnings
    );
    assert_eq!(first, second);
}
