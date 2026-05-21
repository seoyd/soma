mod support;

use std::fs;

use serde_json::json;
use soma_zero::{
    KrxEvidenceWarningClosureConfig, MeasuredTargetDeltaV8Status, Sprint92KrxWarningClosureRunner,
};
use support::{shared_fixture_harness as harness, sprint69_support as sprint};

fn config(name: &str) -> KrxEvidenceWarningClosureConfig {
    sprint::sprint92_config_from_example("soma_measured_target_delta_v8.toml", name)
}

#[test]
fn measured_target_delta_defaults_to_sample_backed() {
    let report = Sprint92KrxWarningClosureRunner::default()
        .run_measured_target_delta_v8(&config("measured-delta-default"))
        .expect("report");
    assert_eq!(
        report.delta_status,
        MeasuredTargetDeltaV8Status::SampleBackedOnly
    );
    assert!(!report.measured);
    assert!(report.sample_backed);
}

#[test]
fn measured_target_delta_can_use_measured_inputs_and_warning_free_flag() {
    let mut config = config("measured-delta-measured");
    let dir = harness::temp_output_dir_for_test("measured-delta-measured");
    let bundle = dir.join("measured_target_delta_v7.txt");
    fs::write(
        &bundle,
        serde_json::to_string_pretty(&json!({
            "report_id": "sample",
            "target_count_before": 4,
            "target_count_after": 3,
            "krx_family_delta": 1,
            "measured": true,
            "sample_backed": false,
            "timing_available": true,
            "delta_status": "MeasuredTargetDeltaReady",
            "reason_codes": ["DeterministicPath", "LocalFileOnly"]
        }))
        .expect("json"),
    )
    .expect("write");
    let mut summary = harness::load_json_fixture::<serde_json::Value>(sprint::example_path(
        "sprint92_data/sprint91_summary.json",
    ));
    summary["assertions_remaining"] = json!(0);
    summary["assertions_migrated"] = json!(11);
    let summary_path = dir.join("sprint91_summary.json");
    fs::write(
        &summary_path,
        serde_json::to_string_pretty(&summary).expect("json"),
    )
    .expect("write summary");
    config.sprint91_bundle_paths = vec![
        summary_path.display().to_string(),
        dir.display().to_string(),
    ];
    config.krx_assertion_migration_paths.clear();
    let report = Sprint92KrxWarningClosureRunner::default()
        .run_measured_target_delta_v8(&config)
        .expect("report");
    assert!(report.measured);
    assert!(!report.sample_backed);
    assert!(report.warning_free_reduction);
    assert_eq!(
        report.delta_status,
        MeasuredTargetDeltaV8Status::MeasuredTargetDeltaReady
    );
}
