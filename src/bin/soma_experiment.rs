use clap::{Parser, Subcommand};

use soma_zero::{
    AblationRunner, AblationStudyConfig, AdjustedPricePolicy, AssetClass, AuthConfig,
    BalancedOutcomeCoverageConfig, BalancedOutcomeCoverageRunner, BarrierProfileRegistryBuilder,
    BarrierProfileRegistryConfig, BaselineSnapshotCoverageConfig, BaselineSnapshotCoverageRunner,
    BatchCounterfactualCompletionConfig, BatchCounterfactualCompletionRunner,
    BatchExperimentRunner, BatchOutcomeLinkageV3Config, BatchOutcomeLinkageV3Runner,
    BoundedKISOfficialEvidenceClosureRunner, BudgetPreference, CandleCoverageClosureConfig,
    CandleCoverageClosureRunner, CandleCoverageMatchOptions, CollectionSizePolicy,
    CommitteeBenchmarkConfig, CommitteeBenchmarkRunner, CommitteeCounterfactualAuditConfig,
    CommitteeCounterfactualAuditRunner, CommitteeCycleConfig, CommitteeDebateReplay,
    CommitteeDiagnosticsConfig, CommitteeDiagnosticsRunner, CommitteeMaterializationConfig,
    CommitteeOfficialBenchmarkConfig, CommitteeOfficialBenchmarkRunner,
    CommitteeOutcomeCoverageConfig, CommitteeOutcomeCoverageRunner, CommitteeOutcomeLinker,
    CommitteeOutcomeLinkerConfig, CommitteeReferencePackConfig, CommitteeReferencePackRunner,
    CommitteeReplayConfig, CommitteeScenarioLoadConfig, CommitteeScenarioLoader,
    CommitteeScenarioMaterializerV2, CommitteeSmokeTestConfig, CommitteeSmokeTestRunner,
    CommitteeV1RunConfig, CommitteeV1Runner, ComparableCommitteeEvidenceConfig,
    ComparableEvidenceBackfillConfig, ComparableEvidenceBackfillRunner, ComparableEvidenceBuilder,
    CompleteRowClosureConfig, CompleteRowClosureRunner, CompleteRowClosureV2Config,
    CompleteRowClosureV2Runner, ControlTowerAutoRefreshConfig, ControlTowerAutoRefreshRunner,
    ControlTowerRefreshConfig, ControlTowerRefreshRunner, ControlTowerV1Builder,
    ControlTowerV1Config, CoreCheckConfig, CoreCheckRunner, CoreCheckedBenchmarkConfig,
    CoreCheckedBenchmarkRunner, CorePerformanceRegressionConfig, CorePerformanceScorecardConfig,
    CorePerformanceScorecardRunner, CoreScorecardRerun, CounterfactualCompletionV2Config,
    CounterfactualCompletionV2Runner, CounterfactualDepthClosureConfig,
    CounterfactualDepthClosureRunner, CounterfactualDepthPlan, DashboardRenderConfig,
    DashboardRenderer, DashboardServeReport, DashboardSnapshotBuilder, DashboardSourceConfig,
    DashboardV1Renderer, DeterministicArtifactDiffConfig, DiversityAwareSufficiencyV2Config,
    DiversityAwareSufficiencyV2Runner, EnvironmentIsolationConfig, EnvironmentIsolationRunner,
    EvidenceClosureConfig, EvidenceClosureRunner, EvidenceGapClosureRunnerV2,
    EvidenceGapClosureV2Config, EvidenceHardeningConfig, EvidenceHardeningRunner, ExperimentConfig,
    ExperimentMatrixConfig, ExperimentMode, ExperimentRunner, ExtModelBPredictionClosureConfig,
    ExtModelBPredictionClosureRunner, ExternalArtifactRegistryRunner,
    ExternalModelArtifactRegistryConfig, ExternalModelResearchOpsConfig,
    ExternalModelResearchOpsRunner, ExternalPredictionEvaluationRunner,
    ExternalPredictionImportV2Config, FillMissingPolicy, FutureWindowRequirementConfig,
    FutureWindowRequirementRunner, FutureWindowScaleOutConfig, FutureWindowScaleOutPlanner,
    KISAuthClosureConfig, KISAuthClosureRunner, KISCollectionPlanV2Config,
    KISCollectionPlanV2Runner, KISEvidenceClosureConfig, KISEvidenceDepthRunConfig,
    KISEvidenceDepthRunRunner, KISEvidenceExpansionPlanV2Config, KISMarketDataActivationConfig,
    KISMarketDataDryRunConfig, KISMarketDataDryRunRunner, KISMarketDataEvidenceSmokeConfig,
    KISMarketDataEvidenceSmokeRunner, KISOfficialMarketDataActivationRunner,
    KISOutcomeLinkClosureConfig, KISOutcomeLinkClosureRunner, KISSymbolWhitelistConfig,
    KRXBoundedCollectionSmokeConfig, KRXOfficialCollectionClosureConfig,
    KRXOfficialCollectionClosureRunner, KRXOfficialEvidenceActivationConfig,
    KRXOfficialEvidenceActivationRunner, KRXOutcomeLinkClosureConfig, KRXOutcomeLinkClosureRunner,
    KRXSymbolWhitelistConfig, KrxSnapshotImportConfig, KrxSnapshotImporter,
    LocalDataOnboardingConfig, MambaReadinessConfig, MambaReadinessRunner,
    MarketDataProviderCatalog, MarketVenue, ModelOpsReviewClosureConfig, ModelOpsRollupConfig,
    ModelOpsRollupRunner, ModelOpsTraceConfig, ModelOpsTraceRunner, ModelReviewClosureRunner,
    MultiRowOfficialEvidenceSetBuilder, MultiRowOfficialEvidenceSetConfig,
    OfficialAiBenchmarkConfig, OfficialAiBenchmarkRunner, OfficialCandleCoverageGapMap,
    OfficialCandleCoveragePack, OfficialCandleCoveragePackConfig,
    OfficialCandleExpansionPlanConfig, OfficialCandleExpansionRunner, OfficialCandleGapConfig,
    OfficialCandleJoinAuditConfig, OfficialCandleJoinAuditRunner, OfficialCollectionPlan,
    OfficialCollectionReport, OfficialCollectionRunner, OfficialCommitteeScenarioPackBuilder,
    OfficialCommitteeScenarioPackConfig, OfficialDiversityRowSelector,
    OfficialDiversityRowSelectorConfig, OfficialEvidenceAcquisitionPlan,
    OfficialEvidenceAcquisitionReport, OfficialEvidenceAcquisitionRunner,
    OfficialEvidenceDiversityGapConfig, OfficialEvidenceDiversityGapRunner,
    OfficialEvidenceDiversitySweepConfig, OfficialEvidenceDiversitySweepRunner,
    OfficialEvidenceExpansionConfig, OfficialEvidenceExpansionRunner,
    OfficialEvidenceReplicationConfig, OfficialEvidenceReplicationRunner,
    OfficialEvidenceRunConfig, OfficialEvidenceRunner, OfficialEvidenceScaleOutConfig,
    OfficialEvidenceScaleOutRunner, OfficialEvidenceSufficiencyV2Config,
    OfficialEvidenceSufficiencyV2Runner, OfficialFutureWindowExtensionConfig,
    OfficialProviderReadinessConfig, OfficialProviderReadinessRunner,
    OfficialReadyMatchClosureConfig, OfficialReadyMatchClosureRunner,
    OfficialReadyRowInventoryConfig, OfficialReadyRowInventoryRunner,
    OfflineEvidenceAttachmentConfig, OfflineEvidenceAttachmentRunner, OperationalRunbookConfig,
    OperationalRunbookRunner, OperationalRunbookV2Config, OperationalRunbookV2Runner,
    OperatorBriefingConfig, OperatorBriefingRunner, OutcomeDiversityAuditConfig,
    OutcomeDiversityAuditRunner, OutcomeLinkDepthClosureV2Config, OutcomeLinkageV3Config,
    OutcomeLinkageV3Runner, OwnerApplyInputConfig, OwnerChecklistClosureConfig,
    OwnerChecklistClosureRunner, OwnerImpactReportConfig, OwnerInputValidateConfig,
    OwnerReviewDisciplineV2Config, OwnerReviewQueueConfig, OwnerThesisBookConfig,
    PredictionHistoryExpansionConfig, PredictionHistoryExpansionRunner,
    PredictionHistoryPackConfig, PreflightValidator, ProviderAuthPreflightConfig,
    ProviderAuthPreflightRunner, ProviderKind, ProviderMarket, ProviderRealityConfig,
    ProviderRealityRunner, ProviderRecommendationRequest, ProviderSimplificationConfig,
    ProviderSimplificationRunner, RawArchivePolicy, RealEvidenceClosureConfig,
    RealEvidenceClosureRunner, RealEvidenceFollowupConfig, RealEvidenceFollowupRunner,
    RealEvidencePredictionRefreshConfig, RealEvidencePredictionRefreshRunner, RequestedOutputSize,
    ResearchCampaignConfig, ResearchCampaignReport, ResearchCampaignRunner,
    RetirementRegressionEvidencePackConfig, RustToolchainModernizationConfig,
    RustToolchainModernizationRunner, ScenarioMaterializationV3Config,
    ScenarioMaterializationV3Runner, SecretRedactionAuditConfig, SecretRedactionAuditRunner,
    SequenceDatasetDriftGuardConfig, SequenceDatasetExportConfig, SequenceDatasetExportRunner,
    SequenceDatasetPreparationConfig, SourceAwareBenchmarkConfig, SourceAwareBenchmarkRunner,
    Sprint14Runner, StrategyUseCase, SufficiencyClosureConfig, SufficiencyClosureRunner,
    SystemIntegrationReviewConfig, SystemIntegrationReviewRunner, Timeframe,
    TrinityCommitteeOperationalLoopConfig, TrinityOperationalLoopRunner,
    UnexpectedDiffTriageConfig, UnexpectedDiffTriageRunner, VenueCoverageExpansionPlan,
    YFinanceImportConfig, YahooResearchEvidenceConfig, YahooResearchEvidenceReport,
    YahooResearchEvidenceRunner, ablation_report_to_text, active_persona_cards_lite,
    build_candle_coverage_match_computation, build_default_provider_catalog,
    build_join_repair_plan, build_official_candle_acquisition_plan,
    build_official_future_window_extension_plan, build_official_vs_yfinance_interpretation,
    build_real_evidence_rerun_plan, core_checked_benchmark_report_to_text, diff_report_to_text,
    evaluate_strategy_data_compatibility, evidence_closure_report_to_text,
    generate_owner_action_draft_bundle, kis_auth_readiness_report_to_text,
    kis_candle_sufficiency_to_text, kis_collection_batch_plan_to_text, kis_endpoint_policy_to_text,
    kis_krx_migration_to_text, kis_official_activation_report_to_text,
    kis_outcome_link_closure_to_text, kis_symbol_whitelist_to_text,
    krx_auth_readiness_report_to_text, krx_candle_sufficiency_to_text,
    krx_collection_batch_plan_to_text, krx_collection_closure_report_to_text,
    krx_collection_dry_run_to_text, krx_evidence_job_plan_to_text,
    krx_official_activation_report_to_text, krx_outcome_link_closure_to_text,
    krx_symbol_whitelist_to_text, load_pack_from_path_or_config,
    official_ai_benchmark_report_to_text, official_evidence_acquisition_report_to_text,
    official_evidence_expansion_report_to_text, official_vs_yfinance_to_text,
    parse_provider_subject, prepare_dashboard_open, provider_auth_preflight_report_to_text,
    provider_readiness_report_to_text, provider_reality_report_to_text,
    real_evidence_report_to_text, recommend_provider, run_candidate_generation_only,
    run_committee_cycle_from_config, run_deterministic_artifact_diff, run_owner_apply_input,
    run_owner_impact_report, run_owner_input_validation, run_owner_review_queue,
    run_owner_thesis_book, source_aware_benchmark_report_to_text, sprint14_report_to_text,
    venue_coverage_report_to_text, yahoo_research_report_to_text,
};
use soma_zero::{
    CandleExpansionRealReductionConfig, CliSmokeCostReductionConfig, CommitteeCompletionGateConfig,
    CommitteeEvidenceExpansionConfig, CommitteeReferenceClosureConfig,
    CommitteeReferenceClosureRunner, CoreCommitteeMambaReadinessRunner, CoreCompletionV2Config,
    DashboardRendererRealReductionConfig, ExternalPredictionRealReductionConfig,
    KrxEvidenceRealReductionConfig, KrxEvidenceWarningClosureConfig,
    OfficialEvidenceDepthExpansionConfig, OfficialEvidenceDepthExpansionRunner,
    PrototypeComparisonInterpretationConfig, PrototypeComparisonInterpretationRunner,
    RealWorkspaceTimeoutAttributionConfig, RepeatedWorkspaceTimingConfig,
    ResidualWorkspaceBinaryAuditConfig, SequenceCoreCandidateRegistryConfig,
    SequenceCorePrototypeComparisonConfig, SequenceCorePrototypeComparisonRunner,
    SequenceCoreStorageMaterializationRunner, SevenBlockerFamilyRecoveryConfig,
    Sprint83AcceptanceRecoveryConfig, Sprint83AcceptanceRecoveryRunner,
    Sprint84TestCostReductionRunner, Sprint85WorkspaceGateRecoveryRunner,
    Sprint86ResidualGateRecoveryRunner, Sprint87CompileGateRecoveryRunner,
    Sprint88SevenBlockerRecoveryRunner, Sprint89CandleRecoveryRunner,
    Sprint90ExternalPredictionRecoveryRunner, Sprint91KrxEvidenceRecoveryRunner,
    Sprint92KrxWarningClosureRunner, Sprint93TimeoutAttributionRunner,
    Sprint94DashboardRendererRecoveryRunner, TestBinaryConsolidationConfig, TestOptimizationRunner,
    TrainingDataArtifactPopulationConfig, TrainingDataStorageConfig,
    TrainingDataStorageMaterializationConfig, WorkspaceCompileGraphAuditConfig,
    WorkspaceWideTestSurfaceAuditConfig,
};

use soma_zero::{
    BaselineSignalRealReductionConfig, CommitteeCliSafetyReductionConfig,
    CommitteeQualityHardeningConfig, CommitteeQualityWarningClosureConfig,
    CounterfactualBackfillRealReductionConfig, Sprint95CommitteeCliSafetyRecoveryRunner,
    Sprint96BaselineSignalRecoveryRunner, Sprint97CounterfactualBackfillRecoveryRunner,
    Sprint98CommitteeOwnedCoreConfig, Sprint98CommitteeOwnedCoreRunner,
    Sprint99CommitteeQualityHardeningBundle, Sprint99CommitteeQualityHardeningRunner,
    Sprint100CommitteeClosureBundle, Sprint100CommitteeClosureRunner,
};
use soma_zero::{
    ConsolidationStopResumeGovernanceBundle, ConsolidationStopResumeGovernanceConfig,
    ConsolidationStopResumeGovernanceRunner,
};
use soma_zero::{
    DeferredRealObservationExecutionBundle, DeferredRealObservationExecutionConfig,
    DeferredRealObservationExecutionRunner,
};
use soma_zero::{
    DualAgentWorkflowConfig, Sprint104DualAgentPaperLifecycleBundle,
    Sprint104DualAgentPaperLifecycleRunner,
};
use soma_zero::{
    EighteenArchetypePaperRotationConfig, Sprint102PaperRotationBundle,
    Sprint102PaperRotationRunner,
};
use soma_zero::{
    InvestorArchetypeIngestionConfig, Sprint101InvestorArchetypeIngestionBundle,
    Sprint101InvestorArchetypeIngestionRunner,
};
use soma_zero::{
    MinimalAiCommitteeCycleConfig, run_autonomous_paper_committee_loop_from_config_path,
    run_batch_committee_cycle_from_config_path,
    run_batch_committee_cycle_with_state_from_config_path,
    run_minimal_committee_cycle_from_config_path,
};
use soma_zero::{
    MixedFamilyIsolationV1Bundle, MixedFamilyIsolationV1Config, MixedFamilyIsolationV1Runner,
};
use soma_zero::{
    PaperRotationWarningClosureConfig, Sprint103PaperRotationClosureBundle,
    Sprint103PaperRotationClosureRunner,
};
use soma_zero::{
    RealWorkspaceObservationDrilldownBundle, RealWorkspaceObservationDrilldownConfig,
    RealWorkspaceObservationDrilldownRunner,
};
use soma_zero::{
    SafeConsolidationPatchV1Bundle, SafeConsolidationPatchV1Config, SafeConsolidationPatchV1Runner,
};
use soma_zero::{
    SafeConsolidationPatchV2Bundle, SafeConsolidationPatchV2Config, SafeConsolidationPatchV2Runner,
};
use soma_zero::{
    SafeConsolidationPatchV3Bundle, SafeConsolidationPatchV3Config, SafeConsolidationPatchV3Runner,
};
use soma_zero::{
    SafeConsolidationPatchV4Bundle, SafeConsolidationPatchV4Config, SafeConsolidationPatchV4Runner,
};
use soma_zero::{
    Sprint105VerificationPatchClosureBundle, Sprint105VerificationPatchClosureConfig,
    Sprint105VerificationPatchClosureRunner,
};
use soma_zero::{
    WorkspaceAcceptanceRecoveryV7Bundle, WorkspaceAcceptanceRecoveryV7Config,
    WorkspaceAcceptanceRecoveryV7Runner,
};
use soma_zero::{
    WorkspaceDiagnosticPilotV1Bundle, WorkspaceDiagnosticPilotV1Config,
    WorkspaceDiagnosticPilotV1Runner,
};
use soma_zero::{
    WorkspaceTimeoutReductionQueueBundle, WorkspaceTimeoutReductionQueueConfig,
    WorkspaceTimeoutReductionQueueRunner,
};
use soma_zero::{
    WorkspaceTimeoutRootCauseBundle, WorkspaceTimeoutRootCauseConfig,
    WorkspaceTimeoutRootCauseRunner,
};
use soma_zero::{
    WorkspaceTimeoutTrackExecutionBundle, WorkspaceTimeoutTrackExecutionConfig,
    WorkspaceTimeoutTrackExecutionRunner,
};

#[derive(Debug, Parser)]
#[command(
    name = "soma-experiment",
    about = "Research-only local experiment harness. No live trading, broker, or network commands."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

fn load_local_sprint88_config(
    config: &str,
    command_name: &str,
) -> Result<SevenBlockerFamilyRecoveryConfig, String> {
    if config.contains("://") {
        Err(format!("{command_name} config path must be local"))
    } else {
        SevenBlockerFamilyRecoveryConfig::from_toml_path(std::path::Path::new(config))
    }
}

fn load_local_sprint89_config(
    config: &str,
    command_name: &str,
) -> Result<CandleExpansionRealReductionConfig, String> {
    if config.contains("://") {
        Err(format!("{command_name} config path must be local"))
    } else {
        CandleExpansionRealReductionConfig::from_toml_path(std::path::Path::new(config))
    }
}

fn load_local_sprint90_config(
    config: &str,
    command_name: &str,
) -> Result<ExternalPredictionRealReductionConfig, String> {
    if config.contains("://") {
        Err(format!("{command_name} config path must be local"))
    } else {
        ExternalPredictionRealReductionConfig::from_toml_path(std::path::Path::new(config))
    }
}

fn load_local_sprint91_config(
    config: &str,
    command_name: &str,
) -> Result<KrxEvidenceRealReductionConfig, String> {
    if config.contains("://") {
        Err(format!("{command_name} config path must be local"))
    } else {
        KrxEvidenceRealReductionConfig::from_toml_path(std::path::Path::new(config))
    }
}

fn load_local_sprint92_config(
    config: &str,
    command_name: &str,
) -> Result<KrxEvidenceWarningClosureConfig, String> {
    if config.contains("://") {
        Err(format!("{command_name} config path must be local"))
    } else {
        KrxEvidenceWarningClosureConfig::from_toml_path(std::path::Path::new(config))
    }
}

fn load_local_sprint93_config(
    config: &str,
    command_name: &str,
) -> Result<RealWorkspaceTimeoutAttributionConfig, String> {
    if config.contains("://") {
        Err(format!("{command_name} config path must be local"))
    } else {
        RealWorkspaceTimeoutAttributionConfig::from_toml_path(std::path::Path::new(config))
    }
}

fn load_local_sprint94_config(
    config: &str,
    command_name: &str,
) -> Result<DashboardRendererRealReductionConfig, String> {
    if config.contains("://") {
        Err(format!("{command_name} config path must be local"))
    } else {
        DashboardRendererRealReductionConfig::from_toml_path(std::path::Path::new(config))
    }
}

fn load_local_sprint95_config(
    config: &str,
    command_name: &str,
) -> Result<CommitteeCliSafetyReductionConfig, String> {
    if config.contains("://") {
        Err(format!("{command_name} config path must be local"))
    } else {
        CommitteeCliSafetyReductionConfig::from_toml_path(std::path::Path::new(config))
    }
}

fn load_local_sprint96_config(
    config: &str,
    command_name: &str,
) -> Result<BaselineSignalRealReductionConfig, String> {
    if config.contains("://") {
        Err(format!("{command_name} config path must be local"))
    } else {
        BaselineSignalRealReductionConfig::from_toml_path(std::path::Path::new(config))
    }
}

fn load_local_sprint97_config(
    config: &str,
    command_name: &str,
) -> Result<CounterfactualBackfillRealReductionConfig, String> {
    if config.contains("://") {
        Err(format!("{command_name} config path must be local"))
    } else {
        CounterfactualBackfillRealReductionConfig::from_toml_path(std::path::Path::new(config))
    }
}

fn load_local_sprint98_config(
    config: &str,
    command_name: &str,
) -> Result<Sprint98CommitteeOwnedCoreConfig, String> {
    if config.contains("://") {
        Err(format!("{command_name} config path must be local"))
    } else {
        Sprint98CommitteeOwnedCoreConfig::from_toml_path(std::path::Path::new(config))
    }
}

fn load_local_sprint99_config(
    config: &str,
    command_name: &str,
) -> Result<CommitteeQualityHardeningConfig, String> {
    if config.contains("://") {
        Err(format!("{command_name} config path must be local"))
    } else {
        CommitteeQualityHardeningConfig::from_toml_path(std::path::Path::new(config))
    }
}

fn run_sprint99_bundle(
    config: &str,
    command_name: &str,
) -> Result<Sprint99CommitteeQualityHardeningBundle, String> {
    load_local_sprint99_config(config, command_name).and_then(|config| {
        Sprint99CommitteeQualityHardeningRunner::default()
            .run_sprint99_committee_quality_hardening(&config)
    })
}

fn load_local_sprint100_config(
    config: &str,
    command_name: &str,
) -> Result<CommitteeQualityWarningClosureConfig, String> {
    if config.contains("://") {
        Err(format!("{command_name} config path must be local"))
    } else {
        CommitteeQualityWarningClosureConfig::from_toml_path(std::path::Path::new(config))
    }
}

fn run_sprint100_bundle(
    config: &str,
    command_name: &str,
) -> Result<Sprint100CommitteeClosureBundle, String> {
    load_local_sprint100_config(config, command_name).and_then(|config| {
        Sprint100CommitteeClosureRunner::default().run_sprint100_committee_closure(&config)
    })
}

fn load_local_sprint101_config(
    config: &str,
    command_name: &str,
) -> Result<InvestorArchetypeIngestionConfig, String> {
    if config.contains("://") {
        Err(format!("{command_name} config path must be local"))
    } else {
        InvestorArchetypeIngestionConfig::from_toml_path(std::path::Path::new(config))
    }
}

fn run_sprint101_bundle(
    config: &str,
    command_name: &str,
) -> Result<Sprint101InvestorArchetypeIngestionBundle, String> {
    load_local_sprint101_config(config, command_name).and_then(|config| {
        Sprint101InvestorArchetypeIngestionRunner::default()
            .run_sprint101_investor_archetype_ingestion(&config)
    })
}

fn print_sprint101_report<T: serde::Serialize>(
    config: &str,
    command_name: &str,
    warning: &str,
    select: impl FnOnce(Sprint101InvestorArchetypeIngestionBundle) -> T,
) -> Result<(), String> {
    run_sprint101_bundle(config, command_name).and_then(|report| {
        let value = select(report);
        print_json_report(warning, &value)
    })
}

fn print_json_report<T: serde::Serialize>(warning: &str, report: &T) -> Result<(), String> {
    println!("{warning}");
    println!(
        "{}",
        serde_json::to_string_pretty(report).map_err(|err| err.to_string())?
    );
    Ok(())
}

fn load_local_sprint102_config(
    config: &str,
    command_name: &str,
) -> Result<EighteenArchetypePaperRotationConfig, String> {
    if config.contains("://") {
        Err(format!("{command_name} config path must be local"))
    } else {
        EighteenArchetypePaperRotationConfig::from_toml_path(std::path::Path::new(config))
    }
}

fn run_sprint102_bundle(
    config: &str,
    command_name: &str,
) -> Result<Sprint102PaperRotationBundle, String> {
    load_local_sprint102_config(config, command_name).and_then(|config| {
        Sprint102PaperRotationRunner::default().run_sprint102_paper_rotation(&config)
    })
}

fn print_sprint102_report<T: serde::Serialize>(
    config: &str,
    command_name: &str,
    warning: &str,
    select: impl FnOnce(Sprint102PaperRotationBundle) -> T,
) -> Result<(), String> {
    run_sprint102_bundle(config, command_name).and_then(|report| {
        let value = select(report);
        print_json_report(warning, &value)
    })
}

fn load_local_sprint103_config(
    config: &str,
    command_name: &str,
) -> Result<PaperRotationWarningClosureConfig, String> {
    if config.contains("://") {
        Err(format!("{command_name} config path must be local"))
    } else {
        PaperRotationWarningClosureConfig::from_toml_path(std::path::Path::new(config))
    }
}

fn run_sprint103_bundle(
    config: &str,
    command_name: &str,
) -> Result<Sprint103PaperRotationClosureBundle, String> {
    load_local_sprint103_config(config, command_name)
        .and_then(|config| Sprint103PaperRotationClosureRunner::default().run(&config))
}

fn print_sprint103_report<T: serde::Serialize>(
    config: &str,
    command_name: &str,
    warning: &str,
    select: impl FnOnce(Sprint103PaperRotationClosureBundle) -> T,
) -> Result<(), String> {
    run_sprint103_bundle(config, command_name).and_then(|report| {
        let value = select(report);
        print_json_report(warning, &value)
    })
}

fn load_local_sprint104_config(
    config: &str,
    command_name: &str,
) -> Result<DualAgentWorkflowConfig, String> {
    if config.contains("://") {
        Err(format!("{command_name} config path must be local"))
    } else {
        DualAgentWorkflowConfig::from_toml_path(std::path::Path::new(config))
    }
}

fn run_sprint104_bundle(
    config: &str,
    command_name: &str,
) -> Result<Sprint104DualAgentPaperLifecycleBundle, String> {
    load_local_sprint104_config(config, command_name)
        .and_then(|config| Sprint104DualAgentPaperLifecycleRunner::default().run(&config))
}

fn print_sprint104_report<T: serde::Serialize>(
    config: &str,
    command_name: &str,
    warning: &str,
    select: impl FnOnce(Sprint104DualAgentPaperLifecycleBundle) -> T,
) -> Result<(), String> {
    run_sprint104_bundle(config, command_name).and_then(|report| {
        let value = select(report);
        print_json_report(warning, &value)
    })
}

fn load_local_sprint105_config(
    config: &str,
    command_name: &str,
) -> Result<Sprint105VerificationPatchClosureConfig, String> {
    if config.contains("://") {
        Err(format!("{command_name} config path must be local"))
    } else {
        Sprint105VerificationPatchClosureConfig::from_toml_path(std::path::Path::new(config))
    }
}

fn run_sprint105_bundle(
    config: &str,
    command_name: &str,
) -> Result<Sprint105VerificationPatchClosureBundle, String> {
    load_local_sprint105_config(config, command_name)
        .and_then(|config| Sprint105VerificationPatchClosureRunner::default().run(&config))
}

fn print_sprint105_report<T: serde::Serialize>(
    config: &str,
    command_name: &str,
    warning: &str,
    select: impl FnOnce(Sprint105VerificationPatchClosureBundle) -> T,
) -> Result<(), String> {
    run_sprint105_bundle(config, command_name).and_then(|report| {
        let value = select(report);
        print_json_report(warning, &value)
    })
}

fn load_local_sprint106_config(
    config: &str,
    command_name: &str,
) -> Result<WorkspaceAcceptanceRecoveryV7Config, String> {
    if config.contains("://") {
        Err(format!("{command_name} config path must be local"))
    } else {
        WorkspaceAcceptanceRecoveryV7Config::from_toml_path(std::path::Path::new(config))
    }
}

fn run_sprint106_bundle(
    config: &str,
    command_name: &str,
) -> Result<WorkspaceAcceptanceRecoveryV7Bundle, String> {
    load_local_sprint106_config(config, command_name)
        .and_then(|config| WorkspaceAcceptanceRecoveryV7Runner::default().run(&config))
}

fn print_sprint106_report<T: serde::Serialize>(
    config: &str,
    command_name: &str,
    warning: &str,
    select: impl FnOnce(WorkspaceAcceptanceRecoveryV7Bundle) -> T,
) -> Result<(), String> {
    run_sprint106_bundle(config, command_name).and_then(|report| {
        let value = select(report);
        print_json_report(warning, &value)
    })
}

fn load_local_sprint107_config(
    config: &str,
    command_name: &str,
) -> Result<SafeConsolidationPatchV1Config, String> {
    if config.contains("://") {
        Err(format!("{command_name} config path must be local"))
    } else {
        SafeConsolidationPatchV1Config::from_toml_path(std::path::Path::new(config))
    }
}

fn run_sprint107_bundle(
    config: &str,
    command_name: &str,
) -> Result<SafeConsolidationPatchV1Bundle, String> {
    load_local_sprint107_config(config, command_name)
        .and_then(|config| SafeConsolidationPatchV1Runner::default().run(&config))
}

fn print_sprint107_report<T: serde::Serialize>(
    config: &str,
    command_name: &str,
    warning: &str,
    select: impl FnOnce(SafeConsolidationPatchV1Bundle) -> T,
) -> Result<(), String> {
    run_sprint107_bundle(config, command_name).and_then(|report| {
        let value = select(report);
        print_json_report(warning, &value)
    })
}

fn load_local_sprint108_config(
    config: &str,
    command_name: &str,
) -> Result<SafeConsolidationPatchV2Config, String> {
    if config.contains("://") {
        Err(format!("{command_name} config path must be local"))
    } else {
        SafeConsolidationPatchV2Config::from_toml_path(std::path::Path::new(config))
    }
}

fn run_sprint108_bundle(
    config: &str,
    command_name: &str,
) -> Result<SafeConsolidationPatchV2Bundle, String> {
    load_local_sprint108_config(config, command_name)
        .and_then(|config| SafeConsolidationPatchV2Runner::default().run(&config))
}

fn print_sprint108_report<T: serde::Serialize>(
    config: &str,
    command_name: &str,
    warning: &str,
    select: impl FnOnce(SafeConsolidationPatchV2Bundle) -> T,
) -> Result<(), String> {
    run_sprint108_bundle(config, command_name).and_then(|report| {
        let value = select(report);
        print_json_report(warning, &value)
    })
}

fn load_local_sprint109_config(
    config: &str,
    command_name: &str,
) -> Result<SafeConsolidationPatchV3Config, String> {
    if config.contains("://") {
        Err(format!("{command_name} config path must be local"))
    } else {
        SafeConsolidationPatchV3Config::from_toml_path(std::path::Path::new(config))
    }
}

fn run_sprint109_bundle(
    config: &str,
    command_name: &str,
) -> Result<SafeConsolidationPatchV3Bundle, String> {
    load_local_sprint109_config(config, command_name)
        .and_then(|config| SafeConsolidationPatchV3Runner::default().run(&config))
}

fn print_sprint109_report<T: serde::Serialize>(
    config: &str,
    command_name: &str,
    warning: &str,
    select: impl FnOnce(SafeConsolidationPatchV3Bundle) -> T,
) -> Result<(), String> {
    run_sprint109_bundle(config, command_name).and_then(|report| {
        let value = select(report);
        print_json_report(warning, &value)
    })
}

fn load_local_sprint110_config(
    config: &str,
    command_name: &str,
) -> Result<SafeConsolidationPatchV4Config, String> {
    if config.contains("://") {
        Err(format!("{command_name} config path must be local"))
    } else {
        SafeConsolidationPatchV4Config::from_toml_path(std::path::Path::new(config))
    }
}

fn run_sprint110_bundle(
    config: &str,
    command_name: &str,
) -> Result<SafeConsolidationPatchV4Bundle, String> {
    load_local_sprint110_config(config, command_name)
        .and_then(|config| SafeConsolidationPatchV4Runner::default().run(&config))
}

fn print_sprint110_report<T: serde::Serialize>(
    config: &str,
    command_name: &str,
    warning: &str,
    select: impl FnOnce(SafeConsolidationPatchV4Bundle) -> T,
) -> Result<(), String> {
    run_sprint110_bundle(config, command_name).and_then(|report| {
        let value = select(report);
        print_json_report(warning, &value)
    })
}

fn load_local_sprint111_config(
    config: &str,
    command_name: &str,
) -> Result<WorkspaceTimeoutRootCauseConfig, String> {
    if config.contains("://") {
        Err(format!("{command_name} config path must be local"))
    } else {
        WorkspaceTimeoutRootCauseConfig::from_toml_path(std::path::Path::new(config))
    }
}

fn run_sprint111_bundle(
    config: &str,
    command_name: &str,
) -> Result<WorkspaceTimeoutRootCauseBundle, String> {
    load_local_sprint111_config(config, command_name)
        .and_then(|config| WorkspaceTimeoutRootCauseRunner::default().run(&config))
}

fn print_sprint111_report<T: serde::Serialize>(
    config: &str,
    command_name: &str,
    warning: &str,
    select: impl FnOnce(WorkspaceTimeoutRootCauseBundle) -> T,
) -> Result<(), String> {
    run_sprint111_bundle(config, command_name).and_then(|report| {
        let value = select(report);
        print_json_report(warning, &value)
    })
}

fn load_local_sprint112_config(
    config: &str,
    command_name: &str,
) -> Result<WorkspaceDiagnosticPilotV1Config, String> {
    if config.contains("://") {
        Err(format!("{command_name} config path must be local"))
    } else {
        WorkspaceDiagnosticPilotV1Config::from_toml_path(std::path::Path::new(config))
    }
}

fn run_sprint112_bundle(
    config: &str,
    command_name: &str,
) -> Result<WorkspaceDiagnosticPilotV1Bundle, String> {
    load_local_sprint112_config(config, command_name)
        .and_then(|config| WorkspaceDiagnosticPilotV1Runner::default().run(&config))
}

fn print_sprint112_report<T: serde::Serialize>(
    config: &str,
    command_name: &str,
    warning: &str,
    select: impl FnOnce(WorkspaceDiagnosticPilotV1Bundle) -> T,
) -> Result<(), String> {
    run_sprint112_bundle(config, command_name).and_then(|report| {
        let value = select(report);
        print_json_report(warning, &value)
    })
}

fn load_local_sprint113_config(
    config: &str,
    command_name: &str,
) -> Result<RealWorkspaceObservationDrilldownConfig, String> {
    if config.contains("://") {
        Err(format!("{command_name} config path must be local"))
    } else {
        RealWorkspaceObservationDrilldownConfig::from_toml_path(std::path::Path::new(config))
    }
}

fn run_sprint113_bundle(
    config: &str,
    command_name: &str,
) -> Result<RealWorkspaceObservationDrilldownBundle, String> {
    load_local_sprint113_config(config, command_name)
        .and_then(|config| RealWorkspaceObservationDrilldownRunner::default().run(&config))
}

fn print_sprint113_report<T: serde::Serialize>(
    config: &str,
    command_name: &str,
    warning: &str,
    select: impl FnOnce(RealWorkspaceObservationDrilldownBundle) -> T,
) -> Result<(), String> {
    run_sprint113_bundle(config, command_name).and_then(|report| {
        let value = select(report);
        print_json_report(warning, &value)
    })
}

fn load_local_sprint114_config(
    config: &str,
    command_name: &str,
) -> Result<MixedFamilyIsolationV1Config, String> {
    if config.contains("://") {
        Err(format!("{command_name} config path must be local"))
    } else {
        MixedFamilyIsolationV1Config::from_toml_path(std::path::Path::new(config))
    }
}

fn run_sprint114_bundle(
    config: &str,
    command_name: &str,
) -> Result<MixedFamilyIsolationV1Bundle, String> {
    load_local_sprint114_config(config, command_name)
        .and_then(|config| MixedFamilyIsolationV1Runner::default().run(&config))
}

fn print_sprint114_report<T: serde::Serialize>(
    config: &str,
    command_name: &str,
    warning: &str,
    select: impl FnOnce(MixedFamilyIsolationV1Bundle) -> T,
) -> Result<(), String> {
    run_sprint114_bundle(config, command_name).and_then(|report| {
        let value = select(report);
        print_json_report(warning, &value)
    })
}

fn load_local_sprint115_config(
    config: &str,
    command_name: &str,
) -> Result<ConsolidationStopResumeGovernanceConfig, String> {
    if config.contains("://") {
        Err(format!("{command_name} config path must be local"))
    } else {
        ConsolidationStopResumeGovernanceConfig::from_toml_path(std::path::Path::new(config))
    }
}

fn run_sprint115_bundle(
    config: &str,
    command_name: &str,
) -> Result<ConsolidationStopResumeGovernanceBundle, String> {
    load_local_sprint115_config(config, command_name)
        .and_then(|config| ConsolidationStopResumeGovernanceRunner::default().run(&config))
}

fn print_sprint115_report<T: serde::Serialize>(
    config: &str,
    command_name: &str,
    warning: &str,
    select: impl FnOnce(ConsolidationStopResumeGovernanceBundle) -> T,
) -> Result<(), String> {
    run_sprint115_bundle(config, command_name).and_then(|report| {
        let value = select(report);
        print_json_report(warning, &value)
    })
}

fn load_local_sprint116_config(
    config: &str,
    command_name: &str,
) -> Result<WorkspaceTimeoutTrackExecutionConfig, String> {
    if config.contains("://") {
        Err(format!("{command_name} config path must be local"))
    } else {
        WorkspaceTimeoutTrackExecutionConfig::from_toml_path(std::path::Path::new(config))
    }
}

fn run_sprint116_bundle(
    config: &str,
    command_name: &str,
) -> Result<WorkspaceTimeoutTrackExecutionBundle, String> {
    load_local_sprint116_config(config, command_name)
        .and_then(|config| WorkspaceTimeoutTrackExecutionRunner::default().run(&config))
}

fn print_sprint116_report<T: serde::Serialize>(
    config: &str,
    command_name: &str,
    warning: &str,
    select: impl FnOnce(WorkspaceTimeoutTrackExecutionBundle) -> T,
) -> Result<(), String> {
    run_sprint116_bundle(config, command_name).and_then(|report| {
        let value = select(report);
        print_json_report(warning, &value)
    })
}

fn load_local_sprint117_config(
    config: &str,
    command_name: &str,
) -> Result<DeferredRealObservationExecutionConfig, String> {
    if config.contains("://") {
        Err(format!("{command_name} config path must be local"))
    } else {
        DeferredRealObservationExecutionConfig::from_toml_path(std::path::Path::new(config))
    }
}

fn run_sprint117_bundle(
    config: &str,
    command_name: &str,
) -> Result<DeferredRealObservationExecutionBundle, String> {
    load_local_sprint117_config(config, command_name)
        .and_then(|config| DeferredRealObservationExecutionRunner::default().run(&config))
}

fn print_sprint117_report<T: serde::Serialize>(
    config: &str,
    command_name: &str,
    warning: &str,
    select: impl FnOnce(DeferredRealObservationExecutionBundle) -> T,
) -> Result<(), String> {
    run_sprint117_bundle(config, command_name).and_then(|report| {
        let value = select(report);
        print_json_report(warning, &value)
    })
}

fn load_local_sprint118_config(
    config: &str,
    command_name: &str,
) -> Result<WorkspaceTimeoutReductionQueueConfig, String> {
    if config.contains("://") {
        Err(format!("{command_name} config path must be local"))
    } else {
        WorkspaceTimeoutReductionQueueConfig::from_toml_path(std::path::Path::new(config))
    }
}

fn run_sprint118_bundle(
    config: &str,
    command_name: &str,
) -> Result<WorkspaceTimeoutReductionQueueBundle, String> {
    load_local_sprint118_config(config, command_name)
        .and_then(|config| WorkspaceTimeoutReductionQueueRunner::default().run(&config))
}

fn print_sprint118_report<T: serde::Serialize>(
    config: &str,
    command_name: &str,
    warning: &str,
    select: impl FnOnce(WorkspaceTimeoutReductionQueueBundle) -> T,
) -> Result<(), String> {
    run_sprint118_bundle(config, command_name).and_then(|report| {
        let value = select(report);
        print_json_report(warning, &value)
    })
}

fn run_minimal_ai_committee_cycle(
    config: &str,
    command_name: &str,
) -> Result<serde_json::Value, String> {
    if config.contains("://") {
        Err(format!("{command_name} config path must be local"))
    } else {
        let parsed = MinimalAiCommitteeCycleConfig::from_toml_path(std::path::Path::new(config))?;
        parsed.validate()?;
        if parsed.autonomous_paper_run {
            serde_json::to_value(run_autonomous_paper_committee_loop_from_config_path(
                std::path::Path::new(config),
            )?)
            .map_err(|err| err.to_string())
        } else if parsed.batch_mode {
            if parsed.emit_owner_summary
                || parsed.emit_owner_console_view
                || parsed.emit_reconsideration_view
                || parsed.owner_feedback_path.is_some()
                || parsed.member_state_input_path.is_some()
                || parsed.member_state_output_path.is_some()
            {
                serde_json::to_value(run_batch_committee_cycle_with_state_from_config_path(
                    std::path::Path::new(config),
                )?)
                .map_err(|err| err.to_string())
            } else {
                serde_json::to_value(run_batch_committee_cycle_from_config_path(
                    std::path::Path::new(config),
                )?)
                .map_err(|err| err.to_string())
            }
        } else {
            serde_json::to_value(run_minimal_committee_cycle_from_config_path(
                std::path::Path::new(config),
            )?)
            .map_err(|err| err.to_string())
        }
    }
}

#[derive(Debug, Subcommand)]
enum Commands {
    Run {
        #[arg(long)]
        config: String,
    },
    Batch {
        #[arg(long)]
        config: String,
    },
    Ablation {
        #[arg(long)]
        config: String,
    },
    Sprint14 {
        #[arg(long = "from-ablation")]
        from_ablation: String,
        #[arg(long)]
        out: String,
    },
    EvidenceClose {
        #[arg(long)]
        config: String,
    },
    RealEvidence {
        #[arg(long)]
        config: String,
    },
    DataPreflight {
        #[arg(long)]
        input: String,
        #[arg(long)]
        out: String,
        #[arg(long)]
        symbol: String,
        #[arg(long)]
        timeframe: String,
    },
    OnboardData {
        #[arg(long)]
        config: Option<String>,
        #[arg(long)]
        input: Option<String>,
        #[arg(long)]
        out: Option<String>,
        #[arg(long)]
        symbol: Option<String>,
        #[arg(long)]
        timeframe: Option<String>,
    },
    ImportKrxSnapshot {
        #[arg(long)]
        input: String,
        #[arg(long)]
        out: String,
        #[arg(long)]
        date: Option<String>,
        #[arg(long)]
        symbol: Option<String>,
    },
    CollectCandles {
        #[arg(long)]
        provider: String,
        #[arg(long)]
        symbol: String,
        #[arg(long)]
        venue: Option<String>,
        #[arg(long)]
        timeframe: String,
        #[arg(long)]
        start: Option<String>,
        #[arg(long)]
        end: Option<String>,
        #[arg(long)]
        out: String,
        #[arg(long)]
        fixture: Option<String>,
        #[arg(long, default_value = "leave-gaps")]
        fill_missing: String,
        #[arg(long)]
        max_rows: Option<usize>,
        #[arg(long)]
        max_requests: Option<usize>,
        #[arg(long)]
        max_days: Option<usize>,
        #[arg(long, default_value = "compact")]
        raw_archive: String,
        #[arg(long)]
        outputsize: Option<String>,
        #[arg(long)]
        allow_full_history: bool,
        #[arg(long)]
        api_key_env_var: Option<String>,
        #[arg(long)]
        api_secret_env_var: Option<String>,
        #[arg(long)]
        auth_header_name: Option<String>,
        #[arg(long)]
        query_param_name: Option<String>,
        #[arg(long)]
        endpoint_template: Option<String>,
        #[arg(long, default_value = "raw")]
        adjusted_price: String,
    },
    Campaign {
        #[arg(long)]
        config: String,
    },
    CollectPlan {
        #[arg(long)]
        config: String,
    },
    EvidenceRun {
        #[arg(long = "from-collection")]
        from_collection: String,
        #[arg(long)]
        out: String,
    },
    CollectAndEvaluate {
        #[arg(long)]
        config: String,
    },
    AiBenchmark {
        #[arg(long)]
        config: String,
    },
    CollectTrainEvaluate {
        #[arg(long)]
        config: String,
    },
    /// Research-only Mamba readiness audit; no training, inference, or live trading.
    MambaReadiness {
        #[arg(long)]
        config: String,
    },
    /// Research-only core completion audit; core completion never implies live trading readiness or profitability.
    CoreCompletionAudit {
        #[arg(long)]
        config: String,
    },
    /// Research-only sequence dataset readiness gate; no live trading, no execution, and no network tests.
    SequenceReadiness {
        #[arg(long)]
        config: String,
    },
    /// Research-only Mamba3 readiness v2 audit; runtime remains deferred and only external-prototype-only research may pass.
    MambaReadinessV2 {
        #[arg(long)]
        config: String,
    },
    /// Research-only model escalation decision; no live trading, broker, order, or account path is allowed.
    ModelEscalationDecision {
        #[arg(long)]
        config: String,
    },
    /// External research-only Mamba3Fin-lite prototype plan; no Rust runtime, no training, and no live trading.
    MambaPrototypePlan {
        #[arg(long)]
        config: String,
    },
    /// Research-only core hardening report; no live trading, broker, or account paths.
    CoreCheck {
        #[arg(long)]
        config: Option<String>,
        #[arg(long)]
        out: Option<String>,
    },
    /// Research-only core performance scorecard; no live trading, broker, order, or account paths.
    CorePerformance {
        #[arg(long)]
        config: String,
    },
    /// Research-only core bottleneck report; usefulness remains unproven without official evidence.
    CoreBottleneck {
        #[arg(long)]
        config: String,
    },
    /// Research-only core regression guard; no live trading, broker, or account paths.
    CoreRegression {
        #[arg(long)]
        config: String,
    },
    /// Research-only official-data benchmark gated by core-check.
    CoreBenchmark {
        #[arg(long)]
        config: String,
    },
    /// Research-only provider auth preflight using env-var names only.
    ProviderAuthCheck {
        #[arg(long)]
        config: String,
    },
    /// Research-only KRX auth readiness using env-var presence only; never prints secret values.
    KrxAuthReadiness {
        #[arg(long)]
        config: String,
    },
    /// Research-only bounded KRX symbol whitelist; no wildcard, no all-symbol scans.
    KrxSymbolWhitelist {
        #[arg(long)]
        config: String,
    },
    /// Research-only KRX evidence plan; local-first, secret-safe, and no live collection in tests.
    KrxEvidencePlan {
        #[arg(long)]
        config: String,
    },
    /// Research-only KRX official activation; market-data-only and never live trading.
    KrxOfficialActivate {
        #[arg(long)]
        config: String,
    },
    /// Research-only bounded KRX auth/endpoint dry run; market-data-only and secret-safe.
    KrxCollectionDryRun {
        #[arg(long)]
        config: String,
    },
    /// Research-only bounded KRX collection plan; market-data-only, local-first, deterministic, and secret-safe.
    KrxCollectionPlan {
        #[arg(long)]
        config: String,
    },
    /// Research-only bounded KRX collection closure; market-data-only, secret-safe, and no broker/order/account/live-trading path.
    KrxBoundedCollect {
        #[arg(long)]
        config: String,
    },
    /// Research-only KRX candle sufficiency report; market-data-only, secret-safe, and no-lookahead constrained.
    KrxCandleSufficiency {
        #[arg(long)]
        config: String,
    },
    /// Research-only KRX outcome-link closure; market-data-only, secret-safe, and never implies live trading.
    KrxOutcomeLinkClose {
        #[arg(long)]
        config: String,
    },
    /// Research-only KRX collection closure; market-data-only, bounded, local-first, and secret-safe.
    KrxCollectionClose {
        #[arg(long)]
        config: String,
    },
    /// Research-only KIS auth readiness; market-data-only and never prints secrets.
    KisAuthReadiness {
        #[arg(long)]
        config: String,
    },
    /// Research-only KIS auth closure; env-only, secret-safe, and market-data-only.
    KisAuthClose {
        #[arg(long)]
        config: String,
    },
    /// Research-only KIS endpoint policy; broker/order/account surfaces stay denied.
    KisEndpointPolicy {
        #[arg(long)]
        config: String,
    },
    /// Research-only KIS symbol whitelist; bounded, local-only, and market-data-only.
    KisSymbolWhitelist {
        #[arg(long)]
        config: String,
    },
    /// Research-only KIS collection planning; bounded, deterministic, and local-first.
    KisCollectionPlan {
        #[arg(long)]
        config: String,
    },
    /// Research-only KIS market-data dry-run; no network and live collection stays disabled by default.
    KisMarketDataDryRun {
        #[arg(long)]
        config: String,
    },
    /// Research-only KIS collection plan v2; deterministic, market-data-only, and local-first.
    KisCollectionPlanV2 {
        #[arg(long)]
        config: String,
    },
    /// Research-only KIS activation; market-data-only, secret-safe, and paper-only.
    KisMarketDataActivate {
        #[arg(long)]
        config: String,
    },
    /// Research-only bounded KIS market-data smoke; local-first, paper-only, and secret-safe.
    KisMarketDataSmoke {
        #[arg(long)]
        config: String,
    },
    /// Research-only KIS candle sufficiency; no-lookahead and market-data-only.
    KisCandleSufficiency {
        #[arg(long)]
        config: String,
    },
    /// Research-only KIS outcome-link closure; paper-only and market-data-only.
    KisOutcomeLinkClose {
        #[arg(long)]
        config: String,
    },
    /// Research-only KIS-vs-KRX migration report; operational only, never a performance claim.
    KisKrxMigration {
        #[arg(long)]
        config: String,
    },
    /// Research-only provider catalog for bounded official and professional source onboarding.
    ProviderCatalog,
    /// Research-only provider readiness report; no live trading, broker, or account commands.
    ProviderReadiness {
        #[arg(long)]
        config: String,
    },
    /// Research-only provider selector; no secrets printed and yfinance is never official.
    ProviderSelect {
        #[arg(long)]
        market: String,
    },
    /// Research-only provider reality report for freshness, cost, entitlement, and compatibility.
    ProviderReality {
        #[arg(long)]
        config: String,
    },
    /// Research-only KIS-first provider simplification; market-data-only and never enables broker/order/account paths.
    ProviderSimplify {
        #[arg(long)]
        config: String,
    },
    /// Read-only local dashboard snapshot; no broker/order/account controls and no live execution.
    DashboardSnapshot {
        #[arg(long)]
        config: String,
    },
    /// Read-only local dashboard renderer; static local HTML/JSON/TXT only.
    DashboardRender {
        #[arg(long)]
        config: String,
    },
    /// Research-only read-only Control Tower v1 bundle; local-only, deterministic, paper-only, and no execution path.
    ControlTowerV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only KIS evidence depth run; local-only, deterministic, paper-only, and never implies profitability or live readiness.
    KisEvidenceDepthRun {
        #[arg(long)]
        config: String,
    },
    /// Read-only local Control Tower refresh from local artifacts; no secrets, no execution, and no account/order controls.
    ControlTowerRefresh {
        #[arg(long)]
        config: String,
    },
    /// Research-only environment isolation report for deterministic KIS/KRX tests.
    KisEnvIsolationReport {
        #[arg(long)]
        config: String,
    },
    /// Secret-safe local artifact audit; rejects leaked secrets, tokens, account, and order fields.
    SecretRedactionAudit {
        #[arg(long)]
        config: String,
    },
    /// Read-only local Control Tower auto-refresh bundle from Sprint 58 smoke artifacts.
    ControlTowerAutoRefresh {
        #[arg(long)]
        config: String,
    },
    /// Paper-only operational runbook with exact local CLI sequence; no live trading, no broker/order/account commands.
    OperationalRunbook {
        #[arg(long)]
        config: String,
    },
    /// Paper-only Sprint 58 operational runbook with exact local CLI sequence.
    OperationalRunbookV2 {
        #[arg(long)]
        config: String,
    },
    /// Research-only system integration review; paper-only, local-only, and never implies live trading or profitability.
    SystemReview {
        #[arg(long)]
        config: String,
    },
    /// Research-only evidence hardening bundle; local-only, paper-only, and never a live-trading approval.
    EvidenceHardening {
        #[arg(long)]
        config: String,
    },
    /// Research-only outcome-link coverage report; no live trading, no lookahead, and no execution path.
    OutcomeLinkCoverage {
        #[arg(long)]
        config: String,
    },
    /// Research-only counterfactual coverage report; paper-only, local-only, and no broker/order/account path.
    CounterfactualCoverage {
        #[arg(long)]
        config: String,
    },
    /// Research-only owner review ergonomics report; manual owner review only, paper-only, and never executable.
    ReviewErgonomics {
        #[arg(long)]
        config: String,
    },
    /// Research-only UI framework decision; local UI only for now and no heavy framework migration in this sprint.
    UiFrameworkDecision {
        #[arg(long)]
        config: String,
    },
    /// Research-only Mamba3 application timing gate; runtime remains deferred and sequence dataset comes first.
    MambaApplicationTiming {
        #[arg(long)]
        config: String,
    },
    /// Research-only bounded KIS evidence expansion plan v2; local-only and operator live data stays disabled by default.
    KisEvidenceExpansionPlanV2 {
        #[arg(long)]
        config: String,
    },
    /// Research-only bounded KIS evidence closure; paper-only, local-only, and never a live-trading approval.
    KisEvidenceClosure {
        #[arg(long)]
        config: String,
    },
    /// Research-only outcome-link depth closure v2; no-lookahead safe and local-only.
    OutcomeLinkDepthCloseV2 {
        #[arg(long)]
        config: String,
    },
    /// Research-only owner review discipline v2; manual-only, paper-only, and never executable.
    OwnerReviewDisciplineV2 {
        #[arg(long)]
        config: String,
    },
    /// Research-only sequence dataset readiness hardening; no training, no inference, and no live trading.
    SequenceReadinessHardening {
        #[arg(long)]
        config: String,
    },
    /// Research-only sequence window preview; bounded export planning only.
    SequenceWindowPreview {
        #[arg(long)]
        config: String,
    },
    /// Research-only no-lookahead sequence proof; deterministic local audit only.
    NoLookaheadSequenceProof {
        #[arg(long)]
        config: String,
    },
    /// Research-only bounded sequence dataset export; no training, no live trading, and no runtime Mamba path.
    SequenceDatasetExport {
        #[arg(long)]
        config: String,
    },
    /// Research-only sequence dataset quality report; deterministic local export checks only.
    SequenceDatasetQuality {
        #[arg(long)]
        config: String,
    },
    /// Research-only sequence dataset drift guard; deterministic manifest drift only.
    SequenceDatasetDrift {
        #[arg(long)]
        config: String,
    },
    /// Research-only sequence dataset replay check; deterministic local replay only.
    SequenceDatasetReplayCheck {
        #[arg(long)]
        config: String,
    },
    /// Research-only external bridge readiness; import/evaluation only and never training.
    ExternalBridgeReadiness {
        #[arg(long)]
        config: String,
    },
    /// Research-only Mamba3Fin external prototype gate; runtime stays deferred and planning-only.
    Mamba3finPrototypeGate {
        #[arg(long)]
        config: String,
    },
    /// Research-only external prediction CSV import; no training, no live inference, and local-only paths.
    ExternalPredictionImportV2 {
        #[arg(long)]
        config: String,
    },
    /// Research-only external model evaluation; deterministic offline metrics only and never a profitability claim.
    ExternalModelEvaluate {
        #[arg(long)]
        config: String,
    },
    /// Research-only external-vs-Trinity comparison; diagnostic comparison only and never a live decision path.
    ExternalVsTrinity {
        #[arg(long)]
        config: String,
    },
    /// Research-only external prediction ablation; diagnostic stress test only and never training.
    ExternalPredictionAblation {
        #[arg(long)]
        config: String,
    },
    /// Research-only external model promotion gate; research-candidate-only and never live promotion.
    ExternalModelPromotionGate {
        #[arg(long)]
        config: String,
    },
    /// Research-only Mamba3Fin-lite contract; runtime remains deferred and no Mamba runtime is implemented.
    Mamba3finContract {
        #[arg(long)]
        config: String,
    },
    /// Research-only external artifact registry; no training, no live inference, and local-only artifact paths only.
    ExternalArtifactRegistry {
        #[arg(long)]
        config: String,
    },
    /// Research-only external evaluation history; offline version deltas only and never a deployment claim.
    ExternalEvaluationHistory {
        #[arg(long)]
        config: String,
    },
    /// Research-only calibration drift; offline calibration tracking only and never live inference.
    CalibrationDrift {
        #[arg(long)]
        config: String,
    },
    /// Research-only external model version comparison; diagnostic-only and never a live decision path.
    ExternalModelVersionComparison {
        #[arg(long)]
        config: String,
    },
    /// Research-only conservative external leaderboard; no deployment, no training, and no live promotion.
    ConservativeExternalLeaderboard {
        #[arg(long)]
        config: String,
    },
    /// Research-only external registry audit; local-only artifact scanning with no broker, order, or account actions.
    ExternalRegistryAudit {
        #[arg(long)]
        config: String,
    },
    /// Research-only external model research ops; no training, no live promotion, and local-only report inputs only.
    ExternalModelResearchOps {
        #[arg(long)]
        config: String,
    },
    /// Research-only external model review queue; offline-only review bookkeeping with no live execution path.
    ExternalModelReviewQueue {
        #[arg(long)]
        config: String,
    },
    /// Research-only external model watchlist; offline-only and never a deployment approval.
    ExternalModelWatchlist {
        #[arg(long)]
        config: String,
    },
    /// Research-only model comparability matrix; diagnostic-only compatibility checks from local artifacts.
    ModelComparabilityMatrix {
        #[arg(long)]
        config: String,
    },
    /// Research-only artifact completeness scoring; local-only artifact presence checks.
    ArtifactCompleteness {
        #[arg(long)]
        config: String,
    },
    /// Research-only model evidence risk profile; no live promotion and no runtime inference path.
    ModelRiskProfile {
        #[arg(long)]
        config: String,
    },
    /// Research-only model leaderboard changelog; no deployment claim and no live decision path.
    ModelLeaderboardChangelog {
        #[arg(long)]
        config: String,
    },
    /// Research-only model review closure; no live/runtime inference and no training path.
    ModelReviewClose {
        #[arg(long)]
        config: String,
    },
    /// Research-only prediction history pack; offline-only multi-version coverage and no training path.
    PredictionHistoryPack {
        #[arg(long)]
        config: String,
    },
    /// Research-only model ops decision log; offline-only review trace with no deployment semantics.
    ModelOpsDecisionLog {
        #[arg(long)]
        config: String,
    },
    /// Research-only model ops operator QA; no deployment, no runtime, and no unsafe controls.
    ModelOpsOperatorQa {
        #[arg(long)]
        config: String,
    },
    /// Research-only model ops regression guard; deterministic offline guard only.
    ModelOpsRegressionGuard {
        #[arg(long)]
        config: String,
    },
    /// Read-only Control Tower model ops refresh; static/local only with no execution controls.
    ControlTowerModelOpsRefresh {
        #[arg(long)]
        config: String,
    },
    /// Research-only model ops rollup; offline-only, no-training, and one conservative summary card per model version.
    ModelOpsRollup {
        #[arg(long)]
        config: String,
    },
    /// Research-only regression explainability; offline-only human-readable regression causes.
    ModelRegressionExplain {
        #[arg(long)]
        config: String,
    },
    /// Research-only operator QA rollup; research-only deduped static review guidance with no execution path.
    OperatorQaRollup {
        #[arg(long)]
        config: String,
    },
    /// Research-only decision log rollup; diagnostic aggregation only with no deployment claim.
    DecisionLogRollup {
        #[arg(long)]
        config: String,
    },
    /// Research-only model risk rollup; no live promotion and no runtime inference.
    ModelRiskRollup {
        #[arg(long)]
        config: String,
    },
    /// Research-only model action priority; copy-only local suggestions and no execution command.
    ModelActionPriority {
        #[arg(long)]
        config: String,
    },
    /// Read-only Control Tower model ops rollup; static/local only, read-only, with no train/live/order controls.
    ControlTowerModelOpsRollup {
        #[arg(long)]
        config: String,
    },
    /// research-only model ops trace; static/read-only local drill-down only with no training or live execution path.
    ModelOpsTrace {
        #[arg(long)]
        config: String,
    },
    /// research-only model trace index; local-only artifact lineage with no browser execution.
    ModelTraceIndex {
        #[arg(long)]
        config: String,
    },
    /// research-only decision conflict trace; static conservative conflict review only.
    ModelDecisionConflicts {
        #[arg(long)]
        config: String,
    },
    /// diagnostic-only regression evidence trace; offline-only baseline/current evidence links.
    ModelRegressionTrace {
        #[arg(long)]
        config: String,
    },
    /// operator QA only evidence trace; read-only checklist linkage with no execution path.
    ModelQaTrace {
        #[arg(long)]
        config: String,
    },
    /// research-only model action trace; copy-only rationale and no execution command.
    ModelActionTrace {
        #[arg(long)]
        config: String,
    },
    /// deterministic model version diff trace; local-only comparison with no runtime or live path.
    ModelVersionDiffTrace {
        #[arg(long)]
        config: String,
    },
    /// static/read-only baseline snapshot coverage; paper-only local audit with no training or live execution path.
    BaselineSnapshotCoverage {
        #[arg(long)]
        config: String,
    },
    /// research-only comparison target registry; local-only mapping with no promotion or execution path.
    ComparisonTargetRegistry {
        #[arg(long)]
        config: String,
    },
    /// diagnostic-only missing comparison targets; conservative closure audit with no runtime action.
    MissingComparisonTargets {
        #[arg(long)]
        config: String,
    },
    /// research-only trace completeness audit; coverage audit only with static local output.
    TraceCompletenessAudit {
        #[arg(long)]
        config: String,
    },
    /// research-only downgrade evidence audit; conservative evidence review only with no execution path.
    DowngradeEvidenceAudit {
        #[arg(long)]
        config: String,
    },
    /// deterministic snapshot diff integrity; local-only comparison audit with no runtime or live path.
    SnapshotDiffIntegrity {
        #[arg(long)]
        config: String,
    },
    /// read-only control tower trace coverage; static local coverage panel with no train/live/order controls.
    ControlTowerTraceCoverage {
        #[arg(long)]
        config: String,
    },
    /// research-only unexpected diff triage; static local explanation only with no runtime/live/training path.
    UnexpectedDiffTriage {
        #[arg(long)]
        config: String,
    },
    /// research-only snapshot diff classification; deterministic local classification only.
    SnapshotDiffClassify {
        #[arg(long)]
        config: String,
    },
    /// research-only contract alignment audit v2; static explanation only with no model execution.
    ContractAlignmentAuditV2 {
        #[arg(long)]
        config: String,
    },
    /// research-only owner review closure v2; conservative paper-only closure with no runtime/live promotion.
    OwnerReviewCloseV2 {
        #[arg(long)]
        config: String,
    },
    /// research-only trace warning reduction; static warning interpretation with no hidden suppression.
    TraceWarningReduce {
        #[arg(long)]
        config: String,
    },
    /// research-only downgrade evidence closure plan; conservative static planning only.
    DowngradeEvidenceClosurePlan {
        #[arg(long)]
        config: String,
    },
    /// research-only diff root cause report; deterministic offline explanation only.
    DiffRootCause {
        #[arg(long)]
        config: String,
    },
    /// research-only model version review disposition; paper-only recommendation layer with no live/runtime path.
    ModelVersionReviewDisposition {
        #[arg(long)]
        config: String,
    },
    /// read-only control tower diff triage panel; static local triage summary with no controls or execution.
    ControlTowerDiffTriage {
        #[arg(long)]
        config: String,
    },
    /// static/read-only operator briefing bundle; paper-only local summary with no training, live inference, or order/account path.
    OperatorBriefing {
        #[arg(long)]
        config: String,
    },
    /// paper-only owner action checklist; copy-only local checklist with no execution path.
    OwnerActionChecklist {
        #[arg(long)]
        config: String,
    },
    /// static operator decision queue; local-only review queue with no execution controls.
    OperatorDecisionQueue {
        #[arg(long)]
        config: String,
    },
    /// local-only briefing delta report; deterministic comparison against a previous static briefing.
    BriefingDelta {
        #[arg(long)]
        config: String,
    },
    /// conservative leaderboard warning closure report; research-only and no live/order promotion path.
    LeaderboardWarningClosure {
        #[arg(long)]
        config: String,
    },
    /// conservative retirement evidence completion report; paper-only and retirement never means deletion.
    RetirementEvidenceCompletion {
        #[arg(long)]
        config: String,
    },
    /// read-only Control Tower briefing panel; one-screen static summary with no live/order/account controls.
    ControlTowerBriefing {
        #[arg(long)]
        config: String,
    },
    /// local-only offline evidence attachment registry and bundle; research-only, static/read-only, and no live readiness.
    OfflineEvidenceAttach {
        #[arg(long)]
        config: String,
    },
    /// research-only prediction history expansion; offline-only CSV/model-card attachment and no training path.
    PredictionHistoryExpand {
        #[arg(long)]
        config: String,
    },
    /// conservative retirement regression evidence pack; diagnostic-only or retirement support with no deletion claim.
    RetirementRegressionPack {
        #[arg(long)]
        config: String,
    },
    /// research-only evidence gap closure v2; static reduction audit only with no execution path.
    EvidenceGapCloseV2 {
        #[arg(long)]
        config: String,
    },
    /// paper-only owner checklist closure; evidence-based reduction only and no execution command.
    OwnerChecklistClose {
        #[arg(long)]
        config: String,
    },
    /// monitoring-only direct-watch score; static/read-only and never live trading readiness.
    DirectWatchScore {
        #[arg(long)]
        config: String,
    },
    /// static briefing readiness gate; no live readiness, no execution, and no unsafe controls.
    BriefingReadinessGate {
        #[arg(long)]
        config: String,
    },
    /// research-only ext-model-b prediction closure; local-only fixture closure and never training or live inference.
    ExtModelBPredictionClose {
        #[arg(long)]
        config: String,
    },
    /// research-only prediction coverage finalization; bounded fixture coverage only and never a profitability claim.
    PredictionCoverageFinalize {
        #[arg(long)]
        config: String,
    },
    /// research-only evidence gap final closure; no live readiness, no execution, and no broker/order/account path.
    EvidenceGapFinalClose {
        #[arg(long)]
        config: String,
    },
    /// monitoring-only direct-watch final gate; static/read-only and never execution or live trading readiness.
    DirectWatchFinalGate {
        #[arg(long)]
        config: String,
    },
    /// static Control Tower final refresh; read-only local dashboard outputs and no train/live/order buttons.
    ControlTowerFinalRefresh {
        #[arg(long)]
        config: String,
    },
    /// research-only Sprint 73 workspace acceptance; records fmt/check/full-workspace/focused/CLI results.
    Sprint73WorkspaceAcceptance {
        #[arg(long)]
        config: String,
    },
    /// market-data-only real evidence follow-up; static/read-only local audit only with no live trading.
    RealEvidenceFollowup {
        #[arg(long)]
        config: String,
    },
    /// local-only real evidence attachment; attaches local official/KIS artifacts only and never executes trading flows.
    RealEvidenceAttach {
        #[arg(long)]
        config: String,
    },
    /// research-only KIS real evidence validation; canonical CSV validation only and no runtime/live path.
    KisRealEvidenceValidate {
        #[arg(long)]
        config: String,
    },
    /// provenance required real evidence audit; local-only source metadata check with no network side effects.
    RealProvenanceAudit {
        #[arg(long)]
        config: String,
    },
    /// preflight required real evidence audit; local-only preflight verification with no live trading path.
    RealPreflightAudit {
        #[arg(long)]
        config: String,
    },
    /// no live trading real evidence outcome readiness; static/report-only outcome gate.
    RealOutcomeReadiness {
        #[arg(long)]
        config: String,
    },
    /// no training real evidence sequence readiness; offline dataset readiness only and never training/runtime.
    RealSequenceReadiness {
        #[arg(long)]
        config: String,
    },
    /// no live inference real evidence model ops impact; offline follow-up only and never runtime inference.
    RealModelopsImpact {
        #[arg(long)]
        config: String,
    },
    /// warning not hidden Control Tower reduction; explicit safe explanations only with static/read-only output.
    ControlTowerWarningReduce {
        #[arg(long)]
        config: String,
    },
    /// monitoring-only direct-watch warning rationale; clarifies warnings without enabling execution.
    DirectWatchWarningRationale {
        #[arg(long)]
        config: String,
    },
    /// copyable commands only real evidence runbook; emits local command sequence only and no execution buttons.
    RealEvidenceRunbook {
        #[arg(long)]
        config: String,
    },
    /// research-only prediction requirements; refreshed prediction needs only, no training/runtime/live path.
    RealPredictionRequirements {
        #[arg(long)]
        config: String,
    },
    /// no training refreshed prediction plan; local-only offline import planning and never model training.
    RealPredictionRefreshPlan {
        #[arg(long)]
        config: String,
    },
    /// schema validation only refreshed prediction import; local-only CSV validation with no runtime inference.
    RealPredictionImport {
        #[arg(long)]
        config: String,
    },
    /// offline metrics only external reevaluation; research-only reevaluation and no profitability claim.
    RealExternalReevaluate {
        #[arg(long)]
        config: String,
    },
    /// research-only leaderboard refresh; no deployment, no live inference, and no execution path.
    RealLeaderboardRefresh {
        #[arg(long)]
        config: String,
    },
    /// offline modelops refresh only; no live inference, no training, and no broker/order/account path.
    RealModelopsRefresh {
        #[arg(long)]
        config: String,
    },
    /// warning not hidden stale closure; closes or explains ModelPredictionsStale without hiding warnings.
    ModelPredictionsStaleClose {
        #[arg(long)]
        config: String,
    },
    /// static read-only warning closure v2; keeps deferred warnings visible by design with no execution controls.
    ControlTowerWarningCloseV2 {
        #[arg(long)]
        config: String,
    },
    /// monitoring-only direct-watch gate after refreshed evidence; never enables execution or live trading.
    DirectWatchPostEvidenceGate {
        #[arg(long)]
        config: String,
    },
    /// copyable commands only modelops runbook; no training, no live inference, and no command execution.
    RealModelopsRunbook {
        #[arg(long)]
        config: String,
    },
    /// stable-only toolchain modernization bundle; local-only, no nightly, no runtime feature expansion, and no live trading path.
    RustToolchainModernize {
        #[arg(long)]
        config: String,
    },
    /// local-only stable toolchain report; diagnostic only and never changes live/runtime behavior.
    ToolchainVersionReport {
        #[arg(long)]
        config: String,
    },
    /// local-only cargo workspace audit; no runtime feature changes, no broker/order/account path, and no training.
    CargoWorkspaceAudit {
        #[arg(long)]
        config: String,
    },
    /// deterministic test tier plan; preserves the full workspace final gate and never deletes safety coverage.
    TestTierPlan {
        #[arg(long)]
        config: String,
    },
    /// diagnostic-only runtime budget; timing guidance only and never weakens the final workspace gate.
    TestRuntimeBudget {
        #[arg(long)]
        config: String,
    },
    /// no-test-deletion slow inventory; categorizes slow areas without removing safety tests.
    SlowTestInventory {
        #[arg(long)]
        config: String,
    },
    /// safety smoke retained CLI tiering; local-only representative smoke planning without live/runtime paths.
    CliSmokeTiering {
        #[arg(long)]
        config: String,
    },
    /// copyable commands only developer speed runbook; optional local accelerators only with no live trading path.
    DeveloperSpeedRunbook {
        #[arg(long)]
        config: String,
    },
    /// full workspace final acceptance report; tiered iteration is allowed but full workspace remains the ship gate.
    WorkspaceAcceptanceV2 {
        #[arg(long)]
        config: String,
    },
    /// no fake timing repeated workspace timing report; local-only sample-backed or opt-in real measurement only.
    RepeatedWorkspaceTiming {
        #[arg(long)]
        config: String,
    },
    /// diagnostic-only test binary cost report; identifies heavy binaries without changing behavior.
    TestBinaryCost {
        #[arg(long)]
        config: String,
    },
    /// no semantic changes fixture setup cost report; local-only setup/load analysis only.
    FixtureSetupCost {
        #[arg(long)]
        config: String,
    },
    /// deterministic-only artifact render cost report; cache analysis only and never hides failures.
    ArtifactRenderCost {
        #[arg(long)]
        config: String,
    },
    /// safety smoke retained CLI reduction report; required smoke stays visible and no command family disappears.
    CliSmokeCostReduce {
        #[arg(long)]
        config: String,
    },
    /// no test deletion fixture dedup plan; setup/load reuse planning only with manual review when needed.
    FixtureDedupPlan {
        #[arg(long)]
        config: String,
    },
    /// no secret cache fixture cache plan; local-only deterministic cache policy planning only.
    FixtureCachePlan {
        #[arg(long)]
        config: String,
    },
    /// no hidden failures artifact render cache plan; deterministic fingerprinting required.
    ArtifactRenderCachePlan {
        #[arg(long)]
        config: String,
    },
    /// manual review test support refactor plan; no behavior change or safety coverage weakening.
    TestSupportRefactorPlan {
        #[arg(long)]
        config: String,
    },
    /// estimate-only dev loop savings report; labels confidence and never overclaims speedup.
    DevLoopSavingsEstimate {
        #[arg(long)]
        config: String,
    },
    /// full workspace final acceptance v3; repeated timing and cost plans are advisory only and full gate is preserved.
    WorkspaceAcceptanceV3 {
        #[arg(long)]
        config: String,
    },
    /// research-only core completion v2 report; freezes core contract boundaries and never implies live trading readiness.
    CoreCompletionV2 {
        #[arg(long)]
        config: String,
    },
    /// contract-only Mamba3Fin spec; defines inputs/heads only and does not implement runtime or training.
    Mamba3finCoreContract {
        #[arg(long)]
        config: String,
    },
    /// runtime deferred readiness gate; checks prerequisites only and never enables Mamba runtime.
    Mamba3RuntimeReadiness {
        #[arg(long)]
        config: String,
    },
    /// no expansion committee gate; keeps exactly three active personas and blocks 6/12 growth.
    CommitteeCompletionGate {
        #[arg(long)]
        config: String,
    },
    /// research-only committee materialization plan v2; local-only gap closure planning with no persona expansion.
    CommitteeMaterializationPlanV2 {
        #[arg(long)]
        config: String,
    },
    /// no training storage decision; freezes manifest/storage policy only and does not train any model.
    TrainingDataStorageDecision {
        #[arg(long)]
        config: String,
    },
    /// storage spec only training-data registry view; defines registry metadata without runtime or training.
    TrainingDataRegistrySpec {
        #[arg(long)]
        config: String,
    },
    /// storage spec only training-data layout plan; static directory contract only with no runtime behavior.
    TrainingDataLayoutPlan {
        #[arg(long)]
        config: String,
    },
    /// storage spec only training-data lineage plan; deterministic lineage contract only.
    TrainingDataLineageSpec {
        #[arg(long)]
        config: String,
    },
    /// staged/deferred Mamba3 roadmap; orders prerequisites without implementing runtime or training.
    Mamba3ImplementationRoadmap {
        #[arg(long)]
        config: String,
    },
    /// contract-only sequence-core registry; compares Mamba3Fin and Gated DeltaNet without runtime or training.
    SequenceCoreRegistry {
        #[arg(long)]
        config: String,
    },
    /// runtime deferred Gated DeltaNet contract; defines state/projection/gate fields only.
    GatedDeltanetCoreContract {
        #[arg(long)]
        config: String,
    },
    /// no runtime Gated DeltaNet readiness gate; prerequisite-only and research-only.
    GatedDeltanetReadiness {
        #[arg(long)]
        config: String,
    },
    /// offline comparison only candidate plan; never implies runtime, training, or live use.
    SequenceCoreComparisonPlan {
        #[arg(long)]
        config: String,
    },
    /// prediction CSV only external prototype contract; external research only.
    SequenceCoreExternalContract {
        #[arg(long)]
        config: String,
    },
    /// no fake data storage materializer; creates local placeholder manifests only.
    TrainingStorageMaterialize {
        #[arg(long)]
        config: String,
    },
    /// manifest checks only storage integrity report; validates local placeholder artifacts.
    TrainingStorageIntegrity {
        #[arg(long)]
        config: String,
    },
    /// no runtime/training family storage contracts; artifact policy only.
    ModelFamilyStorageContract {
        #[arg(long)]
        config: String,
    },
    /// read-only sequence core panel; static status only with no command execution.
    ControlTowerSequenceCore {
        #[arg(long)]
        config: String,
    },
    /// prototype-only external comparison bundle; prediction CSV only and never runtime or training.
    SequenceCorePrototypeCompare {
        #[arg(long)]
        config: String,
    },
    /// prototype artifact registry only; local artifact comparability without runtime or training.
    SequenceCorePrototypeRegistry {
        #[arg(long)]
        config: String,
    },
    /// prediction CSV only Mamba3Fin import; external prototype artifact validation only.
    Mamba3finPrototypeImport {
        #[arg(long)]
        config: String,
    },
    /// prediction CSV only Gated DeltaNet import; external prototype artifact validation only.
    GatedDeltanetPrototypeImport {
        #[arg(long)]
        config: String,
    },
    /// offline-only prototype evaluation; diagnostic metrics only and never deployable.
    SequenceCorePrototypeEvaluate {
        #[arg(long)]
        config: String,
    },
    /// Trinity-only committee evidence expansion; no 6/12 activation and no live path.
    CommitteeEvidenceExpandV2 {
        #[arg(long)]
        config: String,
    },
    /// diagnostic-only committee vs sequence comparison; no runtime or live implication.
    CommitteeVsSequenceCore {
        #[arg(long)]
        config: String,
    },
    /// no fake data artifact population; local references only and never training.
    TrainingArtifactPopulate {
        #[arg(long)]
        config: String,
    },
    /// populated manifest checks only; validates local references and secret safety.
    TrainingPopulatedIntegrity {
        #[arg(long)]
        config: String,
    },
    /// read-only sequence prototype panel; static comparison status only.
    ControlTowerSequencePrototype {
        #[arg(long)]
        config: String,
    },
    /// interpretation-only prototype interpretation bundle; diagnostic-only and no runtime selection.
    PrototypeInterpretation {
        #[arg(long)]
        config: String,
    },
    /// diagnostic-only prototype confidence report; source-weighted confidence only.
    PrototypeConfidence {
        #[arg(long)]
        config: String,
    },
    /// no runtime selection prototype winner gate; diagnostic-only interpretation only.
    PrototypeWinnerGate {
        #[arg(long)]
        config: String,
    },
    /// offline-only prototype disagreement matrix; disagreement review only.
    PrototypeDisagreement {
        #[arg(long)]
        config: String,
    },
    /// diagnostic-only prototype failure mode report; no deployment implication.
    PrototypeFailureModes {
        #[arg(long)]
        config: String,
    },
    /// diagnostic-only prototype calibration and risk synthesis; no runtime implication.
    PrototypeCalibrationRisk {
        #[arg(long)]
        config: String,
    },
    /// defensive-axis interpretation only; NoTrade and RiskDenied remain first-class.
    NoTradeRiskDeniedInterpretation {
        #[arg(long)]
        config: String,
    },
    /// Trinity-only committee reference audit; no 6/12 activation and no live path.
    CommitteeReferenceAuditV2 {
        #[arg(long)]
        config: String,
    },
    /// Trinity-only committee reference closure plan; depth planning only.
    CommitteeReferenceDepthPlan {
        #[arg(long)]
        config: String,
    },
    /// offline-only committee vs sequence disagreement review; diagnostic-only.
    CommitteeSequenceDisagreement {
        #[arg(long)]
        config: String,
    },
    /// runtime deferred evidence-weighted decision gate; never selects runtime or training.
    SequenceCoreDecisionGate {
        #[arg(long)]
        config: String,
    },
    /// training lineage completeness audit; provenance only and no training.
    TrainingLineageCompleteness {
        #[arg(long)]
        config: String,
    },
    /// read-only prototype interpretation panel; static interpretation status only.
    ControlTowerPrototypeInterpretation {
        #[arg(long)]
        config: String,
    },
    /// offline-only official evidence depth expansion; local evidence only and never runtime or training.
    OfficialEvidenceDepthExpand {
        #[arg(long)]
        config: String,
    },
    /// Trinity-only committee reference closure; official reference closure only with no persona expansion.
    CommitteeReferenceClose {
        #[arg(long)]
        config: String,
    },
    /// scenario-only official pack v3; local evidence packing only.
    OfficialScenarioPackV3 {
        #[arg(long)]
        config: String,
    },
    /// outcome-only official pack v3; no-lookahead bounded local evidence only.
    OfficialOutcomePackV3 {
        #[arg(long)]
        config: String,
    },
    /// baseline-only official pack v3; defensive baselines remain first-class.
    OfficialBaselinePackV3 {
        #[arg(long)]
        config: String,
    },
    /// NoTrade-only official counterfactual pack v3; defensive interpretation only.
    OfficialNotradePackV3 {
        #[arg(long)]
        config: String,
    },
    /// RiskDenied-only official counterfactual pack v3; defensive interpretation only.
    OfficialRiskdeniedPackV3 {
        #[arg(long)]
        config: String,
    },
    /// defensive counterfactual depth only; diagnostic-only and never runtime approval.
    DefensiveCounterfactualDepth {
        #[arg(long)]
        config: String,
    },
    /// official reference quality only; provenance and completeness diagnostics.
    OfficialReferenceQuality {
        #[arg(long)]
        config: String,
    },
    /// official reference diversity only; symbol/timeframe/horizon diagnostics.
    OfficialReferenceDiversity {
        #[arg(long)]
        config: String,
    },
    /// official no-lookahead audit only; leakage guard diagnostics.
    OfficialReferenceNoLookahead {
        #[arg(long)]
        config: String,
    },
    /// official source-boundary audit only; source promotion guard diagnostics.
    OfficialReferenceSourceBoundary {
        #[arg(long)]
        config: String,
    },
    /// sequence-core confidence rerun only; evidence-depth confidence diagnostics.
    SequenceCoreConfidenceRerun {
        #[arg(long)]
        config: String,
    },
    /// evidence-weighted decision gate v2; runtime still deferred and never training/live.
    SequenceCoreDecisionGateV2 {
        #[arg(long)]
        config: String,
    },
    /// read-only official evidence depth panel; static status only.
    ControlTowerEvidenceDepth {
        #[arg(long)]
        config: String,
    },
    /// Sprint 83 acceptance recovery bundle; acceptance diagnostics only and never runtime/training/live.
    Sprint83AcceptanceRecovery {
        #[arg(long)]
        config: String,
    },
    /// full workspace acceptance recovery status only; honest pass/fail or blocked state.
    FullWorkspaceAcceptanceRecovery {
        #[arg(long)]
        config: String,
    },
    /// long-running compilation diagnosis only; diagnostic-only and no hidden pass/fail.
    LongCompilationDiagnosis {
        #[arg(long)]
        config: String,
    },
    /// evidence-depth fixture audit only; source-boundary/no-lookahead fixture checks.
    EvidenceDepthFixtureAudit {
        #[arg(long)]
        config: String,
    },
    /// evidence-depth fixture normalization only; deterministic fixture view diagnostics.
    EvidenceDepthFixtureNormalize {
        #[arg(long)]
        config: String,
    },
    /// evidence-depth fixture completeness only; critical fixture presence diagnostics.
    EvidenceDepthFixtureCompleteness {
        #[arg(long)]
        config: String,
    },
    /// evidence-depth determinism regression only; repeatability diagnostics.
    EvidenceDepthDeterminismRegression {
        #[arg(long)]
        config: String,
    },
    /// Sprint 82 smoke compression only; representative vs exhaustive smoke diagnostics.
    Sprint82SmokeCompress {
        #[arg(long)]
        config: String,
    },
    /// fixture boundary audit v2 only; no-lookahead and source-boundary diagnostics.
    FixtureBoundaryAuditV2 {
        #[arg(long)]
        config: String,
    },
    /// test runtime recovery plan only; runtime-cost reduction planning with safety preserved.
    TestRuntimeRecoveryPlan {
        #[arg(long)]
        config: String,
    },
    /// workspace acceptance recovery gate only; ship-gate diagnostics with runtime deferred.
    WorkspaceAcceptanceRecoveryGate {
        #[arg(long)]
        config: String,
    },
    /// read-only Sprint 83 recovery panel; static recovery status only.
    ControlTowerSprint83Recovery {
        #[arg(long)]
        config: String,
    },
    /// Sprint 84 test-cost reduction bundle; no safety deletion, no runtime/training/live behavior.
    Sprint84TestCostReduce {
        #[arg(long)]
        config: String,
    },
    /// test-binary consolidation report only; preserves assertions and keeps high-risk files separate.
    TestBinaryConsolidate {
        #[arg(long)]
        config: String,
    },
    /// shared fixture harness report only; deterministic helper migration status.
    SharedFixtureHarnessReport {
        #[arg(long)]
        config: String,
    },
    /// representative CLI smoke harness only; safety smoke remains retained.
    RepresentativeSmokeHarness {
        #[arg(long)]
        config: String,
    },
    /// exhaustive CLI smoke manifest only; full/release documentation and never runtime.
    ExhaustiveSmokeManifest {
        #[arg(long)]
        config: String,
    },
    /// safety CLI smoke manifest only; required help/local-only/forbidden-command checks.
    SafetySmokeManifest {
        #[arg(long)]
        config: String,
    },
    /// CLI smoke execution policy only; quick/sprint/full/release safety tiers.
    CliSmokeExecutionPolicy {
        #[arg(long)]
        config: String,
    },
    /// test runtime before/after report only; measured or sample-backed with no fake timing.
    TestRuntimeBeforeAfter {
        #[arg(long)]
        config: String,
    },
    /// workspace final gate v2 only; no fake pass and focused suites never replace full workspace.
    WorkspaceFinalGateV2 {
        #[arg(long)]
        config: String,
    },
    /// read-only Control Tower test cost panel; no runtime/live/order/account/browser controls.
    ControlTowerTestCost {
        #[arg(long)]
        config: String,
    },
    /// Sprint 85 workspace-wide gate recovery bundle; grouped domains only, local-only, and never runtime/training/live.
    Sprint85WorkspaceGateRecover {
        #[arg(long)]
        config: String,
    },
    /// workspace-wide test surface audit only; remaining bottlenecks and consolidation candidates.
    WorkspaceTestSurfaceAudit {
        #[arg(long)]
        config: String,
    },
    /// remaining test binary inventory only; family counts and keep-separate candidates.
    RemainingTestBinaryInventory {
        #[arg(long)]
        config: String,
    },
    /// domain grouped suite plan only; representative grouping without replacing the full workspace gate.
    DomainSuitePlan {
        #[arg(long)]
        config: String,
    },
    /// shared fixture harness adoption only; deterministic helper migration status.
    SharedFixtureAdoption {
        #[arg(long)]
        config: String,
    },
    /// workspace smoke policy v2 only; quick/sprint/full/release plus safety smoke.
    WorkspaceSmokePolicyV2 {
        #[arg(long)]
        config: String,
    },
    /// workspace acceptance attempt v3 only; honest full-workspace blocked/pass/fail status.
    WorkspaceAcceptanceAttemptV3 {
        #[arg(long)]
        config: String,
    },
    /// full gate recovery v3 only; previous vs current gate state.
    FullGateRecoveryV3 {
        #[arg(long)]
        config: String,
    },
    /// workspace blocker drilldown only; remaining blocker family and next actions.
    WorkspaceBlockerDrilldown {
        #[arg(long)]
        config: String,
    },
    /// read-only Control Tower workspace gate v2 panel; no runtime/live/order/account/browser controls.
    ControlTowerWorkspaceGateV2 {
        #[arg(long)]
        config: String,
    },
    /// Sprint 86 residual workspace gate recovery bundle; compile-only truth is diagnostic only and never runtime/training/live.
    Sprint86ResidualGateRecover {
        #[arg(long)]
        config: String,
    },
    /// residual workspace binary audit only; no test deletion and deterministic family surface only.
    ResidualBinaryAudit {
        #[arg(long)]
        config: String,
    },
    /// residual family classifier only; deterministic classification of remaining broad binaries.
    ResidualFamilyClassifier {
        #[arg(long)]
        config: String,
    },
    /// residual consolidation plan only; preserve assertions and record keep-separate reasons.
    ResidualConsolidationPlan {
        #[arg(long)]
        config: String,
    },
    /// legacy integration migration only; moved assertions and kept-separate files.
    LegacyIntegrationMigration {
        #[arg(long)]
        config: String,
    },
    /// compile-only workspace attempt only; diagnostic only and not full execution.
    CompileOnlyWorkspaceAttempt {
        #[arg(long)]
        config: String,
    },
    /// cargo test no-run gate only; compile-only status and residual blockers.
    CargoTestNoRunGate {
        #[arg(long)]
        config: String,
    },
    /// full workspace attempt v4 only; no fake pass and compile-only never counts as full acceptance.
    FullWorkspaceAttemptV4 {
        #[arg(long)]
        config: String,
    },
    /// full gate recovery v4 only; previous/current gate states plus compile-only truth.
    FullGateRecoveryV4 {
        #[arg(long)]
        config: String,
    },
    /// residual blocker drilldown v2 only; blocker family, crates, and recommended suite target.
    ResidualBlockerDrilldownV2 {
        #[arg(long)]
        config: String,
    },
    /// workspace binary delta v2 only; sample-backed binary surface reduction report.
    WorkspaceBinaryDeltaV2 {
        #[arg(long)]
        config: String,
    },
    /// safety coverage preservation v2 only; safety guards stay required.
    SafetyCoveragePreservationV2 {
        #[arg(long)]
        config: String,
    },
    /// read-only Control Tower workspace gate v3 panel; no train/runtime/live/order/account/browser controls.
    ControlTowerWorkspaceGateV3 {
        #[arg(long)]
        config: String,
    },
    /// Sprint 87 compile gate recovery bundle; acceptance recovery only and never runtime/training/live work.
    Sprint87CompileGateRecover {
        #[arg(long)]
        config: String,
    },
    /// workspace compile graph audit only; no deletion, no hidden skips, and no safety weakening.
    WorkspaceCompileGraphAudit {
        #[arg(long)]
        config: String,
    },
    /// test target fanout only; diagnostic target-count and grouping-candidate report.
    TestTargetFanout {
        #[arg(long)]
        config: String,
    },
    /// dev dependency fanout only; diagnostic heavy/repeated compile report.
    DevDependencyFanout {
        #[arg(long)]
        config: String,
    },
    /// feature unification audit only; no hidden skips and no safety feature removal.
    FeatureUnificationAudit {
        #[arg(long)]
        config: String,
    },
    /// compile family classifier v2 only; deterministic broad family classification.
    CompileFamilyClassifierV2 {
        #[arg(long)]
        config: String,
    },
    /// compile-heavy consolidation plan only; preserve assertions and document keep-separate cases.
    CompileHeavyConsolidationPlan {
        #[arg(long)]
        config: String,
    },
    /// compile-only attempt v2 only; compile-only never implies full acceptance.
    CompileOnlyAttemptV2 {
        #[arg(long)]
        config: String,
    },
    /// no-run acceptance gate v2 only; compile-only interpretation and not full execution.
    NoRunAcceptanceGateV2 {
        #[arg(long)]
        config: String,
    },
    /// full workspace attempt v5 only; no fake pass and no compile-only overclaim.
    FullWorkspaceAttemptV5 {
        #[arg(long)]
        config: String,
    },
    /// compile gate recovery only; previous/current gate states with honest blocked truth.
    CompileGateRecovery {
        #[arg(long)]
        config: String,
    },
    /// compile blocker drilldown v3 only; blocker family, files, crates, and next actions.
    CompileBlockerDrilldownV3 {
        #[arg(long)]
        config: String,
    },
    /// test target delta v3 only; sample-backed target surface reduction.
    TestTargetDeltaV3 {
        #[arg(long)]
        config: String,
    },
    /// safety coverage preservation v3 only; safety guards stay required.
    SafetyCoveragePreservationV3 {
        #[arg(long)]
        config: String,
    },
    /// read-only Control Tower compile gate v4 panel; no train/runtime/live/order/account/browser controls.
    ControlTowerCompileGateV4 {
        #[arg(long)]
        config: String,
    },
    /// Sprint 88 seven blocker recovery bundle; acceptance recovery only and never runtime/training/live work.
    Sprint88SevenBlockerRecover {
        #[arg(long)]
        config: String,
    },
    /// seven blocker family recovery only; ordered blocker queue and primary next family stay explicit.
    SevenBlockerFamilyRecovery {
        #[arg(long)]
        config: String,
    },
    /// per-family compile probe only; not full acceptance and never a full workspace claim.
    PerFamilyCompileProbe {
        #[arg(long)]
        config: String,
    },
    /// per-family no-run probe only; compile-only interpretation and not full workspace acceptance.
    PerFamilyNoRunProbe {
        #[arg(long)]
        config: String,
    },
    /// per-family execution probe only; focused family execution never implies workspace acceptance.
    PerFamilyExecutionProbe {
        #[arg(long)]
        config: String,
    },
    /// CandleExpansionOps recovery only; grouped suite reuse without weakening source/no-lookahead/storage coverage.
    CandleExpansionRecovery {
        #[arg(long)]
        config: String,
    },
    /// ExternalPrediction recovery only; research-only prediction validation with runtime still deferred.
    ExternalPredictionRecovery {
        #[arg(long)]
        config: String,
    },
    /// KrxEvidence recovery only; market-data-only preservation and no order/account path.
    KrxEvidenceRecovery {
        #[arg(long)]
        config: String,
    },
    /// DashboardRenderer recovery only; static/read-only rendering with no POST/actions/browser execution.
    DashboardRendererRecovery {
        #[arg(long)]
        config: String,
    },
    /// CommitteeCliSafety isolation only; keep separate unless a safer split preserves all CLI checks.
    CommitteeCliSafetyIsolation {
        #[arg(long)]
        config: String,
    },
    /// BaselineSignal recovery only; conservative NoTrade and Risk Governor veto remain preserved.
    BaselineSignalRecovery {
        #[arg(long)]
        config: String,
    },
    /// CounterfactualBackfill recovery only; deterministic NoTrade/RiskDenied counterfactual preservation.
    CounterfactualBackfillRecovery {
        #[arg(long)]
        config: String,
    },
    /// dev dependency impact probe only; compile-cost factors only and no blind dependency removal.
    DevDependencyImpactProbe {
        #[arg(long)]
        config: String,
    },
    /// feature variant impact probe only; repeated variants stay explicit and unsafe merges remain blocked.
    FeatureVariantImpactProbe {
        #[arg(long)]
        config: String,
    },
    /// measured target delta v4 only; sample-backed unless real measurement exists.
    MeasuredTargetDeltaV4 {
        #[arg(long)]
        config: String,
    },
    /// real no-run gate attempt v3 only; compile-only and never equal to full workspace acceptance.
    RealNoRunGateAttemptV3 {
        #[arg(long)]
        config: String,
    },
    /// real full workspace gate attempt v6 only; no fake pass and full acceptance only when finished and passed.
    RealFullWorkspaceGateAttemptV6 {
        #[arg(long)]
        config: String,
    },
    /// gate rerun after each family only; rerun status stays separate from final full workspace truth.
    GateRerunAfterEachFamily {
        #[arg(long)]
        config: String,
    },
    /// workspace gate recovery v5 only; previous/current no-run and full gate states stay honest.
    WorkspaceGateRecoveryV5 {
        #[arg(long)]
        config: String,
    },
    /// remaining blocker queue v4 only; ordered family queue and primary next blocker stay explicit.
    RemainingBlockerQueueV4 {
        #[arg(long)]
        config: String,
    },
    /// safety coverage preservation v4 only; no-live/no-broker/no-runtime/no-training guards stay required.
    SafetyCoveragePreservationV4 {
        #[arg(long)]
        config: String,
    },
    /// read-only Control Tower seven blocker panel; no train/runtime/live/order/account/browser controls.
    ControlTowerSevenBlocker {
        #[arg(long)]
        config: String,
    },
    /// Sprint 89 candle recovery bundle; CandleExpansionOps-only, preserve assertions, and keep runtime deferred.
    Sprint89CandleRecover {
        #[arg(long)]
        config: String,
    },
    /// candle real reduction plan only; preserve assertions and keep donor lineage explicit.
    CandleRealReductionPlan {
        #[arg(long)]
        config: String,
    },
    /// candle assertion migration only; no assertion deletion and keep-separate reasons stay explicit.
    CandleAssertionMigration {
        #[arg(long)]
        config: String,
    },
    /// candle fixture/setup reduction only; shared fixture harness and deterministic output stay explicit.
    CandleFixtureSetupReduction {
        #[arg(long)]
        config: String,
    },
    /// candle compile impact only; measured-vs-sample-backed target delta stays explicit.
    CandleCompileImpact {
        #[arg(long)]
        config: String,
    },
    /// candle no-run rerun only; compile-only interpretation and not full workspace acceptance.
    CandleNoRunRerun {
        #[arg(long)]
        config: String,
    },
    /// candle full gate rerun only; full workspace only when finished and passed.
    CandleFullGateRerun {
        #[arg(long)]
        config: String,
    },
    /// seven blocker queue progress v5 only; queue advancement rules stay explicit.
    SevenBlockerQueueProgressV5 {
        #[arg(long)]
        config: String,
    },
    /// measured target delta v5 only; measured data and sample-backed fallback stay distinct.
    MeasuredTargetDeltaV5 {
        #[arg(long)]
        config: String,
    },
    /// real no-run gate attempt v4 only; compile-only truth stays separate from full acceptance.
    RealNoRunGateAttemptV4 {
        #[arg(long)]
        config: String,
    },
    /// real full workspace gate attempt v7 only; no fake pass and only a finished passing workspace accepts.
    RealFullWorkspaceGateAttemptV7 {
        #[arg(long)]
        config: String,
    },
    /// workspace gate recovery v6 only; previous/current gate states remain honest after candle reduction.
    WorkspaceGateRecoveryV6 {
        #[arg(long)]
        config: String,
    },
    /// remaining blocker queue v5 only; ordered families and the primary next family stay explicit.
    RemainingBlockerQueueV5 {
        #[arg(long)]
        config: String,
    },
    /// safety coverage preservation v5 only; preserve no-live/no-broker/no-runtime/no-training guards.
    SafetyCoveragePreservationV5 {
        #[arg(long)]
        config: String,
    },
    /// read-only Control Tower candle recovery panel; no train/runtime/live/order/account/browser controls.
    ControlTowerCandleRecovery {
        #[arg(long)]
        config: String,
    },
    /// Sprint 90 external prediction recovery bundle; ExternalPrediction-only and preserves schema/model-card/runtime guards.
    Sprint90ExternalPredictionRecover {
        #[arg(long)]
        config: String,
    },
    /// external prediction real reduction plan only; preserve assertions and donor lineage before any merge claim.
    ExternalPredictionRealReductionPlan {
        #[arg(long)]
        config: String,
    },
    /// external prediction assertion migration only; no assertion deletion and keep-separate reasons stay explicit.
    ExternalPredictionAssertionMigration {
        #[arg(long)]
        config: String,
    },
    /// external prediction fixture/setup reduction only; deterministic output and shared harness usage stay explicit.
    ExternalPredictionFixtureSetupReduction {
        #[arg(long)]
        config: String,
    },
    /// external prediction feature variant reduction only; unsafe variants remain explicit.
    ExternalPredictionFeatureVariantReduction {
        #[arg(long)]
        config: String,
    },
    /// external prediction compile impact only; measured-vs-sample-backed evidence stays explicit.
    ExternalPredictionCompileImpact {
        #[arg(long)]
        config: String,
    },
    /// external prediction no-run rerun only; compile-only interpretation and not full workspace acceptance.
    ExternalPredictionNoRunRerun {
        #[arg(long)]
        config: String,
    },
    /// external prediction full gate rerun only; full workspace only when finished and passed.
    ExternalPredictionFullGateRerun {
        #[arg(long)]
        config: String,
    },
    /// external prediction schema preservation only; schema, duplicate, probability, and forbidden-column guards stay explicit.
    ExternalPredictionSchemaPreservation {
        #[arg(long)]
        config: String,
    },
    /// external prediction model-card preservation only; model-card and runtime-deferred guards stay explicit.
    ExternalPredictionModelCardPreservation {
        #[arg(long)]
        config: String,
    },
    /// external prediction evaluation preservation only; research-only evaluation/promotion semantics stay explicit.
    ExternalPredictionEvaluationPreservation {
        #[arg(long)]
        config: String,
    },
    /// seven blocker queue progress v6 only; ExternalPrediction-vs-KrxEvidence advancement rules stay explicit.
    SevenBlockerQueueProgressV6 {
        #[arg(long)]
        config: String,
    },
    /// measured target delta v6 only; measured and sample-backed states stay distinct.
    MeasuredTargetDeltaV6 {
        #[arg(long)]
        config: String,
    },
    /// real no-run gate attempt v5 only; compile-only truth stays separate from full acceptance.
    RealNoRunGateAttemptV5 {
        #[arg(long)]
        config: String,
    },
    /// real full workspace gate attempt v8 only; only a finished passing workspace accepts.
    RealFullWorkspaceGateAttemptV8 {
        #[arg(long)]
        config: String,
    },
    /// workspace gate recovery v7 only; previous/current gate states remain honest after external reduction.
    WorkspaceGateRecoveryV7 {
        #[arg(long)]
        config: String,
    },
    /// remaining blocker queue v6 only; ordered families and the next family stay explicit.
    RemainingBlockerQueueV6 {
        #[arg(long)]
        config: String,
    },
    /// safety coverage preservation v6 only; no-live/no-broker/no-runtime/no-training guards stay required.
    SafetyCoveragePreservationV6 {
        #[arg(long)]
        config: String,
    },
    /// read-only Control Tower external prediction recovery panel; no train/runtime/live/order/account/browser controls.
    ControlTowerExternalPredictionRecovery {
        #[arg(long)]
        config: String,
    },
    /// Sprint 91 KRX evidence recovery bundle; KrxEvidence-only, market-data-only, and preserves auth/template/source gates.
    Sprint91KrxEvidenceRecover {
        #[arg(long)]
        config: String,
    },
    /// krx evidence real reduction plan only; preserve assertions before any suite merge claim.
    KrxEvidenceRealReductionPlan {
        #[arg(long)]
        config: String,
    },
    /// krx evidence assertion migration only; no deletion and keep-separate reasons stay explicit.
    KrxEvidenceAssertionMigration {
        #[arg(long)]
        config: String,
    },
    /// krx evidence fixture/setup reduction only; deterministic output and shared harness usage stay explicit.
    KrxEvidenceFixtureSetupReduction {
        #[arg(long)]
        config: String,
    },
    /// krx evidence auth boundary preservation only; no secret values and missing-auth behavior stays explicit.
    KrxEvidenceAuthBoundaryPreservation {
        #[arg(long)]
        config: String,
    },
    /// krx evidence endpoint-template preservation only; endpoint template required and market-data-only request building stays explicit.
    KrxEvidenceEndpointTemplatePreservation {
        #[arg(long)]
        config: String,
    },
    /// krx evidence source-boundary preservation only; no source promotion and no-lookahead semantics stay explicit.
    KrxEvidenceSourceBoundaryPreservation {
        #[arg(long)]
        config: String,
    },
    /// krx evidence market-data-only preservation only; no order/account/broker execution path exists.
    KrxEvidenceMarketDataOnlyPreservation {
        #[arg(long)]
        config: String,
    },
    /// krx evidence compile impact only; measured-vs-sample-backed evidence stays honest.
    KrxEvidenceCompileImpact {
        #[arg(long)]
        config: String,
    },
    /// krx evidence no-run rerun only; compile-only interpretation and never full workspace acceptance.
    KrxEvidenceNoRunRerun {
        #[arg(long)]
        config: String,
    },
    /// krx evidence full gate rerun only; full workspace only when finished and passed.
    KrxEvidenceFullGateRerun {
        #[arg(long)]
        config: String,
    },
    /// seven blocker queue progress v7 only; KrxEvidence-vs-DashboardRenderer advancement rules stay explicit.
    SevenBlockerQueueProgressV7 {
        #[arg(long)]
        config: String,
    },
    /// measured target delta v7 only; measured and sample-backed states stay distinct.
    MeasuredTargetDeltaV7 {
        #[arg(long)]
        config: String,
    },
    /// real no-run gate attempt v6 only; compile-only truth stays separate from full acceptance.
    RealNoRunGateAttemptV6 {
        #[arg(long)]
        config: String,
    },
    /// real full workspace gate attempt v9 only; only a finished passing workspace accepts.
    RealFullWorkspaceGateAttemptV9 {
        #[arg(long)]
        config: String,
    },
    /// workspace gate recovery v8 only; previous/current gate states remain honest after KrxEvidence reduction.
    WorkspaceGateRecoveryV8 {
        #[arg(long)]
        config: String,
    },
    /// remaining blocker queue v7 only; ordered families and the next family stay explicit.
    RemainingBlockerQueueV7 {
        #[arg(long)]
        config: String,
    },
    /// safety coverage preservation v7 only; no-live/no-broker/no-runtime/no-training guards stay required.
    SafetyCoveragePreservationV7 {
        #[arg(long)]
        config: String,
    },
    /// read-only Control Tower KRX evidence recovery panel; no train/runtime/live/order/account/browser controls.
    ControlTowerKrxEvidenceRecovery {
        #[arg(long)]
        config: String,
    },
    /// Sprint 92 KRX warning closure bundle; warning closure only, market-data-only, and no secret weakening.
    Sprint92KrxWarningClose {
        #[arg(long)]
        config: String,
    },
    /// krx warning closure only; closes warning-backed KRX state conservatively and never fakes workspace recovery.
    KrxWarningClosure {
        #[arg(long)]
        config: String,
    },
    /// krx secret-safety isolation only; isolated redaction sentinel stays explicit and secret weakening is never allowed.
    KrxSecretSafetyIsolation {
        #[arg(long)]
        config: String,
    },
    /// krx raw archive redaction coverage only; raw archive redaction is required and local-only.
    KrxRawArchiveRedactionCoverage {
        #[arg(long)]
        config: String,
    },
    /// krx manual review closure only; warning closure remains conservative and explicit.
    KrxManualReviewClose {
        #[arg(long)]
        config: String,
    },
    /// krx genuine reduction gate only; warning-backed vs genuine closure stays explicit.
    KrxGenuineReductionGate {
        #[arg(long)]
        config: String,
    },
    /// krx queue advancement gate only; DashboardRenderer entry only after explicit KRX closure.
    KrxQueueAdvancementGate {
        #[arg(long)]
        config: String,
    },
    /// krx real gate cause drilldown only; no-run/full causes stay explicit and local-only.
    KrxRealGateCauseDrilldown {
        #[arg(long)]
        config: String,
    },
    /// krx no-run timeout cause only; compile-only timeout interpretation never equals full acceptance.
    KrxNoRunTimeoutCause {
        #[arg(long)]
        config: String,
    },
    /// krx full workspace timeout cause only; full workspace only accepts a finished passing run.
    KrxFullWorkspaceTimeoutCause {
        #[arg(long)]
        config: String,
    },
    /// DashboardRenderer entry gate only; entry gate only and never reduction completion.
    DashboardRendererEntryGate {
        #[arg(long)]
        config: String,
    },
    /// DashboardRenderer readiness precheck only; static HTML and read-only checks stay explicit.
    DashboardRendererReadinessPrecheck {
        #[arg(long)]
        config: String,
    },
    /// measured target delta v8 only; measured and sample-backed states stay distinct.
    MeasuredTargetDeltaV8 {
        #[arg(long)]
        config: String,
    },
    /// real no-run gate attempt v7 only; compile-only truth stays separate from full workspace acceptance.
    RealNoRunGateAttemptV7 {
        #[arg(long)]
        config: String,
    },
    /// real full workspace gate attempt v10 only; only a finished passing workspace accepts.
    RealFullWorkspaceGateAttemptV10 {
        #[arg(long)]
        config: String,
    },
    /// workspace gate recovery v9 only; previous/current gate states remain honest after warning closure.
    WorkspaceGateRecoveryV9 {
        #[arg(long)]
        config: String,
    },
    /// remaining blocker queue v8 only; ordered families and DashboardRenderer entry allowance stay explicit.
    RemainingBlockerQueueV8 {
        #[arg(long)]
        config: String,
    },
    /// safety coverage preservation v8 only; no-live/no-broker/no-runtime/no-training guards stay required.
    SafetyCoveragePreservationV8 {
        #[arg(long)]
        config: String,
    },
    /// read-only Control Tower KRX warning closure panel; no train/runtime/live/order/account/browser controls.
    ControlTowerKrxWarningClosure {
        #[arg(long)]
        config: String,
    },
    /// Sprint 93 timeout attribution only; timeout attribution only, local-only, and never DashboardRenderer reduction.
    Sprint93TimeoutAttribution {
        #[arg(long)]
        config: String,
    },
    /// real timeout attribution only; research-only timeout attribution report without claiming full workspace acceptance.
    RealTimeoutAttribution {
        #[arg(long)]
        config: String,
    },
    /// real no-run diagnostic pass only; diagnostic, not acceptance, and compile-only visibility only.
    RealNoRunDiagnosticPass {
        #[arg(long)]
        config: String,
    },
    /// real full diagnostic pass only; diagnostic, not full gate, and never a finished quiet acceptance claim.
    RealFullDiagnosticPass {
        #[arg(long)]
        config: String,
    },
    /// cargo message capture only; secret-safe capture of cargo JSON messages for timeout attribution.
    CargoMessageCapture {
        #[arg(long)]
        config: String,
    },
    /// active rustc snapshot only; process snapshots stay local and command-line redaction is enforced.
    ActiveRustcSnapshot {
        #[arg(long)]
        config: String,
    },
    /// target dir growth only; deterministic local target-dir growth attribution only.
    TargetDirGrowth {
        #[arg(long)]
        config: String,
    },
    /// cargo target progress timeline only; deterministic timeline reconstruction for timeout attribution only.
    CargoTargetProgressTimeline {
        #[arg(long)]
        config: String,
    },
    /// quiet vs diagnostic gate comparison only; diagnostic visibility never replaces quiet acceptance gates.
    QuietVsDiagnosticGate {
        #[arg(long)]
        config: String,
    },
    /// KRX non-primary proof only; DashboardRenderer needs proof before entry can advance.
    KrxNonPrimaryProof {
        #[arg(long)]
        config: String,
    },
    /// unknown timeout closure only; unknown timeout interpretation stays explicit and conservative.
    UnknownTimeoutClosure {
        #[arg(long)]
        config: String,
    },
    /// workspace timeout attribution decision only; queue movement stays explicit and research-only.
    WorkspaceTimeoutAttributionDecision {
        #[arg(long)]
        config: String,
    },
    /// DashboardRenderer entry release gate only; entry only and never DashboardRenderer reduction completion.
    DashboardRendererEntryReleaseGate {
        #[arg(long)]
        config: String,
    },
    /// DashboardRenderer reduction hold only; reduction not started and kept explicitly held in Sprint 93.
    DashboardRendererReductionHold {
        #[arg(long)]
        config: String,
    },
    /// workspace gate recovery v10 only; timeout attribution improvement stays separate from full workspace acceptance.
    WorkspaceGateRecoveryV10 {
        #[arg(long)]
        config: String,
    },
    /// remaining blocker queue v9 only; queue advancement stays explicit after timeout attribution.
    RemainingBlockerQueueV9 {
        #[arg(long)]
        config: String,
    },
    /// safety coverage preservation v9 only; no-live/no-broker/no-runtime/no-training guards stay required.
    SafetyCoveragePreservationV9 {
        #[arg(long)]
        config: String,
    },
    /// read-only Control Tower timeout attribution panel; read-only with no train/runtime/live/order/account/browser controls.
    ControlTowerTimeoutAttribution {
        #[arg(long)]
        config: String,
    },
    /// DashboardRenderer-only Sprint 94 recovery bundle; preserve assertions, stay static/read-only, and never imply full workspace acceptance.
    Sprint94DashboardRendererRecover {
        #[arg(long)]
        config: String,
    },
    /// DashboardRenderer real reduction plan only; preserve assertions and keep donor/target coverage explicit.
    DashboardRendererRealReductionPlan {
        #[arg(long)]
        config: String,
    },
    /// DashboardRenderer assertion migration only; no assertion deletion and keep isolation reasons explicit.
    DashboardRendererAssertionMigration {
        #[arg(long)]
        config: String,
    },
    /// DashboardRenderer fixture/setup reduction only; shared fixture harness reuse without semantic drift.
    DashboardRendererFixtureSetupReduction {
        #[arg(long)]
        config: String,
    },
    /// DashboardRenderer static safety preservation only; static/read-only/paper-only UI guarantees stay required.
    DashboardRendererStaticSafetyPreservation {
        #[arg(long)]
        config: String,
    },
    /// DashboardRenderer secret redaction preservation only; no secrets in HTML/JSON/TXT or diagnostics.
    DashboardRendererSecretRedactionPreservation {
        #[arg(long)]
        config: String,
    },
    /// DashboardRenderer no browser execution only; no browser execution, no JS dependency, no POST/forms.
    DashboardRendererNoBrowserExecution {
        #[arg(long)]
        config: String,
    },
    /// DashboardRenderer no action control only; no order/account/trade controls and no live/runtime/train buttons.
    DashboardRendererNoActionControl {
        #[arg(long)]
        config: String,
    },
    /// DashboardRenderer determinism preservation only; deterministic state/render/storage fingerprints stay required.
    DashboardRendererDeterminismPreservation {
        #[arg(long)]
        config: String,
    },
    /// DashboardRenderer golden output reduction only; preserve HTML/JSON/TXT checks with no hidden bless updates.
    DashboardRendererGoldenOutputReduction {
        #[arg(long)]
        config: String,
    },
    /// DashboardRenderer compile impact only; sample-backed or measured reduction only, never a fake timing claim.
    DashboardRendererCompileImpact {
        #[arg(long)]
        config: String,
    },
    /// DashboardRenderer no-run rerun only; no-run only and never a full workspace acceptance claim.
    DashboardRendererNoRunRerun {
        #[arg(long)]
        config: String,
    },
    /// DashboardRenderer full gate rerun only; full workspace only and honest if still blocked.
    DashboardRendererFullGateRerun {
        #[arg(long)]
        config: String,
    },
    /// DashboardRenderer entry consumed only; entry release consumption stays tied to DashboardRenderer reduction only.
    DashboardRendererEntryConsumed {
        #[arg(long)]
        config: String,
    },
    /// Seven-blocker queue progress v10 only; queue advancement stays explicit and conservative after DashboardRenderer.
    SevenBlockerQueueProgressV10 {
        #[arg(long)]
        config: String,
    },
    /// Measured target delta v10 only; sample-backed or measured target-count deltas only.
    MeasuredTargetDeltaV10 {
        #[arg(long)]
        config: String,
    },
    /// Real no-run gate attempt v9 only; honest compile-only status after DashboardRenderer reduction.
    RealNoRunGateAttemptV9 {
        #[arg(long)]
        config: String,
    },
    /// Real full workspace gate attempt v12 only; honest full workspace status after DashboardRenderer reduction.
    RealFullWorkspaceGateAttemptV12 {
        #[arg(long)]
        config: String,
    },
    /// Workspace gate recovery v11 only; DashboardRenderer reduction stays separate from quiet workspace acceptance.
    WorkspaceGateRecoveryV11 {
        #[arg(long)]
        config: String,
    },
    /// Remaining blocker queue v10 only; CommitteeCliSafety remains isolated and DashboardRenderer must be explicit.
    RemainingBlockerQueueV10 {
        #[arg(long)]
        config: String,
    },
    /// Safety coverage preservation v10 only; no-live/no-broker/no-runtime/no-training guards stay required.
    SafetyCoveragePreservationV10 {
        #[arg(long)]
        config: String,
    },
    /// Read-only Control Tower DashboardRenderer recovery panel; no train/runtime/live/order/account/browser controls.
    ControlTowerDashboardRendererRecovery {
        #[arg(long)]
        config: String,
    },
    /// CommitteeCliSafety-only Sprint 95 recovery bundle; preserve CLI safety, keep isolation explicit, and never imply full workspace acceptance.
    Sprint95CommitteeCliSafetyRecover {
        #[arg(long)]
        config: String,
    },
    /// CommitteeCliSafety reduction plan only; preserve assertions and keep grouped-suite representation explicit.
    CommitteeCliSafetyReductionPlan {
        #[arg(long)]
        config: String,
    },
    /// CommitteeCliSafety isolation decision only; explicit permanent sentinel versus grouped-suite representation.
    CommitteeCliSafetyIsolationDecision {
        #[arg(long)]
        config: String,
    },
    /// CommitteeCliSafety assertion migration only; no assertion deletion and explicit isolated sentinel reasons.
    CommitteeCliSafetyAssertionMigration {
        #[arg(long)]
        config: String,
    },
    /// CommitteeCliSafety fixture/setup reduction only; shared fixture harness reuse without semantic drift.
    CommitteeCliSafetyFixtureSetupReduction {
        #[arg(long)]
        config: String,
    },
    /// CommitteeCliSafety remote path preservation only; remote paths rejected and local-only paths preserved.
    CommitteeCliSafetyRemotePathPreservation {
        #[arg(long)]
        config: String,
    },
    /// CommitteeCliSafety help text preservation only; research-only, paper-only, local-only, no-runtime, and no-training wording stays intact.
    CommitteeCliSafetyHelpTextPreservation {
        #[arg(long)]
        config: String,
    },
    /// CommitteeCliSafety forbidden command preservation only; forbidden commands absent and no train/runtime/live/order/account surface added.
    CommitteeCliSafetyForbiddenCommandPreservation {
        #[arg(long)]
        config: String,
    },
    /// CommitteeCliSafety runtime deferred preservation only; runtime remains deferred and no training/live inference is implemented.
    CommitteeCliSafetyRuntimeDeferredPreservation {
        #[arg(long)]
        config: String,
    },
    /// CommitteeCliSafety persona expansion guard only; keep exactly three active personas and no runtime committee judge expansion.
    CommitteeCliSafetyPersonaExpansionGuard {
        #[arg(long)]
        config: String,
    },
    /// CommitteeCliSafety order/account guard only; no broker/order/account/balance/buying-power controls or commands.
    CommitteeCliSafetyOrderAccountGuard {
        #[arg(long)]
        config: String,
    },
    /// CommitteeCliSafety browser execution guard only; no dashboard serve, browser execution, POST/action, or JS dependency.
    CommitteeCliSafetyBrowserExecutionGuard {
        #[arg(long)]
        config: String,
    },
    /// CommitteeCliSafety determinism preservation only; deterministic help, command surface, and read-only Control Tower status stay required.
    CommitteeCliSafetyDeterminismPreservation {
        #[arg(long)]
        config: String,
    },
    /// CommitteeCliSafety compile impact only; sample-backed or measured evidence only and never a fake timing claim.
    CommitteeCliSafetyCompileImpact {
        #[arg(long)]
        config: String,
    },
    /// CommitteeCliSafety no-run rerun only; compile-only gate status stays separate from full workspace acceptance.
    CommitteeCliSafetyNoRunRerun {
        #[arg(long)]
        config: String,
    },
    /// CommitteeCliSafety full gate rerun only; quiet full workspace gate only and honest if still blocked.
    CommitteeCliSafetyFullGateRerun {
        #[arg(long)]
        config: String,
    },
    /// CommitteeCliSafety entry consumed only; explicit isolated sentinel closure consumes the CommitteeCliSafety queue entry only.
    CommitteeCliSafetyEntryConsumed {
        #[arg(long)]
        config: String,
    },
    /// BaselineSignal entry gate only; entry only and never BaselineSignal reduction completion.
    BaselineSignalEntryGate {
        #[arg(long)]
        config: String,
    },
    /// BaselineSignal readiness precheck only; entry gate support only and never BaselineSignal reduction.
    BaselineSignalReadinessPrecheck {
        #[arg(long)]
        config: String,
    },
    /// Seven-blocker queue progress v11 only; explicit CommitteeCliSafety isolation-decision closure and BaselineSignal entry readiness.
    SevenBlockerQueueProgressV11 {
        #[arg(long)]
        config: String,
    },
    /// Measured target delta v11 only; sample-backed or measured CommitteeCliSafety delta only.
    MeasuredTargetDeltaV11 {
        #[arg(long)]
        config: String,
    },
    /// Real no-run gate attempt v10 only; honest compile-only status after CommitteeCliSafety closure/isolation.
    RealNoRunGateAttemptV10 {
        #[arg(long)]
        config: String,
    },
    /// Real full workspace gate attempt v13 only; honest quiet full workspace status after CommitteeCliSafety closure/isolation.
    RealFullWorkspaceGateAttemptV13 {
        #[arg(long)]
        config: String,
    },
    /// Workspace gate recovery v12 only; CommitteeCliSafety closure/isolation remains separate from finished quiet acceptance.
    WorkspaceGateRecoveryV12 {
        #[arg(long)]
        config: String,
    },
    /// Remaining blocker queue v11 only; BaselineSignal entry may advance only after explicit CommitteeCliSafety closure/isolation.
    RemainingBlockerQueueV11 {
        #[arg(long)]
        config: String,
    },
    /// Safety coverage preservation v11 only; no-live/no-broker/no-runtime/no-training/no-browser guards remain required.
    SafetyCoveragePreservationV11 {
        #[arg(long)]
        config: String,
    },
    /// Read-only Control Tower CommitteeCliSafety recovery panel; no train/runtime/live/order/account/browser controls.
    ControlTowerCommitteeCliSafetyRecovery {
        #[arg(long)]
        config: String,
    },
    /// BaselineSignal-only Sprint 96 recovery bundle; preserve NoTrade, Risk Governor veto, data-quality denial, source boundary, no-lookahead, and research-only semantics.
    Sprint96BaselineSignalRecover {
        #[arg(long)]
        config: String,
    },
    /// BaselineSignal reduction plan only; grouped suite reuse only and no speculative feature work.
    BaselineSignalRealReductionPlan {
        #[arg(long)]
        config: String,
    },
    /// BaselineSignal real reduction report only; conservative reduction status only.
    BaselineSignalRealReduction {
        #[arg(long)]
        config: String,
    },
    /// BaselineSignal assertion migration only; move coverage conservatively with explicit sentinels retained.
    BaselineSignalAssertionMigration {
        #[arg(long)]
        config: String,
    },
    /// BaselineSignal fixture/setup reduction only; shared fixture harness reuse without semantic drift.
    BaselineSignalFixtureSetupReduction {
        #[arg(long)]
        config: String,
    },
    /// BaselineSignal feature/regime flow preservation only; feature order, regime classification, and score flow stay intact.
    #[command(name = "baseline-signal-feature-regime-preservation")]
    BaselineSignalFeatureRegimeFlowPreservation {
        #[arg(long)]
        config: String,
    },
    /// BaselineSignal NoTrade default preservation only; conservative NoTrade remains the default.
    #[command(name = "baseline-signal-notrade-default-preservation")]
    BaselineSignalNoTradeDefaultPreservation {
        #[arg(long)]
        config: String,
    },
    /// BaselineSignal poor-data-quality denial only; bad or missing data stays denied.
    BaselineSignalPoorDataQualityDenial {
        #[arg(long)]
        config: String,
    },
    /// BaselineSignal Risk Governor veto preservation only; hard veto stays absolute.
    BaselineSignalRiskGovernorVetoPreservation {
        #[arg(long)]
        config: String,
    },
    /// BaselineSignal source-boundary preservation only; no source promotion and local research boundaries stay explicit.
    BaselineSignalSourceBoundaryPreservation {
        #[arg(long)]
        config: String,
    },
    /// BaselineSignal no-lookahead preservation only; future outcomes never leak into signal inputs.
    BaselineSignalNoLookaheadPreservation {
        #[arg(long)]
        config: String,
    },
    /// BaselineSignal research-only preservation only; no live/runtime/training/order/account path.
    BaselineSignalResearchOnlyPreservation {
        #[arg(long)]
        config: String,
    },
    /// BaselineSignal determinism preservation only; grouped suite and reports stay deterministic.
    BaselineSignalDeterminismPreservation {
        #[arg(long)]
        config: String,
    },
    /// BaselineSignal compile impact only; measured or sample-backed evidence only.
    BaselineSignalCompileImpact {
        #[arg(long)]
        config: String,
    },
    /// BaselineSignal no-run rerun only; compile-only rerun status stays separate from full quiet workspace acceptance.
    BaselineSignalNoRunRerun {
        #[arg(long)]
        config: String,
    },
    /// BaselineSignal full gate rerun only; quiet workspace acceptance stays honest and explicit.
    BaselineSignalFullGateRerun {
        #[arg(long)]
        config: String,
    },
    /// BaselineSignal entry consumed only; explicit Sprint 95 BaselineSignal entry closure only.
    BaselineSignalEntryConsumed {
        #[arg(long)]
        config: String,
    },
    /// CounterfactualBackfill entry gate only; entry/precheck only and never reduction.
    CounterfactualBackfillEntryGate {
        #[arg(long)]
        config: String,
    },
    /// CounterfactualBackfill readiness precheck only; no-trade/risk-denied/no-lookahead checks only.
    CounterfactualBackfillReadinessPrecheck {
        #[arg(long)]
        config: String,
    },
    /// Seven-blocker queue progress v12 only; BaselineSignal closure and CounterfactualBackfill next-family status stay explicit.
    SevenBlockerQueueProgressV12 {
        #[arg(long)]
        config: String,
    },
    /// Measured target delta v12 only; sample-backed or measured BaselineSignal delta only.
    MeasuredTargetDeltaV12 {
        #[arg(long)]
        config: String,
    },
    /// Real no-run gate attempt v11 only; honest compile-only status after BaselineSignal reduction.
    RealNoRunGateAttemptV11 {
        #[arg(long)]
        config: String,
    },
    /// Real full workspace gate attempt v14 only; honest quiet full workspace status after BaselineSignal reduction.
    RealFullWorkspaceGateAttemptV14 {
        #[arg(long)]
        config: String,
    },
    /// Workspace gate recovery v13 only; BaselineSignal reduction remains separate from finished quiet acceptance.
    WorkspaceGateRecoveryV13 {
        #[arg(long)]
        config: String,
    },
    /// Remaining blocker queue v12 only; CounterfactualBackfill entry allowance stays explicit.
    RemainingBlockerQueueV12 {
        #[arg(long)]
        config: String,
    },
    /// Safety coverage preservation v12 only; no-live/no-broker/no-runtime/no-training/no-browser guards remain required.
    SafetyCoveragePreservationV12 {
        #[arg(long)]
        config: String,
    },
    /// Read-only Control Tower BaselineSignal recovery panel; research-only status output only.
    ControlTowerBaselineSignalRecovery {
        #[arg(long)]
        config: String,
    },
    /// CounterfactualBackfill-only Sprint 97 recovery bundle; conservative local-only reduction with queue closure and workspace truth kept separate.
    Sprint97CounterfactualBackfillRecover {
        #[arg(long)]
        config: String,
    },
    /// CounterfactualBackfill reduction plan only; grouped suite reuse and explicit sentinel retention.
    CounterfactualBackfillRealReductionPlan {
        #[arg(long)]
        config: String,
    },
    /// CounterfactualBackfill real reduction report only; conservative reduction status only.
    CounterfactualBackfillRealReduction {
        #[arg(long)]
        config: String,
    },
    /// CounterfactualBackfill assertion migration only; move coverage conservatively with explicit sentinels retained.
    CounterfactualBackfillAssertionMigration {
        #[arg(long)]
        config: String,
    },
    /// CounterfactualBackfill fixture/setup reduction only; shared fixture harness reuse without semantic drift.
    CounterfactualBackfillFixtureSetupReduction {
        #[arg(long)]
        config: String,
    },
    #[command(name = "counterfactual-backfill-notrade-preservation")]
    CounterfactualBackfillNoTradePreservation {
        #[arg(long)]
        config: String,
    },
    #[command(name = "counterfactual-backfill-riskdenied-preservation")]
    CounterfactualBackfillRiskDeniedPreservation {
        #[arg(long)]
        config: String,
    },
    CounterfactualBackfillDefensiveValuePreservation {
        #[arg(long)]
        config: String,
    },
    CounterfactualBackfillOpportunityCostPreservation {
        #[arg(long)]
        config: String,
    },
    CounterfactualBackfillNoFabricatedOutcome {
        #[arg(long)]
        config: String,
    },
    CounterfactualBackfillNoLookaheadPreservation {
        #[arg(long)]
        config: String,
    },
    CounterfactualBackfillSourceBoundaryPreservation {
        #[arg(long)]
        config: String,
    },
    CounterfactualBackfillResearchOnlyPreservation {
        #[arg(long)]
        config: String,
    },
    CounterfactualBackfillDeterminismPreservation {
        #[arg(long)]
        config: String,
    },
    CounterfactualBackfillCompileImpact {
        #[arg(long)]
        config: String,
    },
    CounterfactualBackfillNoRunRerun {
        #[arg(long)]
        config: String,
    },
    CounterfactualBackfillFullGateRerun {
        #[arg(long)]
        config: String,
    },
    CounterfactualBackfillEntryConsumed {
        #[arg(long)]
        config: String,
    },
    FinalBlockerQueueClosureGate {
        #[arg(long)]
        config: String,
    },
    FinalBlockerQueueClosure {
        #[arg(long)]
        config: String,
    },
    WorkspaceAcceptanceTruthGate {
        #[arg(long)]
        config: String,
    },
    WorkspaceAcceptanceRemainingRisk {
        #[arg(long)]
        config: String,
    },
    SevenBlockerQueueProgressV13 {
        #[arg(long)]
        config: String,
    },
    MeasuredTargetDeltaV13 {
        #[arg(long)]
        config: String,
    },
    RealNoRunGateAttemptV12 {
        #[arg(long)]
        config: String,
    },
    RealFullWorkspaceGateAttemptV15 {
        #[arg(long)]
        config: String,
    },
    WorkspaceGateRecoveryV14 {
        #[arg(long)]
        config: String,
    },
    RemainingBlockerQueueV13 {
        #[arg(long)]
        config: String,
    },
    SafetyCoveragePreservationV13 {
        #[arg(long)]
        config: String,
    },
    ControlTowerCounterfactualBackfillRecovery {
        #[arg(long)]
        config: String,
    },
    /// Research-only architecture report; central core is deprecated and each committee member owns its own AI core.
    CommitteeOwnedCoreArchitecture {
        #[arg(long)]
        config: String,
    },
    /// Research-only investor-style registry; public philosophy-inspired archetypes only and no impersonation.
    InvestorStyleRegistry {
        #[arg(long)]
        config: String,
    },
    /// Research-only committee member specs; each paper-only member owns its own core contract.
    AiCommitteeMemberSpecs {
        #[arg(long)]
        config: String,
    },
    /// Research-only member-owned core contracts; runtime stays deferred and no central core exists.
    CommitteeMemberCoreContracts {
        #[arg(long)]
        config: String,
    },
    /// Research-only learning policy; offline study only and no broker/account or training access.
    CommitteeMemberLearningPolicy {
        #[arg(long)]
        config: String,
    },
    /// Research-only member proposals; entry timing proposals stay paper-only and local-only.
    CommitteeMemberProposals {
        #[arg(long)]
        config: String,
    },
    /// Research-only entry timing proposals; timing windows never imply broker execution.
    EntryTimingProposals {
        #[arg(long)]
        config: String,
    },
    /// Research-only debate trigger; one member proposal can convene a paper-only debate session.
    CommitteeDebateTrigger {
        #[arg(long)]
        config: String,
    },
    /// Research-only paper-only debate session; support, oppose, wait, and risk-deny remain offline only.
    CommitteeDebateSession {
        #[arg(long)]
        config: String,
    },
    /// Research-only chairman governance policy; chair cannot bypass Risk Governor.
    ChairmanGovernancePolicy {
        #[arg(long)]
        config: String,
    },
    /// Research-only chairman rule proposal; adaptable rules require audit before paper use.
    ChairmanRuleProposal {
        #[arg(long)]
        config: String,
    },
    /// Research-only chairman rulebook version; versioned and audited, never live-applied.
    ChairmanRulebookVersion {
        #[arg(long)]
        config: String,
    },
    /// Research-only rule adaptation audit; adaptive governance stays paper-only and audited.
    RuleAdaptationAudit {
        #[arg(long)]
        config: String,
    },
    /// Research-only promotion/demotion policy; multi-axis evidence beats raw profit-only ranking.
    PromotionDemotionPolicy {
        #[arg(long)]
        config: String,
    },
    /// Research-only member scorecards; multi-axis ranking only, no live capital allocation.
    MemberScorecards {
        #[arg(long)]
        config: String,
    },
    /// Research-only member promotion/demotion decisions; paper roster governance only.
    MemberPromotionDemotion {
        #[arg(long)]
        config: String,
    },
    /// Research-only committee roster lifecycle; watchlist/diagnostic states stay paper-only.
    CommitteeRosterLifecycle {
        #[arg(long)]
        config: String,
    },
    /// Research-only paper-only committee decision; no broker execution or account controls.
    PaperOnlyCommitteeDecision {
        #[arg(long)]
        config: String,
    },
    /// Read-only AI committee panel; static status only with no runtime, live, order, account, or browser controls.
    ControlTowerAiCommittee {
        #[arg(long)]
        config: String,
    },
    /// Research-only Sprint 98 bundle; committee-owned cores replace the deprecated central-core assumption.
    Sprint98CommitteeOwnedCore {
        #[arg(long)]
        config: String,
    },
    /// Research-only paper-only committee quality hardening; no central AI core, no runtime, no training, and no live trading.
    Sprint99CommitteeQualityHarden {
        #[arg(long)]
        config: String,
    },
    /// Research-only proposal quality report; proposal is not order execution and remains paper-only.
    CommitteeMemberProposalQuality {
        #[arg(long)]
        config: String,
    },
    /// Research-only entry timing quality report; entry timing remains paper-only and never an order.
    EntryTimingProposalQuality {
        #[arg(long)]
        config: String,
    },
    /// Research-only committee debate quality report; debate stays paper-only with no live inference.
    CommitteeDebateQuality {
        #[arg(long)]
        config: String,
    },
    /// Research-only debate evidence sufficiency report; local evidence only with source-boundary guard.
    DebateEvidenceSufficiency {
        #[arg(long)]
        config: String,
    },
    /// Research-only chairman rulebook quality report; versioned paper-only governance with no live rule mutation.
    ChairmanRulebookQuality {
        #[arg(long)]
        config: String,
    },
    /// Research-only chairman rule risk audit v2; paper-only governance cannot bypass Risk Governor.
    ChairmanRuleRiskAuditV2 {
        #[arg(long)]
        config: String,
    },
    /// Research-only rulebook version diff; paper-only rule deltas only, never live changes.
    RulebookVersionDiff {
        #[arg(long)]
        config: String,
    },
    /// Research-only promotion/demotion calibration; roster research only and not capital allocation.
    PromotionDemotionCalibration {
        #[arg(long)]
        config: String,
    },
    /// Research-only member scorecard calibration; scorecards calibrate paper research only.
    MemberScorecardCalibration {
        #[arg(long)]
        config: String,
    },
    /// Research-only member overfit risk report; no training or live adaptation exists.
    MemberOverfitRisk {
        #[arg(long)]
        config: String,
    },
    /// Research-only member style drift report; archetypes stay public-philosophy-inspired and non-impersonating.
    MemberStyleDrift {
        #[arg(long)]
        config: String,
    },
    /// Research-only investor style blindspot report; no investor impersonation and no private strategy claim.
    InvestorStyleBlindspot {
        #[arg(long)]
        config: String,
    },
    /// Research-only committee roster balance report; roster diversity supports paper review only.
    CommitteeRosterBalance {
        #[arg(long)]
        config: String,
    },
    /// Research-only paper-only decision replay; no broker, order, or account path exists.
    PaperOnlyDecisionReplay {
        #[arg(long)]
        config: String,
    },
    /// Research-only paper decision trace completeness report; traceability only, never execution.
    PaperDecisionTraceCompleteness {
        #[arg(long)]
        config: String,
    },
    /// Research-only Risk Governor debate handoff report; final veto remains required.
    RiskGovernorDebateHandoff {
        #[arg(long)]
        config: String,
    },
    /// Research-only architecture regression guard; no central AI core and no runtime leak allowed.
    CommitteeArchitectureRegressionGuard {
        #[arg(long)]
        config: String,
    },
    /// Research-only workspace acceptance truth closure plan; focused tests never replace full workspace acceptance.
    WorkspaceAcceptanceTruthClosurePlan {
        #[arg(long)]
        config: String,
    },
    /// Research-only workspace acceptance attempt record; full workspace acceptance requires real cargo test --workspace --quiet completion.
    WorkspaceAcceptanceAttemptV16 {
        #[arg(long)]
        config: String,
    },
    /// Research-only safety coverage preservation v15; preserves no live trading, no broker/account, and no runtime LLM path.
    SafetyCoveragePreservationV15 {
        #[arg(long)]
        config: String,
    },
    /// Read-only AI committee quality panel; static status only with no train/runtime/live/order/account/browser controls.
    ControlTowerAiCommitteeQuality {
        #[arg(long)]
        config: String,
    },
    /// Research-only Sprint 100 committee closure; paper-only warning closure only, no central AI core, no runtime, no training, no live inference, and no live trading.
    Sprint100CommitteeClosure {
        #[arg(long)]
        config: String,
    },
    /// Research-only Sprint 101 investor archetype ingestion; paper-only archetype normalization only, no impersonation, no central AI core, no runtime, no training, no live inference, and no live trading.
    Sprint101InvestorArchetypeIngest {
        #[arg(long)]
        config: String,
    },
    /// Research-only investor archetype ingestion report; public-philosophy-inspired archetypes only with no live activation of 18 agents.
    InvestorArchetypeIngestion {
        #[arg(long)]
        config: String,
    },
    /// Research-only investor source confidence report; local-only source weighting with no training or live inference.
    InvestorSourceConfidence {
        #[arg(long)]
        config: String,
    },
    /// Research-only investor safety normalization report; filters myths, unsupported claims, and impersonation risk without runtime implementation.
    InvestorSafetyNormalization {
        #[arg(long)]
        config: String,
    },
    /// Research-only investor feature vector cards; archetype feature cards only and not trained models.
    InvestorFeatureVectorCards {
        #[arg(long)]
        config: String,
    },
    /// Research-only investor do-not-learn guards; blocks unsafe claims and private-life myths with no training path.
    InvestorDoNotLearnGuards {
        #[arg(long)]
        config: String,
    },
    /// Research-only investor impersonation risk report; no exact investor clone and no live agent activation.
    InvestorImpersonationRisk {
        #[arg(long)]
        config: String,
    },
    /// Research-only investor unverified claim filter; removes unsupported claims and unofficial quotes.
    InvestorUnverifiedClaimFilter {
        #[arg(long)]
        config: String,
    },
    /// Research-only investor private-life myth filter; keeps only useful routines that map to auditable behavior.
    InvestorPrivateLifeMythFilter {
        #[arg(long)]
        config: String,
    },
    /// Research-only 18 investor registry; staged paper roster only and does not imply 18 live AI agents.
    EighteenInvestorRegistry {
        #[arg(long)]
        config: String,
    },
    /// Research-only style group taxonomy; keeps short-term, long-term, crypto, and common risk separated.
    StyleGroupTaxonomy {
        #[arg(long)]
        config: String,
    },
    /// Research-only style conflict matrix; explicit paper debate routing only with Risk Governor final veto preserved.
    StyleConflictMatrix {
        #[arg(long)]
        config: String,
    },
    /// Research-only regime routing policy; routes paper-only committee groups without runtime LLM live decision path.
    RegimeRoutingPolicy {
        #[arg(long)]
        config: String,
    },
    /// Research-only multi-expert committee topology; committee-owned archetype design only and no central AI core.
    MultiExpertCommitteeTopology {
        #[arg(long)]
        config: String,
    },
    /// Research-only member confidence weight policy; source reliability weighting only and not trade authority.
    MemberConfidenceWeightPolicy {
        #[arg(long)]
        config: String,
    },
    /// Research-only member feature scope mapping; archetype feature boundaries only with no training or runtime path.
    MemberFeatureScopeMapping {
        #[arg(long)]
        config: String,
    },
    /// Research-only member learning data cards; offline-study-only cards and not training artifacts for runtime deployment.
    MemberLearningDataCards {
        #[arg(long)]
        config: String,
    },
    /// Research-only archetype-to-member mapping; staged paper mapping only and no auto-activation of 18 live agents.
    ArchetypeToMemberMapping {
        #[arg(long)]
        config: String,
    },
    /// Research-only 18-member roster plan; paper-only watchlist and diagnostic staging with no live capital allocation.
    EighteenRosterPlan {
        #[arg(long)]
        config: String,
    },
    /// Research-only 18-member activation gate; paper-only gate only and live activation stays forbidden.
    EighteenActivationGate {
        #[arg(long)]
        config: String,
    },
    /// Research-only paper roster expansion gate; paper-only roster growth only and not live expansion.
    PaperRosterExpansionGate {
        #[arg(long)]
        config: String,
    },
    /// Research-only chairman style governance v2; chairman cannot bypass Risk Governor and may not activate live members.
    ChairmanStyleGovernanceV2 {
        #[arg(long)]
        config: String,
    },
    /// Research-only promotion/demotion policy v2; roster management only and not capital allocation.
    PromotionDemotionPolicyV2 {
        #[arg(long)]
        config: String,
    },
    /// Read-only investor archetype Control Tower panel; static output only with no train/runtime/live/order/account/browser controls.
    ControlTowerInvestorArchetype {
        #[arg(long)]
        config: String,
    },
    /// Research-only Sprint 102 paper rotation; paper-only dry-run only with no impersonation, no central AI core, no runtime implementation, no training, no live inference, and no live trading.
    Sprint102PaperRotation {
        #[arg(long)]
        config: String,
    },
    /// Research-only paper rotation scenario pack; paper-only scenario planning only.
    PaperRotationScenarioPack {
        #[arg(long)]
        config: String,
    },
    /// Research-only paper rotation market context set; preserves source boundaries and no-lookahead proof only.
    PaperRotationMarketContext {
        #[arg(long)]
        config: String,
    },
    /// Research-only archetype group rotation plan; routes short-term, long-term, crypto, and common risk groups with paper-only semantics.
    ArchetypeGroupRotationPlan {
        #[arg(long)]
        config: String,
    },
    /// Research-only archetype member selection; watchlist use is explicit and diagnostic members stay excluded by default.
    ArchetypeMemberSelection {
        #[arg(long)]
        config: String,
    },
    /// Research-only lower-confidence evidence hardening; Wonyotti, Larry Williams, and Arthur Hayes stay warning-backed and are not silently upgraded.
    LowerConfidenceEvidenceHardening {
        #[arg(long)]
        config: String,
    },
    /// Research-only weak-source candidate review; warning-backed candidates remain reviewable and down-weighted.
    WeakSourceCandidateReview {
        #[arg(long)]
        config: String,
    },
    /// Research-only Wonyotti evidence hardening; no impersonation, no training, and no silent confidence upgrade.
    WonyottiEvidenceHardening {
        #[arg(long)]
        config: String,
    },
    /// Research-only Larry Williams evidence hardening; seasonal evidence remains paper-only and not an execution rule.
    LarryWilliamsEvidenceHardening {
        #[arg(long)]
        config: String,
    },
    /// Research-only Arthur Hayes evidence hardening; leverage commentary remains lower-authority and paper-only.
    ArthurHayesEvidenceHardening {
        #[arg(long)]
        config: String,
    },
    /// Research-only paper member proposal run; proposal is not an order, broker action, or account command.
    PaperMemberProposalRun {
        #[arg(long)]
        config: String,
    },
    /// Research-only paper entry timing run; timing windows are paper-only and never execution permission.
    PaperEntryTimingRun {
        #[arg(long)]
        config: String,
    },
    /// Research-only group debate trigger; paper-only trigger only with no runtime LLM live decision path.
    GroupDebateTrigger {
        #[arg(long)]
        config: String,
    },
    /// Research-only group debate session; paper-only debate only with no live agent activation.
    GroupDebateSession {
        #[arg(long)]
        config: String,
    },
    /// Research-only cross-group debate conflict report; explicit conflict handling only.
    CrossGroupDebateConflict {
        #[arg(long)]
        config: String,
    },
    /// Research-only chairman synthesis dry-run; paper-only synthesis only and cannot bypass Risk Governor.
    ChairmanSynthesisDryRun {
        #[arg(long)]
        config: String,
    },
    /// Research-only chairman style weight audit; paper-only weight adjustment audit with no risk override.
    ChairmanStyleWeightAudit {
        #[arg(long)]
        config: String,
    },
    /// Research-only Risk Governor paper handoff; final veto remains mandatory and broker/live execution stays forbidden.
    RiskGovernorPaperHandoff {
        #[arg(long)]
        config: String,
    },
    /// Research-only paper decision trace v2; no broker/live execution and no order/account path.
    PaperDecisionTraceV2 {
        #[arg(long)]
        config: String,
    },
    /// Research-only paper decision replay v2; replay is audit-only and not live execution.
    PaperDecisionReplayV2 {
        #[arg(long)]
        config: String,
    },
    /// Research-only proposal expectation trace; proxies are not profit claims or live recommendations.
    ProposalExpectationTrace {
        #[arg(long)]
        config: String,
    },
    /// Research-only NoTrade/RiskDenied committee trace; audit-only safety trace with no order path.
    NotradeRiskdeniedCommitteeTrace {
        #[arg(long)]
        config: String,
    },
    /// Research-only regime-routed dry-run; routes paper-only groups without runtime implementation.
    RegimeRoutedDryRun {
        #[arg(long)]
        config: String,
    },
    /// Research-only multi-expert rotation coverage; paper-only coverage accounting only.
    MultiExpertRotationCoverage {
        #[arg(long)]
        config: String,
    },
    /// Research-only paper roster expansion usage; paper-only roster use only and no 18-live activation.
    PaperRosterExpansionUsage {
        #[arg(long)]
        config: String,
    },
    /// Research-only 18 activation safety; explicit no-live-activation check only.
    EighteenActivationSafety {
        #[arg(long)]
        config: String,
    },
    /// Research-only workspace truth closure plan v3; focused tests still cannot claim full workspace acceptance.
    WorkspaceTruthClosurePlanV3 {
        #[arg(long)]
        config: String,
    },
    /// Research-only workspace acceptance attempt v18; honest record only and not a fake pass/fail.
    WorkspaceAcceptanceAttemptV18 {
        #[arg(long)]
        config: String,
    },
    /// Research-only safety coverage preservation v18; preserves no live trading, no broker/order/account, and no runtime path.
    SafetyCoveragePreservationV18 {
        #[arg(long)]
        config: String,
    },
    /// Read-only Control Tower paper rotation panel; static/read-only output with no train/runtime/live/order/account/browser controls.
    ControlTowerPaperRotation {
        #[arg(long)]
        config: String,
    },
    /// Research-only Sprint 103 paper rotation closure; warning-closure-only, paper-only, no central AI core, no runtime implementation, no training, no live inference, and no live trading.
    Sprint103PaperRotationClose {
        #[arg(long)]
        config: String,
    },
    /// Research-only paper rotation warning closure; warning-closure-only and never a live readiness claim.
    PaperRotationWarningClosure {
        #[arg(long)]
        config: String,
    },
    /// Research-only rotation plan warning closure; keeps group routing paper-only and local-only.
    RotationPlanWarningClosure {
        #[arg(long)]
        config: String,
    },
    /// Research-only member selection warning closure; watchlist usage is explicit and live activation stays forbidden.
    MemberSelectionWarningClosure {
        #[arg(long)]
        config: String,
    },
    /// Research-only lower-confidence evidence closure; no silent confidence upgrade for Wonyotti, Larry Williams, or Arthur Hayes.
    LowerConfidenceEvidenceClosure {
        #[arg(long)]
        config: String,
    },
    /// Research-only Wonyotti warning closure; exact return claims stay blocked and no impersonation is allowed.
    WonyottiWarningClosure {
        #[arg(long)]
        config: String,
    },
    /// Research-only Larry Williams warning closure; exact numeric rules stay downweighted and paper-only.
    LarryWilliamsWarningClosure {
        #[arg(long)]
        config: String,
    },
    /// Research-only Arthur Hayes warning closure; leverage risk stays guarded and no runtime path is added.
    ArthurHayesWarningClosure {
        #[arg(long)]
        config: String,
    },
    /// Research-only proposal run warning closure; proposals remain paper semantics only and never orders.
    ProposalRunWarningClosure {
        #[arg(long)]
        config: String,
    },
    /// Research-only entry timing warning closure; entry timing stays paper-only and never execution permission.
    EntryTimingWarningClosure {
        #[arg(long)]
        config: String,
    },
    /// Research-only debate session warning closure; paper-only debate only with no runtime LLM live decision path.
    DebateSessionWarningClosure {
        #[arg(long)]
        config: String,
    },
    /// Research-only NeedMoreEvidence resolution plan; documents remaining evidence items without enabling live execution.
    NeedMoreEvidenceResolutionPlan {
        #[arg(long)]
        config: String,
    },
    /// Research-only cross-group conflict closure; explicit closure only with Risk Governor veto preserved.
    CrossGroupConflictClosure {
        #[arg(long)]
        config: String,
    },
    /// Research-only chairman synthesis warning closure; paper-only governance only and chairman cannot bypass Risk Governor.
    ChairmanSynthesisWarningClosure {
        #[arg(long)]
        config: String,
    },
    /// Research-only style weight audit warning closure; source-confidence caps stay enforced and no unsafe override is allowed.
    StyleWeightAuditWarningClosure {
        #[arg(long)]
        config: String,
    },
    /// Research-only Risk Governor handoff warning closure v2; final veto stays mandatory and broker/live execution remains false.
    RiskGovernorHandoffWarningClosureV2 {
        #[arg(long)]
        config: String,
    },
    /// Research-only paper trace warning closure; trace remains audit-only with no broker, order, or live execution path.
    PaperTraceWarningClosure {
        #[arg(long)]
        config: String,
    },
    /// Research-only paper replay warning closure v2; replay remains local-only audit output and not live execution.
    PaperReplayWarningClosureV2 {
        #[arg(long)]
        config: String,
    },
    /// Research-only expectation trace warning closure; expectation proxies are bounded and never profit claims.
    ExpectationTraceWarningClosure {
        #[arg(long)]
        config: String,
    },
    /// Research-only NoTrade/RiskDenied trace warning closure; audit-only paper semantics with no order/account command.
    NotradeRiskdeniedTraceWarningClosure {
        #[arg(long)]
        config: String,
    },
    /// Research-only regime routing warning closure; routing stays paper-only and deterministic.
    RegimeRoutingWarningClosure {
        #[arg(long)]
        config: String,
    },
    /// Research-only multi-expert coverage warning closure; coverage accounting only with no 18-live activation.
    MultiExpertCoverageWarningClosure {
        #[arg(long)]
        config: String,
    },
    /// Research-only paper roster usage warning closure; watchlist use stays paper-only and explicit.
    PaperRosterUsageWarningClosure {
        #[arg(long)]
        config: String,
    },
    /// Research-only watchlist member usage policy; paper-only watchlist use only and live activation forbidden.
    WatchlistMemberUsagePolicy {
        #[arg(long)]
        config: String,
    },
    /// Research-only SaylorTreasury watchlist audit; paper-only usage only and no live activation.
    SaylorTreasuryWatchlistAudit {
        #[arg(long)]
        config: String,
    },
    /// Research-only multi-scenario paper replay; paper-only replay calibration with no broker, no order, and no live trading.
    MultiScenarioPaperReplay {
        #[arg(long)]
        config: String,
    },
    /// Research-only scenario outcome expectation matrix; bounded paper proxies only and never profit claims.
    ScenarioOutcomeExpectationMatrix {
        #[arg(long)]
        config: String,
    },
    /// Research-only committee decision stability report; replay stability only and not live readiness.
    CommitteeDecisionStability {
        #[arg(long)]
        config: String,
    },
    /// Research-only paper NoTrade justification; NoTrade is a valid defensive paper outcome and not a failure.
    PaperNotradeJustification {
        #[arg(long)]
        config: String,
    },
    /// Research-only paper NeedMoreEvidence justification; unresolved evidence remains paper-only and no live escalation occurs.
    PaperNeedMoreEvidenceJustification {
        #[arg(long)]
        config: String,
    },
    /// Research-only Risk Governor NoTrade reason audit; final veto reasoning only and no bypass path.
    RiskGovernorNotradeReasonAudit {
        #[arg(long)]
        config: String,
    },
    /// Research-only paper rotation readiness gate v2; paper readiness only and live rotation remains forbidden.
    PaperRotationReadinessGateV2 {
        #[arg(long)]
        config: String,
    },
    /// Research-only workspace truth closure plan v4; focused tests remain separate from full workspace acceptance.
    WorkspaceTruthClosurePlanV4 {
        #[arg(long)]
        config: String,
    },
    /// Research-only workspace acceptance attempt v19; honest record only with no fake pass/fail or fake timing.
    WorkspaceAcceptanceAttemptV19 {
        #[arg(long)]
        config: String,
    },
    /// Research-only safety coverage preservation v19; preserves no training, no live inference, and no broker/order/account path.
    SafetyCoveragePreservationV19 {
        #[arg(long)]
        config: String,
    },
    /// Read-only Control Tower paper rotation closure panel; static/read-only output with no train/runtime/live/order/account/browser controls or activate-all-18-live button.
    ControlTowerPaperRotationClosure {
        #[arg(long)]
        config: String,
    },
    /// Research-only Sprint 104 dual-agent paper lifecycle; paper-only, dual-agent workflow only, verification is not full workspace acceptance, no runtime implementation, no training, no live inference, no live trading, no order/account command, no runtime LLM live decision path, no investor impersonation, no auto-activation of 18 live agents, no silent confidence upgrade, and remote paths rejected.
    Sprint104DualAgentPaperLifecycle {
        #[arg(long)]
        config: String,
    },
    /// Research-only dual-agent workflow policy; implementation stays 5.4, verification stays 5.5, and verification is not full workspace acceptance.
    DualAgentWorkflowPolicy {
        #[arg(long)]
        config: String,
    },
    /// Research-only implementation agent role; 5.4 implementation only and never a verification-only or live-trading path.
    ImplementationAgentRole {
        #[arg(long)]
        config: String,
    },
    /// Research-only verification agent role; 5.5 verification only and verification is not full workspace acceptance.
    VerificationAgentRole {
        #[arg(long)]
        config: String,
    },
    /// Research-only prompt compliance verification; dual-agent workflow only and verification remains separate from full workspace acceptance.
    PromptComplianceVerification {
        #[arg(long)]
        config: String,
    },
    /// Research-only safety invariant verification; no runtime implementation, no training, no live inference, no live trading, and no order/account command.
    SafetyInvariantVerification {
        #[arg(long)]
        config: String,
    },
    /// Research-only architecture regression verification; preserves committee-owned architecture and forbids central AI core regression.
    ArchitectureRegressionVerification {
        #[arg(long)]
        config: String,
    },
    /// Research-only test coverage verification; focused verification only and not full workspace acceptance.
    TestCoverageVerification {
        #[arg(long)]
        config: String,
    },
    /// Research-only final verification gate; verification is not full workspace acceptance and live trading remains forbidden.
    FinalVerificationGate {
        #[arg(long)]
        config: String,
    },
    /// Research-only paper batch replay; paper-only batch replay with no broker, no order/account command, and no live trading.
    PaperBatchReplay {
        #[arg(long)]
        config: String,
    },
    /// Research-only paper candidate lifecycle; paper candidate is not an order and no live execution is allowed.
    PaperCandidateLifecycle {
        #[arg(long)]
        config: String,
    },
    /// Research-only paper candidate promotion gate; paper-only promotion only and never a promote-to-live path.
    PaperCandidatePromotionGate {
        #[arg(long)]
        config: String,
    },
    /// Research-only paper candidate NoTrade gate; paper candidate is not an order and no execution permission is created.
    PaperCandidateNotradeGate {
        #[arg(long)]
        config: String,
    },
    /// Research-only paper candidate RiskDenied gate; Risk Governor remains final veto and no live execution is allowed.
    PaperCandidateRiskdeniedGate {
        #[arg(long)]
        config: String,
    },
    /// Research-only Risk Governor batch veto; paper-only veto accounting only and no broker/order/account path.
    RiskGovernorBatchVeto {
        #[arg(long)]
        config: String,
    },
    /// Research-only lower-confidence carry-forward; paper-only carry-forward with no silent confidence upgrade and no live activation.
    LowerConfidenceCarryForward {
        #[arg(long)]
        config: String,
    },
    /// Read-only Control Tower dual-agent panel; static/read-only output with no train/runtime/live/order/account/browser controls.
    ControlTowerDualAgent {
        #[arg(long)]
        config: String,
    },
    /// Read-only Control Tower paper candidate lifecycle panel; static/read-only output with no promote-to-live button, no order button, and no account panel.
    ControlTowerPaperCandidateLifecycle {
        #[arg(long)]
        config: String,
    },
    /// Research-only Sprint 105 verification patch closure; paper-only closure only, verification patch closure only, verification is not full workspace acceptance, no runtime implementation, no training, no live inference, no live trading, no order/account command, no runtime LLM live decision path, no investor impersonation, no auto-activation of 18 live agents, no silent confidence upgrade, and remote paths rejected.
    Sprint105VerificationPatchClose {
        #[arg(long)]
        config: String,
    },
    /// Research-only verification finding closure; patch-closure only and verification is not full workspace acceptance.
    VerificationFindingClosure {
        #[arg(long)]
        config: String,
    },
    /// Research-only review patch effect report; explicit patch effects only and not cargo acceptance.
    ReviewPatchEffect {
        #[arg(long)]
        config: String,
    },
    /// Research-only overclaim regression guard; full acceptance requires finished && passed only.
    OverclaimRegressionGuard {
        #[arg(long)]
        config: String,
    },
    /// Research-only workspace attempt truth hardening; full workspace remains separate and honest unfinished attempts stay visible.
    WorkspaceAttemptTruthHardening {
        #[arg(long)]
        config: String,
    },
    /// Research-only safety boolean coverage audit; uses actual guard booleans and never assumes runtime/live safety implicitly.
    SafetyBooleanCoverageAudit {
        #[arg(long)]
        config: String,
    },
    /// Research-only PaperRejected transition audit; paper-only lifecycle only and never an order or live transition.
    PaperRejectedTransitionAudit {
        #[arg(long)]
        config: String,
    },
    /// Research-only Risk Governor required transition audit; Risk Governor required for risk-sensitive transitions and no bypass allowed.
    RiskRequiredTransitionAudit {
        #[arg(long)]
        config: String,
    },
    /// Research-only missing artifact finding policy; missing docs/tests/examples become findings and never silent success.
    MissingArtifactFindingPolicy {
        #[arg(long)]
        config: String,
    },
    /// Research-only final verification gate v2; verification is not full workspace acceptance and full acceptance still requires finished && passed cargo workspace.
    FinalVerificationGateV2 {
        #[arg(long)]
        config: String,
    },
    /// Research-only dual-agent review loop v2; patch-closure accounting only and no live execution path.
    DualAgentReviewLoopV2 {
        #[arg(long)]
        config: String,
    },
    /// Research-only paper lifecycle warning closure; paper candidate is not an order, not execution, and no live transition is allowed.
    PaperLifecycleWarningClosure {
        #[arg(long)]
        config: String,
    },
    /// Research-only paper candidate transition coverage; paper-only transition audit only and no order path.
    PaperCandidateTransitionCoverage {
        #[arg(long)]
        config: String,
    },
    /// Research-only paper candidate gate completeness; paper-only candidate gates only and no live transition path.
    PaperCandidateGateCompleteness {
        #[arg(long)]
        config: String,
    },
    /// Research-only paper candidate evidence depth closure; paper evidence closure only and no broker/order/account implication.
    PaperCandidateEvidenceDepthClosure {
        #[arg(long)]
        config: String,
    },
    /// Research-only paper candidate trace closure; paper-only trace completeness only and no runtime decision path.
    PaperCandidateTraceClosure {
        #[arg(long)]
        config: String,
    },
    /// Research-only paper candidate stability closure; replay/stability only and not live readiness.
    PaperCandidateStabilityClosure {
        #[arg(long)]
        config: String,
    },
    /// Research-only Risk Governor batch veto warning closure; paper-only veto closure with final veto preserved.
    RiskGovernorBatchVetoWarningClosure {
        #[arg(long)]
        config: String,
    },
    /// Research-only Risk Governor no-bypass audit v2; Risk Governor remains required and no bypass is allowed.
    RiskGovernorNoBypassAuditV2 {
        #[arg(long)]
        config: String,
    },
    /// Research-only lower-confidence carry-forward closure; no silent confidence upgrade and no live activation.
    LowerConfidenceCarryForwardClosure {
        #[arg(long)]
        config: String,
    },
    /// Research-only paper lifecycle readiness gate v2; paper-only lifecycle only and live lifecycle remains forbidden.
    PaperLifecycleReadinessGateV2 {
        #[arg(long)]
        config: String,
    },
    /// Research-only paper candidate batch replay v2; paper-only replay only and no broker/live execution path.
    PaperCandidateBatchReplayV2 {
        #[arg(long)]
        config: String,
    },
    /// Research-only workspace acceptance truth recovery plan v6; full workspace separate from verification and focused tests.
    WorkspaceAcceptanceTruthRecoveryPlanV6 {
        #[arg(long)]
        config: String,
    },
    /// Research-only workspace compile-cost diagnosis v2; compile-cost diagnostics only and no fake full-pass claim.
    WorkspaceCompileCostDiagnosisV2 {
        #[arg(long)]
        config: String,
    },
    /// Research-only focused-vs-full gate bridge v2; verification is not full workspace acceptance.
    FocusedVsFullGateBridgeV2 {
        #[arg(long)]
        config: String,
    },
    /// Research-only safety coverage preservation v21; no runtime, no training, no live inference, and no broker/order/account path.
    SafetyCoveragePreservationV21 {
        #[arg(long)]
        config: String,
    },
    /// Read-only Control Tower verification patch closure panel; static/read-only output with no verification execution button and no train/runtime/live/order/account/browser controls.
    ControlTowerVerificationPatchClosure {
        #[arg(long)]
        config: String,
    },
    /// Read-only Control Tower paper lifecycle closure panel; static/read-only output with no promote-to-live button and no order/account controls.
    ControlTowerPaperLifecycleClosure {
        #[arg(long)]
        config: String,
    },
    /// Research-only Sprint 106 workspace acceptance recovery; workspace acceptance recovery only, focused is not full, no-run is not full acceptance, verification is not acceptance, no runtime implementation, no training, no live inference, no live trading, no order/account command, no safety test deletion, no hidden skips, and remote paths rejected.
    Sprint106WorkspaceAcceptanceRecover {
        #[arg(long)]
        config: String,
    },
    /// Research-only real no-run completion v22; no-run is not full acceptance and acceptance remains separate.
    RealNoRunCompletionV22 {
        #[arg(long)]
        config: String,
    },
    /// Research-only real full workspace attempt v22; finished and passed required for full workspace acceptance.
    RealFullWorkspaceAttemptV22 {
        #[arg(long)]
        config: String,
    },
    /// Research-only workspace compile-cost profile v3; compile/test diagnostics only and never a full acceptance claim.
    WorkspaceCompileCostProfileV3 {
        #[arg(long)]
        config: String,
    },
    /// Research-only cargo JSON no-run capture v2; no-run is not full acceptance and remote paths are rejected.
    CargoJsonNoRunCaptureV2 {
        #[arg(long)]
        config: String,
    },
    /// Research-only test binary inventory v3; inventory only with safety sentinels preserved and no hidden skips.
    TestBinaryInventoryV3 {
        #[arg(long)]
        config: String,
    },
    /// Research-only test binary explosion attribution; attribution only with no assertion deletion.
    TestBinaryExplosionAttribution {
        #[arg(long)]
        config: String,
    },
    /// Research-only integration target cost ranking; diagnostic ranking only and not full acceptance.
    IntegrationTargetCostRanking {
        #[arg(long)]
        config: String,
    },
    /// Research-only long-running rustc snapshot v2; compile observation only and not runtime readiness.
    LongRunningRustcSnapshotV2 {
        #[arg(long)]
        config: String,
    },
    /// Research-only fixture setup cost attribution v2; deterministic fixture/setup attribution only.
    FixtureSetupCostAttributionV2 {
        #[arg(long)]
        config: String,
    },
    /// Research-only artifact render cost attribution v2; local-only artifact rendering analysis.
    ArtifactRenderCostAttributionV2 {
        #[arg(long)]
        config: String,
    },
    /// Research-only CLI smoke cost attribution v2; representative, exhaustive, and safety smoke remain separate.
    CliSmokeCostAttributionV2 {
        #[arg(long)]
        config: String,
    },
    /// Research-only high-cost test family clusters; unsafe consolidation remains isolated.
    HighCostTestFamilyClusters {
        #[arg(long)]
        config: String,
    },
    /// Research-only safe test binary consolidation plan v2; no assertion deletion and safety sentinels remain preserved.
    SafeTestBinaryConsolidationPlanV2 {
        #[arg(long)]
        config: String,
    },
    /// Research-only shared fixture harness expansion plan v2; deterministic helper expansion only.
    SharedFixtureHarnessExpansionPlanV2 {
        #[arg(long)]
        config: String,
    },
    /// Research-only CLI smoke tiering plan v2; safety smoke remains separate.
    CliSmokeTieringPlanV2 {
        #[arg(long)]
        config: String,
    },
    /// Research-only workspace no-run recovery gate v7; no-run is not full acceptance.
    WorkspaceNoRunRecoveryGateV7 {
        #[arg(long)]
        config: String,
    },
    /// Research-only workspace full acceptance gate v7; finished and passed required for acceptance.
    WorkspaceFullAcceptanceGateV7 {
        #[arg(long)]
        config: String,
    },
    /// Research-only focused-vs-full bridge v3; focused is not full workspace acceptance.
    FocusedVsFullBridgeV3 {
        #[arg(long)]
        config: String,
    },
    /// Research-only acceptance truth gate v7; focused is not full and verification is not full.
    AcceptanceTruthGateV7 {
        #[arg(long)]
        config: String,
    },
    /// Research-only acceptance recovery patch plan; no assertion deletion and no hidden skips.
    AcceptanceRecoveryPatchPlan {
        #[arg(long)]
        config: String,
    },
    /// Research-only acceptance recovery verification; safety preserved and no hidden skips.
    AcceptanceRecoveryVerification {
        #[arg(long)]
        config: String,
    },
    /// Research-only safety coverage preservation v22; safety preserved with no runtime/live/order/account path.
    SafetyCoveragePreservationV22 {
        #[arg(long)]
        config: String,
    },
    /// Read-only Control Tower workspace acceptance recovery panel v7; static/read-only output with no run-tests button, no train button, no runtime button, no live button, and no order/account controls.
    ControlTowerWorkspaceAcceptanceRecoveryV7 {
        #[arg(long)]
        config: String,
    },
    /// Research-only first safe consolidation patch; no assertion deletion, no safety sentinel deletion, focused-is-not-full, no-run-is-not-full, verification-is-not-acceptance, no runtime implementation, no training, no live inference, no live trading, no order/account command, no hidden skips, local-only paths, and remote paths rejected.
    Sprint107SafeConsolidationPatch {
        #[arg(long)]
        config: String,
    },
    /// Research-only safe consolidation patch selection; first safe consolidation patch only and CommitteeCliSafety stays isolated.
    SafeConsolidationPatchSelection {
        #[arg(long)]
        config: String,
    },
    /// Research-only consolidation candidate risk review; high-risk sentinels stay rejected and no assertion deletion is allowed.
    ConsolidationCandidateRiskReview {
        #[arg(long)]
        config: String,
    },
    /// Research-only assertion migration ledger v1; no assertion deletion and no hidden skips.
    AssertionMigrationLedgerV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only assertion preservation verification v1; preserved assertions only and no silent deletion.
    AssertionPreservationVerificationV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only safety sentinel preservation v1; sentinels preserved and high-risk safety targets stay isolated.
    SafetySentinelPreservationV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only shared fixture harness application v1; deterministic output, local-only validation, and no secret caching.
    SharedFixtureHarnessApplicationV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only shared TOML builder application v1; local-only path validation preserved and remote paths rejected.
    SharedTomlBuilderApplicationV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only shared output-dir helper application v1; deterministic output roots preserved with no silent deletion.
    SharedOutputDirHelperApplicationV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only shared render helper application v1; stable ordering preserved and no runtime/UI execution implied.
    SharedRenderHelperApplicationV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only artifact render cache application v1; opt-in only, local-only, and secret-free.
    ArtifactRenderCacheApplicationV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only CLI smoke tiering application v1; representative, exhaustive, and safety smoke stay separated.
    CliSmokeTieringApplicationV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only consolidated test target manifest v1; first safe consolidation patch only.
    ConsolidatedTestTargetManifestV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only retired narrow target manifest v1; retirement only after assertion migration with equivalent coverage.
    RetiredNarrowTargetManifestV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only test binary delta v4; sample-backed delta is not a measured reduction claim.
    TestBinaryDeltaV4 {
        #[arg(long)]
        config: String,
    },
    /// Research-only post-patch workspace no-run v23; no-run is not full acceptance.
    PostPatchWorkspaceNoRunV23 {
        #[arg(long)]
        config: String,
    },
    /// Research-only post-patch workspace full v23; finished and passed required for full workspace acceptance.
    PostPatchWorkspaceFullV23 {
        #[arg(long)]
        config: String,
    },
    /// Research-only workspace no-run recovery gate v8; no-run is not full acceptance.
    WorkspaceNoRunRecoveryGateV8 {
        #[arg(long)]
        config: String,
    },
    /// Research-only workspace full acceptance gate v8; finished and passed required for acceptance.
    WorkspaceFullAcceptanceGateV8 {
        #[arg(long)]
        config: String,
    },
    /// Research-only acceptance truth gate v8; focused is not full, CLI smoke is not full, and verification is not full.
    AcceptanceTruthGateV8 {
        #[arg(long)]
        config: String,
    },
    /// Read-only Control Tower safe consolidation patch panel v1; static/read-only output with no run-tests button and no train/runtime/live/order/account controls.
    ControlTowerSafeConsolidationPatchV1 {
        #[arg(long)]
        config: String,
    },
    /// Read-only Control Tower workspace acceptance recovery panel v8; static/read-only output with no run-tests button and no train/runtime/live/order/account controls.
    ControlTowerWorkspaceAcceptanceRecoveryV8 {
        #[arg(long)]
        config: String,
    },
    /// Research-only second safe consolidation patch v2; paper-only, second-smallest-patch-only, no assertion deletion, no safety sentinel deletion, focused-is-not-full, no-run-is-not-full, verification-is-not-acceptance, no runtime implementation, no training, no live inference, no live trading, no order/account command, no hidden skips, local-only paths, and remote paths rejected.
    Sprint108SafeConsolidationPatchV2 {
        #[arg(long)]
        config: String,
    },
    /// Research-only Sprint 107 verification reconciliation; 5.5 verification is not acceptance.
    Sprint107VerificationReconcile {
        #[arg(long)]
        config: String,
    },
    /// Research-only independent verification closure v1; verification closure does not imply full workspace acceptance.
    IndependentVerificationClosureV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only verification patch carry-forward; previously fixed verification patches must remain effective.
    VerificationPatchCarryForward {
        #[arg(long)]
        config: String,
    },
    /// Research-only second safe consolidation patch selection; previous retired target is not reselected.
    SecondSafeConsolidationPatchSelection {
        #[arg(long)]
        config: String,
    },
    /// Research-only assertion migration ledger v2; no assertion deletion.
    AssertionMigrationLedgerV2 {
        #[arg(long)]
        config: String,
    },
    /// Research-only equivalent coverage proof v1; coverage required before retirement.
    EquivalentCoverageProofV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only retired target safety audit v2; unsafe retirement stays blocked.
    RetiredTargetSafetyAuditV2 {
        #[arg(long)]
        config: String,
    },
    /// Research-only safety sentinel preservation v2; sentinels remain isolated and preserved.
    SafetySentinelPreservationV2 {
        #[arg(long)]
        config: String,
    },
    /// Research-only shared fixture harness expansion v2; deterministic output preserved.
    SharedFixtureHarnessExpansionV2 {
        #[arg(long)]
        config: String,
    },
    /// Research-only shared render helper expansion v2; deterministic ordering preserved.
    SharedRenderHelperExpansionV2 {
        #[arg(long)]
        config: String,
    },
    /// Research-only CLI smoke tiering application v2; safety smoke remains explicit.
    CliSmokeTieringApplicationV2 {
        #[arg(long)]
        config: String,
    },
    /// Research-only test binary delta v5; sample-backed delta is not measured.
    TestBinaryDeltaV5 {
        #[arg(long)]
        config: String,
    },
    /// Research-only extended no-run observation v1; timeout observation is not full acceptance.
    ExtendedNoRunObservationV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only timeout cleanup verification v1; timeout is not pass.
    TimeoutCleanupVerificationV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only workspace no-run recovery gate v9; no-run is not full acceptance.
    WorkspaceNoRunRecoveryGateV9 {
        #[arg(long)]
        config: String,
    },
    /// Research-only workspace full acceptance gate v9; finished and passed required for acceptance.
    WorkspaceFullAcceptanceGateV9 {
        #[arg(long)]
        config: String,
    },
    /// Research-only acceptance truth gate v9; focused is not full, CLI smoke is not full, and verification is not full.
    AcceptanceTruthGateV9 {
        #[arg(long)]
        config: String,
    },
    /// Read-only Control Tower safe consolidation patch panel v2; static/read-only output with no run-tests button and no train/runtime/live/order/account/browser controls.
    ControlTowerSafeConsolidationPatchV2 {
        #[arg(long)]
        config: String,
    },
    /// Read-only Control Tower workspace acceptance recovery panel v9; static/read-only output with no run-tests button and no train/runtime/live/order/account/browser controls.
    ControlTowerWorkspaceAcceptanceRecoveryV9 {
        #[arg(long)]
        config: String,
    },
    /// Research-only third safe consolidation patch v3; paper-only, third-smallest-patch-only, no assertion deletion, no safety sentinel deletion, focused-is-not-full, no-run-is-not-full, and local-only paths only.
    Sprint109SafeConsolidationPatchV3 {
        #[arg(long)]
        config: String,
    },
    /// Research-only Sprint 108 verification carry-forward; 5.5 verification is not acceptance and carry-forward is not full workspace acceptance.
    Sprint108VerificationCarryForward {
        #[arg(long)]
        config: String,
    },
    /// Research-only previous patch ledger carry-forward; cumulative ledger truth only.
    PreviousPatchLedgerCarryForward {
        #[arg(long)]
        config: String,
    },
    /// Research-only cumulative assertion migration ledger; no assertion deletion.
    CumulativeAssertionMigrationLedger {
        #[arg(long)]
        config: String,
    },
    /// Research-only third safe consolidation patch selection; previous retired targets stay excluded.
    ThirdSafeConsolidationPatchSelection {
        #[arg(long)]
        config: String,
    },
    /// Research-only assertion migration ledger v3; no assertion deletion.
    AssertionMigrationLedgerV3 {
        #[arg(long)]
        config: String,
    },
    /// Research-only equivalent coverage proof v2; coverage required before retirement.
    EquivalentCoverageProofV2 {
        #[arg(long)]
        config: String,
    },
    /// Research-only retired target safety audit v3; unsafe retirement stays blocked.
    RetiredTargetSafetyAuditV3 {
        #[arg(long)]
        config: String,
    },
    /// Research-only safety sentinel preservation v3; sentinels remain isolated and preserved.
    SafetySentinelPreservationV3 {
        #[arg(long)]
        config: String,
    },
    /// Research-only shared fixture harness expansion v3; deterministic output preserved.
    SharedFixtureHarnessExpansionV3 {
        #[arg(long)]
        config: String,
    },
    /// Research-only shared render helper expansion v3; deterministic ordering preserved.
    SharedRenderHelperExpansionV3 {
        #[arg(long)]
        config: String,
    },
    /// Research-only CLI smoke tiering application v3; safety smoke remains explicit.
    CliSmokeTieringApplicationV3 {
        #[arg(long)]
        config: String,
    },
    /// Research-only test binary delta v6; cumulative sample-backed delta is not measured.
    TestBinaryDeltaV6 {
        #[arg(long)]
        config: String,
    },
    /// Research-only cumulative binary delta v1; sample-backed delta is not measured.
    CumulativeBinaryDeltaV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only extended no-run observation v2; timeout observation is not full acceptance.
    ExtendedNoRunObservationV2 {
        #[arg(long)]
        config: String,
    },
    /// Research-only workspace cargo JSON progress capture v3; progress is not acceptance.
    WorkspaceCargoJsonProgressV3 {
        #[arg(long)]
        config: String,
    },
    /// Research-only timeout cleanup verification v2; timeout is not pass.
    TimeoutCleanupVerificationV2 {
        #[arg(long)]
        config: String,
    },
    /// Research-only workspace no-run recovery gate v10; no-run is not full acceptance.
    WorkspaceNoRunRecoveryGateV10 {
        #[arg(long)]
        config: String,
    },
    /// Research-only workspace full acceptance gate v10; finished and passed required for acceptance.
    WorkspaceFullAcceptanceGateV10 {
        #[arg(long)]
        config: String,
    },
    /// Research-only acceptance truth gate v10; focused is not full, CLI smoke is not full, and verification is not full.
    AcceptanceTruthGateV10 {
        #[arg(long)]
        config: String,
    },
    /// Read-only Control Tower safe consolidation patch panel v3; static/read-only output with no run-tests button and no train/runtime/live/order/account/browser controls.
    ControlTowerSafeConsolidationPatchV3 {
        #[arg(long)]
        config: String,
    },
    /// Read-only Control Tower workspace acceptance recovery panel v10; static/read-only output with no run-tests button and no train/runtime/live/order/account/browser controls.
    ControlTowerWorkspaceAcceptanceRecoveryV10 {
        #[arg(long)]
        config: String,
    },
    /// Research-only fourth safe consolidation patch v4; Sprint 109 validation reconciliation is not full acceptance and only one low-risk target may be retired.
    Sprint110SafeConsolidationPatchV4 {
        #[arg(long)]
        config: String,
    },
    /// Research-only Sprint 109 validation reconciliation; focused/CLI/build imports are not full workspace acceptance.
    Sprint109ValidationReconcile {
        #[arg(long)]
        config: String,
    },
    /// Research-only Sprint 109 focused suite import; focused pass is not full workspace acceptance.
    Sprint109FocusedSuiteImport {
        #[arg(long)]
        config: String,
    },
    /// Research-only Sprint 109 CLI smoke import; CLI smoke is not full workspace acceptance.
    Sprint109CliSmokeImport {
        #[arg(long)]
        config: String,
    },
    /// Research-only Sprint 109 cargo build import; cargo build is not full workspace acceptance.
    Sprint109CargoBuildImport {
        #[arg(long)]
        config: String,
    },
    /// Research-only Sprint 109 workspace timeout import; timeout cleanup is not pass and not acceptance.
    Sprint109WorkspaceTimeoutImport {
        #[arg(long)]
        config: String,
    },
    /// Research-only fourth safe consolidation patch selection; prior retired targets stay excluded.
    FourthSafeConsolidationPatchSelection {
        #[arg(long)]
        config: String,
    },
    /// Research-only assertion migration ledger v4; no assertion deletion.
    AssertionMigrationLedgerV4 {
        #[arg(long)]
        config: String,
    },
    /// Research-only cumulative assertion migration ledger v2; no assertion deletion.
    CumulativeAssertionMigrationLedgerV2 {
        #[arg(long)]
        config: String,
    },
    /// Research-only equivalent coverage proof v3; coverage required before retirement.
    EquivalentCoverageProofV3 {
        #[arg(long)]
        config: String,
    },
    /// Research-only retired target safety audit v4; unsafe retirement stays blocked.
    RetiredTargetSafetyAuditV4 {
        #[arg(long)]
        config: String,
    },
    /// Research-only safety sentinel preservation v4; sentinels remain isolated and preserved.
    SafetySentinelPreservationV4 {
        #[arg(long)]
        config: String,
    },
    /// Research-only shared fixture harness expansion v4; deterministic output preserved.
    SharedFixtureHarnessExpansionV4 {
        #[arg(long)]
        config: String,
    },
    /// Research-only shared render helper expansion v4; deterministic ordering preserved.
    SharedRenderHelperExpansionV4 {
        #[arg(long)]
        config: String,
    },
    /// Research-only CLI smoke tiering application v4; safety smoke remains explicit.
    CliSmokeTieringApplicationV4 {
        #[arg(long)]
        config: String,
    },
    /// Research-only test binary delta v7; cumulative sample-backed delta is not measured.
    TestBinaryDeltaV7 {
        #[arg(long)]
        config: String,
    },
    /// Research-only cumulative binary delta v2; sample-backed cumulative delta is not measured.
    CumulativeBinaryDeltaV2 {
        #[arg(long)]
        config: String,
    },
    /// Research-only extended no-run observation v3; timeout observation is not acceptance.
    ExtendedNoRunObservationV3 {
        #[arg(long)]
        config: String,
    },
    /// Research-only workspace cargo JSON progress capture v4; progress is not acceptance.
    WorkspaceCargoJsonProgressV4 {
        #[arg(long)]
        config: String,
    },
    /// Research-only timeout cleanup verification v3; timeout cleanup is not pass.
    TimeoutCleanupVerificationV3 {
        #[arg(long)]
        config: String,
    },
    /// Research-only workspace no-run recovery gate v11; no-run is not full acceptance.
    WorkspaceNoRunRecoveryGateV11 {
        #[arg(long)]
        config: String,
    },
    /// Research-only workspace full acceptance gate v11; finished and passed required for acceptance.
    WorkspaceFullAcceptanceGateV11 {
        #[arg(long)]
        config: String,
    },
    /// Research-only acceptance truth gate v11; focused is not full, no-run is not full, cargo build is not full, and CLI smoke is not full.
    AcceptanceTruthGateV11 {
        #[arg(long)]
        config: String,
    },
    /// Read-only Control Tower safe consolidation patch panel v4; static/read-only output with no run-tests button and no train/runtime/live/order/account/browser controls.
    ControlTowerSafeConsolidationPatchV4 {
        #[arg(long)]
        config: String,
    },
    /// Read-only Control Tower workspace acceptance recovery panel v11; static/read-only output with no run-tests button and no train/runtime/live/order/account/browser controls.
    ControlTowerWorkspaceAcceptanceRecoveryV11 {
        #[arg(long)]
        config: String,
    },
    /// Research-only Sprint 111 workspace timeout root-cause bundle; timeout-root-cause-only, fifth patch not auto-applied, and full acceptance remains separate.
    Sprint111WorkspaceTimeoutRootCause {
        #[arg(long)]
        config: String,
    },
    /// Research-only Sprint 110 baseline truth import; supporting evidence only and never full acceptance.
    Sprint110BaselineTruthImport {
        #[arg(long)]
        config: String,
    },
    /// Research-only cumulative safe patch ledger v3; cumulative sample-backed delta is not measured timing.
    CumulativeSafePatchLedgerV3 {
        #[arg(long)]
        config: String,
    },
    /// Research-only workspace timeout root-cause report; cargo progress remains diagnostic only.
    WorkspaceTimeoutRootCause {
        #[arg(long)]
        config: String,
    },
    /// Research-only workspace no-run progress trace v1; no-run is not full acceptance.
    WorkspaceNoRunProgressTraceV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only cargo JSON progress capture v5; progress evidence is diagnostic only and not acceptance.
    CargoJsonProgressCaptureV5 {
        #[arg(long)]
        config: String,
    },
    /// Research-only cargo artifact progress timeline; artifact progress is diagnostic only and not acceptance.
    CargoArtifactProgressTimeline {
        #[arg(long)]
        config: String,
    },
    /// Research-only cargo target stall attribution; timeout root-cause-only and no fifth patch auto-apply.
    CargoTargetStallAttribution {
        #[arg(long)]
        config: String,
    },
    /// Research-only integration test binary stall report; diagnostic only and no safety sentinel deletion.
    IntegrationTestBinaryStall {
        #[arg(long)]
        config: String,
    },
    /// Research-only test family fanout map v2; sentinel isolation stays explicit and preserved.
    TestFamilyFanoutMapV2 {
        #[arg(long)]
        config: String,
    },
    /// Research-only workspace target cluster map v2; timeout-root-cause-only and local-only.
    WorkspaceTargetClusterMapV2 {
        #[arg(long)]
        config: String,
    },
    /// Research-only high-fanout residual target report; already retired targets stay excluded.
    HighFanoutResidualTarget {
        #[arg(long)]
        config: String,
    },
    /// Research-only remaining safe candidate pool; fifth patch is not auto-applied and sentinels stay excluded.
    RemainingSafeCandidatePool {
        #[arg(long)]
        config: String,
    },
    /// Research-only fifth patch candidate preselection; paper-only and not an applied patch.
    FifthPatchCandidatePreselection {
        #[arg(long)]
        config: String,
    },
    /// Research-only fifth patch decision gate; patch is not auto-applied and full acceptance stays separate.
    FifthPatchDecisionGate {
        #[arg(long)]
        config: String,
    },
    /// Research-only assertion ledger continuity check v1; no assertion deletion and no hidden skips.
    AssertionLedgerContinuityCheckV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only equivalent coverage continuity check v1; equivalent coverage remains mandatory before retirement.
    EquivalentCoverageContinuityCheckV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only timeout window adequacy v1; timeout cleanup is not pass and timeout is not acceptance.
    TimeoutWindowAdequacyV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only acceptance evidence strength v1; only full finished and passed workspace tests can claim full acceptance.
    AcceptanceEvidenceStrengthV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only workspace recovery decision v1; timeout-root-cause-only and fifth patch remains gated.
    WorkspaceRecoveryDecisionV1 {
        #[arg(long)]
        config: String,
    },
    /// Read-only Control Tower workspace timeout root-cause panel; static/read-only with no run-tests button and no train/runtime/live/order/account controls.
    ControlTowerWorkspaceTimeoutRootCause {
        #[arg(long)]
        config: String,
    },
    /// Read-only Control Tower fifth patch decision panel; static/read-only with no apply-patch button and no train/runtime/live/order/account controls.
    ControlTowerFifthPatchDecision {
        #[arg(long)]
        config: String,
    },
    /// Research-only Sprint 112 workspace diagnostic pilot bundle; research-only, paper-only, diagnostic-only, nextest-is-not-acceptance, sccache-is-not-speedup-proof, fifth-patch-not-applied, and local-only.
    Sprint112WorkspaceDiagnosticPilot {
        #[arg(long)]
        config: String,
    },
    /// Research-only Sprint 111 baseline truth import for Sprint 112; supporting evidence only and never full acceptance.
    Sprint111BaselineTruthImport {
        #[arg(long)]
        config: String,
    },
    /// Research-only nextest availability v1; diagnostic-only and not workspace acceptance.
    NextestAvailabilityV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only nextest pilot execution v1; diagnostic-only and nextest is not acceptance.
    NextestPilotExecutionV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only nextest slow target attribution v1; diagnostic-only and no hidden skips.
    NextestSlowTargetAttributionV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only sccache availability v1; diagnostic-only and not speedup proof.
    SccacheAvailabilityV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only sccache local-only policy v1; local-only, deterministic, and no secrets.
    SccacheLocalOnlyPolicyV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only sccache effect estimate v1; diagnostic-only and no guaranteed speedup claim.
    SccacheEffectEstimateV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only cargo check timing capture v1; diagnostic-only and not full acceptance.
    CargoCheckTimingCaptureV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only cargo build timing capture v1; diagnostic-only and not full acceptance.
    CargoBuildTimingCaptureV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only cargo JSON progress capture v6; diagnostic-only and not acceptance.
    CargoJsonProgressCaptureV6 {
        #[arg(long)]
        config: String,
    },
    /// Research-only workspace diagnostic evidence matrix v1; full acceptance requires real finished and passed full workspace tests.
    WorkspaceDiagnosticEvidenceMatrixV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only workspace timeout root-cause v2; observed and inferred evidence remain separate.
    WorkspaceTimeoutRootCauseV2 {
        #[arg(long)]
        config: String,
    },
    /// Research-only remaining safe candidate pool v2; fifth patch remains gated and not applied.
    RemainingSafeCandidatePoolV2 {
        #[arg(long)]
        config: String,
    },
    /// Research-only fifth patch decision gate v2; re-evaluation only and no patch application.
    FifthPatchDecisionGateV2 {
        #[arg(long)]
        config: String,
    },
    /// Research-only fifth patch no-apply guarantee v1; proves no fifth patch applied in this sprint.
    FifthPatchNoApplyGuaranteeV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only acceptance evidence strength v2; only full finished and passed workspace tests can claim full acceptance.
    AcceptanceEvidenceStrengthV2 {
        #[arg(long)]
        config: String,
    },
    /// Research-only workspace recovery decision v2; diagnostic-only and fifth patch remains separate.
    WorkspaceRecoveryDecisionV2 {
        #[arg(long)]
        config: String,
    },
    /// Read-only Control Tower workspace diagnostic pilot panel; static/read-only with no run button and no train/runtime/live/order/account controls.
    ControlTowerWorkspaceDiagnosticPilot {
        #[arg(long)]
        config: String,
    },
    /// Read-only Control Tower fifth patch reevaluation panel; static/read-only with no apply-patch button and no train/runtime/live/order/account controls.
    ControlTowerFifthPatchReevaluation {
        #[arg(long)]
        config: String,
    },
    /// Research-only Sprint 113 real workspace observation drilldown; research-only, paper-only, real-observation-diagnostic, fifth-patch-not-applied, nextest-is-not-cargo-workspace-acceptance, sccache-is-not-speedup-proof, cargo-progress-is-not-acceptance, timeout-cleanup-is-not-pass, focused-is-not-full, CLI-smoke-is-not-full, cargo-build-is-not-full, no-run-is-not-full, no assertion deletion, no safety sentinel deletion, no runtime implementation, no training, no live inference, no live trading, no order/account command, no runtime LLM live decision path, no investor impersonation, no auto-activation of 18 live agents, no silent confidence upgrade, no safety test deletion, no hidden skips, local-only paths, and remote paths rejected.
    Sprint113RealWorkspaceObservation {
        #[arg(long)]
        config: String,
    },
    /// Research-only Sprint 112 baseline truth import for Sprint 113; supporting-only and imported_as_full_acceptance=false.
    Sprint112BaselineTruthImport {
        #[arg(long)]
        config: String,
    },
    /// Research-only Sprint 112 verification patch carry-forward for Sprint 113; storage_report, summary, actual cleanup counts, actual cargo JSON parsing, real observation preservation, and LowRiskCandidate gate stay carried forward.
    Sprint112VerificationPatchCarryForward {
        #[arg(long)]
        config: String,
    },
    /// Research-only suspect target family registry v1; retired and sentinel targets stay excluded.
    SuspectTargetFamilyRegistryV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only real cargo no-run observation v1; diagnostic-only and no-run is not full acceptance.
    RealCargoNoRunObservationV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only real cargo JSON progress observation v1; progress is not acceptance and actual JSON parsing stays explicit.
    RealCargoJsonProgressObservationV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only real nextest probe v1; diagnostic-only and nextest is not cargo workspace acceptance.
    RealNextestProbeV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only real sccache probe v1; local-only diagnostic-only and not speedup proof.
    RealSccacheProbeV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only workspace timeout root-cause v3; observed and inferred evidence stay separate and no fifth patch is applied.
    WorkspaceTimeoutRootCauseV3 {
        #[arg(long)]
        config: String,
    },
    /// Research-only fifth patch decision gate v3; gate-upgrade only, next sprint only, and patch not applied.
    FifthPatchDecisionGateV3 {
        #[arg(long)]
        config: String,
    },
    /// Research-only fifth patch no-apply guarantee v2; proves no fifth patch applied, no files retired, and no assertions moved.
    FifthPatchNoApplyGuaranteeV2 {
        #[arg(long)]
        config: String,
    },
    /// Research-only acceptance truth gate v14; only a finished and passed full workspace run can claim full acceptance.
    AcceptanceTruthGateV14 {
        #[arg(long)]
        config: String,
    },
    /// Read-only Control Tower real workspace observation panel; static/read-only with no run button and no train/runtime/live/order/account controls.
    ControlTowerRealWorkspaceObservation {
        #[arg(long)]
        config: String,
    },
    /// Read-only Control Tower fifth patch evidence gate panel; static/read-only with no apply-patch button and no train/runtime/live/order/account controls.
    ControlTowerFifthPatchEvidenceGate {
        #[arg(long)]
        config: String,
    },
    /// Research-only Sprint 114 mixed-family isolation bundle; research-only, paper-only, mixed-family-isolation-only, fifth-patch-not-applied, fifth-patch-ready-does-not-mean-applied, focused-is-not-full, CLI-smoke-is-not-full, cargo-build-is-not-full, no-run-is-not-full, cargo-progress-is-not-acceptance, timeout-cleanup-is-not-pass, no assertion deletion, no safety sentinel deletion, no runtime implementation, no training, no live inference, no live trading, no order/account command, no runtime LLM live decision path, no investor impersonation, no auto-activation of 18 live agents, no silent confidence upgrade, no safety test deletion, no hidden skips, local-only paths, and remote paths rejected.
    Sprint114MixedFamilyIsolation {
        #[arg(long)]
        config: String,
    },
    /// Research-only Sprint 113 baseline truth import for Sprint 114; supporting-only and imported_as_full_acceptance=false.
    Sprint113BaselineTruthImport {
        #[arg(long)]
        config: String,
    },
    /// Research-only still-mixed family registry v1; preserves isolated families and suspect targets without applying the fifth patch.
    StillMixedFamilyRegistryV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only integration fanout narrowing v1; observed and inferred evidence remain separate.
    IntegrationFanoutNarrowingV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only link-time narrowing v1; observed and inferred evidence remain separate.
    LinkTimeNarrowingV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only macro-expansion narrowing v1; observed and inferred evidence remain separate.
    MacroExpansionNarrowingV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only suspect target decomposition v1; target-level pressure only and no fifth patch application.
    SuspectTargetDecompositionV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only target assertion inventory v1; no assertion deletion and no silent confidence upgrade.
    TargetAssertionInventoryV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only assertion migration feasibility drilldown v1; fifth patch not applied and readiness never means applied.
    AssertionMigrationFeasibilityDrilldownV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only equivalent coverage feasibility drilldown v1; destination proof required before any later patch recommendation.
    EquivalentCoverageFeasibilityDrilldownV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only fifth patch decision gate v4; gate-only, next sprint only, and patch not applied.
    FifthPatchDecisionGateV4 {
        #[arg(long)]
        config: String,
    },
    /// Research-only fifth patch apply plan for next sprint; plan only, next sprint only, and patch not applied.
    FifthPatchApplyPlanForNextSprint {
        #[arg(long)]
        config: String,
    },
    /// Research-only fifth patch no-apply guarantee v3; proves no fifth patch applied, no files retired, and no assertions moved.
    FifthPatchNoApplyGuaranteeV3 {
        #[arg(long)]
        config: String,
    },
    /// Research-only candidate stop consolidation v1; stop-consolidation is allowed and does not apply a patch.
    CandidateStopConsolidationV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only cargo JSON suspect target trace v1; progress is not acceptance.
    CargoJsonSuspectTargetTraceV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only link/macro evidence matrix v1; observed and inferred evidence stay separate.
    LinkMacroEvidenceMatrixV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only integration fanout evidence matrix v1; observed and inferred evidence stay separate.
    IntegrationFanoutEvidenceMatrixV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only acceptance truth gate v15; only a finished and passed full workspace run can claim full acceptance.
    AcceptanceTruthGateV15 {
        #[arg(long)]
        config: String,
    },
    /// Read-only Control Tower mixed-family isolation panel; static/read-only with no run button and no train/runtime/live/order/account controls.
    ControlTowerMixedFamilyIsolation {
        #[arg(long)]
        config: String,
    },
    /// Read-only Control Tower fifth patch readiness v4 panel; static/read-only with no apply-patch button, no run button, and no train/runtime/live/order/account controls.
    ControlTowerFifthPatchReadinessV4 {
        #[arg(long)]
        config: String,
    },
    /// Research-only Sprint 115 consolidation stop/resume governance bundle; research-only, paper-only, consolidation-governance-only, fifth-patch-not-applied, no-target-retirement, no-assertion-movement, stop-consolidation-is-valid, focused-is-not-full, CLI-smoke-is-not-full, cargo-build-is-not-full, no-run-is-not-full, cargo-progress-is-not-acceptance, timeout-cleanup-is-not-pass, no assertion deletion, no safety sentinel deletion, no runtime implementation, no training, no live inference, no live trading, no order/account command, no runtime LLM live decision path, no investor impersonation, no auto-activation of 18 live agents, no silent confidence upgrade, no safety test deletion, no hidden skips, local-only paths, and remote paths rejected.
    Sprint115ConsolidationGovernance {
        #[arg(long)]
        config: String,
    },
    /// Research-only Sprint 114 baseline truth import for Sprint 115; supporting-only and imported_as_full_acceptance=false.
    Sprint114BaselineTruthImport {
        #[arg(long)]
        config: String,
    },
    /// Research-only Sprint 114 stop recommendation carry-forward; stop-consolidation is valid and fifth patch remains not applied.
    Sprint114StopRecommendationCarryForward {
        #[arg(long)]
        config: String,
    },
    /// Research-only consolidation stop decision v1; stop-consolidation is valid and does not apply the fifth patch.
    ConsolidationStopDecisionV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only consolidation resume decision v1; resume requires proof first and does not move assertions.
    ConsolidationResumeDecisionV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only consolidation decision matrix v1; pause/stop/split are valid outcomes and no patch is applied.
    ConsolidationDecisionMatrixV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only assertion destination proof plan v1; proof before movement and no assertion movement this sprint.
    AssertionDestinationProofPlanV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only assertion destination capacity v1; capacity is proof-only and no assertions are moved.
    AssertionDestinationCapacityV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only evidence blur risk v1; evidence blur can block consolidation resume.
    EvidenceBlurRiskV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only assertion destination proof gate v1; proof must pass before any later movement.
    AssertionDestinationProofGateV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only evidence blur risk gate v1; controlled blur does not mean the fifth patch was applied.
    EvidenceBlurRiskGateV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only fifth patch resume gate v5; gate-only, later sprint only, and patch not applied.
    FifthPatchResumeGateV5 {
        #[arg(long)]
        config: String,
    },
    /// Research-only fifth patch stop gate v1; stop is valid and patch not applied.
    FifthPatchStopGateV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only candidate stop consolidation v2; honest stop recommendation with resume only after proof.
    CandidateStopConsolidationV2 {
        #[arg(long)]
        config: String,
    },
    /// Research-only consolidation track pause v1; pause is valid and no consolidation is applied.
    ConsolidationTrackPauseV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only workspace timeout track split v1; split diagnostics from consolidation and keep acceptance blocked.
    WorkspaceTimeoutTrackSplitV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only workspace timeout diagnostic track plan v1; diagnostic-only and not acceptance.
    WorkspaceTimeoutDiagnosticTrackPlanV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only workspace no-run observation plan v2; no-run is not full acceptance.
    WorkspaceNoRunObservationPlanV2 {
        #[arg(long)]
        config: String,
    },
    /// Research-only workspace full observation plan v2; only a finished and passed full workspace run can claim full acceptance.
    WorkspaceFullObservationPlanV2 {
        #[arg(long)]
        config: String,
    },
    /// Research-only acceptance truth gate v16; only a finished and passed full workspace run can claim full acceptance.
    AcceptanceTruthGateV16 {
        #[arg(long)]
        config: String,
    },
    /// Read-only Control Tower consolidation governance panel; static/read-only with no apply button, no run button, and no train/runtime/live/order/account controls.
    ControlTowerConsolidationGovernance {
        #[arg(long)]
        config: String,
    },
    /// Read-only Control Tower workspace timeout track panel; static/read-only with no run button and no train/runtime/live/order/account controls.
    ControlTowerWorkspaceTimeoutTrack {
        #[arg(long)]
        config: String,
    },
    /// Research-only Sprint 116 workspace timeout execution bundle; research-only, paper-only, timeout-track-only, consolidation-paused, fifth-patch-not-applied, no-assertion-movement, no-target-retirement, focused-is-not-full, CLI-smoke-is-not-full, cargo-build-is-not-full, no-run-is-not-full, cargo-progress-is-not-acceptance, artifact-ordering-is-not-acceptance, timeout-cleanup-is-not-pass, no assertion deletion, no safety sentinel deletion, no runtime implementation, no training, no live inference, no live trading, no order/account command, no runtime LLM live decision path, no investor impersonation, no auto-activation of 18 live agents, no silent confidence upgrade, no safety test deletion, no hidden skips, local-only paths, and remote paths rejected.
    Sprint116WorkspaceTimeoutTrack {
        #[arg(long)]
        config: String,
    },
    /// Research-only Sprint 115 baseline truth import for Sprint 116; supporting-only and imported_as_full_acceptance=false.
    Sprint115BaselineTruthImport {
        #[arg(long)]
        config: String,
    },
    /// Research-only consolidation paused carry-forward; consolidation remains paused and no fifth patch is applied.
    ConsolidationPausedCarryForward {
        #[arg(long)]
        config: String,
    },
    /// Research-only workspace timeout track activation v1; timeout-track-only and consolidation remains separated.
    WorkspaceTimeoutTrackActivationV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only workspace timeout observation backlog import v1; timeout-track-only backlog import with no acceptance upgrade.
    WorkspaceTimeoutObservationBacklogImportV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only workspace timeout observation backlog burndown v1; timeout-track-only and backlog reduction is not acceptance.
    WorkspaceTimeoutObservationBacklogBurndownV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only real no-run observation attempt v17; no-run-is-not-full and timeout remains diagnostic-only.
    RealNoRunObservationAttemptV17 {
        #[arg(long)]
        config: String,
    },
    /// Research-only real full workspace observation attempt v17; only a finished and passed full workspace run can claim full acceptance.
    RealFullWorkspaceObservationAttemptV17 {
        #[arg(long)]
        config: String,
    },
    /// Research-only real cargo JSON observation attempt v17; cargo progress is not acceptance and parsing is diagnostic-only.
    RealCargoJsonObservationAttemptV17 {
        #[arg(long)]
        config: String,
    },
    /// Research-only timeout cleanup consistency v1; timeout-cleanup-is-not-pass and counts are diagnostic-only.
    TimeoutCleanupConsistencyV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only cargo JSON parse quality v1; cargo-progress-is-not-acceptance and parsing remains diagnostic-only.
    CargoJsonParseQualityV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only workspace timeout evidence matrix v2; supporting-only evidence remains distinct from acceptance.
    WorkspaceTimeoutEvidenceMatrixV2 {
        #[arg(long)]
        config: String,
    },
    /// Research-only workspace timeout root cause v4; timeout-track-only and evidence-backed without overclaim.
    WorkspaceTimeoutRootCauseV4 {
        #[arg(long)]
        config: String,
    },
    /// Research-only acceptance truth gate v17; only full finished and passed workspace tests can claim full acceptance.
    AcceptanceTruthGateV17 {
        #[arg(long)]
        config: String,
    },
    /// Research-only workspace no-run recovery gate v17; no-run-is-not-full and no-run completion does not imply full acceptance.
    WorkspaceNoRunRecoveryGateV17 {
        #[arg(long)]
        config: String,
    },
    /// Research-only workspace full acceptance gate v17; full pass required and supporting-only evidence cannot claim full acceptance.
    WorkspaceFullAcceptanceGateV17 {
        #[arg(long)]
        config: String,
    },
    /// Read-only Control Tower workspace timeout track execution panel; static/read-only with no run button, no action button, and no train/runtime/live/order/account controls.
    ControlTowerWorkspaceTimeoutTrackExecution {
        #[arg(long)]
        config: String,
    },
    /// Read-only Control Tower acceptance truth panel v17; static/read-only with no action button and no train/runtime/live/order/account controls.
    ControlTowerAcceptanceTruthV17 {
        #[arg(long)]
        config: String,
    },
    /// Research-only Sprint 117 deferred real observation bundle; research-only, paper-only, deferred-real-observation-only, actual-observation-not-fixture, consolidation-paused, fifth-patch-not-applied, no-assertion-movement, no-target-retirement, focused-is-not-full, CLI-smoke-is-not-full, cargo-build-is-not-full, no-run-is-not-full, cargo-json-is-not-acceptance, timeout-cleanup-is-not-pass, no assertion deletion, no safety sentinel deletion, no runtime implementation, no training, no live inference, no live trading, no order/account command, no runtime LLM live decision path, no investor impersonation, no auto-activation of 18 live agents, no silent confidence upgrade, no safety test deletion, no hidden skips, local-only paths, and remote paths rejected.
    Sprint117DeferredRealObservation {
        #[arg(long)]
        config: String,
    },
    /// Research-only Sprint 116 baseline truth import; supporting-only and imported_as_full_acceptance=false.
    Sprint116BaselineTruthImport {
        #[arg(long)]
        config: String,
    },
    /// Research-only deferred observation selection v1; deferred-real-observation-only and actual-observation-not-fixture.
    DeferredObservationSelectionV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only deferred observation execution plan v1; deterministic order RealCargoJson, RealNoRun, RealFullWorkspace.
    DeferredObservationExecutionPlanV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only real no-run execution v18; no-run-is-not-full and timeout-cleanup-is-not-pass.
    RealNoRunExecutionV18 {
        #[arg(long)]
        config: String,
    },
    /// Research-only real full workspace execution v18; only a finished and passed full workspace run can claim full acceptance.
    RealFullWorkspaceExecutionV18 {
        #[arg(long)]
        config: String,
    },
    /// Research-only real cargo JSON execution v18; cargo-json-is-not-acceptance and parsing remains diagnostic-only.
    RealCargoJsonExecutionV18 {
        #[arg(long)]
        config: String,
    },
    /// Research-only cargo JSON actual parse v2; progress is not acceptance and actual parsing must stay separate from fixtures.
    CargoJsonActualParseV2 {
        #[arg(long)]
        config: String,
    },
    /// Research-only observation fixture separation v1; fixture must not overwrite actual observation.
    ObservationFixtureSeparationV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only actual-vs-carried-forward evidence v1; actual-observation-not-fixture and supporting-only evidence stays separate.
    ActualVsCarriedForwardEvidenceV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only observation backlog completion v2; backlog completion is not full workspace acceptance.
    ObservationBacklogCompletionV2 {
        #[arg(long)]
        config: String,
    },
    /// Research-only workspace timeout evidence matrix v3; supporting-only evidence remains distinct from acceptance.
    WorkspaceTimeoutEvidenceMatrixV3 {
        #[arg(long)]
        config: String,
    },
    /// Research-only workspace no-run recovery gate v18; no-run-is-not-full and no-run completion does not imply full acceptance.
    WorkspaceNoRunRecoveryGateV18 {
        #[arg(long)]
        config: String,
    },
    /// Research-only workspace full acceptance gate v18; only full finished and passed workspace tests can claim full acceptance.
    WorkspaceFullAcceptanceGateV18 {
        #[arg(long)]
        config: String,
    },
    /// Research-only acceptance truth gate v18; only full finished and passed workspace tests can claim full acceptance, and supporting-only evidence cannot claim full acceptance.
    AcceptanceTruthGateV18 {
        #[arg(long)]
        config: String,
    },
    /// Read-only Control Tower deferred observation execution panel; static/read-only with no run button, no action button, and no train/runtime/live/order/account controls.
    ControlTowerDeferredObservationExecution {
        #[arg(long)]
        config: String,
    },
    /// Read-only Control Tower acceptance truth panel v18; static/read-only with no action button and no train/runtime/live/order/account controls.
    ControlTowerAcceptanceTruthV18 {
        #[arg(long)]
        config: String,
    },
    /// Research-only Sprint 118 timeout reduction queue bundle; research-only, paper-only, timeout-reduction-only, consolidation-paused, fifth-patch-not-applied, no-assertion-movement, no-target-retirement, no-run-is-not-full, cargo-json-is-not-acceptance, stderr-is-not-acceptance, timeout-cleanup-is-not-pass, focused-is-not-full, CLI-smoke-is-not-full, cargo-build-is-not-full, no assertion deletion, no safety sentinel deletion, no runtime implementation, no training, no live inference, no live trading, no order/account command, no runtime LLM live decision path, no investor impersonation, no auto-activation of 18 live agents, no silent confidence upgrade, no safety test deletion, no hidden skips, local-only paths, and remote paths rejected.
    Sprint118TimeoutReductionQueue {
        #[arg(long)]
        config: String,
    },
    /// Research-only Sprint 117 baseline truth import for Sprint 118; supporting-only and imported_as_full_acceptance=false.
    Sprint117BaselineTruthImport {
        #[arg(long)]
        config: String,
    },
    /// Research-only cargo JSON failure reason analysis v1; cargo JSON is supporting-only and not acceptance.
    CargoJsonFailureReasonAnalysisV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only cargo JSON reason line classification v1; cargo JSON is diagnostic-only and not acceptance.
    CargoJsonReasonLineClassificationV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only cargo JSON target blocker extraction v1; blocker extraction narrows follow-up only.
    CargoJsonTargetBlockerExtractionV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only workspace timeout reduction hypothesis v1; queue-ready does not mean timeout solved.
    WorkspaceTimeoutReductionHypothesisV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only workspace timeout reduction queue v1; timeout-reduction-only and supporting-only until truthful full pass exists.
    WorkspaceTimeoutReductionQueueV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only truthful no-run attempt v19; no-run-is-not-full and timeout cannot pass.
    TruthfulNoRunAttemptV19 {
        #[arg(long)]
        config: String,
    },
    /// Research-only truthful full workspace attempt v19; only a finished and passed full run may claim full acceptance.
    TruthfulFullWorkspaceAttemptV19 {
        #[arg(long)]
        config: String,
    },
    /// Research-only truthful cargo JSON attempt v19; cargo JSON remains diagnostic-only and not acceptance.
    TruthfulCargoJsonAttemptV19 {
        #[arg(long)]
        config: String,
    },
    /// Research-only workspace timeout evidence matrix v4; full acceptance depends only on truthful full pass.
    WorkspaceTimeoutEvidenceMatrixV4 {
        #[arg(long)]
        config: String,
    },
    /// Research-only workspace timeout root-cause v6; narrowed evidence is still not acceptance.
    WorkspaceTimeoutRootCauseV6 {
        #[arg(long)]
        config: String,
    },
    /// Research-only workspace no-run recovery gate v19; no-run-is-not-full and no-run completion does not imply full acceptance.
    WorkspaceNoRunRecoveryGateV19 {
        #[arg(long)]
        config: String,
    },
    /// Research-only workspace full acceptance gate v19; only full finished and passed workspace tests can claim full acceptance.
    WorkspaceFullAcceptanceGateV19 {
        #[arg(long)]
        config: String,
    },
    /// Research-only acceptance truth gate v19; only full finished and passed workspace tests can claim full acceptance.
    AcceptanceTruthGateV19 {
        #[arg(long)]
        config: String,
    },
    /// Read-only Control Tower timeout reduction queue panel; static/read-only with no run button, no apply button, and no train/runtime/live/order/account controls.
    ControlTowerTimeoutReductionQueue {
        #[arg(long)]
        config: String,
    },
    /// Read-only Control Tower acceptance truth panel v19; static/read-only with no action button and no train/runtime/live/order/account controls.
    ControlTowerAcceptanceTruthV19 {
        #[arg(long)]
        config: String,
    },
    /// Minimal paper-only AI committee member opinion/event cycle; deterministic mock local logic, no broker/order/account, no training, no live inference.
    MinimalAiCommitteeCycle {
        #[arg(long)]
        config: String,
    },
    /// Research-only proposal warning closure; proposal remains paper-only research output and not order execution.
    ProposalWarningClosure {
        #[arg(long)]
        config: String,
    },
    /// Research-only proposal evidence completeness; local-only evidence accounting with no live data entitlement.
    ProposalEvidenceCompleteness {
        #[arg(long)]
        config: String,
    },
    /// Research-only proposal risk field completeness; paper-only risk scaffolding with Risk Governor still required.
    ProposalRiskFieldCompleteness {
        #[arg(long)]
        config: String,
    },
    /// Research-only entry timing condition completeness; paper-only timing with no broker, order, or account path.
    EntryTimingConditionCompleteness {
        #[arg(long)]
        config: String,
    },
    /// Research-only debate evidence closure; closes paper-only evidence gaps without enabling live debate.
    DebateEvidenceClosure {
        #[arg(long)]
        config: String,
    },
    /// Research-only debate evidence gap plan; local-only remediation planning with source-boundary guard.
    DebateEvidenceGapPlan {
        #[arg(long)]
        config: String,
    },
    /// Research-only debate dissent coverage; tracks disagreement for paper-only committee review.
    DebateDissentCoverage {
        #[arg(long)]
        config: String,
    },
    /// Research-only debate participation balance; paper-only role coverage with no runtime activation.
    DebateParticipationBalance {
        #[arg(long)]
        config: String,
    },
    /// Research-only chairman unsafe rule closure; paper-only repair only with no live rule mutation.
    ChairmanUnsafeRuleClosure {
        #[arg(long)]
        config: String,
    },
    /// Research-only chairman rulebook repair plan; no central AI core, no auto-apply, no live rule mutation.
    ChairmanRulebookRepairPlan {
        #[arg(long)]
        config: String,
    },
    /// Research-only chairman rulebook v2 draft; paper-only governance draft with live use forbidden.
    ChairmanRulebookV2Draft {
        #[arg(long)]
        config: String,
    },
    /// Research-only chairman rulebook approval gate; allows paper-only review only and blocks live activation.
    ChairmanRulebookApprovalGate {
        #[arg(long)]
        config: String,
    },
    /// Research-only rule audit trail completeness; audit-only review with no live mutation path.
    RuleAuditTrailCompleteness {
        #[arg(long)]
        config: String,
    },
    /// Research-only rulebook diff risk closure; paper-only diff closure with no runtime implementation.
    RulebookDiffRiskClosure {
        #[arg(long)]
        config: String,
    },
    /// Research-only scorecard warning closure; roster research only and not capital allocation.
    ScorecardWarningClosure {
        #[arg(long)]
        config: String,
    },
    /// Research-only scorecard evidence depth; paper committee evidence only with no training.
    ScorecardEvidenceDepth {
        #[arg(long)]
        config: String,
    },
    /// Research-only promotion/demotion stability; research roster management only, not capital allocation.
    PromotionDemotionStability {
        #[arg(long)]
        config: String,
    },
    /// Research-only overfit warning closure; no training and no live adaptation path.
    OverfitWarningClosure {
        #[arg(long)]
        config: String,
    },
    /// Research-only roster balance warning closure; archetypes stay non-impersonating and paper-only.
    RosterBalanceWarningClosure {
        #[arg(long)]
        config: String,
    },
    /// Research-only paper replay warning closure; no broker, order, or account command exists.
    PaperReplayWarningClosure {
        #[arg(long)]
        config: String,
    },
    /// Research-only NeedMoreEvidence closure; paper-only closure record and not execution.
    PaperNeedMoreEvidenceClosure {
        #[arg(long)]
        config: String,
    },
    /// Research-only Risk Governor handoff warning closure; final veto remains required.
    RiskHandoffWarningClosure {
        #[arg(long)]
        config: String,
    },
    /// Research-only Risk Governor final veto trace; audit-only trace with no bypass path.
    RiskFinalVetoTrace {
        #[arg(long)]
        config: String,
    },
    /// Research-only committee paper readiness gate; paper-loop only, does not imply broker execution, and never allows live-loop activation.
    CommitteePaperReadinessGate {
        #[arg(long)]
        config: String,
    },
    /// Research-only committee paper loop dry-run plan; paper-only planning with live loop forbidden.
    CommitteePaperLoopDryRunPlan {
        #[arg(long)]
        config: String,
    },
    /// Research-only workspace truth closure plan v2; focused tests never replace full workspace acceptance.
    WorkspaceTruthClosurePlanV2 {
        #[arg(long)]
        config: String,
    },
    /// Research-only workspace acceptance attempt v17; full workspace acceptance requires real cargo test --workspace --quiet completion.
    WorkspaceAcceptanceAttemptV17 {
        #[arg(long)]
        config: String,
    },
    /// Research-only safety coverage preservation v16; preserves no runtime LLM path, no training, no broker/account, and no live trading.
    SafetyCoveragePreservationV16 {
        #[arg(long)]
        config: String,
    },
    /// Read-only AI committee closure panel; static status only with no train/runtime/live/order/account/browser controls or auto-rule-apply button.
    ControlTowerAiCommitteeClosure {
        #[arg(long)]
        config: String,
    },
    /// Research-only deterministic artifact diff; fixture drift only, local-only, and never a performance claim.
    SystemBenchmarkDiff {
        #[arg(long)]
        config: String,
    },
    /// Research-only manual ship acceptance checklist; manual paper-ops gate only, no live trading.
    ManualShipChecklist {
        #[arg(long)]
        config: String,
    },
    /// Research-only system ship gate; paper-ops-monitoring only, no broker/order/account/live path.
    SystemShipGate {
        #[arg(long)]
        config: String,
    },
    /// Research-only candidate generation from local evidence bundles; candidate generation never implies approval or live execution.
    CandidateGenerate {
        #[arg(long)]
        config: String,
    },
    /// Paper-only deterministic committee cycle replay; no broker, order, account, or live execution path exists.
    CommitteeCycle {
        #[arg(long)]
        config: String,
    },
    /// No-live deterministic Trinity operational loop for paper-only monitoring; owner and risk remain audited local controls.
    TrinityOperationalLoop {
        #[arg(long)]
        config: String,
    },
    /// Simulated-only paper lifecycle monitor; reports paper positions without broker accounts, holdings, or order ids.
    PaperLifecycleReport {
        #[arg(long)]
        config: String,
    },
    /// Audit-only deterministic operational timeline; emits local review events and never touches live systems.
    OperationalAuditTimeline {
        #[arg(long)]
        config: String,
    },
    /// Research-only local owner action draft bundle; paper-only local draft files only, no execution path, and never auto-applies.
    DashboardActionDrafts {
        #[arg(long)]
        config: String,
    },
    /// Local-file dashboard open helper; prints or resolves only generated local html paths.
    DashboardOpen {
        #[arg(long)]
        config: String,
    },
    /// Localhost-only dashboard serve remains deferred unless GET-only static serving is trivially safe.
    DashboardServe {
        #[arg(long)]
        config: String,
    },
    /// Research-only local owner input validator; structured audited input only, paper-only, and never live.
    OwnerInputValidate {
        #[arg(long)]
        config: String,
    },
    /// Research-only local owner review queue builder; audited local review only, paper-only, and no broker/account path.
    OwnerReviewQueue {
        #[arg(long)]
        config: String,
    },
    /// Research-only local owner apply simulation; paper-only and never live trading or broker execution.
    OwnerApplyInput {
        #[arg(long)]
        config: String,
    },
    /// Research-only local owner impact report; paper-only and no broker/order/account/live path.
    OwnerImpactReport {
        #[arg(long)]
        config: String,
    },
    /// Research-only local owner thesis book view; notes are diagnostics, not signals, and remain paper-only context only.
    OwnerThesisBook {
        #[arg(long)]
        config: String,
    },
    /// Research-only strategy/data compatibility check.
    StrategyDataCheck {
        #[arg(long)]
        provider: String,
        #[arg(long = "use-case")]
        use_case: String,
    },
    /// Research-only provider recommendation for bounded planning.
    ProviderRecommend {
        #[arg(long)]
        market: String,
        #[arg(long = "use-case")]
        use_case: String,
        #[arg(long)]
        budget: String,
    },
    /// Research-only executable evidence plan from provider reality or explicit lanes.
    EvidencePlan {
        #[arg(long)]
        config: String,
    },
    /// Research-only bounded evidence execution; no broker/order/account/live commands.
    EvidenceExecute {
        #[arg(long)]
        config: String,
    },
    /// Research-only readiness matrix across market/use-case/source lanes.
    ReadinessMatrix {
        #[arg(long)]
        config: String,
    },
    /// Research-only committee smoke using exactly three personas; no broker/order/account/live paths.
    CommitteeSmoke {
        #[arg(long)]
        config: String,
    },
    /// Research-only committee scenario loading from local report summaries only.
    CommitteeLoadScenarios {
        #[arg(long)]
        config: String,
    },
    /// Research-only deterministic committee debate replay from local scenarios only.
    CommitteeReplay {
        #[arg(long)]
        config: String,
    },
    /// Research-only committee diagnostics and six-person design-review gate.
    CommitteeDiagnostics {
        #[arg(long)]
        config: String,
    },
    /// Research-only persona card dump for the active committee MVP.
    PersonaCards,
    /// Research-only Committee V1 operational bundle; no live trading, broker, or account paths.
    CommitteeV1 {
        #[arg(long)]
        config: String,
    },
    /// Research-only committee scenario materialization from local artifacts only.
    CommitteeMaterialize {
        #[arg(long)]
        config: String,
    },
    /// Research-only committee benchmark with core-check and local-only artifacts.
    CommitteeBenchmark {
        #[arg(long)]
        config: String,
    },
    /// Research-only official committee scenario packing from local artifacts only.
    CommitteePackOfficial {
        #[arg(long)]
        config: String,
    },
    /// Research-only local outcome/baseline/external linking for committee scenarios only.
    CommitteeLinkOutcomes {
        #[arg(long)]
        config: String,
    },
    /// Research-only official committee benchmark from local packs only.
    CommitteeOfficialBenchmark {
        #[arg(long)]
        config: String,
    },
    /// Research-only committee outcome coverage and sufficiency bundle; no live trading.
    CommitteeOutcomeCoverage {
        #[arg(long)]
        config: String,
    },
    /// Research-only local counterfactual audit from local candles and scenario packs only.
    CommitteeCounterfactualAudit {
        #[arg(long)]
        config: String,
    },
    /// Research-only committee performance evidence matrix from local coverage inputs only.
    CommitteePerformanceMatrix {
        #[arg(long)]
        config: String,
    },
    /// Research-only committee reference-pack generation from local scenarios and candles only.
    CommitteeBuildReferences {
        #[arg(long)]
        config: String,
    },
    /// Research-only candle alignment for committee reference packs from local candles only.
    CommitteeAlignCandles {
        #[arg(long)]
        config: String,
    },
    /// Research-only sufficiency closure for committee reference packs; no live trading.
    CommitteeSufficiencyClose {
        #[arg(long)]
        config: String,
    },
    /// Research-only comparable committee evidence from local artifacts only.
    ComparableEvidence {
        #[arg(long)]
        config: String,
    },
    /// Research-only official candle coverage pack from local canonical candles only.
    CandlePack {
        #[arg(long)]
        config: String,
    },
    /// Research-only candle coverage matcher from local packs and comparable bundles only.
    CandleCoverageMatch {
        #[arg(long)]
        config: String,
    },
    /// Research-only comparable evidence candle backfill from local packs only.
    ComparableBackfill {
        #[arg(long)]
        config: String,
    },
    /// Research-only candle coverage closure from local-only packs, bundles, and reruns.
    CandleCoverageClose {
        #[arg(long)]
        config: String,
    },
    /// Research-only official candle gap map from local comparable evidence and candle packs only.
    CandleGapMap {
        #[arg(long)]
        config: String,
    },
    /// Research-only official candle expansion plan from local gap maps, readiness, and reality only.
    CandleExpansionPlan {
        #[arg(long)]
        config: String,
    },
    /// Research-only official candle expansion operator actions from a local plan only.
    CandleExpansionActions {
        #[arg(long)]
        config: String,
    },
    /// Research-only bounded official candle expansion loop from local-only artifacts.
    CandleExpand {
        #[arg(long)]
        config: String,
    },
    /// Research-only official candle join audit from local comparable rows and candle packs only.
    CandleJoinAudit {
        #[arg(long)]
        config: String,
    },
    /// Research-only repair planner for local official candle join issues only.
    CandleJoinRepairPlan {
        #[arg(long)]
        config: String,
    },
    /// Research-only official-ready match closure from local join audit and repair actions only.
    OfficialReadyMatchClose {
        #[arg(long)]
        config: String,
    },
    /// Research-only official-ready row inventory from local artifacts only; no broker/order/account/live paths.
    OfficialReadyRowInventory {
        #[arg(long)]
        config: String,
    },
    /// Research-only scenario materialization v3 from local official-ready rows only.
    ScenarioMaterializeV3 {
        #[arg(long)]
        config: String,
    },
    /// Research-only complete comparable row closure from local-only artifacts and configs.
    CompleteRowClose {
        #[arg(long)]
        config: String,
    },
    /// Research-only future-window requirement audit from local-only artifacts and candle CSVs.
    FutureWindowRequirements {
        #[arg(long)]
        config: String,
    },
    /// Research-only local-first future-window extension planning from local-only artifacts.
    FutureWindowExtensionPlan {
        #[arg(long)]
        config: String,
    },
    /// Research-only outcome linkage v3 from local-only official-ready rows and candle CSVs.
    OutcomeLinkageV3 {
        #[arg(long)]
        config: String,
    },
    /// Research-only counterfactual completion v2 from local-only outcome linkage artifacts.
    CounterfactualCompleteV2 {
        #[arg(long)]
        config: String,
    },
    /// Research-only complete row closure v2 from local-only configs, bundles, and candle CSVs.
    CompleteRowCloseV2 {
        #[arg(long)]
        config: String,
    },
    /// Research-only lineage rendering for official candle joins from local artifacts only.
    CandleLineage {
        #[arg(long)]
        config: String,
    },
    /// Research-only counterfactual depth plan from local comparable evidence only.
    CounterfactualDepthPlan {
        #[arg(long)]
        config: String,
    },
    /// Research-only counterfactual depth closure from local-only configs and artifacts.
    CounterfactualDepthClose {
        #[arg(long)]
        config: String,
    },
    /// Research-only scorecard rerun summary from local-only closure config.
    ScorecardRerun {
        #[arg(long)]
        config: String,
    },
    /// Research-only official replication runner using local artifacts only; no live trading, broker, or account paths.
    OfficialReplication {
        #[arg(long)]
        config: String,
    },
    /// Research-only official artifact inventory from local-only paths; no broker/order/account/live commands.
    OfficialArtifactInventory {
        #[arg(long)]
        config: String,
    },
    /// Research-only official row injection from local-only artifacts; no live trading or broker paths.
    OfficialRowInject {
        #[arg(long)]
        config: String,
    },
    /// Research-only official evidence expansion behind bounded collection and core-check.
    EvidenceExpand {
        #[arg(long)]
        config: String,
    },
    /// Research-only auth-aware official evidence acquisition with bounded local collection.
    OfficialAcquire {
        #[arg(long)]
        config: String,
    },
    /// Research-only venue coverage report for bounded official evidence.
    OfficialCoverage {
        #[arg(long)]
        config: String,
    },
    /// Research-only source-aware benchmark across official and yfinance evidence.
    SourceBenchmark {
        #[arg(long)]
        config: String,
    },
    /// Research-only local import of canonical yfinance CSV plus provenance/manifest.
    YfinanceImport {
        #[arg(long)]
        config: String,
    },
    /// Research-only aggregate report across yfinance import bridges.
    YahooResearch {
        #[arg(long)]
        config: String,
    },
    /// Conservative comparison between official-ready evidence and yfinance research-only evidence.
    OfficialVsYfinance {
        #[arg(long)]
        official_report: Option<String>,
        #[arg(long)]
        yfinance_report: String,
        #[arg(long)]
        out: Option<String>,
        #[arg(long)]
        official_metric: Option<f64>,
        #[arg(long)]
        yfinance_metric: Option<f64>,
    },
    Compare {
        #[arg(long)]
        current: String,
        #[arg(long)]
        previous: String,
    },
    Baseline {
        #[arg(long)]
        data: String,
        #[arg(long)]
        symbol: String,
        #[arg(long)]
        timeframe: String,
        #[arg(long)]
        out: String,
    },
    /// Research-only bounded multi-row official evidence set builder; local files only.
    MultiRowOfficialSet {
        #[arg(long)]
        config: String,
    },
    /// Research-only future-window scaleout planner; local files only.
    FutureWindowScaleoutPlan {
        #[arg(long)]
        config: String,
    },
    /// Research-only batch outcome linkage v3; local files only.
    BatchOutcomeLinkageV3 {
        #[arg(long)]
        config: String,
    },
    /// Research-only batch counterfactual completion; local files only.
    BatchCounterfactualComplete {
        #[arg(long)]
        config: String,
    },
    /// Research-only official evidence sufficiency v2; local files only.
    OfficialEvidenceSufficiencyV2 {
        #[arg(long)]
        config: String,
    },
    /// Research-only official evidence scaleout rerun; paper-only, local-only, never live.
    OfficialEvidenceScaleout {
        #[arg(long)]
        config: String,
    },
    /// Research-only barrier profile registry; preregistration only, local-only, never live.
    BarrierProfiles {
        #[arg(long)]
        config: String,
    },
    /// Research-only official evidence diversity gap mapping; local-only.
    OfficialDiversityGapMap {
        #[arg(long)]
        config: String,
    },
    /// Research-only official diversity row selector; local-only and no outcome peeking.
    OfficialDiversityRowSelect {
        #[arg(long)]
        config: String,
    },
    /// Research-only outcome diversity audit; local-only.
    OutcomeDiversityAudit {
        #[arg(long)]
        config: String,
    },
    /// Research-only balanced outcome coverage report; local-only.
    BalancedOutcomeCoverage {
        #[arg(long)]
        config: String,
    },
    /// Research-only diversity-aware sufficiency v2; local-only and never implies profitability.
    DiversitySufficiencyV2 {
        #[arg(long)]
        config: String,
    },
    /// Research-only official evidence diversity sweep; local-only and never live.
    OfficialEvidenceDiversitySweep {
        #[arg(long)]
        config: String,
    },
    Dataset {
        #[arg(long)]
        data: String,
        #[arg(long)]
        symbol: String,
        #[arg(long)]
        timeframe: String,
        #[arg(long)]
        out: String,
    },
}

fn main() {
    let worker = std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(|| {
            let cli = Cli::parse();
            match cli.command {
        Commands::Run { config } => ExperimentConfig::from_toml_path(std::path::Path::new(&config))
            .map_err(|err| err.to_string())
            .map(|config| {
                let bundle = ExperimentRunner::default().run(&config);
                println!("{}", bundle.to_deterministic_summary());
            }),
        Commands::Batch { config } => {
            ExperimentMatrixConfig::from_toml_path(std::path::Path::new(&config))
                .map_err(|err| err.to_string())
                .map(|config| {
                    let report = BatchExperimentRunner::default().run_matrix(&config);
                    println!("{}", report.aggregate_benchmark.to_markdown_table_string());
                    println!(
                        "expansion_decision={:?}",
                        report.expansion_readiness.decision
                    );
                })
        }
        Commands::Ablation { config } => {
            AblationStudyConfig::from_toml_path(std::path::Path::new(&config))
                .map_err(|err| err.to_string())
                .map(|config| {
                    let report = AblationRunner::default().run_study(&config);
                    println!("{}", ablation_report_to_text(&report));
                })
        }
        Commands::Sprint14 { from_ablation, out } => Ok(()).and_then(|_| {
            if from_ablation.contains("://") || out.contains("://") {
                return Err("sprint14 paths must be local".to_string());
            }
            let report = Sprint14Runner::default()
                .run_from_ablation_report_path(std::path::Path::new(&from_ablation))?;
            report.write_to_dir(std::path::Path::new(&out))?;
            println!("{}", sprint14_report_to_text(&report));
            Ok(())
        }),
        Commands::EvidenceClose { config } => {
            if config.contains("://") {
                Err("evidence-close config path must be local".to_string())
            } else {
                EvidenceClosureConfig::from_toml_path(std::path::Path::new(&config))
                    .map_err(|err| err.to_string())
                    .map(|config| {
                        let report = EvidenceClosureRunner::default().run_closure(&config);
                        println!("{}", evidence_closure_report_to_text(&report));
                    })
            }
        }
        Commands::RealEvidence { config } => {
            if config.contains("://") {
                Err("real-evidence config path must be local".to_string())
            } else {
                RealEvidenceClosureConfig::from_toml_path(std::path::Path::new(&config))
                    .map_err(|err| err.to_string())
                    .map(|config| {
                        let report = RealEvidenceClosureRunner::default().run(&config);
                        println!("{}", real_evidence_report_to_text(&report));
                    })
            }
        }
        Commands::DataPreflight {
            input,
            out,
            symbol,
            timeframe,
        } => Ok(()).and_then(|_| {
            if input.contains("://") || out.contains("://") {
                return Err("data-preflight paths must be local".to_string());
            }
            let config = LocalDataOnboardingConfig {
                onboarding_id: "cli-data-preflight".to_string(),
                input_path: input,
                output_root: out.clone(),
                symbol: Some(symbol),
                timeframe: Some(parse_timeframe(&timeframe)),
                ..LocalDataOnboardingConfig::default()
            };
            let report = PreflightValidator::default().run(&config);
            report.write_to_dir(std::path::Path::new(&out))?;
            println!("{}", report.to_text());
            Ok(())
        }),
        Commands::OnboardData {
            config,
            input,
            out,
            symbol,
            timeframe,
        } => Ok(()).and_then(|_| {
            let onboarding = if let Some(config_path) = config {
                if config_path.contains("://") {
                    return Err("onboard-data config path must be local".to_string());
                }
                LocalDataOnboardingConfig::from_toml_path(std::path::Path::new(&config_path))?
            } else {
                let input =
                    input.ok_or_else(|| "onboard-data requires --config or --input".to_string())?;
                let out = out
                    .ok_or_else(|| "onboard-data requires --out when using --input".to_string())?;
                if input.contains("://") || out.contains("://") {
                    return Err("onboard-data paths must be local".to_string());
                }
                LocalDataOnboardingConfig {
                    onboarding_id: "cli-onboard-data".to_string(),
                    input_path: input,
                    output_root: out,
                    symbol,
                    timeframe: timeframe.as_deref().map(parse_timeframe),
                    ..LocalDataOnboardingConfig::default()
                }
            };
            let report = PreflightValidator::default().run(&onboarding);
            let plan = build_real_evidence_rerun_plan(
                &onboarding,
                report,
                soma_zero::ConfigGenerationPolicy::ReadyOnly,
            );
            plan.write_to_dir(std::path::Path::new(&onboarding.output_root))?;
            println!("{}", plan.to_text());
            Ok(())
        }),
        Commands::ImportKrxSnapshot {
            input,
            out,
            date,
            symbol,
        } => Ok(()).and_then(|_| {
            if input.contains("://") || out.contains("://") {
                return Err("import-krx-snapshot paths must be local".to_string());
            }
            let report = KrxSnapshotImporter::default().import(&KrxSnapshotImportConfig {
                import_id: "cli-krx-snapshot-import".to_string(),
                input_path: input,
                output_root: out,
                snapshot_date: date,
                symbol_filter: symbol,
                reason_codes: vec![soma_zero::ReasonCode::DeterministicPath],
            })?;
            println!("{}", report.to_text());
            Ok(())
        }),
        Commands::CollectCandles {
            provider,
            symbol,
            venue,
            timeframe,
            start,
            end,
            out,
            fixture,
            fill_missing,
            max_rows,
            max_requests,
            max_days,
            raw_archive,
            outputsize,
            allow_full_history,
            api_key_env_var,
            api_secret_env_var,
            auth_header_name,
            query_param_name,
            endpoint_template,
            adjusted_price,
        } => Ok(()).and_then(|_| {
            if out.contains("://")
                || fixture
                    .as_deref()
                    .is_some_and(|value| value.contains("://"))
            {
                return Err("collect-candles paths must be local".to_string());
            }
            let provider_kind = parse_provider(&provider)?;
            let mut size_policy = CollectionSizePolicy::default();
            if let Some(max_rows) = max_rows {
                size_policy.max_rows_per_symbol = max_rows;
            }
            if let Some(max_requests) = max_requests {
                size_policy.max_requests_per_run = max_requests;
            }
            if let Some(max_days) = max_days {
                size_policy.max_days_per_run = max_days;
            }
            size_policy.raw_archive_policy = parse_raw_archive_policy(&raw_archive);
            let auth_config = if api_key_env_var.is_some()
                || api_secret_env_var.is_some()
                || auth_header_name.is_some()
                || query_param_name.is_some()
            {
                Some(AuthConfig {
                    provider_kind,
                    api_key_env_var,
                    api_secret_env_var,
                    auth_header_name,
                    query_param_name,
                    allow_missing_for_mock: fixture.is_some(),
                    reason_codes: vec![soma_zero::ReasonCode::DeterministicPath],
                })
            } else {
                None
            };
            let result =
                soma_zero::CollectorRunner::default().run(&soma_zero::CandleFetchRequest {
                    request_id: "cli-collect-candles".to_string(),
                    provider_kind,
                    symbol,
                    market_venue: venue.as_deref().map(parse_market_venue).transpose()?,
                    asset_class: infer_asset_class(provider_kind),
                    timeframe: parse_timeframe(&timeframe),
                    start_timestamp_ms: start.as_deref().map(parse_timestamp_like).transpose()?,
                    end_timestamp_ms: end.as_deref().map(parse_timestamp_like).transpose()?,
                    output_root: out,
                    limit_per_request: None,
                    include_raw_archive: parse_raw_archive_policy(&raw_archive)
                        != RawArchivePolicy::None,
                    fill_missing_policy: parse_fill_missing_policy(&fill_missing),
                    fixture_path: fixture,
                    adjusted_price_policy: parse_adjusted_price_policy(&adjusted_price),
                    collection_size_policy: size_policy,
                    auth_config,
                    endpoint_template,
                    requested_output_size: outputsize
                        .as_deref()
                        .map(parse_requested_output_size)
                        .transpose()?,
                    allow_full_history_override: allow_full_history,
                    reason_codes: vec![soma_zero::ReasonCode::DeterministicPath],
                })?;
            println!("{}", result.to_text());
            Ok(())
        }),
        Commands::Campaign { config } => {
            ResearchCampaignConfig::from_toml_path(std::path::Path::new(&config))
                .map_err(|err| err.to_string())
                .map(|config| {
                    let report = ResearchCampaignRunner::default().run_campaign(&config);
                    println!("{}", report.aggregate.to_markdown_table_string());
                    println!("campaign_decision={:?}", report.readiness_report.decision);
                })
        }
        Commands::CollectPlan { config } => Ok(()).and_then(|_| {
            if config.contains("://") {
                return Err("collect-plan config path must be local".to_string());
            }
            let plan = OfficialCollectionPlan::from_toml_path(std::path::Path::new(&config))?;
            let report = OfficialCollectionRunner::default().run_plan(&plan);
            println!("{}", report.to_text());
            Ok(())
        }),
        Commands::EvidenceRun {
            from_collection,
            out,
        } => Ok(()).and_then(|_| {
            if from_collection.contains("://") || out.contains("://") {
                return Err("evidence-run paths must be local".to_string());
            }
            let report = OfficialEvidenceRunner::default().run(&OfficialEvidenceRunConfig {
                collection_report_path: Some(from_collection),
                generated_rerun_configs: Vec::new(),
                output_root: out,
                run_real_evidence: true,
                run_batch: true,
                run_ablation: true,
                require_ready_entries: false,
                min_ready_entries: 1,
                min_outcome_records: 20,
                min_comparable_variants: 2,
                reason_codes: vec![soma_zero::ReasonCode::DeterministicPath],
            });
            println!("{}", report.to_text());
            Ok(())
        }),
        Commands::CollectAndEvaluate { config } => Ok(()).and_then(|_| {
            if config.contains("://") {
                return Err("collect-and-evaluate config path must be local".to_string());
            }
            let plan = OfficialCollectionPlan::from_toml_path(std::path::Path::new(&config))?;
            let collection_report = OfficialCollectionRunner::default().run_plan(&plan);
            let collection_report_path = std::path::Path::new(&plan.output_root)
                .join(&plan.plan_id)
                .join("official_collection_report.json");
            let evidence_output = std::path::Path::new(&plan.output_root)
                .join(&plan.plan_id)
                .join("official_evidence_run");
            let evidence_report =
                OfficialEvidenceRunner::default().run(&OfficialEvidenceRunConfig {
                    collection_report_path: Some(collection_report_path.display().to_string()),
                    generated_rerun_configs: Vec::new(),
                    output_root: evidence_output.display().to_string(),
                    run_real_evidence: true,
                    run_batch: true,
                    run_ablation: true,
                    require_ready_entries: false,
                    min_ready_entries: 1,
                    min_outcome_records: 20,
                    min_comparable_variants: 2,
                    reason_codes: vec![soma_zero::ReasonCode::DeterministicPath],
                });
            println!("{}", collection_report.to_text());
            println!("{}", evidence_report.to_text());
            Ok(())
        }),
        Commands::AiBenchmark { config } | Commands::CollectTrainEvaluate { config } => {
            if config.contains("://") {
                Err("ai-benchmark config path must be local".to_string())
            } else {
                OfficialAiBenchmarkConfig::from_toml_path(std::path::Path::new(&config))
                    .map_err(|err| err.to_string())
                    .map(|config| {
                        let report = OfficialAiBenchmarkRunner::default().run(&config);
                        println!("{}", official_ai_benchmark_report_to_text(&report));
                    })
            }
        }
        Commands::MambaReadiness { config } => {
            if config.contains("://") {
                Err("mamba-readiness config path must be local".to_string())
            } else {
                MambaReadinessConfig::from_toml_path(std::path::Path::new(&config))
                    .map_err(|err| err.to_string())
                    .and_then(|config| {
                        let report = MambaReadinessRunner::default()
                            .run(&config)
                            .map_err(|err| err.to_string())?;
                        println!("{}", report.to_text());
                        Ok(())
                    })
            }
        }
        Commands::CoreCompletionAudit { config } => {
            if config.contains("://") {
                Err("core-completion-audit config path must be local".to_string())
            } else {
                soma_zero::CoreCompletionAuditConfig::from_toml_path(std::path::Path::new(&config))
                    .map_err(|err| err.to_string())
                    .and_then(|config| {
                        let (report, _) = soma_zero::CoreCompletionAuditRunner::default().run(&config)?;
                        println!("{}", report.to_text());
                        Ok(())
                    })
            }
        }
        Commands::SequenceReadiness { config } => {
            if config.contains("://") {
                Err("sequence-readiness config path must be local".to_string())
            } else {
                soma_zero::SequenceDatasetReadinessConfig::from_toml_path(std::path::Path::new(&config))
                    .map_err(|err| err.to_string())
                    .and_then(|config| {
                        let report = soma_zero::SequenceDatasetReadinessRunner::default().run(&config)?;
                        println!("{}", report.to_text());
                        Ok(())
                    })
            }
        }
        Commands::MambaReadinessV2 { config } => {
            if config.contains("://") {
                Err("mamba-readiness-v2 config path must be local".to_string())
            } else {
                soma_zero::MambaReadinessV2Config::from_toml_path(std::path::Path::new(&config))
                    .map_err(|err| err.to_string())
                    .and_then(|config| {
                        let report = soma_zero::MambaReadinessV2Runner::default().run(&config)?;
                        println!("{}", report.to_text());
                        Ok(())
                    })
            }
        }
        Commands::ModelEscalationDecision { config } => {
            if config.contains("://") {
                Err("model-escalation-decision config path must be local".to_string())
            } else {
                soma_zero::ModelEscalationDecisionV2Config::from_toml_path(std::path::Path::new(&config))
                    .map_err(|err| err.to_string())
                    .and_then(|config| {
                        let report = soma_zero::ModelEscalationDecisionRunner::default().run(&config)?;
                        println!("{}", report.to_text());
                        Ok(())
                    })
            }
        }
        Commands::MambaPrototypePlan { config } => {
            if config.contains("://") {
                Err("mamba-prototype-plan config path must be local".to_string())
            } else {
                soma_zero::Mamba3FinLitePrototypePlanConfig::from_toml_path(std::path::Path::new(&config))
                    .map_err(|err| err.to_string())
                    .and_then(|config| {
                        let report = soma_zero::Mamba3FinLitePrototypePlanRunner::default().run(&config)?;
                        println!("{}", report.to_text());
                        Ok(())
                    })
            }
        }
        Commands::CoreCheck { config, out } => {
            if config.as_ref().is_some_and(|value| value.contains("://"))
                || out.as_ref().is_some_and(|value| value.contains("://"))
            {
                Err("core-check paths must be local".to_string())
            } else {
                let config_result = match (config, out) {
                    (Some(config), None) => {
                        CoreCheckConfig::from_toml_path(std::path::Path::new(&config))
                            .map_err(|err| err.to_string())
                    }
                    (None, Some(out)) => Ok(CoreCheckConfig {
                        output_root: out,
                        ..CoreCheckConfig::default()
                    }),
                    (None, None) => Ok(CoreCheckConfig::default()),
                    (Some(config_path), Some(out)) => {
                        CoreCheckConfig::from_toml_path(std::path::Path::new(&config_path))
                            .map_err(|err| err.to_string())
                            .map(|config| CoreCheckConfig {
                                output_root: out,
                                ..config
                            })
                    }
                };
                config_result.and_then(|config| {
                    let report = CoreCheckRunner::default()
                        .run(&config)
                        .map_err(|err| err.to_string())?;
                    println!("{}", report.to_text());
                    Ok(())
                })
            }
        }
        Commands::CoreBenchmark { config } => {
            if config.contains("://") {
                Err("core-benchmark config path must be local".to_string())
            } else {
                CoreCheckedBenchmarkConfig::from_toml_path(std::path::Path::new(&config))
                    .map_err(|err| err.to_string())
                    .and_then(|config| {
                        let report = CoreCheckedBenchmarkRunner::default()
                            .run(&config)
                            .map_err(|err| err.to_string())?;
                        println!("{}", core_checked_benchmark_report_to_text(&report));
                        Ok(())
                    })
            }
        }
        Commands::CorePerformance { config } => {
            if config.contains("://") {
                Err("core-performance config path must be local".to_string())
            } else {
                CorePerformanceScorecardConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let bundle = CorePerformanceScorecardRunner::default().run(&config)?;
                        println!(
                            "research_only_warning=core performance scorecards remain research-only and never imply live trading"
                        );
                        println!("{}", bundle.scorecard.to_text());
                        Ok(())
                    })
            }
        }
        Commands::CoreBottleneck { config } => {
            if config.contains("://") {
                Err("core-bottleneck config path must be local".to_string())
            } else {
                CorePerformanceScorecardConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let bundle = CorePerformanceScorecardRunner::default().run(&config)?;
                        println!(
                            "research_only_warning=core bottleneck output remains research-only and paper-only"
                        );
                        println!("{}", bundle.scorecard.bottleneck_report.to_text());
                        Ok(())
                    })
            }
        }
        Commands::CoreRegression { config } => {
            if config.contains("://") {
                Err("core-regression config path must be local".to_string())
            } else {
                CorePerformanceRegressionConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = soma_zero::CorePerformanceRegressionReport::from_config(&config)?;
                        println!(
                            "research_only_warning=core regression remains a research-only stability guard"
                        );
                        println!("{}", report.to_text());
                        Ok(())
                    })
            }
        }
        Commands::ProviderAuthCheck { config } => {
            if config.contains("://") {
                Err("provider-auth-check config path must be local".to_string())
            } else {
                ProviderAuthPreflightConfig::from_toml_path(std::path::Path::new(&config))
                    .map_err(|err| err.to_string())
                    .map(|config| {
                        let report = ProviderAuthPreflightRunner::default().run(&config);
                        println!("{}", provider_auth_preflight_report_to_text(&report));
                    })
            }
        }
        Commands::KrxAuthReadiness { config } => {
            if config.contains("://") {
                Err("krx-auth-readiness config path must be local".to_string())
            } else {
                KRXOfficialEvidenceActivationConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = soma_zero::KRXAuthReadinessReport::from_config(&config);
                        println!(
                            "research_only_warning=krx auth readiness is market-data-only and secret-safe"
                        );
                        println!("{}", krx_auth_readiness_report_to_text(&report));
                        Ok(())
                    })
            }
        }
        Commands::KrxSymbolWhitelist { config } => {
            if config.contains("://") {
                Err("krx-symbol-whitelist config path must be local".to_string())
            } else {
                KRXSymbolWhitelistConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        config.validate()?;
                        let report = config.build();
                        println!(
                            "research_only_warning=krx symbol whitelist stays bounded and market-data-only"
                        );
                        println!("{}", krx_symbol_whitelist_to_text(&report));
                        Ok(())
                    })
            }
        }
        Commands::KrxEvidencePlan { config } => {
            if config.contains("://") {
                Err("krx-evidence-plan config path must be local".to_string())
            } else {
                KRXOfficialEvidenceActivationConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        config.validate()?;
                        let auth = soma_zero::KRXAuthReadinessReport::from_config(&config);
                        let whitelist = if let Some(path) = config.symbol_whitelist_path.as_deref() {
                            KRXSymbolWhitelistConfig::from_toml_path(std::path::Path::new(path))?
                                .build()
                        } else {
                            KRXSymbolWhitelistConfig {
                                whitelist_id: format!("{}-derived", config.activation_id),
                                output_root: config.output_root.clone(),
                                max_symbols: config.max_symbols,
                                ..KRXSymbolWhitelistConfig::default()
                            }
                            .build()
                        };
                        let plan = soma_zero::KRXEvidenceJobPlan::build(&config, &auth, &whitelist);
                        println!(
                            "research_only_warning=krx evidence plans stay local-first and never imply live collection"
                        );
                        println!("{}", krx_evidence_job_plan_to_text(&plan));
                        Ok(())
                    })
            }
        }
        Commands::KrxOfficialActivate { config } => {
            if config.contains("://") {
                Err("krx-official-activate config path must be local".to_string())
            } else {
                KRXOfficialEvidenceActivationConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let bundle = KRXOfficialEvidenceActivationRunner::default().run(&config)?;
                        println!(
                            "research_only_warning=krx official activation remains market-data-only and never implies live trading"
                        );
                        println!(
                            "secret_safety_warning=krx activation renders no secret values and only uses env-var auth locally"
                        );
                        println!(
                            "{}",
                            krx_official_activation_report_to_text(&bundle.activation_report)
                        );
                        Ok(())
                    })
            }
        }
        Commands::KrxCollectionDryRun { config } => {
            if config.contains("://") {
                Err("krx-collection-dry-run config path must be local".to_string())
            } else {
                KRXBoundedCollectionSmokeConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        config.validate()?;
                        let whitelist = config.load_whitelist()?;
                        let report = config.build_dry_run_report(&whitelist);
                        println!(
                            "research_only_warning=krx collection dry run is market-data-only, secret-safe, and never live trading"
                        );
                        println!(
                            "market_data_only_warning=no broker, order, or account API is exercised by this command"
                        );
                        println!(
                            "secret_safety_warning=KRX_API_KEY is never printed and endpoint previews remain redacted"
                        );
                        println!("{}", krx_collection_dry_run_to_text(&report));
                        Ok(())
                    })
            }
        }
        Commands::KrxCollectionPlan { config } => {
            if config.contains("://") {
                Err("krx-collection-plan config path must be local".to_string())
            } else {
                KRXBoundedCollectionSmokeConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        config.validate()?;
                        let whitelist = config.load_whitelist()?;
                        let dry_run = config.build_dry_run_report(&whitelist);
                        let plan =
                            soma_zero::KRXCollectionBatchPlan::build(&config, &dry_run, &whitelist);
                        println!(
                            "research_only_warning=krx collection planning stays bounded, local-first, and market-data-only"
                        );
                        println!(
                            "market_data_only_warning=no broker, order, or account API exists in this planning command"
                        );
                        println!(
                            "secret_safety_warning=KRX_API_KEY is never rendered and endpoint previews stay redacted"
                        );
                        println!("{}", krx_collection_batch_plan_to_text(&plan));
                        Ok(())
                    })
            }
        }
        Commands::KrxBoundedCollect { config } => {
            if config.contains("://") {
                Err("krx-bounded-collect config path must be local".to_string())
            } else {
                KRXOfficialCollectionClosureConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|mut config| {
                        config.run_candle_sufficiency = false;
                        config.run_outcome_link_closure = false;
                        config.run_downstream_rerun_v2 = false;
                        let bundle = KRXOfficialCollectionClosureRunner::default().run(&config)?;
                        println!(
                            "research_only_warning=krx bounded collect remains market-data-only and never implies live trading"
                        );
                        println!(
                            "market_data_only_warning=no broker, order, or account API is exposed by this command"
                        );
                        println!(
                            "secret_safety_warning=KRX_API_KEY is never printed and endpoint previews remain redacted"
                        );
                        println!(
                            "{}",
                            krx_collection_closure_report_to_text(&bundle.collection_closure_report)
                        );
                        Ok(())
                    })
            }
        }
        Commands::KrxCandleSufficiency { config } => {
            if config.contains("://") {
                Err("krx-candle-sufficiency config path must be local".to_string())
            } else {
                KRXOutcomeLinkClosureConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        config.validate()?;
                        let report = soma_zero::KRXCandleSufficiencyReport::build(
                            &format!("{}-cli-candle-sufficiency", config.closure_id),
                            &config.krx_canonical_csv_paths,
                            config.barrier_profile_registry_path.as_deref(),
                        );
                        println!(
                            "research_only_warning=krx candle sufficiency is market-data-only and no-lookahead constrained"
                        );
                        println!(
                            "market_data_only_warning=no broker, order, or account API exists in this command"
                        );
                        println!(
                            "secret_safety_warning=closure configs are local-only and never render secrets"
                        );
                        println!("{}", krx_candle_sufficiency_to_text(&report));
                        Ok(())
                    })
            }
        }
        Commands::KrxOutcomeLinkClose { config } => {
            if config.contains("://") {
                Err("krx-outcome-link-close config path must be local".to_string())
            } else {
                KRXOutcomeLinkClosureConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = KRXOutcomeLinkClosureRunner::default().run(&config)?;
                        println!(
                            "research_only_warning=krx outcome-link closure stays paper-only and market-data-only"
                        );
                        println!(
                            "market_data_only_warning=no broker, order, or account API is exposed by this command"
                        );
                        println!(
                            "secret_safety_warning=env secrets are never printed and local-only paths are enforced"
                        );
                        println!("{}", krx_outcome_link_closure_to_text(&report));
                        Ok(())
                    })
            }
        }
        Commands::KrxCollectionClose { config } => {
            if config.contains("://") {
                Err("krx-collection-close config path must be local".to_string())
            } else {
                KRXOfficialCollectionClosureConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let bundle = KRXOfficialCollectionClosureRunner::default().run(&config)?;
                        println!(
                            "research_only_warning=krx collection closure remains market-data-only, paper-only, and deterministic"
                        );
                        println!(
                            "market_data_only_warning=no broker, order, account, runtime-LLM, or Mamba command exists here"
                        );
                        println!(
                            "secret_safety_warning=KRX_API_KEY is never rendered and endpoint previews stay redacted"
                        );
                        println!(
                            "{}",
                            krx_collection_closure_report_to_text(&bundle.collection_closure_report)
                        );
                        Ok(())
                    })
            }
        }
        Commands::KisAuthReadiness { config } => {
            if config.contains("://") {
                Err("kis-auth-readiness config path must be local".to_string())
            } else {
                KISMarketDataActivationConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = soma_zero::KISAuthReadinessReport::from_config(&config);
                        println!(
                            "research_only_warning=kis auth readiness is research-only, market-data-only, and secret-safe"
                        );
                        println!(
                            "secret_safety_warning=env-var names and redacted base-url previews only; no secret values are rendered"
                        );
                        println!("{}", kis_auth_readiness_report_to_text(&report));
                        Ok(())
                    })
            }
        }
        Commands::KisAuthClose { config } => {
            if config.contains("://") {
                Err("kis-auth-close config path must be local".to_string())
            } else {
                KISAuthClosureConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let report = KISAuthClosureRunner::default().run(&config)?;
                        println!("{}", report.to_text());
                        Ok(())
                    },
                )
            }
        }
        Commands::KisEndpointPolicy { config } => {
            if config.contains("://") {
                Err("kis-endpoint-policy config path must be local".to_string())
            } else {
                KISMarketDataActivationConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let policy = if let Some(path) = config.endpoint_policy_path.as_deref() {
                            soma_zero::KISEndpointPolicy::from_toml_path(std::path::Path::new(path))?
                        } else {
                            soma_zero::KISEndpointPolicy::default()
                        };
                        let report =
                            policy.report_for_categories(&config.requested_endpoint_categories());
                        println!(
                            "research_only_warning=kis endpoint policy is research-only and market-data-only"
                        );
                        println!(
                            "broker_order_account_warning=broker, order, account, balance, holdings, and execution surfaces remain denied"
                        );
                        println!("{}", kis_endpoint_policy_to_text(&report));
                        Ok(())
                    })
            }
        }
        Commands::KisSymbolWhitelist { config } => {
            if config.contains("://") {
                Err("kis-symbol-whitelist config path must be local".to_string())
            } else {
                KISSymbolWhitelistConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        config.validate()?;
                        let report = config.build();
                        println!(
                            "research_only_warning=kis symbol whitelist is bounded, local-only, and market-data-only"
                        );
                        println!("scope_warning=no wildcard or all-symbol scans are permitted");
                        println!("{}", kis_symbol_whitelist_to_text(&report));
                        Ok(())
                    },
                )
            }
        }
        Commands::KisCollectionPlan { config } => {
            if config.contains("://") {
                Err("kis-collection-plan config path must be local".to_string())
            } else {
                KISMarketDataActivationConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        config.validate()?;
                        let auth = soma_zero::KISAuthReadinessReport::from_config(&config);
                        let policy = if let Some(path) = config.endpoint_policy_path.as_deref() {
                            soma_zero::KISEndpointPolicy::from_toml_path(std::path::Path::new(path))?
                        } else {
                            soma_zero::KISEndpointPolicy::default()
                        };
                        let endpoint_report =
                            policy.report_for_categories(&config.requested_endpoint_categories());
                        let mut whitelist = KISSymbolWhitelistConfig {
                            whitelist_id: format!("{}-symbols", config.activation_id),
                            output_root: config.output_root.clone(),
                            max_symbols: config.max_domestic_symbols.max(config.max_overseas_symbols),
                            ..KISSymbolWhitelistConfig::default()
                        };
                        for path in [
                            config.domestic_symbol_whitelist_path.as_deref(),
                            config.overseas_symbol_whitelist_path.as_deref(),
                        ]
                        .into_iter()
                        .flatten()
                        {
                            let loaded =
                                KISSymbolWhitelistConfig::from_toml_path(std::path::Path::new(path))?;
                            whitelist.symbols.extend(loaded.symbols);
                        }
                        whitelist.validate()?;
                        let plan = soma_zero::KISCollectionBatchPlan::build(
                            &config,
                            &auth,
                            &policy,
                            &endpoint_report,
                            &whitelist.build(),
                        );
                        println!(
                            "research_only_warning=kis collection planning is bounded, deterministic, local-first, and market-data-only"
                        );
                        println!(
                            "broker_order_account_warning=no broker, order, or account API is exposed by this command"
                        );
                        println!("{}", kis_collection_batch_plan_to_text(&plan));
                        Ok(())
                    })
            }
        }
        Commands::KisMarketDataDryRun { config } => {
            if config.contains("://") {
                Err("kis-market-data-dry-run config path must be local".to_string())
            } else {
                KISMarketDataDryRunConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let report = KISMarketDataDryRunRunner::default().run(&config)?;
                        println!("{}", report.to_text());
                        Ok(())
                    },
                )
            }
        }
        Commands::KisCollectionPlanV2 { config } => {
            if config.contains("://") {
                Err("kis-collection-plan-v2 config path must be local".to_string())
            } else {
                KISCollectionPlanV2Config::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let report = KISCollectionPlanV2Runner::default().run(&config)?;
                        println!("{}", report.to_text());
                        Ok(())
                    },
                )
            }
        }
        Commands::KisMarketDataActivate { config } => {
            if config.contains("://") {
                Err("kis-market-data-activate config path must be local".to_string())
            } else {
                KISMarketDataActivationConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let bundle = KISOfficialMarketDataActivationRunner::default().run(&config)?;
                        println!(
                            "research_only_warning=kis activation is research-only, market-data-only, paper-only, and secret-safe"
                        );
                        println!(
                            "broker_order_account_warning=broker, order, account, balance, holdings, and execution surfaces remain denied"
                        );
                        println!(
                            "{}",
                            kis_official_activation_report_to_text(&bundle.activation_report)
                        );
                        Ok(())
                    })
            }
        }
        Commands::KisMarketDataSmoke { config } => {
            if config.contains("://") {
                Err("kis-market-data-smoke config path must be local".to_string())
            } else {
                KISMarketDataEvidenceSmokeConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let bundle = KISMarketDataEvidenceSmokeRunner::default().run(&config)?;
                        println!("{}", bundle.market_data_smoke_report.to_text());
                        println!("{}", bundle.control_tower_auto_refresh_report.to_text());
                        println!("{}", bundle.operational_runbook_v2_report.to_text());
                        Ok(())
                    })
            }
        }
        Commands::KisCandleSufficiency { config } => {
            if config.contains("://") {
                Err("kis-candle-sufficiency config path must be local".to_string())
            } else {
                KISOutcomeLinkClosureConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        config.validate()?;
                        let report = soma_zero::KISCandleSufficiencyReport::build(
                            &format!("{}-cli-candle-sufficiency", config.closure_id),
                            &config.kis_canonical_csv_paths,
                            config.barrier_profile_registry_path.as_deref(),
                        );
                        println!(
                            "research_only_warning=kis candle sufficiency is market-data-only and no-lookahead constrained"
                        );
                        println!(
                            "broker_order_account_warning=no broker, order, or account API is exposed by this command"
                        );
                        println!("{}", kis_candle_sufficiency_to_text(&report));
                        Ok(())
                    })
            }
        }
        Commands::KisOutcomeLinkClose { config } => {
            if config.contains("://") {
                Err("kis-outcome-link-close config path must be local".to_string())
            } else {
                KISOutcomeLinkClosureConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = KISOutcomeLinkClosureRunner::default().run(&config)?;
                        println!(
                            "research_only_warning=kis outcome-link closure is paper-only and market-data-only"
                        );
                        println!(
                            "broker_order_account_warning=no broker, order, or account API is exposed by this command"
                        );
                        println!("{}", kis_outcome_link_closure_to_text(&report));
                        Ok(())
                    })
            }
        }
        Commands::KisKrxMigration { config } => {
            if config.contains("://") {
                Err("kis-krx-migration config path must be local".to_string())
            } else {
                KISMarketDataActivationConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let bundle = KISOfficialMarketDataActivationRunner::default().run(&config)?;
                        println!(
                            "research_only_warning=kis migration is operational only and not a performance claim"
                        );
                        println!(
                            "broker_order_account_warning=krx remains reference/fallback and KIS broker endpoints stay denied"
                        );
                        println!("{}", kis_krx_migration_to_text(&bundle.provider_migration_report));
                        Ok(())
                    })
            }
        }
        Commands::ProviderCatalog => Ok(()).map(|_| {
            let catalog: MarketDataProviderCatalog = build_default_provider_catalog();
            println!("{}", catalog.to_text());
        }),
        Commands::ProviderReadiness { config } => {
            if config.contains("://") {
                Err("provider-readiness config path must be local".to_string())
            } else {
                OfficialProviderReadinessConfig::from_toml_path(std::path::Path::new(&config))
                    .map_err(|err| err.to_string())
                    .and_then(|config| {
                        let report = OfficialProviderReadinessRunner::default().run(&config);
                        report.write_to_dir(std::path::Path::new(&config.output_dir))?;
                        println!("{}", provider_readiness_report_to_text(&report));
                        Ok(())
                    })
            }
        }
        Commands::ProviderSelect { market } => Ok(()).and_then(|_| {
            let market = parse_provider_market(&market)?;
            let config = OfficialProviderReadinessConfig {
                report_id: format!("provider-select-{market:?}"),
                markets: vec![market],
                ..OfficialProviderReadinessConfig::default()
            };
            let report = OfficialProviderReadinessRunner::default().run(&config);
            let result = report
                .selection_results
                .iter()
                .find(|result| result.market == market)
                .ok_or_else(|| "selection result missing".to_string())?;
            println!(
                "market={:?}\nstatus={:?}\nselected={}\nfallback={}\nmissing_auth={}\ndeferred={}",
                result.market,
                result.status,
                result
                    .selected_provider
                    .map(provider_kind_label)
                    .unwrap_or_default(),
                result
                    .fallback_selected
                    .map(provider_kind_label)
                    .unwrap_or_default(),
                result
                    .missing_auth_providers
                    .iter()
                    .copied()
                    .map(provider_kind_label)
                    .collect::<Vec<_>>()
                    .join("|"),
                result
                    .deferred_providers
                    .iter()
                    .copied()
                    .map(provider_kind_label)
                    .collect::<Vec<_>>()
                    .join("|"),
            );
            Ok(())
        }),
        Commands::ProviderReality { config } => {
            if config.contains("://") {
                Err("provider-reality config path must be local".to_string())
            } else {
                ProviderRealityConfig::from_toml_path(std::path::Path::new(&config))
                    .map_err(|err| err.to_string())
                    .and_then(|config| {
                        let report = ProviderRealityRunner::default().run(&config)?;
                        report.write_to_dir(std::path::Path::new(&config.output_dir))?;
                        println!("{}", provider_reality_report_to_text(&report));
                        Ok(())
                    })
            }
        }
        Commands::ProviderSimplify { config } => {
            if config.contains("://") {
                Err("provider-simplify config path must be local".to_string())
            } else {
                ProviderSimplificationConfig::from_toml_path(std::path::Path::new(&config))
                    .map_err(|err| err.to_string())
                    .and_then(|config| {
                        let report = ProviderSimplificationRunner::default().run(&config);
                        report.write_to_dir(&config.artifact_dir())?;
                        println!("{}", report.to_text());
                        Ok(())
                    })
            }
        }
        Commands::DashboardSnapshot { config } => {
            if config.contains("://") {
                Err("dashboard-snapshot config path must be local".to_string())
            } else {
                DashboardSourceConfig::from_toml_path(std::path::Path::new(&config))
                    .map_err(|err| err.to_string())
                    .and_then(|config| {
                        let state = DashboardSnapshotBuilder::default().build_and_write(&config)?;
                        println!("{}", state.to_text());
                        Ok(())
                    })
            }
        }
        Commands::DashboardRender { config } => {
            if config.contains("://") {
                Err("dashboard-render config path must be local".to_string())
            } else {
                DashboardRenderConfig::from_toml_path(std::path::Path::new(&config))
                    .map_err(|err| err.to_string())
                    .and_then(|config| {
                        let report = DashboardRenderer::default().render(&config)?;
                        println!("{}", report.to_text());
                        Ok(())
                    })
            }
        }
        Commands::ControlTowerV1 { config } => {
            if config.contains("://") {
                Err("control-tower-v1 config path must be local".to_string())
            } else {
                let config_path = std::path::Path::new(&config);
                ControlTowerV1Config::from_toml_path(config_path)
                    .map_err(|err| err.to_string())
                    .and_then(|config| {
                        let state = ControlTowerV1Builder::default().build(&config, Some(config_path))?;
                        let report = DashboardV1Renderer::default().render(&state, &config)?;
                        println!("{}", report.to_text());
                        Ok(())
                    })
            }
        }
        Commands::KisEvidenceDepthRun { config } => {
            if config.contains("://") {
                Err("kis-evidence-depth-run config path must be local".to_string())
            } else {
                let config_path = std::path::Path::new(&config);
                KISEvidenceDepthRunConfig::from_toml_path(config_path)
                    .and_then(|config| {
                        let bundle = KISEvidenceDepthRunRunner::default().run(&config, Some(config_path))?;
                        println!("{}", bundle.kis_evidence_depth_report.to_text());
                        Ok(())
                    })
            }
        }
        Commands::ControlTowerRefresh { config } => {
            if config.contains("://") {
                Err("control-tower-refresh config path must be local".to_string())
            } else {
                let config_path = std::path::Path::new(&config);
                ControlTowerRefreshConfig::from_toml_path(config_path)
                    .and_then(|config| {
                        let output = ControlTowerRefreshRunner::default()
                            .run(&config, Some(config_path), None, None)?;
                        println!("{}", output.report.to_text());
                        Ok(())
                    })
            }
        }
        Commands::KisEnvIsolationReport { config } => {
            if config.contains("://") {
                Err("kis-env-isolation-report config path must be local".to_string())
            } else {
                EnvironmentIsolationConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let report = EnvironmentIsolationRunner::default().run(&config)?;
                        println!("{}", report.to_text());
                        Ok(())
                    },
                )
            }
        }
        Commands::SecretRedactionAudit { config } => {
            if config.contains("://") {
                Err("secret-redaction-audit config path must be local".to_string())
            } else {
                SecretRedactionAuditConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let report = SecretRedactionAuditRunner::default().run(&config)?;
                        println!("{}", report.to_text());
                        Ok(())
                    },
                )
            }
        }
        Commands::ControlTowerAutoRefresh { config } => {
            if config.contains("://") {
                Err("control-tower-auto-refresh config path must be local".to_string())
            } else {
                ControlTowerAutoRefreshConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = ControlTowerAutoRefreshRunner::default().run(&config)?;
                        println!("{}", report.to_text());
                        Ok(())
                    })
            }
        }
        Commands::OperationalRunbook { config } => {
            if config.contains("://") {
                Err("operational-runbook config path must be local".to_string())
            } else {
                OperationalRunbookConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = OperationalRunbookRunner::default().run(&config)?;
                        println!("{}", report.to_text());
                        Ok(())
                    })
            }
        }
        Commands::OperationalRunbookV2 { config } => {
            if config.contains("://") {
                Err("operational-runbook-v2 config path must be local".to_string())
            } else {
                OperationalRunbookV2Config::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let report = OperationalRunbookV2Runner::default().run(&config)?;
                        println!("{}", report.to_text());
                        Ok(())
                    },
                )
            }
        }
        Commands::SystemReview { config } => {
            if config.contains("://") {
                Err("system-review config path must be local".to_string())
            } else {
                SystemIntegrationReviewConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let bundle = SystemIntegrationReviewRunner::default().run(&config)?;
                        println!(
                            "research_only_warning=system review is research-only, paper-only, and local-only"
                        );
                        println!(
                            "no_live_warning=system review never implies live trading, broker execution, or profitability"
                        );
                        println!("{}", bundle.to_json_string()?);
                        Ok(())
                    })
            }
        }
        Commands::EvidenceHardening { config } => {
            if config.contains("://") {
                Err("evidence-hardening config path must be local".to_string())
            } else {
                EvidenceHardeningConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let bundle = EvidenceHardeningRunner::default().run(&config)?;
                        println!(
                            "research_only_warning=evidence hardening is research-only, paper-only, local-only, and deterministic"
                        );
                        println!("{}", bundle.to_json_string()?);
                        Ok(())
                    },
                )
            }
        }
        Commands::OutcomeLinkCoverage { config } => {
            if config.contains("://") {
                Err("outcome-link-coverage config path must be local".to_string())
            } else {
                EvidenceHardeningConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let bundle = EvidenceHardeningRunner::default().run(&config)?;
                        println!(
                            "no_live_warning=outcome link coverage remains research-only and never implies live trading"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&bundle.outcome_link_coverage_report)
                                .map_err(|err| err.to_string())?
                        );
                        Ok(())
                    },
                )
            }
        }
        Commands::CounterfactualCoverage { config } => {
            if config.contains("://") {
                Err("counterfactual-coverage config path must be local".to_string())
            } else {
                EvidenceHardeningConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let bundle = EvidenceHardeningRunner::default().run(&config)?;
                        println!(
                            "paper_only_warning=counterfactual coverage is paper-only research evidence and never a broker path"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&bundle.counterfactual_coverage_report)
                                .map_err(|err| err.to_string())?
                        );
                        Ok(())
                    },
                )
            }
        }
        Commands::ReviewErgonomics { config } => {
            if config.contains("://") {
                Err("review-ergonomics config path must be local".to_string())
            } else {
                EvidenceHardeningConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let bundle = EvidenceHardeningRunner::default().run(&config)?;
                        println!(
                            "owner_review_warning=review ergonomics is a manual owner-review aid only and never executes commands"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&bundle.manual_review_ergonomics_report)
                                .map_err(|err| err.to_string())?
                        );
                        Ok(())
                    },
                )
            }
        }
        Commands::UiFrameworkDecision { config } => {
            if config.contains("://") {
                Err("ui-framework-decision config path must be local".to_string())
            } else {
                EvidenceHardeningConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let bundle = EvidenceHardeningRunner::default().run(&config)?;
                        println!(
                            "local_ui_warning=ui framework decision keeps local static UI now and rejects cloud dashboards for this sprint"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&bundle.ui_framework_decision_report)
                                .map_err(|err| err.to_string())?
                        );
                        Ok(())
                    },
                )
            }
        }
        Commands::MambaApplicationTiming { config } => {
            if config.contains("://") {
                Err("mamba-application-timing config path must be local".to_string())
            } else {
                EvidenceHardeningConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let bundle = EvidenceHardeningRunner::default().run(&config)?;
                        println!(
                            "runtime_deferred_warning=mamba application timing remains research-only and runtime deferred"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&bundle.mamba3_application_timing_report)
                                .map_err(|err| err.to_string())?
                        );
                        Ok(())
                    },
                )
            }
        }
        Commands::KisEvidenceExpansionPlanV2 { config } => {
            if config.contains("://") {
                Err("kis-evidence-expansion-plan-v2 config path must be local".to_string())
            } else {
                KISEvidenceExpansionPlanV2Config::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report =
                            BoundedKISOfficialEvidenceClosureRunner::default().build_expansion_plan(&config)?;
                        println!(
                            "research_only_warning=kis evidence expansion planning remains local-only and never enables live trading"
                        );
                        println!("{}", report.to_json_string()?);
                        Ok(())
                    })
            }
        }
        Commands::KisEvidenceClosure { config } => {
            if config.contains("://") {
                Err("kis-evidence-closure config path must be local".to_string())
            } else {
                KISEvidenceClosureConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let bundle =
                            BoundedKISOfficialEvidenceClosureRunner::default().run_kis_evidence_closure(&config)?;
                        println!(
                            "paper_only_warning=kis evidence closure is bounded, local-only, and never a live-trading approval"
                        );
                        println!("{}", bundle.to_json_string()?);
                        Ok(())
                    },
                )
            }
        }
        Commands::OutcomeLinkDepthCloseV2 { config } => {
            if config.contains("://") {
                Err("outcome-link-depth-close-v2 config path must be local".to_string())
            } else {
                OutcomeLinkDepthClosureV2Config::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = BoundedKISOfficialEvidenceClosureRunner::default()
                            .run_outcome_link_depth_closure_v2(&config)?;
                        println!(
                            "no_live_warning=outcome-link depth closure stays research-only and no-lookahead constrained"
                        );
                        println!("{}", report.to_json_string()?);
                        Ok(())
                    })
            }
        }
        Commands::OwnerReviewDisciplineV2 { config } => {
            if config.contains("://") {
                Err("owner-review-discipline-v2 config path must be local".to_string())
            } else {
                OwnerReviewDisciplineV2Config::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = BoundedKISOfficialEvidenceClosureRunner::default()
                            .run_owner_review_discipline_v2(&config)?;
                        println!(
                            "owner_review_warning=owner review discipline is manual-only, paper-only, and never executes broker actions"
                        );
                        println!("{}", report.to_json_string()?);
                        Ok(())
                    })
            }
        }
        Commands::SequenceReadinessHardening { config } => {
            if config.contains("://") {
                Err("sequence-readiness-hardening config path must be local".to_string())
            } else {
                SequenceDatasetPreparationConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = BoundedKISOfficialEvidenceClosureRunner::default()
                            .run_sequence_readiness_hardening(&config)?;
                        println!(
                            "runtime_deferred_warning=sequence readiness hardening prepares dataset export only and keeps mamba runtime deferred"
                        );
                        println!("{}", report.to_json_string()?);
                        Ok(())
                    })
            }
        }
        Commands::SequenceWindowPreview { config } => {
            if config.contains("://") {
                Err("sequence-window-preview config path must be local".to_string())
            } else {
                SequenceDatasetPreparationConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = BoundedKISOfficialEvidenceClosureRunner::default()
                            .build_sequence_window_preview(&config)?;
                        println!(
                            "preview_only_warning=sequence window preview is bounded planning only and never triggers training"
                        );
                        println!("{}", report.to_json_string()?);
                        Ok(())
                    })
            }
        }
        Commands::NoLookaheadSequenceProof { config } => {
            if config.contains("://") {
                Err("no-lookahead-sequence-proof config path must be local".to_string())
            } else {
                SequenceDatasetPreparationConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = BoundedKISOfficialEvidenceClosureRunner::default()
                            .build_no_lookahead_sequence_proof(&config)?;
                        println!(
                            "audit_only_warning=no-lookahead sequence proof is a deterministic local audit only"
                        );
                        println!("{}", report.to_json_string()?);
                        Ok(())
                    })
            }
        }
        Commands::SequenceDatasetExport { config } => {
            if config.contains("://") {
                Err("sequence-dataset-export config path must be local".to_string())
            } else {
                SequenceDatasetExportConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let bundle = SequenceDatasetExportRunner::default().run(&config)?;
                        println!(
                            "research_only_warning=sequence dataset export is bounded local export only and never training or live trading"
                        );
                        println!("{}", bundle.to_json_string()?);
                        Ok(())
                    },
                )
            }
        }
        Commands::SequenceDatasetQuality { config } => {
            if config.contains("://") {
                Err("sequence-dataset-quality config path must be local".to_string())
            } else {
                SequenceDatasetExportConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let report = SequenceDatasetExportRunner::default().run_quality(&config)?;
                        println!(
                            "research_only_warning=sequence dataset quality is a deterministic export audit only"
                        );
                        println!("{}", report.to_json_string()?);
                        Ok(())
                    },
                )
            }
        }
        Commands::SequenceDatasetDrift { config } => {
            if config.contains("://") {
                Err("sequence-dataset-drift config path must be local".to_string())
            } else {
                SequenceDatasetDriftGuardConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = SequenceDatasetExportRunner::default().run_drift_guard(&config)?;
                        println!(
                            "deterministic_warning=sequence dataset drift compares local manifests only"
                        );
                        println!("{}", report.to_json_string()?);
                        Ok(())
                    })
            }
        }
        Commands::SequenceDatasetReplayCheck { config } => {
            if config.contains("://") {
                Err("sequence-dataset-replay-check config path must be local".to_string())
            } else {
                SequenceDatasetExportConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |_| {
                        let report = SequenceDatasetExportRunner::default()
                            .run_replay_check(std::path::Path::new(&config))?;
                        println!(
                            "deterministic_warning=sequence dataset replay check reruns the same local export twice"
                        );
                        println!("{}", report.to_json_string()?);
                        Ok(())
                    },
                )
            }
        }
        Commands::ExternalBridgeReadiness { config } => {
            if config.contains("://") {
                Err("external-bridge-readiness config path must be local".to_string())
            } else {
                SequenceDatasetExportConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let report = SequenceDatasetExportRunner::default()
                            .run_external_bridge_readiness(&config)?;
                        println!(
                            "bridge_warning=external bridge readiness is import and evaluation only, never training"
                        );
                        println!("{}", report.to_json_string()?);
                        Ok(())
                    },
                )
            }
        }
        Commands::Mamba3finPrototypeGate { config } => {
            if config.contains("://") {
                Err("mamba3fin-prototype-gate config path must be local".to_string())
            } else {
                SequenceDatasetExportConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let report = SequenceDatasetExportRunner::default()
                            .run_mamba3fin_prototype_gate(&config)?;
                        println!(
                            "runtime_deferred_warning=mamba3fin prototype gate is planning-only and keeps runtime deferred"
                        );
                        println!("{}", report.to_json_string()?);
                        Ok(())
                    },
                )
            }
        }
        Commands::ExternalPredictionImportV2 { config } => {
            if config.contains("://") {
                Err("external-prediction-import-v2 config path must be local".to_string())
            } else {
                ExternalPredictionImportV2Config::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report =
                            ExternalPredictionEvaluationRunner::default().run_import(&config)?;
                        println!(
                            "research_only_warning=external prediction import is local-only CSV validation and never training or live inference"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::ExternalModelEvaluate { config } => {
            if config.contains("://") {
                Err("external-model-evaluate config path must be local".to_string())
            } else {
                ExternalPredictionImportV2Config::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report =
                            ExternalPredictionEvaluationRunner::default().run_evaluation(&config)?;
                        println!(
                            "research_only_warning=external model evaluation is deterministic offline scoring only and never a profitability claim"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::ExternalVsTrinity { config } => {
            if config.contains("://") {
                Err("external-vs-trinity config path must be local".to_string())
            } else {
                ExternalPredictionImportV2Config::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report =
                            ExternalPredictionEvaluationRunner::default().run_comparison(&config)?;
                        println!(
                            "diagnostic_warning=external-vs-trinity is an offline diagnostic comparison only"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::ExternalPredictionAblation { config } => {
            if config.contains("://") {
                Err("external-prediction-ablation config path must be local".to_string())
            } else {
                ExternalPredictionImportV2Config::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report =
                            ExternalPredictionEvaluationRunner::default().run_ablation(&config)?;
                        println!(
                            "diagnostic_warning=external prediction ablation is a deterministic stress check only"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::ExternalModelPromotionGate { config } => {
            if config.contains("://") {
                Err("external-model-promotion-gate config path must be local".to_string())
            } else {
                ExternalPredictionImportV2Config::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = ExternalPredictionEvaluationRunner::default()
                            .run_promotion_gate(&config)?;
                        println!(
                            "research_only_warning=external model promotion gate never implies live promotion or deployment"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::Mamba3finContract { config } => {
            if config.contains("://") {
                Err("mamba3fin-contract config path must be local".to_string())
            } else {
                ExternalPredictionImportV2Config::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report =
                            ExternalPredictionEvaluationRunner::default().run_mamba_contract(&config)?;
                        println!(
                            "runtime_deferred_warning=mamba3fin-contract is a planning-only contract and no runtime exists"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::ExternalArtifactRegistry { config } => {
            if config.contains("://") {
                Err("external-artifact-registry config path must be local".to_string())
            } else {
                ExternalModelArtifactRegistryConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report =
                            ExternalArtifactRegistryRunner::default().run_registry(&config)?;
                        println!(
                            "research_only_warning=external artifact registry is local-only offline bookkeeping and never training or live inference"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::ExternalEvaluationHistory { config } => {
            if config.contains("://") {
                Err("external-evaluation-history config path must be local".to_string())
            } else {
                ExternalModelArtifactRegistryConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report =
                            ExternalArtifactRegistryRunner::default().run_history(&config)?;
                        println!(
                            "research_only_warning=external evaluation history is an offline version-delta report only"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::CalibrationDrift { config } => {
            if config.contains("://") {
                Err("calibration-drift config path must be local".to_string())
            } else {
                ExternalModelArtifactRegistryConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = ExternalArtifactRegistryRunner::default()
                            .run_calibration_drift(&config)?;
                        println!(
                            "diagnostic_warning=calibration-drift is offline calibration tracking only"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::ExternalModelVersionComparison { config } => {
            if config.contains("://") {
                Err("external-model-version-comparison config path must be local".to_string())
            } else {
                ExternalModelArtifactRegistryConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = ExternalArtifactRegistryRunner::default()
                            .run_version_comparison(&config)?;
                        println!(
                            "diagnostic_warning=external model version comparison is diagnostic-only and never a live decision path"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::ConservativeExternalLeaderboard { config } => {
            if config.contains("://") {
                Err("conservative-external-leaderboard config path must be local".to_string())
            } else {
                ExternalModelArtifactRegistryConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report =
                            ExternalArtifactRegistryRunner::default().run_leaderboard(&config)?;
                        println!(
                            "research_only_warning=conservative external leaderboard is offline-only and never a deployment or profitability claim"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::ExternalRegistryAudit { config } => {
            if config.contains("://") {
                Err("external-registry-audit config path must be local".to_string())
            } else {
                ExternalModelArtifactRegistryConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = ExternalArtifactRegistryRunner::default().run_audit(&config)?;
                        println!(
                            "research_only_warning=external registry audit is a local-only safety scan with no broker, order, or account actions"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::ExternalModelResearchOps { config } => {
            if config.contains("://") {
                Err("external-model-research-ops config path must be local".to_string())
            } else {
                ExternalModelResearchOpsConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = ExternalModelResearchOpsRunner::default().run(&config)?;
                        println!(
                            "research_only_warning=external model research ops is local-only offline workflow orchestration and never training or live promotion"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::ExternalModelReviewQueue { config } => {
            if config.contains("://") {
                Err("external-model-review-queue config path must be local".to_string())
            } else {
                ExternalModelResearchOpsConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = ExternalModelResearchOpsRunner::default().run_review_queue(&config)?;
                        println!(
                            "research_only_warning=external model review queue is research-only offline review bookkeeping"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::ExternalModelWatchlist { config } => {
            if config.contains("://") {
                Err("external-model-watchlist config path must be local".to_string())
            } else {
                ExternalModelResearchOpsConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = ExternalModelResearchOpsRunner::default().run_watchlist(&config)?;
                        println!(
                            "research_only_warning=external model watchlist is offline-only and never a deployment approval"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::ModelComparabilityMatrix { config } => {
            if config.contains("://") {
                Err("model-comparability-matrix config path must be local".to_string())
            } else {
                ExternalModelResearchOpsConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = ExternalModelResearchOpsRunner::default()
                            .run_comparability_matrix(&config)?;
                        println!(
                            "diagnostic_warning=model comparability matrix is diagnostic-only compatibility analysis from local artifacts"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::ArtifactCompleteness { config } => {
            if config.contains("://") {
                Err("artifact-completeness config path must be local".to_string())
            } else {
                ExternalModelResearchOpsConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = ExternalModelResearchOpsRunner::default()
                            .run_artifact_completeness(&config)?;
                        println!(
                            "research_only_warning=artifact completeness is local-only artifact scoring with no runtime or deployment semantics"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::ModelRiskProfile { config } => {
            if config.contains("://") {
                Err("model-risk-profile config path must be local".to_string())
            } else {
                ExternalModelResearchOpsConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = ExternalModelResearchOpsRunner::default().run_risk_profile(&config)?;
                        println!(
                            "research_only_warning=model risk profile is conservative evidence review only and never live promotion"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::ModelLeaderboardChangelog { config } => {
            if config.contains("://") {
                Err("model-leaderboard-changelog config path must be local".to_string())
            } else {
                ExternalModelResearchOpsConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = ExternalModelResearchOpsRunner::default()
                            .run_leaderboard_changelog(&config)?;
                        println!(
                            "research_only_warning=model leaderboard changelog is offline-only and never a deployment or profitability claim"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::ModelReviewClose { config } => {
            if config.contains("://") {
                Err("model-review-close config path must be local".to_string())
            } else {
                ModelOpsReviewClosureConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = ModelReviewClosureRunner::default().run(&config)?;
                        println!(
                            "research_only_warning=model review closure is offline-only and never enables live/runtime inference or training"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::PredictionHistoryPack { config } => {
            if config.contains("://") {
                Err("prediction-history-pack config path must be local".to_string())
            } else {
                PredictionHistoryPackConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report =
                            ModelReviewClosureRunner::default().run_prediction_history_pack(&config)?;
                        println!(
                            "research_only_warning=prediction history pack is research-only offline version coverage and never training"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::ModelOpsDecisionLog { config } => {
            if config.contains("://") {
                Err("model-ops-decision-log config path must be local".to_string())
            } else {
                ModelOpsReviewClosureConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = ModelReviewClosureRunner::default().run_decision_log(&config)?;
                        println!(
                            "research_only_warning=model ops decision log is offline-only review trace and never a deployment claim"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::ModelOpsOperatorQa { config } => {
            if config.contains("://") {
                Err("model-ops-operator-qa config path must be local".to_string())
            } else {
                ModelOpsReviewClosureConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = ModelReviewClosureRunner::default().run_operator_qa(&config)?;
                        println!(
                            "research_only_warning=model ops operator qa is no-deployment offline guidance only"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::ModelOpsRegressionGuard { config } => {
            if config.contains("://") {
                Err("model-ops-regression-guard config path must be local".to_string())
            } else {
                ModelOpsReviewClosureConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report =
                            ModelReviewClosureRunner::default().run_regression_guard(&config)?;
                        println!(
                            "research_only_warning=model ops regression guard is deterministic offline drift detection only"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::ControlTowerModelOpsRefresh { config } => {
            if config.contains("://") {
                Err("control-tower-model-ops-refresh config path must be local".to_string())
            } else {
                ModelOpsReviewClosureConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report =
                            ModelReviewClosureRunner::default().run_control_tower_refresh(&config)?;
                        println!(
                            "research_only_warning=control tower model ops refresh is read-only static refresh only"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::ModelOpsRollup { config } => {
            if config.contains("://") {
                Err("model-ops-rollup config path must be local".to_string())
            } else {
                ModelOpsRollupConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let report = ModelOpsRollupRunner::default().run(&config)?;
                        println!(
                            "research_only_warning=model ops rollup is offline-only, paper-only, no-training, and no-live-inference"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    },
                )
            }
        }
        Commands::ModelRegressionExplain { config } => {
            if config.contains("://") {
                Err("model-regression-explain config path must be local".to_string())
            } else {
                ModelOpsRollupConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let report = ModelOpsRollupRunner::default().run_regression_explain(&config)?;
                        println!(
                            "research_only_warning=model regression explain is offline-only explainability with no training or runtime inference"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    },
                )
            }
        }
        Commands::OperatorQaRollup { config } => {
            if config.contains("://") {
                Err("operator-qa-rollup config path must be local".to_string())
            } else {
                ModelOpsRollupConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let report = ModelOpsRollupRunner::default().run_operator_qa_rollup(&config)?;
                        println!(
                            "research_only_warning=operator qa rollup is research-only deduped review guidance with no execution path"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    },
                )
            }
        }
        Commands::DecisionLogRollup { config } => {
            if config.contains("://") {
                Err("decision-log-rollup config path must be local".to_string())
            } else {
                ModelOpsRollupConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let report = ModelOpsRollupRunner::default().run_decision_log_rollup(&config)?;
                        println!(
                            "research_only_warning=decision log rollup is diagnostic-only static aggregation with no deployment semantics"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    },
                )
            }
        }
        Commands::ModelRiskRollup { config } => {
            if config.contains("://") {
                Err("model-risk-rollup config path must be local".to_string())
            } else {
                ModelOpsRollupConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let report = ModelOpsRollupRunner::default().run_model_risk_rollup(&config)?;
                        println!(
                            "research_only_warning=model risk rollup is offline-only and never a live promotion path"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    },
                )
            }
        }
        Commands::ModelActionPriority { config } => {
            if config.contains("://") {
                Err("model-action-priority config path must be local".to_string())
            } else {
                ModelOpsRollupConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let report = ModelOpsRollupRunner::default().run_action_priority(&config)?;
                        println!(
                            "research_only_warning=model action priority is copy-only local guidance with no execution path"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    },
                )
            }
        }
        Commands::ControlTowerModelOpsRollup { config } => {
            if config.contains("://") {
                Err("control-tower-model-ops-rollup config path must be local".to_string())
            } else {
                ModelOpsRollupConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let report =
                            ModelOpsRollupRunner::default().run_control_tower_model_ops_rollup(&config)?;
                        println!(
                            "research_only_warning=control tower model ops rollup is read-only static visibility only"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    },
                )
            }
        }
        Commands::ModelOpsTrace { config } => {
            if config.contains("://") {
                Err("model-ops-trace config path must be local".to_string())
            } else {
                ModelOpsTraceConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let report = ModelOpsTraceRunner::default().run(&config)?;
                        println!(
                            "research_only_warning=model ops trace is static/read-only, paper-only, local-only, and no-training"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    },
                )
            }
        }
        Commands::ModelTraceIndex { config } => {
            if config.contains("://") {
                Err("model-trace-index config path must be local".to_string())
            } else {
                ModelOpsTraceConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let report = ModelOpsTraceRunner::default().run_artifact_trace_index(&config)?;
                        println!(
                            "research_only_warning=model trace index is local-only artifact lineage with no execution path"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    },
                )
            }
        }
        Commands::ModelDecisionConflicts { config } => {
            if config.contains("://") {
                Err("model-decision-conflicts config path must be local".to_string())
            } else {
                ModelOpsTraceConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let report = ModelOpsTraceRunner::default().run_decision_conflicts(&config)?;
                        println!(
                            "research_only_warning=model decision conflicts is research-only conservative conflict review"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    },
                )
            }
        }
        Commands::ModelRegressionTrace { config } => {
            if config.contains("://") {
                Err("model-regression-trace config path must be local".to_string())
            } else {
                ModelOpsTraceConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let report = ModelOpsTraceRunner::default().run_regression_trace(&config)?;
                        println!(
                            "research_only_warning=model regression trace is diagnostic offline evidence only"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    },
                )
            }
        }
        Commands::ModelQaTrace { config } => {
            if config.contains("://") {
                Err("model-qa-trace config path must be local".to_string())
            } else {
                ModelOpsTraceConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let report = ModelOpsTraceRunner::default().run_qa_trace(&config)?;
                        println!(
                            "research_only_warning=model qa trace is operator QA only and read-only"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    },
                )
            }
        }
        Commands::ModelActionTrace { config } => {
            if config.contains("://") {
                Err("model-action-trace config path must be local".to_string())
            } else {
                ModelOpsTraceConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let report = ModelOpsTraceRunner::default().run_action_trace(&config)?;
                        println!(
                            "research_only_warning=model action trace is copy-only rationale with no execution"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    },
                )
            }
        }
        Commands::ModelVersionDiffTrace { config } => {
            if config.contains("://") {
                Err("model-version-diff-trace config path must be local".to_string())
            } else {
                ModelOpsTraceConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let report = ModelOpsTraceRunner::default().run_model_version_diff_trace(&config)?;
                        println!(
                            "research_only_warning=model version diff trace is deterministic local comparison only"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    },
                )
            }
        }
        Commands::BaselineSnapshotCoverage { config } => {
            if config.contains("://") {
                Err("baseline-snapshot-coverage config path must be local".to_string())
            } else {
                BaselineSnapshotCoverageConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let report = BaselineSnapshotCoverageRunner::default()
                            .run_baseline_snapshot_coverage(&config)?;
                        println!(
                            "research_only_warning=baseline snapshot coverage is static/read-only, paper-only, local-only, and no-training"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    },
                )
            }
        }
        Commands::ComparisonTargetRegistry { config } => {
            if config.contains("://") {
                Err("comparison-target-registry config path must be local".to_string())
            } else {
                BaselineSnapshotCoverageConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let report = BaselineSnapshotCoverageRunner::default()
                            .run_comparison_target_registry(&config)?;
                        println!(
                            "research_only_warning=comparison target registry is research-only local mapping with no promotion path"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    },
                )
            }
        }
        Commands::MissingComparisonTargets { config } => {
            if config.contains("://") {
                Err("missing-comparison-targets config path must be local".to_string())
            } else {
                BaselineSnapshotCoverageConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let report = BaselineSnapshotCoverageRunner::default()
                            .run_missing_comparison_targets(&config)?;
                        println!(
                            "research_only_warning=missing comparison targets is diagnostic conservative closure audit only"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    },
                )
            }
        }
        Commands::TraceCompletenessAudit { config } => {
            if config.contains("://") {
                Err("trace-completeness-audit config path must be local".to_string())
            } else {
                BaselineSnapshotCoverageConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let report = BaselineSnapshotCoverageRunner::default()
                            .run_trace_completeness_audit(&config)?;
                        println!(
                            "research_only_warning=trace completeness audit is a coverage audit with static local output only"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    },
                )
            }
        }
        Commands::DowngradeEvidenceAudit { config } => {
            if config.contains("://") {
                Err("downgrade-evidence-audit config path must be local".to_string())
            } else {
                BaselineSnapshotCoverageConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let report = BaselineSnapshotCoverageRunner::default()
                            .run_downgrade_evidence_audit(&config)?;
                        println!(
                            "research_only_warning=downgrade evidence audit is conservative static evidence review only"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    },
                )
            }
        }
        Commands::SnapshotDiffIntegrity { config } => {
            if config.contains("://") {
                Err("snapshot-diff-integrity config path must be local".to_string())
            } else {
                BaselineSnapshotCoverageConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let report = BaselineSnapshotCoverageRunner::default()
                            .run_snapshot_diff_integrity(&config)?;
                        println!(
                            "research_only_warning=snapshot diff integrity is deterministic local comparison audit only"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    },
                )
            }
        }
        Commands::ControlTowerTraceCoverage { config } => {
            if config.contains("://") {
                Err("control-tower-trace-coverage config path must be local".to_string())
            } else {
                BaselineSnapshotCoverageConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let report = BaselineSnapshotCoverageRunner::default()
                            .run_control_tower_trace_coverage(&config)?;
                        println!(
                            "research_only_warning=control tower trace coverage is read-only static local coverage only"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    },
                )
            }
        }
        Commands::UnexpectedDiffTriage { config } => {
            if config.contains("://") {
                Err("unexpected-diff-triage config path must be local".to_string())
            } else {
                UnexpectedDiffTriageConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let report = UnexpectedDiffTriageRunner::default()
                            .run_unexpected_diff_triage(&config)?;
                        println!(
                            "research_only_warning=unexpected diff triage is static/read-only, paper-only, local-only, and no-training"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    },
                )
            }
        }
        Commands::SnapshotDiffClassify { config } => {
            if config.contains("://") {
                Err("snapshot-diff-classify config path must be local".to_string())
            } else {
                UnexpectedDiffTriageConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let report = UnexpectedDiffTriageRunner::default()
                            .run_snapshot_diff_classification(&config)?;
                        println!(
                            "research_only_warning=snapshot diff classification is deterministic local explanation only"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    },
                )
            }
        }
        Commands::ContractAlignmentAuditV2 { config } => {
            if config.contains("://") {
                Err("contract-alignment-audit-v2 config path must be local".to_string())
            } else {
                UnexpectedDiffTriageConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let report = UnexpectedDiffTriageRunner::default()
                            .run_contract_alignment_audit_v2(&config)?;
                        println!(
                            "research_only_warning=contract alignment audit v2 is static local contract explanation only"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    },
                )
            }
        }
        Commands::OwnerReviewCloseV2 { config } => {
            if config.contains("://") {
                Err("owner-review-close-v2 config path must be local".to_string())
            } else {
                UnexpectedDiffTriageConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let report = UnexpectedDiffTriageRunner::default()
                            .run_owner_review_closure_v2(&config)?;
                        println!(
                            "research_only_warning=owner review closure v2 is paper-only conservative closure with no live/runtime/training path"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    },
                )
            }
        }
        Commands::TraceWarningReduce { config } => {
            if config.contains("://") {
                Err("trace-warning-reduce config path must be local".to_string())
            } else {
                UnexpectedDiffTriageConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let report = UnexpectedDiffTriageRunner::default()
                            .run_trace_warning_reduction(&config)?;
                        println!(
                            "research_only_warning=trace warning reduction is explicit static warning review only"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    },
                )
            }
        }
        Commands::DowngradeEvidenceClosurePlan { config } => {
            if config.contains("://") {
                Err("downgrade-evidence-closure-plan config path must be local".to_string())
            } else {
                UnexpectedDiffTriageConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let report = UnexpectedDiffTriageRunner::default()
                            .run_downgrade_evidence_closure_plan(&config)?;
                        println!(
                            "research_only_warning=downgrade evidence closure plan is conservative static planning only"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    },
                )
            }
        }
        Commands::DiffRootCause { config } => {
            if config.contains("://") {
                Err("diff-root-cause config path must be local".to_string())
            } else {
                UnexpectedDiffTriageConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let report = UnexpectedDiffTriageRunner::default()
                            .run_diff_root_cause(&config)?;
                        println!(
                            "research_only_warning=diff root cause is deterministic offline explanation only"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    },
                )
            }
        }
        Commands::ModelVersionReviewDisposition { config } => {
            if config.contains("://") {
                Err("model-version-review-disposition config path must be local".to_string())
            } else {
                UnexpectedDiffTriageConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let report = UnexpectedDiffTriageRunner::default()
                            .run_model_version_review_disposition(&config)?;
                        println!(
                            "research_only_warning=model version review disposition is paper-only recommendation output with no promotion path"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    },
                )
            }
        }
        Commands::ControlTowerDiffTriage { config } => {
            if config.contains("://") {
                Err("control-tower-diff-triage config path must be local".to_string())
            } else {
                UnexpectedDiffTriageConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let report = UnexpectedDiffTriageRunner::default()
                            .run_control_tower_diff_triage(&config)?;
                        println!(
                            "research_only_warning=control tower diff triage is read-only static local triage only"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    },
                )
            }
        }
        Commands::OperatorBriefing { config } => {
            if config.contains("://") {
                Err("operator-briefing config path must be local".to_string())
            } else {
                OperatorBriefingConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let report = OperatorBriefingRunner::default()
                            .run_operator_briefing(&config)?;
                        println!(
                            "research_only_warning=operator briefing is static/read-only, paper-only, local-only, no-training, no-live-inference, and no order/account path"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    },
                )
            }
        }
        Commands::OwnerActionChecklist { config } => {
            if config.contains("://") {
                Err("owner-action-checklist config path must be local".to_string())
            } else {
                OperatorBriefingConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let report = OperatorBriefingRunner::default()
                            .run_owner_action_checklist(&config)?;
                        println!(
                            "paper_only_warning=owner action checklist is paper-only, copy-only, local-only, and has no execution path"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    },
                )
            }
        }
        Commands::OperatorDecisionQueue { config } => {
            if config.contains("://") {
                Err("operator-decision-queue config path must be local".to_string())
            } else {
                OperatorBriefingConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let report = OperatorBriefingRunner::default()
                            .run_operator_decision_queue(&config)?;
                        println!(
                            "research_only_warning=operator decision queue is static/read-only, local-only, and provides no execution controls"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    },
                )
            }
        }
        Commands::BriefingDelta { config } => {
            if config.contains("://") {
                Err("briefing-delta config path must be local".to_string())
            } else {
                OperatorBriefingConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let report =
                            OperatorBriefingRunner::default().run_briefing_delta(&config)?;
                        println!(
                            "local_only_warning=briefing delta compares local static briefing artifacts only"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    },
                )
            }
        }
        Commands::LeaderboardWarningClosure { config } => {
            if config.contains("://") {
                Err("leaderboard-warning-closure config path must be local".to_string())
            } else {
                OperatorBriefingConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let report = OperatorBriefingRunner::default()
                            .run_leaderboard_warning_closure(&config)?;
                        println!(
                            "conservative_warning=leaderboard warning closure is research-only, conservative, local-only, and never a live promotion path"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    },
                )
            }
        }
        Commands::RetirementEvidenceCompletion { config } => {
            if config.contains("://") {
                Err("retirement-evidence-completion config path must be local".to_string())
            } else {
                OperatorBriefingConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let report = OperatorBriefingRunner::default()
                            .run_retirement_evidence_completion(&config)?;
                        println!(
                            "conservative_warning=retirement evidence completion is paper-only, local-only, and retirement never means deletion"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    },
                )
            }
        }
        Commands::ControlTowerBriefing { config } => {
            if config.contains("://") {
                Err("control-tower-briefing config path must be local".to_string())
            } else {
                OperatorBriefingConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let report = OperatorBriefingRunner::default()
                            .run_control_tower_briefing(&config)?;
                        println!(
                            "research_only_warning=control tower briefing is static/read-only, local-only, no-live, and no order/account controls exist"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    },
                )
            }
        }
        Commands::OfflineEvidenceAttach { config } => {
            if config.contains("://") {
                Err("offline-evidence-attach config path must be local".to_string())
            } else {
                OfflineEvidenceAttachmentConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = OfflineEvidenceAttachmentRunner::default().run(&config)?;
                        println!(
                            "research_only_warning=offline evidence attachment is local-only, static/read-only, paper-only, and never a live readiness claim"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::PredictionHistoryExpand { config } => {
            if config.contains("://") {
                Err("prediction-history-expand config path must be local".to_string())
            } else {
                PredictionHistoryExpansionConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let (plan, report) =
                            PredictionHistoryExpansionRunner::default().run(&config)?;
                        println!(
                            "research_only_warning=prediction history expansion is offline-only, local-only, and never training or live inference"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&serde_json::json!({
                                "prediction_history_expansion_plan": plan,
                                "prediction_history_expansion_report": report,
                            }))
                            .map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::RetirementRegressionPack { config } => {
            if config.contains("://") {
                Err("retirement-regression-pack config path must be local".to_string())
            } else {
                RetirementRegressionEvidencePackConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = OfflineEvidenceAttachmentRunner::default()
                            .run_retirement_regression_pack(&config)?;
                        println!(
                            "conservative_warning=retirement regression pack is local-only, conservative, and retirement never means deletion"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::EvidenceGapCloseV2 { config } => {
            if config.contains("://") {
                Err("evidence-gap-close-v2 config path must be local".to_string())
            } else {
                EvidenceGapClosureV2Config::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let report = EvidenceGapClosureRunnerV2::default().run(&config)?;
                        println!(
                            "research_only_warning=evidence gap closure v2 is static local audit only and never enables execution"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    },
                )
            }
        }
        Commands::OwnerChecklistClose { config } => {
            if config.contains("://") {
                Err("owner-checklist-close config path must be local".to_string())
            } else {
                OwnerChecklistClosureConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let report = OwnerChecklistClosureRunner::default().run(&config)?;
                        println!(
                            "paper_only_warning=owner checklist closure is evidence-based, no-execution, and local-only"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    },
                )
            }
        }
        Commands::DirectWatchScore { config } => {
            if config.contains("://") {
                Err("direct-watch-score config path must be local".to_string())
            } else {
                OfflineEvidenceAttachmentConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report =
                            OfflineEvidenceAttachmentRunner::default().run_direct_watch_score(&config)?;
                        println!(
                            "monitoring_only_warning=direct watch score is monitoring-only, paper-only, and never live trading readiness"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::BriefingReadinessGate { config } => {
            if config.contains("://") {
                Err("briefing-readiness-gate config path must be local".to_string())
            } else {
                OfflineEvidenceAttachmentConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = OfflineEvidenceAttachmentRunner::default()
                            .run_briefing_readiness_gate(&config)?;
                        println!(
                            "no_live_readiness_warning=briefing readiness gate is static/read-only, no execution, and never a live readiness approval"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::ExtModelBPredictionClose { config } => {
            if config.contains("://") {
                Err("ext-model-b-prediction-close config path must be local".to_string())
            } else {
                ExtModelBPredictionClosureConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = ExtModelBPredictionClosureRunner::default()
                            .run_prediction_closure(&config)?;
                        println!(
                            "research_only_warning=ext-model-b prediction closure is local-only, bounded fixture closure, and never training or live inference"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::PredictionCoverageFinalize { config } => {
            if config.contains("://") {
                Err("prediction-coverage-finalize config path must be local".to_string())
            } else {
                ExtModelBPredictionClosureConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = ExtModelBPredictionClosureRunner::default()
                            .run_prediction_coverage_finalization(&config)?;
                        println!(
                            "research_only_warning=prediction coverage finalization is research-only, bounded, and never a training or deploy path"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::EvidenceGapFinalClose { config } => {
            if config.contains("://") {
                Err("evidence-gap-final-close config path must be local".to_string())
            } else {
                ExtModelBPredictionClosureConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = ExtModelBPredictionClosureRunner::default()
                            .run_evidence_gap_final_closure(&config)?;
                        println!(
                            "no_live_warning=evidence gap final closure is static local audit only and never live trading readiness"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::DirectWatchFinalGate { config } => {
            if config.contains("://") {
                Err("direct-watch-final-gate config path must be local".to_string())
            } else {
                ExtModelBPredictionClosureConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = ExtModelBPredictionClosureRunner::default()
                            .run_direct_watch_final_gate(&config)?;
                        println!(
                            "monitoring_only_warning=direct watch final gate is monitoring-only, static/read-only, and never execution or live trading readiness"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::ControlTowerFinalRefresh { config } => {
            if config.contains("://") {
                Err("control-tower-final-refresh config path must be local".to_string())
            } else {
                ExtModelBPredictionClosureConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = ExtModelBPredictionClosureRunner::default()
                            .run_control_tower_final_refresh(&config)?;
                        println!(
                            "read_only_warning=control tower final refresh is static/read-only, local-only, and contains no train/live/order/account controls"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::Sprint73WorkspaceAcceptance { config } => {
            if config.contains("://") {
                Err("sprint73-workspace-acceptance config path must be local".to_string())
            } else {
                ExtModelBPredictionClosureConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = ExtModelBPredictionClosureRunner::default()
                            .run_with_workspace_acceptance(&config)?
                            .sprint73_workspace_acceptance_report;
                        println!(
                            "workspace_acceptance_warning=sprint73 workspace acceptance records full-workspace verification only and never implies live readiness"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::RealEvidenceFollowup { config } => {
            if config.contains("://") {
                Err("real-evidence-followup config path must be local".to_string())
            } else {
                RealEvidenceFollowupConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let report = RealEvidenceFollowupRunner::default().run(&config)?;
                        println!(
                            "market_data_only_warning=real evidence follow-up is research-only, paper-only, market-data-only, static/read-only, and never live trading"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    },
                )
            }
        }
        Commands::RealEvidenceAttach { config } => {
            if config.contains("://") {
                Err("real-evidence-attach config path must be local".to_string())
            } else {
                RealEvidenceFollowupConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let report =
                            RealEvidenceFollowupRunner::default().run_attachment_report(&config)?;
                        println!(
                            "local_only_warning=real evidence attach is local-only, market-data-only, and never broker/order/account execution"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    },
                )
            }
        }
        Commands::KisRealEvidenceValidate { config } => {
            if config.contains("://") {
                Err("kis-real-evidence-validate config path must be local".to_string())
            } else {
                RealEvidenceFollowupConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let report = RealEvidenceFollowupRunner::default().run_validation(&config)?;
                        println!(
                            "research_only_warning=kis real evidence validation is research-only, local-only, and never a runtime/live path"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    },
                )
            }
        }
        Commands::RealProvenanceAudit { config } => {
            if config.contains("://") {
                Err("real-provenance-audit config path must be local".to_string())
            } else {
                RealEvidenceFollowupConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let report =
                            RealEvidenceFollowupRunner::default().run_provenance_audit(&config)?;
                        println!(
                            "provenance_required_warning=real provenance audit requires explicit local provenance and never uses remote paths"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    },
                )
            }
        }
        Commands::RealPreflightAudit { config } => {
            if config.contains("://") {
                Err("real-preflight-audit config path must be local".to_string())
            } else {
                RealEvidenceFollowupConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let report =
                            RealEvidenceFollowupRunner::default().run_preflight_audit(&config)?;
                        println!(
                            "preflight_required_warning=real preflight audit requires local preflight evidence before evaluation use"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    },
                )
            }
        }
        Commands::RealOutcomeReadiness { config } => {
            if config.contains("://") {
                Err("real-outcome-readiness config path must be local".to_string())
            } else {
                RealEvidenceFollowupConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let report =
                            RealEvidenceFollowupRunner::default().run_outcome_readiness(&config)?;
                        println!(
                            "no_live_trading_warning=real outcome readiness is paper-only, research-only, and never a live trading path"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    },
                )
            }
        }
        Commands::RealSequenceReadiness { config } => {
            if config.contains("://") {
                Err("real-sequence-readiness config path must be local".to_string())
            } else {
                RealEvidenceFollowupConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let report =
                            RealEvidenceFollowupRunner::default().run_sequence_readiness(&config)?;
                        println!(
                            "no_training_warning=real sequence readiness is offline-only, no-training, and no runtime path"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    },
                )
            }
        }
        Commands::RealModelopsImpact { config } => {
            if config.contains("://") {
                Err("real-modelops-impact config path must be local".to_string())
            } else {
                RealEvidenceFollowupConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let report =
                            RealEvidenceFollowupRunner::default().run_model_ops_impact(&config)?;
                        println!(
                            "no_live_inference_warning=real modelops impact is offline follow-up only and never live inference"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    },
                )
            }
        }
        Commands::ControlTowerWarningReduce { config } => {
            if config.contains("://") {
                Err("control-tower-warning-reduce config path must be local".to_string())
            } else {
                RealEvidenceFollowupConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let report =
                            RealEvidenceFollowupRunner::default().run_warning_reduction(&config)?;
                        println!(
                            "warning_not_hidden=control tower warning reduction explains or reduces warnings but never hides them"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    },
                )
            }
        }
        Commands::DirectWatchWarningRationale { config } => {
            if config.contains("://") {
                Err("direct-watch-warning-rationale config path must be local".to_string())
            } else {
                RealEvidenceFollowupConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let report = RealEvidenceFollowupRunner::default()
                            .run_direct_watch_warning_rationale(&config)?;
                        println!(
                            "monitoring_only_warning=direct-watch warning rationale is monitoring-only and never enables execution"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    },
                )
            }
        }
        Commands::RealEvidenceRunbook { config } => {
            if config.contains("://") {
                Err("real-evidence-runbook config path must be local".to_string())
            } else {
                RealEvidenceFollowupConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let report = RealEvidenceFollowupRunner::default().run_runbook(&config)?;
                        println!(
                            "copyable_commands_only=real evidence runbook emits copyable commands only and never execution buttons"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    },
                )
            }
        }
        Commands::RealPredictionRequirements { config } => {
            if config.contains("://") {
                Err("real-prediction-requirements config path must be local".to_string())
            } else {
                RealEvidencePredictionRefreshConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = RealEvidencePredictionRefreshRunner::default()
                            .run_prediction_requirements(&config)?;
                        println!(
                            "research_only_warning=real prediction requirements are research-only, paper-only, static/read-only, local-only, and never training or live trading"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::RealPredictionRefreshPlan { config } => {
            if config.contains("://") {
                Err("real-prediction-refresh-plan config path must be local".to_string())
            } else {
                RealEvidencePredictionRefreshConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = RealEvidencePredictionRefreshRunner::default()
                            .run_prediction_refresh_plan(&config)?;
                        println!(
                            "no_training_warning=real prediction refresh plan is offline-only, copy-only, and never model training or runtime inference"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::RealPredictionImport { config } => {
            if config.contains("://") {
                Err("real-prediction-import config path must be local".to_string())
            } else {
                RealEvidencePredictionRefreshConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = RealEvidencePredictionRefreshRunner::default()
                            .run_prediction_import(&config)?;
                        println!(
                            "schema_validation_warning=real prediction import performs local schema validation only and never enables runtime/live inference"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::RealExternalReevaluate { config } => {
            if config.contains("://") {
                Err("real-external-reevaluate config path must be local".to_string())
            } else {
                RealEvidencePredictionRefreshConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = RealEvidencePredictionRefreshRunner::default()
                            .run_external_reevaluation(&config)?;
                        println!(
                            "offline_metrics_warning=real external reevaluation is offline-only, research-only, and never a profitability or live-use claim"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::RealLeaderboardRefresh { config } => {
            if config.contains("://") {
                Err("real-leaderboard-refresh config path must be local".to_string())
            } else {
                RealEvidencePredictionRefreshConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = RealEvidencePredictionRefreshRunner::default()
                            .run_leaderboard_refresh(&config)?;
                        println!(
                            "no_deployment_warning=real leaderboard refresh is research-only and never implies deployment, runtime inference, or live trading"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::RealModelopsRefresh { config } => {
            if config.contains("://") {
                Err("real-modelops-refresh config path must be local".to_string())
            } else {
                RealEvidencePredictionRefreshConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = RealEvidencePredictionRefreshRunner::default()
                            .run_model_ops_refresh(&config)?;
                        println!(
                            "no_live_inference_warning=real modelops refresh is offline-only, paper-only, and never live inference, training, or broker execution"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::ModelPredictionsStaleClose { config } => {
            if config.contains("://") {
                Err("model-predictions-stale-close config path must be local".to_string())
            } else {
                RealEvidencePredictionRefreshConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = RealEvidencePredictionRefreshRunner::default()
                            .run_stale_closure(&config)?;
                        println!(
                            "warning_not_hidden=model predictions stale closure only closes or explains the warning and never hides it"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::ControlTowerWarningCloseV2 { config } => {
            if config.contains("://") {
                Err("control-tower-warning-close-v2 config path must be local".to_string())
            } else {
                RealEvidencePredictionRefreshConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = RealEvidencePredictionRefreshRunner::default()
                            .run_warning_closure_v2(&config)?;
                        println!(
                            "read_only_warning=control tower warning closure v2 is static/read-only, local-only, and never adds execution controls"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::DirectWatchPostEvidenceGate { config } => {
            if config.contains("://") {
                Err("direct-watch-post-evidence-gate config path must be local".to_string())
            } else {
                RealEvidencePredictionRefreshConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = RealEvidencePredictionRefreshRunner::default()
                            .run_direct_watch_post_evidence_gate(&config)?;
                        println!(
                            "monitoring_only_warning=direct watch post-evidence gate remains monitoring-only, paper-only, and never execution readiness"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::RealModelopsRunbook { config } => {
            if config.contains("://") {
                Err("real-modelops-runbook config path must be local".to_string())
            } else {
                RealEvidencePredictionRefreshConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = RealEvidencePredictionRefreshRunner::default()
                            .run_model_ops_runbook(&config)?;
                        println!(
                            "copyable_commands_only=real modelops runbook emits copyable commands only and never training, live inference, or execution buttons"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::RustToolchainModernize { config } => {
            if config.contains("://") {
                Err("rust-toolchain-modernize config path must be local".to_string())
            } else {
                RustToolchainModernizationConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let bundle = RustToolchainModernizationRunner::default().run(&config)?;
                        println!(
                            "stable_only_warning=rust toolchain modernization is local-only, stable-toolchain-only, keeps the full workspace final gate, and never adds live trading, order/account paths, runtime LLM, Mamba runtime, or model training"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&bundle).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::ToolchainVersionReport { config } => {
            if config.contains("://") {
                Err("toolchain-version-report config path must be local".to_string())
            } else {
                RustToolchainModernizationConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = RustToolchainModernizationRunner::default()
                            .run_toolchain_version_report(&config)?;
                        println!(
                            "local_only_warning=toolchain version report is local-only, stable-only, and never changes runtime or live behavior"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::CargoWorkspaceAudit { config } => {
            if config.contains("://") {
                Err("cargo-workspace-audit config path must be local".to_string())
            } else {
                RustToolchainModernizationConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report =
                            RustToolchainModernizationRunner::default().run_cargo_workspace_audit(&config)?;
                        println!(
                            "no_runtime_feature_changes=cargo workspace audit is diagnostic-only and does not add broker/order/account paths, runtime LLM, Mamba runtime, or training"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::TestTierPlan { config } => {
            if config.contains("://") {
                Err("test-tier-plan config path must be local".to_string())
            } else {
                RustToolchainModernizationConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = RustToolchainModernizationRunner::default().run_test_tier_plan(&config)?;
                        println!(
                            "full_gate_preserved=test tier planning only shortens iteration loops; full workspace acceptance remains the final ship gate"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::TestRuntimeBudget { config } => {
            if config.contains("://") {
                Err("test-runtime-budget config path must be local".to_string())
            } else {
                RustToolchainModernizationConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report =
                            RustToolchainModernizationRunner::default().run_test_runtime_budget(&config)?;
                        println!(
                            "timing_diagnostic_warning=test runtime budget is diagnostic timing guidance only and never weakens final workspace acceptance"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::SlowTestInventory { config } => {
            if config.contains("://") {
                Err("slow-test-inventory config path must be local".to_string())
            } else {
                RustToolchainModernizationConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report =
                            RustToolchainModernizationRunner::default().run_slow_test_inventory(&config)?;
                        println!(
                            "no_test_deletion_warning=slow test inventory classifies expensive paths but never deletes safety tests"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::CliSmokeTiering { config } => {
            if config.contains("://") {
                Err("cli-smoke-tiering config path must be local".to_string())
            } else {
                RustToolchainModernizationConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report =
                            RustToolchainModernizationRunner::default().run_cli_smoke_tiering(&config)?;
                        println!(
                            "safety_smoke_retained=cli smoke tiering keeps safety help checks visible while reducing repetitive exhaustive reruns"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::DeveloperSpeedRunbook { config } => {
            if config.contains("://") {
                Err("developer-speed-runbook config path must be local".to_string())
            } else {
                RustToolchainModernizationConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = RustToolchainModernizationRunner::default()
                            .run_developer_speed_runbook(&config)?;
                        println!(
                            "copyable_commands_only=developer speed runbook emits copyable local commands only and never adds live/runtime features"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::WorkspaceAcceptanceV2 { config } => {
            if config.contains("://") {
                Err("workspace-acceptance-v2 config path must be local".to_string())
            } else {
                RustToolchainModernizationConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = RustToolchainModernizationRunner::default()
                            .run_workspace_acceptance_v2(&config)?;
                        println!(
                            "full_workspace_final=workspace acceptance v2 allows tiered iteration but full workspace remains the final ship gate"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::RepeatedWorkspaceTiming { config } => {
            if config.contains("://") {
                Err("repeated-workspace-timing config path must be local".to_string())
            } else {
                RepeatedWorkspaceTimingConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report =
                            TestOptimizationRunner::default().run_repeated_workspace_timing(&config)?;
                        println!(
                            "no_fake_timing_warning=repeated workspace timing is local-only, sample-backed or opt-in real timing only, and never fakes timing data"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::TestBinaryCost { config } => {
            if config.contains("://") {
                Err("test-binary-cost config path must be local".to_string())
            } else {
                RepeatedWorkspaceTimingConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = TestOptimizationRunner::default().run_test_binary_cost(&config)?;
                        println!(
                            "diagnostic_only_warning=test binary cost reporting is diagnostic-only and never changes runtime behavior or safety coverage"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::FixtureSetupCost { config } => {
            if config.contains("://") {
                Err("fixture-setup-cost config path must be local".to_string())
            } else {
                RepeatedWorkspaceTimingConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report =
                            TestOptimizationRunner::default().run_fixture_setup_cost(&config)?;
                        println!(
                            "no_semantic_changes=fixture setup cost reporting analyzes repeated setup/loading only and does not change fixture semantics"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::ArtifactRenderCost { config } => {
            if config.contains("://") {
                Err("artifact-render-cost config path must be local".to_string())
            } else {
                RepeatedWorkspaceTimingConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report =
                            TestOptimizationRunner::default().run_artifact_render_cost(&config)?;
                        println!(
                            "deterministic_only_warning=artifact render cost analysis stays deterministic and never hides render failures"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::CliSmokeCostReduce { config } => {
            if config.contains("://") {
                Err("cli-smoke-cost-reduce config path must be local".to_string())
            } else {
                CliSmokeCostReductionConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let report = soma_zero::build_cli_smoke_cost_reduction_report(&config, None);
                        println!(
                            "safety_smoke_retained=cli smoke reduction keeps required safety/help smoke visible and never hides a command family"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    },
                )
            }
        }
        Commands::FixtureDedupPlan { config } => {
            if config.contains("://") {
                Err("fixture-dedup-plan config path must be local".to_string())
            } else {
                RepeatedWorkspaceTimingConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report =
                            TestOptimizationRunner::default().run_fixture_dedup_plan(&config)?;
                        println!(
                            "no_test_deletion=fixture dedup planning only targets setup/loading reuse and never deletes tests"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::FixtureCachePlan { config } => {
            if config.contains("://") {
                Err("fixture-cache-plan config path must be local".to_string())
            } else {
                RepeatedWorkspaceTimingConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report =
                            TestOptimizationRunner::default().run_fixture_cache_plan(&config)?;
                        println!(
                            "no_secret_cache=fixture cache planning is local-only, deterministic, and must not store secrets"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::ArtifactRenderCachePlan { config } => {
            if config.contains("://") {
                Err("artifact-render-cache-plan config path must be local".to_string())
            } else {
                RepeatedWorkspaceTimingConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = TestOptimizationRunner::default()
                            .run_artifact_render_cache_plan(&config)?;
                        println!(
                            "no_hidden_failures=artifact render cache planning requires deterministic fingerprints and must never hide render failures"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::TestSupportRefactorPlan { config } => {
            if config.contains("://") {
                Err("test-support-refactor-plan config path must be local".to_string())
            } else {
                RepeatedWorkspaceTimingConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = TestOptimizationRunner::default()
                            .run_test_support_refactor_plan(&config)?;
                        println!(
                            "manual_review_warning=test support refactor planning stays conservative and requires manual review for behavior-sensitive followups"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::DevLoopSavingsEstimate { config } => {
            if config.contains("://") {
                Err("dev-loop-savings-estimate config path must be local".to_string())
            } else {
                RepeatedWorkspaceTimingConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = TestOptimizationRunner::default()
                            .run_dev_loop_savings_estimate(&config)?;
                        println!(
                            "estimate_only_warning=dev loop savings are labeled estimates and never guaranteed speedups without measured data"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::WorkspaceAcceptanceV3 { config } => {
            if config.contains("://") {
                Err("workspace-acceptance-v3 config path must be local".to_string())
            } else {
                RepeatedWorkspaceTimingConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report =
                            TestOptimizationRunner::default().run_workspace_acceptance_v3(&config)?;
                        println!(
                            "full_gate_preserved=workspace acceptance v3 keeps full workspace test as the final ship gate"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::CoreCompletionV2 { config } => {
            if config.contains("://") {
                Err("core-completion-v2 config path must be local".to_string())
            } else {
                CoreCompletionV2Config::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let report = CoreCommitteeMambaReadinessRunner::default()
                            .run_core_completion_v2(&config)?;
                        println!(
                            "research_only_warning=core completion v2 is research-only, paper-only, local-only, and never implies live trading readiness"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    },
                )
            }
        }
        Commands::Mamba3finCoreContract { config } => {
            if config.contains("://") {
                Err("mamba3fin-core-contract config path must be local".to_string())
            } else {
                CoreCompletionV2Config::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let report = CoreCommitteeMambaReadinessRunner::default()
                            .run_mamba3fin_core_contract(&config)?;
                        println!(
                            "contract_only_warning=mamba3fin core contract is contract-only, local-only, and does not implement runtime or training"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    },
                )
            }
        }
        Commands::Mamba3RuntimeReadiness { config } => {
            if config.contains("://") {
                Err("mamba3-runtime-readiness config path must be local".to_string())
            } else {
                CoreCompletionV2Config::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let report = CoreCommitteeMambaReadinessRunner::default()
                            .run_mamba3_runtime_readiness(&config)?;
                        println!(
                            "runtime_deferred_warning=mamba3 runtime readiness is a local-only gate and runtime remains deferred in this sprint"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    },
                )
            }
        }
        Commands::CommitteeCompletionGate { config } => {
            if config.contains("://") {
                Err("committee-completion-gate config path must be local".to_string())
            } else {
                CommitteeCompletionGateConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = CoreCommitteeMambaReadinessRunner::default()
                            .run_committee_completion_gate(&config)?;
                        println!(
                            "no_expansion_warning=committee completion gate is research-only, keeps exactly three active personas, and allows no expansion"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::CommitteeMaterializationPlanV2 { config } => {
            if config.contains("://") {
                Err("committee-materialization-plan-v2 config path must be local".to_string())
            } else {
                CommitteeCompletionGateConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = CoreCommitteeMambaReadinessRunner::default()
                            .run_committee_materialization_plan_v2(&config)?;
                        println!(
                            "materialization_only_warning=committee materialization plan v2 is local-only planning and never expands active personas"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::TrainingDataStorageDecision { config } => {
            if config.contains("://") {
                Err("training-data-storage-decision config path must be local".to_string())
            } else {
                TrainingDataStorageConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let report = CoreCommitteeMambaReadinessRunner::default()
                            .run_training_data_storage_decision(&config)?;
                        println!(
                            "no_training_warning=training data storage decision freezes storage format only and does not perform model training"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    },
                )
            }
        }
        Commands::TrainingDataRegistrySpec { config } => {
            if config.contains("://") {
                Err("training-data-registry-spec config path must be local".to_string())
            } else {
                TrainingDataStorageConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let report = CoreCommitteeMambaReadinessRunner::default()
                            .run_training_data_registry_spec(&config)?;
                        println!(
                            "storage_spec_only=training data registry spec is storage spec only and does not imply runtime or training"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    },
                )
            }
        }
        Commands::TrainingDataLayoutPlan { config } => {
            if config.contains("://") {
                Err("training-data-layout-plan config path must be local".to_string())
            } else {
                TrainingDataStorageConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let report = CoreCommitteeMambaReadinessRunner::default()
                            .run_training_data_layout_plan(&config)?;
                        println!(
                            "storage_layout_only=training data layout plan is a local-only storage contract with no runtime behavior"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    },
                )
            }
        }
        Commands::TrainingDataLineageSpec { config } => {
            if config.contains("://") {
                Err("training-data-lineage-spec config path must be local".to_string())
            } else {
                TrainingDataStorageConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let report = CoreCommitteeMambaReadinessRunner::default()
                            .run_training_data_lineage_spec(&config)?;
                        println!(
                            "lineage_only_warning=training data lineage spec is deterministic storage contract only and never performs training"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    },
                )
            }
        }
        Commands::Mamba3ImplementationRoadmap { config } => {
            if config.contains("://") {
                Err("mamba3-implementation-roadmap config path must be local".to_string())
            } else {
                CoreCompletionV2Config::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let report = CoreCommitteeMambaReadinessRunner::default()
                            .run_mamba3_implementation_roadmap(&config)?;
                        println!(
                            "staged_deferred_warning=mamba3 implementation roadmap is staged/deferred only and does not implement runtime or training"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    },
                )
            }
        }
        Commands::SequenceCoreRegistry { config } => {
            if config.contains("://") {
                Err("sequence-core-registry config path must be local".to_string())
            } else {
                SequenceCoreCandidateRegistryConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = SequenceCoreStorageMaterializationRunner::default()
                            .run_sequence_core_registry(&config)?;
                        println!(
                            "contract_only_warning=sequence core registry is contract-only, research-only, and never implies runtime or training"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::GatedDeltanetCoreContract { config } => {
            if config.contains("://") {
                Err("gated-deltanet-core-contract config path must be local".to_string())
            } else {
                SequenceCoreCandidateRegistryConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = SequenceCoreStorageMaterializationRunner::default()
                            .run_gated_deltanet_core_contract(&config)?;
                        println!(
                            "runtime_deferred_warning=gated deltanet core contract is contract-only and runtime remains deferred"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::GatedDeltanetReadiness { config } => {
            if config.contains("://") {
                Err("gated-deltanet-readiness config path must be local".to_string())
            } else {
                SequenceCoreCandidateRegistryConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = SequenceCoreStorageMaterializationRunner::default()
                            .run_gated_deltanet_readiness(&config)?;
                        println!(
                            "no_runtime_warning=gated deltanet readiness is prerequisite-only and does not implement runtime"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::SequenceCoreComparisonPlan { config } => {
            if config.contains("://") {
                Err("sequence-core-comparison-plan config path must be local".to_string())
            } else {
                SequenceCoreCandidateRegistryConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = SequenceCoreStorageMaterializationRunner::default()
                            .run_sequence_core_comparison_plan(&config)?;
                        println!(
                            "offline_only_warning=sequence core comparison plan is offline comparison only and never implies runtime or training"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::SequenceCoreExternalContract { config } => {
            if config.contains("://") {
                Err("sequence-core-external-contract config path must be local".to_string())
            } else {
                SequenceCoreCandidateRegistryConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = SequenceCoreStorageMaterializationRunner::default()
                            .run_sequence_core_external_contract(&config)?;
                        println!(
                            "prediction_csv_only_warning=sequence core external contract is prediction CSV only and external research only"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::TrainingStorageMaterialize { config } => {
            if config.contains("://") {
                Err("training-storage-materialize config path must be local".to_string())
            } else {
                TrainingDataStorageMaterializationConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = SequenceCoreStorageMaterializationRunner::default()
                            .run_training_storage_materialize(&config)?;
                        println!(
                            "no_fake_data_warning=training storage materialize writes explicit placeholders only and never fakes data availability"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::TrainingStorageIntegrity { config } => {
            if config.contains("://") {
                Err("training-storage-integrity config path must be local".to_string())
            } else {
                TrainingDataStorageMaterializationConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = SequenceCoreStorageMaterializationRunner::default()
                            .run_training_storage_integrity(&config)?;
                        println!(
                            "manifest_check_warning=training storage integrity performs manifest checks only on local artifacts"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::ModelFamilyStorageContract { config } => {
            if config.contains("://") {
                Err("model-family-storage-contract config path must be local".to_string())
            } else {
                TrainingDataStorageMaterializationConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = SequenceCoreStorageMaterializationRunner::default()
                            .run_model_family_storage_contract(&config)?;
                        println!(
                            "no_runtime_training_warning=model family storage contract defines artifact policy only and never enables runtime or training"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::ControlTowerSequenceCore { config } => {
            if config.contains("://") {
                Err("control-tower-sequence-core config path must be local".to_string())
            } else {
                SequenceCoreCandidateRegistryConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = SequenceCoreStorageMaterializationRunner::default()
                            .run_control_tower_sequence_core(&config)?;
                        println!(
                            "read_only_warning=control tower sequence core is read-only status output with no command execution"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::SequenceCorePrototypeCompare { config } => {
            if config.contains("://") {
                Err("sequence-core-prototype-compare config path must be local".to_string())
            } else {
                SequenceCorePrototypeComparisonConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let bundle = SequenceCorePrototypeComparisonRunner::default().run(&config)?;
                        println!(
                            "prototype_only_warning=sequence core prototype compare is offline, paper-only, prediction-csv-only, and never runtime or training"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&bundle)
                                .map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::SequenceCorePrototypeRegistry { config } => {
            if config.contains("://") {
                Err("sequence-core-prototype-registry config path must be local".to_string())
            } else {
                SequenceCorePrototypeComparisonConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = SequenceCorePrototypeComparisonRunner::default()
                            .run_sequence_core_prototype_registry(&config)?;
                        println!(
                            "prototype_registry_warning=sequence core prototype registry is artifact-only and never implies runtime or training"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report)
                                .map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::Mamba3finPrototypeImport { config } => {
            if config.contains("://") {
                Err("mamba3fin-prototype-import config path must be local".to_string())
            } else {
                SequenceCorePrototypeComparisonConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = SequenceCorePrototypeComparisonRunner::default()
                            .run_mamba3fin_prototype_import(&config)?;
                        println!(
                            "prediction_csv_only_warning=mamba3fin prototype import validates prediction csv artifacts only"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report)
                                .map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::GatedDeltanetPrototypeImport { config } => {
            if config.contains("://") {
                Err("gated-deltanet-prototype-import config path must be local".to_string())
            } else {
                SequenceCorePrototypeComparisonConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = SequenceCorePrototypeComparisonRunner::default()
                            .run_gated_deltanet_prototype_import(&config)?;
                        println!(
                            "prediction_csv_only_warning=gated deltanet prototype import validates prediction csv artifacts only"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report)
                                .map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::SequenceCorePrototypeEvaluate { config } => {
            if config.contains("://") {
                Err("sequence-core-prototype-evaluate config path must be local".to_string())
            } else {
                SequenceCorePrototypeComparisonConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = SequenceCorePrototypeComparisonRunner::default()
                            .run_sequence_core_prototype_evaluate(&config)?;
                        println!(
                            "offline_only_warning=sequence core prototype evaluate is offline-only and diagnostic-only"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report)
                                .map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::CommitteeEvidenceExpandV2 { config } => {
            if config.contains("://") {
                Err("committee-evidence-expand-v2 config path must be local".to_string())
            } else {
                CommitteeEvidenceExpansionConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = SequenceCorePrototypeComparisonRunner::default()
                            .run_committee_evidence_expand_v2(&config)?;
                        println!(
                            "no_expansion_warning=committee evidence expand v2 keeps Trinity-only and never activates 6/12 personas"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report)
                                .map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::CommitteeVsSequenceCore { config } => {
            if config.contains("://") {
                Err("committee-vs-sequence-core config path must be local".to_string())
            } else {
                SequenceCorePrototypeComparisonConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = SequenceCorePrototypeComparisonRunner::default()
                            .run_committee_vs_sequence_core(&config)?;
                        println!(
                            "diagnostic_warning=committee vs sequence core remains diagnostic-only and never implies deployment"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report)
                                .map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::TrainingArtifactPopulate { config } => {
            if config.contains("://") {
                Err("training-artifact-populate config path must be local".to_string())
            } else {
                TrainingDataArtifactPopulationConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = SequenceCorePrototypeComparisonRunner::default()
                            .run_training_artifact_populate(&config)?;
                        println!(
                            "no_fake_data_warning=training artifact populate writes local references only and never fakes data availability"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report)
                                .map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::TrainingPopulatedIntegrity { config } => {
            if config.contains("://") {
                Err("training-populated-integrity config path must be local".to_string())
            } else {
                TrainingDataArtifactPopulationConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = SequenceCorePrototypeComparisonRunner::default()
                            .run_training_populated_integrity(&config)?;
                        println!(
                            "manifest_check_warning=training populated integrity validates local references and secret safety only"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report)
                                .map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::ControlTowerSequencePrototype { config } => {
            if config.contains("://") {
                Err("control-tower-sequence-prototype config path must be local".to_string())
            } else {
                SequenceCorePrototypeComparisonConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = SequenceCorePrototypeComparisonRunner::default()
                            .run_control_tower_sequence_prototype(&config)?;
                        println!(
                            "read_only_warning=control tower sequence prototype is read-only status output with no command execution"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report)
                                .map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::PrototypeInterpretation { config } => {
            if config.contains("://") {
                Err("prototype-interpretation config path must be local".to_string())
            } else {
                PrototypeComparisonInterpretationConfig::from_toml_path(std::path::Path::new(
                    &config,
                ))
                .and_then(|config| {
                    let report = PrototypeComparisonInterpretationRunner::default()
                        .run_prototype_interpretation(&config)?;
                    println!(
                        "interpretation_only_warning=prototype interpretation is research-only, paper-only, diagnostic-only, and never runtime selection"
                    );
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                    );
                    Ok(())
                })
            }
        }
        Commands::PrototypeConfidence { config } => {
            if config.contains("://") {
                Err("prototype-confidence config path must be local".to_string())
            } else {
                PrototypeComparisonInterpretationConfig::from_toml_path(std::path::Path::new(
                    &config,
                ))
                .and_then(|config| {
                    let report =
                        PrototypeComparisonInterpretationRunner::default()
                            .run_prototype_confidence(&config)?;
                    println!(
                        "diagnostic_warning=prototype confidence is source-weighted diagnostic output only"
                    );
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                    );
                    Ok(())
                })
            }
        }
        Commands::PrototypeWinnerGate { config } => {
            if config.contains("://") {
                Err("prototype-winner-gate config path must be local".to_string())
            } else {
                PrototypeComparisonInterpretationConfig::from_toml_path(std::path::Path::new(
                    &config,
                ))
                .and_then(|config| {
                    let report = PrototypeComparisonInterpretationRunner::default()
                        .run_prototype_winner_gate(&config)?;
                    println!(
                        "no_runtime_selection_warning=prototype winner gate is diagnostic-only and cannot select runtime"
                    );
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                    );
                    Ok(())
                })
            }
        }
        Commands::PrototypeDisagreement { config } => {
            if config.contains("://") {
                Err("prototype-disagreement config path must be local".to_string())
            } else {
                PrototypeComparisonInterpretationConfig::from_toml_path(std::path::Path::new(
                    &config,
                ))
                .and_then(|config| {
                    let report = PrototypeComparisonInterpretationRunner::default()
                        .run_prototype_disagreement(&config)?;
                    println!(
                        "offline_only_warning=prototype disagreement is offline-only diagnostic review with no runtime implication"
                    );
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                    );
                    Ok(())
                })
            }
        }
        Commands::PrototypeFailureModes { config } => {
            if config.contains("://") {
                Err("prototype-failure-modes config path must be local".to_string())
            } else {
                PrototypeComparisonInterpretationConfig::from_toml_path(std::path::Path::new(
                    &config,
                ))
                .and_then(|config| {
                    let report = PrototypeComparisonInterpretationRunner::default()
                        .run_prototype_failure_modes(&config)?;
                    println!(
                        "diagnostic_warning=prototype failure modes are research-only diagnostics and never deployment approval"
                    );
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                    );
                    Ok(())
                })
            }
        }
        Commands::PrototypeCalibrationRisk { config } => {
            if config.contains("://") {
                Err("prototype-calibration-risk config path must be local".to_string())
            } else {
                PrototypeComparisonInterpretationConfig::from_toml_path(std::path::Path::new(
                    &config,
                ))
                .and_then(|config| {
                    let report = PrototypeComparisonInterpretationRunner::default()
                        .run_prototype_calibration_risk(&config)?;
                    println!(
                        "diagnostic_warning=prototype calibration risk synthesis is interpretation-only and no runtime exists"
                    );
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                    );
                    Ok(())
                })
            }
        }
        Commands::NoTradeRiskDeniedInterpretation { config } => {
            if config.contains("://") {
                Err("no-trade-risk-denied-interpretation config path must be local".to_string())
            } else {
                PrototypeComparisonInterpretationConfig::from_toml_path(std::path::Path::new(
                    &config,
                ))
                .and_then(|config| {
                    let report = PrototypeComparisonInterpretationRunner::default()
                        .run_no_trade_risk_denied_interpretation(&config)?;
                    println!(
                        "defensive_axes_warning=no-trade and risk-denied interpretation remains diagnostic-only and defensive-first"
                    );
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                    );
                    Ok(())
                })
            }
        }
        Commands::CommitteeReferenceAuditV2 { config } => {
            if config.contains("://") {
                Err("committee-reference-audit-v2 config path must be local".to_string())
            } else {
                PrototypeComparisonInterpretationConfig::from_toml_path(std::path::Path::new(
                    &config,
                ))
                .and_then(|config| {
                    let report = PrototypeComparisonInterpretationRunner::default()
                        .run_committee_reference_audit_v2(&config)?;
                    println!(
                        "trinity_only_warning=committee reference audit v2 is Trinity-only and does not activate 6/12 personas"
                    );
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                    );
                    Ok(())
                })
            }
        }
        Commands::CommitteeReferenceDepthPlan { config } => {
            if config.contains("://") {
                Err("committee-reference-depth-plan config path must be local".to_string())
            } else {
                PrototypeComparisonInterpretationConfig::from_toml_path(std::path::Path::new(
                    &config,
                ))
                .and_then(|config| {
                    let report = PrototypeComparisonInterpretationRunner::default()
                        .run_committee_reference_depth_plan(&config)?;
                    println!(
                        "depth_plan_warning=committee reference depth plan is research-only closure planning with Trinity fixed"
                    );
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                    );
                    Ok(())
                })
            }
        }
        Commands::CommitteeSequenceDisagreement { config } => {
            if config.contains("://") {
                Err("committee-sequence-disagreement config path must be local".to_string())
            } else {
                PrototypeComparisonInterpretationConfig::from_toml_path(std::path::Path::new(
                    &config,
                ))
                .and_then(|config| {
                    let report = PrototypeComparisonInterpretationRunner::default()
                        .run_committee_sequence_disagreement(&config)?;
                    println!(
                        "offline_only_warning=committee sequence disagreement is offline-only and diagnostic-only"
                    );
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                    );
                    Ok(())
                })
            }
        }
        Commands::SequenceCoreDecisionGate { config } => {
            if config.contains("://") {
                Err("sequence-core-decision-gate config path must be local".to_string())
            } else {
                PrototypeComparisonInterpretationConfig::from_toml_path(std::path::Path::new(
                    &config,
                ))
                .and_then(|config| {
                    let report = PrototypeComparisonInterpretationRunner::default()
                        .run_sequence_core_decision_gate(&config)?;
                    println!(
                        "runtime_deferred_warning=sequence core decision gate keeps runtime deferred and never selects training or live inference"
                    );
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                    );
                    Ok(())
                })
            }
        }
        Commands::TrainingLineageCompleteness { config } => {
            if config.contains("://") {
                Err("training-lineage-completeness config path must be local".to_string())
            } else {
                PrototypeComparisonInterpretationConfig::from_toml_path(std::path::Path::new(
                    &config,
                ))
                .and_then(|config| {
                    let report = PrototypeComparisonInterpretationRunner::default()
                        .run_training_lineage_completeness(&config)?;
                    println!(
                        "lineage_only_warning=training lineage completeness is provenance-only and does not imply model training"
                    );
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                    );
                    Ok(())
                })
            }
        }
        Commands::ControlTowerPrototypeInterpretation { config } => {
            if config.contains("://") {
                Err("control-tower-prototype-interpretation config path must be local".to_string())
            } else {
                PrototypeComparisonInterpretationConfig::from_toml_path(std::path::Path::new(
                    &config,
                ))
                .and_then(|config| {
                    let report = PrototypeComparisonInterpretationRunner::default()
                        .run_control_tower_prototype_interpretation(&config)?;
                    println!(
                        "read_only_warning=control tower prototype interpretation is read-only static status output"
                    );
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                    );
                    Ok(())
                })
            }
        }
        Commands::OfficialEvidenceDepthExpand { config } => {
            if config.contains("://") {
                Err("official-evidence-depth-expand config path must be local".to_string())
            } else {
                OfficialEvidenceDepthExpansionConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = OfficialEvidenceDepthExpansionRunner::default()
                            .run_official_evidence_depth_expand(&config)?;
                        println!(
                            "research_only_warning=official evidence depth expansion is offline-only, local-only, paper-only, and never runtime or training"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::CommitteeReferenceClose { config } => {
            if config.contains("://") {
                Err("committee-reference-close config path must be local".to_string())
            } else {
                CommitteeReferenceClosureConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = CommitteeReferenceClosureRunner::default().run(&config)?;
                        println!(
                            "trinity_only_warning=committee reference closure is Trinity-only, official-reference-only, and never persona expansion"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::OfficialScenarioPackV3 { config } => {
            if config.contains("://") {
                Err("official-scenario-pack-v3 config path must be local".to_string())
            } else {
                OfficialEvidenceDepthExpansionConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = OfficialEvidenceDepthExpansionRunner::default()
                            .run_official_scenario_pack_v3(&config)?;
                        println!(
                            "local_pack_warning=official scenario pack v3 is local-only evidence packaging with no runtime implication"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::OfficialOutcomePackV3 { config } => {
            if config.contains("://") {
                Err("official-outcome-pack-v3 config path must be local".to_string())
            } else {
                OfficialEvidenceDepthExpansionConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = OfficialEvidenceDepthExpansionRunner::default()
                            .run_official_outcome_pack_v3(&config)?;
                        println!(
                            "no_lookahead_warning=official outcome pack v3 is local-only and bounded by no-lookahead checks"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::OfficialBaselinePackV3 { config } => {
            if config.contains("://") {
                Err("official-baseline-pack-v3 config path must be local".to_string())
            } else {
                OfficialEvidenceDepthExpansionConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = OfficialEvidenceDepthExpansionRunner::default()
                            .run_official_baseline_pack_v3(&config)?;
                        println!(
                            "defensive_baseline_warning=official baseline pack v3 keeps Trinity, NoTrade, and RiskDenied as defensive-first references"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::OfficialNotradePackV3 { config } => {
            if config.contains("://") {
                Err("official-notrade-pack-v3 config path must be local".to_string())
            } else {
                OfficialEvidenceDepthExpansionConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = OfficialEvidenceDepthExpansionRunner::default()
                            .run_official_notrade_pack_v3(&config)?;
                        println!(
                            "defensive_axes_warning=official no-trade pack v3 is defensive-only interpretation and never runtime selection"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::OfficialRiskdeniedPackV3 { config } => {
            if config.contains("://") {
                Err("official-riskdenied-pack-v3 config path must be local".to_string())
            } else {
                OfficialEvidenceDepthExpansionConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = OfficialEvidenceDepthExpansionRunner::default()
                            .run_official_riskdenied_pack_v3(&config)?;
                        println!(
                            "defensive_axes_warning=official risk-denied pack v3 is defensive-only interpretation and never runtime selection"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::DefensiveCounterfactualDepth { config } => {
            if config.contains("://") {
                Err("defensive-counterfactual-depth config path must be local".to_string())
            } else {
                OfficialEvidenceDepthExpansionConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = OfficialEvidenceDepthExpansionRunner::default()
                            .run_defensive_counterfactual_depth(&config)?;
                        println!(
                            "diagnostic_warning=defensive counterfactual depth is research-only defensive analysis with no runtime implication"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::OfficialReferenceQuality { config } => {
            if config.contains("://") {
                Err("official-reference-quality config path must be local".to_string())
            } else {
                OfficialEvidenceDepthExpansionConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = OfficialEvidenceDepthExpansionRunner::default()
                            .run_official_reference_quality(&config)?;
                        println!(
                            "quality_only_warning=official reference quality is provenance/completeness diagnostics only"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::OfficialReferenceDiversity { config } => {
            if config.contains("://") {
                Err("official-reference-diversity config path must be local".to_string())
            } else {
                OfficialEvidenceDepthExpansionConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = OfficialEvidenceDepthExpansionRunner::default()
                            .run_official_reference_diversity(&config)?;
                        println!(
                            "diversity_only_warning=official reference diversity is coverage diagnostics only"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::OfficialReferenceNoLookahead { config } => {
            if config.contains("://") {
                Err("official-reference-no-lookahead config path must be local".to_string())
            } else {
                OfficialEvidenceDepthExpansionConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = OfficialEvidenceDepthExpansionRunner::default()
                            .run_official_reference_no_lookahead(&config)?;
                        println!(
                            "audit_warning=official no-lookahead audit is leakage guard output only"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::OfficialReferenceSourceBoundary { config } => {
            if config.contains("://") {
                Err("official-reference-source-boundary config path must be local".to_string())
            } else {
                OfficialEvidenceDepthExpansionConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = OfficialEvidenceDepthExpansionRunner::default()
                            .run_official_reference_source_boundary(&config)?;
                        println!(
                            "audit_warning=official source-boundary audit is promotion-guard output only"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::SequenceCoreConfidenceRerun { config } => {
            if config.contains("://") {
                Err("sequence-core-confidence-rerun config path must be local".to_string())
            } else {
                OfficialEvidenceDepthExpansionConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = OfficialEvidenceDepthExpansionRunner::default()
                            .run_sequence_core_confidence_rerun(&config)?;
                        println!(
                            "diagnostic_warning=sequence core confidence rerun is evidence-depth-only diagnostic output"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::SequenceCoreDecisionGateV2 { config } => {
            if config.contains("://") {
                Err("sequence-core-decision-gate-v2 config path must be local".to_string())
            } else {
                OfficialEvidenceDepthExpansionConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = OfficialEvidenceDepthExpansionRunner::default()
                            .run_sequence_core_decision_gate_v2(&config)?;
                        println!(
                            "runtime_deferred_warning=decision gate v2 remains research-only and keeps runtime, training, and live inference disabled"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::ControlTowerEvidenceDepth { config } => {
            if config.contains("://") {
                Err("control-tower-evidence-depth config path must be local".to_string())
            } else {
                OfficialEvidenceDepthExpansionConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = OfficialEvidenceDepthExpansionRunner::default()
                            .run_control_tower_evidence_depth(&config)?;
                        println!(
                            "read_only_warning=control tower evidence depth is read-only static status output with no runtime implication"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::Sprint83AcceptanceRecovery { config } => {
            if config.contains("://") {
                    Err("sprint83-acceptance-recovery config path must be local".to_string())
            } else {
                    Sprint83AcceptanceRecoveryConfig::from_toml_path(std::path::Path::new(&config))
                        .and_then(|config| {
                            let report = Sprint83AcceptanceRecoveryRunner::default()
                                .run_sprint83_acceptance_recovery(&config)?;
                            println!(
                                "acceptance_recovery_warning=sprint83 acceptance recovery is research-only, paper-only, local-only, and never runtime or training"
                            );
                            println!(
                                "{}",
                                serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                            );
                            Ok(())
                        })
            }
        }
        Commands::FullWorkspaceAcceptanceRecovery { config } => {
            if config.contains("://") {
                    Err("full-workspace-acceptance-recovery config path must be local".to_string())
            } else {
                    Sprint83AcceptanceRecoveryConfig::from_toml_path(std::path::Path::new(&config))
                        .and_then(|config| {
                            let report = Sprint83AcceptanceRecoveryRunner::default()
                                .run_full_workspace_acceptance_recovery(&config)?;
                            println!(
                                "honest_status_warning=full workspace acceptance recovery reports pass, fail, or blocked state honestly"
                            );
                            println!(
                                "{}",
                                serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                            );
                            Ok(())
                        })
            }
        }
        Commands::LongCompilationDiagnosis { config } => {
            if config.contains("://") {
                    Err("long-compilation-diagnosis config path must be local".to_string())
            } else {
                    Sprint83AcceptanceRecoveryConfig::from_toml_path(std::path::Path::new(&config))
                        .and_then(|config| {
                            let report = Sprint83AcceptanceRecoveryRunner::default()
                                .run_long_compilation_diagnosis(&config)?;
                            println!(
                                "diagnostic_warning=long compilation diagnosis is diagnostic-only and does not fake timing or pass/fail"
                            );
                            println!(
                                "{}",
                                serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                            );
                            Ok(())
                        })
            }
        }
        Commands::EvidenceDepthFixtureAudit { config } => {
            if config.contains("://") {
                    Err("evidence-depth-fixture-audit config path must be local".to_string())
            } else {
                    Sprint83AcceptanceRecoveryConfig::from_toml_path(std::path::Path::new(&config))
                        .and_then(|config| {
                            let report = Sprint83AcceptanceRecoveryRunner::default()
                                .run_evidence_depth_fixture_audit(&config)?;
                            println!(
                                "fixture_audit_warning=evidence-depth fixture audit is source-boundary and no-lookahead diagnostics only"
                            );
                            println!(
                                "{}",
                                serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                            );
                            Ok(())
                        })
            }
        }
        Commands::EvidenceDepthFixtureNormalize { config } => {
            if config.contains("://") {
                    Err("evidence-depth-fixture-normalize config path must be local".to_string())
            } else {
                    Sprint83AcceptanceRecoveryConfig::from_toml_path(std::path::Path::new(&config))
                        .and_then(|config| {
                            let report = Sprint83AcceptanceRecoveryRunner::default()
                                .run_evidence_depth_fixture_normalize(&config)?;
                            println!(
                                "fixture_normalization_warning=evidence-depth fixture normalization is deterministic-view diagnostics only"
                            );
                            println!(
                                "{}",
                                serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                            );
                            Ok(())
                        })
            }
        }
        Commands::EvidenceDepthFixtureCompleteness { config } => {
            if config.contains("://") {
                    Err("evidence-depth-fixture-completeness config path must be local".to_string())
            } else {
                    Sprint83AcceptanceRecoveryConfig::from_toml_path(std::path::Path::new(&config))
                        .and_then(|config| {
                            let report = Sprint83AcceptanceRecoveryRunner::default()
                                .run_evidence_depth_fixture_completeness(&config)?;
                            println!(
                                "fixture_completeness_warning=evidence-depth fixture completeness is conservative diagnostics only"
                            );
                            println!(
                                "{}",
                                serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                            );
                            Ok(())
                        })
            }
        }
        Commands::EvidenceDepthDeterminismRegression { config } => {
            if config.contains("://") {
                    Err("evidence-depth-determinism-regression config path must be local".to_string())
            } else {
                    Sprint83AcceptanceRecoveryConfig::from_toml_path(std::path::Path::new(&config))
                        .and_then(|config| {
                            let report = Sprint83AcceptanceRecoveryRunner::default()
                                .run_evidence_depth_determinism_regression(&config)?;
                            println!(
                                "determinism_warning=fixture determinism regression is repeatability diagnostics only"
                            );
                            println!(
                                "{}",
                                serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                            );
                            Ok(())
                        })
            }
        }
        Commands::Sprint82SmokeCompress { config } => {
            if config.contains("://") {
                    Err("sprint82-smoke-compress config path must be local".to_string())
            } else {
                    Sprint83AcceptanceRecoveryConfig::from_toml_path(std::path::Path::new(&config))
                        .and_then(|config| {
                            let report = Sprint83AcceptanceRecoveryRunner::default()
                                .run_sprint82_smoke_compress(&config)?;
                            println!(
                                "smoke_compression_warning=sprint82 smoke compression keeps safety smoke preserved and remains diagnostic-only"
                            );
                            println!(
                                "{}",
                                serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                            );
                            Ok(())
                        })
            }
        }
        Commands::FixtureBoundaryAuditV2 { config } => {
            if config.contains("://") {
                    Err("fixture-boundary-audit-v2 config path must be local".to_string())
            } else {
                    Sprint83AcceptanceRecoveryConfig::from_toml_path(std::path::Path::new(&config))
                        .and_then(|config| {
                            let report = Sprint83AcceptanceRecoveryRunner::default()
                                .run_fixture_boundary_audit_v2(&config)?;
                            println!(
                                "boundary_audit_warning=fixture boundary audit v2 is source-boundary and no-lookahead diagnostics only"
                            );
                            println!(
                                "{}",
                                serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                            );
                            Ok(())
                        })
            }
        }
        Commands::TestRuntimeRecoveryPlan { config } => {
            if config.contains("://") {
                    Err("test-runtime-recovery-plan config path must be local".to_string())
            } else {
                    Sprint83AcceptanceRecoveryConfig::from_toml_path(std::path::Path::new(&config))
                        .and_then(|config| {
                            let report = Sprint83AcceptanceRecoveryRunner::default()
                                .run_test_runtime_recovery_plan(&config)?;
                            println!(
                                "runtime_cost_warning=test runtime recovery plan is planning-only and preserves safety coverage"
                            );
                            println!(
                                "{}",
                                serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                            );
                            Ok(())
                        })
            }
        }
        Commands::WorkspaceAcceptanceRecoveryGate { config } => {
            if config.contains("://") {
                    Err("workspace-acceptance-recovery-gate config path must be local".to_string())
            } else {
                    Sprint83AcceptanceRecoveryConfig::from_toml_path(std::path::Path::new(&config))
                        .and_then(|config| {
                            let report = Sprint83AcceptanceRecoveryRunner::default()
                                .run_workspace_acceptance_recovery_gate(&config)?;
                            println!(
                                "ship_gate_warning=workspace acceptance recovery gate remains research-only and cannot enable runtime or training"
                            );
                            println!(
                                "{}",
                                serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                            );
                            Ok(())
                        })
            }
        }
        Commands::ControlTowerSprint83Recovery { config } => {
            if config.contains("://") {
                    Err("control-tower-sprint83-recovery config path must be local".to_string())
            } else {
                    Sprint83AcceptanceRecoveryConfig::from_toml_path(std::path::Path::new(&config))
                        .and_then(|config| {
                            let report = Sprint83AcceptanceRecoveryRunner::default()
                                .run_control_tower_sprint83_recovery(&config)?;
                            println!(
                                "read_only_warning=control tower sprint83 recovery is read-only static status output with no browser execution"
                            );
                            println!(
                                "{}",
                                serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                            );
                            Ok(())
                        })
            }
        }
        Commands::Sprint84TestCostReduce { config } => {
            if config.contains("://") {
                Err("sprint84-test-cost-reduce config path must be local".to_string())
            } else {
                TestBinaryConsolidationConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = Sprint84TestCostReductionRunner::default()
                            .run_sprint84_test_cost_reduce(&config)?;
                        println!(
                            "test_cost_warning=sprint84 test cost reduction is research-only, local-only, and never deletes safety coverage or enables runtime/training/live"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::TestBinaryConsolidate { config } => {
            if config.contains("://") {
                Err("test-binary-consolidate config path must be local".to_string())
            } else {
                TestBinaryConsolidationConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = Sprint84TestCostReductionRunner::default()
                            .run_test_binary_consolidate(&config)?;
                        println!(
                            "consolidation_warning=test binary consolidation preserves assertions, safety smoke, and determinism checks"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::SharedFixtureHarnessReport { config } => {
            if config.contains("://") {
                Err("shared-fixture-harness-report config path must be local".to_string())
            } else {
                TestBinaryConsolidationConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = Sprint84TestCostReductionRunner::default()
                            .run_shared_fixture_harness_report(&config)?;
                        println!(
                            "fixture_harness_warning=shared fixture harness remains deterministic and does not introduce semantic/runtime changes"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::RepresentativeSmokeHarness { config } => {
            if config.contains("://") {
                Err("representative-smoke-harness config path must be local".to_string())
            } else {
                TestBinaryConsolidationConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = Sprint84TestCostReductionRunner::default()
                            .run_representative_smoke_harness(&config)?;
                        println!(
                            "representative_smoke_warning=representative smoke is a faster loop only and keeps safety smoke retained"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::ExhaustiveSmokeManifest { config } => {
            if config.contains("://") {
                Err("exhaustive-smoke-manifest config path must be local".to_string())
            } else {
                TestBinaryConsolidationConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = Sprint84TestCostReductionRunner::default()
                            .run_exhaustive_smoke_manifest(&config)?;
                        println!(
                            "exhaustive_smoke_warning=exhaustive smoke is documented for full/release flows and never implies runtime readiness"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::SafetySmokeManifest { config } => {
            if config.contains("://") {
                Err("safety-smoke-manifest config path must be local".to_string())
            } else {
                TestBinaryConsolidationConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = Sprint84TestCostReductionRunner::default()
                            .run_safety_smoke_manifest(&config)?;
                        println!(
                            "safety_smoke_warning=safety smoke remains required and preserves help, remote-path rejection, and forbidden-command checks"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::CliSmokeExecutionPolicy { config } => {
            if config.contains("://") {
                Err("cli-smoke-execution-policy config path must be local".to_string())
            } else {
                TestBinaryConsolidationConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = Sprint84TestCostReductionRunner::default()
                            .run_cli_smoke_execution_policy(&config)?;
                        println!(
                            "smoke_policy_warning=CLI smoke policy separates quick, sprint, full, release, and safety tiers without deleting safety coverage"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::TestRuntimeBeforeAfter { config } => {
            if config.contains("://") {
                Err("test-runtime-before-after config path must be local".to_string())
            } else {
                TestBinaryConsolidationConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = Sprint84TestCostReductionRunner::default()
                            .run_test_runtime_before_after(&config)?;
                        println!(
                            "runtime_before_after_warning=test runtime before/after stays measured-or-sample-backed only and never fakes timing"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::WorkspaceFinalGateV2 { config } => {
            if config.contains("://") {
                Err("workspace-final-gate-v2 config path must be local".to_string())
            } else {
                TestBinaryConsolidationConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = Sprint84TestCostReductionRunner::default()
                            .run_workspace_final_gate_v2(&config)?;
                        println!(
                            "final_gate_warning=workspace final gate v2 never fakes pass/fail and focused suites do not replace full workspace acceptance"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::ControlTowerTestCost { config } => {
            if config.contains("://") {
                Err("control-tower-test-cost config path must be local".to_string())
            } else {
                TestBinaryConsolidationConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = Sprint84TestCostReductionRunner::default()
                            .run_control_tower_test_cost(&config)?;
                        println!(
                            "read_only_warning=control tower test cost is read-only static status output with no train/runtime/live/order/account/browser controls"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::Sprint85WorkspaceGateRecover { config } => {
            if config.contains("://") {
                Err("sprint85-workspace-gate-recover config path must be local".to_string())
            } else {
                WorkspaceWideTestSurfaceAuditConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = Sprint85WorkspaceGateRecoveryRunner::default()
                            .run_sprint85_workspace_gate_recover(&config)?;
                        println!(
                            "workspace_gate_warning=sprint85 workspace gate recovery is research-only, local-only, and never enables runtime/training/live"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::WorkspaceTestSurfaceAudit { config } => {
            if config.contains("://") {
                Err("workspace-test-surface-audit config path must be local".to_string())
            } else {
                WorkspaceWideTestSurfaceAuditConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = Sprint85WorkspaceGateRecoveryRunner::default()
                            .run_workspace_test_surface_audit(&config)?;
                        println!(
                            "audit_warning=workspace-wide test surface audit is local-only and does not replace the full workspace gate"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::RemainingTestBinaryInventory { config } => {
            if config.contains("://") {
                Err("remaining-test-binary-inventory config path must be local".to_string())
            } else {
                WorkspaceWideTestSurfaceAuditConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = Sprint85WorkspaceGateRecoveryRunner::default()
                            .run_remaining_test_binary_inventory(&config)?;
                        println!(
                            "inventory_warning=remaining binary inventory is deterministic and keeps keep-separate candidates explicit"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::DomainSuitePlan { config } => {
            if config.contains("://") {
                Err("domain-suite-plan config path must be local".to_string())
            } else {
                WorkspaceWideTestSurfaceAuditConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = Sprint85WorkspaceGateRecoveryRunner::default()
                            .run_domain_suite_plan(&config)?;
                        println!(
                            "domain_suite_warning=grouped domain suites preserve representative coverage but do not replace full workspace acceptance"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::SharedFixtureAdoption { config } => {
            if config.contains("://") {
                Err("shared-fixture-adoption config path must be local".to_string())
            } else {
                WorkspaceWideTestSurfaceAuditConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = Sprint85WorkspaceGateRecoveryRunner::default()
                            .run_shared_fixture_adoption(&config)?;
                        println!(
                            "fixture_adoption_warning=shared fixture adoption remains deterministic and does not introduce runtime/live behavior"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::WorkspaceSmokePolicyV2 { config } => {
            if config.contains("://") {
                Err("workspace-smoke-policy-v2 config path must be local".to_string())
            } else {
                WorkspaceWideTestSurfaceAuditConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = Sprint85WorkspaceGateRecoveryRunner::default()
                            .run_workspace_smoke_policy_v2(&config)?;
                        println!(
                            "smoke_policy_warning=workspace smoke policy keeps safety smoke mandatory and never implies runtime readiness"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::WorkspaceAcceptanceAttemptV3 { config } => {
            if config.contains("://") {
                Err("workspace-acceptance-attempt-v3 config path must be local".to_string())
            } else {
                WorkspaceWideTestSurfaceAuditConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = Sprint85WorkspaceGateRecoveryRunner::default()
                            .run_workspace_acceptance_attempt_v3(&config)?;
                        println!(
                            "acceptance_warning=workspace acceptance attempt v3 never fakes pass/fail and focused suites do not replace full workspace acceptance"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::FullGateRecoveryV3 { config } => {
            if config.contains("://") {
                Err("full-gate-recovery-v3 config path must be local".to_string())
            } else {
                WorkspaceWideTestSurfaceAuditConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = Sprint85WorkspaceGateRecoveryRunner::default()
                            .run_full_gate_recovery_v3(&config)?;
                        println!(
                            "gate_recovery_warning=full gate recovery v3 is read-only status with honest blocked/pass/fail reporting"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::WorkspaceBlockerDrilldown { config } => {
            if config.contains("://") {
                Err("workspace-blocker-drilldown config path must be local".to_string())
            } else {
                WorkspaceWideTestSurfaceAuditConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = Sprint85WorkspaceGateRecoveryRunner::default()
                            .run_workspace_blocker_drilldown(&config)?;
                        println!(
                            "blocker_drilldown_warning=blocker drilldown is diagnostic-only and does not claim full-workspace success"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::ControlTowerWorkspaceGateV2 { config } => {
            if config.contains("://") {
                Err("control-tower-workspace-gate-v2 config path must be local".to_string())
            } else {
                WorkspaceWideTestSurfaceAuditConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = Sprint85WorkspaceGateRecoveryRunner::default()
                            .run_control_tower_workspace_gate_v2(&config)?;
                        println!(
                            "read_only_warning=control tower workspace gate v2 is read-only static status output with no train/runtime/live/order/account/browser controls"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::Sprint86ResidualGateRecover { config } => {
            if config.contains("://") {
                Err("sprint86-residual-gate-recover config path must be local".to_string())
            } else {
                ResidualWorkspaceBinaryAuditConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = Sprint86ResidualGateRecoveryRunner::default()
                            .run_sprint86_residual_gate_recover(&config)?;
                        println!(
                            "residual_gate_warning=sprint86 residual gate recovery is research-only, local-only, and keeps compile-only distinct from full acceptance"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::ResidualBinaryAudit { config } => {
            if config.contains("://") {
                Err("residual-binary-audit config path must be local".to_string())
            } else {
                ResidualWorkspaceBinaryAuditConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = Sprint86ResidualGateRecoveryRunner::default()
                            .run_residual_binary_audit(&config)?;
                        println!(
                            "residual_audit_warning=residual binary audit is deterministic and does not delete tests or claim full workspace success"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::ResidualFamilyClassifier { config } => {
            if config.contains("://") {
                Err("residual-family-classifier config path must be local".to_string())
            } else {
                ResidualWorkspaceBinaryAuditConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = Sprint86ResidualGateRecoveryRunner::default()
                            .run_residual_family_classifier(&config)?;
                        println!(
                            "classifier_warning=residual family classification is deterministic and keeps unknowns explicit"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::ResidualConsolidationPlan { config } => {
            if config.contains("://") {
                Err("residual-consolidation-plan config path must be local".to_string())
            } else {
                ResidualWorkspaceBinaryAuditConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = Sprint86ResidualGateRecoveryRunner::default()
                            .run_residual_consolidation_plan(&config)?;
                        println!(
                            "consolidation_warning=residual consolidation preserves assertions, records keep-separate reasons, and does not replace the full workspace gate"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::LegacyIntegrationMigration { config } => {
            if config.contains("://") {
                Err("legacy-integration-migration config path must be local".to_string())
            } else {
                ResidualWorkspaceBinaryAuditConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = Sprint86ResidualGateRecoveryRunner::default()
                            .run_legacy_integration_migration(&config)?;
                        println!(
                            "migration_warning=legacy migration moves assertions conservatively and keeps high-risk files separate when needed"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::CompileOnlyWorkspaceAttempt { config } => {
            if config.contains("://") {
                Err("compile-only-workspace-attempt config path must be local".to_string())
            } else {
                ResidualWorkspaceBinaryAuditConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = Sprint86ResidualGateRecoveryRunner::default()
                            .run_compile_only_workspace_attempt(&config)?;
                        println!(
                            "compile_only_warning=compile-only workspace attempt is diagnostic only and never implies full acceptance"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::CargoTestNoRunGate { config } => {
            if config.contains("://") {
                Err("cargo-test-no-run-gate config path must be local".to_string())
            } else {
                ResidualWorkspaceBinaryAuditConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = Sprint86ResidualGateRecoveryRunner::default()
                            .run_cargo_test_no_run_gate(&config)?;
                        println!(
                            "no_run_warning=cargo test no-run gate is compile-only interpretation and not full execution"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::FullWorkspaceAttemptV4 { config } => {
            if config.contains("://") {
                Err("full-workspace-attempt-v4 config path must be local".to_string())
            } else {
                ResidualWorkspaceBinaryAuditConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = Sprint86ResidualGateRecoveryRunner::default()
                            .run_full_workspace_attempt_v4(&config)?;
                        println!(
                            "workspace_attempt_warning=full workspace attempt v4 never fakes pass/fail and compile-only never counts as full acceptance"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::FullGateRecoveryV4 { config } => {
            if config.contains("://") {
                Err("full-gate-recovery-v4 config path must be local".to_string())
            } else {
                ResidualWorkspaceBinaryAuditConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = Sprint86ResidualGateRecoveryRunner::default()
                            .run_full_gate_recovery_v4(&config)?;
                        println!(
                            "gate_recovery_warning=full gate recovery v4 keeps compile-only and full-workspace truth separate"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::ResidualBlockerDrilldownV2 { config } => {
            if config.contains("://") {
                Err("residual-blocker-drilldown-v2 config path must be local".to_string())
            } else {
                ResidualWorkspaceBinaryAuditConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = Sprint86ResidualGateRecoveryRunner::default()
                            .run_residual_blocker_drilldown_v2(&config)?;
                        println!(
                            "drilldown_warning=residual blocker drilldown is diagnostic only and keeps remaining blockers explicit"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::WorkspaceBinaryDeltaV2 { config } => {
            if config.contains("://") {
                Err("workspace-binary-delta-v2 config path must be local".to_string())
            } else {
                ResidualWorkspaceBinaryAuditConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = Sprint86ResidualGateRecoveryRunner::default()
                            .run_workspace_binary_delta_v2(&config)?;
                        println!(
                            "binary_delta_warning=workspace binary delta v2 is sample-backed unless separately measured"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::SafetyCoveragePreservationV2 { config } => {
            if config.contains("://") {
                Err("safety-coverage-preservation-v2 config path must be local".to_string())
            } else {
                ResidualWorkspaceBinaryAuditConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = Sprint86ResidualGateRecoveryRunner::default()
                            .run_safety_coverage_preservation_v2(&config)?;
                        println!(
                            "safety_warning=safety coverage preservation v2 keeps no-live/no-broker/no-runtime/no-training guards required"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::ControlTowerWorkspaceGateV3 { config } => {
            if config.contains("://") {
                Err("control-tower-workspace-gate-v3 config path must be local".to_string())
            } else {
                ResidualWorkspaceBinaryAuditConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = Sprint86ResidualGateRecoveryRunner::default()
                            .run_control_tower_workspace_gate_v3(&config)?;
                        println!(
                            "read_only_warning=control tower workspace gate v3 is read-only static status output with no train/runtime/live/order/account/browser controls"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::Sprint87CompileGateRecover { config } => {
            if config.contains("://") {
                Err("sprint87-compile-gate-recover config path must be local".to_string())
            } else {
                WorkspaceCompileGraphAuditConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = Sprint87CompileGateRecoveryRunner::default()
                            .run_sprint87_compile_gate_recover(&config)?;
                        println!(
                            "compile_gate_warning=sprint87 compile gate recovery is research-only acceptance recovery and never runtime/training/live work"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::WorkspaceCompileGraphAudit { config } => {
            if config.contains("://") {
                Err("workspace-compile-graph-audit config path must be local".to_string())
            } else {
                WorkspaceCompileGraphAuditConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = Sprint87CompileGateRecoveryRunner::default()
                            .run_workspace_compile_graph_audit(&config)?;
                        println!(
                            "compile_graph_warning=workspace compile graph audit is diagnostic only and does not delete tests or replace full acceptance"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::TestTargetFanout { config } => {
            if config.contains("://") {
                Err("test-target-fanout config path must be local".to_string())
            } else {
                WorkspaceCompileGraphAuditConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = Sprint87CompileGateRecoveryRunner::default()
                            .run_test_target_fanout(&config)?;
                        println!(
                            "fanout_warning=test target fanout is diagnostic only and broad suites do not replace full workspace acceptance"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::DevDependencyFanout { config } => {
            if config.contains("://") {
                Err("dev-dependency-fanout config path must be local".to_string())
            } else {
                WorkspaceCompileGraphAuditConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = Sprint87CompileGateRecoveryRunner::default()
                            .run_dev_dependency_fanout(&config)?;
                        println!(
                            "dev_dependency_warning=dev dependency fanout is diagnostic only and does not remove dependencies blindly"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::FeatureUnificationAudit { config } => {
            if config.contains("://") {
                Err("feature-unification-audit config path must be local".to_string())
            } else {
                WorkspaceCompileGraphAuditConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = Sprint87CompileGateRecoveryRunner::default()
                            .run_feature_unification_audit(&config)?;
                        println!(
                            "feature_unification_warning=feature unification remains diagnostic and must not hide skips or remove safety features"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::CompileFamilyClassifierV2 { config } => {
            if config.contains("://") {
                Err("compile-family-classifier-v2 config path must be local".to_string())
            } else {
                WorkspaceCompileGraphAuditConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = Sprint87CompileGateRecoveryRunner::default()
                            .run_compile_family_classifier_v2(&config)?;
                        println!(
                            "classifier_warning=compile family classification is deterministic and keeps unknowns explicit"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::CompileHeavyConsolidationPlan { config } => {
            if config.contains("://") {
                Err("compile-heavy-consolidation-plan config path must be local".to_string())
            } else {
                WorkspaceCompileGraphAuditConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = Sprint87CompileGateRecoveryRunner::default()
                            .run_compile_heavy_consolidation_plan(&config)?;
                        println!(
                            "consolidation_warning=compile-heavy consolidation preserves assertions, records keep-separate cases, and never replaces the full workspace gate"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::CompileOnlyAttemptV2 { config } => {
            if config.contains("://") {
                Err("compile-only-attempt-v2 config path must be local".to_string())
            } else {
                WorkspaceCompileGraphAuditConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = Sprint87CompileGateRecoveryRunner::default()
                            .run_compile_only_attempt_v2(&config)?;
                        println!(
                            "compile_only_warning=compile-only attempt v2 is diagnostic only and never implies full acceptance"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::NoRunAcceptanceGateV2 { config } => {
            if config.contains("://") {
                Err("no-run-acceptance-gate-v2 config path must be local".to_string())
            } else {
                WorkspaceCompileGraphAuditConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = Sprint87CompileGateRecoveryRunner::default()
                            .run_no_run_acceptance_gate_v2(&config)?;
                        println!(
                            "no_run_warning=no-run acceptance gate v2 is compile-only interpretation and not full execution"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::FullWorkspaceAttemptV5 { config } => {
            if config.contains("://") {
                Err("full-workspace-attempt-v5 config path must be local".to_string())
            } else {
                WorkspaceCompileGraphAuditConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = Sprint87CompileGateRecoveryRunner::default()
                            .run_full_workspace_attempt_v5(&config)?;
                        println!(
                            "workspace_attempt_warning=full workspace attempt v5 never fakes pass/fail and compile-only never counts as full acceptance"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::CompileGateRecovery { config } => {
            if config.contains("://") {
                Err("compile-gate-recovery config path must be local".to_string())
            } else {
                WorkspaceCompileGraphAuditConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = Sprint87CompileGateRecoveryRunner::default()
                            .run_compile_gate_recovery(&config)?;
                        println!(
                            "gate_recovery_warning=compile gate recovery keeps compile-only, no-run, and full-workspace truth separate"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::CompileBlockerDrilldownV3 { config } => {
            if config.contains("://") {
                Err("compile-blocker-drilldown-v3 config path must be local".to_string())
            } else {
                WorkspaceCompileGraphAuditConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = Sprint87CompileGateRecoveryRunner::default()
                            .run_compile_blocker_drilldown_v3(&config)?;
                        println!(
                            "drilldown_warning=compile blocker drilldown v3 is diagnostic only and keeps remaining blockers explicit"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::TestTargetDeltaV3 { config } => {
            if config.contains("://") {
                Err("test-target-delta-v3 config path must be local".to_string())
            } else {
                WorkspaceCompileGraphAuditConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = Sprint87CompileGateRecoveryRunner::default()
                            .run_test_target_delta_v3(&config)?;
                        println!(
                            "target_delta_warning=test target delta v3 is sample-backed unless separately measured"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::SafetyCoveragePreservationV3 { config } => {
            if config.contains("://") {
                Err("safety-coverage-preservation-v3 config path must be local".to_string())
            } else {
                WorkspaceCompileGraphAuditConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = Sprint87CompileGateRecoveryRunner::default()
                            .run_safety_coverage_preservation_v3(&config)?;
                        println!(
                            "safety_warning=safety coverage preservation v3 keeps no-live/no-broker/no-runtime/no-training guards required"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::ControlTowerCompileGateV4 { config } => {
            if config.contains("://") {
                Err("control-tower-compile-gate-v4 config path must be local".to_string())
            } else {
                WorkspaceCompileGraphAuditConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = Sprint87CompileGateRecoveryRunner::default()
                            .run_control_tower_compile_gate_v4(&config)?;
                        println!(
                            "read_only_warning=control tower compile gate v4 is read-only static status output with no train/runtime/live/order/account/browser controls"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::Sprint88SevenBlockerRecover { config } => load_local_sprint88_config(
            &config,
            "sprint88-seven-blocker-recover",
        )
        .and_then(|config| {
            let report = Sprint88SevenBlockerRecoveryRunner::default()
                .run_sprint88_seven_blocker_recover(&config)?;
            print_json_report(
                "seven_blocker_warning=sprint88 seven blocker recovery is research-only acceptance recovery and never runtime/training/live work",
                &report,
            )
        }),
        Commands::SevenBlockerFamilyRecovery { config } => load_local_sprint88_config(
            &config,
            "seven-blocker-family-recovery",
        )
        .and_then(|config| {
            let report = Sprint88SevenBlockerRecoveryRunner::default()
                .run_seven_blocker_family_recovery(&config)?;
            print_json_report(
                "seven_blocker_queue_warning=seven blocker family recovery keeps the ordered queue explicit and does not claim full workspace acceptance",
                &report,
            )
        }),
        Commands::PerFamilyCompileProbe { config } => load_local_sprint88_config(
            &config,
            "per-family-compile-probe",
        )
        .and_then(|config| {
            let report = Sprint88SevenBlockerRecoveryRunner::default()
                .run_per_family_compile_probe(&config)?;
            print_json_report(
                "per_family_probe_warning=per-family compile probes are diagnostic only and never equal full workspace acceptance",
                &report,
            )
        }),
        Commands::PerFamilyNoRunProbe { config } => load_local_sprint88_config(
            &config,
            "per-family-no-run-probe",
        )
        .and_then(|config| {
            let report = Sprint88SevenBlockerRecoveryRunner::default()
                .run_per_family_no_run_probe(&config)?;
            print_json_report(
                "per_family_no_run_warning=per-family no-run probes remain compile-only interpretation and not full workspace acceptance",
                &report,
            )
        }),
        Commands::PerFamilyExecutionProbe { config } => load_local_sprint88_config(
            &config,
            "per-family-execution-probe",
        )
        .and_then(|config| {
            let report = Sprint88SevenBlockerRecoveryRunner::default()
                .run_per_family_execution_probe(&config)?;
            print_json_report(
                "per_family_execution_warning=per-family execution probes are focused diagnostics and do not imply full workspace acceptance",
                &report,
            )
        }),
        Commands::CandleExpansionRecovery { config } => load_local_sprint88_config(
            &config,
            "candle-expansion-recovery",
        )
        .and_then(|config| {
            let report = Sprint88SevenBlockerRecoveryRunner::default()
                .run_candle_expansion_recovery(&config)?;
            print_json_report(
                "candle_recovery_warning=candle expansion recovery preserves source/no-lookahead/storage coverage and stays research-only",
                &report,
            )
        }),
        Commands::ExternalPredictionRecovery { config } => load_local_sprint88_config(
            &config,
            "external-prediction-recovery",
        )
        .and_then(|config| {
            let report = Sprint88SevenBlockerRecoveryRunner::default()
                .run_external_prediction_recovery(&config)?;
            print_json_report(
                "external_recovery_warning=external prediction recovery preserves research-only validation and keeps runtime deferred",
                &report,
            )
        }),
        Commands::KrxEvidenceRecovery { config } => load_local_sprint88_config(
            &config,
            "krx-evidence-recovery",
        )
        .and_then(|config| {
            let report = Sprint88SevenBlockerRecoveryRunner::default()
                .run_krx_evidence_recovery(&config)?;
            print_json_report(
                "krx_recovery_warning=krx evidence recovery stays market-data-only and never adds order/account paths",
                &report,
            )
        }),
        Commands::DashboardRendererRecovery { config } => load_local_sprint88_config(
            &config,
            "dashboard-renderer-recovery",
        )
        .and_then(|config| {
            let report = Sprint88SevenBlockerRecoveryRunner::default()
                .run_dashboard_renderer_recovery(&config)?;
            print_json_report(
                "dashboard_recovery_warning=dashboard renderer recovery is static/read-only and never adds POST/actions/browser execution",
                &report,
            )
        }),
        Commands::CommitteeCliSafetyIsolation { config } => load_local_sprint88_config(
            &config,
            "committee-cli-safety-isolation",
        )
        .and_then(|config| {
            let report = Sprint88SevenBlockerRecoveryRunner::default()
                .run_committee_cli_safety_isolation(&config)?;
            print_json_report(
                "committee_isolation_warning=committee_cli_safety stays isolated unless a safer split preserves all CLI safety checks",
                &report,
            )
        }),
        Commands::BaselineSignalRecovery { config } => load_local_sprint88_config(
            &config,
            "baseline-signal-recovery",
        )
        .and_then(|config| {
            let report = Sprint88SevenBlockerRecoveryRunner::default()
                .run_baseline_signal_recovery(&config)?;
            print_json_report(
                "baseline_recovery_warning=baseline signal recovery preserves conservative NoTrade defaults and Risk Governor veto",
                &report,
            )
        }),
        Commands::CounterfactualBackfillRecovery { config } => load_local_sprint88_config(
            &config,
            "counterfactual-backfill-recovery",
        )
        .and_then(|config| {
            let report = Sprint88SevenBlockerRecoveryRunner::default()
                .run_counterfactual_backfill_recovery(&config)?;
            print_json_report(
                "counterfactual_recovery_warning=counterfactual backfill recovery preserves deterministic NoTrade/RiskDenied coverage and no-lookahead",
                &report,
            )
        }),
        Commands::DevDependencyImpactProbe { config } => load_local_sprint88_config(
            &config,
            "dev-dependency-impact-probe",
        )
        .and_then(|config| {
            let report = Sprint88SevenBlockerRecoveryRunner::default()
                .run_dev_dependency_impact_probe(&config)?;
            print_json_report(
                "dev_dependency_impact_warning=dev dependency impact probing is diagnostic only and does not remove dependencies blindly",
                &report,
            )
        }),
        Commands::FeatureVariantImpactProbe { config } => load_local_sprint88_config(
            &config,
            "feature-variant-impact-probe",
        )
        .and_then(|config| {
            let report = Sprint88SevenBlockerRecoveryRunner::default()
                .run_feature_variant_impact_probe(&config)?;
            print_json_report(
                "feature_variant_warning=feature variant impact remains diagnostic and unsafe unification stays explicit",
                &report,
            )
        }),
        Commands::MeasuredTargetDeltaV4 { config } => load_local_sprint88_config(
            &config,
            "measured-target-delta-v4",
        )
        .and_then(|config| {
            let report = Sprint88SevenBlockerRecoveryRunner::default()
                .run_measured_target_delta_v4(&config)?;
            print_json_report(
                "measured_delta_warning=measured target delta v4 stays sample-backed unless real measurement exists",
                &report,
            )
        }),
        Commands::RealNoRunGateAttemptV3 { config } => load_local_sprint88_config(
            &config,
            "real-no-run-gate-attempt-v3",
        )
        .and_then(|config| {
            let report = Sprint88SevenBlockerRecoveryRunner::default()
                .run_real_no_run_gate_attempt_v3(&config)?;
            print_json_report(
                "real_no_run_warning=real no-run gate attempt v3 is compile-only and never full workspace acceptance",
                &report,
            )
        }),
        Commands::RealFullWorkspaceGateAttemptV6 { config } => load_local_sprint88_config(
            &config,
            "real-full-workspace-gate-attempt-v6",
        )
        .and_then(|config| {
            let report = Sprint88SevenBlockerRecoveryRunner::default()
                .run_real_full_workspace_gate_attempt_v6(&config)?;
            print_json_report(
                "real_full_warning=real full workspace gate attempt v6 never fakes pass/fail and only accepts a finished passing workspace run",
                &report,
            )
        }),
        Commands::GateRerunAfterEachFamily { config } => load_local_sprint88_config(
            &config,
            "gate-rerun-after-each-family",
        )
        .and_then(|config| {
            let report = Sprint88SevenBlockerRecoveryRunner::default()
                .run_gate_rerun_after_each_family(&config)?;
            print_json_report(
                "rerun_warning=gate rerun after each family is diagnostic only and keeps no-run/full truth separate",
                &report,
            )
        }),
        Commands::WorkspaceGateRecoveryV5 { config } => load_local_sprint88_config(
            &config,
            "workspace-gate-recovery-v5",
        )
        .and_then(|config| {
            let report = Sprint88SevenBlockerRecoveryRunner::default()
                .run_workspace_gate_recovery_v5(&config)?;
            print_json_report(
                "workspace_gate_warning=workspace gate recovery v5 keeps previous/current no-run/full states honest and conservative",
                &report,
            )
        }),
        Commands::RemainingBlockerQueueV4 { config } => load_local_sprint88_config(
            &config,
            "remaining-blocker-queue-v4",
        )
        .and_then(|config| {
            let report = Sprint88SevenBlockerRecoveryRunner::default()
                .run_remaining_blocker_queue_v4(&config)?;
            print_json_report(
                "remaining_queue_warning=remaining blocker queue v4 keeps the primary next family explicit and local-only",
                &report,
            )
        }),
        Commands::SafetyCoveragePreservationV4 { config } => load_local_sprint88_config(
            &config,
            "safety-coverage-preservation-v4",
        )
        .and_then(|config| {
            let report = Sprint88SevenBlockerRecoveryRunner::default()
                .run_safety_coverage_preservation_v4(&config)?;
            print_json_report(
                "safety_warning=safety coverage preservation v4 keeps no-live/no-broker/no-runtime/no-training guards required",
                &report,
            )
        }),
        Commands::ControlTowerSevenBlocker { config } => load_local_sprint88_config(
            &config,
            "control-tower-seven-blocker",
        )
        .and_then(|config| {
            let report = Sprint88SevenBlockerRecoveryRunner::default()
                .run_control_tower_seven_blocker(&config)?;
            print_json_report(
                "read_only_warning=control tower seven blocker panel is read-only static status output with no train/runtime/live/order/account/browser controls",
                &report,
            )
        }),
        Commands::Sprint89CandleRecover { config } => {
            load_local_sprint89_config(&config, "sprint89-candle-recover").and_then(|config| {
                let report = Sprint89CandleRecoveryRunner::default()
                    .run_sprint89_candle_recover(&config)?;
                print_json_report(
                    "candle_recovery_warning=sprint89 candle recovery is CandleExpansionOps-only, preserves assertions, and never implies live/runtime/training scope",
                    &report,
                )
            })
        }
        Commands::CandleRealReductionPlan { config } => {
            load_local_sprint89_config(&config, "candle-real-reduction-plan").and_then(|config| {
                let report = Sprint89CandleRecoveryRunner::default()
                    .run_candle_real_reduction_plan(&config)?;
                print_json_report(
                    "candle_plan_warning=candle real reduction planning preserves assertions and keeps donor lineage explicit before any merge claim",
                    &report,
                )
            })
        }
        Commands::CandleAssertionMigration { config } => load_local_sprint89_config(
            &config,
            "candle-assertion-migration",
        )
        .and_then(|config| {
            let report = Sprint89CandleRecoveryRunner::default()
                .run_candle_assertion_migration(&config)?;
            print_json_report(
                "candle_assertion_warning=candle assertion migration never deletes assertions and records any keep-separate reasons",
                &report,
            )
        }),
        Commands::CandleFixtureSetupReduction { config } => load_local_sprint89_config(
            &config,
            "candle-fixture-setup-reduction",
        )
        .and_then(|config| {
            let report = Sprint89CandleRecoveryRunner::default()
                .run_candle_fixture_setup_reduction(&config)?;
            print_json_report(
                "candle_fixture_warning=candle fixture/setup reduction stays deterministic, local-only, and keeps shared harness usage explicit",
                &report,
            )
        }),
        Commands::CandleCompileImpact { config } => {
            load_local_sprint89_config(&config, "candle-compile-impact").and_then(|config| {
                let report = Sprint89CandleRecoveryRunner::default()
                    .run_candle_compile_impact(&config)?;
                print_json_report(
                    "candle_compile_warning=candle compile impact keeps measured and sample-backed evidence distinct and never fakes timings",
                    &report,
                )
            })
        }
        Commands::CandleNoRunRerun { config } => {
            load_local_sprint89_config(&config, "candle-no-run-rerun").and_then(|config| {
                let report = Sprint89CandleRecoveryRunner::default()
                    .run_candle_no_run_rerun(&config)?;
                print_json_report(
                    "candle_no_run_warning=candle no-run rerun is compile-only interpretation and never equals full workspace acceptance",
                    &report,
                )
            })
        }
        Commands::CandleFullGateRerun { config } => {
            load_local_sprint89_config(&config, "candle-full-gate-rerun").and_then(|config| {
                let report = Sprint89CandleRecoveryRunner::default()
                    .run_candle_full_gate_rerun(&config)?;
                print_json_report(
                    "candle_full_warning=candle full gate rerun only accepts a finished passing workspace run and never fakes pass/fail",
                    &report,
                )
            })
        }
        Commands::SevenBlockerQueueProgressV5 { config } => load_local_sprint89_config(
            &config,
            "seven-blocker-queue-progress-v5",
        )
        .and_then(|config| {
            let report = Sprint89CandleRecoveryRunner::default()
                .run_seven_blocker_queue_progress_v5(&config)?;
            print_json_report(
                "queue_progress_warning=queue progress v5 keeps CandleExpansionOps-vs-ExternalPrediction advancement evidence explicit",
                &report,
            )
        }),
        Commands::MeasuredTargetDeltaV5 { config } => {
            load_local_sprint89_config(&config, "measured-target-delta-v5").and_then(|config| {
                let report = Sprint89CandleRecoveryRunner::default()
                    .run_measured_target_delta_v5(&config)?;
                print_json_report(
                    "measured_delta_warning=measured target delta v5 keeps measured and sample-backed states explicit",
                    &report,
                )
            })
        }
        Commands::RealNoRunGateAttemptV4 { config } => load_local_sprint89_config(
            &config,
            "real-no-run-gate-attempt-v4",
        )
        .and_then(|config| {
            let report = Sprint89CandleRecoveryRunner::default()
                .run_real_no_run_gate_attempt_v4(&config)?;
            print_json_report(
                "real_no_run_warning=real no-run gate attempt v4 remains compile-only and does not imply full workspace acceptance",
                &report,
            )
        }),
        Commands::RealFullWorkspaceGateAttemptV7 { config } => load_local_sprint89_config(
            &config,
            "real-full-workspace-gate-attempt-v7",
        )
        .and_then(|config| {
            let report = Sprint89CandleRecoveryRunner::default()
                .run_real_full_workspace_gate_attempt_v7(&config)?;
            print_json_report(
                "real_full_warning=real full workspace gate attempt v7 never fakes pass/fail and only accepts a finished passing workspace run",
                &report,
            )
        }),
        Commands::WorkspaceGateRecoveryV6 { config } => load_local_sprint89_config(
            &config,
            "workspace-gate-recovery-v6",
        )
        .and_then(|config| {
            let report = Sprint89CandleRecoveryRunner::default()
                .run_workspace_gate_recovery_v6(&config)?;
            print_json_report(
                "workspace_gate_warning=workspace gate recovery v6 keeps previous/current no-run/full states honest after candle reduction",
                &report,
            )
        }),
        Commands::RemainingBlockerQueueV5 { config } => load_local_sprint89_config(
            &config,
            "remaining-blocker-queue-v5",
        )
        .and_then(|config| {
            let report = Sprint89CandleRecoveryRunner::default()
                .run_remaining_blocker_queue_v5(&config)?;
            print_json_report(
                "remaining_queue_warning=remaining blocker queue v5 keeps the next family explicit and never hides still-blocked families",
                &report,
            )
        }),
        Commands::SafetyCoveragePreservationV5 { config } => load_local_sprint89_config(
            &config,
            "safety-coverage-preservation-v5",
        )
        .and_then(|config| {
            let report = Sprint89CandleRecoveryRunner::default()
                .run_safety_coverage_preservation_v5(&config)?;
            print_json_report(
                "safety_warning=safety coverage preservation v5 keeps no-live/no-broker/no-runtime/no-training guards required",
                &report,
            )
        }),
        Commands::ControlTowerCandleRecovery { config } => load_local_sprint89_config(
            &config,
            "control-tower-candle-recovery",
        )
        .and_then(|config| {
            let report = Sprint89CandleRecoveryRunner::default()
                .run_control_tower_candle_recovery(&config)?;
            print_json_report(
                "read_only_warning=control tower candle recovery panel is read-only static status output with no train/runtime/live/order/account/browser controls",
                &report,
            )
        }),
        Commands::Sprint90ExternalPredictionRecover { config } => load_local_sprint90_config(
            &config,
            "sprint90-external-prediction-recover",
        )
        .and_then(|config| {
            let report = Sprint90ExternalPredictionRecoveryRunner::default()
                .run_sprint90_external_prediction_recover(&config)?;
            print_json_report(
                "external_recovery_warning=sprint90 external prediction recovery is ExternalPrediction-only, preserves schema/model-card/runtime guards, and never implies live/runtime/training scope",
                &report,
            )
        }),
        Commands::ExternalPredictionRealReductionPlan { config } => load_local_sprint90_config(
            &config,
            "external-prediction-real-reduction-plan",
        )
        .and_then(|config| {
            let report = Sprint90ExternalPredictionRecoveryRunner::default()
                .run_external_prediction_real_reduction_plan(&config)?;
            print_json_report(
                "external_plan_warning=external prediction real reduction planning preserves assertions and keeps donor lineage explicit before any merge claim",
                &report,
            )
        }),
        Commands::ExternalPredictionAssertionMigration { config } => load_local_sprint90_config(
            &config,
            "external-prediction-assertion-migration",
        )
        .and_then(|config| {
            let report = Sprint90ExternalPredictionRecoveryRunner::default()
                .run_external_prediction_assertion_migration(&config)?;
            print_json_report(
                "external_assertion_warning=external prediction assertion migration never deletes assertions and records any keep-separate reasons",
                &report,
            )
        }),
        Commands::ExternalPredictionFixtureSetupReduction { config } => load_local_sprint90_config(
            &config,
            "external-prediction-fixture-setup-reduction",
        )
        .and_then(|config| {
            let report = Sprint90ExternalPredictionRecoveryRunner::default()
                .run_external_prediction_fixture_setup_reduction(&config)?;
            print_json_report(
                "external_fixture_warning=external prediction fixture/setup reduction stays deterministic, local-only, and keeps shared harness usage explicit",
                &report,
            )
        }),
        Commands::ExternalPredictionFeatureVariantReduction { config } => load_local_sprint90_config(
            &config,
            "external-prediction-feature-variant-reduction",
        )
        .and_then(|config| {
            let report = Sprint90ExternalPredictionRecoveryRunner::default()
                .run_external_prediction_feature_variant_reduction(&config)?;
            print_json_report(
                "external_feature_variant_warning=external prediction feature variant reduction keeps unsafe variants explicit and never introduces hidden safety bypasses",
                &report,
            )
        }),
        Commands::ExternalPredictionCompileImpact { config } => load_local_sprint90_config(
            &config,
            "external-prediction-compile-impact",
        )
        .and_then(|config| {
            let report = Sprint90ExternalPredictionRecoveryRunner::default()
                .run_external_prediction_compile_impact(&config)?;
            print_json_report(
                "external_compile_warning=external prediction compile impact keeps measured and sample-backed evidence distinct and never fakes timings",
                &report,
            )
        }),
        Commands::ExternalPredictionNoRunRerun { config } => load_local_sprint90_config(
            &config,
            "external-prediction-no-run-rerun",
        )
        .and_then(|config| {
            let report = Sprint90ExternalPredictionRecoveryRunner::default()
                .run_external_prediction_no_run_rerun(&config)?;
            print_json_report(
                "external_no_run_warning=external prediction no-run rerun is compile-only interpretation and never equals full workspace acceptance",
                &report,
            )
        }),
        Commands::ExternalPredictionFullGateRerun { config } => load_local_sprint90_config(
            &config,
            "external-prediction-full-gate-rerun",
        )
        .and_then(|config| {
            let report = Sprint90ExternalPredictionRecoveryRunner::default()
                .run_external_prediction_full_gate_rerun(&config)?;
            print_json_report(
                "external_full_warning=external prediction full gate rerun only accepts a finished passing workspace run and never fakes pass/fail",
                &report,
            )
        }),
        Commands::ExternalPredictionSchemaPreservation { config } => load_local_sprint90_config(
            &config,
            "external-prediction-schema-preservation",
        )
        .and_then(|config| {
            let report = Sprint90ExternalPredictionRecoveryRunner::default()
                .run_external_prediction_schema_preservation(&config)?;
            print_json_report(
                "external_schema_warning=external prediction schema preservation keeps sequence, duplicate, probability, and forbidden-column guards explicit",
                &report,
            )
        }),
        Commands::ExternalPredictionModelCardPreservation { config } => load_local_sprint90_config(
            &config,
            "external-prediction-model-card-preservation",
        )
        .and_then(|config| {
            let report = Sprint90ExternalPredictionRecoveryRunner::default()
                .run_external_prediction_model_card_preservation(&config)?;
            print_json_report(
                "external_model_card_warning=external prediction model-card preservation keeps runtime/training/live-inference forbidden states explicit",
                &report,
            )
        }),
        Commands::ExternalPredictionEvaluationPreservation { config } => load_local_sprint90_config(
            &config,
            "external-prediction-evaluation-preservation",
        )
        .and_then(|config| {
            let report = Sprint90ExternalPredictionRecoveryRunner::default()
                .run_external_prediction_evaluation_preservation(&config)?;
            print_json_report(
                "external_eval_warning=external prediction evaluation preservation stays offline-only, trinity-only, and research-only",
                &report,
            )
        }),
        Commands::SevenBlockerQueueProgressV6 { config } => load_local_sprint90_config(
            &config,
            "seven-blocker-queue-progress-v6",
        )
        .and_then(|config| {
            let report = Sprint90ExternalPredictionRecoveryRunner::default()
                .run_seven_blocker_queue_progress_v6(&config)?;
            print_json_report(
                "queue_progress_warning=queue progress v6 keeps ExternalPrediction-vs-KrxEvidence advancement evidence explicit",
                &report,
            )
        }),
        Commands::MeasuredTargetDeltaV6 { config } => load_local_sprint90_config(
            &config,
            "measured-target-delta-v6",
        )
        .and_then(|config| {
            let report = Sprint90ExternalPredictionRecoveryRunner::default()
                .run_measured_target_delta_v6(&config)?;
            print_json_report(
                "measured_delta_warning=measured target delta v6 keeps measured and sample-backed states explicit",
                &report,
            )
        }),
        Commands::RealNoRunGateAttemptV5 { config } => load_local_sprint90_config(
            &config,
            "real-no-run-gate-attempt-v5",
        )
        .and_then(|config| {
            let report = Sprint90ExternalPredictionRecoveryRunner::default()
                .run_real_no_run_gate_attempt_v5(&config)?;
            print_json_report(
                "real_no_run_warning=real no-run gate attempt v5 remains compile-only and does not imply full workspace acceptance",
                &report,
            )
        }),
        Commands::RealFullWorkspaceGateAttemptV8 { config } => load_local_sprint90_config(
            &config,
            "real-full-workspace-gate-attempt-v8",
        )
        .and_then(|config| {
            let report = Sprint90ExternalPredictionRecoveryRunner::default()
                .run_real_full_workspace_gate_attempt_v8(&config)?;
            print_json_report(
                "real_full_warning=real full workspace gate attempt v8 never fakes pass/fail and only accepts a finished passing workspace run",
                &report,
            )
        }),
        Commands::WorkspaceGateRecoveryV7 { config } => load_local_sprint90_config(
            &config,
            "workspace-gate-recovery-v7",
        )
        .and_then(|config| {
            let report = Sprint90ExternalPredictionRecoveryRunner::default()
                .run_workspace_gate_recovery_v7(&config)?;
            print_json_report(
                "workspace_gate_warning=workspace gate recovery v7 keeps previous/current no-run/full states honest after external prediction reduction",
                &report,
            )
        }),
        Commands::RemainingBlockerQueueV6 { config } => load_local_sprint90_config(
            &config,
            "remaining-blocker-queue-v6",
        )
        .and_then(|config| {
            let report = Sprint90ExternalPredictionRecoveryRunner::default()
                .run_remaining_blocker_queue_v6(&config)?;
            print_json_report(
                "remaining_queue_warning=remaining blocker queue v6 keeps the next family explicit and never hides still-blocked families",
                &report,
            )
        }),
        Commands::SafetyCoveragePreservationV6 { config } => load_local_sprint90_config(
            &config,
            "safety-coverage-preservation-v6",
        )
        .and_then(|config| {
            let report = Sprint90ExternalPredictionRecoveryRunner::default()
                .run_safety_coverage_preservation_v6(&config)?;
            print_json_report(
                "safety_warning=safety coverage preservation v6 keeps no-live/no-broker/no-runtime/no-training guards required",
                &report,
            )
        }),
        Commands::ControlTowerExternalPredictionRecovery { config } => load_local_sprint90_config(
            &config,
            "control-tower-external-prediction-recovery",
        )
        .and_then(|config| {
            let report = Sprint90ExternalPredictionRecoveryRunner::default()
                .run_control_tower_external_prediction_recovery(&config)?;
            print_json_report(
                "read_only_warning=control tower external prediction recovery panel is read-only static status output with no train/runtime/live/order/account/browser controls",
                &report,
            )
        }),
        Commands::Sprint91KrxEvidenceRecover { config } => {
            load_local_sprint91_config(&config, "sprint91-krx-evidence-recover").and_then(|config| {
                let report = Sprint91KrxEvidenceRecoveryRunner::default()
                    .run_sprint91_krx_evidence_recover(&config)?;
                print_json_report(
                    "krx_recovery_warning=sprint91 krx evidence recovery is KrxEvidence-only, market-data-only, local-only, and never implies live/runtime/training scope",
                    &report,
                )
            })
        }
        Commands::KrxEvidenceRealReductionPlan { config } => {
            load_local_sprint91_config(&config, "krx-evidence-real-reduction-plan").and_then(
                |config| {
                    let report = Sprint91KrxEvidenceRecoveryRunner::default()
                        .run_krx_evidence_real_reduction_plan(&config)?;
                    print_json_report(
                        "krx_plan_warning=krx evidence real reduction planning preserves assertions and donor lineage before any merge claim",
                        &report,
                    )
                },
            )
        }
        Commands::KrxEvidenceAssertionMigration { config } => {
            load_local_sprint91_config(&config, "krx-evidence-assertion-migration").and_then(
                |config| {
                    let report = Sprint91KrxEvidenceRecoveryRunner::default()
                        .run_krx_evidence_assertion_migration(&config)?;
                    print_json_report(
                        "krx_assertion_warning=krx evidence assertion migration never deletes assertions and records any keep-separate reasons",
                        &report,
                    )
                },
            )
        }
        Commands::KrxEvidenceFixtureSetupReduction { config } => {
            load_local_sprint91_config(&config, "krx-evidence-fixture-setup-reduction").and_then(
                |config| {
                    let report = Sprint91KrxEvidenceRecoveryRunner::default()
                        .run_krx_evidence_fixture_setup_reduction(&config)?;
                    print_json_report(
                        "krx_fixture_warning=krx evidence fixture/setup reduction stays deterministic, local-only, and keeps shared harness usage explicit",
                        &report,
                    )
                },
            )
        }
        Commands::KrxEvidenceAuthBoundaryPreservation { config } => {
            load_local_sprint91_config(&config, "krx-evidence-auth-boundary-preservation").and_then(
                |config| {
                    let report = Sprint91KrxEvidenceRecoveryRunner::default()
                        .run_krx_evidence_auth_boundary_preservation(&config)?;
                    print_json_report(
                        "krx_auth_warning=krx evidence auth boundary preservation keeps missing-auth behavior explicit and never renders secret values",
                        &report,
                    )
                },
            )
        }
        Commands::KrxEvidenceEndpointTemplatePreservation { config } => load_local_sprint91_config(
            &config,
            "krx-evidence-endpoint-template-preservation",
        )
        .and_then(|config| {
            let report = Sprint91KrxEvidenceRecoveryRunner::default()
                .run_krx_evidence_endpoint_template_preservation(&config)?;
            print_json_report(
                "krx_endpoint_warning=krx evidence endpoint-template preservation keeps endpoint template requirements and market-data-only request building explicit",
                &report,
            )
        }),
        Commands::KrxEvidenceSourceBoundaryPreservation { config } => load_local_sprint91_config(
            &config,
            "krx-evidence-source-boundary-preservation",
        )
        .and_then(|config| {
            let report = Sprint91KrxEvidenceRecoveryRunner::default()
                .run_krx_evidence_source_boundary_preservation(&config)?;
            print_json_report(
                "krx_source_warning=krx evidence source-boundary preservation keeps official-vs-fixture boundaries explicit and never promotes sources",
                &report,
            )
        }),
        Commands::KrxEvidenceMarketDataOnlyPreservation { config } => load_local_sprint91_config(
            &config,
            "krx-evidence-market-data-only-preservation",
        )
        .and_then(|config| {
            let report = Sprint91KrxEvidenceRecoveryRunner::default()
                .run_krx_evidence_market_data_only_preservation(&config)?;
            print_json_report(
                "krx_market_data_warning=krx evidence market-data-only preservation keeps no order/account/broker execution path explicit",
                &report,
            )
        }),
        Commands::KrxEvidenceCompileImpact { config } => {
            load_local_sprint91_config(&config, "krx-evidence-compile-impact").and_then(|config| {
                let report = Sprint91KrxEvidenceRecoveryRunner::default()
                    .run_krx_evidence_compile_impact(&config)?;
                print_json_report(
                    "krx_compile_warning=krx evidence compile impact keeps measured and sample-backed evidence distinct and never fakes timings",
                    &report,
                )
            })
        }
        Commands::KrxEvidenceNoRunRerun { config } => {
            load_local_sprint91_config(&config, "krx-evidence-no-run-rerun").and_then(|config| {
                let report = Sprint91KrxEvidenceRecoveryRunner::default()
                    .run_krx_evidence_no_run_rerun(&config)?;
                print_json_report(
                    "krx_no_run_warning=krx evidence no-run rerun is compile-only interpretation and never equals full workspace acceptance",
                    &report,
                )
            })
        }
        Commands::KrxEvidenceFullGateRerun { config } => {
            load_local_sprint91_config(&config, "krx-evidence-full-gate-rerun").and_then(|config| {
                let report = Sprint91KrxEvidenceRecoveryRunner::default()
                    .run_krx_evidence_full_gate_rerun(&config)?;
                print_json_report(
                    "krx_full_warning=krx evidence full gate rerun only accepts a finished passing workspace run and never fakes pass/fail",
                    &report,
                )
            })
        }
        Commands::SevenBlockerQueueProgressV7 { config } => {
            load_local_sprint91_config(&config, "seven-blocker-queue-progress-v7").and_then(
                |config| {
                    let report = Sprint91KrxEvidenceRecoveryRunner::default()
                        .run_seven_blocker_queue_progress_v7(&config)?;
                    print_json_report(
                        "queue_progress_warning=queue progress v7 keeps KrxEvidence-vs-DashboardRenderer advancement evidence explicit",
                        &report,
                    )
                },
            )
        }
        Commands::MeasuredTargetDeltaV7 { config } => {
            load_local_sprint91_config(&config, "measured-target-delta-v7").and_then(|config| {
                let report = Sprint91KrxEvidenceRecoveryRunner::default()
                    .run_measured_target_delta_v7(&config)?;
                print_json_report(
                    "measured_delta_warning=measured target delta v7 keeps measured and sample-backed states explicit",
                    &report,
                )
            })
        }
        Commands::RealNoRunGateAttemptV6 { config } => {
            load_local_sprint91_config(&config, "real-no-run-gate-attempt-v6").and_then(|config| {
                let report = Sprint91KrxEvidenceRecoveryRunner::default()
                    .run_real_no_run_gate_attempt_v6(&config)?;
                print_json_report(
                    "real_no_run_warning=real no-run gate attempt v6 remains compile-only and does not imply full workspace acceptance",
                    &report,
                )
            })
        }
        Commands::RealFullWorkspaceGateAttemptV9 { config } => load_local_sprint91_config(
            &config,
            "real-full-workspace-gate-attempt-v9",
        )
        .and_then(|config| {
            let report = Sprint91KrxEvidenceRecoveryRunner::default()
                .run_real_full_workspace_gate_attempt_v9(&config)?;
            print_json_report(
                "real_full_warning=real full workspace gate attempt v9 never fakes pass/fail and only accepts a finished passing workspace run",
                &report,
            )
        }),
        Commands::WorkspaceGateRecoveryV8 { config } => {
            load_local_sprint91_config(&config, "workspace-gate-recovery-v8").and_then(|config| {
                let report = Sprint91KrxEvidenceRecoveryRunner::default()
                    .run_workspace_gate_recovery_v8(&config)?;
                print_json_report(
                    "workspace_gate_warning=workspace gate recovery v8 keeps previous/current no-run/full states honest after krx evidence reduction",
                    &report,
                )
            })
        }
        Commands::RemainingBlockerQueueV7 { config } => {
            load_local_sprint91_config(&config, "remaining-blocker-queue-v7").and_then(|config| {
                let report = Sprint91KrxEvidenceRecoveryRunner::default()
                    .run_remaining_blocker_queue_v7(&config)?;
                print_json_report(
                    "remaining_queue_warning=remaining blocker queue v7 keeps the next family explicit and never hides still-blocked families",
                    &report,
                )
            })
        }
        Commands::SafetyCoveragePreservationV7 { config } => {
            load_local_sprint91_config(&config, "safety-coverage-preservation-v7").and_then(
                |config| {
                    let report = Sprint91KrxEvidenceRecoveryRunner::default()
                        .run_safety_coverage_preservation_v7(&config)?;
                    print_json_report(
                        "safety_warning=safety coverage preservation v7 keeps no-live/no-broker/no-runtime/no-training guards required",
                        &report,
                    )
                },
            )
        }
        Commands::ControlTowerKrxEvidenceRecovery { config } => load_local_sprint91_config(
            &config,
            "control-tower-krx-evidence-recovery",
        )
        .and_then(|config| {
            let report = Sprint91KrxEvidenceRecoveryRunner::default()
                .run_control_tower_krx_evidence_recovery(&config)?;
            print_json_report(
                "read_only_warning=control tower krx evidence recovery panel is read-only static status output with no train/runtime/live/order/account/browser controls",
                &report,
            )
        }),
        Commands::Sprint92KrxWarningClose { config } => {
            load_local_sprint92_config(&config, "sprint92-krx-warning-close").and_then(|config| {
                let report = Sprint92KrxWarningClosureRunner::default()
                    .run_sprint92_krx_warning_close(&config)?;
                print_json_report(
                    "krx_warning_closure_warning=sprint92 krx warning closure is research-only, market-data-only, local-only, and never implies runtime/training/live scope",
                    &report,
                )
            })
        }
        Commands::KrxWarningClosure { config } => {
            load_local_sprint92_config(&config, "krx-warning-closure").and_then(|config| {
                let report = Sprint92KrxWarningClosureRunner::default()
                    .run_krx_warning_closure(&config)?;
                print_json_report(
                    "krx_warning_closure_warning=krx warning closure keeps warning-backed vs explicit isolated closure honest",
                    &report,
                )
            })
        }
        Commands::KrxSecretSafetyIsolation { config } => {
            load_local_sprint92_config(&config, "krx-secret-safety-isolation").and_then(|config| {
                let report = Sprint92KrxWarningClosureRunner::default()
                    .run_krx_secret_safety_isolation(&config)?;
                print_json_report(
                    "krx_secret_warning=krx secret-safety isolation keeps the raw archive sentinel explicit and never weakens secret handling",
                    &report,
                )
            })
        }
        Commands::KrxRawArchiveRedactionCoverage { config } => load_local_sprint92_config(
            &config,
            "krx-raw-archive-redaction-coverage",
        )
        .and_then(|config| {
            let report = Sprint92KrxWarningClosureRunner::default()
                .run_krx_raw_archive_redaction_coverage(&config)?;
            print_json_report(
                "krx_raw_archive_warning=krx raw archive redaction coverage requires redaction assertions and never exposes secret values",
                &report,
            )
        }),
        Commands::KrxManualReviewClose { config } => {
            load_local_sprint92_config(&config, "krx-manual-review-close").and_then(|config| {
                let report = Sprint92KrxWarningClosureRunner::default()
                    .run_krx_manual_review_close(&config)?;
                print_json_report(
                    "krx_manual_review_warning=krx manual review closure stays conservative and never fakes full workspace recovery",
                    &report,
                )
            })
        }
        Commands::KrxGenuineReductionGate { config } => {
            load_local_sprint92_config(&config, "krx-genuine-reduction-gate").and_then(|config| {
                let report = Sprint92KrxWarningClosureRunner::default()
                    .run_krx_genuine_reduction_gate(&config)?;
                print_json_report(
                    "krx_genuine_warning=krx genuine reduction gate keeps warning-backed vs genuine states explicit",
                    &report,
                )
            })
        }
        Commands::KrxQueueAdvancementGate { config } => {
            load_local_sprint92_config(&config, "krx-queue-advancement-gate").and_then(|config| {
                let report = Sprint92KrxWarningClosureRunner::default()
                    .run_krx_queue_advancement_gate(&config)?;
                print_json_report(
                    "krx_queue_warning=krx queue advancement gate never skips unresolved warning or gate-cause interpretation",
                    &report,
                )
            })
        }
        Commands::KrxRealGateCauseDrilldown { config } => load_local_sprint92_config(
            &config,
            "krx-real-gate-cause-drilldown",
        )
        .and_then(|config| {
            let report = Sprint92KrxWarningClosureRunner::default()
                .run_krx_real_gate_cause_drilldown(&config)?;
            print_json_report(
                "krx_gate_cause_warning=krx real gate cause drilldown keeps no-run/full causes explicit and never overclaims recovery",
                &report,
            )
        }),
        Commands::KrxNoRunTimeoutCause { config } => {
            load_local_sprint92_config(&config, "krx-no-run-timeout-cause").and_then(|config| {
                let report = Sprint92KrxWarningClosureRunner::default()
                    .run_krx_no_run_timeout_cause(&config)?;
                print_json_report(
                    "krx_no_run_warning=krx no-run timeout cause is compile-only interpretation and never equals full workspace acceptance",
                    &report,
                )
            })
        }
        Commands::KrxFullWorkspaceTimeoutCause { config } => load_local_sprint92_config(
            &config,
            "krx-full-workspace-timeout-cause",
        )
        .and_then(|config| {
            let report = Sprint92KrxWarningClosureRunner::default()
                .run_krx_full_workspace_timeout_cause(&config)?;
            print_json_report(
                "krx_full_warning=krx full workspace timeout cause only accepts a finished passing workspace run",
                &report,
            )
        }),
        Commands::DashboardRendererEntryGate { config } => {
            load_local_sprint92_config(&config, "dashboard-renderer-entry-gate").and_then(|config| {
                let report = Sprint92KrxWarningClosureRunner::default()
                    .run_dashboard_renderer_entry_gate(&config)?;
                print_json_report(
                    "dashboard_entry_warning=dashboard renderer entry gate is entry-only and never implies reduction completion",
                    &report,
                )
            })
        }
        Commands::DashboardRendererReadinessPrecheck { config } => load_local_sprint92_config(
            &config,
            "dashboard-renderer-readiness-precheck",
        )
        .and_then(|config| {
            let report = Sprint92KrxWarningClosureRunner::default()
                .run_dashboard_renderer_readiness_precheck(&config)?;
            print_json_report(
                "dashboard_precheck_warning=dashboard renderer precheck keeps static HTML, read-only, and no-browser semantics explicit",
                &report,
            )
        }),
        Commands::MeasuredTargetDeltaV8 { config } => {
            load_local_sprint92_config(&config, "measured-target-delta-v8").and_then(|config| {
                let report = Sprint92KrxWarningClosureRunner::default()
                    .run_measured_target_delta_v8(&config)?;
                print_json_report(
                    "measured_delta_warning=measured target delta v8 keeps measured and sample-backed states explicit",
                    &report,
                )
            })
        }
        Commands::RealNoRunGateAttemptV7 { config } => {
            load_local_sprint92_config(&config, "real-no-run-gate-attempt-v7").and_then(|config| {
                let report = Sprint92KrxWarningClosureRunner::default()
                    .run_real_no_run_gate_attempt_v7(&config)?;
                print_json_report(
                    "real_no_run_warning=real no-run gate attempt v7 remains compile-only and does not imply full workspace acceptance",
                    &report,
                )
            })
        }
        Commands::RealFullWorkspaceGateAttemptV10 { config } => load_local_sprint92_config(
            &config,
            "real-full-workspace-gate-attempt-v10",
        )
        .and_then(|config| {
            let report = Sprint92KrxWarningClosureRunner::default()
                .run_real_full_workspace_gate_attempt_v10(&config)?;
            print_json_report(
                "real_full_warning=real full workspace gate attempt v10 never fakes pass/fail and only accepts a finished passing workspace run",
                &report,
            )
        }),
        Commands::WorkspaceGateRecoveryV9 { config } => {
            load_local_sprint92_config(&config, "workspace-gate-recovery-v9").and_then(|config| {
                let report = Sprint92KrxWarningClosureRunner::default()
                    .run_workspace_gate_recovery_v9(&config)?;
                print_json_report(
                    "workspace_gate_warning=workspace gate recovery v9 keeps previous/current no-run/full states honest after krx warning closure",
                    &report,
                )
            })
        }
        Commands::RemainingBlockerQueueV8 { config } => {
            load_local_sprint92_config(&config, "remaining-blocker-queue-v8").and_then(|config| {
                let report = Sprint92KrxWarningClosureRunner::default()
                    .run_remaining_blocker_queue_v8(&config)?;
                print_json_report(
                    "remaining_queue_warning=remaining blocker queue v8 keeps the next family explicit and never hides still-blocked families",
                    &report,
                )
            })
        }
        Commands::SafetyCoveragePreservationV8 { config } => {
            load_local_sprint92_config(&config, "safety-coverage-preservation-v8").and_then(|config| {
                let report = Sprint92KrxWarningClosureRunner::default()
                    .run_safety_coverage_preservation_v8(&config)?;
                print_json_report(
                    "safety_warning=safety coverage preservation v8 keeps no-live/no-broker/no-runtime/no-training guards required",
                    &report,
                )
            })
        }
        Commands::ControlTowerKrxWarningClosure { config } => load_local_sprint92_config(
            &config,
            "control-tower-krx-warning-closure",
        )
        .and_then(|config| {
            let report = Sprint92KrxWarningClosureRunner::default()
                .run_control_tower_krx_warning_closure(&config)?;
            print_json_report(
                "read_only_warning=control tower krx warning closure panel is read-only static status output with no train/runtime/live/order/account/browser controls",
                &report,
            )
        }),
        Commands::Sprint93TimeoutAttribution { config } => {
            load_local_sprint93_config(&config, "sprint93-timeout-attribution").and_then(|config| {
                let report = Sprint93TimeoutAttributionRunner::default()
                    .run_sprint93_timeout_attribution(&config)?;
                print_json_report(
                    "timeout_attribution_warning=sprint93 timeout attribution is research-only, local-only, deterministic, and never begins DashboardRenderer reduction",
                    &report,
                )
            })
        }
        Commands::RealTimeoutAttribution { config } => {
            load_local_sprint93_config(&config, "real-timeout-attribution").and_then(|config| {
                let report = Sprint93TimeoutAttributionRunner::default()
                    .run_real_timeout_attribution(&config)?;
                print_json_report(
                    "timeout_attribution_warning=real timeout attribution stays diagnostic/research-only and never claims full workspace acceptance",
                    &report,
                )
            })
        }
        Commands::RealNoRunDiagnosticPass { config } => {
            load_local_sprint93_config(&config, "real-no-run-diagnostic-pass").and_then(|config| {
                let report = Sprint93TimeoutAttributionRunner::default()
                    .run_real_no_run_diagnostic_pass(&config)?;
                print_json_report(
                    "diagnostic_warning=real no-run diagnostic pass is diagnostic, not acceptance, and never equals full workspace pass",
                    &report,
                )
            })
        }
        Commands::RealFullDiagnosticPass { config } => {
            load_local_sprint93_config(&config, "real-full-diagnostic-pass").and_then(|config| {
                let report = Sprint93TimeoutAttributionRunner::default()
                    .run_real_full_workspace_diagnostic_pass(&config)?;
                print_json_report(
                    "diagnostic_warning=real full diagnostic pass is diagnostic, not the final quiet gate, and never fakes pass/fail",
                    &report,
                )
            })
        }
        Commands::CargoMessageCapture { config } => {
            load_local_sprint93_config(&config, "cargo-message-capture").and_then(|config| {
                let report = Sprint93TimeoutAttributionRunner::default()
                    .run_cargo_message_capture(&config)?;
                print_json_report(
                    "capture_warning=cargo message capture is secret-safe local capture for timeout attribution only",
                    &report,
                )
            })
        }
        Commands::ActiveRustcSnapshot { config } => {
            load_local_sprint93_config(&config, "active-rustc-snapshot").and_then(|config| {
                let report = Sprint93TimeoutAttributionRunner::default()
                    .run_active_rustc_snapshot(&config)?;
                print_json_report(
                    "snapshot_warning=active rustc snapshot keeps local process capture redacted and secret-safe",
                    &report,
                )
            })
        }
        Commands::TargetDirGrowth { config } => {
            load_local_sprint93_config(&config, "target-dir-growth").and_then(|config| {
                let report = Sprint93TimeoutAttributionRunner::default()
                    .run_target_dir_growth(&config)?;
                print_json_report(
                    "target_dir_warning=target dir growth is local deterministic observation only and never a fake measurement claim",
                    &report,
                )
            })
        }
        Commands::CargoTargetProgressTimeline { config } => {
            load_local_sprint93_config(&config, "cargo-target-progress-timeline").and_then(|config| {
                let report = Sprint93TimeoutAttributionRunner::default()
                    .run_cargo_target_progress_timeline(&config)?;
                print_json_report(
                    "timeline_warning=cargo target progress timeline is local deterministic timeout attribution output only",
                    &report,
                )
            })
        }
        Commands::QuietVsDiagnosticGate { config } => {
            load_local_sprint93_config(&config, "quiet-vs-diagnostic-gate").and_then(|config| {
                let report = Sprint93TimeoutAttributionRunner::default()
                    .run_quiet_vs_diagnostic_gate(&config)?;
                print_json_report(
                    "comparison_warning=quiet vs diagnostic gate comparison never treats diagnostic visibility as quiet acceptance",
                    &report,
                )
            })
        }
        Commands::KrxNonPrimaryProof { config } => {
            load_local_sprint93_config(&config, "krx-non-primary-proof").and_then(|config| {
                let report = Sprint93TimeoutAttributionRunner::default()
                    .run_krx_non_primary_proof(&config)?;
                print_json_report(
                    "krx_proof_warning=krx non-primary proof stays research-only and DashboardRenderer needs explicit proof before entry release",
                    &report,
                )
            })
        }
        Commands::UnknownTimeoutClosure { config } => {
            load_local_sprint93_config(&config, "unknown-timeout-closure").and_then(|config| {
                let report = Sprint93TimeoutAttributionRunner::default()
                    .run_unknown_timeout_closure(&config)?;
                print_json_report(
                    "unknown_timeout_warning=unknown timeout closure keeps no-run/full attribution interpretation explicit and honest",
                    &report,
                )
            })
        }
        Commands::WorkspaceTimeoutAttributionDecision { config } => {
            load_local_sprint93_config(&config, "workspace-timeout-attribution-decision").and_then(|config| {
                let report = Sprint93TimeoutAttributionRunner::default()
                    .run_workspace_timeout_attribution_decision(&config)?;
                print_json_report(
                    "decision_warning=workspace timeout attribution decision never skips KRX/non-KRX interpretation or DashboardRenderer entry gating",
                    &report,
                )
            })
        }
        Commands::DashboardRendererEntryReleaseGate { config } => {
            load_local_sprint93_config(&config, "dashboard-renderer-entry-release-gate").and_then(|config| {
                let report = Sprint93TimeoutAttributionRunner::default()
                    .run_dashboard_renderer_entry_release_gate(&config)?;
                print_json_report(
                    "dashboard_entry_warning=dashboard renderer entry release gate is entry only and never DashboardRenderer reduction completion",
                    &report,
                )
            })
        }
        Commands::DashboardRendererReductionHold { config } => {
            load_local_sprint93_config(&config, "dashboard-renderer-reduction-hold").and_then(|config| {
                let report = Sprint93TimeoutAttributionRunner::default()
                    .run_dashboard_renderer_reduction_hold(&config)?;
                print_json_report(
                    "dashboard_hold_warning=dashboard renderer reduction hold keeps reduction not started even when entry is released",
                    &report,
                )
            })
        }
        Commands::WorkspaceGateRecoveryV10 { config } => {
            load_local_sprint93_config(&config, "workspace-gate-recovery-v10").and_then(|config| {
                let report = Sprint93TimeoutAttributionRunner::default()
                    .run_workspace_gate_recovery_v10(&config)?;
                print_json_report(
                    "workspace_gate_warning=workspace gate recovery v10 keeps timeout attribution improvement separate from finished quiet acceptance",
                    &report,
                )
            })
        }
        Commands::RemainingBlockerQueueV9 { config } => {
            load_local_sprint93_config(&config, "remaining-blocker-queue-v9").and_then(|config| {
                let report = Sprint93TimeoutAttributionRunner::default()
                    .run_remaining_blocker_queue_v9(&config)?;
                print_json_report(
                    "remaining_queue_warning=remaining blocker queue v9 keeps queue advancement explicit and never hides blocked families",
                    &report,
                )
            })
        }
        Commands::SafetyCoveragePreservationV9 { config } => {
            load_local_sprint93_config(&config, "safety-coverage-preservation-v9").and_then(|config| {
                let report = Sprint93TimeoutAttributionRunner::default()
                    .run_safety_coverage_preservation_v9(&config)?;
                print_json_report(
                    "safety_warning=safety coverage preservation v9 keeps no-live/no-broker/no-runtime/no-training guards and secret-free diagnostics required",
                    &report,
                )
            })
        }
        Commands::ControlTowerTimeoutAttribution { config } => {
            load_local_sprint93_config(&config, "control-tower-timeout-attribution").and_then(|config| {
                let report = Sprint93TimeoutAttributionRunner::default()
                    .run_control_tower_timeout_attribution(&config)?;
                print_json_report(
                    "read_only_warning=control tower timeout attribution panel is read-only static status output with no train/runtime/live/order/account/browser controls",
                    &report,
                )
            })
        }
        Commands::Sprint94DashboardRendererRecover { config } => {
            load_local_sprint94_config(&config, "sprint94-dashboard-renderer-recover").and_then(
                |config| {
                    let report = Sprint94DashboardRendererRecoveryRunner::default()
                        .run_sprint94_dashboard_renderer_recover(&config)?;
                    print_json_report(
                        "dashboard_renderer_warning=sprint94 DashboardRenderer recovery is research-only, DashboardRenderer-only, static/read-only, and never equals full workspace acceptance",
                        &report,
                    )
                },
            )
        }
        Commands::DashboardRendererRealReductionPlan { config } => {
            load_local_sprint94_config(&config, "dashboard-renderer-real-reduction-plan").and_then(
                |config| {
                    let report = Sprint94DashboardRendererRecoveryRunner::default()
                        .run_dashboard_renderer_real_reduction_plan(&config)?;
                    print_json_report(
                        "dashboard_renderer_warning=real reduction plan preserves assertions and keeps DashboardRenderer-only scope explicit",
                        &report,
                    )
                },
            )
        }
        Commands::DashboardRendererAssertionMigration { config } => {
            load_local_sprint94_config(&config, "dashboard-renderer-assertion-migration").and_then(
                |config| {
                    let report = Sprint94DashboardRendererRecoveryRunner::default()
                        .run_dashboard_renderer_assertion_migration(&config)?;
                    print_json_report(
                        "dashboard_renderer_warning=assertion migration never deletes assertions and keeps isolation reasons explicit",
                        &report,
                    )
                },
            )
        }
        Commands::DashboardRendererFixtureSetupReduction { config } => {
            load_local_sprint94_config(&config, "dashboard-renderer-fixture-setup-reduction")
                .and_then(|config| {
                    let report = Sprint94DashboardRendererRecoveryRunner::default()
                        .run_dashboard_renderer_fixture_setup_reduction(&config)?;
                    print_json_report(
                        "dashboard_renderer_warning=fixture/setup reduction is local-only and preserves deterministic output",
                        &report,
                    )
                })
        }
        Commands::DashboardRendererStaticSafetyPreservation { config } => {
            load_local_sprint94_config(&config, "dashboard-renderer-static-safety-preservation")
                .and_then(|config| {
                    let report = Sprint94DashboardRendererRecoveryRunner::default()
                        .run_dashboard_renderer_static_safety_preservation(&config)?;
                    print_json_report(
                        "dashboard_renderer_warning=static safety preservation keeps the dashboard static/read-only/paper-only with no external assets",
                        &report,
                    )
                })
        }
        Commands::DashboardRendererSecretRedactionPreservation { config } => {
            load_local_sprint94_config(
                &config,
                "dashboard-renderer-secret-redaction-preservation",
            )
            .and_then(|config| {
                let report = Sprint94DashboardRendererRecoveryRunner::default()
                    .run_dashboard_renderer_secret_redaction_preservation(&config)?;
                print_json_report(
                    "dashboard_renderer_warning=secret redaction preservation keeps HTML/JSON/TXT and diagnostics secret-free",
                    &report,
                )
            })
        }
        Commands::DashboardRendererNoBrowserExecution { config } => {
            load_local_sprint94_config(&config, "dashboard-renderer-no-browser-execution").and_then(
                |config| {
                    let report = Sprint94DashboardRendererRecoveryRunner::default()
                        .run_dashboard_renderer_no_browser_execution(&config)?;
                    print_json_report(
                        "dashboard_renderer_warning=no browser execution preservation keeps no JS, no POST/forms, and no active serve controls",
                        &report,
                    )
                },
            )
        }
        Commands::DashboardRendererNoActionControl { config } => {
            load_local_sprint94_config(&config, "dashboard-renderer-no-action-control").and_then(
                |config| {
                    let report = Sprint94DashboardRendererRecoveryRunner::default()
                        .run_dashboard_renderer_no_action_control(&config)?;
                    print_json_report(
                        "dashboard_renderer_warning=no action control preservation keeps train/runtime/live/order/account/trade controls absent",
                        &report,
                    )
                },
            )
        }
        Commands::DashboardRendererDeterminismPreservation { config } => {
            load_local_sprint94_config(&config, "dashboard-renderer-determinism-preservation")
                .and_then(|config| {
                    let report = Sprint94DashboardRendererRecoveryRunner::default()
                        .run_dashboard_renderer_determinism_preservation(&config)?;
                    print_json_report(
                        "dashboard_renderer_warning=determinism preservation keeps state/render/storage fingerprints stable",
                        &report,
                    )
                })
        }
        Commands::DashboardRendererGoldenOutputReduction { config } => {
            load_local_sprint94_config(&config, "dashboard-renderer-golden-output-reduction")
                .and_then(|config| {
                    let report = Sprint94DashboardRendererRecoveryRunner::default()
                        .run_dashboard_renderer_golden_output_reduction(&config)?;
                    print_json_report(
                        "dashboard_renderer_warning=golden output reduction preserves HTML/JSON/TXT checks with no hidden bless update",
                        &report,
                    )
                })
        }
        Commands::DashboardRendererCompileImpact { config } => {
            load_local_sprint94_config(&config, "dashboard-renderer-compile-impact").and_then(
                |config| {
                    let report = Sprint94DashboardRendererRecoveryRunner::default()
                        .run_dashboard_renderer_compile_impact(&config)?;
                    print_json_report(
                        "dashboard_renderer_warning=compile impact is sample-backed or measured only and never a fake timing claim",
                        &report,
                    )
                },
            )
        }
        Commands::DashboardRendererNoRunRerun { config } => {
            load_local_sprint94_config(&config, "dashboard-renderer-no-run-rerun").and_then(
                |config| {
                    let report = Sprint94DashboardRendererRecoveryRunner::default()
                        .run_dashboard_renderer_no_run_rerun(&config)?;
                    print_json_report(
                        "dashboard_renderer_warning=no-run rerun is compile-only and never implies full workspace acceptance",
                        &report,
                    )
                },
            )
        }
        Commands::DashboardRendererFullGateRerun { config } => {
            load_local_sprint94_config(&config, "dashboard-renderer-full-gate-rerun").and_then(
                |config| {
                    let report = Sprint94DashboardRendererRecoveryRunner::default()
                        .run_dashboard_renderer_full_gate_rerun(&config)?;
                    print_json_report(
                        "dashboard_renderer_warning=full gate rerun is the quiet workspace gate only and stays honest if still blocked",
                        &report,
                    )
                },
            )
        }
        Commands::DashboardRendererEntryConsumed { config } => {
            load_local_sprint94_config(&config, "dashboard-renderer-entry-consumed").and_then(
                |config| {
                    let report = Sprint94DashboardRendererRecoveryRunner::default()
                        .run_dashboard_renderer_entry_consumed(&config)?;
                    print_json_report(
                        "dashboard_renderer_warning=entry consumption is tied to DashboardRenderer reduction only and never to full workspace acceptance",
                        &report,
                    )
                },
            )
        }
        Commands::SevenBlockerQueueProgressV10 { config } => {
            load_local_sprint94_config(&config, "seven-blocker-queue-progress-v10").and_then(
                |config| {
                    let report = Sprint94DashboardRendererRecoveryRunner::default()
                        .run_seven_blocker_queue_progress_v10(&config)?;
                    print_json_report(
                        "queue_warning=queue progress v10 keeps DashboardRenderer reduction, CommitteeCliSafety isolation, and remaining blockers explicit",
                        &report,
                    )
                },
            )
        }
        Commands::MeasuredTargetDeltaV10 { config } => {
            load_local_sprint94_config(&config, "measured-target-delta-v10").and_then(|config| {
                let report = Sprint94DashboardRendererRecoveryRunner::default()
                    .run_measured_target_delta_v10(&config)?;
                print_json_report(
                    "dashboard_renderer_warning=measured target delta v10 stays explicit when only sample-backed evidence exists",
                    &report,
                )
            })
        }
        Commands::RealNoRunGateAttemptV9 { config } => {
            load_local_sprint94_config(&config, "real-no-run-gate-attempt-v9").and_then(
                |config| {
                    let report = Sprint94DashboardRendererRecoveryRunner::default()
                        .run_real_no_run_gate_attempt_v9(&config)?;
                    print_json_report(
                        "dashboard_renderer_warning=real no-run gate attempt v9 keeps compile-only status honest after DashboardRenderer reduction",
                        &report,
                    )
                },
            )
        }
        Commands::RealFullWorkspaceGateAttemptV12 { config } => {
            load_local_sprint94_config(&config, "real-full-workspace-gate-attempt-v12").and_then(
                |config| {
                    let report = Sprint94DashboardRendererRecoveryRunner::default()
                        .run_real_full_workspace_gate_attempt_v12(&config)?;
                    print_json_report(
                        "dashboard_renderer_warning=real full workspace gate attempt v12 keeps quiet workspace status honest after DashboardRenderer reduction",
                        &report,
                    )
                },
            )
        }
        Commands::WorkspaceGateRecoveryV11 { config } => {
            load_local_sprint94_config(&config, "workspace-gate-recovery-v11").and_then(
                |config| {
                    let report = Sprint94DashboardRendererRecoveryRunner::default()
                        .run_workspace_gate_recovery_v11(&config)?;
                    print_json_report(
                        "workspace_gate_warning=workspace gate recovery v11 keeps DashboardRenderer reduction separate from finished quiet acceptance",
                        &report,
                    )
                },
            )
        }
        Commands::RemainingBlockerQueueV10 { config } => {
            load_local_sprint94_config(&config, "remaining-blocker-queue-v10").and_then(
                |config| {
                    let report = Sprint94DashboardRendererRecoveryRunner::default()
                        .run_remaining_blocker_queue_v10(&config)?;
                    print_json_report(
                        "remaining_queue_warning=remaining blocker queue v10 keeps CommitteeCliSafety isolated and blocked families explicit",
                        &report,
                    )
                },
            )
        }
        Commands::SafetyCoveragePreservationV10 { config } => {
            load_local_sprint94_config(&config, "safety-coverage-preservation-v10").and_then(
                |config| {
                    let report = Sprint94DashboardRendererRecoveryRunner::default()
                        .run_safety_coverage_preservation_v10(&config)?;
                    print_json_report(
                        "safety_warning=safety coverage preservation v10 keeps no-live/no-broker/no-runtime/no-training and dashboard safety guards required",
                        &report,
                    )
                },
            )
        }
        Commands::ControlTowerDashboardRendererRecovery { config } => {
            load_local_sprint94_config(&config, "control-tower-dashboard-renderer-recovery")
                .and_then(|config| {
                    let report = Sprint94DashboardRendererRecoveryRunner::default()
                        .run_control_tower_dashboard_renderer_recovery(&config)?;
                    print_json_report(
                        "read_only_warning=control tower dashboard renderer recovery panel is read-only status output with no train/runtime/live/order/account/browser controls",
                        &report,
                    )
                })
        }
        Commands::Sprint95CommitteeCliSafetyRecover { config } => {
            load_local_sprint95_config(&config, "sprint95-committee-cli-safety-recover").and_then(
                |config| {
                    let report = Sprint95CommitteeCliSafetyRecoveryRunner::default()
                        .run_sprint95_committee_cli_safety_recover(&config)?;
                    print_json_report(
                        "committee_cli_safety_warning=sprint95 CommitteeCliSafety recovery is research-only, CommitteeCliSafety-only, local-only, and never equals full workspace acceptance",
                        &report,
                    )
                },
            )
        }
        Commands::CommitteeCliSafetyReductionPlan { config } => {
            load_local_sprint95_config(&config, "committee-cli-safety-reduction-plan").and_then(
                |config| {
                    let report = Sprint95CommitteeCliSafetyRecoveryRunner::default()
                        .run_committee_cli_safety_reduction_plan(&config)?;
                    print_json_report(
                        "committee_cli_safety_warning=reduction plan preserves assertions and keeps CommitteeCliSafety-only scope explicit",
                        &report,
                    )
                },
            )
        }
        Commands::CommitteeCliSafetyIsolationDecision { config } => {
            load_local_sprint95_config(&config, "committee-cli-safety-isolation-decision")
                .and_then(|config| {
                    let report = Sprint95CommitteeCliSafetyRecoveryRunner::default()
                        .run_committee_cli_safety_isolation_decision(&config)?;
                    print_json_report(
                        "committee_cli_safety_warning=isolation decision keeps permanent sentinel treatment explicit before any queue advancement",
                        &report,
                    )
                })
        }
        Commands::CommitteeCliSafetyAssertionMigration { config } => {
            load_local_sprint95_config(&config, "committee-cli-safety-assertion-migration")
                .and_then(|config| {
                    let report = Sprint95CommitteeCliSafetyRecoveryRunner::default()
                        .run_committee_cli_safety_assertion_migration(&config)?;
                    print_json_report(
                        "committee_cli_safety_warning=assertion migration never deletes assertions and keeps isolated sentinel reasons explicit",
                        &report,
                    )
                })
        }
        Commands::CommitteeCliSafetyFixtureSetupReduction { config } => {
            load_local_sprint95_config(&config, "committee-cli-safety-fixture-setup-reduction")
                .and_then(|config| {
                    let report = Sprint95CommitteeCliSafetyRecoveryRunner::default()
                        .run_committee_cli_safety_fixture_setup_reduction(&config)?;
                    print_json_report(
                        "committee_cli_safety_warning=fixture/setup reduction is local-only and preserves deterministic output",
                        &report,
                    )
                })
        }
        Commands::CommitteeCliSafetyRemotePathPreservation { config } => {
            load_local_sprint95_config(&config, "committee-cli-safety-remote-path-preservation")
                .and_then(|config| {
                    let report = Sprint95CommitteeCliSafetyRecoveryRunner::default()
                        .run_committee_cli_safety_remote_path_preservation(&config)?;
                    print_json_report(
                        "committee_cli_safety_warning=remote paths rejected and local-only paths preserved",
                        &report,
                    )
                })
        }
        Commands::CommitteeCliSafetyHelpTextPreservation { config } => {
            load_local_sprint95_config(&config, "committee-cli-safety-help-text-preservation")
                .and_then(|config| {
                    let report = Sprint95CommitteeCliSafetyRecoveryRunner::default()
                        .run_committee_cli_safety_help_text_preservation(&config)?;
                    print_json_report(
                        "committee_cli_safety_warning=help text stays research-only, paper-only, local-only, no-runtime, and no-training",
                        &report,
                    )
                })
        }
        Commands::CommitteeCliSafetyForbiddenCommandPreservation { config } => {
            load_local_sprint95_config(&config, "committee-cli-safety-forbidden-command-preservation")
                .and_then(|config| {
                    let report = Sprint95CommitteeCliSafetyRecoveryRunner::default()
                        .run_committee_cli_safety_forbidden_command_preservation(&config)?;
                    print_json_report(
                        "committee_cli_safety_warning=forbidden commands stay absent and no train/runtime/live/order/account surface is added",
                        &report,
                    )
                })
        }
        Commands::CommitteeCliSafetyRuntimeDeferredPreservation { config } => {
            load_local_sprint95_config(&config, "committee-cli-safety-runtime-deferred-preservation")
                .and_then(|config| {
                    let report = Sprint95CommitteeCliSafetyRecoveryRunner::default()
                        .run_committee_cli_safety_runtime_deferred_preservation(&config)?;
                    print_json_report(
                        "committee_cli_safety_warning=runtime remains deferred with no training, live inference, or runtime implementation",
                        &report,
                    )
                })
        }
        Commands::CommitteeCliSafetyPersonaExpansionGuard { config } => {
            load_local_sprint95_config(&config, "committee-cli-safety-persona-expansion-guard")
                .and_then(|config| {
                    let report = Sprint95CommitteeCliSafetyRecoveryRunner::default()
                        .run_committee_cli_safety_persona_expansion_guard(&config)?;
                    print_json_report(
                        "committee_cli_safety_warning=persona expansion stays guarded with exactly three active personas and no runtime judge",
                        &report,
                    )
                })
        }
        Commands::CommitteeCliSafetyOrderAccountGuard { config } => {
            load_local_sprint95_config(&config, "committee-cli-safety-order-account-guard")
                .and_then(|config| {
                    let report = Sprint95CommitteeCliSafetyRecoveryRunner::default()
                        .run_committee_cli_safety_order_account_guard(&config)?;
                    print_json_report(
                        "committee_cli_safety_warning=no broker/order/account/balance controls or commands are introduced",
                        &report,
                    )
                })
        }
        Commands::CommitteeCliSafetyBrowserExecutionGuard { config } => {
            load_local_sprint95_config(&config, "committee-cli-safety-browser-execution-guard")
                .and_then(|config| {
                    let report = Sprint95CommitteeCliSafetyRecoveryRunner::default()
                        .run_committee_cli_safety_browser_execution_guard(&config)?;
                    print_json_report(
                        "committee_cli_safety_warning=no dashboard serve, browser execution, POST/action, or JS dependency is introduced",
                        &report,
                    )
                })
        }
        Commands::CommitteeCliSafetyDeterminismPreservation { config } => {
            load_local_sprint95_config(&config, "committee-cli-safety-determinism-preservation")
                .and_then(|config| {
                    let report = Sprint95CommitteeCliSafetyRecoveryRunner::default()
                        .run_committee_cli_safety_determinism_preservation(&config)?;
                    print_json_report(
                        "committee_cli_safety_warning=deterministic CLI help, command surface, and read-only panel output stay required",
                        &report,
                    )
                })
        }
        Commands::CommitteeCliSafetyCompileImpact { config } => {
            load_local_sprint95_config(&config, "committee-cli-safety-compile-impact").and_then(
                |config| {
                    let report = Sprint95CommitteeCliSafetyRecoveryRunner::default()
                        .run_committee_cli_safety_compile_impact(&config)?;
                    print_json_report(
                        "committee_cli_safety_warning=compile impact is sample-backed or measured only and never a fake timing claim",
                        &report,
                    )
                },
            )
        }
        Commands::CommitteeCliSafetyNoRunRerun { config } => {
            load_local_sprint95_config(&config, "committee-cli-safety-no-run-rerun").and_then(
                |config| {
                    let report = Sprint95CommitteeCliSafetyRecoveryRunner::default()
                        .run_committee_cli_safety_no_run_rerun(&config)?;
                    print_json_report(
                        "committee_cli_safety_warning=no-run rerun is compile-only and never implies full workspace acceptance",
                        &report,
                    )
                },
            )
        }
        Commands::CommitteeCliSafetyFullGateRerun { config } => {
            load_local_sprint95_config(&config, "committee-cli-safety-full-gate-rerun").and_then(
                |config| {
                    let report = Sprint95CommitteeCliSafetyRecoveryRunner::default()
                        .run_committee_cli_safety_full_gate_rerun(&config)?;
                    print_json_report(
                        "committee_cli_safety_warning=full gate rerun is the quiet workspace gate only and stays honest if still blocked",
                        &report,
                    )
                },
            )
        }
        Commands::CommitteeCliSafetyEntryConsumed { config } => {
            load_local_sprint95_config(&config, "committee-cli-safety-entry-consumed").and_then(
                |config| {
                    let report = Sprint95CommitteeCliSafetyRecoveryRunner::default()
                        .run_committee_cli_safety_entry_consumed(&config)?;
                    print_json_report(
                        "committee_cli_safety_warning=entry consumption is tied to explicit CommitteeCliSafety closure/isolation only",
                        &report,
                    )
                },
            )
        }
        Commands::BaselineSignalEntryGate { config } => {
            load_local_sprint95_config(&config, "baseline-signal-entry-gate").and_then(|config| {
                let report = Sprint95CommitteeCliSafetyRecoveryRunner::default()
                    .run_baseline_signal_entry_gate(&config)?;
                print_json_report(
                    "baseline_signal_warning=entry gate only; BaselineSignal reduction does not begin in Sprint 95",
                    &report,
                )
            })
        }
        Commands::BaselineSignalReadinessPrecheck { config } => {
            load_local_sprint95_config(&config, "baseline-signal-readiness-precheck")
                .and_then(|config| {
                    let report = Sprint95CommitteeCliSafetyRecoveryRunner::default()
                        .run_baseline_signal_readiness_precheck(&config)?;
                    print_json_report(
                        "baseline_signal_warning=readiness precheck only; no BaselineSignal reduction is performed in Sprint 95",
                        &report,
                    )
                })
        }
        Commands::SevenBlockerQueueProgressV11 { config } => {
            load_local_sprint95_config(&config, "seven-blocker-queue-progress-v11").and_then(
                |config| {
                    let report = Sprint95CommitteeCliSafetyRecoveryRunner::default()
                        .run_seven_blocker_queue_progress_v11(&config)?;
                    print_json_report(
                        "queue_warning=queue progress v11 keeps CommitteeCliSafety closure/isolation, BaselineSignal entry, and remaining blockers explicit",
                        &report,
                    )
                },
            )
        }
        Commands::MeasuredTargetDeltaV11 { config } => {
            load_local_sprint95_config(&config, "measured-target-delta-v11").and_then(|config| {
                let report = Sprint95CommitteeCliSafetyRecoveryRunner::default()
                    .run_measured_target_delta_v11(&config)?;
                print_json_report(
                    "committee_cli_safety_warning=measured target delta v11 stays explicit when only sample-backed evidence exists",
                    &report,
                )
            })
        }
        Commands::RealNoRunGateAttemptV10 { config } => {
            load_local_sprint95_config(&config, "real-no-run-gate-attempt-v10").and_then(
                |config| {
                    let report = Sprint95CommitteeCliSafetyRecoveryRunner::default()
                        .run_real_no_run_gate_attempt_v10(&config)?;
                    print_json_report(
                        "committee_cli_safety_warning=real no-run gate attempt v10 keeps compile-only status honest after CommitteeCliSafety closure/isolation",
                        &report,
                    )
                },
            )
        }
        Commands::RealFullWorkspaceGateAttemptV13 { config } => {
            load_local_sprint95_config(&config, "real-full-workspace-gate-attempt-v13").and_then(
                |config| {
                    let report = Sprint95CommitteeCliSafetyRecoveryRunner::default()
                        .run_real_full_workspace_gate_attempt_v13(&config)?;
                    print_json_report(
                        "committee_cli_safety_warning=real full workspace gate attempt v13 keeps quiet workspace status honest after CommitteeCliSafety closure/isolation",
                        &report,
                    )
                },
            )
        }
        Commands::WorkspaceGateRecoveryV12 { config } => {
            load_local_sprint95_config(&config, "workspace-gate-recovery-v12").and_then(
                |config| {
                    let report = Sprint95CommitteeCliSafetyRecoveryRunner::default()
                        .run_workspace_gate_recovery_v12(&config)?;
                    print_json_report(
                        "workspace_gate_warning=workspace gate recovery v12 keeps CommitteeCliSafety closure/isolation separate from finished quiet acceptance",
                        &report,
                    )
                },
            )
        }
        Commands::RemainingBlockerQueueV11 { config } => {
            load_local_sprint95_config(&config, "remaining-blocker-queue-v11").and_then(
                |config| {
                    let report = Sprint95CommitteeCliSafetyRecoveryRunner::default()
                        .run_remaining_blocker_queue_v11(&config)?;
                    print_json_report(
                        "remaining_queue_warning=remaining blocker queue v11 keeps isolated families and BaselineSignal entry allowance explicit",
                        &report,
                    )
                },
            )
        }
        Commands::SafetyCoveragePreservationV11 { config } => {
            load_local_sprint95_config(&config, "safety-coverage-preservation-v11").and_then(
                |config| {
                    let report = Sprint95CommitteeCliSafetyRecoveryRunner::default()
                        .run_safety_coverage_preservation_v11(&config)?;
                    print_json_report(
                        "safety_warning=safety coverage preservation v11 keeps no-live/no-broker/no-runtime/no-training/no-browser guards required",
                        &report,
                    )
                },
            )
        }
        Commands::ControlTowerCommitteeCliSafetyRecovery { config } => {
            load_local_sprint95_config(&config, "control-tower-committee-cli-safety-recovery")
                .and_then(|config| {
                    let report = Sprint95CommitteeCliSafetyRecoveryRunner::default()
                        .run_control_tower_committee_cli_safety_recovery(&config)?;
                    print_json_report(
                        "read_only_warning=control tower CommitteeCliSafety recovery panel is read-only status output with no train/runtime/live/order/account/browser controls",
                        &report,
                    )
                })
        }
        Commands::Sprint96BaselineSignalRecover { config } => {
            load_local_sprint96_config(&config, "sprint96-baseline-signal-recover").and_then(
                |config| {
                    let report = Sprint96BaselineSignalRecoveryRunner::default()
                        .run_sprint96_baseline_signal_recover(&config)?;
                    print_json_report(
                        "baseline_signal_warning=sprint96 BaselineSignal recovery is research-only, local-only, conservative, and never implies live/runtime/order/account activation",
                        &report,
                    )
                },
            )
        }
        Commands::BaselineSignalRealReductionPlan { config } => {
            load_local_sprint96_config(&config, "baseline-signal-real-reduction-plan").and_then(
                |config| {
                    let report = Sprint96BaselineSignalRecoveryRunner::default()
                        .run_sprint96_baseline_signal_recover(&config)?
                        .baseline_signal_real_reduction_plan;
                    print_json_report(
                        "baseline_signal_warning=reduction plan preserves conservative semantics and keeps CounterfactualBackfill entry-only",
                        &report,
                    )
                },
            )
        }
        Commands::BaselineSignalRealReduction { config } => {
            load_local_sprint96_config(&config, "baseline-signal-real-reduction").and_then(
                |config| {
                    let report = Sprint96BaselineSignalRecoveryRunner::default()
                        .run_sprint96_baseline_signal_recover(&config)?
                        .baseline_signal_real_reduction_report;
                    print_json_report(
                        "baseline_signal_warning=real reduction status stays conservative and separate from workspace acceptance",
                        &report,
                    )
                },
            )
        }
        Commands::BaselineSignalAssertionMigration { config } => {
            load_local_sprint96_config(&config, "baseline-signal-assertion-migration").and_then(
                |config| {
                    let report = Sprint96BaselineSignalRecoveryRunner::default()
                        .run_sprint96_baseline_signal_recover(&config)?
                        .baseline_signal_assertion_migration_report;
                    print_json_report(
                        "baseline_signal_warning=assertion migration is conservative and retains explicit sentinels",
                        &report,
                    )
                },
            )
        }
        Commands::BaselineSignalFixtureSetupReduction { config } => {
            load_local_sprint96_config(&config, "baseline-signal-fixture-setup-reduction")
                .and_then(|config| {
                    let report = Sprint96BaselineSignalRecoveryRunner::default()
                        .run_sprint96_baseline_signal_recover(&config)?
                        .baseline_signal_fixture_setup_reduction_report;
                    print_json_report(
                        "baseline_signal_warning=fixture/setup reduction reuses shared harness and preserves deterministic output",
                        &report,
                    )
                })
        }
        Commands::BaselineSignalFeatureRegimeFlowPreservation { config } => {
            load_local_sprint96_config(&config, "baseline-signal-feature-regime-preservation")
                .and_then(|config| {
                    let report = Sprint96BaselineSignalRecoveryRunner::default()
                        .run_sprint96_baseline_signal_recover(&config)?
                        .baseline_signal_feature_regime_flow_preservation_report;
                    print_json_report(
                        "baseline_signal_warning=feature/regime flow stays explicit and deterministic",
                        &report,
                    )
                })
        }
        Commands::BaselineSignalNoTradeDefaultPreservation { config } => {
            load_local_sprint96_config(&config, "baseline-signal-notrade-default-preservation")
                .and_then(|config| {
                    let report = Sprint96BaselineSignalRecoveryRunner::default()
                        .run_sprint96_baseline_signal_recover(&config)?
                        .baseline_signal_no_trade_default_preservation_report;
                    print_json_report(
                        "baseline_signal_warning=NoTrade remains the conservative default",
                        &report,
                    )
                })
        }
        Commands::BaselineSignalPoorDataQualityDenial { config } => {
            load_local_sprint96_config(&config, "baseline-signal-poor-data-quality-denial")
                .and_then(|config| {
                    let report = Sprint96BaselineSignalRecoveryRunner::default()
                        .run_sprint96_baseline_signal_recover(&config)?
                        .baseline_signal_poor_data_quality_denial_report;
                    print_json_report(
                        "baseline_signal_warning=poor-data-quality denial remains explicit and conservative",
                        &report,
                    )
                })
        }
        Commands::BaselineSignalRiskGovernorVetoPreservation { config } => {
            load_local_sprint96_config(&config, "baseline-signal-risk-governor-veto-preservation")
                .and_then(|config| {
                    let report = Sprint96BaselineSignalRecoveryRunner::default()
                        .run_sprint96_baseline_signal_recover(&config)?
                        .baseline_signal_risk_governor_veto_preservation_report;
                    print_json_report(
                        "baseline_signal_warning=Risk Governor hard veto remains absolute",
                        &report,
                    )
                })
        }
        Commands::BaselineSignalSourceBoundaryPreservation { config } => {
            load_local_sprint96_config(&config, "baseline-signal-source-boundary-preservation")
                .and_then(|config| {
                    let report = Sprint96BaselineSignalRecoveryRunner::default()
                        .run_sprint96_baseline_signal_recover(&config)?
                        .baseline_signal_source_boundary_preservation_report;
                    print_json_report(
                        "baseline_signal_warning=source boundary remains local, research-only, and non-promoting",
                        &report,
                    )
                })
        }
        Commands::BaselineSignalNoLookaheadPreservation { config } => {
            load_local_sprint96_config(&config, "baseline-signal-no-lookahead-preservation")
                .and_then(|config| {
                    let report = Sprint96BaselineSignalRecoveryRunner::default()
                        .run_sprint96_baseline_signal_recover(&config)?
                        .baseline_signal_no_lookahead_preservation_report;
                    print_json_report(
                        "baseline_signal_warning=no-lookahead guarantees remain required",
                        &report,
                    )
                })
        }
        Commands::BaselineSignalResearchOnlyPreservation { config } => {
            load_local_sprint96_config(&config, "baseline-signal-research-only-preservation")
                .and_then(|config| {
                    let report = Sprint96BaselineSignalRecoveryRunner::default()
                        .run_sprint96_baseline_signal_recover(&config)?
                        .baseline_signal_research_only_preservation_report;
                    print_json_report(
                        "baseline_signal_warning=research-only/paper-only/local-only semantics remain required",
                        &report,
                    )
                })
        }
        Commands::BaselineSignalDeterminismPreservation { config } => {
            load_local_sprint96_config(&config, "baseline-signal-determinism-preservation")
                .and_then(|config| {
                    let report = Sprint96BaselineSignalRecoveryRunner::default()
                        .run_sprint96_baseline_signal_recover(&config)?
                        .baseline_signal_determinism_preservation_report;
                    print_json_report(
                        "baseline_signal_warning=deterministic grouped-suite and report output remain required",
                        &report,
                    )
                })
        }
        Commands::BaselineSignalCompileImpact { config } => {
            load_local_sprint96_config(&config, "baseline-signal-compile-impact").and_then(
                |config| {
                    let report = Sprint96BaselineSignalRecoveryRunner::default()
                        .run_sprint96_baseline_signal_recover(&config)?
                        .baseline_signal_compile_impact_report;
                    print_json_report(
                        "baseline_signal_warning=compile impact is sample-backed or measured only and never a fake timing claim",
                        &report,
                    )
                },
            )
        }
        Commands::BaselineSignalNoRunRerun { config } => {
            load_local_sprint96_config(&config, "baseline-signal-no-run-rerun").and_then(
                |config| {
                    let report = Sprint96BaselineSignalRecoveryRunner::default()
                        .run_sprint96_baseline_signal_recover(&config)?
                        .baseline_signal_no_run_gate_rerun_report;
                    print_json_report(
                        "baseline_signal_warning=no-run rerun remains compile-only and separate from quiet full workspace acceptance",
                        &report,
                    )
                },
            )
        }
        Commands::BaselineSignalFullGateRerun { config } => {
            load_local_sprint96_config(&config, "baseline-signal-full-gate-rerun").and_then(
                |config| {
                    let report = Sprint96BaselineSignalRecoveryRunner::default()
                        .run_sprint96_baseline_signal_recover(&config)?
                        .baseline_signal_full_gate_rerun_report;
                    print_json_report(
                        "baseline_signal_warning=full gate rerun stays explicit and never claimed when quiet workspace acceptance did not finish",
                        &report,
                    )
                },
            )
        }
        Commands::BaselineSignalEntryConsumed { config } => {
            load_local_sprint96_config(&config, "baseline-signal-entry-consumed").and_then(
                |config| {
                    let report = Sprint96BaselineSignalRecoveryRunner::default()
                        .run_sprint96_baseline_signal_recover(&config)?
                        .baseline_signal_entry_consumed_report;
                    print_json_report(
                        "baseline_signal_warning=entry consumption is tied to explicit Sprint 95 BaselineSignal entry closure only",
                        &report,
                    )
                },
            )
        }
        Commands::CounterfactualBackfillEntryGate { config } => {
            load_local_sprint96_config(&config, "counterfactual-backfill-entry-gate").and_then(
                |config| {
                    let report = Sprint96BaselineSignalRecoveryRunner::default()
                        .run_sprint96_baseline_signal_recover(&config)?
                        .counterfactual_backfill_entry_gate;
                    print_json_report(
                        "counterfactual_warning=entry gate only; CounterfactualBackfill reduction does not begin in Sprint 96",
                        &report,
                    )
                },
            )
        }
        Commands::CounterfactualBackfillReadinessPrecheck { config } => {
            load_local_sprint96_config(
                &config,
                "counterfactual-backfill-readiness-precheck",
            )
            .and_then(|config| {
                let report = Sprint96BaselineSignalRecoveryRunner::default()
                    .run_sprint96_baseline_signal_recover(&config)?
                    .counterfactual_backfill_readiness_precheck_report;
                print_json_report(
                    "counterfactual_warning=readiness precheck only; CounterfactualBackfill remains entry-only in Sprint 96",
                    &report,
                )
            })
        }
        Commands::SevenBlockerQueueProgressV12 { config } => {
            load_local_sprint96_config(&config, "seven-blocker-queue-progress-v12").and_then(
                |config| {
                    let report = Sprint96BaselineSignalRecoveryRunner::default()
                        .run_sprint96_baseline_signal_recover(&config)?
                        .seven_blocker_queue_progress_report_v12;
                    print_json_report(
                        "queue_warning=queue progress v12 keeps BaselineSignal closure and CounterfactualBackfill next-family status explicit",
                        &report,
                    )
                },
            )
        }
        Commands::MeasuredTargetDeltaV12 { config } => {
            load_local_sprint96_config(&config, "measured-target-delta-v12").and_then(
                |config| {
                    let report = Sprint96BaselineSignalRecoveryRunner::default()
                        .run_sprint96_baseline_signal_recover(&config)?
                        .measured_target_delta_report_v12;
                    print_json_report(
                        "baseline_signal_warning=measured target delta v12 stays explicit when only sample-backed evidence exists",
                        &report,
                    )
                },
            )
        }
        Commands::RealNoRunGateAttemptV11 { config } => {
            load_local_sprint96_config(&config, "real-no-run-gate-attempt-v11").and_then(
                |config| {
                    let report = Sprint96BaselineSignalRecoveryRunner::default()
                        .run_sprint96_baseline_signal_recover(&config)?
                        .real_no_run_gate_attempt_v11;
                    print_json_report(
                        "baseline_signal_warning=real no-run gate attempt v11 keeps compile-only status honest after BaselineSignal reduction",
                        &report,
                    )
                },
            )
        }
        Commands::RealFullWorkspaceGateAttemptV14 { config } => {
            load_local_sprint96_config(&config, "real-full-workspace-gate-attempt-v14").and_then(
                |config| {
                    let report = Sprint96BaselineSignalRecoveryRunner::default()
                        .run_sprint96_baseline_signal_recover(&config)?
                        .real_full_workspace_gate_attempt_v14;
                    print_json_report(
                        "baseline_signal_warning=real full workspace gate attempt v14 keeps quiet workspace status honest after BaselineSignal reduction",
                        &report,
                    )
                },
            )
        }
        Commands::WorkspaceGateRecoveryV13 { config } => {
            load_local_sprint96_config(&config, "workspace-gate-recovery-v13").and_then(
                |config| {
                    let report = Sprint96BaselineSignalRecoveryRunner::default()
                        .run_sprint96_baseline_signal_recover(&config)?
                        .workspace_gate_recovery_v13;
                    print_json_report(
                        "workspace_gate_warning=workspace gate recovery v13 keeps BaselineSignal reduction separate from finished quiet acceptance",
                        &report,
                    )
                },
            )
        }
        Commands::RemainingBlockerQueueV12 { config } => {
            load_local_sprint96_config(&config, "remaining-blocker-queue-v12").and_then(
                |config| {
                    let report = Sprint96BaselineSignalRecoveryRunner::default()
                        .run_sprint96_baseline_signal_recover(&config)?
                        .remaining_blocker_queue_v12;
                    print_json_report(
                        "remaining_queue_warning=remaining blocker queue v12 keeps CounterfactualBackfill entry allowance explicit",
                        &report,
                    )
                },
            )
        }
        Commands::SafetyCoveragePreservationV12 { config } => {
            load_local_sprint96_config(&config, "safety-coverage-preservation-v12").and_then(
                |config| {
                    let report = Sprint96BaselineSignalRecoveryRunner::default()
                        .run_sprint96_baseline_signal_recover(&config)?
                        .safety_coverage_preservation_report_v12;
                    print_json_report(
                        "safety_warning=safety coverage preservation v12 keeps no-live/no-broker/no-runtime/no-training/no-browser guards required",
                        &report,
                    )
                },
            )
        }
        Commands::ControlTowerBaselineSignalRecovery { config } => {
            load_local_sprint96_config(&config, "control-tower-baseline-signal-recovery")
                .and_then(|config| {
                    let report = Sprint96BaselineSignalRecoveryRunner::default()
                        .run_sprint96_baseline_signal_recover(&config)?
                        .control_tower_baseline_signal_recovery_panel;
                    print_json_report(
                        "read_only_warning=control tower BaselineSignal recovery panel is read-only status output with no train/runtime/live/order/account/browser controls",
                        &report,
                    )
                })
        }
        Commands::Sprint97CounterfactualBackfillRecover { config } => {
            load_local_sprint97_config(&config, "sprint97-counterfactual-backfill-recover").and_then(
                |config| {
                    let report = Sprint97CounterfactualBackfillRecoveryRunner::default()
                        .run_sprint97_counterfactual_backfill_recover(&config)?;
                    print_json_report(
                        "counterfactual_warning=sprint97 CounterfactualBackfill recovery is research-only, local-only, deterministic, and never implies live/runtime/order/account activation",
                        &report,
                    )
                },
            )
        }
        Commands::CounterfactualBackfillRealReductionPlan { config } => {
            load_local_sprint97_config(&config, "counterfactual-backfill-real-reduction-plan").and_then(
                |config| {
                    let report = Sprint97CounterfactualBackfillRecoveryRunner::default()
                        .run_sprint97_counterfactual_backfill_recover(&config)?
                        .counterfactual_backfill_real_reduction_plan;
                    print_json_report(
                        "counterfactual_warning=reduction plan preserves NoTrade/RiskDenied/no-lookahead/source-boundary semantics and keeps queue closure separate from workspace acceptance",
                        &report,
                    )
                },
            )
        }
        Commands::CounterfactualBackfillRealReduction { config } => {
            load_local_sprint97_config(&config, "counterfactual-backfill-real-reduction").and_then(
                |config| {
                    let report = Sprint97CounterfactualBackfillRecoveryRunner::default()
                        .run_sprint97_counterfactual_backfill_recover(&config)?
                        .counterfactual_backfill_real_reduction_report;
                    print_json_report(
                        "counterfactual_warning=real reduction status stays conservative and separate from quiet workspace acceptance",
                        &report,
                    )
                },
            )
        }
        Commands::CounterfactualBackfillAssertionMigration { config } => {
            load_local_sprint97_config(&config, "counterfactual-backfill-assertion-migration").and_then(
                |config| {
                    let report = Sprint97CounterfactualBackfillRecoveryRunner::default()
                        .run_sprint97_counterfactual_backfill_recover(&config)?
                        .counterfactual_backfill_assertion_migration_report;
                    print_json_report(
                        "counterfactual_warning=assertion migration is conservative and retains explicit sentinels",
                        &report,
                    )
                },
            )
        }
        Commands::CounterfactualBackfillFixtureSetupReduction { config } => {
            load_local_sprint97_config(&config, "counterfactual-backfill-fixture-setup-reduction").and_then(
                |config| {
                    let report = Sprint97CounterfactualBackfillRecoveryRunner::default()
                        .run_sprint97_counterfactual_backfill_recover(&config)?
                        .counterfactual_backfill_fixture_setup_reduction_report;
                    print_json_report(
                        "counterfactual_warning=fixture/setup reduction reuses shared harness and preserves deterministic output",
                        &report,
                    )
                },
            )
        }
        Commands::CounterfactualBackfillNoTradePreservation { config } => {
            load_local_sprint97_config(&config, "counterfactual-backfill-notrade-preservation").and_then(
                |config| {
                    let report = Sprint97CounterfactualBackfillRecoveryRunner::default()
                        .run_sprint97_counterfactual_backfill_recover(&config)?
                        .counterfactual_backfill_no_trade_preservation_report;
                    print_json_report("counterfactual_warning=NoTrade remains meaningful and may still be the best outcome", &report)
                },
            )
        }
        Commands::CounterfactualBackfillRiskDeniedPreservation { config } => {
            load_local_sprint97_config(&config, "counterfactual-backfill-riskdenied-preservation").and_then(
                |config| {
                    let report = Sprint97CounterfactualBackfillRecoveryRunner::default()
                        .run_sprint97_counterfactual_backfill_recover(&config)?
                        .counterfactual_backfill_risk_denied_preservation_report;
                    print_json_report("counterfactual_warning=RiskDenied remains defensive veto evidence and opportunity cost cannot override Risk Governor", &report)
                },
            )
        }
        Commands::CounterfactualBackfillDefensiveValuePreservation { config } => {
            load_local_sprint97_config(&config, "counterfactual-backfill-defensive-value-preservation").and_then(|config| {
                let report = Sprint97CounterfactualBackfillRecoveryRunner::default().run_sprint97_counterfactual_backfill_recover(&config)?.counterfactual_backfill_defensive_value_preservation_report;
                print_json_report("counterfactual_warning=defensive value remains avoided-loss evidence and is never a profit claim", &report)
            })
        }
        Commands::CounterfactualBackfillOpportunityCostPreservation { config } => {
            load_local_sprint97_config(&config, "counterfactual-backfill-opportunity-cost-preservation").and_then(|config| {
                let report = Sprint97CounterfactualBackfillRecoveryRunner::default().run_sprint97_counterfactual_backfill_recover(&config)?.counterfactual_backfill_opportunity_cost_preservation_report;
                print_json_report("counterfactual_warning=opportunity cost remains visible but cannot override Risk Governor", &report)
            })
        }
        Commands::CounterfactualBackfillNoFabricatedOutcome { config } => {
            load_local_sprint97_config(&config, "counterfactual-backfill-no-fabricated-outcome").and_then(|config| {
                let report = Sprint97CounterfactualBackfillRecoveryRunner::default().run_sprint97_counterfactual_backfill_recover(&config)?.counterfactual_backfill_no_fabricated_outcome_report;
                print_json_report("counterfactual_warning=missing outcomes remain missing and are never fabricated", &report)
            })
        }
        Commands::CounterfactualBackfillNoLookaheadPreservation { config } => {
            load_local_sprint97_config(&config, "counterfactual-backfill-no-lookahead-preservation").and_then(|config| {
                let report = Sprint97CounterfactualBackfillRecoveryRunner::default().run_sprint97_counterfactual_backfill_recover(&config)?.counterfactual_backfill_no_lookahead_preservation_report;
                print_json_report("counterfactual_warning=no-lookahead guarantees remain required", &report)
            })
        }
        Commands::CounterfactualBackfillSourceBoundaryPreservation { config } => {
            load_local_sprint97_config(&config, "counterfactual-backfill-source-boundary-preservation").and_then(|config| {
                let report = Sprint97CounterfactualBackfillRecoveryRunner::default().run_sprint97_counterfactual_backfill_recover(&config)?.counterfactual_backfill_source_boundary_preservation_report;
                print_json_report("counterfactual_warning=source boundary remains local, research-only, and non-promoting", &report)
            })
        }
        Commands::CounterfactualBackfillResearchOnlyPreservation { config } => {
            load_local_sprint97_config(&config, "counterfactual-backfill-research-only-preservation").and_then(|config| {
                let report = Sprint97CounterfactualBackfillRecoveryRunner::default().run_sprint97_counterfactual_backfill_recover(&config)?.counterfactual_backfill_research_only_preservation_report;
                print_json_report("counterfactual_warning=research-only/paper-only/local-only semantics remain required", &report)
            })
        }
        Commands::CounterfactualBackfillDeterminismPreservation { config } => {
            load_local_sprint97_config(&config, "counterfactual-backfill-determinism-preservation").and_then(|config| {
                let report = Sprint97CounterfactualBackfillRecoveryRunner::default().run_sprint97_counterfactual_backfill_recover(&config)?.counterfactual_backfill_determinism_preservation_report;
                print_json_report("counterfactual_warning=deterministic grouped-suite and report output remain required", &report)
            })
        }
        Commands::CounterfactualBackfillCompileImpact { config } => {
            load_local_sprint97_config(&config, "counterfactual-backfill-compile-impact").and_then(|config| {
                let report = Sprint97CounterfactualBackfillRecoveryRunner::default().run_sprint97_counterfactual_backfill_recover(&config)?.counterfactual_backfill_compile_impact_report;
                print_json_report("counterfactual_warning=compile impact is sample-backed or measured only and never a fake timing claim", &report)
            })
        }
        Commands::CounterfactualBackfillNoRunRerun { config } => {
            load_local_sprint97_config(&config, "counterfactual-backfill-no-run-rerun").and_then(|config| {
                let report = Sprint97CounterfactualBackfillRecoveryRunner::default().run_sprint97_counterfactual_backfill_recover(&config)?.counterfactual_backfill_no_run_gate_rerun_report;
                print_json_report("counterfactual_warning=no-run rerun remains compile-only and separate from quiet full workspace acceptance", &report)
            })
        }
        Commands::CounterfactualBackfillFullGateRerun { config } => {
            load_local_sprint97_config(&config, "counterfactual-backfill-full-gate-rerun").and_then(|config| {
                let report = Sprint97CounterfactualBackfillRecoveryRunner::default().run_sprint97_counterfactual_backfill_recover(&config)?.counterfactual_backfill_full_gate_rerun_report;
                print_json_report("counterfactual_warning=full gate rerun stays explicit and never claimed when quiet workspace acceptance did not finish", &report)
            })
        }
        Commands::CounterfactualBackfillEntryConsumed { config } => {
            load_local_sprint97_config(&config, "counterfactual-backfill-entry-consumed").and_then(|config| {
                let report = Sprint97CounterfactualBackfillRecoveryRunner::default().run_sprint97_counterfactual_backfill_recover(&config)?.counterfactual_backfill_entry_consumed_report;
                print_json_report("counterfactual_warning=entry consumption is tied to explicit Sprint 96 CounterfactualBackfill entry closure only", &report)
            })
        }
        Commands::FinalBlockerQueueClosureGate { config } => {
            load_local_sprint97_config(&config, "final-blocker-queue-closure-gate").and_then(|config| {
                let report = Sprint97CounterfactualBackfillRecoveryRunner::default().run_sprint97_counterfactual_backfill_recover(&config)?.final_blocker_queue_closure_gate;
                print_json_report("queue_warning=final queue closure gate stays separate from workspace acceptance truth", &report)
            })
        }
        Commands::FinalBlockerQueueClosure { config } => {
            load_local_sprint97_config(&config, "final-blocker-queue-closure").and_then(|config| {
                let report = Sprint97CounterfactualBackfillRecoveryRunner::default().run_sprint97_counterfactual_backfill_recover(&config)?.final_blocker_queue_closure_report;
                print_json_report("queue_warning=final blocker queue closure does not claim full workspace acceptance", &report)
            })
        }
        Commands::WorkspaceAcceptanceTruthGate { config } => {
            load_local_sprint97_config(&config, "workspace-acceptance-truth-gate").and_then(|config| {
                let report = Sprint97CounterfactualBackfillRecoveryRunner::default().run_sprint97_counterfactual_backfill_recover(&config)?.workspace_acceptance_truth_gate;
                print_json_report("workspace_gate_warning=workspace acceptance truth stays explicit and requires real quiet full workspace pass", &report)
            })
        }
        Commands::WorkspaceAcceptanceRemainingRisk { config } => {
            load_local_sprint97_config(&config, "workspace-acceptance-remaining-risk").and_then(|config| {
                let report = Sprint97CounterfactualBackfillRecoveryRunner::default().run_sprint97_counterfactual_backfill_recover(&config)?.workspace_acceptance_remaining_risk_report;
                print_json_report("workspace_gate_warning=remaining workspace risk stays explicit until quiet full workspace acceptance passes", &report)
            })
        }
        Commands::SevenBlockerQueueProgressV13 { config } => {
            load_local_sprint97_config(&config, "seven-blocker-queue-progress-v13").and_then(|config| {
                let report = Sprint97CounterfactualBackfillRecoveryRunner::default().run_sprint97_counterfactual_backfill_recover(&config)?.seven_blocker_queue_progress_report_v13;
                print_json_report("queue_warning=queue progress v13 keeps final queue closure and workspace acceptance distinction explicit", &report)
            })
        }
        Commands::MeasuredTargetDeltaV13 { config } => {
            load_local_sprint97_config(&config, "measured-target-delta-v13").and_then(|config| {
                let report = Sprint97CounterfactualBackfillRecoveryRunner::default().run_sprint97_counterfactual_backfill_recover(&config)?.measured_target_delta_report_v13;
                print_json_report("counterfactual_warning=measured target delta v13 stays explicit when only sample-backed evidence exists", &report)
            })
        }
        Commands::RealNoRunGateAttemptV12 { config } => {
            load_local_sprint97_config(&config, "real-no-run-gate-attempt-v12").and_then(|config| {
                let report = Sprint97CounterfactualBackfillRecoveryRunner::default().run_sprint97_counterfactual_backfill_recover(&config)?.real_no_run_gate_attempt_v12;
                print_json_report("counterfactual_warning=real no-run gate attempt v12 keeps compile-only status honest after CounterfactualBackfill reduction", &report)
            })
        }
        Commands::RealFullWorkspaceGateAttemptV15 { config } => {
            load_local_sprint97_config(&config, "real-full-workspace-gate-attempt-v15").and_then(|config| {
                let report = Sprint97CounterfactualBackfillRecoveryRunner::default().run_sprint97_counterfactual_backfill_recover(&config)?.real_full_workspace_gate_attempt_v15;
                print_json_report("counterfactual_warning=real full workspace gate attempt v15 keeps quiet workspace status honest after CounterfactualBackfill reduction", &report)
            })
        }
        Commands::WorkspaceGateRecoveryV14 { config } => {
            load_local_sprint97_config(&config, "workspace-gate-recovery-v14").and_then(|config| {
                let report = Sprint97CounterfactualBackfillRecoveryRunner::default().run_sprint97_counterfactual_backfill_recover(&config)?.workspace_gate_recovery_v14;
                print_json_report("workspace_gate_warning=workspace gate recovery v14 keeps queue closure separate from finished quiet acceptance", &report)
            })
        }
        Commands::RemainingBlockerQueueV13 { config } => {
            load_local_sprint97_config(&config, "remaining-blocker-queue-v13").and_then(|config| {
                let report = Sprint97CounterfactualBackfillRecoveryRunner::default().run_sprint97_counterfactual_backfill_recover(&config)?.remaining_blocker_queue_v13;
                print_json_report("remaining_queue_warning=remaining blocker queue v13 keeps final queue closure and workspace acceptance claim status explicit", &report)
            })
        }
        Commands::SafetyCoveragePreservationV13 { config } => {
            load_local_sprint97_config(&config, "safety-coverage-preservation-v13").and_then(|config| {
                let report = Sprint97CounterfactualBackfillRecoveryRunner::default().run_sprint97_counterfactual_backfill_recover(&config)?.safety_coverage_preservation_report_v13;
                print_json_report("safety_warning=safety coverage preservation v13 keeps no-live/no-broker/no-runtime/no-training/no-browser guards required", &report)
            })
        }
        Commands::ControlTowerCounterfactualBackfillRecovery { config } => {
            load_local_sprint97_config(&config, "control-tower-counterfactual-backfill-recovery").and_then(|config| {
                let report = Sprint97CounterfactualBackfillRecoveryRunner::default().run_sprint97_counterfactual_backfill_recover(&config)?.control_tower_counterfactual_backfill_recovery_panel;
                print_json_report("read_only_warning=control tower CounterfactualBackfill recovery panel is read-only status output with no train/runtime/live/order/account/browser controls", &report)
            })
        }
        Commands::CommitteeOwnedCoreArchitecture { config } => {
            load_local_sprint98_config(&config, "committee-owned-core-architecture").and_then(
                |config| {
                    let report = Sprint98CommitteeOwnedCoreRunner::default()
                        .run_sprint98_committee_owned_core(&config)?
                        .committee_owned_ai_core_architecture;
                    print_json_report(
                        "architecture_warning=central core deprecated; each committee member owns its own AI core in research-only paper mode",
                        &report,
                    )
                },
            )
        }
        Commands::InvestorStyleRegistry { config } => {
            load_local_sprint98_config(&config, "investor-style-registry").and_then(|config| {
                let report = Sprint98CommitteeOwnedCoreRunner::default()
                    .run_sprint98_committee_owned_core(&config)?
                    .investor_style_archetype_registry;
                print_json_report(
                    "style_warning=public philosophy-inspired archetypes only; no impersonation or proprietary strategy claims",
                    &report,
                )
            })
        }
        Commands::AiCommitteeMemberSpecs { config } => {
            load_local_sprint98_config(&config, "ai-committee-member-specs").and_then(|config| {
                let report = Sprint98CommitteeOwnedCoreRunner::default()
                    .run_sprint98_committee_owned_core(&config)?
                    .ai_committee_member_specs;
                print_json_report(
                    "member_warning=each AI committee member owns its own core contract and stays paper-only",
                    &report,
                )
            })
        }
        Commands::CommitteeMemberCoreContracts { config } => {
            load_local_sprint98_config(&config, "committee-member-core-contracts").and_then(
                |config| {
                    let report = Sprint98CommitteeOwnedCoreRunner::default()
                        .run_sprint98_committee_owned_core(&config)?
                        .ai_committee_member_core_contracts;
                    print_json_report(
                        "core_warning=member-owned core contracts keep runtime, training, and live inference deferred",
                        &report,
                    )
                },
            )
        }
        Commands::CommitteeMemberLearningPolicy { config } => {
            load_local_sprint98_config(&config, "committee-member-learning-policy").and_then(
                |config| {
                    let report = Sprint98CommitteeOwnedCoreRunner::default()
                        .run_sprint98_committee_owned_core(&config)?
                        .ai_committee_member_learning_policies;
                    print_json_report(
                        "learning_warning=offline study only; no broker/account access and no model training",
                        &report,
                    )
                },
            )
        }
        Commands::CommitteeMemberProposals { config } => {
            load_local_sprint98_config(&config, "committee-member-proposals").and_then(|config| {
                let report = Sprint98CommitteeOwnedCoreRunner::default()
                    .run_sprint98_committee_owned_core(&config)?
                    .ai_committee_member_proposals;
                print_json_report(
                    "proposal_warning=entry timing proposals stay paper-only, local-only, and never imply execution",
                    &report,
                )
            })
        }
        Commands::EntryTimingProposals { config } => {
            load_local_sprint98_config(&config, "entry-timing-proposals").and_then(|config| {
                let report = Sprint98CommitteeOwnedCoreRunner::default()
                    .run_sprint98_committee_owned_core(&config)?
                    .entry_timing_proposals;
                print_json_report(
                    "timing_warning=entry timing windows are proposal artifacts only and never broker orders",
                    &report,
                )
            })
        }
        Commands::CommitteeDebateTrigger { config } => {
            load_local_sprint98_config(&config, "committee-debate-trigger").and_then(|config| {
                let report = Sprint98CommitteeOwnedCoreRunner::default()
                    .run_sprint98_committee_owned_core(&config)?
                    .committee_debate_trigger;
                print_json_report(
                    "debate_warning=member proposals can trigger paper-only debate and never live trading",
                    &report,
                )
            })
        }
        Commands::CommitteeDebateSession { config } => {
            load_local_sprint98_config(&config, "committee-debate-session").and_then(|config| {
                let report = Sprint98CommitteeOwnedCoreRunner::default()
                    .run_sprint98_committee_owned_core(&config)?
                    .committee_debate_session;
                print_json_report(
                    "debate_warning=committee debate stays paper-only with support, oppose, wait, and risk-deny turns",
                    &report,
                )
            })
        }
        Commands::ChairmanGovernancePolicy { config } => {
            load_local_sprint98_config(&config, "chairman-governance-policy").and_then(
                |config| {
                    let report = Sprint98CommitteeOwnedCoreRunner::default()
                        .run_sprint98_committee_owned_core(&config)?
                        .chairman_ai_governance_policy;
                    print_json_report(
                        "chair_warning=chairman governance is audited paper-only policy and cannot bypass Risk Governor",
                        &report,
                    )
                },
            )
        }
        Commands::ChairmanRuleProposal { config } => {
            load_local_sprint98_config(&config, "chairman-rule-proposal").and_then(|config| {
                let report = Sprint98CommitteeOwnedCoreRunner::default()
                    .run_sprint98_committee_owned_core(&config)?
                    .chairman_rule_proposals;
                print_json_report(
                    "chair_warning=chairman rule proposals require audit and remain paper-only",
                    &report,
                )
            })
        }
        Commands::ChairmanRulebookVersion { config } => {
            load_local_sprint98_config(&config, "chairman-rulebook-version").and_then(
                |config| {
                    let report = Sprint98CommitteeOwnedCoreRunner::default()
                        .run_sprint98_committee_owned_core(&config)?
                        .chairman_rulebook_version;
                    print_json_report(
                        "chair_warning=rulebook versions are audited, versioned, and forbidden from live use",
                        &report,
                    )
                },
            )
        }
        Commands::RuleAdaptationAudit { config } => {
            load_local_sprint98_config(&config, "rule-adaptation-audit").and_then(|config| {
                let report = Sprint98CommitteeOwnedCoreRunner::default()
                    .run_sprint98_committee_owned_core(&config)?
                    .rule_adaptation_audit;
                print_json_report(
                    "audit_warning=rule adaptation stays research-only and checks safety plus overfit risk",
                    &report,
                )
            })
        }
        Commands::PromotionDemotionPolicy { config } => {
            load_local_sprint98_config(&config, "promotion-demotion-policy").and_then(
                |config| {
                    let report = Sprint98CommitteeOwnedCoreRunner::default()
                        .run_sprint98_committee_owned_core(&config)?
                        .promotion_demotion_policy;
                    print_json_report(
                        "promotion_warning=promotion/demotion is multi-axis and not raw-profit-only",
                        &report,
                    )
                },
            )
        }
        Commands::MemberScorecards { config } => {
            load_local_sprint98_config(&config, "member-scorecards").and_then(|config| {
                let report = Sprint98CommitteeOwnedCoreRunner::default()
                    .run_sprint98_committee_owned_core(&config)?
                    .multi_axis_member_scorecards;
                print_json_report(
                    "scorecard_warning=member scorecards rank paper-only performance, calibration, risk, and debate quality",
                    &report,
                )
            })
        }
        Commands::MemberPromotionDemotion { config } => {
            load_local_sprint98_config(&config, "member-promotion-demotion").and_then(
                |config| {
                    let report = Sprint98CommitteeOwnedCoreRunner::default()
                        .run_sprint98_committee_owned_core(&config)?
                        .member_promotion_demotion_decisions;
                    print_json_report(
                        "promotion_warning=member promotion/demotion decisions govern paper roster status only",
                        &report,
                    )
                },
            )
        }
        Commands::CommitteeRosterLifecycle { config } => {
            load_local_sprint98_config(&config, "committee-roster-lifecycle").and_then(
                |config| {
                    let report = Sprint98CommitteeOwnedCoreRunner::default()
                        .run_sprint98_committee_owned_core(&config)?
                        .committee_roster_lifecycle;
                    print_json_report(
                        "roster_warning=committee roster lifecycle remains research-only and preserves isolated safety sentinels",
                        &report,
                    )
                },
            )
        }
        Commands::PaperOnlyCommitteeDecision { config } => {
            load_local_sprint98_config(&config, "paper-only-committee-decision").and_then(
                |config| {
                    let report = Sprint98CommitteeOwnedCoreRunner::default()
                        .run_sprint98_committee_owned_core(&config)?
                        .paper_only_committee_decision_record;
                    print_json_report(
                        "paper_warning=paper-only committee decisions never allow broker execution or live trading",
                        &report,
                    )
                },
            )
        }
        Commands::ControlTowerAiCommittee { config } => {
            load_local_sprint98_config(&config, "control-tower-ai-committee").and_then(
                |config| {
                    let report = Sprint98CommitteeOwnedCoreRunner::default()
                        .run_sprint98_committee_owned_core(&config)?
                        .control_tower_ai_committee_panel;
                    print_json_report(
                        "read_only_warning=control tower AI committee panel is static read-only output with no train/runtime/live/order/account/browser controls",
                        &report,
                    )
                },
            )
        }
        Commands::Sprint98CommitteeOwnedCore { config } => {
            load_local_sprint98_config(&config, "sprint98-committee-owned-core").and_then(
                |config| {
                    let report = Sprint98CommitteeOwnedCoreRunner::default()
                        .run_sprint98_committee_owned_core(&config)?;
                    print_json_report(
                        "sprint98_warning=committee-owned-core architecture correction is research-only, paper-only, local-only, with no central-core assumption and no runtime/training/live implementation",
                        &report,
                    )
                },
            )
        }
        Commands::Sprint99CommitteeQualityHarden { config } => {
            run_sprint99_bundle(&config, "sprint99-committee-quality-harden").and_then(|report| {
                print_json_report(
                    "sprint99_warning=paper-only committee quality hardening only; no central AI core, no runtime implementation, no training, no live inference, and no live trading",
                    &report,
                )
            })
        }
        Commands::CommitteeMemberProposalQuality { config } => {
            run_sprint99_bundle(&config, "committee-member-proposal-quality").and_then(|report| {
                print_json_report(
                    "proposal_quality_warning=proposal quality is research-only paper review; proposal is not order execution",
                    &report.committee_member_proposal_quality_report,
                )
            })
        }
        Commands::EntryTimingProposalQuality { config } => {
            run_sprint99_bundle(&config, "entry-timing-proposal-quality").and_then(|report| {
                print_json_report(
                    "entry_timing_warning=entry timing proposals stay paper-only and never become broker orders",
                    &report.entry_timing_proposal_quality_report,
                )
            })
        }
        Commands::CommitteeDebateQuality { config } => {
            run_sprint99_bundle(&config, "committee-debate-quality").and_then(|report| {
                print_json_report(
                    "debate_quality_warning=committee debate quality is paper-only and does not enable live LLM debate",
                    &report.committee_debate_quality_report,
                )
            })
        }
        Commands::DebateEvidenceSufficiency { config } => {
            run_sprint99_bundle(&config, "debate-evidence-sufficiency").and_then(|report| {
                print_json_report(
                    "evidence_warning=debate evidence sufficiency is local-only, source-bound, and not a live data entitlement",
                    &report.debate_evidence_sufficiency_report,
                )
            })
        }
        Commands::ChairmanRulebookQuality { config } => {
            run_sprint99_bundle(&config, "chairman-rulebook-quality").and_then(|report| {
                print_json_report(
                    "rulebook_warning=chairman rulebook quality remains paper-only with no live rule mutation and no Risk Governor bypass",
                    &report.chairman_rulebook_quality_report,
                )
            })
        }
        Commands::ChairmanRuleRiskAuditV2 { config } => {
            run_sprint99_bundle(&config, "chairman-rule-risk-audit-v2").and_then(|report| {
                print_json_report(
                    "rule_audit_warning=chairman rule risk audit v2 keeps governance audited, paper-only, and blocked from live use",
                    &report.chairman_rule_proposal_risk_audit_v2,
                )
            })
        }
        Commands::RulebookVersionDiff { config } => {
            run_sprint99_bundle(&config, "rulebook-version-diff").and_then(|report| {
                print_json_report(
                    "rulebook_diff_warning=rulebook version diff reports paper-only deltas and never live rule application",
                    &report.rulebook_version_diff_report,
                )
            })
        }
        Commands::PromotionDemotionCalibration { config } => {
            run_sprint99_bundle(&config, "promotion-demotion-calibration").and_then(|report| {
                print_json_report(
                    "promotion_warning=promotion/demotion calibration is roster research only and not capital allocation",
                    &report.promotion_demotion_calibration_report,
                )
            })
        }
        Commands::MemberScorecardCalibration { config } => {
            run_sprint99_bundle(&config, "member-scorecard-calibration").and_then(|report| {
                print_json_report(
                    "scorecard_warning=member scorecard calibration is research-only and never auto-promotes anything to live trading",
                    &report.member_scorecard_calibration_report,
                )
            })
        }
        Commands::MemberOverfitRisk { config } => {
            run_sprint99_bundle(&config, "member-overfit-risk").and_then(|report| {
                print_json_report(
                    "overfit_warning=member overfit risk is reported for paper governance only; no model training exists here",
                    &report.member_overfit_risk_report,
                )
            })
        }
        Commands::MemberStyleDrift { config } => {
            run_sprint99_bundle(&config, "member-style-drift").and_then(|report| {
                print_json_report(
                    "style_warning=style drift review preserves public-philosophy archetypes only and no investor impersonation",
                    &report.member_style_drift_report,
                )
            })
        }
        Commands::InvestorStyleBlindspot { config } => {
            run_sprint99_bundle(&config, "investor-style-blindspot").and_then(|report| {
                print_json_report(
                    "blindspot_warning=investor style blindspots are safety documentation only with no private strategy or impersonation claim",
                    &report.investor_style_blindspot_report,
                )
            })
        }
        Commands::CommitteeRosterBalance { config } => {
            run_sprint99_bundle(&config, "committee-roster-balance").and_then(|report| {
                print_json_report(
                    "roster_warning=committee roster balance remains paper-only research status with no live capital allocation",
                    &report.committee_roster_balance_report,
                )
            })
        }
        Commands::PaperOnlyDecisionReplay { config } => {
            run_sprint99_bundle(&config, "paper-only-decision-replay").and_then(|report| {
                print_json_report(
                    "replay_warning=paper-only decision replay never touches broker/order/account paths",
                    &report.paper_only_decision_replay_report,
                )
            })
        }
        Commands::PaperDecisionTraceCompleteness { config } => {
            run_sprint99_bundle(&config, "paper-decision-trace-completeness").and_then(|report| {
                print_json_report(
                    "trace_warning=paper decision trace completeness is audit-only and never an execution path",
                    &report.paper_decision_trace_completeness_report,
                )
            })
        }
        Commands::RiskGovernorDebateHandoff { config } => {
            run_sprint99_bundle(&config, "risk-governor-debate-handoff").and_then(|report| {
                print_json_report(
                    "risk_warning=Risk Governor debate handoff preserves final veto and no bypass path",
                    &report.risk_governor_debate_handoff_report,
                )
            })
        }
        Commands::CommitteeArchitectureRegressionGuard { config } => {
            run_sprint99_bundle(&config, "committee-architecture-regression-guard").and_then(
                |report| {
                    print_json_report(
                        "architecture_guard_warning=no central AI core, no runtime leak, no training leak, and no live execution path allowed",
                        &report.committee_owned_architecture_regression_guard,
                    )
                },
            )
        }
        Commands::WorkspaceAcceptanceTruthClosurePlan { config } => {
            run_sprint99_bundle(&config, "workspace-acceptance-truth-closure-plan").and_then(
                |report| {
                    print_json_report(
                        "workspace_truth_warning=full workspace acceptance remains separate; focused tests cannot claim full acceptance",
                        &report.workspace_acceptance_truth_closure_plan,
                    )
                },
            )
        }
        Commands::WorkspaceAcceptanceAttemptV16 { config } => {
            run_sprint99_bundle(&config, "workspace-acceptance-attempt-v16").and_then(|report| {
                print_json_report(
                    "workspace_attempt_warning=workspace acceptance attempt v16 is an honest record only; full workspace requires real cargo test --workspace --quiet completion",
                    &report.workspace_acceptance_attempt_v16,
                )
            })
        }
        Commands::SafetyCoveragePreservationV15 { config } => {
            run_sprint99_bundle(&config, "safety-coverage-preservation-v15").and_then(|report| {
                print_json_report(
                    "safety_warning=safety coverage v15 preserves no live trading, no broker/order/account, no runtime LLM path, and no browser execution",
                    &report.safety_coverage_preservation_report_v15,
                )
            })
        }
        Commands::ControlTowerAiCommitteeQuality { config } => {
            run_sprint99_bundle(&config, "control-tower-ai-committee-quality").and_then(
                |report| {
                    print_json_report(
                        "read_only_warning=control tower AI committee quality panel is static read-only output with no train/runtime/live/order/account/browser controls",
                        &report.control_tower_ai_committee_quality_panel,
                    )
                },
            )
        }
        Commands::Sprint100CommitteeClosure { config } => {
            run_sprint100_bundle(&config, "sprint100-committee-closure").and_then(|report| {
                print_json_report(
                    "sprint100_warning=research-only paper-only committee warning closure; no central AI core, no runtime implementation, no training, no live inference, no live trading, and no order/account command",
                    &report,
                )
            })
        }
        Commands::Sprint101InvestorArchetypeIngest { config } => print_sprint101_report(
            &config,
            "sprint101-investor-archetype-ingest",
            "sprint101_warning=research-only paper-only investor-archetype ingestion; no impersonation, no central AI core, no runtime implementation, no training, no live inference, no live trading, no order/account command, and no auto-activation of 18 live agents",
            |report| report,
        ),
        Commands::InvestorArchetypeIngestion { config } => print_sprint101_report(
            &config,
            "investor-archetype-ingestion",
            "investor_archetype_warning=archetype ingestion is public-philosophy-inspired only, not investor impersonation, training, runtime, or live trading",
            |report| report.investor_archetype_ingestion_report,
        ),
        Commands::InvestorSourceConfidence { config } => print_sprint101_report(
            &config,
            "investor-source-confidence",
            "source_confidence_warning=source confidence is local-only research weighting and not a live inference or training path",
            |report| report.investor_archetype_source_confidence_report,
        ),
        Commands::InvestorSafetyNormalization { config } => print_sprint101_report(
            &config,
            "investor-safety-normalization",
            "safety_normalization_warning=safety normalization filters impersonation, unsupported claims, and myths without runtime implementation",
            |report| report.investor_archetype_safety_normalization_report,
        ),
        Commands::InvestorFeatureVectorCards { config } => print_sprint101_report(
            &config,
            "investor-feature-vector-cards",
            "feature_vector_warning=feature vector cards are paper-only archetype cards and not trained models or live agents",
            |report| report.investor_style_feature_vector_cards,
        ),
        Commands::InvestorDoNotLearnGuards { config } => print_sprint101_report(
            &config,
            "investor-do-not-learn-guards",
            "do_not_learn_warning=do-not-learn guards block private-life myths, unsupported claims, and unsafe numeric rules with no training path",
            |report| report.investor_style_do_not_learn_guards,
        ),
        Commands::InvestorImpersonationRisk { config } => print_sprint101_report(
            &config,
            "investor-impersonation-risk",
            "impersonation_warning=impersonation risk review preserves archetype-only wording and forbids exact investor clones",
            |report| report.investor_impersonation_risk_report,
        ),
        Commands::InvestorUnverifiedClaimFilter { config } => print_sprint101_report(
            &config,
            "investor-unverified-claim-filter",
            "claim_filter_warning=unsupported claims and unofficial quotes are filtered from archetype learning scope",
            |report| report.investor_unverified_claim_filter_report,
        ),
        Commands::InvestorPrivateLifeMythFilter { config } => print_sprint101_report(
            &config,
            "investor-private-life-myth-filter",
            "private_life_warning=private-life myths are removed and only useful auditable routines may be preserved",
            |report| report.investor_private_life_myth_filter_report,
        ),
        Commands::EighteenInvestorRegistry { config } => print_sprint101_report(
            &config,
            "eighteen-investor-registry",
            "registry_warning=18-investor registry is a paper-only research registry and does not imply 18 live AI agents",
            |report| report.eighteen_investor_candidate_registry,
        ),
        Commands::StyleGroupTaxonomy { config } => print_sprint101_report(
            &config,
            "style-group-taxonomy",
            "taxonomy_warning=style taxonomy keeps short-term, long-term, crypto, and common risk logic separated",
            |report| report.style_group_taxonomy_report,
        ),
        Commands::StyleConflictMatrix { config } => print_sprint101_report(
            &config,
            "style-conflict-matrix",
            "conflict_warning=style conflict routing is paper-only and keeps Risk Governor final veto intact",
            |report| report.style_conflict_matrix,
        ),
        Commands::RegimeRoutingPolicy { config } => print_sprint101_report(
            &config,
            "regime-routing-policy",
            "routing_warning=regime routing is research-only and never a runtime LLM live decision path",
            |report| report.regime_routing_policy,
        ),
        Commands::MultiExpertCommitteeTopology { config } => print_sprint101_report(
            &config,
            "multi-expert-committee-topology",
            "topology_warning=multi-expert topology preserves committee-owned cores and no central AI core regression",
            |report| report.multi_expert_committee_topology,
        ),
        Commands::MemberConfidenceWeightPolicy { config } => print_sprint101_report(
            &config,
            "member-confidence-weight-policy",
            "confidence_weight_warning=confidence weights reflect source reliability only and do not grant trade authority",
            |report| report.member_style_confidence_weight_policy,
        ),
        Commands::MemberFeatureScopeMapping { config } => print_sprint101_report(
            &config,
            "member-feature-scope-mapping",
            "feature_scope_warning=feature scope mapping is paper-only research structure with no training or runtime use",
            |report| report.member_feature_scope_mapping_report,
        ),
        Commands::MemberLearningDataCards { config } => print_sprint101_report(
            &config,
            "member-learning-data-cards",
            "learning_data_warning=learning data cards are offline-study-only and not model-training or deployment artifacts",
            |report| report.member_learning_data_card_report,
        ),
        Commands::ArchetypeToMemberMapping { config } => print_sprint101_report(
            &config,
            "archetype-to-member-mapping",
            "mapping_warning=archetype mapping is staged paper roster design only with no auto-activation of 18 live agents",
            |report| report.archetype_to_committee_member_mapping_report,
        ),
        Commands::EighteenRosterPlan { config } => print_sprint101_report(
            &config,
            "eighteen-roster-plan",
            "roster_warning=18-member roster planning is research-only paper staging and not live capital allocation",
            |report| report.eighteen_investor_committee_roster_plan,
        ),
        Commands::EighteenActivationGate { config } => print_sprint101_report(
            &config,
            "eighteen-activation-gate",
            "activation_warning=18-member activation gate is paper-only and explicitly forbids live activation",
            |report| report.eighteen_member_activation_gate,
        ),
        Commands::PaperRosterExpansionGate { config } => print_sprint101_report(
            &config,
            "paper-roster-expansion-gate",
            "paper_roster_warning=paper roster expansion is separate from live activation and keeps workspace truth separate",
            |report| report.paper_only_roster_expansion_gate,
        ),
        Commands::ChairmanStyleGovernanceV2 { config } => print_sprint101_report(
            &config,
            "chairman-style-governance-v2",
            "chairman_style_warning=chairman governance remains paper-only, audit-gated, and cannot bypass Risk Governor",
            |report| report.chairman_style_governance_policy_v2,
        ),
        Commands::PromotionDemotionPolicyV2 { config } => print_sprint101_report(
            &config,
            "promotion-demotion-policy-v2",
            "promotion_v2_warning=promotion/demotion v2 is research roster management only and not live capital allocation",
            |report| report.promotion_demotion_policy_v2_for_18_styles,
        ),
        Commands::ControlTowerInvestorArchetype { config } => print_sprint101_report(
            &config,
            "control-tower-investor-archetype",
            "read_only_warning=control tower investor archetype panel is static read-only output with no train/runtime/live/order/account/browser controls or activate-all-live button",
            |report| report.control_tower_investor_archetype_panel,
        ),
        Commands::Sprint102PaperRotation { config } => print_sprint102_report(
            &config,
            "sprint102-paper-rotation",
            "sprint102_warning=research-only paper-only paper-rotation dry-run; no impersonation, no central AI core, no runtime implementation, no training, no live inference, no live trading, no order/account command, and no auto-activation of 18 live agents",
            |report| report,
        ),
        Commands::PaperRotationScenarioPack { config } => print_sprint102_report(
            &config,
            "paper-rotation-scenario-pack",
            "paper_rotation_warning=paper-only rotation scenario pack is local-only research scaffolding and not live activation",
            |report| report.paper_rotation_scenario_pack,
        ),
        Commands::PaperRotationMarketContext { config } => print_sprint102_report(
            &config,
            "paper-rotation-market-context",
            "paper_context_warning=paper-only market context preserves source boundaries and no-lookahead proof with no runtime implementation",
            |report| report.paper_rotation_market_context_set,
        ),
        Commands::ArchetypeGroupRotationPlan { config } => print_sprint102_report(
            &config,
            "archetype-group-rotation-plan",
            "rotation_plan_warning=group rotation remains paper-only and routes groups without live execution",
            |report| report.archetype_group_rotation_plan,
        ),
        Commands::ArchetypeMemberSelection { config } => print_sprint102_report(
            &config,
            "archetype-member-selection",
            "member_selection_warning=member selection is paper-only roster use with explicit watchlist handling and no live activation",
            |report| report.archetype_member_selection_report,
        ),
        Commands::LowerConfidenceEvidenceHardening { config } => print_sprint102_report(
            &config,
            "lower-confidence-evidence-hardening",
            "hardening_warning=lower-confidence evidence hardening covers warning-backed candidates only and does not silently upgrade Wonyotti, Larry Williams, or Arthur Hayes",
            |report| report.lower_confidence_evidence_hardening_report,
        ),
        Commands::WeakSourceCandidateReview { config } => print_sprint102_report(
            &config,
            "weak-source-candidate-review",
            "weak_source_warning=weak-source review remains local-only, paper-only, and keeps warning-backed candidates down-weighted",
            |report| report.weak_source_candidate_review_report,
        ),
        Commands::WonyottiEvidenceHardening { config } => print_sprint102_report(
            &config,
            "wonyotti-evidence-hardening",
            "wonyotti_warning=wonyotti evidence hardening stays paper-only with no silent upgrade and no impersonation",
            |report| report.wonyotti_evidence_hardening_report,
        ),
        Commands::LarryWilliamsEvidenceHardening { config } => print_sprint102_report(
            &config,
            "larry-williams-evidence-hardening",
            "larry_warning=larry williams evidence hardening keeps seasonal evidence research-only and not an order rule",
            |report| report.larry_williams_evidence_hardening_report,
        ),
        Commands::ArthurHayesEvidenceHardening { config } => print_sprint102_report(
            &config,
            "arthur-hayes-evidence-hardening",
            "arthur_warning=arthur hayes evidence hardening remains paper-only with leverage-risk guards and no runtime implementation",
            |report| report.arthur_hayes_evidence_hardening_report,
        ),
        Commands::PaperMemberProposalRun { config } => print_sprint102_report(
            &config,
            "paper-member-proposal-run",
            "proposal_warning=paper member proposal run is paper-only and a proposal is not an order, broker action, or live execution path",
            |report| report.paper_only_member_proposal_run,
        ),
        Commands::PaperEntryTimingRun { config } => print_sprint102_report(
            &config,
            "paper-entry-timing-run",
            "entry_timing_warning=paper entry timing remains paper-only and never becomes execution permission",
            |report| report.paper_only_entry_timing_proposal_run,
        ),
        Commands::GroupDebateTrigger { config } => print_sprint102_report(
            &config,
            "group-debate-trigger",
            "debate_trigger_warning=group debate trigger is paper-only and not live AI agent debate",
            |report| report.group_debate_trigger_report,
        ),
        Commands::GroupDebateSession { config } => print_sprint102_report(
            &config,
            "group-debate-session",
            "debate_session_warning=group debate session is paper-only debate with no live agent activation or runtime LLM live decision path",
            |report| report.group_debate_session_report,
        ),
        Commands::CrossGroupDebateConflict { config } => print_sprint102_report(
            &config,
            "cross-group-debate-conflict",
            "conflict_warning=cross-group conflict handling is paper-only and keeps Risk Governor final veto intact",
            |report| report.cross_group_debate_conflict_report,
        ),
        Commands::ChairmanSynthesisDryRun { config } => print_sprint102_report(
            &config,
            "chairman-synthesis-dry-run",
            "chairman_warning=chairman synthesis is paper-only governance only and cannot bypass Risk Governor",
            |report| report.chairman_synthesis_dry_run_report,
        ),
        Commands::ChairmanStyleWeightAudit { config } => print_sprint102_report(
            &config,
            "chairman-style-weight-audit",
            "chairman_audit_warning=chairman style weight audit is paper-only and records no risk-governor override attempt",
            |report| report.chairman_style_weight_adjustment_audit,
        ),
        Commands::RiskGovernorPaperHandoff { config } => print_sprint102_report(
            &config,
            "risk-governor-paper-handoff",
            "risk_warning=risk governor paper handoff preserves final veto, broker_execution_allowed=false, and live_execution_allowed=false",
            |report| report.risk_governor_paper_handoff_report,
        ),
        Commands::PaperDecisionTraceV2 { config } => print_sprint102_report(
            &config,
            "paper-decision-trace-v2",
            "trace_warning=paper decision trace v2 is audit-only with no broker, no live execution, and no order/account path",
            |report| report.paper_decision_trace_v2,
        ),
        Commands::PaperDecisionReplayV2 { config } => print_sprint102_report(
            &config,
            "paper-decision-replay-v2",
            "replay_warning=paper decision replay v2 is audit-only and never a live execution path",
            |report| report.paper_decision_replay_v2_report,
        ),
        Commands::ProposalExpectationTrace { config } => print_sprint102_report(
            &config,
            "proposal-expectation-trace",
            "expectation_warning=proposal expectation trace uses proxies only and does not claim profit or live trading readiness",
            |report| report.proposal_outcome_expectation_trace,
        ),
        Commands::NotradeRiskdeniedCommitteeTrace { config } => print_sprint102_report(
            &config,
            "notrade-riskdenied-committee-trace",
            "notrade_warning=no-trade/riskdenied trace is audit-only and preserves paper-only semantics with no order path",
            |report| report.no_trade_risk_denied_committee_trace,
        ),
        Commands::RegimeRoutedDryRun { config } => print_sprint102_report(
            &config,
            "regime-routed-dry-run",
            "regime_warning=regime-routed dry-run is research-only paper routing and not runtime implementation",
            |report| report.regime_routed_committee_dry_run_report,
        ),
        Commands::MultiExpertRotationCoverage { config } => print_sprint102_report(
            &config,
            "multi-expert-rotation-coverage",
            "coverage_warning=multi-expert rotation coverage is paper-only coverage accounting with no live roster activation",
            |report| report.multi_expert_rotation_coverage_report,
        ),
        Commands::PaperRosterExpansionUsage { config } => print_sprint102_report(
            &config,
            "paper-roster-expansion-usage",
            "roster_usage_warning=paper roster expansion usage stays paper-only and never activates 18 live agents",
            |report| report.paper_roster_expansion_usage_report,
        ),
        Commands::EighteenActivationSafety { config } => print_sprint102_report(
            &config,
            "eighteen-activation-safety",
            "activation_safety_warning=18 activation safety keeps live activation forbidden and paper-only activation explicit",
            |report| report.eighteen_archetype_activation_safety_report,
        ),
        Commands::WorkspaceTruthClosurePlanV3 { config } => print_sprint102_report(
            &config,
            "workspace-truth-closure-plan-v3",
            "workspace_truth_warning=full workspace acceptance remains separate and focused tests cannot claim full acceptance",
            |report| report.workspace_acceptance_truth_closure_plan_v3,
        ),
        Commands::WorkspaceAcceptanceAttemptV18 { config } => print_sprint102_report(
            &config,
            "workspace-acceptance-attempt-v18",
            "workspace_attempt_warning=workspace acceptance attempt v18 is an honest record only; full workspace requires real cargo test --workspace --quiet completion",
            |report| report.workspace_acceptance_attempt_v18,
        ),
        Commands::SafetyCoveragePreservationV18 { config } => print_sprint102_report(
            &config,
            "safety-coverage-preservation-v18",
            "safety_warning=safety coverage v18 preserves no live trading, no broker/order/account, no runtime LLM path, and no browser execution",
            |report| report.safety_coverage_preservation_report_v18,
        ),
        Commands::ControlTowerPaperRotation { config } => print_sprint102_report(
            &config,
            "control-tower-paper-rotation",
            "read_only_warning=control tower paper rotation panel is static read-only output with no train/runtime/live/order/account/browser controls or activate-all-18-live button",
            |report| report.control_tower_paper_rotation_panel,
        ),
        Commands::Sprint103PaperRotationClose { config } => print_sprint103_report(
            &config,
            "sprint103-paper-rotation-close",
            "sprint103_warning=research-only paper-only warning-closure-only closure run; no order, no central AI core, no runtime implementation, no training, no live inference, no live trading, no order/account command, no investor impersonation, no auto-activation of 18 live agents, no silent confidence upgrade, and remote paths rejected",
            |report| report,
        ),
        Commands::PaperRotationWarningClosure { config } => print_sprint103_report(
            &config,
            "paper-rotation-warning-closure",
            "paper_rotation_warning=research-only paper-only warning-closure-only output and not live readiness",
            |report| report.paper_rotation_warning_closure_report,
        ),
        Commands::RotationPlanWarningClosure { config } => print_sprint103_report(
            &config,
            "rotation-plan-warning-closure",
            "rotation_plan_warning=rotation closure stays paper-only and never activates live agents",
            |report| report.rotation_plan_warning_closure_report,
        ),
        Commands::MemberSelectionWarningClosure { config } => print_sprint103_report(
            &config,
            "member-selection-warning-closure",
            "member_selection_warning=member selection closure keeps watchlist usage explicit and live activation forbidden",
            |report| report.member_selection_warning_closure_report,
        ),
        Commands::LowerConfidenceEvidenceClosure { config } => print_sprint103_report(
            &config,
            "lower-confidence-evidence-closure",
            "hardening_warning=lower-confidence evidence closure is warning-closure-only and does not silently upgrade Wonyotti, Larry Williams, or Arthur Hayes",
            |report| report.lower_confidence_evidence_closure_report,
        ),
        Commands::WonyottiWarningClosure { config } => print_sprint103_report(
            &config,
            "wonyotti-warning-closure",
            "wonyotti_warning=wonyotti warning closure keeps exact return claims blocked, no impersonation, and no silent confidence upgrade",
            |report| report.wonyotti_warning_closure_report,
        ),
        Commands::LarryWilliamsWarningClosure { config } => print_sprint103_report(
            &config,
            "larry-williams-warning-closure",
            "larry_warning=larry williams warning closure keeps exact numeric rules downweighted and paper-only",
            |report| report.larry_williams_warning_closure_report,
        ),
        Commands::ArthurHayesWarningClosure { config } => print_sprint103_report(
            &config,
            "arthur-hayes-warning-closure",
            "arthur_warning=arthur hayes warning closure preserves leverage risk guards and no runtime implementation",
            |report| report.arthur_hayes_warning_closure_report,
        ),
        Commands::ProposalRunWarningClosure { config } => print_sprint103_report(
            &config,
            "proposal-run-warning-closure",
            "proposal_warning=proposal run closure is paper-only and a proposal is not an order, broker action, or live execution path",
            |report| report.proposal_run_warning_closure_report,
        ),
        Commands::EntryTimingWarningClosure { config } => print_sprint103_report(
            &config,
            "entry-timing-warning-closure",
            "entry_timing_warning=entry timing closure remains paper-only and never becomes execution permission",
            |report| report.entry_timing_run_warning_closure_report,
        ),
        Commands::DebateSessionWarningClosure { config } => print_sprint103_report(
            &config,
            "debate-session-warning-closure",
            "debate_session_warning=debate closure is paper-only debate with no live activation or runtime LLM live decision path",
            |report| report.debate_session_warning_closure_report,
        ),
        Commands::NeedMoreEvidenceResolutionPlan { config } => print_sprint103_report(
            &config,
            "need-more-evidence-resolution-plan",
            "need_more_evidence_warning=NeedMoreEvidence planning remains paper-only and does not enable live execution",
            |report| report.need_more_evidence_resolution_plan,
        ),
        Commands::CrossGroupConflictClosure { config } => print_sprint103_report(
            &config,
            "cross-group-conflict-closure",
            "conflict_warning=cross-group conflict closure is paper-only and keeps Risk Governor final veto intact",
            |report| report.cross_group_conflict_closure_report,
        ),
        Commands::ChairmanSynthesisWarningClosure { config } => print_sprint103_report(
            &config,
            "chairman-synthesis-warning-closure",
            "chairman_warning=chairman synthesis closure is paper-only governance only and cannot bypass Risk Governor",
            |report| report.chairman_synthesis_warning_closure_report,
        ),
        Commands::StyleWeightAuditWarningClosure { config } => print_sprint103_report(
            &config,
            "style-weight-audit-warning-closure",
            "chairman_audit_warning=style weight closure is paper-only and records no risk-governor override attempt",
            |report| report.style_weight_audit_warning_closure_report,
        ),
        Commands::RiskGovernorHandoffWarningClosureV2 { config } => print_sprint103_report(
            &config,
            "risk-governor-handoff-warning-closure-v2",
            "risk_warning=risk governor handoff closure preserves final veto, broker_execution_allowed=false, and live_execution_allowed=false",
            |report| report.risk_governor_handoff_warning_closure_report_v2,
        ),
        Commands::PaperTraceWarningClosure { config } => print_sprint103_report(
            &config,
            "paper-trace-warning-closure",
            "trace_warning=paper trace closure is audit-only with no broker, no live execution, and no order/account path",
            |report| report.paper_trace_warning_closure_report,
        ),
        Commands::PaperReplayWarningClosureV2 { config } => print_sprint103_report(
            &config,
            "paper-replay-warning-closure-v2",
            "replay_warning=paper replay closure is audit-only and never a live execution path",
            |report| report.paper_replay_warning_closure_report_v2,
        ),
        Commands::ExpectationTraceWarningClosure { config } => print_sprint103_report(
            &config,
            "expectation-trace-warning-closure",
            "expectation_warning=expectation closure uses proxies only and does not claim profit or live trading readiness",
            |report| report.expectation_trace_warning_closure_report,
        ),
        Commands::NotradeRiskdeniedTraceWarningClosure { config } => print_sprint103_report(
            &config,
            "notrade-riskdenied-trace-warning-closure",
            "notrade_warning=no-trade/riskdenied closure is audit-only and preserves paper-only semantics with no order path",
            |report| report.notrade_riskdenied_trace_warning_closure_report,
        ),
        Commands::RegimeRoutingWarningClosure { config } => print_sprint103_report(
            &config,
            "regime-routing-warning-closure",
            "regime_warning=regime routing closure is research-only paper routing and not runtime implementation",
            |report| report.regime_routing_warning_closure_report,
        ),
        Commands::MultiExpertCoverageWarningClosure { config } => print_sprint103_report(
            &config,
            "multi-expert-coverage-warning-closure",
            "coverage_warning=multi-expert coverage closure is paper-only coverage accounting with no live roster activation",
            |report| report.multi_expert_coverage_warning_closure_report,
        ),
        Commands::PaperRosterUsageWarningClosure { config } => print_sprint103_report(
            &config,
            "paper-roster-usage-warning-closure",
            "roster_usage_warning=paper roster usage closure stays paper-only and never activates 18 live agents",
            |report| report.paper_roster_usage_warning_closure_report,
        ),
        Commands::WatchlistMemberUsagePolicy { config } => print_sprint103_report(
            &config,
            "watchlist-member-usage-policy",
            "watchlist_warning=watchlist usage policy is paper-only, live activation forbidden, and explicit review required",
            |report| report.watchlist_member_usage_policy,
        ),
        Commands::SaylorTreasuryWatchlistAudit { config } => print_sprint103_report(
            &config,
            "saylor-treasury-watchlist-audit",
            "saylor_warning=saylor treasury watchlist audit is paper-only and live activation remains forbidden",
            |report| report.saylor_treasury_watchlist_usage_audit,
        ),
        Commands::MultiScenarioPaperReplay { config } => print_sprint103_report(
            &config,
            "multi-scenario-paper-replay",
            "multi_scenario_warning=multi-scenario replay is paper-only calibration only with no broker, no order, and no live trading",
            |report| report.multi_scenario_paper_replay_report,
        ),
        Commands::ScenarioOutcomeExpectationMatrix { config } => print_sprint103_report(
            &config,
            "scenario-outcome-expectation-matrix",
            "matrix_warning=scenario expectation matrix uses bounded paper proxies only and not profit claims",
            |report| report.scenario_outcome_expectation_matrix,
        ),
        Commands::CommitteeDecisionStability { config } => print_sprint103_report(
            &config,
            "committee-decision-stability",
            "stability_warning=committee decision stability is replay-only analysis and not live readiness",
            |report| report.committee_decision_stability_report,
        ),
        Commands::PaperNotradeJustification { config } => print_sprint103_report(
            &config,
            "paper-notrade-justification",
            "notrade_justification_warning=paper NoTrade justification treats NoTrade as a valid defensive outcome and not failure",
            |report| report.paper_notrade_justification_report,
        ),
        Commands::PaperNeedMoreEvidenceJustification { config } => print_sprint103_report(
            &config,
            "paper-need-more-evidence-justification",
            "need_more_evidence_justification_warning=NeedMoreEvidence remains a valid paper outcome and does not imply live escalation",
            |report| report.paper_need_more_evidence_justification_report,
        ),
        Commands::RiskGovernorNotradeReasonAudit { config } => print_sprint103_report(
            &config,
            "risk-governor-notrade-reason-audit",
            "risk_audit_warning=Risk Governor NoTrade audit is paper-only reasoning review with final veto preserved",
            |report| report.risk_governor_notrade_reason_audit,
        ),
        Commands::PaperRotationReadinessGateV2 { config } => print_sprint103_report(
            &config,
            "paper-rotation-readiness-gate-v2",
            "gate_warning=paper rotation readiness gate v2 is paper-only and live rotation remains forbidden",
            |report| report.paper_rotation_readiness_gate_v2,
        ),
        Commands::WorkspaceTruthClosurePlanV4 { config } => print_sprint103_report(
            &config,
            "workspace-truth-closure-plan-v4",
            "workspace_truth_warning=workspace truth v4 keeps full workspace acceptance separate and focused tests cannot claim full acceptance",
            |report| report.workspace_acceptance_truth_closure_plan_v4,
        ),
        Commands::WorkspaceAcceptanceAttemptV19 { config } => print_sprint103_report(
            &config,
            "workspace-acceptance-attempt-v19",
            "workspace_attempt_warning=workspace acceptance attempt v19 is an honest record only; real cargo test --workspace --quiet completion is still required",
            |report| report.workspace_acceptance_attempt_v19,
        ),
        Commands::SafetyCoveragePreservationV19 { config } => print_sprint103_report(
            &config,
            "safety-coverage-preservation-v19",
            "safety_warning=safety coverage v19 preserves no live trading, no broker/order/account, no runtime LLM path, and no browser execution",
            |report| report.safety_coverage_preservation_report_v19,
        ),
        Commands::ControlTowerPaperRotationClosure { config } => print_sprint103_report(
            &config,
            "control-tower-paper-rotation-closure",
            "read_only_warning=control tower paper rotation closure panel is static read-only output with no train/runtime/live/order/account/browser controls or activate-all-18-live button",
            |report| report.control_tower_paper_rotation_closure_panel,
        ),
        Commands::Sprint104DualAgentPaperLifecycle { config } => print_sprint104_report(
            &config,
            "sprint104-dual-agent-paper-lifecycle",
            "sprint104_warning=research-only paper-only dual-agent workflow; verification-is-not-full-acceptance, paper-candidate-not-order, no runtime implementation, no training, no live inference, no live trading, no order/account command, no runtime LLM live decision path, no investor impersonation, no auto-activation of 18 live agents, no silent confidence upgrade, no safety test deletion, local-only paths, and remote paths rejected",
            |report| report,
        ),
        Commands::DualAgentWorkflowPolicy { config } => print_sprint104_report(
            &config,
            "dual-agent-workflow-policy",
            "workflow_warning=research-only paper-only dual-agent workflow; verification-is-not-full-acceptance and no live execution path exists",
            |report| report.dual_agent_workflow_policy,
        ),
        Commands::ImplementationAgentRole { config } => print_sprint104_report(
            &config,
            "implementation-agent-role",
            "implementation_warning=research-only 5.4 implementation role only; no runtime implementation, no training, and no live trading",
            |report| report.implementation_agent_role_report,
        ),
        Commands::VerificationAgentRole { config } => print_sprint104_report(
            &config,
            "verification-agent-role",
            "verification_warning=research-only 5.5 verification role only; verification-is-not-full-acceptance and no live trading readiness claim",
            |report| report.verification_agent_role_report,
        ),
        Commands::PromptComplianceVerification { config } => print_sprint104_report(
            &config,
            "prompt-compliance-verification",
            "prompt_warning=research-only dual-agent prompt compliance verification; verification-is-not-full-acceptance and paper-candidate-not-order",
            |report| report.prompt_compliance_verification_report,
        ),
        Commands::SafetyInvariantVerification { config } => print_sprint104_report(
            &config,
            "safety-invariant-verification",
            "safety_warning=research-only safety invariant verification; no runtime implementation, no training, no live inference, no live trading, no order/account command, and no runtime LLM live decision path",
            |report| report.safety_invariant_verification_report,
        ),
        Commands::ArchitectureRegressionVerification { config } => print_sprint104_report(
            &config,
            "architecture-regression-verification",
            "architecture_warning=research-only architecture verification; committee-owned architecture only, no central AI core regression, and Risk Governor final veto preserved",
            |report| report.architecture_regression_verification_report,
        ),
        Commands::TestCoverageVerification { config } => print_sprint104_report(
            &config,
            "test-coverage-verification",
            "test_warning=research-only focused test verification only; verification-is-not-full-acceptance and no safety test deletion",
            |report| report.test_coverage_verification_report,
        ),
        Commands::FinalVerificationGate { config } => print_sprint104_report(
            &config,
            "final-verification-gate",
            "final_verification_warning=research-only final verification gate; verification-is-not-full-acceptance and live trading remains forbidden",
            |report| report.final_verification_gate,
        ),
        Commands::PaperBatchReplay { config } => print_sprint104_report(
            &config,
            "paper-batch-replay",
            "batch_replay_warning=research-only paper-only batch replay; paper proposals are not orders and no broker/order/account path exists",
            |report| report.paper_rotation_batch_replay_report,
        ),
        Commands::PaperCandidateLifecycle { config } => print_sprint104_report(
            &config,
            "paper-candidate-lifecycle",
            "paper_lifecycle_warning=research-only paper candidate lifecycle; paper candidate is not an order, no live execution, and no promote-to-live path",
            |report| report.paper_candidate_lifecycle_state_machine,
        ),
        Commands::PaperCandidatePromotionGate { config } => print_sprint104_report(
            &config,
            "paper-candidate-promotion-gate",
            "promotion_warning=research-only paper candidate promotion gate; paper-only promotion only and no live execution path",
            |report| report.paper_candidate_promotion_gate,
        ),
        Commands::PaperCandidateNotradeGate { config } => print_sprint104_report(
            &config,
            "paper-candidate-notrade-gate",
            "paper_notrade_warning=research-only paper candidate NoTrade gate; paper candidate is not an order and no execution permission is created",
            |report| report.paper_candidate_no_trade_gate,
        ),
        Commands::PaperCandidateRiskdeniedGate { config } => print_sprint104_report(
            &config,
            "paper-candidate-riskdenied-gate",
            "paper_riskdenied_warning=research-only paper candidate RiskDenied gate; Risk Governor remains final veto and no live execution is allowed",
            |report| report.paper_candidate_risk_denied_gate,
        ),
        Commands::RiskGovernorBatchVeto { config } => print_sprint104_report(
            &config,
            "risk-governor-batch-veto",
            "risk_batch_warning=research-only Risk Governor batch veto; paper-only final veto only with no broker/order/account path",
            |report| report.risk_governor_batch_veto_report,
        ),
        Commands::LowerConfidenceCarryForward { config } => print_sprint104_report(
            &config,
            "lower-confidence-carry-forward",
            "carry_forward_warning=research-only lower-confidence carry-forward; paper-only only, no silent confidence upgrade, and no live activation",
            |report| report.lower_confidence_carry_forward_policy,
        ),
        Commands::ControlTowerDualAgent { config } => print_sprint104_report(
            &config,
            "control-tower-dual-agent",
            "read_only_warning=control tower dual-agent panel is static/read-only output with no train/runtime/live/order/account/browser controls",
            |report| report.control_tower_dual_agent_panel,
        ),
        Commands::ControlTowerPaperCandidateLifecycle { config } => print_sprint104_report(
            &config,
            "control-tower-paper-candidate-lifecycle",
            "read_only_warning=control tower paper candidate lifecycle panel is static/read-only output with no promote-to-live button, no order button, and no account panel",
            |report| report.control_tower_paper_candidate_lifecycle_panel,
        ),
        Commands::Sprint105VerificationPatchClose { config } => print_sprint105_report(
            &config,
            "sprint105-verification-patch-close",
            "sprint105_warning=research-only paper-only verification-patch-closure workflow; verification-is-not-full-acceptance, paper-candidate-not-order, no runtime implementation, no training, no live inference, no live trading, no order/account command, no runtime LLM live decision path, no investor impersonation, no auto-activation of 18 live agents, no silent confidence upgrade, no safety test deletion, local-only paths, and remote paths rejected",
            |report| report,
        ),
        Commands::VerificationFindingClosure { config } => print_sprint105_report(
            &config,
            "verification-finding-closure",
            "finding_closure_warning=research-only verification finding closure only; verification-is-not-full-acceptance and findings stay explicit",
            |report| report.verification_finding_closure_report,
        ),
        Commands::ReviewPatchEffect { config } => print_sprint105_report(
            &config,
            "review-patch-effect",
            "review_patch_warning=research-only review patch effect report only; patch closure does not imply cargo workspace acceptance",
            |report| report.review_patch_effect_report,
        ),
        Commands::OverclaimRegressionGuard { config } => print_sprint105_report(
            &config,
            "overclaim-regression-guard",
            "overclaim_guard_warning=full workspace acceptance requires finished && passed only; focused tests, verification, and no-run are not full acceptance",
            |report| report.overclaim_regression_guard_report,
        ),
        Commands::WorkspaceAttemptTruthHardening { config } => print_sprint105_report(
            &config,
            "workspace-attempt-truth-hardening",
            "workspace_truth_warning=workspace truth hardening keeps unfinished attempts visible and full workspace separate from focused verification",
            |report| report.workspace_attempt_truth_hardening_report,
        ),
        Commands::SafetyBooleanCoverageAudit { config } => print_sprint105_report(
            &config,
            "safety-boolean-coverage-audit",
            "safety_boolean_warning=research-only safety boolean coverage audit uses actual guard booleans and adds no runtime, training, live inference, or live trading path",
            |report| report.safety_boolean_coverage_audit_report,
        ),
        Commands::PaperRejectedTransitionAudit { config } => print_sprint105_report(
            &config,
            "paper-rejected-transition-audit",
            "paper_rejected_warning=research-only paper lifecycle audit only; paper candidate is not an order and PaperRejected cannot become live execution",
            |report| report.paper_rejected_transition_audit_report,
        ),
        Commands::RiskRequiredTransitionAudit { config } => print_sprint105_report(
            &config,
            "risk-required-transition-audit",
            "risk_transition_warning=Risk Governor required transition audit only; Risk Governor remains required and final veto stays preserved",
            |report| report.risk_governor_required_transition_audit_report,
        ),
        Commands::MissingArtifactFindingPolicy { config } => print_sprint105_report(
            &config,
            "missing-artifact-finding-policy",
            "artifact_policy_warning=missing docs/tests/examples become findings instead of silent success and verification is not full acceptance",
            |report| report.missing_artifact_finding_policy_report,
        ),
        Commands::FinalVerificationGateV2 { config } => print_sprint105_report(
            &config,
            "final-verification-gate-v2",
            "final_verification_v2_warning=FinalVerificationGateV2 is verification patch closure only and not cargo workspace acceptance",
            |report| report.final_verification_gate_v2,
        ),
        Commands::DualAgentReviewLoopV2 { config } => print_sprint105_report(
            &config,
            "dual-agent-review-loop-v2",
            "review_loop_warning=research-only dual-agent review loop v2 only; verification-is-not-full-acceptance and no live execution path exists",
            |report| report.dual_agent_review_loop_v2_report,
        ),
        Commands::PaperLifecycleWarningClosure { config } => print_sprint105_report(
            &config,
            "paper-lifecycle-warning-closure",
            "paper_lifecycle_closure_warning=paper lifecycle warning closure is research-only paper-only output; paper candidate is not order execution",
            |report| report.paper_lifecycle_warning_closure_report,
        ),
        Commands::PaperCandidateTransitionCoverage { config } => print_sprint105_report(
            &config,
            "paper-candidate-transition-coverage",
            "transition_coverage_warning=paper candidate transition coverage is research-only and no live transition or order path is created",
            |report| report.paper_candidate_transition_coverage_report,
        ),
        Commands::PaperCandidateGateCompleteness { config } => print_sprint105_report(
            &config,
            "paper-candidate-gate-completeness",
            "gate_completeness_warning=paper candidate gate completeness is research-only and paper candidate is not an order",
            |report| report.paper_candidate_gate_completeness_report,
        ),
        Commands::PaperCandidateEvidenceDepthClosure { config } => print_sprint105_report(
            &config,
            "paper-candidate-evidence-depth-closure",
            "evidence_depth_warning=paper candidate evidence depth closure is research-only and remains non-executable",
            |report| report.paper_candidate_evidence_depth_closure_report,
        ),
        Commands::PaperCandidateTraceClosure { config } => print_sprint105_report(
            &config,
            "paper-candidate-trace-closure",
            "trace_closure_warning=paper candidate trace closure is research-only and no runtime LLM live decision path exists",
            |report| report.paper_candidate_trace_closure_report,
        ),
        Commands::PaperCandidateStabilityClosure { config } => print_sprint105_report(
            &config,
            "paper-candidate-stability-closure",
            "stability_closure_warning=paper candidate stability closure is replay-only and not live readiness",
            |report| report.paper_candidate_stability_closure_report,
        ),
        Commands::RiskGovernorBatchVetoWarningClosure { config } => print_sprint105_report(
            &config,
            "risk-governor-batch-veto-warning-closure",
            "risk_veto_closure_warning=Risk Governor batch veto warning closure is research-only paper-only output and final veto remains preserved",
            |report| report.risk_governor_batch_veto_warning_closure_report,
        ),
        Commands::RiskGovernorNoBypassAuditV2 { config } => print_sprint105_report(
            &config,
            "risk-governor-no-bypass-audit-v2",
            "no_bypass_warning=research-only Risk Governor no-bypass audit only; chairman/member/owner bypass remains forbidden",
            |report| report.risk_governor_no_bypass_audit_v2,
        ),
        Commands::LowerConfidenceCarryForwardClosure { config } => print_sprint105_report(
            &config,
            "lower-confidence-carry-forward-closure",
            "lower_confidence_warning=research-only lower-confidence carry-forward closure; no silent confidence upgrade and no live activation",
            |report| report.lower_confidence_carry_forward_closure_report,
        ),
        Commands::PaperLifecycleReadinessGateV2 { config } => print_sprint105_report(
            &config,
            "paper-lifecycle-readiness-gate-v2",
            "lifecycle_gate_warning=paper lifecycle readiness gate v2 is paper-only and live lifecycle remains forbidden",
            |report| report.paper_lifecycle_readiness_gate_v2,
        ),
        Commands::PaperCandidateBatchReplayV2 { config } => print_sprint105_report(
            &config,
            "paper-candidate-batch-replay-v2",
            "batch_replay_v2_warning=paper candidate batch replay v2 is research-only paper-only replay with no broker/live execution path",
            |report| report.paper_candidate_batch_replay_v2_report,
        ),
        Commands::WorkspaceAcceptanceTruthRecoveryPlanV6 { config } => print_sprint105_report(
            &config,
            "workspace-acceptance-truth-recovery-plan-v6",
            "workspace_recovery_warning=workspace acceptance truth recovery keeps full workspace separate from focused verification and requires honest finished && passed evidence",
            |report| report.workspace_acceptance_truth_recovery_plan_v6,
        ),
        Commands::WorkspaceCompileCostDiagnosisV2 { config } => print_sprint105_report(
            &config,
            "workspace-compile-cost-diagnosis-v2",
            "compile_cost_warning=workspace compile-cost diagnosis is diagnostic-only and does not fake full workspace pass/fail",
            |report| report.workspace_compile_cost_diagnosis_v2,
        ),
        Commands::FocusedVsFullGateBridgeV2 { config } => print_sprint105_report(
            &config,
            "focused-vs-full-gate-bridge-v2",
            "bridge_warning=focused-vs-full gate bridge v2 keeps verification and focused passes separate from full workspace acceptance",
            |report| report.focused_vs_full_gate_bridge_v2,
        ),
        Commands::SafetyCoveragePreservationV21 { config } => print_sprint105_report(
            &config,
            "safety-coverage-preservation-v21",
            "safety_v21_warning=safety coverage v21 preserves no runtime, no training, no live inference, no live trading, and no broker/order/account path",
            |report| report.safety_coverage_preservation_report_v21,
        ),
        Commands::ControlTowerVerificationPatchClosure { config } => print_sprint105_report(
            &config,
            "control-tower-verification-patch-closure",
            "read_only_warning=control tower verification patch closure panel is static/read-only output with no verification execution button and no train/runtime/live/order/account/browser controls",
            |report| report.control_tower_verification_patch_closure_panel,
        ),
        Commands::ControlTowerPaperLifecycleClosure { config } => print_sprint105_report(
            &config,
            "control-tower-paper-lifecycle-closure",
            "read_only_warning=control tower paper lifecycle closure panel is static/read-only output with no promote-to-live button and no order/account controls",
            |report| report.control_tower_paper_lifecycle_closure_panel,
        ),
        Commands::Sprint106WorkspaceAcceptanceRecover { config } => print_sprint106_report(
            &config,
            "sprint106-workspace-acceptance-recover",
            "sprint106_warning=research-only workspace acceptance recovery only; focused-is-not-full, no-run-is-not-full-acceptance, verification-is-not-acceptance, no runtime implementation, no training, no live inference, no live trading, no order/account command, no safety test deletion, no hidden skips, local-only paths, and remote paths rejected",
            |report| report,
        ),
        Commands::RealNoRunCompletionV22 { config } => print_sprint106_report(
            &config,
            "real-no-run-completion-v22",
            "no_run_warning=no-run is not full acceptance; completion only records honest cargo test --workspace --no-run --quiet status",
            |report| report.real_no_run_completion_attempt_v22,
        ),
        Commands::RealFullWorkspaceAttemptV22 { config } => print_sprint106_report(
            &config,
            "real-full-workspace-attempt-v22",
            "full_workspace_warning=finished and passed required for full workspace acceptance; no other status can claim acceptance",
            |report| report.real_full_workspace_attempt_v22,
        ),
        Commands::WorkspaceCompileCostProfileV3 { config } => print_sprint106_report(
            &config,
            "workspace-compile-cost-profile-v3",
            "compile_cost_warning=workspace compile-cost profile is diagnostic-only and never a full acceptance claim",
            |report| report.workspace_compile_cost_profile_v3,
        ),
        Commands::CargoJsonNoRunCaptureV2 { config } => print_sprint106_report(
            &config,
            "cargo-json-no-run-capture-v2",
            "cargo_json_warning=no-run capture is diagnostic-only and no-run is not full acceptance",
            |report| report.cargo_json_no_run_capture_v2,
        ),
        Commands::TestBinaryInventoryV3 { config } => print_sprint106_report(
            &config,
            "test-binary-inventory-v3",
            "inventory_warning=test binary inventory preserves safety sentinels and does not imply any hidden skip",
            |report| report.test_binary_inventory_report_v3,
        ),
        Commands::TestBinaryExplosionAttribution { config } => print_sprint106_report(
            &config,
            "test-binary-explosion-attribution",
            "explosion_warning=test binary explosion attribution is diagnostic-only and no assertion deletion is allowed",
            |report| report.test_binary_explosion_attribution_report,
        ),
        Commands::IntegrationTargetCostRanking { config } => print_sprint106_report(
            &config,
            "integration-target-cost-ranking",
            "ranking_warning=integration target cost ranking is diagnostic-only and not full workspace acceptance",
            |report| report.integration_target_cost_ranking_report,
        ),
        Commands::LongRunningRustcSnapshotV2 { config } => print_sprint106_report(
            &config,
            "long-running-rustc-snapshot-v2",
            "rustc_warning=rustc snapshot is compile observation only and not runtime readiness",
            |report| report.long_running_rustc_target_snapshot_v2,
        ),
        Commands::FixtureSetupCostAttributionV2 { config } => print_sprint106_report(
            &config,
            "fixture-setup-cost-attribution-v2",
            "fixture_warning=fixture/setup attribution is deterministic-only and remains local-only",
            |report| report.fixture_setup_cost_attribution_v2,
        ),
        Commands::ArtifactRenderCostAttributionV2 { config } => print_sprint106_report(
            &config,
            "artifact-render-cost-attribution-v2",
            "artifact_render_warning=artifact render attribution is local-only and does not imply runtime/UI execution",
            |report| report.artifact_render_cost_attribution_v2,
        ),
        Commands::CliSmokeCostAttributionV2 { config } => print_sprint106_report(
            &config,
            "cli-smoke-cost-attribution-v2",
            "cli_smoke_warning=representative, exhaustive, and safety smoke remain separate and CLI smoke is not full acceptance",
            |report| report.cli_smoke_cost_attribution_v2,
        ),
        Commands::HighCostTestFamilyClusters { config } => print_sprint106_report(
            &config,
            "high-cost-test-family-clusters",
            "cluster_warning=high-cost family clustering preserves unsafe consolidation boundaries and isolated sentinels",
            |report| report.high_cost_test_family_cluster_report,
        ),
        Commands::SafeTestBinaryConsolidationPlanV2 { config } => print_sprint106_report(
            &config,
            "safe-test-binary-consolidation-plan-v2",
            "consolidation_warning=no assertion deletion; safety sentinels remain preserved and unsafe consolidation stays blocked",
            |report| report.safe_test_binary_consolidation_plan_v2,
        ),
        Commands::SharedFixtureHarnessExpansionPlanV2 { config } => print_sprint106_report(
            &config,
            "shared-fixture-harness-expansion-plan-v2",
            "shared_fixture_warning=shared fixture harness expansion is deterministic-only and local-only",
            |report| report.shared_fixture_harness_expansion_plan_v2,
        ),
        Commands::CliSmokeTieringPlanV2 { config } => print_sprint106_report(
            &config,
            "cli-smoke-tiering-plan-v2",
            "tiering_warning=CLI smoke tiering keeps representative, exhaustive, and safety smoke separate",
            |report| report.cli_smoke_tiering_plan_v2,
        ),
        Commands::WorkspaceNoRunRecoveryGateV7 { config } => print_sprint106_report(
            &config,
            "workspace-no-run-recovery-gate-v7",
            "no_run_gate_warning=no-run is not full acceptance; this gate only reports recovery status for cargo test --workspace --no-run --quiet",
            |report| report.workspace_no_run_recovery_gate_v7,
        ),
        Commands::WorkspaceFullAcceptanceGateV7 { config } => print_sprint106_report(
            &config,
            "workspace-full-acceptance-gate-v7",
            "full_gate_warning=finished and passed required for full workspace acceptance; focused, verification, and no-run cannot claim it",
            |report| report.workspace_full_acceptance_gate_v7,
        ),
        Commands::FocusedVsFullBridgeV3 { config } => print_sprint106_report(
            &config,
            "focused-vs-full-bridge-v3",
            "bridge_warning=focused is not full workspace acceptance and CLI smoke cannot set full acceptance",
            |report| report.focused_vs_full_bridge_v3,
        ),
        Commands::AcceptanceTruthGateV7 { config } => print_sprint106_report(
            &config,
            "acceptance-truth-gate-v7",
            "acceptance_truth_warning=focused is not full, verification is not full, and no-run is not full pass",
            |report| report.acceptance_truth_gate_v7,
        ),
        Commands::AcceptanceRecoveryPatchPlan { config } => print_sprint106_report(
            &config,
            "acceptance-recovery-patch-plan",
            "patch_plan_warning=no assertion deletion, no hidden skips, and safe consolidation must stay explicit",
            |report| report.acceptance_recovery_patch_plan,
        ),
        Commands::AcceptanceRecoveryVerification { config } => print_sprint106_report(
            &config,
            "acceptance-recovery-verification",
            "verification_warning=acceptance recovery verification preserves assertions, CLI safety, determinism, and no hidden skips",
            |report| report.acceptance_recovery_verification_report,
        ),
        Commands::SafetyCoveragePreservationV22 { config } => print_sprint106_report(
            &config,
            "safety-coverage-preservation-v22",
            "safety_v22_warning=safety preserved; no runtime, no training, no live inference, no live trading, and no order/account path",
            |report| report.safety_coverage_preservation_report_v22,
        ),
        Commands::ControlTowerWorkspaceAcceptanceRecoveryV7 { config } => print_sprint106_report(
            &config,
            "control-tower-workspace-acceptance-recovery-v7",
            "read_only_warning=control tower workspace acceptance recovery panel is static/read-only output with no run-tests button, no train button, no runtime button, no live button, and no order/account controls",
            |report| report.control_tower_workspace_acceptance_recovery_panel_v7,
        ),
        Commands::Sprint107SafeConsolidationPatch { config } => print_sprint107_report(
            &config,
            "sprint107-safe-consolidation-patch",
            "sprint107_warning=research-only first safe consolidation patch only; paper-only, safe-consolidation-only, no assertion deletion, no safety sentinel deletion, focused-is-not-full, no-run-is-not-full-acceptance, verification-is-not-acceptance, no runtime implementation, no training, no live inference, no live trading, no order/account command, no hidden skips, local-only paths, and remote paths rejected",
            |report| report,
        ),
        Commands::SafeConsolidationPatchSelection { config } => print_sprint107_report(
            &config,
            "safe-consolidation-patch-selection",
            "selection_warning=first safe consolidation patch only; high-risk sentinels and CommitteeCliSafety remain isolated",
            |report| report.safe_consolidation_patch_selection_report,
        ),
        Commands::ConsolidationCandidateRiskReview { config } => print_sprint107_report(
            &config,
            "consolidation-candidate-risk-review",
            "risk_review_warning=high-risk candidates stay rejected and no assertion deletion is allowed",
            |report| report.consolidation_candidate_risk_review_report,
        ),
        Commands::AssertionMigrationLedgerV1 { config } => print_sprint107_report(
            &config,
            "assertion-migration-ledger-v1",
            "assertion_ledger_warning=no assertion deletion; migrated assertions must remain explicit",
            |report| report.assertion_migration_ledger_v1,
        ),
        Commands::AssertionPreservationVerificationV1 { config } => print_sprint107_report(
            &config,
            "assertion-preservation-verification-v1",
            "assertion_preservation_warning=no assertion deletion and no silent deletion; equivalent coverage must stay explicit",
            |report| report.assertion_preservation_verification_report_v1,
        ),
        Commands::SafetySentinelPreservationV1 { config } => print_sprint107_report(
            &config,
            "safety-sentinel-preservation-v1",
            "safety_sentinel_warning=CommitteeCliSafety, workspace CLI safety, determinism, and paper lifecycle sentinels remain preserved",
            |report| report.safety_sentinel_preservation_report_v1,
        ),
        Commands::SharedFixtureHarnessApplicationV1 { config } => print_sprint107_report(
            &config,
            "shared-fixture-harness-application-v1",
            "shared_fixture_warning=shared fixture harness stays local-only, deterministic, and secret-free",
            |report| report.shared_fixture_harness_application_report_v1,
        ),
        Commands::SharedTomlBuilderApplicationV1 { config } => print_sprint107_report(
            &config,
            "shared-toml-builder-application-v1",
            "shared_toml_warning=shared TOML builder preserves local-only validation and rejects remote paths",
            |report| report.shared_toml_builder_application_report_v1,
        ),
        Commands::SharedOutputDirHelperApplicationV1 { config } => print_sprint107_report(
            &config,
            "shared-output-dir-helper-application-v1",
            "shared_output_dir_warning=shared output-dir helper preserves deterministic cleanup and no silent deletion",
            |report| report.shared_output_dir_helper_application_report_v1,
        ),
        Commands::SharedRenderHelperApplicationV1 { config } => print_sprint107_report(
            &config,
            "shared-render-helper-application-v1",
            "shared_render_warning=shared render helper is deterministic-only and never runtime/UI execution",
            |report| report.shared_render_helper_application_report_v1,
        ),
        Commands::ArtifactRenderCacheApplicationV1 { config } => print_sprint107_report(
            &config,
            "artifact-render-cache-application-v1",
            "artifact_cache_warning=artifact cache is opt-in only, local-only, and secret-free",
            |report| report.artifact_render_cache_application_report_v1,
        ),
        Commands::CliSmokeTieringApplicationV1 { config } => print_sprint107_report(
            &config,
            "cli-smoke-tiering-application-v1",
            "cli_smoke_warning=CLI smoke tiering preserves safety smoke and representative smoke never implies full acceptance",
            |report| report.cli_smoke_tiering_application_report_v1,
        ),
        Commands::ConsolidatedTestTargetManifestV1 { config } => print_sprint107_report(
            &config,
            "consolidated-test-target-manifest-v1",
            "consolidated_manifest_warning=first safe consolidation patch only; high-risk sentinels remain isolated",
            |report| report.consolidated_test_target_manifest_v1,
        ),
        Commands::RetiredNarrowTargetManifestV1 { config } => print_sprint107_report(
            &config,
            "retired-narrow-target-manifest-v1",
            "retired_manifest_warning=retirement is allowed only after assertion migration and equivalent coverage; no assertion deletion",
            |report| report.retired_narrow_target_manifest_v1,
        ),
        Commands::TestBinaryDeltaV4 { config } => print_sprint107_report(
            &config,
            "test-binary-delta-v4",
            "binary_delta_warning=sample-backed delta is not a measured reduction claim",
            |report| report.test_binary_delta_report_v4,
        ),
        Commands::PostPatchWorkspaceNoRunV23 { config } => print_sprint107_report(
            &config,
            "post-patch-workspace-no-run-v23",
            "post_patch_no_run_warning=no-run is not full acceptance; this only records honest cargo test --workspace --no-run --quiet status",
            |report| report.post_patch_workspace_no_run_attempt_v23,
        ),
        Commands::PostPatchWorkspaceFullV23 { config } => print_sprint107_report(
            &config,
            "post-patch-workspace-full-v23",
            "post_patch_full_warning=finished and passed required for full workspace acceptance; no other status can claim acceptance",
            |report| report.post_patch_workspace_full_attempt_v23,
        ),
        Commands::WorkspaceNoRunRecoveryGateV8 { config } => print_sprint107_report(
            &config,
            "workspace-no-run-recovery-gate-v8",
            "no_run_gate_warning=no-run is not full acceptance; this gate only reports recovery status for cargo test --workspace --no-run --quiet",
            |report| report.workspace_no_run_recovery_gate_v8,
        ),
        Commands::WorkspaceFullAcceptanceGateV8 { config } => print_sprint107_report(
            &config,
            "workspace-full-acceptance-gate-v8",
            "full_gate_warning=finished and passed required for full workspace acceptance; focused, verification, CLI smoke, and no-run cannot claim it",
            |report| report.workspace_full_acceptance_gate_v8,
        ),
        Commands::AcceptanceTruthGateV8 { config } => print_sprint107_report(
            &config,
            "acceptance-truth-gate-v8",
            "acceptance_truth_warning=focused is not full, CLI smoke is not full, verification is not full, and no-run is not full pass",
            |report| report.acceptance_truth_gate_v8,
        ),
        Commands::ControlTowerSafeConsolidationPatchV1 { config } => print_sprint107_report(
            &config,
            "control-tower-safe-consolidation-patch-v1",
            "read_only_warning=control tower safe consolidation patch panel is static/read-only output with no run-tests button and no train/runtime/live/order/account controls",
            |report| report.control_tower_safe_consolidation_patch_panel_v1,
        ),
        Commands::ControlTowerWorkspaceAcceptanceRecoveryV8 { config } => print_sprint107_report(
            &config,
            "control-tower-workspace-acceptance-recovery-v8",
            "read_only_warning=control tower workspace acceptance recovery panel is static/read-only output with no run-tests button and no train/runtime/live/order/account controls",
            |report| report.control_tower_workspace_acceptance_recovery_panel_v8,
        ),
        Commands::Sprint108SafeConsolidationPatchV2 { config } => print_sprint108_report(
            &config,
            "sprint108-safe-consolidation-patch-v2",
            "sprint108_warning=research-only second safe consolidation patch only; paper-only, safe-consolidation-only, second-smallest-patch-only, no assertion deletion, no safety sentinel deletion, focused-is-not-full, no-run-is-not-full-acceptance, verification-is-not-acceptance, no runtime implementation, no training, no live inference, no live trading, no order/account command, no hidden skips, local-only paths, and remote paths rejected",
            |report| report,
        ),
        Commands::Sprint107VerificationReconcile { config } => print_sprint108_report(
            &config,
            "sprint107-verification-reconcile",
            "verification_reconcile_warning=5.5 verification is not acceptance and reconciliation does not imply full workspace pass",
            |report| report.sprint107_verification_reconciliation_report,
        ),
        Commands::IndependentVerificationClosureV1 { config } => print_sprint108_report(
            &config,
            "independent-verification-closure-v1",
            "independent_verification_warning=independent verification closure is not full workspace acceptance",
            |report| report.independent_verification_closure_report_v1,
        ),
        Commands::VerificationPatchCarryForward { config } => print_sprint108_report(
            &config,
            "verification-patch-carry-forward",
            "carry_forward_warning=verification patch carry-forward is diagnostic-only and does not imply full workspace acceptance",
            |report| report.verification_patch_carry_forward_report,
        ),
        Commands::SecondSafeConsolidationPatchSelection { config } => print_sprint108_report(
            &config,
            "second-safe-consolidation-patch-selection",
            "second_selection_warning=second safe consolidation patch only; previous retired target is not reselected and sentinels remain isolated",
            |report| report.second_safe_consolidation_patch_selection_report,
        ),
        Commands::AssertionMigrationLedgerV2 { config } => print_sprint108_report(
            &config,
            "assertion-migration-ledger-v2",
            "assertion_ledger_v2_warning=no assertion deletion; migrated assertions must remain explicit",
            |report| report.assertion_migration_ledger_v2,
        ),
        Commands::EquivalentCoverageProofV1 { config } => print_sprint108_report(
            &config,
            "equivalent-coverage-proof-v1",
            "equivalent_coverage_warning=coverage required before retirement and no assertion deletion is allowed",
            |report| report.equivalent_coverage_proof_report_v1,
        ),
        Commands::RetiredTargetSafetyAuditV2 { config } => print_sprint108_report(
            &config,
            "retired-target-safety-audit-v2",
            "retired_target_warning=unsafe retirement stays blocked and high-risk sentinels remain isolated",
            |report| report.retired_target_safety_audit_report_v2,
        ),
        Commands::SafetySentinelPreservationV2 { config } => print_sprint108_report(
            &config,
            "safety-sentinel-preservation-v2",
            "safety_sentinel_v2_warning=CommitteeCliSafety, workspace CLI safety, determinism, and paper lifecycle sentinels remain preserved",
            |report| report.safety_sentinel_preservation_report_v2,
        ),
        Commands::SharedFixtureHarnessExpansionV2 { config } => print_sprint108_report(
            &config,
            "shared-fixture-harness-expansion-v2",
            "shared_fixture_v2_warning=shared fixture harness expansion stays deterministic, local-only, and secret-free",
            |report| report.shared_fixture_harness_expansion_application_report_v2,
        ),
        Commands::SharedRenderHelperExpansionV2 { config } => print_sprint108_report(
            &config,
            "shared-render-helper-expansion-v2",
            "shared_render_v2_warning=shared render helper expansion is deterministic-only and never runtime/UI execution",
            |report| report.shared_render_helper_expansion_report_v2,
        ),
        Commands::CliSmokeTieringApplicationV2 { config } => print_sprint108_report(
            &config,
            "cli-smoke-tiering-application-v2",
            "cli_smoke_v2_warning=CLI smoke tiering keeps safety smoke explicit and CLI smoke is not full acceptance",
            |report| report.cli_smoke_tiering_application_report_v2,
        ),
        Commands::TestBinaryDeltaV5 { config } => print_sprint108_report(
            &config,
            "test-binary-delta-v5",
            "binary_delta_v5_warning=sample-backed delta is not measured and cannot claim measured reduction",
            |report| report.test_binary_delta_report_v5,
        ),
        Commands::ExtendedNoRunObservationV1 { config } => print_sprint108_report(
            &config,
            "extended-no-run-observation-v1",
            "extended_no_run_warning=extended no-run observation is diagnostic-only and no-run is not full acceptance",
            |report| report.extended_no_run_observation_report_v1,
        ),
        Commands::TimeoutCleanupVerificationV1 { config } => print_sprint108_report(
            &config,
            "timeout-cleanup-verification-v1",
            "timeout_cleanup_warning=timeout cleanup is not pass and timeout is not full acceptance",
            |report| report.timeout_cleanup_verification_report_v1,
        ),
        Commands::WorkspaceNoRunRecoveryGateV9 { config } => print_sprint108_report(
            &config,
            "workspace-no-run-recovery-gate-v9",
            "no_run_gate_v9_warning=no-run is not full acceptance; this gate only reports recovery status for cargo test --workspace --no-run --quiet",
            |report| report.workspace_no_run_recovery_gate_v9,
        ),
        Commands::WorkspaceFullAcceptanceGateV9 { config } => print_sprint108_report(
            &config,
            "workspace-full-acceptance-gate-v9",
            "full_gate_v9_warning=finished and passed required for full workspace acceptance; focused, CLI smoke, verification, and no-run cannot claim it",
            |report| report.workspace_full_acceptance_gate_v9,
        ),
        Commands::AcceptanceTruthGateV9 { config } => print_sprint108_report(
            &config,
            "acceptance-truth-gate-v9",
            "acceptance_truth_v9_warning=focused is not full, CLI smoke is not full, verification is not full, and no-run is not full pass",
            |report| report.acceptance_truth_gate_v9,
        ),
        Commands::ControlTowerSafeConsolidationPatchV2 { config } => print_sprint108_report(
            &config,
            "control-tower-safe-consolidation-patch-v2",
            "read_only_warning=control tower safe consolidation patch panel is static/read-only output with no run-tests button and no train/runtime/live/order/account/browser controls",
            |report| report.control_tower_safe_consolidation_patch_panel_v2,
        ),
        Commands::ControlTowerWorkspaceAcceptanceRecoveryV9 { config } => print_sprint108_report(
            &config,
            "control-tower-workspace-acceptance-recovery-v9",
            "read_only_warning=control tower workspace acceptance recovery panel is static/read-only output with no run-tests button and no train/runtime/live/order/account/browser controls",
            |report| report.control_tower_workspace_acceptance_recovery_panel_v9,
        ),
        Commands::Sprint109SafeConsolidationPatchV3 { config } => print_sprint109_report(
            &config,
            "sprint109-safe-consolidation-patch-v3",
            "sprint109_warning=research-only third safe consolidation patch only; paper-only, safe-consolidation-only, third-smallest-patch-only, no assertion deletion, no safety sentinel deletion, focused-is-not-full, no-run-is-not-full-acceptance, progress-is-not-acceptance, no runtime implementation, no training, no live inference, no live trading, no order/account command, no hidden skips, local-only paths, and remote paths rejected",
            |report| report,
        ),
        Commands::Sprint108VerificationCarryForward { config } => print_sprint109_report(
            &config,
            "sprint108-verification-carry-forward",
            "verification_carry_forward_warning=verification carry-forward is not full workspace acceptance",
            |report| report.sprint108_verification_carry_forward_report,
        ),
        Commands::PreviousPatchLedgerCarryForward { config } => print_sprint109_report(
            &config,
            "previous-patch-ledger-carry-forward",
            "previous_patch_ledger_warning=previous patch ledger carry-forward is diagnostic-only and does not imply full workspace acceptance",
            |report| report.previous_patch_ledger_carry_forward_report,
        ),
        Commands::CumulativeAssertionMigrationLedger { config } => print_sprint109_report(
            &config,
            "cumulative-assertion-migration-ledger",
            "cumulative_ledger_warning=no assertion deletion; cumulative ledger remains explicit",
            |report| report.cumulative_assertion_migration_ledger_report,
        ),
        Commands::ThirdSafeConsolidationPatchSelection { config } => print_sprint109_report(
            &config,
            "third-safe-consolidation-patch-selection",
            "third_selection_warning=third safe consolidation patch only; previous retired targets stay excluded and sentinels remain isolated",
            |report| report.third_safe_consolidation_patch_selection_report,
        ),
        Commands::AssertionMigrationLedgerV3 { config } => print_sprint109_report(
            &config,
            "assertion-migration-ledger-v3",
            "assertion_ledger_v3_warning=no assertion deletion; migrated assertions must remain explicit",
            |report| report.assertion_migration_ledger_v3,
        ),
        Commands::EquivalentCoverageProofV2 { config } => print_sprint109_report(
            &config,
            "equivalent-coverage-proof-v2",
            "equivalent_coverage_v2_warning=coverage required before retirement and no assertion deletion is allowed",
            |report| report.equivalent_coverage_proof_report_v2,
        ),
        Commands::RetiredTargetSafetyAuditV3 { config } => print_sprint109_report(
            &config,
            "retired-target-safety-audit-v3",
            "retired_target_v3_warning=unsafe retirement stays blocked and high-risk sentinels remain isolated",
            |report| report.retired_target_safety_audit_report_v3,
        ),
        Commands::SafetySentinelPreservationV3 { config } => print_sprint109_report(
            &config,
            "safety-sentinel-preservation-v3",
            "safety_sentinel_v3_warning=CommitteeCliSafety, workspace CLI safety, determinism, and paper lifecycle sentinels remain preserved",
            |report| report.safety_sentinel_preservation_report_v3,
        ),
        Commands::SharedFixtureHarnessExpansionV3 { config } => print_sprint109_report(
            &config,
            "shared-fixture-harness-expansion-v3",
            "shared_fixture_v3_warning=shared fixture harness expansion stays deterministic, local-only, and secret-free",
            |report| report.shared_fixture_harness_expansion_application_report_v3,
        ),
        Commands::SharedRenderHelperExpansionV3 { config } => print_sprint109_report(
            &config,
            "shared-render-helper-expansion-v3",
            "shared_render_v3_warning=shared render helper expansion is deterministic-only and never runtime/UI execution",
            |report| report.shared_render_helper_expansion_report_v3,
        ),
        Commands::CliSmokeTieringApplicationV3 { config } => print_sprint109_report(
            &config,
            "cli-smoke-tiering-application-v3",
            "cli_smoke_v3_warning=CLI smoke tiering keeps safety smoke explicit and CLI smoke is not full acceptance",
            |report| report.cli_smoke_tiering_application_report_v3,
        ),
        Commands::TestBinaryDeltaV6 { config } => print_sprint109_report(
            &config,
            "test-binary-delta-v6",
            "binary_delta_v6_warning=sample-backed delta is not measured and cannot claim measured reduction",
            |report| report.test_binary_delta_report_v6,
        ),
        Commands::CumulativeBinaryDeltaV1 { config } => print_sprint109_report(
            &config,
            "cumulative-binary-delta-v1",
            "cumulative_binary_delta_warning=sample-backed cumulative delta is not measured and cannot claim measured reduction",
            |report| report.cumulative_binary_delta_report_v1,
        ),
        Commands::ExtendedNoRunObservationV2 { config } => print_sprint109_report(
            &config,
            "extended-no-run-observation-v2",
            "extended_no_run_v2_warning=extended no-run observation is diagnostic-only and no-run is not full acceptance",
            |report| report.extended_no_run_observation_report_v2,
        ),
        Commands::WorkspaceCargoJsonProgressV3 { config } => print_sprint109_report(
            &config,
            "workspace-cargo-json-progress-v3",
            "cargo_json_progress_warning=cargo JSON progress is diagnostic-only and not acceptance",
            |report| report.workspace_cargo_json_progress_capture_v3,
        ),
        Commands::TimeoutCleanupVerificationV2 { config } => print_sprint109_report(
            &config,
            "timeout-cleanup-verification-v2",
            "timeout_cleanup_v2_warning=timeout cleanup is not pass and timeout is not full acceptance",
            |report| report.timeout_cleanup_verification_report_v2,
        ),
        Commands::WorkspaceNoRunRecoveryGateV10 { config } => print_sprint109_report(
            &config,
            "workspace-no-run-recovery-gate-v10",
            "no_run_gate_v10_warning=no-run is not full acceptance; this gate only reports recovery status for cargo test --workspace --no-run --quiet",
            |report| report.workspace_no_run_recovery_gate_v10,
        ),
        Commands::WorkspaceFullAcceptanceGateV10 { config } => print_sprint109_report(
            &config,
            "workspace-full-acceptance-gate-v10",
            "full_gate_v10_warning=finished and passed required for full workspace acceptance; focused, CLI smoke, verification, progress, and no-run cannot claim it",
            |report| report.workspace_full_acceptance_gate_v10,
        ),
        Commands::AcceptanceTruthGateV10 { config } => print_sprint109_report(
            &config,
            "acceptance-truth-gate-v10",
            "acceptance_truth_v10_warning=focused is not full, CLI smoke is not full, verification is not full, progress is not full, and no-run is not full pass",
            |report| report.acceptance_truth_gate_v10,
        ),
        Commands::ControlTowerSafeConsolidationPatchV3 { config } => print_sprint109_report(
            &config,
            "control-tower-safe-consolidation-patch-v3",
            "read_only_warning=control tower safe consolidation patch panel is static/read-only output with no run-tests button and no train/runtime/live/order/account/browser controls",
            |report| report.control_tower_safe_consolidation_patch_panel_v3,
        ),
        Commands::ControlTowerWorkspaceAcceptanceRecoveryV10 { config } => print_sprint109_report(
            &config,
            "control-tower-workspace-acceptance-recovery-v10",
            "read_only_warning=control tower workspace acceptance recovery panel is static/read-only output with no run-tests button and no train/runtime/live/order/account/browser controls",
            |report| report.control_tower_workspace_acceptance_recovery_panel_v10,
        ),
        Commands::Sprint110SafeConsolidationPatchV4 { config } => print_sprint110_report(
            &config,
            "sprint110-safe-consolidation-patch-v4",
            "sprint110_warning=research-only fourth safe consolidation patch only; Sprint109 validation import is not full acceptance; no assertion deletion; no safety sentinel deletion; equivalent coverage required; focused-is-not-full; no-run-is-not-full; cargo-build-is-not-full; CLI-smoke-is-not-full; verification-is-not-acceptance; timeout-cleanup-is-not-pass; no runtime implementation; no training; no live inference; no live trading; no order/account command; no hidden skips; local-only paths; and remote paths rejected",
            |report| report,
        ),
        Commands::Sprint109ValidationReconcile { config } => print_sprint110_report(
            &config,
            "sprint109-validation-reconcile",
            "sprint109_validation_warning=focused suite, CLI smoke, cargo build, and timeout cleanup imports are not full workspace acceptance",
            |report| report.sprint109_external_validation_reconciliation_report,
        ),
        Commands::Sprint109FocusedSuiteImport { config } => print_sprint110_report(
            &config,
            "sprint109-focused-suite-import",
            "focused_import_warning=focused suite import is not full workspace acceptance",
            |report| report.sprint109_focused_suite_result_import_report,
        ),
        Commands::Sprint109CliSmokeImport { config } => print_sprint110_report(
            &config,
            "sprint109-cli-smoke-import",
            "cli_smoke_import_warning=CLI smoke import is not full workspace acceptance",
            |report| report.sprint109_cli_smoke_result_import_report,
        ),
        Commands::Sprint109CargoBuildImport { config } => print_sprint110_report(
            &config,
            "sprint109-cargo-build-import",
            "cargo_build_import_warning=cargo build import is not full workspace acceptance",
            |report| report.sprint109_cargo_build_result_import_report,
        ),
        Commands::Sprint109WorkspaceTimeoutImport { config } => print_sprint110_report(
            &config,
            "sprint109-workspace-timeout-import",
            "workspace_timeout_import_warning=timeout cleanup is not pass and timeout import is not workspace acceptance",
            |report| report.sprint109_workspace_timeout_import_report,
        ),
        Commands::FourthSafeConsolidationPatchSelection { config } => print_sprint110_report(
            &config,
            "fourth-safe-consolidation-patch-selection",
            "fourth_selection_warning=fourth safe consolidation patch only; previously retired targets stay excluded and sentinels remain isolated",
            |report| report.fourth_safe_consolidation_patch_selection_report,
        ),
        Commands::AssertionMigrationLedgerV4 { config } => print_sprint110_report(
            &config,
            "assertion-migration-ledger-v4",
            "assertion_ledger_v4_warning=no assertion deletion; migrated assertions must remain explicit",
            |report| report.assertion_migration_ledger_v4,
        ),
        Commands::CumulativeAssertionMigrationLedgerV2 { config } => print_sprint110_report(
            &config,
            "cumulative-assertion-migration-ledger-v2",
            "cumulative_ledger_v2_warning=no assertion deletion; cumulative ledger remains explicit",
            |report| report.cumulative_assertion_migration_ledger_report,
        ),
        Commands::EquivalentCoverageProofV3 { config } => print_sprint110_report(
            &config,
            "equivalent-coverage-proof-v3",
            "equivalent_coverage_v3_warning=coverage required before retirement and no assertion deletion is allowed",
            |report| report.equivalent_coverage_proof_report_v3,
        ),
        Commands::RetiredTargetSafetyAuditV4 { config } => print_sprint110_report(
            &config,
            "retired-target-safety-audit-v4",
            "retired_target_v4_warning=unsafe retirement stays blocked and high-risk sentinels remain isolated",
            |report| report.retired_target_safety_audit_report_v4,
        ),
        Commands::SafetySentinelPreservationV4 { config } => print_sprint110_report(
            &config,
            "safety-sentinel-preservation-v4",
            "safety_sentinel_v4_warning=CommitteeCliSafety, workspace CLI safety, determinism, and paper lifecycle sentinels remain preserved",
            |report| report.safety_sentinel_preservation_report_v4,
        ),
        Commands::SharedFixtureHarnessExpansionV4 { config } => print_sprint110_report(
            &config,
            "shared-fixture-harness-expansion-v4",
            "shared_fixture_v4_warning=shared fixture harness expansion stays deterministic, local-only, and secret-free",
            |report| report.shared_fixture_harness_expansion_application_report_v4,
        ),
        Commands::SharedRenderHelperExpansionV4 { config } => print_sprint110_report(
            &config,
            "shared-render-helper-expansion-v4",
            "shared_render_v4_warning=shared render helper expansion is deterministic-only and never runtime/UI execution",
            |report| report.shared_render_helper_expansion_report_v4,
        ),
        Commands::CliSmokeTieringApplicationV4 { config } => print_sprint110_report(
            &config,
            "cli-smoke-tiering-application-v4",
            "cli_smoke_v4_warning=CLI smoke tiering keeps safety smoke explicit and CLI smoke is not full acceptance",
            |report| report.cli_smoke_tiering_application_report_v4,
        ),
        Commands::TestBinaryDeltaV7 { config } => print_sprint110_report(
            &config,
            "test-binary-delta-v7",
            "binary_delta_v7_warning=sample-backed delta is not measured and cannot claim measured reduction",
            |report| report.test_binary_delta_report_v7,
        ),
        Commands::CumulativeBinaryDeltaV2 { config } => print_sprint110_report(
            &config,
            "cumulative-binary-delta-v2",
            "cumulative_binary_delta_v2_warning=sample-backed cumulative delta is not measured and cannot claim measured reduction",
            |report| report.cumulative_binary_delta_report_v2,
        ),
        Commands::ExtendedNoRunObservationV3 { config } => print_sprint110_report(
            &config,
            "extended-no-run-observation-v3",
            "extended_no_run_v3_warning=extended no-run observation is diagnostic-only and no-run is not full acceptance",
            |report| report.extended_no_run_observation_report_v3,
        ),
        Commands::WorkspaceCargoJsonProgressV4 { config } => print_sprint110_report(
            &config,
            "workspace-cargo-json-progress-v4",
            "cargo_json_progress_v4_warning=cargo JSON progress is diagnostic-only and not acceptance",
            |report| report.workspace_cargo_json_progress_capture_v4,
        ),
        Commands::TimeoutCleanupVerificationV3 { config } => print_sprint110_report(
            &config,
            "timeout-cleanup-verification-v3",
            "timeout_cleanup_v3_warning=timeout cleanup is not pass and timeout is not full acceptance",
            |report| report.timeout_cleanup_verification_report_v3,
        ),
        Commands::WorkspaceNoRunRecoveryGateV11 { config } => print_sprint110_report(
            &config,
            "workspace-no-run-recovery-gate-v11",
            "no_run_gate_v11_warning=no-run is not full acceptance; this gate only reports recovery status for cargo test --workspace --no-run --quiet",
            |report| report.workspace_no_run_recovery_gate_v11,
        ),
        Commands::WorkspaceFullAcceptanceGateV11 { config } => print_sprint110_report(
            &config,
            "workspace-full-acceptance-gate-v11",
            "full_gate_v11_warning=finished and passed required for full workspace acceptance; focused, CLI smoke, cargo build, verification, progress, and no-run cannot claim it",
            |report| report.workspace_full_acceptance_gate_v11,
        ),
        Commands::AcceptanceTruthGateV11 { config } => print_sprint110_report(
            &config,
            "acceptance-truth-gate-v11",
            "acceptance_truth_v11_warning=focused is not full, CLI smoke is not full, cargo build is not full, verification is not full, progress is not full, and no-run is not full pass",
            |report| report.acceptance_truth_gate_v11,
        ),
        Commands::ControlTowerSafeConsolidationPatchV4 { config } => print_sprint110_report(
            &config,
            "control-tower-safe-consolidation-patch-v4",
            "read_only_warning=control tower safe consolidation patch panel is static/read-only output with no run-tests button and no train/runtime/live/order/account/browser controls",
            |report| report.control_tower_safe_consolidation_patch_panel_v4,
        ),
        Commands::ControlTowerWorkspaceAcceptanceRecoveryV11 { config } => print_sprint110_report(
            &config,
            "control-tower-workspace-acceptance-recovery-v11",
            "read_only_warning=control tower workspace acceptance recovery panel is static/read-only output with no run-tests button and no train/runtime/live/order/account/browser controls",
            |report| report.control_tower_workspace_acceptance_recovery_panel_v11,
        ),
        Commands::Sprint111WorkspaceTimeoutRootCause { config } => print_sprint111_report(
            &config,
            "sprint111-workspace-timeout-root-cause",
            "sprint111_warning=research-only paper-only timeout-root-cause-only bundle; fifth-patch-not-auto-applied; focused-is-not-full; CLI-smoke-is-not-full; cargo-build-is-not-full; no-run-is-not-full; cargo-progress-is-not-acceptance; timeout-cleanup-is-not-pass; no assertion deletion; no safety sentinel deletion; equivalent coverage required; no runtime implementation; no training; no live inference; no live trading; no order/account command; no runtime LLM live decision path; no investor impersonation; no auto-activation of 18 live agents; no silent confidence upgrade; no safety test deletion; no hidden skips; local-only paths; and remote paths rejected",
            |report| report,
        ),
        Commands::Sprint110BaselineTruthImport { config } => print_sprint111_report(
            &config,
            "sprint110-baseline-truth-import",
            "sprint110_truth_import_warning=focused, CLI smoke, cargo build, no-run timeout, and cleanup import are supporting only and never full acceptance",
            |report| report.sprint110_baseline_truth_import_report,
        ),
        Commands::CumulativeSafePatchLedgerV3 { config } => print_sprint111_report(
            &config,
            "cumulative-safe-patch-ledger-v3",
            "cumulative_safe_patch_ledger_v3_warning=no assertion deletion; retired targets remain visible; sample-backed delta is not measured timing",
            |report| report.cumulative_safe_patch_ledger_v3,
        ),
        Commands::WorkspaceTimeoutRootCause { config } => print_sprint111_report(
            &config,
            "workspace-timeout-root-cause",
            "workspace_timeout_root_cause_warning=timeout-root-cause-only; cargo progress and timeout evidence are diagnostic only and not acceptance",
            |report| report.workspace_timeout_root_cause_report,
        ),
        Commands::WorkspaceNoRunProgressTraceV1 { config } => print_sprint111_report(
            &config,
            "workspace-no-run-progress-trace-v1",
            "workspace_no_run_progress_warning=no-run progress is diagnostic/supporting only and is not full acceptance",
            |report| report.workspace_no_run_progress_trace_v1,
        ),
        Commands::CargoJsonProgressCaptureV5 { config } => print_sprint111_report(
            &config,
            "cargo-json-progress-capture-v5",
            "cargo_json_progress_v5_warning=cargo progress help stays diagnostic-only and not acceptance",
            |report| report.cargo_json_progress_capture_v5,
        ),
        Commands::CargoArtifactProgressTimeline { config } => print_sprint111_report(
            &config,
            "cargo-artifact-progress-timeline",
            "cargo_artifact_progress_warning=artifact timeline is diagnostic-only and timeout cleanup is not pass",
            |report| report.cargo_artifact_progress_timeline,
        ),
        Commands::CargoTargetStallAttribution { config } => print_sprint111_report(
            &config,
            "cargo-target-stall-attribution",
            "cargo_target_stall_warning=stall attribution is timeout-root-cause-only and does not claim workspace acceptance",
            |report| report.cargo_target_stall_attribution_report,
        ),
        Commands::IntegrationTestBinaryStall { config } => print_sprint111_report(
            &config,
            "integration-test-binary-stall",
            "integration_test_binary_stall_warning=integration stall evidence is diagnostic-only and no safety sentinel deletion is allowed",
            |report| report.integration_test_binary_stall_report,
        ),
        Commands::TestFamilyFanoutMapV2 { config } => print_sprint111_report(
            &config,
            "test-family-fanout-map-v2",
            "test_family_fanout_warning=fanout map is diagnostic-only and sentinel clusters remain isolated",
            |report| report.test_family_fanout_map_v2,
        ),
        Commands::WorkspaceTargetClusterMapV2 { config } => print_sprint111_report(
            &config,
            "workspace-target-cluster-map-v2",
            "workspace_target_cluster_map_warning=cluster mapping is timeout-root-cause-only and not a patch application command",
            |report| report.workspace_target_cluster_map_v2,
        ),
        Commands::HighFanoutResidualTarget { config } => print_sprint111_report(
            &config,
            "high-fanout-residual-target",
            "high_fanout_residual_warning=residual target reporting is diagnostic-only and already retired targets stay excluded",
            |report| report.high_fanout_residual_target_report,
        ),
        Commands::RemainingSafeCandidatePool { config } => print_sprint111_report(
            &config,
            "remaining-safe-candidate-pool",
            "remaining_candidate_pool_warning=candidate pool is research-only, excludes already retired/sentinel targets, and does not auto-apply a patch",
            |report| report.remaining_safe_consolidation_candidate_pool_report,
        ),
        Commands::FifthPatchCandidatePreselection { config } => print_sprint111_report(
            &config,
            "fifth-patch-candidate-preselection",
            "fifth_patch_preselection_warning=paper-only candidate preselection does not apply a patch and keeps equivalent coverage mandatory",
            |report| report.fifth_patch_candidate_preselection_report,
        ),
        Commands::FifthPatchDecisionGate { config } => print_sprint111_report(
            &config,
            "fifth-patch-decision-gate",
            "fifth_patch_decision_warning=patch is not auto-applied; equivalent coverage, assertion migration, sentinel preservation, no hidden skips, and timeout evidence remain mandatory",
            |report| report.fifth_patch_decision_gate,
        ),
        Commands::AssertionLedgerContinuityCheckV1 { config } => print_sprint111_report(
            &config,
            "assertion-ledger-continuity-check-v1",
            "assertion_ledger_continuity_warning=no assertion deletion and cumulative ledger continuity must remain explicit",
            |report| report.assertion_ledger_continuity_check_v1,
        ),
        Commands::EquivalentCoverageContinuityCheckV1 { config } => print_sprint111_report(
            &config,
            "equivalent-coverage-continuity-check-v1",
            "equivalent_coverage_continuity_warning=equivalent coverage remains mandatory before any retirement",
            |report| report.equivalent_coverage_continuity_check_v1,
        ),
        Commands::TimeoutWindowAdequacyV1 { config } => print_sprint111_report(
            &config,
            "timeout-window-adequacy-v1",
            "timeout_window_warning=timeout length affects observation quality but timeout is neither correctness failure nor pass",
            |report| report.timeout_window_adequacy_report_v1,
        ),
        Commands::AcceptanceEvidenceStrengthV1 { config } => print_sprint111_report(
            &config,
            "acceptance-evidence-strength-v1",
            "acceptance_evidence_warning=focused, CLI smoke, cargo build, no-run, and cargo progress do not claim full acceptance; full finished workspace pass is required",
            |report| report.acceptance_evidence_strength_report_v1,
        ),
        Commands::WorkspaceRecoveryDecisionV1 { config } => print_sprint111_report(
            &config,
            "workspace-recovery-decision-v1",
            "workspace_recovery_warning=workspace recovery decision is timeout-root-cause-only and fifth patch remains gated",
            |report| report.workspace_recovery_decision_report_v1,
        ),
        Commands::ControlTowerWorkspaceTimeoutRootCause { config } => print_sprint111_report(
            &config,
            "control-tower-workspace-timeout-root-cause",
            "read_only_warning=control tower workspace timeout root-cause panel is static/read-only output with no run-tests button and no train/runtime/live/order/account controls",
            |report| report.control_tower_workspace_timeout_root_cause_panel,
        ),
        Commands::ControlTowerFifthPatchDecision { config } => print_sprint111_report(
            &config,
            "control-tower-fifth-patch-decision",
            "read_only_warning=control tower fifth patch decision panel is static/read-only output with no apply-patch button, no run-tests button, and no train/runtime/live/order/account controls",
            |report| report.control_tower_fifth_patch_decision_panel,
        ),
        Commands::Sprint112WorkspaceDiagnosticPilot { config } => print_sprint112_report(
            &config,
            "sprint112-workspace-diagnostic-pilot",
            "sprint112_warning=research-only paper-only diagnostic-only bundle; nextest-is-not-acceptance; sccache-is-not-speedup-proof; fifth-patch-not-applied; focused-is-not-full; CLI-smoke-is-not-full; cargo-build-is-not-full; no-run-is-not-full; cargo-progress-is-not-acceptance; timeout-cleanup-is-not-pass; no assertion deletion; no safety sentinel deletion; no runtime implementation; no training; no live inference; no live trading; no order/account command; no runtime LLM live decision path; no investor impersonation; no auto-activation of 18 live agents; no silent confidence upgrade; no safety test deletion; no hidden skips; local-only paths; and remote paths rejected",
            |report| report,
        ),
        Commands::Sprint111BaselineTruthImport { config } => print_sprint112_report(
            &config,
            "sprint111-baseline-truth-import",
            "sprint111_truth_import_warning=focused, CLI smoke, cargo check/build, timeout, progress, and cleanup import remain supporting only and never full acceptance",
            |report| report.sprint111_baseline_truth_import_report,
        ),
        Commands::NextestAvailabilityV1 { config } => print_sprint112_report(
            &config,
            "nextest-availability-v1",
            "nextest_warning=research-only paper-only diagnostic-only; nextest-is-not-acceptance; no hidden skips; local-only paths; and remote paths rejected",
            |report| report.nextest_availability_report_v1,
        ),
        Commands::NextestPilotExecutionV1 { config } => print_sprint112_report(
            &config,
            "nextest-pilot-execution-v1",
            "nextest_execution_warning=research-only paper-only diagnostic-only; nextest-is-not-acceptance; fifth-patch-not-applied; and local-only paths",
            |report| report.nextest_pilot_execution_report_v1,
        ),
        Commands::NextestSlowTargetAttributionV1 { config } => print_sprint112_report(
            &config,
            "nextest-slow-target-attribution-v1",
            "nextest_slow_warning=research-only paper-only diagnostic-only; nextest-is-not-acceptance; no hidden skips; and local-only paths",
            |report| report.nextest_slow_target_attribution_report_v1,
        ),
        Commands::SccacheAvailabilityV1 { config } => print_sprint112_report(
            &config,
            "sccache-availability-v1",
            "sccache_warning=research-only paper-only diagnostic-only; sccache-is-not-speedup-proof; local-only paths; remote cache forbidden; and no secrets",
            |report| report.sccache_availability_report_v1,
        ),
        Commands::SccacheLocalOnlyPolicyV1 { config } => print_sprint112_report(
            &config,
            "sccache-local-only-policy-v1",
            "sccache_policy_warning=research-only paper-only diagnostic-only; local-only cache; remote cache forbidden; secret cache forbidden; deterministic keys required; and cache failure must not hide failure",
            |report| report.sccache_local_only_policy_report_v1,
        ),
        Commands::SccacheEffectEstimateV1 { config } => print_sprint112_report(
            &config,
            "sccache-effect-estimate-v1",
            "sccache_effect_warning=research-only paper-only diagnostic-only; sccache-is-not-speedup-proof; no guaranteed speedup claim; and local-only paths",
            |report| report.sccache_effect_estimate_report_v1,
        ),
        Commands::CargoCheckTimingCaptureV1 { config } => print_sprint112_report(
            &config,
            "cargo-check-timing-capture-v1",
            "cargo_check_warning=research-only paper-only diagnostic-only; cargo-check-is-not-full; full finished and passed workspace tests are still required",
            |report| report.cargo_check_timing_capture_v1,
        ),
        Commands::CargoBuildTimingCaptureV1 { config } => print_sprint112_report(
            &config,
            "cargo-build-timing-capture-v1",
            "cargo_build_warning=research-only paper-only diagnostic-only; cargo-build-is-not-full; full finished and passed workspace tests are still required",
            |report| report.cargo_build_timing_capture_v1,
        ),
        Commands::CargoJsonProgressCaptureV6 { config } => print_sprint112_report(
            &config,
            "cargo-json-progress-capture-v6",
            "cargo_json_v6_warning=research-only paper-only diagnostic-only; cargo-progress-is-not-acceptance; timeout-cleanup-is-not-pass; and local-only paths",
            |report| report.cargo_json_progress_capture_v6,
        ),
        Commands::WorkspaceDiagnosticEvidenceMatrixV1 { config } => print_sprint112_report(
            &config,
            "workspace-diagnostic-evidence-matrix-v1",
            "diagnostic_matrix_warning=research-only paper-only diagnostic-only; nextest-is-not-acceptance; sccache-is-not-speedup-proof; focused-is-not-full; CLI-smoke-is-not-full; cargo-build-is-not-full; no-run-is-not-full; cargo-progress-is-not-acceptance; and full finished+passed workspace tests are required for full acceptance",
            |report| report.workspace_diagnostic_evidence_matrix_v1,
        ),
        Commands::WorkspaceTimeoutRootCauseV2 { config } => print_sprint112_report(
            &config,
            "workspace-timeout-root-cause-v2",
            "root_cause_v2_warning=research-only paper-only diagnostic-only; observed-vs-inferred evidence remains separate; timeout-cleanup-is-not-pass; and no fifth patch is applied",
            |report| report.workspace_timeout_root_cause_report_v2,
        ),
        Commands::RemainingSafeCandidatePoolV2 { config } => print_sprint112_report(
            &config,
            "remaining-safe-candidate-pool-v2",
            "candidate_pool_v2_warning=research-only paper-only diagnostic-only; fifth-patch-not-applied; no broad consolidation; no assertion deletion; no safety sentinel deletion; and local-only paths",
            |report| report.remaining_safe_candidate_pool_report_v2,
        ),
        Commands::FifthPatchDecisionGateV2 { config } => print_sprint112_report(
            &config,
            "fifth-patch-decision-gate-v2",
            "fifth_patch_v2_warning=research-only paper-only diagnostic-only; re-evaluation only; fifth-patch-not-applied; nextest-is-not-acceptance; sccache-is-not-speedup-proof; no assertion deletion; no safety sentinel deletion; and no hidden skips",
            |report| report.fifth_patch_decision_gate_v2,
        ),
        Commands::FifthPatchNoApplyGuaranteeV1 { config } => print_sprint112_report(
            &config,
            "fifth-patch-no-apply-guarantee-v1",
            "fifth_patch_no_apply_warning=research-only paper-only diagnostic-only; fifth-patch-not-applied; no files retired; no assertions moved; and local-only paths",
            |report| report.fifth_patch_no_apply_guarantee_report_v1,
        ),
        Commands::AcceptanceEvidenceStrengthV2 { config } => print_sprint112_report(
            &config,
            "acceptance-evidence-strength-v2",
            "acceptance_v2_warning=research-only paper-only diagnostic-only; nextest-is-not-acceptance; sccache-is-not-speedup-proof; focused-is-not-full; CLI-smoke-is-not-full; cargo-build-is-not-full; no-run-is-not-full; cargo-progress-is-not-acceptance; and full finished+passed workspace tests are required",
            |report| report.acceptance_evidence_strength_report_v2,
        ),
        Commands::WorkspaceRecoveryDecisionV2 { config } => print_sprint112_report(
            &config,
            "workspace-recovery-decision-v2",
            "workspace_recovery_v2_warning=research-only paper-only diagnostic-only; fifth-patch-not-applied; nextest-is-not-acceptance; sccache-is-not-speedup-proof; and local-only paths",
            |report| report.workspace_recovery_decision_report_v2,
        ),
        Commands::ControlTowerWorkspaceDiagnosticPilot { config } => print_sprint112_report(
            &config,
            "control-tower-workspace-diagnostic-pilot",
            "read_only_warning=control tower workspace diagnostic pilot panel is static/read-only output with no run button, no train/runtime/live/order/account controls, no browser execution, and no patch application controls",
            |report| report.control_tower_workspace_diagnostic_pilot_panel,
        ),
        Commands::ControlTowerFifthPatchReevaluation { config } => print_sprint112_report(
            &config,
            "control-tower-fifth-patch-reevaluation",
            "read_only_warning=control tower fifth patch reevaluation panel is static/read-only output with no apply-patch button, no run button, and no train/runtime/live/order/account controls",
            |report| report.control_tower_fifth_patch_reevaluation_panel,
        ),
        Commands::Sprint113RealWorkspaceObservation { config } => print_sprint113_report(
            &config,
            "sprint113-real-workspace-observation",
            "sprint113_warning=research-only paper-only real-observation-diagnostic bundle; fifth-patch-not-applied; nextest-is-not-cargo-workspace-acceptance; sccache-is-not-speedup-proof; cargo-progress-is-not-acceptance; timeout-cleanup-is-not-pass; focused-is-not-full; CLI-smoke-is-not-full; cargo-build-is-not-full; no-run-is-not-full; no assertion deletion; no safety sentinel deletion; no runtime implementation; no training; no live inference; no live trading; no order/account command; no runtime LLM live decision path; no investor impersonation; no auto-activation of 18 live agents; no silent confidence upgrade; no safety test deletion; no hidden skips; local-only paths; and remote paths rejected",
            |report| report,
        ),
        Commands::Sprint112BaselineTruthImport { config } => print_sprint113_report(
            &config,
            "sprint112-baseline-truth-import",
            "sprint112_truth_import_warning=sprint112 truth import stays supporting-only and imported_as_full_acceptance=false",
            |report| report.sprint112_baseline_truth_import_report,
        ),
        Commands::Sprint112VerificationPatchCarryForward { config } => print_sprint113_report(
            &config,
            "sprint112-verification-patch-carry-forward",
            "sprint112_patch_warning=storage_report and summary stay included; actual cleanup counts stay actual; actual cargo JSON parsing stays required; real observations are not overwritten by fixtures; and fifth gate still requires a LowRiskCandidate",
            |report| report.sprint112_verification_patch_carry_forward_report,
        ),
        Commands::SuspectTargetFamilyRegistryV1 { config } => print_sprint113_report(
            &config,
            "suspect-target-family-registry-v1",
            "suspect_registry_warning=research-only paper-only diagnostic-only; retired targets excluded; sentinels excluded; and local-only paths",
            |report| report.suspect_target_family_registry_v1,
        ),
        Commands::RealCargoNoRunObservationV1 { config } => print_sprint113_report(
            &config,
            "real-cargo-no-run-observation-v1",
            "real_no_run_warning=research-only paper-only real-observation-diagnostic; timeout-cleanup-is-not-pass; no-run-is-not-full; and local-only paths",
            |report| report.real_cargo_no_run_observation_v1,
        ),
        Commands::RealCargoJsonProgressObservationV1 { config } => print_sprint113_report(
            &config,
            "real-cargo-json-progress-observation-v1",
            "cargo_json_v1_warning=research-only paper-only real-observation-diagnostic; cargo-progress-is-not-acceptance; actual JSON parsing remains explicit; timeout-cleanup-is-not-pass; and local-only paths",
            |report| report.real_cargo_json_progress_observation_v1,
        ),
        Commands::RealNextestProbeV1 { config } => print_sprint113_report(
            &config,
            "real-nextest-probe-v1",
            "nextest_probe_warning=research-only paper-only real-observation-diagnostic; nextest-is-not-cargo-workspace-acceptance; no hidden skips; and local-only paths",
            |report| report.real_nextest_probe_execution_report_v1,
        ),
        Commands::RealSccacheProbeV1 { config } => print_sprint113_report(
            &config,
            "real-sccache-probe-v1",
            "sccache_probe_warning=research-only paper-only real-observation-diagnostic; sccache-is-not-speedup-proof; local-only cache only; remote cache forbidden; and no secrets",
            |report| report.real_sccache_probe_execution_report_v1,
        ),
        Commands::WorkspaceTimeoutRootCauseV3 { config } => print_sprint113_report(
            &config,
            "workspace-timeout-root-cause-v3",
            "root_cause_v3_warning=research-only paper-only real-observation-diagnostic; observed-vs-inferred evidence remains separate; timeout-cleanup-is-not-pass; and no fifth patch is applied",
            |report| report.workspace_timeout_root_cause_report_v3,
        ),
        Commands::FifthPatchDecisionGateV3 { config } => print_sprint113_report(
            &config,
            "fifth-patch-decision-gate-v3",
            "fifth_patch_v3_warning=research-only paper-only real-observation-diagnostic; gate-only; fifth-patch-not-applied; allowed_for_next_sprint never means applied; no assertion deletion; no safety sentinel deletion; and no hidden skips",
            |report| report.fifth_patch_decision_gate_v3,
        ),
        Commands::FifthPatchNoApplyGuaranteeV2 { config } => print_sprint113_report(
            &config,
            "fifth-patch-no-apply-guarantee-v2",
            "fifth_patch_no_apply_v2_warning=research-only paper-only real-observation-diagnostic; fifth-patch-not-applied; no files retired; no assertions moved; and local-only paths",
            |report| report.fifth_patch_no_apply_guarantee_report_v2,
        ),
        Commands::AcceptanceTruthGateV14 { config } => print_sprint113_report(
            &config,
            "acceptance-truth-gate-v14",
            "acceptance_v14_warning=research-only paper-only real-observation-diagnostic; nextest-is-not-cargo-workspace-acceptance; sccache-is-not-speedup-proof; cargo-progress-is-not-acceptance; timeout-cleanup-is-not-pass; focused-is-not-full; CLI-smoke-is-not-full; cargo-build-is-not-full; no-run-is-not-full; and only full finished and passed workspace tests can claim full acceptance",
            |report| report.acceptance_truth_gate_v14,
        ),
        Commands::ControlTowerRealWorkspaceObservation { config } => print_sprint113_report(
            &config,
            "control-tower-real-workspace-observation",
            "read_only_warning=control tower real workspace observation panel is static/read-only output with no run button, no apply-patch button, and no train/runtime/live/order/account controls",
            |report| report.control_tower_real_workspace_observation_panel,
        ),
        Commands::ControlTowerFifthPatchEvidenceGate { config } => print_sprint113_report(
            &config,
            "control-tower-fifth-patch-evidence-gate",
            "read_only_warning=control tower fifth patch evidence gate panel is static/read-only output with no apply-patch button, no run button, and no train/runtime/live/order/account controls",
            |report| report.control_tower_fifth_patch_evidence_gate_panel,
        ),
        Commands::Sprint114MixedFamilyIsolation { config } => print_sprint114_report(
            &config,
            "sprint114-mixed-family-isolation",
            "sprint114_warning=research-only paper-only mixed-family-isolation-only bundle; fifth-patch-not-applied; fifth-patch-ready-does-not-mean-applied; focused-is-not-full; CLI-smoke-is-not-full; cargo-build-is-not-full; no-run-is-not-full; cargo-progress-is-not-acceptance; timeout-cleanup-is-not-pass; no assertion deletion; no safety sentinel deletion; no runtime implementation; no training; no live inference; no live trading; no order/account command; no runtime LLM live decision path; no investor impersonation; no auto-activation of 18 live agents; no silent confidence upgrade; no safety test deletion; no hidden skips; local-only paths; and remote paths rejected",
            |report| report,
        ),
        Commands::Sprint113BaselineTruthImport { config } => print_sprint114_report(
            &config,
            "sprint113-baseline-truth-import",
            "sprint113_truth_import_warning=sprint113 truth import stays supporting-only and imported_as_full_acceptance=false",
            |report| report.sprint113_baseline_truth_import_report,
        ),
        Commands::StillMixedFamilyRegistryV1 { config } => print_sprint114_report(
            &config,
            "still-mixed-family-registry-v1",
            "mixed_family_registry_warning=research-only paper-only mixed-family-isolation-only; isolated families are carried forward; fifth-patch-not-applied; and local-only paths",
            |report| report.still_mixed_family_registry_v1,
        ),
        Commands::IntegrationFanoutNarrowingV1 { config } => print_sprint114_report(
            &config,
            "integration-fanout-narrowing-v1",
            "integration_narrowing_warning=research-only paper-only mixed-family-isolation-only; observed-vs-inferred evidence remains separate; fifth-patch-not-applied; and local-only paths",
            |report| report.integration_fanout_narrowing_report_v1,
        ),
        Commands::LinkTimeNarrowingV1 { config } => print_sprint114_report(
            &config,
            "link-time-narrowing-v1",
            "link_time_warning=research-only paper-only mixed-family-isolation-only; observed-vs-inferred evidence remains separate; fifth-patch-not-applied; and local-only paths",
            |report| report.link_time_narrowing_report_v1,
        ),
        Commands::MacroExpansionNarrowingV1 { config } => print_sprint114_report(
            &config,
            "macro-expansion-narrowing-v1",
            "macro_expansion_warning=research-only paper-only mixed-family-isolation-only; observed-vs-inferred evidence remains separate; fifth-patch-not-applied; and local-only paths",
            |report| report.macro_expansion_narrowing_report_v1,
        ),
        Commands::SuspectTargetDecompositionV1 { config } => print_sprint114_report(
            &config,
            "suspect-target-decomposition-v1",
            "suspect_target_decomposition_warning=research-only paper-only mixed-family-isolation-only; target decomposition is diagnostic-only; fifth-patch-not-applied; and local-only paths",
            |report| report.suspect_target_decomposition_report_v1,
        ),
        Commands::TargetAssertionInventoryV1 { config } => print_sprint114_report(
            &config,
            "target-assertion-inventory-v1",
            "assertion_inventory_warning=research-only paper-only mixed-family-isolation-only; no assertion deletion; no safety test deletion; fifth-patch-not-applied; and local-only paths",
            |report| report.target_assertion_inventory_report_v1,
        ),
        Commands::AssertionMigrationFeasibilityDrilldownV1 { config } => print_sprint114_report(
            &config,
            "assertion-migration-feasibility-drilldown-v1",
            "assertion_migration_warning=research-only paper-only mixed-family-isolation-only; fifth-patch-not-applied; fifth-patch-ready-does-not-mean-applied; no assertion deletion; no hidden skips; and local-only paths",
            |report| report.assertion_migration_feasibility_drilldown_report_v1,
        ),
        Commands::EquivalentCoverageFeasibilityDrilldownV1 { config } => print_sprint114_report(
            &config,
            "equivalent-coverage-feasibility-drilldown-v1",
            "equivalent_coverage_warning=research-only paper-only mixed-family-isolation-only; equivalent coverage proof is required; fifth-patch-not-applied; and local-only paths",
            |report| report.equivalent_coverage_feasibility_drilldown_report_v1,
        ),
        Commands::FifthPatchDecisionGateV4 { config } => print_sprint114_report(
            &config,
            "fifth-patch-decision-gate-v4",
            "fifth_patch_v4_warning=research-only paper-only mixed-family-isolation-only; gate-only; fifth-patch-not-applied; fifth-patch-ready-does-not-mean-applied; no assertion deletion; no safety sentinel deletion; no hidden skips; and local-only paths",
            |report| report.fifth_patch_decision_gate_v4,
        ),
        Commands::FifthPatchApplyPlanForNextSprint { config } => print_sprint114_report(
            &config,
            "fifth-patch-apply-plan-for-next-sprint",
            "fifth_patch_apply_plan_warning=research-only paper-only mixed-family-isolation-only; next sprint only; plan-only; fifth-patch-not-applied; no files retired; no assertions moved; and local-only paths",
            |report| report.fifth_patch_apply_plan_for_next_sprint_v1,
        ),
        Commands::FifthPatchNoApplyGuaranteeV3 { config } => print_sprint114_report(
            &config,
            "fifth-patch-no-apply-guarantee-v3",
            "fifth_patch_no_apply_v3_warning=research-only paper-only mixed-family-isolation-only; fifth-patch-not-applied; no files retired; no assertions moved; and local-only paths",
            |report| report.fifth_patch_no_apply_guarantee_report_v3,
        ),
        Commands::CandidateStopConsolidationV1 { config } => print_sprint114_report(
            &config,
            "candidate-stop-consolidation-v1",
            "candidate_stop_warning=research-only paper-only mixed-family-isolation-only; stop-consolidation is allowed; fifth-patch-not-applied; and local-only paths",
            |report| report.candidate_stop_consolidation_report_v1,
        ),
        Commands::CargoJsonSuspectTargetTraceV1 { config } => print_sprint114_report(
            &config,
            "cargo-json-suspect-target-trace-v1",
            "cargo_json_trace_warning=research-only paper-only mixed-family-isolation-only; cargo-progress-is-not-acceptance; timeout-cleanup-is-not-pass; fifth-patch-not-applied; and local-only paths",
            |report| report.cargo_json_suspect_target_trace_v1,
        ),
        Commands::LinkMacroEvidenceMatrixV1 { config } => print_sprint114_report(
            &config,
            "link-macro-evidence-matrix-v1",
            "link_macro_matrix_warning=research-only paper-only mixed-family-isolation-only; observed-vs-inferred evidence remains separate; fifth-patch-not-applied; and local-only paths",
            |report| report.link_macro_evidence_matrix_v1,
        ),
        Commands::IntegrationFanoutEvidenceMatrixV1 { config } => print_sprint114_report(
            &config,
            "integration-fanout-evidence-matrix-v1",
            "integration_matrix_warning=research-only paper-only mixed-family-isolation-only; observed-vs-inferred evidence remains separate; fifth-patch-not-applied; and local-only paths",
            |report| report.integration_fanout_evidence_matrix_v1,
        ),
        Commands::AcceptanceTruthGateV15 { config } => print_sprint114_report(
            &config,
            "acceptance-truth-gate-v15",
            "acceptance_v15_warning=research-only paper-only mixed-family-isolation-only; focused-is-not-full; CLI-smoke-is-not-full; cargo-build-is-not-full; no-run-is-not-full; timeout-cleanup-is-not-pass; cargo-progress-is-not-acceptance; and only full finished and passed workspace tests can claim full acceptance",
            |report| report.acceptance_truth_gate_v15,
        ),
        Commands::ControlTowerMixedFamilyIsolation { config } => print_sprint114_report(
            &config,
            "control-tower-mixed-family-isolation",
            "read_only_warning=control tower mixed-family isolation panel is static/read-only output with no run button, no apply-patch button, and no train/runtime/live/order/account controls",
            |report| report.control_tower_mixed_family_isolation_panel,
        ),
        Commands::ControlTowerFifthPatchReadinessV4 { config } => print_sprint114_report(
            &config,
            "control-tower-fifth-patch-readiness-v4",
            "read_only_warning=control tower fifth patch readiness panel is static/read-only output with no apply-patch button, no run button, and no train/runtime/live/order/account controls",
            |report| report.control_tower_fifth_patch_readiness_panel_v4,
        ),
        Commands::Sprint115ConsolidationGovernance { config } => print_sprint115_report(
            &config,
            "sprint115-consolidation-governance",
            "sprint115_warning=research-only paper-only consolidation-governance-only bundle; fifth-patch-not-applied; no-target-retirement; no-assertion-movement; stop-consolidation-is-valid; focused-is-not-full; CLI-smoke-is-not-full; cargo-build-is-not-full; no-run-is-not-full; cargo-progress-is-not-acceptance; timeout-cleanup-is-not-pass; no assertion deletion; no safety sentinel deletion; no runtime implementation; no training; no live inference; no live trading; no order/account command; no runtime LLM live decision path; no investor impersonation; no auto-activation of 18 live agents; no silent confidence upgrade; no safety test deletion; no hidden skips; local-only paths; and remote paths rejected",
            |report| report,
        ),
        Commands::Sprint114BaselineTruthImport { config } => print_sprint115_report(
            &config,
            "sprint114-baseline-truth-import",
            "sprint114_truth_import_warning=sprint114 truth import stays supporting-only and imported_as_full_acceptance=false",
            |report| report.sprint114_baseline_truth_import_report,
        ),
        Commands::Sprint114StopRecommendationCarryForward { config } => print_sprint115_report(
            &config,
            "sprint114-stop-recommendation-carry-forward",
            "sprint114_stop_warning=research-only paper-only consolidation-governance-only; stop-consolidation-is-valid; fifth-patch-not-applied; and local-only paths",
            |report| report.sprint114_stop_recommendation_carry_forward_report,
        ),
        Commands::ConsolidationStopDecisionV1 { config } => print_sprint115_report(
            &config,
            "consolidation-stop-decision-v1",
            "consolidation_stop_warning=research-only paper-only consolidation-governance-only; stop-consolidation-is-valid; fifth-patch-not-applied; no-target-retirement; no-assertion-movement; and local-only paths",
            |report| report.consolidation_stop_decision_report_v1,
        ),
        Commands::ConsolidationResumeDecisionV1 { config } => print_sprint115_report(
            &config,
            "consolidation-resume-decision-v1",
            "consolidation_resume_warning=research-only paper-only consolidation-governance-only; proof before movement; fifth-patch-not-applied; and local-only paths",
            |report| report.consolidation_resume_decision_report_v1,
        ),
        Commands::ConsolidationDecisionMatrixV1 { config } => print_sprint115_report(
            &config,
            "consolidation-decision-matrix-v1",
            "consolidation_matrix_warning=research-only paper-only consolidation-governance-only; pause-stop-split are valid outcomes; fifth-patch-not-applied; and local-only paths",
            |report| report.consolidation_decision_matrix_v1,
        ),
        Commands::AssertionDestinationProofPlanV1 { config } => print_sprint115_report(
            &config,
            "assertion-destination-proof-plan-v1",
            "assertion_proof_plan_warning=research-only paper-only consolidation-governance-only; proof before movement; no-assertion-movement; and local-only paths",
            |report| report.assertion_destination_proof_plan_v1,
        ),
        Commands::AssertionDestinationCapacityV1 { config } => print_sprint115_report(
            &config,
            "assertion-destination-capacity-v1",
            "assertion_capacity_warning=research-only paper-only consolidation-governance-only; capacity proof only; no-assertion-movement; and local-only paths",
            |report| report.assertion_destination_capacity_report_v1,
        ),
        Commands::EvidenceBlurRiskV1 { config } => print_sprint115_report(
            &config,
            "evidence-blur-risk-v1",
            "evidence_blur_warning=research-only paper-only consolidation-governance-only; evidence blur can block consolidation; fifth-patch-not-applied; and local-only paths",
            |report| report.evidence_blur_risk_report_v1,
        ),
        Commands::AssertionDestinationProofGateV1 { config } => print_sprint115_report(
            &config,
            "assertion-destination-proof-gate-v1",
            "assertion_proof_gate_warning=research-only paper-only consolidation-governance-only; proof before movement; fifth-patch-not-applied; and local-only paths",
            |report| report.assertion_destination_proof_gate_v1,
        ),
        Commands::EvidenceBlurRiskGateV1 { config } => print_sprint115_report(
            &config,
            "evidence-blur-risk-gate-v1",
            "evidence_blur_gate_warning=research-only paper-only consolidation-governance-only; controlled blur does not mean patch applied; and local-only paths",
            |report| report.evidence_blur_risk_gate_v1,
        ),
        Commands::FifthPatchResumeGateV5 { config } => print_sprint115_report(
            &config,
            "fifth-patch-resume-gate-v5",
            "fifth_patch_resume_warning=research-only paper-only consolidation-governance-only; later sprint only; patch not applied; no-target-retirement; no-assertion-movement; and local-only paths",
            |report| report.fifth_patch_resume_gate_v5,
        ),
        Commands::FifthPatchStopGateV1 { config } => print_sprint115_report(
            &config,
            "fifth-patch-stop-gate-v1",
            "fifth_patch_stop_warning=research-only paper-only consolidation-governance-only; stop-consolidation-is-valid; patch not applied; and local-only paths",
            |report| report.fifth_patch_stop_gate_v1,
        ),
        Commands::CandidateStopConsolidationV2 { config } => print_sprint115_report(
            &config,
            "candidate-stop-consolidation-v2",
            "candidate_stop_v2_warning=research-only paper-only consolidation-governance-only; honest stop recommendation; patch not applied; and local-only paths",
            |report| report.candidate_stop_consolidation_report_v2,
        ),
        Commands::ConsolidationTrackPauseV1 { config } => print_sprint115_report(
            &config,
            "consolidation-track-pause-v1",
            "consolidation_pause_warning=research-only paper-only consolidation-governance-only; pause is valid; patch not applied; and local-only paths",
            |report| report.consolidation_track_pause_report_v1,
        ),
        Commands::WorkspaceTimeoutTrackSplitV1 { config } => print_sprint115_report(
            &config,
            "workspace-timeout-track-split-v1",
            "workspace_timeout_split_warning=research-only paper-only consolidation-governance-only; workspace diagnostics split from consolidation; focused-is-not-full; and local-only paths",
            |report| report.workspace_timeout_track_split_report_v1,
        ),
        Commands::WorkspaceTimeoutDiagnosticTrackPlanV1 { config } => print_sprint115_report(
            &config,
            "workspace-timeout-diagnostic-track-plan-v1",
            "workspace_timeout_plan_warning=research-only paper-only consolidation-governance-only; diagnostic-only; cargo-progress-is-not-acceptance; and local-only paths",
            |report| report.workspace_timeout_diagnostic_track_plan_v1,
        ),
        Commands::WorkspaceNoRunObservationPlanV2 { config } => print_sprint115_report(
            &config,
            "workspace-no-run-observation-plan-v2",
            "workspace_no_run_plan_warning=research-only paper-only consolidation-governance-only; no-run-is-not-full; timeout-cleanup-is-not-pass; and local-only paths",
            |report| report.workspace_no_run_observation_plan_v2,
        ),
        Commands::WorkspaceFullObservationPlanV2 { config } => print_sprint115_report(
            &config,
            "workspace-full-observation-plan-v2",
            "workspace_full_plan_warning=research-only paper-only consolidation-governance-only; focused-is-not-full; CLI-smoke-is-not-full; cargo-build-is-not-full; and only full workspace pass can claim full acceptance",
            |report| report.workspace_full_observation_plan_v2,
        ),
        Commands::AcceptanceTruthGateV16 { config } => print_sprint115_report(
            &config,
            "acceptance-truth-gate-v16",
            "acceptance_v16_warning=research-only paper-only consolidation-governance-only; focused-is-not-full; CLI-smoke-is-not-full; cargo-build-is-not-full; no-run-is-not-full; cargo-progress-is-not-acceptance; timeout-cleanup-is-not-pass; and only full finished and passed workspace tests can claim full acceptance",
            |report| report.acceptance_truth_gate_v16,
        ),
        Commands::ControlTowerConsolidationGovernance { config } => print_sprint115_report(
            &config,
            "control-tower-consolidation-governance",
            "read_only_warning=control tower consolidation governance panel is static/read-only output with no apply button, no run button, and no train/runtime/live/order/account controls",
            |report| report.control_tower_consolidation_governance_panel,
        ),
        Commands::ControlTowerWorkspaceTimeoutTrack { config } => print_sprint115_report(
            &config,
            "control-tower-workspace-timeout-track",
            "read_only_warning=control tower workspace timeout track panel is static/read-only output with no run button and no train/runtime/live/order/account controls",
            |report| report.control_tower_workspace_timeout_track_panel,
        ),
        Commands::Sprint116WorkspaceTimeoutTrack { config } => print_sprint116_report(
            &config,
            "sprint116-workspace-timeout-track",
            "sprint116_warning=research-only paper-only timeout-track-only bundle; consolidation-paused; fifth-patch-not-applied; no-assertion-movement; no-target-retirement; focused-is-not-full; CLI-smoke-is-not-full; cargo-build-is-not-full; no-run-is-not-full; cargo-progress-is-not-acceptance; artifact-ordering-is-not-acceptance; timeout-cleanup-is-not-pass; no assertion deletion; no safety sentinel deletion; no runtime implementation; no training; no live inference; no live trading; no order/account command; no runtime LLM live decision path; no investor impersonation; no auto-activation of 18 live agents; no silent confidence upgrade; no safety test deletion; no hidden skips; local-only paths; and remote paths rejected",
            |report| report,
        ),
        Commands::Sprint115BaselineTruthImport { config } => print_sprint116_report(
            &config,
            "sprint115-baseline-truth-import",
            "sprint115_truth_import_warning=sprint115 truth import stays supporting-only and imported_as_full_acceptance=false",
            |report| report.sprint115_baseline_truth_import_report,
        ),
        Commands::ConsolidationPausedCarryForward { config } => print_sprint116_report(
            &config,
            "consolidation-paused-carry-forward",
            "consolidation_paused_warning=research-only paper-only timeout-track-only; consolidation-paused; fifth-patch-not-applied; no-assertion-movement; no-target-retirement; and local-only paths",
            |report| report.consolidation_paused_carry_forward_report,
        ),
        Commands::WorkspaceTimeoutTrackActivationV1 { config } => print_sprint116_report(
            &config,
            "workspace-timeout-track-activation-v1",
            "workspace_timeout_activation_warning=research-only paper-only timeout-track-only; consolidation-paused; focused-is-not-full; and local-only paths",
            |report| report.workspace_timeout_track_activation_report_v1,
        ),
        Commands::WorkspaceTimeoutObservationBacklogImportV1 { config } => print_sprint116_report(
            &config,
            "workspace-timeout-observation-backlog-import-v1",
            "workspace_timeout_backlog_warning=research-only paper-only timeout-track-only; backlog import is diagnostic-only; and local-only paths",
            |report| report.workspace_timeout_observation_backlog_import_report_v1,
        ),
        Commands::WorkspaceTimeoutObservationBacklogBurndownV1 { config } => print_sprint116_report(
            &config,
            "workspace-timeout-observation-backlog-burndown-v1",
            "workspace_timeout_burndown_warning=research-only paper-only timeout-track-only; backlog reduction is not acceptance; and local-only paths",
            |report| report.workspace_timeout_observation_backlog_burn_down_report_v1,
        ),
        Commands::RealNoRunObservationAttemptV17 { config } => print_sprint116_report(
            &config,
            "real-no-run-observation-attempt-v17",
            "real_no_run_warning=research-only paper-only timeout-track-only; no-run-is-not-full; timeout-cleanup-is-not-pass; and local-only paths",
            |report| report.real_no_run_observation_attempt_v17,
        ),
        Commands::RealFullWorkspaceObservationAttemptV17 { config } => print_sprint116_report(
            &config,
            "real-full-workspace-observation-attempt-v17",
            "real_full_warning=research-only paper-only timeout-track-only; focused-is-not-full; CLI-smoke-is-not-full; cargo-build-is-not-full; and only full workspace pass can claim full acceptance",
            |report| report.real_full_workspace_observation_attempt_v17,
        ),
        Commands::RealCargoJsonObservationAttemptV17 { config } => print_sprint116_report(
            &config,
            "real-cargo-json-observation-attempt-v17",
            "real_cargo_json_warning=research-only paper-only timeout-track-only; cargo-progress-is-not-acceptance; artifact-ordering-is-not-acceptance; and local-only paths",
            |report| report.real_cargo_json_observation_attempt_v17,
        ),
        Commands::TimeoutCleanupConsistencyV1 { config } => print_sprint116_report(
            &config,
            "timeout-cleanup-consistency-v1",
            "timeout_cleanup_warning=research-only paper-only timeout-track-only; timeout-cleanup-is-not-pass; actual counts only; and local-only paths",
            |report| report.timeout_cleanup_consistency_report_v1,
        ),
        Commands::CargoJsonParseQualityV1 { config } => print_sprint116_report(
            &config,
            "cargo-json-parse-quality-v1",
            "cargo_json_parse_warning=research-only paper-only timeout-track-only; cargo-progress-is-not-acceptance; actual parsing only; and local-only paths",
            |report| report.cargo_json_parse_quality_report_v1,
        ),
        Commands::WorkspaceTimeoutEvidenceMatrixV2 { config } => print_sprint116_report(
            &config,
            "workspace-timeout-evidence-matrix-v2",
            "workspace_timeout_matrix_warning=research-only paper-only timeout-track-only; supporting-only evidence remains distinct from acceptance; and local-only paths",
            |report| report.workspace_timeout_evidence_matrix_v2,
        ),
        Commands::WorkspaceTimeoutRootCauseV4 { config } => print_sprint116_report(
            &config,
            "workspace-timeout-root-cause-v4",
            "workspace_timeout_root_cause_warning=research-only paper-only timeout-track-only; conservative evidence-backed root cause only; and local-only paths",
            |report| report.workspace_timeout_root_cause_report_v4,
        ),
        Commands::AcceptanceTruthGateV17 { config } => print_sprint116_report(
            &config,
            "acceptance-truth-gate-v17",
            "acceptance_v17_warning=research-only paper-only timeout-track-only; focused-is-not-full; CLI-smoke-is-not-full; cargo-build-is-not-full; no-run-is-not-full; cargo-progress-is-not-acceptance; artifact-ordering-is-not-acceptance; timeout-cleanup-is-not-pass; and only full finished and passed workspace tests can claim full acceptance",
            |report| report.acceptance_truth_gate_v17,
        ),
        Commands::WorkspaceNoRunRecoveryGateV17 { config } => print_sprint116_report(
            &config,
            "workspace-no-run-recovery-gate-v17",
            "workspace_no_run_recovery_warning=research-only paper-only timeout-track-only; no-run-is-not-full; no-run completion does not imply full acceptance; and local-only paths",
            |report| report.workspace_no_run_recovery_gate_v17,
        ),
        Commands::WorkspaceFullAcceptanceGateV17 { config } => print_sprint116_report(
            &config,
            "workspace-full-acceptance-gate-v17",
            "workspace_full_acceptance_warning=research-only paper-only timeout-track-only; only full finished and passed workspace tests can claim full acceptance; and local-only paths",
            |report| report.workspace_full_acceptance_gate_v17,
        ),
        Commands::ControlTowerWorkspaceTimeoutTrackExecution { config } => print_sprint116_report(
            &config,
            "control-tower-workspace-timeout-track-execution",
            "read_only_warning=control tower workspace timeout track execution panel is static/read-only output with no run button, no action button, and no train/runtime/live/order/account controls",
            |report| report.control_tower_workspace_timeout_track_execution_panel,
        ),
        Commands::ControlTowerAcceptanceTruthV17 { config } => print_sprint116_report(
            &config,
            "control-tower-acceptance-truth-v17",
            "read_only_warning=control tower acceptance truth panel is static/read-only output with no action button and no train/runtime/live/order/account controls",
            |report| report.control_tower_acceptance_truth_panel_v17,
        ),
        Commands::Sprint117DeferredRealObservation { config } => print_sprint117_report(
            &config,
            "sprint117-deferred-real-observation",
            "sprint117_warning=research-only paper-only deferred-real-observation-only bundle; actual-observation-not-fixture; consolidation-paused; fifth-patch-not-applied; no-assertion-movement; no-target-retirement; focused-is-not-full; CLI-smoke-is-not-full; cargo-build-is-not-full; no-run-is-not-full; cargo-json-is-not-acceptance; timeout-cleanup-is-not-pass; no assertion deletion; no safety sentinel deletion; no runtime implementation; no training; no live inference; no live trading; no order/account command; no runtime LLM live decision path; no investor impersonation; no auto-activation of 18 live agents; no silent confidence upgrade; no safety test deletion; no hidden skips; local-only paths; and remote paths rejected",
            |report| report,
        ),
        Commands::Sprint116BaselineTruthImport { config } => print_sprint117_report(
            &config,
            "sprint116-baseline-truth-import",
            "sprint116_truth_import_warning=sprint116 truth import stays supporting-only and imported_as_full_acceptance=false",
            |report| report.sprint116_baseline_truth_import_report,
        ),
        Commands::DeferredObservationSelectionV1 { config } => print_sprint117_report(
            &config,
            "deferred-observation-selection-v1",
            "deferred_observation_selection_warning=research-only paper-only deferred-real-observation-only; actual-observation-not-fixture; and local-only paths",
            |report| report.deferred_observation_selection_report_v1,
        ),
        Commands::DeferredObservationExecutionPlanV1 { config } => print_sprint117_report(
            &config,
            "deferred-observation-execution-plan-v1",
            "deferred_observation_plan_warning=research-only paper-only deferred-real-observation-only; deterministic order RealCargoJson, RealNoRun, RealFullWorkspace; and local-only paths",
            |report| report.deferred_observation_execution_plan_v1,
        ),
        Commands::RealNoRunExecutionV18 { config } => print_sprint117_report(
            &config,
            "real-no-run-execution-v18",
            "real_no_run_warning=research-only paper-only deferred-real-observation-only; no-run-is-not-full; timeout-cleanup-is-not-pass; and local-only paths",
            |report| report.real_no_run_execution_report_v18,
        ),
        Commands::RealFullWorkspaceExecutionV18 { config } => print_sprint117_report(
            &config,
            "real-full-workspace-execution-v18",
            "real_full_warning=research-only paper-only deferred-real-observation-only; focused-is-not-full; CLI-smoke-is-not-full; cargo-build-is-not-full; and only full workspace pass can claim full acceptance",
            |report| report.real_full_workspace_execution_report_v18,
        ),
        Commands::RealCargoJsonExecutionV18 { config } => print_sprint117_report(
            &config,
            "real-cargo-json-execution-v18",
            "real_cargo_json_warning=research-only paper-only deferred-real-observation-only; cargo-json-is-not-acceptance; actual-observation-not-fixture; and local-only paths",
            |report| report.real_cargo_json_execution_report_v18,
        ),
        Commands::CargoJsonActualParseV2 { config } => print_sprint117_report(
            &config,
            "cargo-json-actual-parse-v2",
            "cargo_json_parse_warning=research-only paper-only deferred-real-observation-only; cargo-json-is-not-acceptance; actual parsing only; and local-only paths",
            |report| report.cargo_json_actual_parse_report_v2,
        ),
        Commands::ObservationFixtureSeparationV1 { config } => print_sprint117_report(
            &config,
            "observation-fixture-separation-v1",
            "observation_fixture_separation_warning=research-only paper-only deferred-real-observation-only; fixture must not overwrite actual observation; and local-only paths",
            |report| report.observation_fixture_separation_report_v1,
        ),
        Commands::ActualVsCarriedForwardEvidenceV1 { config } => print_sprint117_report(
            &config,
            "actual-vs-carried-forward-evidence-v1",
            "actual_vs_carried_forward_warning=research-only paper-only deferred-real-observation-only; actual-observation-not-fixture; and local-only paths",
            |report| report.actual_vs_carried_forward_evidence_report_v1,
        ),
        Commands::ObservationBacklogCompletionV2 { config } => print_sprint117_report(
            &config,
            "observation-backlog-completion-v2",
            "observation_backlog_warning=research-only paper-only deferred-real-observation-only; backlog completion is not full acceptance; and local-only paths",
            |report| report.observation_backlog_completion_report_v2,
        ),
        Commands::WorkspaceTimeoutEvidenceMatrixV3 { config } => print_sprint117_report(
            &config,
            "workspace-timeout-evidence-matrix-v3",
            "workspace_timeout_matrix_warning=research-only paper-only deferred-real-observation-only; supporting-only evidence remains distinct from acceptance; and local-only paths",
            |report| report.workspace_timeout_evidence_matrix_v3,
        ),
        Commands::WorkspaceNoRunRecoveryGateV18 { config } => print_sprint117_report(
            &config,
            "workspace-no-run-recovery-gate-v18",
            "workspace_no_run_recovery_warning=research-only paper-only deferred-real-observation-only; no-run-is-not-full; no-run completion does not imply full acceptance; and local-only paths",
            |report| report.workspace_no_run_recovery_gate_v18,
        ),
        Commands::WorkspaceFullAcceptanceGateV18 { config } => print_sprint117_report(
            &config,
            "workspace-full-acceptance-gate-v18",
            "workspace_full_acceptance_warning=research-only paper-only deferred-real-observation-only; only full finished and passed workspace tests can claim full acceptance; and local-only paths",
            |report| report.workspace_full_acceptance_gate_v18,
        ),
        Commands::AcceptanceTruthGateV18 { config } => print_sprint117_report(
            &config,
            "acceptance-truth-gate-v18",
            "acceptance_v18_warning=research-only paper-only deferred-real-observation-only; focused-is-not-full; CLI-smoke-is-not-full; cargo-build-is-not-full; no-run-is-not-full; cargo-json-is-not-acceptance; timeout-cleanup-is-not-pass; and only full finished and passed workspace tests can claim full acceptance",
            |report| report.acceptance_truth_gate_v18,
        ),
        Commands::ControlTowerDeferredObservationExecution { config } => print_sprint117_report(
            &config,
            "control-tower-deferred-observation-execution",
            "read_only_warning=control tower deferred observation execution panel is static/read-only output with no run button, no action button, and no train/runtime/live/order/account controls",
            |report| report.control_tower_deferred_observation_execution_panel,
        ),
        Commands::ControlTowerAcceptanceTruthV18 { config } => print_sprint117_report(
            &config,
            "control-tower-acceptance-truth-v18",
            "read_only_warning=control tower acceptance truth panel is static/read-only output with no action button and no train/runtime/live/order/account controls",
            |report| report.control_tower_acceptance_truth_panel_v18,
        ),
        Commands::Sprint118TimeoutReductionQueue { config } => print_sprint118_report(
            &config,
            "sprint118-timeout-reduction-queue",
            "sprint118_warning=research-only paper-only timeout-reduction-only bundle; consolidation-paused; fifth-patch-not-applied; no-assertion-movement; no-target-retirement; no-run-is-not-full; cargo-json-is-not-acceptance; stderr-is-not-acceptance; timeout-cleanup-is-not-pass; focused-is-not-full; CLI-smoke-is-not-full; cargo-build-is-not-full; no assertion deletion; no safety sentinel deletion; no runtime implementation; no training; no live inference; no live trading; no order/account command; no runtime LLM live decision path; no investor impersonation; no auto-activation of 18 live agents; no silent confidence upgrade; no safety test deletion; no hidden skips; local-only paths; and remote paths rejected",
            |report| report,
        ),
        Commands::Sprint117BaselineTruthImport { config } => print_sprint118_report(
            &config,
            "sprint117-baseline-truth-import",
            "sprint117_truth_import_warning=sprint117 truth import stays supporting-only and imported_as_full_acceptance=false",
            |report| report.sprint117_baseline_truth_import_report,
        ),
        Commands::CargoJsonFailureReasonAnalysisV1 { config } => print_sprint118_report(
            &config,
            "cargo-json-failure-reason-analysis-v1",
            "cargo_json_failure_reason_warning=research-only paper-only timeout-reduction-only; cargo-json-is-not-acceptance; stderr-is-not-acceptance; and local-only paths",
            |report| report.cargo_json_failure_reason_analysis_report_v1,
        ),
        Commands::CargoJsonReasonLineClassificationV1 { config } => print_sprint118_report(
            &config,
            "cargo-json-reason-line-classification-v1",
            "cargo_json_reason_line_warning=research-only paper-only timeout-reduction-only; cargo-json-is-not-acceptance; and local-only paths",
            |report| report.cargo_json_reason_line_classification_report_v1,
        ),
        Commands::CargoJsonTargetBlockerExtractionV1 { config } => print_sprint118_report(
            &config,
            "cargo-json-target-blocker-extraction-v1",
            "cargo_json_blocker_warning=research-only paper-only timeout-reduction-only; blocker extraction narrows follow-up only; and local-only paths",
            |report| report.cargo_json_target_blocker_extraction_report_v1,
        ),
        Commands::WorkspaceTimeoutReductionHypothesisV1 { config } => print_sprint118_report(
            &config,
            "workspace-timeout-reduction-hypothesis-v1",
            "timeout_hypothesis_warning=research-only paper-only timeout-reduction-only; queue-ready does not mean timeout solved; and local-only paths",
            |report| report.workspace_timeout_reduction_hypothesis_report_v1,
        ),
        Commands::WorkspaceTimeoutReductionQueueV1 { config } => print_sprint118_report(
            &config,
            "workspace-timeout-reduction-queue-v1",
            "timeout_queue_warning=research-only paper-only timeout-reduction-only; queue-ready does not mean timeout solved; and local-only paths",
            |report| report.workspace_timeout_reduction_queue_v1,
        ),
        Commands::TruthfulNoRunAttemptV19 { config } => print_sprint118_report(
            &config,
            "truthful-no-run-attempt-v19",
            "truthful_no_run_warning=research-only paper-only timeout-reduction-only; no-run-is-not-full; timeout-cleanup-is-not-pass; and local-only paths",
            |report| report.truthful_no_run_attempt_v19,
        ),
        Commands::TruthfulFullWorkspaceAttemptV19 { config } => print_sprint118_report(
            &config,
            "truthful-full-workspace-attempt-v19",
            "truthful_full_warning=research-only paper-only timeout-reduction-only; focused-is-not-full; CLI-smoke-is-not-full; cargo-build-is-not-full; and only full finished and passed workspace tests can claim full acceptance",
            |report| report.truthful_full_workspace_attempt_v19,
        ),
        Commands::TruthfulCargoJsonAttemptV19 { config } => print_sprint118_report(
            &config,
            "truthful-cargo-json-attempt-v19",
            "truthful_cargo_json_warning=research-only paper-only timeout-reduction-only; cargo-json-is-not-acceptance; stderr-is-not-acceptance; and local-only paths",
            |report| report.truthful_cargo_json_attempt_v19,
        ),
        Commands::WorkspaceTimeoutEvidenceMatrixV4 { config } => print_sprint118_report(
            &config,
            "workspace-timeout-evidence-matrix-v4",
            "workspace_timeout_matrix_warning=research-only paper-only timeout-reduction-only; supporting-only evidence remains distinct from acceptance; and local-only paths",
            |report| report.workspace_timeout_evidence_matrix_v4,
        ),
        Commands::WorkspaceTimeoutRootCauseV6 { config } => print_sprint118_report(
            &config,
            "workspace-timeout-root-cause-v6",
            "workspace_timeout_root_cause_warning=research-only paper-only timeout-reduction-only; narrowed blocker evidence is still not acceptance; and local-only paths",
            |report| report.workspace_timeout_root_cause_report_v6,
        ),
        Commands::WorkspaceNoRunRecoveryGateV19 { config } => print_sprint118_report(
            &config,
            "workspace-no-run-recovery-gate-v19",
            "workspace_no_run_recovery_warning=research-only paper-only timeout-reduction-only; no-run-is-not-full; no-run completion does not imply full acceptance; and local-only paths",
            |report| report.workspace_no_run_recovery_gate_v19,
        ),
        Commands::WorkspaceFullAcceptanceGateV19 { config } => print_sprint118_report(
            &config,
            "workspace-full-acceptance-gate-v19",
            "workspace_full_acceptance_warning=research-only paper-only timeout-reduction-only; only full finished and passed workspace tests can claim full acceptance; and local-only paths",
            |report| report.workspace_full_acceptance_gate_v19,
        ),
        Commands::AcceptanceTruthGateV19 { config } => print_sprint118_report(
            &config,
            "acceptance-truth-gate-v19",
            "acceptance_v19_warning=research-only paper-only timeout-reduction-only; focused-is-not-full; CLI-smoke-is-not-full; cargo-build-is-not-full; no-run-is-not-full; cargo-json-is-not-acceptance; stderr-is-not-acceptance; timeout-cleanup-is-not-pass; and only full finished and passed workspace tests can claim full acceptance",
            |report| report.acceptance_truth_gate_v19,
        ),
        Commands::ControlTowerTimeoutReductionQueue { config } => print_sprint118_report(
            &config,
            "control-tower-timeout-reduction-queue",
            "read_only_warning=control tower timeout reduction queue panel is static/read-only output with no run button, no apply button, and no train/runtime/live/order/account controls",
            |report| report.control_tower_timeout_reduction_queue_panel,
        ),
        Commands::ControlTowerAcceptanceTruthV19 { config } => print_sprint118_report(
            &config,
            "control-tower-acceptance-truth-v19",
            "read_only_warning=control tower acceptance truth panel is static/read-only output with no action button and no train/runtime/live/order/account controls",
            |report| report.control_tower_acceptance_truth_panel_v19,
        ),
        Commands::MinimalAiCommitteeCycle { config } => run_minimal_ai_committee_cycle(
            &config,
            "minimal-ai-committee-cycle",
        )
        .and_then(|report| {
            print_json_report(
                "minimal_ai_committee_warning=paper-only local mock/offline fixture member logic; program orchestrates and AI members analyze; no broker/order/account, no model training, no live inference, no real trading",
                &report,
            )
        }),
        Commands::ProposalWarningClosure { config } => {
            run_sprint100_bundle(&config, "proposal-warning-closure").and_then(|report| {
                print_json_report(
                    "proposal_warning=proposal warning closure is research-only paper review; proposal remains paper-only and not order execution",
                    &report.proposal_quality_warning_closure_report,
                )
            })
        }
        Commands::ProposalEvidenceCompleteness { config } => {
            run_sprint100_bundle(&config, "proposal-evidence-completeness").and_then(|report| {
                print_json_report(
                    "proposal_evidence_warning=proposal evidence completeness is local-only and never a live data entitlement",
                    &report.proposal_evidence_completeness_report,
                )
            })
        }
        Commands::ProposalRiskFieldCompleteness { config } => {
            run_sprint100_bundle(&config, "proposal-risk-field-completeness").and_then(|report| {
                print_json_report(
                    "proposal_risk_warning=proposal risk field completeness stays paper-only with Risk Governor still required",
                    &report.proposal_risk_field_completeness_report,
                )
            })
        }
        Commands::EntryTimingConditionCompleteness { config } => {
            run_sprint100_bundle(&config, "entry-timing-condition-completeness").and_then(|report| {
                print_json_report(
                    "entry_timing_warning=entry timing completeness stays paper-only and never becomes a broker/order/account command",
                    &report.entry_timing_condition_completeness_report,
                )
            })
        }
        Commands::DebateEvidenceClosure { config } => {
            run_sprint100_bundle(&config, "debate-evidence-closure").and_then(|report| {
                print_json_report(
                    "debate_closure_warning=debate evidence closure is research-only, local-only, and does not enable runtime LLM live debate",
                    &report.debate_needs_more_evidence_closure_report,
                )
            })
        }
        Commands::DebateEvidenceGapPlan { config } => {
            run_sprint100_bundle(&config, "debate-evidence-gap-plan").and_then(|report| {
                print_json_report(
                    "debate_gap_warning=evidence gap planning is local-only and keeps source-boundary and no-lookahead guards explicit",
                    &report.debate_evidence_gap_plan,
                )
            })
        }
        Commands::DebateDissentCoverage { config } => {
            run_sprint100_bundle(&config, "debate-dissent-coverage").and_then(|report| {
                print_json_report(
                    "debate_dissent_warning=dissent coverage is paper-only committee review and not live agent debate",
                    &report.debate_dissent_coverage_report,
                )
            })
        }
        Commands::DebateParticipationBalance { config } => {
            run_sprint100_bundle(&config, "debate-participation-balance").and_then(|report| {
                print_json_report(
                    "debate_participation_warning=participation balance is research-only with no auto-promotion-to-live",
                    &report.debate_member_participation_balance_report,
                )
            })
        }
        Commands::ChairmanUnsafeRuleClosure { config } => {
            run_sprint100_bundle(&config, "chairman-unsafe-rule-closure").and_then(|report| {
                print_json_report(
                    "chairman_rule_warning=unsafe chairman rule closure is paper-only and allows no live rule mutation",
                    &report.chairman_unsafe_rule_closure_report,
                )
            })
        }
        Commands::ChairmanRulebookRepairPlan { config } => {
            run_sprint100_bundle(&config, "chairman-rulebook-repair-plan").and_then(|report| {
                print_json_report(
                    "chairman_repair_warning=rulebook repair planning is research-only, no central AI core, and no auto-rule-apply",
                    &report.chairman_rulebook_safety_repair_plan,
                )
            })
        }
        Commands::ChairmanRulebookV2Draft { config } => {
            run_sprint100_bundle(&config, "chairman-rulebook-v2-draft").and_then(|report| {
                print_json_report(
                    "chairman_v2_warning=rulebook v2 draft is paper-only governance with live use forbidden",
                    &report.chairman_rulebook_v2_draft,
                )
            })
        }
        Commands::ChairmanRulebookApprovalGate { config } => {
            run_sprint100_bundle(&config, "chairman-rulebook-approval-gate").and_then(|report| {
                print_json_report(
                    "chairman_approval_warning=rulebook approval gate allows paper-only review only and blocks live activation",
                    &report.chairman_rulebook_approval_gate,
                )
            })
        }
        Commands::RuleAuditTrailCompleteness { config } => {
            run_sprint100_bundle(&config, "rule-audit-trail-completeness").and_then(|report| {
                print_json_report(
                    "rule_audit_warning=rule audit trail completeness is audit-only and never live mutation",
                    &report.chairman_rule_audit_trail_completeness_report,
                )
            })
        }
        Commands::RulebookDiffRiskClosure { config } => {
            run_sprint100_bundle(&config, "rulebook-diff-risk-closure").and_then(|report| {
                print_json_report(
                    "rulebook_diff_warning=rulebook diff risk closure is paper-only and never runtime implementation",
                    &report.rulebook_diff_risk_closure_report,
                )
            })
        }
        Commands::ScorecardWarningClosure { config } => {
            run_sprint100_bundle(&config, "scorecard-warning-closure").and_then(|report| {
                print_json_report(
                    "scorecard_warning=scorecard warning closure is roster research only and not capital allocation",
                    &report.scorecard_calibration_warning_closure_report,
                )
            })
        }
        Commands::ScorecardEvidenceDepth { config } => {
            run_sprint100_bundle(&config, "scorecard-evidence-depth").and_then(|report| {
                print_json_report(
                    "scorecard_depth_warning=scorecard evidence depth is paper-only and adds no training path",
                    &report.scorecard_evidence_depth_report,
                )
            })
        }
        Commands::PromotionDemotionStability { config } => {
            run_sprint100_bundle(&config, "promotion-demotion-stability").and_then(|report| {
                print_json_report(
                    "promotion_warning=promotion/demotion stability is research roster management only and not capital allocation",
                    &report.promotion_demotion_stability_report,
                )
            })
        }
        Commands::OverfitWarningClosure { config } => {
            run_sprint100_bundle(&config, "overfit-warning-closure").and_then(|report| {
                print_json_report(
                    "overfit_warning=overfit warning closure is research-only with no training or live adaptation",
                    &report.overfit_warning_closure_report,
                )
            })
        }
        Commands::RosterBalanceWarningClosure { config } => {
            run_sprint100_bundle(&config, "roster-balance-warning-closure").and_then(|report| {
                print_json_report(
                    "roster_warning=roster balance closure preserves public-philosophy archetypes only and no investor impersonation",
                    &report.roster_balance_warning_closure_report,
                )
            })
        }
        Commands::PaperReplayWarningClosure { config } => {
            run_sprint100_bundle(&config, "paper-replay-warning-closure").and_then(|report| {
                print_json_report(
                    "paper_replay_warning=paper replay warning closure never touches broker/order/account commands",
                    &report.paper_decision_replay_warning_closure_report,
                )
            })
        }
        Commands::PaperNeedMoreEvidenceClosure { config } => {
            run_sprint100_bundle(&config, "paper-need-more-evidence-closure").and_then(|report| {
                print_json_report(
                    "paper_need_more_evidence_warning=NeedMoreEvidence closure is paper-only and never a live execution path",
                    &report.paper_decision_need_more_evidence_closure_report,
                )
            })
        }
        Commands::RiskHandoffWarningClosure { config } => {
            run_sprint100_bundle(&config, "risk-handoff-warning-closure").and_then(|report| {
                print_json_report(
                    "risk_handoff_warning=Risk Governor handoff closure preserves final veto and no bypass path",
                    &report.risk_governor_handoff_warning_closure_report,
                )
            })
        }
        Commands::RiskFinalVetoTrace { config } => {
            run_sprint100_bundle(&config, "risk-final-veto-trace").and_then(|report| {
                print_json_report(
                    "risk_trace_warning=Risk Governor final veto trace is audit-only and keeps runtime live decision paths forbidden",
                    &report.risk_governor_final_veto_trace_report,
                )
            })
        }
        Commands::CommitteePaperReadinessGate { config } => {
            run_sprint100_bundle(&config, "committee-paper-readiness-gate").and_then(|report| {
                print_json_report(
                    "paper_gate_warning=committee paper readiness is paper-loop only and does not imply broker execution or live trading",
                    &report.committee_paper_readiness_gate,
                )
            })
        }
        Commands::CommitteePaperLoopDryRunPlan { config } => {
            run_sprint100_bundle(&config, "committee-paper-loop-dry-run-plan").and_then(|report| {
                print_json_report(
                    "paper_dry_run_warning=paper loop dry-run planning is research-only and keeps live loop forbidden",
                    &report.committee_paper_loop_dry_run_plan,
                )
            })
        }
        Commands::WorkspaceTruthClosurePlanV2 { config } => {
            run_sprint100_bundle(&config, "workspace-truth-closure-plan-v2").and_then(|report| {
                print_json_report(
                    "workspace_truth_warning=full workspace acceptance remains separate; focused tests never replace full workspace acceptance",
                    &report.workspace_acceptance_truth_closure_plan_v2,
                )
            })
        }
        Commands::WorkspaceAcceptanceAttemptV17 { config } => {
            run_sprint100_bundle(&config, "workspace-acceptance-attempt-v17").and_then(|report| {
                print_json_report(
                    "workspace_attempt_warning=workspace acceptance attempt v17 is an honest record only; full workspace requires real cargo test --workspace --quiet completion",
                    &report.workspace_acceptance_attempt_v17,
                )
            })
        }
        Commands::SafetyCoveragePreservationV16 { config } => {
            run_sprint100_bundle(&config, "safety-coverage-preservation-v16").and_then(|report| {
                print_json_report(
                    "safety_warning=safety coverage v16 preserves no runtime LLM path, no training, no broker/account, no live trading, and no safety test deletion",
                    &report.safety_coverage_preservation_report_v16,
                )
            })
        }
        Commands::ControlTowerAiCommitteeClosure { config } => {
            run_sprint100_bundle(&config, "control-tower-ai-committee-closure").and_then(|report| {
                print_json_report(
                    "read_only_warning=control tower AI committee closure panel is static read-only output with no train/runtime/live/order/account/browser controls or auto-rule-apply button",
                    &report.control_tower_ai_committee_closure_panel,
                )
            })
        }
        Commands::SystemBenchmarkDiff { config } => {
            if config.contains("://") {
                    Err("system-benchmark-diff config path must be local".to_string())
            } else {
                DeterministicArtifactDiffConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = run_deterministic_artifact_diff(&config)?;
                        println!(
                            "deterministic_warning=system benchmark diff compares local deterministic artifacts only"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::ManualShipChecklist { config } => {
            if config.contains("://") {
                Err("manual-ship-checklist config path must be local".to_string())
            } else {
                SystemIntegrationReviewConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let bundle = SystemIntegrationReviewRunner::default().run(&config)?;
                        println!(
                            "paper_only_warning=manual ship checklist is a paper-ops manual gate only"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&bundle.manual_ship_acceptance_checklist)
                                .map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::SystemShipGate { config } => {
            if config.contains("://") {
                Err("system-ship-gate config path must be local".to_string())
            } else {
                SystemIntegrationReviewConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let bundle = SystemIntegrationReviewRunner::default().run(&config)?;
                        println!(
                            "no_live_warning=system ship gate is paper-ops-monitoring only and never a live-trading approval"
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&bundle.system_ship_gate_report)
                                .map_err(|err| err.to_string())?
                        );
                        Ok(())
                    })
            }
        }
        Commands::CandidateGenerate { config } => {
            if config.contains("://") {
                Err("candidate-generate config path must be local".to_string())
            } else {
                TrinityCommitteeOperationalLoopConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = run_candidate_generation_only(&config)?;
                        println!("{}", report.to_text());
                        Ok(())
                    })
            }
        }
        Commands::CommitteeCycle { config } => {
            if config.contains("://") {
                Err("committee-cycle config path must be local".to_string())
            } else {
                CommitteeCycleConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let record = run_committee_cycle_from_config(&config)?;
                        println!("{}", record.to_text());
                        Ok(())
                    },
                )
            }
        }
        Commands::TrinityOperationalLoop { config } => {
            if config.contains("://") {
                Err("trinity-operational-loop config path must be local".to_string())
            } else {
                TrinityCommitteeOperationalLoopConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let bundle = TrinityOperationalLoopRunner::default().run(&config)?;
                        println!("{}", bundle.report.to_text());
                        Ok(())
                    })
            }
        }
        Commands::PaperLifecycleReport { config } => {
            if config.contains("://") {
                Err("paper-lifecycle-report config path must be local".to_string())
            } else {
                TrinityCommitteeOperationalLoopConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let bundle = TrinityOperationalLoopRunner::default().run(&config)?;
                        println!("{}", bundle.paper_position_lifecycle_report.to_text());
                        Ok(())
                    })
            }
        }
        Commands::OperationalAuditTimeline { config } => {
            if config.contains("://") {
                Err("operational-audit-timeline config path must be local".to_string())
            } else {
                TrinityCommitteeOperationalLoopConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let bundle = TrinityOperationalLoopRunner::default().run(&config)?;
                        println!("{}", bundle.operational_audit_timeline.to_text());
                        Ok(())
                    })
            }
        }
        Commands::DashboardActionDrafts { config } => {
            if config.contains("://") {
                Err("dashboard-action-drafts config path must be local".to_string())
            } else {
                let config_path = std::path::Path::new(&config);
                ControlTowerV1Config::from_toml_path(config_path)
                    .map_err(|err| err.to_string())
                    .and_then(|config| {
                        let state = ControlTowerV1Builder::default().build(&config, Some(config_path))?;
                        let bundle = generate_owner_action_draft_bundle(&state, &config)?;
                        println!("{}", bundle.to_text());
                        Ok(())
                    })
            }
        }
        Commands::DashboardOpen { config } => {
            if config.contains("://") {
                Err("dashboard-open config path must be local".to_string())
            } else {
                let config_path = std::path::Path::new(&config);
                ControlTowerV1Config::from_toml_path(config_path)
                    .map_err(|err| err.to_string())
                    .and_then(|config| {
                        let state = ControlTowerV1Builder::default().build(&config, Some(config_path))?;
                        let report = DashboardV1Renderer::default().render(&state, &config)?;
                        let html_path = report
                            .html_path
                            .clone()
                            .ok_or_else(|| "dashboard-open requires html rendering to be enabled".to_string())?;
                        let open_report = prepare_dashboard_open(&config.artifact_dir(), std::path::Path::new(&html_path), false)?;
                        println!("{}", open_report.to_text());
                        Ok(())
                    })
            }
        }
        Commands::DashboardServe { config } => {
            if config.contains("://") {
                Err("dashboard-serve config path must be local".to_string())
            } else {
                ControlTowerV1Config::from_toml_path(std::path::Path::new(&config))
                    .map_err(|err| err.to_string())
                    .and_then(|config| {
                        config.validate()?;
                        let report = DashboardServeReport::deferred();
                        println!("{}", report.to_text());
                        Ok(())
                    })
            }
        }
        Commands::OwnerInputValidate { config } => {
            if config.contains("://") {
                Err("owner-input-validate config path must be local".to_string())
            } else {
                OwnerInputValidateConfig::from_toml_path(std::path::Path::new(&config))
                    .map_err(|err| err.to_string())
                    .and_then(|config| {
                        if !config.validate_local_paths().is_empty() {
                            return Err(
                                "owner-input-validate config path must be local".to_string()
                            );
                        }
                        let report = run_owner_input_validation(&config);
                        println!("{}", report.to_text());
                        Ok(())
                    })
            }
        }
        Commands::OwnerReviewQueue { config } => {
            if config.contains("://") {
                Err("owner-review-queue config path must be local".to_string())
            } else {
                OwnerReviewQueueConfig::from_toml_path(std::path::Path::new(&config))
                    .map_err(|err| err.to_string())
                    .and_then(|config| {
                        let queue = run_owner_review_queue(&config)?;
                        println!("{}", queue.to_text());
                        Ok(())
                    })
            }
        }
        Commands::OwnerApplyInput { config } => {
            if config.contains("://") {
                Err("owner-apply-input config path must be local".to_string())
            } else {
                OwnerApplyInputConfig::from_toml_path(std::path::Path::new(&config))
                    .map_err(|err| err.to_string())
                    .and_then(|config| {
                        let report = run_owner_apply_input(&config)?;
                        println!("{}", report.to_text());
                        Ok(())
                    })
            }
        }
        Commands::OwnerImpactReport { config } => {
            if config.contains("://") {
                Err("owner-impact-report config path must be local".to_string())
            } else {
                OwnerImpactReportConfig::from_toml_path(std::path::Path::new(&config))
                    .map_err(|err| err.to_string())
                    .and_then(|config| {
                        let report = run_owner_impact_report(&config)?;
                        println!("{}", report.to_text());
                        Ok(())
                    })
            }
        }
        Commands::OwnerThesisBook { config } => {
            if config.contains("://") {
                Err("owner-thesis-book config path must be local".to_string())
            } else {
                OwnerThesisBookConfig::from_toml_path(std::path::Path::new(&config))
                    .map_err(|err| err.to_string())
                    .and_then(|config| {
                        let book = run_owner_thesis_book(&config)?;
                        println!("{}", book.to_text());
                        Ok(())
                    })
            }
        }
        Commands::StrategyDataCheck { provider, use_case } => Ok(()).and_then(|_| {
            let provider_subject = parse_provider_subject(&provider)?;
            let use_case = parse_strategy_use_case(&use_case)?;
            let result = evaluate_strategy_data_compatibility(provider_subject, use_case, None);
            println!(
                "provider={:?}\nuse_case={:?}\ncompatible={}\nblockers={}\nwarnings={}\nlimitations={}",
                result.provider_subject,
                result.use_case,
                result.compatible,
                result.blockers.join("|"),
                result.warnings.join("|"),
                result.limitations.join("|"),
            );
            Ok(())
        }),
        Commands::ProviderRecommend {
            market,
            use_case,
            budget,
        } => Ok(()).and_then(|_| {
            let market = parse_provider_market(&market)?;
            let use_case = parse_strategy_use_case(&use_case)?;
            let budget_preference = parse_budget_preference(&budget)?;
            let recommendation = recommend_provider(
                &ProviderRecommendationRequest {
                    market,
                    desired_use_case: use_case,
                    budget_preference,
                    need_realtime: matches!(
                        use_case,
                        StrategyUseCase::RealtimeScalping
                            | StrategyUseCase::RealtimeExecutionSimulation
                    ),
                    need_official_readiness: market != ProviderMarket::Crypto,
                    max_data_size_preference: Some("compact".to_string()),
                    reason_codes: vec![soma_zero::ReasonCode::DeterministicPath],
                },
                &soma_zero::ProviderEntitlementPreflightRunner::default()
                    .run(&soma_zero::ProviderEntitlementPreflightConfig::default()),
            );
            println!(
                "status={:?}\nprimary={:?}\nfallbacks={:?}\nresearch_fallbacks={}\nrequired_operator_actions={}",
                recommendation.status,
                recommendation.primary_provider,
                recommendation.fallback_providers,
                recommendation.research_fallbacks.join("|"),
                recommendation.required_operator_actions.join(" | "),
            );
            Ok(())
        }),
        Commands::EvidencePlan { config } => {
            if config.contains("://") {
                Err("evidence-plan config path must be local".to_string())
            } else {
                soma_zero::ExecutableEvidencePlanConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let builder = soma_zero::EvidencePlanBuilder::default();
                        let plan = if let Some(path) = &config.provider_reality_report_path {
                            let report = soma_zero::ProviderRealityReport::from_json_path(
                                std::path::Path::new(path),
                            )?;
                            builder.from_provider_reality(&report, &config)?
                        } else {
                            builder.from_explicit_lanes(&config)?
                        };
                        plan.write_to_dir(&config.output_dir())?;
                        println!("{}", soma_zero::executable_evidence_plan_to_text(&plan));
                        Ok(())
                    })
            }
        }
        Commands::EvidenceExecute { config } => {
            if config.contains("://") {
                Err("evidence-execute config path must be local".to_string())
            } else {
                soma_zero::ExecutableEvidencePlanConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = soma_zero::ProviderRealityEvidenceExecutor::default().run(&config)?;
                        report.write_to_dir(&config.output_dir())?;
                        println!(
                            "{}",
                            soma_zero::provider_reality_evidence_report_to_text(&report)
                        );
                        Ok(())
                    })
            }
        }
        Commands::ReadinessMatrix { config } => {
            if config.contains("://") {
                Err("readiness-matrix config path must be local".to_string())
            } else {
                soma_zero::ExecutableEvidencePlanConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = soma_zero::ProviderRealityEvidenceExecutor::default().run(&config)?;
                        report.write_to_dir(&config.output_dir())?;
                        println!("{}", soma_zero::readiness_matrix_to_text(&report.readiness_matrix));
                        Ok(())
                })
            }
        }
        Commands::CommitteeSmoke { config } => {
            if config.contains("://") {
                Err("committee-smoke config path must be local".to_string())
            } else {
                CommitteeSmokeTestConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = CommitteeSmokeTestRunner.run(&config)?;
                        report.write_to_dir(&config.output_dir())?;
                        println!("{}", report.to_text());
                        Ok(())
                    })
            }
        }
        Commands::CommitteeLoadScenarios { config } => {
            if config.contains("://") {
                Err("committee-load-scenarios config path must be local".to_string())
            } else {
                CommitteeScenarioLoadConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = CommitteeScenarioLoader::default().load(&config)?;
                        report.write_to_dir(&config.output_dir())?;
                        println!("{}", report.to_text());
                        Ok(())
                    })
            }
        }
        Commands::CommitteeReplay { config } => {
            if config.contains("://") {
                Err("committee-replay config path must be local".to_string())
            } else {
                CommitteeReplayConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let report = CommitteeDebateReplay::default().run(&config)?;
                        report.write_to_dir(&config.output_dir())?;
                        println!("{}", report.to_text());
                        Ok(())
                    },
                )
            }
        }
        Commands::CommitteeDiagnostics { config } => {
            if config.contains("://") {
                Err("committee-diagnostics config path must be local".to_string())
            } else {
                CommitteeDiagnosticsConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = CommitteeDiagnosticsRunner::default().run(&config)?;
                        report.write_to_dir(&config.output_dir())?;
                        println!("{}", report.to_text());
                        Ok(())
                    })
            }
        }
        Commands::PersonaCards => Ok(()).map(|_| {
            for card in active_persona_cards_lite() {
                println!(
                    "persona_id={};active={};group={:?};horizon={:?};role={:?};archetype_label={}",
                    card.persona_id,
                    card.active,
                    card.group,
                    card.horizon,
                    card.role,
                    card.archetype_label
                );
            }
            println!("research_only_warning=committee MVP remains paper/research only");
        }),
        Commands::CommitteeV1 { config } => {
            if config.contains("://") {
                Err("committee-v1 config path must be local".to_string())
            } else {
                CommitteeV1RunConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let report = CommitteeV1Runner::default().run(&config)?;
                        report.write_to_dir(&config.output_dir())?;
                        println!("{}", report.to_text());
                        Ok(())
                    },
                )
            }
        }
        Commands::CommitteeMaterialize { config } => {
            if config.contains("://") {
                Err("committee-materialize config path must be local".to_string())
            } else {
                CommitteeMaterializationConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report =
                            CommitteeScenarioMaterializerV2::default().materialize(&config)?;
                        report.write_to_dir(&config.output_dir())?;
                        println!("{}", report.to_text());
                        Ok(())
                    })
            }
        }
        Commands::CommitteeBenchmark { config } => {
            if config.contains("://") {
                Err("committee-benchmark config path must be local".to_string())
            } else {
                CommitteeBenchmarkConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let report = CommitteeBenchmarkRunner::default().run(&config)?;
                        report.write_to_dir(&config.output_dir())?;
                        println!("{}", report.to_text());
                        Ok(())
                    },
                )
            }
        }
        Commands::CommitteePackOfficial { config } => {
            if config.contains("://") {
                Err("committee-pack-official config path must be local".to_string())
            } else {
                OfficialCommitteeScenarioPackConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let pack = OfficialCommitteeScenarioPackBuilder::default().build(&config)?;
                        pack.write_to_dir(&config.output_dir())?;
                        println!("research_only_warning=official committee pack remains research/paper only");
                        println!("{}", pack.to_text());
                        Ok(())
                    })
            }
        }
        Commands::CommitteeLinkOutcomes { config } => {
            if config.contains("://") {
                Err("committee-link-outcomes config path must be local".to_string())
            } else {
                CommitteeOutcomeLinkerConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let linked = CommitteeOutcomeLinker::default().link_from_config(&config)?;
                        linked.write_to_dir(&config.output_dir())?;
                        println!("research_only_warning=committee outcome linking remains research/paper only");
                        println!("{}", linked.to_text());
                        Ok(())
                    })
            }
        }
        Commands::CommitteeOfficialBenchmark { config } => {
            if config.contains("://") {
                Err("committee-official-benchmark config path must be local".to_string())
            } else {
                CommitteeOfficialBenchmarkConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let bundle = CommitteeOfficialBenchmarkRunner::default().run_bundle(&config)?;
                        bundle.write_to_dir(&config.output_dir())?;
                        println!("research_only_warning=official committee benchmark remains research/paper only");
                        println!("{}", bundle.final_summary);
                        Ok(())
                    })
            }
        }
        Commands::CommitteeOutcomeCoverage { config } => {
            if config.contains("://") {
                Err("committee-outcome-coverage config path must be local".to_string())
            } else {
                CommitteeOutcomeCoverageConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let bundle = CommitteeOutcomeCoverageRunner::default().run(&config)?;
                        println!("research_only_warning=committee outcome coverage remains research/paper only");
                        println!(
                            "final_status={:?};final_recommendation={:?}",
                            bundle.final_status, bundle.final_recommendation
                        );
                        Ok(())
                    })
            }
        }
        Commands::CommitteeCounterfactualAudit { config } => {
            if config.contains("://") {
                Err("committee-counterfactual-audit config path must be local".to_string())
            } else {
                CommitteeCounterfactualAuditConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = CommitteeCounterfactualAuditRunner::default().run(&config)?;
                        report.write_to_dir(&config.output_dir())?;
                        println!("research_only_warning=committee counterfactual audit remains research/paper only");
                        println!("{}", report.to_text());
                        Ok(())
                    })
            }
        }
        Commands::CommitteePerformanceMatrix { config } => {
            if config.contains("://") {
                Err("committee-performance-matrix config path must be local".to_string())
            } else {
                CommitteeOutcomeCoverageConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let bundle = CommitteeOutcomeCoverageRunner::default().run(&config)?;
                        println!("research_only_warning=committee performance matrix remains research/paper only");
                        println!("{}", bundle.performance_matrix.to_text());
                        Ok(())
                    })
            }
        }
        Commands::CommitteeBuildReferences { config } => {
            if config.contains("://") {
                Err("committee-build-references config path must be local".to_string())
            } else {
                CommitteeReferencePackConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let bundle = CommitteeReferencePackRunner::default().run(&config)?;
                        println!("research_only_warning=committee reference pack remains research/paper only");
                        println!(
                            "final_status={:?};final_recommendation={:?}",
                            bundle.final_status, bundle.final_recommendation
                        );
                        Ok(())
                    })
            }
        }
        Commands::CommitteeAlignCandles { config } => {
            if config.contains("://") {
                Err("committee-align-candles config path must be local".to_string())
            } else {
                CommitteeReferencePackConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = CommitteeReferencePackRunner::default().align_candles(&config)?;
                        report.write_to_dir(&config.output_dir())?;
                        println!("research_only_warning=committee candle alignment remains research/paper only");
                        println!("{}", report.to_text());
                        Ok(())
                    })
            }
        }
        Commands::CommitteeSufficiencyClose { config } => {
            if config.contains("://") {
                Err("committee-sufficiency-close config path must be local".to_string())
            } else {
                SufficiencyClosureConfig::from_toml_path(std::path::Path::new(&config)).and_then(
                    |config| {
                        let generated_pack = if config.generated_reference_pack_path.ends_with(".toml")
                        {
                            CommitteeReferencePackRunner::default().build_reference_pack(
                                &CommitteeReferencePackConfig::from_toml_path(std::path::Path::new(
                                    &config.generated_reference_pack_path,
                                ))?,
                            )?
                        } else {
                            soma_zero::GeneratedCommitteeReferencePack::from_json_path(
                                std::path::Path::new(&config.generated_reference_pack_path),
                            )?
                        };
                        let report =
                            SufficiencyClosureRunner::default().run_with_pack(&config, &generated_pack)?;
                        std::fs::create_dir_all(config.output_dir())
                            .map_err(|err| err.to_string())?;
                        std::fs::write(
                            config.output_dir().join("sufficiency_closure.txt"),
                            report.to_text(),
                        )
                        .map_err(|err| err.to_string())?;
                        println!("research_only_warning=committee sufficiency closure remains research/paper only");
                        println!("{}", report.to_text());
                        Ok(())
                    },
                )
            }
        }
        Commands::MultiRowOfficialSet { config } => {
            if config.contains("://") {
                Err("multi-row-official-set config path must be local".to_string())
            } else {
                MultiRowOfficialEvidenceSetConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let set = MultiRowOfficialEvidenceSetBuilder::default().build(&config)?;
                        set.write_to_dir(&config.output_dir())?;
                        println!("research_only_warning=multi-row official evidence sets remain research-only, paper-only, and local-only");
                        println!("{}", set.to_text());
                        Ok(())
                    })
            }
        }
        Commands::FutureWindowScaleoutPlan { config } => {
            if config.contains("://") {
                Err("future-window-scaleout-plan config path must be local".to_string())
            } else {
                FutureWindowScaleOutConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let plan = FutureWindowScaleOutPlanner::default().plan(&config)?;
                        plan.write_to_dir(&config.output_dir())?;
                        println!("research_only_warning=future-window scaleout planning remains research-only, bounded, and local-only");
                        println!("{}", plan.to_text());
                        Ok(())
                    })
            }
        }
        Commands::BatchOutcomeLinkageV3 { config } => {
            if config.contains("://") {
                Err("batch-outcome-linkage-v3 config path must be local".to_string())
            } else {
                BatchOutcomeLinkageV3Config::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = BatchOutcomeLinkageV3Runner::default().run(&config)?;
                        report.write_to_dir(&config.output_dir())?;
                        println!("research_only_warning=batch outcome linkage v3 remains research-only, paper-only, and local-only");
                        println!("{}", report.to_text());
                        Ok(())
                    })
            }
        }
        Commands::BatchCounterfactualComplete { config } => {
            if config.contains("://") {
                Err("batch-counterfactual-complete config path must be local".to_string())
            } else {
                BatchCounterfactualCompletionConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = BatchCounterfactualCompletionRunner::default().run(&config)?;
                        report.write_to_dir(&config.output_dir())?;
                        println!("research_only_warning=batch counterfactual completion remains research-only, paper-only, and local-only");
                        println!("{}", report.to_text());
                        Ok(())
                    })
            }
        }
        Commands::OfficialEvidenceSufficiencyV2 { config } => {
            if config.contains("://") {
                Err("official-evidence-sufficiency-v2 config path must be local".to_string())
            } else {
                OfficialEvidenceSufficiencyV2Config::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = OfficialEvidenceSufficiencyV2Runner::default().run(&config)?;
                        report.write_to_dir(&config.output_dir())?;
                        println!("research_only_warning=official evidence sufficiency v2 remains research-only, paper-only, and never implies usefulness or live readiness");
                        println!("{}", report.to_text());
                        Ok(())
                    })
            }
        }
        Commands::OfficialEvidenceScaleout { config } => {
            if config.contains("://") {
                Err("official-evidence-scaleout config path must be local".to_string())
            } else {
                OfficialEvidenceScaleOutConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let bundle = OfficialEvidenceScaleOutRunner::default().run(&config)?;
                        bundle.write_to_dir(&config.output_dir())?;
                        println!("research_only_warning=official evidence scaleout remains research-only, paper-only, local-only, and never implies live trading");
                        println!("{}", bundle.final_summary);
                        Ok(())
                    })
            }
        }
        Commands::BarrierProfiles { config } => {
            if config.contains("://") {
                Err("barrier-profiles config path must be local".to_string())
            } else {
                BarrierProfileRegistryConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let registry = BarrierProfileRegistryBuilder::default().build(&config)?;
                        registry.write_to_dir(&config.output_dir())?;
                        println!("research_only_warning=barrier profiles remain research-only preregistration artifacts, local-only, and never imply live trading");
                        println!("{}", registry.to_text());
                        Ok(())
                    })
            }
        }
        Commands::OfficialDiversityGapMap { config } => {
            if config.contains("://") {
                Err("official-diversity-gap-map config path must be local".to_string())
            } else {
                OfficialEvidenceDiversityGapConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = OfficialEvidenceDiversityGapRunner::default().run(&config)?;
                        report.write_to_dir(&config.output_dir())?;
                        println!("research_only_warning=official diversity gap mapping remains research-only, local-only, and never implies profitability");
                        println!("{}", report.to_text());
                        Ok(())
                    })
            }
        }
        Commands::OfficialDiversityRowSelect { config } => {
            if config.contains("://") {
                Err("official-diversity-row-select config path must be local".to_string())
            } else {
                OfficialDiversityRowSelectorConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = OfficialDiversityRowSelector::default().run(&config)?;
                        report.write_to_dir(&std::path::PathBuf::from("target/soma_official_diversity_row_selector").join(&config.selector_id))?;
                        println!("research_only_warning=official diversity row selection remains research-only, local-only, and must not peek at future outcomes");
                        println!("{}", report.to_text());
                        Ok(())
                    })
            }
        }
        Commands::OutcomeDiversityAudit { config } => {
            if config.contains("://") {
                Err("outcome-diversity-audit config path must be local".to_string())
            } else {
                OutcomeDiversityAuditConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = OutcomeDiversityAuditRunner::default().run(&config)?;
                        report.write_to_dir(&config.output_dir())?;
                        println!("research_only_warning=outcome diversity audit remains research-only, local-only, and never implies profitability");
                        println!("{}", report.to_text());
                        Ok(())
                    })
            }
        }
        Commands::BalancedOutcomeCoverage { config } => {
            if config.contains("://") {
                Err("balanced-outcome-coverage config path must be local".to_string())
            } else {
                BalancedOutcomeCoverageConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = BalancedOutcomeCoverageRunner::default().run(&config)?;
                        report.write_to_dir(&config.output_dir())?;
                        println!("research_only_warning=balanced outcome coverage remains research-only, local-only, and never implies signal quality or profitability");
                        println!("{}", report.to_text());
                        Ok(())
                    })
            }
        }
        Commands::DiversitySufficiencyV2 { config } => {
            if config.contains("://") {
                Err("diversity-sufficiency-v2 config path must be local".to_string())
            } else {
                DiversityAwareSufficiencyV2Config::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = DiversityAwareSufficiencyV2Runner::default().run(&config)?;
                        report.write_to_dir(&config.output_dir())?;
                        println!("research_only_warning=diversity-aware sufficiency remains research-only, local-only, and never implies deployment or profitability");
                        println!("{}", report.to_text());
                        Ok(())
                    })
            }
        }
        Commands::OfficialEvidenceDiversitySweep { config } => {
            if config.contains("://") {
                Err("official-evidence-diversity-sweep config path must be local".to_string())
            } else {
                OfficialEvidenceDiversitySweepConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let bundle = OfficialEvidenceDiversitySweepRunner::default().run(&config)?;
                        println!("research_only_warning=official evidence diversity sweep remains research-only, paper-only, local-only, and never implies live trading");
                        println!("{}", bundle.final_summary);
                        Ok(())
                    })
            }
        }
        Commands::ComparableEvidence { config } => {
            if config.contains("://") {
                Err("comparable-evidence config path must be local".to_string())
            } else {
                ComparableCommitteeEvidenceConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let bundle = ComparableEvidenceBuilder::default().build(&config)?;
                        println!(
                            "research_only_warning=comparable evidence remains research-only and never implies live trading"
                        );
                        println!("{}", bundle.to_text());
                        Ok(())
                    })
            }
        }
        ,
        Commands::CandlePack { config } => {
            if config.contains("://") {
                Err("candle-pack config path must be local".to_string())
            } else {
                OfficialCandleCoveragePackConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let pack = OfficialCandleCoveragePack::build(&config)?;
                        pack.write_to_dir(&config.output_dir())?;
                        println!(
                            "research_only_warning=official candle coverage packs remain research-only and never imply live trading"
                        );
                        println!("{}", pack.to_text());
                        Ok(())
                    })
            }
        }
        Commands::CandleCoverageMatch { config } => {
            if config.contains("://") {
                Err("candle-coverage-match config path must be local".to_string())
            } else {
                CandleCoverageClosureConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|closure| {
                        let pack_path = closure
                            .candle_pack_config_path
                            .as_deref()
                            .ok_or_else(|| "candle-coverage-match requires candle_pack_config_path".to_string())?;
                        let pack = load_pack_from_path_or_config(pack_path)?;
                        let rows = if let Some(backfill_path) = closure.backfill_config_path.as_deref() {
                            let cfg = ComparableEvidenceBackfillConfig::from_toml_path(std::path::Path::new(backfill_path))?;
                            let mut rows = Vec::new();
                            for path in cfg.comparable_evidence_bundle_paths {
                                rows.extend(soma_zero::ComparableCommitteeEvidenceBundle::from_json_path(std::path::Path::new(&path))?.rows);
                            }
                            rows
                        } else {
                            Vec::new()
                        };
                        let computation = build_candle_coverage_match_computation(
                            &rows,
                            &pack,
                            &CandleCoverageMatchOptions::default(),
                        );
                        println!(
                            "research_only_warning=candle coverage matching remains research-only and local-only"
                        );
                        println!("{}", computation.match_report.to_text());
                        Ok(())
                    })
            }
        }
        Commands::ComparableBackfill { config } => {
            if config.contains("://") {
                Err("comparable-backfill config path must be local".to_string())
            } else {
                ComparableEvidenceBackfillConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let result = ComparableEvidenceBackfillRunner::default().run_bundle(&config)?;
                        result.write_to_dir(&config.output_dir())?;
                        println!(
                            "research_only_warning=comparable candle backfill remains research-only and never fabricates outcomes"
                        );
                        println!("{}", result.report.to_text());
                        Ok(())
                    })
            }
        }
        Commands::CandleCoverageClose { config } => {
            if config.contains("://") {
                Err("candle-coverage-close config path must be local".to_string())
            } else {
                CandleCoverageClosureConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let bundle = CandleCoverageClosureRunner::default().run_bundle(&config)?;
                        println!(
                            "research_only_warning=candle coverage closure remains research-only and paper-only"
                        );
                        println!("{}", bundle.closure_report.to_text());
                        Ok(())
                    })
            }
        }
        Commands::CandleGapMap { config } => {
            if config.contains("://") {
                Err("candle-gap-map config path must be local".to_string())
            } else {
                OfficialCandleGapConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = OfficialCandleCoverageGapMap::build(&config)?;
                        report.write_to_dir(&config.output_dir())?;
                        println!(
                            "research_only_warning=official candle gap mapping remains research-only and local-only"
                        );
                        println!("{}", report.to_text());
                        Ok(())
                    })
            }
        }
        Commands::CandleExpansionPlan { config } => {
            if config.contains("://") {
                Err("candle-expansion-plan config path must be local".to_string())
            } else {
                OfficialCandleExpansionPlanConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let plan = build_official_candle_acquisition_plan(&config)?;
                        plan.write_to_dir(&config.output_dir())?;
                        println!(
                            "research_only_warning=official candle expansion planning remains research-only and local-only"
                        );
                        println!("{}", plan.to_text());
                        Ok(())
                    })
            }
        }
        Commands::CandleExpansionActions { config } => {
            if config.contains("://") {
                Err("candle-expansion-actions config path must be local".to_string())
            } else {
                OfficialCandleExpansionPlanConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let plan = build_official_candle_acquisition_plan(&config)?;
                        println!(
                            "research_only_warning=official candle expansion actions remain research-only and local-only"
                        );
                        println!(
                            "{}",
                            plan.operator_actions
                                .iter()
                                .map(|action| action.to_text())
                                .collect::<Vec<_>>()
                                .join("\n")
                        );
                        Ok(())
                    })
            }
        }
        Commands::CandleExpand { config } => {
            if config.contains("://") {
                Err("candle-expand config path must be local".to_string())
            } else {
                OfficialCandleExpansionPlanConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let bundle = OfficialCandleExpansionRunner::default().run_bundle(&config)?;
                        println!(
                            "research_only_warning=official candle expansion remains research-only, paper-only, and local-first"
                        );
                        println!("{}", bundle.final_summary);
                        Ok(())
                    })
            }
        }
        Commands::CandleJoinAudit { config } => {
            if config.contains("://") {
                Err("candle-join-audit config path must be local".to_string())
            } else {
                OfficialCandleJoinAuditConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = OfficialCandleJoinAuditRunner::default().run(&config)?;
                        report.write_to_dir(&config.output_dir())?;
                        println!(
                            "research_only_warning=official candle join audit remains research-only, paper-only, and local-only"
                        );
                        println!("{}", report.to_text());
                        Ok(())
                    })
            }
        }
        Commands::CandleJoinRepairPlan { config } => {
            if config.contains("://") {
                Err("candle-join-repair-plan config path must be local".to_string())
            } else {
                OfficialCandleJoinAuditConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = OfficialCandleJoinAuditRunner::default().run(&config)?;
                        let plan = build_join_repair_plan(&config, &report.candidate_report);
                        std::fs::create_dir_all(config.output_dir()).map_err(|err| err.to_string())?;
                        std::fs::write(config.output_dir().join("join_repair_plan.txt"), plan.to_text())
                            .map_err(|err| err.to_string())?;
                        println!(
                            "research_only_warning=official candle join repair planning remains research-only and local-only"
                        );
                        println!("{}", plan.to_text());
                        Ok(())
                    })
            }
        }
        Commands::OfficialReadyMatchClose { config } => {
            if config.contains("://") {
                Err("official-ready-match-close config path must be local".to_string())
            } else {
                OfficialReadyMatchClosureConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let bundle = OfficialReadyMatchClosureRunner::default().run(&config)?;
                        bundle.write_to_dir(&config.output_dir())?;
                        println!(
                            "research_only_warning=official-ready match closure remains research-only, paper-only, and local-only"
                        );
                        println!("{}", bundle.final_summary);
                        Ok(())
                    })
            }
        }
        Commands::OfficialReadyRowInventory { config } => {
            if config.contains("://") {
                Err("official-ready-row-inventory config path must be local".to_string())
            } else {
                OfficialReadyRowInventoryConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = OfficialReadyRowInventoryRunner::default().run(&config)?;
                        report.write_to_dir(&config.output_dir())?;
                        println!(
                            "research_only_warning=official-ready row inventory remains research-only, paper-only, and local-only"
                        );
                        println!("{}", report.to_text());
                        Ok(())
                    })
            }
        }
        Commands::ScenarioMaterializeV3 { config } => {
            if config.contains("://") {
                Err("scenario-materialize-v3 config path must be local".to_string())
            } else {
                ScenarioMaterializationV3Config::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = ScenarioMaterializationV3Runner::default().run(&config)?;
                        report.write_to_dir(&config.output_dir())?;
                        println!(
                            "research_only_warning=scenario materialization v3 remains research-only, paper-only, and local-only"
                        );
                        println!("{}", report.to_text());
                        Ok(())
                    })
            }
        }
        Commands::CompleteRowClose { config } => {
            if config.contains("://") {
                Err("complete-row-close config path must be local".to_string())
            } else {
                CompleteRowClosureConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let bundle = CompleteRowClosureRunner::default().run(&config)?;
                        println!(
                            "research_only_warning=complete row closure remains research-only, paper-only, and local-only"
                        );
                        println!("{}", bundle.final_summary);
                        Ok(())
                    })
            }
        }
        Commands::FutureWindowRequirements { config } => {
            if config.contains("://") {
                Err("future-window-requirements config path must be local".to_string())
            } else {
                FutureWindowRequirementConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = FutureWindowRequirementRunner::default().run(&config)?;
                        report.write_to_dir(&config.output_dir())?;
                        println!(
                            "research_only_warning=future-window requirements remain research-only, local-only, and never imply live trading"
                        );
                        println!("{}", report.to_text());
                        Ok(())
                    })
            }
        }
        Commands::FutureWindowExtensionPlan { config } => {
            if config.contains("://") {
                Err("future-window-extension-plan config path must be local".to_string())
            } else {
                OfficialFutureWindowExtensionConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let plan = build_official_future_window_extension_plan(&config)?;
                        plan.write_to_dir(&config.output_dir())?;
                        println!(
                            "research_only_warning=future-window extension planning remains research-only, local-only, and provider collection stays disabled by default"
                        );
                        println!("{}", plan.to_text());
                        Ok(())
                    })
            }
        }
        Commands::OutcomeLinkageV3 { config } => {
            if config.contains("://") {
                Err("outcome-linkage-v3 config path must be local".to_string())
            } else {
                OutcomeLinkageV3Config::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = OutcomeLinkageV3Runner::default().run(&config)?;
                        report.write_to_dir(&config.output_dir())?;
                        println!(
                            "research_only_warning=outcome linkage v3 remains research-only, local-only, and no-lookahead-safe only"
                        );
                        println!("{}", report.to_text());
                        Ok(())
                    })
            }
        }
        Commands::CounterfactualCompleteV2 { config } => {
            if config.contains("://") {
                Err("counterfactual-complete-v2 config path must be local".to_string())
            } else {
                CounterfactualCompletionV2Config::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = CounterfactualCompletionV2Runner::default().run(&config)?;
                        report.write_to_dir(&config.output_dir())?;
                        println!(
                            "research_only_warning=counterfactual completion v2 remains research-only, paper-only, and outcome-dependent"
                        );
                        println!("{}", report.to_text());
                        Ok(())
                    })
            }
        }
        Commands::CompleteRowCloseV2 { config } => {
            if config.contains("://") {
                Err("complete-row-close-v2 config path must be local".to_string())
            } else {
                CompleteRowClosureV2Config::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let bundle = CompleteRowClosureV2Runner::default().run(&config)?;
                        println!(
                            "research_only_warning=complete row closure v2 remains research-only, paper-only, and local-only"
                        );
                        println!("{}", bundle.final_summary);
                        Ok(())
                    })
            }
        }
        Commands::CandleLineage { config } => {
            if config.contains("://") {
                Err("candle-lineage config path must be local".to_string())
            } else {
                OfficialCandleJoinAuditConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let report = OfficialCandleJoinAuditRunner::default().run(&config)?;
                        std::fs::create_dir_all(config.output_dir()).map_err(|err| err.to_string())?;
                        std::fs::write(
                            config.output_dir().join("official_candle_lineage.txt"),
                            report.lineage_report.to_text(),
                        )
                        .map_err(|err| err.to_string())?;
                        println!(
                            "research_only_warning=official candle lineage remains research-only and local-only"
                        );
                        println!("{}", report.lineage_report.to_text());
                        Ok(())
                    })
            }
        }
        Commands::CounterfactualDepthPlan { config } => {
            if config.contains("://") {
                Err("counterfactual-depth-plan config path must be local".to_string())
            } else {
                ComparableCommitteeEvidenceConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let bundle = ComparableEvidenceBuilder::default().build(&config)?;
                        let plan = CounterfactualDepthPlan::from_bundle(&config, &bundle);
                        println!(
                            "research_only_warning=counterfactual depth planning remains research-only and local-only"
                        );
                        println!("{}", plan.to_text());
                        Ok(())
                    })
            }
        }
        Commands::CounterfactualDepthClose { config } => {
            if config.contains("://") {
                Err("counterfactual-depth-close config path must be local".to_string())
            } else {
                CounterfactualDepthClosureConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let bundle = CounterfactualDepthClosureRunner::default().run_bundle(&config)?;
                        println!(
                            "research_only_warning=counterfactual depth closure remains research-only and never implies live trading"
                        );
                        println!("{}", bundle.closure_report.to_text());
                        Ok(())
                    })
            }
        }
        Commands::ScorecardRerun { config } => {
            if config.contains("://") {
                Err("scorecard-rerun config path must be local".to_string())
            } else {
                CounterfactualDepthClosureConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        if let Some(scorecard_path) = &config.core_performance_config_path {
                            let bundle = CoreScorecardRerun::default().run_bundle(scorecard_path)?;
                            let summary = CoreScorecardRerun::default().summarize(
                                None,
                                Some(&bundle.scorecard),
                                vec!["scorecard rerun invoked without previous baseline".to_string()],
                                true,
                            );
                            println!(
                                "research_only_warning=scorecard rerun output remains research-only and paper-only"
                            );
                            println!("{}", summary.to_text());
                            Ok(())
                        } else {
                            Err("scorecard-rerun requires core_performance_config_path".to_string())
                        }
                    })
            }
        }
        Commands::OfficialReplication { config } => {
            if config.contains("://") {
                Err("official-replication config path must be local".to_string())
            } else {
                OfficialEvidenceReplicationConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let bundle = OfficialEvidenceReplicationRunner::default().run_bundle(&config)?;
                        println!("research_only_warning=official replication remains research-only and paper-only");
                        println!("{}", bundle.final_summary);
                        Ok(())
                    })
            }
        }
        Commands::OfficialArtifactInventory { config } => {
            if config.contains("://") {
                Err("official-artifact-inventory config path must be local".to_string())
            } else {
                OfficialEvidenceReplicationConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let inventory =
                            OfficialEvidenceReplicationRunner::default().inventory(&config)?;
                        println!("research_only_warning=official artifact inventory remains research-only and local-only");
                        println!("{}", inventory.to_text());
                        Ok(())
                    })
            }
        }
        Commands::OfficialRowInject { config } => {
            if config.contains("://") {
                Err("official-row-inject config path must be local".to_string())
            } else {
                OfficialEvidenceReplicationConfig::from_toml_path(std::path::Path::new(&config))
                    .and_then(|config| {
                        let result =
                            OfficialEvidenceReplicationRunner::default().row_injection(&config)?;
                        println!("research_only_warning=official row injection remains research-only and paper-only");
                        println!("{}", result.to_text());
                        Ok(())
                    })
            }
        }
        Commands::EvidenceExpand { config } => {
            if config.contains("://") {
                Err("evidence-expand config path must be local".to_string())
            } else {
                OfficialEvidenceExpansionConfig::from_toml_path(std::path::Path::new(&config))
                    .map_err(|err| err.to_string())
                    .and_then(|config| {
                        let report = OfficialEvidenceExpansionRunner::default()
                            .run(&config)
                            .map_err(|err| err.to_string())?;
                        println!("{}", official_evidence_expansion_report_to_text(&report));
                        Ok(())
                    })
            }
        }
        Commands::OfficialAcquire { config } => {
            if config.contains("://") {
                Err("official-acquire config path must be local".to_string())
            } else {
                OfficialEvidenceAcquisitionPlan::from_toml_path(std::path::Path::new(&config))
                    .map_err(|err| err.to_string())
                    .and_then(|plan| {
                        let report = OfficialEvidenceAcquisitionRunner::default()
                            .run(&plan)
                            .map_err(|err| err.to_string())?;
                        println!("{}", official_evidence_acquisition_report_to_text(&report));
                        Ok(())
                    })
            }
        }
        Commands::OfficialCoverage { config } => {
            if config.contains("://") {
                Err("official-coverage config path must be local".to_string())
            } else {
                VenueCoverageExpansionPlan::from_toml_path(std::path::Path::new(&config))
                    .map_err(|err| err.to_string())
                    .and_then(|plan| {
                        let collection_report =
                            if let Some(path) = plan.existing_collection_report_path.as_deref() {
                                Some(
                                    OfficialCollectionReport::from_json_path(std::path::Path::new(
                                        path,
                                    ))
                                    .map_err(|err| err.to_string())?,
                                )
                            } else {
                                None
                            };
                        let report = soma_zero::build_venue_coverage_report(
                            &plan,
                            collection_report.as_ref(),
                            None,
                        );
                        println!("{}", venue_coverage_report_to_text(&report));
                        Ok(())
                    })
            }
        }
        Commands::SourceBenchmark { config } => {
            if config.contains("://") {
                Err("source-benchmark config path must be local".to_string())
            } else {
                SourceAwareBenchmarkConfig::from_toml_path(std::path::Path::new(&config))
                    .map_err(|err| err.to_string())
                    .and_then(|config| {
                        let report = SourceAwareBenchmarkRunner::default().run(&config)?;
                        println!("{}", source_aware_benchmark_report_to_text(&report));
                        Ok(())
                    })
            }
        }
        Commands::YfinanceImport { config } => {
            if config.contains("://") {
                Err("yfinance-import config path must be local".to_string())
            } else {
                YFinanceImportConfig::from_toml_path(std::path::Path::new(&config))
                    .map_err(|err| err.to_string())
                    .and_then(|config| {
                        let report = soma_zero::run_yfinance_preflight_bridge(&config)?;
                        println!("{}", report.to_text());
                        Ok(())
                    })
            }
        }
        Commands::YahooResearch { config } => {
            if config.contains("://") {
                Err("yahoo-research config path must be local".to_string())
            } else {
                YahooResearchEvidenceConfig::from_toml_path(std::path::Path::new(&config))
                    .map_err(|err| err.to_string())
                    .and_then(|config| {
                        let report = YahooResearchEvidenceRunner::default().run(&config)?;
                        println!("{}", yahoo_research_report_to_text(&report));
                        Ok(())
                    })
            }
        }
        Commands::OfficialVsYfinance {
            official_report,
            yfinance_report,
            out,
            official_metric,
            yfinance_metric,
        } => Ok(()).and_then(|_| {
            if official_report
                .as_deref()
                .is_some_and(|value| value.contains("://"))
                || yfinance_report.contains("://")
                || out.as_deref().is_some_and(|value| value.contains("://"))
            {
                return Err("official-vs-yfinance paths must be local".to_string());
            }
            let official_ready_count = if let Some(path) = official_report.as_deref() {
                load_official_ready_count(path)?
            } else {
                0
            };
            let yfinance_research_count = load_yfinance_research_count(&yfinance_report)?;
            let interpretation = build_official_vs_yfinance_interpretation(
                official_ready_count,
                yfinance_research_count,
                official_metric,
                yfinance_metric,
            );
            if let Some(out_dir) = out.as_deref() {
                interpretation.write_to_dir(std::path::Path::new(out_dir))?;
            }
            println!("{}", official_vs_yfinance_to_text(&interpretation));
            Ok(())
        }),
        Commands::Compare { current, previous } => Ok(()).and_then(|_| {
            if current.contains("://") || previous.contains("://") {
                return Err("compare paths must be local".to_string());
            }
            let current_report =
                ResearchCampaignReport::from_json_path(std::path::Path::new(&current))?;
            let previous_report =
                ResearchCampaignReport::from_json_path(std::path::Path::new(&previous))?;
            let diff = soma_zero::experiment::diff::build_campaign_diff_report(
                &current_report.aggregate,
                Some(&previous_report.aggregate),
                Some(&previous_report.campaign_id),
            );
            println!("{}", diff_report_to_text(&diff));
            Ok(())
        }),
        Commands::Baseline {
            data,
            symbol,
            timeframe,
            out,
        } => Ok(()).map(|_| {
            let bundle = ExperimentRunner::default().run(&ExperimentConfig::baseline_only(
                "cli-baseline",
                symbol,
                data,
                parse_timeframe(&timeframe),
                out,
            ));
            println!("{}", bundle.to_deterministic_summary());
        }),
        Commands::Dataset {
            data,
            symbol,
            timeframe,
            out,
        } => Ok(()).map(|_| {
            let bundle = ExperimentRunner::default().run(&ExperimentConfig {
                mode: ExperimentMode::DatasetExportOnly,
                ..ExperimentConfig::dataset_export_only(
                    "cli-dataset",
                    symbol,
                    data,
                    parse_timeframe(&timeframe),
                    out,
                )
            });
            println!("{}", bundle.to_deterministic_summary());
        }),
            }
            .unwrap_or_else(|err| {
                eprintln!("soma-experiment: {err}");
                std::process::exit(1);
            });
        });
    match worker {
        Ok(handle) => {
            if handle.join().is_err() {
                eprintln!("soma-experiment: CLI worker panicked");
                std::process::exit(1);
            }
        }
        Err(err) => {
            eprintln!("soma-experiment: failed to start CLI worker: {err}");
            std::process::exit(1);
        }
    }
}

fn parse_timeframe(value: &str) -> Timeframe {
    match value {
        "1m" | "OneMinute" => Timeframe::OneMinute,
        "5m" | "FiveMinute" => Timeframe::FiveMinute,
        "15m" | "FifteenMinute" => Timeframe::FifteenMinute,
        "1h" | "OneHour" => Timeframe::OneHour,
        "1d" | "OneDay" => Timeframe::OneDay,
        _ => Timeframe::OneMinute,
    }
}

fn parse_provider(value: &str) -> Result<ProviderKind, String> {
    match value {
        "upbit" => Ok(ProviderKind::Upbit),
        "korbit" => Ok(ProviderKind::Korbit),
        "krx" => Ok(ProviderKind::KrxOpenApi),
        "data-go-kr" | "datagokr" => Ok(ProviderKind::DataGoKrFscStockPrice),
        "alphavantage" => Ok(ProviderKind::AlphaVantage),
        "alpaca" => Ok(ProviderKind::Alpaca),
        "kis" | "kis-market-data" => Ok(ProviderKind::KoreaInvestmentMarketData),
        "polygon" => Ok(ProviderKind::PolygonProfessional),
        "nasdaq-data-link" => Ok(ProviderKind::NasdaqDataLink),
        "koscom" => Ok(ProviderKind::KoscomProfessional),
        "mock-fixture" => Ok(ProviderKind::MockFixture),
        _ => Err(format!("unsupported provider: {value}")),
    }
}

fn parse_fill_missing_policy(value: &str) -> FillMissingPolicy {
    match value {
        "insert-empty" => FillMissingPolicy::InsertEmptyRows,
        "reject" => FillMissingPolicy::RejectIfGaps,
        _ => FillMissingPolicy::LeaveGaps,
    }
}

fn parse_provider_market(value: &str) -> Result<ProviderMarket, String> {
    match value {
        "crypto" => Ok(ProviderMarket::Crypto),
        "korean-equity" => Ok(ProviderMarket::KoreanEquity),
        "us-equity" => Ok(ProviderMarket::USEquity),
        "global-equity" => Ok(ProviderMarket::GlobalEquity),
        _ => Err(format!("unsupported market: {value}")),
    }
}

fn parse_strategy_use_case(value: &str) -> Result<StrategyUseCase, String> {
    match value {
        "eod-swing" => Ok(StrategyUseCase::EodSwing),
        "daily-portfolio-research" => Ok(StrategyUseCase::DailyPortfolioResearch),
        "intraday-swing" => Ok(StrategyUseCase::IntradaySwing),
        "realtime-scalping" => Ok(StrategyUseCase::RealtimeScalping),
        "realtime-execution-simulation" => Ok(StrategyUseCase::RealtimeExecutionSimulation),
        "source-comparison" => Ok(StrategyUseCase::SourceComparison),
        "model-prototype-research" => Ok(StrategyUseCase::ModelPrototypeResearch),
        _ => Err(format!("unsupported use-case: {value}")),
    }
}

fn parse_budget_preference(value: &str) -> Result<BudgetPreference, String> {
    match value {
        "free-only" => Ok(BudgetPreference::FreeOnly),
        "free-or-low-cost" => Ok(BudgetPreference::FreeOrLowCost),
        "paid-allowed" => Ok(BudgetPreference::PaidAllowed),
        "professional-allowed" => Ok(BudgetPreference::ProfessionalAllowed),
        _ => Err(format!("unsupported budget preference: {value}")),
    }
}

fn parse_market_venue(value: &str) -> Result<MarketVenue, String> {
    match value {
        "KRX" | "krx" => Ok(MarketVenue::KRX),
        "KOSPI" | "kospi" => Ok(MarketVenue::KOSPI),
        "KOSDAQ" | "kosdaq" => Ok(MarketVenue::KOSDAQ),
        "NASDAQ" | "nasdaq" => Ok(MarketVenue::NASDAQ),
        "NYSE" | "nyse" => Ok(MarketVenue::NYSE),
        "AMEX" | "amex" => Ok(MarketVenue::AMEX),
        "US" | "us" => Ok(MarketVenue::US),
        "UPBIT" | "upbit" => Ok(MarketVenue::Upbit),
        _ => Err(format!("unsupported venue: {value}")),
    }
}

fn infer_asset_class(provider: ProviderKind) -> AssetClass {
    match provider {
        ProviderKind::Upbit | ProviderKind::Binance | ProviderKind::Korbit => AssetClass::Crypto,
        ProviderKind::KrxOpenApi
        | ProviderKind::DataGoKrFscStockPrice
        | ProviderKind::AlphaVantage
        | ProviderKind::Alpaca
        | ProviderKind::KoreaInvestmentMarketData
        | ProviderKind::PolygonProfessional
        | ProviderKind::NasdaqDataLink
        | ProviderKind::KoscomProfessional => AssetClass::Equity,
        ProviderKind::MockFixture | ProviderKind::Unknown => AssetClass::Unknown,
    }
}

fn provider_kind_label(provider: ProviderKind) -> String {
    match provider {
        ProviderKind::Upbit => "upbit".to_string(),
        ProviderKind::Binance => "binance".to_string(),
        ProviderKind::Korbit => "korbit".to_string(),
        ProviderKind::KrxOpenApi => "krx".to_string(),
        ProviderKind::DataGoKrFscStockPrice => "data-go-kr-fsc-stock-price".to_string(),
        ProviderKind::AlphaVantage => "alphavantage".to_string(),
        ProviderKind::Alpaca => "alpaca".to_string(),
        ProviderKind::KoreaInvestmentMarketData => "kis-market-data-only".to_string(),
        ProviderKind::PolygonProfessional => "polygon".to_string(),
        ProviderKind::NasdaqDataLink => "nasdaq-data-link".to_string(),
        ProviderKind::KoscomProfessional => "koscom".to_string(),
        ProviderKind::MockFixture => "mock-fixture".to_string(),
        ProviderKind::Unknown => "unknown".to_string(),
    }
}

fn parse_timestamp_like(value: &str) -> Result<u64, String> {
    if value.len() == 8 && value.chars().all(|ch| ch.is_ascii_digit()) {
        let year = value[0..4].parse::<i32>().map_err(|err| err.to_string())?;
        let month = value[4..6].parse::<u32>().map_err(|err| err.to_string())?;
        let day = value[6..8].parse::<u32>().map_err(|err| err.to_string())?;
        let days = days_from_civil(year, month, day);
        let millis = days
            .checked_mul(86_400_000)
            .ok_or_else(|| "timestamp overflow".to_string())?;
        return u64::try_from(millis).map_err(|_| "timestamp overflow".to_string());
    }
    value.parse::<u64>().map_err(|err| err.to_string())
}

fn parse_raw_archive_policy(value: &str) -> RawArchivePolicy {
    match value {
        "none" => RawArchivePolicy::None,
        "headers" => RawArchivePolicy::HeadersOnly,
        "full" => RawArchivePolicy::FullRawAllowedOnlyWithExplicitFlag,
        _ => RawArchivePolicy::CompactJson,
    }
}

fn parse_requested_output_size(value: &str) -> Result<RequestedOutputSize, String> {
    match value {
        "compact" => Ok(RequestedOutputSize::Compact),
        "full" => Ok(RequestedOutputSize::Full),
        _ => Err(format!("unsupported outputsize: {value}")),
    }
}

fn parse_adjusted_price_policy(value: &str) -> AdjustedPricePolicy {
    match value {
        "adjusted" => AdjustedPricePolicy::Adjusted,
        "both" => AdjustedPricePolicy::BothIfAvailable,
        _ => AdjustedPricePolicy::Raw,
    }
}

fn load_official_ready_count(path: &str) -> Result<usize, String> {
    let text = std::fs::read_to_string(path).map_err(|err| err.to_string())?;
    if let Ok(report) = serde_json::from_str::<OfficialEvidenceAcquisitionReport>(&text) {
        return Ok(report
            .collection_report
            .as_ref()
            .map(|value| value.ready_entries_count)
            .unwrap_or(0));
    }
    let value: serde_json::Value = serde_json::from_str(&text).map_err(|err| err.to_string())?;
    Ok(value
        .get("collection_report")
        .and_then(|report| report.get("ready_entries_count"))
        .and_then(|value| value.as_u64())
        .unwrap_or(0) as usize)
}

fn load_yfinance_research_count(path: &str) -> Result<usize, String> {
    let text = std::fs::read_to_string(path).map_err(|err| err.to_string())?;
    if let Ok(report) = serde_json::from_str::<YahooResearchEvidenceReport>(&text) {
        return Ok(report.yfinance_symbols.len());
    }
    let value: serde_json::Value = serde_json::from_str(&text).map_err(|err| err.to_string())?;
    Ok(value
        .get("yfinance_symbols")
        .and_then(|value| value.as_array())
        .map(|value| value.len())
        .unwrap_or(0))
}

fn days_from_civil(year: i32, month: u32, day: u32) -> i64 {
    let year = year - i32::from(month <= 2);
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let yoe = year - era * 400;
    let mp = month as i32 + if month > 2 { -3 } else { 9 };
    let doy = (153 * mp + 2) / 5 + day as i32 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    i64::from(era * 146_097 + doe - 719_468)
}
