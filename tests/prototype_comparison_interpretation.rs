use serde_json::Value;
use soma_zero::{
    CommitteeReferenceCoverageAuditStatus, EvidenceWeightedSequenceCoreDecisionStatus,
    PrototypeComparisonConfidenceStatus, PrototypeComparisonInterpretationRunner,
    PrototypeComparisonInterpretationStatus, PrototypeWinnerInterpretationStatus,
};

#[path = "support/sprint69_support.rs"]
mod support;

#[test]
fn sprint81_interpretation_bundle_is_constructed_and_conservative() {
    let bundle = support::run_sprint81_bundle(
        "soma_prototype_interpretation.toml",
        "interpretation-bundle",
    );
    let expected: Value = support::read_json(support::example_path(
        "sprint81_data/expected_interpretation_summary.json",
    ));

    assert_eq!(
        bundle
            .prototype_comparison_interpretation_report
            .interpretation_status,
        PrototypeComparisonInterpretationStatus::PrototypeInterpretationReady
    );
    assert_eq!(
        bundle
            .prototype_comparison_confidence_report
            .confidence_status,
        PrototypeComparisonConfidenceStatus::ConfidenceReady
    );
    assert_eq!(
        bundle
            .prototype_winner_interpretation_gate
            .selected_interpretation,
        PrototypeWinnerInterpretationStatus::MixedOrInconclusive
    );
    assert_eq!(
        bundle.committee_reference_coverage_audit_v2.audit_status,
        CommitteeReferenceCoverageAuditStatus::CommitteeReferenceCoverageReady
    );
    assert_eq!(
        bundle
            .evidence_weighted_sequence_core_decision_gate
            .decision_status,
        EvidenceWeightedSequenceCoreDecisionStatus::KeepBothAsResearchCandidates
    );
    assert!(
        !bundle
            .evidence_weighted_sequence_core_decision_gate
            .runtime_allowed
    );
    assert!(
        !bundle
            .evidence_weighted_sequence_core_decision_gate
            .training_allowed
    );
    assert!(
        !bundle
            .evidence_weighted_sequence_core_decision_gate
            .live_inference_allowed
    );
    assert_eq!(
        expected["interpretation_status"].as_str(),
        Some("PrototypeInterpretationReady")
    );
}

#[test]
fn sprint81_config_defaults_and_remote_guard_hold() {
    let config = support::sprint81_interpretation_config_from_example(
        "soma_prototype_interpretation.toml",
        "interpretation-config",
    );
    assert!(config.require_common_dataset);
    assert!(config.require_committee_reference_coverage);
    assert!(config.require_no_trade_reference);
    assert!(config.require_risk_denied_reference);
    assert!(config.require_training_lineage_integrity);

    let mut remote = config.clone();
    remote.sequence_core_prototype_comparison_paths =
        vec!["https://example.com/prototype.json".to_string()];
    let error = remote.validate().expect_err("remote paths rejected");
    assert!(error.contains("must be local"));
}

#[test]
fn sprint81_weak_official_and_counterfactual_depth_stays_conservative() {
    let weak_population = support::write_support_json(
        "sprint81-weak-population",
        "training.json",
        &serde_json::json!({
            "population_id": "training-data-artifact-population",
            "artifacts_added": 4,
            "registry_entries_added": 0,
            "dataset_versions_added": 0,
            "lineage_edges_added": 0,
            "source_class_entries_added": 0,
            "skipped_artifacts": 0,
            "invalid_artifacts": 0,
            "population_status": "ArtifactsPopulatedWithWarnings",
            "artifact_counts_by_kind": { "prediction_csv": 1, "model_card": 1 },
            "artifact_counts_by_source_class": { "ResearchOnly": 1 }
        }),
    );
    let weak_counterfactual = support::write_support_json(
        "sprint81-weak-counterfactual",
        "counterfactual.json",
        &serde_json::json!({
            "pack_id": "committee-counterfactual-pack",
            "no_trade_counterfactual_count": 0,
            "risk_denied_counterfactual_count": 0,
            "defensive_value_count": 0,
            "opportunity_cost_count": 0,
            "pack_status": "NeedNoTradeCounterfactuals"
        }),
    );
    let weak_scenarios = support::write_support_json(
        "sprint81-weak-scenarios",
        "scenarios.json",
        &serde_json::json!({
            "pack_id": "committee-scenario-pack-v2",
            "scenarios": ["KRX:005930:1d:bull:tp"],
            "official_scenario_count": 0,
            "research_only_count": 0,
            "diagnostic_only_count": 0,
            "fixture_only_count": 1,
            "source_class_summary": { "FixtureOnly": 1 },
            "scenario_status": "DiagnosticOnly"
        }),
    );
    let mut config = support::sprint81_interpretation_config_from_example(
        "soma_prototype_interpretation.toml",
        "interpretation-weak",
    );
    config.training_artifact_population_paths = vec![weak_population];
    config.committee_counterfactual_pack_paths = vec![weak_counterfactual];
    config.committee_scenario_pack_paths = vec![weak_scenarios];

    let bundle = PrototypeComparisonInterpretationRunner::default()
        .run(&config)
        .expect("weak evidence bundle");
    assert_eq!(
        bundle.committee_reference_coverage_audit_v2.audit_status,
        CommitteeReferenceCoverageAuditStatus::NeedMoreOfficialReferences
    );
    assert_eq!(
        bundle
            .evidence_weighted_sequence_core_decision_gate
            .decision_status,
        EvidenceWeightedSequenceCoreDecisionStatus::NeedMoreEvidence
    );
}
