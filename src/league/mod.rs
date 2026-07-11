pub mod balanced_outcome_coverage;
pub mod barrier_profile_registry;
pub mod baseline_reference_backfill;
pub mod baseline_reference_generator;
pub mod batch_counterfactual_completion;
pub mod batch_outcome_linkage_v3;
pub mod candidate_generation;
pub mod candidate_lifecycle;
pub mod candle_acquisition_job;
pub mod candle_alignment;
pub mod candle_coverage_closure;
pub mod candle_coverage_closure_bundle;
pub mod candle_coverage_match;
pub mod candle_coverage_storage;
pub mod candle_expansion_closure;
pub mod candle_expansion_operator_actions;
pub mod chair_calibration;
pub mod chair_diagnostics;
pub mod chair_v0;
pub mod committee_actionability;
pub mod committee_artifact_resolver;
pub mod committee_attribution;
pub mod committee_benchmark;
pub mod committee_benchmark_bundle;
pub mod committee_benchmark_readiness;
pub mod committee_counterfactual_audit;
pub mod committee_counterfactual_builder;
pub mod committee_cycle_runner;
pub mod committee_decision;
pub mod committee_decision_quality;
pub mod committee_diagnostics;
pub mod committee_evaluation;
pub mod committee_evidence_quality;
pub mod committee_evidence_sufficiency;
pub mod committee_materialization;
pub mod committee_official_benchmark;
pub mod committee_outcome_coverage;
pub mod committee_outcome_coverage_bundle;
pub mod committee_outcome_coverage_runner;
pub mod committee_outcome_linked_comparison;
pub mod committee_outcome_linker;
pub mod committee_outcome_reference;
pub mod committee_performance_matrix;
pub mod committee_reference_pack;
pub mod committee_reference_pack_bundle;
pub mod committee_reference_pack_runner;
pub mod committee_replay;
pub mod committee_risk_bridge;
pub mod committee_scenario_loader;
pub mod committee_smoke;
pub mod committee_v1;
pub mod committee_v1_bundle;
pub mod committee_v1_readiness;
pub mod committee_v1_runner;
pub mod committee_value_attribution;
pub mod committee_vs_baseline;
pub mod committee_work_queue;
pub mod comparable_committee_evidence;
pub mod comparable_evidence_backfill;
pub mod comparable_evidence_builder;
pub mod comparable_evidence_quality;
pub mod complete_comparable_row_builder;
pub mod complete_row_closure;
pub mod complete_row_closure_bundle;
pub mod complete_row_closure_v2;
pub mod complete_row_closure_v2_bundle;
pub mod core_bottleneck_movement;
pub mod counterfactual_backfill_plan;
pub mod counterfactual_completion_v2;
pub mod counterfactual_depth_closure;
pub mod counterfactual_depth_closure_bundle;
pub mod counterfactual_depth_plan;
pub mod counterfactual_reference_generator;
pub mod cycle_risk_skeptic;
pub mod diversity_aware_sufficiency_v2;
pub mod doctrine;
pub mod evaluation;
pub mod future_window_extension_job;
pub mod future_window_requirements;
pub mod future_window_scaleout;
pub mod gap_expansion_consistency;
pub mod join_repair_plan;
pub mod match_key_normalization;
pub mod minimal_ai_committee_core;
pub mod momentum_trend_fast;
pub mod multi_row_official_evidence;
pub mod official_candle_coverage;
pub mod official_candle_coverage_pack;
pub mod official_candle_expansion_bundle;
pub mod official_candle_expansion_plan;
pub mod official_candle_expansion_runner;
pub mod official_candle_gap_map;
pub mod official_candle_join_audit;
pub mod official_candle_lineage;
pub mod official_committee_benchmark_bundle;
pub mod official_committee_pack;
pub mod official_committee_readiness;
pub mod official_diversity_row_selector;
pub mod official_evidence_diversity_bundle;
pub mod official_evidence_diversity_gap;
pub mod official_evidence_diversity_sweep;
pub mod official_evidence_replication;
pub mod official_evidence_scaleout;
pub mod official_evidence_scaleout_bundle;
pub mod official_evidence_sufficiency_v2;
pub mod official_future_window_extension;
pub mod official_ready_match_closure;
pub mod official_ready_match_closure_bundle;
pub mod official_ready_row_inventory;
pub mod official_reference_replication;
pub mod official_replication_bundle;
pub mod official_replication_inventory;
pub mod official_replication_operator_actions;
pub mod official_row_injection;
pub mod official_sufficiency_replication;
pub mod operational_audit_timeline;
pub mod outcome_diversity_audit;
pub mod outcome_linkage_v3;
pub mod outcome_reference_backfill;
pub mod paper_position_lifecycle;
pub mod persona;
pub mod persona_card;
pub mod persona_card_lite;
pub mod persona_conflict_matrix;
pub mod persona_operational_status;
pub mod persona_scorer;
pub mod persona_vote;
pub mod reference_pack_quality;
pub mod risk_bridge_diagnostics;
pub mod risk_calibration;
pub mod row_candle_candidate;
pub mod scenario_materialization_closure;
pub mod scenario_materialization_v3;
pub mod scoring;
pub mod shadow;
pub mod six_persona_readiness;
pub mod sprint102_paper_rotation;
pub mod sprint103_paper_rotation_closure;
pub mod sprint104_dual_agent_paper_lifecycle;
pub mod sprint105_verification_patch_closure;
pub mod sprint106_workspace_acceptance_recovery;
pub mod sprint107_safe_consolidation_patch;
pub mod sprint108_safe_consolidation_patch_v2;
pub mod sprint109_safe_consolidation_patch_v3;
pub mod sprint110_safe_consolidation_patch_v4;
pub mod sprint111_workspace_timeout_root_cause;
pub mod sprint112_workspace_diagnostic_pilot;
pub mod sprint113_real_workspace_observation;
pub mod sprint114_mixed_family_isolation;
pub mod sprint115_consolidation_governance;
pub mod sprint116_workspace_timeout_track;
pub mod sprint117_deferred_real_observation;
pub mod sprint118_timeout_reduction_queue;
pub mod sprint98_committee_owned_core;
pub mod sufficiency_closure;
pub mod tier;
pub mod timeframe_alignment;
pub mod timestamp_alignment_v2;
pub mod trinity_operational_loop;
pub mod trinity_personas;
pub mod triple_barrier_reference_builder;
pub mod value_quality_filter;
pub mod voice;

pub use balanced_outcome_coverage::{
    BalancedOutcomeCoverageCell, BalancedOutcomeCoverageConfig, BalancedOutcomeCoverageReport,
    BalancedOutcomeCoverageRunner, BalancedOutcomeCoverageStatus,
    load_balanced_outcome_coverage_from_path_or_config,
};
pub use barrier_profile_registry::{
    BarrierProfile, BarrierProfileIntendedUse, BarrierProfileKind, BarrierProfileRegistry,
    BarrierProfileRegistryBuilder, BarrierProfileRegistryConfig,
    load_barrier_profile_registry_from_path_or_config,
};
pub use baseline_reference_backfill::{
    BaselineBackfillSource, BaselineReferenceBackfillPlan, BaselineReferenceBackfillPlanItem,
    build_baseline_reference_backfill_plan,
};
pub use baseline_reference_generator::{
    BaselineGenerationResult, BaselineReferenceGenerator, BaselineReferencePolicy,
    BaselineReferenceSource, LoadedBaselineReference,
};
pub use batch_counterfactual_completion::{
    BatchCounterfactualCompletionConfig, BatchCounterfactualCompletionReport,
    BatchCounterfactualCompletionRunner, load_batch_counterfactual_completion_from_path_or_config,
};
pub use batch_outcome_linkage_v3::{
    BatchOutcomeLinkageV3Config, BatchOutcomeLinkageV3Report, BatchOutcomeLinkageV3Runner,
    load_batch_outcome_linkage_v3_from_path_or_config,
};
pub use candidate_generation::{
    CandidateEvidenceClass, CandidateGenerationFromEvidence, CandidateGenerationInput,
    CandidateGenerationReport, CandidateGenerationSettings, CandidateGenerationStatus,
    CandidateSourceKind, GeneratedCandidate, SkippedCandidate, write_candidate_generation_report,
};
pub use candidate_lifecycle::{
    CandidateLifecycleEvent, CandidateLifecycleStateMachine, CandidateLifecycleStatus,
    CandidateLifecycleTransition,
};
pub use candle_acquisition_job::{
    CandleAcquisitionJob, CandleAcquisitionJobKind, CandleAcquisitionJobStatus,
    CandleAcquisitionPlan,
};
pub use candle_alignment::{
    CandleAligner, CandleAlignmentOverallStatus, CandleAlignmentRecord, CandleAlignmentReport,
    CandleAlignmentStatus,
};
pub use candle_coverage_closure::{
    CandleCoverageClosureConfig, CandleCoverageClosureFinalStatus,
    CandleCoverageClosureRecommendation, CandleCoverageClosureReport, CandleCoverageClosureRunner,
};
pub use candle_coverage_closure_bundle::CandleCoverageClosureBundle;
pub use candle_coverage_match::{
    CandleCoverageMatch, CandleCoverageMatchComputation, CandleCoverageMatchOptions,
    CandleCoverageMatchReport, CandleCoverageMatchStatus, CandleCoverageStatus,
    build_candle_coverage_match_computation,
};
pub use candle_coverage_storage::{
    CandleCoverageArtifactSize, CandleCoverageStorageReport, build_candle_coverage_storage_report,
};
pub use candle_expansion_closure::{
    CandleExpansionClosureReport, CandleExpansionClosureStatus,
    build_candle_expansion_closure_report,
};
pub use candle_expansion_operator_actions::{
    CandleExpansionActionPriority, CandleExpansionOperatorAction,
    build_candle_expansion_operator_actions, rebuild_plan_actions,
};
pub use chair_calibration::{
    CalibrationDirection, CalibrationSafetyImpact, ChairCalibrationRecommendation,
    ChairCalibrationReport, ChairCalibrationSuggestion, build_chair_calibration_report,
};
pub use chair_diagnostics::{
    ChairDiagnosticStatus, ChairDiagnosticsReport, SpeakerFilterReason, SpeakerSelectionTrace,
    build_chair_diagnostics,
};
pub use chair_v0::ChairV0;
pub use committee_actionability::{
    CommitteeActionabilityReport, CommitteeActionabilityStatus,
    build_committee_actionability_report,
};
pub use committee_artifact_resolver::{
    CommitteeArtifactDescriptor, CommitteeArtifactKind, CommitteeArtifactResolver,
};
pub use committee_attribution::{
    CommitteeAttributionReport, CommitteeAttributionStatus, PersonaContribution,
    build_committee_attribution_report,
};
pub use committee_benchmark::{
    CommitteeBenchmarkConfig, CommitteeBenchmarkFinalStatus, CommitteeBenchmarkReport,
    CommitteeBenchmarkRunner,
};
pub use committee_benchmark_bundle::{
    CommitteeBenchmarkBundle, CommitteeBenchmarkDiagnosticsSummary,
};
pub use committee_benchmark_readiness::{
    CommitteeBenchmarkNextRecommendation, CommitteeBenchmarkReadinessReport,
    CommitteeBenchmarkReadinessStatus, build_committee_benchmark_readiness_report,
};
pub use committee_counterfactual_audit::{
    CommitteeCounterfactualAuditConfig, CommitteeCounterfactualAuditReport,
    CommitteeCounterfactualAuditRunner, CommitteeCounterfactualAuditStatus,
    build_committee_counterfactual_audit_report,
};
pub use committee_counterfactual_builder::{
    CommitteeCounterfactualBuildConfig, CommitteeCounterfactualBuilder,
    CommitteeCounterfactualRecord, CommitteeCounterfactualType, CounterfactualBuildStatus,
    horizon_bars_for_row, load_local_candle_series_map,
};
pub use committee_cycle_runner::{
    CommitteeCycleConfig, CommitteeCycleInput, CommitteeCycleOwnerContext, CommitteeCycleRecord,
    CommitteeCycleRunner, load_generated_candidate_from_path, load_owner_inputs_from_paths,
    load_risk_snapshot_from_paths, run_committee_cycle_from_config, write_committee_cycle_record,
};
pub use committee_decision::{
    ChairCommitteeConfig, CommitteeDebateReport, CommitteeDecision, CommitteeDecisionRecord,
    CommitteeInput, PersonaCluster,
};
pub use committee_decision_quality::{
    CommitteeDecisionQualityReport, CommitteeDecisionQualityStatus,
    build_committee_decision_quality_report,
};
pub use committee_diagnostics::{
    CommitteeDiagnosticsAggregate, CommitteeDiagnosticsBundle, CommitteeDiagnosticsConfig,
    CommitteeDiagnosticsRecommendation, CommitteeDiagnosticsRunner, CommitteeDiagnosticsStatus,
};
pub use committee_evaluation::{
    CommitteeEvaluationRecommendation, CommitteeEvaluationScaffold, PersonaEvaluationMetric,
    build_committee_evaluation_scaffold,
};
pub use committee_evidence_quality::{
    CommitteeEvidenceQualityReport, CommitteeEvidenceQualityStatus,
    build_committee_evidence_quality_report,
};
pub use committee_evidence_sufficiency::{
    CommitteeEvidenceSufficiencyGateConfig, CommitteeEvidenceSufficiencyGateResult,
    CommitteeEvidenceSufficiencyStatus, evaluate_committee_evidence_sufficiency,
};
pub use committee_materialization::{
    CommitteeMaterializationConfig, CommitteeScenarioMaterializerV2,
};
pub use committee_official_benchmark::{
    CommitteeOfficialBenchmarkConfig, CommitteeOfficialBenchmarkFinalStatus,
    CommitteeOfficialBenchmarkReport, CommitteeOfficialBenchmarkRunner,
};
pub use committee_outcome_coverage::{
    CommitteeOutcomeCoverageConfig, CommitteeOutcomeCoverageReport, CommitteeOutcomeCoverageStatus,
    OutcomeCoverageCell, build_committee_outcome_coverage_report,
};
pub use committee_outcome_coverage_bundle::{
    CommitteeOutcomeCoverageBundle, CommitteeOutcomeCoverageBundleStatus,
    CommitteeOutcomeCoverageRecommendation,
};
pub use committee_outcome_coverage_runner::CommitteeOutcomeCoverageRunner;
pub use committee_outcome_linked_comparison::{
    CommitteeOutcomeLinkedComparison, CommitteeOutcomeLinkedComparisonStatus,
    build_committee_outcome_linked_comparison,
};
pub use committee_outcome_linker::{
    CommitteeOutcomeLinkSummary, CommitteeOutcomeLinker, CommitteeOutcomeLinkerConfig,
    OutcomeLinkedCommitteeScenarioPack, OutcomeLinkedCommitteeScenarioRow,
};
pub use committee_outcome_reference::{
    CommitteeBaselineAction, CommitteeBaselineReference, CommitteeExternalReference,
    CommitteeOutcomeReference, CommitteeTripleBarrierLabel,
};
pub use committee_performance_matrix::{
    CommitteePerformanceEvidenceMatrix, CommitteePerformanceStatus, EvidenceStrength,
    PerformanceEvidenceCell, build_committee_performance_evidence_matrix,
};
pub use committee_reference_pack::{
    CommitteeReferencePackConfig, GeneratedCommitteeReference, GeneratedCommitteeReferencePack,
    GeneratedReferenceKind, GeneratedReferenceSource, GeneratedReferenceStatus,
};
pub use committee_reference_pack_bundle::{
    CommitteeReferencePackBundle, CommitteeReferencePackFinalStatus,
    CommitteeReferencePackRecommendation,
};
pub use committee_reference_pack_runner::CommitteeReferencePackRunner;
pub use committee_replay::{
    CommitteeDebateReplay, CommitteeReplayConfig, CommitteeReplayRecord, CommitteeReplayReport,
};
pub use committee_risk_bridge::{CommitteeFinalAction, CommitteeOutcome, CommitteeRiskBridge};
pub use committee_scenario_loader::{
    CommitteeScenarioLoadConfig, CommitteeScenarioLoader, CommitteeScenarioMaterializationLevel,
    CommitteeScenarioRow, CommitteeScenarioSet, CommitteeScenarioSourceKind,
};
pub use committee_smoke::{
    CommitteeSmokeFinalStatus, CommitteeSmokeRecommendation, CommitteeSmokeTestConfig,
    CommitteeSmokeTestReport, CommitteeSmokeTestRunner,
};
pub use committee_v1::CommitteeV1RunConfig;
pub use committee_v1_bundle::{
    ChairDiagnosticsSummary, CommitteeV1FinalStatus, CommitteeV1ReportBundle,
    RiskDiagnosticsSummary,
};
pub use committee_v1_readiness::{
    CommitteeV1NextRecommendation, CommitteeV1ReadinessReport, CommitteeV1ReadinessStatus,
    build_committee_v1_readiness_report,
};
pub use committee_v1_runner::CommitteeV1Runner;
pub use committee_value_attribution::{
    CommitteeValueAttributionInputs, CommitteeValueAttributionReport,
    CommitteeValueAttributionStatus, build_committee_value_attribution_report,
};
pub use committee_vs_baseline::{
    CommitteeVsBaselineComparison, CommitteeVsBaselineStatus,
    build_committee_vs_baseline_comparison,
};
pub use committee_work_queue::{
    CommitteeTaskKind, CommitteeTaskStatus, CommitteeWorkItem, CommitteeWorkQueue,
    build_committee_work_queue,
};
pub use comparable_committee_evidence::{
    ComparableCommitteeEvidenceBundle, ComparableCommitteeEvidenceConfig,
    ComparableCommitteeEvidenceRow, ComparableEvidenceSourceClass, infer_market_from_symbol,
};
pub use comparable_evidence_backfill::{
    ComparableEvidenceBackfillConfig, ComparableEvidenceBackfillReport,
    ComparableEvidenceBackfillResult, ComparableEvidenceBackfillRunner,
    ComparableEvidenceBackfillStatus,
};
pub use comparable_evidence_builder::ComparableEvidenceBuilder;
pub use comparable_evidence_quality::{
    ComparableEvidenceQualityReport, ComparableEvidenceQualityStatus,
    build_comparable_evidence_quality_report,
};
pub use complete_comparable_row_builder::{
    CompleteComparableRowBuildRecord, CompleteComparableRowBuildStatus,
    CompleteComparableRowBuilder, CompleteComparableRowBuilderConfig, CompleteComparableRowBundle,
};
pub use complete_row_closure::{
    CompleteRowClosureConfig, CompleteRowClosureRecommendation, CompleteRowClosureReport,
    CompleteRowClosureRunner, CompleteRowClosureStatus,
};
pub use complete_row_closure_bundle::{
    CompleteRowClosureBundle, CompleteRowClosureStorageReport,
    build_complete_row_closure_final_summary, build_complete_row_closure_storage_report,
};
pub use complete_row_closure_v2::{
    CompleteRowClosureV2Config, CompleteRowClosureV2Recommendation, CompleteRowClosureV2Report,
    CompleteRowClosureV2Runner, CompleteRowClosureV2Status,
};
pub use complete_row_closure_v2_bundle::{
    CompleteRowClosureV2Bundle, build_complete_row_closure_v2_storage_report,
    build_complete_row_closure_v2_summary,
};
pub use core_bottleneck_movement::{
    CoreBottleneckMovementKind, CoreBottleneckMovementReport, build_core_bottleneck_movement_report,
};
pub use counterfactual_backfill_plan::{
    CounterfactualBackfillGapKind, CounterfactualBackfillPlan, CounterfactualBackfillPlanItem,
    CounterfactualBackfillSuggestedAction, build_counterfactual_backfill_plan,
};
pub use counterfactual_completion_v2::{
    CounterfactualCompletionRecord, CounterfactualCompletionV2Config,
    CounterfactualCompletionV2RecordStatus, CounterfactualCompletionV2Report,
    CounterfactualCompletionV2Runner, CounterfactualCompletionV2Status,
    load_counterfactual_completion_v2_from_path_or_config,
};
pub use counterfactual_depth_closure::{
    ComparableEvidenceCountSummary, CounterfactualDepthClosureConfig,
    CounterfactualDepthClosureReport, CounterfactualDepthClosureRunner,
    CounterfactualDepthClosureStatus, CounterfactualDepthFinalRecommendation,
};
pub use counterfactual_depth_closure_bundle::CounterfactualDepthClosureBundle;
pub use counterfactual_depth_plan::{
    CounterfactualDepthPlan, CounterfactualDepthPlanItem, CounterfactualGapKind,
    CounterfactualSuggestedBuilder,
};
pub use counterfactual_reference_generator::{
    CounterfactualReferenceGenerator, CounterfactualReferencePolicy,
};
pub use cycle_risk_skeptic::CycleRiskSkeptic;
pub use diversity_aware_sufficiency_v2::{
    DiversityAwareSufficiencyV2Config, DiversityAwareSufficiencyV2Report,
    DiversityAwareSufficiencyV2Runner, DiversityAwareSufficiencyV2Status,
    load_diversity_aware_sufficiency_v2_from_path_or_config,
};
pub use doctrine::{
    DoctrineCheck, DoctrineObservation, DoctrineViolation, check_doctrine,
    doctrine_consistency_score, doctrine_violation_penalty,
};
pub use evaluation::{
    PersonaEvaluationInput, PersonaEvaluationOutput, SurvivalScoreComponents,
    build_persona_evaluation_inputs, calibration_score, composite_survival_score,
    correlation_penalty, drawdown_control_score, evaluate_persona, net_expectancy_after_cost_score,
    overconfidence_penalty, overtrade_penalty, regime_fit_score, risk_efficiency_score,
    silence_value_score,
};
pub use future_window_extension_job::{
    FutureWindowExtensionJob, FutureWindowExtensionJobKind, FutureWindowExtensionJobStatus,
};
pub use future_window_requirements::{
    FutureBar, FutureWindowGapKind, FutureWindowRequirementConfig, FutureWindowRequirementItem,
    FutureWindowRequirementReport, FutureWindowRequirementRunner, FutureWindowRequirementStatus,
    load_descriptor_map_from_paths, load_future_bars_from_csv, load_future_window_inputs,
    load_future_window_requirement_from_path_or_config,
};
pub use future_window_scaleout::{
    FutureWindowScaleOutConfig, FutureWindowScaleOutGroup, FutureWindowScaleOutJobKind,
    FutureWindowScaleOutPlan, FutureWindowScaleOutPlanner,
    load_future_window_scaleout_plan_from_path_or_config,
};
pub use gap_expansion_consistency::{
    GapExpansionConsistencyReport, GapExpansionConsistencyStatus,
    build_gap_expansion_consistency_report,
};
pub use join_repair_plan::{
    JoinRepairAction, JoinRepairActionKind, JoinRepairPlan, JoinRepairPlanStatus,
    build_join_repair_plan,
};
pub use match_key_normalization::{
    MatchKeyNormalizationAggregate, MatchKeyNormalizationOptions, MatchKeyNormalizationReport,
    MatchKeyNormalizationStatus, NormalizedMatchKey, RawMatchKey, SymbolAliasEntry, SymbolAliasMap,
    TimeframeAliasEntry, TimeframeAliasMap, TimestampPolicyEntry, TimestampPolicyKind,
    TimestampPolicyMap, build_match_key_normalization_aggregate, infer_timestamp_policy,
    load_symbol_alias_map, load_timeframe_alias_map, load_timestamp_policy_map,
    normalize_row_match_key, normalized_symbols, reports_by_row_id,
};
pub use minimal_ai_committee_core::*;
pub use momentum_trend_fast::MomentumTrendFast;
pub use multi_row_official_evidence::{
    MultiRowOfficialEvidenceItem, MultiRowOfficialEvidenceSet, MultiRowOfficialEvidenceSetBuilder,
    MultiRowOfficialEvidenceSetConfig, MultiRowOfficialEvidenceStatus,
    load_multi_row_official_evidence_set_from_path_or_config,
};
pub use official_candle_coverage::{
    OfficialCandleCoverageReport, OfficialCandleCoverageRunner, OfficialCandleCoverageStatus,
    load_candle_series_from_paths, materialize_official_candle_series,
};
pub use official_candle_coverage_pack::{
    CandleCsvTimestampSeries, OfficialCandleCoveragePack, OfficialCandleCoveragePackConfig,
    OfficialCandleSeriesDescriptor, OfficialCandleSeriesSourceClass,
    load_candle_csv_timestamp_series, load_pack_from_path_or_config, normalize_symbol,
    normalize_timeframe_label, timeframe_seconds,
};
pub use official_candle_expansion_bundle::{
    CandleExpansionArtifactSize, CandleExpansionStorageReport, OfficialCandleExpansionBundle,
    build_candle_expansion_storage_report, build_expansion_final_summary,
};
pub use official_candle_expansion_plan::{
    OfficialCandleExpansionPlanConfig, build_official_candle_acquisition_plan, load_gap_map,
};
pub use official_candle_expansion_runner::{
    CandleExpansionCounts, OfficialCandleExpansionFinalStatus,
    OfficialCandleExpansionRecommendation, OfficialCandleExpansionReport,
    OfficialCandleExpansionRunner,
};
pub use official_candle_gap_map::{
    OfficialCandleCoverageGapMap, OfficialCandleGapCell, OfficialCandleGapConfig,
    OfficialCandleGapInputs, OfficialCandleGapKind, OfficialCandleGapStatus,
    build_gap_map_from_inputs, load_gap_inputs, load_gap_map_from_path_or_config,
};
pub use official_candle_join_audit::{
    OfficialCandleJoinAuditConfig, OfficialCandleJoinAuditReport, OfficialCandleJoinAuditRunner,
    OfficialCandleJoinAuditStatus, load_join_audit_expansion_reports, load_join_audit_gap_maps,
    load_join_audit_pack, load_join_audit_rows, merge_official_packs,
};
pub use official_candle_lineage::{
    OfficialCandleLineageNode, OfficialCandleLineageReport, OfficialCandleLineageStage,
    OfficialCandleLineageTerminalStatus, OfficialCandleLineageTrace,
    build_official_candle_lineage_report,
};
pub use official_committee_benchmark_bundle::CommitteeOfficialBenchmarkBundle;
pub use official_committee_pack::{
    OfficialCommitteePackSourceKind, OfficialCommitteeScenarioPack,
    OfficialCommitteeScenarioPackBuilder, OfficialCommitteeScenarioPackConfig,
};
pub use official_committee_readiness::{
    OfficialCommitteeEvidenceReadinessReport, OfficialCommitteeEvidenceReadinessStatus,
    build_official_committee_evidence_readiness_report,
};
pub use official_diversity_row_selector::{
    DiversitySelectionReason, OfficialDiversityCandidateRow, OfficialDiversityRowSelector,
    OfficialDiversityRowSelectorConfig, OfficialDiversityRowSelectorReport,
    OfficialDiversityRowSelectorStatus, OfficialDiversitySweepConfig,
    load_official_diversity_row_selector_report_from_path_or_config,
};
pub use official_evidence_diversity_bundle::{
    OfficialEvidenceDiversitySweepBundle, build_official_evidence_diversity_reason_codes,
    build_official_evidence_diversity_summary,
};
pub use official_evidence_diversity_gap::{
    OfficialEvidenceDiversityGapCell, OfficialEvidenceDiversityGapConfig,
    OfficialEvidenceDiversityGapKind, OfficialEvidenceDiversityGapMap,
    OfficialEvidenceDiversityGapRunner, OfficialEvidenceDiversityGapStatus,
    load_official_evidence_diversity_gap_map_from_path_or_config,
};
pub use official_evidence_diversity_sweep::{
    OfficialEvidenceDiversityStorageReport, OfficialEvidenceDiversitySweepConfig,
    OfficialEvidenceDiversitySweepRecommendation, OfficialEvidenceDiversitySweepReport,
    OfficialEvidenceDiversitySweepRunner, OfficialEvidenceDiversitySweepStatus,
};
pub use official_evidence_replication::{
    OfficialEvidenceReplicationConfig, OfficialEvidenceReplicationFinalStatus,
    OfficialEvidenceReplicationRecommendation, OfficialEvidenceReplicationReport,
    OfficialEvidenceReplicationRunner,
};
pub use official_evidence_scaleout::{
    OfficialEvidenceScaleOutConfig, OfficialEvidenceScaleOutRecommendation,
    OfficialEvidenceScaleOutReport, OfficialEvidenceScaleOutRunner, OfficialEvidenceScaleOutStatus,
    OfficialEvidenceScaleOutStorageReport,
};
pub use official_evidence_scaleout_bundle::OfficialEvidenceScaleOutBundle;
pub use official_evidence_sufficiency_v2::{
    OfficialEvidenceSufficiencyV2Config, OfficialEvidenceSufficiencyV2Counts,
    OfficialEvidenceSufficiencyV2Recommendation, OfficialEvidenceSufficiencyV2Report,
    OfficialEvidenceSufficiencyV2Runner, OfficialEvidenceSufficiencyV2Status,
    load_official_evidence_sufficiency_v2_from_path_or_config,
};
pub use official_future_window_extension::{
    FutureWindowExtensionPlan, OfficialFutureWindowExtensionConfig,
    build_official_future_window_extension_plan,
};
pub use official_ready_match_closure::{
    OfficialReadyMatchClosureConfig, OfficialReadyMatchClosureRecommendation,
    OfficialReadyMatchClosureReport, OfficialReadyMatchClosureRunner,
    OfficialReadyMatchClosureStatus,
};
pub use official_ready_match_closure_bundle::{
    OfficialReadyMatchClosureBundle, build_official_ready_match_final_summary,
    build_official_ready_match_storage_report, closure_bundle_with_storage,
};
pub use official_ready_row_inventory::{
    OfficialReadyRowCompletenessStatus, OfficialReadyRowInventoryConfig,
    OfficialReadyRowInventoryItem, OfficialReadyRowInventoryReport,
    OfficialReadyRowInventoryRunner, OfficialReadyRowInventoryStatus,
};
pub use official_reference_replication::{
    OfficialReferenceReplicationArtifacts, OfficialReferenceReplicationReport,
    OfficialReferenceReplicationRunner, OfficialReferenceReplicationStatus,
    build_linked_pack_from_reference_pack,
};
pub use official_replication_bundle::OfficialEvidenceReplicationBundle;
pub use official_replication_inventory::{
    OfficialReplicationArtifactDescriptor, OfficialReplicationArtifactInventory,
    OfficialReplicationArtifactKind, OfficialReplicationInventoryResolver,
};
pub use official_replication_operator_actions::{
    OfficialReplicationActionPriority, OfficialReplicationOperatorAction,
    OfficialReplicationOperatorActionPlan, OfficialReplicationOperatorActionPlanner,
};
pub use official_row_injection::{
    OfficialEvidenceBoundary, OfficialRowInjectionPolicy, OfficialRowInjectionResult,
    OfficialRowInjector, OfficialSkippedRow, classify_row_boundary,
};
pub use official_sufficiency_replication::{
    OfficialSufficiencyReplicationBuilder, OfficialSufficiencyReplicationReport,
    OfficialSufficiencyReplicationStatus,
};
pub use operational_audit_timeline::{
    OperationalAuditEvent, OperationalAuditTimeline, OperationalEventKind,
};
pub use outcome_diversity_audit::{
    OutcomeDiversityAuditConfig, OutcomeDiversityAuditReport, OutcomeDiversityAuditRunner,
    OutcomeDiversityStatus, load_outcome_diversity_audit_from_path_or_config,
};
pub use outcome_linkage_v3::{
    OutcomeLinkageV3Config, OutcomeLinkageV3Record, OutcomeLinkageV3RecordStatus,
    OutcomeLinkageV3Report, OutcomeLinkageV3Runner, OutcomeLinkageV3Status,
    load_outcome_linkage_v3_from_path_or_config,
};
pub use outcome_reference_backfill::{
    OutcomeBackfillGapKind, OutcomeBackfillSuggestedAction, OutcomeReferenceBackfillPlan,
    OutcomeReferenceBackfillPlanItem, build_outcome_reference_backfill_plan,
};
pub use paper_position_lifecycle::{
    PaperPositionLifecycleEvent, PaperPositionLifecycleReport,
    build_paper_position_lifecycle_report, open_paper_position,
};
pub use persona::Persona;
pub use persona_card::{
    AgentAttributionSummary, AgentConsistencyStatus, AgentCrossSourceConsistencyRow,
    AgentCrossSourceConsistencyTable, AgentDoctrine, AgentFeedback, AgentFeedbackBuildError,
    AgentFeedbackOutcomeKind, AgentId, AgentKind, AgentLearningSummary, AgentMemorySummary,
    AgentPerformanceByQualityRow, AgentPerformanceByQualityTable, AgentPerformanceRow,
    AgentPerformanceTable, AgentProposal, AgentStateJournal, AgentStateJournalError,
    AgentStateSnapshot, AgentStatus, AgentVersion, AgentVoiceState, AggregateBaselineComparison,
    AggregateBaselineOverallStatus, AggregatePredictionQualitySummary, BaselinePerformanceMetrics,
    BaselineStrategyKind, BatchLearningSummary, BatchOwnerLearningReport, BatchReplayConfig,
    BatchReplayError, BatchReplayInput, BatchReplayMode, BatchReplayResult, BatchReplaySource,
    BatchReplaySourceResult, CadenceTolerance, CanonicalAgentState, ChairReviewSummary,
    ChairRewardPenalty, ChairTierAction, CooldownTickMode, CrossSourceConsistencyReport,
    DataQualityBucket, EvaluationProfile, EvidenceAggregationMethod, EvidenceTriageDimensionStatus,
    EvidenceTriageSummary, ExpectedCadence, FeedbackContext, FeedbackCycleResult,
    HistoricalEvidencePack, HistoricalEvidencePackConfig, HistoricalEvidencePackError,
    HistoricalEvidencePackEvaluationConfig, HistoricalEvidencePackEvaluationResult,
    HistoricalEvidencePackManifest, HistoricalEvidenceSource,
    HistoricalEvidenceSourceEvaluationResult, HistoricalEvidenceSourceKind,
    HistoricalEvidenceSourceSpec, HistoricalOhlcvRow, HistoricalOwnerReportError,
    HistoricalReplayAdapter, HistoricalReplayConfig, HistoricalReplayDataset,
    HistoricalReplayError, Horizon, ImmutableDoctrine, LearningChainSummary, LocalCsvSourceResult,
    LocalDataQualitySummary, LocalDataSourceError, LocalDataSourceKind, LocalDataSourceProfile,
    LocalDataSourceRegistry, LocalSymbolPolicy, LocalTimestampUnit, ManualAdjustedClosePolicy,
    ManualHistoricalDailyDataset, ManualHistoricalDailyImportConfig,
    ManualHistoricalDailyImportError, ManualHistoricalDailyRow, ManualHistoricalDateRange,
    MarketEvidenceResult, MarketTriageResult, MultiSymbolProofGateReport, MutablePolicy,
    OwnerActionItem, OwnerAdvisorySummary, OwnerAgentLearningView,
    OwnerEvidenceLocalTrialRunResult, OwnerEvidenceManifestStatus,
    OwnerEvidenceReportEmissionConfig, OwnerEvidenceReportEmissionResult,
    OwnerEvidenceTriageReport, OwnerEvidenceTrialConfig, OwnerEvidenceTrialResult,
    OwnerEvidenceTrialStatus, OwnerLearningReport, OwnerLearningReportError, OwnerReviewCommand,
    OwnerReviewResponse, PaperFillEvidence, PaperLearningChainConfig, PaperLearningChainError,
    PaperLearningChainInput, PaperLearningChainResult, PaperLearningEpisode,
    PaperLearningEpisodeResult, PaperLearningLoopConfig, PaperLearningLoopError,
    PaperLearningLoopInput, PaperLearningLoopReport, PaperLearningLoopResult, PaperOutcomeContext,
    PaperOutcomeKind, PaperReplayConfig, PaperReplayError, PaperReplayInput, PaperReplayResult,
    PersonaCard, PredictionQualityMetrics, PredictionQualitySample, ProofGateComparison,
    ProofGateReport, ProofGateStatus, QualityReplayPolicy, ReplayAttributionSummary,
    RiskReviewSummary, SandboxPromotionCandidate, SandboxPromotionStatus, SandboxReviewSummary,
    SourceConsistencyDiagnostics, SourceOrderPolicy, SourcePerformanceRow, SourcePerformanceTable,
    SourceQualityScore, SourceQualityThresholds, VoiceAdaptationComparison,
    VoiceAdaptationValidity, VoiceAdaptationValidityStatus, VoiceConfig,
    WalkForwardCommitteeConfig, WalkForwardConfig, WalkForwardEvaluationError,
    WalkForwardEvaluationInput, WalkForwardEvaluationResult, WalkForwardSplit,
    WalkForwardWindowResult, active_persona_cards, apply_cooldown_tick_after_episode,
    apply_feedback_to_memory_summary, apply_paper_feedback_cycle,
    build_agent_feedback_from_paper_outcome, build_batch_owner_learning_report,
    build_multi_symbol_proof_gate_report, build_owner_learning_report,
    build_owner_learning_report_from_historical_replay,
    build_owner_learning_report_from_local_csv_source, build_proof_gate_report,
    build_sandbox_promotion_candidate, build_walk_forward_splits, canonical_agent_state_from_card,
    canonical_current_agent_states, classify_agent_status, clear_expired_cooldown,
    compute_chair_reward_penalty, compute_prediction_quality_metrics, cycle_risk_skeptic_card,
    detect_doctrine_violation, discover_owner_evidence_manifest_path,
    emit_owner_evidence_triage_report_local, evaluate_historical_evidence_pack,
    future_agent_placeholder_state, handle_owner_review_command, horizon_from_bars,
    is_agent_available_for_live_decision, load_historical_evidence_pack_from_manifest,
    momentum_trend_fast_card, normalize_dataset_to_candle_series,
    parse_historical_evidence_pack_manifest_json, parse_local_csv_with_profile,
    parse_manual_historical_daily_csv, persona_card_by_id, render_batch_owner_learning_report_text,
    render_multi_symbol_proof_gate_report_text, render_owner_evidence_triage_report_text,
    render_owner_learning_report_json_like, render_owner_learning_report_markdown,
    render_owner_learning_report_text, render_proof_gate_report_text,
    run_3_agent_paper_learning_chain, run_3_agent_paper_learning_loop, run_3_agent_paper_replay,
    run_local_dataset_batch_replay, run_owner_historical_evidence_trial,
    run_owner_historical_evidence_trial_from_local_candidates, run_walk_forward_evaluation,
    to_daily_candle_series, update_agent_voice_state, validate_historical_evidence_pack,
    validate_manual_historical_daily_dataset, value_quality_filter_card,
};
pub use persona_card_lite::{
    DoctrineRule, PersonaCardLite, PersonaGroup, PersonaHorizon, PersonaMutablePolicy, PersonaRole,
    active_persona_cards_lite, all_persona_cards_lite, cycle_regime_guard_card,
    defensive_value_risk_card, persona_card_lite_by_id, trend_breakout_fast_card,
};
pub use persona_conflict_matrix::{
    PersonaConflictMatrix, PersonaConflictPair, PersonaConflictStatus,
    build_persona_conflict_matrix,
};
pub use persona_operational_status::{
    PersonaOperationalStatus, PersonaOperationalView, TrinityOperationalStatusReport,
    build_status_report_from_votes, idle_trinity_operational_status_report,
};
pub use persona_scorer::{PersonaScorer, PersonaScoringInput};
pub use persona_vote::{PersonaStance, PersonaVote};
pub use reference_pack_quality::{
    ReferencePackQualityReport, ReferencePackQualityStatus, build_reference_pack_quality_report,
};
pub use risk_bridge_diagnostics::{
    RiskBridgeDiagnosticStatus, RiskBridgeDiagnosticsReport, build_risk_bridge_diagnostics,
};
pub use risk_calibration::{
    RiskCalibrationArea, RiskCalibrationDirection, RiskCalibrationRecommendation,
    RiskCalibrationReport, RiskCalibrationSafetyImpact, RiskCalibrationSuggestion,
    build_risk_calibration_report,
};
pub use row_candle_candidate::{
    RowCandleCandidate, RowCandleCandidateBucket, RowCandleCandidateOptions,
    RowCandleCandidateReport, RowCandleCandidateReportStatus, RowCandleCandidateStatus,
    buckets_by_row_id, build_row_candle_candidate_report,
};
pub use scenario_materialization_closure::{
    ScenarioMaterializationWeakClosureReport, ScenarioMaterializationWeakClosureStatus,
    build_scenario_materialization_weak_closure_report,
};
pub use scenario_materialization_v3::{
    ScenarioMaterializationV3Config, ScenarioMaterializationV3Level,
    ScenarioMaterializationV3Record, ScenarioMaterializationV3Report,
    ScenarioMaterializationV3Runner, ScenarioMaterializationV3Status,
};
pub use scoring::{
    HypotheticalTradeOutcome, SurvivalComponents, silence_value, survival_score,
    update_voice_power, violation_outcome,
};
pub use shadow::ShadowVoteRecord;
pub use six_persona_readiness::{
    SixPersonaDesignReadinessConfig, SixPersonaDesignReadinessReport,
    SixPersonaDesignRecommendation, evaluate_six_persona_design_readiness,
};
pub use sprint98_committee_owned_core::{
    AICommitteeMemberAnalysisLoop, AICommitteeMemberAnalysisLoopStatus,
    AICommitteeMemberCoreContract, AICommitteeMemberCoreFamily, AICommitteeMemberCoreStatus,
    AICommitteeMemberLearningMode, AICommitteeMemberLearningPolicy,
    AICommitteeMemberLearningPolicyStatus, AICommitteeMemberProposal,
    AICommitteeMemberProposalStatus, AICommitteeMemberRole, AICommitteeMemberSpec,
    AICommitteeMemberStatus, ChairmanAiGovernancePolicy, ChairmanAiGovernancePolicyStatus,
    ChairmanRuleAuditTrailCompletenessReport, ChairmanRuleAuditTrailCompletenessStatus,
    ChairmanRuleAuthority, ChairmanRuleProposal, ChairmanRuleProposalKind,
    ChairmanRuleProposalRiskAuditV2, ChairmanRuleProposalRiskAuditV2Status,
    ChairmanRuleProposalStatus, ChairmanRulebookApprovalGate, ChairmanRulebookApprovalStatus,
    ChairmanRulebookQualityReport, ChairmanRulebookQualityStatus, ChairmanRulebookSafetyRepairPlan,
    ChairmanRulebookSafetyRepairPlanStatus, ChairmanRulebookV2Draft, ChairmanRulebookV2DraftStatus,
    ChairmanRulebookVersion, ChairmanRulebookVersionStatus, ChairmanStyleGovernancePolicyV2,
    ChairmanUnsafeRuleClosureReport, ChairmanUnsafeRuleClosureStatus, ChairmanUnsafeRuleItem,
    CommitteeConsensusState, CommitteeDebateQualityReport, CommitteeDebateQualityStatus,
    CommitteeDebateSession, CommitteeDebateSessionStatus, CommitteeDebateStance,
    CommitteeDebateTrigger, CommitteeDebateTriggerReason, CommitteeDebateTriggerStatus,
    CommitteeMemberDebateTurn, CommitteeMemberDebateTurnStatus,
    CommitteeMemberProposalQualityReport, CommitteeMemberProposalQualityStatus,
    CommitteeOwnedAiCoreArchitecture, CommitteeOwnedAiCoreArchitectureStatus,
    CommitteeOwnedArchitectureRegressionGuard, CommitteeOwnedArchitectureRegressionStatus,
    CommitteeOwnedCoreRegistry, CommitteeOwnedCoreRegistryStatus, CommitteePaperLoopDryRunPlan,
    CommitteePaperLoopDryRunStatus, CommitteePaperLoopDryRunStep, CommitteePaperReadinessGate,
    CommitteePaperReadinessGateStatus, CommitteeProposalAction, CommitteeQualityHardeningConfig,
    CommitteeQualityWarningClosureConfig, CommitteeQualityWarningClosureRunner,
    CommitteeRosterBalanceReport, CommitteeRosterBalanceStatus, CommitteeRosterLifecycle,
    CommitteeRosterLifecycleStatus, ConfidenceWeightPolicyStatus,
    ControlTowerAiCommitteeClosurePanel, ControlTowerAiCommitteePanel,
    ControlTowerAiCommitteePanelStatus, ControlTowerAiCommitteeQualityPanel,
    ControlTowerAiCommitteeQualityRow, ControlTowerAiCommitteeRow,
    ControlTowerInvestorArchetypeCandidateRow, ControlTowerInvestorArchetypeGroupRow,
    ControlTowerInvestorArchetypePanel, CryptoGroupStatus, CryptoMemberGroup,
    DebateDissentCoverageReport, DebateDissentCoverageStatus, DebateEvidenceGap,
    DebateEvidenceGapKind, DebateEvidenceGapPlan, DebateEvidenceGapPlanStatus,
    DebateEvidenceSufficiencyReport, DebateEvidenceSufficiencyStatus,
    DebateMemberParticipationBalanceReport, DebateMemberParticipationBalanceStatus,
    DebateNeedsMoreEvidenceClosureReport, DebateNeedsMoreEvidenceClosureStatus,
    DoNotLearnBlockedItemKind, EighteenInvestorCandidateRegistry,
    EighteenInvestorCandidateRegistryStatus, EighteenInvestorCommitteeRosterPlan,
    EighteenInvestorCommitteeRosterPlanStatus, EighteenMemberActivationGate,
    EighteenMemberActivationGateStatus, EntryTimingConditionCompletenessReport,
    EntryTimingConditionCompletenessStatus, EntryTimingProposal, EntryTimingProposalQualityReport,
    EntryTimingProposalQualityStatus, EntryTimingProposalStatus, EntryTimingWindow,
    InvestorArchetypeCandidate, InvestorArchetypeCandidateStatus, InvestorArchetypeIngestionConfig,
    InvestorArchetypeIngestionReport, InvestorArchetypeIngestionStatus,
    InvestorArchetypeSafetyNormalizationReport, InvestorArchetypeSafetyNormalizationStatus,
    InvestorArchetypeSourceCategory, InvestorArchetypeSourceConfidenceEntry,
    InvestorArchetypeSourceConfidenceReport, InvestorArchetypeSourceConfidenceStatus,
    InvestorAssetScope, InvestorConfidenceGrade, InvestorImpersonationRiskReport,
    InvestorImpersonationRiskRow, InvestorImpersonationRiskStatus,
    InvestorPrivateLifeMythFilterReport, InvestorPrivateLifeMythFilterStatus,
    InvestorStyleArchetype, InvestorStyleArchetypeKind, InvestorStyleBlindspotReport,
    InvestorStyleBlindspotStatus, InvestorStyleDoNotLearnGuard, InvestorStyleDoNotLearnGuardStatus,
    InvestorStyleFeatureCardStatus, InvestorStyleFeatureVectorCard, InvestorStyleGroupKind,
    InvestorStyleMemberRegistry, InvestorStyleRegistryStatus, InvestorStyleStatus,
    InvestorTimeHorizon, InvestorUnverifiedClaimFilterReport, InvestorUnverifiedClaimFilterStatus,
    LearningDataCardsStatus, LongTermEquityGroupStatus, LongTermEquityMemberGroup,
    MarketContextForCommittee, MarketContextForCommitteeStatus, MemberEvidenceRequirementPolicy,
    MemberFeatureScopeMappingReport, MemberLearningDataCard, MemberLearningDataCardReport,
    MemberOverfitRiskReport, MemberOverfitRiskStatus, MemberPromotionDemotionAction,
    MemberPromotionDemotionDecision, MemberPromotionDemotionDecisionStatus,
    MemberScorecardCalibrationReport, MemberScorecardCalibrationStatus,
    MemberStyleConfidenceWeightPolicy, MemberStyleDriftReport, MemberStyleDriftStatus,
    MultiAxisMemberScorecard, MultiAxisMemberScorecardStatus, MultiExpertCommitteeTopology,
    MultiExpertTopologyStatus, OverfitWarningClosureReport, OverfitWarningClosureStatus,
    PaperDecisionNeedMoreEvidenceClosureReport, PaperDecisionNeedMoreEvidenceClosureStatus,
    PaperDecisionReplayWarningClosureReport, PaperDecisionReplayWarningClosureStatus,
    PaperDecisionTraceCompletenessReport, PaperDecisionTraceCompletenessStatus,
    PaperOnlyCommitteeDecisionKind, PaperOnlyCommitteeDecisionRecord,
    PaperOnlyDecisionReplayReport, PaperOnlyDecisionReplayStatus, PaperOnlyRosterExpansionGate,
    PaperOnlyRosterExpansionGateStatus, PreservedHabitKind, PromotionAxis,
    PromotionDemotionCalibrationReport, PromotionDemotionCalibrationStatus,
    PromotionDemotionPolicy, PromotionDemotionPolicyStatus, PromotionDemotionPolicyV2For18Styles,
    PromotionDemotionStabilityReport, PromotionDemotionStabilityStatus, PromotionPolicyV2Status,
    ProposalEvidenceCompletenessReport, ProposalEvidenceCompletenessStatus,
    ProposalQualityWarningClosureReport, ProposalQualityWarningClosureStatus,
    ProposalRiskFieldCompletenessReport, ProposalRiskFieldCompletenessStatus, RegimeRouteEntry,
    RegimeRoutingPolicy, RegimeRoutingStatus, RiskGovernorDebateHandoffReport,
    RiskGovernorDebateHandoffStatus, RiskGovernorFinalVetoTraceReport,
    RiskGovernorFinalVetoTraceStatus, RiskGovernorHandoffWarningClosureReport,
    RiskGovernorHandoffWarningClosureStatus, RosterBalanceWarningClosureReport,
    RosterBalanceWarningClosureStatus, RuleAdaptationAudit, RuleAdaptationAuditStatus,
    RulebookDiffRiskClosureReport, RulebookDiffRiskClosureStatus, RulebookRepairAction,
    RulebookVersionDiffReport, RulebookVersionDiffStatus, SafetyCoveragePreservationReportV14,
    SafetyCoveragePreservationReportV14Status, SafetyCoveragePreservationReportV15,
    SafetyCoveragePreservationReportV15Status, SafetyCoveragePreservationReportV16,
    SafetyCoveragePreservationReportV16Status, SafetyCoveragePreservationReportV17,
    SafetyCoveragePreservationReportV17Status, ScorecardCalibrationWarningClosureReport,
    ScorecardCalibrationWarningClosureStatus, ScorecardEvidenceDepthReport,
    ScorecardEvidenceDepthStatus, ShortTermSwingGroupStatus, ShortTermSwingMemberGroup,
    Sprint97SummaryImport, Sprint98CommitteeOwnedCoreBundle, Sprint98CommitteeOwnedCoreConfig,
    Sprint98CommitteeOwnedCoreRunner, Sprint98CommitteeOwnedCoreStorageReport,
    Sprint99CommitteeQualityHardeningBundle, Sprint99CommitteeQualityHardeningRunner,
    Sprint99CommitteeQualityHardeningStorageReport, Sprint100CommitteeClosureBundle,
    Sprint100CommitteeClosureRunner, Sprint100CommitteeClosureStorageReport,
    Sprint101InvestorArchetypeIngestionBundle, Sprint101InvestorArchetypeIngestionRunner,
    Sprint101InvestorArchetypeIngestionStorageReport, StyleConflictEntry, StyleConflictMatrix,
    StyleConflictMatrixStatus, StyleConflictResolutionPolicy, StyleGroupTaxonomyReport,
    StyleGroupTaxonomyStatus, WorkspaceAcceptanceAttemptV16, WorkspaceAcceptanceAttemptV17,
    WorkspaceAcceptanceTruthClosurePlan, WorkspaceAcceptanceTruthClosurePlanV2,
    WorkspaceAcceptanceTruthClosureStatus, WorkspaceAcceptanceTruthImport,
};
pub use sprint102_paper_rotation::{
    ArchetypeGroupRotationPlan, ArchetypeGroupRotationPlanStatus, ArchetypeMemberSelectionReport,
    ArchetypeMemberSelectionStatus, ArthurHayesEvidenceHardeningReport,
    ArthurHayesEvidenceHardeningStatus, ChairmanDryRunRecommendation,
    ChairmanStyleWeightAdjustmentAudit, ChairmanStyleWeightAdjustmentAuditStatus,
    ChairmanSynthesisDryRunReport, ChairmanSynthesisDryRunStatus, ControlTowerPaperRotationPanel,
    CrossGroupConflictKind, CrossGroupConflictResolution, CrossGroupDebateConflictReport,
    CrossGroupDebateConflictStatus, EighteenArchetypeActivationSafetyReport,
    EighteenArchetypeActivationSafetyStatus, EighteenArchetypePaperRotationConfig,
    GroupDebateSessionReport, GroupDebateSessionStatus, GroupDebateTriggerKind,
    GroupDebateTriggerReport, GroupDebateTriggerStatus, LarryWilliamsEvidenceHardeningReport,
    LarryWilliamsEvidenceHardeningStatus, LowerConfidenceEvidenceHardeningPlan,
    LowerConfidenceEvidenceHardeningPlanStatus, LowerConfidenceEvidenceHardeningReport,
    LowerConfidenceEvidenceHardeningReportStatus, LowerConfidenceHardeningAction,
    MultiExpertRotationCoverageReport, MultiExpertRotationCoverageStatus,
    NoTradeRiskDeniedCommitteeTrace, NoTradeRiskDeniedCommitteeTraceStatus,
    PaperDecisionReplayV2Report, PaperDecisionReplayV2Status, PaperDecisionTraceV2,
    PaperDecisionTraceV2Status, PaperEntryTimingProposalRecord, PaperOnlyEntryTimingProposalRun,
    PaperOnlyEntryTimingProposalRunStatus, PaperOnlyMemberProposalRun,
    PaperOnlyMemberProposalRunStatus, PaperOnlyProposalRecord, PaperRosterExpansionUsageReport,
    PaperRosterExpansionUsageStatus, PaperRotationMarketContext, PaperRotationMarketContextSet,
    PaperRotationMarketContextSetStatus, PaperRotationMarketCoverage, PaperRotationRegimeCoverage,
    PaperRotationScenario, PaperRotationScenarioPack, PaperRotationScenarioPackStatus,
    ProposalOutcomeExpectationTrace, ProposalOutcomeExpectationTraceStatus,
    RegimeRoutedCommitteeDryRunReport, RegimeRoutedCommitteeDryRunStatus,
    RiskGovernorPaperHandoffReport, RiskGovernorPaperHandoffStatus, RiskGovernorPaperVetoResult,
    SafetyCoveragePreservationReportV18, Sprint102PaperRotationBundle,
    Sprint102PaperRotationRunner, Sprint102PaperRotationStorageReport, WeakSourceCandidateReview,
    WeakSourceCandidateReviewReport, WeakSourceCandidateReviewReportStatus,
    WonyottiEvidenceHardeningReport, WonyottiEvidenceHardeningStatus,
    WorkspaceAcceptanceAttemptV18, WorkspaceAcceptanceTruthClosurePlanV3,
};
pub use sprint103_paper_rotation_closure::{
    ArthurHayesWarningClosureReport, ChairmanSynthesisWarningClosureReport,
    CommitteeDecisionStabilityReport, ControlTowerPaperRotationClosurePanel,
    DebateSessionWarningClosureReport, EntryTimingRunWarningClosureReport,
    ExpectationTraceWarningClosureReport, LarryWilliamsWarningClosureReport,
    LowerConfidenceEvidenceClosureReport, MemberSelectionWarningClosureReport,
    MultiExpertCoverageWarningClosureReport, MultiScenarioPaperReplayPack,
    MultiScenarioPaperReplayReport, NeedMoreEvidenceItem, NeedMoreEvidenceResolutionPlan,
    NoTradeRiskDeniedTraceWarningClosureReport, PaperNeedMoreEvidenceJustificationReport,
    PaperNoTradeJustificationReport, PaperReplayWarningClosureReportV2,
    PaperRosterUsageWarningClosureReport, PaperRotationReadinessGateV2,
    PaperRotationWarningClosureConfig, PaperRotationWarningClosureReport,
    PaperRotationWarningClosureRunner, PaperTraceWarningClosureReport,
    ProposalRunWarningClosureReport, RegimeRoutingWarningClosureReport,
    RiskGovernorHandoffWarningClosureReportV2, RiskGovernorNoTradeReasonAudit,
    RotationPlanWarningClosureReport, SafetyCoveragePreservationReportV19,
    SaylorTreasuryWatchlistUsageAudit, ScenarioOutcomeExpectationMatrix,
    ScenarioOutcomeExpectationRow, Sprint103PaperRotationClosureBundle,
    Sprint103PaperRotationClosureRunner, Sprint103PaperRotationClosureStorageReport,
    StyleWeightAuditWarningClosureReport, WatchlistMemberUsagePolicy, WonyottiWarningClosureReport,
    WorkspaceAcceptanceAttemptV19, WorkspaceAcceptanceTruthClosurePlanV4,
};
pub use sprint104_dual_agent_paper_lifecycle::*;
pub use sprint105_verification_patch_closure::*;
pub use sprint106_workspace_acceptance_recovery::*;
pub use sprint107_safe_consolidation_patch::*;
pub use sprint108_safe_consolidation_patch_v2::*;
pub use sprint109_safe_consolidation_patch_v3::*;
pub use sprint110_safe_consolidation_patch_v4::*;
pub use sprint111_workspace_timeout_root_cause::*;
pub use sprint112_workspace_diagnostic_pilot::*;
pub use sprint113_real_workspace_observation::*;
pub use sprint114_mixed_family_isolation::*;
pub use sprint115_consolidation_governance::*;
pub use sprint116_workspace_timeout_track::*;
pub use sprint117_deferred_real_observation::*;
pub use sprint118_timeout_reduction_queue::*;
pub use sufficiency_closure::{
    SufficiencyClosureConfig, SufficiencyClosureCounts, SufficiencyClosureFinalRecommendation,
    SufficiencyClosureReport, SufficiencyClosureRunner, SufficiencyClosureStatus,
};
pub use tier::{TierAction, tier_from_voice_power};
pub use timeframe_alignment::{
    TimeframeAlignmentInput, TimeframeAlignmentOverallStatus, TimeframeAlignmentRecord,
    TimeframeAlignmentReport, TimeframeAlignmentStatus, build_timeframe_alignment_report,
};
pub use timestamp_alignment_v2::{
    TimestampAlignmentV2Input, TimestampAlignmentV2Options, TimestampAlignmentV2OverallStatus,
    TimestampAlignmentV2Record, TimestampAlignmentV2Report, TimestampAlignmentV2Status,
    build_timestamp_alignment_v2_report, count_file_bytes,
};
pub use trinity_operational_loop::{
    TrinityCommitteeOperationalLoopConfig, TrinityOperationalLoopBundle,
    TrinityOperationalLoopFinalStatus, TrinityOperationalLoopRecommendation,
    TrinityOperationalLoopReport, TrinityOperationalLoopRunner, load_paper_positions_from_paths,
    run_candidate_generation_only,
};
pub use trinity_personas::{
    CycleRegimeGuardScorer, DefensiveValueRiskScorer, TrendBreakoutFastScorer,
    active_trinity_scorers,
};
pub use triple_barrier_reference_builder::{
    TripleBarrierReferenceBuildResult, TripleBarrierReferenceBuilder, TripleBarrierReferenceConfig,
    TripleBarrierReferenceSource, TripleBarrierTieBreakPolicy,
};
pub use value_quality_filter::ValueQualityFilter;

use crate::core::{InvestorVote, MarketSnapshot, SignalOutput};

pub fn default_league_votes(market: &MarketSnapshot, signal: &SignalOutput) -> Vec<InvestorVote> {
    vec![
        MomentumTrendFast::default().vote(market, signal),
        ValueQualityFilter::default().vote(market, signal),
        CycleRiskSkeptic::default().vote(market, signal),
    ]
}
