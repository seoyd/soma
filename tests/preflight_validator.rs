mod common;

use soma_zero::{PreflightFinalStatus, ReasonCode};

#[test]
fn missing_file_returns_reason_coded_missing_file() {
    let mut config = common::onboarding_config("missing-file", "generic_ohlcv_valid.csv");
    config.input_path = common::output_dir("missing-file")
        .join("does_not_exist.csv")
        .display()
        .to_string();
    let report = common::run_preflight(&config);
    assert_eq!(report.final_status, PreflightFinalStatus::MissingFile);
    assert!(report.reason_codes.contains(&ReasonCode::MissingFile));
}

#[test]
fn unsupported_format_returns_unsupported_format() {
    let path = common::write_temp_csv("unsupported-format", "foo,bar,baz\n1,2,3\n");
    let mut config = common::onboarding_config("unsupported-format", "generic_ohlcv_valid.csv");
    config.input_path = path.display().to_string();
    let report = common::run_preflight(&config);
    assert_eq!(report.final_status, PreflightFinalStatus::UnsupportedFormat);
}

#[test]
fn ambiguous_format_in_strict_mode_returns_ambiguous() {
    let path = common::write_temp_csv(
        "ambiguous-format",
        "timestamp_ms,open,high,low,close\n1,2,3,4,5\n2,3,4,5,6\n",
    );
    let mut config = common::onboarding_config("ambiguous-format", "generic_ohlcv_valid.csv");
    config.input_path = path.display().to_string();
    let report = common::run_preflight(&config);
    assert_eq!(report.final_status, PreflightFinalStatus::AmbiguousFormat);
}

#[test]
fn valid_fixture_can_be_ready_for_real_evidence_only_when_user_supplied() {
    let ready = common::run_preflight(&common::onboarding_config(
        "ready-fixture",
        "generic_ohlcv_valid_alt.csv",
    ));
    assert_eq!(
        ready.final_status,
        PreflightFinalStatus::ReadyForRealEvidence
    );

    let mut blocked = common::onboarding_config("not-user-supplied", "generic_ohlcv_valid_alt.csv");
    blocked.user_supplied = false;
    let blocked_report = common::run_preflight(&blocked);
    assert_eq!(
        blocked_report.final_status,
        PreflightFinalStatus::NotRealLocalEligible
    );
}

#[test]
fn low_quality_fixture_and_insufficient_rows_are_rejected_conservatively() {
    let low_quality = common::run_preflight(&common::onboarding_config(
        "low-quality",
        "generic_ohlcv_bad_ohlc.csv",
    ));
    assert_eq!(
        low_quality.final_status,
        PreflightFinalStatus::DataQualityTooLow
    );

    let mut insufficient = common::onboarding_config("insufficient", "generic_ohlcv_valid.csv");
    insufficient.min_rows_for_preflight = 100;
    let insufficient_report = common::run_preflight(&insufficient);
    assert_eq!(
        insufficient_report.final_status,
        PreflightFinalStatus::NeedsMoreRows
    );
}

#[test]
fn duplicate_gap_and_out_of_order_conditions_surface_reason_codes() {
    let gap_report = common::run_preflight(&common::onboarding_config(
        "gap-report",
        "generic_ohlcv_gaps.csv",
    ));
    assert!(gap_report.reason_codes.contains(&ReasonCode::GapDetected));

    let out_of_order_report = common::run_preflight(&common::onboarding_config(
        "out-of-order-report",
        "generic_ohlcv_out_of_order.csv",
    ));
    assert!(
        out_of_order_report
            .reason_codes
            .contains(&ReasonCode::OutOfOrderTimestampDetected)
    );
}

#[test]
fn preflight_report_rendering_is_deterministic() {
    let config =
        common::onboarding_config("deterministic-preflight", "generic_ohlcv_valid_alt.csv");
    let report_a = common::run_preflight(&config);
    let report_b = common::run_preflight(&config);
    assert_eq!(report_a.to_text(), report_b.to_text());
}
