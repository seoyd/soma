#[path = "support/sprint69_support.rs"]
mod support;

use std::fs;

#[test]
fn sprint73_outputs_are_deterministic() {
    let first = support::run_sprint73_bundle(
        "soma_ext_model_b_prediction_close.toml",
        "ext-model-b-prediction-closure-determinism-a",
    );
    let second = support::run_sprint73_bundle(
        "soma_ext_model_b_prediction_close.toml",
        "ext-model-b-prediction-closure-determinism-b",
    );

    assert_eq!(
        first.ext_model_b_prediction_closure_report,
        second.ext_model_b_prediction_closure_report
    );
    assert_eq!(
        first.prediction_coverage_finalization_report,
        second.prediction_coverage_finalization_report
    );
    assert_eq!(
        first.evidence_gap_final_closure_report,
        second.evidence_gap_final_closure_report
    );
    assert_eq!(
        first.direct_watch_readiness_final_gate,
        second.direct_watch_readiness_final_gate
    );
    assert_eq!(
        first.control_tower_briefing_final_refresh.refresh_id,
        second.control_tower_briefing_final_refresh.refresh_id
    );
    assert_eq!(
        first
            .control_tower_briefing_final_refresh
            .briefing_state_before,
        second
            .control_tower_briefing_final_refresh
            .briefing_state_before
    );
    assert_eq!(
        first
            .control_tower_briefing_final_refresh
            .briefing_state_after,
        second
            .control_tower_briefing_final_refresh
            .briefing_state_after
    );
    assert_eq!(
        first
            .control_tower_briefing_final_refresh
            .direct_watch_gate_status,
        second
            .control_tower_briefing_final_refresh
            .direct_watch_gate_status
    );
    assert_eq!(
        first
            .control_tower_briefing_final_refresh
            .evidence_gap_status,
        second
            .control_tower_briefing_final_refresh
            .evidence_gap_status
    );
    assert_eq!(
        first
            .control_tower_briefing_final_refresh
            .prediction_history_status,
        second
            .control_tower_briefing_final_refresh
            .prediction_history_status
    );
    assert_eq!(
        first
            .control_tower_briefing_final_refresh
            .retirement_evidence_status,
        second
            .control_tower_briefing_final_refresh
            .retirement_evidence_status
    );
    assert_eq!(
        first
            .control_tower_briefing_final_refresh
            .owner_checklist_status,
        second
            .control_tower_briefing_final_refresh
            .owner_checklist_status
    );
    assert_eq!(
        first.control_tower_briefing_final_refresh.refresh_status,
        second.control_tower_briefing_final_refresh.refresh_status
    );
    assert_eq!(
        first.control_tower_briefing_final_refresh.reason_codes,
        second.control_tower_briefing_final_refresh.reason_codes
    );

    let first_fragment = fs::read_to_string(
        support::sprint73_output_path("ext-model-b-prediction-closure-determinism-a")
            .join("sprint73-ext-model-b-prediction-gap-closure")
            .join("fragments")
            .join("direct_watch_final.html"),
    )
    .expect("first fragment");
    let second_fragment = fs::read_to_string(
        support::sprint73_output_path("ext-model-b-prediction-closure-determinism-b")
            .join("sprint73-ext-model-b-prediction-gap-closure")
            .join("fragments")
            .join("direct_watch_final.html"),
    )
    .expect("second fragment");
    assert_eq!(first_fragment, second_fragment);
}
