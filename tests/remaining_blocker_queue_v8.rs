mod support;

use std::fs;

use serde_json::json;
use soma_zero::{
    KrxEvidenceWarningClosureConfig, RemainingBlockerQueueV8Status, Sprint92KrxWarningClosureRunner,
};
use support::{shared_fixture_harness as harness, sprint69_support as sprint};

fn config(name: &str) -> KrxEvidenceWarningClosureConfig {
    sprint::sprint92_config_from_example("soma_remaining_blocker_queue_v8.toml", name)
}

#[test]
fn remaining_queue_defaults_to_krx_blocked() {
    let report = Sprint92KrxWarningClosureRunner::default()
        .run_remaining_blocker_queue_v8(&config("remaining-queue-default"))
        .expect("report");
    assert_eq!(
        report.queue_status,
        RemainingBlockerQueueV8Status::QueueBlockedByKrx
    );
    assert_eq!(report.primary_next_family, "KrxEvidence");
}

#[test]
fn remaining_queue_can_be_ready_or_advanced_to_dashboard_renderer() {
    let mut ready = config("remaining-queue-ready");
    let mut summary = harness::load_json_fixture::<serde_json::Value>(sprint::example_path(
        "sprint92_data/sprint91_summary.json",
    ));
    summary["non_krx_targets_seen"] = json!(["DashboardRenderer"]);
    let dir = harness::temp_output_dir_for_test("remaining-queue-ready");
    let summary_path = dir.join("sprint91_summary.json");
    fs::write(
        &summary_path,
        serde_json::to_string_pretty(&summary).expect("json"),
    )
    .expect("write");
    ready.sprint91_bundle_paths = vec![summary_path.display().to_string()];
    let ready_report = Sprint92KrxWarningClosureRunner::default()
        .run_remaining_blocker_queue_v8(&ready)
        .expect("ready report");
    assert_eq!(
        ready_report.queue_status,
        RemainingBlockerQueueV8Status::QueueReady
    );
    assert_eq!(ready_report.primary_next_family, "DashboardRenderer");

    ready.allow_dashboard_renderer_entry_if_closed = true;
    let advanced_report = Sprint92KrxWarningClosureRunner::default()
        .run_remaining_blocker_queue_v8(&ready)
        .expect("advanced report");
    assert_eq!(
        advanced_report.queue_status,
        RemainingBlockerQueueV8Status::QueueAdvanced
    );
    assert!(advanced_report.dashboard_renderer_entry_allowed);
}
