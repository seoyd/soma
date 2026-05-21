mod support;

use soma_zero::ControlTowerAiCommitteePanel;
use support::shared_fixture_harness::load_json_fixture;
use support::sprint69_support::example_path;
use support::sprint98_support::run_sprint98;

#[test]
fn control_tower_panel_stays_read_only_and_matches_fixture() {
    let bundle = run_sprint98(
        "soma_sprint98_committee_owned_core.toml",
        "control-tower-ai-committee",
    );
    let expected: ControlTowerAiCommitteePanel = load_json_fixture(example_path(
        "sprint98_data/control_tower_ai_committee_expected.json",
    ));
    assert_eq!(bundle.control_tower_ai_committee_panel, expected);
    let panel = &bundle.control_tower_ai_committee_panel;
    assert!(!panel.member_rows.is_empty());
    assert!(!panel.active_debate_sessions.is_empty());
    assert!(!panel.recent_entry_timing_proposals.is_empty());
    assert!(panel.chairman_rulebook_status.contains("Rulebook"));
    assert!(panel.promotion_demotion_summary.contains("promotions="));
    assert!(panel.risk_governor_summary.contains("cannot bypass"));
    assert!(panel.runtime_deferred_summary.contains("runtime-deferred"));
    let warning_blob = panel.warnings.join(" ").to_ascii_lowercase();
    for forbidden in ["train", "runtime", "live", "order", "account", "browser"] {
        assert!(
            warning_blob.contains(forbidden),
            "missing {forbidden} in warnings"
        );
    }
}
