mod support;

use std::fs;
use std::path::PathBuf;

use soma_zero::{PaperOnlyDecisionReplayReport, PaperOnlyDecisionReplayStatus};
use support::sprint99_support::run_sprint99;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("sprint99_data")
        .join(name)
}

#[test]
fn paper_only_decision_replay_matches_expected_fixture() {
    let bundle = run_sprint99(
        "soma_paper_only_decision_replay.toml",
        "paper-only-decision-replay",
    );
    let expected: PaperOnlyDecisionReplayReport = serde_json::from_str(
        &fs::read_to_string(fixture_path("paper_decision_replay_expected.json"))
            .expect("read fixture"),
    )
    .expect("parse fixture");
    assert_eq!(bundle.paper_only_decision_replay_report, expected);
    assert_eq!(
        bundle.paper_only_decision_replay_report.replay_status,
        PaperOnlyDecisionReplayStatus::ReplayReadyWithWarnings
    );
    assert_eq!(
        bundle
            .paper_only_decision_replay_report
            .broker_execution_allowed_count,
        0
    );
    assert_eq!(
        bundle
            .paper_only_decision_replay_report
            .live_execution_allowed_count,
        0
    );
}
