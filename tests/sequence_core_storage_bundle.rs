#[path = "support/sprint69_support.rs"]
mod support;

use serde_json::Value;
use serde_json::json;
use soma_zero::{
    ControlTowerSequenceCorePanel, GatedDeltaNetDataDependencyStatus,
    GatedDeltaNetRuntimeReadinessStatus, SequenceCoreCandidateComparisonStatus,
    SequenceCoreCandidateFamily, SequenceCoreCandidateRegistryConfig,
    SequenceCoreCandidateRegistryStatus, SequenceCoreExternalPrototypeBackend,
    SequenceCoreExternalPrototypeContractStatus, SequenceCoreStorageMaterializationRunner,
    StorageArtifactPolicy, TrainingDataStorageIntegrityStatus,
    TrainingDataStorageMaterializationConfig, TrainingDataStorageMaterializationStatus,
};

#[test]
fn sprint79_bundle_builds_expected_registry_storage_and_panel() {
    let bundle = support::run_sprint79_bundle(
        "soma_sequence_core_registry.toml",
        "soma_training_storage_materialize.toml",
        "bundle-ready",
    );
    assert_eq!(
        bundle.sequence_core_candidate_registry.registry_status,
        SequenceCoreCandidateRegistryStatus::SequenceCoreRegistryReady
    );
    assert_eq!(bundle.sequence_core_candidate_registry.candidate_count, 2);
    assert_eq!(
        bundle
            .sequence_core_candidate_registry
            .neural_candidate_count,
        2
    );
    assert_eq!(
        bundle
            .sequence_core_candidate_registry
            .runtime_deferred_count,
        2
    );
    assert!(
        bundle
            .sequence_core_candidate_registry
            .candidates
            .iter()
            .any(|candidate| candidate.family == SequenceCoreCandidateFamily::Mamba3Fin)
    );
    assert!(
        bundle
            .sequence_core_candidate_registry
            .candidates
            .iter()
            .any(|candidate| candidate.family == SequenceCoreCandidateFamily::GatedDeltaNet)
    );

    assert!(
        bundle
            .gated_deltanet_core_contract
            .projection_spec
            .q_projection_required
    );
    assert!(
        bundle
            .gated_deltanet_core_contract
            .projection_spec
            .k_projection_required
    );
    assert!(
        bundle
            .gated_deltanet_core_contract
            .projection_spec
            .v_projection_required
    );
    assert!(
        bundle
            .gated_deltanet_core_contract
            .gated_delta_rule_spec
            .decay_gate_required
    );
    assert!(
        bundle
            .gated_deltanet_core_contract
            .target_heads
            .contains(&"p_take_profit".to_string())
    );
    assert!(
        bundle
            .gated_deltanet_core_contract
            .target_heads
            .contains(&"expected_return".to_string())
    );

    assert_eq!(
        bundle
            .gated_deltanet_runtime_readiness_gate
            .readiness_status,
        GatedDeltaNetRuntimeReadinessStatus::ReadyForExternalPrototypeOnly
    );
    assert_eq!(
        bundle
            .gated_deltanet_data_dependency_report
            .dependency_status,
        GatedDeltaNetDataDependencyStatus::DependenciesReady
    );
    assert_eq!(
        bundle
            .sequence_core_candidate_comparison_plan
            .comparison_status,
        SequenceCoreCandidateComparisonStatus::ComparisonPlanReady
    );
    assert!(
        bundle
            .sequence_core_candidate_comparison_plan
            .metrics
            .contains(&"no_trade_comparison".to_string())
    );
    assert!(
        bundle
            .sequence_core_candidate_comparison_plan
            .metrics
            .contains(&"risk_denied_defensive_value".to_string())
    );
    assert_eq!(
        bundle
            .sequence_core_external_prototype_contract
            .allowed_backend,
        SequenceCoreExternalPrototypeBackend::PredictionCsvOnly
    );
    assert_eq!(
        bundle
            .sequence_core_external_prototype_contract
            .contract_status,
        SequenceCoreExternalPrototypeContractStatus::ContractReady
    );

    assert_eq!(
        bundle
            .training_data_storage_materialization_report
            .materialization_status,
        TrainingDataStorageMaterializationStatus::StorageMaterialized
    );
    assert_eq!(
        bundle
            .training_data_storage_materialization_report
            .written_manifests
            .len(),
        6
    );
    assert_eq!(
        bundle
            .training_data_storage_integrity_check
            .integrity_status,
        TrainingDataStorageIntegrityStatus::StorageIntegrityReady
    );
    assert_eq!(bundle.model_family_storage_contracts.len(), 5);
    assert!(
        bundle
            .model_family_storage_contracts
            .iter()
            .any(|contract| {
                contract.family == SequenceCoreCandidateFamily::Mamba3Fin
                    && contract.runtime_artifact_policy == StorageArtifactPolicy::Deferred
            })
    );
    assert!(
        bundle
            .model_family_storage_contracts
            .iter()
            .any(|contract| {
                contract.family == SequenceCoreCandidateFamily::ExternalTabular
                    && contract.runtime_artifact_policy == StorageArtifactPolicy::ExternalOnly
            })
    );

    let panel: &ControlTowerSequenceCorePanel = &bundle.control_tower_sequence_core_panel;
    assert_eq!(
        panel.candidate_registry_status,
        SequenceCoreCandidateRegistryStatus::SequenceCoreRegistryReady
    );
    assert_eq!(
        panel.storage_materialization_status,
        TrainingDataStorageMaterializationStatus::StorageMaterialized
    );
    assert_eq!(
        panel.storage_integrity_status,
        TrainingDataStorageIntegrityStatus::StorageIntegrityReady
    );
    assert!(panel.runtime_deferred_summary.contains("deferred"));

    let materialization_config = support::sprint79_materialization_config_from_example(
        "soma_training_storage_materialize.toml",
        "bundle-ready",
    );
    let output_dir = materialization_config.output_dir();
    for file_name in [
        "sequence_core_candidate_registry.txt",
        "gated_deltanet_core_contract.txt",
        "gated_deltanet_runtime_readiness_gate.txt",
        "gated_deltanet_data_dependency.txt",
        "sequence_core_candidate_comparison_plan.txt",
        "sequence_core_external_prototype_contract.txt",
        "training_data_storage_materialization.txt",
        "training_data_storage_integrity_check.txt",
        "model_family_storage_contracts.txt",
        "control_tower_sequence_core_panel.txt",
        "storage_report.txt",
        "summary.txt",
    ] {
        assert!(output_dir.join(file_name).exists(), "{file_name}");
    }

    let placeholder: Value = support::read_json(
        materialization_config
            .storage_dir()
            .join("registry")
            .join("dataset_registry.json"),
    );
    assert_eq!(
        placeholder
            .get("data_available")
            .and_then(|value| value.as_bool()),
        Some(false)
    );
    assert_eq!(
        placeholder
            .get("placeholder")
            .and_then(|value| value.as_bool()),
        Some(true)
    );
}

#[test]
fn sprint79_configs_construct_and_reject_remote_paths() {
    let registry = support::sprint79_registry_config_from_example(
        "soma_sequence_core_registry.toml",
        "cfg-registry",
    );
    assert!(registry.require_mamba3fin);
    assert!(registry.require_gated_deltanet);
    assert!(registry.require_common_input_spec);
    assert!(registry.require_common_prediction_heads);
    assert!(registry.require_risk_integration);

    let mut remote_registry = SequenceCoreCandidateRegistryConfig::default();
    remote_registry.registry_id = "remote-registry".to_string();
    remote_registry.output_root = "https://example.com/out".to_string();
    assert!(
        remote_registry
            .validate()
            .unwrap_err()
            .contains("must be local")
    );

    let materialization = support::sprint79_materialization_config_from_example(
        "soma_training_storage_materialize.toml",
        "cfg-materialization",
    );
    assert!(materialization.create_directories);
    assert!(materialization.write_placeholder_manifests);

    let mut remote_materialization = TrainingDataStorageMaterializationConfig::default();
    remote_materialization.materialization_id = "remote-materialization".to_string();
    remote_materialization.output_root = "https://example.com/out".to_string();
    assert!(
        remote_materialization
            .validate()
            .unwrap_err()
            .contains("must be local")
    );
}

#[test]
fn sprint79_registry_and_dependency_failures_are_specific() {
    let mismatch_override = support::write_support_json(
        "sprint79-registry-mismatch",
        "override.json",
        &json!({
            "gated_deltanet_contract_expected": {
                "input_tensor_spec": "mismatched gated tensor spec"
            }
        }),
    );
    let mut registry = support::sprint79_registry_config_from_example(
        "soma_sequence_core_registry.toml",
        "registry-mismatch",
    );
    registry
        .gated_deltanet_contract_paths
        .push(mismatch_override);
    let materialization = support::sprint79_materialization_config_from_example(
        "soma_training_storage_materialize.toml",
        "registry-mismatch",
    );
    let bundle = SequenceCoreStorageMaterializationRunner::default()
        .run(&registry, &materialization)
        .expect("run mismatch bundle");
    assert_eq!(
        bundle.sequence_core_candidate_registry.registry_status,
        SequenceCoreCandidateRegistryStatus::InputSpecMismatch
    );
    assert_eq!(
        bundle
            .gated_deltanet_data_dependency_report
            .dependency_status,
        GatedDeltaNetDataDependencyStatus::MissingCandidateRegistry
    );

    let mut missing_gated = support::sprint79_registry_config_from_example(
        "soma_sequence_core_registry.toml",
        "registry-missing-gated",
    );
    missing_gated.gated_deltanet_contract_paths.clear();
    assert_eq!(
        SequenceCoreStorageMaterializationRunner::default()
            .run_sequence_core_registry(&missing_gated)
            .expect("missing gated")
            .registry_status,
        SequenceCoreCandidateRegistryStatus::MissingGatedDeltaNet
    );

    let mut missing_mamba = support::sprint79_registry_config_from_example(
        "soma_sequence_core_registry.toml",
        "registry-missing-mamba",
    );
    missing_mamba.mamba3fin_contract_paths.clear();
    assert_eq!(
        SequenceCoreStorageMaterializationRunner::default()
            .run_sequence_core_registry(&missing_mamba)
            .expect("missing mamba")
            .registry_status,
        SequenceCoreCandidateRegistryStatus::MissingMamba3Fin
    );
}

#[test]
fn sprint79_readiness_and_comparison_failures_are_specific() {
    let runner = SequenceCoreStorageMaterializationRunner::default();

    let training_override = support::write_support_json(
        "sprint79-readiness-training",
        "override.json",
        &json!({
            "training_data_registry_spec": {
                "training_storage_manifest_present": false
            }
        }),
    );
    let mut training_config = support::sprint79_registry_config_from_example(
        "soma_gated_deltanet_readiness.toml",
        "readiness-training",
    );
    training_config
        .training_data_storage_paths
        .push(training_override);
    assert_eq!(
        runner
            .run_gated_deltanet_readiness(&training_config)
            .expect("training blocked")
            .readiness_status,
        GatedDeltaNetRuntimeReadinessStatus::BlockedByTrainingDataStorage
    );

    let evidence_override = support::write_support_json(
        "sprint79-readiness-evidence",
        "override.json",
        &json!({
            "committee_gate_state": {
                "evidence_depth_ready": false
            }
        }),
    );
    let mut evidence_config = support::sprint79_registry_config_from_example(
        "soma_gated_deltanet_readiness.toml",
        "readiness-evidence",
    );
    evidence_config.committee_gate_paths.push(evidence_override);
    assert_eq!(
        runner
            .run_gated_deltanet_readiness(&evidence_config)
            .expect("evidence blocked")
            .readiness_status,
        GatedDeltaNetRuntimeReadinessStatus::BlockedByEvidenceDepth
    );

    let comparison_override = support::write_support_json(
        "sprint79-comparison-dataset",
        "override.json",
        &json!({
            "sequence_core_registry_expected": {
                "common_dataset_ready": false
            }
        }),
    );
    let mut comparison_config = support::sprint79_registry_config_from_example(
        "soma_sequence_core_comparison_plan.toml",
        "comparison-dataset",
    );
    comparison_config
        .mamba3fin_contract_paths
        .push(comparison_override);
    assert_eq!(
        runner
            .run_sequence_core_comparison_plan(&comparison_config)
            .expect("comparison plan")
            .comparison_status,
        SequenceCoreCandidateComparisonStatus::NeedCommonDataset
    );

    let external_override = support::write_support_json(
        "sprint79-external-dataset",
        "override.json",
        &json!({
            "training_data_registry_spec": {
                "dataset_registry_present": false
            }
        }),
    );
    let mut external_config = support::sprint79_registry_config_from_example(
        "soma_sequence_core_external_contract.toml",
        "external-dataset",
    );
    external_config
        .training_data_storage_paths
        .push(external_override);
    assert_eq!(
        runner
            .run_sequence_core_external_contract(&external_config)
            .expect("external contract")
            .contract_status,
        SequenceCoreExternalPrototypeContractStatus::MissingDatasetRegistry
    );
}

#[test]
fn sprint79_storage_integrity_detects_missing_manifests() {
    let materialization = support::sprint79_materialization_config_from_example(
        "soma_training_storage_materialize.toml",
        "integrity-missing",
    );
    SequenceCoreStorageMaterializationRunner::default()
        .run_training_storage_materialize(&materialization)
        .expect("materialize first");
    std::fs::remove_file(
        materialization
            .storage_dir()
            .join("registry")
            .join("lineage_manifest.json"),
    )
    .expect("remove manifest");
    assert_eq!(
        SequenceCoreStorageMaterializationRunner::default()
            .run_training_storage_integrity(&materialization)
            .expect("integrity")
            .integrity_status,
        TrainingDataStorageIntegrityStatus::MissingManifests
    );
}
