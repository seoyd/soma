mod support;

use support::sprint100_support::run_sprint100;
use support::sprint101_support::run_sprint101;

#[test]
fn investor_archetype_ingestion_matches_requirements() {
    let _ = run_sprint100(
        "soma_sprint100_committee_closure.toml",
        "investor-archetype-ingestion-sprint100-base",
    );
    let bundle = run_sprint101(
        "soma_sprint101_investor_archetype_ingest.toml",
        "investor-archetype-ingestion",
    );
    assert_eq!(
        bundle.eighteen_investor_candidate_registry.candidates.len(),
        18
    );
    let summary = bundle.final_summary;
    for heading in [
        "## 1. Sprint summary",
        "## 6. Investor archetype ingestion",
        "## 34. Control Tower investor archetype panel",
        "## 46. Next gstack sprint recommendation",
    ] {
        assert!(summary.contains(heading), "missing heading {heading}");
    }
}

#[test]
fn sprint101_config_defaults_are_safe() {
    let config = soma_zero::InvestorArchetypeIngestionConfig::default();
    assert!(config.require_no_impersonation);
    assert!(config.require_source_confidence);
    assert!(config.require_do_not_learn_guards);
    assert!(config.require_style_grouping);
    assert!(config.require_feature_vectors);
    assert!(config.require_regime_routing);
    assert!(config.require_paper_only);
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
fn sprint101_config_rejects_remote_paths() {
    let mut config = soma_zero::InvestorArchetypeIngestionConfig::default();
    config.investor_material_paths = Some(vec!["https://example.com/investor.md".to_string()]);
    let err = config.validate().expect_err("should reject remote path");
    assert!(err.contains("must be local"));
}
