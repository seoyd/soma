mod common;
#[path = "support/sprint68_support.rs"]
mod sprint68_support;

use std::fs;
use std::path::PathBuf;

use soma_zero::{
    DecisionConflictKind, DecisionConflictResolution, ModelOpsActionPriorityReport,
    RegressionCauseKind,
};

#[test]
fn conflict_regression_qa_action_and_panel_outputs_match_expected_fixtures() {
    let bundle = sprint68_support::run_trace("soma_model_ops_trace.toml", "trace-reports");

    let expected_conflicts: serde_json::Value = sprint68_support::read_json(
        sprint68_support::example_path("sprint68_data/expected_conflict_trace.json"),
    );
    assert_eq!(
        expected_conflicts["conflict_count"].as_u64(),
        Some(bundle.decision_conflict_trace_report.conflict_count as u64)
    );
    let conflict = bundle
        .decision_conflict_trace_report
        .conflicts
        .first()
        .expect("conflict trace");
    assert_eq!(conflict.conflict_kind, DecisionConflictKind::OwnerVsPolicy);
    assert_eq!(
        conflict.recommended_resolution,
        DecisionConflictResolution::RequestOwnerReview
    );

    let expected_regression: serde_json::Value = sprint68_support::read_json(
        sprint68_support::example_path("sprint68_data/expected_regression_trace.json"),
    );
    assert_eq!(
        expected_regression["coverage_regression_count"].as_u64(),
        Some(
            bundle
                .regression_evidence_trace_report
                .coverage_regression_count as u64
        )
    );
    let coverage = bundle
        .regression_evidence_trace_report
        .traces
        .iter()
        .find(|item| item.cause_kind == RegressionCauseKind::CoverageRegression)
        .expect("coverage trace");
    assert_eq!(coverage.baseline_value.as_deref(), Some("1.0000"));
    assert_eq!(coverage.current_value.as_deref(), Some("0.6667"));
    assert_eq!(coverage.delta.as_deref(), Some("1.0000 -> 0.6667"));

    assert_eq!(bundle.operator_qa_evidence_trace_report.ready_count, 2);
    assert!(
        bundle
            .operator_qa_evidence_trace_report
            .traces
            .iter()
            .all(|item| item.source_artifact.is_some())
    );

    let ext_model_b_action = bundle
        .action_priority_trace_report
        .traces
        .iter()
        .find(|item| item.model_id == "ext-model-b" && item.model_version == "1.0.0")
        .expect("action trace");
    assert!(ext_model_b_action.safe_to_run);
    assert!(!ext_model_b_action.why_this_action.is_empty());
    assert!(!ext_model_b_action.why_not_stronger_action.is_empty());
    let command = ext_model_b_action
        .command_suggestion
        .as_deref()
        .expect("command suggestion");
    for forbidden in ["train", "live", "runtime", "order", "account", "broker"] {
        assert!(
            !command.contains(forbidden),
            "unsafe command text: {forbidden}"
        );
    }

    assert_eq!(bundle.control_tower_model_trace_panel.cards_with_trace, 4);
    assert_eq!(
        bundle.control_tower_model_trace_panel.cards_missing_trace,
        0
    );
    assert!(
        bundle
            .control_tower_model_trace_panel
            .conflict_summary
            .contains("ConflictsDetected")
    );
    assert!(
        bundle
            .control_tower_model_trace_panel
            .regression_summary
            .contains("RegressionTraceReady")
    );
    assert_eq!(
        bundle.control_tower_model_trace_panel.mamba_deferred_status,
        "HoldMamba3RuntimeDeferred"
    );
    assert_eq!(
        bundle
            .control_tower_model_trace_panel
            .per_model_trace_links
            .len(),
        4
    );

    let expected_trace: serde_json::Value = sprint68_support::read_json(
        sprint68_support::example_path("sprint68_data/expected_trace_summary.json"),
    );
    assert_eq!(
        expected_trace["control_tower_model_trace_panel"]["cards_with_trace"].as_u64(),
        Some(bundle.control_tower_model_trace_panel.cards_with_trace as u64)
    );
    assert_eq!(
        expected_trace["decision_conflict_trace_report"]["conflict_count"].as_u64(),
        Some(bundle.decision_conflict_trace_report.conflict_count as u64)
    );
}

#[test]
fn static_fragments_are_generated_and_safe() {
    let bundle = sprint68_support::run_trace("soma_model_ops_trace.toml", "trace-fragments");
    let links = bundle
        .control_tower_model_trace_panel
        .per_model_trace_links
        .get("ext-model-b:1.0.0")
        .expect("fragment links");
    let trace_fragment = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("sprint68-tests")
        .join("trace-fragments")
        .join("sprint68-model-ops-trace")
        .join(links.get("trace_summary_path").expect("trace path"));
    let html = fs::read_to_string(trace_fragment).expect("read trace fragment");
    for forbidden in [
        "<script",
        "<form",
        "POST",
        "http://",
        "https://",
        "KIS_APP_KEY",
        "KIS_APP_SECRET",
    ] {
        assert!(
            !html.contains(forbidden),
            "unexpected fragment content: {forbidden}"
        );
    }
    assert!(html.contains("read-only"));
    assert!(html.contains("static trace fragment"));
}

#[test]
fn action_trace_fixture_stays_deterministic() {
    let bundle = sprint68_support::run_trace("soma_model_action_trace.toml", "trace-action-only");
    let expected: ModelOpsActionPriorityReport = sprint68_support::read_json(
        sprint68_support::example_path("sprint68_data/model_ops_action_priority.json"),
    );
    assert_eq!(bundle.action_priority_trace_report.required_count, 2);
    assert_eq!(
        expected
            .primary_next_action
            .as_ref()
            .map(|item| item.model_id.as_str()),
        Some("ext-model-b")
    );
}
