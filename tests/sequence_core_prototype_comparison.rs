#[path = "support/sprint69_support.rs"]
mod support;

use serde_json::Value;
use serde_json::json;
use soma_zero::{
    CommitteeVsSequenceCoreComparisonStatus, SequenceCorePrototypeArtifactRegistryStatus,
    SequenceCorePrototypeCalibrationStatus, SequenceCorePrototypeComparisonRunner,
    SequenceCorePrototypeComparisonStatus, SequenceCorePrototypeFinalRecommendation,
    SequenceCorePrototypePredictionImportStatus, SequenceCorePrototypePromotionStatus,
    SequenceCorePrototypeRiskInteractionStatus, TrainingDataPopulatedIntegrityStatus,
};

#[test]
fn sprint80_bundle_builds_expected_comparison_bundle() {
    let bundle =
        support::run_sprint80_bundle("soma_sequence_core_prototype_compare.toml", "bundle-ready");
    assert_eq!(
        bundle
            .sequence_core_prototype_artifact_registry
            .registry_status,
        SequenceCorePrototypeArtifactRegistryStatus::PrototypeArtifactRegistryReady
    );
    assert_eq!(
        bundle.mamba3fin_prediction_import_report.import_status,
        SequenceCorePrototypePredictionImportStatus::ImportReady
    );
    assert_eq!(
        bundle.gated_deltanet_prediction_import_report.import_status,
        SequenceCorePrototypePredictionImportStatus::ImportReady
    );
    assert_eq!(
        bundle.prototype_comparison_report.comparison_status,
        SequenceCorePrototypeComparisonStatus::Mixed
    );
    assert_eq!(
        bundle.prototype_comparison_report.final_recommendation,
        SequenceCorePrototypeFinalRecommendation::NeedCommitteeComparison
    );
    assert_eq!(
        bundle.prototype_calibration_report.calibration_status,
        SequenceCorePrototypeCalibrationStatus::CalibrationReady
    );
    assert_eq!(
        bundle.prototype_risk_interaction_report.risk_status,
        SequenceCorePrototypeRiskInteractionStatus::RiskInteractionReady
    );
    assert_eq!(
        bundle
            .committee_vs_sequence_core_comparison_report
            .comparison_status,
        CommitteeVsSequenceCoreComparisonStatus::Mixed
    );
    assert_eq!(
        bundle.prototype_promotion_gate.promotion_status,
        SequenceCorePrototypePromotionStatus::ResearchCandidateOnly
    );
    assert_eq!(
        bundle
            .training_data_populated_integrity_report
            .integrity_status,
        TrainingDataPopulatedIntegrityStatus::PopulatedIntegrityReady
    );
    assert!(bundle.final_summary.contains("prototype_comparison_status"));
    assert!(bundle.final_summary.contains("artifact_population_status"));

    let config = support::sprint80_compare_config_from_example(
        "soma_sequence_core_prototype_compare.toml",
        "bundle-ready",
    );
    let output_dir = config.output_dir();
    for file_name in [
        "prototype_artifact_registry.txt",
        "mamba3fin_artifact_spec.txt",
        "gated_deltanet_artifact_spec.txt",
        "mamba3fin_prediction_import.txt",
        "gated_deltanet_prediction_import.txt",
        "mamba3fin_evaluation.txt",
        "gated_deltanet_evaluation.txt",
        "prototype_comparison.txt",
        "prototype_calibration.txt",
        "prototype_risk_interaction.txt",
        "prototype_ablation.txt",
        "prototype_promotion_gate.txt",
        "committee_scenario_pack_v2.txt",
        "committee_outcome_reference_pack_v2.txt",
        "committee_counterfactual_reference_pack.txt",
        "committee_vs_sequence_core_comparison.txt",
        "training_data_artifact_population.txt",
        "training_data_populated_integrity.txt",
        "control_tower_sequence_prototype_panel.txt",
        "storage_report.txt",
        "summary.txt",
    ] {
        assert!(output_dir.join(file_name).exists(), "{file_name}");
    }
}

#[test]
fn sprint80_config_defaults_and_remote_rejection_hold() {
    let config = support::sprint80_compare_config_from_example(
        "soma_sequence_core_prototype_compare.toml",
        "cfg",
    );
    assert!(config.require_common_dataset);
    assert!(config.require_common_feature_schema);
    assert!(config.require_common_label_manifest);
    assert!(config.require_common_split);
    assert!(config.require_model_cards);
    assert!(config.require_prediction_schema);

    let mut remote = config.clone();
    remote.output_root = "https://example.com/out".to_string();
    assert!(remote.validate().unwrap_err().contains("must be local"));
}

#[test]
fn sprint80_import_and_registry_failures_are_specific() {
    let unknown_csv = support::write_support_text(
        "sprint80-unknown-seq",
        "mamba.csv",
        "sequence_id,confidence,expected_return,expected_drawdown,p_stop_loss,p_take_profit,p_time_expired,rank_score\nunknown-seq,0.9,0.1,0.02,0.1,0.6,0.3,0.9\n",
    );
    let mut unknown = support::sprint80_compare_config_from_example(
        "soma_sequence_core_prototype_compare.toml",
        "unknown-seq",
    );
    unknown.mamba3fin_prediction_csv_paths = vec![unknown_csv];
    assert_eq!(
        SequenceCorePrototypeComparisonRunner::default()
            .run_mamba3fin_prototype_import(&unknown)
            .expect("unknown import")
            .import_status,
        SequenceCorePrototypePredictionImportStatus::UnknownSequenceIds
    );

    let duplicate_csv = support::write_support_text(
        "sprint80-duplicate-seq",
        "mamba.csv",
        "sequence_id,confidence,expected_return,expected_drawdown,p_stop_loss,p_take_profit,p_time_expired,rank_score\nseq-001,0.9,0.1,0.02,0.1,0.6,0.3,0.9\nseq-001,0.8,0.1,0.02,0.1,0.6,0.3,0.8\n",
    );
    let mut duplicate = support::sprint80_compare_config_from_example(
        "soma_sequence_core_prototype_compare.toml",
        "duplicate-seq",
    );
    duplicate.mamba3fin_prediction_csv_paths = vec![duplicate_csv];
    assert_eq!(
        SequenceCorePrototypeComparisonRunner::default()
            .run_mamba3fin_prototype_import(&duplicate)
            .expect("duplicate import")
            .import_status,
        SequenceCorePrototypePredictionImportStatus::DuplicatePredictions
    );

    let invalid_csv = support::write_support_text(
        "sprint80-invalid-prob",
        "gated.csv",
        "sequence_id,confidence,expected_return,expected_drawdown,p_stop_loss,p_take_profit,p_time_expired,rank_score\nseq-001,1.4,0.1,0.02,0.1,0.6,0.3,0.9\n",
    );
    let mut invalid = support::sprint80_compare_config_from_example(
        "soma_sequence_core_prototype_compare.toml",
        "invalid-prob",
    );
    invalid.gated_deltanet_prediction_csv_paths = vec![invalid_csv];
    assert_eq!(
        SequenceCorePrototypeComparisonRunner::default()
            .run_gated_deltanet_prototype_import(&invalid)
            .expect("invalid import")
            .import_status,
        SequenceCorePrototypePredictionImportStatus::InvalidPredictions
    );

    let gated_override = support::write_support_json(
        "sprint80-gated-contract-missing",
        "gated.json",
        &json!({
            "gated_deltanet_model_card": {
                "model_id": "gated-deltanet-prototype",
                "model_version": "v0.1",
                "family": "GatedDeltaNet",
                "prediction_schema_version": "prediction_csv_v1",
                "feature_schema_hash": "0b33c49d8b0f9d95d26cbd194ae66f6a6fbee6d6a4f20f2a0b34d6f5c6437776",
                "label_manifest_hash": "6bc22a91864555a5f6f87b1146dce7d74f7cfa4924ca972401fdcefd5aca3ec8",
                "dataset_version": "sprint80-dataset-v1",
                "split_policy": "walk_forward_v1",
                "no_lookahead_proof_ref": "proof:no-lookahead:sprint80",
                "fields": ["model_id", "model_version", "feature_schema_hash", "label_manifest_hash", "live_use_forbidden"],
                "live_use_forbidden": true,
                "gated_delta_contract_ref": "",
                "state_spec_ref": ""
            }
        }),
    );
    let mut missing_contract = support::sprint80_compare_config_from_example(
        "soma_sequence_core_prototype_compare.toml",
        "gated-contract-missing",
    );
    missing_contract.gated_deltanet_model_card_paths = vec![gated_override];
    let bundle = SequenceCorePrototypeComparisonRunner::default()
        .run(&missing_contract)
        .expect("bundle with missing contract ref");
    assert_eq!(
        bundle
            .sequence_core_prototype_artifact_registry
            .registry_status,
        SequenceCorePrototypeArtifactRegistryStatus::ContractMismatch
    );
}

#[test]
fn sprint80_population_writes_reference_manifests() {
    let population = support::sprint80_population_config_from_example(
        "soma_training_artifact_populate.toml",
        "population-manifests",
    );
    let report = SequenceCorePrototypeComparisonRunner::default()
        .run_training_artifact_populate(&population)
        .expect("population");
    assert!(report.artifacts_added >= 4);

    let manifest: Value = support::read_json(
        population
            .storage_dir()
            .join("registry")
            .join("dataset_registry.json"),
    );
    assert_eq!(
        manifest
            .get("data_available")
            .and_then(|value| value.as_bool()),
        Some(false)
    );
    assert!(
        manifest
            .get("artifacts")
            .and_then(|value| value.as_array())
            .is_some()
    );
}
