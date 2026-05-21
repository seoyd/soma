#[path = "support/sprint69_support.rs"]
mod support;

use soma_zero::{LeaderboardWarningClosureStatus, OperatorBriefingRunner};

#[test]
fn leaderboard_warning_closure_matches_fixture_and_explains_without_promotion() {
    let bundle = support::run_briefing(
        "soma_leaderboard_warning_closure.toml",
        "leaderboard-warning-closure",
    );
    let expected = support::read_json::<soma_zero::LeaderboardWarningClosureReport>(
        support::example_path("sprint71_data/leaderboard_warning_ext_model_a_1_2_0.json"),
    );
    assert_eq!(bundle.leaderboard_warning_closure_report, expected);
    assert_eq!(
        bundle.leaderboard_warning_closure_report.closure_status,
        LeaderboardWarningClosureStatus::WarningExplained
    );
    assert!(
        !bundle
            .leaderboard_warning_closure_report
            .closure_reason
            .to_ascii_lowercase()
            .contains("promote")
    );
}

#[test]
fn leaderboard_warning_closure_needs_more_evidence_when_sources_are_missing() {
    let mut config = support::briefing_config_from_example(
        "soma_leaderboard_warning_closure.toml",
        "leaderboard-warning-closure-missing",
    );
    config.model_ops_rollup_paths.clear();
    config.conservative_leaderboard_paths.clear();
    let report = OperatorBriefingRunner::default()
        .run_leaderboard_warning_closure(&config)
        .expect("leaderboard closure");
    assert_eq!(
        report.closure_status,
        LeaderboardWarningClosureStatus::NeedsMoreEvidence
    );
}
