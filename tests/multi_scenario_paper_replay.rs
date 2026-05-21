mod support;

use serde_json::to_value;
use support::sprint103_support::{read_fixture, run_sprint103};

#[test]
fn multi_scenario_paper_replay_matches_expected_fixture() {
    let bundle = run_sprint103(
        "soma_multi_scenario_paper_replay.toml",
        "multi-scenario-paper-replay",
    );
    let actual = to_value(&bundle.multi_scenario_paper_replay_report).expect("actual");
    let expected: serde_json::Value =
        read_fixture("sprint103_data/multi_scenario_replay_expected.json");
    assert_eq!(actual, expected);
    assert_eq!(
        bundle.multi_scenario_paper_replay_pack.replay_count,
        bundle.scenario_outcome_expectation_matrix.scenario_count
    );
}
