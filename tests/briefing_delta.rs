#[path = "support/sprint69_support.rs"]
mod support;

use soma_zero::BriefingDeltaKind;

#[test]
fn briefing_delta_detects_status_warning_owner_and_command_changes() {
    let bundle = support::run_briefing("soma_briefing_delta.toml", "briefing-delta");
    let delta = bundle.briefing_delta_report.expect("delta report");

    assert!(
        delta
            .records
            .iter()
            .any(|item| item.delta_kind == BriefingDeltaKind::StatusChanged)
    );
    assert!(
        delta
            .records
            .iter()
            .any(|item| item.delta_kind == BriefingDeltaKind::WarningAdded)
    );
    assert!(
        delta
            .records
            .iter()
            .any(|item| item.delta_kind == BriefingDeltaKind::WarningRemoved)
    );
    assert!(
        delta
            .records
            .iter()
            .any(|item| item.delta_kind == BriefingDeltaKind::OwnerActionResolved)
    );
    assert!(
        delta
            .records
            .iter()
            .any(|item| item.delta_kind == BriefingDeltaKind::CommandChanged)
    );
}
