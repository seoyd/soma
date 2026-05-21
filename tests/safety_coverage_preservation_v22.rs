mod support;

use soma_zero::WorkspaceAcceptanceRecoveryV7Runner;
use support::sprint106_support::{read_fixture, run_sprint106, sprint106_config_from_example};

#[test]
fn safety_coverage_v22_matches_expected_preserved_guards() {
    let bundle = run_sprint106(
        "soma_safety_coverage_preservation_v22.toml",
        "safety_coverage_preservation_v22",
    );
    let actual =
        serde_json::to_value(&bundle.safety_coverage_preservation_report_v22).expect("actual");
    let expected: serde_json::Value =
        read_fixture("sprint106_data/safety_coverage_v22_expected.json");
    assert_eq!(actual, expected);
    assert_eq!(
        bundle.safety_coverage_preservation_report_v22.safety_status,
        "SafetyCoveragePreservedV22"
    );
}

#[test]
fn safety_coverage_v22_regresses_when_safety_preservation_is_disabled() {
    let mut config = sprint106_config_from_example(
        "soma_safety_coverage_preservation_v22.toml",
        "safety_coverage_preservation_v22_disabled",
    );
    config.require_safety_preservation = false;

    let bundle = WorkspaceAcceptanceRecoveryV7Runner::default()
        .run(&config)
        .expect("run sprint106");

    assert_eq!(
        bundle.safety_coverage_preservation_report_v22.safety_status,
        "SafetyCoverageRegressionV22"
    );
    assert!(
        !bundle
            .safety_coverage_preservation_report_v22
            .live_trading_guard_present
    );
}
