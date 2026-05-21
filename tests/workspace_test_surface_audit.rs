mod support;

use soma_zero::{Sprint85WorkspaceGateRecoveryRunner, WorkspaceWideTestSurfaceAuditStatus};
use support::sprint69_support as sprint;

#[test]
fn workspace_test_surface_audit_reports_named_bottlenecks() {
    let config = sprint::sprint85_config_from_example(
        "soma_workspace_test_surface_audit.toml",
        "workspace-test-surface-audit-test",
    );
    let report = Sprint85WorkspaceGateRecoveryRunner::default()
        .run_workspace_test_surface_audit(&config)
        .expect("audit");
    assert_eq!(
        report.audit_status,
        WorkspaceWideTestSurfaceAuditStatus::TestSurfaceAuditReadyWithWarnings
    );
    assert_eq!(report.total_test_binaries_before, Some(16));
    assert_eq!(report.named_bottleneck_binaries.len(), 3);
    assert!(
        report
            .named_bottleneck_binaries
            .iter()
            .any(|name| name.ends_with("complete_row_closure_v2.rs"))
    );
}
