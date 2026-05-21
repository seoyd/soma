mod common;
#[path = "support/sprint65_support.rs"]
mod sprint65_support;

use soma_zero::{
    ConservativeExternalLeaderboard, ExternalModelResearchOpsRunner,
    ExternalModelVersionComparisonReport, ExternalModelVersionComparisonStatus,
    LeaderboardChangeKind,
};

fn change_for<'a>(
    changelog: &'a soma_zero::ModelLeaderboardChangeLog,
    model_id: &str,
    model_version: &str,
) -> &'a soma_zero::ModelLeaderboardChange {
    changelog
        .changes
        .iter()
        .find(|change| change.model_id == model_id && change.model_version == model_version)
        .expect("leaderboard change")
}

#[test]
fn rank_up_no_change_and_newly_blocked_are_detected() {
    let config = sprint65_support::research_ops_config_from_example(
        "soma_model_leaderboard_changelog.toml",
        "leaderboard-changelog-base",
    );
    let changelog = ExternalModelResearchOpsRunner::default()
        .run_leaderboard_changelog(&config)
        .expect("run base changelog");
    assert_eq!(
        change_for(&changelog, "ext-model-a", "1.1.0").change_kind,
        LeaderboardChangeKind::RankUp
    );
    assert_eq!(
        change_for(&changelog, "ext-model-a", "1.0.0").change_kind,
        LeaderboardChangeKind::NoChange
    );
    assert_eq!(
        change_for(&changelog, "ext-model-b", "1.0.0").change_kind,
        LeaderboardChangeKind::NewlyBlocked
    );
}

#[test]
fn newly_eligible_is_detected() {
    let mut config = sprint65_support::research_ops_config_from_example(
        "soma_model_leaderboard_changelog.toml",
        "leaderboard-changelog-newly-eligible",
    );
    let mut comparison: ExternalModelVersionComparisonReport =
        sprint65_support::read_json(&config.model_version_comparison_paths[0]);
    comparison.previous_version = None;
    comparison.comparison_status = ExternalModelVersionComparisonStatus::Improved;
    config.model_version_comparison_paths[0] = sprint65_support::write_support_json(
        "leaderboard-changelog-newly-eligible",
        "comparison.json",
        &comparison,
    );
    let changelog = ExternalModelResearchOpsRunner::default()
        .run_leaderboard_changelog(&config)
        .expect("run newly eligible changelog");
    assert_eq!(
        change_for(&changelog, "ext-model-a", "1.1.0").change_kind,
        LeaderboardChangeKind::NewlyEligible
    );
}

#[test]
fn rank_down_and_score_changed_are_detected() {
    let mut rank_down = sprint65_support::research_ops_config_from_example(
        "soma_model_leaderboard_changelog.toml",
        "leaderboard-changelog-rank-down",
    );
    let mut leaderboard: ConservativeExternalLeaderboard =
        sprint65_support::read_json(&rank_down.conservative_leaderboard_paths[0]);
    for entry in &mut leaderboard.entries {
        if entry.model_id == "ext-model-a" && entry.model_version == "1.1.0" {
            entry.rank = Some(2);
        } else if entry.model_id == "ext-model-a" && entry.model_version == "1.0.0" {
            entry.rank = Some(1);
        }
    }
    rank_down.conservative_leaderboard_paths[0] = sprint65_support::write_support_json(
        "leaderboard-changelog-rank-down",
        "leaderboard.json",
        &leaderboard,
    );
    let rank_down_log = ExternalModelResearchOpsRunner::default()
        .run_leaderboard_changelog(&rank_down)
        .expect("run rank down changelog");
    assert_eq!(
        change_for(&rank_down_log, "ext-model-a", "1.1.0").change_kind,
        LeaderboardChangeKind::RankDown
    );

    let mut score_changed = sprint65_support::research_ops_config_from_example(
        "soma_model_leaderboard_changelog.toml",
        "leaderboard-changelog-score-changed",
    );
    let mut leaderboard: ConservativeExternalLeaderboard =
        sprint65_support::read_json(&score_changed.conservative_leaderboard_paths[0]);
    for entry in &mut leaderboard.entries {
        if entry.model_id == "ext-model-a" {
            entry.rank = Some(1);
        }
    }
    score_changed.conservative_leaderboard_paths[0] = sprint65_support::write_support_json(
        "leaderboard-changelog-score-changed",
        "leaderboard.json",
        &leaderboard,
    );
    let mut comparison: ExternalModelVersionComparisonReport =
        sprint65_support::read_json(&score_changed.model_version_comparison_paths[0]);
    comparison.comparison_status = ExternalModelVersionComparisonStatus::Improved;
    score_changed.model_version_comparison_paths[0] = sprint65_support::write_support_json(
        "leaderboard-changelog-score-changed",
        "comparison.json",
        &comparison,
    );
    let score_changed_log = ExternalModelResearchOpsRunner::default()
        .run_leaderboard_changelog(&score_changed)
        .expect("run score changed changelog");
    assert_eq!(
        change_for(&score_changed_log, "ext-model-a", "1.1.0").change_kind,
        LeaderboardChangeKind::ScoreChanged
    );
}

#[test]
fn leaderboard_changelog_is_deterministic() {
    let first = sprint65_support::research_ops_config_from_example(
        "soma_model_leaderboard_changelog.toml",
        "leaderboard-changelog-determinism-first",
    );
    let second = sprint65_support::research_ops_config_from_example(
        "soma_model_leaderboard_changelog.toml",
        "leaderboard-changelog-determinism-second",
    );
    let first_log = ExternalModelResearchOpsRunner::default()
        .run_leaderboard_changelog(&first)
        .expect("run first changelog");
    let second_log = ExternalModelResearchOpsRunner::default()
        .run_leaderboard_changelog(&second)
        .expect("run second changelog");
    assert_eq!(first_log, second_log);
}
