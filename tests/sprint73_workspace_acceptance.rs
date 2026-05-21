#[path = "support/sprint69_support.rs"]
mod support;

use soma_zero::{WorkspaceAcceptanceCheck, WorkspaceAcceptanceStatus};

fn passed_check(name: &str, command: &str) -> WorkspaceAcceptanceCheck {
    WorkspaceAcceptanceCheck {
        name: name.to_string(),
        command: command.to_string(),
        passed: true,
        output_summary: Some("ok".to_string()),
        reason_codes: Vec::new(),
    }
}

#[test]
fn workspace_acceptance_is_full_when_all_checks_pass() {
    let report = support::build_workspace_acceptance_report(
        "workspace-acceptance-full",
        vec![
            passed_check("cargo fmt --all", "cargo fmt --all"),
            passed_check("cargo check --workspace", "cargo check --workspace"),
            passed_check(
                "cargo test --workspace --quiet",
                "cargo test --workspace --quiet",
            ),
            passed_check("focused Sprint 73 test suite", "cargo test focused"),
            passed_check("Sprint 73 CLI smoke commands", "soma_experiment smoke"),
        ],
    );
    assert_eq!(
        report.acceptance_status,
        WorkspaceAcceptanceStatus::FullWorkspaceAccepted
    );
    assert!(report.full_workspace_test_passed);
}

#[test]
fn workspace_acceptance_is_focused_only_when_full_workspace_test_is_missing() {
    let report = support::build_workspace_acceptance_report(
        "workspace-acceptance-focused",
        vec![
            passed_check("cargo fmt --all", "cargo fmt --all"),
            passed_check("cargo check --workspace", "cargo check --workspace"),
            passed_check("focused Sprint 73 test suite", "cargo test focused"),
            passed_check("Sprint 73 CLI smoke commands", "soma_experiment smoke"),
        ],
    );
    assert_eq!(
        report.acceptance_status,
        WorkspaceAcceptanceStatus::FocusedOnly
    );
    assert!(!report.full_workspace_test_passed);
}
