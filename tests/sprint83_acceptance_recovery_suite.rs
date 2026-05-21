mod support;

use serde_json::Value;
use soma_zero::{
    CommitteeReferenceFixturePackV2Status, CompilationBottleneckKind,
    DefensiveCounterfactualFixturePackV2Status, EvidenceDepthDeterminismRegressionStatus,
    EvidenceDepthFixtureAuditStatus, EvidenceDepthFixtureCompletenessStatus,
    EvidenceDepthFixtureNormalizationStatus, FullWorkspaceAcceptanceRecoveryStatus,
    LongRunningCompilationDiagnosisStatus, NoLookaheadSourceBoundaryFixtureAuditV2Status,
    OfficialEvidenceDepthFixturePackV2Status, Sprint82CliSmokeCompressionStatus,
    Sprint83AcceptanceRecoveryBundle, Sprint83AcceptanceRecoveryRunner, TestRuntimeRecoveryAction,
    TestRuntimeRecoveryPlanStatus, WorkspaceAcceptanceRecoveryGateStatus,
};
use support::{shared_fixture_harness as harness, sprint69_support as sprint};

#[test]
fn sprint83_bundle_is_constructed_and_conservative() {
    let bundle = sprint::run_sprint83_bundle(
        "soma_sprint83_acceptance_recovery.toml",
        "sprint83-acceptance-recovery-suite",
    );
    let expected: Value = harness::load_json_fixture(sprint::example_path(
        "sprint83_data/recovery_gate_expected.json",
    ));

    assert_eq!(
        bundle
            .full_workspace_acceptance_recovery_report
            .acceptance_status,
        FullWorkspaceAcceptanceRecoveryStatus::FullWorkspaceBlockedByCompilation
    );
    assert_eq!(
        bundle.workspace_acceptance_recovery_gate.gate_status,
        WorkspaceAcceptanceRecoveryGateStatus::BlockedByFullWorkspace
    );
    assert_eq!(
        expected["gate_status"].as_str(),
        Some("BlockedByFullWorkspace")
    );
    harness::assert_no_secret_like_values(&bundle.final_summary);
    harness::assert_no_order_account_fields(&bundle.final_summary);
    harness::assert_no_runtime_fields(&bundle.final_summary);
}

#[test]
fn sprint83_config_defaults_and_remote_guard_hold() {
    let config = sprint::sprint83_recovery_config_from_example(
        "soma_sprint83_acceptance_recovery.toml",
        "sprint83-recovery-config-suite",
    );
    assert!(config.require_full_workspace_test);
    assert!(config.require_focused_tests);
    assert!(config.require_cli_smoke);
    assert!(config.require_fixture_determinism);
    assert!(config.require_source_boundary_audit);
    assert!(config.require_no_lookahead_audit);

    let mut remote = config.clone();
    remote.sprint82_report_paths = vec!["https://example.com/report.json".to_string()];
    let error = remote.validate().expect_err("remote paths rejected");
    assert!(error.contains("must be local"));
}

#[test]
fn sprint83_focused_only_state_is_not_full_acceptance() {
    let mut summary: Value = harness::load_json_fixture(sprint::example_path(
        "sprint83_data/sprint82_report_summary.json",
    ));
    summary["full_workspace_test_started"] = Value::Bool(false);
    summary["full_workspace_test_finished"] = Value::Bool(false);
    summary["full_workspace_test_passed"] = Value::Null;
    let summary_path =
        sprint::write_support_json("sprint83-focused-only", "summary.json", &summary);

    let mut config = sprint::sprint83_recovery_config_from_example(
        "soma_sprint83_acceptance_recovery.toml",
        "sprint83-focused-only-suite",
    );
    config.sprint82_report_paths = vec![summary_path];

    let report = Sprint83AcceptanceRecoveryRunner::default()
        .run_full_workspace_acceptance_recovery(&config)
        .expect("focused-only report");
    assert_eq!(
        report.acceptance_status,
        FullWorkspaceAcceptanceRecoveryStatus::FocusedOnly
    );
}

#[test]
fn sprint83_full_workspace_report_keeps_blocked_state_honest() {
    let bundle = sprint::run_sprint83_bundle(
        "soma_full_workspace_acceptance_recovery.toml",
        "sprint83-full-workspace-report-suite",
    );
    assert!(bundle.full_workspace_acceptance_recovery_report.fmt_passed);
    assert!(
        bundle
            .full_workspace_acceptance_recovery_report
            .check_passed
    );
    assert!(
        bundle
            .full_workspace_acceptance_recovery_report
            .focused_tests_passed
    );
    assert!(
        bundle
            .full_workspace_acceptance_recovery_report
            .cli_smoke_passed
    );
    assert!(
        bundle
            .full_workspace_acceptance_recovery_report
            .full_workspace_test_started
    );
    assert!(
        !bundle
            .full_workspace_acceptance_recovery_report
            .full_workspace_test_finished
    );
    assert_eq!(
        bundle
            .full_workspace_acceptance_recovery_report
            .acceptance_status,
        FullWorkspaceAcceptanceRecoveryStatus::FullWorkspaceBlockedByCompilation
    );
}

#[test]
fn sprint83_long_compilation_diagnosis_is_reported() {
    let bundle = sprint::run_sprint83_bundle(
        "soma_long_compilation_diagnosis.toml",
        "sprint83-long-compilation-diagnosis-suite",
    );

    assert_eq!(
        bundle
            .long_running_compilation_diagnosis_report
            .diagnosis_status,
        LongRunningCompilationDiagnosisStatus::DiagnosisReady
    );
    assert_eq!(
        bundle
            .long_running_compilation_diagnosis_report
            .bottleneck_kind,
        CompilationBottleneckKind::TestBinaryExplosion
    );
    assert!(
        !bundle
            .long_running_compilation_diagnosis_report
            .suspected_test_binaries
            .is_empty()
    );
}

#[test]
fn sprint83_fixture_hardening_reports_and_packs_are_ready() {
    let bundle = sprint::run_sprint83_bundle(
        "soma_evidence_depth_fixture_audit.toml",
        "sprint83-fixture-hardening-suite",
    );
    let expected: Value = harness::load_json_fixture(sprint::example_path(
        "sprint83_data/evidence_depth_fixture_audit_expected.json",
    ));
    let manifest: Value = harness::load_json_fixture(sprint::example_path(
        "sprint83_data/sprint82_fixture_manifest.json",
    ));

    assert_eq!(
        bundle.evidence_depth_fixture_audit_report.audit_status,
        EvidenceDepthFixtureAuditStatus::FixturesReadyWithWarnings
    );
    assert_eq!(
        bundle
            .evidence_depth_fixture_normalization_report
            .normalization_status,
        EvidenceDepthFixtureNormalizationStatus::NormalizationReady
    );
    assert_eq!(
        bundle
            .evidence_depth_fixture_completeness_report
            .completeness_status,
        EvidenceDepthFixtureCompletenessStatus::FixtureCompletenessReady
    );
    assert_eq!(
        bundle.official_evidence_depth_fixture_pack_v2.pack_status,
        OfficialEvidenceDepthFixturePackV2Status::FixturePackReady
    );
    assert_eq!(
        bundle.committee_reference_fixture_pack_v2.pack_status,
        CommitteeReferenceFixturePackV2Status::CommitteeFixturePackReady
    );
    assert_eq!(
        bundle.defensive_counterfactual_fixture_pack_v2.pack_status,
        DefensiveCounterfactualFixturePackV2Status::DefensiveFixturePackReady
    );
    assert_eq!(
        expected["audit_status"].as_str(),
        Some("FixturesReadyWithWarnings")
    );
    harness::assert_source_boundary_preserved(&manifest);
    harness::assert_no_lookahead_preserved(&manifest);
}

#[test]
fn sprint83_fixture_determinism_regression_is_ready() {
    let bundle = sprint::run_sprint83_bundle(
        "soma_evidence_depth_determinism_regression.toml",
        "sprint83-determinism-regression-suite",
    );
    let expected: Value = harness::load_json_fixture(sprint::example_path(
        "sprint83_data/determinism_regression_expected.json",
    ));

    assert_eq!(
        bundle
            .evidence_depth_determinism_regression_report
            .regression_status,
        EvidenceDepthDeterminismRegressionStatus::DeterminismReady
    );
    assert!(
        bundle
            .evidence_depth_determinism_regression_report
            .fingerprint_match
    );
    assert_eq!(
        expected["regression_status"].as_str(),
        Some("DeterminismReady")
    );
}

#[test]
fn sprint83_smoke_compression_and_runtime_plan_preserve_safety() {
    let bundle = sprint::run_sprint83_bundle(
        "soma_test_runtime_recovery_plan.toml",
        "sprint83-runtime-plan-suite",
    );
    assert_eq!(
        bundle.sprint82_cli_smoke_compression_report.smoke_status,
        Sprint82CliSmokeCompressionStatus::SmokeCompressionReady
    );
    assert!(
        bundle
            .sprint82_cli_smoke_compression_report
            .coverage_preserved
    );
    assert!(
        bundle
            .sprint82_cli_smoke_compression_report
            .representative_cli_commands
            .len()
            < bundle
                .sprint82_cli_smoke_compression_report
                .original_cli_commands
                .len()
    );
    assert_eq!(
        bundle.test_runtime_recovery_plan.plan_status,
        TestRuntimeRecoveryPlanStatus::RecoveryPlanReady
    );
    assert!(
        bundle
            .test_runtime_recovery_plan
            .actions
            .contains(&TestRuntimeRecoveryAction::InvestigateTestBinaryExplosion)
    );
    assert!(
        bundle
            .test_runtime_recovery_plan
            .actions
            .contains(&TestRuntimeRecoveryAction::SplitFullWorkspaceByTier)
    );
}

#[test]
fn sprint83_fixture_boundary_audit_and_gate_remain_conservative() {
    let bundle = sprint::run_sprint83_bundle(
        "soma_workspace_acceptance_recovery_gate.toml",
        "sprint83-boundary-gate-suite",
    );
    let expected: Value = harness::load_json_fixture(sprint::example_path(
        "sprint83_data/recovery_gate_expected.json",
    ));

    assert_eq!(
        bundle
            .no_lookahead_source_boundary_fixture_audit_v2
            .audit_status,
        NoLookaheadSourceBoundaryFixtureAuditV2Status::FixtureBoundaryReady
    );
    assert_eq!(
        bundle.workspace_acceptance_recovery_gate.gate_status,
        WorkspaceAcceptanceRecoveryGateStatus::BlockedByFullWorkspace
    );
    assert_eq!(
        bundle
            .workspace_acceptance_recovery_gate
            .full_workspace_status,
        FullWorkspaceAcceptanceRecoveryStatus::FullWorkspaceBlockedByCompilation
    );
    assert!(
        bundle
            .workspace_acceptance_recovery_gate
            .safety_coverage_preserved
    );
    assert_eq!(
        expected["full_workspace_status"].as_str(),
        Some("FullWorkspaceBlockedByCompilation")
    );
}

#[test]
fn sprint83_fixture_boundary_audit_blocks_source_boundary_issues() {
    let mut manifest: Value = harness::load_json_fixture(sprint::example_path(
        "sprint83_data/sprint82_fixture_manifest.json",
    ));
    manifest["fixtures"][0]["source_boundary_fields_present"] = Value::Bool(false);
    let manifest_path =
        sprint::write_support_json("sprint83-boundary-bad", "manifest.json", &manifest);

    let mut config = sprint::sprint83_recovery_config_from_example(
        "soma_fixture_boundary_audit_v2.toml",
        "sprint83-boundary-bad-suite",
    );
    config.sprint82_fixture_paths = vec![manifest_path];

    let report = Sprint83AcceptanceRecoveryRunner::default()
        .run_fixture_boundary_audit_v2(&config)
        .expect("boundary audit");
    assert_eq!(
        report.audit_status,
        NoLookaheadSourceBoundaryFixtureAuditV2Status::SourceBoundaryViolation
    );
}

#[test]
fn sprint83_recovery_panel_is_read_only_and_runtime_deferred() {
    let bundle = sprint::run_sprint83_bundle(
        "soma_control_tower_sprint83_recovery.toml",
        "sprint83-recovery-panel-suite",
    );
    assert_eq!(
        bundle
            .control_tower_sprint83_recovery_panel
            .full_workspace_acceptance_status,
        FullWorkspaceAcceptanceRecoveryStatus::FullWorkspaceBlockedByCompilation
    );
    assert_eq!(
        bundle
            .control_tower_sprint83_recovery_panel
            .recovery_gate_status,
        WorkspaceAcceptanceRecoveryGateStatus::BlockedByFullWorkspace
    );
    assert!(
        bundle
            .control_tower_sprint83_recovery_panel
            .runtime_deferred_status
            .contains("runtime deferred")
    );
}

#[test]
fn sprint83_cli_help_and_local_only_guards_are_present() {
    let bin = env!("CARGO_BIN_EXE_soma_experiment");
    let expected = [
        ("sprint83-acceptance-recovery", "acceptance recovery bundle"),
        (
            "full-workspace-acceptance-recovery",
            "full workspace acceptance recovery status",
        ),
        (
            "long-compilation-diagnosis",
            "long-running compilation diagnosis",
        ),
        (
            "evidence-depth-fixture-audit",
            "evidence-depth fixture audit",
        ),
        (
            "evidence-depth-fixture-normalize",
            "evidence-depth fixture normalization",
        ),
        (
            "evidence-depth-fixture-completeness",
            "evidence-depth fixture completeness",
        ),
        (
            "evidence-depth-determinism-regression",
            "evidence-depth determinism regression",
        ),
        ("sprint82-smoke-compress", "Sprint 82 smoke compression"),
        ("fixture-boundary-audit-v2", "fixture boundary audit v2"),
        ("test-runtime-recovery-plan", "test runtime recovery plan"),
        (
            "workspace-acceptance-recovery-gate",
            "workspace acceptance recovery gate",
        ),
        (
            "control-tower-sprint83-recovery",
            "Sprint 83 recovery panel",
        ),
    ];
    for (command, text) in expected {
        let help = std::process::Command::new(bin)
            .args([command, "--help"])
            .output()
            .expect("help");
        assert!(help.status.success());
        let stdout = String::from_utf8(help.stdout).expect("stdout");
        assert!(stdout.contains("--config"));
        assert!(stdout.to_lowercase().contains(&text.to_lowercase()));
    }

    let root_help = std::process::Command::new(bin)
        .arg("--help")
        .output()
        .expect("root help");
    let root_stdout = String::from_utf8(root_help.stdout).expect("stdout");
    assert!(root_stdout.contains("sprint83-acceptance-recovery"));
    assert!(root_stdout.contains("workspace-acceptance-recovery-gate"));
    assert!(root_stdout.contains("control-tower-sprint83-recovery"));
    assert!(!root_stdout.contains("train-model"));
    assert!(!root_stdout.contains("live-inference"));

    for command in [
        "sprint83-acceptance-recovery",
        "full-workspace-acceptance-recovery",
        "long-compilation-diagnosis",
        "evidence-depth-fixture-audit",
        "evidence-depth-fixture-normalize",
        "evidence-depth-fixture-completeness",
        "evidence-depth-determinism-regression",
        "sprint82-smoke-compress",
        "fixture-boundary-audit-v2",
        "test-runtime-recovery-plan",
        "workspace-acceptance-recovery-gate",
        "control-tower-sprint83-recovery",
    ] {
        let remote = std::process::Command::new(bin)
            .args([command, "--config", "https://example.com/sprint83.toml"])
            .output()
            .expect("remote config");
        assert!(!remote.status.success());
        let stderr = String::from_utf8(remote.stderr).expect("stderr");
        assert!(stderr.contains("must be local"));
    }
}

#[test]
fn sprint83_grouped_suite_is_deterministic() {
    let first: Sprint83AcceptanceRecoveryBundle = sprint::run_sprint83_bundle(
        "soma_sprint83_acceptance_recovery.toml",
        "sprint83-determinism-suite-a",
    );
    let second: Sprint83AcceptanceRecoveryBundle = sprint::run_sprint83_bundle(
        "soma_sprint83_acceptance_recovery.toml",
        "sprint83-determinism-suite-b",
    );
    assert_eq!(first, second);
}
