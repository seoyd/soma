mod common;

use soma_zero::{CommitteeV1RunConfig, CommitteeV1Runner};

#[test]
fn committee_v1_config_can_be_constructed() {
    let cfg = CommitteeV1RunConfig::default();
    assert_eq!(cfg.run_id, "committee_v1");
    assert!(cfg.run_quality_metrics);
    assert!(cfg.run_calibration_suggestions);
}

#[test]
fn committee_v1_remote_paths_are_rejected() {
    let cfg = CommitteeV1RunConfig {
        scenario_load_config_path: Some("https://example.com/config.toml".to_string()),
        ..CommitteeV1RunConfig::default()
    };
    assert!(cfg.validate().is_err());
}

#[test]
fn committee_v1_max_bounds_are_enforced() {
    let too_many_scenarios = CommitteeV1RunConfig {
        max_scenarios: 51,
        ..CommitteeV1RunConfig::default()
    };
    let too_many_decisions = CommitteeV1RunConfig {
        max_decisions: 51,
        ..CommitteeV1RunConfig::default()
    };
    assert!(too_many_scenarios.validate().is_err());
    assert!(too_many_decisions.validate().is_err());
}

#[test]
fn committee_v1_toml_has_no_live_or_broker_fields() {
    let toml = CommitteeV1RunConfig::default()
        .to_toml_string()
        .expect("toml");
    assert!(!toml.contains("broker"));
    assert!(!toml.contains("account"));
    assert!(!toml.contains("llm"));
}

#[test]
fn committee_v1_calibration_suggestions_never_auto_apply() {
    let cfg = CommitteeV1RunConfig {
        run_id: "committee-v1-config-auto-apply".to_string(),
        output_root: common::output_dir("committee-v1-config-auto-apply")
            .display()
            .to_string(),
        ..CommitteeV1RunConfig::default()
    };
    let report = CommitteeV1Runner::default().run(&cfg).expect("run");
    assert!(
        report
            .chair_calibration_report
            .suggestions
            .iter()
            .all(|suggestion| !suggestion.apply_automatically)
    );
    assert!(
        report
            .risk_calibration_report
            .suggestions
            .iter()
            .all(|suggestion| !suggestion.apply_automatically)
    );
}
