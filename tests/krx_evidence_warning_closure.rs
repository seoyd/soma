mod support;

use soma_zero::{
    CompileFamilyV2, KrxEvidenceWarningClosureConfig, KrxEvidenceWarningClosureStatus,
    Sprint92KrxWarningClosureRunner,
};
use support::{shared_fixture_harness as harness, sprint69_support as sprint};

#[test]
fn config_defaults_stay_local_and_krx_only() {
    let config = KrxEvidenceWarningClosureConfig::default();
    assert_eq!(config.target_family, CompileFamilyV2::KrxEvidence);
    assert!(config.require_secret_safety_isolation_decision);
    assert!(config.require_raw_archive_redaction_coverage);
    assert!(config.require_warning_free_reduction);
    assert!(config.require_manual_review_closure);
    let json = serde_json::to_string(&config).expect("json");
    assert!(!json.contains("runtime"));
    assert!(!json.contains("training"));
    assert!(!json.contains("://"));
}

#[test]
fn config_rejects_remote_paths() {
    let mut config = KrxEvidenceWarningClosureConfig::default();
    config.sprint91_bundle_paths = vec!["https://example.com/bundle".to_string()];
    assert!(config.validate().is_err());
}

#[test]
fn warning_closure_matches_expected_fixture_and_is_deterministic() {
    let config = sprint::sprint92_config_from_example(
        "soma_krx_warning_closure.toml",
        "krx-warning-closure-expected",
    );
    let runner = Sprint92KrxWarningClosureRunner::default();
    let first = runner.run_krx_warning_closure(&config).expect("first");
    let second = runner.run_krx_warning_closure(&config).expect("second");
    let expected = harness::load_json_fixture(sprint::example_path(
        "sprint92_data/krx_warning_closure_expected.json",
    ));
    assert_eq!(first, expected);
    assert_eq!(first, second);
    assert_eq!(
        first.closure_status,
        KrxEvidenceWarningClosureStatus::KrxWarningsClosedWithIsolatedSentinel
    );
}
