mod support;

use support::sprint102_support::run_sprint102;

#[test]
fn archetype_group_rotation_and_member_selection_are_deterministic() {
    let left = run_sprint102(
        "soma_archetype_group_rotation_plan.toml",
        "sprint102-rotation-left",
    );
    let right = run_sprint102(
        "soma_archetype_group_rotation_plan.toml",
        "sprint102-rotation-right",
    );
    assert_eq!(
        left.archetype_group_rotation_plan,
        right.archetype_group_rotation_plan
    );
    assert_eq!(
        left.archetype_member_selection_report,
        right.archetype_member_selection_report
    );
    assert!(
        !left
            .archetype_group_rotation_plan
            .short_term_swing_assignments
            .is_empty()
    );
    assert!(
        !left
            .archetype_group_rotation_plan
            .long_term_equity_assignments
            .is_empty()
    );
    assert!(
        !left
            .archetype_group_rotation_plan
            .crypto_assignments
            .is_empty()
    );
    assert!(
        !left
            .archetype_group_rotation_plan
            .common_risk_assignments
            .is_empty()
    );
    assert!(
        left.archetype_member_selection_report
            .diagnostic_members
            .iter()
            .all(|m| !left
                .archetype_member_selection_report
                .selected_members
                .contains(m))
    );
    assert!(
        !left
            .archetype_member_selection_report
            .watchlist_members
            .is_empty()
    );
}
