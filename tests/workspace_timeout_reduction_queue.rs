mod support;

use std::process::Command;

use serde_json::to_value;
use soma_zero::{
    AcceptanceTruthGateV19, CargoJsonFailureReasonAnalysisReportV1,
    CargoJsonReasonLineClassificationReportV1, CargoJsonTargetBlockerExtractionReportV1,
    CommandObservation, ControlTowerTimeoutReductionQueuePanel, Sprint117BaselineTruthImportReport,
    TruthfulFullWorkspaceAttemptV19, TruthfulNoRunAttemptV19, WorkspaceAcceptanceRecoveryV7Config,
    WorkspaceTimeoutEvidenceMatrixV4, WorkspaceTimeoutReductionHypothesisReportV1,
    WorkspaceTimeoutReductionQueueConfig, WorkspaceTimeoutReductionQueueV1,
    build_acceptance_truth_gate_v19, build_truthful_full_workspace_attempt_v19,
    build_truthful_no_run_attempt_v19, build_workspace_full_acceptance_gate_v19,
};
use support::sprint118_support::{read_fixture, run_sprint118};

#[test]
fn workspace_timeout_reduction_queue_matches_expected_and_config_is_safe() {
    let bundle = run_sprint118(
        "soma_workspace_timeout_reduction_queue_v1.toml",
        "workspace-timeout-reduction-queue",
    );
    let expected: WorkspaceTimeoutReductionQueueV1 =
        read_fixture("sprint118_data/timeout_reduction_queue_expected.json");
    assert_eq!(bundle.workspace_timeout_reduction_queue_v1, expected);
    assert!(bundle.final_summary.contains("## 1. Sprint summary"));
    assert!(
        bundle
            .final_summary
            .contains("## 44. Acceptance truth gate v19")
    );
    assert!(
        bundle
            .final_summary
            .contains("## 67. Next gstack sprint recommendation")
    );
    assert_eq!(bundle.final_summary.matches("\n## ").count() + 1, 67);
    let config = WorkspaceTimeoutReductionQueueConfig::default();
    assert!(config.validate().is_ok());
    assert!(config.require_cargo_json_reason_analysis);
    assert!(config.require_timeout_reduction_queue);
    assert!(config.require_acceptance_truth_gate);
    assert!(!config.allow_fifth_patch_application);
    assert!(!config.allow_assertion_movement);
    assert!(!config.allow_test_target_retirement);
    let acceptance_recovery_config = WorkspaceAcceptanceRecoveryV7Config::default();
    assert!(acceptance_recovery_config.require_no_hidden_skips);
    assert!(acceptance_recovery_config.require_no_assertion_deletion);
    let remote = WorkspaceTimeoutReductionQueueConfig {
        sprint117_truth_paths: Some(vec!["https://example.invalid/config.json".to_string()]),
        ..config.clone()
    };
    assert!(remote.validate().is_err());
    let json = to_value(config).expect("serialize config");
    let text = json.to_string();
    for forbidden in [
        "runtime_enabled",
        "training_enabled",
        "broker",
        "order",
        "account",
    ] {
        assert!(
            !text.contains(forbidden),
            "forbidden field leaked: {forbidden}"
        );
    }
}

#[test]
fn sprint117_baseline_truth_import_matches_expected() {
    let bundle = run_sprint118(
        "soma_sprint117_baseline_truth_import.toml",
        "sprint117-baseline-truth-import",
    );
    let expected: Sprint117BaselineTruthImportReport =
        read_fixture("sprint118_data/baseline_truth_import_expected.json");
    assert_eq!(bundle.sprint117_baseline_truth_import_report, expected);
    assert!(
        bundle
            .sprint117_baseline_truth_import_report
            .focused_tests_passed
    );
    assert!(
        bundle
            .sprint117_baseline_truth_import_report
            .cli_smoke_passed
    );
    assert!(
        bundle
            .sprint117_baseline_truth_import_report
            .cargo_check_passed
    );
    assert!(
        bundle
            .sprint117_baseline_truth_import_report
            .cargo_build_passed
    );
    assert!(
        !bundle
            .sprint117_baseline_truth_import_report
            .imported_as_full_acceptance
    );
}

#[test]
fn cargo_json_failure_reason_analysis_matches_expected() {
    let bundle = run_sprint118(
        "soma_cargo_json_failure_reason_analysis_v1.toml",
        "cargo-json-failure-reason-analysis-v1",
    );
    let expected: CargoJsonFailureReasonAnalysisReportV1 =
        read_fixture("sprint118_data/cargo_json_failure_reason_analysis_expected.json");
    assert_eq!(
        bundle.cargo_json_failure_reason_analysis_report_v1,
        expected
    );
    let expected_lines: CargoJsonReasonLineClassificationReportV1 =
        read_fixture("sprint118_data/cargo_json_reason_line_classification_expected.json");
    assert_eq!(
        bundle.cargo_json_reason_line_classification_report_v1,
        expected_lines
    );
}

#[test]
fn cargo_json_target_blocker_extraction_is_deterministic() {
    let bundle = run_sprint118(
        "soma_cargo_json_target_blocker_extraction_v1.toml",
        "cargo-json-target-blocker-extraction-v1",
    );
    let report: CargoJsonTargetBlockerExtractionReportV1 =
        bundle.cargo_json_target_blocker_extraction_report_v1;
    assert!(
        report
            .target_blockers
            .iter()
            .any(|value| value.contains("workspace_cli_integration"))
    );
    assert!(
        report
            .suspect_targets
            .iter()
            .any(|value| value.contains("macro_link_heavy_suite"))
    );
    assert!(
        report
            .artifact_blockers
            .iter()
            .any(|value| value.contains("workspace_timeout_guard"))
    );
}

#[test]
fn workspace_timeout_reduction_hypothesis_matches_expected() {
    let bundle = run_sprint118(
        "soma_workspace_timeout_reduction_hypothesis_v1.toml",
        "workspace-timeout-reduction-hypothesis-v1",
    );
    let expected: WorkspaceTimeoutReductionHypothesisReportV1 =
        read_fixture("sprint118_data/timeout_reduction_hypothesis_expected.json");
    assert_eq!(
        bundle.workspace_timeout_reduction_hypothesis_report_v1,
        expected
    );
    assert!(
        bundle
            .workspace_timeout_reduction_hypothesis_report_v1
            .hypotheses
            .iter()
            .any(|value| value == "IntegrationTestBinaryFanout")
    );
}

#[test]
fn truthful_no_run_attempt_handles_not_run_timeout_and_pass() {
    let bundle = run_sprint118(
        "soma_truthful_no_run_attempt_v19.toml",
        "truthful-no-run-attempt-v19",
    );
    let expected: TruthfulNoRunAttemptV19 =
        read_fixture("sprint118_data/truthful_no_run_attempt_expected.json");
    assert_eq!(bundle.truthful_no_run_attempt_v19, expected);
    let timeout = build_truthful_no_run_attempt_v19(
        Some(&CommandObservation {
            attempted: true,
            finished: false,
            passed: None,
            duration_ms: Some(1),
            timeout_ms: Some(420000),
            exit_code: Some(124),
            timed_out: true,
            stdout: String::new(),
        }),
        Some(420000),
    );
    assert!(!timeout.recovered);
    let success = build_truthful_no_run_attempt_v19(
        Some(&CommandObservation {
            attempted: true,
            finished: true,
            passed: Some(true),
            duration_ms: Some(1),
            timeout_ms: Some(420000),
            exit_code: Some(0),
            timed_out: false,
            stdout: String::new(),
        }),
        Some(420000),
    );
    assert!(success.recovered);
}

#[test]
fn truthful_full_workspace_attempt_requires_finished_pass() {
    let bundle = run_sprint118(
        "soma_truthful_full_workspace_attempt_v19.toml",
        "truthful-full-workspace-attempt-v19",
    );
    let expected: TruthfulFullWorkspaceAttemptV19 =
        read_fixture("sprint118_data/truthful_full_workspace_attempt_expected.json");
    assert_eq!(bundle.truthful_full_workspace_attempt_v19, expected);
    let timeout = build_truthful_full_workspace_attempt_v19(
        Some(&CommandObservation {
            attempted: true,
            finished: false,
            passed: None,
            duration_ms: Some(1),
            timeout_ms: Some(420000),
            exit_code: Some(124),
            timed_out: true,
            stdout: String::new(),
        }),
        Some(420000),
    );
    assert!(!timeout.full_workspace_accepted);
    let success = build_truthful_full_workspace_attempt_v19(
        Some(&CommandObservation {
            attempted: true,
            finished: true,
            passed: Some(true),
            duration_ms: Some(1),
            timeout_ms: Some(420000),
            exit_code: Some(0),
            timed_out: false,
            stdout: String::new(),
        }),
        Some(420000),
    );
    assert!(success.full_workspace_accepted);
}

#[test]
fn workspace_timeout_evidence_matrix_matches_expected() {
    let bundle = run_sprint118(
        "soma_workspace_timeout_evidence_matrix_v4.toml",
        "workspace-timeout-evidence-matrix-v4",
    );
    let expected: WorkspaceTimeoutEvidenceMatrixV4 =
        read_fixture("sprint118_data/workspace_timeout_evidence_matrix_v4_expected.json");
    assert_eq!(bundle.workspace_timeout_evidence_matrix_v4, expected);
    assert!(
        !bundle
            .workspace_timeout_evidence_matrix_v4
            .supports_acceptance
    );
}

#[test]
fn acceptance_truth_gate_v19_requires_full_finished_and_passed() {
    let bundle = run_sprint118(
        "soma_acceptance_truth_gate_v19.toml",
        "acceptance-truth-gate-v19",
    );
    let expected: AcceptanceTruthGateV19 =
        read_fixture("sprint118_data/acceptance_truth_gate_v19_expected.json");
    assert_eq!(bundle.acceptance_truth_gate_v19, expected);
    let full = build_truthful_full_workspace_attempt_v19(
        Some(&CommandObservation {
            attempted: true,
            finished: true,
            passed: Some(true),
            duration_ms: Some(1),
            timeout_ms: Some(420000),
            exit_code: Some(0),
            timed_out: false,
            stdout: String::new(),
        }),
        Some(420000),
    );
    let gate = build_workspace_full_acceptance_gate_v19(&full);
    assert!(build_acceptance_truth_gate_v19(&gate).can_claim_full_acceptance);
}

#[test]
fn control_tower_timeout_reduction_queue_panel_is_read_only() {
    let bundle = run_sprint118(
        "soma_control_tower_timeout_reduction_queue.toml",
        "control-tower-timeout-reduction-queue",
    );
    let expected: ControlTowerTimeoutReductionQueuePanel =
        read_fixture("sprint118_data/control_tower_timeout_reduction_queue_expected.json");
    assert_eq!(bundle.control_tower_timeout_reduction_queue_panel, expected);
    let panel = bundle.control_tower_timeout_reduction_queue_panel;
    assert!(panel.static_read_only);
    assert!(panel.no_run_button);
    assert!(panel.no_apply_button);
    assert!(panel.no_train_runtime_live_order_account_controls);
}

#[test]
fn sprint118_cli_help_has_required_warnings_and_no_forbidden_commands() {
    let binary = env!("CARGO_BIN_EXE_soma_experiment");
    let help = Command::new(binary)
        .arg("--help")
        .output()
        .expect("run help");
    assert!(help.status.success());
    let help_text = String::from_utf8_lossy(&help.stdout);
    for required in [
        "sprint118-timeout-reduction-queue",
        "sprint117-baseline-truth-import",
        "acceptance-truth-gate-v19",
        "control-tower-acceptance-truth-v19",
    ] {
        assert!(help_text.contains(required));
    }
    for forbidden in [
        "
  train-model",
        "
  live-inference",
        "
  mamba-runtime",
        "
  gated-runtime",
        "
  broker",
        "
  order",
        "
  account",
    ] {
        assert!(
            !help_text.contains(forbidden),
            "unexpected command: {forbidden}"
        );
    }
    let sprint_help = Command::new(binary)
        .args(["sprint118-timeout-reduction-queue", "--help"])
        .output()
        .expect("run sprint help");
    assert!(String::from_utf8_lossy(&sprint_help.stdout).contains("timeout-reduction-only"));
    let cargo_json_help = Command::new(binary)
        .args(["cargo-json-failure-reason-analysis-v1", "--help"])
        .output()
        .expect("run cargo json help");
    assert!(
        String::from_utf8_lossy(&cargo_json_help.stdout).contains("cargo JSON is supporting-only")
    );
    let full_help = Command::new(binary)
        .args(["truthful-full-workspace-attempt-v19", "--help"])
        .output()
        .expect("run full help");
    assert!(
        String::from_utf8_lossy(&full_help.stdout)
            .contains("only a finished and passed full run may claim full acceptance")
    );
    let no_run_help = Command::new(binary)
        .args(["workspace-no-run-recovery-gate-v19", "--help"])
        .output()
        .expect("run no-run help");
    assert!(String::from_utf8_lossy(&no_run_help.stdout).contains("no-run-is-not-full"));
    let remote = Command::new(binary)
        .args([
            "sprint118-timeout-reduction-queue",
            "--config",
            "https://example.invalid/config.toml",
        ])
        .output()
        .expect("run remote config rejection");
    assert!(!remote.status.success());
    let stderr = String::from_utf8_lossy(&remote.stderr);
    assert!(
        stderr.contains("config path must be local")
            || stderr.contains("must use local-only paths")
    );
}

#[test]
fn sprint118_bundle_is_deterministic() {
    let mut first = run_sprint118(
        "soma_sprint118_timeout_reduction_queue.toml",
        "sprint118-determinism-a",
    );
    let mut second = run_sprint118(
        "soma_sprint118_timeout_reduction_queue.toml",
        "sprint118-determinism-b",
    );
    assert_eq!(first.storage_report.file_count, 51);
    first.storage_report.output_dir = "<normalized>".to_string();
    second.storage_report.output_dir = "<normalized>".to_string();
    let mut first_value = serde_json::to_value(first).expect("first json");
    let mut second_value = serde_json::to_value(second).expect("second json");
    first_value["storage_report"]["output_dir"] =
        serde_json::Value::String("<normalized>".to_string());
    second_value["storage_report"]["output_dir"] =
        serde_json::Value::String("<normalized>".to_string());
    assert_eq!(first_value, second_value);
    let summary = first_value["final_summary"].as_str().expect("summary");
    assert!(summary.contains("## 1. Sprint summary"));
    assert!(summary.contains("## 44. Acceptance truth gate v19"));
    assert!(summary.contains("## 67. Next gstack sprint recommendation"));
    assert_eq!(summary.matches("\n## ").count() + 1, 67);
}
