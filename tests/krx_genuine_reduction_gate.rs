mod support;

use std::fs;

use serde_json::json;
use soma_zero::{
    KrxEvidenceGenuineReductionGateStatus, KrxEvidenceWarningClosureConfig,
    Sprint92KrxWarningClosureRunner,
};
use support::{shared_fixture_harness as harness, sprint69_support as sprint};

fn config(name: &str) -> KrxEvidenceWarningClosureConfig {
    sprint::sprint92_config_from_example("soma_krx_genuine_reduction_gate.toml", name)
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
    config.sprint91_bundle_paths = vec![summary_path.display().to_string()];
}

#[test]
fn genuine_gate_matches_expected_fixture() {
    let report = Sprint92KrxWarningClosureRunner::default()
        .run_krx_genuine_reduction_gate(&config("krx-genuine-default"))
        .expect("report");
    let expected = harness::load_json_fixture(sprint::example_path(
        "sprint92_data/krx_genuine_gate_expected.json",
    ));
    assert_eq!(report, expected);
    assert_eq!(
        report.gate_status,
        KrxEvidenceGenuineReductionGateStatus::KrxEvidenceReducedWithIsolatedSentinel
    );
}

#[test]
fn genuine_gate_supports_full_close_warning_backed_and_unsafe_states() {
    let mut full = config("krx-genuine-full");
    full.require_warning_free_reduction = true;
    let mut summary = harness::load_json_fixture::<serde_json::Value>(sprint::example_path(
        "sprint92_data/sprint91_summary.json",
    ));
    summary["assertions_remaining"] = json!(0);
    summary["assertions_migrated"] = json!(11);
    override_summary(&mut full, "krx-genuine-full", summary);
    full.krx_assertion_migration_paths.clear();
    let full_report = Sprint92KrxWarningClosureRunner::default()
        .run_krx_genuine_reduction_gate(&full)
        .expect("full report");
    assert_eq!(
        full_report.gate_status,
        KrxEvidenceGenuineReductionGateStatus::KrxEvidenceGenuinelyReduced
    );

    let mut warning = config("krx-genuine-warning");
    let dir = harness::temp_output_dir_for_test("krx-genuine-warning");
    let raw = dir.join("krx_raw_archive_secret_safety.rs");
    fs::write(
        &raw,
        "#[test]\nfn archive_redaction_assertions() {\n    let rendered = \"auth=redacted\";\n    assert!(rendered.contains(\"auth=redacted\"));\n    assert!(\"KRX_API_KEY\".contains(\"KRX_API_KEY\"));\n    assert!(!rendered.contains(\"KRX_API_KEY\")); // !rendered.contains(\"KRX_API_KEY\")\n}\n",
    )
    .expect("write raw");
    let dashboard = dir.join("dashboard_renderer.rs");
    fs::write(&dashboard, "pub enum ExecuteOrder {}\n").expect("write dashboard");
    warning.krx_secret_safety_paths = vec![raw.display().to_string()];
    warning.krx_raw_archive_paths = vec![raw.display().to_string()];
    warning
        .workspace_gate_paths
        .insert(0, dashboard.display().to_string());
    let warning_report = Sprint92KrxWarningClosureRunner::default()
        .run_krx_genuine_reduction_gate(&warning)
        .expect("warning report");
    assert_eq!(
        warning_report.gate_status,
        KrxEvidenceGenuineReductionGateStatus::KrxEvidenceStillWarningBacked
    );

    fs::write(&raw, "#[test]\nfn bad() { assert!(true); }\n").expect("write raw unsafe");
    let unsafe_report = Sprint92KrxWarningClosureRunner::default()
        .run_krx_genuine_reduction_gate(&warning)
        .expect("unsafe report");
    assert_eq!(
        unsafe_report.gate_status,
        KrxEvidenceGenuineReductionGateStatus::KrxEvidenceUnsafeToAdvance
    );
}
