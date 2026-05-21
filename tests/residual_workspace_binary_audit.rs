mod support;

use soma_zero::{ResidualWorkspaceBinaryAuditStatus, Sprint86ResidualGateRecoveryRunner};
use support::sprint69_support as sprint;

#[test]
fn residual_workspace_binary_audit_reports_observed_families() {
    let config = sprint::sprint86_config_from_example(
        "soma_residual_binary_audit.toml",
        "residual-workspace-binary-audit-test",
    );
    let report = Sprint86ResidualGateRecoveryRunner::default()
        .run_residual_binary_audit(&config)
        .expect("audit");
    assert_eq!(
        report.audit_status,
        ResidualWorkspaceBinaryAuditStatus::ResidualAuditReadyWithWarnings
    );
    assert_eq!(report.observed_binaries.len(), 7);
    assert!(
        report
            .residual_binaries
            .iter()
            .any(|name| name.ends_with("official_expansion_status_mapping.rs"))
    );
}
