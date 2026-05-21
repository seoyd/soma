#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Serialize, de::DeserializeOwned};
use soma_zero::{
    BaselineSignalRealReductionConfig, CounterfactualBackfillRealReductionConfig,
    Sprint96BaselineSignalRecoveryBundle, Sprint96BaselineSignalRecoveryRunner,
    Sprint97CounterfactualBackfillRecoveryBundle, Sprint97CounterfactualBackfillRecoveryRunner,
};
use soma_zero::{
    BaselineSnapshotCoverageBundle, BaselineSnapshotCoverageConfig, BaselineSnapshotCoverageRunner,
    CandleExpansionRealReductionConfig, CommitteeCompletionGateConfig,
    CommitteeEvidenceExpansionConfig, CommitteeReferenceClosureConfig,
    CommitteeReferenceClosureRunner, CoreCommitteeMambaReadinessBundle,
    CoreCommitteeMambaReadinessRunner, CoreCompletionV2Config,
    DashboardRendererRealReductionConfig, ExtModelBPredictionClosureConfig,
    ExtModelBPredictionClosureRunner, ExtModelBPredictionGapClosureBundle,
    ExternalPredictionRealReductionConfig, KrxEvidenceRealReductionConfig,
    KrxEvidenceWarningClosureConfig, OfficialEvidenceDepthExpansionBundle,
    OfficialEvidenceDepthExpansionConfig, OfficialEvidenceDepthExpansionRunner,
    OfflineEvidenceAttachmentBundle, OfflineEvidenceAttachmentConfig,
    OfflineEvidenceAttachmentRunner, OperatorBriefingBundle, OperatorBriefingConfig,
    OperatorBriefingRunner, PredictionHistoryExpansionConfig, PredictionHistoryExpansionPlan,
    PredictionHistoryExpansionReport, PredictionHistoryExpansionRunner,
    PrototypeComparisonInterpretationBundle, PrototypeComparisonInterpretationConfig,
    PrototypeComparisonInterpretationRunner, RealEvidenceFollowupBundle,
    RealEvidenceFollowupConfig, RealEvidenceFollowupRunner, RealEvidencePredictionRefreshBundle,
    RealEvidencePredictionRefreshConfig, RealEvidencePredictionRefreshRunner,
    RealWorkspaceTimeoutAttributionConfig, RepeatedWorkspaceTimingConfig,
    ResidualWorkspaceBinaryAuditConfig, RetirementRegressionEvidencePack,
    RetirementRegressionEvidencePackConfig, RustToolchainModernizationBundle,
    RustToolchainModernizationConfig, RustToolchainModernizationRunner,
    SequenceCoreCandidateRegistryConfig, SequenceCorePrototypeComparisonBundle,
    SequenceCorePrototypeComparisonConfig, SequenceCorePrototypeComparisonRunner,
    SequenceCoreStorageMaterializationBundle, SequenceCoreStorageMaterializationRunner,
    SevenBlockerFamilyRecoveryConfig, Sprint73WorkspaceAcceptanceReport,
    Sprint83AcceptanceRecoveryBundle, Sprint83AcceptanceRecoveryConfig,
    Sprint83AcceptanceRecoveryRunner, Sprint84TestCostReductionBundle,
    Sprint84TestCostReductionRunner, Sprint85WorkspaceGateRecoveryBundle,
    Sprint85WorkspaceGateRecoveryRunner, Sprint86ResidualGateRecoveryBundle,
    Sprint86ResidualGateRecoveryRunner, Sprint87CompileGateRecoveryBundle,
    Sprint87CompileGateRecoveryRunner, Sprint88SevenBlockerRecoveryBundle,
    Sprint88SevenBlockerRecoveryRunner, Sprint89CandleRecoveryBundle, Sprint89CandleRecoveryRunner,
    Sprint90ExternalPredictionRecoveryBundle, Sprint90ExternalPredictionRecoveryRunner,
    Sprint91KrxEvidenceRecoveryBundle, Sprint91KrxEvidenceRecoveryRunner,
    Sprint92KrxWarningClosureBundle, Sprint92KrxWarningClosureRunner,
    Sprint93TimeoutAttributionBundle, Sprint93TimeoutAttributionRunner,
    Sprint94DashboardRendererRecoveryBundle, Sprint94DashboardRendererRecoveryRunner,
    TestBinaryConsolidationConfig, TestOptimizationBundle, TestOptimizationRunner,
    TrainingDataArtifactPopulationConfig, TrainingDataStorageConfig,
    TrainingDataStorageMaterializationConfig, UnexpectedDiffTriageBundle,
    UnexpectedDiffTriageConfig, UnexpectedDiffTriageRunner, WorkspaceAcceptanceCheck,
    WorkspaceCompileGraphAuditConfig, WorkspaceWideTestSurfaceAuditConfig,
    build_sprint73_workspace_acceptance_report,
};
use soma_zero::{
    CommitteeCliSafetyReductionConfig, Sprint95CommitteeCliSafetyRecoveryBundle,
    Sprint95CommitteeCliSafetyRecoveryRunner,
};

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn example_path(name: &str) -> PathBuf {
    project_root().join("examples").join(name)
}

pub fn output_dir(name: &str) -> PathBuf {
    let path = project_root()
        .join("target")
        .join("sprint69-tests")
        .join(name);
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("create sprint69 output dir");
    path
}

pub fn absolutize(path: &str) -> PathBuf {
    let candidate = Path::new(path);
    if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        project_root().join(candidate)
    }
}

pub fn coverage_config_from_example(
    name: &str,
    output_name: &str,
) -> BaselineSnapshotCoverageConfig {
    let mut config = BaselineSnapshotCoverageConfig::from_toml_path(&example_path(name))
        .expect("parse sprint69 config");
    absolutize_coverage_paths(&mut config);
    config.output_root = output_dir(output_name).display().to_string();
    config
}

pub fn run_coverage(name: &str, output_name: &str) -> BaselineSnapshotCoverageBundle {
    let config = coverage_config_from_example(name, output_name);
    BaselineSnapshotCoverageRunner::default()
        .run(&config)
        .expect("run sprint69 coverage")
}

pub fn read_json<T: DeserializeOwned>(path: impl AsRef<Path>) -> T {
    serde_json::from_str(&fs::read_to_string(path).expect("read sprint69 json"))
        .expect("parse sprint69 json")
}

pub fn write_support_json<T: Serialize>(output_name: &str, file_name: &str, value: &T) -> String {
    let dir = output_dir(output_name);
    let path = dir.join(file_name);
    let text = serde_json::to_string_pretty(value).expect("serialize sprint69 json");
    fs::write(&path, text).expect("write sprint69 json");
    path.display().to_string()
}

pub fn write_support_text(output_name: &str, file_name: &str, text: &str) -> String {
    let dir = output_dir(output_name);
    let path = dir.join(file_name);
    fs::write(&path, text).expect("write sprint69 text");
    path.display().to_string()
}

pub fn triage_output_dir(name: &str) -> PathBuf {
    let path = project_root()
        .join("target")
        .join("sprint70-tests")
        .join(name);
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("create sprint70 output dir");
    path
}

pub fn triage_config_from_example(name: &str, output_name: &str) -> UnexpectedDiffTriageConfig {
    let mut config = UnexpectedDiffTriageConfig::from_toml_path(&example_path(name))
        .expect("parse sprint70 config");
    config.coverage_config_path = absolutize(&config.coverage_config_path)
        .display()
        .to_string();
    config.output_root = triage_output_dir(output_name).display().to_string();
    config
}

pub fn run_triage(name: &str, output_name: &str) -> UnexpectedDiffTriageBundle {
    let config = triage_config_from_example(name, output_name);
    UnexpectedDiffTriageRunner::default()
        .run(&config)
        .expect("run sprint70 triage")
}

pub fn briefing_output_dir(name: &str) -> PathBuf {
    let path = briefing_output_path(name);
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("create sprint71 output dir");
    path
}

pub fn briefing_output_path(name: &str) -> PathBuf {
    let path = project_root()
        .join("target")
        .join("sprint71-tests")
        .join(name);
    path
}

pub fn briefing_config_from_example(name: &str, output_name: &str) -> OperatorBriefingConfig {
    let mut config =
        OperatorBriefingConfig::from_toml_path(&example_path(name)).expect("parse sprint71 config");
    for paths in [
        &mut config.unexpected_diff_triage_paths,
        &mut config.control_tower_diff_triage_paths,
        &mut config.trace_coverage_paths,
        &mut config.model_ops_rollup_paths,
        &mut config.model_ops_trace_paths,
        &mut config.baseline_snapshot_coverage_paths,
        &mut config.conservative_leaderboard_paths,
        &mut config.owner_review_paths,
        &mut config.kis_evidence_paths,
        &mut config.sequence_dataset_paths,
        &mut config.trinity_loop_paths,
        &mut config.risk_report_paths,
        &mut config.previous_briefing_paths,
    ] {
        for path in paths.iter_mut() {
            *path = absolutize(path).display().to_string();
        }
    }
    config.output_root = briefing_output_dir(output_name).display().to_string();
    config
}

pub fn run_briefing(name: &str, output_name: &str) -> OperatorBriefingBundle {
    let config = briefing_config_from_example(name, output_name);
    OperatorBriefingRunner::default()
        .run(&config)
        .expect("run sprint71 briefing")
}

pub fn attachment_output_dir(name: &str) -> PathBuf {
    let path = attachment_output_path(name);
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("create sprint72 output dir");
    path
}

pub fn attachment_output_path(name: &str) -> PathBuf {
    project_root()
        .join("target")
        .join("sprint72-tests")
        .join(name)
}

pub fn attachment_config_from_example(
    name: &str,
    output_name: &str,
) -> OfflineEvidenceAttachmentConfig {
    let mut config = OfflineEvidenceAttachmentConfig::from_toml_path(&example_path(name))
        .expect("parse sprint72 attachment config");
    for paths in [
        &mut config.operator_briefing_paths,
        &mut config.evidence_gap_checklist_paths,
        &mut config.kis_evidence_paths,
        &mut config.sequence_dataset_paths,
        &mut config.prediction_csv_paths,
        &mut config.model_card_paths,
        &mut config.regression_evidence_paths,
        &mut config.owner_note_paths,
        &mut config.retirement_evidence_paths,
        &mut config.leaderboard_warning_paths,
    ] {
        for path in paths.iter_mut() {
            *path = absolutize(path).display().to_string();
        }
    }
    config.output_root = attachment_output_dir(output_name).display().to_string();
    config
}

pub fn run_offline_attachment(name: &str, output_name: &str) -> OfflineEvidenceAttachmentBundle {
    let config = attachment_config_from_example(name, output_name);
    OfflineEvidenceAttachmentRunner::default()
        .run(&config)
        .expect("run sprint72 offline attachment")
}

pub fn prediction_history_config_from_example(name: &str) -> PredictionHistoryExpansionConfig {
    let mut config = PredictionHistoryExpansionConfig::from_toml_path(&example_path(name))
        .expect("parse sprint72 prediction history config");
    for paths in [
        &mut config.current_prediction_history_paths,
        &mut config.prediction_csv_paths,
        &mut config.model_card_paths,
        &mut config.sequence_export_manifest_paths,
    ] {
        for path in paths.iter_mut() {
            *path = absolutize(path).display().to_string();
        }
    }
    config
}

pub fn run_prediction_history_expand(
    name: &str,
) -> (
    PredictionHistoryExpansionPlan,
    PredictionHistoryExpansionReport,
) {
    let config = prediction_history_config_from_example(name);
    PredictionHistoryExpansionRunner::default()
        .run(&config)
        .expect("run sprint72 prediction history expansion")
}

pub fn retirement_pack_config_from_example(name: &str) -> RetirementRegressionEvidencePackConfig {
    let mut config = RetirementRegressionEvidencePackConfig::from_toml_path(&example_path(name))
        .expect("parse sprint72 retirement pack config");
    for paths in [
        &mut config.regression_evidence_paths,
        &mut config.calibration_drift_paths,
        &mut config.risk_profile_paths,
        &mut config.leaderboard_history_paths,
        &mut config.comparison_paths,
        &mut config.owner_review_paths,
    ] {
        for path in paths.iter_mut() {
            *path = absolutize(path).display().to_string();
        }
    }
    config
}

pub fn run_retirement_pack(name: &str) -> RetirementRegressionEvidencePack {
    let config = retirement_pack_config_from_example(name);
    OfflineEvidenceAttachmentRunner::default()
        .run_retirement_regression_pack(&config)
        .expect("run sprint72 retirement pack")
}

pub fn sprint73_output_dir(name: &str) -> PathBuf {
    let path = sprint73_output_path(name);
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("create sprint73 output dir");
    path
}

pub fn sprint73_output_path(name: &str) -> PathBuf {
    project_root()
        .join("target")
        .join("sprint73-tests")
        .join(name)
}

pub fn sprint73_config_from_example(
    name: &str,
    output_name: &str,
) -> ExtModelBPredictionClosureConfig {
    let mut config = ExtModelBPredictionClosureConfig::from_toml_path(&example_path(name))
        .expect("parse sprint73 config");
    for paths in [
        &mut config.previous_prediction_history_paths,
        &mut config.new_prediction_csv_paths,
        &mut config.model_card_paths,
        &mut config.sequence_export_manifest_paths,
        &mut config.offline_evidence_attachment_paths,
    ] {
        for path in paths.iter_mut() {
            *path = absolutize(path).display().to_string();
        }
    }
    config.output_root = sprint73_output_dir(output_name).display().to_string();
    config
}

pub fn run_sprint73_bundle(name: &str, output_name: &str) -> ExtModelBPredictionGapClosureBundle {
    let config = sprint73_config_from_example(name, output_name);
    ExtModelBPredictionClosureRunner::default()
        .run(&config)
        .expect("run sprint73 bundle")
}

pub fn build_workspace_acceptance_report(
    report_id: &str,
    checks: Vec<WorkspaceAcceptanceCheck>,
) -> Sprint73WorkspaceAcceptanceReport {
    build_sprint73_workspace_acceptance_report(report_id, checks)
}

pub fn sprint74_output_dir(name: &str) -> PathBuf {
    let path = sprint74_output_path(name);
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("create sprint74 output dir");
    path
}

pub fn sprint74_output_path(name: &str) -> PathBuf {
    project_root()
        .join("target")
        .join("sprint74-tests")
        .join(name)
}

pub fn sprint74_config_from_example(name: &str, output_name: &str) -> RealEvidenceFollowupConfig {
    let mut config = RealEvidenceFollowupConfig::from_toml_path(&example_path(name))
        .expect("parse sprint74 config");
    for paths in [
        &mut config.sprint73_final_bundle_paths,
        &mut config.operator_briefing_paths,
        &mut config.kis_canonical_csv_paths,
        &mut config.kis_provenance_paths,
        &mut config.kis_preflight_paths,
        &mut config.kis_manifest_paths,
        &mut config.kis_collection_plan_paths,
        &mut config.sequence_export_manifest_paths,
        &mut config.model_ops_paths,
    ] {
        for path in paths.iter_mut() {
            *path = absolutize(path).display().to_string();
        }
    }
    config.output_root = sprint74_output_dir(output_name).display().to_string();
    config
}

pub fn run_sprint74_bundle(name: &str, output_name: &str) -> RealEvidenceFollowupBundle {
    let config = sprint74_config_from_example(name, output_name);
    RealEvidenceFollowupRunner::default()
        .run(&config)
        .expect("run sprint74 bundle")
}

pub fn sprint75_output_dir(name: &str) -> PathBuf {
    let path = sprint75_output_path(name);
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("create sprint75 output dir");
    path
}

pub fn sprint75_output_path(name: &str) -> PathBuf {
    project_root()
        .join("target")
        .join("sprint75-tests")
        .join(name)
}

pub fn sprint75_config_from_example(
    name: &str,
    output_name: &str,
) -> RealEvidencePredictionRefreshConfig {
    let mut config = RealEvidencePredictionRefreshConfig::from_toml_path(&example_path(name))
        .expect("parse sprint75 config");
    for paths in [
        &mut config.real_evidence_followup_paths,
        &mut config.real_modelops_impact_paths,
        &mut config.sequence_readiness_paths,
        &mut config.sequence_export_manifest_paths,
        &mut config.new_prediction_csv_paths,
        &mut config.model_card_paths,
        &mut config.external_model_reference_paths,
        &mut config.control_tower_real_evidence_refresh_paths,
    ] {
        for path in paths.iter_mut() {
            *path = absolutize(path).display().to_string();
        }
    }
    config.output_root = sprint75_output_dir(output_name).display().to_string();
    config
}

pub fn run_sprint75_bundle(name: &str, output_name: &str) -> RealEvidencePredictionRefreshBundle {
    let config = sprint75_config_from_example(name, output_name);
    RealEvidencePredictionRefreshRunner::default()
        .run(&config)
        .expect("run sprint75 bundle")
}

pub fn sprint76_output_dir(name: &str) -> PathBuf {
    let path = sprint76_output_path(name);
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("create sprint76 output dir");
    path
}

pub fn sprint76_output_path(name: &str) -> PathBuf {
    project_root()
        .join("target")
        .join("sprint76-tests")
        .join(name)
}

pub fn sprint76_config_from_example(
    name: &str,
    output_name: &str,
) -> RustToolchainModernizationConfig {
    let mut config = RustToolchainModernizationConfig::from_toml_path(&example_path(name))
        .expect("parse sprint76 config");
    for path in &mut config.current_toolchain_report_paths {
        *path = absolutize(path).display().to_string();
    }
    config.output_root = sprint76_output_dir(output_name).display().to_string();
    config
}

pub fn run_sprint76_bundle(name: &str, output_name: &str) -> RustToolchainModernizationBundle {
    let config = sprint76_config_from_example(name, output_name);
    RustToolchainModernizationRunner::default()
        .run(&config)
        .expect("run sprint76 bundle")
}

pub fn sprint77_output_dir(name: &str) -> PathBuf {
    let path = sprint77_output_path(name);
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("create sprint77 output dir");
    path
}

pub fn sprint77_output_path(name: &str) -> PathBuf {
    project_root()
        .join("target")
        .join("sprint77-tests")
        .join(name)
}

pub fn sprint77_config_from_example(
    name: &str,
    output_name: &str,
) -> RepeatedWorkspaceTimingConfig {
    let mut config = RepeatedWorkspaceTimingConfig::from_toml_path(&example_path(name))
        .expect("parse sprint77 config");
    for path in &mut config.sample_timing_paths {
        *path = absolutize(path).display().to_string();
    }
    config.output_root = sprint77_output_dir(output_name).display().to_string();
    config
}

pub fn run_sprint77_bundle(name: &str, output_name: &str) -> TestOptimizationBundle {
    let config = sprint77_config_from_example(name, output_name);
    TestOptimizationRunner::default()
        .run(&config)
        .expect("run sprint77 bundle")
}

pub fn sprint78_output_dir(name: &str) -> PathBuf {
    let path = sprint78_output_path(name);
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("create sprint78 output dir");
    path
}

pub fn sprint78_output_path(name: &str) -> PathBuf {
    project_root()
        .join("target")
        .join("sprint78-tests")
        .join(name)
}

pub fn sprint78_core_config_from_example(name: &str, output_name: &str) -> CoreCompletionV2Config {
    let mut config = CoreCompletionV2Config::from_toml_path(&example_path(name))
        .expect("parse sprint78 core config");
    for paths in [
        &mut config.core_check_paths,
        &mut config.committee_v1_paths,
        &mut config.sequence_export_paths,
        &mut config.real_evidence_paths,
        &mut config.model_ops_paths,
        &mut config.toolchain_speed_paths,
    ] {
        for path in paths.iter_mut() {
            *path = absolutize(path).display().to_string();
        }
    }
    config.output_root = sprint78_output_dir(output_name).display().to_string();
    config
}

pub fn sprint78_committee_config_from_example(
    name: &str,
    output_name: &str,
) -> CommitteeCompletionGateConfig {
    let mut config = CommitteeCompletionGateConfig::from_toml_path(&example_path(name))
        .expect("parse sprint78 committee config");
    for paths in [
        &mut config.committee_v1_paths,
        &mut config.official_benchmark_paths,
        &mut config.outcome_link_paths,
        &mut config.chair_risk_calibration_paths,
        &mut config.real_evidence_paths,
    ] {
        for path in paths.iter_mut() {
            *path = absolutize(path).display().to_string();
        }
    }
    config.output_root = sprint78_output_dir(output_name).display().to_string();
    config
}

pub fn sprint78_training_config_from_example(
    name: &str,
    output_name: &str,
) -> TrainingDataStorageConfig {
    let mut config = TrainingDataStorageConfig::from_toml_path(&example_path(name))
        .expect("parse sprint78 training config");
    config.output_root = sprint78_output_dir(output_name).display().to_string();
    config.storage_root = absolutize(&config.storage_root).display().to_string();
    config.raw_root = absolutize(&config.raw_root).display().to_string();
    config.canonical_root = absolutize(&config.canonical_root).display().to_string();
    config.features_root = absolutize(&config.features_root).display().to_string();
    config.labels_root = absolutize(&config.labels_root).display().to_string();
    config.sequences_root = absolutize(&config.sequences_root).display().to_string();
    config.predictions_root = absolutize(&config.predictions_root).display().to_string();
    config.model_cards_root = absolutize(&config.model_cards_root).display().to_string();
    config.evaluations_root = absolutize(&config.evaluations_root).display().to_string();
    config.registry_root = absolutize(&config.registry_root).display().to_string();
    config
}

pub fn run_sprint78_bundle(name: &str, output_name: &str) -> CoreCommitteeMambaReadinessBundle {
    let config = sprint78_core_config_from_example(name, output_name);
    CoreCommitteeMambaReadinessRunner::default()
        .run(&config)
        .expect("run sprint78 bundle")
}

pub fn sprint79_output_dir(name: &str) -> PathBuf {
    let path = sprint79_output_path(name);
    fs::create_dir_all(&path).expect("create sprint79 output dir");
    path
}

pub fn sprint79_output_path(name: &str) -> PathBuf {
    project_root()
        .join("target")
        .join("sprint79-tests")
        .join(name)
}

pub fn reset_sprint79_output_dir(name: &str) -> PathBuf {
    let path = sprint79_output_path(name);
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("reset sprint79 output dir");
    path
}

pub fn sprint79_registry_config_from_example(
    name: &str,
    output_name: &str,
) -> SequenceCoreCandidateRegistryConfig {
    let mut config = SequenceCoreCandidateRegistryConfig::from_toml_path(&example_path(name))
        .expect("parse sprint79 registry config");
    for paths in [
        &mut config.mamba3fin_contract_paths,
        &mut config.gated_deltanet_contract_paths,
        &mut config.training_data_storage_paths,
        &mut config.committee_gate_paths,
    ] {
        for path in paths.iter_mut() {
            *path = absolutize(path).display().to_string();
        }
    }
    config.output_root = sprint79_output_dir(output_name).display().to_string();
    config
}

pub fn sprint79_materialization_config_from_example(
    name: &str,
    output_name: &str,
) -> TrainingDataStorageMaterializationConfig {
    let mut config = TrainingDataStorageMaterializationConfig::from_toml_path(&example_path(name))
        .expect("parse sprint79 materialization config");
    for paths in [
        &mut config.training_data_storage_config_paths,
        &mut config.training_data_layout_plan_paths,
        &mut config.training_data_registry_spec_paths,
    ] {
        for path in paths.iter_mut() {
            *path = absolutize(path).display().to_string();
        }
    }
    let root = sprint79_output_dir(output_name);
    config.output_root = root.display().to_string();
    config.storage_root = root.join("data").display().to_string();
    config
}

pub fn run_sprint79_bundle(
    registry_name: &str,
    materialization_name: &str,
    output_name: &str,
) -> SequenceCoreStorageMaterializationBundle {
    reset_sprint79_output_dir(output_name);
    let registry = sprint79_registry_config_from_example(registry_name, output_name);
    let materialization =
        sprint79_materialization_config_from_example(materialization_name, output_name);
    SequenceCoreStorageMaterializationRunner::default()
        .run(&registry, &materialization)
        .expect("run sprint79 bundle")
}

pub fn sprint80_output_path(name: &str) -> PathBuf {
    project_root()
        .join("target")
        .join("sprint80-tests")
        .join(name)
}

pub fn sprint80_output_dir(name: &str) -> PathBuf {
    let path = sprint80_output_path(name);
    fs::create_dir_all(&path).expect("create sprint80 output dir");
    path
}

pub fn reset_sprint80_output_dir(name: &str) -> PathBuf {
    let path = sprint80_output_path(name);
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("reset sprint80 output dir");
    path
}

pub fn sprint80_compare_config_from_example(
    name: &str,
    output_name: &str,
) -> SequenceCorePrototypeComparisonConfig {
    let mut config = SequenceCorePrototypeComparisonConfig::from_toml_path(&example_path(name))
        .expect("parse sprint80 compare config");
    for paths in [
        &mut config.sequence_core_registry_paths,
        &mut config.training_storage_manifest_paths,
        &mut config.dataset_registry_paths,
        &mut config.sequence_export_manifest_paths,
        &mut config.feature_schema_manifest_paths,
        &mut config.label_manifest_paths,
        &mut config.split_manifest_paths,
        &mut config.no_lookahead_proof_paths,
        &mut config.mamba3fin_prediction_csv_paths,
        &mut config.gated_deltanet_prediction_csv_paths,
        &mut config.mamba3fin_model_card_paths,
        &mut config.gated_deltanet_model_card_paths,
        &mut config.committee_evidence_paths,
    ] {
        for path in paths.iter_mut() {
            *path = absolutize(path).display().to_string();
        }
    }
    config.output_root = sprint80_output_dir(output_name).display().to_string();
    config
}

pub fn sprint80_committee_config_from_example(
    name: &str,
    output_name: &str,
) -> CommitteeEvidenceExpansionConfig {
    let mut config = CommitteeEvidenceExpansionConfig::from_toml_path(&example_path(name))
        .expect("parse sprint80 committee config");
    for paths in [
        &mut config.committee_completion_gate_paths,
        &mut config.materialization_plan_paths,
        &mut config.real_evidence_paths,
        &mut config.sequence_export_manifest_paths,
        &mut config.outcome_reference_paths,
        &mut config.baseline_reference_paths,
        &mut config.no_trade_reference_paths,
        &mut config.risk_denied_reference_paths,
    ] {
        for path in paths.iter_mut() {
            *path = absolutize(path).display().to_string();
        }
    }
    config.output_root = sprint80_output_dir(output_name).display().to_string();
    config
}

pub fn sprint80_population_config_from_example(
    name: &str,
    output_name: &str,
) -> TrainingDataArtifactPopulationConfig {
    let mut config = TrainingDataArtifactPopulationConfig::from_toml_path(&example_path(name))
        .expect("parse sprint80 population config");
    for paths in [
        &mut config.registry_manifest_paths,
        &mut config.sequence_export_manifest_paths,
        &mut config.feature_schema_manifest_paths,
        &mut config.label_manifest_paths,
        &mut config.split_manifest_paths,
        &mut config.prediction_csv_paths,
        &mut config.model_card_paths,
        &mut config.evaluation_report_paths,
        &mut config.committee_pack_paths,
    ] {
        for path in paths.iter_mut() {
            *path = absolutize(path).display().to_string();
        }
    }
    let root = sprint80_output_dir(output_name);
    config.output_root = root.display().to_string();
    config.storage_root = root.join("data").display().to_string();
    config
}

pub fn run_sprint80_bundle(name: &str, output_name: &str) -> SequenceCorePrototypeComparisonBundle {
    reset_sprint80_output_dir(output_name);
    let config = sprint80_compare_config_from_example(name, output_name);
    SequenceCorePrototypeComparisonRunner::default()
        .run(&config)
        .expect("run sprint80 bundle")
}

pub fn sprint81_output_path(name: &str) -> PathBuf {
    project_root()
        .join("target")
        .join("sprint81-tests")
        .join(name)
}

pub fn sprint81_output_dir(name: &str) -> PathBuf {
    let path = sprint81_output_path(name);
    fs::create_dir_all(&path).expect("create sprint81 output dir");
    path
}

pub fn reset_sprint81_output_dir(name: &str) -> PathBuf {
    let path = sprint81_output_path(name);
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("reset sprint81 output dir");
    path
}

pub fn sprint81_interpretation_config_from_example(
    name: &str,
    output_name: &str,
) -> PrototypeComparisonInterpretationConfig {
    let mut config = PrototypeComparisonInterpretationConfig::from_toml_path(&example_path(name))
        .expect("parse sprint81 interpretation config");
    for paths in [
        &mut config.sequence_core_prototype_comparison_paths,
        &mut config.prototype_evaluation_paths,
        &mut config.prototype_calibration_paths,
        &mut config.prototype_risk_interaction_paths,
        &mut config.prototype_ablation_paths,
        &mut config.prototype_promotion_gate_paths,
        &mut config.committee_vs_sequence_paths,
        &mut config.committee_scenario_pack_paths,
        &mut config.committee_outcome_pack_paths,
        &mut config.committee_counterfactual_pack_paths,
        &mut config.training_artifact_population_paths,
        &mut config.training_populated_integrity_paths,
    ] {
        for path in paths.iter_mut() {
            *path = absolutize(path).display().to_string();
        }
    }
    config.output_root = sprint81_output_dir(output_name).display().to_string();
    config
}

pub fn run_sprint81_bundle(
    name: &str,
    output_name: &str,
) -> PrototypeComparisonInterpretationBundle {
    reset_sprint81_output_dir(output_name);
    let config = sprint81_interpretation_config_from_example(name, output_name);
    PrototypeComparisonInterpretationRunner::default()
        .run(&config)
        .expect("run sprint81 bundle")
}

pub fn sprint82_output_path(name: &str) -> PathBuf {
    project_root()
        .join("target")
        .join("sprint82-tests")
        .join(name)
}

pub fn sprint82_output_dir(name: &str) -> PathBuf {
    let path = sprint82_output_path(name);
    fs::create_dir_all(&path).expect("create sprint82 output dir");
    path
}

pub fn reset_sprint82_output_dir(name: &str) -> PathBuf {
    let path = sprint82_output_path(name);
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("reset sprint82 output dir");
    path
}

pub fn sprint82_evidence_config_from_example(
    name: &str,
    output_name: &str,
) -> OfficialEvidenceDepthExpansionConfig {
    let mut config = OfficialEvidenceDepthExpansionConfig::from_toml_path(&example_path(name))
        .expect("parse sprint82 evidence config");
    for paths in [
        &mut config.sprint81_interpretation_paths,
        &mut config.real_evidence_paths,
        &mut config.kis_canonical_csv_paths,
        &mut config.kis_provenance_paths,
        &mut config.kis_preflight_paths,
        &mut config.sequence_export_manifest_paths,
        &mut config.feature_schema_manifest_paths,
        &mut config.label_manifest_paths,
        &mut config.committee_reference_paths,
    ] {
        for path in paths.iter_mut() {
            *path = absolutize(path).display().to_string();
        }
    }
    config.output_root = sprint82_output_dir(output_name).display().to_string();
    config
}

pub fn sprint82_closure_config_from_example(
    name: &str,
    output_name: &str,
) -> CommitteeReferenceClosureConfig {
    let mut config = CommitteeReferenceClosureConfig::from_toml_path(&example_path(name))
        .expect("parse sprint82 closure config");
    for paths in [
        &mut config.committee_reference_audit_paths,
        &mut config.committee_reference_depth_plan_paths,
        &mut config.official_evidence_depth_paths,
        &mut config.scenario_pack_paths,
        &mut config.outcome_pack_paths,
        &mut config.baseline_pack_paths,
        &mut config.no_trade_pack_paths,
        &mut config.risk_denied_pack_paths,
        &mut config.chair_trace_paths,
        &mut config.risk_trace_paths,
    ] {
        for path in paths.iter_mut() {
            *path = absolutize(path).display().to_string();
        }
    }
    config.output_root = sprint82_output_dir(output_name).display().to_string();
    config
}

pub fn run_sprint82_bundle(name: &str, output_name: &str) -> OfficialEvidenceDepthExpansionBundle {
    reset_sprint82_output_dir(output_name);
    let config = sprint82_evidence_config_from_example(name, output_name);
    OfficialEvidenceDepthExpansionRunner::default()
        .run(&config)
        .expect("run sprint82 bundle")
}

pub fn run_sprint82_closure(
    name: &str,
    output_name: &str,
) -> soma_zero::CommitteeReferenceClosureReport {
    reset_sprint82_output_dir(output_name);
    let config = sprint82_closure_config_from_example(name, output_name);
    CommitteeReferenceClosureRunner::default()
        .run(&config)
        .expect("run sprint82 closure")
}

pub fn sprint83_output_path(name: &str) -> PathBuf {
    project_root()
        .join("target")
        .join("sprint83-tests")
        .join(name)
}

pub fn sprint83_output_dir(name: &str) -> PathBuf {
    let path = sprint83_output_path(name);
    fs::create_dir_all(&path).expect("create sprint83 output dir");
    path
}

pub fn reset_sprint83_output_dir(name: &str) -> PathBuf {
    let path = sprint83_output_path(name);
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("reset sprint83 output dir");
    path
}

pub fn sprint83_recovery_config_from_example(
    name: &str,
    output_name: &str,
) -> Sprint83AcceptanceRecoveryConfig {
    let mut config = Sprint83AcceptanceRecoveryConfig::from_toml_path(&example_path(name))
        .expect("parse sprint83 recovery config");
    for paths in [
        &mut config.sprint82_report_paths,
        &mut config.sprint82_fixture_paths,
        &mut config.sprint82_cli_smoke_paths,
        &mut config.workspace_timing_paths,
    ] {
        for path in paths.iter_mut() {
            *path = absolutize(path).display().to_string();
        }
    }
    config.output_root = sprint83_output_dir(output_name).display().to_string();
    config
}

pub fn run_sprint83_bundle(name: &str, output_name: &str) -> Sprint83AcceptanceRecoveryBundle {
    reset_sprint83_output_dir(output_name);
    let config = sprint83_recovery_config_from_example(name, output_name);
    Sprint83AcceptanceRecoveryRunner::default()
        .run(&config)
        .expect("run sprint83 bundle")
}

pub fn sprint84_output_path(name: &str) -> PathBuf {
    project_root()
        .join("target")
        .join("sprint84-tests")
        .join(name)
}

pub fn sprint84_output_dir(name: &str) -> PathBuf {
    let path = sprint84_output_path(name);
    fs::create_dir_all(&path).expect("create sprint84 output dir");
    path
}

pub fn reset_sprint84_output_dir(name: &str) -> PathBuf {
    let path = sprint84_output_path(name);
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("reset sprint84 output dir");
    path
}

pub fn sprint84_config_from_example(
    name: &str,
    output_name: &str,
) -> TestBinaryConsolidationConfig {
    let mut config = TestBinaryConsolidationConfig::from_toml_path(&example_path(name))
        .expect("parse sprint84 test cost config");
    for paths in [
        &mut config.sprint83_recovery_paths,
        &mut config.test_inventory_paths,
    ] {
        for path in paths.iter_mut() {
            *path = absolutize(path).display().to_string();
        }
    }
    config.output_root = sprint84_output_dir(output_name).display().to_string();
    config
}

pub fn run_sprint84_bundle(name: &str, output_name: &str) -> Sprint84TestCostReductionBundle {
    reset_sprint84_output_dir(output_name);
    let config = sprint84_config_from_example(name, output_name);
    Sprint84TestCostReductionRunner::default()
        .run(&config)
        .expect("run sprint84 bundle")
}

pub fn sprint85_output_path(name: &str) -> PathBuf {
    project_root()
        .join("target")
        .join("sprint85-tests")
        .join(name)
}

pub fn sprint85_output_dir(name: &str) -> PathBuf {
    let path = sprint85_output_path(name);
    fs::create_dir_all(&path).expect("create sprint85 output dir");
    path
}

pub fn reset_sprint85_output_dir(name: &str) -> PathBuf {
    let path = sprint85_output_path(name);
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("reset sprint85 output dir");
    path
}

pub fn sprint85_config_from_example(
    name: &str,
    output_name: &str,
) -> WorkspaceWideTestSurfaceAuditConfig {
    let mut config = WorkspaceWideTestSurfaceAuditConfig::from_toml_path(&example_path(name))
        .expect("parse sprint85 workspace gate config");
    for paths in [
        &mut config.sprint84_bundle_paths,
        &mut config.test_inventory_paths,
        &mut config.cargo_metadata_paths,
    ] {
        for path in paths.iter_mut() {
            *path = absolutize(path).display().to_string();
        }
    }
    config.output_root = sprint85_output_dir(output_name).display().to_string();
    config
}

pub fn run_sprint85_bundle(name: &str, output_name: &str) -> Sprint85WorkspaceGateRecoveryBundle {
    reset_sprint85_output_dir(output_name);
    let config = sprint85_config_from_example(name, output_name);
    Sprint85WorkspaceGateRecoveryRunner::default()
        .run(&config)
        .expect("run sprint85 bundle")
}

pub fn sprint86_output_path(name: &str) -> PathBuf {
    project_root()
        .join("target")
        .join("sprint86-tests")
        .join(name)
}

pub fn sprint86_output_dir(name: &str) -> PathBuf {
    let path = sprint86_output_path(name);
    fs::create_dir_all(&path).expect("create sprint86 output dir");
    path
}

pub fn reset_sprint86_output_dir(name: &str) -> PathBuf {
    let path = sprint86_output_path(name);
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("reset sprint86 output dir");
    path
}

pub fn sprint86_config_from_example(
    name: &str,
    output_name: &str,
) -> ResidualWorkspaceBinaryAuditConfig {
    let mut config = ResidualWorkspaceBinaryAuditConfig::from_toml_path(&example_path(name))
        .expect("parse sprint86 residual gate config");
    for paths in [
        &mut config.sprint85_bundle_paths,
        &mut config.cargo_metadata_paths,
        &mut config.full_workspace_attempt_paths,
    ] {
        for path in paths.iter_mut() {
            *path = absolutize(path).display().to_string();
        }
    }
    config.output_root = sprint86_output_dir(output_name).display().to_string();
    config
}

pub fn run_sprint86_bundle(name: &str, output_name: &str) -> Sprint86ResidualGateRecoveryBundle {
    reset_sprint86_output_dir(output_name);
    let config = sprint86_config_from_example(name, output_name);
    Sprint86ResidualGateRecoveryRunner::default()
        .run(&config)
        .expect("run sprint86 bundle")
}

pub fn sprint87_output_path(name: &str) -> PathBuf {
    project_root()
        .join("target")
        .join("sprint87-tests")
        .join(name)
}

pub fn sprint87_output_dir(name: &str) -> PathBuf {
    let path = sprint87_output_path(name);
    fs::create_dir_all(&path).expect("create sprint87 output dir");
    path
}

pub fn reset_sprint87_output_dir(name: &str) -> PathBuf {
    let path = sprint87_output_path(name);
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("reset sprint87 output dir");
    path
}

pub fn sprint87_config_from_example(
    name: &str,
    output_name: &str,
) -> WorkspaceCompileGraphAuditConfig {
    let mut config = WorkspaceCompileGraphAuditConfig::from_toml_path(&example_path(name))
        .expect("parse sprint87 compile gate config");
    for paths in [
        &mut config.sprint86_bundle_paths,
        &mut config.cargo_metadata_paths,
        &mut config.cargo_timings_paths,
        &mut config.compile_only_attempt_paths,
        &mut config.full_workspace_attempt_paths,
    ] {
        for path in paths.iter_mut() {
            *path = absolutize(path).display().to_string();
        }
    }
    config.output_root = sprint87_output_dir(output_name).display().to_string();
    config
}

pub fn run_sprint87_bundle(name: &str, output_name: &str) -> Sprint87CompileGateRecoveryBundle {
    reset_sprint87_output_dir(output_name);
    let config = sprint87_config_from_example(name, output_name);
    Sprint87CompileGateRecoveryRunner::default()
        .run(&config)
        .expect("run sprint87 bundle")
}

pub fn sprint88_output_path(name: &str) -> PathBuf {
    project_root()
        .join("target")
        .join("sprint88-tests")
        .join(name)
}

pub fn sprint88_output_dir(name: &str) -> PathBuf {
    let path = sprint88_output_path(name);
    fs::create_dir_all(&path).expect("create sprint88 output dir");
    path
}

pub fn reset_sprint88_output_dir(name: &str) -> PathBuf {
    let path = sprint88_output_path(name);
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("reset sprint88 output dir");
    path
}

pub fn sprint88_config_from_example(
    name: &str,
    output_name: &str,
) -> SevenBlockerFamilyRecoveryConfig {
    let mut config = SevenBlockerFamilyRecoveryConfig::from_toml_path(&example_path(name))
        .expect("parse sprint88 seven blocker config");
    for paths in [
        &mut config.sprint87_bundle_paths,
        &mut config.compile_gate_report_paths,
        &mut config.blocker_drilldown_paths,
        &mut config.cargo_metadata_paths,
        &mut config.cargo_timings_paths,
    ] {
        for path in paths.iter_mut() {
            *path = absolutize(path).display().to_string();
        }
    }
    config.output_root = sprint88_output_dir(output_name).display().to_string();
    config
}

pub fn run_sprint88_bundle(name: &str, output_name: &str) -> Sprint88SevenBlockerRecoveryBundle {
    reset_sprint88_output_dir(output_name);
    let config = sprint88_config_from_example(name, output_name);
    Sprint88SevenBlockerRecoveryRunner::default()
        .run(&config)
        .expect("run sprint88 bundle")
}

pub fn sprint89_output_path(name: &str) -> PathBuf {
    project_root()
        .join("target")
        .join("sprint89-tests")
        .join(name)
}

pub fn sprint89_output_dir(name: &str) -> PathBuf {
    let path = sprint89_output_path(name);
    fs::create_dir_all(&path).expect("create sprint89 output dir");
    path
}

pub fn reset_sprint89_output_dir(name: &str) -> PathBuf {
    let path = sprint89_output_path(name);
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("reset sprint89 output dir");
    path
}

pub fn sprint89_config_from_example(
    name: &str,
    output_name: &str,
) -> CandleExpansionRealReductionConfig {
    let mut config = CandleExpansionRealReductionConfig::from_toml_path(&example_path(name))
        .expect("parse sprint89 candle reduction config");
    for paths in [
        &mut config.sprint88_bundle_paths,
        &mut config.candle_expansion_suite_paths,
        &mut config.official_candle_expansion_runner_paths,
        &mut config.cargo_metadata_paths,
        &mut config.cargo_timings_paths,
    ] {
        for path in paths.iter_mut() {
            *path = absolutize(path).display().to_string();
        }
    }
    config.output_root = sprint89_output_dir(output_name).display().to_string();
    config
}

pub fn run_sprint89_bundle(name: &str, output_name: &str) -> Sprint89CandleRecoveryBundle {
    reset_sprint89_output_dir(output_name);
    let config = sprint89_config_from_example(name, output_name);
    Sprint89CandleRecoveryRunner::default()
        .run(&config)
        .expect("run sprint89 bundle")
}

pub fn sprint90_output_path(name: &str) -> PathBuf {
    project_root()
        .join("target")
        .join("sprint90-tests")
        .join(name)
}

pub fn sprint90_output_dir(name: &str) -> PathBuf {
    let path = sprint90_output_path(name);
    fs::create_dir_all(&path).expect("create sprint90 output dir");
    path
}

pub fn reset_sprint90_output_dir(name: &str) -> PathBuf {
    let path = sprint90_output_path(name);
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("reset sprint90 output dir");
    path
}

pub fn sprint90_config_from_example(
    name: &str,
    output_name: &str,
) -> ExternalPredictionRealReductionConfig {
    let mut config = ExternalPredictionRealReductionConfig::from_toml_path(&example_path(name))
        .expect("parse sprint90 external prediction config");
    for paths in [
        &mut config.sprint89_bundle_paths,
        &mut config.external_prediction_suite_paths,
        &mut config.external_prediction_import_v2_paths,
        &mut config.sequence_core_prototype_paths,
        &mut config.model_card_fixture_paths,
        &mut config.prediction_schema_fixture_paths,
        &mut config.cargo_metadata_paths,
        &mut config.cargo_timings_paths,
    ] {
        for path in paths.iter_mut() {
            *path = absolutize(path).display().to_string();
        }
    }
    config.output_root = sprint90_output_dir(output_name).display().to_string();
    config
}

pub fn run_sprint90_bundle(
    name: &str,
    output_name: &str,
) -> Sprint90ExternalPredictionRecoveryBundle {
    reset_sprint90_output_dir(output_name);
    let config = sprint90_config_from_example(name, output_name);
    Sprint90ExternalPredictionRecoveryRunner::default()
        .run(&config)
        .expect("run sprint90 bundle")
}

pub fn sprint91_output_path(name: &str) -> PathBuf {
    project_root()
        .join("target")
        .join("sprint91-tests")
        .join(name)
}

pub fn sprint91_output_dir(name: &str) -> PathBuf {
    let path = sprint91_output_path(name);
    fs::create_dir_all(&path).expect("create sprint91 output dir");
    path
}

pub fn reset_sprint91_output_dir(name: &str) -> PathBuf {
    let path = sprint91_output_path(name);
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("reset sprint91 output dir");
    path
}

pub fn sprint91_config_from_example(
    name: &str,
    output_name: &str,
) -> KrxEvidenceRealReductionConfig {
    let mut config = KrxEvidenceRealReductionConfig::from_toml_path(&example_path(name))
        .expect("parse sprint91 krx evidence config");
    for paths in [
        &mut config.sprint90_bundle_paths,
        &mut config.krx_evidence_suite_paths,
        &mut config.krx_evidence_job_plan_paths,
        &mut config.krx_collection_paths,
        &mut config.krx_auth_fixture_paths,
        &mut config.krx_endpoint_template_fixture_paths,
        &mut config.cargo_metadata_paths,
        &mut config.cargo_timings_paths,
    ] {
        for path in paths.iter_mut() {
            *path = absolutize(path).display().to_string();
        }
    }
    config.output_root = sprint91_output_dir(output_name).display().to_string();
    config
}

pub fn run_sprint91_bundle(name: &str, output_name: &str) -> Sprint91KrxEvidenceRecoveryBundle {
    reset_sprint91_output_dir(output_name);
    let config = sprint91_config_from_example(name, output_name);
    Sprint91KrxEvidenceRecoveryRunner::default()
        .run(&config)
        .expect("run sprint91 bundle")
}

fn absolutize_coverage_paths(config: &mut BaselineSnapshotCoverageConfig) {
    for paths in [
        &mut config.model_ops_trace_bundle_paths,
        &mut config.model_version_diff_trace_paths,
        &mut config.baseline_snapshot_paths,
        &mut config.current_snapshot_paths,
        &mut config.model_version_summary_card_paths,
        &mut config.regression_evidence_trace_paths,
        &mut config.prediction_history_pack_paths,
        &mut config.external_model_research_ops_paths,
        &mut config.conservative_leaderboard_paths,
    ] {
        for path in paths.iter_mut() {
            *path = absolutize(path).display().to_string();
        }
    }
}

pub fn sprint92_output_path(name: &str) -> PathBuf {
    project_root()
        .join("target")
        .join("sprint92-tests")
        .join(name)
}

pub fn sprint92_output_dir(name: &str) -> PathBuf {
    let path = sprint92_output_path(name);
    fs::create_dir_all(&path).expect("create sprint92 output dir");
    path
}

pub fn reset_sprint92_output_dir(name: &str) -> PathBuf {
    let path = sprint92_output_path(name);
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("reset sprint92 output dir");
    path
}

pub fn sprint92_config_from_example(
    name: &str,
    output_name: &str,
) -> KrxEvidenceWarningClosureConfig {
    let mut config = KrxEvidenceWarningClosureConfig::from_toml_path(&example_path(name))
        .expect("parse sprint92 krx warning closure config");
    for paths in [
        &mut config.sprint91_bundle_paths,
        &mut config.krx_evidence_reduction_paths,
        &mut config.krx_assertion_migration_paths,
        &mut config.krx_secret_safety_paths,
        &mut config.krx_raw_archive_paths,
        &mut config.krx_market_data_boundary_paths,
        &mut config.workspace_gate_paths,
    ] {
        for path in paths.iter_mut() {
            *path = absolutize(path).display().to_string();
        }
    }
    config.output_root = sprint92_output_dir(output_name).display().to_string();
    config
}

pub fn sprint93_output_path(name: &str) -> PathBuf {
    project_root()
        .join("target")
        .join("sprint93-tests")
        .join(name)
}

pub fn sprint93_output_dir(name: &str) -> PathBuf {
    let path = sprint93_output_path(name);
    fs::create_dir_all(&path).expect("create sprint93 output dir");
    path
}

pub fn reset_sprint93_output_dir(name: &str) -> PathBuf {
    let path = sprint93_output_path(name);
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("reset sprint93 output dir");
    path
}

pub fn sprint93_config_from_example(
    name: &str,
    output_name: &str,
) -> RealWorkspaceTimeoutAttributionConfig {
    let mut config = RealWorkspaceTimeoutAttributionConfig::from_toml_path(&example_path(name))
        .expect("parse sprint93 timeout attribution config");
    for paths in [
        &mut config.sprint92_bundle_paths,
        &mut config.krx_warning_closure_paths,
        &mut config.previous_no_run_attempt_paths,
        &mut config.previous_full_attempt_paths,
        &mut config.dashboard_renderer_precheck_paths,
        &mut config.cargo_metadata_paths,
        &mut config.cargo_message_paths,
        &mut config.rustc_process_snapshot_paths,
        &mut config.target_dir_snapshot_paths,
    ] {
        for path in paths.iter_mut() {
            *path = absolutize(path).display().to_string();
        }
    }
    config.output_root = sprint93_output_dir(output_name).display().to_string();
    config
}

pub fn run_sprint93_bundle(name: &str, output_name: &str) -> Sprint93TimeoutAttributionBundle {
    reset_sprint93_output_dir(output_name);
    let config = sprint93_config_from_example(name, output_name);
    Sprint93TimeoutAttributionRunner::default()
        .run(&config)
        .expect("run sprint93 bundle")
}

pub fn sprint94_output_path(name: &str) -> PathBuf {
    project_root()
        .join("target")
        .join("sprint94-tests")
        .join(name)
}

pub fn sprint94_output_dir(name: &str) -> PathBuf {
    let path = sprint94_output_path(name);
    fs::create_dir_all(&path).expect("create sprint94 output dir");
    path
}

pub fn reset_sprint94_output_dir(name: &str) -> PathBuf {
    let path = sprint94_output_path(name);
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("reset sprint94 output dir");
    path
}

pub fn sprint94_config_from_example(
    name: &str,
    output_name: &str,
) -> DashboardRendererRealReductionConfig {
    let mut config = DashboardRendererRealReductionConfig::from_toml_path(&example_path(name))
        .expect("parse sprint94 dashboard renderer config");
    for paths in [
        &mut config.sprint93_bundle_paths,
        &mut config.dashboard_renderer_suite_paths,
        &mut config.dashboard_renderer_paths,
        &mut config.dashboard_snapshot_paths,
        &mut config.control_tower_panel_paths,
        &mut config.dashboard_secret_redaction_paths,
        &mut config.cargo_metadata_paths,
        &mut config.cargo_timings_paths,
    ] {
        for path in paths.iter_mut() {
            *path = absolutize(path).display().to_string();
        }
    }
    config.output_root = sprint94_output_dir(output_name).display().to_string();
    config
}

pub fn run_sprint94_bundle(
    name: &str,
    output_name: &str,
) -> Sprint94DashboardRendererRecoveryBundle {
    reset_sprint94_output_dir(output_name);
    let config = sprint94_config_from_example(name, output_name);
    Sprint94DashboardRendererRecoveryRunner::default()
        .run(&config)
        .expect("run sprint94 bundle")
}

pub fn sprint95_output_path(name: &str) -> PathBuf {
    project_root()
        .join("target")
        .join("sprint95-tests")
        .join(name)
}

pub fn sprint95_output_dir(name: &str) -> PathBuf {
    let path = sprint95_output_path(name);
    fs::create_dir_all(&path).expect("create sprint95 output dir");
    path
}

pub fn reset_sprint95_output_dir(name: &str) -> PathBuf {
    let path = sprint95_output_path(name);
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("reset sprint95 output dir");
    path
}

pub fn sprint95_config_from_example(
    name: &str,
    output_name: &str,
) -> CommitteeCliSafetyReductionConfig {
    let mut config = CommitteeCliSafetyReductionConfig::from_toml_path(&example_path(name))
        .expect("parse sprint95 committee cli safety config");
    for paths in [
        &mut config.sprint94_bundle_paths,
        &mut config.committee_cli_safety_paths,
        &mut config.workspace_cli_safety_suite_paths,
        &mut config.workspace_safety_guard_suite_paths,
        &mut config.command_surface_manifest_paths,
        &mut config.help_text_fixture_paths,
        &mut config.cargo_metadata_paths,
        &mut config.cargo_timings_paths,
    ] {
        for path in paths.iter_mut() {
            *path = absolutize(path).display().to_string();
        }
    }
    config.output_root = sprint95_output_dir(output_name).display().to_string();
    config
}

pub fn run_sprint95_bundle(
    name: &str,
    output_name: &str,
) -> Sprint95CommitteeCliSafetyRecoveryBundle {
    reset_sprint95_output_dir(output_name);
    let config = sprint95_config_from_example(name, output_name);
    Sprint95CommitteeCliSafetyRecoveryRunner::default()
        .run(&config)
        .expect("run sprint95 bundle")
}

pub fn sprint96_output_path(name: &str) -> PathBuf {
    project_root()
        .join("target")
        .join("sprint96-tests")
        .join(name)
}

pub fn sprint96_output_dir(name: &str) -> PathBuf {
    let path = sprint96_output_path(name);
    fs::create_dir_all(&path).expect("create sprint96 output dir");
    path
}

pub fn reset_sprint96_output_dir(name: &str) -> PathBuf {
    let path = sprint96_output_path(name);
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("reset sprint96 output dir");
    path
}

pub fn sprint96_config_from_example(
    name: &str,
    output_name: &str,
) -> BaselineSignalRealReductionConfig {
    let mut config = BaselineSignalRealReductionConfig::from_toml_path(&example_path(name))
        .expect("parse sprint96 baseline signal config");
    for paths in [
        &mut config.sprint95_bundle_paths,
        &mut config.baseline_signal_suite_paths,
        &mut config.baseline_signal_paths,
        &mut config.no_trade_fixture_paths,
        &mut config.risk_governor_fixture_paths,
        &mut config.feature_regime_fixture_paths,
        &mut config.data_quality_fixture_paths,
        &mut config.cargo_metadata_paths,
        &mut config.cargo_timings_paths,
    ] {
        for path in paths.iter_mut() {
            *path = absolutize(path).display().to_string();
        }
    }
    config.output_root = sprint96_output_dir(output_name).display().to_string();
    config
}

pub fn run_sprint96_bundle(name: &str, output_name: &str) -> Sprint96BaselineSignalRecoveryBundle {
    reset_sprint96_output_dir(output_name);
    let config = sprint96_config_from_example(name, output_name);
    Sprint96BaselineSignalRecoveryRunner::default()
        .run(&config)
        .expect("run sprint96 bundle")
}

pub fn run_sprint92_bundle(name: &str, output_name: &str) -> Sprint92KrxWarningClosureBundle {
    reset_sprint92_output_dir(output_name);
    let config = sprint92_config_from_example(name, output_name);
    Sprint92KrxWarningClosureRunner::default()
        .run(&config)
        .expect("run sprint92 bundle")
}

pub fn sprint97_output_path(name: &str) -> PathBuf {
    project_root()
        .join("target")
        .join("sprint97-tests")
        .join(name)
}

pub fn sprint97_output_dir(name: &str) -> PathBuf {
    let path = sprint97_output_path(name);
    fs::create_dir_all(&path).expect("create sprint97 output dir");
    path
}

pub fn reset_sprint97_output_dir(name: &str) -> PathBuf {
    let path = sprint97_output_path(name);
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("reset sprint97 output dir");
    path
}

pub fn sprint97_config_from_example(
    name: &str,
    output_name: &str,
) -> CounterfactualBackfillRealReductionConfig {
    let mut config = CounterfactualBackfillRealReductionConfig::from_toml_path(&example_path(name))
        .expect("parse sprint97 counterfactual config");
    for paths in [
        &mut config.sprint96_bundle_paths,
        &mut config.counterfactual_backfill_suite_paths,
        &mut config.counterfactual_backfill_paths,
        &mut config.no_trade_fixture_paths,
        &mut config.risk_denied_fixture_paths,
        &mut config.defensive_value_fixture_paths,
        &mut config.opportunity_cost_fixture_paths,
        &mut config.no_fabricated_outcome_paths,
        &mut config.no_lookahead_fixture_paths,
        &mut config.source_boundary_paths,
        &mut config.cargo_metadata_paths,
        &mut config.cargo_timings_paths,
    ] {
        for path in paths.iter_mut() {
            *path = absolutize(path).display().to_string();
        }
    }
    config.output_root = sprint97_output_dir(output_name).display().to_string();
    config
}

pub fn run_sprint97_bundle(
    name: &str,
    output_name: &str,
) -> Sprint97CounterfactualBackfillRecoveryBundle {
    reset_sprint97_output_dir(output_name);
    let config = sprint97_config_from_example(name, output_name);
    Sprint97CounterfactualBackfillRecoveryRunner::default()
        .run(&config)
        .expect("run sprint97 bundle")
}
