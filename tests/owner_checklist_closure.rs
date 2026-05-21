#[path = "support/sprint69_support.rs"]
mod support;

use soma_zero::OwnerChecklistClosureStatus;

#[test]
fn owner_checklist_closure_closes_items_instead_of_hiding_them() {
    let bundle = support::run_offline_attachment(
        "soma_offline_evidence_attach.toml",
        "owner-checklist-closure",
    );
    let report = bundle.owner_checklist_closure_report;

    assert_eq!(report.checklist_items_before, 4);
    assert_eq!(report.checklist_items_after, 0);
    assert_eq!(
        report.checklist_items_before,
        report.closed_items.len() + report.remaining_items.len()
    );
    assert_eq!(
        report.closure_status,
        OwnerChecklistClosureStatus::OwnerChecklistClosed
    );
}
