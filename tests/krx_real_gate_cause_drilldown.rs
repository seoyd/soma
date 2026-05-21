mod support;

use std::fs;

use serde_json::json;
use soma_zero::{
    KrxEvidenceRealGateCauseDrilldownStatus, KrxEvidenceWarningClosureConfig,
    Sprint92KrxWarningClosureRunner,
};
use support::{shared_fixture_harness as harness, sprint69_support as sprint};

fn config(name: &str) -> KrxEvidenceWarningClosureConfig {
    sprint::sprint92_config_from_example("soma_krx_real_gate_cause_drilldown.toml", name)
}

#[test]
fn gate_cause_reports_need_more_observation_by_default() {
    let report = Sprint92KrxWarningClosureRunner::default()
        .run_krx_real_gate_cause_drilldown(&config("krx-gate-cause-default"))
        .expect("report");
    assert_eq!(
        report.drilldown_status,
        KrxEvidenceRealGateCauseDrilldownStatus::NeedMoreObservation
    );
}

#[test]
fn gate_cause_can_be_non_krx_or_still_primary() {
    let mut ready = config("krx-gate-cause-non-krx");
    let mut summary = harness::load_json_fixture::<serde_json::Value>(sprint::example_path(
        "sprint92_data/sprint91_summary.json",
    ));
    summary["non_krx_tests_seen"] = json!(["DashboardRenderer"]);
    let dir = harness::temp_output_dir_for_test("krx-gate-cause-non-krx");
    let summary_path = dir.join("sprint91_summary.json");
    fs::write(
        &summary_path,
        serde_json::to_string_pretty(&summary).expect("json"),
    )
    .expect("write");
    ready.sprint91_bundle_paths = vec![summary_path.display().to_string()];
    let ready_report = Sprint92KrxWarningClosureRunner::default()
        .run_krx_real_gate_cause_drilldown(&ready)
        .expect("ready report");
    assert_eq!(
        ready_report.drilldown_status,
        KrxEvidenceRealGateCauseDrilldownStatus::KrxNoLongerPrimaryGateBlocker
    );

    let mut unsafe_cfg = config("krx-gate-cause-primary");
    let raw = dir.join("krx_raw_archive_secret_safety.rs");
    fs::write(&raw, "#[test]\nfn bad() { assert!(true); }\n").expect("write raw");
    unsafe_cfg.krx_secret_safety_paths = vec![raw.display().to_string()];
    unsafe_cfg.krx_raw_archive_paths = vec![raw.display().to_string()];
    let unsafe_report = Sprint92KrxWarningClosureRunner::default()
        .run_krx_real_gate_cause_drilldown(&unsafe_cfg)
        .expect("unsafe report");
    assert_eq!(
        unsafe_report.drilldown_status,
        KrxEvidenceRealGateCauseDrilldownStatus::KrxStillPrimaryGateBlocker
    );
}
