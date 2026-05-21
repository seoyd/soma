mod support;

use std::fs;
use std::path::PathBuf;

use soma_zero::PaperDecisionReplayWarningClosureReport;
use support::sprint100_support::run_sprint100;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("sprint100_data")
        .join(name)
}

#[test]
fn paper_replay_warning_closure_matches_expected_fixture() {
    let bundle = run_sprint100(
        "soma_paper_replay_warning_closure.toml",
        "paper-replay-warning-closure",
    );
    let expected: PaperDecisionReplayWarningClosureReport = serde_json::from_str(
        &fs::read_to_string(fixture_path("paper_replay_warning_closure_expected.json"))
            .expect("read fixture"),
    )
    .expect("parse fixture");
    assert_eq!(
        bundle.paper_decision_replay_warning_closure_report,
        expected
    );
}
