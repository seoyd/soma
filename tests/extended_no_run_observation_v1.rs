mod support;

use soma_zero::SafeConsolidationPatchV2Runner;
use support::sprint108_support::{read_fixture, run_sprint108, sprint108_config_from_example};

#[test]
fn extended_no_run_observation_matches_expected_fixture() {
    let bundle = run_sprint108(
        "soma_extended_no_run_observation_v1.toml",
        "extended-no-run-observation-v1",
    );
    let actual =
        serde_json::to_value(&bundle.extended_no_run_observation_report_v1).expect("actual");
    let expected: serde_json::Value =
        read_fixture("sprint108_data/extended_no_run_observation_expected.json");
    assert_eq!(actual, expected);
    assert_eq!(
        bundle
            .extended_no_run_observation_report_v1
            .observation_status,
        "DiagnosticOnly"
    );
}

#[test]
fn extended_no_run_observation_records_timeout_when_enabled() {
    let mut config = sprint108_config_from_example(
        "soma_extended_no_run_observation_v1.toml",
        "extended-no-run-observation-v1-timeout",
    );
    config.run_real_no_run_after_patch = true;
    config.no_run_timeout_ms = Some(1);
    let bundle = SafeConsolidationPatchV2Runner::default()
        .run(&config)
        .expect("run");
    assert!(bundle.extended_no_run_observation_report_v1.attempted);
    assert!(
        matches!(
            bundle
                .extended_no_run_observation_report_v1
                .observation_status
                .as_str(),
            "ExtendedNoRunObservationTimedOutCleanly" | "ExtendedNoRunObservationReady"
        ),
        "unexpected status {}",
        bundle
            .extended_no_run_observation_report_v1
            .observation_status
    );
}
