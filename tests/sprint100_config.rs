mod support;

use soma_zero::CommitteeQualityWarningClosureConfig;

#[test]
fn sprint100_config_defaults_are_safe() {
    let config = CommitteeQualityWarningClosureConfig::default();
    assert!(config.require_proposal_warning_closure);
    assert!(config.require_debate_evidence_closure);
    assert!(config.require_unsafe_rule_closure);
    assert!(config.require_scorecard_warning_closure);
    assert!(config.require_replay_warning_closure);
    assert!(config.require_risk_handoff_warning_closure);
    assert!(config.require_paper_readiness_gate);
    assert!(config.require_workspace_truth_separation);
    assert!(config.preserve_committee_owned_architecture);
    assert!(config.preserve_runtime_deferred);
    assert!(config.preserve_safety_guards);
    let text = config.to_toml_string().expect("toml");
    assert!(!text.contains("runtime_allowed"));
    assert!(!text.contains("training_allowed"));
    assert!(!text.contains("broker"));
    assert!(!text.contains("order"));
    assert!(!text.contains("account"));
}

#[test]
fn sprint100_config_rejects_remote_paths() {
    let mut config = CommitteeQualityWarningClosureConfig::default();
    config.sprint99_bundle_paths = Some(vec!["https://example.com/sprint99.json".to_string()]);
    let err = config.validate().expect_err("should reject remote path");
    assert!(err.contains("must be local"));
}
