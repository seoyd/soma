mod support;

use std::fs;

use serde_json::json;
use soma_zero::{
    KrxEvidenceWarningClosureConfig, KrxSecretSafetyIsolationDecisionStatus,
    KrxSecretSafetyRiskLevel, Sprint92KrxWarningClosureRunner,
};
use support::{shared_fixture_harness as harness, sprint69_support as sprint};

fn config(name: &str) -> KrxEvidenceWarningClosureConfig {
    sprint::sprint92_config_from_example("soma_krx_secret_safety_isolation.toml", name)
}

fn override_summary(
    config: &mut KrxEvidenceWarningClosureConfig,
    test_name: &str,
    summary: serde_json::Value,
) {
    let dir = harness::temp_output_dir_for_test(test_name);
    let summary_path = dir.join("sprint91_summary.json");
    fs::write(
        &summary_path,
        serde_json::to_string_pretty(&summary).expect("json"),
    )
    .expect("write");
    config
        .sprint91_bundle_paths
        .insert(0, summary_path.display().to_string());
    config.output_root = dir.display().to_string();
}

#[test]
fn secret_safety_default_matches_expected_fixture() {
    let report = Sprint92KrxWarningClosureRunner::default()
        .run_krx_secret_safety_isolation(&config("krx-secret-safety-default"))
        .expect("report");
    let expected = harness::load_json_fixture(sprint::example_path(
        "sprint92_data/krx_secret_safety_isolation_expected.json",
    ));
    assert_eq!(report, expected);
    assert_eq!(
        report.decision_status,
        KrxSecretSafetyIsolationDecisionStatus::KeepIsolatedSentinel
    );
    assert_eq!(report.safety_risk, KrxSecretSafetyRiskLevel::Low);
}

#[test]
fn secret_safety_supports_safe_to_migrate_and_workspace_representation() {
    let mut safe = config("krx-secret-safety-safe");
    let mut summary = json!(harness::load_json_fixture::<serde_json::Value>(
        sprint::example_path("sprint92_data/sprint91_summary.json",)
    ));
    summary["assertions_remaining"] = json!(0);
    summary["assertions_migrated"] = json!(11);
    let safe_dir = harness::temp_output_dir_for_test("krx-secret-safety-safe");
    let safe_summary_path = safe_dir.join("sprint91_summary.json");
    fs::write(
        &safe_summary_path,
        serde_json::to_string_pretty(&summary).expect("json"),
    )
    .expect("write summary");
    safe.sprint91_bundle_paths = vec![safe_summary_path.display().to_string()];
    safe.krx_assertion_migration_paths.clear();
    safe.output_root = safe_dir.display().to_string();
    let safe_report = Sprint92KrxWarningClosureRunner::default()
        .run_krx_secret_safety_isolation(&safe)
        .expect("safe report");
    assert_eq!(
        safe_report.decision_status,
        KrxSecretSafetyIsolationDecisionStatus::SafeToMigrateToKrxSuite
    );

    let mut represented = config("krx-secret-safety-represented");
    let represented_summary = harness::load_json_fixture::<serde_json::Value>(
        sprint::example_path("sprint92_data/sprint91_summary.json"),
    );
    override_summary(
        &mut represented,
        "krx-secret-safety-represented",
        represented_summary,
    );
    let dir = harness::temp_output_dir_for_test("krx-secret-safety-represented-workspace");
    let workspace = dir.join("workspace_safety_guard_suite.rs");
    fs::write(&workspace, "#[test]\nfn keeps_archive_redaction_assertions() { let _ = \"archive_redaction_assertions\"; }\n").expect("write workspace");
    represented
        .workspace_gate_paths
        .insert(0, workspace.display().to_string());
    let represented_report = Sprint92KrxWarningClosureRunner::default()
        .run_krx_secret_safety_isolation(&represented)
        .expect("represented report");
    assert_eq!(
        represented_report.decision_status,
        KrxSecretSafetyIsolationDecisionStatus::RepresentedInWorkspaceSafetySuite
    );
}

#[test]
fn secret_safety_detects_unsafe_merge_and_is_deterministic() {
    let mut unsafe_config = config("krx-secret-safety-unsafe");
    let dir = harness::temp_output_dir_for_test("krx-secret-safety-unsafe-raw");
    let raw = dir.join("krx_raw_archive_secret_safety.rs");
    fs::write(
        &raw,
        "#[test]\nfn not_enough_redaction() { assert!(true); }\n",
    )
    .expect("write raw");
    unsafe_config.krx_secret_safety_paths = vec![raw.display().to_string()];
    unsafe_config.krx_raw_archive_paths = vec![raw.display().to_string()];
    let first = Sprint92KrxWarningClosureRunner::default()
        .run_krx_secret_safety_isolation(&unsafe_config)
        .expect("first");
    let second = Sprint92KrxWarningClosureRunner::default()
        .run_krx_secret_safety_isolation(&unsafe_config)
        .expect("second");
    assert_eq!(first, second);
    assert_eq!(
        first.decision_status,
        KrxSecretSafetyIsolationDecisionStatus::UnsafeToMerge
    );
}
