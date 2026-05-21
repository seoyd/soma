mod support;

use support::sprint106_support::run_sprint106;

#[test]
fn full_workspace_attempt_defaults_to_not_run_and_acceptance_stays_false() {
    let bundle = run_sprint106(
        "soma_real_full_workspace_attempt_v22.toml",
        "real_full_workspace_attempt_v22",
    );
    let report = bundle.real_full_workspace_attempt_v22;
    assert_eq!(report.full_status, "NotRun");
    assert!(!report.finished);
    assert_ne!(report.passed, Some(true));
    assert!(
        !bundle
            .workspace_full_acceptance_gate_v7
            .full_workspace_accepted
    );
}
