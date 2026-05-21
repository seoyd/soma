mod support;

use std::fs;

use serde_json::json;
use soma_zero::{
    DashboardRendererEntryGateStatus, KrxEvidenceWarningClosureConfig,
    Sprint92KrxWarningClosureRunner,
};
use support::{shared_fixture_harness as harness, sprint69_support as sprint};

fn config(name: &str) -> KrxEvidenceWarningClosureConfig {
    sprint::sprint92_config_from_example("soma_dashboard_renderer_entry_gate.toml", name)
}

#[test]
fn dashboard_entry_matches_expected_fixture() {
    let report = Sprint92KrxWarningClosureRunner::default()
        .run_dashboard_renderer_entry_gate(&config("dashboard-entry-default"))
        .expect("report");
    let expected = harness::load_json_fixture(sprint::example_path(
        "sprint92_data/dashboard_renderer_entry_gate_expected.json",
    ));
    assert_eq!(report, expected);
    assert_eq!(
        report.gate_status,
        DashboardRendererEntryGateStatus::DashboardRendererEntryBlockedByUnknownGateCause
    );
}

#[test]
fn dashboard_entry_supports_ready_warning_and_secret_blocks() {
    let mut ready = config("dashboard-entry-ready");
    ready.allow_dashboard_renderer_entry_if_closed = true;
    let mut summary = harness::load_json_fixture::<serde_json::Value>(sprint::example_path(
        "sprint92_data/sprint91_summary.json",
    ));
    summary["non_krx_targets_seen"] = json!(["DashboardRenderer"]);
    let dir = harness::temp_output_dir_for_test("dashboard-entry-ready");
    let summary_path = dir.join("sprint91_summary.json");
    fs::write(
        &summary_path,
        serde_json::to_string_pretty(&summary).expect("json"),
    )
    .expect("write");
    ready.sprint91_bundle_paths = vec![summary_path.display().to_string()];
    let ready_report = Sprint92KrxWarningClosureRunner::default()
        .run_dashboard_renderer_entry_gate(&ready)
        .expect("ready report");
    assert_eq!(
        ready_report.gate_status,
        DashboardRendererEntryGateStatus::DashboardRendererEntryReady
    );

    let mut warning = config("dashboard-entry-warning");
    let raw = dir.join("krx_raw_archive_secret_safety.rs");
    fs::write(
        &raw,
        "#[test]\nfn archive_redaction_assertions() {\n    let rendered = \"auth=redacted\";\n    assert!(rendered.contains(\"auth=redacted\"));\n    assert!(\"KRX_API_KEY\".contains(\"KRX_API_KEY\"));\n    assert!(!rendered.contains(\"KRX_API_KEY\")); // !rendered.contains(\"KRX_API_KEY\")\n}\n",
    )
    .expect("write raw warning");
    let dashboard = dir.join("dashboard_renderer.rs");
    fs::write(&dashboard, "pub enum ExecuteOrder {}\n").expect("write dashboard warning");
    warning.krx_secret_safety_paths = vec![raw.display().to_string()];
    warning.krx_raw_archive_paths = vec![raw.display().to_string()];
    warning
        .workspace_gate_paths
        .insert(0, dashboard.display().to_string());
    let warning_report = Sprint92KrxWarningClosureRunner::default()
        .run_dashboard_renderer_entry_gate(&warning)
        .expect("warning report");
    assert_eq!(
        warning_report.gate_status,
        DashboardRendererEntryGateStatus::DashboardRendererEntryBlockedByKrxWarnings
    );

    let mut secret = config("dashboard-entry-secret");
    fs::write(&raw, "#[test]\nfn bad() { assert!(true); }\n").expect("write raw secret");
    secret.krx_secret_safety_paths = vec![raw.display().to_string()];
    secret.krx_raw_archive_paths = vec![raw.display().to_string()];
    let secret_report = Sprint92KrxWarningClosureRunner::default()
        .run_dashboard_renderer_entry_gate(&secret)
        .expect("secret report");
    assert_eq!(
        secret_report.gate_status,
        DashboardRendererEntryGateStatus::DashboardRendererEntryBlockedBySecretSafety
    );
}
