#[path = "support/sprint69_support.rs"]
mod support;

use soma_zero::{CargoWorkspaceAuditStatus, RustToolchainModernizationRunner};

#[test]
fn cargo_workspace_audit_reports_resolver_editions_and_profiles() {
    let config = support::sprint76_config_from_example(
        "soma_cargo_workspace_audit.toml",
        "cargo-workspace-audit",
    );
    let report = RustToolchainModernizationRunner::default()
        .run_cargo_workspace_audit(&config)
        .expect("cargo workspace audit");
    assert_eq!(report.workspace_resolver, "3");
    assert!(report.edition_summary.contains("2024"));
    assert!(report.member_count >= 1);
    assert!(report.crate_count >= 1);
    assert!(report.binary_count >= 1);
    assert!(report.profile_dev_summary.contains("incremental=true"));
    assert!(report.profile_test_summary.contains("lto=false"));
    assert!(matches!(
        report.audit_status,
        CargoWorkspaceAuditStatus::WorkspaceAuditReady
            | CargoWorkspaceAuditStatus::HeavyDepsDetected
    ));
}

#[test]
fn cargo_workspace_audit_is_deterministic() {
    let config = support::sprint76_config_from_example(
        "soma_cargo_workspace_audit.toml",
        "cargo-workspace-audit-deterministic",
    );
    let runner = RustToolchainModernizationRunner::default();
    let first = runner
        .run_cargo_workspace_audit(&config)
        .expect("first audit");
    let second = runner
        .run_cargo_workspace_audit(&config)
        .expect("second audit");
    assert_eq!(first, second);
}
