mod support;

use std::fs;

use serde_json::json;
use soma_zero::{
    KrxEvidenceWarningClosureConfig, KrxTimeoutCauseStatus, Sprint92KrxWarningClosureRunner,
};
use support::{shared_fixture_harness as harness, sprint69_support as sprint};

fn config(name: &str) -> KrxEvidenceWarningClosureConfig {
    sprint::sprint92_config_from_example("soma_krx_no_run_timeout_cause.toml", name)
}

fn summary_with(field: &str, value: serde_json::Value) -> serde_json::Value {
    let mut summary = harness::load_json_fixture::<serde_json::Value>(sprint::example_path(
        "sprint92_data/sprint91_summary.json",
    ));
    summary[field] = value;
    summary
}

#[test]
fn timeout_causes_default_to_unknown() {
    let runner = Sprint92KrxWarningClosureRunner::default();
    let config = config("krx-timeout-default");
    assert_eq!(
        runner
            .run_krx_no_run_timeout_cause(&config)
            .expect("no-run")
            .timeout_cause_status,
        KrxTimeoutCauseStatus::UnknownTimeout
    );
    assert_eq!(
        runner
            .run_krx_full_workspace_timeout_cause(&config)
            .expect("full")
            .timeout_cause_status,
        KrxTimeoutCauseStatus::UnknownTimeout
    );
}

#[test]
fn timeout_causes_detect_krx_and_non_krx_cases() {
    let dir = harness::temp_output_dir_for_test("krx-timeout-cases");
    let path = dir.join("sprint91_summary.json");

    let mut krx_cfg = config("krx-timeout-krx");
    fs::write(
        &path,
        serde_json::to_string_pretty(&summary_with("krx_targets_seen", json!(["KrxEvidence"])))
            .expect("json"),
    )
    .expect("write");
    krx_cfg.sprint91_bundle_paths = vec![path.display().to_string()];
    assert_eq!(
        Sprint92KrxWarningClosureRunner::default()
            .run_krx_no_run_timeout_cause(&krx_cfg)
            .expect("krx")
            .timeout_cause_status,
        KrxTimeoutCauseStatus::KrxRelatedTimeout
    );

    let mut non_krx_cfg = config("krx-timeout-non-krx");
    fs::write(
        &path,
        serde_json::to_string_pretty(&summary_with(
            "non_krx_targets_seen",
            json!(["DashboardRenderer"]),
        ))
        .expect("json"),
    )
    .expect("write");
    non_krx_cfg.sprint91_bundle_paths = vec![path.display().to_string()];
    assert_eq!(
        Sprint92KrxWarningClosureRunner::default()
            .run_krx_no_run_timeout_cause(&non_krx_cfg)
            .expect("non krx")
            .timeout_cause_status,
        KrxTimeoutCauseStatus::NonKrxTimeout
    );
}
