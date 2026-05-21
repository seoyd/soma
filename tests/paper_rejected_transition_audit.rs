mod support;

use support::sprint105_support::run_sprint105;

#[test]
fn paper_rejected_transition_is_reachable_and_non_live() {
    let bundle = run_sprint105(
        "soma_paper_rejected_transition_audit.toml",
        "paper_rejected_transition_audit",
    );
    let report = &bundle.paper_rejected_transition_audit_report;
    assert!(report.paper_rejected_state_present);
    assert!(report.paper_rejected_reachable);
    assert!(!report.paper_rejected_can_go_live);
    assert!(!report.paper_rejected_can_become_order);
    assert!(report.archive_transition_present);
}
