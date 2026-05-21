#[path = "support/sprint69_support.rs"]
mod support;

use serde_json::json;
use soma_zero::{
    CommitteeChairRiskCompletionStatus, CommitteeCompletionGateStatus,
    CoreCommitteeMambaReadinessRunner, CoreCompletionV2Config, CoreCompletionV2Status,
    DatasetVersionScheme, Mamba3FinDataDependencyStatus, Mamba3FinRuntimeReadinessStatus,
    Mamba3FinTargetHead, StorageFormatCandidate, TrainingDataStorageConfig,
    TrainingDataStorageDecisionStatus,
};

#[test]
fn sprint78_bundle_builds_expected_contracts_and_outputs() {
    let config = support::sprint78_core_config_from_example(
        "soma_core_completion_v2.toml",
        "bundle-contracts",
    );
    let output_dir = config.output_dir();
    let bundle = CoreCommitteeMambaReadinessRunner::default()
        .run(&config)
        .expect("run sprint78 bundle");

    assert_eq!(
        bundle.core_completion_v2_report.status,
        CoreCompletionV2Status::CoreContractFrozen
    );
    assert!(bundle.core_completion_v2_report.research_core_ready);
    assert!(!bundle.core_completion_v2_report.live_core_ready);
    assert_eq!(
        bundle.committee_completion_gate_report.gate_status,
        CommitteeCompletionGateStatus::CommitteeCoreReady
    );
    assert_eq!(
        bundle
            .committee_completion_gate_report
            .active_personas
            .len(),
        3
    );
    assert_eq!(
        bundle.mamba3fin_runtime_readiness_gate.readiness_status,
        Mamba3FinRuntimeReadinessStatus::ReadyForStorageImplementation
    );
    assert_eq!(
        bundle.training_data_storage_decision_report.selected_now,
        StorageFormatCandidate::CsvJsonManifest
    );
    assert_eq!(
        bundle.training_data_storage_decision_report.decision_status,
        TrainingDataStorageDecisionStatus::CsvJsonManifestSelected
    );
    assert_eq!(bundle.mamba3fin_core_contract.model_family, "Mamba3Fin");
    assert!(
        bundle
            .mamba3fin_core_contract
            .target_heads
            .contains(&Mamba3FinTargetHead::PTakeProfit)
    );
    assert!(
        bundle
            .mamba3fin_core_contract
            .target_heads
            .contains(&Mamba3FinTargetHead::PStopLoss)
    );
    assert!(
        bundle
            .mamba3fin_core_contract
            .target_heads
            .contains(&Mamba3FinTargetHead::ExpectedReturn)
    );
    assert!(
        bundle
            .mamba3fin_core_contract
            .target_heads
            .contains(&Mamba3FinTargetHead::ExpectedDrawdown)
    );
    assert!(bundle.mamba3fin_core_contract.risk_integration_required);
    assert!(
        bundle
            .feature_label_storage_contract
            .feature_schema_manifest_required
    );
    assert!(
        bundle
            .feature_label_storage_contract
            .no_post_hoc_label_tuning
    );
    assert!(bundle.sequence_training_data_contract.no_random_split);
    assert!(bundle.sequence_training_data_contract.no_lookahead_required);
    assert!(
        bundle
            .prediction_model_card_storage_contract
            .live_use_forbidden_required
    );
    assert_eq!(
        bundle.dataset_versioning_policy.selected_scheme,
        DatasetVersionScheme::DateHash
    );
    assert!(
        bundle
            .training_data_layout_plan
            .directories
            .contains(&"data/raw/".to_string())
    );
    assert!(
        bundle
            .training_data_layout_plan
            .directories
            .contains(&"data/model_cards/".to_string())
    );
    assert!(
        !bundle
            .training_data_lineage_spec
            .lineage_fingerprint
            .is_empty()
    );
    assert_eq!(
        bundle
            .control_tower_core_mamba_data_panel
            .storage_format_decision,
        StorageFormatCandidate::CsvJsonManifest
    );
    assert!(
        bundle
            .control_tower_core_mamba_data_panel
            .deferred_items
            .iter()
            .any(|item| item.contains("runtime"))
    );
    assert!(bundle.final_summary.contains("training_storage_status"));
    assert!(bundle.final_summary.contains("runtime"));

    for file_name in [
        "core_completion_v2.txt",
        "mamba3fin_core_contract.txt",
        "mamba3fin_runtime_readiness_gate.txt",
        "mamba3fin_data_dependency.txt",
        "committee_completion_gate.txt",
        "committee_materialization_plan_v2.txt",
        "committee_chair_risk_completion.txt",
        "training_data_storage_decision.txt",
        "training_data_registry_spec.txt",
        "training_data_layout_plan.txt",
        "training_data_lineage_spec.txt",
        "dataset_versioning_policy.txt",
        "feature_label_storage_contract.txt",
        "sequence_training_data_contract.txt",
        "prediction_model_card_storage_contract.txt",
        "storage_format_decision.txt",
        "mamba3_implementation_roadmap.txt",
        "control_tower_core_mamba_data_panel.txt",
        "storage_report.txt",
        "summary.txt",
    ] {
        assert!(output_dir.join(file_name).exists(), "{file_name}");
    }
}

#[test]
fn sprint78_configs_are_constructible_and_reject_remote_paths() {
    let core =
        support::sprint78_core_config_from_example("soma_core_completion_v2.toml", "cfg-core");
    assert_eq!(core.core_id, "sprint78-core");
    assert!(!core.core_check_paths.is_empty());

    let mut remote_core = CoreCompletionV2Config::default();
    remote_core.core_id = "remote-core".to_string();
    remote_core.output_root = "https://example.com/out".to_string();
    assert!(
        remote_core
            .validate()
            .unwrap_err()
            .contains("must be local")
    );

    let mut committee = support::sprint78_committee_config_from_example(
        "soma_committee_completion_gate.toml",
        "cfg-committee",
    );
    committee.committee_v1_paths = vec!["https://example.com/committee.json".to_string()];
    assert!(committee.validate().unwrap_err().contains("must be local"));

    let training = support::sprint78_training_config_from_example(
        "soma_training_data_storage_decision.toml",
        "cfg-training",
    );
    assert_eq!(
        training.preferred_primary_format,
        StorageFormatCandidate::CsvJsonManifest
    );
    assert!(!training.allow_parquet);
    assert!(!training.allow_arrow);
    assert!(training.require_source_class);
    assert!(training.require_provenance_ref);
    assert!(training.require_preflight_ref);
    assert!(training.require_no_lookahead_proof);

    let mut remote_training = TrainingDataStorageConfig::default();
    remote_training.storage_id = "remote-training".to_string();
    remote_training.output_root = "https://example.com/storage".to_string();
    assert!(
        remote_training
            .validate()
            .unwrap_err()
            .contains("must be local")
    );
}

#[test]
fn sprint78_core_runtime_and_dependency_blockers_are_specific() {
    let override_path = support::write_support_json(
        "sprint78-core-blockers",
        "override.json",
        &json!({
            "core_current_state": {
                "training_storage_contract_ready": false
            },
            "sequence_export_state": {
                "training_data_storage_manifest_present": false,
                "feature_schema_manifest_present": false,
                "sequence_export_manifest_present": false,
                "no_lookahead_proof_present": false
            }
        }),
    );
    let mut config =
        support::sprint78_core_config_from_example("soma_core_completion_v2.toml", "core-blockers");
    config.core_check_paths.push(override_path.clone());
    config.sequence_export_paths.push(override_path);

    let bundle = CoreCommitteeMambaReadinessRunner::default()
        .run(&config)
        .expect("run blocked bundle");
    assert_eq!(
        bundle.core_completion_v2_report.status,
        CoreCompletionV2Status::CoreNeedsTrainingDataContract
    );
    assert_eq!(
        bundle.mamba3fin_runtime_readiness_gate.readiness_status,
        Mamba3FinRuntimeReadinessStatus::BlockedByTrainingDataStorage
    );
    assert_eq!(
        bundle.mamba3fin_data_dependency_report.dependency_status,
        Mamba3FinDataDependencyStatus::MissingTrainingStorage
    );
    assert!(
        bundle
            .mamba3fin_data_dependency_report
            .missing_artifacts
            .contains(&"feature_schema_manifest".to_string())
    );
    assert!(
        bundle
            .mamba3fin_data_dependency_report
            .missing_artifacts
            .contains(&"sequence_export_manifest".to_string())
    );
    assert!(
        bundle
            .mamba3fin_data_dependency_report
            .missing_artifacts
            .contains(&"no_lookahead_proof".to_string())
    );
}

#[test]
fn sprint78_runtime_blockers_cover_evidence_committee_and_budget() {
    let runner = CoreCommitteeMambaReadinessRunner::default();

    let evidence_override = support::write_support_json(
        "sprint78-runtime-evidence",
        "override.json",
        &json!({
            "real_evidence_state": {
                "evidence_depth_ready": false
            }
        }),
    );
    let mut evidence_config = support::sprint78_core_config_from_example(
        "soma_core_completion_v2.toml",
        "runtime-evidence",
    );
    evidence_config.real_evidence_paths.push(evidence_override);
    assert_eq!(
        runner
            .run_mamba3_runtime_readiness(&evidence_config)
            .expect("runtime evidence")
            .readiness_status,
        Mamba3FinRuntimeReadinessStatus::BlockedByEvidenceDepth
    );

    let committee_override = support::write_support_json(
        "sprint78-runtime-committee",
        "override.json",
        &json!({
            "model_ops_state": {
                "committee_comparison_ready": false
            }
        }),
    );
    let mut committee_config = support::sprint78_core_config_from_example(
        "soma_core_completion_v2.toml",
        "runtime-committee",
    );
    committee_config.model_ops_paths.push(committee_override);
    assert_eq!(
        runner
            .run_mamba3_runtime_readiness(&committee_config)
            .expect("runtime committee")
            .readiness_status,
        Mamba3FinRuntimeReadinessStatus::BlockedByCommitteeGate
    );

    let budget_override = support::write_support_json(
        "sprint78-runtime-budget",
        "override.json",
        &json!({
            "model_ops_state": {
                "test_budget_ready": false
            }
        }),
    );
    let mut budget_config = support::sprint78_core_config_from_example(
        "soma_core_completion_v2.toml",
        "runtime-budget",
    );
    budget_config.model_ops_paths.push(budget_override);
    assert_eq!(
        runner
            .run_mamba3_runtime_readiness(&budget_config)
            .expect("runtime budget")
            .readiness_status,
        Mamba3FinRuntimeReadinessStatus::BlockedByTestBudget
    );
}

#[test]
fn sprint78_committee_gate_and_chair_risk_failures_are_specific() {
    let runner = CoreCommitteeMambaReadinessRunner::default();

    let expansion_override = support::write_support_json(
        "sprint78-committee-expansion",
        "override.json",
        &json!({
            "committee_v1_state": {
                "active_personas": ["a", "b", "c", "d"],
                "official_scenario_count": 12,
                "outcome_linked_rows": 256
            }
        }),
    );
    let mut expansion_config = support::sprint78_committee_config_from_example(
        "soma_committee_completion_gate.toml",
        "committee-expansion",
    );
    expansion_config.committee_v1_paths.push(expansion_override);
    assert_eq!(
        runner
            .run_committee_completion_gate(&expansion_config)
            .expect("committee expansion")
            .gate_status,
        CommitteeCompletionGateStatus::CommitteeResearchOnly
    );

    let official_override = support::write_support_json(
        "sprint78-committee-official",
        "override.json",
        &json!({
            "committee_v1_state": {
                "official_scenario_count": 0,
                "outcome_linked_rows": 256
            }
        }),
    );
    let mut official_config = support::sprint78_committee_config_from_example(
        "soma_committee_completion_gate.toml",
        "committee-official",
    );
    official_config.committee_v1_paths.push(official_override);
    assert_eq!(
        runner
            .run_committee_completion_gate(&official_config)
            .expect("committee official")
            .gate_status,
        CommitteeCompletionGateStatus::CommitteeNeedsOfficialScenarios
    );

    let outcome_override = support::write_support_json(
        "sprint78-committee-outcome",
        "override.json",
        &json!({
            "committee_v1_state": {
                "official_scenario_count": 12,
                "outcome_linked_rows": 0
            }
        }),
    );
    let mut outcome_config = support::sprint78_committee_config_from_example(
        "soma_committee_completion_gate.toml",
        "committee-outcome",
    );
    outcome_config.committee_v1_paths.push(outcome_override);
    assert_eq!(
        runner
            .run_committee_completion_gate(&outcome_config)
            .expect("committee outcome")
            .gate_status,
        CommitteeCompletionGateStatus::CommitteeNeedsOutcomeLinks
    );

    let risk_override = support::write_support_json(
        "sprint78-committee-risk",
        "override.json",
        &json!({
            "committee_v1_state": {
                "official_scenario_count": 12,
                "outcome_linked_rows": 256,
                "chair_ready": false,
                "risk_handoff_ready": false,
                "speaker_trace_ready": false,
                "filter_trace_ready": false,
                "groupthink_guard_ready": false
            }
        }),
    );
    let mut risk_config = support::sprint78_committee_config_from_example(
        "soma_committee_completion_gate.toml",
        "committee-risk",
    );
    risk_config.committee_v1_paths.push(risk_override);
    assert_eq!(
        runner
            .run_committee_completion_gate(&risk_config)
            .expect("committee risk")
            .gate_status,
        CommitteeCompletionGateStatus::CommitteeNeedsChairRiskClosure
    );
    assert_eq!(
        runner
            .run_committee_materialization_plan_v2(&risk_config)
            .expect("committee materialization")
            .plan_status,
        soma_zero::CommitteeMaterializationPlanV2Status::MaterializationPlanReady
    );
    assert_eq!(
        runner
            .run_committee_materialization_plan_v2(
                &support::sprint78_committee_config_from_example(
                    "soma_committee_materialization_plan_v2.toml",
                    "committee-materialization-ready",
                )
            )
            .expect("committee materialization ready")
            .planned_artifacts
            .len(),
        6
    );
    assert_eq!(
        runner
            .run_committee_materialization_plan_v2(
                &support::sprint78_committee_config_from_example(
                    "soma_committee_materialization_plan_v2.toml",
                    "committee-materialization-default",
                )
            )
            .expect("committee materialization default")
            .plan_status,
        soma_zero::CommitteeMaterializationPlanV2Status::MaterializationPlanReady
    );
    assert_eq!(
        runner
            .run_committee_completion_gate(&support::sprint78_committee_config_from_example(
                "soma_committee_completion_gate.toml",
                "committee-default",
            ))
            .expect("committee default")
            .gate_status,
        CommitteeCompletionGateStatus::CommitteeCoreReady
    );
    assert_eq!(
        runner
            .run_committee_materialization_plan_v2(
                &support::sprint78_committee_config_from_example(
                    "soma_committee_materialization_plan_v2.toml",
                    "committee-default-plan",
                )
            )
            .expect("committee default plan")
            .plan_status,
        soma_zero::CommitteeMaterializationPlanV2Status::MaterializationPlanReady
    );
    assert_eq!(
        runner
            .run_committee_completion_gate(&support::sprint78_committee_config_from_example(
                "soma_committee_completion_gate.toml",
                "committee-default-two",
            ))
            .expect("committee default two")
            .exactly_three_active,
        true
    );
    assert_eq!(
        runner
            .run_committee_completion_gate(&support::sprint78_committee_config_from_example(
                "soma_committee_completion_gate.toml",
                "committee-default-three",
            ))
            .expect("committee default three")
            .official_scenario_count,
        12
    );
    assert_eq!(
        runner
            .run_committee_completion_gate(&support::sprint78_committee_config_from_example(
                "soma_committee_completion_gate.toml",
                "committee-default-four",
            ))
            .expect("committee default four")
            .outcome_linked_rows,
        256
    );
    assert_eq!(
        runner
            .run_committee_materialization_plan_v2(&risk_config)
            .expect("committee materialization risk")
            .missing_materialization_steps
            .contains(&"refresh no-trade counterfactual pack".to_string()),
        true
    );
    assert_eq!(
        runner
            .run_committee_materialization_plan_v2(&risk_config)
            .expect("committee materialization risk again")
            .missing_materialization_steps
            .contains(&"refresh risk-denied counterfactual pack".to_string()),
        true
    );
    assert_eq!(
        runner
            .run_committee_completion_gate(&risk_config)
            .expect("committee risk again")
            .final_recommendation,
        "NeedChairRiskClosure"
    );

    let chair_risk = CoreCommitteeMambaReadinessRunner::default()
        .run(&{
            let mut config = support::sprint78_core_config_from_example(
                "soma_core_completion_v2.toml",
                "chair-risk-bundle",
            );
            config.committee_v1_paths.push(support::write_support_json(
                "sprint78-chair-risk",
                "override.json",
                &json!({
                    "committee_v1_state": {
                        "speaker_trace_ready": false,
                        "filter_trace_ready": false,
                        "groupthink_guard_ready": false
                    }
                }),
            ));
            config
        })
        .expect("chair risk bundle");
    assert_eq!(
        chair_risk
            .committee_chair_risk_completion_report
            .completion_status,
        CommitteeChairRiskCompletionStatus::NeedChairTrace
    );
}

#[test]
fn sprint78_training_registry_lineage_and_panel_are_wired() {
    let runner = CoreCommitteeMambaReadinessRunner::default();
    let training = support::sprint78_training_config_from_example(
        "soma_training_data_storage_decision.toml",
        "training-contracts",
    );
    let decision = runner
        .run_training_data_storage_decision(&training)
        .expect("training decision");
    assert_eq!(
        decision.decision_status,
        TrainingDataStorageDecisionStatus::CsvJsonManifestSelected
    );
    assert_eq!(
        decision.selected_later,
        Some(StorageFormatCandidate::ParquetJsonManifest)
    );
    assert!(
        decision
            .rejected_candidates
            .contains(&StorageFormatCandidate::ArrowIpcJsonManifest)
    );

    let registry = runner
        .run_training_data_registry_spec(&training)
        .expect("training registry");
    assert!(registry.dataset_id.contains("sprint78-storage"));
    assert!(!registry.dataset_version.is_empty());
    assert!(!registry.feature_schema_hash.is_empty());
    assert!(!registry.label_manifest_hash.is_empty());
    assert_eq!(
        registry.source_class,
        "ResearchOnlyPrimaryWithOfficialEvidenceRefs"
    );

    let layout = runner
        .run_training_data_layout_plan(&support::sprint78_training_config_from_example(
            "soma_training_data_layout_plan.toml",
            "training-layout",
        ))
        .expect("training layout");
    for directory in [
        "data/raw/",
        "data/canonical/",
        "data/features/",
        "data/labels/",
        "data/sequences/",
        "data/predictions/",
        "data/model_cards/",
        "data/evaluations/",
        "data/registry/",
    ] {
        assert!(
            layout.directories.contains(&directory.to_string()),
            "{directory}"
        );
    }

    let lineage = runner
        .run_training_data_lineage_spec(&support::sprint78_training_config_from_example(
            "soma_training_data_lineage_spec.toml",
            "training-lineage",
        ))
        .expect("training lineage");
    assert!(lineage.raw_to_canonical.contains("canonical"));
    assert!(lineage.canonical_to_features.contains("feature"));
    assert!(lineage.features_to_labels.contains("label"));
    assert!(lineage.labels_to_sequences.contains("sequence"));
    assert!(lineage.sequences_to_predictions.contains("prediction"));
    assert!(lineage.predictions_to_evaluations.contains("evaluation"));
}
