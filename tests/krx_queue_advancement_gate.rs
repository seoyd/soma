mod support;

use serde_json::json;
use soma_zero::{
    KrxEvidenceQueueAdvancementGateStatus, KrxEvidenceWarningClosureConfig,
    Sprint92KrxWarningClosureRunner,
};
use support::{shared_fixture_harness as harness, sprint69_support as sprint};

fn config(name: &str) -> KrxEvidenceWarningClosureConfig {
    sprint::sprint92_config_from_example("soma_krx_queue_advancement_gate.toml", name)
}

#[test]
fn queue_advancement_default_remains_blocked() {
    let report = Sprint92KrxWarningClosureRunner::default()
        .run_krx_queue_advancement_gate(&config("krx-queue-default"))
        .expect("report");
    assert_eq!(
        report.gate_status,
        KrxEvidenceQueueAdvancementGateStatus::DashboardRendererEntryBlocked
    );
    assert_eq!(report.primary_next_family, "KrxEvidence");
}

#[test]
fn queue_advancement_can_be_ready_or_still_primary() {
    let mut ready = config("krx-queue-ready");
    ready.allow_dashboard_renderer_entry_if_closed = true;
    let mut summary = harness::load_json_fixture::<serde_json::Value>(sprint::example_path(
        "sprint92_data/sprint91_summary.json",
    ));
    summary["non_krx_targets_seen"] = json!(["DashboardRenderer"]);
    let dir = harness::temp_output_dir_for_test("krx-queue-ready");
    let summary_path = dir.join("sprint91_summary.json");
    std::fs::write(
        &summary_path,
        serde_json::to_string_pretty(&summary).expect("json"),
    )
    .expect("write");
    ready.sprint91_bundle_paths = vec![summary_path.display().to_string()];
    let ready_report = Sprint92KrxWarningClosureRunner::default()
        .run_krx_queue_advancement_gate(&ready)
        .expect("ready report");
    assert_eq!(
        ready_report.gate_status,
        KrxEvidenceQueueAdvancementGateStatus::DashboardRendererEntryReady
    );
    assert_eq!(ready_report.primary_next_family, "DashboardRenderer");

    let mut unsafe_cfg = config("krx-queue-primary");
    let raw = dir.join("krx_raw_archive_secret_safety.rs");
    std::fs::write(&raw, "#[test]\nfn bad() { assert!(true); }\n").expect("write raw");
    unsafe_cfg.krx_secret_safety_paths = vec![raw.display().to_string()];
    unsafe_cfg.krx_raw_archive_paths = vec![raw.display().to_string()];
    let unsafe_report = Sprint92KrxWarningClosureRunner::default()
        .run_krx_queue_advancement_gate(&unsafe_cfg)
        .expect("unsafe report");
    assert_eq!(
        unsafe_report.gate_status,
        KrxEvidenceQueueAdvancementGateStatus::KrxStillPrimary
    );
}
