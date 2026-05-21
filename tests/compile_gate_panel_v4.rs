mod support;

use soma_zero::{
    CompileOnlyAttemptV2Status, DevDependencyFanoutStatus, FullWorkspaceAcceptanceAttemptV5Status,
    NoRunAcceptanceGateV2Status, SafetyCoveragePreservationReportV3Status,
    Sprint87CompileGateRecoveryRunner, TestTargetFanoutStatus, WorkspaceCompileGraphAuditStatus,
    WorkspaceFeatureUnificationStatus,
};
use support::sprint69_support as sprint;

#[test]
fn compile_gate_panel_v4_shows_statuses_and_remaining_blockers() {
    let config = sprint::sprint87_config_from_example(
        "soma_control_tower_compile_gate_v4.toml",
        "compile-gate-panel-v4",
    );
    let report = Sprint87CompileGateRecoveryRunner::default()
        .run_control_tower_compile_gate_v4(&config)
        .expect("panel");
    assert_eq!(
        report.compile_graph_audit_status,
        WorkspaceCompileGraphAuditStatus::CompileGraphAuditReadyWithWarnings
    );
    assert_eq!(
        report.test_target_fanout_status,
        TestTargetFanoutStatus::FanoutReportReadyWithWarnings
    );
    assert_eq!(
        report.dev_dependency_fanout_status,
        DevDependencyFanoutStatus::HeavyFanoutDetected
    );
    assert_eq!(
        report.feature_unification_status,
        WorkspaceFeatureUnificationStatus::FeatureUnificationReadyWithWarnings
    );
    assert_eq!(
        report.compile_only_status,
        CompileOnlyAttemptV2Status::CompileOnlyStillBlocked
    );
    assert_eq!(
        report.no_run_gate_status,
        NoRunAcceptanceGateV2Status::NoRunGateStillBlocked
    );
    assert_eq!(
        report.full_workspace_status,
        FullWorkspaceAcceptanceAttemptV5Status::CompileOnlyBlocked
    );
    assert_eq!(
        report.safety_coverage_status,
        SafetyCoveragePreservationReportV3Status::SafetyCoveragePreserved
    );
    assert!(
        report
            .remaining_blockers
            .contains(&"CandleExpansionOps".to_string())
    );
    assert!(report.runtime_deferred_status.contains("runtime-deferred"));
}
