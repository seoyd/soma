#[path = "support/sprint69_support.rs"]
mod support;

use serde_json::Value;
use soma_zero::DirectWatchReadinessStatusV2;

#[test]
fn direct_watch_readiness_score_matches_expected_fixture_and_stays_non_live() {
    let bundle = support::run_offline_attachment(
        "soma_offline_evidence_attach.toml",
        "direct-watch-readiness",
    );
    let score = bundle.direct_watch_readiness_score_v2;
    let expected: Value = support::read_json(support::example_path(
        "sprint72_data/expected_direct_watch_score.json",
    ));

    assert_eq!(
        expected["current_score_min"].as_u64().unwrap(),
        score.current_score_min as u64
    );
    assert_eq!(
        expected["current_score_max"].as_u64().unwrap(),
        score.current_score_max as u64
    );
    assert_eq!(
        expected["target_score_min"].as_u64().unwrap(),
        score.target_score_min as u64
    );
    assert_eq!(
        expected["target_score_max"].as_u64().unwrap(),
        score.target_score_max as u64
    );
    assert_eq!(
        expected["briefing_ready"].as_bool().unwrap(),
        score.briefing_ready
    );
    assert_eq!(
        expected["owner_checklist_ready"].as_bool().unwrap(),
        score.owner_checklist_ready
    );
    assert_eq!(
        expected["evidence_gap_reduced"].as_bool().unwrap(),
        score.evidence_gap_reduced
    );
    assert_eq!(
        expected["model_attention_clear"].as_bool().unwrap(),
        score.model_attention_clear
    );
    assert_eq!(
        expected["next_actions_clear"].as_bool().unwrap(),
        score.next_actions_clear
    );
    assert_eq!(
        expected["forbidden_controls_absent"].as_bool().unwrap(),
        score.forbidden_controls_absent
    );
    assert_eq!(
        score.readiness_status,
        DirectWatchReadinessStatusV2::NeedsEvidence
    );
    assert!(!score.briefing_ready);
    assert!(score.forbidden_controls_absent);
}
