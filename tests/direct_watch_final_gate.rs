#[path = "support/sprint69_support.rs"]
mod support;

use serde_json::Value;
use soma_zero::DirectWatchReadinessFinalStatus;

#[test]
fn direct_watch_final_gate_matches_expected_fixture_and_remains_monitoring_only() {
    let bundle = support::run_sprint73_bundle(
        "soma_direct_watch_final_gate.toml",
        "direct-watch-final-gate",
    );
    let report = bundle.direct_watch_readiness_final_gate;
    let expected: Value = support::read_json(support::example_path(
        "sprint73_data/expected_direct_watch_final_gate.json",
    ));

    assert_eq!(
        expected["previous_score_min"].as_u64().unwrap(),
        report.previous_score_min as u64
    );
    assert_eq!(
        expected["previous_score_max"].as_u64().unwrap(),
        report.previous_score_max as u64
    );
    assert_eq!(
        expected["current_score_min"].as_u64().unwrap(),
        report.current_score_min as u64
    );
    assert_eq!(
        expected["current_score_max"].as_u64().unwrap(),
        report.current_score_max as u64
    );
    assert!(report.evidence_gap_closed);
    assert!(report.owner_checklist_closed);
    assert!(report.prediction_history_sufficient);
    assert!(report.forbidden_controls_absent);
    assert!(report.static_only);
    assert!(report.paper_only);
    assert_eq!(
        report.gate_status,
        DirectWatchReadinessFinalStatus::DirectWatchReadyWithWarnings
    );
}
