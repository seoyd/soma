#[path = "support/sprint69_support.rs"]
mod support;

use soma_zero::{BriefingSeverity, OwnerActionChecklistItemKind};

#[test]
fn owner_action_checklist_contains_expected_items_and_safety_constraints() {
    let bundle =
        support::run_briefing("soma_owner_action_checklist.toml", "owner-action-checklist");
    let kinds = bundle
        .owner_action_checklist
        .items
        .iter()
        .map(|item| item.item_kind)
        .collect::<Vec<_>>();
    assert!(kinds.contains(&OwnerActionChecklistItemKind::ReviewModelDisposition));
    assert!(kinds.contains(&OwnerActionChecklistItemKind::ReviewLeaderboardWarning));
    assert!(kinds.contains(&OwnerActionChecklistItemKind::ReviewRetirementEvidence));
    assert!(kinds.contains(&OwnerActionChecklistItemKind::ProvideMorePredictions));

    for item in &bundle.owner_action_checklist.items {
        assert!(item.forbidden_actions.iter().any(|value| value == "live"));
        assert!(
            item.forbidden_actions
                .iter()
                .any(|value| value == "runtime")
        );
        assert!(
            item.forbidden_actions
                .iter()
                .any(|value| value == "training")
        );
        assert!(item.forbidden_actions.iter().any(|value| value == "order"));
        assert!(
            item.forbidden_actions
                .iter()
                .any(|value| value == "account")
        );
        if let Some(command) = &item.command_suggestion {
            assert!(command.contains("cargo run --quiet --bin soma_experiment --"));
        }
    }

    assert!(
        bundle
            .owner_action_checklist
            .items
            .iter()
            .any(|item| item.severity == BriefingSeverity::Blocked)
    );
}
