#[path = "support/sprint69_support.rs"]
mod support;

use serde_json::Value;
use soma_zero::{
    DirectWatchPostEvidenceGateStatus, ModelPredictionsStaleClosureStatus,
    RealEvidencePredictionRefreshConfig, WorkspaceAcceptanceCheck, WorkspaceAcceptanceStatus,
    build_sprint75_workspace_acceptance_report,
};

#[test]
fn config_defaults_are_conservative() {
    let config = RealEvidencePredictionRefreshConfig::default();
    assert!(config.require_sequence_id_match);
    assert!(config.require_model_card);
    assert!(config.require_probability_sanity);
    assert!(config.require_no_duplicate_predictions);
    assert!(config.require_local_paths);
}

#[test]
fn sprint75_bundle_matches_expected_example() {
    let bundle = support::run_sprint75_bundle(
        "soma_real_prediction_requirements.toml",
        "real-evidence-prediction-refresh-example",
    );
    let expected_requirements: Value = support::read_json(support::example_path(
        "sprint75_data/expected_prediction_requirements.json",
    ));
    let expected_stale: Value = support::read_json(support::example_path(
        "sprint75_data/expected_stale_closure.json",
    ));
    let expected_gate: Value = support::read_json(support::example_path(
        "sprint75_data/expected_direct_watch_post_evidence_gate.json",
    ));
    assert_eq!(
        bundle.prediction_requirement_report.required_count,
        expected_requirements["required_count"]
            .as_u64()
            .expect("required count") as usize
    );
    assert_eq!(
        bundle.prediction_requirement_report.recommended_count,
        expected_requirements["recommended_count"]
            .as_u64()
            .expect("recommended count") as usize
    );
    assert_eq!(
        format!(
            "{:?}",
            bundle.prediction_requirement_report.requirement_status
        ),
        expected_requirements["status"].as_str().expect("status")
    );
    assert_eq!(
        format!(
            "{:?}",
            bundle.model_predictions_stale_closure_report.closure_status
        ),
        expected_stale["closure_status"]
            .as_str()
            .expect("closure status")
    );
    assert_eq!(
        bundle.model_predictions_stale_closure_report.closure_status,
        ModelPredictionsStaleClosureStatus::StaleClosed
    );
    assert_eq!(
        format!("{:?}", bundle.direct_watch_post_evidence_gate.gate_status),
        expected_gate["gate_status"].as_str().expect("gate status")
    );
    assert_eq!(
        bundle.direct_watch_post_evidence_gate.gate_status,
        DirectWatchPostEvidenceGateStatus::DirectWatchReadyWithWarnings
    );
    assert!(
        bundle
            .final_summary
            .contains("RealEvidencePredictionsRefreshed")
    );
}

#[test]
fn sprint75_workspace_acceptance_builder_marks_success() {
    let checks = vec![
        WorkspaceAcceptanceCheck {
            name: "cargo fmt --all".to_string(),
            command: "cargo fmt --all".to_string(),
            passed: true,
            output_summary: None,
            reason_codes: Vec::new(),
        },
        WorkspaceAcceptanceCheck {
            name: "cargo check --workspace".to_string(),
            command: "cargo check --workspace".to_string(),
            passed: true,
            output_summary: None,
            reason_codes: Vec::new(),
        },
        WorkspaceAcceptanceCheck {
            name: "cargo test --workspace --quiet".to_string(),
            command: "cargo test --workspace --quiet".to_string(),
            passed: true,
            output_summary: None,
            reason_codes: Vec::new(),
        },
        WorkspaceAcceptanceCheck {
            name: "focused Sprint 75 test suite".to_string(),
            command: "cargo test --quiet".to_string(),
            passed: true,
            output_summary: None,
            reason_codes: Vec::new(),
        },
        WorkspaceAcceptanceCheck {
            name: "Sprint 75 CLI smoke commands".to_string(),
            command: "cli smoke".to_string(),
            passed: true,
            output_summary: None,
            reason_codes: Vec::new(),
        },
    ];
    let report = build_sprint75_workspace_acceptance_report("sprint75-workspace", checks);
    assert_eq!(
        report.acceptance_status,
        WorkspaceAcceptanceStatus::FullWorkspaceAccepted
    );
    assert!(report.full_workspace_test_passed);
    assert!(report.cli_smoke_passed);
}
