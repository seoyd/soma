mod support;

use serde_json::to_value;
use support::sprint104_support::{read_fixture, run_sprint104};

#[test]
fn paper_batch_replay_plan_and_report_are_ready() {
    let bundle = run_sprint104(
        "soma_sprint104_dual_agent_paper_lifecycle.toml",
        "paper_batch_replay",
    );
    let actual = to_value(&bundle.paper_rotation_batch_replay_report).expect("actual");
    let expected: serde_json::Value =
        read_fixture("sprint104_data/paper_batch_replay_expected.json");
    assert_eq!(actual, expected);
    assert!(
        bundle
            .paper_rotation_batch_replay_plan
            .plan_status
            .starts_with("BatchReplayPlanReady")
    );
    assert_eq!(bundle.paper_rotation_batch_replay_report.no_trade_count, 2);
    assert_eq!(
        bundle
            .paper_rotation_batch_replay_report
            .need_more_evidence_count,
        2
    );
    assert_eq!(
        bundle
            .paper_rotation_batch_replay_report
            .broker_execution_allowed_count,
        0
    );
    assert_eq!(
        bundle
            .paper_rotation_batch_replay_report
            .live_execution_allowed_count,
        0
    );
}

#[test]
fn paper_batch_replay_report_is_deterministic() {
    let left = run_sprint104(
        "soma_sprint104_dual_agent_paper_lifecycle.toml",
        "paper_batch_replay_left",
    );
    let right = run_sprint104(
        "soma_sprint104_dual_agent_paper_lifecycle.toml",
        "paper_batch_replay_right",
    );
    assert_eq!(
        serde_json::to_string(&left.paper_rotation_batch_replay_report).expect("left"),
        serde_json::to_string(&right.paper_rotation_batch_replay_report).expect("right")
    );
}
