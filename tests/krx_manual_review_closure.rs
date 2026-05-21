mod support;

use soma_zero::{
    KrxEvidenceManualReviewClosureStatus, KrxEvidenceWarningClosureConfig,
    Sprint92KrxWarningClosureRunner,
};
use support::{shared_fixture_harness as harness, sprint69_support as sprint};

fn config(name: &str) -> KrxEvidenceWarningClosureConfig {
    sprint::sprint92_config_from_example("soma_krx_manual_review_close.toml", name)
}

#[test]
fn manual_review_closure_matches_expected_fixture() {
    let report = Sprint92KrxWarningClosureRunner::default()
        .run_krx_manual_review_close(&config("krx-manual-review-default"))
        .expect("report");
    let expected = harness::load_json_fixture(sprint::example_path(
        "sprint92_data/krx_manual_review_closure_expected.json",
    ));
    assert_eq!(report, expected);
    assert_eq!(
        report.closure_status,
        KrxEvidenceManualReviewClosureStatus::ManualReviewClosedWithIsolatedSentinel
    );
    assert_eq!(report.manual_review_required_count, 0);
}

#[test]
fn manual_review_can_stay_needed_or_turn_unsafe() {
    let mut needed = config("krx-manual-review-needed");
    needed.require_secret_safety_isolation_decision = false;
    let needed_report = Sprint92KrxWarningClosureRunner::default()
        .run_krx_manual_review_close(&needed)
        .expect("needed report");
    assert_eq!(
        needed_report.closure_status,
        KrxEvidenceManualReviewClosureStatus::ManualReviewStillNeeded
    );

    let mut unsafe_cfg = config("krx-manual-review-unsafe");
    let dir = harness::temp_output_dir_for_test("krx-manual-review-unsafe");
    let raw = dir.join("krx_raw_archive_secret_safety.rs");
    std::fs::write(&raw, "#[test]\nfn bad() { assert!(true); }\n").expect("write raw");
    unsafe_cfg.krx_secret_safety_paths = vec![raw.display().to_string()];
    unsafe_cfg.krx_raw_archive_paths = vec![raw.display().to_string()];
    let unsafe_report = Sprint92KrxWarningClosureRunner::default()
        .run_krx_manual_review_close(&unsafe_cfg)
        .expect("unsafe report");
    assert_eq!(
        unsafe_report.closure_status,
        KrxEvidenceManualReviewClosureStatus::UnsafeToClose
    );
}
