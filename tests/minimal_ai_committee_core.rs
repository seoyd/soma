use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use soma_zero::league::minimal_ai_committee_core as core;
use soma_zero::league::minimal_ai_committee_core::{
    AICommitteeMember, AICommitteeMemberStatus, AIRuntimeMode,
    AdapterContractLockAcceptanceGateStatus, AdapterContractLockRunConfig,
    AdapterContractLockRunResult, AdapterContractLockStatus,
    AdapterContractRegressionHarnessStatus, AdapterContractValidationStatus,
    AdapterGoldenBaselineBootstrapConfig, AdapterGoldenBaselineBootstrapStatus,
    AdapterGoldenBaselineDiffStatus, AdapterGoldenBaselinePolicy,
    AdapterGoldenBaselineValidationStatus, AdapterGoldenSnapshotBaselineFile,
    AdapterInputFeatureGroup, AdapterOutputHeadGroup, AdapterOutputValueGuardStatus,
    AdapterOutputValuePolicy, AdapterRuntimeEntryNextAllowedStep,
    AdapterRuntimeEntryReadinessStatus, AdapterSchemaDriftStatus, AdapterSchemaValidationStatus,
    AdapterShapeValidationStatus, AdapterSkeletonDryRunConfig, AdapterSkeletonDryRunStatus,
    AdapterToMicroKernelBridgeValidationStatus, AdapterToMicroKernelMappingStatus,
    AdditionalPaperEvidenceSeed, AiMemberBrain, AiMemberCoreRegistry,
    AmbiguousEvidenceResolutionPolicy, ApprovedEvidenceApplyConfig, ApprovedEvidenceApplyMode,
    ApprovedEvidenceApplyStatus, ArchetypeRiskBias, ArchetypeStyleCardRegistry, ArchetypeStyleTag,
    AutoApprovalEndToEndRunConfig, AutoApprovalSuccessExactLinkType, AutonomousPaperCycleMode,
    BacktestCostModel, BacktestEntryPriceSource, BacktestExitPriceSource, BacktestLabelContract,
    BacktestLabelContractStatus, BacktestLabelHorizon, BacktestLeakageGuard,
    BatchCommitteeCycleInput, BatchCommitteeCycleResult, BatchCommitteeCycleWithStateInput,
    BatchToMicroKernelBridgeRunConfig, BatchToMicroKernelBridgeStatus, ChairmanFinalAction,
    CollectedNewsItem, CommitteeStateExportConfig, CommitteeStateExportInput,
    CommitteeStateSnapshotSource, CoreAwareMemberBrainAdapter, CoreRuntimeStatus,
    DailyBriefStorageUpdateInput, DataRouterInput, DecisionTimelinePhase, DeterministicMockBrain,
    DummyPredictionPolicy, EnrichedEvidencePromotionRunConfig,
    EvidenceBackfillAndPromotionRunConfig, EvidencePreference, GatedDeltaNetMemoryConfigV0,
    GatedDeltaNetMemoryStateV0, HeadShapeStatus, IndependentMemberRole, InvestmentEventQueue,
    InvestorArchetypeStyleCard, LabelConfidenceUpgradePolicy, LabelConfidenceUpgradeStatus,
    LabelEvidenceItems, LabelEvidenceRecord, LabelEvidenceValidationStatus, LabelPromotionStatus,
    LabelQualityStatus, LossMetricReadinessStatus, Mamba3GatedDeltaNetCoreSpec,
    Mamba3TemporalCellConfigV0, Mamba3TemporalCellStateV0, MarketScope, MemberActivationPolicy,
    MemberCalibrationEvalStatus, MemberCalibrationEvalSuggestedAdjustment, MemberCoreFamily,
    MemberExperienceCommitteeContext, MemberExperienceInputContext,
    MemberExperienceOpinionSnapshot, MemberExperienceOutcome, MemberExperienceRecord,
    MemberExperienceStore, MemberInputPacket, MemberLearningDataContractSummaryStatus,
    MemberLearningDataContractValidationStatus, MemberLearningLabel, MemberLearningSignal,
    MemberMemoryState, MemberOpinion, MemberResearchTaskType, MemberScoreUpdateReason,
    MemberSelectionSkipReason, MemberStance, MemberStateStore, MemberStyleStatus, MemoryCoreKind,
    MemoryCountBucket, MicroKernelBucketDimStatus, MicroKernelLabModePolicy,
    MicroKernelNumericScale, MicroKernelRuntimeSafetyStatus, MicroKernelSequenceAssemblyStatus,
    MicroKernelSequencePadPolicy, MicroKernelSequenceTruncatePolicy, MicroKernelWarningKind,
    MicroKernelWarningNormalizationStatus, MinimalAiCommitteeCycleConfig, MockSafeHttpTransport,
    NewsCacheStore, NewsCollectionConfig, NewsCollectionSourceMode, NewsProviderConfig,
    NewsProviderKind, NewsProviderRunMode, NewsProviderRunStatus, NewsProviderStatus,
    NewsProviderTrustLevel, NextActionType, NextOwnerActionType,
    NoPersistenceTrainingSimulationConfig, NoPersistenceTrainingSimulationGateStatus,
    NoPersistenceTrainingSimulationStatus, NoWeightUpdateGuardProofStatus,
    OfflineMemberBrainAdapter, OfflineMemberOpinionFixture, OfflineMemberOutputBatch,
    OfflineTrainerContractStatus, OfflineTrainerDesignStatusLevel, OfflineTrainerDryRunConfigV2,
    OfflineTrainerDryRunStatus, OfflineTrainerReadinessNextStep, OfflineTrainingDesignGateStatus,
    OfflineTrainingDesignStatus, OfflineTrainingMetric, OfflineTrainingMetricContract,
    OfflineTrainingReadinessStatus, OwnerActionAfterApplyFilePolicy, OwnerActionApplyMode,
    OwnerActionComposerConfig, OwnerActionConsumptionConfig, OwnerActionDuplicatePolicy,
    OwnerActionFile, OwnerActionProcessedStatus, OwnerAttentionAction,
    OwnerAttentionActionSafetyStatus, OwnerAttentionActionType, OwnerAttentionInbox,
    OwnerAttentionInboxStatus, OwnerAttentionPriority, OwnerAttentionQueue,
    OwnerAttentionTriageInput, OwnerAttentionType, OwnerConfirmationPolicy,
    OwnerConsoleTerminalOptions, OwnerDailyBriefStore, OwnerFeedbackOutcome, OwnerFeedbackType,
    OwnerIntentPolicy, OwnerIntentPolicyLanguage, OwnerIntentRule, OwnerNaturalInput,
    OwnerNaturalInputIntent, OwnerPaperReviewAnswer, OwnerSafetyBlockedCategory, OwnerSafetyRule,
    OwnerSafetyRuleSeverity, PaperLabelReviewDecision, PaperLabelReviewDecisionFile,
    PaperLabelReviewDecisionKind, PaperLabelReviewer, PaperLabelValidationPolicy,
    PaperOutcomeEvidenceFile, PaperOutcomeEvidenceHorizon, PaperOutcomeEvidenceMatchConfidence,
    PaperOutcomeEvidenceMatchType, PaperOutcomeEvidenceRecord, PaperOutcomeEvidenceValidationHint,
    PaperOutcomeFixture, PaperPriceBar, PaperPriceMoveLabelPolicy, PaperPriceSeries,
    PaperPriceSeriesStore, PaperScenarioRunConfig, PreferredMarketBias, PreferredTimeHorizon,
    PromotionSafetyDeltaStatus, RealArchetypeIntakePolicy, RecommendedLabelAction,
    RecommendedNextData, RecommendedPaperEvidenceAction, ReplayCoverageTargetConfig,
    ReplayCoverageTargetStatus, ReplayDataset, ReplayDatasetFilter, ReplayEvidenceWorkbenchConfig,
    ReplayExample, ReplayFeatureSanitizer, ReplayInputFeatures, ReplayLabelConfidence,
    ReplayLabelSource, ReplayLeakageIssueType, ReplayLeakageSeverity, ReplayLeakageStatus,
    ReplayPostDecisionContext, ReplayQualityStatus, ReplaySection, ReplayTarget,
    ReplayTargetLabels, ReplayTrainingInclusionPolicy, ResearchAutoRunConfig, ResearchEvidenceKind,
    ResearchNetworkMode, ResearchRunConfig, ResearchRunMode, ResearchSourceDescriptor,
    ResearchSourceKind, ResearchSourceRegistry, ResearchSourceTrustLevel,
    ResearchToPaperEvidenceConversionPolicy, RiskAlignmentStatus, RiskGovernorStatus,
    RssContentTypeGate, RssContentTypeStatus, RssFetchPilotConfig, RssFetchPilotStatus,
    RssXmlParseConfig, RssXmlParseStatus, RuntimeAdapterEntryAuditLog,
    RuntimeAdapterEntryDecisionKind, RuntimeAdapterEntryGateRunConfig,
    RuntimeAdapterEntryGateStatus, RuntimeEscalationHarnessStatus, SafeHttpClientPolicy,
    SafeHttpFetchStatus, SafeHttpMethod, SafeHttpRequest, SafeHttpResponse, SafeNewsFetchPolicy,
    SanitizedFieldSafetyStatus, SanitizedReplayDatasetBuildConfig,
    SanitizedReplayDatasetBuildStatus, SelfGrowingEvidenceCandidate,
    SelfGrowingEvidenceCandidateStatus, SelfGrowingEvidenceKind, SelfGrowingEvidenceLabelSource,
    SelfGrowingEvidencePromotionDecisionStatus, SelfGrowingEvidencePromotionRunConfig,
    SelfGrowingEvidenceStagingStore, SelfGrowingReplayEvidenceConfig, SequenceCoreKind,
    ShadowTrainingOperation, ShapeContractEnforcementBridgeStatus, SimulatedPaperOutcome,
    SmartCoreDebugInterpretationStatusV0, SmartCoreDebugOutputSafetyStatusV0,
    SmartCoreHeadProjectionConfigV0, SmartCoreHeadProjectionDryRunConfigV0,
    SmartCoreHeadProjectionDryRunStatusV0, SmartCoreHeadProjectionOutputModeV0,
    SmartCoreMicroKernelComponentV0, SmartCoreMicroKernelConfigV0,
    SmartCoreMicroKernelDryRunConfigV0, SmartCoreMicroKernelStateV0, SmartCoreMicroKernelStatusV0,
    SmartCoreRuntimeCapability, SmartCoreV2AdapterStatus, SmartCoreV2ComponentStatus,
    SmartCoreV2Family, SmartCoreV2HeadKind, SmartCoreV2LossContract, SmartCoreV2LossHead,
    SmartCoreV2TrainingBatch, SnapshotFileNamingPolicy, SourceConfidence, SourceTrustStatus,
    StagedEvidenceFailureReason, StagedEvidenceLinkMatchType, StagedEvidenceLinkResolutionPolicy,
    StagedEvidencePriceEnrichment, StagedEvidencePriceEnrichmentPolicy,
    StagedEvidencePriceEnrichmentStatus, StagedEvidenceReviewAnalysis, StagedEvidenceReviewReason,
    StyleCardStatus, StyleMappingMode, TemporalFeatureBoundaryPolicy, TinyLossSimulationConfig,
    TinyNoWeightTrainingDryRunConfig, TinyParameterInitPolicyV0, TinyTrainingAllowedNextStep,
    TinyTrainingEligibilityPolicy, TinyTrainingEligibilityStatus, TinyTrainingForbiddenOperation,
    TrainerWarningKind, TrainerWarningSeverity, TrainingBatchIteratorConfig, TrainingBatchSplit,
    TrainingCandidateBuildConfig, TrainingCandidateDataset, TrainingCandidateRefreshPolicy,
    TrainingFeatureSchema, TrainingSimulationSafetyStatus, TrainingSplitConfig,
    TrainingSplitResult, TrainingSplitStratifyBy, TrainingTargetHead, TrainingTargetSchema,
    ValidatedPaperEvidencePromotionStatus, ValidatedRatioExpansionRunConfig, WatchlistCandidate,
    WatchlistCandidateStatus, WatchlistCandidateStore, WatchlistRecheckConfig,
    WatchlistRecheckSkipReason, WeakLabelClosureApplyConfig, WeakLabelClosureStatus,
    WeakLabelReviewApplyConfig, WeakReplayLabelInventory, WeakReplayLabelItem,
    WeakReplayLabelPriority, WeakReplayLabelReason, WeakReplaySuggestedEvidence,
    analyze_staged_evidence_review_queue, apply_approved_evidence_to_training_candidates,
    apply_evidence_backfill_patch_to_local_json, apply_self_growing_candidate_enrichment_patch,
    apply_weak_label_closure_plan, apply_weak_label_review_decisions,
    assemble_microkernel_sequence, bootstrap_adapter_golden_baseline, bucketize_text_feature,
    build_adapter_input_shape_from_training_batch, build_adapter_registry_for_members,
    build_adapter_shape_golden_snapshots, build_ai_research_packets,
    build_ambiguous_evidence_inventory, build_collection_queue_from_gaps,
    build_committee_state_snapshot, build_evidence_backfill_patch,
    build_expected_adapter_output_shape, build_member_improvement_plan,
    build_member_learning_data_contracts_for_registry, build_member_microkernel_bridge_profile,
    build_member_smartcore_adapter_profile, build_no_persistence_training_simulation_brief,
    build_offline_trainer_dry_run_spec, build_offline_trainer_readiness_brief,
    build_owner_console_read_model, build_paper_evidence_expansion_batch,
    build_paper_evidence_records_from_price_move_candidates, build_pre_decision_memory_snapshot,
    build_replay_dataset_from_experience_store, build_replay_evidence_workbench,
    build_replay_training_inclusion_mask, build_sanitized_replay_dataset_from_experience_store,
    build_self_growing_candidate_enrichment_patch, build_shadow_training_step_plan,
    build_smart_core_v2_specs_for_members, build_smartcore_v2_batch_spec,
    build_staged_evidence_review_queue, build_tasks_from_weak_labels,
    build_tiny_training_experiment_contract, build_training_batches,
    build_training_candidate_dataset, build_training_feature_row, build_training_split,
    build_training_target_row, build_validated_label_ratio_expansion_plan,
    build_validated_replay_dataset, build_validated_replay_dataset_with_paper_evidence,
    build_weak_label_closure_inventory, build_weak_label_closure_plan,
    build_weak_label_review_queue, build_weak_replay_label_inventory, built_in_owner_intent_policy,
    check_auto_approval_success_path, check_backtest_label_contract, check_replay_data_leakage,
    collect_news_from_providers, collect_news_snapshots, collect_research_evidence,
    compare_adapter_shape_golden_snapshots, compare_current_snapshot_to_expected_baseline,
    compose_owner_action_from_read_model, compute_member_calibration_summaries,
    compute_member_calibration_summary, compute_promotion_safety_delta,
    compute_promotion_success_metrics, compute_replay_coverage_matrix, consume_owner_action_file,
    consume_owner_action_file_with_previous_run, convert_collected_news_to_snapshots,
    convert_research_evidence_to_paper_outcome_evidence, convert_rss_items_to_collected_news,
    create_three_member_pilot_roster, default_adapter_input_schema_v1,
    default_adapter_output_schema_v1, default_adapter_to_microkernel_bridge_v1,
    default_head_specs_for_member, default_owner_intent_policy_load_result,
    default_three_member_canonical_id_map, detect_sparse_coverage_cells,
    enrich_staged_evidence_with_local_price_series, evaluate_adapter_schema_drift,
    evaluate_adapter_skeleton_safety, evaluate_label_confidence_upgrade, evaluate_label_promotion,
    evaluate_member_calibration, evaluate_microkernel_runtime_safety_v0,
    evaluate_no_persistence_training_simulation_gate, evaluate_offline_trainer_design_status,
    evaluate_offline_training_design_gate, evaluate_offline_training_readiness,
    evaluate_offline_training_readiness_with_label_ratio_threshold,
    evaluate_offline_training_readiness_with_thresholds, evaluate_replay_coverage_targets,
    evaluate_replay_dataset_quality, evaluate_research_source_trust,
    evaluate_risk_governor_alignment, evaluate_self_growing_evidence_promotion,
    evaluate_smartcore_adapter_skeleton_readiness, evaluate_tiny_training_eligibility,
    evaluate_training_simulation_safety, extract_member_experiences_from_batch_cycle,
    fetch_safe_http_text, from_vec_1d, from_vec_2d, gated_deltanet_memory_step_v0,
    generate_deterministic_self_review_notes, generate_evidence_match_candidates,
    generate_member_self_review_notes, generate_price_move_label_candidates,
    init_gated_deltanet_memory_params_v0, init_mamba3_temporal_cell_params_v0,
    init_smartcore_microkernel_params_v0, load_adapter_golden_snapshot_baseline_from_local_json,
    load_owner_attention_actions_from_local_json, load_owner_console_read_model_from_local_file,
    load_owner_feedback_from_local_json, load_owner_intent_policy_from_local_file,
    load_paper_label_review_decisions_from_local_json, load_paper_outcome_evidence_from_local_json,
    load_price_series_store_from_local_json, mac_mini_local_policy, mamba3_temporal_cell_step_v0,
    mamba3_temporal_sequence_forward_v0, map_style_cards_to_three_member_pilot,
    map_training_batch_to_microkernel_sequence_v0, market_committee_layouts,
    match_paper_outcome_evidence_to_replay, matvec, news_cache_entries_from_collected_items,
    news_cache_entries_to_news_snapshots, normalize_microkernel_warnings,
    normalize_trainer_warnings, pack_numeric_features, parse_owner_natural_input,
    parse_owner_natural_input_with_policy, parse_rss_xml_fixture,
    promote_labels_with_paper_evidence, prove_no_weight_update_path,
    refresh_training_candidate_dataset_from_promotions, reject_full_article_like_summary,
    render_owner_console_terminal_view, resolve_ambiguous_evidence, resolve_canonical_member_id,
    resolve_staged_evidence_links, route_data_to_ai_members, run_adapter_contract_lock,
    run_adapter_contract_lock_v2, run_adapter_skeleton_dry_run, run_auto_approval_end_to_end,
    run_autonomous_paper_committee_loop_from_config_path,
    run_batch_committee_cycle_from_config_path, run_batch_committee_cycle_with_state,
    run_batch_committee_cycle_with_state_from_config_path, run_batch_to_microkernel_bridge,
    run_daily_brief_storage_update, run_enriched_evidence_promotion,
    run_evidence_backfill_and_promotion_with_inputs, run_minimal_committee_cycle,
    run_news_provider, run_no_persistence_training_simulation,
    run_offline_trainer_data_loader_dry_run, run_offline_trainer_dry_run_v2,
    run_owner_attention_triage, run_owner_console_viewer, run_paper_scenario_collection,
    run_replay_quality_evaluation, run_replay_quality_evaluation_with_thresholds,
    run_research_auto_run, run_research_auto_run_with_rss_transport, run_research_packet_pipeline,
    run_runtime_adapter_entry_gate, run_runtime_escalation_negative_harness,
    run_self_growing_evidence_promotion, run_self_growing_replay_evidence,
    run_single_allowlisted_rss_fetch_pilot, run_single_allowlisted_rss_fetch_pilot_with_transport,
    run_smartcore_head_projection_dry_run_v0, run_smartcore_microkernel_dry_run_v0,
    run_tiny_no_weight_training_dry_run, run_validated_ratio_expansion_with_inputs,
    run_watchlist_recheck_cycle, save_adapter_golden_snapshot_baseline_to_local_json,
    save_committee_state_snapshot, save_price_series_store_to_local_json,
    score_self_growing_evidence_candidate, simulate_tiny_label_losses,
    smartcore_microkernel_forward_v0, summarize_adapter_runtime_entry_readiness,
    summarize_label_quality, summarize_loss_metric_readiness, summarize_member_batch_readiness,
    summarize_member_learning_data_contracts, summarize_paper_outcome_evidence_quality,
    summarize_tiny_loss_by_member, validate_adapter_golden_snapshot_baseline,
    validate_adapter_input_schema_v1, validate_adapter_output_schema_v1,
    validate_adapter_to_microkernel_bridge_v1, validate_batch_against_adapter_contract_v2,
    validate_batch_against_adapter_profile, validate_gated_deltanet_memory_adapter_spec,
    validate_head_adapter_spec, validate_head_projection_config_v0,
    validate_head_projection_params_v0, validate_loss_contract_against_batch,
    validate_mamba3_temporal_adapter_spec, validate_member_id_mapping,
    validate_member_learning_data_contract, validate_metric_contract_against_dataset,
    validate_no_adapter_output_values, validate_paper_label, validate_paper_outcome_evidence_file,
    validate_rss_content_type, validate_safe_http_request, validate_safe_http_response,
    validate_smartcore_v2_head_shapes, validate_sparse_event_attention_adapter_spec,
    write_committee_state_export, write_owner_natural_input_action_file, zeros_1d,
};

fn single_cycle_config() -> MinimalAiCommitteeCycleConfig {
    MinimalAiCommitteeCycleConfig {
        input_path: Some("examples/minimal_ai_committee_core_sample.json".to_string()),
        offline_member_opinion_path: Some(
            "examples/minimal_ai_committee_offline_member_sample.json".to_string(),
        ),
        offline_member_output_batch_path: None,
        batch_mode: false,
        member_state_input_path: None,
        member_state_output_path: None,
        emit_owner_summary: false,
        emit_owner_console_view: false,
        owner_feedback_path: None,
        owner_comment_text: None,
        owner_comment_path: None,
        owner_intent_policy_path: None,
        emit_reconsideration_view: false,
        member_experience_store_input_path: None,
        member_experience_store_output_path: None,
        replay_dataset_output_path: None,
        emit_learning_summary: false,
        emit_replay_dataset_summary: false,
        replay_quality_eval_enabled: false,
        replay_quality_eval_output_path: None,
        min_replay_examples_required: 10,
        min_examples_per_member_required: 2,
        replay_sanitization_enabled: false,
        sanitized_replay_dataset_output_path: None,
        strict_temporal_boundary: true,
        include_post_decision_context_for_audit: true,
        reject_on_blocking_leakage: true,
        replay_coverage_eval_enabled: false,
        replay_coverage_target_min_total: 10,
        replay_coverage_collection_queue_output_path: None,
        paper_scenario_collection_enabled: false,
        paper_outcome_fixture_path: None,
        scenario_run_output_path: None,
        label_validation_enabled: false,
        validated_replay_dataset_output_path: None,
        label_quality_summary_output_path: None,
        min_validated_label_ratio_required: 0.5,
        paper_label_validation_policy_path: None,
        backtest_label_contract_path: None,
        label_validation_with_evidence_enabled: false,
        paper_outcome_evidence_path: None,
        paper_outcome_evidence_quality_output_path: None,
        validated_replay_with_evidence_output_path: None,
        evidence_backfill_enabled: false,
        evidence_backfill_dry_run: true,
        evidence_backfill_apply_patch: false,
        evidence_backfill_output_path: None,
        evidence_backfill_min_validated_ratio: 0.5,
        evidence_backfill_emit_summary: false,
        validated_ratio_expansion_enabled: false,
        validated_ratio_expansion_dry_run: true,
        paper_price_series_path: None,
        generated_paper_evidence_output_path: None,
        validated_ratio_target: 0.5,
        validated_ratio_expansion_output_path: None,
        weak_label_review_enabled: false,
        weak_label_review_decision_path: None,
        weak_label_review_output_path: None,
        replay_training_inclusion_mask_output_path: None,
        weak_label_review_dry_run: true,
        exclude_weak_labels_from_training_design: true,
        weak_label_closure_enabled: false,
        weak_label_closure_dry_run: true,
        training_candidate_dataset_output_path: None,
        training_split_output_path: None,
        offline_trainer_dry_run_enabled: false,
        offline_trainer_dry_run_output_path: None,
        offline_trainer_v2_enabled: false,
        offline_trainer_v2_batch_size: 8,
        offline_trainer_v2_output_path: None,
        offline_trainer_design_status_output_path: None,
        trainer_readiness_brief_enabled: false,
        trainer_readiness_brief_output_path: None,
        tiny_training_eligibility_gate_enabled: false,
        tiny_training_contract_output_path: None,
        min_tiny_training_examples_required: 8,
        min_tiny_training_members_required: 3,
        tiny_no_weight_loss_simulation_enabled: false,
        tiny_no_weight_loss_simulation_output_path: None,
        tiny_loss_batch_size: 8,
        tiny_loss_enabled_heads: vec![
            soma_zero::league::minimal_ai_committee_core::TrainingTargetHead::Stance,
            soma_zero::league::minimal_ai_committee_core::TrainingTargetHead::ConfidenceCalibration,
            soma_zero::league::minimal_ai_committee_core::TrainingTargetHead::Risk,
            soma_zero::league::minimal_ai_committee_core::TrainingTargetHead::EvidenceNeed,
        ],
        tiny_loss_prediction_policy:
            soma_zero::league::minimal_ai_committee_core::DummyPredictionPolicy::default(),
        no_persistence_training_gate_enabled: false,
        no_persistence_training_simulation_enabled: false,
        no_persistence_training_simulation_output_path: None,
        no_persistence_training_brief_output_path: None,
        no_persistence_max_epochs: 1,
        no_persistence_max_steps: 3,
        smartcore_adapter_skeleton_gate_enabled: false,
        adapter_skeleton_dry_run_enabled: false,
        adapter_skeleton_output_path: None,
        adapter_skeleton_include_sparse_event_attention: true,
        adapter_skeleton_validate_batches: true,
        adapter_skeleton_require_runtime_deferred: true,
        adapter_skeleton_require_training_deferred: true,
        adapter_contract_lock_enabled: false,
        adapter_contract_golden_snapshot_output_path: None,
        adapter_contract_expected_snapshot_path: None,
        adapter_contract_require_schema_version_match: true,
        adapter_contract_fail_on_unmatched_batch: true,
        adapter_contract_fail_on_unknown_member_alias: true,
        adapter_contract_fail_on_output_values: true,
        adapter_contract_lock_v2_enabled: false,
        adapter_expected_golden_baseline_path: None,
        adapter_bootstrap_golden_baseline_path: None,
        adapter_bootstrap_missing_baseline: false,
        adapter_write_golden_baseline_if_missing: false,
        adapter_fail_on_missing_baseline: true,
        adapter_allow_schema_version_bump: false,
        adapter_run_regression_harness: false,
        adapter_contract_acceptance_output_path: None,
        runtime_adapter_entry_gate_enabled: false,
        runtime_entry_audit_output_path: None,
        runtime_entry_requested_capabilities: vec![
            SmartCoreRuntimeCapability::ShapeValidation,
            SmartCoreRuntimeCapability::BuildInputShape,
            SmartCoreRuntimeCapability::BuildOutputShape,
            SmartCoreRuntimeCapability::ValidateAdapterContract,
            SmartCoreRuntimeCapability::ValidateGoldenBaseline,
        ],
        runtime_entry_run_negative_harness: false,
        runtime_entry_fail_on_forbidden_capability: true,
        runtime_entry_fail_on_contract_not_locked: true,
        runtime_entry_fail_on_baseline_drift: true,
        runtime_entry_fail_on_safety_violation: true,
        smartcore_microkernel_v0_enabled: false,
        smartcore_microkernel_lab_mode: false,
        smartcore_microkernel_output_path: None,
        smartcore_microkernel_sequence_len: 4,
        smartcore_microkernel_input_dim: 8,
        smartcore_microkernel_temporal_state_dim: 8,
        smartcore_microkernel_memory_dim: 8,
        smartcore_microkernel_output_dim: 8,
        smartcore_microkernel_use_training_candidates: true,
        smartcore_microkernel_synthetic_fallback: true,
        microkernel_bridge_enabled: false,
        microkernel_bridge_sequence_len: 4,
        microkernel_bridge_input_dim: 8,
        microkernel_bridge_fail_on_warning: false,
        microkernel_bridge_output_path: None,
        smartcore_head_projection_v0_enabled: false,
        smartcore_head_projection_output_path: None,
        smartcore_enable_stance_head: true,
        smartcore_enable_risk_head: true,
        smartcore_enable_evidence_head: true,
        smartcore_enable_confidence_head: true,
        smartcore_enable_uncertainty_head: true,
        smartcore_enable_expected_return_head: false,
        smartcore_shadow_alignment_enabled: false,
        smartcore_shadow_alignment_output_path: None,
        smartcore_shadow_include_batch_member_opinions: true,
        smartcore_shadow_include_replay_targets: true,
        smartcore_shadow_include_risk_governor_targets: true,
        smartcore_emit_owner_debug_cards: true,
        smartcore_mismatch_self_growing_enabled: false,
        smartcore_mismatch_max_tasks_total: 12,
        smartcore_mismatch_max_tasks_per_member: 4,
        smartcore_calibration_dataset_output_path: None,
        smartcore_mismatch_task_output_path: None,
        smartcore_mismatch_emit_owner_debug_summary: true,
        smartcore_mismatch_learning_loop_enabled: false,
        smartcore_mismatch_learning_dry_run: true,
        smartcore_execute_mismatch_research_tasks: false,
        smartcore_approve_calibration_targets: false,
        smartcore_refresh_calibration_dataset: false,
        smartcore_recheck_alignment: false,
        smartcore_calibration_dataset_input_path: None,
        smartcore_mismatch_learning_loop_output_path: None,
        smartcore_recalibration_enabled: false,
        smartcore_recalibration_dry_run: true,
        smartcore_recalibration_rule_table_output_path: None,
        smartcore_calibrated_debug_output_path: None,
        smartcore_recalibration_result_output_path: None,
        smartcore_recalibration_min_support: 2,
        smartcore_recalibration_max_rules_per_member_head: 2,
        smartcore_recalibration_emit_owner_summary: true,
        smartcore_shadow_opinion_enabled: false,
        smartcore_shadow_opinion_output_path: None,
        smartcore_shadow_compare_member_opinion: false,
        smartcore_shadow_target_eval: false,
        smartcore_shadow_emit_owner_debug: true,
        smartcore_shadow_stability_enabled: false,
        smartcore_shadow_stability_repeats: 3,
        smartcore_shadow_stability_output_path: None,
        smartcore_shadow_expand_agreement_targets: false,
        smartcore_shadow_target_collection_queue_output_path: None,
        smartcore_shadow_stability_emit_owner_summary: true,
        smartcore_shadow_scenario_sweep_enabled: false,
        smartcore_shadow_scenario_set_path: None,
        smartcore_shadow_scenario_repeats: 3,
        smartcore_shadow_scenario_max_count: 5,
        smartcore_shadow_scenario_output_path: None,
        smartcore_observer_readiness_gate_enabled: false,
        smartcore_observer_min_scenarios_required: 3,
        smartcore_shadow_scenario_emit_owner_summary: true,
        smartcore_observer_lane_enabled: false,
        smartcore_observer_output_path: None,
        smartcore_observer_compare_member_opinion: true,
        smartcore_observer_compare_chairman: true,
        smartcore_observer_compare_risk_governor: true,
        smartcore_observer_target_coverage_closure_enabled: true,
        smartcore_observer_emit_owner_section: true,
        observer_target_closure_enabled: false,
        observer_target_closure_dry_run: true,
        observer_target_closure_output_path: None,
        observer_target_set_output_path: None,
        observer_comparison_ledger_path: None,
        observer_readiness_hardening_enabled: false,
        observer_coverage_closure_emit_owner_summary: true,
        observer_target_apply_trend_enabled: false,
        observer_target_apply_dry_run: true,
        observer_target_apply_targets: false,
        observer_target_store_input_path: None,
        observer_target_store_output_path: None,
        observer_ledger_trend_enabled: true,
        observer_readiness_v2_enabled: false,
        observer_trend_summary_enabled: false,
        observer_apply_trend_output_path: None,
        observer_seed_apply_trend_enabled: false,
        observer_seed_apply_dry_run: true,
        observer_seed_apply_targets: false,
        observer_seed_target_store_output_path: None,
        observer_seed_apply_output_path: None,
        observer_seed_require_approved_target: true,
        observer_seed_rerun_comparison: true,
        observer_seed_compute_ledger_trend: true,
        observer_seed_recheck_readiness: true,
        observer_seed_emit_owner_summary: true,
        observer_approved_apply_governance_enabled: false,
        observer_approved_apply_mode: core::ObserverExplicitApplyMode::DryRun,
        observer_approved_apply_dry_run: true,
        observer_approved_target_store_input_path: None,
        observer_approved_target_store_output_path: None,
        observer_approved_apply_output_path: None,
        observer_approved_apply_recheck_readiness: true,
        chairman_governance_contract_prepare_enabled: true,
        chairman_governance_readiness_check_enabled: true,
        observer_approved_apply_emit_owner_summary: true,
        observer_apply_verify_chairman_shadow_enabled: false,
        observer_apply_verify_mode: core::ObserverExplicitApplyMode::DryRun,
        observer_apply_verify_dry_run: true,
        observer_apply_verify_target_store_output_path: None,
        observer_apply_verify_output_path: None,
        observer_apply_verify_config_path: None,
        observer_apply_verify_emit_owner_summary: true,
        chairman_shadow_governance_enabled: true,
        training_candidate_min_examples: None,
        self_growing_replay_enabled: false,
        research_source_registry_path: None,
        self_growing_max_tasks: 16,
        self_growing_max_evidence_records: 32,
        self_growing_allow_network_sources: false,
        research_evidence_output_path: None,
        self_growing_replay_output_path: None,
        emit_research_task_summary: false,
        self_growing_evidence_staging_enabled: false,
        self_growing_evidence_promotion_enabled: false,
        self_growing_evidence_promotion_dry_run: true,
        self_growing_evidence_apply_promotions: false,
        self_growing_refresh_training_candidates: false,
        self_growing_staging_store_path: None,
        self_growing_approved_evidence_output_path: None,
        self_growing_training_candidate_output_path: None,
        enriched_evidence_promotion_enabled: false,
        enriched_evidence_promotion_dry_run: true,
        enriched_evidence_apply_patch: false,
        enriched_evidence_apply_promotions: false,
        enriched_evidence_refresh_training_candidates: false,
        enriched_staging_output_path: None,
        enriched_approved_evidence_output_path: None,
        enriched_training_candidate_output_path: None,
        auto_approval_e2e_enabled: false,
        auto_approval_e2e_dry_run: true,
        auto_approval_success_staging_path: None,
        auto_approval_success_price_series_path: None,
        auto_approval_apply_promotions: false,
        auto_approval_refresh_training_candidates: false,
        auto_approval_approved_evidence_output_path: None,
        auto_approval_training_candidate_output_path: None,
        autonomous_paper_run: false,
        run_id: None,
        market_scopes: Vec::new(),
        symbols: Vec::new(),
        max_cycles: 1,
        cycle_mode: AutonomousPaperCycleMode::SingleShot,
        require_owner_confirmation: OwnerConfirmationPolicy::Never,
        local_market_data_path: None,
        local_news_path: None,
        news_collection_enabled: false,
        news_collection_config_path: None,
        news_provider_config_path: None,
        research_run_enabled: false,
        emit_research_run_summary: false,
        emit_research_packet_summary: false,
        research_auto_run_enabled: false,
        news_cache_input_path: None,
        news_cache_output_path: None,
        news_network_mode: NewsProviderRunMode::OfflineOnly,
        news_fetch_policy: None,
        rss_xml_fixture_path: None,
        rss_fetch_pilot_enabled: false,
        rss_fetch_pilot_url: None,
        rss_fetch_allowed_domains: Vec::new(),
        rss_fetch_source_label: None,
        rss_network_enabled: false,
        rss_safe_http_timeout_ms: 3_000,
        rss_safe_http_rate_limit_ms: 1_000,
        rss_safe_http_max_response_bytes: 262_144,
        rss_allowed_content_types: vec![
            "application/rss+xml".to_string(),
            "application/xml".to_string(),
            "text/xml".to_string(),
            "application/atom+xml".to_string(),
        ],
        rss_allow_redirects: false,
        rss_allow_missing_content_type: false,
        run_committee_cycle_from_research_packets: false,
        paper_only: true,
        owner_attention_inbox_input_path: None,
        owner_attention_inbox_output_path: None,
        owner_attention_actions_path: None,
        watchlist_candidate_input_path: None,
        watchlist_candidate_output_path: None,
        emit_owner_attention_inbox: false,
        enable_watchlist_recheck: false,
        watchlist_input_path: None,
        watchlist_output_path: None,
        max_candidates_per_cycle: 3,
        include_risk_blocked: false,
        include_needs_evidence: true,
        emit_owner_daily_brief: false,
        owner_daily_brief_store_input_path: None,
        owner_daily_brief_store_output_path: None,
        committee_state_snapshot_output_path: None,
        emit_committee_state_snapshot: false,
        committee_state_export_root_path: None,
        write_latest_snapshot: false,
        write_history_snapshot: false,
        write_snapshot_index: false,
        write_owner_console_read_model: false,
        committee_state_schema_version: None,
        max_snapshot_history_entries: None,
        inline_offline_member_opinions: Vec::new(),
        inline_input: None,
        pilot_roster: Some("three_member".to_string()),
        paper_outcome: Some(
            soma_zero::league::minimal_ai_committee_core::SimulatedPaperOutcome::Positive,
        ),
        archetype_style_cards_path: Some(
            "examples/investor_archetype_style_cards.sample.json".to_string(),
        ),
        style_mapping_mode: StyleMappingMode::LocalFixture,
    }
}

#[test]
fn market_scope_split_exists_without_runtime_paths() {
    let scopes = [
        MarketScope::KoreaShortTerm,
        MarketScope::KoreaLongTerm,
        MarketScope::UsShortTerm,
        MarketScope::UsLongTerm,
        MarketScope::CryptoShortTerm,
        MarketScope::CryptoLongTerm,
    ];
    let text = serde_json::to_string(&scopes).expect("serialize scopes");
    for required in [
        "KoreaShortTerm",
        "KoreaLongTerm",
        "UsShortTerm",
        "UsLongTerm",
        "CryptoShortTerm",
        "CryptoLongTerm",
    ] {
        assert!(text.contains(required));
    }
}

#[test]
fn minimal_ai_committee_cycle_runs_paper_only_event_loop() {
    let result = run_minimal_committee_cycle(
        single_cycle_config()
            .load_input()
            .expect("load single input"),
    )
    .expect("minimal committee cycle runs");

    assert_eq!(result.selected_scope, MarketScope::KoreaShortTerm);
    assert_eq!(result.symbol, "005930.KS");
    assert_eq!(result.routed_packet_count, 3);
    assert_eq!(
        result.activation_plan.committee_name,
        "KoreaShortTermCommittee"
    );
    assert_eq!(result.activation_plan.selected_member_ids.len(), 3);
    assert!(result.activation_plan.estimated_memory_hint_mb <= 150);
    assert!(
        result
            .activation_plan
            .runtime_status_by_member
            .iter()
            .all(
                |status| status.runtime_status == CoreRuntimeStatus::OfflineFixture
                    || status.member_id == "crypto-liquidity"
            )
    );
    assert_eq!(result.member_roles.len(), 3);
    assert!(result.member_roles.iter().any(|role| {
        role.member_id == "trend-kr-short" && role.role == IndependentMemberRole::TrendEntry
    }));
    assert_eq!(result.memory_states.len(), 3);
    assert!(result.memory_states.iter().all(|state| {
        state.recent_symbols.contains(&"005930.KS".to_string()) && state.recent_opinion_count == 1
    }));
    assert_eq!(result.learning_journal_entry_count, 3);
    assert_eq!(result.learning_journals.len(), 3);
    let registry = result
        .style_card_registry
        .as_ref()
        .expect("style card registry");
    assert_eq!(registry.cards.len(), 18);
    assert!(registry.review_required_count >= 3);
    let style_mapping = result
        .three_member_style_mapping
        .as_ref()
        .expect("three-member style mapping");
    assert_eq!(style_mapping.trend_entry_blend.member_id, "trend-kr-short");
    assert_eq!(result.style_influenced_profiles.len(), 3);
    assert!(result.style_influenced_profiles.iter().any(|profile| {
        profile.member_id == "risk-kr-short"
            && profile
                .decision_bias_notes
                .iter()
                .any(|note| note.contains("Risk Governor"))
    }));
    let journal_member_ids: std::collections::BTreeSet<_> = result
        .learning_journals
        .iter()
        .map(|journal| journal.member_id.as_str())
        .collect();
    assert_eq!(journal_member_ids.len(), 3);
    assert!(result.learning_journal_entries.iter().any(|entry| {
        entry.member_id == "risk-kr-short"
            && entry.learning_signal == MemberLearningSignal::Reinforce
    }));
    assert_eq!(result.triggered_event_count, 1);
    assert_eq!(result.committee_session_count, 1);
    assert_eq!(
        result.distributed_member_ids,
        vec![
            "trend-kr-short".to_string(),
            "risk-kr-short".to_string(),
            "evidence-kr-short".to_string()
        ]
    );
    assert!(
        !result
            .distributed_member_ids
            .contains(&"crypto-liquidity".to_string())
    );
    assert_eq!(result.member_opinions.len(), 3);
    assert!(result.member_opinions.iter().all(|opinion| {
        opinion.symbol == "005930.KS" && opinion.market_scope == MarketScope::KoreaShortTerm
    }));
    let trend_opinion = result
        .member_opinions
        .iter()
        .find(|opinion| opinion.member_id == "trend-kr-short")
        .expect("trend opinion");
    assert_eq!(trend_opinion.stance, MemberStance::BuyProposal);
    assert!(trend_opinion.event_triggered);
    assert!(
        trend_opinion
            .evidence_notes
            .contains(&"offline fixture opinion".to_string())
    );
    assert_ne!(
        trend_opinion.evidence_notes,
        result
            .member_opinions
            .iter()
            .find(|opinion| opinion.member_id == "risk-kr-short")
            .expect("risk opinion")
            .evidence_notes
    );
    let risk_opinion = result
        .member_opinions
        .iter()
        .find(|opinion| opinion.member_id == "risk-kr-short")
        .expect("risk opinion");
    assert_eq!(risk_opinion.stance, MemberStance::NoTrade);
    assert!(risk_opinion.event_triggered);
    let evidence_opinion = result
        .member_opinions
        .iter()
        .find(|opinion| opinion.member_id == "evidence-kr-short")
        .expect("evidence opinion");
    assert_eq!(evidence_opinion.stance, MemberStance::Hold);
    assert!(!evidence_opinion.event_triggered);

    let event = result.event.as_ref().expect("investment event");
    assert_eq!(event.proposed_by_member_id, "trend-kr-short");
    let session = result
        .committee_session
        .as_ref()
        .expect("committee session");
    assert_eq!(session.invited_members.len(), 3);
    assert!(session.risk_flags.contains(&"high_volatility".to_string()));

    let decision = result
        .chairman_decision
        .as_ref()
        .expect("chairman decision");
    assert_eq!(decision.final_action, ChairmanFinalAction::RiskVetoed);
    assert_eq!(decision.risk_governor_status, RiskGovernorStatus::Vetoed);
    assert!(decision.rationale.contains("Risk Governor vetoed"));
    assert!(result.learning_journal_entries.iter().all(|entry| {
        entry.journal_id.contains(&decision.decision_id) && entry.note.contains("no model training")
    }));

    let risk_update = result
        .score_updates
        .iter()
        .find(|update| update.member_id == "risk-kr-short")
        .expect("risk score update");
    assert_eq!(
        risk_update.update_reason,
        MemberScoreUpdateReason::HelpfulDissent
    );
    assert!(risk_update.new_voice_weight > risk_update.previous_voice_weight);
    let trend_update = result
        .score_updates
        .iter()
        .find(|update| update.member_id == "trend-kr-short")
        .expect("trend score update");
    assert_eq!(
        trend_update.update_reason,
        MemberScoreUpdateReason::RiskyCall
    );
    assert!(trend_update.new_voice_weight < trend_update.previous_voice_weight);

    assert!(result.safety_summary.paper_only);
    assert!(result.safety_summary.no_real_order_path);
    assert!(result.safety_summary.no_broker_order_account);
    assert!(result.safety_summary.no_model_training);
    assert!(result.safety_summary.no_live_inference);
}

#[test]
fn minimal_ai_committee_cycle_is_deterministic_and_cli_is_safe() {
    let first = run_minimal_committee_cycle(
        single_cycle_config()
            .load_input()
            .expect("load first input"),
    )
    .expect("first run");
    let second = run_minimal_committee_cycle(
        single_cycle_config()
            .load_input()
            .expect("load second input"),
    )
    .expect("second run");
    assert_eq!(first, second);

    let binary = env!("CARGO_BIN_EXE_soma_experiment");
    let output = Command::new(binary)
        .args([
            "minimal-ai-committee-cycle",
            "--config",
            "examples/soma_minimal_ai_committee_core.toml",
        ])
        .output()
        .expect("run minimal committee CLI");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("minimal_ai_committee_warning"));
    assert!(stdout.contains("paper-only local mock/offline fixture member logic"));
    assert!(stdout.contains("\"batch_id\": \"offline-batch-sprint130-sample\""));
    assert!(stdout.contains("\"routed_packet_count\": 18"));
    assert!(stdout.contains("\"member_opinion_count\": 18"));
    assert!(stdout.contains("\"event_count\": 5"));
    assert!(stdout.contains("\"events_by_symbol\""));
    assert!(stdout.contains("\"risk_veto_count\""));
    assert!(stdout.contains("\"score_update_count\""));
    assert!(stdout.contains("\"member_experience_records\""));
    assert!(stdout.contains("\"replay_dataset\""));
    assert!(stdout.contains("\"calibration_summaries\""));
    assert!(stdout.contains("\"smart_core_v2_specs\""));
    assert!(stdout.contains("\"replay_quality_eval\""));
    assert!(stdout.contains("\"offline_training_readiness_gate\""));
    assert!(stdout.contains("\"sanitized_replay_build\""));
    assert!(stdout.contains("\"sanitized_count\""));
    assert!(stdout.contains("\"rejected_count\""));
    assert!(stdout.contains("\"readiness_status\""));
    assert!(stdout.contains("\"owner_summary\""));
    assert!(stdout.contains("\"owner_console_view\""));
    assert!(stdout.contains("\"owner_feedback_reconsideration\""));
    assert!(stdout.contains("\"attention_queue\""));
    assert!(stdout.contains("\"owner_attention_triage\""));
    assert!(stdout.contains("\"generated_owner_feedback_count\""));
    assert!(stdout.contains("\"generated_watchlist_candidate_count\""));
    assert!(stdout.contains("\"generated_watchlist_candidates\""));
    assert!(stdout.contains("\"watchlist_recheck\""));
    assert!(stdout.contains("\"owner_daily_brief\""));
    assert!(stdout.contains("\"lifecycle_events\""));
    assert!(stdout.contains("\"paper_decision_archive\""));
    assert!(stdout.contains("\"member_status_rows\""));
    assert!(stdout.contains("\"next_action_rows\""));
    assert!(stdout.contains("\"member_voice_changes\""));
    assert!(stdout.contains("paper-only explanation"));
    assert!(stdout.contains("offline batch opinion"));
    assert!(stdout.contains("\"no_broker_order_account\": true"));
    assert!(stdout.contains("\"final_action\": \"RiskVetoed\""));

    let help = Command::new(binary)
        .args(["minimal-ai-committee-cycle", "--help"])
        .output()
        .expect("run minimal committee help");
    assert!(help.status.success());
    let help_text = String::from_utf8_lossy(&help.stdout);
    assert!(help_text.contains("no broker/order/account"));
    assert!(help_text.contains("no training"));
    assert!(help_text.contains("no live inference"));

    let remote = Command::new(binary)
        .args([
            "minimal-ai-committee-cycle",
            "--config",
            "https://example.invalid/config.toml",
        ])
        .output()
        .expect("run remote config rejection");
    assert!(!remote.status.success());
    assert!(String::from_utf8_lossy(&remote.stderr).contains("config path must be local"));
}

#[test]
fn offline_member_brain_adapter_returns_fixture_and_falls_back_safely() {
    let fixtures = vec![OfflineMemberOpinionFixture {
        member_id: "offline-member".to_string(),
        symbol: "005930.KS".to_string(),
        market_scope: MarketScope::KoreaShortTerm,
        stance: MemberStance::BuyProposal,
        confidence: 0.77,
        expected_return_hint: 0.02,
        risk_hint: 0.03,
        evidence_notes: vec![
            "offline fixture opinion".to_string(),
            "local file only".to_string(),
        ],
        event_triggered: true,
        event_reason: Some("fixture event".to_string()),
    }];
    let adapter = OfflineMemberBrainAdapter { fixtures };
    let packet = MemberInputPacket {
        member_id: "offline-member".to_string(),
        market_data: serde_json::from_value(serde_json::json!({
            "symbol": "005930.KS",
            "market_scope": "KoreaShortTerm",
            "timestamp": "2026-05-21T09:00:00+09:00",
            "price": 78000.0,
            "change_pct": 2.0,
            "volume": 1000.0,
            "volatility_hint": 0.03,
            "source_label": "test"
        }))
        .expect("market data"),
        news: Vec::new(),
        owner_context: Some("paper-only".to_string()),
        previous_member_score: Some(0.6),
    };
    let opinion = adapter.produce_opinion(&packet);
    assert_eq!(opinion.stance, MemberStance::BuyProposal);
    assert!(opinion.event_triggered);
    assert!(
        opinion
            .evidence_notes
            .contains(&"local file only".to_string())
    );

    let missing_packet = MemberInputPacket {
        member_id: "missing-member".to_string(),
        ..packet
    };
    let fallback = adapter.produce_opinion(&missing_packet);
    assert_eq!(fallback.stance, MemberStance::NeedMoreEvidence);
    assert!(!fallback.event_triggered);
    assert!(
        fallback
            .evidence_notes
            .iter()
            .any(|note| note.contains("no external model"))
    );
}

#[test]
fn offline_fixture_path_is_local_only() {
    let err = OfflineMemberBrainAdapter::from_json_path(std::path::Path::new(
        "https://example.invalid/offline.json",
    ))
    .expect_err("remote offline fixture must fail");
    assert!(err.contains("must be local"));

    let config = MinimalAiCommitteeCycleConfig {
        input_path: Some("examples/minimal_ai_committee_core_sample.json".to_string()),
        offline_member_opinion_path: Some("https://example.invalid/offline.json".to_string()),
        offline_member_output_batch_path: None,
        batch_mode: false,
        member_state_input_path: None,
        member_state_output_path: None,
        emit_owner_summary: false,
        emit_owner_console_view: false,
        owner_feedback_path: None,
        owner_comment_text: None,
        owner_comment_path: None,
        owner_intent_policy_path: None,
        emit_reconsideration_view: false,
        member_experience_store_input_path: None,
        member_experience_store_output_path: None,
        replay_dataset_output_path: None,
        emit_learning_summary: false,
        emit_replay_dataset_summary: false,
        replay_quality_eval_enabled: false,
        replay_quality_eval_output_path: None,
        min_replay_examples_required: 10,
        min_examples_per_member_required: 2,
        replay_sanitization_enabled: false,
        sanitized_replay_dataset_output_path: None,
        strict_temporal_boundary: true,
        include_post_decision_context_for_audit: true,
        reject_on_blocking_leakage: true,
        replay_coverage_eval_enabled: false,
        replay_coverage_target_min_total: 10,
        replay_coverage_collection_queue_output_path: None,
        paper_scenario_collection_enabled: false,
        paper_outcome_fixture_path: None,
        scenario_run_output_path: None,
        label_validation_enabled: false,
        validated_replay_dataset_output_path: None,
        label_quality_summary_output_path: None,
        min_validated_label_ratio_required: 0.5,
        paper_label_validation_policy_path: None,
        backtest_label_contract_path: None,
        label_validation_with_evidence_enabled: false,
        paper_outcome_evidence_path: None,
        paper_outcome_evidence_quality_output_path: None,
        validated_replay_with_evidence_output_path: None,
        evidence_backfill_enabled: false,
        evidence_backfill_dry_run: true,
        evidence_backfill_apply_patch: false,
        evidence_backfill_output_path: None,
        evidence_backfill_min_validated_ratio: 0.5,
        evidence_backfill_emit_summary: false,
        validated_ratio_expansion_enabled: false,
        validated_ratio_expansion_dry_run: true,
        paper_price_series_path: None,
        generated_paper_evidence_output_path: None,
        validated_ratio_target: 0.5,
        validated_ratio_expansion_output_path: None,
        weak_label_review_enabled: false,
        weak_label_review_decision_path: None,
        weak_label_review_output_path: None,
        replay_training_inclusion_mask_output_path: None,
        weak_label_review_dry_run: true,
        exclude_weak_labels_from_training_design: true,
        weak_label_closure_enabled: false,
        weak_label_closure_dry_run: true,
        training_candidate_dataset_output_path: None,
        training_split_output_path: None,
        offline_trainer_dry_run_enabled: false,
        offline_trainer_dry_run_output_path: None,
        offline_trainer_v2_enabled: false,
        offline_trainer_v2_batch_size: 8,
        offline_trainer_v2_output_path: None,
        offline_trainer_design_status_output_path: None,
        trainer_readiness_brief_enabled: false,
        trainer_readiness_brief_output_path: None,
        tiny_training_eligibility_gate_enabled: false,
        tiny_training_contract_output_path: None,
        min_tiny_training_examples_required: 8,
        min_tiny_training_members_required: 3,
        tiny_no_weight_loss_simulation_enabled: false,
        tiny_no_weight_loss_simulation_output_path: None,
        tiny_loss_batch_size: 8,
        tiny_loss_enabled_heads: vec![
            soma_zero::league::minimal_ai_committee_core::TrainingTargetHead::Stance,
            soma_zero::league::minimal_ai_committee_core::TrainingTargetHead::ConfidenceCalibration,
            soma_zero::league::minimal_ai_committee_core::TrainingTargetHead::Risk,
            soma_zero::league::minimal_ai_committee_core::TrainingTargetHead::EvidenceNeed,
        ],
        tiny_loss_prediction_policy:
            soma_zero::league::minimal_ai_committee_core::DummyPredictionPolicy::default(),
        no_persistence_training_gate_enabled: false,
        no_persistence_training_simulation_enabled: false,
        no_persistence_training_simulation_output_path: None,
        no_persistence_training_brief_output_path: None,
        no_persistence_max_epochs: 1,
        no_persistence_max_steps: 3,
        smartcore_adapter_skeleton_gate_enabled: false,
        adapter_skeleton_dry_run_enabled: false,
        adapter_skeleton_output_path: None,
        adapter_skeleton_include_sparse_event_attention: true,
        adapter_skeleton_validate_batches: true,
        adapter_skeleton_require_runtime_deferred: true,
        adapter_skeleton_require_training_deferred: true,
        adapter_contract_lock_enabled: false,
        adapter_contract_golden_snapshot_output_path: None,
        adapter_contract_expected_snapshot_path: None,
        adapter_contract_require_schema_version_match: true,
        adapter_contract_fail_on_unmatched_batch: true,
        adapter_contract_fail_on_unknown_member_alias: true,
        adapter_contract_fail_on_output_values: true,
        adapter_contract_lock_v2_enabled: false,
        adapter_expected_golden_baseline_path: None,
        adapter_bootstrap_golden_baseline_path: None,
        adapter_bootstrap_missing_baseline: false,
        adapter_write_golden_baseline_if_missing: false,
        adapter_fail_on_missing_baseline: true,
        adapter_allow_schema_version_bump: false,
        adapter_run_regression_harness: false,
        adapter_contract_acceptance_output_path: None,
        runtime_adapter_entry_gate_enabled: false,
        runtime_entry_audit_output_path: None,
        runtime_entry_requested_capabilities: vec![
            SmartCoreRuntimeCapability::ShapeValidation,
            SmartCoreRuntimeCapability::BuildInputShape,
            SmartCoreRuntimeCapability::BuildOutputShape,
            SmartCoreRuntimeCapability::ValidateAdapterContract,
            SmartCoreRuntimeCapability::ValidateGoldenBaseline,
        ],
        runtime_entry_run_negative_harness: false,
        runtime_entry_fail_on_forbidden_capability: true,
        runtime_entry_fail_on_contract_not_locked: true,
        runtime_entry_fail_on_baseline_drift: true,
        runtime_entry_fail_on_safety_violation: true,
        smartcore_microkernel_v0_enabled: false,
        smartcore_microkernel_lab_mode: false,
        smartcore_microkernel_output_path: None,
        smartcore_microkernel_sequence_len: 4,
        smartcore_microkernel_input_dim: 8,
        smartcore_microkernel_temporal_state_dim: 8,
        smartcore_microkernel_memory_dim: 8,
        smartcore_microkernel_output_dim: 8,
        smartcore_microkernel_use_training_candidates: true,
        smartcore_microkernel_synthetic_fallback: true,
        microkernel_bridge_enabled: false,
        microkernel_bridge_sequence_len: 4,
        microkernel_bridge_input_dim: 8,
        microkernel_bridge_fail_on_warning: false,
        microkernel_bridge_output_path: None,
        smartcore_head_projection_v0_enabled: false,
        smartcore_head_projection_output_path: None,
        smartcore_enable_stance_head: true,
        smartcore_enable_risk_head: true,
        smartcore_enable_evidence_head: true,
        smartcore_enable_confidence_head: true,
        smartcore_enable_uncertainty_head: true,
        smartcore_enable_expected_return_head: false,
        smartcore_shadow_alignment_enabled: false,
        smartcore_shadow_alignment_output_path: None,
        smartcore_shadow_include_batch_member_opinions: true,
        smartcore_shadow_include_replay_targets: true,
        smartcore_shadow_include_risk_governor_targets: true,
        smartcore_emit_owner_debug_cards: true,
        smartcore_mismatch_self_growing_enabled: false,
        smartcore_mismatch_max_tasks_total: 12,
        smartcore_mismatch_max_tasks_per_member: 4,
        smartcore_calibration_dataset_output_path: None,
        smartcore_mismatch_task_output_path: None,
        smartcore_mismatch_emit_owner_debug_summary: true,
        smartcore_mismatch_learning_loop_enabled: false,
        smartcore_mismatch_learning_dry_run: true,
        smartcore_execute_mismatch_research_tasks: false,
        smartcore_approve_calibration_targets: false,
        smartcore_refresh_calibration_dataset: false,
        smartcore_recheck_alignment: false,
        smartcore_calibration_dataset_input_path: None,
        smartcore_mismatch_learning_loop_output_path: None,
        smartcore_recalibration_enabled: false,
        smartcore_recalibration_dry_run: true,
        smartcore_recalibration_rule_table_output_path: None,
        smartcore_calibrated_debug_output_path: None,
        smartcore_recalibration_result_output_path: None,
        smartcore_recalibration_min_support: 2,
        smartcore_recalibration_max_rules_per_member_head: 2,
        smartcore_recalibration_emit_owner_summary: true,
        smartcore_shadow_opinion_enabled: false,
        smartcore_shadow_opinion_output_path: None,
        smartcore_shadow_compare_member_opinion: false,
        smartcore_shadow_target_eval: false,
        smartcore_shadow_emit_owner_debug: true,
        smartcore_shadow_stability_enabled: false,
        smartcore_shadow_stability_repeats: 3,
        smartcore_shadow_stability_output_path: None,
        smartcore_shadow_expand_agreement_targets: false,
        smartcore_shadow_target_collection_queue_output_path: None,
        smartcore_shadow_stability_emit_owner_summary: true,
        smartcore_shadow_scenario_sweep_enabled: false,
        smartcore_shadow_scenario_set_path: None,
        smartcore_shadow_scenario_repeats: 3,
        smartcore_shadow_scenario_max_count: 5,
        smartcore_shadow_scenario_output_path: None,
        smartcore_observer_readiness_gate_enabled: false,
        smartcore_observer_min_scenarios_required: 3,
        smartcore_shadow_scenario_emit_owner_summary: true,
        smartcore_observer_lane_enabled: false,
        smartcore_observer_output_path: None,
        smartcore_observer_compare_member_opinion: true,
        smartcore_observer_compare_chairman: true,
        smartcore_observer_compare_risk_governor: true,
        smartcore_observer_target_coverage_closure_enabled: true,
        smartcore_observer_emit_owner_section: true,
        observer_target_closure_enabled: false,
        observer_target_closure_dry_run: true,
        observer_target_closure_output_path: None,
        observer_target_set_output_path: None,
        observer_comparison_ledger_path: None,
        observer_readiness_hardening_enabled: false,
        observer_coverage_closure_emit_owner_summary: true,
        observer_target_apply_trend_enabled: false,
        observer_target_apply_dry_run: true,
        observer_target_apply_targets: false,
        observer_target_store_input_path: None,
        observer_target_store_output_path: None,
        observer_ledger_trend_enabled: true,
        observer_readiness_v2_enabled: false,
        observer_trend_summary_enabled: false,
        observer_apply_trend_output_path: None,
        observer_seed_apply_trend_enabled: false,
        observer_seed_apply_dry_run: true,
        observer_seed_apply_targets: false,
        observer_seed_target_store_output_path: None,
        observer_seed_apply_output_path: None,
        observer_seed_require_approved_target: true,
        observer_seed_rerun_comparison: true,
        observer_seed_compute_ledger_trend: true,
        observer_seed_recheck_readiness: true,
        observer_seed_emit_owner_summary: true,
        observer_approved_apply_governance_enabled: false,
        observer_approved_apply_mode: core::ObserverExplicitApplyMode::DryRun,
        observer_approved_apply_dry_run: true,
        observer_approved_target_store_input_path: None,
        observer_approved_target_store_output_path: None,
        observer_approved_apply_output_path: None,
        observer_approved_apply_recheck_readiness: true,
        chairman_governance_contract_prepare_enabled: true,
        chairman_governance_readiness_check_enabled: true,
        observer_approved_apply_emit_owner_summary: true,
        observer_apply_verify_chairman_shadow_enabled: false,
        observer_apply_verify_mode: core::ObserverExplicitApplyMode::DryRun,
        observer_apply_verify_dry_run: true,
        observer_apply_verify_target_store_output_path: None,
        observer_apply_verify_output_path: None,
        observer_apply_verify_config_path: None,
        observer_apply_verify_emit_owner_summary: true,
        chairman_shadow_governance_enabled: true,
        training_candidate_min_examples: None,
        self_growing_replay_enabled: false,
        research_source_registry_path: None,
        self_growing_max_tasks: 16,
        self_growing_max_evidence_records: 32,
        self_growing_allow_network_sources: false,
        research_evidence_output_path: None,
        self_growing_replay_output_path: None,
        emit_research_task_summary: false,
        self_growing_evidence_staging_enabled: false,
        self_growing_evidence_promotion_enabled: false,
        self_growing_evidence_promotion_dry_run: true,
        self_growing_evidence_apply_promotions: false,
        self_growing_refresh_training_candidates: false,
        self_growing_staging_store_path: None,
        self_growing_approved_evidence_output_path: None,
        self_growing_training_candidate_output_path: None,
        enriched_evidence_promotion_enabled: false,
        enriched_evidence_promotion_dry_run: true,
        enriched_evidence_apply_patch: false,
        enriched_evidence_apply_promotions: false,
        enriched_evidence_refresh_training_candidates: false,
        enriched_staging_output_path: None,
        enriched_approved_evidence_output_path: None,
        enriched_training_candidate_output_path: None,
        auto_approval_e2e_enabled: false,
        auto_approval_e2e_dry_run: true,
        auto_approval_success_staging_path: None,
        auto_approval_success_price_series_path: None,
        auto_approval_apply_promotions: false,
        auto_approval_refresh_training_candidates: false,
        auto_approval_approved_evidence_output_path: None,
        auto_approval_training_candidate_output_path: None,
        autonomous_paper_run: false,
        run_id: None,
        market_scopes: Vec::new(),
        symbols: Vec::new(),
        max_cycles: 1,
        cycle_mode: AutonomousPaperCycleMode::SingleShot,
        require_owner_confirmation: OwnerConfirmationPolicy::Never,
        local_market_data_path: None,
        local_news_path: None,
        news_collection_enabled: false,
        news_collection_config_path: None,
        news_provider_config_path: None,
        research_run_enabled: false,
        emit_research_run_summary: false,
        emit_research_packet_summary: false,
        research_auto_run_enabled: false,
        news_cache_input_path: None,
        news_cache_output_path: None,
        news_network_mode: NewsProviderRunMode::OfflineOnly,
        news_fetch_policy: None,
        rss_xml_fixture_path: None,
        rss_fetch_pilot_enabled: false,
        rss_fetch_pilot_url: None,
        rss_fetch_allowed_domains: Vec::new(),
        rss_fetch_source_label: None,
        rss_network_enabled: false,
        rss_safe_http_timeout_ms: 3_000,
        rss_safe_http_rate_limit_ms: 1_000,
        rss_safe_http_max_response_bytes: 262_144,
        rss_allowed_content_types: vec![
            "application/rss+xml".to_string(),
            "application/xml".to_string(),
            "text/xml".to_string(),
            "application/atom+xml".to_string(),
        ],
        rss_allow_redirects: false,
        rss_allow_missing_content_type: false,
        run_committee_cycle_from_research_packets: false,
        paper_only: true,
        owner_attention_inbox_input_path: None,
        owner_attention_inbox_output_path: None,
        owner_attention_actions_path: None,
        watchlist_candidate_input_path: None,
        watchlist_candidate_output_path: None,
        emit_owner_attention_inbox: false,
        enable_watchlist_recheck: false,
        watchlist_input_path: None,
        watchlist_output_path: None,
        max_candidates_per_cycle: 3,
        include_risk_blocked: false,
        include_needs_evidence: true,
        emit_owner_daily_brief: false,
        owner_daily_brief_store_input_path: None,
        owner_daily_brief_store_output_path: None,
        committee_state_snapshot_output_path: None,
        emit_committee_state_snapshot: false,
        committee_state_export_root_path: None,
        write_latest_snapshot: false,
        write_history_snapshot: false,
        write_snapshot_index: false,
        write_owner_console_read_model: false,
        committee_state_schema_version: None,
        max_snapshot_history_entries: None,
        inline_offline_member_opinions: Vec::new(),
        inline_input: None,
        pilot_roster: None,
        paper_outcome: None,
        archetype_style_cards_path: None,
        style_mapping_mode: StyleMappingMode::None,
    };
    let err = config
        .validate()
        .expect_err("remote offline fixture config must fail");
    assert!(err.contains("offline_member_opinion_path must be local"));

    let mut experience_config = single_cycle_config();
    experience_config.member_experience_store_output_path = Some("../bad.json".to_string());
    let err = experience_config
        .validate()
        .expect_err("experience output traversal must fail");
    assert!(err.contains("member_experience_store_output_path"));
    assert!(err.contains("parent-directory traversal"));

    let mut quality_config = single_cycle_config();
    quality_config.replay_quality_eval_output_path =
        Some("https://example.invalid/eval.json".to_string());
    let err = quality_config
        .validate()
        .expect_err("quality eval remote output must fail");
    assert!(err.contains("replay_quality_eval_output_path"));
    assert!(err.contains("must be local"));
}

#[test]
fn offline_member_output_batch_loads_rejects_unsafe_content_and_matches_packets() {
    let load = OfflineMemberOutputBatch::from_json_path(std::path::Path::new(
        "examples/minimal_offline_member_output_batch.sample.json",
    ))
    .expect("load offline output batch");
    assert_eq!(load.batch_id, "offline-batch-sprint130-sample");
    assert_eq!(load.loaded_count, 5);
    assert_eq!(load.invalid_count, 0);
    assert_eq!(load.duplicate_count, 0);
    assert_eq!(load.unmatched_count, 0);
    assert!(
        load.safety_notes
            .iter()
            .any(|note| note.contains("no network"))
    );

    let err = OfflineMemberOutputBatch::from_json_path(std::path::Path::new(
        "https://example.invalid/batch.json",
    ))
    .expect_err("remote batch path must fail");
    assert!(err.contains("must be local"));

    let unsafe_json = serde_json::json!({
        "batch_id": "unsafe",
        "created_at": "2026-05-22T17:43:38+09:00",
        "source_label": "local-test",
        "broker": "not allowed",
        "opinions": []
    })
    .to_string();
    let err =
        OfflineMemberOutputBatch::from_json_str(&unsafe_json).expect_err("broker field must fail");
    assert!(err.contains("unsafe field"));

    let unsafe_key_json = serde_json::json!({
        "batch_id": "unsafe-key",
        "created_at": "2026-05-22T17:43:38+09:00",
        "source_label": "local-test",
        "order_instruction": "not allowed",
        "opinions": []
    })
    .to_string();
    let err = OfflineMemberOutputBatch::from_json_str(&unsafe_key_json)
        .expect_err("order instruction field must fail");
    assert!(err.contains("unsafe field"));

    let unsafe_claim_json = serde_json::json!({
        "batch_id": "unsafe-claim",
        "created_at": "2026-05-22T17:43:38+09:00",
        "source_label": "local-test",
        "opinions": [{
            "member_id": "trend-kr-short",
            "symbol": "005930.KS",
            "market_scope": "KoreaShortTerm",
            "stance": "BuyProposal",
            "confidence": 0.7,
            "expected_return_hint": 0.02,
            "risk_hint": 0.03,
            "evidence_notes": ["guaranteed returns from a private strategy"],
            "event_triggered": true,
            "event_reason": "unsafe"
        }]
    })
    .to_string();
    let err = OfflineMemberOutputBatch::from_json_str(&unsafe_claim_json)
        .expect_err("guaranteed return/private strategy claim must fail");
    assert!(err.contains("unsafe claim"));

    let unsafe_impersonation_json = serde_json::json!({
        "batch_id": "unsafe-impersonation",
        "created_at": "2026-05-22T17:43:38+09:00",
        "source_label": "local-test",
        "opinions": [{
            "member_id": "trend-kr-short",
            "symbol": "005930.KS",
            "market_scope": "KoreaShortTerm",
            "stance": "BuyProposal",
            "confidence": 0.7,
            "expected_return_hint": 0.02,
            "risk_hint": 0.03,
            "evidence_notes": ["trades like Warren Buffett AI"],
            "event_triggered": true,
            "event_reason": "unsafe"
        }]
    })
    .to_string();
    let err = OfflineMemberOutputBatch::from_json_str(&unsafe_impersonation_json)
        .expect_err("impersonation claim must fail");
    assert!(err.contains("unsafe claim"));

    let adapter = OfflineMemberBrainAdapter {
        fixtures: load.opinions,
    };
    let packet = MemberInputPacket {
        member_id: "risk-kr-short".to_string(),
        market_data: serde_json::from_value(serde_json::json!({
            "symbol": "BTCUSDT",
            "market_scope": "CryptoShortTerm",
            "timestamp": "2026-05-21T00:00:00Z",
            "price": 108000.0,
            "change_pct": 4.8,
            "volume": 4100000000.0,
            "volatility_hint": 0.09,
            "source_label": "local-sample"
        }))
        .expect("market data"),
        news: Vec::new(),
        owner_context: Some("paper-only".to_string()),
        previous_member_score: Some(0.64),
    };
    let opinion = adapter.produce_opinion(&packet);
    assert_eq!(opinion.member_id, "risk-kr-short");
    assert_eq!(opinion.symbol, "BTCUSDT");
    assert_eq!(opinion.market_scope, MarketScope::CryptoShortTerm);
    assert_eq!(opinion.stance, MemberStance::NoTrade);
}

#[test]
fn batch_committee_cycle_routes_multi_symbol_events_and_preserves_safety() {
    let sample = std::fs::read_to_string("examples/minimal_ai_committee_multi_market_sample.json")
        .expect("batch input sample");
    let batch_input: BatchCommitteeCycleInput =
        serde_json::from_str(&sample).expect("parse batch input sample");
    assert_eq!(batch_input.market_data.len(), 6);

    let result = run_batch_committee_cycle_from_config_path(std::path::Path::new(
        "examples/soma_minimal_ai_committee_core.toml",
    ))
    .expect("batch cycle runs");
    assert_eq!(result.batch_id, "offline-batch-sprint130-sample");
    assert_eq!(result.routed_packet_count, 18);
    assert_eq!(result.member_opinion_count, 18);
    assert_eq!(result.event_queue.event_count, 5);
    assert_eq!(
        result.committee_sessions.len(),
        result.event_queue.event_count
    );
    assert_eq!(
        result.chairman_decisions.len(),
        result.event_queue.event_count
    );
    assert!(result.risk_veto_count >= 1);
    assert_eq!(result.score_update_count, result.score_updates.len());
    assert_eq!(
        result.learning_journal_entry_count,
        result.learning_journal_entries.len()
    );
    assert!(result.learning_journal_entry_count >= result.score_update_count);
    assert!(result.events_by_symbol.contains_key("005930.KS"));
    assert!(result.events_by_symbol.contains_key("BTCUSDT"));
    assert!(result.event_queue.symbols.contains(&"AAPL".to_string()));
    assert!(
        result
            .event_queue
            .market_scopes
            .contains(&MarketScope::CryptoShortTerm)
    );
    assert_eq!(
        result
            .event_queue
            .events
            .first()
            .expect("first event")
            .event_type,
        soma_zero::league::minimal_ai_committee_core::InvestmentEventType::RiskWarning
    );
    let highest = result
        .event_queue
        .highest_confidence_event()
        .expect("highest confidence event");
    assert!(highest.triggering_opinion.confidence >= 0.8);

    let missing_fallback = result
        .member_opinions
        .iter()
        .find(|opinion| opinion.symbol == "MSFT" && opinion.member_id == "trend-kr-short")
        .expect("missing offline opinion fallback");
    assert_eq!(missing_fallback.stance, MemberStance::NeedMoreEvidence);
    assert!(!missing_fallback.event_triggered);
    assert!(
        missing_fallback
            .evidence_notes
            .iter()
            .any(|note| note.contains("offline fixture missing"))
    );

    assert!(result.safety_summary.paper_only);
    assert!(result.safety_summary.no_real_order_path);
    assert!(result.safety_summary.no_broker_order_account);
    assert!(result.safety_summary.no_model_training);
    assert!(result.safety_summary.no_live_inference);
    assert_eq!(result.experience_record_count, result.member_opinion_count);
    assert_eq!(
        result.member_experience_records.len(),
        result.member_opinion_count
    );
    assert_eq!(
        result.replay_example_count,
        result.replay_dataset.example_count
    );
    assert!(
        result
            .member_experience_records
            .iter()
            .all(|record| record.paper_only)
    );
    assert!(result.replay_dataset.paper_only);
    assert!(
        result
            .paper_only_offline_learning_warning
            .contains("no training")
    );
    let mut opinion_member_ids: Vec<String> = result
        .member_opinions
        .iter()
        .map(|opinion| opinion.member_id.clone())
        .collect();
    opinion_member_ids.sort();
    opinion_member_ids.dedup();
    assert_eq!(result.smart_core_v2_specs.len(), opinion_member_ids.len());
    assert!(
        result
            .smart_core_v2_specs
            .iter()
            .all(|spec| spec.runtime_status == CoreRuntimeStatus::RuntimeDeferred)
    );
    let quality_eval = result
        .replay_quality_eval
        .as_ref()
        .expect("quality eval emitted");
    assert!(quality_eval.paper_only);
    assert_eq!(
        quality_eval.quality_summary.example_count,
        result.replay_dataset.example_count
    );
    assert_eq!(
        quality_eval.leakage_check.leakage_status,
        ReplayLeakageStatus::NoLeakageDetected
    );

    let second = run_batch_committee_cycle_from_config_path(std::path::Path::new(
        "examples/soma_minimal_ai_committee_core.toml",
    ))
    .expect("second batch cycle");
    assert_eq!(result, second);
}

#[test]
fn member_experience_records_capture_independent_paper_learning_memory() {
    let result = run_batch_committee_cycle_with_state_from_config_path(std::path::Path::new(
        "examples/soma_minimal_ai_committee_core.toml",
    ))
    .expect("stateful batch result");
    let owner_summary = result.owner_summary.as_ref().expect("owner summary");
    let extracted = extract_member_experiences_from_batch_cycle(
        &result.batch_result,
        Some(&result.state_update),
        Some(owner_summary),
        None,
    );

    assert_eq!(extracted.len(), result.batch_result.member_opinion_count);
    assert!(extracted.iter().all(|record| record.paper_only));
    assert!(
        extracted
            .iter()
            .all(|record| record.input_context.owner_context_summary.is_some())
    );
    assert!(
        extracted
            .iter()
            .all(|record| record.input_context.memory_state_summary.is_some())
    );
    assert!(extracted.iter().any(|record| {
        matches!(
            record.learning_label,
            MemberLearningLabel::Reinforce
                | MemberLearningLabel::CalibrateDown
                | MemberLearningLabel::NeedMoreEvidence
        )
    }));
    assert!(extracted.iter().any(|record| {
        matches!(
            record.outcome,
            MemberExperienceOutcome::RiskVetoSavedLoss
                | MemberExperienceOutcome::BadRiskCall
                | MemberExperienceOutcome::PaperPositive
                | MemberExperienceOutcome::PaperNegative
        )
    }));
    assert!(extracted.iter().any(|record| {
        record.member_id == "risk-kr-short"
            && record.attribution == MemberScoreUpdateReason::HelpfulDissent
            && record.outcome == MemberExperienceOutcome::RiskVetoSavedLoss
            && record.learning_label == MemberLearningLabel::Reinforce
    }));
    assert!(extracted.iter().any(|record| {
        record.member_id == "trend-kr-short"
            && record.attribution == MemberScoreUpdateReason::RiskyCall
            && record.outcome == MemberExperienceOutcome::BadRiskCall
            && record.learning_label == MemberLearningLabel::CalibrateDown
    }));
}

#[test]
fn replay_dataset_filter_builds_member_specific_offline_examples() {
    let result = run_batch_committee_cycle_from_config_path(std::path::Path::new(
        "examples/soma_minimal_ai_committee_core.toml",
    ))
    .expect("batch cycle result");
    let store = MemberExperienceStore::new(
        "unit-experience-store",
        result.member_experience_records.clone(),
    );
    assert_eq!(store.record_count, result.member_experience_records.len());
    assert!(!store.records_by_member("risk-kr-short").is_empty());
    assert!(!store.records_by_symbol("005930.KS").is_empty());
    assert!(store.recent_records("risk-kr-short", 1).len() <= 1);
    assert!(
        store
            .summarize_member_learning("risk-kr-short")
            .expect("risk member summary")
            .total_records
            > 0
    );
    let store_path = std::path::Path::new("target/unit_member_experience_store.json");
    store
        .save_to_local_json(store_path)
        .expect("save experience store");
    let loaded_store =
        MemberExperienceStore::load_from_local_json(store_path).expect("load experience store");
    assert_eq!(loaded_store, store);
    std::fs::remove_file(store_path).expect("remove experience store fixture");
    let unsafe_store_path = std::path::Path::new("target/unit_member_experience_store_unsafe.json");
    let mut unsafe_store_value = serde_json::to_value(&store).expect("store value");
    unsafe_store_value["records"][0]["paper_only"] = serde_json::Value::Bool(false);
    std::fs::write(
        unsafe_store_path,
        serde_json::to_string_pretty(&unsafe_store_value).expect("unsafe store json"),
    )
    .expect("write unsafe experience store");
    let err =
        MemberExperienceStore::load_from_local_json(unsafe_store_path).expect_err("unsafe record");
    assert!(err.contains("records must be paper-only"));
    std::fs::remove_file(unsafe_store_path).expect("remove unsafe experience store");
    let err = MemberExperienceStore::load_from_local_json(std::path::Path::new(
        "https://example.invalid/experience.json",
    ))
    .expect_err("remote experience store path must fail");
    assert!(err.contains("must be local"));
    let filter = ReplayDatasetFilter {
        member_id: Some("risk-kr-short".to_string()),
        market_scope: Some(MarketScope::KoreaShortTerm),
        symbol: None,
        min_confidence: Some(0.5),
        include_outcomes: Vec::new(),
        exclude_low_confidence_review_required: true,
    };
    let dataset = build_replay_dataset_from_experience_store(&store, &filter);

    assert!(dataset.paper_only);
    assert_eq!(dataset.generated_from_store_id, "unit-experience-store");
    assert!(dataset.example_count > 0);
    assert_eq!(dataset.example_count, dataset.examples.len());
    assert_eq!(store.export_replay_examples(&filter), dataset.examples);
    let replay_path = std::path::Path::new("target/unit_member_replay_dataset.json");
    dataset
        .save_to_local_json(replay_path)
        .expect("save replay dataset");
    let loaded = ReplayDataset::load_from_local_json(replay_path).expect("load replay dataset");
    assert_eq!(loaded, dataset);
    std::fs::remove_file(replay_path).expect("remove replay dataset fixture");
    let unsafe_replay_path = std::path::Path::new("target/unit_member_replay_dataset_unsafe.json");
    let mut unsafe_replay_value = serde_json::to_value(&dataset).expect("replay value");
    unsafe_replay_value["examples"][0]["paper_only"] = serde_json::Value::Bool(false);
    std::fs::write(
        unsafe_replay_path,
        serde_json::to_string_pretty(&unsafe_replay_value).expect("unsafe replay json"),
    )
    .expect("write unsafe replay dataset");
    let err = ReplayDataset::load_from_local_json(unsafe_replay_path).expect_err("unsafe replay");
    assert!(err.contains("examples must be paper-only"));
    std::fs::remove_file(unsafe_replay_path).expect("remove unsafe replay dataset");
    let err = ReplayDataset::load_from_local_json(std::path::Path::new(
        "https://example.invalid/replay.json",
    ))
    .expect_err("remote replay dataset path must fail");
    assert!(err.contains("must be local"));
    assert!(dataset.examples.iter().all(|example| {
        example.member_id == "risk-kr-short"
            && example.market_scope == MarketScope::KoreaShortTerm
            && example.paper_only
            && example.sample_weight >= 1.0
    }));
}

#[test]
fn calibration_summary_and_self_review_notes_are_deterministic() {
    let result = run_batch_committee_cycle_from_config_path(std::path::Path::new(
        "examples/soma_minimal_ai_committee_core.toml",
    ))
    .expect("batch cycle result");
    let store = MemberExperienceStore::new(
        "unit-experience-store",
        result.member_experience_records.clone(),
    );
    let first_summary = compute_member_calibration_summaries(&store);
    let second_summary = compute_member_calibration_summaries(&store);
    let single_summary =
        compute_member_calibration_summary(&store.records_by_member("risk-kr-short"));
    let first_notes = generate_member_self_review_notes(&store);
    let second_notes = generate_member_self_review_notes(&store);
    let deterministic_notes = generate_deterministic_self_review_notes(&store.records);

    assert_eq!(first_summary, second_summary);
    assert_eq!(first_notes, second_notes);
    assert_eq!(first_notes, deterministic_notes);
    assert_eq!(single_summary.member_id, "risk-kr-short");
    assert_eq!(first_summary.len(), store.member_count);
    assert!(
        first_summary
            .iter()
            .all(|summary| summary.total_records > 0)
    );
    assert!(first_notes.iter().all(|note| note.paper_only));
    assert!(first_notes.iter().any(|note| {
        note.next_behavior_hint.contains("evidence")
            || note.next_behavior_hint.contains("confidence")
            || note.next_behavior_hint.contains("risk")
    }));
}

#[test]
fn smart_core_v2_contract_is_deferred_and_preserves_member_independence() {
    let roster = create_three_member_pilot_roster(MarketScope::KoreaShortTerm);
    let specs = build_smart_core_v2_specs_for_members(&roster);

    assert_eq!(specs.len(), 3);
    assert!(specs.iter().all(|spec| {
        spec.runtime_status == CoreRuntimeStatus::RuntimeDeferred
            && !spec.replay_dataset_ready
            && spec.core_family == SmartCoreV2Family::Mamba3GatedDeltaNetSparseEvent
            && spec.temporal_core == SmartCoreV2ComponentStatus::Mamba3Deferred
            && spec.memory_core == SmartCoreV2ComponentStatus::GatedDeltaNetDeferred
            && spec.event_attention_core == SmartCoreV2ComponentStatus::SparseEventAttentionDeferred
            && spec.heads.calibration_head == SmartCoreV2ComponentStatus::Deferred
            && spec.heads.risk_head == SmartCoreV2ComponentStatus::Deferred
            && spec.heads.expected_return_head == SmartCoreV2ComponentStatus::Deferred
            && spec.heads.uncertainty_head == SmartCoreV2ComponentStatus::Deferred
            && spec
                .notes
                .iter()
                .any(|note| note.contains("no training or live inference"))
    }));
    let mut member_ids: Vec<String> = specs.iter().map(|spec| spec.member_id.clone()).collect();
    member_ids.sort();
    member_ids.dedup();
    assert_eq!(member_ids.len(), 3);
}

#[test]
fn replay_quality_summary_counts_distribution_and_coverage() {
    let result = run_batch_committee_cycle_from_config_path(std::path::Path::new(
        "examples/soma_minimal_ai_committee_core.toml",
    ))
    .expect("batch cycle result");
    let summary = evaluate_replay_dataset_quality(&result.replay_dataset);

    assert_eq!(summary.dataset_id, result.replay_dataset.dataset_id);
    assert_eq!(summary.example_count, result.replay_dataset.example_count);
    assert!(summary.member_count > 0);
    assert!(summary.symbols_covered.contains(&"005930.KS".to_string()));
    assert!(
        summary
            .market_scopes_covered
            .contains(&MarketScope::KoreaShortTerm)
    );
    assert!(summary.stance_distribution.values().sum::<usize>() == summary.example_count);
    assert!(summary.outcome_label_distribution.values().sum::<usize>() == summary.example_count);
    assert!(summary.paper_only);
    assert!(matches!(
        summary.quality_status,
        ReplayQualityStatus::Ready | ReplayQualityStatus::ReadyWithWarnings
    ));

    let mut stale_metadata = result.replay_dataset.clone();
    stale_metadata.member_count += 1;
    let stale_summary = evaluate_replay_dataset_quality(&stale_metadata);
    assert_eq!(stale_summary.member_count, summary.member_count);
    assert_eq!(
        stale_summary.quality_status,
        ReplayQualityStatus::UnsafeForTraining
    );
}

#[test]
fn replay_leakage_check_blocks_future_decision_and_broker_fields() {
    let result = run_batch_committee_cycle_from_config_path(std::path::Path::new(
        "examples/soma_minimal_ai_committee_core.toml",
    ))
    .expect("batch cycle result");
    let clean = check_replay_data_leakage(&result.replay_dataset);
    assert_eq!(clean.leakage_status, ReplayLeakageStatus::NoLeakageDetected);
    assert!(clean.leakage_findings.is_empty());

    let mut leaked = result.replay_dataset.clone();
    leaked.examples[0].input_features.market_data_summary =
        "future outcome PaperPositive target label".to_string();
    leaked.examples[0].input_features.news_summary =
        "chairman decision RiskVetoed broker account order".to_string();
    leaked.examples[0].input_features.memory_state_summary = Some("risk_veto=1".to_string());
    let check = check_replay_data_leakage(&leaked);

    assert_eq!(check.leakage_status, ReplayLeakageStatus::UnsafeForTraining);
    assert!(check.leakage_findings.iter().any(|finding| {
        finding.issue_type == ReplayLeakageIssueType::FutureOutcomeInInput
            && finding.severity == ReplayLeakageSeverity::Blocking
    }));
    assert!(
        check.leakage_findings.iter().any(|finding| {
            finding.issue_type == ReplayLeakageIssueType::ChairmanDecisionInInput
        })
    );
    assert!(check.leakage_findings.iter().any(|finding| {
        finding.issue_type == ReplayLeakageIssueType::BrokerAccountFieldDetected
    }));
    assert!(check.leakage_findings.iter().any(|finding| {
        finding.issue_type == ReplayLeakageIssueType::RiskVetoInInput
            && finding.field_name == "memory_state_summary"
    }));
}

#[test]
fn member_calibration_eval_detects_overconfidence_and_insufficient_data() {
    let result = run_batch_committee_cycle_from_config_path(std::path::Path::new(
        "examples/soma_minimal_ai_committee_core.toml",
    ))
    .expect("batch cycle result");
    let store = MemberExperienceStore::new(
        "calibration-eval-store",
        result.member_experience_records.clone(),
    );
    let mut trend_records = store.records_by_member("trend-kr-short");
    for record in &mut trend_records {
        record.member_opinion.confidence = 0.92;
        record.outcome = MemberExperienceOutcome::PaperNegative;
    }
    let overconfident = evaluate_member_calibration(&trend_records, "trend-kr-short");
    assert_eq!(
        overconfident.calibration_status,
        MemberCalibrationEvalStatus::Overconfident
    );
    assert_eq!(
        overconfident.suggested_adjustment,
        MemberCalibrationEvalSuggestedAdjustment::LowerConfidence
    );
    assert!(overconfident.overconfidence_rate > 0.0);

    let insufficient = evaluate_member_calibration(&trend_records[0..1], "trend-kr-short");
    assert_eq!(
        insufficient.calibration_status,
        MemberCalibrationEvalStatus::InsufficientData
    );
}

#[test]
fn risk_governor_alignment_and_readiness_gate_are_paper_only() {
    let result = run_batch_committee_cycle_from_config_path(std::path::Path::new(
        "examples/soma_minimal_ai_committee_core.toml",
    ))
    .expect("batch cycle result");
    let store = MemberExperienceStore::new(
        "readiness-eval-store",
        result.member_experience_records.clone(),
    );
    let risk_eval = evaluate_risk_governor_alignment(&store);
    assert!(risk_eval.paper_only);
    assert!(risk_eval.total_vetoes > 0);
    assert!(risk_eval.helpful_veto_rate >= 0.0);
    assert!(matches!(
        risk_eval.risk_alignment_status,
        RiskAlignmentStatus::Aligned
            | RiskAlignmentStatus::PossiblyTooStrict
            | RiskAlignmentStatus::PossiblyTooLoose
            | RiskAlignmentStatus::InsufficientData
    ));

    let ready_with_warnings =
        evaluate_offline_training_readiness_with_thresholds(&result.replay_dataset, &store, 5, 1);
    assert!(ready_with_warnings.paper_only);
    assert!(ready_with_warnings.ready_for_offline_training);
    assert!(
        matches!(
            ready_with_warnings.readiness_status,
            OfflineTrainingReadinessStatus::ReadyForOfflineTraining
                | OfflineTrainingReadinessStatus::ReadyWithWarnings
        ),
        "Sprint 160 closure should leave this fixture ready for offline design checks"
    );
    assert_eq!(
        ready_with_warnings.label_source_status,
        ReplayCoverageTargetStatus::Met
    );
    assert!(
        ready_with_warnings.validated_label_ratio
            >= ready_with_warnings.min_validated_label_ratio_required
    );

    let mut leaked = result.replay_dataset.clone();
    leaked.examples[0].input_features.market_data_summary =
        "future outcome copied into feature".to_string();
    let blocked = evaluate_offline_training_readiness(&leaked, &store);
    assert_eq!(
        blocked.readiness_status,
        OfflineTrainingReadinessStatus::BlockedByLeakage
    );

    let mut tiny = result.replay_dataset.clone();
    tiny.examples.truncate(1);
    tiny.example_count = tiny.examples.len();
    tiny.member_count = 1;
    let needs_more = evaluate_offline_training_readiness(&tiny, &store);
    assert_eq!(
        needs_more.readiness_status,
        OfflineTrainingReadinessStatus::NeedsMoreData
    );
}

#[test]
fn risk_governor_alignment_does_not_count_unvetoed_bad_risk_as_helpful_veto() {
    let result = run_batch_committee_cycle_from_config_path(std::path::Path::new(
        "examples/soma_minimal_ai_committee_core.toml",
    ))
    .expect("batch cycle result");
    let mut bad_risk_call = result.member_experience_records[0].clone();
    bad_risk_call.committee_context.risk_governor_status = Some(RiskGovernorStatus::Passed);
    bad_risk_call.outcome = MemberExperienceOutcome::BadRiskCall;
    let store = MemberExperienceStore::new("unvetoed-risk-eval-store", vec![bad_risk_call]);

    let risk_eval = evaluate_risk_governor_alignment(&store);

    assert_eq!(risk_eval.total_vetoes, 0);
    assert_eq!(risk_eval.veto_followed_by_negative_outcome_count, 0);
    assert_eq!(
        risk_eval.risk_alignment_status,
        RiskAlignmentStatus::PossiblyTooLoose
    );
    assert!(risk_eval.notes.iter().any(|note| {
        note.contains("bad risk call") && note.contains("not Risk Governor vetoes")
    }));
}

#[test]
fn quality_eval_result_and_improvement_plans_are_deterministic() {
    let result = run_batch_committee_cycle_from_config_path(std::path::Path::new(
        "examples/soma_minimal_ai_committee_core.toml",
    ))
    .expect("batch cycle result");
    let store = MemberExperienceStore::new(
        "quality-eval-store",
        result.member_experience_records.clone(),
    );
    let first = run_replay_quality_evaluation_with_thresholds(&store, &result.replay_dataset, 5, 1);
    let second =
        run_replay_quality_evaluation_with_thresholds(&store, &result.replay_dataset, 5, 1);
    assert_eq!(first, second);
    assert!(first.paper_only);
    assert!(first.safety_summary.no_model_training);
    assert!(first.safety_summary.no_live_inference);
    assert_eq!(
        first.leakage_check.leakage_status,
        ReplayLeakageStatus::NoLeakageDetected
    );
    assert!(!first.member_improvement_plans.is_empty());
    assert!(
        first
            .offline_training_readiness_gate
            .recommended_next_data
            .iter()
            .all(|item| matches!(
                item,
                RecommendedNextData::MoreRiskVetoCases
                    | RecommendedNextData::MorePositiveOutcomes
                    | RecommendedNextData::MoreNegativeOutcomes
                    | RecommendedNextData::MoreNeedMoreEvidenceCases
                    | RecommendedNextData::MoreMarketScopes
                    | RecommendedNextData::MoreSymbols
                    | RecommendedNextData::MoreLongHorizonCases
                    | RecommendedNextData::MoreCryptoVolatilityCases
            ))
    );

    let mut overconfident_eval =
        evaluate_member_calibration(&store.records_by_member("trend-kr-short"), "trend-kr-short");
    overconfident_eval.calibration_status = MemberCalibrationEvalStatus::Overconfident;
    overconfident_eval.suggested_adjustment =
        MemberCalibrationEvalSuggestedAdjustment::LowerConfidence;
    let plan =
        build_member_improvement_plan(&overconfident_eval, &first.offline_training_readiness_gate);
    assert!(
        plan.suggested_behavior_adjustments
            .iter()
            .any(|item| item.contains("lower confidence"))
    );
    assert!(
        plan.data_collection_needs
            .contains(&RecommendedNextData::MoreRiskVetoCases)
    );
    assert!(
        plan.core_v2_notes
            .calibration_head_notes
            .iter()
            .any(|note| { note.contains("validated confidence/outcome labels") })
    );

    let default_eval = run_replay_quality_evaluation(&store, &result.replay_dataset);
    assert_eq!(
        default_eval.quality_summary.dataset_id,
        result.replay_dataset.dataset_id
    );
    let output_path = std::path::Path::new("target/unit_replay_quality_eval.json");
    first
        .save_to_local_json(output_path)
        .expect("save quality eval");
    std::fs::remove_file(output_path).expect("remove quality eval");
    let err = first
        .save_to_local_json(std::path::Path::new("https://example.invalid/eval.json"))
        .expect_err("remote quality eval output must fail");
    assert!(err.contains("must be local"));
}

#[test]
fn replay_feature_sanitizer_removes_temporal_leakage_terms() {
    let sanitizer = ReplayFeatureSanitizer::default();
    let field = sanitizer.sanitize_input_text(
        "owner_context_summary",
        "RiskVeto risk_veto ChairmanDecision chairman_action outcome learning_label",
    );

    assert_eq!(field.safety_status, SanitizedFieldSafetyStatus::Sanitized);
    assert!(field.removed_terms.iter().any(|term| term == "RiskVeto"));
    assert!(field.removed_terms.iter().any(|term| term == "risk_veto"));
    assert!(!field.sanitized_text.contains("RiskVeto"));
    assert!(!field.sanitized_text.contains("chairman_action"));
    assert!(!field.sanitized_text.contains("outcome"));

    let blocking = sanitizer.sanitize_input_text("owner_context_summary", "broker account order");
    assert_eq!(
        blocking.safety_status,
        SanitizedFieldSafetyStatus::BlockingLeakage
    );

    let policy = TemporalFeatureBoundaryPolicy::default();
    assert!(policy.forbid_chairman_decision_in_input);
    assert!(policy.forbid_risk_veto_in_input);
    assert_eq!(
        DecisionTimelinePhase::PreDecisionInput,
        DecisionTimelinePhase::PreDecisionInput
    );
}

#[test]
fn pre_decision_memory_snapshot_uses_buckets_not_raw_outcome_text() {
    let memory = MemberMemoryState {
        member_id: "trend-kr-short".to_string(),
        recent_symbols: vec!["005930.KS".to_string()],
        recent_opinion_count: 7,
        recent_event_count: 4,
        recent_good_call_count: 6,
        recent_bad_call_count: 2,
        recent_risk_veto_count: 1,
        notes: vec![
            "PaperPositive outcome should not be copied".to_string(),
            "NeedMoreEvidence tendency".to_string(),
        ],
    };
    let snapshot = build_pre_decision_memory_snapshot(&memory, Some("cycle-1"));
    let serialized = serde_json::to_string(&snapshot).expect("snapshot json");

    assert_eq!(snapshot.member_id, "trend-kr-short");
    assert_eq!(
        snapshot.recent_good_call_count_bucket,
        MemoryCountBucket::High
    );
    assert_eq!(
        snapshot.recent_bad_call_count_bucket,
        MemoryCountBucket::Low
    );
    assert_eq!(
        snapshot.recent_risk_veto_count_bucket,
        MemoryCountBucket::Low
    );
    assert!(!serialized.contains("PaperPositive"));
    assert!(!serialized.contains("outcome should not be copied"));
    assert!(snapshot.paper_only);
}

#[test]
fn sanitized_replay_dataset_separates_inputs_targets_and_post_decision_context() {
    let stateful = run_batch_committee_cycle_with_state_from_config_path(std::path::Path::new(
        "examples/soma_minimal_ai_committee_core.toml",
    ))
    .expect("stateful batch result");
    let store = MemberExperienceStore::new(
        "sanitize-store",
        stateful.batch_result.member_experience_records.clone(),
    );
    let raw_dataset =
        build_replay_dataset_from_experience_store(&store, &ReplayDatasetFilter::default());
    let raw_leakage = check_replay_data_leakage(&raw_dataset);
    assert!(matches!(
        raw_leakage.leakage_status,
        ReplayLeakageStatus::LeakageDetected | ReplayLeakageStatus::UnsafeForTraining
    ));
    assert!(
        raw_leakage
            .leakage_findings
            .iter()
            .any(|finding| finding.section == ReplaySection::InputFeatures)
    );

    let build = build_sanitized_replay_dataset_from_experience_store(
        &store,
        SanitizedReplayDatasetBuildConfig {
            source_experience_store_id: "sanitize-store".to_string(),
            strict_temporal_boundary: true,
            include_post_decision_context_for_audit: true,
            include_prior_memory_buckets: true,
            reject_on_blocking_leakage: true,
            paper_only: true,
        },
    );
    assert!(matches!(
        build.build_status,
        SanitizedReplayDatasetBuildStatus::Built
            | SanitizedReplayDatasetBuildStatus::BuiltWithWarnings
    ));
    assert!(build.sanitized_count > 0);
    assert_eq!(build.rejected_count, 0);
    assert!(build.dataset.paper_only);
    assert!(build.dataset.examples.iter().all(|example| {
        let feature_notes = example
            .sanitized_input_features
            .as_ref()
            .map(|features| features.feature_safety_notes.join(" ").to_ascii_lowercase())
            .unwrap_or_default();
        example.sanitized_input_features.is_some()
            && example.target_labels.is_some()
            && example.post_decision_context.is_some()
            && !feature_notes.contains("riskveto")
            && !feature_notes.contains("risk_veto")
            && !feature_notes.contains("chairman")
            && !feature_notes.contains("outcome")
            && !feature_notes.contains("learning_label")
            && !feature_notes.contains("broker")
            && !feature_notes.contains("account")
            && !feature_notes.contains("order")
            && !example
                .input_features
                .owner_context_summary
                .clone()
                .unwrap_or_default()
                .to_ascii_lowercase()
                .contains("riskveto")
            && !example
                .input_features
                .memory_state_summary
                .clone()
                .unwrap_or_default()
                .to_ascii_lowercase()
                .contains("risk_veto")
    }));
    assert!(build.dataset.examples.iter().any(|example| {
        example.target_labels.as_ref().is_some_and(|labels| {
            matches!(
                labels.outcome_label,
                MemberExperienceOutcome::RiskVetoSavedLoss
                    | MemberExperienceOutcome::BadRiskCall
                    | MemberExperienceOutcome::PaperPositive
            )
        })
    }));
    let sanitized_leakage = check_replay_data_leakage(&build.dataset);
    assert_eq!(
        sanitized_leakage.leakage_status,
        ReplayLeakageStatus::NoLeakageDetected
    );
}

#[test]
fn leakage_checker_allows_separated_post_decision_context_but_blocks_input() {
    let stateful = run_batch_committee_cycle_with_state_from_config_path(std::path::Path::new(
        "examples/soma_minimal_ai_committee_core.toml",
    ))
    .expect("stateful batch result");
    let store = MemberExperienceStore::new(
        "section-store",
        stateful.batch_result.member_experience_records.clone(),
    );
    let mut dataset = build_sanitized_replay_dataset_from_experience_store(
        &store,
        SanitizedReplayDatasetBuildConfig {
            source_experience_store_id: "section-store".to_string(),
            strict_temporal_boundary: true,
            include_post_decision_context_for_audit: true,
            include_prior_memory_buckets: true,
            reject_on_blocking_leakage: true,
            paper_only: true,
        },
    )
    .dataset;
    dataset.examples[0].post_decision_context = Some(ReplayPostDecisionContext {
        chairman_action: Some(ChairmanFinalAction::RiskVetoed),
        risk_governor_status: Some(RiskGovernorStatus::Vetoed),
        disagreement_level: 0.5,
        other_member_stances: vec!["RiskVeto ChairmanDecision outcome".to_string()],
        risk_flags: vec!["risk_veto".to_string()],
        paper_only: true,
    });
    assert_eq!(
        check_replay_data_leakage(&dataset).leakage_status,
        ReplayLeakageStatus::NoLeakageDetected
    );

    dataset.examples[0].input_features.news_summary =
        "risk_veto ChairmanDecision outcome target label".to_string();
    let check = check_replay_data_leakage(&dataset);
    assert!(matches!(
        check.leakage_status,
        ReplayLeakageStatus::LeakageDetected | ReplayLeakageStatus::UnsafeForTraining
    ));
    assert!(check.leakage_findings.iter().any(|finding| {
        finding.section == ReplaySection::InputFeatures
            && finding.issue_type == ReplayLeakageIssueType::RiskVetoInInput
    }));
}

#[test]
fn sanitized_readiness_recheck_resolves_leakage_block_without_fake_ready() {
    let stateful = run_batch_committee_cycle_with_state_from_config_path(std::path::Path::new(
        "examples/soma_minimal_ai_committee_core.toml",
    ))
    .expect("stateful batch result");
    let store = MemberExperienceStore::new(
        "readiness-sanitize-store",
        stateful.batch_result.member_experience_records.clone(),
    );
    let raw_dataset =
        build_replay_dataset_from_experience_store(&store, &ReplayDatasetFilter::default());
    let raw_gate = evaluate_offline_training_readiness(&raw_dataset, &store);
    assert_eq!(
        raw_gate.readiness_status,
        OfflineTrainingReadinessStatus::BlockedByLeakage
    );

    let sanitized = build_sanitized_replay_dataset_from_experience_store(
        &store,
        SanitizedReplayDatasetBuildConfig {
            source_experience_store_id: "readiness-sanitize-store".to_string(),
            strict_temporal_boundary: true,
            include_post_decision_context_for_audit: true,
            include_prior_memory_buckets: true,
            reject_on_blocking_leakage: true,
            paper_only: true,
        },
    );
    let sanitized_gate =
        evaluate_offline_training_readiness_with_thresholds(&sanitized.dataset, &store, 20, 3);
    assert_ne!(
        sanitized_gate.readiness_status,
        OfflineTrainingReadinessStatus::BlockedByLeakage
    );
    assert!(matches!(
        sanitized_gate.readiness_status,
        OfflineTrainingReadinessStatus::NeedsMoreData
            | OfflineTrainingReadinessStatus::ReadyWithWarnings
            | OfflineTrainingReadinessStatus::ReadyForOfflineTraining
    ));
    if sanitized_gate.actual_example_count < 20 {
        assert_eq!(
            sanitized_gate.readiness_status,
            OfflineTrainingReadinessStatus::NeedsMoreData
        );
    }
}

#[test]
fn replay_coverage_matrix_targets_and_queue_are_deterministic() {
    let stateful = run_batch_committee_cycle_with_state_from_config_path(std::path::Path::new(
        "examples/soma_minimal_ai_committee_core.toml",
    ))
    .expect("stateful batch result");
    let dataset = &stateful.batch_result.replay_dataset;

    let matrix = compute_replay_coverage_matrix(dataset);
    assert_eq!(matrix.dataset_id, dataset.dataset_id);
    assert_eq!(matrix.total_examples, dataset.example_count);
    assert!(matrix.by_member_id.contains_key("trend-kr-short"));
    assert!(matrix.by_event_type.values().sum::<usize>() >= dataset.example_count);
    assert!(matrix.weak_label_count < dataset.example_count);
    assert!(matrix.weak_label_count > 0);

    let target_config = ReplayCoverageTargetConfig {
        min_examples_total: dataset.example_count + 10,
        min_total_examples: dataset.example_count + 10,
        min_examples_per_member: 1,
        min_examples_per_market_scope: 1,
        min_examples_per_event_type: 1,
        require_non_weak_label_source: true,
        allowed_weak_label_ratio: 0.25,
        paper_only: true,
        ..ReplayCoverageTargetConfig::default()
    };
    let gaps = detect_sparse_coverage_cells(dataset, &target_config);
    assert!(gaps.iter().any(|gap| gap.gap_id == "gap-total-examples"));

    let eval = evaluate_replay_coverage_targets(dataset, target_config);
    assert_eq!(
        eval.target_status,
        ReplayCoverageTargetStatus::NeedsCoverage
    );
    assert!(
        eval.gaps
            .iter()
            .all(|gap| gap.gap_id != "gap-label-source-weak")
    );
    assert_eq!(eval.coverage_gap_count, eval.gaps.len());
    assert!(eval.collection_queue_recommended);

    let first_queue = build_collection_queue_from_gaps("coverage-queue", dataset, &eval.gaps);
    let second_queue = build_collection_queue_from_gaps("coverage-queue", dataset, &eval.gaps);
    assert_eq!(first_queue, second_queue);
    assert_eq!(first_queue.item_count, eval.gaps.len());
    assert!(first_queue.paper_only);
    assert!(
        first_queue
            .items
            .iter()
            .all(|item| item.label_source == ReplayLabelSource::ReviewRequired)
    );
}

#[test]
fn paper_scenario_fixture_label_source_is_preserved() {
    let stateful = run_batch_committee_cycle_with_state_from_config_path(std::path::Path::new(
        "examples/soma_minimal_ai_committee_core.toml",
    ))
    .expect("stateful batch result");
    let dataset = &stateful.batch_result.replay_dataset;
    let eval = evaluate_replay_coverage_targets(
        dataset,
        ReplayCoverageTargetConfig {
            min_examples_total: dataset.example_count + 10,
            min_total_examples: dataset.example_count + 10,
            min_examples_per_member: 1,
            min_examples_per_market_scope: 1,
            min_examples_per_event_type: 1,
            require_non_weak_label_source: false,
            paper_only: true,
            ..ReplayCoverageTargetConfig::default()
        },
    );
    let mut queue = build_collection_queue_from_gaps("fixture-label-queue", dataset, &eval.gaps);
    queue.items = queue.next_items(1);
    queue.item_count = queue.items.len();
    let item = queue.items.first().expect("queue item").clone();
    let fixture = PaperOutcomeFixture {
        fixture_id: "manual-fixture-1".to_string(),
        symbol: item
            .symbol
            .clone()
            .unwrap_or_else(|| "PAPER-SCENARIO".to_string()),
        market_scope: item.market_scope.unwrap_or(MarketScope::KoreaShortTerm),
        outcome: SimulatedPaperOutcome::Positive,
        simulated_result: SimulatedPaperOutcome::Positive,
        label_source: ReplayLabelSource::ManualPaperLabel,
        confidence: ReplayLabelConfidence::ReviewRequired,
        label_confidence: ReplayLabelConfidence::ReviewRequired,
        note: "manual paper label still requires review".to_string(),
        notes: Vec::new(),
        paper_only: true,
    };

    let scenario = run_paper_scenario_collection(PaperScenarioRunConfig {
        scenario_run_id: "unit-paper-scenario-fixture-label-run".to_string(),
        collection_items: queue.items.clone(),
        collection_queue: queue,
        market_data_path: None,
        news_path: None,
        news_cache_path: None,
        offline_member_output_batch_path: None,
        member_state_input_path: None,
        paper_outcome_fixture_path: None,
        max_items: None,
        sanitize_after_run: true,
        outcome_fixtures: vec![fixture],
        paper_only: true,
    })
    .expect("paper scenario run");

    assert!(scenario.replay_dataset.examples.iter().all(|example| {
        example
            .target_labels
            .as_ref()
            .map(|labels| {
                labels.label_source == ReplayLabelSource::ManualPaperLabel
                    && labels.label_confidence == ReplayLabelConfidence::ReviewRequired
            })
            .unwrap_or(false)
    }));
}

#[test]
fn paper_scenario_collection_generates_weak_labels_and_blocks_ready_state() {
    let stateful = run_batch_committee_cycle_with_state_from_config_path(std::path::Path::new(
        "examples/soma_minimal_ai_committee_core.toml",
    ))
    .expect("stateful batch result");
    let dataset = &stateful.batch_result.replay_dataset;
    let eval = evaluate_replay_coverage_targets(
        dataset,
        ReplayCoverageTargetConfig {
            min_examples_total: dataset.example_count + 100,
            min_total_examples: dataset.example_count + 100,
            min_examples_per_member: 1,
            min_examples_per_market_scope: 1,
            min_examples_per_event_type: 1,
            require_non_weak_label_source: true,
            allowed_weak_label_ratio: 0.25,
            paper_only: true,
            ..ReplayCoverageTargetConfig::default()
        },
    );
    let mut queue = build_collection_queue_from_gaps("scenario-run-queue", dataset, &eval.gaps);
    queue.items = queue.next_items(1);
    queue.item_count = queue.items.len();
    queue.high_priority_count = queue
        .items
        .iter()
        .filter(|item| {
            item.priority
                == soma_zero::league::minimal_ai_committee_core::ReplayCoverageGapPriority::High
        })
        .count();

    let scenario = run_paper_scenario_collection(PaperScenarioRunConfig {
        scenario_run_id: "unit-paper-scenario-run".to_string(),
        collection_items: queue.items.clone(),
        collection_queue: queue,
        market_data_path: None,
        news_path: None,
        news_cache_path: None,
        offline_member_output_batch_path: None,
        member_state_input_path: None,
        paper_outcome_fixture_path: None,
        max_items: None,
        sanitize_after_run: true,
        outcome_fixtures: Vec::new(),
        paper_only: true,
    })
    .expect("paper scenario run");

    assert!(scenario.paper_only);
    assert!(scenario.safety_summary.no_broker_order_account);
    assert_eq!(scenario.generated_batch_count, 1);
    assert!(scenario.generated_experience_record_count > 0);
    assert!(scenario.replay_dataset.examples.iter().all(|example| {
        example
            .target_labels
            .as_ref()
            .map(|labels| {
                labels.label_source == ReplayLabelSource::SimulatedFixture
                    && labels.label_confidence == ReplayLabelConfidence::Medium
            })
            .unwrap_or(false)
    }));
    assert_eq!(
        scenario
            .replay_quality_eval
            .offline_training_readiness_gate
            .label_source_status,
        ReplayCoverageTargetStatus::BlockedByWeakLabels
    );
    assert_ne!(
        scenario
            .replay_quality_eval
            .offline_training_readiness_gate
            .readiness_status,
        OfflineTrainingReadinessStatus::ReadyForOfflineTraining
    );
}

fn sample_label_evidence(
    replay_id: Option<String>,
    candidate_label: MemberExperienceOutcome,
    candidate_source: ReplayLabelSource,
    price_move: Option<f64>,
) -> LabelEvidenceRecord {
    LabelEvidenceRecord {
        evidence_id: "label-evidence-unit".to_string(),
        replay_id,
        experience_id: None,
        symbol: "005930.KS".to_string(),
        market_scope: MarketScope::KoreaShortTerm,
        decision_id: Some("decision-unit".to_string()),
        candidate_label,
        candidate_source,
        evidence_items: LabelEvidenceItems {
            price_move_evidence: price_move,
            risk_veto_evidence: Some(RiskGovernorStatus::Passed),
            news_context_evidence: Some("paper-only local summary".to_string()),
            decision_trace_evidence: Some("paper decision trace".to_string()),
            paper_horizon_evidence: Some("5 bars".to_string()),
        },
        validation_status: LabelEvidenceValidationStatus::Unchecked,
        confidence: ReplayLabelConfidence::High,
        validator_notes: vec!["unit paper evidence".to_string()],
        paper_only: true,
    }
}

fn ready_backtest_contract() -> BacktestLabelContract {
    BacktestLabelContract {
        contract_id: "backtest-contract-unit".to_string(),
        horizon: BacktestLabelHorizon::ShortTerm,
        entry_price_source: BacktestEntryPriceSource::DecisionTimestampClose,
        exit_price_source: BacktestExitPriceSource::HorizonClose,
        slippage_model: BacktestCostModel::FixedBps,
        fee_model: BacktestCostModel::FixedBps,
        leakage_guard: vec![
            BacktestLeakageGuard::NoFutureInput,
            BacktestLeakageGuard::NoPostDecisionFeature,
        ],
        paper_only: true,
    }
}

fn sprint156_replay_example(
    replay_id: &str,
    symbol: &str,
    market_scope: MarketScope,
) -> ReplayExample {
    ReplayExample {
        replay_id: replay_id.to_string(),
        member_id: "TrendEntryAI".to_string(),
        symbol: symbol.to_string(),
        market_scope,
        input_features: ReplayInputFeatures {
            market_data_summary: format!("{symbol} pre-decision market summary"),
            news_summary: "local paper-only news summary".to_string(),
            owner_context_summary: Some("owner context before decision".to_string()),
            memory_state_summary: Some("member memory before decision".to_string()),
        },
        target: ReplayTarget {
            stance: MemberStance::BuyProposal,
            confidence_calibration: MemberLearningLabel::Reinforce,
            risk_label: "risk_passed".to_string(),
            evidence_label: "paper_evidence_pending".to_string(),
            outcome_label: MemberExperienceOutcome::Unknown,
        },
        sanitized_input_features: None,
        target_labels: Some(ReplayTargetLabels {
            stance_target: MemberStance::BuyProposal,
            confidence_calibration_target: MemberLearningLabel::Reinforce,
            risk_label: "risk_passed".to_string(),
            evidence_label: "paper_evidence_pending".to_string(),
            outcome_label: MemberExperienceOutcome::Unknown,
            attribution_label: MemberScoreUpdateReason::Neutral,
            label_source: ReplayLabelSource::ReviewRequired,
            label_confidence: ReplayLabelConfidence::ReviewRequired,
            paper_only: true,
        }),
        post_decision_context: None,
        sample_weight: 1.0,
        paper_only: true,
    }
}

fn sprint156_replay_dataset(examples: Vec<ReplayExample>) -> ReplayDataset {
    ReplayDataset {
        dataset_id: "sprint156-paper-evidence-dataset".to_string(),
        example_count: examples.len(),
        member_count: 1,
        examples,
        generated_from_store_id: "sprint156-paper-evidence-store".to_string(),
        paper_only: true,
    }
}

fn sprint156_paper_evidence(
    evidence_id: &str,
    replay_id: Option<&str>,
    symbol: &str,
    market_scope: MarketScope,
    candidate_label: MemberExperienceOutcome,
    label_source: ReplayLabelSource,
    confidence: ReplayLabelConfidence,
    price_change_pct: Option<f64>,
) -> PaperOutcomeEvidenceRecord {
    PaperOutcomeEvidenceRecord {
        evidence_id: evidence_id.to_string(),
        symbol: symbol.to_string(),
        market_scope,
        decision_id: None,
        event_id: None,
        replay_id: replay_id.map(str::to_string),
        experience_id: None,
        horizon: PaperOutcomeEvidenceHorizon::ShortTerm,
        horizon_bars: Some(5),
        reference_price: Some(100.0),
        horizon_price: price_change_pct.map(|change| 100.0 * (1.0 + change)),
        price_change_pct,
        candidate_label,
        label_source,
        label_confidence: confidence,
        evidence_notes: vec!["paper decision trace reviewed locally".to_string()],
        validation_hint: Some(PaperOutcomeEvidenceValidationHint::PromoteIfPolicyPasses),
        paper_only: true,
    }
}

fn sprint158_price_series(
    symbol: &str,
    market_scope: MarketScope,
    closes: &[f64],
) -> PaperPriceSeries {
    PaperPriceSeries {
        series_id: format!("series-{symbol}-{market_scope:?}"),
        symbol: symbol.to_string(),
        market_scope,
        source_label: "unit-local-price-series".to_string(),
        bars: closes
            .iter()
            .enumerate()
            .map(|(index, close)| PaperPriceBar {
                symbol: symbol.to_string(),
                market_scope,
                timestamp: format!("2026-05-2{:01}T09:00:00Z", index),
                open: Some(*close),
                high: Some(*close),
                low: Some(*close),
                close: *close,
                volume: Some(1_000.0 + index as f64),
                source_label: "unit-local-price-series".to_string(),
                paper_only: true,
            })
            .collect(),
        paper_only: true,
    }
}

fn sprint158_price_series_store() -> PaperPriceSeriesStore {
    PaperPriceSeriesStore {
        store_id: "sprint158-price-series-store".to_string(),
        series: vec![
            sprint158_price_series("AAPL", MarketScope::UsShortTerm, &[100.0, 101.0, 103.0]),
            sprint158_price_series(
                "MSFT",
                MarketScope::UsLongTerm,
                &[100.0, 99.0, 98.0, 97.0, 94.0],
            ),
            sprint158_price_series(
                "BTCUSDT",
                MarketScope::CryptoShortTerm,
                &[100.0, 100.2, 100.1],
            ),
            sprint158_price_series("ETHUSDT", MarketScope::CryptoLongTerm, &[100.0, 100.5]),
        ],
        paper_only: true,
    }
}

fn sprint158_experience_store() -> MemberExperienceStore {
    MemberExperienceStore::new(
        "sprint158-experience-store",
        vec![MemberExperienceRecord {
            experience_id: "example-aapl".to_string(),
            member_id: "us-short-trend".to_string(),
            symbol: "AAPL".to_string(),
            market_scope: MarketScope::UsShortTerm,
            cycle_id: None,
            event_id: None,
            session_id: None,
            decision_id: Some("decision-aapl".to_string()),
            input_context: MemberExperienceInputContext {
                market_data_summary: "AAPL market".to_string(),
                news_summary: "AAPL news".to_string(),
                owner_context_summary: None,
                style_blend_summary: None,
                memory_state_summary: None,
            },
            member_opinion: MemberExperienceOpinionSnapshot {
                stance: MemberStance::BuyProposal,
                confidence: 0.7,
                expected_return_hint: 0.02,
                risk_hint: 0.2,
                evidence_notes: vec!["decision trace".to_string()],
                event_triggered: true,
            },
            committee_context: MemberExperienceCommitteeContext {
                disagreement_level: 0.0,
                other_member_stances: Vec::new(),
                chairman_action: None,
                risk_governor_status: None,
                risk_flags: Vec::new(),
            },
            outcome: MemberExperienceOutcome::Unknown,
            attribution: MemberScoreUpdateReason::Neutral,
            learning_label: MemberLearningLabel::Reinforce,
            created_at: Some("2026-05-20T09:00:00Z".to_string()),
            paper_only: true,
        }],
    )
}

fn sprint159_replay_example(
    replay_id: &str,
    symbol: &str,
    market_scope: MarketScope,
    label_source: ReplayLabelSource,
    label_confidence: ReplayLabelConfidence,
    outcome_label: MemberExperienceOutcome,
) -> ReplayExample {
    let mut example = sprint156_replay_example(replay_id, symbol, market_scope);
    example.target.outcome_label = outcome_label;
    if let Some(labels) = &mut example.target_labels {
        labels.outcome_label = outcome_label;
        labels.label_source = label_source;
        labels.label_confidence = label_confidence;
    }
    example
}

fn sprint159_review_decision(
    decision_id: &str,
    review_item_id: &str,
    replay_id: &str,
    reviewer: PaperLabelReviewer,
    decision: PaperLabelReviewDecisionKind,
    confidence: ReplayLabelConfidence,
) -> PaperLabelReviewDecision {
    PaperLabelReviewDecision {
        decision_id: decision_id.to_string(),
        review_item_id: review_item_id.to_string(),
        replay_id: replay_id.to_string(),
        reviewer,
        decision,
        confidence,
        evidence_notes: vec!["paper-only local review".to_string()],
        paper_only: true,
    }
}

fn sprint160_training_example(
    replay_id: &str,
    member_id: &str,
    symbol: &str,
    market_scope: MarketScope,
    label_source: ReplayLabelSource,
    label_confidence: ReplayLabelConfidence,
    outcome_label: MemberExperienceOutcome,
    sanitized_market_summary: &str,
) -> ReplayExample {
    let mut example = sprint159_replay_example(
        replay_id,
        symbol,
        market_scope,
        label_source,
        label_confidence,
        outcome_label,
    );
    example.member_id = member_id.to_string();
    example.sanitized_input_features = Some(
        ReplayFeatureSanitizer::default().sanitize_replay_input_features(&ReplayInputFeatures {
            market_data_summary: sanitized_market_summary.to_string(),
            news_summary: format!("{symbol} paper-only news"),
            owner_context_summary: Some("owner before decision".to_string()),
            memory_state_summary: Some("member memory before decision".to_string()),
        }),
    );
    example
}

fn sprint166_balanced_pilot_replay_dataset() -> ReplayDataset {
    sprint156_replay_dataset(vec![
        sprint160_training_example(
            "s166-trend-train",
            "TrendEntryAI",
            "AAPL",
            MarketScope::UsShortTerm,
            ReplayLabelSource::ValidatedPaperLabel,
            ReplayLabelConfidence::High,
            MemberExperienceOutcome::PaperPositive,
            "AAPL trend train",
        ),
        sprint160_training_example(
            "s166-trend-validation",
            "TrendEntryAI",
            "AAPL",
            MarketScope::UsShortTerm,
            ReplayLabelSource::ValidatedPaperLabel,
            ReplayLabelConfidence::Medium,
            MemberExperienceOutcome::PaperNegative,
            "AAPL trend validation",
        ),
        sprint160_training_example(
            "s166-trend-test",
            "TrendEntryAI",
            "AAPL",
            MarketScope::UsShortTerm,
            ReplayLabelSource::ValidatedPaperLabel,
            ReplayLabelConfidence::High,
            MemberExperienceOutcome::PaperNeutral,
            "AAPL trend test",
        ),
        sprint160_training_example(
            "s166-risk-train",
            "RiskGuardAI",
            "MSFT",
            MarketScope::UsLongTerm,
            ReplayLabelSource::ValidatedPaperLabel,
            ReplayLabelConfidence::High,
            MemberExperienceOutcome::PaperPositive,
            "MSFT risk train",
        ),
        sprint160_training_example(
            "s166-risk-validation",
            "RiskGuardAI",
            "MSFT",
            MarketScope::UsLongTerm,
            ReplayLabelSource::ValidatedPaperLabel,
            ReplayLabelConfidence::Medium,
            MemberExperienceOutcome::PaperNegative,
            "MSFT risk validation",
        ),
        sprint160_training_example(
            "s166-risk-test",
            "RiskGuardAI",
            "MSFT",
            MarketScope::UsLongTerm,
            ReplayLabelSource::ValidatedPaperLabel,
            ReplayLabelConfidence::High,
            MemberExperienceOutcome::PaperNeutral,
            "MSFT risk test",
        ),
        sprint160_training_example(
            "s166-evidence-train",
            "EvidenceRegimeAI",
            "NVDA",
            MarketScope::UsShortTerm,
            ReplayLabelSource::ValidatedPaperLabel,
            ReplayLabelConfidence::High,
            MemberExperienceOutcome::PaperPositive,
            "NVDA evidence train",
        ),
        sprint160_training_example(
            "s166-evidence-validation",
            "EvidenceRegimeAI",
            "NVDA",
            MarketScope::UsShortTerm,
            ReplayLabelSource::ValidatedPaperLabel,
            ReplayLabelConfidence::Medium,
            MemberExperienceOutcome::PaperNegative,
            "NVDA evidence validation",
        ),
        sprint160_training_example(
            "s166-evidence-test",
            "EvidenceRegimeAI",
            "NVDA",
            MarketScope::UsShortTerm,
            ReplayLabelSource::ValidatedPaperLabel,
            ReplayLabelConfidence::High,
            MemberExperienceOutcome::PaperNeutral,
            "NVDA evidence test",
        ),
    ])
}

fn sprint166_balanced_training_split(dataset: &TrainingCandidateDataset) -> TrainingSplitResult {
    let ids_by_member: std::collections::BTreeMap<String, Vec<String>> = dataset
        .examples
        .iter()
        .fold(std::collections::BTreeMap::new(), |mut acc, example| {
            acc.entry(example.member_id.clone())
                .or_default()
                .push(example.training_example_id.clone());
            acc
        });
    let mut train_ids = Vec::new();
    let mut validation_ids = Vec::new();
    let mut test_ids = Vec::new();
    for member_id in ["EvidenceRegimeAI", "RiskGuardAI", "TrendEntryAI"] {
        let ids = ids_by_member
            .get(member_id)
            .expect("balanced sprint166 dataset for pilot member");
        train_ids.push(ids[0].clone());
        validation_ids.push(ids[1].clone());
        test_ids.push(ids[2].clone());
    }
    TrainingSplitResult {
        dataset_id: dataset.dataset_id.clone(),
        train_ids,
        validation_ids,
        test_ids,
        train_count: 3,
        validation_count: 3,
        test_count: 3,
        split_warnings: Vec::new(),
        paper_only: true,
    }
}

#[test]
fn paper_outcome_evidence_ingestion_rejects_unsafe_and_counts_duplicates() {
    let path = std::path::Path::new("target/sprint156_paper_evidence.json");
    std::fs::create_dir_all("target").expect("target dir");
    let mut duplicate = sprint156_paper_evidence(
        "evidence-positive",
        Some("replay-positive"),
        "005930.KS",
        MarketScope::KoreaShortTerm,
        MemberExperienceOutcome::PaperPositive,
        ReplayLabelSource::ManualPaperLabel,
        ReplayLabelConfidence::High,
        Some(0.02),
    );
    duplicate.symbol = "000660.KS".to_string();
    let file = PaperOutcomeEvidenceFile {
        schema_version: "paper-outcome-evidence.v1".to_string(),
        evidence_file_id: "sprint156-load".to_string(),
        created_at: None,
        source_label: "unit-local-paper-evidence".to_string(),
        records: vec![
            sprint156_paper_evidence(
                "evidence-positive",
                Some("replay-positive"),
                "005930.KS",
                MarketScope::KoreaShortTerm,
                MemberExperienceOutcome::PaperPositive,
                ReplayLabelSource::ManualPaperLabel,
                ReplayLabelConfidence::High,
                Some(0.02),
            ),
            duplicate,
            sprint156_paper_evidence(
                "evidence-needs-review",
                None,
                "AAPL",
                MarketScope::UsShortTerm,
                MemberExperienceOutcome::Unknown,
                ReplayLabelSource::ReviewRequired,
                ReplayLabelConfidence::ReviewRequired,
                None,
            ),
        ],
        paper_only: true,
    };
    std::fs::write(path, serde_json::to_string_pretty(&file).expect("json")).expect("write");
    let loaded = load_paper_outcome_evidence_from_local_json(path).expect("load evidence");
    assert!(loaded.loaded);
    assert_eq!(loaded.evidence_file_id, "sprint156-load");
    assert_eq!(loaded.record_count, 2);
    assert_eq!(loaded.duplicate_count, 1);
    assert_eq!(loaded.rejected_count, 1);

    let remote_err = load_paper_outcome_evidence_from_local_json(std::path::Path::new(
        "https://bad/evidence.json",
    ))
    .expect_err("remote evidence path rejected");
    assert!(remote_err.contains("must be local"));

    let unsafe_path = std::path::Path::new("target/sprint156_unsafe_paper_evidence.json");
    std::fs::write(
        unsafe_path,
        r#"{
  "schema_version": "paper-outcome-evidence.v1",
  "evidence_file_id": "unsafe",
  "source_label": "unit",
  "paper_only": true,
  "records": [{
    "evidence_id": "unsafe-order",
    "symbol": "005930.KS",
    "market_scope": "KoreaShortTerm",
    "horizon": "ShortTerm",
    "candidate_label": "PaperPositive",
    "label_source": "ManualPaperLabel",
    "label_confidence": "High",
    "account": "forbidden",
    "order": "forbidden",
    "pnl": 10.0,
    "price_change_pct": 0.02,
    "paper_only": true
  }]
}"#,
    )
    .expect("write unsafe");
    let unsafe_loaded =
        load_paper_outcome_evidence_from_local_json(unsafe_path).expect("load unsafe result");
    assert_eq!(unsafe_loaded.record_count, 0);
    assert_eq!(unsafe_loaded.rejected_count, 1);
    assert!(
        unsafe_loaded
            .safety_warnings
            .iter()
            .any(|warning| warning.contains("unsafe field"))
    );

    let false_file = PaperOutcomeEvidenceFile {
        paper_only: false,
        records: vec![sprint156_paper_evidence(
            "paper-only-false",
            None,
            "005930.KS",
            MarketScope::KoreaShortTerm,
            MemberExperienceOutcome::PaperPositive,
            ReplayLabelSource::ManualPaperLabel,
            ReplayLabelConfidence::High,
            Some(0.02),
        )],
        ..file
    };
    let validated = validate_paper_outcome_evidence_file(false_file);
    assert!(!validated.loaded);
    assert_eq!(validated.rejected_count, 1);
    let _ = std::fs::remove_file(path);
    let _ = std::fs::remove_file(unsafe_path);
}

#[test]
fn paper_outcome_evidence_matching_is_prioritized_and_ambiguous_is_safe() {
    let dataset = sprint156_replay_dataset(vec![
        sprint156_replay_example("replay-exp-1", "005930.KS", MarketScope::KoreaShortTerm),
        sprint156_replay_example("replay-exp-2", "005930.KS", MarketScope::KoreaShortTerm),
        sprint156_replay_example("decision-3", "MSFT", MarketScope::UsLongTerm),
    ]);
    let mut by_experience = sprint156_paper_evidence(
        "match-experience",
        None,
        "005930.KS",
        MarketScope::KoreaShortTerm,
        MemberExperienceOutcome::PaperPositive,
        ReplayLabelSource::ManualPaperLabel,
        ReplayLabelConfidence::Medium,
        Some(0.02),
    );
    by_experience.experience_id = Some("exp-2".to_string());
    let mut by_decision = sprint156_paper_evidence(
        "match-decision",
        None,
        "MSFT",
        MarketScope::UsLongTerm,
        MemberExperienceOutcome::PaperNegative,
        ReplayLabelSource::ManualPaperLabel,
        ReplayLabelConfidence::Medium,
        Some(-0.02),
    );
    by_decision.decision_id = Some("decision-3".to_string());
    let mut unknown_horizon = sprint156_paper_evidence(
        "unknown-horizon",
        None,
        "MSFT",
        MarketScope::UsLongTerm,
        MemberExperienceOutcome::PaperNegative,
        ReplayLabelSource::ManualPaperLabel,
        ReplayLabelConfidence::Medium,
        Some(-0.02),
    );
    unknown_horizon.horizon = PaperOutcomeEvidenceHorizon::Unknown;
    let matching = match_paper_outcome_evidence_to_replay(
        &dataset,
        &[
            sprint156_paper_evidence(
                "match-replay",
                Some("replay-exp-1"),
                "005930.KS",
                MarketScope::KoreaShortTerm,
                MemberExperienceOutcome::PaperPositive,
                ReplayLabelSource::ManualPaperLabel,
                ReplayLabelConfidence::High,
                Some(0.02),
            ),
            by_experience,
            by_decision,
            sprint156_paper_evidence(
                "ambiguous-symbol",
                None,
                "005930.KS",
                MarketScope::KoreaShortTerm,
                MemberExperienceOutcome::PaperPositive,
                ReplayLabelSource::ManualPaperLabel,
                ReplayLabelConfidence::High,
                Some(0.02),
            ),
            unknown_horizon,
        ],
    );
    assert_eq!(matching.matched_count, 3);
    assert_eq!(matching.unmatched_count, 1);
    assert_eq!(matching.ambiguous_count, 1);
    assert!(
        matching
            .ambiguous_evidence_ids
            .contains(&"ambiguous-symbol".to_string())
    );
}

#[test]
fn sprint157_inventory_candidates_and_resolver_stay_advisory() {
    let dataset = sprint156_replay_dataset(vec![
        sprint156_replay_example("replay-exp-1", "005930.KS", MarketScope::KoreaShortTerm),
        sprint156_replay_example("replay-exp-2", "005930.KS", MarketScope::KoreaShortTerm),
        sprint156_replay_example("decision-msft", "MSFT", MarketScope::UsLongTerm),
    ]);
    let store = MemberExperienceStore::new("sprint157-store", Vec::new());
    let exact_replay = sprint156_paper_evidence(
        "exact-replay",
        Some("replay-exp-1"),
        "005930.KS",
        MarketScope::KoreaShortTerm,
        MemberExperienceOutcome::PaperPositive,
        ReplayLabelSource::ManualPaperLabel,
        ReplayLabelConfidence::High,
        Some(0.02),
    );
    let mut exact_decision = sprint156_paper_evidence(
        "exact-decision",
        None,
        "MSFT",
        MarketScope::UsLongTerm,
        MemberExperienceOutcome::PaperNegative,
        ReplayLabelSource::ManualPaperLabel,
        ReplayLabelConfidence::Medium,
        Some(-0.02),
    );
    exact_decision.decision_id = Some("decision-msft".to_string());
    let ambiguous = sprint156_paper_evidence(
        "ambiguous-missing-replay",
        None,
        "005930.KS",
        MarketScope::KoreaShortTerm,
        MemberExperienceOutcome::PaperPositive,
        ReplayLabelSource::ManualPaperLabel,
        ReplayLabelConfidence::Medium,
        Some(0.02),
    );
    let mut unknown_horizon = sprint156_paper_evidence(
        "unknown-horizon",
        None,
        "TSLA",
        MarketScope::UsShortTerm,
        MemberExperienceOutcome::PaperNeutral,
        ReplayLabelSource::ReviewRequired,
        ReplayLabelConfidence::ReviewRequired,
        None,
    );
    unknown_horizon.horizon = PaperOutcomeEvidenceHorizon::Unknown;
    let evidence_records = vec![
        exact_replay.clone(),
        exact_decision.clone(),
        ambiguous.clone(),
        unknown_horizon.clone(),
    ];
    let matching = match_paper_outcome_evidence_to_replay(&dataset, &evidence_records);
    let inventory = build_ambiguous_evidence_inventory(&matching, &evidence_records);
    assert_eq!(inventory.ambiguous_count, 1);
    assert_eq!(inventory.no_match_count, 1);
    assert_eq!(inventory.missing_replay_id_count, 3);
    assert_eq!(inventory.unknown_horizon_count, 1);

    let replay_candidates = generate_evidence_match_candidates(&exact_replay, &dataset, &store);
    assert_eq!(
        replay_candidates[0].match_type,
        PaperOutcomeEvidenceMatchType::ReplayId
    );
    assert_eq!(
        replay_candidates[0].match_confidence,
        PaperOutcomeEvidenceMatchConfidence::High
    );
    assert!(replay_candidates[0].promotion_safe);

    let decision_candidates = generate_evidence_match_candidates(&exact_decision, &dataset, &store);
    assert_eq!(
        decision_candidates[0].match_type,
        PaperOutcomeEvidenceMatchType::DecisionId
    );
    assert!(decision_candidates[0].promotion_safe);

    let symbol_candidates = generate_evidence_match_candidates(&ambiguous, &dataset, &store);
    assert!(
        symbol_candidates
            .iter()
            .all(|candidate| !candidate.promotion_safe)
    );
    let unknown_candidates = generate_evidence_match_candidates(&unknown_horizon, &dataset, &store);
    assert!(unknown_candidates.is_empty());

    let resolution = resolve_ambiguous_evidence(
        &inventory,
        &evidence_records,
        &dataset,
        &store,
        &AmbiguousEvidenceResolutionPolicy::default(),
    );
    assert!(
        resolution
            .unresolved_evidence_ids
            .contains(&"ambiguous-missing-replay".to_string())
    );
    assert!(
        resolution
            .unresolved_evidence_ids
            .contains(&"unknown-horizon".to_string())
    );
    assert_eq!(resolution.safe_suggestion_count, 0);
}

#[test]
fn sprint157_backfill_patch_is_local_and_label_side_only() {
    let original = sprint156_paper_evidence(
        "needs-backfill",
        None,
        "MSFT",
        MarketScope::UsLongTerm,
        MemberExperienceOutcome::PaperNegative,
        ReplayLabelSource::ManualPaperLabel,
        ReplayLabelConfidence::Medium,
        Some(-0.02),
    );
    let evidence_file = PaperOutcomeEvidenceFile {
        schema_version: "paper-outcome-evidence.v1".to_string(),
        evidence_file_id: "sprint157-patch-file".to_string(),
        created_at: None,
        source_label: "test".to_string(),
        records: vec![original.clone()],
        paper_only: true,
    };
    let suggestion = soma_zero::league::minimal_ai_committee_core::EvidenceResolutionSuggestion {
        evidence_id: "needs-backfill".to_string(),
        suggested_replay_id: Some("replay-msft".to_string()),
        suggested_experience_id: Some("exp-msft".to_string()),
        suggested_decision_id: Some("decision-msft".to_string()),
        suggested_match_type: PaperOutcomeEvidenceMatchType::DecisionId,
        suggested_confidence: PaperOutcomeEvidenceMatchConfidence::High,
        promotion_safe_after_patch: true,
        reason: "test exact decision match".to_string(),
        required_manual_confirmation: false,
        paper_only: true,
    };
    let patch = build_evidence_backfill_patch(&evidence_file, &[suggestion]);
    assert_eq!(patch.patched_records.len(), 1);
    assert_eq!(
        patch.patched_records[0].replay_id.as_deref(),
        Some("replay-msft")
    );
    assert_eq!(
        patch.patched_records[0].candidate_label,
        original.candidate_label
    );
    assert_eq!(
        patch.patched_records[0].price_change_pct,
        original.price_change_pct
    );

    let missing_path = "target/sprint157-dry-run-should-not-exist.json";
    let _ = std::fs::remove_file(missing_path);
    assert!(!std::path::Path::new(missing_path).exists());

    let remote_err = apply_evidence_backfill_patch_to_local_json(
        &evidence_file,
        &patch,
        "https://example.invalid/backfill.json",
    )
    .expect_err("remote patch output must fail");
    assert!(remote_err.contains("local"));

    let output_path = "target/sprint157-backfilled-evidence.json";
    let result = apply_evidence_backfill_patch_to_local_json(&evidence_file, &patch, output_path)
        .expect("apply local patch");
    assert_eq!(result.applied_count, 1);
    assert!(std::path::Path::new(output_path).exists());
    let _ = std::fs::remove_file(output_path);
}

#[test]
fn sprint157_dry_run_backfill_does_not_feed_rebuild() {
    std::fs::create_dir_all("target").expect("target dir");
    let evidence_input_path = "target/sprint157-dry-run-evidence.json";
    let dry_run_output_path = "target/sprint157-dry-run-output.json";
    let _ = std::fs::remove_file(evidence_input_path);
    let _ = std::fs::remove_file(dry_run_output_path);

    let dataset = sprint156_replay_dataset(vec![sprint156_replay_example(
        "replay-exp-msft",
        "MSFT",
        MarketScope::UsLongTerm,
    )]);
    let mut evidence = sprint156_paper_evidence(
        "decision-only-unknown-horizon",
        None,
        "MSFT",
        MarketScope::UsLongTerm,
        MemberExperienceOutcome::PaperNegative,
        ReplayLabelSource::ManualPaperLabel,
        ReplayLabelConfidence::High,
        Some(-0.02),
    );
    evidence.decision_id = Some("decision-msft".to_string());
    evidence.horizon = PaperOutcomeEvidenceHorizon::Unknown;
    let evidence_file = PaperOutcomeEvidenceFile {
        schema_version: "paper-outcome-evidence.v1".to_string(),
        evidence_file_id: "sprint157-dry-run-evidence-file".to_string(),
        created_at: None,
        source_label: "test".to_string(),
        records: vec![evidence],
        paper_only: true,
    };
    std::fs::write(
        evidence_input_path,
        serde_json::to_string_pretty(&evidence_file).expect("evidence json"),
    )
    .expect("write evidence");

    let store = MemberExperienceStore::new(
        "sprint157-dry-run-store",
        vec![MemberExperienceRecord {
            experience_id: "exp-msft".to_string(),
            member_id: "TrendEntryAI".to_string(),
            symbol: "MSFT".to_string(),
            market_scope: MarketScope::UsLongTerm,
            cycle_id: None,
            event_id: None,
            session_id: None,
            decision_id: Some("decision-msft".to_string()),
            input_context: MemberExperienceInputContext {
                market_data_summary: "MSFT pre-decision market summary".to_string(),
                news_summary: "local paper-only news summary".to_string(),
                owner_context_summary: None,
                style_blend_summary: None,
                memory_state_summary: None,
            },
            member_opinion: MemberExperienceOpinionSnapshot {
                stance: MemberStance::BuyProposal,
                confidence: 0.7,
                expected_return_hint: -0.02,
                risk_hint: 0.2,
                evidence_notes: vec!["decision trace exists".to_string()],
                event_triggered: true,
            },
            committee_context: MemberExperienceCommitteeContext {
                disagreement_level: 0.0,
                other_member_stances: Vec::new(),
                chairman_action: None,
                risk_governor_status: None,
                risk_flags: Vec::new(),
            },
            outcome: MemberExperienceOutcome::PaperNegative,
            attribution: MemberScoreUpdateReason::Neutral,
            learning_label: MemberLearningLabel::Reinforce,
            created_at: None,
            paper_only: true,
        }],
    );
    let result = run_evidence_backfill_and_promotion_with_inputs(
        &EvidenceBackfillAndPromotionRunConfig {
            evidence_input_path: evidence_input_path.to_string(),
            evidence_backfill_output_path: Some(dry_run_output_path.to_string()),
            sanitized_dataset_path: None,
            experience_store_path: None,
            validated_replay_output_path: None,
            label_quality_output_path: None,
            min_validated_label_ratio_required: 0.5,
            dry_run: true,
            apply_backfill_patch: true,
            rebuild_validated_dataset: true,
            paper_only: true,
        },
        &dataset,
        &store,
        &PaperLabelValidationPolicy::default(),
    )
    .expect("dry-run backfill");

    assert_eq!(result.resolution_result.safe_suggestion_count, 1);
    let patch_result = result.patch_result.as_ref().expect("patch result");
    assert_eq!(patch_result.patch.patched_records.len(), 1);
    assert_eq!(patch_result.applied_count, 0);
    assert_eq!(patch_result.skipped_count, 1);
    assert_eq!(patch_result.output_path, None);
    assert!(!std::path::Path::new(dry_run_output_path).exists());

    let build = result
        .validated_build_result
        .as_ref()
        .expect("validated build");
    assert_eq!(build.promoted_count, 0);
    assert_eq!(build.unmatched_evidence_count, 1);

    let preview_result = run_evidence_backfill_and_promotion_with_inputs(
        &EvidenceBackfillAndPromotionRunConfig {
            evidence_input_path: evidence_input_path.to_string(),
            evidence_backfill_output_path: Some(dry_run_output_path.to_string()),
            sanitized_dataset_path: None,
            experience_store_path: None,
            validated_replay_output_path: None,
            label_quality_output_path: None,
            min_validated_label_ratio_required: 0.5,
            dry_run: false,
            apply_backfill_patch: false,
            rebuild_validated_dataset: true,
            paper_only: true,
        },
        &dataset,
        &store,
        &PaperLabelValidationPolicy::default(),
    )
    .expect("preview backfill");
    let preview_patch = preview_result
        .patch_result
        .as_ref()
        .expect("preview patch result");
    assert_eq!(preview_patch.applied_count, 0);
    assert_eq!(preview_patch.output_path, None);
    assert!(!std::path::Path::new(dry_run_output_path).exists());
    assert_eq!(
        preview_result
            .validated_build_result
            .as_ref()
            .expect("preview validated build")
            .promoted_count,
        0
    );

    let _ = std::fs::remove_file(evidence_input_path);
    let _ = std::fs::remove_file(dry_run_output_path);
}

#[test]
fn sprint157_expansion_plan_and_seed_batch_are_safe() {
    let mut examples = Vec::new();
    for index in 0..18 {
        let mut example = sprint156_replay_example(
            &format!("replay-{index}"),
            "005930.KS",
            MarketScope::KoreaShortTerm,
        );
        if index < 2 {
            example.target_labels = Some(ReplayTargetLabels {
                stance_target: example.target.stance,
                confidence_calibration_target: example.target.confidence_calibration,
                risk_label: example.target.risk_label.clone(),
                evidence_label: example.target.evidence_label.clone(),
                outcome_label: MemberExperienceOutcome::PaperPositive,
                attribution_label: MemberScoreUpdateReason::Neutral,
                label_source: ReplayLabelSource::ValidatedPaperLabel,
                label_confidence: ReplayLabelConfidence::High,
                paper_only: true,
            });
        }
        examples.push(example);
    }
    let dataset = sprint156_replay_dataset(examples);
    let load = validate_paper_outcome_evidence_file(PaperOutcomeEvidenceFile {
        schema_version: "paper-outcome-evidence.v1".to_string(),
        evidence_file_id: "sprint157-plan-evidence".to_string(),
        created_at: None,
        source_label: "test".to_string(),
        records: Vec::new(),
        paper_only: true,
    });
    let matching = match_paper_outcome_evidence_to_replay(&dataset, &[]);
    let summary = summarize_paper_outcome_evidence_quality(&load, &matching, &[]);
    let plan = build_validated_label_ratio_expansion_plan(&dataset, &summary, 0.5);
    assert_eq!(plan.current_total_examples, 18);
    assert_eq!(plan.current_validated_label_count, 2);
    assert_eq!(plan.additional_validated_labels_needed, 7);
    assert!(plan.candidate_evidence_ids.is_empty());

    let policy = PaperLabelValidationPolicy::default();
    let valid_seed = AdditionalPaperEvidenceSeed {
        seed_id: "seed-valid".to_string(),
        replay_id: Some("replay-3".to_string()),
        experience_id: None,
        decision_id: None,
        symbol: "005930.KS".to_string(),
        market_scope: MarketScope::KoreaShortTerm,
        horizon: PaperOutcomeEvidenceHorizon::ShortTerm,
        candidate_label: MemberExperienceOutcome::PaperPositive,
        price_change_pct: Some(0.02),
        reference_price: Some(100.0),
        horizon_price: Some(102.0),
        label_source: ReplayLabelSource::ManualPaperLabel,
        label_confidence: ReplayLabelConfidence::Medium,
        evidence_notes: Vec::new(),
        paper_only: true,
    };
    let contradictory_seed = AdditionalPaperEvidenceSeed {
        seed_id: "seed-contradictory".to_string(),
        candidate_label: MemberExperienceOutcome::PaperPositive,
        price_change_pct: Some(-0.02),
        ..valid_seed.clone()
    };
    let simulated_high_seed = AdditionalPaperEvidenceSeed {
        seed_id: "seed-simulated-high".to_string(),
        label_source: ReplayLabelSource::SimulatedFixture,
        label_confidence: ReplayLabelConfidence::High,
        ..valid_seed.clone()
    };
    let batch = build_paper_evidence_expansion_batch(
        &[valid_seed, contradictory_seed, simulated_high_seed],
        &policy,
    );
    assert_eq!(batch.generated_count, 1);
    assert_eq!(batch.rejected_count, 2);
    assert_eq!(batch.generated_records[0].evidence_id, "seed-valid");
}

#[test]
fn sprint157_patched_evidence_rebuild_improves_ratio_without_mutating_inputs() {
    let dataset = sprint156_replay_dataset(vec![
        sprint156_replay_example("replay-1", "005930.KS", MarketScope::KoreaShortTerm),
        sprint156_replay_example("replay-2", "MSFT", MarketScope::UsLongTerm),
    ]);
    let mut before_features: Vec<ReplayInputFeatures> = dataset
        .examples
        .iter()
        .map(|example| example.input_features.clone())
        .collect();
    before_features.sort_by(|left, right| left.market_data_summary.cmp(&right.market_data_summary));
    let policy = PaperLabelValidationPolicy::default();
    let evidence = vec![sprint156_paper_evidence(
        "promote-one",
        Some("replay-1"),
        "005930.KS",
        MarketScope::KoreaShortTerm,
        MemberExperienceOutcome::PaperPositive,
        ReplayLabelSource::ManualPaperLabel,
        ReplayLabelConfidence::High,
        Some(0.02),
    )];
    let build = build_validated_replay_dataset_with_paper_evidence(
        &dataset,
        &evidence,
        &policy,
        soma_zero::league::minimal_ai_committee_core::ValidatedReplayDatasetBuildConfig {
            source_sanitized_dataset_id: dataset.dataset_id.clone(),
            label_validation_policy: policy.clone(),
            backtest_label_contract: None,
            require_validated_labels_for_training: true,
            allow_ready_with_warnings_for_medium_confidence: true,
            reject_rejected_labels: true,
            paper_only: true,
        },
    );
    let summary = summarize_label_quality(&build);
    assert_eq!(summary.validated_label_count, 1);
    assert!(summary.validated_label_ratio > 0.0);
    let after_features: Vec<ReplayInputFeatures> = build
        .dataset
        .examples
        .iter()
        .map(|example| example.input_features.clone())
        .collect();
    assert_eq!(before_features, after_features);

    let readiness = evaluate_offline_training_readiness_with_label_ratio_threshold(
        &build.dataset,
        &MemberExperienceStore::new("empty", Vec::new()),
        10,
        0,
        0.75,
    );
    assert_eq!(
        readiness.readiness_status,
        OfflineTrainingReadinessStatus::NeedsMoreData
    );
    assert!(readiness.paper_only);

    let repeated = build_validated_replay_dataset_with_paper_evidence(
        &dataset,
        &evidence,
        &PaperLabelValidationPolicy::default(),
        soma_zero::league::minimal_ai_committee_core::ValidatedReplayDatasetBuildConfig {
            source_sanitized_dataset_id: dataset.dataset_id.clone(),
            label_validation_policy: PaperLabelValidationPolicy::default(),
            backtest_label_contract: None,
            require_validated_labels_for_training: true,
            allow_ready_with_warnings_for_medium_confidence: true,
            reject_rejected_labels: true,
            paper_only: true,
        },
    );
    assert_eq!(build, repeated);
}

#[test]
fn sprint158_workbench_and_price_series_store_are_local_and_deterministic() {
    let dataset = sprint156_replay_dataset(vec![
        sprint156_replay_example("replay-example-aapl", "AAPL", MarketScope::UsShortTerm),
        sprint156_replay_example("replay-msft", "MSFT", MarketScope::UsLongTerm),
    ]);
    let store = sprint158_experience_store();
    let workbench = build_replay_evidence_workbench(
        &dataset,
        &store,
        &[],
        &ReplayEvidenceWorkbenchConfig {
            workbench_id: "sprint158-workbench".to_string(),
            sanitized_dataset_path: None,
            experience_store_path: None,
            validated_dataset_path: None,
            existing_evidence_path: None,
            output_evidence_draft_path: None,
            output_generated_evidence_path: None,
            output_validated_replay_path: None,
            target_validated_label_ratio: 0.5,
            max_drafts: 10,
            include_only_unvalidated: true,
            include_review_required: true,
            include_ambiguous: false,
            paper_only: true,
        },
    );
    assert_eq!(workbench.draft_count, 2);
    assert!(
        workbench
            .draft_rows
            .iter()
            .all(|row| !row.replay_id.trim().is_empty())
    );
    assert!(
        workbench
            .draft_rows
            .iter()
            .any(|row| row.decision_id.as_deref() == Some("decision-aapl"))
    );

    let output_path = std::path::Path::new("target/sprint158_price_series_store.json");
    save_price_series_store_to_local_json(&sprint158_price_series_store(), output_path)
        .expect("save local price series store");
    let loaded = load_price_series_store_from_local_json(output_path).expect("load price series");
    assert_eq!(loaded.series.len(), 4);
    assert!(
        loaded
            .series_for_symbol_scope("AAPL", MarketScope::UsShortTerm)
            .is_some()
    );
    let remote_err = load_price_series_store_from_local_json(std::path::Path::new(
        "https://example.invalid/price-series.json",
    ))
    .expect_err("remote price series path must fail");
    assert!(remote_err.contains("local"));

    let unsafe_path = std::path::Path::new("target/sprint158_unsafe_price_series.json");
    std::fs::write(
        unsafe_path,
        serde_json::json!({
            "store_id": "unsafe-price-series",
            "paper_only": true,
            "broker_account": "must-not-load",
            "series": []
        })
        .to_string(),
    )
    .expect("write unsafe price series");
    let unsafe_err =
        load_price_series_store_from_local_json(unsafe_path).expect_err("unsafe field rejected");
    assert!(unsafe_err.contains("unsafe"));

    let mut mismatched_store = sprint158_price_series_store();
    mismatched_store.series[0].bars[0].symbol = "MSFT".to_string();
    let mismatch_err = save_price_series_store_to_local_json(&mismatched_store, output_path)
        .expect_err("mismatched bar rejected");
    assert!(mismatch_err.contains("symbol/scope"));

    let _ = std::fs::remove_file(output_path);
    let _ = std::fs::remove_file(unsafe_path);
}

#[test]
fn sprint158_price_labeler_and_generated_evidence_cover_positive_negative_neutral_and_review() {
    let dataset = sprint156_replay_dataset(vec![
        sprint156_replay_example("replay-example-aapl", "AAPL", MarketScope::UsShortTerm),
        sprint156_replay_example("replay-msft", "MSFT", MarketScope::UsLongTerm),
        sprint156_replay_example("replay-btc", "BTCUSDT", MarketScope::CryptoShortTerm),
        sprint156_replay_example("replay-eth", "ETHUSDT", MarketScope::CryptoLongTerm),
    ]);
    let store = sprint158_experience_store();
    let mut workbench = build_replay_evidence_workbench(
        &dataset,
        &store,
        &[],
        &ReplayEvidenceWorkbenchConfig {
            workbench_id: "sprint158-labeler".to_string(),
            sanitized_dataset_path: None,
            experience_store_path: None,
            validated_dataset_path: None,
            existing_evidence_path: None,
            output_evidence_draft_path: None,
            output_generated_evidence_path: None,
            output_validated_replay_path: None,
            target_validated_label_ratio: 0.5,
            max_drafts: 10,
            include_only_unvalidated: true,
            include_review_required: true,
            include_ambiguous: false,
            paper_only: true,
        },
    );
    if let Some(row) = workbench
        .draft_rows
        .iter_mut()
        .find(|row| row.replay_id == "replay-eth")
    {
        row.horizon = PaperOutcomeEvidenceHorizon::Unknown;
    }
    let label_result = generate_price_move_label_candidates(
        &workbench,
        &sprint158_price_series_store(),
        &PaperPriceMoveLabelPolicy::default(),
    );
    let labels_by_replay: std::collections::BTreeMap<_, _> = label_result
        .candidates
        .iter()
        .map(|candidate| (candidate.replay_id.as_str(), candidate.candidate_label))
        .collect();
    assert_eq!(
        labels_by_replay.get("replay-example-aapl"),
        Some(&MemberExperienceOutcome::PaperPositive)
    );
    assert_eq!(
        labels_by_replay.get("replay-msft"),
        Some(&MemberExperienceOutcome::PaperNegative)
    );
    assert_eq!(
        labels_by_replay.get("replay-btc"),
        Some(&MemberExperienceOutcome::PaperNeutral)
    );
    assert_eq!(
        labels_by_replay.get("replay-eth"),
        Some(&MemberExperienceOutcome::Unknown)
    );
    assert!(label_result.needs_review_count >= 1);

    let insufficient_result = generate_price_move_label_candidates(
        &workbench,
        &sprint158_price_series_store(),
        &PaperPriceMoveLabelPolicy {
            min_bars_required: 5,
            ..PaperPriceMoveLabelPolicy::default()
        },
    );
    let insufficient_aapl = insufficient_result
        .candidates
        .iter()
        .find(|candidate| candidate.replay_id == "replay-example-aapl")
        .expect("aapl insufficient-bars candidate");
    assert_eq!(
        insufficient_aapl.candidate_label,
        MemberExperienceOutcome::Unknown
    );
    assert_eq!(
        insufficient_aapl.label_confidence,
        ReplayLabelConfidence::ReviewRequired
    );

    let generated_batch =
        build_paper_evidence_records_from_price_move_candidates(&label_result.candidates);
    assert!(
        generated_batch
            .generated_records
            .iter()
            .all(|record| record.replay_id.is_some())
    );
    let serialized = serde_json::to_string(&generated_batch.generated_records).expect("serialize");
    assert!(!serialized.to_ascii_lowercase().contains("broker"));
    assert!(!serialized.to_ascii_lowercase().contains("account"));
    assert!(!serialized.to_ascii_lowercase().contains("order"));

    let promoted = promote_labels_with_paper_evidence(
        &dataset,
        &generated_batch.generated_records,
        &PaperLabelValidationPolicy::default(),
    );
    assert!(
        promoted.iter().any(|item| {
            item.promotion_status == ValidatedPaperEvidencePromotionStatus::Promoted
        })
    );
}

#[test]
fn sprint158_validated_ratio_expansion_run_respects_dry_run_and_is_repeatable() {
    let dataset = sprint156_replay_dataset(vec![
        sprint156_replay_example("replay-example-aapl", "AAPL", MarketScope::UsShortTerm),
        sprint156_replay_example("replay-msft", "MSFT", MarketScope::UsLongTerm),
        sprint156_replay_example("replay-btc", "BTCUSDT", MarketScope::CryptoShortTerm),
        sprint156_replay_example("replay-eth", "ETHUSDT", MarketScope::CryptoLongTerm),
    ]);
    let mut before_features: Vec<ReplayInputFeatures> = dataset
        .examples
        .iter()
        .map(|example| example.input_features.clone())
        .collect();
    before_features.sort_by(|left, right| left.market_data_summary.cmp(&right.market_data_summary));
    let store = sprint158_experience_store();
    let price_store = sprint158_price_series_store();
    let dry_path = "target/sprint158_generated_dry_run.json";
    let _ = std::fs::remove_file(dry_path);
    let dry_run = run_validated_ratio_expansion_with_inputs(
        &ValidatedRatioExpansionRunConfig {
            run_id: "sprint158-dry-run".to_string(),
            target_validated_label_ratio: 0.9,
            sanitized_dataset_path: None,
            experience_store_path: None,
            existing_evidence_path: None,
            price_series_path: Some("examples/minimal_paper_price_series.sample.json".to_string()),
            generated_evidence_output_path: Some(dry_path.to_string()),
            validated_replay_output_path: None,
            label_quality_output_path: None,
            readiness_output_path: None,
            dry_run: true,
            paper_only: true,
        },
        &dataset,
        &store,
        &[],
        &price_store,
        &PaperLabelValidationPolicy::default(),
    )
    .expect("dry-run expansion");
    assert!(!std::path::Path::new(dry_path).exists());
    assert!(dry_run.new_validated_ratio > dry_run.previous_validated_ratio);
    assert_eq!(
        dry_run.readiness_gate.readiness_status,
        OfflineTrainingReadinessStatus::NeedsMoreData
    );
    let mut after_features: Vec<ReplayInputFeatures> = dry_run
        .validated_build_result
        .dataset
        .examples
        .iter()
        .map(|example| example.input_features.clone())
        .collect();
    after_features.sort_by(|left, right| left.market_data_summary.cmp(&right.market_data_summary));
    assert_eq!(before_features, after_features);

    let non_paper_err = run_validated_ratio_expansion_with_inputs(
        &ValidatedRatioExpansionRunConfig {
            run_id: "sprint158-non-paper-run".to_string(),
            target_validated_label_ratio: 0.5,
            sanitized_dataset_path: None,
            experience_store_path: None,
            existing_evidence_path: None,
            price_series_path: Some("examples/minimal_paper_price_series.sample.json".to_string()),
            generated_evidence_output_path: None,
            validated_replay_output_path: None,
            label_quality_output_path: None,
            readiness_output_path: None,
            dry_run: true,
            paper_only: false,
        },
        &dataset,
        &store,
        &[],
        &price_store,
        &PaperLabelValidationPolicy::default(),
    )
    .expect_err("non-paper expansion rejected");
    assert!(non_paper_err.contains("paper-only"));

    let output_path = "target/sprint158_generated_apply.json";
    let _ = std::fs::remove_file(output_path);
    let apply_run = run_validated_ratio_expansion_with_inputs(
        &ValidatedRatioExpansionRunConfig {
            run_id: "sprint158-apply-run".to_string(),
            target_validated_label_ratio: 0.5,
            sanitized_dataset_path: None,
            experience_store_path: None,
            existing_evidence_path: None,
            price_series_path: Some("examples/minimal_paper_price_series.sample.json".to_string()),
            generated_evidence_output_path: Some(output_path.to_string()),
            validated_replay_output_path: None,
            label_quality_output_path: None,
            readiness_output_path: None,
            dry_run: false,
            paper_only: true,
        },
        &dataset,
        &store,
        &[],
        &price_store,
        &PaperLabelValidationPolicy::default(),
    )
    .expect("apply expansion");
    assert!(std::path::Path::new(output_path).exists());
    assert_eq!(
        apply_run.readiness_gate.readiness_status,
        OfflineTrainingReadinessStatus::ReadyWithWarningsForOfflineTrainingDesign
    );
    assert!(apply_run.safety_summary.no_model_training);
    assert!(apply_run.safety_summary.no_live_inference);
    assert!(apply_run.safety_summary.no_broker_order_account);

    let repeated = run_validated_ratio_expansion_with_inputs(
        &ValidatedRatioExpansionRunConfig {
            run_id: "sprint158-apply-run".to_string(),
            target_validated_label_ratio: 0.5,
            sanitized_dataset_path: None,
            experience_store_path: None,
            existing_evidence_path: None,
            price_series_path: Some("examples/minimal_paper_price_series.sample.json".to_string()),
            generated_evidence_output_path: None,
            validated_replay_output_path: None,
            label_quality_output_path: None,
            readiness_output_path: None,
            dry_run: true,
            paper_only: true,
        },
        &dataset,
        &store,
        &[],
        &price_store,
        &PaperLabelValidationPolicy::default(),
    )
    .expect("repeat expansion");
    assert_eq!(
        apply_run.generated_evidence_batch.generated_records,
        repeated.generated_evidence_batch.generated_records
    );
    let _ = std::fs::remove_file(output_path);
}

#[test]
fn sprint159_weak_label_inventory_and_review_queue_detect_and_sort_weak_labels() {
    let dataset = sprint156_replay_dataset(vec![
        sprint159_replay_example(
            "replay-review-aapl",
            "AAPL",
            MarketScope::UsShortTerm,
            ReplayLabelSource::ReviewRequired,
            ReplayLabelConfidence::ReviewRequired,
            MemberExperienceOutcome::Unknown,
        ),
        sprint159_replay_example(
            "replay-low-msft",
            "MSFT",
            MarketScope::UsLongTerm,
            ReplayLabelSource::ValidatedPaperLabel,
            ReplayLabelConfidence::Low,
            MemberExperienceOutcome::PaperPositive,
        ),
        sprint159_replay_example(
            "replay-amb-btc",
            "BTCUSDT",
            MarketScope::CryptoShortTerm,
            ReplayLabelSource::AmbiguousLabel,
            ReplayLabelConfidence::ReviewRequired,
            MemberExperienceOutcome::Unknown,
        ),
    ]);
    let inventory = build_weak_replay_label_inventory(&dataset, None);
    assert_eq!(inventory.weak_label_count, 3);
    assert_eq!(inventory.review_required_count, 1);
    assert!(inventory.low_confidence_count >= 2);
    assert!(inventory.weak_items.iter().any(|item| {
        item.weakness_reasons
            .contains(&WeakReplayLabelReason::ReviewRequiredSource)
    }));
    assert!(inventory.weak_items.iter().any(|item| {
        item.weakness_reasons
            .contains(&WeakReplayLabelReason::LowConfidence)
    }));

    let queue = build_weak_label_review_queue(&inventory);
    assert_eq!(queue.items.len(), inventory.weak_label_count);
    let sorted = queue.sort_by_priority();
    assert_eq!(sorted.items[0].replay_id, "replay-amb-btc");
    assert_eq!(sorted.next_review_items(2).len(), 2);
    assert!(
        inventory
            .weak_items
            .iter()
            .any(|item| item.priority == WeakReplayLabelPriority::Normal)
    );
    assert!(
        inventory
            .weak_items
            .iter()
            .any(|item| item.priority == WeakReplayLabelPriority::High)
    );
}

#[test]
fn sprint159_review_decision_file_loads_and_rejects_remote_non_paper_and_unsafe_inputs() {
    let path = std::path::Path::new("target/sprint159_review_decisions.json");
    let file = PaperLabelReviewDecisionFile {
        schema_version: "paper-label-review.v1".to_string(),
        decision_file_id: "sprint159-review-file".to_string(),
        decisions: vec![sprint159_review_decision(
            "decision-promote-aapl",
            "review-weak-replay-review-aapl",
            "replay-review-aapl",
            PaperLabelReviewer::PriceEvidenceRule,
            PaperLabelReviewDecisionKind::PromoteToValidatedPaperLabel,
            ReplayLabelConfidence::High,
        )],
        paper_only: true,
    };
    std::fs::write(path, serde_json::to_string_pretty(&file).expect("json")).expect("write");
    let loaded =
        load_paper_label_review_decisions_from_local_json(path).expect("load review decisions");
    assert_eq!(loaded.decision_file_id, "sprint159-review-file");
    assert_eq!(loaded.decisions.len(), 1);

    let remote_err = load_paper_label_review_decisions_from_local_json(std::path::Path::new(
        "https://example.invalid/review-decisions.json",
    ))
    .expect_err("remote review decision path rejected");
    assert!(remote_err.contains("local"));

    std::fs::write(
        "target/sprint159_review_decisions_non_paper.json",
        serde_json::json!({
            "schema_version": "paper-label-review.v1",
            "decision_file_id": "bad",
            "paper_only": false,
            "decisions": []
        })
        .to_string(),
    )
    .expect("write non-paper file");
    let non_paper_err = load_paper_label_review_decisions_from_local_json(std::path::Path::new(
        "target/sprint159_review_decisions_non_paper.json",
    ))
    .expect_err("paper-only validation should fail");
    assert!(non_paper_err.contains("paper-only"));

    let unsafe_path = std::path::Path::new("target/sprint159_review_decisions_unsafe.json");
    std::fs::write(
        unsafe_path,
        serde_json::json!({
            "schema_version": "paper-label-review.v1",
            "decision_file_id": "unsafe",
            "paper_only": true,
            "broker_account": "must-not-load",
            "decisions": []
        })
        .to_string(),
    )
    .expect("write unsafe file");
    let unsafe_err = load_paper_label_review_decisions_from_local_json(unsafe_path)
        .expect_err("unsafe rejected");
    assert!(unsafe_err.contains("unsafe"));

    let _ = std::fs::remove_file(path);
    let _ = std::fs::remove_file("target/sprint159_review_decisions_non_paper.json");
    let _ = std::fs::remove_file(unsafe_path);
}

#[test]
fn sprint159_label_confidence_upgrade_promotes_with_evidence_and_blocks_or_rejects_when_needed() {
    let promote_decision = sprint159_review_decision(
        "decision-promote-aapl",
        "review-weak-replay-review-aapl",
        "replay-review-aapl",
        PaperLabelReviewer::PriceEvidenceRule,
        PaperLabelReviewDecisionKind::PromoteToValidatedPaperLabel,
        ReplayLabelConfidence::High,
    );
    let promoted = evaluate_label_confidence_upgrade(
        &promote_decision,
        &sprint156_paper_evidence(
            "evidence-promote-aapl",
            Some("replay-review-aapl"),
            "AAPL",
            MarketScope::UsShortTerm,
            MemberExperienceOutcome::PaperPositive,
            ReplayLabelSource::ReviewRequired,
            ReplayLabelConfidence::ReviewRequired,
            Some(0.03),
        ),
        &LabelConfidenceUpgradePolicy::default(),
    );
    assert_eq!(
        promoted.upgrade_status,
        LabelConfidenceUpgradeStatus::Upgraded
    );
    assert_eq!(promoted.new_source, ReplayLabelSource::ValidatedPaperLabel);

    let needs_more = evaluate_label_confidence_upgrade(
        &promote_decision,
        &sprint156_paper_evidence(
            "evidence-missing-price",
            Some("replay-review-aapl"),
            "AAPL",
            MarketScope::UsShortTerm,
            MemberExperienceOutcome::PaperPositive,
            ReplayLabelSource::ReviewRequired,
            ReplayLabelConfidence::ReviewRequired,
            None,
        ),
        &LabelConfidenceUpgradePolicy::default(),
    );
    assert_eq!(
        needs_more.upgrade_status,
        LabelConfidenceUpgradeStatus::NeedsMoreEvidence
    );

    let rejected = evaluate_label_confidence_upgrade(
        &promote_decision,
        &sprint156_paper_evidence(
            "evidence-contradictory",
            Some("replay-review-aapl"),
            "AAPL",
            MarketScope::UsShortTerm,
            MemberExperienceOutcome::PaperPositive,
            ReplayLabelSource::ReviewRequired,
            ReplayLabelConfidence::ReviewRequired,
            Some(-0.05),
        ),
        &LabelConfidenceUpgradePolicy::default(),
    );
    assert_eq!(
        rejected.upgrade_status,
        LabelConfidenceUpgradeStatus::Rejected
    );

    let blocked_simulated = evaluate_label_confidence_upgrade(
        &promote_decision,
        &sprint156_paper_evidence(
            "evidence-simulated",
            Some("replay-review-aapl"),
            "AAPL",
            MarketScope::UsShortTerm,
            MemberExperienceOutcome::PaperPositive,
            ReplayLabelSource::SimulatedFixture,
            ReplayLabelConfidence::Low,
            Some(0.03),
        ),
        &LabelConfidenceUpgradePolicy::default(),
    );
    assert_eq!(
        blocked_simulated.upgrade_status,
        LabelConfidenceUpgradeStatus::BlockedByPolicy
    );

    let missing_replay_decision = sprint159_review_decision(
        "decision-missing-replay",
        "review-weak-replay-review-aapl",
        "",
        PaperLabelReviewer::OwnerNaturalComment,
        PaperLabelReviewDecisionKind::PromoteToValidatedPaperLabel,
        ReplayLabelConfidence::High,
    );
    let blocked_missing_replay = evaluate_label_confidence_upgrade(
        &missing_replay_decision,
        &sprint156_paper_evidence(
            "evidence-owner-text",
            Some("replay-review-aapl"),
            "AAPL",
            MarketScope::UsShortTerm,
            MemberExperienceOutcome::PaperPositive,
            ReplayLabelSource::ReviewRequired,
            ReplayLabelConfidence::ReviewRequired,
            Some(0.03),
        ),
        &LabelConfidenceUpgradePolicy::default(),
    );
    assert_eq!(
        blocked_missing_replay.upgrade_status,
        LabelConfidenceUpgradeStatus::BlockedByPolicy
    );

    let mismatched_evidence = evaluate_label_confidence_upgrade(
        &promote_decision,
        &sprint156_paper_evidence(
            "evidence-other-replay",
            Some("replay-other"),
            "AAPL",
            MarketScope::UsShortTerm,
            MemberExperienceOutcome::PaperPositive,
            ReplayLabelSource::ReviewRequired,
            ReplayLabelConfidence::ReviewRequired,
            Some(0.03),
        ),
        &LabelConfidenceUpgradePolicy::default(),
    );
    assert_eq!(
        mismatched_evidence.upgrade_status,
        LabelConfidenceUpgradeStatus::BlockedByPolicy
    );
}

#[test]
fn sprint159_training_inclusion_mask_keeps_only_validated_labels() {
    let dataset = sprint156_replay_dataset(vec![
        sprint159_replay_example(
            "replay-valid-high",
            "AAPL",
            MarketScope::UsShortTerm,
            ReplayLabelSource::ValidatedPaperLabel,
            ReplayLabelConfidence::High,
            MemberExperienceOutcome::PaperPositive,
        ),
        sprint159_replay_example(
            "replay-valid-medium",
            "MSFT",
            MarketScope::UsLongTerm,
            ReplayLabelSource::ValidatedPaperLabel,
            ReplayLabelConfidence::Medium,
            MemberExperienceOutcome::PaperNegative,
        ),
        sprint159_replay_example(
            "replay-review",
            "BTCUSDT",
            MarketScope::CryptoShortTerm,
            ReplayLabelSource::ReviewRequired,
            ReplayLabelConfidence::ReviewRequired,
            MemberExperienceOutcome::Unknown,
        ),
        sprint159_replay_example(
            "replay-ambiguous",
            "ETHUSDT",
            MarketScope::CryptoLongTerm,
            ReplayLabelSource::AmbiguousLabel,
            ReplayLabelConfidence::ReviewRequired,
            MemberExperienceOutcome::Unknown,
        ),
    ]);
    let mask =
        build_replay_training_inclusion_mask(&dataset, &ReplayTrainingInclusionPolicy::default());
    assert_eq!(
        mask.included_replay_ids,
        vec!["replay-valid-high", "replay-valid-medium"]
    );
    assert!(mask.excluded_reasons.contains_key("replay-review"));
    assert_eq!(mask.included_count, 2);
    assert_eq!(mask.excluded_count, 2);
}

#[test]
fn sprint159_apply_review_decisions_updates_target_metadata_only_and_improves_design_readiness() {
    let dataset_path = std::path::Path::new("target/sprint159_apply_dataset.json");
    let decision_path = std::path::Path::new("target/sprint159_apply_decisions.json");
    let evidence_path = std::path::Path::new("target/sprint159_apply_evidence.json");
    let output_dataset_path = std::path::Path::new("target/sprint159_apply_updated_dataset.json");
    let output_quality_path = std::path::Path::new("target/sprint159_apply_label_quality.json");
    let output_readiness_path = std::path::Path::new("target/sprint159_apply_readiness.json");
    let dataset = sprint156_replay_dataset(vec![
        sprint159_replay_example(
            "replay-review-aapl",
            "AAPL",
            MarketScope::UsShortTerm,
            ReplayLabelSource::ReviewRequired,
            ReplayLabelConfidence::ReviewRequired,
            MemberExperienceOutcome::Unknown,
        ),
        sprint159_replay_example(
            "replay-low-msft",
            "MSFT",
            MarketScope::UsLongTerm,
            ReplayLabelSource::ValidatedPaperLabel,
            ReplayLabelConfidence::Low,
            MemberExperienceOutcome::PaperPositive,
        ),
    ]);
    let before_inputs: Vec<_> = dataset
        .examples
        .iter()
        .map(|example| example.input_features.clone())
        .collect();
    dataset
        .save_to_local_json(dataset_path)
        .expect("save sprint159 dataset");
    let decision_file = PaperLabelReviewDecisionFile {
        schema_version: "paper-label-review.v1".to_string(),
        decision_file_id: "sprint159-apply".to_string(),
        decisions: vec![sprint159_review_decision(
            "decision-promote-aapl",
            "review-weak-replay-review-aapl",
            "replay-review-aapl",
            PaperLabelReviewer::PriceEvidenceRule,
            PaperLabelReviewDecisionKind::PromoteToValidatedPaperLabel,
            ReplayLabelConfidence::High,
        )],
        paper_only: true,
    };
    std::fs::write(
        decision_path,
        serde_json::to_string_pretty(&decision_file).expect("json"),
    )
    .expect("write decision file");
    let evidence_file = PaperOutcomeEvidenceFile {
        schema_version: "paper-outcome-evidence.v1".to_string(),
        evidence_file_id: "sprint159-apply-evidence".to_string(),
        created_at: None,
        source_label: "unit-test".to_string(),
        records: vec![sprint156_paper_evidence(
            "evidence-promote-aapl",
            Some("replay-review-aapl"),
            "AAPL",
            MarketScope::UsShortTerm,
            MemberExperienceOutcome::PaperPositive,
            ReplayLabelSource::ReviewRequired,
            ReplayLabelConfidence::ReviewRequired,
            Some(0.03),
        )],
        paper_only: true,
    };
    std::fs::write(
        evidence_path,
        serde_json::to_string_pretty(&evidence_file).expect("json"),
    )
    .expect("write evidence file");

    let result = apply_weak_label_review_decisions(&WeakLabelReviewApplyConfig {
        validated_dataset_path: Some(dataset_path.display().to_string()),
        review_decision_path: Some(decision_path.display().to_string()),
        evidence_path: Some(evidence_path.display().to_string()),
        output_validated_dataset_path: Some(output_dataset_path.display().to_string()),
        output_label_quality_path: Some(output_quality_path.display().to_string()),
        output_readiness_path: Some(output_readiness_path.display().to_string()),
        min_validated_label_ratio_required: 0.5,
        dry_run: false,
        paper_only: true,
    })
    .expect("apply weak label reviews");
    assert_eq!(result.decision_count, 1);
    assert!(result.applied_count >= 1);
    assert_eq!(result.new_validated_ratio, 0.5);
    assert_eq!(
        result.readiness_gate.offline_training_design_status,
        OfflineTrainingDesignStatus::ReadyWithWarnings
    );
    assert_eq!(
        result.readiness_gate.readiness_status,
        OfflineTrainingReadinessStatus::ReadyWithWarningsForOfflineTrainingDesign
    );
    assert!(std::path::Path::new(output_dataset_path).exists());
    assert!(std::path::Path::new(output_quality_path).exists());
    assert!(std::path::Path::new(output_readiness_path).exists());

    let updated_review = result
        .updated_validated_dataset
        .examples
        .iter()
        .find(|example| example.replay_id == "replay-review-aapl")
        .expect("updated review replay");
    assert_eq!(
        updated_review
            .target_labels
            .as_ref()
            .map(|labels| labels.label_source),
        Some(ReplayLabelSource::ValidatedPaperLabel)
    );
    let after_inputs: Vec<_> = result
        .updated_validated_dataset
        .examples
        .iter()
        .map(|example| example.input_features.clone())
        .collect();
    assert_eq!(before_inputs, after_inputs);

    let mask = build_replay_training_inclusion_mask(
        &result.updated_validated_dataset,
        &ReplayTrainingInclusionPolicy::default(),
    );
    assert_eq!(mask.included_replay_ids, vec!["replay-review-aapl"]);
    assert!(
        mask.excluded_replay_ids
            .contains(&"replay-low-msft".to_string())
    );

    let serialized = serde_json::to_string(&result).expect("serialize apply result");
    assert!(!serialized.to_ascii_lowercase().contains("broker"));
    assert!(!serialized.to_ascii_lowercase().contains("account"));
    assert!(!serialized.to_ascii_lowercase().contains("live_trade"));

    let repeated = apply_weak_label_review_decisions(&WeakLabelReviewApplyConfig {
        validated_dataset_path: Some(dataset_path.display().to_string()),
        review_decision_path: Some(decision_path.display().to_string()),
        evidence_path: Some(evidence_path.display().to_string()),
        output_validated_dataset_path: None,
        output_label_quality_path: None,
        output_readiness_path: None,
        min_validated_label_ratio_required: 0.5,
        dry_run: true,
        paper_only: true,
    })
    .expect("repeat apply weak label reviews");
    assert_eq!(result.upgrade_results, repeated.upgrade_results);
    assert_eq!(
        result.updated_validated_dataset.examples,
        repeated.updated_validated_dataset.examples
    );

    let _ = std::fs::remove_file(dataset_path);
    let _ = std::fs::remove_file(decision_path);
    let _ = std::fs::remove_file(evidence_path);
    let _ = std::fs::remove_file(output_dataset_path);
    let _ = std::fs::remove_file(output_quality_path);
    let _ = std::fs::remove_file(output_readiness_path);
}

#[test]
fn sprint160_weak_label_closure_inventory_plan_and_apply_are_deterministic_and_keep_unresolved_excluded()
 {
    std::fs::create_dir_all("target").expect("target dir");
    let dataset_path = std::path::Path::new("target/sprint160_closure_dataset.json");
    let decision_path = std::path::Path::new("target/sprint160_closure_decisions.json");
    let evidence_path = std::path::Path::new("target/sprint160_closure_evidence.json");
    let price_path = std::path::Path::new("target/sprint160_closure_prices.json");
    let output_dataset_path = std::path::Path::new("target/sprint160_closure_updated_dataset.json");
    let output_result_path = std::path::Path::new("target/sprint160_closure_result.json");
    let dataset = sprint156_replay_dataset(vec![
        sprint159_replay_example(
            "replay-close-aapl",
            "AAPL",
            MarketScope::UsShortTerm,
            ReplayLabelSource::ReviewRequired,
            ReplayLabelConfidence::ReviewRequired,
            MemberExperienceOutcome::Unknown,
        ),
        sprint159_replay_example(
            "replay-owner-nvda",
            "NVDA",
            MarketScope::UsShortTerm,
            ReplayLabelSource::AmbiguousLabel,
            ReplayLabelConfidence::ReviewRequired,
            MemberExperienceOutcome::PaperPositive,
        ),
        sprint159_replay_example(
            "",
            "TSLA",
            MarketScope::UsShortTerm,
            ReplayLabelSource::ReviewRequired,
            ReplayLabelConfidence::ReviewRequired,
            MemberExperienceOutcome::PaperPositive,
        ),
        sprint159_replay_example(
            "replay-sim-btc",
            "BTCUSDT",
            MarketScope::CryptoShortTerm,
            ReplayLabelSource::SimulatedFixture,
            ReplayLabelConfidence::ReviewRequired,
            MemberExperienceOutcome::PaperPositive,
        ),
        sprint159_replay_example(
            "replay-reject-eth",
            "ETHUSDT",
            MarketScope::CryptoLongTerm,
            ReplayLabelSource::ManualPaperLabel,
            ReplayLabelConfidence::Low,
            MemberExperienceOutcome::PaperPositive,
        ),
    ]);
    dataset
        .save_to_local_json(dataset_path)
        .expect("save sprint160 closure dataset");
    save_price_series_store_to_local_json(&sprint158_price_series_store(), price_path)
        .expect("save sprint160 price store");
    let evidence_file = PaperOutcomeEvidenceFile {
        schema_version: "paper-outcome-evidence.v1".to_string(),
        evidence_file_id: "sprint160-closure-evidence".to_string(),
        created_at: None,
        source_label: "unit-test".to_string(),
        records: vec![sprint156_paper_evidence(
            "evidence-reject-eth",
            Some("replay-reject-eth"),
            "ETHUSDT",
            MarketScope::CryptoLongTerm,
            MemberExperienceOutcome::PaperPositive,
            ReplayLabelSource::ManualPaperLabel,
            ReplayLabelConfidence::Low,
            Some(-0.05),
        )],
        paper_only: true,
    };
    std::fs::write(
        evidence_path,
        serde_json::to_string_pretty(&evidence_file).expect("json"),
    )
    .expect("write sprint160 evidence");
    let decision_file = PaperLabelReviewDecisionFile {
        schema_version: "paper-label-review.v1".to_string(),
        decision_file_id: "sprint160-closure-decisions".to_string(),
        decisions: vec![sprint159_review_decision(
            "decision-owner-nvda",
            "plan-closure-weak-replay-owner-nvda",
            "replay-owner-nvda",
            PaperLabelReviewer::ManualPaperReview,
            PaperLabelReviewDecisionKind::KeepReviewRequired,
            ReplayLabelConfidence::ReviewRequired,
        )],
        paper_only: true,
    };
    std::fs::write(
        decision_path,
        serde_json::to_string_pretty(&decision_file).expect("json"),
    )
    .expect("write sprint160 decision");

    let evidence_records = evidence_file.records.clone();
    let price_store =
        load_price_series_store_from_local_json(price_path).expect("load price store");
    let inventory =
        build_weak_label_closure_inventory(&dataset, &evidence_records, Some(&price_store));
    assert_eq!(inventory.weak_label_count, 5);
    assert_eq!(inventory.closable_with_existing_evidence_count, 1);
    assert_eq!(inventory.needs_owner_review_count, 1);
    assert_eq!(inventory.needs_id_backfill_count, 1);
    assert_eq!(inventory.keep_excluded_count, 1);
    assert_eq!(inventory.reject_count, 1);
    assert!(inventory.items.iter().any(|item| {
        item.replay_id == "replay-close-aapl"
            && item.closure_status == WeakLabelClosureStatus::ClosableWithExistingEvidence
    }));
    let plan = build_weak_label_closure_plan(&inventory);
    assert_eq!(plan.expected_closable_count, 1);
    assert!(
        plan.planned_actions
            .iter()
            .any(|action| action.replay_id == "replay-owner-nvda" && action.requires_owner_input)
    );

    let result = apply_weak_label_closure_plan(&WeakLabelClosureApplyConfig {
        validated_dataset_path: Some(dataset_path.display().to_string()),
        evidence_path: Some(evidence_path.display().to_string()),
        price_series_path: Some(price_path.display().to_string()),
        owner_review_decision_path: Some(decision_path.display().to_string()),
        output_validated_dataset_path: Some(output_dataset_path.display().to_string()),
        output_closure_result_path: Some(output_result_path.display().to_string()),
        dry_run: false,
        paper_only: true,
    })
    .expect("apply sprint160 closure");
    assert_eq!(result.auto_closed_count, 1);
    assert_eq!(result.owner_review_needed_count, 1);
    assert_eq!(result.rejected_count, 1);
    assert!(result.kept_excluded_count >= 2);
    assert!(std::path::Path::new(output_dataset_path).exists());
    assert!(std::path::Path::new(output_result_path).exists());
    assert_eq!(result.owner_review_prompts.prompt_count, 1);
    assert!(
        result.owner_review_prompts.prompts[0]
            .question
            .contains("이 paper 판단은")
    );
    assert!(
        result.owner_review_prompts.prompts[0]
            .question
            .contains("KeepReviewRequired")
    );
    assert!(
        !result.owner_review_prompts.prompts[0]
            .question
            .contains('{')
    );
    assert_eq!(
        result.owner_review_prompts.prompts[0].allowed_answers,
        vec![
            OwnerPaperReviewAnswer::Positive,
            OwnerPaperReviewAnswer::Negative,
            OwnerPaperReviewAnswer::Neutral,
            OwnerPaperReviewAnswer::KeepReviewRequired,
            OwnerPaperReviewAnswer::Reject,
        ]
    );
    let prompt_serialized = serde_json::to_string(&result.owner_review_prompts)
        .expect("serialize owner review prompts")
        .to_ascii_lowercase();
    for forbidden in [
        "broker",
        "account",
        "broker account",
        "live trading",
        "execute order",
        "place order",
        "submit order",
    ] {
        assert!(
            !prompt_serialized.contains(forbidden),
            "owner review prompt leaked forbidden fragment: {forbidden}"
        );
    }
    let auto_closed = result
        .updated_validated_dataset
        .examples
        .iter()
        .find(|example| example.replay_id == "replay-close-aapl")
        .expect("auto-closed replay");
    assert_eq!(
        auto_closed
            .target_labels
            .as_ref()
            .map(|labels| labels.label_source),
        Some(ReplayLabelSource::ValidatedPaperLabel)
    );
    let rejected = result
        .updated_validated_dataset
        .examples
        .iter()
        .find(|example| example.replay_id == "replay-reject-eth")
        .expect("rejected replay");
    assert_eq!(
        rejected
            .target_labels
            .as_ref()
            .map(|labels| labels.label_source),
        Some(ReplayLabelSource::RejectedLabel)
    );
    assert_eq!(
        result.updated_training_inclusion_mask.included_replay_ids,
        vec!["replay-close-aapl"]
    );
    assert!(
        result
            .updated_training_inclusion_mask
            .excluded_replay_ids
            .contains(&"replay-owner-nvda".to_string())
    );
    let repeated = apply_weak_label_closure_plan(&WeakLabelClosureApplyConfig {
        validated_dataset_path: Some(dataset_path.display().to_string()),
        evidence_path: Some(evidence_path.display().to_string()),
        price_series_path: Some(price_path.display().to_string()),
        owner_review_decision_path: Some(decision_path.display().to_string()),
        output_validated_dataset_path: None,
        output_closure_result_path: None,
        dry_run: true,
        paper_only: true,
    })
    .expect("repeat sprint160 closure");
    assert_eq!(result.inventory, repeated.inventory);
    assert_eq!(result.plan, repeated.plan);
    assert_eq!(result.owner_review_prompts, repeated.owner_review_prompts);
    let serialized = serde_json::to_string(&result).expect("serialize sprint160 closure");
    assert!(!serialized.contains("\"broker\":"));
    assert!(!serialized.contains("\"order\":"));
    assert!(!serialized.contains("\"account\":"));
    assert!(!serialized.contains("\"live_inference\":"));

    let _ = std::fs::remove_file(dataset_path);
    let _ = std::fs::remove_file(decision_path);
    let _ = std::fs::remove_file(evidence_path);
    let _ = std::fs::remove_file(price_path);
    let _ = std::fs::remove_file(output_dataset_path);
    let _ = std::fs::remove_file(output_result_path);
}

#[test]
fn sprint160_closure_refuses_mismatched_existing_evidence_for_auto_close() {
    std::fs::create_dir_all("target").expect("target dir");
    let dataset_path = std::path::Path::new("target/sprint160_mismatch_closure_dataset.json");
    let evidence_path = std::path::Path::new("target/sprint160_mismatch_closure_evidence.json");
    let dataset = sprint156_replay_dataset(vec![sprint159_replay_example(
        "replay-mismatch-aapl",
        "AAPL",
        MarketScope::UsShortTerm,
        ReplayLabelSource::ReviewRequired,
        ReplayLabelConfidence::ReviewRequired,
        MemberExperienceOutcome::Unknown,
    )]);
    dataset
        .save_to_local_json(dataset_path)
        .expect("save sprint160 mismatch dataset");
    let evidence_file = PaperOutcomeEvidenceFile {
        schema_version: "paper-outcome-evidence.v1".to_string(),
        evidence_file_id: "sprint160-mismatch-closure-evidence".to_string(),
        created_at: None,
        source_label: "unit-test".to_string(),
        records: vec![sprint156_paper_evidence(
            "evidence-wrong-symbol-aapl",
            Some("replay-mismatch-aapl"),
            "MSFT",
            MarketScope::UsShortTerm,
            MemberExperienceOutcome::PaperPositive,
            ReplayLabelSource::ManualPaperLabel,
            ReplayLabelConfidence::Medium,
            Some(0.03),
        )],
        paper_only: true,
    };
    std::fs::write(
        evidence_path,
        serde_json::to_string_pretty(&evidence_file).expect("json"),
    )
    .expect("write sprint160 mismatch evidence");

    let evidence_records = load_paper_outcome_evidence_from_local_json(evidence_path)
        .expect("load sprint160 mismatch evidence")
        .records;
    let inventory = build_weak_label_closure_inventory(&dataset, &evidence_records, None);
    assert_eq!(inventory.weak_label_count, 1);
    assert_eq!(inventory.closable_with_existing_evidence_count, 0);
    assert_eq!(inventory.needs_price_evidence_count, 1);

    let result = apply_weak_label_closure_plan(&WeakLabelClosureApplyConfig {
        validated_dataset_path: Some(dataset_path.display().to_string()),
        evidence_path: Some(evidence_path.display().to_string()),
        price_series_path: None,
        owner_review_decision_path: None,
        output_validated_dataset_path: None,
        output_closure_result_path: None,
        dry_run: true,
        paper_only: true,
    })
    .expect("apply sprint160 mismatch closure");
    assert_eq!(result.auto_closed_count, 0);
    assert_eq!(result.new_weak_label_count, 1);
    assert!(
        result
            .updated_training_inclusion_mask
            .excluded_replay_ids
            .contains(&"replay-mismatch-aapl".to_string())
    );

    let _ = std::fs::remove_file(dataset_path);
    let _ = std::fs::remove_file(evidence_path);
}

#[test]
fn sprint160_training_candidate_split_and_dry_run_contract_stay_deterministic() {
    let dataset = sprint156_replay_dataset(vec![
        sprint160_training_example(
            "train-aapl-1",
            "TrendEntryAI",
            "AAPL",
            MarketScope::UsShortTerm,
            ReplayLabelSource::ValidatedPaperLabel,
            ReplayLabelConfidence::Medium,
            MemberExperienceOutcome::PaperPositive,
            "AAPL sanitized one",
        ),
        sprint160_training_example(
            "train-aapl-2",
            "TrendEntryAI",
            "AAPL",
            MarketScope::UsShortTerm,
            ReplayLabelSource::ValidatedPaperLabel,
            ReplayLabelConfidence::High,
            MemberExperienceOutcome::PaperNegative,
            "AAPL sanitized two",
        ),
        sprint160_training_example(
            "train-msft-1",
            "MacroSwingAI",
            "MSFT",
            MarketScope::UsLongTerm,
            ReplayLabelSource::ValidatedPaperLabel,
            ReplayLabelConfidence::Medium,
            MemberExperienceOutcome::PaperPositive,
            "MSFT sanitized one",
        ),
        sprint160_training_example(
            "train-nvda-1",
            "RiskSentinelAI",
            "NVDA",
            MarketScope::UsShortTerm,
            ReplayLabelSource::ValidatedPaperLabel,
            ReplayLabelConfidence::High,
            MemberExperienceOutcome::PaperNeutral,
            "NVDA sanitized one",
        ),
        sprint160_training_example(
            "train-review-skip",
            "TrendEntryAI",
            "QQQ",
            MarketScope::UsShortTerm,
            ReplayLabelSource::ReviewRequired,
            ReplayLabelConfidence::ReviewRequired,
            MemberExperienceOutcome::PaperPositive,
            "QQQ sanitized review",
        ),
        sprint160_training_example(
            "train-sim-skip",
            "MacroSwingAI",
            "BTCUSDT",
            MarketScope::CryptoShortTerm,
            ReplayLabelSource::SimulatedFixture,
            ReplayLabelConfidence::ReviewRequired,
            MemberExperienceOutcome::PaperPositive,
            "BTC sanitized sim",
        ),
        sprint160_training_example(
            "train-not-in-mask",
            "RiskSentinelAI",
            "TSLA",
            MarketScope::UsShortTerm,
            ReplayLabelSource::ValidatedPaperLabel,
            ReplayLabelConfidence::High,
            MemberExperienceOutcome::PaperPositive,
            "TSLA sanitized hidden",
        ),
    ]);
    let inclusion_mask =
        soma_zero::league::minimal_ai_committee_core::ReplayTrainingInclusionMask {
            included_replay_ids: vec![
                "train-aapl-1".to_string(),
                "train-aapl-2".to_string(),
                "train-msft-1".to_string(),
                "train-nvda-1".to_string(),
                "train-review-skip".to_string(),
                "train-sim-skip".to_string(),
            ],
            excluded_replay_ids: vec!["train-not-in-mask".to_string()],
            excluded_reasons: std::collections::BTreeMap::from([(
                "train-not-in-mask".to_string(),
                "excluded by owner selection".to_string(),
            )]),
            included_count: 6,
            excluded_count: 1,
            paper_only: true,
        };
    let candidate_dataset = build_training_candidate_dataset(
        &dataset,
        &inclusion_mask,
        &TrainingCandidateBuildConfig::default(),
    );
    assert_eq!(candidate_dataset.example_count, 4);
    assert_eq!(
        candidate_dataset
            .examples
            .iter()
            .map(|example| example.replay_id.as_str())
            .collect::<Vec<_>>(),
        vec![
            "train-aapl-1",
            "train-aapl-2",
            "train-msft-1",
            "train-nvda-1"
        ]
    );
    assert_eq!(
        candidate_dataset.examples[0]
            .sanitized_input_features
            .market_data_summary,
        "AAPL sanitized one"
    );
    assert_eq!(
        candidate_dataset.member_count, 3,
        "only inclusion-mask-approved validated examples should remain"
    );
    let split_config = TrainingSplitConfig {
        train_ratio: 0.7,
        validation_ratio: 0.15,
        test_ratio: 0.15,
        split_seed: "sprint160-seed".to_string(),
        stratify_by: TrainingSplitStratifyBy::Member,
        paper_only: true,
    };
    let split = build_training_split(&candidate_dataset, &split_config);
    let repeated_split = build_training_split(&candidate_dataset, &split_config);
    assert_eq!(split, repeated_split);
    assert_eq!(
        split.train_count + split.validation_count + split.test_count,
        candidate_dataset.example_count
    );
    let small_dataset = soma_zero::league::minimal_ai_committee_core::TrainingCandidateDataset {
        dataset_id: "small-training-dataset".to_string(),
        source_validated_dataset_id: dataset.dataset_id.clone(),
        examples: candidate_dataset.examples[..2].to_vec(),
        example_count: 2,
        member_count: 1,
        symbol_count: 1,
        market_scope_count: 1,
        label_source_distribution: std::collections::BTreeMap::new(),
        confidence_distribution: std::collections::BTreeMap::new(),
        paper_only: true,
    };
    let small_split = build_training_split(
        &small_dataset,
        &TrainingSplitConfig {
            split_seed: "small-seed".to_string(),
            ..split_config.clone()
        },
    );
    assert!(
        small_split
            .split_warnings
            .iter()
            .any(|warning| warning.contains("dataset too small"))
    );

    let spec = build_offline_trainer_dry_run_spec(&candidate_dataset, &split);
    assert_eq!(
        spec.core_family,
        soma_zero::league::minimal_ai_committee_core::OfflineTrainerCoreFamily::SmartMemberCoreV2
    );
    assert_eq!(
        spec.temporal_core,
        SmartCoreV2ComponentStatus::Mamba3Deferred
    );
    assert_eq!(
        spec.memory_core,
        SmartCoreV2ComponentStatus::GatedDeltaNetDeferred
    );
    let dry_run = run_offline_trainer_data_loader_dry_run(&candidate_dataset, &split, &spec);
    assert_eq!(dry_run.example_count, candidate_dataset.example_count);
    assert!(dry_run.batch_count >= 1);
    assert!(dry_run.no_training_executed);
    assert!(dry_run.no_weight_mutation);
    assert!(matches!(
        dry_run.dry_run_status,
        OfflineTrainerDryRunStatus::Passed | OfflineTrainerDryRunStatus::PassedWithWarnings
    ));
    let serialized = serde_json::to_string(&spec).expect("serialize sprint160 spec");
    assert!(!serialized.contains("\"broker\":"));
    assert!(!serialized.contains("\"order\":"));
    assert!(!serialized.contains("\"account\":"));
    assert!(!serialized.contains("\"live_inference\":"));
}

#[test]
fn sprint160_design_gate_handles_warning_and_safety_blocking_paths() {
    let dataset = sprint156_replay_dataset(vec![
        sprint160_training_example(
            "gate-aapl",
            "TrendEntryAI",
            "AAPL",
            MarketScope::UsShortTerm,
            ReplayLabelSource::ValidatedPaperLabel,
            ReplayLabelConfidence::Medium,
            MemberExperienceOutcome::PaperPositive,
            "AAPL gate one",
        ),
        sprint160_training_example(
            "gate-msft",
            "MacroSwingAI",
            "MSFT",
            MarketScope::UsLongTerm,
            ReplayLabelSource::ValidatedPaperLabel,
            ReplayLabelConfidence::High,
            MemberExperienceOutcome::PaperNegative,
            "MSFT gate one",
        ),
        sprint160_training_example(
            "gate-nvda",
            "RiskSentinelAI",
            "NVDA",
            MarketScope::UsShortTerm,
            ReplayLabelSource::ValidatedPaperLabel,
            ReplayLabelConfidence::Medium,
            MemberExperienceOutcome::PaperNeutral,
            "NVDA gate one",
        ),
        sprint160_training_example(
            "gate-tsla",
            "TrendEntryAI",
            "TSLA",
            MarketScope::UsShortTerm,
            ReplayLabelSource::ValidatedPaperLabel,
            ReplayLabelConfidence::High,
            MemberExperienceOutcome::PaperPositive,
            "TSLA gate one",
        ),
    ]);
    let mask =
        build_replay_training_inclusion_mask(&dataset, &ReplayTrainingInclusionPolicy::default());
    let candidate_dataset =
        build_training_candidate_dataset(&dataset, &mask, &TrainingCandidateBuildConfig::default());
    let split = build_training_split(
        &candidate_dataset,
        &TrainingSplitConfig {
            train_ratio: 0.7,
            validation_ratio: 0.15,
            test_ratio: 0.15,
            split_seed: "gate-seed".to_string(),
            stratify_by: TrainingSplitStratifyBy::Member,
            paper_only: true,
        },
    );
    let dry_run = run_offline_trainer_data_loader_dry_run(
        &candidate_dataset,
        &split,
        &build_offline_trainer_dry_run_spec(&candidate_dataset, &split),
    );
    let quality_eval =
        soma_zero::league::minimal_ai_committee_core::run_replay_quality_evaluation_with_thresholds(
            &MemberExperienceStore::new("sprint160-empty-store", Vec::new()),
            &dataset,
            2,
            1,
        );
    let ready_with_warnings =
        evaluate_offline_training_design_gate(&candidate_dataset, &split, &dry_run, &quality_eval);
    assert_eq!(
        ready_with_warnings.design_status,
        OfflineTrainingDesignGateStatus::ReadyWithWarnings
    );

    let mut unsafe_dataset = dataset.clone();
    unsafe_dataset.examples[0]
        .input_features
        .market_data_summary = "broker account post-result leak".to_string();
    let unsafe_quality_eval =
        soma_zero::league::minimal_ai_committee_core::run_replay_quality_evaluation_with_thresholds(
            &MemberExperienceStore::new("sprint160-unsafe-store", Vec::new()),
            &unsafe_dataset,
            2,
            1,
        );
    assert!(matches!(
        unsafe_quality_eval.leakage_check.leakage_status,
        ReplayLeakageStatus::LeakageDetected | ReplayLeakageStatus::UnsafeForTraining
    ));
    let blocked = evaluate_offline_training_design_gate(
        &candidate_dataset,
        &split,
        &dry_run,
        &unsafe_quality_eval,
    );
    assert_eq!(
        blocked.design_status,
        OfflineTrainingDesignGateStatus::BlockedBySafety
    );
}

#[test]
fn sprint165_feature_and_target_rows_keep_paper_only_contract() {
    let dataset = sprint156_replay_dataset(vec![
        sprint160_training_example(
            "trainer-safe",
            "TrendEntryAI",
            "AAPL",
            MarketScope::UsShortTerm,
            ReplayLabelSource::ValidatedPaperLabel,
            ReplayLabelConfidence::High,
            MemberExperienceOutcome::PaperPositive,
            "AAPL sanitized feature",
        ),
        sprint160_training_example(
            "trainer-review",
            "TrendEntryAI",
            "MSFT",
            MarketScope::UsShortTerm,
            ReplayLabelSource::ReviewRequired,
            ReplayLabelConfidence::ReviewRequired,
            MemberExperienceOutcome::PaperNegative,
            "MSFT sanitized review",
        ),
        sprint160_training_example(
            "trainer-simulated",
            "EvidenceRegimeAI",
            "NVDA",
            MarketScope::UsLongTerm,
            ReplayLabelSource::SimulatedFixture,
            ReplayLabelConfidence::ReviewRequired,
            MemberExperienceOutcome::PaperPositive,
            "NVDA sanitized simulated",
        ),
    ]);
    let mask =
        build_replay_training_inclusion_mask(&dataset, &ReplayTrainingInclusionPolicy::default());
    let candidate_dataset =
        build_training_candidate_dataset(&dataset, &mask, &TrainingCandidateBuildConfig::default());
    let feature_schema = TrainingFeatureSchema::default();
    let target_schema = TrainingTargetSchema::default();

    let safe_example = candidate_dataset
        .examples
        .iter()
        .find(|example| example.replay_id == "trainer-safe")
        .expect("safe candidate example");
    let feature_row =
        build_training_feature_row(safe_example, &feature_schema).expect("safe feature row");
    assert_eq!(
        feature_row.market_features,
        vec!["AAPL sanitized feature".to_string()]
    );
    assert!(
        build_training_target_row(safe_example, &target_schema).is_some(),
        "validated labels should remain targetable"
    );

    let mut leaked_example = safe_example.clone();
    leaked_example.sanitized_input_features.news_summary =
        "broker account result after trade".to_string();
    assert!(
        build_training_feature_row(&leaked_example, &feature_schema).is_err(),
        "broker/account/order style leakage must be rejected"
    );
    let mut leaked_note_example = safe_example.clone();
    leaked_note_example
        .sanitized_input_features
        .feature_safety_notes
        .push("broker account order must not leak through notes".to_string());
    assert!(
        build_training_feature_row(&leaked_note_example, &feature_schema).is_err(),
        "feature safety notes must not leak broker/account/order terms"
    );

    let mut review_candidate = safe_example.clone();
    review_candidate.label_source = ReplayLabelSource::ReviewRequired;
    review_candidate.label_confidence = ReplayLabelConfidence::ReviewRequired;
    assert!(
        build_training_target_row(&review_candidate, &target_schema).is_none(),
        "review-required labels must stay out of trainer targets"
    );

    let mut simulated_candidate = safe_example.clone();
    simulated_candidate.label_source = ReplayLabelSource::SimulatedFixture;
    simulated_candidate.label_confidence = ReplayLabelConfidence::ReviewRequired;
    assert!(
        build_training_target_row(&simulated_candidate, &target_schema).is_none(),
        "simulated fixtures must stay out of trainer targets"
    );
}

#[test]
fn sprint165_batches_and_dry_run_v2_stay_deterministic_and_per_member() {
    let dataset = sprint156_replay_dataset(vec![
        sprint160_training_example(
            "trainer-aapl-1",
            "TrendEntryAI",
            "AAPL",
            MarketScope::UsShortTerm,
            ReplayLabelSource::ValidatedPaperLabel,
            ReplayLabelConfidence::High,
            MemberExperienceOutcome::PaperPositive,
            "AAPL sanitized one",
        ),
        sprint160_training_example(
            "trainer-aapl-2",
            "TrendEntryAI",
            "AAPL",
            MarketScope::UsShortTerm,
            ReplayLabelSource::ValidatedPaperLabel,
            ReplayLabelConfidence::Medium,
            MemberExperienceOutcome::PaperNegative,
            "AAPL sanitized two",
        ),
        sprint160_training_example(
            "trainer-msft-1",
            "RiskGuardAI",
            "MSFT",
            MarketScope::UsLongTerm,
            ReplayLabelSource::ValidatedPaperLabel,
            ReplayLabelConfidence::High,
            MemberExperienceOutcome::PaperPositive,
            "MSFT sanitized one",
        ),
        sprint160_training_example(
            "trainer-msft-2",
            "RiskGuardAI",
            "MSFT",
            MarketScope::UsLongTerm,
            ReplayLabelSource::ValidatedPaperLabel,
            ReplayLabelConfidence::Medium,
            MemberExperienceOutcome::PaperNegative,
            "MSFT sanitized two",
        ),
        sprint160_training_example(
            "trainer-nvda-1",
            "EvidenceRegimeAI",
            "NVDA",
            MarketScope::UsShortTerm,
            ReplayLabelSource::ValidatedPaperLabel,
            ReplayLabelConfidence::High,
            MemberExperienceOutcome::PaperNeutral,
            "NVDA sanitized one",
        ),
        sprint160_training_example(
            "trainer-nvda-2",
            "EvidenceRegimeAI",
            "NVDA",
            MarketScope::UsShortTerm,
            ReplayLabelSource::ValidatedPaperLabel,
            ReplayLabelConfidence::Medium,
            MemberExperienceOutcome::PaperPositive,
            "NVDA sanitized two",
        ),
    ]);
    let mask =
        build_replay_training_inclusion_mask(&dataset, &ReplayTrainingInclusionPolicy::default());
    let candidate_dataset =
        build_training_candidate_dataset(&dataset, &mask, &TrainingCandidateBuildConfig::default());
    let split = build_training_split(
        &candidate_dataset,
        &TrainingSplitConfig {
            train_ratio: 0.7,
            validation_ratio: 0.15,
            test_ratio: 0.15,
            split_seed: "sprint165-batch-seed".to_string(),
            stratify_by: TrainingSplitStratifyBy::Member,
            paper_only: true,
        },
    );
    let feature_schema = TrainingFeatureSchema::default();
    let target_schema = TrainingTargetSchema::default();
    let spec = build_smartcore_v2_batch_spec(2, &feature_schema, &target_schema);
    assert_eq!(
        spec.core_family,
        SmartCoreV2Family::Mamba3GatedDeltaNetSparseEvent
    );
    assert_eq!(
        spec.temporal_core,
        SmartCoreV2ComponentStatus::Mamba3Deferred
    );
    assert_eq!(
        spec.memory_core,
        SmartCoreV2ComponentStatus::GatedDeltaNetDeferred
    );
    assert_eq!(
        spec.event_attention_core,
        SmartCoreV2ComponentStatus::SparseEventAttentionDeferred
    );
    let batch_config = TrainingBatchIteratorConfig {
        batch_size: 2,
        shuffle: false,
        split: TrainingBatchSplit::All,
        member_id: None,
        drop_last: false,
        paper_only: true,
    };
    let first_batches = build_training_batches(
        &candidate_dataset,
        &split,
        &feature_schema,
        &target_schema,
        &batch_config,
    )
    .expect("first batch build");
    let repeated_batches = build_training_batches(
        &candidate_dataset,
        &split,
        &feature_schema,
        &target_schema,
        &batch_config,
    )
    .expect("repeat batch build");
    assert_eq!(first_batches, repeated_batches);
    assert!(first_batches.batch_count >= 3);
    let member_by_example_id: std::collections::BTreeMap<String, String> = candidate_dataset
        .examples
        .iter()
        .map(|example| {
            (
                example.training_example_id.clone(),
                example.member_id.clone(),
            )
        })
        .collect();
    for batch in &first_batches.batches {
        assert!(
            batch.example_ids.iter().all(|example_id| {
                member_by_example_id
                    .get(example_id)
                    .map(|member_id| member_id == &batch.member_id)
                    .unwrap_or(false)
            }),
            "every batch must remain per-member"
        );
    }
    for member_id in ["TrendEntryAI", "RiskGuardAI", "EvidenceRegimeAI"] {
        assert!(
            first_batches
                .batches
                .iter()
                .any(|batch| batch.member_id == member_id),
            "pilot member did not produce a batch: {member_id}"
        );
    }
    let first_batch = first_batches.batches.first().expect("first trainer batch");
    let mut strict_loss_contract = SmartCoreV2LossContract::default();
    strict_loss_contract
        .enabled_heads
        .push(SmartCoreV2LossHead::ExpectedReturnHintLoss);
    let strict_loss = validate_loss_contract_against_batch(first_batch, &strict_loss_contract);
    assert!(!strict_loss.valid);
    assert!(
        strict_loss
            .missing_targets
            .contains(&"expected_return_target".to_string())
    );
    let metric_contract = OfflineTrainingMetricContract {
        metrics: vec![OfflineTrainingMetric::ExpectedReturnRankCorrelationDeferred],
        ..OfflineTrainingMetricContract::default()
    };
    let metric_validation =
        validate_metric_contract_against_dataset(&candidate_dataset, &metric_contract);
    assert!(metric_validation.valid);
    assert!(
        metric_validation
            .warnings
            .iter()
            .any(|warning| warning.contains("expected return metric remains deferred"))
    );
    let mut weak_candidate_dataset = candidate_dataset.clone();
    weak_candidate_dataset.examples[0].label_source = ReplayLabelSource::ReviewRequired;
    weak_candidate_dataset.examples[0].label_confidence = ReplayLabelConfidence::ReviewRequired;
    let weak_metric_validation =
        validate_metric_contract_against_dataset(&weak_candidate_dataset, &metric_contract);
    assert!(!weak_metric_validation.valid);

    let dry_run_config = OfflineTrainerDryRunConfigV2 {
        run_id: "sprint165-dry-run-v2".to_string(),
        training_candidate_dataset_path: None,
        training_split_path: None,
        batch_size: 2,
        feature_schema_version: "smartcore-v2".to_string(),
        target_schema_version: "smartcore-v2".to_string(),
        validate_loss_contract: true,
        validate_metric_contract: true,
        paper_only: true,
    };
    let first = run_offline_trainer_dry_run_v2(&candidate_dataset, &split, &dry_run_config);
    let second = run_offline_trainer_dry_run_v2(&candidate_dataset, &split, &dry_run_config);
    assert_eq!(first, second);
    assert!(first.train_batch_count >= 1);
    assert!(first.validation_batch_count >= 1);
    assert!(first.test_batch_count >= 1);
    assert_eq!(first.member_batch_counts.len(), 3);
    assert!(first.no_training_executed);
    assert!(first.no_weight_mutation);
    assert!(first.no_checkpoint_written);
    assert!(matches!(
        first.dry_run_status,
        OfflineTrainerDryRunStatus::Passed | OfflineTrainerDryRunStatus::PassedWithWarnings
    ));
}

#[test]
fn sprint165_design_status_can_be_ready_or_blocked_by_safety() {
    let dataset = sprint156_replay_dataset(vec![
        sprint160_training_example(
            "ready-aapl-1",
            "TrendEntryAI",
            "AAPL",
            MarketScope::UsShortTerm,
            ReplayLabelSource::ValidatedPaperLabel,
            ReplayLabelConfidence::High,
            MemberExperienceOutcome::PaperPositive,
            "AAPL ready one",
        ),
        sprint160_training_example(
            "ready-aapl-2",
            "TrendEntryAI",
            "AAPL",
            MarketScope::UsShortTerm,
            ReplayLabelSource::ValidatedPaperLabel,
            ReplayLabelConfidence::Medium,
            MemberExperienceOutcome::PaperNegative,
            "AAPL ready two",
        ),
        sprint160_training_example(
            "ready-msft-1",
            "RiskGuardAI",
            "MSFT",
            MarketScope::UsLongTerm,
            ReplayLabelSource::ValidatedPaperLabel,
            ReplayLabelConfidence::High,
            MemberExperienceOutcome::PaperPositive,
            "MSFT ready one",
        ),
        sprint160_training_example(
            "ready-msft-2",
            "RiskGuardAI",
            "MSFT",
            MarketScope::UsLongTerm,
            ReplayLabelSource::ValidatedPaperLabel,
            ReplayLabelConfidence::Medium,
            MemberExperienceOutcome::PaperNeutral,
            "MSFT ready two",
        ),
        sprint160_training_example(
            "ready-nvda-1",
            "EvidenceRegimeAI",
            "NVDA",
            MarketScope::UsShortTerm,
            ReplayLabelSource::ValidatedPaperLabel,
            ReplayLabelConfidence::High,
            MemberExperienceOutcome::PaperPositive,
            "NVDA ready one",
        ),
        sprint160_training_example(
            "ready-nvda-2",
            "EvidenceRegimeAI",
            "NVDA",
            MarketScope::UsShortTerm,
            ReplayLabelSource::ValidatedPaperLabel,
            ReplayLabelConfidence::Medium,
            MemberExperienceOutcome::PaperNegative,
            "NVDA ready two",
        ),
    ]);
    let mask =
        build_replay_training_inclusion_mask(&dataset, &ReplayTrainingInclusionPolicy::default());
    let candidate_dataset =
        build_training_candidate_dataset(&dataset, &mask, &TrainingCandidateBuildConfig::default());
    let split = build_training_split(
        &candidate_dataset,
        &TrainingSplitConfig {
            train_ratio: 0.7,
            validation_ratio: 0.15,
            test_ratio: 0.15,
            split_seed: "sprint165-design-seed".to_string(),
            stratify_by: TrainingSplitStratifyBy::Member,
            paper_only: true,
        },
    );
    let dry_run_v1 = run_offline_trainer_data_loader_dry_run(
        &candidate_dataset,
        &split,
        &build_offline_trainer_dry_run_spec(&candidate_dataset, &split),
    );
    let quality_eval = run_replay_quality_evaluation_with_thresholds(
        &MemberExperienceStore::new("sprint165-quality-store", Vec::new()),
        &dataset,
        2,
        1,
    );
    let design_gate = evaluate_offline_training_design_gate(
        &candidate_dataset,
        &split,
        &dry_run_v1,
        &quality_eval,
    );
    let dry_run_v2 = run_offline_trainer_dry_run_v2(
        &candidate_dataset,
        &split,
        &OfflineTrainerDryRunConfigV2 {
            run_id: "sprint165-design-ready".to_string(),
            training_candidate_dataset_path: None,
            training_split_path: None,
            batch_size: 2,
            feature_schema_version: "smartcore-v2".to_string(),
            target_schema_version: "smartcore-v2".to_string(),
            validate_loss_contract: false,
            validate_metric_contract: false,
            paper_only: true,
        },
    );
    let label_quality_summary = summarize_label_quality(&build_validated_replay_dataset(
        &dataset,
        &[],
        soma_zero::league::minimal_ai_committee_core::ValidatedReplayDatasetBuildConfig {
            source_sanitized_dataset_id: dataset.dataset_id.clone(),
            label_validation_policy: PaperLabelValidationPolicy::default(),
            backtest_label_contract: None,
            require_validated_labels_for_training: true,
            allow_ready_with_warnings_for_medium_confidence: true,
            reject_rejected_labels: true,
            paper_only: true,
        },
    ));
    let ready_status =
        evaluate_offline_trainer_design_status(&dry_run_v2, &design_gate, &label_quality_summary);
    assert_eq!(
        ready_status.batch_contract_status,
        OfflineTrainerContractStatus::Ready
    );
    assert_eq!(
        ready_status.loss_contract_status,
        OfflineTrainerContractStatus::ContractOnly
    );
    assert_eq!(
        ready_status.metric_contract_status,
        OfflineTrainerContractStatus::ContractOnly
    );
    assert_eq!(
        ready_status.design_status,
        OfflineTrainerDesignStatusLevel::ReadyForTinyTrainingDryRun
    );

    let mut unsafe_dataset = dataset.clone();
    unsafe_dataset.examples[0]
        .input_features
        .market_data_summary = "broker account leak after execution".to_string();
    let unsafe_quality_eval = run_replay_quality_evaluation_with_thresholds(
        &MemberExperienceStore::new("sprint165-unsafe-quality-store", Vec::new()),
        &unsafe_dataset,
        2,
        1,
    );
    let blocked_gate = evaluate_offline_training_design_gate(
        &candidate_dataset,
        &split,
        &dry_run_v1,
        &unsafe_quality_eval,
    );
    let blocked_status =
        evaluate_offline_trainer_design_status(&dry_run_v2, &blocked_gate, &label_quality_summary);
    assert_eq!(
        blocked_status.design_status,
        OfflineTrainerDesignStatusLevel::BlockedBySafety
    );
    assert!(
        blocked_status
            .blockers
            .iter()
            .any(|blocker| blocker.contains("blocked by safety"))
    );
}

#[test]
fn sprint166_warning_normalization_dedupes_expected_and_preserves_blockers() {
    let normalized = normalize_trainer_warnings(&[
        "loss contract remains contract-only; runtime deferred".to_string(),
        "loss contract remains contract-only; runtime deferred".to_string(),
        "Mamba3 runtime deferred".to_string(),
        "training batch iterator batch_size must be positive".to_string(),
        "existing offline training design gate blocked by safety".to_string(),
    ]);
    assert_eq!(normalized.original_warning_count, 5);
    assert_eq!(normalized.normalized_warning_count, 4);
    assert_eq!(normalized.duplicate_warning_count, 1);
    assert_eq!(normalized.expected_deferred_warning_count, 2);
    assert_eq!(normalized.real_blocker_count, 2);
    assert!(normalized.normalized_warnings.iter().any(|warning| {
        matches!(warning.kind, TrainerWarningKind::LossContractOnlyExpected)
            && matches!(warning.severity, TrainerWarningSeverity::Info)
            && warning.expected
            && !warning.blocker
    }));
    assert!(normalized.normalized_warnings.iter().any(|warning| {
        matches!(warning.kind, TrainerWarningKind::ShapeWarning)
            && matches!(warning.severity, TrainerWarningSeverity::Blocking)
            && warning.blocker
    }));
    assert!(normalized.normalized_warnings.iter().any(|warning| {
        matches!(warning.kind, TrainerWarningKind::SafetyBlockingWarning)
            && matches!(warning.severity, TrainerWarningSeverity::Blocking)
            && warning.blocker
    }));
}

#[test]
fn sprint166_member_batch_readiness_and_loss_metric_summary_cover_pilot_members() {
    let dataset = sprint166_balanced_pilot_replay_dataset();
    let mask =
        build_replay_training_inclusion_mask(&dataset, &ReplayTrainingInclusionPolicy::default());
    let candidate_dataset =
        build_training_candidate_dataset(&dataset, &mask, &TrainingCandidateBuildConfig::default());
    let split = sprint166_balanced_training_split(&candidate_dataset);
    let all_batches = build_training_batches(
        &candidate_dataset,
        &split,
        &TrainingFeatureSchema::default(),
        &TrainingTargetSchema::default(),
        &TrainingBatchIteratorConfig {
            batch_size: 1,
            shuffle: false,
            split: TrainingBatchSplit::All,
            member_id: None,
            drop_last: false,
            paper_only: true,
        },
    )
    .expect("balanced sprint166 batches");
    let summaries =
        summarize_member_batch_readiness(&candidate_dataset, &split, &all_batches.batches);
    assert_eq!(summaries.len(), 3);
    assert_eq!(
        summaries
            .iter()
            .map(|summary| summary.member_id.as_str())
            .collect::<Vec<_>>(),
        vec!["TrendEntryAI", "RiskGuardAI", "EvidenceRegimeAI"]
    );
    assert!(summaries.iter().all(|summary| summary.batch_status
        == soma_zero::league::minimal_ai_committee_core::MemberBatchReadinessStatus::Ready));
    let mut alias_candidate_dataset = candidate_dataset.clone();
    for example in &mut alias_candidate_dataset.examples {
        example.member_id = match example.member_id.as_str() {
            "TrendEntryAI" => "trend-kr-short".to_string(),
            "RiskGuardAI" => "risk-kr-short".to_string(),
            "EvidenceRegimeAI" => "evidence-kr-short".to_string(),
            other => other.to_string(),
        };
    }
    let alias_batches = build_training_batches(
        &alias_candidate_dataset,
        &split,
        &TrainingFeatureSchema::default(),
        &TrainingTargetSchema::default(),
        &TrainingBatchIteratorConfig {
            batch_size: 1,
            shuffle: false,
            split: TrainingBatchSplit::All,
            member_id: None,
            drop_last: false,
            paper_only: true,
        },
    )
    .expect("alias sprint166 batches");
    let alias_summaries =
        summarize_member_batch_readiness(&alias_candidate_dataset, &split, &alias_batches.batches);
    assert_eq!(
        alias_summaries
            .iter()
            .map(|summary| summary.member_id.as_str())
            .collect::<Vec<_>>(),
        vec!["TrendEntryAI", "RiskGuardAI", "EvidenceRegimeAI"]
    );
    assert!(alias_summaries.iter().all(|summary| summary.batch_status
        == soma_zero::league::minimal_ai_committee_core::MemberBatchReadinessStatus::Ready));

    let mut thin_candidate_dataset = candidate_dataset.clone();
    thin_candidate_dataset.examples.retain(|example| {
        example.member_id != "EvidenceRegimeAI" || example.replay_id == "s166-evidence-train"
    });
    let thin_split = TrainingSplitResult {
        dataset_id: thin_candidate_dataset.dataset_id.clone(),
        train_ids: thin_candidate_dataset
            .examples
            .iter()
            .filter(|example| {
                matches!(
                    example.replay_id.as_str(),
                    "s166-trend-train" | "s166-risk-train" | "s166-evidence-train"
                )
            })
            .map(|example| example.training_example_id.clone())
            .collect(),
        validation_ids: thin_candidate_dataset
            .examples
            .iter()
            .filter(|example| {
                matches!(
                    example.replay_id.as_str(),
                    "s166-trend-validation" | "s166-risk-validation"
                )
            })
            .map(|example| example.training_example_id.clone())
            .collect(),
        test_ids: thin_candidate_dataset
            .examples
            .iter()
            .filter(|example| {
                matches!(
                    example.replay_id.as_str(),
                    "s166-trend-test" | "s166-risk-test"
                )
            })
            .map(|example| example.training_example_id.clone())
            .collect(),
        train_count: 3,
        validation_count: 2,
        test_count: 2,
        split_warnings: Vec::new(),
        paper_only: true,
    };
    let thin_batches = build_training_batches(
        &thin_candidate_dataset,
        &thin_split,
        &TrainingFeatureSchema::default(),
        &TrainingTargetSchema::default(),
        &TrainingBatchIteratorConfig {
            batch_size: 1,
            shuffle: false,
            split: TrainingBatchSplit::All,
            member_id: None,
            drop_last: false,
            paper_only: true,
        },
    )
    .expect("thin sprint166 batches");
    let thin_summaries = summarize_member_batch_readiness(
        &thin_candidate_dataset,
        &thin_split,
        &thin_batches.batches,
    );
    let evidence_summary = thin_summaries
        .iter()
        .find(|summary| summary.member_id == "EvidenceRegimeAI")
        .expect("evidence regime summary");
    assert!(matches!(
        evidence_summary.batch_status,
        soma_zero::league::minimal_ai_committee_core::MemberBatchReadinessStatus::ReadyWithWarnings
    ));
    assert!(
        evidence_summary
            .warnings
            .iter()
            .any(|warning| warning.contains("thin") || warning.contains("validation split"))
    );

    let mut zero_member_dataset = thin_candidate_dataset.clone();
    zero_member_dataset
        .examples
        .retain(|example| example.member_id != "EvidenceRegimeAI");
    let zero_member_split = TrainingSplitResult {
        dataset_id: zero_member_dataset.dataset_id.clone(),
        train_ids: zero_member_dataset
            .examples
            .iter()
            .filter(|example| {
                matches!(
                    example.replay_id.as_str(),
                    "s166-trend-train" | "s166-risk-train"
                )
            })
            .map(|example| example.training_example_id.clone())
            .collect(),
        validation_ids: zero_member_dataset
            .examples
            .iter()
            .filter(|example| {
                matches!(
                    example.replay_id.as_str(),
                    "s166-trend-validation" | "s166-risk-validation"
                )
            })
            .map(|example| example.training_example_id.clone())
            .collect(),
        test_ids: zero_member_dataset
            .examples
            .iter()
            .filter(|example| {
                matches!(
                    example.replay_id.as_str(),
                    "s166-trend-test" | "s166-risk-test"
                )
            })
            .map(|example| example.training_example_id.clone())
            .collect(),
        train_count: 2,
        validation_count: 2,
        test_count: 2,
        split_warnings: Vec::new(),
        paper_only: true,
    };
    let zero_summaries =
        summarize_member_batch_readiness(&zero_member_dataset, &zero_member_split, &[]);
    let zero_evidence_summary = zero_summaries
        .iter()
        .find(|summary| summary.member_id == "EvidenceRegimeAI")
        .expect("missing evidence regime summary");
    assert!(matches!(
        zero_evidence_summary.batch_status,
        soma_zero::league::minimal_ai_committee_core::MemberBatchReadinessStatus::InsufficientData
    ));

    let default_loss_validation = validate_loss_contract_against_batch(
        &all_batches.batches[0],
        &SmartCoreV2LossContract::default(),
    );
    let default_metric_validation = validate_metric_contract_against_dataset(
        &candidate_dataset,
        &OfflineTrainingMetricContract::default(),
    );
    let contract_only_summary =
        summarize_loss_metric_readiness(&default_loss_validation, &default_metric_validation);
    assert_eq!(
        contract_only_summary.loss_contract_status,
        LossMetricReadinessStatus::ContractOnly
    );
    assert_eq!(
        contract_only_summary.metric_contract_status,
        LossMetricReadinessStatus::ContractOnly
    );

    let missing_loss_validation = validate_loss_contract_against_batch(
        &all_batches.batches[0],
        &SmartCoreV2LossContract {
            contract_id: "missing-expected-return".to_string(),
            enabled_heads: vec![SmartCoreV2LossHead::ExpectedReturnHintLoss],
            head_weights: [("ExpectedReturnHintLoss".to_string(), 1.0)]
                .into_iter()
                .collect(),
            loss_status:
                soma_zero::league::minimal_ai_committee_core::SmartCoreV2LossStatus::ContractOnly,
            paper_only: true,
        },
    );
    let mut metric_candidate_dataset = candidate_dataset.clone();
    metric_candidate_dataset.examples[0].label_confidence = ReplayLabelConfidence::ReviewRequired;
    metric_candidate_dataset.examples[0]
        .target_labels
        .label_confidence = ReplayLabelConfidence::ReviewRequired;
    let missing_metric_validation = validate_metric_contract_against_dataset(
        &metric_candidate_dataset,
        &OfflineTrainingMetricContract::default(),
    );
    let missing_summary =
        summarize_loss_metric_readiness(&missing_loss_validation, &missing_metric_validation);
    assert_eq!(
        missing_summary.loss_contract_status,
        LossMetricReadinessStatus::MissingTargets
    );
    assert_eq!(
        missing_summary.metric_contract_status,
        LossMetricReadinessStatus::MissingTargets
    );
}

#[test]
fn sprint166_readiness_brief_and_gate_are_deterministic_and_eligible_with_expected_warnings() {
    let dataset = sprint166_balanced_pilot_replay_dataset();
    let mask =
        build_replay_training_inclusion_mask(&dataset, &ReplayTrainingInclusionPolicy::default());
    let candidate_dataset =
        build_training_candidate_dataset(&dataset, &mask, &TrainingCandidateBuildConfig::default());
    let split = sprint166_balanced_training_split(&candidate_dataset);
    let dry_run_v1 = run_offline_trainer_data_loader_dry_run(
        &candidate_dataset,
        &split,
        &build_offline_trainer_dry_run_spec(&candidate_dataset, &split),
    );
    let quality_eval = run_replay_quality_evaluation_with_thresholds(
        &MemberExperienceStore::new("sprint166-quality-store", Vec::new()),
        &dataset,
        2,
        1,
    );
    let design_gate = evaluate_offline_training_design_gate(
        &candidate_dataset,
        &split,
        &dry_run_v1,
        &quality_eval,
    );
    let dry_run_v2 = run_offline_trainer_dry_run_v2(
        &candidate_dataset,
        &split,
        &OfflineTrainerDryRunConfigV2 {
            run_id: "sprint166-readiness".to_string(),
            training_candidate_dataset_path: None,
            training_split_path: None,
            batch_size: 1,
            feature_schema_version: "smartcore-v2".to_string(),
            target_schema_version: "smartcore-v2".to_string(),
            validate_loss_contract: true,
            validate_metric_contract: true,
            paper_only: true,
        },
    );
    let label_quality_summary = summarize_label_quality(&build_validated_replay_dataset(
        &dataset,
        &[],
        soma_zero::league::minimal_ai_committee_core::ValidatedReplayDatasetBuildConfig {
            source_sanitized_dataset_id: dataset.dataset_id.clone(),
            label_validation_policy: PaperLabelValidationPolicy::default(),
            backtest_label_contract: None,
            require_validated_labels_for_training: true,
            allow_ready_with_warnings_for_medium_confidence: true,
            reject_rejected_labels: true,
            paper_only: true,
        },
    ));
    let trainer_design_status =
        evaluate_offline_trainer_design_status(&dry_run_v2, &design_gate, &label_quality_summary);
    let first_brief = build_offline_trainer_readiness_brief(
        &dry_run_v2,
        &trainer_design_status,
        &candidate_dataset,
        &split,
    );
    let repeated_brief = build_offline_trainer_readiness_brief(
        &dry_run_v2,
        &trainer_design_status,
        &candidate_dataset,
        &split,
    );
    assert_eq!(first_brief, repeated_brief);
    assert!(first_brief.no_training_executed);
    assert!(first_brief.no_weight_mutation);
    assert!(first_brief.no_checkpoint_written);
    assert!(
        first_brief
            .human_readable_summary
            .contains("no weight mutation")
    );
    assert!(matches!(
        first_brief.next_allowed_step,
        OfflineTrainerReadinessNextStep::TinyTrainingDryRunAllowedWithWarnings
    ));
    let gate =
        evaluate_tiny_training_eligibility(&first_brief, &TinyTrainingEligibilityPolicy::default());
    assert!(gate.eligible_for_tiny_training_dry_run);
    assert_eq!(
        gate.eligibility_status,
        TinyTrainingEligibilityStatus::EligibleWithWarnings
    );
    assert!(gate.no_checkpoint_written);
    assert!(gate.no_live_inference);
    assert!(gate.no_broker_order_account);
}

#[test]
fn sprint166_tiny_training_gate_blocks_safety_and_weak_labels() {
    let dataset = sprint166_balanced_pilot_replay_dataset();
    let mask =
        build_replay_training_inclusion_mask(&dataset, &ReplayTrainingInclusionPolicy::default());
    let candidate_dataset =
        build_training_candidate_dataset(&dataset, &mask, &TrainingCandidateBuildConfig::default());
    let split = sprint166_balanced_training_split(&candidate_dataset);
    let dry_run_v1 = run_offline_trainer_data_loader_dry_run(
        &candidate_dataset,
        &split,
        &build_offline_trainer_dry_run_spec(&candidate_dataset, &split),
    );
    let dry_run_v2 = run_offline_trainer_dry_run_v2(
        &candidate_dataset,
        &split,
        &OfflineTrainerDryRunConfigV2 {
            run_id: "sprint166-safety".to_string(),
            training_candidate_dataset_path: None,
            training_split_path: None,
            batch_size: 1,
            feature_schema_version: "smartcore-v2".to_string(),
            target_schema_version: "smartcore-v2".to_string(),
            validate_loss_contract: true,
            validate_metric_contract: true,
            paper_only: true,
        },
    );
    let label_quality_summary = summarize_label_quality(&build_validated_replay_dataset(
        &dataset,
        &[],
        soma_zero::league::minimal_ai_committee_core::ValidatedReplayDatasetBuildConfig {
            source_sanitized_dataset_id: dataset.dataset_id.clone(),
            label_validation_policy: PaperLabelValidationPolicy::default(),
            backtest_label_contract: None,
            require_validated_labels_for_training: true,
            allow_ready_with_warnings_for_medium_confidence: true,
            reject_rejected_labels: true,
            paper_only: true,
        },
    ));
    let mut unsafe_dataset = dataset.clone();
    unsafe_dataset.examples[0]
        .input_features
        .market_data_summary = "broker account leak after execution".to_string();
    let unsafe_quality_eval = run_replay_quality_evaluation_with_thresholds(
        &MemberExperienceStore::new("sprint166-unsafe-quality-store", Vec::new()),
        &unsafe_dataset,
        2,
        1,
    );
    let blocked_gate = evaluate_offline_training_design_gate(
        &candidate_dataset,
        &split,
        &dry_run_v1,
        &unsafe_quality_eval,
    );
    let blocked_design_status =
        evaluate_offline_trainer_design_status(&dry_run_v2, &blocked_gate, &label_quality_summary);
    let blocked_brief = build_offline_trainer_readiness_brief(
        &dry_run_v2,
        &blocked_design_status,
        &candidate_dataset,
        &split,
    );
    let safety_gate = evaluate_tiny_training_eligibility(
        &blocked_brief,
        &TinyTrainingEligibilityPolicy::default(),
    );
    assert!(!safety_gate.eligible_for_tiny_training_dry_run);
    assert_eq!(
        safety_gate.eligibility_status,
        TinyTrainingEligibilityStatus::BlockedBySafety
    );
    assert!(!safety_gate.no_leakage_detected);

    let quality_eval = run_replay_quality_evaluation_with_thresholds(
        &MemberExperienceStore::new("sprint166-weak-label-store", Vec::new()),
        &dataset,
        2,
        1,
    );
    let design_gate = evaluate_offline_training_design_gate(
        &candidate_dataset,
        &split,
        &dry_run_v1,
        &quality_eval,
    );
    let design_status =
        evaluate_offline_trainer_design_status(&dry_run_v2, &design_gate, &label_quality_summary);
    let mut weak_label_brief = build_offline_trainer_readiness_brief(
        &dry_run_v2,
        &design_status,
        &candidate_dataset,
        &split,
    );
    weak_label_brief.member_summaries[0]
        .label_source_distribution
        .insert("ReviewRequired".to_string(), 1);
    let weak_label_gate = evaluate_tiny_training_eligibility(
        &weak_label_brief,
        &TinyTrainingEligibilityPolicy::default(),
    );
    assert!(!weak_label_gate.weak_label_excluded);
    assert_eq!(
        weak_label_gate.eligibility_status,
        TinyTrainingEligibilityStatus::BlockedBySafety
    );
}

#[test]
fn sprint166_tiny_training_experiment_contract_stays_no_weight_update_only() {
    let dataset = sprint166_balanced_pilot_replay_dataset();
    let mask =
        build_replay_training_inclusion_mask(&dataset, &ReplayTrainingInclusionPolicy::default());
    let candidate_dataset =
        build_training_candidate_dataset(&dataset, &mask, &TrainingCandidateBuildConfig::default());
    let split = sprint166_balanced_training_split(&candidate_dataset);
    let dry_run_v1 = run_offline_trainer_data_loader_dry_run(
        &candidate_dataset,
        &split,
        &build_offline_trainer_dry_run_spec(&candidate_dataset, &split),
    );
    let quality_eval = run_replay_quality_evaluation_with_thresholds(
        &MemberExperienceStore::new("sprint166-contract-store", Vec::new()),
        &dataset,
        2,
        1,
    );
    let design_gate = evaluate_offline_training_design_gate(
        &candidate_dataset,
        &split,
        &dry_run_v1,
        &quality_eval,
    );
    let dry_run_v2 = run_offline_trainer_dry_run_v2(
        &candidate_dataset,
        &split,
        &OfflineTrainerDryRunConfigV2 {
            run_id: "sprint166-contract".to_string(),
            training_candidate_dataset_path: None,
            training_split_path: None,
            batch_size: 1,
            feature_schema_version: "smartcore-v2".to_string(),
            target_schema_version: "smartcore-v2".to_string(),
            validate_loss_contract: true,
            validate_metric_contract: true,
            paper_only: true,
        },
    );
    let label_quality_summary = summarize_label_quality(&build_validated_replay_dataset(
        &dataset,
        &[],
        soma_zero::league::minimal_ai_committee_core::ValidatedReplayDatasetBuildConfig {
            source_sanitized_dataset_id: dataset.dataset_id.clone(),
            label_validation_policy: PaperLabelValidationPolicy::default(),
            backtest_label_contract: None,
            require_validated_labels_for_training: true,
            allow_ready_with_warnings_for_medium_confidence: true,
            reject_rejected_labels: true,
            paper_only: true,
        },
    ));
    let trainer_design_status =
        evaluate_offline_trainer_design_status(&dry_run_v2, &design_gate, &label_quality_summary);
    let brief = build_offline_trainer_readiness_brief(
        &dry_run_v2,
        &trainer_design_status,
        &candidate_dataset,
        &split,
    );
    let gate =
        evaluate_tiny_training_eligibility(&brief, &TinyTrainingEligibilityPolicy::default());
    let contract = build_tiny_training_experiment_contract(&gate);
    assert_eq!(
        contract.allowed_next_step,
        TinyTrainingAllowedNextStep::NoWeightUpdateLossSimulation
    );
    assert_ne!(
        contract.allowed_next_step,
        TinyTrainingAllowedNextStep::TinyNoPersistenceTrainingSimulation
    );
    assert!(
        contract
            .forbidden_operations
            .contains(&TinyTrainingForbiddenOperation::OptimizerStep)
    );
    assert!(
        contract
            .forbidden_operations
            .contains(&TinyTrainingForbiddenOperation::GradientBackprop)
    );
    assert!(
        contract
            .forbidden_operations
            .contains(&TinyTrainingForbiddenOperation::CheckpointWrite)
    );
    assert!(
        contract
            .forbidden_operations
            .contains(&TinyTrainingForbiddenOperation::LiveInference)
    );
    assert!(
        contract
            .forbidden_operations
            .contains(&TinyTrainingForbiddenOperation::BrokerOrderAccount)
    );
}

#[test]
fn sprint167_config_policy_and_head_shape_validation_stay_safe() {
    assert!(
        TinyLossSimulationConfig {
            no_optimizer: false,
            ..TinyLossSimulationConfig::default()
        }
        .validate()
        .is_err()
    );
    assert!(
        TinyLossSimulationConfig {
            no_backprop: false,
            ..TinyLossSimulationConfig::default()
        }
        .validate()
        .is_err()
    );
    assert!(
        TinyLossSimulationConfig {
            no_checkpoint: false,
            ..TinyLossSimulationConfig::default()
        }
        .validate()
        .is_err()
    );
    assert!(
        TinyLossSimulationConfig {
            require_validated_labels: false,
            ..TinyLossSimulationConfig::default()
        }
        .validate()
        .is_err()
    );
    assert!(
        TinyLossSimulationConfig {
            include_train_split: false,
            include_validation_split: false,
            include_test_split: false,
            ..TinyLossSimulationConfig::default()
        }
        .validate()
        .is_err()
    );
    assert!(!DummyPredictionPolicy::default().allow_target_echo);
    assert!(
        DummyPredictionPolicy {
            allow_target_echo: true,
            ..DummyPredictionPolicy::default()
        }
        .validate()
        .is_err()
    );

    let feature_schema = TrainingFeatureSchema::default();
    let valid_target_schema = TrainingTargetSchema {
        target_heads: vec![
            TrainingTargetHead::Stance,
            TrainingTargetHead::ConfidenceCalibration,
            TrainingTargetHead::Risk,
            TrainingTargetHead::EvidenceNeed,
        ],
        ..TrainingTargetSchema::default()
    };
    let valid_shape = validate_smartcore_v2_head_shapes(
        &build_smartcore_v2_batch_spec(2, &feature_schema, &valid_target_schema),
        &valid_target_schema,
        &valid_target_schema.target_heads,
    );
    assert_eq!(valid_shape.shape_status, HeadShapeStatus::Valid);

    let missing_target_schema = TrainingTargetSchema {
        target_heads: vec![
            TrainingTargetHead::ConfidenceCalibration,
            TrainingTargetHead::Risk,
            TrainingTargetHead::EvidenceNeed,
        ],
        ..TrainingTargetSchema::default()
    };
    let invalid_shape = validate_smartcore_v2_head_shapes(
        &build_smartcore_v2_batch_spec(2, &feature_schema, &missing_target_schema),
        &missing_target_schema,
        &[TrainingTargetHead::Stance],
    );
    assert_eq!(invalid_shape.shape_status, HeadShapeStatus::Invalid);
    assert!(
        invalid_shape
            .invalid_heads
            .iter()
            .any(|head| head == "Stance")
    );

    let deferred_shape = validate_smartcore_v2_head_shapes(
        &build_smartcore_v2_batch_spec(2, &feature_schema, &valid_target_schema),
        &valid_target_schema,
        &[TrainingTargetHead::ExpectedReturnHint],
    );
    assert_eq!(
        deferred_shape.shape_status,
        HeadShapeStatus::ValidWithWarnings
    );
    assert!(
        deferred_shape
            .missing_heads
            .iter()
            .any(|head| head == "ExpectedReturnHint")
    );
}

#[test]
fn sprint167_dummy_loss_simulation_and_guard_proof_are_deterministic() {
    let dataset = sprint166_balanced_pilot_replay_dataset();
    let mask =
        build_replay_training_inclusion_mask(&dataset, &ReplayTrainingInclusionPolicy::default());
    let candidate_dataset =
        build_training_candidate_dataset(&dataset, &mask, &TrainingCandidateBuildConfig::default());
    let split = sprint166_balanced_training_split(&candidate_dataset);
    let batches = build_training_batches(
        &candidate_dataset,
        &split,
        &TrainingFeatureSchema::default(),
        &TrainingTargetSchema {
            target_heads: vec![
                TrainingTargetHead::Stance,
                TrainingTargetHead::ConfidenceCalibration,
                TrainingTargetHead::Risk,
                TrainingTargetHead::EvidenceNeed,
            ],
            ..TrainingTargetSchema::default()
        },
        &TrainingBatchIteratorConfig {
            batch_size: 1,
            shuffle: false,
            split: TrainingBatchSplit::All,
            member_id: None,
            drop_last: false,
            paper_only: true,
        },
    )
    .expect("sprint167 all batches");
    let config = TinyLossSimulationConfig {
        simulation_id: "sprint167-sim".to_string(),
        batch_size: 1,
        enabled_heads: vec![
            TrainingTargetHead::Stance,
            TrainingTargetHead::ConfidenceCalibration,
            TrainingTargetHead::Risk,
            TrainingTargetHead::EvidenceNeed,
        ],
        ..TinyLossSimulationConfig::default()
    };
    let first =
        simulate_tiny_label_losses(&batches.batches, &config, &DummyPredictionPolicy::default());
    let second =
        simulate_tiny_label_losses(&batches.batches, &config, &DummyPredictionPolicy::default());
    assert_eq!(first, second);
    assert!(first.batch_count >= 1);
    assert!(first.example_count >= 9);
    assert!(matches!(
        first.simulation_status,
        OfflineTrainerDryRunStatus::Passed | OfflineTrainerDryRunStatus::PassedWithWarnings
    ));
    let member_summaries = summarize_tiny_loss_by_member(&first);
    assert_eq!(
        member_summaries
            .iter()
            .map(|summary| summary.member_id.as_str())
            .take(3)
            .collect::<Vec<_>>(),
        vec!["TrendEntryAI", "RiskGuardAI", "EvidenceRegimeAI"]
    );
    let guard = prove_no_weight_update_path(&config, &first);
    assert!(!guard.optimizer_present);
    assert!(!guard.gradient_computation_present);
    assert!(!guard.checkpoint_write_present);
    assert_eq!(
        guard.proof_status,
        NoWeightUpdateGuardProofStatus::Preserved
    );
    let violated_guard = prove_no_weight_update_path(
        &TinyLossSimulationConfig {
            no_optimizer: false,
            ..config.clone()
        },
        &first,
    );
    assert_eq!(
        violated_guard.proof_status,
        NoWeightUpdateGuardProofStatus::Violated
    );

    let train_batches = build_training_batches(
        &candidate_dataset,
        &split,
        &TrainingFeatureSchema::default(),
        &TrainingTargetSchema {
            target_heads: config.enabled_heads.clone(),
            ..TrainingTargetSchema::default()
        },
        &TrainingBatchIteratorConfig {
            batch_size: 1,
            shuffle: false,
            split: TrainingBatchSplit::Train,
            member_id: None,
            drop_last: false,
            paper_only: true,
        },
    )
    .expect("sprint167 train batches");
    let validation_batches = build_training_batches(
        &candidate_dataset,
        &split,
        &TrainingFeatureSchema::default(),
        &TrainingTargetSchema {
            target_heads: config.enabled_heads.clone(),
            ..TrainingTargetSchema::default()
        },
        &TrainingBatchIteratorConfig {
            batch_size: 1,
            shuffle: false,
            split: TrainingBatchSplit::Validation,
            member_id: None,
            drop_last: false,
            paper_only: true,
        },
    )
    .expect("sprint167 validation batches");
    let mut split_batches = train_batches.batches.clone();
    split_batches.extend(validation_batches.batches.clone());
    let train_only = simulate_tiny_label_losses(
        &split_batches,
        &TinyLossSimulationConfig {
            include_validation_split: false,
            include_test_split: false,
            ..config.clone()
        },
        &DummyPredictionPolicy::default(),
    );
    assert!(train_only.split_counts.contains_key("Train"));
    assert!(!train_only.split_counts.contains_key("Validation"));
}

#[test]
fn sprint167_tiny_no_weight_dry_run_stays_safe_and_is_deterministic() {
    let dataset = sprint166_balanced_pilot_replay_dataset();
    let mask =
        build_replay_training_inclusion_mask(&dataset, &ReplayTrainingInclusionPolicy::default());
    let candidate_dataset =
        build_training_candidate_dataset(&dataset, &mask, &TrainingCandidateBuildConfig::default());
    let split = sprint166_balanced_training_split(&candidate_dataset);
    let config = TinyNoWeightTrainingDryRunConfig {
        run_id: "sprint167-dry-run".to_string(),
        training_candidate_dataset_path: None,
        training_split_path: None,
        batch_size: 1,
        prediction_policy: DummyPredictionPolicy::default(),
        enabled_heads: vec![
            TrainingTargetHead::Stance,
            TrainingTargetHead::ConfidenceCalibration,
            TrainingTargetHead::Risk,
            TrainingTargetHead::EvidenceNeed,
            TrainingTargetHead::ExpectedReturnHint,
        ],
        write_output_path: None,
        dry_run: true,
        paper_only: true,
    };
    let first = run_tiny_no_weight_training_dry_run(&candidate_dataset, &split, &config);
    let second = run_tiny_no_weight_training_dry_run(&candidate_dataset, &split, &config);
    assert_eq!(first, second);
    assert!(matches!(
        first.dry_run_status,
        OfflineTrainerDryRunStatus::Passed | OfflineTrainerDryRunStatus::PassedWithWarnings
    ));
    assert!(!first.no_weight_update_guard_proof.optimizer_present);
    assert!(
        !first
            .no_weight_update_guard_proof
            .gradient_computation_present
    );
    assert!(!first.no_weight_update_guard_proof.backprop_present);
    assert!(!first.no_weight_update_guard_proof.weight_mutation_present);
    assert!(!first.no_weight_update_guard_proof.checkpoint_write_present);
    assert!(!first.no_weight_update_guard_proof.model_runtime_present);
    assert!(!first.no_weight_update_guard_proof.live_inference_present);
    assert!(
        !first
            .no_weight_update_guard_proof
            .broker_order_account_present
    );
    assert_eq!(
            first.next_allowed_step,
            soma_zero::league::minimal_ai_committee_core::TinyNoWeightTrainingNextStep::KeepNoWeightSimulationOnly
    );

    let mut aliased_candidate_dataset = candidate_dataset.clone();
    for example in &mut aliased_candidate_dataset.examples {
        example.member_id = match example.member_id.as_str() {
            "TrendEntryAI" => "trend-kr-short".to_string(),
            "RiskGuardAI" => "risk-kr-short".to_string(),
            "EvidenceRegimeAI" => "evidence-kr-short".to_string(),
            member_id => member_id.to_string(),
        };
    }
    let aliased = run_tiny_no_weight_training_dry_run(&aliased_candidate_dataset, &split, &config);
    for member_id in ["TrendEntryAI", "RiskGuardAI", "EvidenceRegimeAI"] {
        let summary = aliased
            .member_loss_summaries
            .iter()
            .find(|summary| summary.member_id == member_id)
            .expect("canonical aliased member summary");
        assert!(summary.example_count > 0);
        assert_ne!(
            summary.summary_status,
            soma_zero::league::minimal_ai_committee_core::MemberTinyLossSummaryStatus::InsufficientData
        );
    }
    assert!(
        !aliased
            .member_loss_summaries
            .iter()
            .any(|summary| summary.member_id == "trend-kr-short")
    );

    let leaky_policy = run_tiny_no_weight_training_dry_run(
        &candidate_dataset,
        &split,
        &TinyNoWeightTrainingDryRunConfig {
            prediction_policy: DummyPredictionPolicy {
                allow_target_echo: true,
                ..DummyPredictionPolicy::default()
            },
            ..config
        },
    );
    assert_eq!(
        leaky_policy.dry_run_status,
        OfflineTrainerDryRunStatus::Failed
    );
    assert_eq!(
        leaky_policy.next_allowed_step,
        soma_zero::league::minimal_ai_committee_core::TinyNoWeightTrainingNextStep::FixBlockingIssues
    );
    assert!(
        leaky_policy
            .loss_simulation_result
            .warnings
            .iter()
            .any(|warning| warning.contains("target echo"))
    );
}

#[test]
fn sprint167_no_persistence_gate_and_dependency_guard_stay_safe() {
    let dataset = sprint166_balanced_pilot_replay_dataset();
    let mask =
        build_replay_training_inclusion_mask(&dataset, &ReplayTrainingInclusionPolicy::default());
    let candidate_dataset =
        build_training_candidate_dataset(&dataset, &mask, &TrainingCandidateBuildConfig::default());
    let split = sprint166_balanced_training_split(&candidate_dataset);
    let mut tiny_dry_run = run_tiny_no_weight_training_dry_run(
        &candidate_dataset,
        &split,
        &TinyNoWeightTrainingDryRunConfig {
            run_id: "sprint167-gate".to_string(),
            training_candidate_dataset_path: None,
            training_split_path: None,
            batch_size: 1,
            prediction_policy: DummyPredictionPolicy::default(),
            enabled_heads: vec![
                TrainingTargetHead::Stance,
                TrainingTargetHead::ConfidenceCalibration,
                TrainingTargetHead::Risk,
                TrainingTargetHead::EvidenceNeed,
                TrainingTargetHead::ExpectedReturnHint,
            ],
            write_output_path: None,
            dry_run: true,
            paper_only: true,
        },
    );
    tiny_dry_run.eligibility_gate = Some(
        soma_zero::league::minimal_ai_committee_core::TinyTrainingEligibilityGate {
            gate_id: "eligible-gate".to_string(),
            trainer_design_status: OfflineTrainerDesignStatusLevel::ReadyWithWarnings,
            dry_run_status: OfflineTrainerDryRunStatus::PassedWithWarnings,
            blocking_warning_count: 0,
            expected_deferred_warning_count: 1,
            min_training_examples_required: 8,
            actual_training_examples: tiny_dry_run.loss_simulation_result.example_count,
            min_members_required: 3,
            actual_members_with_examples: 3,
            weak_label_excluded: true,
            no_leakage_detected: true,
            no_broker_order_account: true,
            no_live_inference: true,
            no_checkpoint_written: true,
            eligible_for_tiny_training_dry_run: true,
            eligibility_status: TinyTrainingEligibilityStatus::EligibleWithWarnings,
            blockers: Vec::new(),
            warnings: vec!["expected deferred warnings remain".to_string()],
            paper_only: true,
        },
    );
    let gate = evaluate_no_persistence_training_simulation_gate(&tiny_dry_run);
    assert_eq!(
        gate.gate_status,
        NoPersistenceTrainingSimulationGateStatus::AllowedWithWarnings
    );
    assert!(gate.allow_next_no_persistence_simulation);

    let mut blocked = tiny_dry_run.clone();
    blocked.no_weight_update_guard_proof.proof_status = NoWeightUpdateGuardProofStatus::Violated;
    let blocked_gate = evaluate_no_persistence_training_simulation_gate(&blocked);
    assert_eq!(
        blocked_gate.gate_status,
        NoPersistenceTrainingSimulationGateStatus::BlockedBySafety
    );

    let mut failed_dry_run = tiny_dry_run.clone();
    failed_dry_run.dry_run_status = OfflineTrainerDryRunStatus::Failed;
    let failed_gate = evaluate_no_persistence_training_simulation_gate(&failed_dry_run);
    assert_eq!(
        failed_gate.gate_status,
        NoPersistenceTrainingSimulationGateStatus::BlockedBySafety
    );

    let cargo_toml = std::fs::read_to_string("Cargo.toml").expect("read Cargo.toml");
    let lowered = cargo_toml.to_ascii_lowercase();
    assert!(!lowered.contains("python"));
    assert!(!lowered.contains("pytorch"));
    assert!(!lowered.contains("tensorflow"));
}

fn sprint168_training_inputs(
    batch_size: usize,
) -> (
    TrainingCandidateDataset,
    TrainingSplitResult,
    Vec<SmartCoreV2TrainingBatch>,
) {
    let dataset = sprint166_balanced_pilot_replay_dataset();
    let mask =
        build_replay_training_inclusion_mask(&dataset, &ReplayTrainingInclusionPolicy::default());
    let candidate_dataset =
        build_training_candidate_dataset(&dataset, &mask, &TrainingCandidateBuildConfig::default());
    let split = sprint166_balanced_training_split(&candidate_dataset);
    let target_schema = TrainingTargetSchema {
        target_heads: vec![
            TrainingTargetHead::Stance,
            TrainingTargetHead::ConfidenceCalibration,
            TrainingTargetHead::Risk,
            TrainingTargetHead::EvidenceNeed,
        ],
        ..TrainingTargetSchema::default()
    };
    let build_for_split = |split_kind| {
        build_training_batches(
            &candidate_dataset,
            &split,
            &TrainingFeatureSchema::default(),
            &target_schema,
            &TrainingBatchIteratorConfig {
                batch_size,
                shuffle: false,
                split: split_kind,
                member_id: None,
                drop_last: false,
                paper_only: true,
            },
        )
        .expect("sprint168 batches")
        .batches
    };
    let mut batches = Vec::new();
    batches.extend(build_for_split(TrainingBatchSplit::Train));
    batches.extend(build_for_split(TrainingBatchSplit::Validation));
    batches.extend(build_for_split(TrainingBatchSplit::Test));
    (candidate_dataset, split, batches)
}

#[test]
fn sprint168_config_and_shadow_plan_forbid_training_ops() {
    assert!(
        NoPersistenceTrainingSimulationConfig {
            allow_optimizer: true,
            ..NoPersistenceTrainingSimulationConfig::default()
        }
        .validate()
        .is_err()
    );
    assert!(
        NoPersistenceTrainingSimulationConfig {
            allow_backprop: true,
            ..NoPersistenceTrainingSimulationConfig::default()
        }
        .validate()
        .is_err()
    );
    assert!(
        NoPersistenceTrainingSimulationConfig {
            allow_checkpoint_write: true,
            ..NoPersistenceTrainingSimulationConfig::default()
        }
        .validate()
        .is_err()
    );
    let target_schema = TrainingTargetSchema {
        target_heads: vec![
            TrainingTargetHead::Stance,
            TrainingTargetHead::ConfidenceCalibration,
            TrainingTargetHead::Risk,
            TrainingTargetHead::EvidenceNeed,
        ],
        ..TrainingTargetSchema::default()
    };
    let plan = build_shadow_training_step_plan(
        &NoPersistenceTrainingSimulationConfig {
            simulation_id: "sprint168-plan".to_string(),
            batch_size: 2,
            enabled_heads: target_schema.target_heads.clone(),
            ..NoPersistenceTrainingSimulationConfig::default()
        },
        &build_smartcore_v2_batch_spec(2, &TrainingFeatureSchema::default(), &target_schema),
    );
    assert!(plan.plan_valid);
    assert!(
        plan.operations
            .contains(&ShadowTrainingOperation::SkipOptimizerStep)
    );
    assert!(
        plan.operations
            .contains(&ShadowTrainingOperation::SkipBackprop)
    );
    assert!(
        plan.operations
            .contains(&ShadowTrainingOperation::SkipCheckpoint)
    );
    assert!(
        plan.operations
            .contains(&ShadowTrainingOperation::SkipWeightMutation)
    );
}

#[test]
fn sprint168_simulation_produces_deterministic_step_and_epoch_records() {
    let (candidate_dataset, split, batches) = sprint168_training_inputs(1);
    let config = NoPersistenceTrainingSimulationConfig {
        simulation_id: "sprint168-sim".to_string(),
        dataset_id: Some(candidate_dataset.dataset_id.clone()),
        split_id: Some(split.dataset_id.clone()),
        batch_size: 1,
        max_epochs: 2,
        max_steps: 3,
        enabled_heads: vec![
            TrainingTargetHead::Stance,
            TrainingTargetHead::ConfidenceCalibration,
            TrainingTargetHead::Risk,
            TrainingTargetHead::EvidenceNeed,
        ],
        ..NoPersistenceTrainingSimulationConfig::default()
    };
    let first =
        run_no_persistence_training_simulation(&candidate_dataset, &split, &batches, &config);
    let second =
        run_no_persistence_training_simulation(&candidate_dataset, &split, &batches, &config);
    assert_eq!(first, second);
    assert_eq!(first.epoch_results.len(), 2);
    assert!(!first.step_records.is_empty());
    assert_eq!(first.epoch_results[0].step_count, 3);
    assert!(first.epoch_results[0].aggregate_dummy_loss >= 0.0);
    let candidate_ids: std::collections::BTreeSet<String> = candidate_dataset
        .examples
        .iter()
        .map(|example| example.training_example_id.clone())
        .collect();
    assert!(first.step_records.iter().all(|step| {
        !step.example_ids.is_empty()
            && step
                .example_ids
                .iter()
                .all(|example_id| candidate_ids.contains(example_id))
    }));
    assert!(
        first
            .step_records
            .iter()
            .all(|step| !step.optimizer_step_executed
                && !step.backprop_executed
                && !step.weight_mutation_executed
                && !step.checkpoint_written)
    );
    assert!(first.no_training_executed);
    assert!(first.no_weight_mutation);
    assert!(first.no_checkpoint);
    assert!(matches!(
        first.simulation_status,
        NoPersistenceTrainingSimulationStatus::Passed
            | NoPersistenceTrainingSimulationStatus::PassedWithWarnings
    ));
    assert!(first.no_model_runtime);
    assert!(first.no_live_inference);
    assert!(first.no_broker_order_account);
}

#[test]
fn sprint168_safety_guard_detects_forbidden_and_weak_labels() {
    let (candidate_dataset, split, batches) = sprint168_training_inputs(1);
    let config = NoPersistenceTrainingSimulationConfig {
        simulation_id: "sprint168-safety".to_string(),
        dataset_id: Some(candidate_dataset.dataset_id.clone()),
        split_id: Some(split.dataset_id.clone()),
        batch_size: 1,
        max_epochs: 1,
        max_steps: 3,
        enabled_heads: vec![
            TrainingTargetHead::Stance,
            TrainingTargetHead::ConfidenceCalibration,
            TrainingTargetHead::Risk,
            TrainingTargetHead::EvidenceNeed,
        ],
        ..NoPersistenceTrainingSimulationConfig::default()
    };
    let simulation =
        run_no_persistence_training_simulation(&candidate_dataset, &split, &batches, &config);
    let guard =
        evaluate_training_simulation_safety(&simulation.step_records, &candidate_dataset, &config);
    assert!(matches!(
        guard.safety_status,
        TrainingSimulationSafetyStatus::Preserved
            | TrainingSimulationSafetyStatus::PreservedWithWarnings
    ));
    assert!(!guard.live_inference_detected);
    assert!(!guard.broker_order_account_detected);
    let mut injected_steps = simulation.step_records.clone();
    injected_steps[0].optimizer_step_executed = true;
    let injected_guard =
        evaluate_training_simulation_safety(&injected_steps, &candidate_dataset, &config);
    assert_eq!(
        injected_guard.safety_status,
        TrainingSimulationSafetyStatus::Violated
    );
    let mut non_candidate_steps = simulation.step_records.clone();
    non_candidate_steps[0]
        .example_ids
        .push("not-a-training-candidate".to_string());
    let non_candidate_guard =
        evaluate_training_simulation_safety(&non_candidate_steps, &candidate_dataset, &config);
    assert!(non_candidate_guard.non_training_candidate_example_detected);
    assert_eq!(
        non_candidate_guard.safety_status,
        TrainingSimulationSafetyStatus::Violated
    );
    let mut weak_dataset = candidate_dataset.clone();
    weak_dataset.examples[0].label_confidence = ReplayLabelConfidence::Low;
    let weak_guard =
        evaluate_training_simulation_safety(&simulation.step_records, &weak_dataset, &config);
    assert!(weak_guard.weak_label_included_detected);
    assert_eq!(
        weak_guard.safety_status,
        TrainingSimulationSafetyStatus::Violated
    );
}

#[test]
fn sprint168_adapter_gate_and_brief_keep_runtime_deferred() {
    let (candidate_dataset, split, batches) = sprint168_training_inputs(1);
    let simulation = run_no_persistence_training_simulation(
        &candidate_dataset,
        &split,
        &batches,
        &NoPersistenceTrainingSimulationConfig {
            simulation_id: "sprint168-adapter".to_string(),
            dataset_id: Some(candidate_dataset.dataset_id.clone()),
            split_id: Some(split.dataset_id.clone()),
            batch_size: 1,
            max_epochs: 1,
            max_steps: 3,
            enabled_heads: vec![
                TrainingTargetHead::Stance,
                TrainingTargetHead::ConfidenceCalibration,
                TrainingTargetHead::Risk,
                TrainingTargetHead::EvidenceNeed,
            ],
            ..NoPersistenceTrainingSimulationConfig::default()
        },
    );
    let gate = evaluate_smartcore_adapter_skeleton_readiness(&simulation, None);
    assert!(gate.allow_adapter_skeleton_next);
    let brief = build_no_persistence_training_simulation_brief(&simulation, &gate);
    assert!(
        brief
            .human_readable_summary
            .contains("Mamba3/Gated runtime still deferred")
    );
    let mut blocked_simulation = simulation.clone();
    blocked_simulation.safety_guard.safety_status = TrainingSimulationSafetyStatus::Violated;
    let blocked_gate = evaluate_smartcore_adapter_skeleton_readiness(&blocked_simulation, None);
    assert!(!blocked_gate.allow_adapter_skeleton_next);
    let cargo_toml = std::fs::read_to_string("Cargo.toml").expect("read Cargo.toml");
    let lowered = cargo_toml.to_ascii_lowercase();
    assert!(!lowered.contains("python"));
    assert!(!lowered.contains("pytorch"));
    assert!(!lowered.contains("tensorflow"));
}

#[test]
fn sprint169_member_profiles_and_registry_remain_shape_only() {
    let members = create_three_member_pilot_roster(MarketScope::UsShortTerm);
    let specs = build_smart_core_v2_specs_for_members(&members);
    assert_eq!(specs.len(), members.len());

    let trend_member = members
        .iter()
        .find(|member| member.role == Some(IndependentMemberRole::TrendEntry))
        .expect("trend member");
    let trend_spec = specs
        .iter()
        .find(|spec| spec.member_id == trend_member.member_id)
        .expect("trend spec");
    let trend_profile = build_member_smartcore_adapter_profile(trend_member, trend_spec, false);
    assert!(matches!(
        trend_profile.adapter_status,
        SmartCoreV2AdapterStatus::ShapeOnly
    ));
    assert!(matches!(
        trend_profile.mamba3_spec.runtime_status,
        SmartCoreV2AdapterStatus::RuntimeDeferred
    ));
    assert!(matches!(
        trend_profile.gated_deltanet_spec.training_status,
        SmartCoreV2AdapterStatus::TrainingDeferred
    ));
    assert_eq!(
        validate_mamba3_temporal_adapter_spec(&trend_profile.mamba3_spec).validation_status,
        AdapterShapeValidationStatus::Valid
    );
    assert_eq!(
        validate_gated_deltanet_memory_adapter_spec(&trend_profile.gated_deltanet_spec)
            .validation_status,
        AdapterShapeValidationStatus::Valid
    );
    assert_eq!(trend_profile.sparse_event_attention_spec, None);
    assert_eq!(
        trend_profile.head_specs[0].head_kind,
        SmartCoreV2HeadKind::Stance
    );
    assert_eq!(
        trend_profile.head_specs[2].head_kind,
        SmartCoreV2HeadKind::ExpectedReturnHint
    );
    assert!(
        default_head_specs_for_member(&trend_member.member_id)
            .iter()
            .all(|spec| validate_head_adapter_spec(spec).validation_status
                == AdapterShapeValidationStatus::Valid)
    );

    let registry = build_adapter_registry_for_members(&members, true);
    assert_eq!(registry.profile_count, members.len());
    assert_eq!(registry.runtime_ready_count, 0);
    assert_eq!(registry.training_ready_count, 0);
    assert_eq!(registry.shape_only_count, members.len());
    assert!(registry.validate_registry());

    let risk_profile = registry
        .profiles
        .iter()
        .find(|profile| profile.role == IndependentMemberRole::RiskGuard)
        .expect("risk profile");
    let evidence_profile = registry
        .profiles
        .iter()
        .find(|profile| profile.role == IndependentMemberRole::EvidenceRegime)
        .expect("evidence profile");
    assert_ne!(
        trend_profile.head_specs[0].head_kind,
        risk_profile.head_specs[0].head_kind
    );
    assert_eq!(
        risk_profile.head_specs[1].head_kind,
        SmartCoreV2HeadKind::Uncertainty
    );
    assert_ne!(
        risk_profile.head_specs[0].head_kind,
        evidence_profile.head_specs[0].head_kind
    );
    assert_eq!(
        evidence_profile.head_specs[1].head_kind,
        SmartCoreV2HeadKind::ConfidenceCalibration
    );
    assert!(
        registry
            .profiles
            .iter()
            .all(|profile| profile.sparse_event_attention_spec.is_some())
    );
    assert!(registry.profiles.iter().all(|profile| {
        validate_sparse_event_attention_adapter_spec(
            profile
                .sparse_event_attention_spec
                .as_ref()
                .expect("sparse event spec"),
        )
        .validation_status
            == AdapterShapeValidationStatus::Valid
    }));
    assert!(registry.profiles.iter().all(|profile| {
        !profile.profile_id.to_ascii_lowercase().contains("moe")
            && !profile.profile_id.to_ascii_lowercase().contains("central")
    }));

    let active_ids = members
        .iter()
        .map(|member| member.member_id.clone())
        .collect::<Vec<_>>();
    assert_eq!(
        registry
            .adapter_profiles_for_active_members(&active_ids)
            .len(),
        members.len()
    );
}

#[test]
fn sprint169_batch_validation_and_output_guard_stay_shape_only() {
    let members = create_three_member_pilot_roster(MarketScope::UsShortTerm);
    let (_candidate_dataset, _split, batches) = sprint168_training_inputs(1);
    let feature_schema = TrainingFeatureSchema::default();
    let target_schema = TrainingTargetSchema {
        target_heads: vec![
            TrainingTargetHead::Stance,
            TrainingTargetHead::ConfidenceCalibration,
            TrainingTargetHead::Risk,
            TrainingTargetHead::EvidenceNeed,
        ],
        ..TrainingTargetSchema::default()
    };
    let registry = build_adapter_registry_for_members(&members, true);
    let first_batch = batches.first().expect("batch");
    let input_shape = build_adapter_input_shape_from_training_batch(first_batch, &feature_schema);
    assert_eq!(input_shape.example_count, first_batch.batch_size);
    let expected_output_shape = build_expected_adapter_output_shape(
        first_batch,
        &target_schema,
        &target_schema.target_heads,
    );
    assert!(expected_output_shape.stance_output_dim > 0);
    let profile = registry
        .get_profile(&first_batch.member_id)
        .expect("matching profile");
    let validation = validate_batch_against_adapter_profile(
        first_batch,
        &feature_schema,
        &target_schema,
        profile,
    );
    assert!(matches!(
        validation.validation_status,
        AdapterShapeValidationStatus::Valid | AdapterShapeValidationStatus::ValidWithWarnings
    ));
    assert!(matches!(
        validation.output_value_guard.guard_status,
        AdapterOutputValueGuardStatus::Preserved
    ));
    assert!(validation.expected_output_shape.stance_output_dim > 0);
    assert!(validation.expected_output_shape.confidence_output_dim > 0);
    assert!(validation.expected_output_shape.risk_output_dim > 0);
    assert!(validation.expected_output_shape.evidence_output_dim > 0);
    assert_eq!(
        validation.expected_output_shape.uncertainty_output_dim,
        None
    );
    assert_eq!(
        validation.expected_output_shape.expected_return_output_dim,
        None
    );
    assert_eq!(
        validation.adapter_profile_status,
        SmartCoreV2AdapterStatus::ShapeOnly
    );
    assert!(matches!(
        validation.member_id.as_str(),
        "TrendEntryAI" | "RiskGuardAI" | "EvidenceRegimeAI"
    ));
    assert!(!validation.output_value_guard.logits_present);
    assert!(!validation.output_value_guard.probabilities_present);
    assert!(!validation.output_value_guard.predictions_present);
    assert!(!validation.output_value_guard.model_scores_present);
    assert_eq!(
        validate_no_adapter_output_values("logits=[0.1,0.2]").guard_status,
        AdapterOutputValueGuardStatus::Violated
    );
}

#[test]
fn sprint169_adapter_dry_run_is_deterministic_and_guarded() {
    let members = create_three_member_pilot_roster(MarketScope::UsShortTerm);
    let (_candidate_dataset, _split, batches) = sprint168_training_inputs(1);
    let feature_schema = TrainingFeatureSchema::default();
    let target_schema = TrainingTargetSchema {
        target_heads: vec![
            TrainingTargetHead::Stance,
            TrainingTargetHead::ConfidenceCalibration,
            TrainingTargetHead::Risk,
            TrainingTargetHead::EvidenceNeed,
        ],
        ..TrainingTargetSchema::default()
    };
    let config = AdapterSkeletonDryRunConfig {
        run_id: "sprint169-adapter-dry-run".to_string(),
        include_mamba3: true,
        include_gated_deltanet: true,
        include_sparse_event_attention: true,
        validate_batches: true,
        require_runtime_deferred: true,
        require_training_deferred: true,
        paper_only: true,
    };
    let first =
        run_adapter_skeleton_dry_run(&members, &batches, &feature_schema, &target_schema, &config);
    let second =
        run_adapter_skeleton_dry_run(&members, &batches, &feature_schema, &target_schema, &config);
    assert_eq!(first, second);
    assert!(matches!(
        first.dry_run_status,
        AdapterSkeletonDryRunStatus::Passed | AdapterSkeletonDryRunStatus::PassedWithWarnings
    ));
    assert_eq!(first.profile_count, members.len());
    assert_eq!(first.valid_profile_count, members.len());
    assert_eq!(first.invalid_profile_count, 0);
    assert_eq!(first.runtime_ready_count, 0);
    assert_eq!(first.training_ready_count, 0);
    assert!(first.all_runtime_deferred);
    assert!(first.all_training_deferred);
    assert!(first.no_forward_method_present);
    assert!(first.no_weight_access_present);
    assert!(first.no_checkpoint_present);

    let safety = evaluate_adapter_skeleton_safety(&first);
    assert_eq!(
        safety.safety_status,
        TrainingSimulationSafetyStatus::Preserved
    );
    assert!(!safety.live_inference_detected);
    assert!(!safety.broker_order_account_detected);

    let mut forward_injected = first.clone();
    forward_injected.no_forward_method_present = false;
    assert_eq!(
        evaluate_adapter_skeleton_safety(&forward_injected).safety_status,
        TrainingSimulationSafetyStatus::Violated
    );

    let mut weight_injected = first.clone();
    weight_injected.no_weight_access_present = false;
    assert_eq!(
        evaluate_adapter_skeleton_safety(&weight_injected).safety_status,
        TrainingSimulationSafetyStatus::Violated
    );

    let mut training_ready_injected = first.clone();
    training_ready_injected.training_ready_count = 1;
    assert_eq!(
        evaluate_adapter_skeleton_safety(&training_ready_injected).safety_status,
        TrainingSimulationSafetyStatus::Violated
    );

    let mut checkpoint_injected = first.clone();
    checkpoint_injected.no_checkpoint_present = false;
    assert_eq!(
        evaluate_adapter_skeleton_safety(&checkpoint_injected).safety_status,
        TrainingSimulationSafetyStatus::Violated
    );

    let mut unmatched_batches = batches.clone();
    unmatched_batches[0].member_id = "unmatched-member".to_string();
    let unmatched = run_adapter_skeleton_dry_run(
        &members,
        &unmatched_batches,
        &feature_schema,
        &target_schema,
        &config,
    );
    assert_eq!(
        unmatched.dry_run_status,
        AdapterSkeletonDryRunStatus::Failed
    );
    assert!(
        unmatched
            .warnings
            .iter()
            .any(|warning| warning.contains("no matching member profile"))
    );

    let cargo_toml = std::fs::read_to_string("Cargo.toml").expect("read Cargo.toml");
    let lowered = cargo_toml.to_ascii_lowercase();
    assert!(!lowered.contains("python"));
    assert!(!lowered.contains("pytorch"));
    assert!(!lowered.contains("tensorflow"));
}

fn sprint170_contract_inputs() -> (
    Vec<AICommitteeMember>,
    TrainingCandidateDataset,
    TrainingSplitResult,
    Vec<SmartCoreV2TrainingBatch>,
    soma_zero::league::minimal_ai_committee_core::SmartCoreAdapterRegistry,
    soma_zero::league::minimal_ai_committee_core::AdapterInputSchemaV1,
    soma_zero::league::minimal_ai_committee_core::AdapterOutputSchemaV1,
    Vec<soma_zero::league::minimal_ai_committee_core::MemberLearningDataContractV1>,
) {
    let members = create_three_member_pilot_roster(MarketScope::UsShortTerm);
    let (dataset, split, batches) = sprint168_training_inputs(1);
    let registry = build_adapter_registry_for_members(&members, true);
    let input_schema = default_adapter_input_schema_v1();
    let output_schema = default_adapter_output_schema_v1();
    let contracts =
        build_member_learning_data_contracts_for_registry(&registry, &input_schema, &output_schema);
    (
        members,
        dataset,
        split,
        batches,
        registry,
        input_schema,
        output_schema,
        contracts,
    )
}

fn sprint171_temp_json_path(stem: &str) -> PathBuf {
    static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    let counter = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("soma-{stem}-{unique}-{counter}.json"))
}

fn sprint175_microkernel_dry_run_fixture() -> (
    usize,
    soma_zero::league::minimal_ai_committee_core::SmartCoreMicroKernelDryRunResultV0,
) {
    let (_members, dataset, _split, _batches, registry, ..) = sprint170_contract_inputs();
    let dry_run = run_smartcore_microkernel_dry_run_v0(
        &dataset,
        &registry,
        &SmartCoreMicroKernelDryRunConfigV0 {
            run_id: "sprint175-microkernel".to_string(),
            member_ids: registry
                .profiles
                .iter()
                .map(|profile| profile.member_id.clone())
                .collect(),
            batch_size: 8,
            sequence_len: 3,
            input_dim: 8,
            temporal_state_dim: 8,
            memory_dim: 8,
            output_dim: 6,
            use_training_candidate_dataset: true,
            synthetic_input_fallback: true,
            paper_only: true,
        },
    )
    .expect("sprint175 microkernel dry-run");
    (registry.profile_count, dry_run)
}

fn sprint176_shadow_alignment_fixture() -> (
    BatchCommitteeCycleResult,
    soma_zero::league::minimal_ai_committee_core::SmartCoreHeadProjectionDryRunResultV0,
) {
    let sample = fs::read_to_string("examples/minimal_ai_committee_multi_market_sample.json")
        .expect("sprint176 batch input");
    let batch_input: BatchCommitteeCycleInput =
        serde_json::from_str(&sample).expect("parse sprint176 batch input");
    let batch_result =
        core::run_batch_committee_cycle(batch_input).expect("sprint176 batch result");
    let (_profile_count, microkernel_dry_run) = sprint175_microkernel_dry_run_fixture();
    let head_projection = run_smartcore_head_projection_dry_run_v0(
        &microkernel_dry_run,
        &SmartCoreHeadProjectionDryRunConfigV0 {
            run_id: "sprint176-head-projection".to_string(),
            enable_stance_head: true,
            enable_risk_head: true,
            enable_evidence_head: true,
            enable_confidence_head: true,
            enable_uncertainty_head: true,
            enable_expected_return_head: false,
            output_path: None,
            paper_only: true,
        },
    )
    .expect("sprint176 head projection");
    (batch_result, head_projection)
}

fn sprint177_mismatch_pipeline_fixture() -> (
    BatchCommitteeCycleResult,
    core::SmartCoreShadowAlignmentRunResult,
    core::SmartCoreMismatchSelfGrowingRunResult,
) {
    let config = MinimalAiCommitteeCycleConfig::from_toml_path(std::path::Path::new(
        "examples/soma_minimal_ai_committee_core.toml",
    ))
    .expect("sprint177 config");
    let batch_result = core::run_batch_committee_cycle(
        config
            .load_batch_input()
            .expect("sprint177 configured batch input"),
    )
    .expect("sprint177 configured batch result");
    let (_sprint176_batch_result, head_projection) = sprint176_shadow_alignment_fixture();
    let shadow_alignment = core::run_smartcore_shadow_alignment(
        &head_projection.debug_output_batch,
        &batch_result,
        Some(&batch_result.replay_dataset),
        &core::SmartCoreShadowAlignmentRunConfig {
            run_id: "sprint177-shadow".to_string(),
            enabled: true,
            include_batch_member_opinions: true,
            include_replay_targets: true,
            include_risk_governor_targets: true,
            output_path: None,
            emit_owner_core_debug_cards: true,
            paper_only: true,
        },
    )
    .expect("sprint177 shadow alignment");
    let pipeline = core::run_smartcore_mismatch_self_growing_pipeline(
        &shadow_alignment,
        &core::SmartCoreMismatchSelfGrowingRunConfig {
            run_id: "sprint177-mismatch".to_string(),
            enabled: true,
            max_tasks_total: 12,
            max_tasks_per_member: 4,
            core_calibration_dataset_output_path: None,
            mismatch_task_output_path: None,
            emit_owner_console_summary: true,
            paper_only: true,
        },
    )
    .expect("sprint177 mismatch pipeline");
    (batch_result, shadow_alignment, pipeline)
}

fn sprint177_sample_mismatch_records() -> Vec<core::SmartCoreShadowMismatchRecord> {
    vec![
        core::SmartCoreShadowMismatchRecord {
            mismatch_id: "risk-under".to_string(),
            member_id: "risk-kr-short".to_string(),
            head: core::SmartCoreShadowHeadKind::Risk,
            debug_bucket: "RiskLow".to_string(),
            target_value: "RiskHigh".to_string(),
            symbol: Some("005930.KS".to_string()),
            market_scope: Some(MarketScope::KoreaShortTerm),
            mismatch_type: core::SmartCoreShadowMismatchType::RiskUnderestimation,
            severity: core::SmartCoreShadowMismatchSeverity::High,
            suggested_data_need: core::SmartCoreShadowSuggestedNextCoreData::MoreRiskLabels,
            paper_only: true,
        },
        core::SmartCoreShadowMismatchRecord {
            mismatch_id: "evidence-over".to_string(),
            member_id: "evidence-kr-short".to_string(),
            head: core::SmartCoreShadowHeadKind::EvidenceNeed,
            debug_bucket: "EvidenceSufficient".to_string(),
            target_value: "NeedMoreEvidence".to_string(),
            symbol: Some("005930.KS".to_string()),
            market_scope: Some(MarketScope::KoreaShortTerm),
            mismatch_type: core::SmartCoreShadowMismatchType::EvidenceOverconfidence,
            severity: core::SmartCoreShadowMismatchSeverity::High,
            suggested_data_need: core::SmartCoreShadowSuggestedNextCoreData::MoreEvidenceLabels,
            paper_only: true,
        },
        core::SmartCoreShadowMismatchRecord {
            mismatch_id: "stance-disagree".to_string(),
            member_id: "trend-kr-short".to_string(),
            head: core::SmartCoreShadowHeadKind::Stance,
            debug_bucket: "PositiveLike".to_string(),
            target_value: "NegativeLike".to_string(),
            symbol: Some("005930.KS".to_string()),
            market_scope: Some(MarketScope::KoreaShortTerm),
            mismatch_type: core::SmartCoreShadowMismatchType::StanceDisagreement,
            severity: core::SmartCoreShadowMismatchSeverity::Medium,
            suggested_data_need: core::SmartCoreShadowSuggestedNextCoreData::MoreStanceLabels,
            paper_only: true,
        },
        core::SmartCoreShadowMismatchRecord {
            mismatch_id: "confidence-over".to_string(),
            member_id: "trend-kr-short".to_string(),
            head: core::SmartCoreShadowHeadKind::ConfidenceCalibration,
            debug_bucket: "ConfidenceHigh".to_string(),
            target_value: "ConfidenceLow".to_string(),
            symbol: Some("BTCUSDT".to_string()),
            market_scope: Some(MarketScope::CryptoShortTerm),
            mismatch_type: core::SmartCoreShadowMismatchType::ConfidenceOverstatement,
            severity: core::SmartCoreShadowMismatchSeverity::Medium,
            suggested_data_need:
                core::SmartCoreShadowSuggestedNextCoreData::MoreConfidenceCalibrationLabels,
            paper_only: true,
        },
    ]
}

fn sprint178_learning_loop_fixture(
    dry_run: bool,
) -> (
    BatchCommitteeCycleResult,
    core::SmartCoreShadowAlignmentRunResult,
    core::SmartCoreMismatchLearningLoopResult,
) {
    let config = MinimalAiCommitteeCycleConfig::from_toml_path(std::path::Path::new(
        "examples/soma_minimal_ai_committee_core.toml",
    ))
    .expect("sprint178 config");
    let batch_result = core::run_batch_committee_cycle(
        config
            .load_batch_input()
            .expect("sprint178 configured batch input"),
    )
    .expect("sprint178 configured batch result");
    let (_sprint176_batch_result, head_projection) = sprint176_shadow_alignment_fixture();
    let shadow_alignment = core::run_smartcore_shadow_alignment(
        &head_projection.debug_output_batch,
        &batch_result,
        Some(&batch_result.replay_dataset),
        &core::SmartCoreShadowAlignmentRunConfig {
            run_id: "sprint178-shadow".to_string(),
            enabled: true,
            include_batch_member_opinions: true,
            include_replay_targets: true,
            include_risk_governor_targets: true,
            output_path: None,
            emit_owner_core_debug_cards: true,
            paper_only: true,
        },
    )
    .expect("sprint178 shadow alignment");
    let source_registry = core::ResearchSourceRegistry::load_from_local_json(std::path::Path::new(
        "examples/research_sources.sample.json",
    ))
    .expect("research source registry");
    let loop_result = core::run_smartcore_mismatch_learning_loop(
        &shadow_alignment,
        &source_registry,
        None,
        &head_projection.debug_output_batch,
        &core::SmartCoreMismatchLearningLoopConfig {
            run_id: "sprint178-learning-loop".to_string(),
            execute_research_tasks: true,
            build_target_candidates: true,
            approve_targets: true,
            refresh_calibration_dataset: true,
            recheck_alignment: true,
            dry_run,
            paper_only: true,
        },
    )
    .expect("sprint178 learning loop");
    (batch_result, shadow_alignment, loop_result)
}

fn sprint179_recalibration_dataset(
    debug_output_batch: &core::SmartCoreDebugOutputBatchV0,
) -> core::CoreCalibrationDataset {
    let risk_debug_bucket = debug_output_batch
        .member_outputs
        .iter()
        .find(|output| output.member_id == "risk-kr-short")
        .and_then(|output| output.risk_head.as_ref())
        .map(|head| {
            format!(
                "{:?}",
                core::normalize_smartcore_head_bucket(
                    core::SmartCoreShadowHeadKind::Risk,
                    format!("{:?}", head.bucket),
                )
                .normalized_value
            )
        })
        .expect("risk debug bucket");
    let evidence_debug_bucket = debug_output_batch
        .member_outputs
        .iter()
        .find(|output| output.member_id == "evidence-kr-short")
        .and_then(|output| output.evidence_need_head.as_ref())
        .map(|head| {
            format!(
                "{:?}",
                core::normalize_smartcore_head_bucket(
                    core::SmartCoreShadowHeadKind::EvidenceNeed,
                    format!("{:?}", head.bucket),
                )
                .normalized_value
            )
        })
        .expect("evidence debug bucket");
    let stance_debug_bucket = debug_output_batch
        .member_outputs
        .iter()
        .find(|output| output.member_id == "trend-kr-short")
        .and_then(|output| output.stance_head.as_ref())
        .map(|head| {
            format!(
                "{:?}",
                core::normalize_smartcore_head_bucket(
                    core::SmartCoreShadowHeadKind::Stance,
                    format!("{:?}", head.bucket),
                )
                .normalized_value
            )
        })
        .expect("stance debug bucket");
    let expected_return_debug_bucket = debug_output_batch
        .member_outputs
        .iter()
        .find(|output| output.member_id == "trend-kr-short")
        .and_then(|output| output.expected_return_head.as_ref())
        .map(|head| {
            format!(
                "{:?}",
                core::normalize_smartcore_head_bucket(
                    core::SmartCoreShadowHeadKind::ExpectedReturnHint,
                    format!("{:?}", head.bucket),
                )
                .normalized_value
            )
        })
        .unwrap_or_else(|| "Deferred".to_string());
    core::build_core_calibration_dataset(&vec![
        core::CoreCalibrationExample {
            calibration_example_id: "sprint179-risk-1".to_string(),
            member_id: "risk-kr-short".to_string(),
            debug_output_id: "risk-debug-1".to_string(),
            target_id: Some("risk-target-1".to_string()),
            symbol: Some("MSFT".to_string()),
            market_scope: Some(MarketScope::UsShortTerm),
            head: core::SmartCoreShadowHeadKind::Risk,
            debug_bucket: risk_debug_bucket.clone(),
            target_bucket: "RiskHigh".to_string(),
            alignment: core::SmartCoreShadowAlignmentStatus::Mismatch,
            mismatch_type: Some(core::SmartCoreShadowMismatchType::RiskUnderestimation),
            suggested_data_need: core::SmartCoreMismatchDataNeed::MoreRiskLabels,
            label_source: core::CoreCalibrationLabelSource::ReplayLabel,
            label_confidence: ReplayLabelConfidence::High,
            paper_only: true,
        },
        core::CoreCalibrationExample {
            calibration_example_id: "sprint179-risk-2".to_string(),
            member_id: "risk-kr-short".to_string(),
            debug_output_id: "risk-debug-2".to_string(),
            target_id: Some("risk-target-2".to_string()),
            symbol: Some("NVDA".to_string()),
            market_scope: Some(MarketScope::UsShortTerm),
            head: core::SmartCoreShadowHeadKind::Risk,
            debug_bucket: risk_debug_bucket.clone(),
            target_bucket: "RiskHigh".to_string(),
            alignment: core::SmartCoreShadowAlignmentStatus::Mismatch,
            mismatch_type: Some(core::SmartCoreShadowMismatchType::RiskUnderestimation),
            suggested_data_need: core::SmartCoreMismatchDataNeed::MoreRiskVetoCases,
            label_source: core::CoreCalibrationLabelSource::ReplayLabel,
            label_confidence: ReplayLabelConfidence::High,
            paper_only: true,
        },
        core::CoreCalibrationExample {
            calibration_example_id: "sprint179-risk-3".to_string(),
            member_id: "risk-kr-short".to_string(),
            debug_output_id: "risk-debug-3".to_string(),
            target_id: Some("risk-target-3".to_string()),
            symbol: Some("TSLA".to_string()),
            market_scope: Some(MarketScope::UsShortTerm),
            head: core::SmartCoreShadowHeadKind::Risk,
            debug_bucket: risk_debug_bucket,
            target_bucket: "RiskHigh".to_string(),
            alignment: core::SmartCoreShadowAlignmentStatus::Mismatch,
            mismatch_type: Some(core::SmartCoreShadowMismatchType::RiskUnderestimation),
            suggested_data_need: core::SmartCoreMismatchDataNeed::MoreRiskLabels,
            label_source: core::CoreCalibrationLabelSource::RiskGovernorStatus,
            label_confidence: ReplayLabelConfidence::High,
            paper_only: true,
        },
        core::CoreCalibrationExample {
            calibration_example_id: "sprint179-evidence-1".to_string(),
            member_id: "evidence-kr-short".to_string(),
            debug_output_id: "evidence-debug-1".to_string(),
            target_id: Some("evidence-target-1".to_string()),
            symbol: Some("AAPL".to_string()),
            market_scope: Some(MarketScope::UsShortTerm),
            head: core::SmartCoreShadowHeadKind::EvidenceNeed,
            debug_bucket: evidence_debug_bucket,
            target_bucket: "NeedMoreEvidence".to_string(),
            alignment: core::SmartCoreShadowAlignmentStatus::Mismatch,
            mismatch_type: Some(core::SmartCoreShadowMismatchType::EvidenceOverconfidence),
            suggested_data_need: core::SmartCoreMismatchDataNeed::MoreNeedMoreEvidenceCases,
            label_source: core::CoreCalibrationLabelSource::PaperOutcomeLabel,
            label_confidence: ReplayLabelConfidence::Medium,
            paper_only: true,
        },
        core::CoreCalibrationExample {
            calibration_example_id: "sprint179-stance-1".to_string(),
            member_id: "trend-kr-short".to_string(),
            debug_output_id: "stance-debug-1".to_string(),
            target_id: Some("stance-target-1".to_string()),
            symbol: Some("005930.KS".to_string()),
            market_scope: Some(MarketScope::KoreaShortTerm),
            head: core::SmartCoreShadowHeadKind::Stance,
            debug_bucket: stance_debug_bucket.clone(),
            target_bucket: stance_debug_bucket.clone(),
            alignment: core::SmartCoreShadowAlignmentStatus::Match,
            mismatch_type: None,
            suggested_data_need: core::SmartCoreMismatchDataNeed::KeepObserving,
            label_source: core::CoreCalibrationLabelSource::ReplayLabel,
            label_confidence: ReplayLabelConfidence::High,
            paper_only: true,
        },
        core::CoreCalibrationExample {
            calibration_example_id: "sprint179-stance-2".to_string(),
            member_id: "trend-kr-short".to_string(),
            debug_output_id: "stance-debug-2".to_string(),
            target_id: Some("stance-target-2".to_string()),
            symbol: Some("BTCUSDT".to_string()),
            market_scope: Some(MarketScope::CryptoShortTerm),
            head: core::SmartCoreShadowHeadKind::Stance,
            debug_bucket: stance_debug_bucket.clone(),
            target_bucket: stance_debug_bucket,
            alignment: core::SmartCoreShadowAlignmentStatus::Match,
            mismatch_type: None,
            suggested_data_need: core::SmartCoreMismatchDataNeed::KeepObserving,
            label_source: core::CoreCalibrationLabelSource::ReplayLabel,
            label_confidence: ReplayLabelConfidence::High,
            paper_only: true,
        },
        core::CoreCalibrationExample {
            calibration_example_id: "sprint179-return-1".to_string(),
            member_id: "trend-kr-short".to_string(),
            debug_output_id: "return-debug-1".to_string(),
            target_id: Some("return-target-1".to_string()),
            symbol: Some("AMZN".to_string()),
            market_scope: Some(MarketScope::UsShortTerm),
            head: core::SmartCoreShadowHeadKind::ExpectedReturnHint,
            debug_bucket: expected_return_debug_bucket.clone(),
            target_bucket: "NegativeLike".to_string(),
            alignment: core::SmartCoreShadowAlignmentStatus::Mismatch,
            mismatch_type: Some(core::SmartCoreShadowMismatchType::StanceDisagreement),
            suggested_data_need: core::SmartCoreMismatchDataNeed::KeepObserving,
            label_source: core::CoreCalibrationLabelSource::PaperOutcomeLabel,
            label_confidence: ReplayLabelConfidence::Medium,
            paper_only: true,
        },
        core::CoreCalibrationExample {
            calibration_example_id: "sprint179-return-2".to_string(),
            member_id: "trend-kr-short".to_string(),
            debug_output_id: "return-debug-2".to_string(),
            target_id: Some("return-target-2".to_string()),
            symbol: Some("META".to_string()),
            market_scope: Some(MarketScope::UsShortTerm),
            head: core::SmartCoreShadowHeadKind::ExpectedReturnHint,
            debug_bucket: expected_return_debug_bucket.clone(),
            target_bucket: "NegativeLike".to_string(),
            alignment: core::SmartCoreShadowAlignmentStatus::Mismatch,
            mismatch_type: Some(core::SmartCoreShadowMismatchType::StanceDisagreement),
            suggested_data_need: core::SmartCoreMismatchDataNeed::KeepObserving,
            label_source: core::CoreCalibrationLabelSource::PaperOutcomeLabel,
            label_confidence: ReplayLabelConfidence::Medium,
            paper_only: true,
        },
        core::CoreCalibrationExample {
            calibration_example_id: "sprint179-return-3".to_string(),
            member_id: "trend-kr-short".to_string(),
            debug_output_id: "return-debug-3".to_string(),
            target_id: Some("return-target-3".to_string()),
            symbol: Some("GOOG".to_string()),
            market_scope: Some(MarketScope::UsShortTerm),
            head: core::SmartCoreShadowHeadKind::ExpectedReturnHint,
            debug_bucket: expected_return_debug_bucket,
            target_bucket: "NegativeLike".to_string(),
            alignment: core::SmartCoreShadowAlignmentStatus::Mismatch,
            mismatch_type: Some(core::SmartCoreShadowMismatchType::StanceDisagreement),
            suggested_data_need: core::SmartCoreMismatchDataNeed::KeepObserving,
            label_source: core::CoreCalibrationLabelSource::PaperOutcomeLabel,
            label_confidence: ReplayLabelConfidence::Medium,
            paper_only: true,
        },
    ])
}

fn sprint179_recalibration_fixture(
    dry_run: bool,
) -> (
    BatchCommitteeCycleResult,
    core::SmartCoreDebugOutputBatchV0,
    core::SmartCoreShadowAlignmentRunResult,
    core::CoreCalibrationDataset,
    core::SmartCoreShadowRecalibrationRunResult,
) {
    let (batch_result, head_projection) = sprint176_shadow_alignment_fixture();
    let shadow_alignment = core::run_smartcore_shadow_alignment(
        &head_projection.debug_output_batch,
        &batch_result,
        Some(&batch_result.replay_dataset),
        &core::SmartCoreShadowAlignmentRunConfig {
            run_id: "sprint179-shadow".to_string(),
            enabled: true,
            include_batch_member_opinions: true,
            include_replay_targets: true,
            include_risk_governor_targets: true,
            output_path: None,
            emit_owner_core_debug_cards: true,
            paper_only: true,
        },
    )
    .expect("sprint179 shadow alignment");
    let dataset = sprint179_recalibration_dataset(&head_projection.debug_output_batch);
    let recalibration_result = core::run_smartcore_shadow_recalibration_pass(
        &head_projection.debug_output_batch,
        &dataset,
        &shadow_alignment,
        &batch_result,
        &core::SmartCoreShadowRecalibrationRunConfig {
            run_id: "sprint179-recalibration".to_string(),
            enabled: true,
            calibration_dataset_path: None,
            rule_table_output_path: None,
            calibrated_debug_output_path: None,
            recalibration_result_output_path: None,
            min_support_for_active_rule: 2,
            max_rules_per_member_head: 2,
            emit_owner_summary: true,
            dry_run,
            paper_only: true,
        },
    )
    .expect("sprint179 recalibration");
    (
        batch_result,
        head_projection.debug_output_batch,
        shadow_alignment,
        dataset,
        recalibration_result,
    )
}

fn sprint180_shadow_opinion_fixture() -> (
    BatchCommitteeCycleResult,
    core::SmartCoreShadowRecalibrationRunResult,
    core::SmartCoreShadowOpinionRunResult,
) {
    let (batch_result, _debug_output_batch, _shadow_alignment, _dataset, recalibration_result) =
        sprint179_recalibration_fixture(false);
    let run_result = core::run_smartcore_shadow_opinion_lane(
        &recalibration_result.calibrated_debug_output_batch,
        &batch_result,
        Some(&batch_result.replay_dataset),
        &core::SmartCoreShadowOpinionRunConfig {
            run_id: "sprint180-shadow-opinion".to_string(),
            enabled: true,
            output_path: None,
            include_member_opinion_comparison: true,
            include_target_eval: true,
            emit_owner_debug_summary: true,
            paper_only: true,
        },
    )
    .expect("sprint180 shadow opinion");
    (batch_result, recalibration_result, run_result)
}

fn sprint181_shadow_stability_fixture(
    expand_targets: bool,
) -> (
    BatchCommitteeCycleResult,
    core::CoreCalibrationDataset,
    core::SmartCoreShadowOpinionRunResult,
    core::SmartCoreShadowStabilityRunResult,
) {
    let (batch_result, _debug_output_batch, _shadow_alignment, dataset, recalibration_result) =
        sprint179_recalibration_fixture(false);
    let shadow_opinion_run = core::run_smartcore_shadow_opinion_lane(
        &recalibration_result.calibrated_debug_output_batch,
        &batch_result,
        Some(&batch_result.replay_dataset),
        &core::SmartCoreShadowOpinionRunConfig {
            run_id: "sprint181-shadow-opinion".to_string(),
            enabled: true,
            output_path: None,
            include_member_opinion_comparison: true,
            include_target_eval: true,
            emit_owner_debug_summary: true,
            paper_only: true,
        },
    )
    .expect("sprint181 shadow opinion");
    let stability_run = core::run_smartcore_shadow_stability_eval(
        &shadow_opinion_run,
        &batch_result,
        Some(&batch_result.replay_dataset),
        expand_targets.then_some(&dataset),
        None,
        &core::SmartCoreShadowStabilityRunConfig {
            run_id: "sprint181-shadow-stability".to_string(),
            enabled: true,
            repeated_run_count: 3,
            include_same_input_repeat: true,
            include_calibrated_output_repeat: true,
            include_shadow_candidate_repeat: true,
            include_target_eval_repeat: true,
            max_allowed_action_flip_rate: 0.0,
            max_allowed_head_flip_rate: 0.0,
            output_path: None,
            paper_only: true,
        },
    )
    .expect("sprint181 shadow stability");
    (batch_result, dataset, shadow_opinion_run, stability_run)
}

fn sprint182_shadow_scenario_context() -> (
    BatchCommitteeCycleResult,
    core::CoreCalibrationDataset,
    core::CalibratedSmartCoreDebugOutputBatchV0,
) {
    let (batch_result, _debug_output_batch, _shadow_alignment, dataset, recalibration_result) =
        sprint179_recalibration_fixture(false);
    (
        batch_result,
        dataset,
        recalibration_result.calibrated_debug_output_batch,
    )
}

fn sprint182_shadow_scenario_sweep_fixture() -> (
    BatchCommitteeCycleResult,
    core::SmartCoreShadowScenarioSweepResult,
) {
    let (batch_result, dataset, calibrated_debug_batch) = sprint182_shadow_scenario_context();
    let mut sweep = core::run_smartcore_shadow_scenario_sweep(
        &calibrated_debug_batch,
        &batch_result,
        &batch_result.replay_dataset,
        Some(&dataset),
        None,
        &core::SmartCoreShadowScenarioSweepConfig {
            run_id: "sprint182-shadow-scenario-sweep".to_string(),
            scenario_set_path: None,
            repeated_run_count: 3,
            max_scenarios: 5,
            include_same_input_determinism: true,
            include_cross_scenario_sensitivity: true,
            expand_targets_per_scenario: true,
            output_path: None,
            paper_only: true,
        },
    )
    .expect("sprint182 scenario sweep");
    sweep.observer_readiness_gate = Some(core::evaluate_smartcore_observer_readiness(
        &sweep,
        &core::SmartCoreObserverReadinessPolicy {
            min_scenarios_required: 3,
            require_zero_same_input_flip_rate: true,
            allow_reasonable_cross_scenario_variation: true,
            require_decision_isolation: true,
            require_no_training: true,
            require_no_live_inference: true,
            paper_only: true,
        },
    ));
    sweep.owner_debug_summary = Some(core::build_owner_shadow_scenario_sweep_debug_summary(
        &sweep,
        sweep
            .observer_readiness_gate
            .as_ref()
            .expect("observer gate"),
    ));
    (batch_result, sweep)
}

fn sprint182_scenario_result(
    scenario_id: &str,
    scenario_kind: core::SmartCoreShadowScenarioKind,
    action: core::SmartCoreShadowOpinionAction,
    head_signature: &str,
    target_quality_status: core::SmartCoreAgreementTargetQualityStatus,
) -> core::SmartCoreShadowScenarioStabilityResult {
    core::SmartCoreShadowScenarioStabilityResult {
        scenario_id: scenario_id.to_string(),
        scenario_kind,
        repeated_run_count: 3,
        sample_count: 9,
        action_flip_rate: 0.0,
        head_bucket_flip_rate: 0.0,
        deterministic_status: core::SmartCoreShadowStabilityDeterministicStatus::Deterministic,
        target_count_before: 3,
        target_count_after: 6,
        target_quality_status,
        mismatch_count_before: Some(0),
        mismatch_count_after: Some(0),
        decision_isolation_status:
            core::SmartCoreShadowStabilityDecisionIsolationRegressionStatus::Preserved,
        action_signature: [
            ("evidence-kr-short".to_string(), action),
            ("risk-kr-short".to_string(), action),
            ("trend-kr-short".to_string(), action),
        ]
        .into_iter()
        .collect(),
        head_signature: [
            ("evidence-kr-short".to_string(), head_signature.to_string()),
            ("risk-kr-short".to_string(), head_signature.to_string()),
            ("trend-kr-short".to_string(), head_signature.to_string()),
        ]
        .into_iter()
        .collect(),
        target_collection_tasks: match target_quality_status {
            core::SmartCoreAgreementTargetQualityStatus::ThinHeadCoverage => vec![
                core::SmartCoreAgreementTargetCollectionTask {
                    task_id: format!("{scenario_id}-risk"),
                    member_id: None,
                    head: Some(core::SmartCoreShadowHeadKind::Risk),
                    target_need: core::SmartCoreAgreementTargetNeed::RiskTarget,
                    priority: core::SmartCoreAgreementTargetTaskPriority::High,
                    reason: "Need more risk targets.".to_string(),
                    paper_only: true,
                },
                core::SmartCoreAgreementTargetCollectionTask {
                    task_id: format!("{scenario_id}-evidence"),
                    member_id: None,
                    head: Some(core::SmartCoreShadowHeadKind::EvidenceNeed),
                    target_need: core::SmartCoreAgreementTargetNeed::EvidenceTarget,
                    priority: core::SmartCoreAgreementTargetTaskPriority::High,
                    reason: "Need more evidence targets.".to_string(),
                    paper_only: true,
                },
            ],
            _ => Vec::new(),
        },
        decision_isolation_violations: Vec::new(),
        paper_only: true,
    }
}

fn sprint183_observer_lane_fixture() -> (
    BatchCommitteeCycleResult,
    core::SmartCoreObserverLaneRunResult,
) {
    let (batch_result, _recalibration_result, shadow_opinion_run) =
        sprint180_shadow_opinion_fixture();
    let (_scenario_batch_result, sweep) = sprint182_shadow_scenario_sweep_fixture();
    let run_result = core::run_smartcore_observer_lane(
        &shadow_opinion_run.candidate_batch,
        &batch_result,
        Some(&sweep.target_coverage_stress),
        &core::SmartCoreObserverLaneRunConfig {
            run_id: "sprint183-observer-lane".to_string(),
            enabled: true,
            output_path: None,
            compare_member_opinion: true,
            compare_chairman_decision: true,
            compare_risk_governor: true,
            build_target_coverage_closure_queue: true,
            emit_owner_observer_section: true,
            paper_only: true,
        },
    )
    .expect("observer lane");
    (batch_result, run_result)
}

fn sprint184_closure_fixture() -> (
    BatchCommitteeCycleResult,
    core::SmartCoreObserverLaneRunResult,
    core::ObserverTargetCoverageClosureRunResult,
    core::ObserverAgreementTargetSet,
    core::ObserverComparisonRerunResult,
    core::ObserverTargetClosureDecisionIsolationGuard,
    core::SmartCoreObserverReadinessHardeningGate,
) {
    let (mut batch_result, observer_run_result) = sprint183_observer_lane_fixture();
    let queue = observer_run_result
        .target_coverage_closure_result
        .as_ref()
        .expect("closure queue")
        .closure_queue
        .clone();
    let closure_result = core::run_observer_target_coverage_closure(
        Some(&queue),
        &batch_result,
        None,
        None,
        None,
        &core::ObserverTargetCoverageClosureRunConfig {
            run_id: "sprint184-closure".to_string(),
            enabled: true,
            closure_queue_input_path: None,
            observer_targets_output_path: None,
            replay_dataset_path: None,
            calibration_dataset_path: None,
            paper_evidence_path: None,
            dry_run: false,
            max_items: 16,
            paper_only: true,
        },
    )
    .expect("closure run");
    let target_set =
        core::refresh_observer_agreement_target_set(&[], &closure_result.target_records)
            .refreshed_target_set;
    let rerun_result =
        core::rerun_observer_comparison_with_refreshed_targets(&observer_run_result, &target_set);
    let decision_isolation_guard = core::evaluate_observer_target_closure_decision_isolation(
        &closure_result,
        &batch_result,
        Some(&batch_result),
    );
    let hardening_gate = core::harden_smartcore_observer_readiness(
        &observer_run_result,
        &closure_result,
        &rerun_result,
        &decision_isolation_guard,
    );
    batch_result.smartcore_observer_lane_run_result = Some(observer_run_result.clone());
    batch_result.observer_target_coverage_closure_run_result = Some(closure_result.clone());
    batch_result.observer_agreement_target_set = Some(target_set.clone());
    batch_result.observer_comparison_rerun_result = Some(rerun_result.clone());
    batch_result.observer_target_closure_decision_isolation_guard =
        Some(decision_isolation_guard.clone());
    batch_result.observer_readiness_hardening_gate = Some(hardening_gate.clone());
    (
        batch_result,
        observer_run_result,
        closure_result,
        target_set,
        rerun_result,
        decision_isolation_guard,
        hardening_gate,
    )
}

fn sprint185_target(
    target_id: &str,
    approval_status: core::ObserverAgreementTargetApprovalStatus,
    source_type: core::ObserverAgreementTargetSource,
    source_confidence: core::SourceConfidence,
    reason: &str,
) -> core::ObserverAgreementTargetRecord {
    core::ObserverAgreementTargetRecord {
        target_id: target_id.to_string(),
        source_closure_item_id: Some(format!("{target_id}-item")),
        source_record_id: Some("batch".to_string()),
        member_id: Some("trend-kr-short".to_string()),
        canonical_member_id: Some("trend-kr-short".to_string()),
        symbol: Some("AAPL".to_string()),
        market_scope: Some(MarketScope::UsShortTerm),
        head: core::ObserverAgreementTargetHead::Stance,
        target_bucket: core::SmartCoreHeadBucketNormalizedValue::PositiveLike,
        source_type,
        source_confidence,
        approval_status,
        reason: reason.to_string(),
        eval_only: true,
        not_input_feature: true,
        paper_only: true,
    }
}

fn sprint185_closure_result_with_targets(
    run_id: &str,
    targets: Vec<core::ObserverAgreementTargetRecord>,
) -> core::ObserverTargetCoverageClosureRunResult {
    core::ObserverTargetCoverageClosureRunResult {
        run_id: run_id.to_string(),
        input_item_count: targets.len(),
        executed_item_count: targets.len(),
        closed_count: targets
            .iter()
            .filter(|target| {
                target.approval_status == core::ObserverAgreementTargetApprovalStatus::Approved
            })
            .count(),
        needs_review_count: targets
            .iter()
            .filter(|target| {
                target.approval_status == core::ObserverAgreementTargetApprovalStatus::NeedsReview
            })
            .count(),
        rejected_count: targets
            .iter()
            .filter(|target| {
                target.approval_status == core::ObserverAgreementTargetApprovalStatus::Rejected
            })
            .count(),
        generated_target_count: targets.len(),
        approved_target_count: targets
            .iter()
            .filter(|target| {
                target.approval_status == core::ObserverAgreementTargetApprovalStatus::Approved
            })
            .count(),
        target_records: targets,
        execution_results: Vec::new(),
        dry_run: false,
        run_status: core::ObserverTargetCoverageClosureRunStatus::Passed,
        paper_only: true,
    }
}

fn sprint185_apply_trend_fixture(
    apply_targets: bool,
    dry_run: bool,
) -> (
    BatchCommitteeCycleResult,
    core::SmartCoreObserverLaneRunResult,
    core::ObserverTargetCoverageClosureRunResult,
    core::ObserverTargetApplyAndTrendRunResult,
    PathBuf,
    PathBuf,
) {
    let (batch_result, observer_run_result, closure_result, ..) = sprint184_closure_fixture();
    let store_path = sprint171_temp_json_path("observer-target-store");
    let ledger_path = sprint171_temp_json_path("observer-ledger-v2");
    let _ = fs::remove_file(&store_path);
    let _ = fs::remove_file(&ledger_path);
    let run_result = core::run_observer_target_apply_and_trend(
        &observer_run_result,
        &closure_result,
        &batch_result,
        &core::ObserverTargetApplyAndTrendRunConfig {
            run_id: "sprint185-apply-trend".to_string(),
            enabled: true,
            closure_result_path: None,
            target_store_input_path: None,
            target_store_output_path: Some(store_path.to_string_lossy().to_string()),
            observer_ledger_path: Some(ledger_path.to_string_lossy().to_string()),
            dry_run,
            apply_targets,
            compute_trend: true,
            recheck_readiness: true,
            emit_owner_summary: true,
            paper_only: true,
        },
    )
    .expect("apply trend run");
    (
        batch_result,
        observer_run_result,
        closure_result,
        run_result,
        store_path,
        ledger_path,
    )
}

fn sprint186_seed_apply_fixture(
    apply_targets: bool,
    dry_run: bool,
) -> (
    BatchCommitteeCycleResult,
    core::SmartCoreObserverLaneRunResult,
    core::ObserverSeedApplyTrendRunResult,
    PathBuf,
    PathBuf,
    PathBuf,
) {
    let (mut batch_result, observer_run_result, _closure_result, apply_trend, ..) =
        sprint185_apply_trend_fixture(true, false);
    batch_result.observer_readiness_v2_gate = Some(apply_trend.readiness_v2);
    let target_store_path = sprint171_temp_json_path("observer-seed-target-store");
    let ledger_path = sprint171_temp_json_path("observer-seed-ledger");
    let output_path = sprint171_temp_json_path("observer-seed-apply-output");
    let _ = fs::remove_file(&target_store_path);
    let _ = fs::remove_file(&ledger_path);
    let _ = fs::remove_file(&output_path);
    let run_result = core::run_observer_seed_apply_trend(
        &batch_result,
        None,
        None,
        None,
        &observer_run_result,
        &core::ObserverSeedApplyTrendRunConfig {
            run_id: "sprint186-seed-apply-trend".to_string(),
            enabled: true,
            dry_run,
            apply_targets,
            target_store_input_path: None,
            target_store_output_path: Some(target_store_path.to_string_lossy().to_string()),
            observer_ledger_path: Some(ledger_path.to_string_lossy().to_string()),
            output_path: Some(output_path.to_string_lossy().to_string()),
            require_approved_target: true,
            rerun_comparison: true,
            compute_ledger_trend: true,
            recheck_readiness: true,
            emit_owner_summary: true,
            paper_only: true,
        },
    )
    .expect("seed apply trend run");
    (
        batch_result,
        observer_run_result,
        run_result,
        target_store_path,
        ledger_path,
        output_path,
    )
}

fn sprint187_apply_governance_fixture(
    apply_mode: core::ObserverExplicitApplyMode,
    dry_run: bool,
) -> (
    BatchCommitteeCycleResult,
    core::SmartCoreObserverLaneRunResult,
    core::ObserverApprovedApplyAndGovernancePrepRunResult,
    PathBuf,
    PathBuf,
    PathBuf,
) {
    let (mut batch_result, observer_run_result, seed_run, ..) =
        sprint186_seed_apply_fixture(true, false);
    batch_result.observer_seed_apply_trend_run_result = Some(seed_run.clone());
    batch_result.observer_agreement_target_store = seed_run
        .controlled_apply_smoke_result
        .target_store_after
        .clone();
    let store_path = sprint171_temp_json_path("observer-approved-apply-store");
    let ledger_path = sprint171_temp_json_path("observer-approved-apply-ledger");
    let output_path = sprint171_temp_json_path("observer-approved-apply-output");
    let _ = fs::remove_file(&store_path);
    let _ = fs::remove_file(&ledger_path);
    let _ = fs::remove_file(&output_path);
    let run_result = core::run_observer_approved_apply_and_governance_prep(
        &batch_result,
        &seed_run
            .controlled_apply_smoke_result
            .seed_conversion_result
            .target_records,
        &observer_run_result,
        &core::ObserverApprovedApplyAndGovernancePrepRunConfig {
            run_id: "sprint187-approved-apply-governance".to_string(),
            enabled: true,
            apply_mode,
            dry_run,
            target_store_input_path: None,
            target_store_output_path: Some(store_path.to_string_lossy().to_string()),
            observer_ledger_path: Some(ledger_path.to_string_lossy().to_string()),
            output_path: Some(output_path.to_string_lossy().to_string()),
            recheck_observer_readiness: true,
            prepare_chairman_governance_contract: true,
            emit_owner_summary: true,
            paper_only: true,
        },
    )
    .expect("approved apply governance run");
    (
        batch_result,
        observer_run_result,
        run_result,
        store_path,
        ledger_path,
        output_path,
    )
}

fn sprint188_apply_verify_shadow_fixture(
    apply_mode: core::ObserverExplicitApplyMode,
    dry_run: bool,
    run_chairman_shadow_governance: bool,
) -> (
    BatchCommitteeCycleResult,
    core::SmartCoreObserverLaneRunResult,
    core::ObserverApplyVerifyAndChairmanShadowRunResult,
    PathBuf,
    PathBuf,
) {
    let (mut batch_result, observer_run_result, seed_run, ..) =
        sprint186_seed_apply_fixture(true, false);
    batch_result.observer_seed_apply_trend_run_result = Some(seed_run.clone());
    batch_result.observer_agreement_target_store = seed_run
        .controlled_apply_smoke_result
        .target_store_after
        .clone();
    let store_path = sprint171_temp_json_path("observer-apply-verify-store");
    let output_path = sprint171_temp_json_path("observer-apply-verify-output");
    let _ = fs::remove_file(&store_path);
    let _ = fs::remove_file(&output_path);
    let run_result = core::run_observer_apply_verify_and_chairman_shadow(
        &batch_result,
        &seed_run
            .controlled_apply_smoke_result
            .seed_conversion_result
            .target_records,
        &observer_run_result,
        &core::ObserverApplyVerifyAndChairmanShadowRunConfig {
            run_id: "sprint188-apply-verify-shadow".to_string(),
            enabled: true,
            apply_verification_config_path: None,
            apply_mode,
            dry_run,
            target_store_output_path: Some(store_path.to_string_lossy().to_string()),
            output_path: Some(output_path.to_string_lossy().to_string()),
            run_chairman_shadow_governance,
            emit_owner_summary: true,
            paper_only: true,
        },
    )
    .expect("apply verify and shadow run");
    (
        batch_result,
        observer_run_result,
        run_result,
        store_path,
        output_path,
    )
}

#[test]
fn sprint170_schema_validation_and_member_aliases_fail_closed() {
    let mut input_schema = default_adapter_input_schema_v1();
    assert_eq!(
        validate_adapter_input_schema_v1(&input_schema).validation_status,
        AdapterSchemaValidationStatus::Valid
    );
    assert!(
        input_schema
            .required_feature_groups
            .contains(&AdapterInputFeatureGroup::Market)
    );
    input_schema.leakage_policy.forbid_target_labels = false;
    input_schema.leakage_policy.forbid_outcome_labels = false;
    input_schema.leakage_policy.forbid_broker_order_account = false;
    assert_eq!(
        validate_adapter_input_schema_v1(&input_schema).validation_status,
        AdapterSchemaValidationStatus::Invalid
    );

    let mut output_schema = default_adapter_output_schema_v1();
    assert_eq!(
        validate_adapter_output_schema_v1(&output_schema).validation_status,
        AdapterSchemaValidationStatus::Valid
    );
    for invalid_policy in [
        AdapterOutputValuePolicy::ContainsLogits,
        AdapterOutputValuePolicy::ContainsProbabilities,
        AdapterOutputValuePolicy::ContainsPredictions,
        AdapterOutputValuePolicy::ContainsModelScores,
        AdapterOutputValuePolicy::ContainsInferenceValues,
    ] {
        output_schema.value_policy = invalid_policy;
        assert_eq!(
            validate_adapter_output_schema_v1(&output_schema).validation_status,
            AdapterSchemaValidationStatus::Invalid
        );
    }
    output_schema.value_policy = AdapterOutputValuePolicy::ShapeOnlyNoValues;
    assert!(
        output_schema
            .required_heads
            .contains(&AdapterOutputHeadGroup::Stance)
    );
    assert!(
        output_schema
            .optional_heads
            .contains(&AdapterOutputHeadGroup::ExpectedReturnHint)
    );

    let canonical_map = default_three_member_canonical_id_map();
    assert_eq!(canonical_map.entries.len(), 3);
    assert_eq!(
        resolve_canonical_member_id("TrendEntryAI").as_deref(),
        Some("trend-kr-short")
    );
    assert_eq!(
        resolve_canonical_member_id("RiskGuardAI").as_deref(),
        Some("risk-kr-short")
    );
    assert_eq!(
        resolve_canonical_member_id("EvidenceRegimeAI").as_deref(),
        Some("evidence-kr-short")
    );
    assert!(resolve_canonical_member_id("UnknownPilotAI").is_none());
    assert!(validate_member_id_mapping("TrendEntryAI", IndependentMemberRole::TrendEntry).valid);
    assert!(!validate_member_id_mapping("UnknownPilotAI", IndependentMemberRole::TrendEntry).valid);
}

#[test]
fn sprint170_member_contract_and_batch_contract_reject_invalid_data() {
    let (_members, dataset, split, batches, registry, input_schema, output_schema, contracts) =
        sprint170_contract_inputs();
    let trend_profile = registry
        .get_profile("trend-kr-short")
        .expect("trend profile");
    let trend_contract = contracts
        .iter()
        .find(|contract| contract.canonical_member_id == "trend-kr-short")
        .expect("trend contract");

    let mut review_dataset = dataset.clone();
    review_dataset.examples[0].member_id = "TrendEntryAI".to_string();
    review_dataset.examples[0].label_source = ReplayLabelSource::ReviewRequired;
    assert_eq!(
        validate_member_learning_data_contract(
            trend_contract,
            &review_dataset,
            &split,
            trend_profile
        )
        .validation_status,
        MemberLearningDataContractValidationStatus::Invalid
    );

    let mut simulated_dataset = dataset.clone();
    simulated_dataset.examples[0].member_id = "TrendEntryAI".to_string();
    simulated_dataset.examples[0].label_source = ReplayLabelSource::SimulatedFixture;
    assert_eq!(
        validate_member_learning_data_contract(
            trend_contract,
            &simulated_dataset,
            &split,
            trend_profile
        )
        .validation_status,
        MemberLearningDataContractValidationStatus::Invalid
    );

    let mut leaked_dataset = dataset.clone();
    leaked_dataset.examples[0].member_id = "TrendEntryAI".to_string();
    leaked_dataset.examples[0]
        .sanitized_input_features
        .market_data_summary = "future outcome target label copied into adapter input".to_string();
    let leaked_result = validate_member_learning_data_contract(
        trend_contract,
        &leaked_dataset,
        &split,
        trend_profile,
    );
    assert_eq!(
        leaked_result.validation_status,
        MemberLearningDataContractValidationStatus::Invalid
    );
    assert!(
        leaked_result
            .schema_violations
            .iter()
            .any(|violation| violation.contains("input leakage"))
    );

    let summary =
        summarize_member_learning_data_contracts(&[validate_member_learning_data_contract(
            trend_contract,
            &dataset,
            &split,
            trend_profile,
        )]);
    assert!(matches!(
        summary.status,
        MemberLearningDataContractSummaryStatus::Valid
            | MemberLearningDataContractSummaryStatus::ValidWithWarnings
    ));

    let first_batch = batches
        .iter()
        .find(|batch| {
            resolve_canonical_member_id(&batch.member_id).as_deref() == Some("trend-kr-short")
        })
        .expect("trend batch");
    let mut alias_batch = first_batch.clone();
    alias_batch.member_id = "TrendEntryAI".to_string();
    for row in &mut alias_batch.feature_rows {
        row.member_id = "TrendEntryAI".to_string();
    }
    let alias_validation = validate_batch_against_adapter_contract_v2(
        &alias_batch,
        trend_profile,
        &input_schema,
        &output_schema,
        trend_contract,
        &default_three_member_canonical_id_map(),
    );
    assert!(matches!(
        alias_validation.validation_status,
        AdapterContractValidationStatus::Valid | AdapterContractValidationStatus::ValidWithWarnings
    ));

    let other_batch = batches
        .iter()
        .find(|batch| batch.member_id != first_batch.member_id)
        .expect("other member batch");
    let mut mixed_batch = first_batch.clone();
    mixed_batch
        .example_ids
        .push(other_batch.example_ids[0].clone());
    mixed_batch
        .feature_rows
        .push(other_batch.feature_rows[0].clone());
    mixed_batch
        .target_rows
        .push(other_batch.target_rows[0].clone());
    mixed_batch.batch_size = mixed_batch.feature_rows.len();
    let mixed_validation = validate_batch_against_adapter_contract_v2(
        &mixed_batch,
        trend_profile,
        &input_schema,
        &output_schema,
        trend_contract,
        &default_three_member_canonical_id_map(),
    );
    assert_eq!(
        mixed_validation.validation_status,
        AdapterContractValidationStatus::Invalid
    );

    let mut unmatched_batch = alias_batch.clone();
    unmatched_batch.member_id = "UnknownPilotAI".to_string();
    let unmatched_validation = validate_batch_against_adapter_contract_v2(
        &unmatched_batch,
        trend_profile,
        &input_schema,
        &output_schema,
        trend_contract,
        &default_three_member_canonical_id_map(),
    );
    assert!(unmatched_validation.unmatched_batch);
    assert_eq!(
        unmatched_validation.validation_status,
        AdapterContractValidationStatus::Invalid
    );

    let mut unknown_row_alias_batch = alias_batch.clone();
    unknown_row_alias_batch.feature_rows[0].member_id = "UnknownPilotAI".to_string();
    unknown_row_alias_batch.target_rows[0].member_id = "UnknownPilotAI".to_string();
    let unknown_row_alias_validation = validate_batch_against_adapter_contract_v2(
        &unknown_row_alias_batch,
        trend_profile,
        &input_schema,
        &output_schema,
        trend_contract,
        &default_three_member_canonical_id_map(),
    );
    assert_eq!(
        unknown_row_alias_validation.validation_status,
        AdapterContractValidationStatus::Invalid
    );
    assert!(
        unknown_row_alias_validation
            .warnings
            .iter()
            .any(|warning| warning.contains("unknown row member aliases"))
    );
}

#[test]
fn sprint170_golden_snapshots_and_drift_guard_are_deterministic() {
    let (_members, _dataset, _split, batches, registry, input_schema, output_schema, contracts) =
        sprint170_contract_inputs();
    let schemas = soma_zero::league::minimal_ai_committee_core::AdapterContractSchemasV1 {
        input_schema,
        output_schema,
    };
    let first = build_adapter_shape_golden_snapshots(&batches, &registry, &schemas, &contracts);
    let second = build_adapter_shape_golden_snapshots(&batches, &registry, &schemas, &contracts);
    assert_eq!(first, second);
    assert_eq!(first.snapshot_count, first.snapshots.len());

    let mut changed = first.clone();
    changed.snapshots[0].output_shape.stance_output_dim += 1;
    let diff = compare_adapter_shape_golden_snapshots(&changed, &first);
    assert_eq!(
        diff.diff_status,
        soma_zero::league::minimal_ai_committee_core::AdapterShapeGoldenSnapshotDiffStatus::ExpectedDiff
    );
    assert_eq!(diff.dimension_changes.len(), 1);

    let drift = evaluate_adapter_schema_drift(&changed, Some(&first));
    assert_eq!(
        drift.drift_status,
        AdapterSchemaDriftStatus::UnexpectedDrift
    );
    assert!(drift.requires_schema_version_bump);

    let mut bumped = changed.clone();
    bumped.schema_version = "V2".to_string();
    let bumped_drift = evaluate_adapter_schema_drift(&bumped, Some(&first));
    assert_eq!(
        bumped_drift.drift_status,
        AdapterSchemaDriftStatus::DriftAllowedWithVersionBump
    );

    let missing_baseline = evaluate_adapter_schema_drift(&first, None);
    assert_eq!(
        missing_baseline.drift_status,
        AdapterSchemaDriftStatus::MissingBaseline
    );
}

#[test]
fn sprint170_contract_lock_run_is_deterministic_and_preserves_safety() {
    let (_members, dataset, split, batches, registry, _input_schema, _output_schema, _contracts) =
        sprint170_contract_inputs();
    let config = AdapterContractLockRunConfig {
        run_id: "sprint170-lock".to_string(),
        adapter_contract_lock_enabled: true,
        write_golden_snapshot_path: None,
        expected_golden_snapshot_path: None,
        require_schema_version_match: true,
        fail_on_unmatched_batch: true,
        fail_on_unknown_member_alias: true,
        fail_on_output_values: true,
        expected_golden_baseline_path: None,
        bootstrap_golden_baseline_path: None,
        bootstrap_missing_baseline: false,
        write_golden_baseline_if_missing: false,
        allow_schema_version_bump: false,
        run_regression_harness: false,
        fail_on_missing_baseline: true,
        paper_only: true,
    };
    let first = run_adapter_contract_lock(&batches, &registry, &dataset, &split, &config)
        .expect("lock run");
    let second = run_adapter_contract_lock(&batches, &registry, &dataset, &split, &config)
        .expect("lock run");
    assert_eq!(first, second);
    assert!(matches!(
        first.lock_status,
        AdapterContractLockStatus::Locked | AdapterContractLockStatus::LockedWithWarnings
    ));
    assert_eq!(first.schema_version, "V1");
    assert_eq!(
        first.batch_contract_validation_count,
        first.batch_contract_validations.len()
    );
    assert_eq!(
        first.golden_snapshot_count,
        first.golden_snapshot_set.snapshot_count
    );
    assert_eq!(
        first.adapter_safety_guard.safety_status,
        TrainingSimulationSafetyStatus::Preserved
    );
    assert!(!first.adapter_safety_guard.forward_method_detected);
    assert!(!first.adapter_safety_guard.runtime_ready_detected);
    assert!(!first.adapter_safety_guard.training_ready_detected);
    assert!(!first.adapter_safety_guard.broker_order_account_detected);

    let mut unknown_alias_batches = batches.clone();
    unknown_alias_batches[0].member_id = "UnknownPilotAI".to_string();
    let failed =
        run_adapter_contract_lock(&unknown_alias_batches, &registry, &dataset, &split, &config)
            .expect("failed lock run");
    assert_eq!(failed.lock_status, AdapterContractLockStatus::Failed);
}

#[test]
fn sprint171_baseline_file_round_trip_and_validation_fail_closed() {
    let (_members, _dataset, _split, batches, registry, input_schema, output_schema, contracts) =
        sprint170_contract_inputs();
    let schemas = soma_zero::league::minimal_ai_committee_core::AdapterContractSchemasV1 {
        input_schema,
        output_schema,
    };
    let snapshot_set =
        build_adapter_shape_golden_snapshots(&batches, &registry, &schemas, &contracts);
    assert_eq!(snapshot_set.snapshot_count, 3);
    let baseline = AdapterGoldenSnapshotBaselineFile {
        schema_version: "V1".to_string(),
        baseline_id: "sprint171-baseline".to_string(),
        created_from_run_id: Some("sprint171".to_string()),
        snapshot_set: snapshot_set.clone(),
        baseline_policy: AdapterGoldenBaselinePolicy::default(),
        paper_only: true,
    };
    assert!(validate_adapter_golden_snapshot_baseline(&baseline).is_ok());

    let path = sprint171_temp_json_path("adapter-baseline-roundtrip");
    save_adapter_golden_snapshot_baseline_to_local_json(&path, &baseline).expect("save baseline");
    let loaded =
        load_adapter_golden_snapshot_baseline_from_local_json(&path).expect("load baseline");
    assert_eq!(loaded, baseline);
    assert!(
        load_adapter_golden_snapshot_baseline_from_local_json(std::path::Path::new(
            "https://example.com/baseline.json"
        ))
        .is_err()
    );
    assert!(
        save_adapter_golden_snapshot_baseline_to_local_json(
            std::path::Path::new("../bad-baseline.json"),
            &baseline,
        )
        .is_err()
    );

    let mut duplicate_snapshot = baseline.clone();
    duplicate_snapshot.snapshot_set.snapshots[1].snapshot_id =
        duplicate_snapshot.snapshot_set.snapshots[0]
            .snapshot_id
            .clone();
    assert!(validate_adapter_golden_snapshot_baseline(&duplicate_snapshot).is_err());

    let mut output_value_leak = baseline.clone();
    output_value_leak.snapshot_set.snapshots[0].member_contract_summary =
        "logits leaked".to_string();
    assert!(validate_adapter_golden_snapshot_baseline(&output_value_leak).is_err());

    let _ = fs::remove_file(path);
}

#[test]
fn sprint171_bootstrap_comparison_and_acceptance_gate_behave_as_expected() {
    let (_members, dataset, split, batches, registry, input_schema, output_schema, contracts) =
        sprint170_contract_inputs();
    let schemas = soma_zero::league::minimal_ai_committee_core::AdapterContractSchemasV1 {
        input_schema,
        output_schema,
    };
    let snapshot_set =
        build_adapter_shape_golden_snapshots(&batches, &registry, &schemas, &contracts);
    let baseline_policy = AdapterGoldenBaselinePolicy::default();
    let bootstrap_path = sprint171_temp_json_path("adapter-baseline-bootstrap");
    let bootstrap = bootstrap_adapter_golden_baseline(&AdapterGoldenBaselineBootstrapConfig {
        bootstrap_id: "sprint171-bootstrap".to_string(),
        output_baseline_path: bootstrap_path.to_string_lossy().to_string(),
        source_snapshot_set: snapshot_set.clone(),
        allow_overwrite: false,
        require_clean_contract_lock: true,
        baseline_policy: baseline_policy.clone(),
        created_from_run_id: Some("sprint171".to_string()),
        paper_only: true,
    })
    .expect("bootstrap baseline");
    assert_eq!(
        bootstrap.validation_status,
        AdapterGoldenBaselineValidationStatus::Valid
    );
    assert_eq!(
        bootstrap.bootstrap_status,
        AdapterGoldenBaselineBootstrapStatus::WroteBaseline
    );
    let skipped = bootstrap_adapter_golden_baseline(&AdapterGoldenBaselineBootstrapConfig {
        bootstrap_id: "sprint171-bootstrap".to_string(),
        output_baseline_path: bootstrap_path.to_string_lossy().to_string(),
        source_snapshot_set: snapshot_set.clone(),
        allow_overwrite: false,
        require_clean_contract_lock: true,
        baseline_policy: baseline_policy.clone(),
        created_from_run_id: Some("sprint171".to_string()),
        paper_only: true,
    })
    .expect("skip existing baseline");
    assert_eq!(
        skipped.bootstrap_status,
        AdapterGoldenBaselineBootstrapStatus::SkippedExisting
    );
    let loaded = load_adapter_golden_snapshot_baseline_from_local_json(&bootstrap_path)
        .expect("load baseline");
    let no_diff = compare_current_snapshot_to_expected_baseline(
        &snapshot_set,
        Some(&loaded),
        &baseline_policy,
    );
    assert_eq!(no_diff.diff_status, AdapterGoldenBaselineDiffStatus::NoDiff);

    let mut changed = snapshot_set.clone();
    changed.snapshots[0].output_shape.stance_output_dim += 1;
    let unexpected =
        compare_current_snapshot_to_expected_baseline(&changed, Some(&loaded), &baseline_policy);
    assert_eq!(
        unexpected.diff_status,
        AdapterGoldenBaselineDiffStatus::UnexpectedDrift
    );

    let mut member_removed = snapshot_set.clone();
    member_removed.snapshots.pop();
    member_removed.snapshot_count = member_removed.snapshots.len();
    let member_set_drift = compare_current_snapshot_to_expected_baseline(
        &member_removed,
        Some(&loaded),
        &baseline_policy,
    );
    assert_eq!(
        member_set_drift.diff_status,
        AdapterGoldenBaselineDiffStatus::UnexpectedDrift
    );
    assert!(!member_set_drift.member_set_diffs.is_empty());

    let mut batch_changed = snapshot_set.clone();
    batch_changed.snapshots[0].batch_id = "unexpected-contract-batch".to_string();
    let batch_set_drift = compare_current_snapshot_to_expected_baseline(
        &batch_changed,
        Some(&loaded),
        &baseline_policy,
    );
    assert_eq!(
        batch_set_drift.diff_status,
        AdapterGoldenBaselineDiffStatus::UnexpectedDrift
    );
    assert!(!batch_set_drift.batch_set_diffs.is_empty());

    let mut input_group_changed = snapshot_set.clone();
    input_group_changed.snapshots[0]
        .input_shape
        .news_feature_dim += 1;
    input_group_changed.snapshots[0].input_shape.feature_dim += 1;
    let input_group_drift = compare_current_snapshot_to_expected_baseline(
        &input_group_changed,
        Some(&loaded),
        &baseline_policy,
    );
    assert_eq!(
        input_group_drift.diff_status,
        AdapterGoldenBaselineDiffStatus::UnexpectedDrift
    );
    assert!(!input_group_drift.input_group_diffs.is_empty());

    let mut output_head_changed = snapshot_set.clone();
    output_head_changed.snapshots[0]
        .output_shape
        .enabled_heads
        .pop();
    let output_head_drift = compare_current_snapshot_to_expected_baseline(
        &output_head_changed,
        Some(&loaded),
        &baseline_policy,
    );
    assert_eq!(
        output_head_drift.diff_status,
        AdapterGoldenBaselineDiffStatus::UnexpectedDrift
    );
    assert!(!output_head_drift.output_head_diffs.is_empty());

    let mut bumped = changed.clone();
    bumped.schema_version = "V2".to_string();
    bumped.snapshots[0].schema_version = "V2".to_string();
    let mut bumped_policy = baseline_policy.clone();
    bumped_policy.allow_schema_version_bump = true;
    let versioned =
        compare_current_snapshot_to_expected_baseline(&bumped, Some(&loaded), &bumped_policy);
    assert_eq!(
        versioned.diff_status,
        AdapterGoldenBaselineDiffStatus::ExpectedVersionedDiff
    );

    let config = AdapterContractLockRunConfig {
        run_id: "sprint171-lock".to_string(),
        adapter_contract_lock_enabled: true,
        write_golden_snapshot_path: None,
        expected_golden_snapshot_path: None,
        require_schema_version_match: true,
        fail_on_unmatched_batch: true,
        fail_on_unknown_member_alias: true,
        fail_on_output_values: true,
        expected_golden_baseline_path: Some(bootstrap_path.to_string_lossy().to_string()),
        bootstrap_golden_baseline_path: None,
        bootstrap_missing_baseline: false,
        write_golden_baseline_if_missing: false,
        allow_schema_version_bump: false,
        run_regression_harness: false,
        fail_on_missing_baseline: true,
        paper_only: true,
    };
    let locked = run_adapter_contract_lock_v2(&batches, &registry, &dataset, &split, &config)
        .expect("locked run");
    assert_eq!(
        locked.baseline_comparison_result.diff_status,
        AdapterGoldenBaselineDiffStatus::NoDiff
    );
    assert!(locked.contract_lock_acceptance_gate.lock_accepted);
    assert_eq!(
        locked.contract_lock_acceptance_gate.gate_status,
        AdapterContractLockAcceptanceGateStatus::Locked
    );

    let mut missing = config.clone();
    missing.expected_golden_baseline_path = None;
    let blocked = run_adapter_contract_lock_v2(&batches, &registry, &dataset, &split, &missing)
        .expect("blocked missing baseline");
    assert_eq!(
        blocked.contract_lock_acceptance_gate.gate_status,
        AdapterContractLockAcceptanceGateStatus::BlockedByMissingBaseline
    );

    let mut bootstrap_mode = missing.clone();
    bootstrap_mode.bootstrap_missing_baseline = true;
    let warning =
        run_adapter_contract_lock_v2(&batches, &registry, &dataset, &split, &bootstrap_mode)
            .expect("bootstrap warning");
    assert_eq!(
        warning.contract_lock_acceptance_gate.gate_status,
        AdapterContractLockAcceptanceGateStatus::LockedWithWarnings
    );

    let failed_bootstrap_path = sprint171_temp_json_path("adapter-baseline-failed-bootstrap");
    let mut failed_bootstrap_config = missing.clone();
    failed_bootstrap_config.bootstrap_missing_baseline = true;
    failed_bootstrap_config.write_golden_baseline_if_missing = true;
    failed_bootstrap_config.bootstrap_golden_baseline_path =
        Some(failed_bootstrap_path.to_string_lossy().to_string());
    let mut unknown_alias_batches = batches.clone();
    unknown_alias_batches[0].member_id = "UnknownPilotAI".to_string();
    let failed_bootstrap = run_adapter_contract_lock_v2(
        &unknown_alias_batches,
        &registry,
        &dataset,
        &split,
        &failed_bootstrap_config,
    )
    .expect("failed bootstrap run");
    assert_eq!(
        failed_bootstrap.lock_status,
        AdapterContractLockStatus::Failed
    );
    assert_eq!(
        failed_bootstrap
            .baseline_bootstrap_result
            .as_ref()
            .expect("bootstrap result")
            .bootstrap_status,
        AdapterGoldenBaselineBootstrapStatus::Blocked
    );
    assert!(!failed_bootstrap_path.exists());

    let _ = fs::remove_file(bootstrap_path);
    let _ = fs::remove_file(failed_bootstrap_path);
}

#[test]
fn sprint171_v2_lock_and_regression_harness_are_deterministic() {
    let (_members, dataset, split, batches, registry, input_schema, output_schema, contracts) =
        sprint170_contract_inputs();
    let schemas = soma_zero::league::minimal_ai_committee_core::AdapterContractSchemasV1 {
        input_schema,
        output_schema,
    };
    let snapshot_set =
        build_adapter_shape_golden_snapshots(&batches, &registry, &schemas, &contracts);
    let baseline = AdapterGoldenSnapshotBaselineFile {
        schema_version: "V1".to_string(),
        baseline_id: "sprint171-deterministic-baseline".to_string(),
        created_from_run_id: Some("sprint171".to_string()),
        snapshot_set,
        baseline_policy: AdapterGoldenBaselinePolicy::default(),
        paper_only: true,
    };
    let baseline_path = sprint171_temp_json_path("adapter-baseline-deterministic");
    save_adapter_golden_snapshot_baseline_to_local_json(&baseline_path, &baseline)
        .expect("save deterministic baseline");

    let config = AdapterContractLockRunConfig {
        run_id: "sprint171-v2-lock".to_string(),
        adapter_contract_lock_enabled: true,
        write_golden_snapshot_path: None,
        expected_golden_snapshot_path: None,
        require_schema_version_match: true,
        fail_on_unmatched_batch: true,
        fail_on_unknown_member_alias: true,
        fail_on_output_values: true,
        expected_golden_baseline_path: Some(baseline_path.to_string_lossy().to_string()),
        bootstrap_golden_baseline_path: None,
        bootstrap_missing_baseline: false,
        write_golden_baseline_if_missing: false,
        allow_schema_version_bump: false,
        run_regression_harness: true,
        fail_on_missing_baseline: true,
        paper_only: true,
    };
    let first = run_adapter_contract_lock_v2(&batches, &registry, &dataset, &split, &config)
        .expect("first");
    let second = run_adapter_contract_lock_v2(&batches, &registry, &dataset, &split, &config)
        .expect("second");
    assert_eq!(first, second);
    assert_eq!(
        first
            .loaded_expected_baseline
            .as_ref()
            .expect("loaded expected baseline")
            .baseline_id,
        "sprint171-deterministic-baseline"
    );
    assert_eq!(
        first.baseline_comparison_result.diff_status,
        AdapterGoldenBaselineDiffStatus::NoDiff
    );
    assert_eq!(
        first.contract_lock_acceptance_gate.gate_status,
        AdapterContractLockAcceptanceGateStatus::Locked
    );
    assert_eq!(
        first
            .regression_harness_result
            .as_ref()
            .expect("regression harness")
            .harness_status,
        AdapterContractRegressionHarnessStatus::Passed
    );
    let harness = first
        .regression_harness_result
        .as_ref()
        .expect("regression harness");
    assert!(harness.case_results.iter().any(|case| {
        case.case_id == "input-dimension-change"
            && case.actual_status == AdapterContractLockAcceptanceGateStatus::BlockedBySchemaDrift
            && case.passed
    }));

    let _ = fs::remove_file(baseline_path);
}

fn sprint172_locked_runtime_entry_inputs() -> (
    AdapterContractLockRunResult,
    soma_zero::league::minimal_ai_committee_core::SmartCoreAdapterRegistry,
    Vec<SmartCoreV2TrainingBatch>,
    PathBuf,
) {
    let (_members, dataset, split, batches, registry, input_schema, output_schema, contracts) =
        sprint170_contract_inputs();
    let schemas = soma_zero::league::minimal_ai_committee_core::AdapterContractSchemasV1 {
        input_schema,
        output_schema,
    };
    let snapshot_set =
        build_adapter_shape_golden_snapshots(&batches, &registry, &schemas, &contracts);
    let baseline = AdapterGoldenSnapshotBaselineFile {
        schema_version: "V1".to_string(),
        baseline_id: "sprint172-runtime-baseline".to_string(),
        created_from_run_id: Some("sprint172".to_string()),
        snapshot_set,
        baseline_policy: AdapterGoldenBaselinePolicy::default(),
        paper_only: true,
    };
    let baseline_path = sprint171_temp_json_path("adapter-runtime-entry-baseline");
    save_adapter_golden_snapshot_baseline_to_local_json(&baseline_path, &baseline)
        .expect("save runtime entry baseline");
    let lock_run = run_adapter_contract_lock_v2(
        &batches,
        &registry,
        &dataset,
        &split,
        &AdapterContractLockRunConfig {
            run_id: "sprint172-contract-lock".to_string(),
            adapter_contract_lock_enabled: true,
            write_golden_snapshot_path: None,
            expected_golden_snapshot_path: None,
            require_schema_version_match: true,
            fail_on_unmatched_batch: true,
            fail_on_unknown_member_alias: true,
            fail_on_output_values: true,
            expected_golden_baseline_path: Some(baseline_path.to_string_lossy().to_string()),
            bootstrap_golden_baseline_path: None,
            bootstrap_missing_baseline: false,
            write_golden_baseline_if_missing: false,
            allow_schema_version_bump: false,
            run_regression_harness: true,
            fail_on_missing_baseline: true,
            paper_only: true,
        },
    )
    .expect("sprint172 contract lock");
    (lock_run, registry, batches, baseline_path)
}

#[test]
fn sprint172_runtime_entry_gate_grants_shape_only_and_is_deterministic() {
    let (lock_run, registry, batches, baseline_path) = sprint172_locked_runtime_entry_inputs();
    let audit_path = sprint171_temp_json_path("runtime-entry-audit");
    let config = RuntimeAdapterEntryGateRunConfig {
        run_id: "sprint172-runtime-entry-gate".to_string(),
        runtime_entry_gate_enabled: true,
        requested_capabilities: vec![
            SmartCoreRuntimeCapability::ShapeValidation,
            SmartCoreRuntimeCapability::BuildInputShape,
            SmartCoreRuntimeCapability::BuildOutputShape,
            SmartCoreRuntimeCapability::ValidateAdapterContract,
            SmartCoreRuntimeCapability::ValidateGoldenBaseline,
        ],
        runtime_entry_audit_output_path: Some(audit_path.to_string_lossy().to_string()),
        allow_shape_only_requests: true,
        fail_on_forbidden_capability: true,
        fail_on_contract_not_locked: true,
        fail_on_baseline_drift: true,
        fail_on_safety_violation: true,
        microkernel_lab_mode_policy: None,
        paper_only: true,
    };
    let first = run_runtime_adapter_entry_gate(&registry, &batches, &lock_run, &config)
        .expect("first runtime entry gate");
    let second = run_runtime_adapter_entry_gate(&registry, &batches, &lock_run, &config)
        .expect("second runtime entry gate");
    assert_eq!(first, second);
    assert_eq!(
        first.gate_status,
        RuntimeAdapterEntryGateStatus::ShapeOnlyGranted
    );
    assert_eq!(
        first.policy.allowed_capabilities,
        vec![
            SmartCoreRuntimeCapability::ShapeValidation,
            SmartCoreRuntimeCapability::BuildInputShape,
            SmartCoreRuntimeCapability::BuildOutputShape,
            SmartCoreRuntimeCapability::ValidateAdapterContract,
            SmartCoreRuntimeCapability::ValidateGoldenBaseline,
        ]
    );
    assert!(
        first
            .policy
            .forbidden_capabilities
            .contains(&SmartCoreRuntimeCapability::RuntimeForward)
    );
    assert_eq!(
        first.bridge.bridge_status,
        ShapeContractEnforcementBridgeStatus::Ready
    );
    assert!(first.decisions.iter().all(|decision| {
        decision.decision == RuntimeAdapterEntryDecisionKind::GrantedShapeOnly
    }));
    assert_eq!(
        first.audit_log.granted_shape_only_count,
        registry.profile_count
    );
    assert!(!first.runtime_capabilities_granted);
    assert!(!first.training_capabilities_granted);
    assert!(!first.weight_capabilities_granted);
    assert!(!first.checkpoint_capabilities_granted);
    assert!(!first.live_inference_granted);
    assert!(!first.broker_order_account_granted);
    let saved_audit =
        RuntimeAdapterEntryAuditLog::load_from_local_json(&audit_path).expect("load audit log");
    assert_eq!(saved_audit, first.audit_log);

    let negative_harness = run_runtime_escalation_negative_harness(&first.policy, &lock_run);
    assert_eq!(
        negative_harness.harness_status,
        RuntimeEscalationHarnessStatus::Passed
    );
    assert!(negative_harness.case_results.iter().any(|case| {
        case.requested_capability == SmartCoreRuntimeCapability::Unknown
            && case.actual_decision == RuntimeAdapterEntryDecisionKind::DeniedForbiddenCapability
            && case.denied
    }));
    let readiness = summarize_adapter_runtime_entry_readiness(&first, Some(&negative_harness));
    assert_eq!(
        readiness.readiness_status,
        AdapterRuntimeEntryReadinessStatus::ShapeOnlyEntryReady
    );
    assert_eq!(
        readiness.next_allowed_step,
        AdapterRuntimeEntryNextAllowedStep::ShapeOnlyMockAdapterOutput
    );
    assert!(readiness.runtime_capabilities_denied > 0);
    assert!(readiness.training_capabilities_denied > 0);
    assert!(readiness.weight_capabilities_denied > 0);
    assert!(readiness.checkpoint_capabilities_denied > 0);
    assert!(readiness.live_inference_denied > 0);
    assert!(readiness.broker_order_account_denied > 0);

    let _ = fs::remove_file(audit_path);
    let _ = fs::remove_file(baseline_path);
}

#[test]
fn sprint172_runtime_entry_gate_denies_forbidden_and_unknown_capabilities() {
    let (lock_run, registry, batches, baseline_path) = sprint172_locked_runtime_entry_inputs();
    let forbidden = run_runtime_adapter_entry_gate(
        &registry,
        &batches,
        &lock_run,
        &RuntimeAdapterEntryGateRunConfig {
            run_id: "sprint172-runtime-entry-forbidden".to_string(),
            runtime_entry_gate_enabled: true,
            requested_capabilities: vec![SmartCoreRuntimeCapability::RuntimeForward],
            runtime_entry_audit_output_path: None,
            allow_shape_only_requests: true,
            fail_on_forbidden_capability: true,
            fail_on_contract_not_locked: true,
            fail_on_baseline_drift: true,
            fail_on_safety_violation: true,
            microkernel_lab_mode_policy: None,
            paper_only: true,
        },
    )
    .expect("forbidden gate");
    assert_eq!(
        forbidden.gate_status,
        RuntimeAdapterEntryGateStatus::Blocked
    );
    assert!(forbidden.decisions.iter().all(|decision| {
        decision.decision == RuntimeAdapterEntryDecisionKind::DeniedForbiddenCapability
            && decision
                .denied_capabilities
                .contains(&SmartCoreRuntimeCapability::RuntimeForward)
    }));
    assert!(forbidden.audit_log.records.iter().all(|record| {
        record.decision == RuntimeAdapterEntryDecisionKind::DeniedForbiddenCapability
            && record
                .denied_capabilities
                .contains(&SmartCoreRuntimeCapability::RuntimeForward)
    }));

    let unknown = run_runtime_adapter_entry_gate(
        &registry,
        &batches,
        &lock_run,
        &RuntimeAdapterEntryGateRunConfig {
            run_id: "sprint172-runtime-entry-unknown".to_string(),
            runtime_entry_gate_enabled: true,
            requested_capabilities: vec![SmartCoreRuntimeCapability::Unknown],
            runtime_entry_audit_output_path: None,
            allow_shape_only_requests: true,
            fail_on_forbidden_capability: true,
            fail_on_contract_not_locked: true,
            fail_on_baseline_drift: true,
            fail_on_safety_violation: true,
            microkernel_lab_mode_policy: None,
            paper_only: true,
        },
    )
    .expect("unknown gate");
    assert_eq!(unknown.gate_status, RuntimeAdapterEntryGateStatus::Blocked);
    assert!(unknown.decisions.iter().all(|decision| {
        decision.decision == RuntimeAdapterEntryDecisionKind::DeniedForbiddenCapability
            && decision
                .denied_capabilities
                .contains(&SmartCoreRuntimeCapability::Unknown)
    }));

    let _ = fs::remove_file(baseline_path);
}

#[test]
fn sprint172_runtime_entry_gate_blocks_on_contract_baseline_and_safety_failures() {
    let (lock_run, registry, batches, baseline_path) = sprint172_locked_runtime_entry_inputs();
    let config = RuntimeAdapterEntryGateRunConfig {
        run_id: "sprint172-runtime-entry-blocked".to_string(),
        runtime_entry_gate_enabled: true,
        requested_capabilities: vec![SmartCoreRuntimeCapability::ValidateGoldenBaseline],
        runtime_entry_audit_output_path: None,
        allow_shape_only_requests: true,
        fail_on_forbidden_capability: true,
        fail_on_contract_not_locked: true,
        fail_on_baseline_drift: true,
        fail_on_safety_violation: true,
        microkernel_lab_mode_policy: None,
        paper_only: true,
    };

    let mut contract_failed = lock_run.clone();
    contract_failed.lock_status = AdapterContractLockStatus::Failed;
    let contract_blocked =
        run_runtime_adapter_entry_gate(&registry, &batches, &contract_failed, &config)
            .expect("contract blocked");
    assert_eq!(
        contract_blocked.gate_status,
        RuntimeAdapterEntryGateStatus::Blocked
    );
    assert!(contract_blocked.decisions.iter().all(|decision| {
        decision.decision == RuntimeAdapterEntryDecisionKind::DeniedContractNotLocked
    }));

    let mut baseline_drift = lock_run.clone();
    baseline_drift.baseline_comparison_result.diff_status =
        AdapterGoldenBaselineDiffStatus::UnexpectedDrift;
    let baseline_blocked =
        run_runtime_adapter_entry_gate(&registry, &batches, &baseline_drift, &config)
            .expect("baseline blocked");
    assert_eq!(
        baseline_blocked.gate_status,
        RuntimeAdapterEntryGateStatus::Blocked
    );
    assert!(baseline_blocked.decisions.iter().all(|decision| {
        decision.decision == RuntimeAdapterEntryDecisionKind::DeniedBaselineDrift
    }));

    let mut safety_failed = lock_run.clone();
    safety_failed.adapter_safety_guard.safety_status = TrainingSimulationSafetyStatus::Violated;
    let safety_blocked =
        run_runtime_adapter_entry_gate(&registry, &batches, &safety_failed, &config)
            .expect("safety blocked");
    assert_eq!(
        safety_blocked.gate_status,
        RuntimeAdapterEntryGateStatus::Blocked
    );
    assert!(safety_blocked.decisions.iter().all(|decision| {
        decision.decision == RuntimeAdapterEntryDecisionKind::DeniedSafetyViolation
    }));
    let readiness = summarize_adapter_runtime_entry_readiness(&safety_blocked, None);
    assert_eq!(
        readiness.readiness_status,
        AdapterRuntimeEntryReadinessStatus::SafetyBlocked
    );
    assert_eq!(
        readiness.next_allowed_step,
        AdapterRuntimeEntryNextAllowedStep::FixSafetyViolation
    );

    let _ = fs::remove_file(baseline_path);
}

#[test]
fn sprint173_tiny_tensor_and_member_cells_follow_core_contracts() {
    assert!(from_vec_1d(vec![f32::NAN]).is_err());
    assert!(from_vec_2d(1, 1, vec![f32::INFINITY]).is_err());

    let matrix = from_vec_2d(2, 3, vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6]).expect("matrix");
    let wrong_vector = from_vec_1d(vec![1.0, 2.0]).expect("wrong vector");
    assert!(matvec(&matrix, &wrong_vector).is_err());

    let mamba_config = Mamba3TemporalCellConfigV0 {
        input_dim: 3,
        state_dim: 4,
        output_dim: 2,
        selective_gate_enabled: true,
        causal: true,
        max_sequence_len: 3,
        parameter_init: TinyParameterInitPolicyV0::DeterministicTiny,
        paper_only: true,
    };
    let mamba_params =
        init_mamba3_temporal_cell_params_v0(&mamba_config, 17).expect("mamba params");
    assert!(!mamba_params.trainable);
    assert!(!mamba_params.persistent);
    let mamba_state = Mamba3TemporalCellStateV0 {
        state: zeros_1d(4),
        step_index: 0,
        paper_only: true,
    };
    let input = from_vec_1d(vec![0.1, -0.2, 0.3]).expect("input");
    let mamba_step =
        mamba3_temporal_cell_step_v0(&input, &mamba_state, &mamba_params, &mamba_config)
            .expect("mamba step");
    assert_eq!(mamba_step.output.dim, 2);
    assert!(mamba_step.output.is_finite());
    assert_eq!(mamba_step.next_state.step_index, 1);

    let mamba_sequence = vec![
        from_vec_1d(vec![0.1, 0.0, 0.2]).expect("seq0"),
        from_vec_1d(vec![0.2, 0.1, -0.1]).expect("seq1"),
        from_vec_1d(vec![-0.1, 0.3, 0.1]).expect("seq2"),
    ];
    let mamba_outputs =
        mamba3_temporal_sequence_forward_v0(&mamba_sequence, &mamba_params, &mamba_config)
            .expect("mamba sequence");
    assert_eq!(
        mamba_outputs
            .iter()
            .map(|output| output.next_state.step_index)
            .collect::<Vec<_>>(),
        vec![1, 2, 3]
    );

    let memory_config = GatedDeltaNetMemoryConfigV0 {
        input_dim: 3,
        memory_dim: 4,
        output_dim: 2,
        erase_gate_enabled: true,
        write_gate_enabled: true,
        max_steps: 3,
        parameter_init: TinyParameterInitPolicyV0::DeterministicTiny,
        paper_only: true,
    };
    let memory_params =
        init_gated_deltanet_memory_params_v0(&memory_config, 19).expect("memory params");
    assert!(!memory_params.trainable);
    assert!(!memory_params.persistent);
    let memory_state = GatedDeltaNetMemoryStateV0 {
        memory: zeros_1d(4),
        step_index: 0,
        paper_only: true,
    };
    let memory_step =
        gated_deltanet_memory_step_v0(&input, &memory_state, &memory_params, &memory_config)
            .expect("memory step");
    assert_eq!(memory_step.output.dim, 2);
    assert!(memory_step.output.is_finite());
    assert_eq!(memory_step.next_memory_state.step_index, 1);
    assert!(memory_step.next_memory_state.memory.is_finite());
    assert!(memory_state.memory.values.iter().all(|value| *value == 0.0));
}

#[test]
fn sprint173_smartcore_forward_and_safety_guard_fail_closed() {
    let config = SmartCoreMicroKernelConfigV0 {
        config_id: "sprint173-direct-forward".to_string(),
        member_id: "trend-entry-ai".to_string(),
        input_dim: 3,
        temporal_state_dim: 4,
        memory_dim: 4,
        hidden_dim: 4,
        output_dim: 6,
        sequence_len: 3,
        enabled_components: vec![
            SmartCoreMicroKernelComponentV0::Mamba3TemporalCellV0,
            SmartCoreMicroKernelComponentV0::GatedDeltaNetMemoryCellV0,
        ],
        sparse_event_attention_enabled: false,
        paper_only: true,
    };
    let params = init_smartcore_microkernel_params_v0(&config, 23).expect("smartcore params");
    assert!(!params.trainable);
    assert!(!params.persistent);
    assert!(!params.mamba3_params.trainable);
    assert!(!params.gated_deltanet_params.trainable);

    let state = SmartCoreMicroKernelStateV0 {
        temporal_state: Mamba3TemporalCellStateV0 {
            state: zeros_1d(config.temporal_state_dim),
            step_index: 0,
            paper_only: true,
        },
        memory_state: GatedDeltaNetMemoryStateV0 {
            memory: zeros_1d(config.memory_dim),
            step_index: 0,
            paper_only: true,
        },
        member_id: config.member_id.clone(),
        step_index: 0,
        paper_only: true,
    };
    let sequence = vec![
        from_vec_1d(vec![0.1, 0.0, 0.2]).expect("input0"),
        from_vec_1d(vec![0.2, 0.1, -0.1]).expect("input1"),
        from_vec_1d(vec![-0.1, 0.3, 0.1]).expect("input2"),
    ];
    let output = smartcore_microkernel_forward_v0(&sequence, &state, &params, &config)
        .expect("smartcore forward");
    assert_eq!(output.pooled_output.dim, 6);
    assert!(output.pooled_output.is_finite());
    assert!(output.sequence_output.iter().all(|tensor| tensor.dim == 4));
    assert!(output.no_training);
    assert!(output.no_weight_update);
    assert!(output.no_checkpoint);

    let guard = evaluate_microkernel_runtime_safety_v0(&output, &params, &config);
    assert_eq!(
        guard.safety_status,
        MicroKernelRuntimeSafetyStatus::Preserved
    );
    assert!(!guard.optimizer_present);
    assert!(!guard.gradient_present);
    assert!(!guard.checkpoint_present);
    assert!(!guard.python_dependency_present);
    assert!(!guard.torch_dependency_present);
    assert!(!guard.tensorflow_dependency_present);
    assert!(!guard.cuda_dependency_present);
    assert!(!guard.broker_order_account_present);

    let mut trainable_params = params.clone();
    trainable_params.trainable = true;
    let trainable_guard =
        evaluate_microkernel_runtime_safety_v0(&output, &trainable_params, &config);
    assert_eq!(
        trainable_guard.safety_status,
        MicroKernelRuntimeSafetyStatus::Violated
    );
    assert!(trainable_guard.trainable_params_present);

    let mut checkpoint_output = output.clone();
    checkpoint_output.no_checkpoint = false;
    let checkpoint_guard =
        evaluate_microkernel_runtime_safety_v0(&checkpoint_output, &params, &config);
    assert_eq!(
        checkpoint_guard.safety_status,
        MicroKernelRuntimeSafetyStatus::Violated
    );
    assert!(checkpoint_guard.checkpoint_present);

    let manifest = fs::read_to_string("Cargo.toml").expect("cargo manifest");
    let lowered = manifest.to_ascii_lowercase();
    for forbidden in [
        "python",
        "pyo3",
        "pytorch",
        "torch",
        "tensorflow",
        "cuda",
        "onnxruntime",
        "candle",
        "burn",
    ] {
        assert!(!lowered.contains(forbidden), "{forbidden} must stay absent");
    }
}

#[test]
fn sprint173_runtime_entry_gate_only_allows_offline_toy_forward_in_lab_mode() {
    let (lock_run, registry, batches, baseline_path) = sprint172_locked_runtime_entry_inputs();
    let denied = run_runtime_adapter_entry_gate(
        &registry,
        &batches,
        &lock_run,
        &RuntimeAdapterEntryGateRunConfig {
            run_id: "sprint173-runtime-entry-no-lab".to_string(),
            runtime_entry_gate_enabled: false,
            requested_capabilities: vec![SmartCoreRuntimeCapability::OfflineToyCoreForward],
            runtime_entry_audit_output_path: None,
            allow_shape_only_requests: true,
            fail_on_forbidden_capability: true,
            fail_on_contract_not_locked: true,
            fail_on_baseline_drift: true,
            fail_on_safety_violation: true,
            microkernel_lab_mode_policy: None,
            paper_only: true,
        },
    )
    .expect("offline toy forward denied without lab mode");
    assert_eq!(denied.gate_status, RuntimeAdapterEntryGateStatus::Blocked);
    assert!(denied.decisions.iter().all(|decision| {
        decision.decision == RuntimeAdapterEntryDecisionKind::DeniedForbiddenCapability
            && decision
                .denied_capabilities
                .contains(&SmartCoreRuntimeCapability::OfflineToyCoreForward)
    }));

    let allowed = run_runtime_adapter_entry_gate(
        &registry,
        &batches,
        &lock_run,
        &RuntimeAdapterEntryGateRunConfig {
            run_id: "sprint173-runtime-entry-lab".to_string(),
            runtime_entry_gate_enabled: true,
            requested_capabilities: vec![SmartCoreRuntimeCapability::OfflineToyCoreForward],
            runtime_entry_audit_output_path: None,
            allow_shape_only_requests: true,
            fail_on_forbidden_capability: true,
            fail_on_contract_not_locked: true,
            fail_on_baseline_drift: true,
            fail_on_safety_violation: true,
            microkernel_lab_mode_policy: Some(MicroKernelLabModePolicy {
                lab_mode_enabled: true,
                allow_offline_toy_forward: true,
                ..MicroKernelLabModePolicy::default()
            }),
            paper_only: true,
        },
    )
    .expect("offline toy forward granted in lab mode");
    assert_eq!(
        allowed.gate_status,
        RuntimeAdapterEntryGateStatus::ShapeOnlyGranted
    );
    assert!(allowed.decisions.iter().all(|decision| {
        decision.decision == RuntimeAdapterEntryDecisionKind::GrantedShapeOnly
            && decision
                .granted_capabilities
                .contains(&SmartCoreRuntimeCapability::OfflineToyCoreForward)
    }));

    let _ = fs::remove_file(baseline_path);
}

#[test]
fn sprint173_microkernel_dry_run_stays_paper_only_and_preserves_safety() {
    let (_members, dataset, _split, _batches, registry, ..) = sprint170_contract_inputs();
    let first = run_smartcore_microkernel_dry_run_v0(
        &dataset,
        &registry,
        &SmartCoreMicroKernelDryRunConfigV0 {
            run_id: "sprint173-microkernel".to_string(),
            member_ids: registry
                .profiles
                .iter()
                .map(|profile| profile.member_id.clone())
                .collect(),
            batch_size: 8,
            sequence_len: 3,
            input_dim: 8,
            temporal_state_dim: 8,
            memory_dim: 8,
            output_dim: 6,
            use_training_candidate_dataset: true,
            synthetic_input_fallback: true,
            paper_only: true,
        },
    )
    .expect("first microkernel dry-run");
    let second = run_smartcore_microkernel_dry_run_v0(
        &dataset,
        &registry,
        &SmartCoreMicroKernelDryRunConfigV0 {
            run_id: "sprint173-microkernel".to_string(),
            member_ids: registry
                .profiles
                .iter()
                .map(|profile| profile.member_id.clone())
                .collect(),
            batch_size: 8,
            sequence_len: 3,
            input_dim: 8,
            temporal_state_dim: 8,
            memory_dim: 8,
            output_dim: 6,
            use_training_candidate_dataset: true,
            synthetic_input_fallback: true,
            paper_only: true,
        },
    )
    .expect("second microkernel dry-run");
    assert_eq!(first, second);
    assert_eq!(first.output_count, registry.profile_count);
    assert_eq!(
        first.microkernel_status,
        SmartCoreMicroKernelStatusV0::Passed
    );
    assert_eq!(first.bridge_status, BatchToMicroKernelBridgeStatus::Passed);
    assert_eq!(first.bucket_dim_status, MicroKernelBucketDimStatus::Aligned);
    assert_eq!(
        first.warning_normalization_result.status,
        MicroKernelWarningNormalizationStatus::Clean
    );
    assert_eq!(
        first.safety_guard.safety_status,
        MicroKernelRuntimeSafetyStatus::Preserved
    );
    assert!(first.no_training);
    assert!(first.no_weight_update);
    assert!(first.no_checkpoint);
    assert!(first.no_live_inference);
    assert!(first.not_investment_signal);
    assert!(first.paper_only);
    assert_eq!(first.member_outputs.len(), registry.profile_count);
    assert!(first.member_outputs.iter().all(|output| {
        output.output.pooled_output.dim == 6
            && output.output.pooled_output.is_finite()
            && output.not_investment_signal
            && output.not_committee_opinion
            && output.paper_only
            && !matches!(
                output.mapping.mapping_status,
                AdapterToMicroKernelMappingStatus::Invalid
            )
    }));
}

#[test]
fn sprint173_microkernel_mapping_rejects_leakage_and_safety_guard_flags_non_finite() {
    let (_members, _dataset, _split, batches, _registry, ..) = sprint170_contract_inputs();
    let mut batch = batches
        .iter()
        .find(|batch| !batch.feature_rows.is_empty())
        .cloned()
        .expect("microkernel test batch");
    batch.feature_rows[0].market_features[0] = "target outcome broker account".to_string();
    let adapter_input_shape =
        build_adapter_input_shape_from_training_batch(&batch, &TrainingFeatureSchema::default());
    let config = soma_zero::league::minimal_ai_committee_core::SmartCoreMicroKernelConfigV0 {
        config_id: "sprint173-map".to_string(),
        member_id: batch.member_id.clone(),
        input_dim: 8,
        temporal_state_dim: 8,
        memory_dim: 8,
        hidden_dim: 8,
        output_dim: 6,
        sequence_len: 4,
        enabled_components: vec![
            soma_zero::league::minimal_ai_committee_core::SmartCoreMicroKernelComponentV0::Mamba3TemporalCellV0,
            soma_zero::league::minimal_ai_committee_core::SmartCoreMicroKernelComponentV0::GatedDeltaNetMemoryCellV0,
        ],
        sparse_event_attention_enabled: false,
        paper_only: true,
    };
    let mapping_error =
        map_training_batch_to_microkernel_sequence_v0(&batch, &adapter_input_shape, &config)
            .expect_err("leakage must be rejected");
    assert!(mapping_error.contains("label leakage") || mapping_error.contains("forbidden"));

    let (_members, dataset, _split, _batches, registry, ..) = sprint170_contract_inputs();
    let dry_run = run_smartcore_microkernel_dry_run_v0(
        &dataset,
        &registry,
        &SmartCoreMicroKernelDryRunConfigV0 {
            run_id: "sprint173-safety".to_string(),
            member_ids: vec![config.member_id.clone()],
            batch_size: 8,
            sequence_len: 4,
            input_dim: 8,
            temporal_state_dim: 8,
            memory_dim: 8,
            output_dim: 6,
            use_training_candidate_dataset: true,
            synthetic_input_fallback: true,
            paper_only: true,
        },
    )
    .expect("microkernel dry-run");
    let mut output = dry_run.member_outputs[0].output.clone();
    output.pooled_output.values[0] = f32::NAN;
    let safety = evaluate_microkernel_runtime_safety_v0(
        &output,
        &soma_zero::league::minimal_ai_committee_core::init_smartcore_microkernel_params_v0(
            &config, 13,
        )
        .expect("params"),
        &config,
    );
    assert_eq!(
        safety.safety_status,
        MicroKernelRuntimeSafetyStatus::Violated
    );
    assert!(!safety.finite_outputs);
}

#[test]
fn sprint174_bridge_contract_aligns_default_layout_and_rejects_dim_mismatch() {
    let config = SmartCoreMicroKernelConfigV0 {
        config_id: "sprint174-bridge-config".to_string(),
        member_id: "trend-entry-ai".to_string(),
        input_dim: 8,
        temporal_state_dim: 8,
        memory_dim: 8,
        hidden_dim: 8,
        output_dim: 6,
        sequence_len: 4,
        enabled_components: vec![
            SmartCoreMicroKernelComponentV0::Mamba3TemporalCellV0,
            SmartCoreMicroKernelComponentV0::GatedDeltaNetMemoryCellV0,
        ],
        sparse_event_attention_enabled: false,
        paper_only: true,
    };
    let bridge =
        default_adapter_to_microkernel_bridge_v1(&config, &default_adapter_input_schema_v1());
    let validation = validate_adapter_to_microkernel_bridge_v1(
        &bridge,
        &default_adapter_input_schema_v1(),
        &config,
    );
    assert_eq!(
        bridge
            .feature_group_layout
            .iter()
            .map(|row| row.dim)
            .sum::<usize>(),
        8
    );
    assert_eq!(
        validation.validation_status,
        AdapterToMicroKernelBridgeValidationStatus::Valid
    );

    let mut mismatched_config = config.clone();
    mismatched_config.input_dim = 7;
    let mismatch_validation = validate_adapter_to_microkernel_bridge_v1(
        &bridge,
        &default_adapter_input_schema_v1(),
        &mismatched_config,
    );
    assert_eq!(
        mismatch_validation.validation_status,
        AdapterToMicroKernelBridgeValidationStatus::Invalid
    );
    assert!(!mismatch_validation.microkernel_config_compatible);
}

#[test]
fn sprint174_bucket_policy_is_deterministic_and_rejects_forbidden_terms_and_nan() {
    let config = SmartCoreMicroKernelConfigV0 {
        config_id: "sprint174-bucket-config".to_string(),
        member_id: "trend-entry-ai".to_string(),
        input_dim: 8,
        temporal_state_dim: 8,
        memory_dim: 8,
        hidden_dim: 8,
        output_dim: 6,
        sequence_len: 4,
        enabled_components: vec![
            SmartCoreMicroKernelComponentV0::Mamba3TemporalCellV0,
            SmartCoreMicroKernelComponentV0::GatedDeltaNetMemoryCellV0,
        ],
        sparse_event_attention_enabled: false,
        paper_only: true,
    };
    let bridge =
        default_adapter_to_microkernel_bridge_v1(&config, &default_adapter_input_schema_v1());
    let policy = soma_zero::league::minimal_ai_committee_core::MicroKernelFeatureBucketPolicyV1 {
        policy_id: "sprint174-policy".to_string(),
        text_hash_bucket_count: 2,
        categorical_bucket_count: 1,
        numeric_scale: MicroKernelNumericScale::ClampUnit,
        unknown_text_bucket: "__unknown_text__".to_string(),
        unknown_category_bucket: "__unknown_category__".to_string(),
        max_text_chars: 96,
        forbid_target_terms: true,
        forbid_broker_order_account_terms: true,
        paper_only: true,
    };
    let first = bucketize_text_feature("earnings beat with stable guide", &policy, 2)
        .expect("bucketize text");
    let second = bucketize_text_feature("earnings beat with stable guide", &policy, 2)
        .expect("bucketize text repeat");
    assert_eq!(first, second);
    assert!(bucketize_text_feature("target outcome leak", &policy, 2).is_err());
    assert!(bucketize_text_feature("broker account order", &policy, 2).is_err());
    assert!(pack_numeric_features(&[f32::NAN], &policy, bridge.numeric_bucket_dim).is_err());
}

#[test]
fn sprint174_sequence_assembly_preserves_order_pads_and_rejects_wrong_dim() {
    let sequence_policy =
        soma_zero::league::minimal_ai_committee_core::MicroKernelSequenceAssemblyPolicyV1 {
            policy_id: "sprint174-sequence".to_string(),
            sequence_len: 4,
            pad_policy: MicroKernelSequencePadPolicy::ZeroPad,
            truncate_policy: MicroKernelSequenceTruncatePolicy::KeepMostRecent,
            preserve_temporal_order: true,
            paper_only: true,
        };
    let feature_vectors = vec![
        soma_zero::league::minimal_ai_committee_core::BucketizedFeatureVector {
            source_replay_id: Some("r1".to_string()),
            member_id: "trend-entry-ai".to_string(),
            symbol: "AAPL".to_string(),
            market_scope: MarketScope::UsShortTerm,
            values: from_vec_1d(vec![0.1, 0.2]).expect("v1"),
            bucket_policy_id: "policy".to_string(),
            feature_safety_status:
                soma_zero::league::minimal_ai_committee_core::MicroKernelFeatureSafetyStatus::Clean,
            removed_or_blocked_terms: Vec::new(),
            paper_only: true,
        },
        soma_zero::league::minimal_ai_committee_core::BucketizedFeatureVector {
            source_replay_id: Some("r2".to_string()),
            member_id: "trend-entry-ai".to_string(),
            symbol: "AAPL".to_string(),
            market_scope: MarketScope::UsShortTerm,
            values: from_vec_1d(vec![0.3, 0.4]).expect("v2"),
            bucket_policy_id: "policy".to_string(),
            feature_safety_status:
                soma_zero::league::minimal_ai_committee_core::MicroKernelFeatureSafetyStatus::Clean,
            removed_or_blocked_terms: Vec::new(),
            paper_only: true,
        },
    ];
    let assembled =
        assemble_microkernel_sequence(&feature_vectors, &sequence_policy, 2).expect("assemble");
    assert_eq!(
        assembled.assembly_status,
        MicroKernelSequenceAssemblyStatus::AssembledWithWarnings
    );
    assert_eq!(assembled.sequence_len, 4);
    assert_eq!(assembled.sequence[0], feature_vectors[0].values);
    assert_eq!(assembled.sequence[1], feature_vectors[1].values);
    assert_eq!(assembled.padded_count, 2);
    assert!(
        assembled.sequence[2]
            .values
            .iter()
            .all(|value| *value == 0.0)
    );

    let bad_vectors = vec![
        soma_zero::league::minimal_ai_committee_core::BucketizedFeatureVector {
            source_replay_id: None,
            member_id: "trend-entry-ai".to_string(),
            symbol: "AAPL".to_string(),
            market_scope: MarketScope::UsShortTerm,
            values: from_vec_1d(vec![0.1, 0.2, 0.3]).expect("bad"),
            bucket_policy_id: "policy".to_string(),
            feature_safety_status:
                soma_zero::league::minimal_ai_committee_core::MicroKernelFeatureSafetyStatus::Clean,
            removed_or_blocked_terms: Vec::new(),
            paper_only: true,
        },
    ];
    let rejected =
        assemble_microkernel_sequence(&bad_vectors, &sequence_policy, 2).expect("reject");
    assert_eq!(
        rejected.assembly_status,
        MicroKernelSequenceAssemblyStatus::Rejected
    );
}

#[test]
fn sprint174_member_bridge_profiles_and_batch_bridge_run_cover_all_members() {
    let (members, dataset, _split, _contract_batches, registry, ..) = sprint170_contract_inputs();
    let (_bridge_dataset, _bridge_split, mut bridge_batches) = sprint168_training_inputs(8);
    for batch in &mut bridge_batches {
        if let Some(last_row) = batch.feature_rows.last().cloned() {
            while batch.feature_rows.len() < 4 {
                batch.feature_rows.push(last_row.clone());
            }
        }
    }
    let member_configs: Vec<_> = registry
        .profiles
        .iter()
        .map(|profile| SmartCoreMicroKernelConfigV0 {
            config_id: format!("bridge-{}", profile.member_id),
            member_id: profile.member_id.clone(),
            input_dim: 8,
            temporal_state_dim: 8,
            memory_dim: 8,
            hidden_dim: 8,
            output_dim: 6,
            sequence_len: 4,
            enabled_components: vec![
                SmartCoreMicroKernelComponentV0::Mamba3TemporalCellV0,
                SmartCoreMicroKernelComponentV0::GatedDeltaNetMemoryCellV0,
            ],
            sparse_event_attention_enabled: false,
            paper_only: true,
        })
        .collect();
    let profile = build_member_microkernel_bridge_profile(
        &members[0],
        &registry.profiles[0],
        &member_configs[0],
    )
    .expect("member bridge profile");
    assert_eq!(
        profile.validation_result.validation_status,
        AdapterToMicroKernelBridgeValidationStatus::Valid
    );

    let mut unknown_member = members[0].clone();
    unknown_member.member_id = "unknown-member-alias".to_string();
    assert!(
        build_member_microkernel_bridge_profile(
            &unknown_member,
            &registry.profiles[0],
            &member_configs[0]
        )
        .is_err()
    );

    let bridge_run = run_batch_to_microkernel_bridge(
        &bridge_batches,
        &registry,
        &member_configs,
        &BatchToMicroKernelBridgeRunConfig {
            run_id: "sprint174-bridge-run".to_string(),
            sequence_len: 4,
            input_dim: 8,
            enforce_exact_dim: true,
            fail_on_warning: false,
            fail_on_leakage: true,
            paper_only: true,
        },
    )
    .expect("batch bridge run");
    assert_eq!(
        bridge_run.member_bridge_profiles.len(),
        registry.profile_count
    );
    assert_eq!(bridge_run.mapped_sequence_count, registry.profile_count);
    assert_eq!(bridge_run.rejected_sequence_count, 0);
    assert_eq!(
        bridge_run.bridge_status,
        BatchToMicroKernelBridgeStatus::Passed,
        "bridge warnings: {:?}",
        bridge_run.warnings
    );
    assert!(bridge_run.sequence_assembly_results.iter().all(|result| {
        matches!(
            result.assembly_status,
            MicroKernelSequenceAssemblyStatus::Assembled
                | MicroKernelSequenceAssemblyStatus::AssembledWithWarnings
        )
    }));

    let dry_run = run_smartcore_microkernel_dry_run_v0(
        &dataset,
        &registry,
        &SmartCoreMicroKernelDryRunConfigV0 {
            run_id: "sprint174-dry-run".to_string(),
            member_ids: registry
                .profiles
                .iter()
                .map(|profile| profile.member_id.clone())
                .collect(),
            batch_size: 8,
            sequence_len: 3,
            input_dim: 8,
            temporal_state_dim: 8,
            memory_dim: 8,
            output_dim: 6,
            use_training_candidate_dataset: true,
            synthetic_input_fallback: true,
            paper_only: true,
        },
    )
    .expect("dry-run");
    assert_eq!(
        dry_run.microkernel_status,
        SmartCoreMicroKernelStatusV0::Passed,
        "dry-run status: {:?}, bridge: {:?}, sequence: {:?}, warnings: {:?}",
        dry_run.microkernel_status,
        dry_run.bridge_status,
        dry_run.sequence_assembly_status,
        dry_run.warning_normalization_result
    );
    assert_eq!(
        dry_run.bridge_status,
        BatchToMicroKernelBridgeStatus::Passed
    );
    assert_eq!(
        dry_run.bucket_dim_status,
        MicroKernelBucketDimStatus::Aligned
    );
}

#[test]
fn sprint174_warning_normalization_blocks_dim_mismatch_and_keeps_expected_warnings() {
    let blocked = normalize_microkernel_warnings(&["bridge layout dim mismatch".to_string()], &[]);
    assert_eq!(
        blocked.status,
        MicroKernelWarningNormalizationStatus::Blocked
    );
    assert!(blocked.normalized_warnings.iter().any(|warning| {
        warning.kind == MicroKernelWarningKind::BridgeDimMismatch && !warning.expected
    }));

    let expected = normalize_microkernel_warnings(
        &[
            "padding applied with zero fill".to_string(),
            "Sparse Event Attention deferred".to_string(),
        ],
        &[],
    );
    assert_eq!(
        expected.status,
        MicroKernelWarningNormalizationStatus::CleanWithExpectedWarnings
    );
    assert_eq!(expected.blocking_warning_count, 0);
    assert_eq!(expected.expected_warning_count, 2);
}

#[test]
fn sprint175_head_projection_config_and_params_fail_closed() {
    let config = SmartCoreHeadProjectionConfigV0 {
        projection_id: "sprint175-config".to_string(),
        member_id: "trend-kr-short".to_string(),
        input_dim: 6,
        hidden_dim: 6,
        enabled_heads: vec![
            SmartCoreV2HeadKind::Stance,
            SmartCoreV2HeadKind::Risk,
            SmartCoreV2HeadKind::EvidenceNeed,
            SmartCoreV2HeadKind::ConfidenceCalibration,
            SmartCoreV2HeadKind::Uncertainty,
        ],
        output_mode: SmartCoreHeadProjectionOutputModeV0::DebugOnly,
        parameter_init: TinyParameterInitPolicyV0::DeterministicTiny,
        paper_only: true,
    };
    validate_head_projection_config_v0(&config).expect("valid head projection config");
    let params =
        soma_zero::league::minimal_ai_committee_core::init_head_projection_params_v0(&config, 17)
            .expect("deterministic params");
    validate_head_projection_params_v0(&params).expect("params must stay non-trainable");
    assert!(params.shared_projection.is_some());
    assert!(params.stance_projection.is_some());
    assert!(params.risk_projection.is_some());
    assert!(params.evidence_projection.is_some());
    assert!(params.confidence_projection.is_some());
    assert!(params.uncertainty_projection.is_some());
    assert!(params.expected_return_projection.is_none());
    assert!(!params.trainable);
    assert!(!params.persistent);
    assert!(params.paper_only);

    let invalid_config = SmartCoreHeadProjectionConfigV0 {
        input_dim: 0,
        ..config.clone()
    };
    assert!(validate_head_projection_config_v0(&invalid_config).is_err());

    let mut invalid_trainable = params.clone();
    invalid_trainable.trainable = true;
    assert!(
        validate_head_projection_params_v0(&invalid_trainable)
            .expect_err("trainable params must fail")
            .contains("trainable=false")
    );

    let mut invalid_persistent = params;
    invalid_persistent.persistent = true;
    assert!(
        validate_head_projection_params_v0(&invalid_persistent)
            .expect_err("persistent params must fail")
            .contains("persistent=false")
    );
}

#[test]
fn sprint175_debug_outputs_stay_finite_and_expected_return_is_deferred_by_default() {
    let (_profile_count, dry_run) = sprint175_microkernel_dry_run_fixture();
    let member_output = &dry_run.member_outputs[0];
    let config = SmartCoreHeadProjectionConfigV0 {
        projection_id: "sprint175-debug".to_string(),
        member_id: member_output.member_id.clone(),
        input_dim: member_output.output.pooled_output.dim,
        hidden_dim: member_output.output.pooled_output.dim,
        enabled_heads: vec![
            SmartCoreV2HeadKind::Stance,
            SmartCoreV2HeadKind::Risk,
            SmartCoreV2HeadKind::EvidenceNeed,
            SmartCoreV2HeadKind::ConfidenceCalibration,
            SmartCoreV2HeadKind::Uncertainty,
        ],
        output_mode: SmartCoreHeadProjectionOutputModeV0::DebugOnly,
        parameter_init: TinyParameterInitPolicyV0::DeterministicTiny,
        paper_only: true,
    };
    let params =
        soma_zero::league::minimal_ai_committee_core::init_head_projection_params_v0(&config, 23)
            .expect("projection params");
    let output = soma_zero::league::minimal_ai_committee_core::build_smartcore_debug_output_v0(
        member_output,
        &params,
        &config,
    )
    .expect("debug output");

    assert!(
        output
            .stance_head
            .as_ref()
            .is_some_and(|head| head.raw_score.is_finite() && head.debug_only && head.paper_only)
    );
    assert!(
        output
            .risk_head
            .as_ref()
            .is_some_and(|head| head.raw_score.is_finite() && head.debug_only && head.paper_only)
    );
    assert!(
        output.evidence_need_head.as_ref().is_some_and(|head| {
            head.raw_score.is_finite() && head.debug_only && head.paper_only
        })
    );
    assert!(
        output
            .confidence_calibration_head
            .as_ref()
            .is_some_and(|head| head.raw_score.is_finite() && head.debug_only && head.paper_only)
    );
    assert!(
        output.uncertainty_head.as_ref().is_some_and(|head| {
            head.raw_score.is_finite() && head.debug_only && head.paper_only
        })
    );
    let expected_return = output
        .expected_return_head
        .as_ref()
        .expect("expected return head output");
    assert!(expected_return.raw_score.is_none());
    assert!(matches!(
        expected_return.bucket,
        soma_zero::league::minimal_ai_committee_core::ToyExpectedReturnBucketV0::Deferred
    ));
    assert!(output.debug_only);
    assert!(output.not_investment_signal);
    assert!(output.not_committee_opinion);
    assert!(output.not_order);
    assert!(output.paper_only);
}

#[test]
fn sprint175_safety_and_interpretation_reject_misuse() {
    let (_profile_count, dry_run) = sprint175_microkernel_dry_run_fixture();
    let member_output = &dry_run.member_outputs[0];
    let config = SmartCoreHeadProjectionConfigV0 {
        projection_id: "sprint175-safety".to_string(),
        member_id: member_output.member_id.clone(),
        input_dim: member_output.output.pooled_output.dim,
        hidden_dim: member_output.output.pooled_output.dim,
        enabled_heads: vec![SmartCoreV2HeadKind::Stance, SmartCoreV2HeadKind::Risk],
        output_mode: SmartCoreHeadProjectionOutputModeV0::DebugOnly,
        parameter_init: TinyParameterInitPolicyV0::DeterministicTiny,
        paper_only: true,
    };
    let params =
        soma_zero::league::minimal_ai_committee_core::init_head_projection_params_v0(&config, 29)
            .expect("projection params");
    let mut output = soma_zero::league::minimal_ai_committee_core::build_smartcore_debug_output_v0(
        member_output,
        &params,
        &config,
    )
    .expect("debug output");
    output.debug_only = false;
    output.not_investment_signal = false;
    let safety =
        soma_zero::league::minimal_ai_committee_core::evaluate_smartcore_debug_output_safety_v0(
            &output,
        );
    assert_eq!(
        safety.safety_status,
        SmartCoreDebugOutputSafetyStatusV0::Violated
    );
    assert!(!safety.violations.is_empty());

    let interpretation =
        soma_zero::league::minimal_ai_committee_core::interpret_smartcore_debug_output_v0(
            &output,
            &soma_zero::league::minimal_ai_committee_core::SmartCoreDebugInterpretationPolicyV0 {
                allow_as_member_opinion: true,
                ..Default::default()
            },
        );
    assert_eq!(
        interpretation.interpretation_status,
        SmartCoreDebugInterpretationStatusV0::Violated
    );
    assert!(!interpretation.warnings.is_empty());
    assert!(interpretation.forbidden_uses.contains(
        &soma_zero::league::minimal_ai_committee_core::SmartCoreDebugForbiddenUseV0::MemberOpinion
    ));
    assert!(interpretation.forbidden_uses.contains(
        &soma_zero::league::minimal_ai_committee_core::SmartCoreDebugForbiddenUseV0::TradeSignal
    ));
}

#[test]
fn sprint175_head_projection_dry_run_is_deterministic_and_covers_all_members() {
    let (profile_count, dry_run) = sprint175_microkernel_dry_run_fixture();
    let output_path = sprint171_temp_json_path("sprint175-head-projection");
    let config = SmartCoreHeadProjectionDryRunConfigV0 {
        run_id: "sprint175-head-projection".to_string(),
        enable_stance_head: true,
        enable_risk_head: true,
        enable_evidence_head: true,
        enable_confidence_head: true,
        enable_uncertainty_head: true,
        enable_expected_return_head: false,
        output_path: Some(output_path.to_string_lossy().into_owned()),
        paper_only: true,
    };
    let first = run_smartcore_head_projection_dry_run_v0(&dry_run, &config).expect("first dry-run");
    let second =
        run_smartcore_head_projection_dry_run_v0(&dry_run, &config).expect("second dry-run");
    assert_eq!(first, second);
    assert_eq!(
        first.dry_run_status,
        SmartCoreHeadProjectionDryRunStatusV0::PassedWithWarnings
    );
    assert_eq!(first.debug_output_batch.output_count, profile_count);
    assert_eq!(first.debug_output_batch.member_outputs.len(), profile_count);
    assert_eq!(
        first.output_safety_summary.safety_status,
        SmartCoreDebugOutputSafetyStatusV0::Preserved
    );
    assert!(first.output_safety_summary.no_label_leakage);
    assert!(first.output_safety_summary.no_broker_order_account);
    assert!(first.interpretation_summary.iter().all(|result| {
        result.interpretation_status == SmartCoreDebugInterpretationStatusV0::DebugOnly
    }));
    assert_eq!(first.per_member_head_buckets.len(), profile_count);
    assert!(first.per_member_head_buckets.values().all(|buckets| {
        buckets
            .iter()
            .any(|bucket| bucket == "expected_return:Deferred")
    }));
    assert!(
        first
            .warnings
            .iter()
            .any(|warning| warning.contains("deferred by default"))
    );
    assert_eq!(first.head_projection_status, first.dry_run_status);
    assert!(first.no_training);
    assert!(first.no_weight_update);
    assert!(first.no_checkpoint);
    assert!(first.no_live_inference);
    assert!(first.debug_only);
    assert!(first.not_investment_signal);
    assert!(first.not_committee_opinion);
    assert!(first.not_order);
    assert!(first.no_broker_order_account);
    assert!(first.paper_only_warning.contains("paper-only"));
    assert!(fs::metadata(&output_path).is_ok());
    let _ = fs::remove_file(output_path);
}

#[test]
fn sprint175_autonomous_config_carries_head_projection_flags() {
    let config = MinimalAiCommitteeCycleConfig::from_toml_path(std::path::Path::new(
        "examples/soma_minimal_ai_committee_core.toml",
    ))
    .expect("config");
    let run_config = config
        .load_autonomous_paper_run_config()
        .expect("autonomous config");
    assert!(run_config.smartcore_head_projection_v0_enabled);
    assert_eq!(
        run_config.smartcore_head_projection_output_path.as_deref(),
        Some("target/minimal_smartcore_head_projection_v0.json")
    );
    assert!(run_config.smartcore_enable_stance_head);
    assert!(run_config.smartcore_enable_risk_head);
    assert!(run_config.smartcore_enable_evidence_head);
    assert!(run_config.smartcore_enable_confidence_head);
    assert!(run_config.smartcore_enable_uncertainty_head);
    assert!(!run_config.smartcore_enable_expected_return_head);
}

#[test]
fn sprint176_shadow_targets_build_from_member_opinions_and_replay_labels() {
    let (batch_result, _head_projection) = sprint176_shadow_alignment_fixture();
    let batch_targets = core::build_shadow_alignment_targets_from_batch_cycle(&batch_result);
    assert!(batch_targets.iter().any(|target| {
        target.source_type == core::SmartCoreShadowAlignmentTargetSourceType::MemberOpinion
    }));
    assert!(batch_targets.iter().any(|target| {
        target.source_type == core::SmartCoreShadowAlignmentTargetSourceType::RiskGovernorStatus
    }));
    assert!(batch_targets.iter().all(|target| target.paper_only));

    let replay_targets =
        core::build_shadow_alignment_targets_from_replay_dataset(&batch_result.replay_dataset);
    assert!(!replay_targets.is_empty());
    assert!(replay_targets.iter().all(|target| {
        target.source_type == core::SmartCoreShadowAlignmentTargetSourceType::ReplayTargetLabel
            && target.paper_only
    }));
}

#[test]
fn sprint176_head_bucket_normalization_maps_shadow_buckets() {
    let stance =
        core::normalize_smartcore_head_bucket(core::SmartCoreShadowHeadKind::Stance, "BuyLike");
    let risk =
        core::normalize_smartcore_head_bucket(core::SmartCoreShadowHeadKind::Risk, "HighRisk");
    let evidence = core::normalize_smartcore_head_bucket(
        core::SmartCoreShadowHeadKind::EvidenceNeed,
        "NeedMoreEvidence",
    );
    assert_eq!(
        stance.normalized_value,
        core::SmartCoreHeadBucketNormalizedValue::PositiveLike
    );
    assert_eq!(
        risk.normalized_value,
        core::SmartCoreHeadBucketNormalizedValue::RiskHigh
    );
    assert_eq!(
        evidence.normalized_value,
        core::SmartCoreHeadBucketNormalizedValue::NeedMoreEvidence
    );
}

#[test]
fn sprint176_debug_output_compares_to_matching_and_mismatching_targets() {
    let (_batch_result, head_projection) = sprint176_shadow_alignment_fixture();
    let debug_output = &head_projection.debug_output_batch.member_outputs[0];
    let stance_value = core::normalize_smartcore_head_bucket(
        core::SmartCoreShadowHeadKind::Stance,
        format!(
            "{:?}",
            debug_output
                .stance_head
                .as_ref()
                .expect("stance head present")
                .bucket
        ),
    )
    .normalized_value;
    let risk_value = core::normalize_smartcore_head_bucket(
        core::SmartCoreShadowHeadKind::Risk,
        format!(
            "{:?}",
            debug_output
                .risk_head
                .as_ref()
                .expect("risk head present")
                .bucket
        ),
    )
    .normalized_value;
    let evidence_value = core::normalize_smartcore_head_bucket(
        core::SmartCoreShadowHeadKind::EvidenceNeed,
        format!(
            "{:?}",
            debug_output
                .evidence_need_head
                .as_ref()
                .expect("evidence head present")
                .bucket
        ),
    )
    .normalized_value;
    let confidence_value = core::normalize_smartcore_head_bucket(
        core::SmartCoreShadowHeadKind::ConfidenceCalibration,
        format!(
            "{:?}",
            debug_output
                .confidence_calibration_head
                .as_ref()
                .expect("confidence head present")
                .bucket
        ),
    )
    .normalized_value;

    let matching = core::compare_smartcore_debug_output_to_target(
        debug_output,
        &core::SmartCoreShadowAlignmentTarget {
            target_id: "matching-target".to_string(),
            member_id: debug_output.member_id.clone(),
            symbol: None,
            market_scope: None,
            source_type: core::SmartCoreShadowAlignmentTargetSourceType::MemberOpinion,
            stance_target: Some(stance_value),
            risk_target: Some(risk_value),
            evidence_target: Some(evidence_value),
            confidence_target: Some(confidence_value),
            outcome_target: None,
            paper_only: true,
        },
    );
    assert_eq!(
        matching.stance_alignment,
        core::SmartCoreShadowAlignmentStatus::Match
    );
    assert_eq!(
        matching.risk_alignment,
        core::SmartCoreShadowAlignmentStatus::Match
    );

    let mismatching = core::compare_smartcore_debug_output_to_target(
        debug_output,
        &core::SmartCoreShadowAlignmentTarget {
            target_id: "mismatch-target".to_string(),
            member_id: debug_output.member_id.clone(),
            symbol: None,
            market_scope: None,
            source_type: core::SmartCoreShadowAlignmentTargetSourceType::ReplayTargetLabel,
            stance_target: Some(match stance_value {
                core::SmartCoreHeadBucketNormalizedValue::PositiveLike => {
                    core::SmartCoreHeadBucketNormalizedValue::NegativeLike
                }
                _ => core::SmartCoreHeadBucketNormalizedValue::PositiveLike,
            }),
            risk_target: Some(match risk_value {
                core::SmartCoreHeadBucketNormalizedValue::RiskHigh => {
                    core::SmartCoreHeadBucketNormalizedValue::RiskLow
                }
                _ => core::SmartCoreHeadBucketNormalizedValue::RiskHigh,
            }),
            evidence_target: Some(match evidence_value {
                core::SmartCoreHeadBucketNormalizedValue::NeedMoreEvidence => {
                    core::SmartCoreHeadBucketNormalizedValue::EvidenceSufficient
                }
                _ => core::SmartCoreHeadBucketNormalizedValue::NeedMoreEvidence,
            }),
            confidence_target: Some(match confidence_value {
                core::SmartCoreHeadBucketNormalizedValue::ConfidenceHigh => {
                    core::SmartCoreHeadBucketNormalizedValue::ConfidenceLow
                }
                _ => core::SmartCoreHeadBucketNormalizedValue::ConfidenceHigh,
            }),
            outcome_target: None,
            paper_only: true,
        },
    );
    assert_eq!(
        mismatching.risk_alignment,
        core::SmartCoreShadowAlignmentStatus::Mismatch
    );
    assert_eq!(
        mismatching.evidence_alignment,
        core::SmartCoreShadowAlignmentStatus::Mismatch
    );
    assert!(!mismatching.mismatch_reasons.is_empty());
}

#[test]
fn sprint176_summary_and_mismatch_records_classify_shadow_gaps() {
    let records = vec![
        core::SmartCoreShadowAlignmentRecord {
            alignment_id: "risk-under".to_string(),
            member_id: "risk-kr-short".to_string(),
            debug_output_id: "risk-debug".to_string(),
            target_id: Some("risk-target".to_string()),
            symbol: Some("005930".to_string()),
            market_scope: Some(MarketScope::KoreaShortTerm),
            stance_alignment: core::SmartCoreShadowAlignmentStatus::Match,
            risk_alignment: core::SmartCoreShadowAlignmentStatus::Mismatch,
            evidence_alignment: core::SmartCoreShadowAlignmentStatus::Match,
            confidence_alignment: core::SmartCoreShadowAlignmentStatus::Match,
            mismatch_reasons: vec!["risk:RiskLow!=RiskHigh".to_string()],
            normalized_debug_buckets: std::collections::BTreeMap::from([(
                "risk".to_string(),
                "RiskLow".to_string(),
            )]),
            normalized_target_values: std::collections::BTreeMap::from([(
                "risk".to_string(),
                "RiskHigh".to_string(),
            )]),
            debug_only: true,
            not_decision_input: true,
            paper_only: true,
        },
        core::SmartCoreShadowAlignmentRecord {
            alignment_id: "evidence-over".to_string(),
            member_id: "evidence-kr-short".to_string(),
            debug_output_id: "evidence-debug".to_string(),
            target_id: Some("evidence-target".to_string()),
            symbol: Some("005930".to_string()),
            market_scope: Some(MarketScope::KoreaShortTerm),
            stance_alignment: core::SmartCoreShadowAlignmentStatus::Unknown,
            risk_alignment: core::SmartCoreShadowAlignmentStatus::Unknown,
            evidence_alignment: core::SmartCoreShadowAlignmentStatus::Mismatch,
            confidence_alignment: core::SmartCoreShadowAlignmentStatus::Unknown,
            mismatch_reasons: vec!["evidence:EvidenceSufficient!=NeedMoreEvidence".to_string()],
            normalized_debug_buckets: std::collections::BTreeMap::from([(
                "evidence".to_string(),
                "EvidenceSufficient".to_string(),
            )]),
            normalized_target_values: std::collections::BTreeMap::from([(
                "evidence".to_string(),
                "NeedMoreEvidence".to_string(),
            )]),
            debug_only: true,
            not_decision_input: true,
            paper_only: true,
        },
    ];
    let summary = core::summarize_smartcore_shadow_alignment(&records);
    assert_eq!(summary.mismatch_count, 2);
    let mismatches = core::build_smartcore_shadow_mismatch_records(&records);
    assert!(mismatches.iter().any(|record| {
        record.mismatch_type == core::SmartCoreShadowMismatchType::RiskUnderestimation
    }));
    assert!(mismatches.iter().any(|record| {
        record.mismatch_type == core::SmartCoreShadowMismatchType::EvidenceOverconfidence
    }));
}

#[test]
fn sprint176_no_decision_bridge_guard_preserves_and_detects_leaks() {
    let (batch_result, head_projection) = sprint176_shadow_alignment_fixture();
    let config = core::SmartCoreShadowAlignmentRunConfig {
        run_id: "sprint176-shadow-guard".to_string(),
        enabled: true,
        include_batch_member_opinions: true,
        include_replay_targets: true,
        include_risk_governor_targets: true,
        output_path: None,
        emit_owner_core_debug_cards: true,
        paper_only: true,
    };
    let run_result = core::run_smartcore_shadow_alignment(
        &head_projection.debug_output_batch,
        &batch_result,
        Some(&batch_result.replay_dataset),
        &config,
    )
    .expect("shadow alignment run");
    assert_eq!(
        run_result.no_decision_bridge_guard_status,
        core::SmartCoreNoDecisionBridgeGuardStatus::Preserved
    );

    let debug_output_id = head_projection.debug_output_batch.member_outputs[0]
        .debug_output_id
        .clone();
    let mut opinion_leak = batch_result.clone();
    opinion_leak.member_opinions[0].event_reason = Some(debug_output_id.clone());
    let guard_from_opinion = core::evaluate_smartcore_no_decision_bridge_guard(
        &opinion_leak,
        &head_projection.debug_output_batch,
        &run_result,
    );
    assert_eq!(
        guard_from_opinion.guard_status,
        core::SmartCoreNoDecisionBridgeGuardStatus::Violated
    );
    assert!(guard_from_opinion.debug_output_used_as_member_opinion);

    let mut trade_signal_leak = batch_result;
    trade_signal_leak.event_queue.queue_id = debug_output_id;
    let guard_from_trade_signal = core::evaluate_smartcore_no_decision_bridge_guard(
        &trade_signal_leak,
        &head_projection.debug_output_batch,
        &run_result,
    );
    assert_eq!(
        guard_from_trade_signal.guard_status,
        core::SmartCoreNoDecisionBridgeGuardStatus::Violated
    );
    assert!(guard_from_trade_signal.debug_output_used_as_trade_signal);
}

#[test]
fn sprint176_owner_core_debug_cards_flow_into_console_and_read_model() {
    let (batch_result, head_projection) = sprint176_shadow_alignment_fixture();
    let shadow_result = core::run_smartcore_shadow_alignment(
        &head_projection.debug_output_batch,
        &batch_result,
        Some(&batch_result.replay_dataset),
        &core::SmartCoreShadowAlignmentRunConfig {
            run_id: "sprint176-owner-debug".to_string(),
            enabled: true,
            include_batch_member_opinions: true,
            include_replay_targets: true,
            include_risk_governor_targets: true,
            output_path: None,
            emit_owner_core_debug_cards: true,
            paper_only: true,
        },
    )
    .expect("shadow alignment run");
    let sample = fs::read_to_string("examples/minimal_ai_committee_multi_market_sample.json")
        .expect("sprint176 state batch input");
    let batch_input: BatchCommitteeCycleInput =
        serde_json::from_str(&sample).expect("parse sprint176 state batch input");
    let mut stateful = run_batch_committee_cycle_with_state(BatchCommitteeCycleWithStateInput {
        batch_input,
        member_state_store: None,
        member_state_output_path: None,
        emit_owner_summary: true,
        emit_owner_console_view: true,
        owner_feedback: Vec::new(),
        emit_reconsideration_view: false,
    })
    .expect("stateful cycle");
    stateful.batch_result.smartcore_shadow_alignment_run_result = Some(shadow_result.clone());
    let console = core::build_owner_committee_console_view(
        &stateful.batch_result,
        &stateful.state_update,
        None,
    );
    let section = console
        .core_debug_section
        .as_ref()
        .expect("core debug section");
    assert_eq!(section.cards.len(), 3);
    assert!(section.cards.iter().all(|card| {
        card.debug_only
            && card.not_investment_signal
            && card.not_committee_opinion
            && card.paper_only
    }));
    let snapshot = core::build_committee_state_snapshot(core::CommitteeStateExportInput {
        member_state_store: Some(core::MemberStateStore {
            store_id: "sprint176-shadow-store".to_string(),
            members: stateful.state_update.updated_member_states.clone(),
            source_label: "shadow-test".to_string(),
            paper_only: true,
        }),
        owner_console_view: Some(console.clone()),
        ..Default::default()
    })
    .expect("snapshot");
    let read_model = core::build_owner_console_read_model("sprint176-schema", &snapshot);
    assert!(read_model.core_debug_section.is_some());
}

#[test]
fn sprint176_shadow_alignment_run_is_deterministic_and_non_mutating() {
    let (batch_result, head_projection) = sprint176_shadow_alignment_fixture();
    let original_decisions = batch_result.chairman_decisions.clone();
    let original_scores = batch_result.score_updates.clone();
    let config = core::SmartCoreShadowAlignmentRunConfig {
        run_id: "sprint176-shadow-run".to_string(),
        enabled: true,
        include_batch_member_opinions: true,
        include_replay_targets: true,
        include_risk_governor_targets: true,
        output_path: None,
        emit_owner_core_debug_cards: true,
        paper_only: true,
    };
    let first = core::run_smartcore_shadow_alignment(
        &head_projection.debug_output_batch,
        &batch_result,
        Some(&batch_result.replay_dataset),
        &config,
    )
    .expect("first run");
    let second = core::run_smartcore_shadow_alignment(
        &head_projection.debug_output_batch,
        &batch_result,
        Some(&batch_result.replay_dataset),
        &config,
    )
    .expect("second run");
    assert_eq!(first, second);
    assert_eq!(batch_result.chairman_decisions, original_decisions);
    assert_eq!(batch_result.score_updates, original_scores);
    assert!(matches!(
        first.run_status,
        core::SmartCoreShadowAlignmentRunStatus::Passed
            | core::SmartCoreShadowAlignmentRunStatus::PassedWithWarnings
    ));
    assert!(first.debug_only);
    assert!(first.not_investment_signal);
    assert!(first.not_committee_opinion);
}

#[test]
fn sprint176_autonomous_config_carries_shadow_alignment_flags() {
    let config = MinimalAiCommitteeCycleConfig::from_toml_path(std::path::Path::new(
        "examples/soma_minimal_ai_committee_core.toml",
    ))
    .expect("config");
    let run_config = config
        .load_autonomous_paper_run_config()
        .expect("autonomous config");
    assert!(run_config.smartcore_shadow_alignment_enabled);
    assert_eq!(
        run_config.smartcore_shadow_alignment_output_path.as_deref(),
        Some("target/minimal_smartcore_shadow_alignment.json")
    );
    assert!(run_config.smartcore_shadow_include_batch_member_opinions);
    assert!(run_config.smartcore_shadow_include_replay_targets);
    assert!(run_config.smartcore_shadow_include_risk_governor_targets);
    assert!(run_config.smartcore_emit_owner_debug_cards);
}

#[test]
fn sprint177_mismatch_data_need_classification_maps_actionable_categories() {
    let records = sprint177_sample_mismatch_records();
    let classified = records
        .iter()
        .map(core::classify_smartcore_mismatch_data_need)
        .collect::<Vec<_>>();
    assert_eq!(
        classified[0].data_need,
        core::SmartCoreMismatchDataNeed::MoreRiskVetoCases
    );
    assert_eq!(
        classified[1].data_need,
        core::SmartCoreMismatchDataNeed::MoreNeedMoreEvidenceCases
    );
    assert_eq!(
        classified[2].data_need,
        core::SmartCoreMismatchDataNeed::MoreStanceLabels
    );
    assert_eq!(
        classified[3].data_need,
        core::SmartCoreMismatchDataNeed::MoreConfidenceCalibrationLabels
    );
}

#[test]
fn sprint177_mismatch_analysis_summary_counts_member_and_head() {
    let classified = sprint177_sample_mismatch_records()
        .iter()
        .map(core::classify_smartcore_mismatch_data_need)
        .collect::<Vec<_>>();
    let summary = core::summarize_smartcore_mismatch_data_needs(&classified);
    assert_eq!(summary.mismatch_count, 4);
    assert_eq!(summary.by_member.get("trend-kr-short"), Some(&2));
    assert_eq!(summary.by_member.get("risk-kr-short"), Some(&1));
    assert_eq!(summary.by_head.get("Risk"), Some(&1));
    assert_eq!(summary.by_head.get("EvidenceNeed"), Some(&1));
}

#[test]
fn sprint177_research_task_generation_creates_role_tasks_and_respects_limits() {
    let classified = sprint177_sample_mismatch_records()
        .iter()
        .map(core::classify_smartcore_mismatch_data_need)
        .collect::<Vec<_>>();
    let full = core::generate_research_tasks_from_smartcore_mismatches(
        &classified,
        &core::MismatchToResearchTaskPolicy {
            max_tasks_total: 12,
            max_tasks_per_member: 4,
            ..Default::default()
        },
    );
    assert!(full.generated_tasks.tasks.iter().any(|task| {
        task.member_id == "RiskGuardAI"
            && task.task_type == MemberResearchTaskType::ReviewRiskVetoCase
    }));
    assert!(full.generated_tasks.tasks.iter().any(|task| {
        task.member_id == "EvidenceRegimeAI"
            && task.task_type == MemberResearchTaskType::ReviewNeedMoreEvidenceCase
    }));

    let limited = core::generate_research_tasks_from_smartcore_mismatches(
        &classified,
        &core::MismatchToResearchTaskPolicy {
            max_tasks_total: 1,
            max_tasks_per_member: 1,
            ..Default::default()
        },
    );
    assert_eq!(limited.generated_count, 1);
    assert!(limited.skipped_count >= 1);
}

#[test]
fn sprint177_core_calibration_dataset_roundtrips_and_rejects_unsafe_fields() {
    let (_batch_result, shadow_alignment, _pipeline) = sprint177_mismatch_pipeline_fixture();
    let examples = core::build_core_calibration_examples(
        &shadow_alignment.alignment_records,
        &shadow_alignment.mismatch_records,
    );
    assert!(!examples.is_empty());
    let dataset = core::build_core_calibration_dataset(&examples);
    let path = sprint171_temp_json_path("sprint177-calibration-dataset");
    core::save_core_calibration_dataset_to_local_json(&dataset, &path).expect("save dataset");
    let loaded = core::load_core_calibration_dataset_from_local_json(&path).expect("load dataset");
    assert!(loaded.paper_only);
    assert_eq!(loaded.example_count, dataset.example_count);
    fs::remove_file(&path).expect("remove dataset");

    let mut unsafe_dataset = dataset.clone();
    unsafe_dataset.paper_only = false;
    let err = core::save_core_calibration_dataset_to_local_json(&unsafe_dataset, &path)
        .expect_err("paper_only=false must fail");
    assert!(err.contains("paper_only"));

    let bad_path = sprint171_temp_json_path("sprint177-calibration-bad");
    fs::write(&bad_path, r#"{"broker":"forbidden"}"#).expect("write bad json");
    let bad_err = core::load_core_calibration_dataset_from_local_json(&bad_path)
        .expect_err("unsafe calibration json must fail");
    assert!(bad_err.contains("unsafe field") || bad_err.contains("broker"));
    fs::remove_file(&bad_path).expect("remove bad json");
}

#[test]
fn sprint177_calibration_quality_summary_and_owner_debug_summary_are_safe() {
    let (_batch_result, _shadow_alignment, pipeline) = sprint177_mismatch_pipeline_fixture();
    let quality = core::summarize_core_calibration_quality(&pipeline.core_calibration_dataset);
    assert!(quality.mismatch_rate >= 0.0);
    assert_eq!(quality.example_count, pipeline.calibration_example_count);
    let debug_summary = core::build_owner_core_calibration_debug_summary(&pipeline);
    assert!(debug_summary.debug_only);
    assert!(debug_summary.not_investment_signal);
    assert!(debug_summary.not_committee_opinion);
    assert!(debug_summary.message.contains("observed only"));
    assert!(debug_summary.message.contains("research tasks"));
}

#[test]
fn sprint177_no_decision_recheck_preserves_and_detects_calibration_leak() {
    let (batch_result, _shadow_alignment, pipeline) = sprint177_mismatch_pipeline_fixture();
    let clean = core::recheck_smartcore_mismatch_no_decision_boundary(&pipeline, &batch_result);
    assert_eq!(
        clean.no_decision_status,
        core::SmartCoreNoDecisionBridgeGuardStatus::Preserved
    );

    let mut leaked = batch_result;
    leaked.member_opinions[0].event_reason =
        Some(pipeline.core_calibration_dataset.dataset_id.clone());
    let leaked_recheck = core::recheck_smartcore_mismatch_no_decision_boundary(&pipeline, &leaked);
    assert_eq!(
        leaked_recheck.no_decision_status,
        core::SmartCoreNoDecisionBridgeGuardStatus::Violated
    );
    assert!(leaked_recheck.calibration_dataset_used_for_decision);
}

#[test]
fn sprint177_pipeline_run_is_deterministic_and_non_mutating() {
    let (batch_result, shadow_alignment, first_pipeline) = sprint177_mismatch_pipeline_fixture();
    let original_decisions = batch_result.chairman_decisions.clone();
    let original_scores = batch_result.score_updates.clone();
    let second_pipeline = core::run_smartcore_mismatch_self_growing_pipeline(
        &shadow_alignment,
        &core::SmartCoreMismatchSelfGrowingRunConfig {
            run_id: "sprint177-mismatch".to_string(),
            enabled: true,
            max_tasks_total: 12,
            max_tasks_per_member: 4,
            core_calibration_dataset_output_path: None,
            mismatch_task_output_path: None,
            emit_owner_console_summary: true,
            paper_only: true,
        },
    )
    .expect("second mismatch pipeline");
    assert_eq!(first_pipeline, second_pipeline);
    assert_eq!(batch_result.chairman_decisions, original_decisions);
    assert_eq!(batch_result.score_updates, original_scores);
    assert!(first_pipeline.generated_research_task_count > 0);
    assert!(first_pipeline.calibration_example_count > 0);
    assert!(first_pipeline.safety_summary.no_model_training);
    assert!(first_pipeline.safety_summary.no_live_inference);
    assert!(first_pipeline.safety_summary.no_broker_order_account);
    assert!(first_pipeline.safety_summary.no_real_order_path);
}

#[test]
fn sprint177_autonomous_config_carries_mismatch_pipeline_flags() {
    let config = MinimalAiCommitteeCycleConfig::from_toml_path(std::path::Path::new(
        "examples/soma_minimal_ai_committee_core.toml",
    ))
    .expect("config");
    let run_config = config
        .load_autonomous_paper_run_config()
        .expect("autonomous config");
    assert!(run_config.smartcore_mismatch_self_growing_enabled);
    assert_eq!(run_config.smartcore_mismatch_max_tasks_total, 12);
    assert_eq!(run_config.smartcore_mismatch_max_tasks_per_member, 4);
    assert_eq!(
        run_config
            .smartcore_calibration_dataset_output_path
            .as_deref(),
        Some("target/minimal_smartcore_calibration_dataset_refreshed.json")
    );
    assert_eq!(
        run_config.smartcore_mismatch_task_output_path.as_deref(),
        Some("target/minimal_smartcore_mismatch_tasks.json")
    );
    assert!(run_config.smartcore_mismatch_emit_owner_debug_summary);
}

#[test]
fn sprint178_mismatch_research_task_execution_loads_local_queue_and_stays_paper_only() {
    let classified = sprint177_sample_mismatch_records()
        .iter()
        .map(core::classify_smartcore_mismatch_data_need)
        .collect::<Vec<_>>();
    let generated = core::generate_research_tasks_from_smartcore_mismatches(
        &classified,
        &core::MismatchToResearchTaskPolicy {
            max_tasks_total: 12,
            max_tasks_per_member: 4,
            ..Default::default()
        },
    );
    let queue = generated.generated_tasks;
    let queue_path = sprint171_temp_json_path("sprint178-mismatch-task-queue");
    fs::write(
        &queue_path,
        serde_json::to_string_pretty(&queue).expect("queue json"),
    )
    .expect("write queue");
    let source_registry = core::ResearchSourceRegistry::load_from_local_json(std::path::Path::new(
        "examples/research_sources.sample.json",
    ))
    .expect("source registry");
    let result = core::execute_mismatch_research_tasks(
        &[],
        &source_registry,
        &core::MismatchResearchTaskExecutionConfig {
            run_id: "sprint178-task-execution".to_string(),
            task_input_path: Some(queue_path.to_string_lossy().into_owned()),
            source_registry_path: None,
            max_tasks: 1,
            max_evidence_per_task: 2,
            allow_network_sources: false,
            paper_only: true,
        },
    )
    .expect("task execution");
    assert_eq!(result.loaded_task_count, queue.tasks.len());
    assert_eq!(result.executed_task_count, 1);
    assert!(result.generated_evidence_count > 0);
    assert!(result.paper_only);
    assert!(
        result
            .skipped_reasons
            .iter()
            .any(|reason| reason.contains("max_tasks=1"))
    );
    assert!(
        result
            .skipped_reasons
            .iter()
            .any(|reason| reason.contains("network research sources remain disabled"))
    );
    assert!(
        result
            .evidence_bundle
            .records
            .iter()
            .all(|record| record.paper_only)
    );
    fs::remove_file(&queue_path).expect("remove queue");
}

#[test]
fn sprint178_target_candidates_and_approval_policy_handle_trust_and_missing_head() {
    let classified = sprint177_sample_mismatch_records()
        .iter()
        .map(core::classify_smartcore_mismatch_data_need)
        .collect::<Vec<_>>();
    let generated = core::generate_research_tasks_from_smartcore_mismatches(
        &classified,
        &core::MismatchToResearchTaskPolicy {
            max_tasks_total: 12,
            max_tasks_per_member: 4,
            ..Default::default()
        },
    );
    let risk_task = generated
        .generated_tasks
        .tasks
        .iter()
        .find(|task| task.task_type == MemberResearchTaskType::ReviewRiskVetoCase)
        .expect("risk task");
    let news_task = generated
        .generated_tasks
        .tasks
        .iter()
        .find(|task| task.task_type == MemberResearchTaskType::ReviewNeedMoreEvidenceCase)
        .expect("news task");
    let stance_task = generated
        .generated_tasks
        .tasks
        .iter()
        .find(|task| task.task_type == MemberResearchTaskType::BuildReplayEvidenceCandidate)
        .expect("stance task");
    let evidence_bundle = core::ResearchEvidenceBundle {
        bundle_id: "sprint178-candidate-bundle".to_string(),
        records: vec![
            core::ResearchEvidenceRecord {
                evidence_id: "trusted-price".to_string(),
                task_id: Some(risk_task.task_id.clone()),
                member_id: risk_task.member_id.clone(),
                symbol: risk_task.symbol.clone(),
                market_scope: risk_task.market_scope,
                kind: core::ResearchEvidenceKind::PriceMove,
                headline: None,
                summary: Some("price_change_pct=0.07 volatility_hint=0.09".to_string()),
                source_id: Some("trusted-local-price".to_string()),
                source_trust_score: Some(0.95),
                related_replay_id: None,
                related_experience_id: None,
                related_decision_id: None,
                related_attention_item_id: None,
                evidence_confidence: ReplayLabelConfidence::High,
                evidence_notes: vec!["price_change_pct=0.07".to_string()],
                paper_only: true,
            },
            core::ResearchEvidenceRecord {
                evidence_id: "review-news".to_string(),
                task_id: Some(news_task.task_id.clone()),
                member_id: news_task.member_id.clone(),
                symbol: news_task.symbol.clone(),
                market_scope: news_task.market_scope,
                kind: core::ResearchEvidenceKind::NewsHeadline,
                headline: Some("review required headline".to_string()),
                summary: Some("coverage gap remains".to_string()),
                source_id: Some("review-required-news".to_string()),
                source_trust_score: Some(0.45),
                related_replay_id: None,
                related_experience_id: None,
                related_decision_id: None,
                related_attention_item_id: None,
                evidence_confidence: ReplayLabelConfidence::Medium,
                evidence_notes: vec![],
                paper_only: true,
            },
            core::ResearchEvidenceRecord {
                evidence_id: "warning-price".to_string(),
                task_id: Some(stance_task.task_id.clone()),
                member_id: stance_task.member_id.clone(),
                symbol: stance_task.symbol.clone(),
                market_scope: stance_task.market_scope,
                kind: core::ResearchEvidenceKind::PriceMove,
                headline: None,
                summary: Some("price_change_pct=-0.03 volatility_hint=0.04".to_string()),
                source_id: Some("usable-with-warnings-price".to_string()),
                source_trust_score: Some(0.72),
                related_replay_id: None,
                related_experience_id: None,
                related_decision_id: None,
                related_attention_item_id: None,
                evidence_confidence: ReplayLabelConfidence::Medium,
                evidence_notes: vec!["price_change_pct=-0.03".to_string()],
                paper_only: true,
            },
            core::ResearchEvidenceRecord {
                evidence_id: "coverage-gap".to_string(),
                task_id: Some(news_task.task_id.clone()),
                member_id: news_task.member_id.clone(),
                symbol: news_task.symbol.clone(),
                market_scope: news_task.market_scope,
                kind: core::ResearchEvidenceKind::CoverageGapEvidence,
                headline: None,
                summary: Some("coverage gap evidence for mismatch".to_string()),
                source_id: Some("trusted-gap-source".to_string()),
                source_trust_score: Some(0.91),
                related_replay_id: None,
                related_experience_id: None,
                related_decision_id: None,
                related_attention_item_id: None,
                evidence_confidence: ReplayLabelConfidence::Medium,
                evidence_notes: vec![],
                paper_only: true,
            },
        ],
        source_scores: vec![
            core::SourceTrustScore {
                source_id: "trusted-local-price".to_string(),
                base_trust_level: core::ResearchSourceTrustLevel::High,
                freshness_score: 1.0,
                consistency_score: 1.0,
                coverage_score: 1.0,
                safety_score: 1.0,
                final_score: 0.98,
                trust_status: core::SourceTrustStatus::Trusted,
                reasons: vec![],
                paper_only: true,
            },
            core::SourceTrustScore {
                source_id: "review-required-news".to_string(),
                base_trust_level: core::ResearchSourceTrustLevel::ReviewRequired,
                freshness_score: 0.5,
                consistency_score: 0.5,
                coverage_score: 0.4,
                safety_score: 0.6,
                final_score: 0.45,
                trust_status: core::SourceTrustStatus::ReviewRequired,
                reasons: vec!["needs review".to_string()],
                paper_only: true,
            },
            core::SourceTrustScore {
                source_id: "usable-with-warnings-price".to_string(),
                base_trust_level: core::ResearchSourceTrustLevel::Medium,
                freshness_score: 0.8,
                consistency_score: 0.7,
                coverage_score: 0.7,
                safety_score: 0.8,
                final_score: 0.72,
                trust_status: core::SourceTrustStatus::UsableWithWarnings,
                reasons: vec!["partial coverage".to_string()],
                paper_only: true,
            },
            core::SourceTrustScore {
                source_id: "trusted-gap-source".to_string(),
                base_trust_level: core::ResearchSourceTrustLevel::High,
                freshness_score: 0.9,
                consistency_score: 0.9,
                coverage_score: 0.9,
                safety_score: 1.0,
                final_score: 0.91,
                trust_status: core::SourceTrustStatus::Trusted,
                reasons: vec![],
                paper_only: true,
            },
        ],
        task_count: 3,
        evidence_count: 4,
        paper_only: true,
    };
    let candidates = core::build_core_calibration_target_candidates(&evidence_bundle, &classified);
    let coverage_gap = candidates
        .iter()
        .find(|candidate| candidate.source_evidence_id == "coverage-gap")
        .expect("coverage gap candidate");
    assert_eq!(
        coverage_gap.head,
        Some(core::SmartCoreShadowHeadKind::EvidenceNeed)
    );
    assert_eq!(
        coverage_gap.target_bucket,
        Some(core::SmartCoreHeadBucketNormalizedValue::NeedMoreEvidence)
    );
    let approval = core::approve_core_calibration_target_candidates(
        &candidates,
        &core::CoreCalibrationTargetApprovalPolicy::default(),
    );
    let trusted = approval
        .approved_candidates
        .iter()
        .find(|candidate| candidate.source_evidence_id == "trusted-price")
        .expect("trusted price approved");
    assert_eq!(trusted.head, Some(core::SmartCoreShadowHeadKind::Risk));
    assert_eq!(
        trusted.target_bucket,
        Some(core::SmartCoreHeadBucketNormalizedValue::RiskHigh)
    );
    assert!(
        approval
            .review_candidates
            .iter()
            .any(|candidate| candidate.source_evidence_id == "review-news")
    );
    assert!(
        approval
            .review_candidates
            .iter()
            .any(|candidate| candidate.source_evidence_id == "warning-price")
    );

    let mut missing_head = trusted.clone();
    missing_head.candidate_id = "missing-head".to_string();
    missing_head.head = None;
    let mut missing_member = trusted.clone();
    missing_member.candidate_id = "missing-member".to_string();
    missing_member.member_id.clear();
    let rejection_result = core::approve_core_calibration_target_candidates(
        &[missing_head, missing_member],
        &core::CoreCalibrationTargetApprovalPolicy::default(),
    );
    assert_eq!(rejection_result.rejected_count, 2);
    assert!(
        rejection_result
            .rejected_candidates
            .iter()
            .all(|candidate| candidate.approval_status
                == core::CoreCalibrationTargetApprovalStatus::Rejected)
    );
    assert_eq!(
        rejection_result.rejected_candidates[0].approval_status,
        core::CoreCalibrationTargetApprovalStatus::Rejected
    );
}

#[test]
fn sprint178_calibration_dataset_refresh_dedupes_and_respects_dry_run_write() {
    let (_batch_result, _shadow_alignment, pipeline) = sprint177_mismatch_pipeline_fixture();
    let approved_candidate = core::CoreCalibrationTargetCandidate {
        candidate_id: "approved-refresh-candidate".to_string(),
        source_task_id: "task-risk".to_string(),
        source_evidence_id: "evidence-risk".to_string(),
        member_id: "RiskGuardAI".to_string(),
        symbol: Some("MSFT".to_string()),
        market_scope: Some(MarketScope::UsShortTerm),
        head: Some(core::SmartCoreShadowHeadKind::Risk),
        target_bucket: Some(core::SmartCoreHeadBucketNormalizedValue::RiskHigh),
        target_source: core::CoreCalibrationTargetSource::PriceMoveEvidence,
        source_trust_status: core::SourceTrustStatus::Trusted,
        label_confidence: ReplayLabelConfidence::High,
        approval_status: core::CoreCalibrationTargetApprovalStatus::Approved,
        paper_only: true,
    };
    let dry_run_path = sprint171_temp_json_path("sprint178-calibration-refresh-dry-run");
    let dry_run = core::refresh_core_calibration_dataset(
        Some(&pipeline.core_calibration_dataset),
        &[approved_candidate.clone(), approved_candidate.clone()],
        &core::CoreCalibrationDatasetRefreshConfig {
            run_id: "sprint178-refresh-dry-run".to_string(),
            existing_dataset_path: None,
            output_dataset_path: Some(dry_run_path.to_string_lossy().into_owned()),
            include_existing: true,
            include_approved_candidates: true,
            dedupe_by_member_head_symbol_target: true,
            dry_run: true,
            paper_only: true,
        },
    )
    .expect("dry-run refresh");
    assert_eq!(dry_run.added_example_count, 1);
    assert_eq!(dry_run.duplicate_count, 1);
    assert_eq!(
        dry_run.new_example_count,
        pipeline.core_calibration_dataset.example_count + 1
    );
    assert!(!dry_run_path.exists());

    let write_path = sprint171_temp_json_path("sprint178-calibration-refresh-write");
    let written = core::refresh_core_calibration_dataset(
        Some(&pipeline.core_calibration_dataset),
        &[approved_candidate],
        &core::CoreCalibrationDatasetRefreshConfig {
            run_id: "sprint178-refresh-write".to_string(),
            existing_dataset_path: None,
            output_dataset_path: Some(write_path.to_string_lossy().into_owned()),
            include_existing: true,
            include_approved_candidates: true,
            dedupe_by_member_head_symbol_target: true,
            dry_run: false,
            paper_only: true,
        },
    )
    .expect("write refresh");
    assert!(write_path.exists());
    let loaded =
        core::load_core_calibration_dataset_from_local_json(&write_path).expect("load refreshed");
    assert_eq!(loaded.example_count, written.new_example_count);
    fs::remove_file(&write_path).expect("remove refresh dataset");
}

#[test]
fn sprint178_alignment_recheck_and_no_decision_guard_remain_diagnostic_only() {
    let (batch_result, shadow_alignment, loop_result) = sprint178_learning_loop_fixture(false);
    let recheck = loop_result
        .alignment_recheck_result
        .as_ref()
        .expect("alignment recheck");
    assert_eq!(
        recheck.previous_mismatch_count,
        Some(shadow_alignment.mismatch_records.len())
    );
    assert!(recheck.new_target_count >= recheck.previous_target_count.unwrap_or(0));
    assert!(
        loop_result.calibration_dataset_new_count >= loop_result.calibration_dataset_previous_count
    );
    let guard =
        core::evaluate_mismatch_learning_no_decision_guard(&loop_result, &batch_result, None);
    assert_eq!(
        guard.guard_status,
        core::SmartCoreNoDecisionBridgeGuardStatus::Preserved
    );
    assert!(loop_result.debug_only);
    assert!(loop_result.not_investment_signal);
    assert!(loop_result.not_committee_opinion);

    let mut leaked_loop = loop_result.clone();
    leaked_loop.target_approval_result = Some(core::CoreCalibrationTargetApprovalResult {
        candidate_count: 1,
        approved_count: 1,
        needs_review_count: 0,
        rejected_count: 0,
        approved_candidates: vec![core::CoreCalibrationTargetCandidate {
            candidate_id: "approved-leak-marker".to_string(),
            source_task_id: "task-risk".to_string(),
            source_evidence_id: "evidence-risk".to_string(),
            member_id: "RiskGuardAI".to_string(),
            symbol: Some("MSFT".to_string()),
            market_scope: Some(MarketScope::UsShortTerm),
            head: Some(core::SmartCoreShadowHeadKind::Risk),
            target_bucket: Some(core::SmartCoreHeadBucketNormalizedValue::RiskHigh),
            target_source: core::CoreCalibrationTargetSource::PriceMoveEvidence,
            source_trust_status: core::SourceTrustStatus::Trusted,
            label_confidence: ReplayLabelConfidence::High,
            approval_status: core::CoreCalibrationTargetApprovalStatus::Approved,
            paper_only: true,
        }],
        review_candidates: Vec::new(),
        rejected_candidates: Vec::new(),
        paper_only: true,
    });
    let mut leaked = batch_result.clone();
    leaked.member_opinions[0].event_reason = Some("approved-leak-marker".to_string());
    let leaked_guard =
        core::evaluate_mismatch_learning_no_decision_guard(&leaked_loop, &leaked, None);
    assert_eq!(
        leaked_guard.guard_status,
        core::SmartCoreNoDecisionBridgeGuardStatus::Violated
    );
    assert!(leaked_guard.calibration_targets_used_as_opinion);
}

#[test]
fn sprint178_learning_loop_is_deterministic_and_non_training() {
    let (_batch_result, _shadow_alignment, first) = sprint178_learning_loop_fixture(false);
    let (_batch_result_again, _shadow_alignment_again, second) =
        sprint178_learning_loop_fixture(false);
    assert_eq!(first, second);
    assert!(first.executed_research_task_count > 0);
    assert!(first.generated_evidence_count > 0);
    assert!(first.target_candidate_count > 0);
    assert!(first.target_approval_result.is_some());
    assert_eq!(
        first.approved_target_count,
        first
            .target_approval_result
            .as_ref()
            .map(|result| result.approved_count)
            .unwrap_or(0)
    );
    assert!(first.safety_summary.no_model_training);
    assert!(first.safety_summary.no_weight_update);
    assert!(first.safety_summary.no_checkpoint);
    assert!(first.safety_summary.no_live_inference);
    assert!(first.safety_summary.no_broker_order_account);
    assert!(first.safety_summary.no_real_order_path);
}

#[test]
fn sprint178_autonomous_config_carries_mismatch_learning_loop_flags() {
    let config = MinimalAiCommitteeCycleConfig::from_toml_path(std::path::Path::new(
        "examples/soma_minimal_ai_committee_core.toml",
    ))
    .expect("config");
    let run_config = config
        .load_autonomous_paper_run_config()
        .expect("autonomous config");
    assert!(run_config.smartcore_mismatch_learning_loop_enabled);
    assert!(run_config.smartcore_mismatch_learning_dry_run);
    assert!(run_config.smartcore_execute_mismatch_research_tasks);
    assert!(run_config.smartcore_approve_calibration_targets);
    assert!(run_config.smartcore_refresh_calibration_dataset);
    assert!(run_config.smartcore_recheck_alignment);
    assert_eq!(
        run_config
            .smartcore_calibration_dataset_input_path
            .as_deref(),
        Some("target/minimal_smartcore_calibration_dataset.json")
    );
    assert_eq!(
        run_config
            .smartcore_calibration_dataset_output_path
            .as_deref(),
        Some("target/minimal_smartcore_calibration_dataset_refreshed.json")
    );
    assert_eq!(
        run_config
            .smartcore_mismatch_learning_loop_output_path
            .as_deref(),
        Some("target/minimal_smartcore_mismatch_learning_loop.json")
    );
}

#[test]
fn sprint179_calibration_stats_count_mismatches_and_group_by_member_head() {
    let (_batch_result, debug_output_batch, _shadow_alignment, dataset, _recalibration_result) =
        sprint179_recalibration_fixture(true);
    let stats = core::compute_smartcore_calibration_stats_v0(&dataset);
    let risk_head = stats
        .per_head_stats
        .iter()
        .find(|head| {
            head.member_id == "risk-kr-short" && head.head == core::SmartCoreShadowHeadKind::Risk
        })
        .expect("risk head stats");
    let evidence_head = stats
        .per_head_stats
        .iter()
        .find(|head| {
            head.member_id == "evidence-kr-short"
                && head.head == core::SmartCoreShadowHeadKind::EvidenceNeed
        })
        .expect("evidence head stats");
    let total_mismatches: usize = stats
        .per_head_stats
        .iter()
        .map(|head| head.mismatch_count)
        .sum();

    assert_eq!(debug_output_batch.member_outputs.len(), 3);
    assert_eq!(stats.total_examples, 9);
    assert_eq!(total_mismatches, 7);
    assert!(stats.overall_mismatch_rate > 0.7);
    assert_eq!(stats.per_head_stats.len(), 4);
    assert_eq!(risk_head.example_count, 3);
    assert_eq!(risk_head.mismatch_count, 3);
    assert_eq!(
        risk_head.dominant_target_bucket.as_deref(),
        Some("RiskHigh")
    );
    assert_eq!(evidence_head.example_count, 1);
    assert_eq!(
        stats.stats_status,
        core::SmartCoreCalibrationStatsStatusV0::ThinData
    );
}

#[test]
fn sprint179_rule_table_creates_active_and_observe_only_rules() {
    let (_batch_result, debug_output_batch, _shadow_alignment, dataset, _recalibration_result) =
        sprint179_recalibration_fixture(true);
    let stats = core::compute_smartcore_calibration_stats_v0(&dataset);
    let rules = core::build_smartcore_calibration_rule_table_v0(
        &stats,
        &core::SmartCoreCalibrationOverlayPolicyV0::default(),
    );
    let risk_rule = rules
        .rules
        .iter()
        .find(|rule| {
            rule.member_id == "risk-kr-short" && rule.head == core::SmartCoreShadowHeadKind::Risk
        })
        .expect("risk rule");
    let evidence_rule = rules
        .rules
        .iter()
        .find(|rule| {
            rule.member_id == "evidence-kr-short"
                && rule.head == core::SmartCoreShadowHeadKind::EvidenceNeed
        })
        .expect("evidence rule");
    let expected_return_rule = rules
        .rules
        .iter()
        .find(|rule| {
            rule.member_id == "trend-kr-short"
                && rule.head == core::SmartCoreShadowHeadKind::ExpectedReturnHint
        })
        .expect("expected return rule");

    assert_eq!(debug_output_batch.member_outputs.len(), 3);
    assert_eq!(
        risk_rule.rule_status,
        core::SmartCoreCalibrationRuleStatusV0::Active
    );
    assert_eq!(
        risk_rule.action,
        core::SmartCoreCalibrationRuleActionV0::RaiseRiskBucket
    );
    assert_eq!(
        evidence_rule.rule_status,
        core::SmartCoreCalibrationRuleStatusV0::ObserveOnly
    );
    assert_eq!(
        expected_return_rule.rule_status,
        core::SmartCoreCalibrationRuleStatusV0::Disabled
    );
}

#[test]
fn sprint179_overlay_policy_defaults_remain_debug_only() {
    let policy = core::SmartCoreCalibrationOverlayPolicyV0::default();

    assert!(policy.debug_only);
    assert!(policy.paper_only);
    assert!(!policy.allow_trade_signal_output);
    assert!(!policy.allow_member_opinion_output);
    assert!(!policy.allow_committee_decision_output);
    assert!(!policy.allow_expected_return_mapping);
}

#[test]
fn sprint179_overlay_changes_only_debug_buckets_and_preserves_original() {
    let (_batch_result, debug_output_batch, _shadow_alignment, dataset, _recalibration_result) =
        sprint179_recalibration_fixture(true);
    let stats = core::compute_smartcore_calibration_stats_v0(&dataset);
    let rules = core::build_smartcore_calibration_rule_table_v0(
        &stats,
        &core::SmartCoreCalibrationOverlayPolicyV0::default(),
    );
    let original_batch = debug_output_batch.clone();
    let calibrated = core::apply_smartcore_calibration_overlay_v0(
        &debug_output_batch,
        &rules,
        &core::SmartCoreCalibrationOverlayPolicyV0::default(),
    );
    let risk_output = calibrated
        .member_outputs
        .iter()
        .find(|output| output.member_id == "risk-kr-short")
        .expect("risk calibrated output");
    let risk_head = risk_output
        .calibrated_heads
        .iter()
        .find(|head| head.head == core::SmartCoreShadowHeadKind::Risk)
        .expect("risk calibrated head");

    assert_eq!(debug_output_batch, original_batch);
    assert!(calibrated.changed_output_count > 0);
    assert!(risk_head.changed);
    assert_ne!(risk_head.original_bucket, risk_head.calibrated_bucket);
    assert!(risk_output.not_investment_signal);
    assert!(risk_output.not_committee_opinion);
    assert!(risk_output.not_order);
}

#[test]
fn sprint179_overlay_safety_guard_detects_member_opinion_and_trade_signal_misuse() {
    let (_batch_result, _debug_output_batch, _shadow_alignment, dataset, recalibration_result) =
        sprint179_recalibration_fixture(true);
    let stats = core::compute_smartcore_calibration_stats_v0(&dataset);
    let rules = core::build_smartcore_calibration_rule_table_v0(
        &stats,
        &core::SmartCoreCalibrationOverlayPolicyV0::default(),
    );
    let preserved = core::evaluate_smartcore_calibration_overlay_safety_v0(
        &recalibration_result.calibrated_debug_output_batch,
        &rules,
    );
    let mut member_leak = recalibration_result.calibrated_debug_output_batch.clone();
    member_leak.member_outputs[0].not_committee_opinion = false;
    let member_violation =
        core::evaluate_smartcore_calibration_overlay_safety_v0(&member_leak, &rules);
    let mut trade_leak = recalibration_result.calibrated_debug_output_batch.clone();
    trade_leak.member_outputs[0].not_investment_signal = false;
    let trade_violation =
        core::evaluate_smartcore_calibration_overlay_safety_v0(&trade_leak, &rules);

    assert_eq!(
        preserved.safety_status,
        core::SmartCoreCalibrationOverlaySafetyStatusV0::Preserved
    );
    assert_eq!(
        member_violation.safety_status,
        core::SmartCoreCalibrationOverlaySafetyStatusV0::Violated
    );
    assert!(!member_violation.not_member_opinion);
    assert_eq!(
        trade_violation.safety_status,
        core::SmartCoreCalibrationOverlaySafetyStatusV0::Violated
    );
    assert!(!trade_violation.not_trade_signal);
}

#[test]
fn sprint179_recalibration_pass_recomputes_alignment_and_preserves_safety() {
    let (_batch_result, _debug_output_batch, shadow_alignment, _dataset, recalibration_result) =
        sprint179_recalibration_fixture(false);
    let interpreted = &recalibration_result.interpretation;
    let owner_summary = recalibration_result
        .owner_core_debug_update
        .as_ref()
        .expect("owner summary");

    assert_eq!(
        recalibration_result.pre_mismatch_count,
        shadow_alignment.mismatch_count
    );
    assert_eq!(
        recalibration_result.post_mismatch_count,
        recalibration_result
            .recalibrated_alignment_result
            .new_mismatch_count,
    );
    assert_eq!(
        recalibration_result.mismatch_delta,
        recalibration_result.post_mismatch_count as isize
            - recalibration_result.pre_mismatch_count as isize
    );
    assert_eq!(
        recalibration_result.no_decision_recheck.guard_status,
        core::SmartCoreNoDecisionBridgeGuardStatus::Preserved
    );
    assert!(recalibration_result.safety_summary.no_model_training);
    assert!(recalibration_result.safety_summary.no_weight_update);
    assert!(recalibration_result.safety_summary.no_checkpoint);
    assert!(recalibration_result.safety_summary.no_live_inference);
    assert!(recalibration_result.safety_summary.no_broker_order_account);
    assert!(owner_summary.debug_only);
    assert!(owner_summary.paper_only);
    assert!(interpreted.debug_only);
}

#[test]
fn sprint179_recalibration_dry_run_writes_no_files() {
    let (batch_result, debug_output_batch, shadow_alignment, dataset, _recalibration_result) =
        sprint179_recalibration_fixture(true);
    let rule_path = sprint171_temp_json_path("sprint179-rule-table");
    let calibrated_path = sprint171_temp_json_path("sprint179-calibrated");
    let result_path = sprint171_temp_json_path("sprint179-result");
    let _ = std::fs::remove_file(&rule_path);
    let _ = std::fs::remove_file(&calibrated_path);
    let _ = std::fs::remove_file(&result_path);

    let dry_run_result = core::run_smartcore_shadow_recalibration_pass(
        &debug_output_batch,
        &dataset,
        &shadow_alignment,
        &batch_result,
        &core::SmartCoreShadowRecalibrationRunConfig {
            run_id: "sprint179-dry-run".to_string(),
            enabled: true,
            calibration_dataset_path: None,
            rule_table_output_path: Some(rule_path.to_string_lossy().into_owned()),
            calibrated_debug_output_path: Some(calibrated_path.to_string_lossy().into_owned()),
            recalibration_result_output_path: Some(result_path.to_string_lossy().into_owned()),
            min_support_for_active_rule: 2,
            max_rules_per_member_head: 2,
            emit_owner_summary: true,
            dry_run: true,
            paper_only: true,
        },
    )
    .expect("dry-run recalibration");

    assert_eq!(
        dry_run_result.run_status,
        core::SmartCoreShadowRecalibrationRunStatus::PassedWithWarnings
    );
    assert!(!rule_path.exists());
    assert!(!calibrated_path.exists());
    assert!(!result_path.exists());
}

#[test]
fn sprint179_recalibration_is_deterministic_and_autonomous_config_carries_flags() {
    let (_batch_result, _debug_output_batch, _shadow_alignment, _dataset, first) =
        sprint179_recalibration_fixture(true);
    let (
        _batch_result_again,
        _debug_output_batch_again,
        _shadow_alignment_again,
        _dataset_again,
        second,
    ) = sprint179_recalibration_fixture(true);
    let config = MinimalAiCommitteeCycleConfig::from_toml_path(std::path::Path::new(
        "examples/soma_minimal_ai_committee_core.toml",
    ))
    .expect("config");
    let run_config = config
        .load_autonomous_paper_run_config()
        .expect("autonomous config");

    assert_eq!(first, second);
    assert!(run_config.smartcore_recalibration_enabled);
    assert!(run_config.smartcore_recalibration_dry_run);
    assert!(run_config.smartcore_recalibration_emit_owner_summary);
    assert_eq!(
        run_config
            .smartcore_recalibration_rule_table_output_path
            .as_deref(),
        Some("target/minimal_smartcore_calibration_rule_table.json")
    );
    assert_eq!(
        run_config.smartcore_calibrated_debug_output_path.as_deref(),
        Some("target/minimal_smartcore_calibrated_debug_output.json")
    );
    assert_eq!(
        run_config
            .smartcore_recalibration_result_output_path
            .as_deref(),
        Some("target/minimal_smartcore_recalibration_result.json")
    );
}

#[test]
fn sprint180_shadow_candidates_build_from_calibrated_output_and_stay_shadow_only() {
    let (_batch_result, _recalibration_result, run_result) = sprint180_shadow_opinion_fixture();

    assert_eq!(run_result.candidate_batch.candidate_count, 3);
    assert_eq!(run_result.candidate_batch.member_count, 3);
    assert!(run_result.candidate_batch.shadow_only);
    assert!(run_result.candidate_batch.debug_only);
    assert!(
        run_result
            .candidate_batch
            .candidates
            .iter()
            .all(|candidate| candidate.shadow_only
                && candidate.debug_only
                && candidate.not_member_opinion
                && candidate.not_committee_input
                && candidate.not_trade_signal
                && candidate.not_order)
    );
}

#[test]
fn sprint180_shadow_candidate_compare_handles_matching_and_mismatching_member_opinions() {
    let (_batch_result, _recalibration_result, run_result) = sprint180_shadow_opinion_fixture();
    let candidate = run_result
        .candidate_batch
        .candidates
        .iter()
        .find(|candidate| {
            !matches!(
                candidate.shadow_action,
                core::SmartCoreShadowOpinionAction::ShadowRiskWarning
                    | core::SmartCoreShadowOpinionAction::ShadowUnknown
            ) && candidate.shadow_confidence != core::SmartCoreShadowOpinionConfidence::Unknown
                && candidate.shadow_risk != core::SmartCoreShadowOpinionRisk::Unknown
        })
        .expect("comparable candidate");
    let matching = MemberOpinion {
        member_id: candidate.member_id.clone(),
        symbol: candidate
            .symbol
            .clone()
            .unwrap_or_else(|| "005930.KS".to_string()),
        market_scope: candidate
            .market_scope
            .unwrap_or(MarketScope::KoreaShortTerm),
        stance: match candidate.shadow_action {
            core::SmartCoreShadowOpinionAction::ShadowBuyLike => MemberStance::BuyProposal,
            core::SmartCoreShadowOpinionAction::ShadowHoldLike => MemberStance::Hold,
            core::SmartCoreShadowOpinionAction::ShadowNoTradeLike => MemberStance::NoTrade,
            core::SmartCoreShadowOpinionAction::ShadowNeedMoreEvidence => {
                MemberStance::NeedMoreEvidence
            }
            core::SmartCoreShadowOpinionAction::ShadowRiskWarning
            | core::SmartCoreShadowOpinionAction::ShadowUnknown => MemberStance::Hold,
        },
        confidence: match candidate.shadow_confidence {
            core::SmartCoreShadowOpinionConfidence::Low => 0.2,
            core::SmartCoreShadowOpinionConfidence::Medium => 0.6,
            core::SmartCoreShadowOpinionConfidence::High => 0.9,
            core::SmartCoreShadowOpinionConfidence::Unknown => 0.5,
        },
        expected_return_hint: 0.01,
        risk_hint: match candidate.shadow_risk {
            core::SmartCoreShadowOpinionRisk::Low => 0.1,
            core::SmartCoreShadowOpinionRisk::Medium => 0.35,
            core::SmartCoreShadowOpinionRisk::High => 0.8,
            core::SmartCoreShadowOpinionRisk::Unknown => 0.3,
        },
        evidence_notes: if candidate.shadow_evidence
            == core::SmartCoreShadowOpinionEvidence::NeedMoreEvidence
        {
            vec!["need more evidence".to_string()]
        } else {
            vec!["evidence sufficient".to_string()]
        },
        event_triggered: false,
        event_reason: None,
    };
    let disagreeing = MemberOpinion {
        stance: MemberStance::NoTrade,
        confidence: 0.1,
        risk_hint: 0.85,
        evidence_notes: vec!["mismatch".to_string()],
        ..matching.clone()
    };
    let agree_record = core::compare_shadow_candidate_to_member_opinion(candidate, Some(&matching));
    let disagree_record =
        core::compare_shadow_candidate_to_member_opinion(candidate, Some(&disagreeing));
    let summary =
        core::summarize_shadow_vs_member_opinion(&[agree_record.clone(), disagree_record.clone()]);

    assert_eq!(
        agree_record.agreement,
        core::ShadowVsMemberOpinionAgreement::Agree
    );
    assert_eq!(
        disagree_record.agreement,
        core::ShadowVsMemberOpinionAgreement::Disagree
    );
    assert_eq!(summary.comparison_count, 2);
    assert_eq!(summary.disagree_count, 1);
}

#[test]
fn sprint180_target_eval_compares_risk_and_evidence_targets() {
    let (_batch_result, _recalibration_result, run_result) = sprint180_shadow_opinion_fixture();
    let candidate = run_result
        .candidate_batch
        .candidates
        .iter()
        .find(|candidate| {
            candidate.shadow_risk != core::SmartCoreShadowOpinionRisk::Unknown
                && candidate.shadow_evidence != core::SmartCoreShadowOpinionEvidence::Unknown
        })
        .expect("candidate with risk/evidence");
    let matching_target = core::SmartCoreShadowAlignmentTarget {
        target_id: "shadow-target-match".to_string(),
        member_id: candidate.member_id.clone(),
        symbol: candidate.symbol.clone(),
        market_scope: candidate.market_scope,
        source_type: core::SmartCoreShadowAlignmentTargetSourceType::RiskGovernorStatus,
        stance_target: None,
        risk_target: Some(match candidate.shadow_risk {
            core::SmartCoreShadowOpinionRisk::Low => {
                core::SmartCoreHeadBucketNormalizedValue::RiskLow
            }
            core::SmartCoreShadowOpinionRisk::Medium => {
                core::SmartCoreHeadBucketNormalizedValue::RiskMedium
            }
            core::SmartCoreShadowOpinionRisk::High => {
                core::SmartCoreHeadBucketNormalizedValue::RiskHigh
            }
            core::SmartCoreShadowOpinionRisk::Unknown => {
                core::SmartCoreHeadBucketNormalizedValue::Unknown
            }
        }),
        evidence_target: Some(match candidate.shadow_evidence {
            core::SmartCoreShadowOpinionEvidence::EvidenceSufficient => {
                core::SmartCoreHeadBucketNormalizedValue::EvidenceSufficient
            }
            core::SmartCoreShadowOpinionEvidence::NeedMoreEvidence => {
                core::SmartCoreHeadBucketNormalizedValue::NeedMoreEvidence
            }
            core::SmartCoreShadowOpinionEvidence::Unknown => {
                core::SmartCoreHeadBucketNormalizedValue::Unknown
            }
        }),
        confidence_target: None,
        outcome_target: None,
        paper_only: true,
    };
    let mismatching_target = core::SmartCoreShadowAlignmentTarget {
        target_id: "shadow-target-mismatch".to_string(),
        risk_target: Some(match candidate.shadow_risk {
            core::SmartCoreShadowOpinionRisk::Low => {
                core::SmartCoreHeadBucketNormalizedValue::RiskHigh
            }
            core::SmartCoreShadowOpinionRisk::Medium => {
                core::SmartCoreHeadBucketNormalizedValue::RiskLow
            }
            core::SmartCoreShadowOpinionRisk::High => {
                core::SmartCoreHeadBucketNormalizedValue::RiskLow
            }
            core::SmartCoreShadowOpinionRisk::Unknown => {
                core::SmartCoreHeadBucketNormalizedValue::RiskMedium
            }
        }),
        evidence_target: Some(
            if candidate.shadow_evidence == core::SmartCoreShadowOpinionEvidence::NeedMoreEvidence {
                core::SmartCoreHeadBucketNormalizedValue::EvidenceSufficient
            } else {
                core::SmartCoreHeadBucketNormalizedValue::NeedMoreEvidence
            },
        ),
        ..matching_target.clone()
    };
    let evals = core::evaluate_shadow_candidate_against_targets(
        candidate,
        &[matching_target, mismatching_target],
    );
    let summary = core::summarize_shadow_candidate_target_eval(&evals);

    assert_eq!(evals.len(), 2);
    assert_eq!(
        evals[0].risk_alignment,
        core::SmartCoreShadowAlignmentStatus::Match
    );
    assert_eq!(
        evals[0].evidence_alignment,
        core::SmartCoreShadowAlignmentStatus::Match
    );
    assert_eq!(
        evals[1].risk_alignment,
        core::SmartCoreShadowAlignmentStatus::Mismatch
    );
    assert_eq!(
        evals[1].evidence_alignment,
        core::SmartCoreShadowAlignmentStatus::Mismatch
    );
    assert!(summary.mismatch_count >= 1);
}

#[test]
fn sprint180_decision_isolation_guard_detects_member_and_trade_leaks() {
    let (batch_result, _recalibration_result, run_result) = sprint180_shadow_opinion_fixture();
    let marker = run_result.candidate_batch.candidates[0]
        .candidate_id
        .clone();
    let mut leaked_member = batch_result.clone();
    leaked_member.member_opinions[0].event_reason = Some(marker.clone());
    let member_violation = core::evaluate_shadow_opinion_decision_isolation(
        &run_result.candidate_batch,
        &leaked_member,
        Some(&leaked_member),
    );
    let mut leaked_source = batch_result.clone();
    leaked_source.member_opinions[0].event_reason = Some(
        run_result.candidate_batch.candidates[0]
            .source_calibrated_output_id
            .clone(),
    );
    let source_violation = core::evaluate_shadow_opinion_decision_isolation(
        &run_result.candidate_batch,
        &leaked_source,
        Some(&leaked_source),
    );
    let mut leaked_trade = batch_result.clone();
    leaked_trade.event_queue.queue_id = marker;
    let trade_violation = core::evaluate_shadow_opinion_decision_isolation(
        &run_result.candidate_batch,
        &leaked_trade,
        Some(&leaked_trade),
    );

    assert_eq!(
        run_result.decision_isolation_guard.guard_status,
        core::SmartCoreShadowOpinionDecisionIsolationGuardStatus::Preserved
    );
    assert_eq!(
        member_violation.guard_status,
        core::SmartCoreShadowOpinionDecisionIsolationGuardStatus::Violated
    );
    assert!(member_violation.shadow_candidate_used_as_member_opinion);
    assert_eq!(
        source_violation.guard_status,
        core::SmartCoreShadowOpinionDecisionIsolationGuardStatus::Violated
    );
    assert!(source_violation.shadow_candidate_used_as_member_opinion);
    assert_eq!(
        trade_violation.guard_status,
        core::SmartCoreShadowOpinionDecisionIsolationGuardStatus::Violated
    );
    assert!(trade_violation.shadow_candidate_used_as_trade_signal);
}

#[test]
fn sprint180_shadow_opinion_run_preserves_decisions_scores_and_owner_debug_safety() {
    let (batch_result, _debug_output_batch, _shadow_alignment, _dataset, recalibration_result) =
        sprint179_recalibration_fixture(false);
    let before = batch_result.clone();
    let run_result = core::run_smartcore_shadow_opinion_lane(
        &recalibration_result.calibrated_debug_output_batch,
        &batch_result,
        Some(&batch_result.replay_dataset),
        &core::SmartCoreShadowOpinionRunConfig {
            run_id: "sprint180-shadow-opinion-safety".to_string(),
            enabled: true,
            output_path: None,
            include_member_opinion_comparison: true,
            include_target_eval: true,
            emit_owner_debug_summary: true,
            paper_only: true,
        },
    )
    .expect("shadow opinion safety run");
    let owner_debug = run_result
        .owner_debug_section
        .as_ref()
        .expect("owner debug section");

    assert_eq!(batch_result, before);
    assert!(run_result.safety_summary.no_model_training);
    assert!(run_result.safety_summary.no_weight_update);
    assert!(run_result.safety_summary.no_checkpoint);
    assert!(run_result.safety_summary.no_live_inference);
    assert!(run_result.safety_summary.no_broker_order_account);
    assert!(
        owner_debug
            .cards
            .iter()
            .all(|card| card.not_investment_signal
                && card.not_committee_opinion
                && card.shadow_only)
    );
}

#[test]
fn sprint180_feedback_plan_is_safe_and_config_carries_flags() {
    let (_batch_result, _recalibration_result, first) = sprint180_shadow_opinion_fixture();
    let (_batch_result_again, _recalibration_result_again, second) =
        sprint180_shadow_opinion_fixture();
    let config = MinimalAiCommitteeCycleConfig::from_toml_path(std::path::Path::new(
        "examples/soma_minimal_ai_committee_core.toml",
    ))
    .expect("config");
    let run_config = config
        .load_autonomous_paper_run_config()
        .expect("autonomous config");

    assert_eq!(first, second);
    assert!(
        first
            .feedback_plan
            .feedback_needs
            .contains(&core::ShadowOpinionFeedbackNeed::DoNotUseForDecisionYet)
    );
    assert!(run_config.smartcore_shadow_opinion_enabled);
    assert!(run_config.smartcore_shadow_compare_member_opinion);
    assert!(run_config.smartcore_shadow_target_eval);
    assert!(run_config.smartcore_shadow_emit_owner_debug);
    assert_eq!(
        run_config.smartcore_shadow_opinion_output_path.as_deref(),
        Some("target/minimal_smartcore_shadow_opinion.json")
    );
}

#[test]
fn sprint181_stability_samples_collect_all_members_and_identical_runs_stay_stable() {
    let (_batch_result, _dataset, _shadow_opinion_run, stability_run) =
        sprint181_shadow_stability_fixture(true);

    assert_eq!(stability_run.repeated_run_count, 3);
    assert_eq!(stability_run.samples.len(), 9);
    assert_eq!(
        stability_run
            .samples
            .iter()
            .map(|sample| sample.member_id.clone())
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        3
    );
    assert_eq!(stability_run.stability_metrics.action_flip_rate, 0.0);
    assert_eq!(stability_run.stability_metrics.head_bucket_flip_rate, 0.0);
}

#[test]
fn sprint181_injected_action_flip_is_detected() {
    let (_batch_result, _dataset, _shadow_opinion_run, stability_run) =
        sprint181_shadow_stability_fixture(true);
    let mut samples = stability_run.samples.clone();
    let flipped_member = samples[0].member_id.clone();
    let sample_to_flip = samples
        .iter_mut()
        .find(|sample| sample.member_id == flipped_member && sample.repeat_index == 1)
        .expect("repeat sample");
    sample_to_flip.shadow_action = core::SmartCoreShadowOpinionAction::ShadowNoTradeLike;
    let metrics = core::compute_shadow_stability_metrics(&samples);
    let mismatches = core::build_shadow_stability_mismatch_records(&samples, &metrics);

    assert!(metrics.action_flip_rate > 0.0);
    assert!(mismatches.iter().any(|record| {
        record.mismatch_kind == core::SmartCoreShadowStabilityMismatchKind::ActionFlip
    }));
}

#[test]
fn sprint181_injected_head_bucket_flip_is_detected() {
    let (_batch_result, _dataset, _shadow_opinion_run, stability_run) =
        sprint181_shadow_stability_fixture(true);
    let mut samples = stability_run.samples.clone();
    let flipped_member = samples[0].member_id.clone();
    let sample_to_flip = samples
        .iter_mut()
        .find(|sample| sample.member_id == flipped_member && sample.repeat_index == 1)
        .expect("repeat sample");
    sample_to_flip.risk_bucket = Some("RiskHigh".to_string());
    let metrics = core::compute_shadow_stability_metrics(&samples);
    let mismatches = core::build_shadow_stability_mismatch_records(&samples, &metrics);

    assert!(metrics.head_bucket_flip_rate > 0.0);
    assert!(mismatches.iter().any(|record| {
        record.mismatch_kind == core::SmartCoreShadowStabilityMismatchKind::RiskBucketFlip
    }));
}

#[test]
fn sprint181_agreement_target_expansion_grows_target_count_with_calibration_targets() {
    let (_batch_result, _dataset, _shadow_opinion_run, stability_run) =
        sprint181_shadow_stability_fixture(true);

    assert!(stability_run.target_expansion_result.new_target_count > 0);
    assert!(
        stability_run.target_expansion_result.new_target_count
            >= stability_run.target_expansion_result.previous_target_count
    );
    assert!(stability_run.target_expansion_result.added_target_count > 0);
}

#[test]
fn sprint181_target_quality_detects_thin_head_coverage_and_builds_collection_tasks() {
    let thin_targets = vec![
        core::SmartCoreShadowAlignmentTarget {
            target_id: "thin-stance-target-1".to_string(),
            member_id: "trend-kr-short".to_string(),
            symbol: Some("005930.KS".to_string()),
            market_scope: Some(MarketScope::KoreaShortTerm),
            source_type: core::SmartCoreShadowAlignmentTargetSourceType::ReplayTargetLabel,
            stance_target: Some(core::SmartCoreHeadBucketNormalizedValue::PositiveLike),
            risk_target: None,
            evidence_target: None,
            confidence_target: None,
            outcome_target: None,
            paper_only: true,
        },
        core::SmartCoreShadowAlignmentTarget {
            target_id: "thin-stance-target-2".to_string(),
            member_id: "risk-kr-short".to_string(),
            symbol: Some("005930.KS".to_string()),
            market_scope: Some(MarketScope::KoreaShortTerm),
            source_type: core::SmartCoreShadowAlignmentTargetSourceType::ReplayTargetLabel,
            stance_target: Some(core::SmartCoreHeadBucketNormalizedValue::NeutralLike),
            risk_target: None,
            evidence_target: None,
            confidence_target: None,
            outcome_target: None,
            paper_only: true,
        },
        core::SmartCoreShadowAlignmentTarget {
            target_id: "thin-stance-target-3".to_string(),
            member_id: "evidence-kr-short".to_string(),
            symbol: Some("005930.KS".to_string()),
            market_scope: Some(MarketScope::KoreaShortTerm),
            source_type: core::SmartCoreShadowAlignmentTargetSourceType::ReplayTargetLabel,
            stance_target: Some(core::SmartCoreHeadBucketNormalizedValue::NegativeLike),
            risk_target: None,
            evidence_target: None,
            confidence_target: None,
            outcome_target: None,
            paper_only: true,
        },
    ];
    let quality = core::summarize_smartcore_agreement_target_quality(&thin_targets);
    let queue = core::build_agreement_target_collection_queue(&quality);

    assert_eq!(
        quality.target_quality_status,
        core::SmartCoreAgreementTargetQualityStatus::ThinHeadCoverage
    );
    assert!(
        queue
            .tasks
            .iter()
            .any(|task| { task.target_need == core::SmartCoreAgreementTargetNeed::RiskTarget })
    );
    assert!(
        queue
            .tasks
            .iter()
            .any(|task| { task.target_need == core::SmartCoreAgreementTargetNeed::EvidenceTarget })
    );
}

#[test]
fn sprint181_owner_stability_summary_is_safe_and_regression_passes() {
    let (_batch_result, _dataset, _shadow_opinion_run, stability_run) =
        sprint181_shadow_stability_fixture(true);
    let owner_summary = core::build_owner_shadow_stability_debug_summary(&stability_run);

    assert_eq!(
        stability_run.decision_isolation_guard.regression_status,
        core::SmartCoreShadowStabilityDecisionIsolationRegressionStatus::Preserved
    );
    assert!(owner_summary.not_investment_signal);
    assert!(owner_summary.not_committee_opinion);
    assert!(owner_summary.debug_only);
}

#[test]
fn sprint181_regression_fails_if_agreement_target_enters_input_features() {
    let (batch_result, _dataset, _shadow_opinion_run, stability_run) =
        sprint181_shadow_stability_fixture(true);
    let mut leaked = batch_result.clone();
    let leaked_target_id = stability_run.target_expansion_result.targets[0]
        .target_id
        .clone();
    leaked.replay_dataset.examples[0]
        .input_features
        .market_data_summary = format!(
        "{} {leaked_target_id}",
        leaked.replay_dataset.examples[0]
            .input_features
            .market_data_summary
    );
    let regression = core::evaluate_shadow_stability_decision_isolation_regression(
        &stability_run,
        &batch_result,
        Some(&leaked),
    );

    assert_eq!(
        regression.regression_status,
        core::SmartCoreShadowStabilityDecisionIsolationRegressionStatus::Violated
    );
    assert!(regression.agreement_target_used_as_input_feature);
}

#[test]
fn sprint181_stability_run_preserves_decisions_scores_and_is_deterministic_with_config_flags() {
    let (batch_result, dataset, shadow_opinion_run, first) =
        sprint181_shadow_stability_fixture(true);
    let before = batch_result.clone();
    let second = core::run_smartcore_shadow_stability_eval(
        &shadow_opinion_run,
        &batch_result,
        Some(&batch_result.replay_dataset),
        Some(&dataset),
        None,
        &core::SmartCoreShadowStabilityRunConfig {
            run_id: "sprint181-shadow-stability".to_string(),
            enabled: true,
            repeated_run_count: 3,
            include_same_input_repeat: true,
            include_calibrated_output_repeat: true,
            include_shadow_candidate_repeat: true,
            include_target_eval_repeat: true,
            max_allowed_action_flip_rate: 0.0,
            max_allowed_head_flip_rate: 0.0,
            output_path: None,
            paper_only: true,
        },
    )
    .expect("second stability run");
    let config = MinimalAiCommitteeCycleConfig::from_toml_path(std::path::Path::new(
        "examples/soma_minimal_ai_committee_core.toml",
    ))
    .expect("config");
    let run_config = config
        .load_autonomous_paper_run_config()
        .expect("autonomous config");

    assert_eq!(batch_result, before);
    assert_eq!(first, second);
    assert!(first.safety_summary.no_model_training);
    assert!(first.safety_summary.no_weight_update);
    assert!(first.safety_summary.no_checkpoint);
    assert!(first.safety_summary.no_live_inference);
    assert!(first.safety_summary.no_broker_order_account);
    assert!(run_config.smartcore_shadow_stability_enabled);
    assert_eq!(run_config.smartcore_shadow_stability_repeats, 3);
    assert!(run_config.smartcore_shadow_expand_agreement_targets);
    assert!(run_config.smartcore_shadow_stability_emit_owner_summary);
    assert_eq!(
        run_config.smartcore_shadow_stability_output_path.as_deref(),
        Some("target/minimal_smartcore_shadow_stability.json")
    );
    assert_eq!(
        run_config
            .smartcore_shadow_target_collection_queue_output_path
            .as_deref(),
        Some("target/minimal_smartcore_target_collection_queue.json")
    );
}

#[test]
fn sprint182_scenario_set_loads_local_json_and_rejects_remote_path() {
    let path = sprint171_temp_json_path("sprint182-shadow-scenarios");
    let scenario_set = core::build_default_shadow_scenario_set();
    std::fs::write(
        &path,
        serde_json::to_string_pretty(&scenario_set).expect("scenario json"),
    )
    .expect("write scenario set");

    let loaded = core::load_shadow_scenario_set_from_local_json(&path).expect("load scenario set");
    assert_eq!(loaded.scenario_count, 5);
    assert_eq!(loaded.scenarios[0].scenario_id, "baseline");

    let remote = core::load_shadow_scenario_set_from_local_json(std::path::Path::new(
        "https://example.com/shadow-scenarios.json",
    ));
    assert!(remote.is_err());
}

#[test]
fn sprint182_scenario_validation_rejects_broker_order_account_fields() {
    let path = sprint171_temp_json_path("sprint182-shadow-scenarios-unsafe");
    std::fs::write(
        &path,
        r#"{"scenario_set_id":"unsafe","scenario_count":1,"scenarios":[{"scenario_id":"unsafe-order","scenario_kind":"Baseline","order_id":"forbidden","paper_only":true}],"paper_only":true}"#,
    )
    .expect("write unsafe scenario set");

    let error = core::load_shadow_scenario_set_from_local_json(&path).expect_err("unsafe scenario");
    assert!(error.contains("unsafe"));
}

#[test]
fn sprint182_same_scenario_repeated_has_zero_action_and_head_flip_rate() {
    let (batch_result, dataset, calibrated_debug_batch) = sprint182_shadow_scenario_context();
    let scenario = core::build_default_shadow_scenario_set().scenarios[0].clone();
    let result = core::run_shadow_stability_for_scenario(
        &scenario,
        &calibrated_debug_batch,
        &batch_result,
        &batch_result.replay_dataset,
        Some(&dataset),
        None,
        &core::SmartCoreShadowScenarioSweepConfig {
            run_id: "sprint182-single-scenario".to_string(),
            scenario_set_path: None,
            repeated_run_count: 3,
            max_scenarios: 5,
            include_same_input_determinism: true,
            include_cross_scenario_sensitivity: true,
            expand_targets_per_scenario: true,
            output_path: None,
            paper_only: true,
        },
    )
    .expect("scenario stability");

    assert_eq!(result.action_flip_rate, 0.0);
    assert_eq!(result.head_bucket_flip_rate, 0.0);
}

#[test]
fn sprint182_injected_same_input_action_flip_fails_determinism() {
    let (_batch_result, _dataset, _shadow_opinion_run, stability_run) =
        sprint181_shadow_stability_fixture(true);
    let mut samples = stability_run.samples.clone();
    let member_id = samples[0].member_id.clone();
    let sample = samples
        .iter_mut()
        .find(|sample| sample.member_id == member_id && sample.repeat_index == 1)
        .expect("flip sample");
    sample.shadow_action = core::SmartCoreShadowOpinionAction::ShadowRiskWarning;

    let metrics = core::compute_shadow_stability_metrics(&samples);
    assert_ne!(
        metrics.deterministic_status,
        core::SmartCoreShadowStabilityDeterministicStatus::Deterministic
    );
}

#[test]
fn sprint182_cross_scenario_sensitivity_detects_too_static_and_too_unstable() {
    let too_static = core::compute_cross_scenario_sensitivity(&[
        sprint182_scenario_result(
            "baseline",
            core::SmartCoreShadowScenarioKind::Baseline,
            core::SmartCoreShadowOpinionAction::ShadowHoldLike,
            "NeutralLike|RiskLow|EvidenceSufficient|ConfidenceMedium|ConfidenceMedium",
            core::SmartCoreAgreementTargetQualityStatus::Sufficient,
        ),
        sprint182_scenario_result(
            "neutral",
            core::SmartCoreShadowScenarioKind::NeutralWatchlist,
            core::SmartCoreShadowOpinionAction::ShadowHoldLike,
            "NeutralLike|RiskLow|EvidenceSufficient|ConfidenceMedium|ConfidenceMedium",
            core::SmartCoreAgreementTargetQualityStatus::Sufficient,
        ),
    ]);
    assert_eq!(
        too_static.sensitivity_status,
        core::SmartCoreCrossScenarioSensitivityStatus::TooStatic
    );

    let too_unstable = core::compute_cross_scenario_sensitivity(&[
        sprint182_scenario_result(
            "trend-a",
            core::SmartCoreShadowScenarioKind::PositiveTrend,
            core::SmartCoreShadowOpinionAction::ShadowBuyLike,
            "PositiveLike|RiskLow|EvidenceSufficient|ConfidenceHigh|ConfidenceLow",
            core::SmartCoreAgreementTargetQualityStatus::Sufficient,
        ),
        sprint182_scenario_result(
            "trend-b",
            core::SmartCoreShadowScenarioKind::NegativeTrend,
            core::SmartCoreShadowOpinionAction::ShadowNoTradeLike,
            "NegativeLike|RiskMedium|EvidenceSufficient|ConfidenceMedium|ConfidenceMedium",
            core::SmartCoreAgreementTargetQualityStatus::Sufficient,
        ),
    ]);
    assert_eq!(
        too_unstable.sensitivity_status,
        core::SmartCoreCrossScenarioSensitivityStatus::TooUnstable
    );
}

#[test]
fn sprint182_target_coverage_stress_detects_thin_and_sufficient_coverage() {
    let thin = core::run_smartcore_target_coverage_stress_test(&[sprint182_scenario_result(
        "thin",
        core::SmartCoreShadowScenarioKind::EvidenceGap,
        core::SmartCoreShadowOpinionAction::ShadowNeedMoreEvidence,
        "NeutralLike|RiskMedium|NeedMoreEvidence|ConfidenceLow|ConfidenceHigh",
        core::SmartCoreAgreementTargetQualityStatus::ThinHeadCoverage,
    )]);
    assert_eq!(
        thin.target_coverage_status,
        core::SmartCoreTargetCoverageStatus::ThinCoverage
    );
    assert!(thin.head_coverage_failures > 0);

    let sufficient = core::run_smartcore_target_coverage_stress_test(&[
        sprint182_scenario_result(
            "baseline",
            core::SmartCoreShadowScenarioKind::Baseline,
            core::SmartCoreShadowOpinionAction::ShadowHoldLike,
            "NeutralLike|RiskLow|EvidenceSufficient|ConfidenceMedium|ConfidenceMedium",
            core::SmartCoreAgreementTargetQualityStatus::Sufficient,
        ),
        sprint182_scenario_result(
            "positive",
            core::SmartCoreShadowScenarioKind::PositiveTrend,
            core::SmartCoreShadowOpinionAction::ShadowBuyLike,
            "PositiveLike|RiskLow|EvidenceSufficient|ConfidenceHigh|ConfidenceLow",
            core::SmartCoreAgreementTargetQualityStatus::Sufficient,
        ),
    ]);
    assert_eq!(
        sufficient.target_coverage_status,
        core::SmartCoreTargetCoverageStatus::Sufficient
    );
}

#[test]
fn sprint182_observer_readiness_allows_blocks_leak_and_blocks_instability() {
    let mut sweep = core::SmartCoreShadowScenarioSweepResult {
        run_id: "sprint182-observer".to_string(),
        scenario_results: vec![
            sprint182_scenario_result(
                "baseline",
                core::SmartCoreShadowScenarioKind::Baseline,
                core::SmartCoreShadowOpinionAction::ShadowHoldLike,
                "NeutralLike|RiskLow|EvidenceSufficient|ConfidenceMedium|ConfidenceMedium",
                core::SmartCoreAgreementTargetQualityStatus::Sufficient,
            ),
            sprint182_scenario_result(
                "positive",
                core::SmartCoreShadowScenarioKind::PositiveTrend,
                core::SmartCoreShadowOpinionAction::ShadowBuyLike,
                "PositiveLike|RiskLow|EvidenceSufficient|ConfidenceHigh|ConfidenceLow",
                core::SmartCoreAgreementTargetQualityStatus::Sufficient,
            ),
            sprint182_scenario_result(
                "high-risk",
                core::SmartCoreShadowScenarioKind::HighRisk,
                core::SmartCoreShadowOpinionAction::ShadowRiskWarning,
                "NeutralLike|RiskHigh|EvidenceSufficient|ConfidenceMedium|ConfidenceMedium",
                core::SmartCoreAgreementTargetQualityStatus::Sufficient,
            ),
        ],
        cross_scenario_sensitivity: core::SmartCoreCrossScenarioSensitivityMetrics {
            scenario_count: 3,
            member_count: 3,
            action_variation_count: 2,
            head_variation_count: 2,
            expected_variation_count: 2,
            suspicious_variation_count: 0,
            per_member_variation: std::collections::BTreeMap::new(),
            per_head_variation: std::collections::BTreeMap::new(),
            sensitivity_status:
                core::SmartCoreCrossScenarioSensitivityStatus::ReasonableSensitivity,
            paper_only: true,
        },
        target_coverage_stress: core::SmartCoreTargetCoverageStressResult {
            scenario_count: 3,
            scenarios_with_sufficient_targets: 3,
            scenarios_with_thin_targets: 0,
            member_coverage_failures: 0,
            head_coverage_failures: 0,
            source_coverage_failures: 0,
            target_quality_by_scenario: std::collections::BTreeMap::new(),
            target_coverage_status: core::SmartCoreTargetCoverageStatus::Sufficient,
            recommended_target_collection_tasks: Vec::new(),
            paper_only: true,
        },
        aggregate_action_flip_rate: 0.0,
        aggregate_head_flip_rate: 0.0,
        deterministic_scenario_count: 3,
        unstable_scenario_count: 0,
        decision_isolation_failures: 0,
        sweep_status: core::SmartCoreShadowScenarioSweepStatus::Passed,
        warnings: Vec::new(),
        observer_readiness_gate: None,
        decision_isolation_regression: core::SmartCoreMultiScenarioDecisionIsolationRegression {
            scenario_count: 3,
            scenario_decision_mutation_detected: false,
            scenario_member_score_mutation_detected: false,
            target_leakage_detected: false,
            shadow_output_used_as_order: false,
            regression_status:
                core::SmartCoreMultiScenarioDecisionIsolationRegressionStatus::Preserved,
            violations: Vec::new(),
            paper_only: true,
        },
        owner_debug_summary: None,
        paper_only: true,
    };
    let policy = core::SmartCoreObserverReadinessPolicy {
        min_scenarios_required: 3,
        require_zero_same_input_flip_rate: true,
        allow_reasonable_cross_scenario_variation: true,
        require_decision_isolation: true,
        require_no_training: true,
        require_no_live_inference: true,
        paper_only: true,
    };
    let allowed = core::evaluate_smartcore_observer_readiness(&sweep, &policy);
    assert!(allowed.observer_lane_allowed);

    sweep.decision_isolation_regression.regression_status =
        core::SmartCoreMultiScenarioDecisionIsolationRegressionStatus::Violated;
    sweep.decision_isolation_regression.violations =
        vec!["baseline: shadow stability changed member score".to_string()];
    let blocked_by_leak = core::evaluate_smartcore_observer_readiness(&sweep, &policy);
    assert_eq!(
        blocked_by_leak.observer_status,
        core::SmartCoreObserverStatus::BlockedByDecisionLeak
    );

    sweep.decision_isolation_regression.regression_status =
        core::SmartCoreMultiScenarioDecisionIsolationRegressionStatus::Preserved;
    sweep.decision_isolation_regression.violations.clear();
    sweep.cross_scenario_sensitivity.sensitivity_status =
        core::SmartCoreCrossScenarioSensitivityStatus::TooUnstable;
    let blocked_by_instability = core::evaluate_smartcore_observer_readiness(&sweep, &policy);
    assert_eq!(
        blocked_by_instability.observer_status,
        core::SmartCoreObserverStatus::BlockedByInstability
    );
}

#[test]
fn sprint182_owner_summary_and_multi_scenario_regression_are_safe() {
    let (_batch_result, sweep) = sprint182_shadow_scenario_sweep_fixture();
    let owner_summary = core::build_owner_shadow_scenario_sweep_debug_summary(
        &sweep,
        sweep
            .observer_readiness_gate
            .as_ref()
            .expect("observer gate in fixture"),
    );
    let regression = core::evaluate_multi_scenario_decision_isolation_regression(&sweep);

    assert!(owner_summary.not_investment_signal);
    assert!(owner_summary.not_committee_opinion);
    assert_eq!(
        regression.regression_status,
        core::SmartCoreMultiScenarioDecisionIsolationRegressionStatus::Preserved
    );
}

#[test]
fn sprint182_multi_scenario_regression_detects_score_mutation() {
    let mut sweep = core::SmartCoreShadowScenarioSweepResult {
        run_id: "sprint182-regression".to_string(),
        scenario_results: vec![{
            let mut result = sprint182_scenario_result(
                "baseline",
                core::SmartCoreShadowScenarioKind::Baseline,
                core::SmartCoreShadowOpinionAction::ShadowHoldLike,
                "NeutralLike|RiskLow|EvidenceSufficient|ConfidenceMedium|ConfidenceMedium",
                core::SmartCoreAgreementTargetQualityStatus::Sufficient,
            );
            result.decision_isolation_violations =
                vec!["shadow stability changed member score".to_string()];
            result.decision_isolation_status =
                core::SmartCoreShadowStabilityDecisionIsolationRegressionStatus::Violated;
            result
        }],
        cross_scenario_sensitivity: core::SmartCoreCrossScenarioSensitivityMetrics {
            scenario_count: 1,
            member_count: 3,
            action_variation_count: 0,
            head_variation_count: 0,
            expected_variation_count: 0,
            suspicious_variation_count: 0,
            per_member_variation: std::collections::BTreeMap::new(),
            per_head_variation: std::collections::BTreeMap::new(),
            sensitivity_status:
                core::SmartCoreCrossScenarioSensitivityStatus::InsufficientScenarios,
            paper_only: true,
        },
        target_coverage_stress: core::SmartCoreTargetCoverageStressResult {
            scenario_count: 1,
            scenarios_with_sufficient_targets: 1,
            scenarios_with_thin_targets: 0,
            member_coverage_failures: 0,
            head_coverage_failures: 0,
            source_coverage_failures: 0,
            target_quality_by_scenario: std::collections::BTreeMap::new(),
            target_coverage_status: core::SmartCoreTargetCoverageStatus::Sufficient,
            recommended_target_collection_tasks: Vec::new(),
            paper_only: true,
        },
        aggregate_action_flip_rate: 0.0,
        aggregate_head_flip_rate: 0.0,
        deterministic_scenario_count: 1,
        unstable_scenario_count: 0,
        decision_isolation_failures: 1,
        sweep_status: core::SmartCoreShadowScenarioSweepStatus::Failed,
        warnings: Vec::new(),
        observer_readiness_gate: None,
        decision_isolation_regression: core::SmartCoreMultiScenarioDecisionIsolationRegression {
            scenario_count: 1,
            scenario_decision_mutation_detected: false,
            scenario_member_score_mutation_detected: false,
            target_leakage_detected: false,
            shadow_output_used_as_order: false,
            regression_status:
                core::SmartCoreMultiScenarioDecisionIsolationRegressionStatus::Preserved,
            violations: Vec::new(),
            paper_only: true,
        },
        owner_debug_summary: None,
        paper_only: true,
    };
    let regression = core::evaluate_multi_scenario_decision_isolation_regression(&sweep);

    assert_eq!(
        regression.regression_status,
        core::SmartCoreMultiScenarioDecisionIsolationRegressionStatus::Violated
    );
    assert!(regression.scenario_member_score_mutation_detected);
    sweep.decision_isolation_regression = regression;
}

#[test]
fn sprint182_sweep_preserves_decisions_scores_safety_and_is_deterministic() {
    let (batch_result, first) = sprint182_shadow_scenario_sweep_fixture();
    let before = batch_result.clone();
    let (_second_batch_result, second) = sprint182_shadow_scenario_sweep_fixture();
    let config = MinimalAiCommitteeCycleConfig::from_toml_path(std::path::Path::new(
        "examples/soma_minimal_ai_committee_core.toml",
    ))
    .expect("config");
    let run_config = config
        .load_autonomous_paper_run_config()
        .expect("autonomous config");

    assert_eq!(batch_result, before);
    assert_eq!(first.scenario_results, second.scenario_results);
    assert_eq!(
        first.aggregate_action_flip_rate,
        second.aggregate_action_flip_rate
    );
    assert_eq!(
        first.aggregate_head_flip_rate,
        second.aggregate_head_flip_rate
    );
    assert!(batch_result.safety_summary.no_model_training);
    assert!(batch_result.safety_summary.no_weight_update);
    assert!(batch_result.safety_summary.no_checkpoint);
    assert!(batch_result.safety_summary.no_live_inference);
    assert!(batch_result.safety_summary.no_broker_order_account);
    assert!(run_config.smartcore_shadow_scenario_sweep_enabled);
    assert_eq!(
        run_config.smartcore_shadow_scenario_set_path.as_deref(),
        Some("examples/smartcore_shadow_scenarios.sample.json")
    );
    assert_eq!(run_config.smartcore_shadow_scenario_repeats, 3);
    assert_eq!(run_config.smartcore_shadow_scenario_max_count, 5);
    assert!(run_config.smartcore_observer_readiness_gate_enabled);
    assert_eq!(run_config.smartcore_observer_min_scenarios_required, 3);
    assert!(run_config.smartcore_shadow_scenario_emit_owner_summary);
    assert_eq!(
        run_config.smartcore_shadow_scenario_output_path.as_deref(),
        Some("target/minimal_smartcore_shadow_scenario_sweep.json")
    );
}

#[test]
fn sprint183_observer_lane_policy_rejects_voting_and_observer_members_are_non_voting() {
    let mut policy = core::default_smartcore_observer_lane_policy();
    policy.allow_vote = true;
    let error = core::validate_smartcore_observer_lane_policy(&policy).expect_err("policy");
    assert!(error.contains("must not allow"));

    let (_batch_result, _recalibration_result, shadow_opinion_run) =
        sprint180_shadow_opinion_fixture();
    let observers = core::build_smartcore_observer_members(
        &shadow_opinion_run.candidate_batch,
        &core::default_three_member_canonical_id_map(),
    )
    .expect("observer members");

    assert!(
        observers
            .iter()
            .all(|observer| observer.voting_power == 0.0)
    );
    assert!(observers.iter().all(|observer| !observer.can_open_event));
    assert!(
        observers
            .iter()
            .all(|observer| !observer.can_join_committee_vote)
    );
    assert!(
        observers
            .iter()
            .all(|observer| !observer.can_change_chairman_decision)
    );
    assert!(
        observers
            .iter()
            .all(|observer| !observer.can_trigger_risk_governor)
    );
}

#[test]
fn sprint183_observed_snapshot_is_read_only_and_observer_lane_is_deterministic() {
    let (batch_result, first) = sprint183_observer_lane_fixture();
    let before = batch_result.clone();
    let (_second_batch_result, second) = sprint183_observer_lane_fixture();

    assert_eq!(batch_result, before);
    assert!(first.observed_snapshot.paper_only);
    assert_eq!(first.observer_members, second.observer_members);
    assert_eq!(first.comparison_records, second.comparison_records);
    assert_eq!(first.disagreement_records, second.disagreement_records);
}

#[test]
fn sprint183_observer_comparison_records_agreements_and_shadow_buy_vs_risk_veto() {
    let (_batch_result, run_result) = sprint183_observer_lane_fixture();
    assert!(run_result.comparison_records.iter().any(|record| matches!(
        record.agreement_with_member,
        core::ObserverCommitteeAgreement::Agree
            | core::ObserverCommitteeAgreement::PartiallyAgree
            | core::ObserverCommitteeAgreement::Disagree
    )));
    assert!(
        run_result
            .comparison_records
            .iter()
            .any(|record| record.agreement_with_chairman
                != core::ObserverCommitteeAgreement::Unknown)
    );
    assert!(
        run_result
            .comparison_records
            .iter()
            .any(|record| record.agreement_with_risk_governor
                != core::ObserverCommitteeAgreement::Unknown)
    );

    let (batch_result, _recalibration_result, shadow_opinion_run) =
        sprint180_shadow_opinion_fixture();
    let observers = core::build_smartcore_observer_members(
        &shadow_opinion_run.candidate_batch,
        &core::default_three_member_canonical_id_map(),
    )
    .expect("observer members");
    let mut candidate = shadow_opinion_run.candidate_batch.candidates[0].clone();
    candidate.shadow_action = core::SmartCoreShadowOpinionAction::ShadowBuyLike;
    let mut vetoed = batch_result.clone();
    for decision in &mut vetoed.chairman_decisions {
        decision.risk_governor_status = RiskGovernorStatus::Vetoed;
    }
    let snapshot = core::build_observed_committee_cycle_snapshot(&vetoed);
    let record = core::compare_observer_to_committee(
        &observers[0],
        &candidate,
        &snapshot,
        &vetoed,
        true,
        true,
        true,
    );
    let disagreements = core::build_observer_disagreement_records(&[record.clone()]);

    assert_eq!(
        record.disagreement_type,
        core::ObserverDisagreementType::ShadowBuyVsRiskVeto
    );
    assert_eq!(disagreements[0].disagreement_type, record.disagreement_type);
}

#[test]
fn sprint183_observer_member_comparison_matches_canonical_aliases() {
    let (batch_result, _recalibration_result, shadow_opinion_run) =
        sprint180_shadow_opinion_fixture();
    let mut alias_batch = shadow_opinion_run.candidate_batch.clone();
    alias_batch
        .candidates
        .retain(|candidate| candidate.member_id == "trend-kr-short");
    alias_batch.candidates[0].member_id = "TrendEntryAI".to_string();
    alias_batch.candidate_count = alias_batch.candidates.len();
    alias_batch.member_count = 1;
    let observers = core::build_smartcore_observer_members(
        &alias_batch,
        &core::default_three_member_canonical_id_map(),
    )
    .expect("observer members");
    let snapshot = core::build_observed_committee_cycle_snapshot(&batch_result);
    let record = core::compare_observer_to_committee(
        &observers[0],
        &alias_batch.candidates[0],
        &snapshot,
        &batch_result,
        true,
        false,
        false,
    );

    assert_eq!(observers[0].canonical_member_id, "trend-kr-short");
    assert!(record.member_action.is_some());
    assert_ne!(
        record.agreement_with_member,
        core::ObserverCommitteeAgreement::Unknown
    );
    assert_ne!(
        record.agreement_with_member,
        core::ObserverCommitteeAgreement::NotComparable
    );
}

#[test]
fn sprint183_target_coverage_closure_queue_builds_risk_and_evidence_items() {
    let stress = core::run_smartcore_target_coverage_stress_test(&[sprint182_scenario_result(
        "thin",
        core::SmartCoreShadowScenarioKind::EvidenceGap,
        core::SmartCoreShadowOpinionAction::ShadowNeedMoreEvidence,
        "NeutralLike|RiskMedium|NeedMoreEvidence|ConfidenceLow|ConfidenceHigh",
        core::SmartCoreAgreementTargetQualityStatus::ThinHeadCoverage,
    )]);
    let disagreements = vec![
        core::SmartCoreObserverDisagreementRecord {
            disagreement_id: "risk-gap".to_string(),
            observer_id: "observer-risk".to_string(),
            source_member_id: "risk-kr-short".to_string(),
            symbol: Some("AAPL".to_string()),
            market_scope: Some(MarketScope::UsShortTerm),
            disagreement_type: core::ObserverDisagreementType::ShadowBuyVsRiskVeto,
            shadow_action: core::SmartCoreShadowOpinionAction::ShadowBuyLike,
            committee_reference: "risk_governor".to_string(),
            severity: core::ObserverComparisonSeverity::High,
            suggested_follow_up: core::SmartCoreObserverSuggestedFollowUp::CollectMoreRiskTargets,
            paper_only: true,
        },
        core::SmartCoreObserverDisagreementRecord {
            disagreement_id: "evidence-gap".to_string(),
            observer_id: "observer-evidence".to_string(),
            source_member_id: "evidence-kr-short".to_string(),
            symbol: Some("AAPL".to_string()),
            market_scope: Some(MarketScope::UsShortTerm),
            disagreement_type: core::ObserverDisagreementType::ShadowNeedEvidenceVsMemberBuy,
            shadow_action: core::SmartCoreShadowOpinionAction::ShadowNeedMoreEvidence,
            committee_reference: "member_opinion".to_string(),
            severity: core::ObserverComparisonSeverity::Medium,
            suggested_follow_up:
                core::SmartCoreObserverSuggestedFollowUp::CollectMoreEvidenceTargets,
            paper_only: true,
        },
    ];
    let queue = core::build_observer_target_coverage_closure_queue(&stress, &disagreements);

    assert!(queue.items.iter().any(|item| {
        item.target_need == core::ObserverTargetCoverageClosureNeed::MoreRiskTargets
    }));
    assert!(queue.items.iter().any(|item| {
        item.target_need == core::ObserverTargetCoverageClosureNeed::MoreEvidenceTargets
    }));
}

#[test]
fn sprint183_observer_safety_guard_passes_and_fails_on_score_or_order() {
    let (batch_result, run_result) = sprint183_observer_lane_fixture();
    assert_eq!(
        run_result.observer_safety_guard.guard_status,
        core::SmartCoreObserverLaneSafetyGuardStatus::Preserved
    );

    let mut score_changed = batch_result.clone();
    score_changed.score_updates[0].new_score += 0.1;
    let score_guard = core::evaluate_smartcore_observer_lane_safety(
        &run_result.observer_members,
        &batch_result,
        Some(&score_changed),
    );
    assert_eq!(
        score_guard.guard_status,
        core::SmartCoreObserverLaneSafetyGuardStatus::Violated
    );
    assert!(score_guard.observer_changed_member_score);

    let mut order_changed = batch_result.clone();
    order_changed.event_queue.events[0].event_id = format!(
        "{}-{}",
        order_changed.event_queue.events[0].event_id, run_result.observer_members[0].observer_id
    );
    let order_guard = core::evaluate_smartcore_observer_lane_safety(
        &run_result.observer_members,
        &batch_result,
        Some(&order_changed),
    );
    assert!(order_guard.observer_created_order);
}

#[test]
fn sprint183_owner_section_and_readiness_recheck_are_non_voting_read_only() {
    let (_batch_result, run_result) = sprint183_observer_lane_fixture();
    let owner_section = run_result
        .owner_observer_section
        .as_ref()
        .expect("owner observer section");
    let readiness = run_result
        .readiness_recheck
        .as_ref()
        .expect("readiness recheck");

    assert!(
        owner_section
            .cards
            .iter()
            .all(|card| card.voting_power == 0.0)
    );
    assert!(owner_section.cards.iter().all(|card| card.non_voting));
    assert!(owner_section.cards.iter().all(|card| card.read_only));
    assert_eq!(
        readiness.readiness_status,
        core::SmartCoreObserverLaneReadinessStatus::NonVotingObserverReadyWithWarnings
    );
}

#[test]
fn sprint183_observer_lane_preserves_safety_and_config_flags() {
    let (batch_result, run_result) = sprint183_observer_lane_fixture();
    let config = MinimalAiCommitteeCycleConfig::from_toml_path(std::path::Path::new(
        "examples/soma_minimal_ai_committee_core.toml",
    ))
    .expect("config");
    let run_config = config
        .load_autonomous_paper_run_config()
        .expect("autonomous config");

    assert!(batch_result.safety_summary.no_model_training);
    assert!(batch_result.safety_summary.no_weight_update);
    assert!(batch_result.safety_summary.no_checkpoint);
    assert!(batch_result.safety_summary.no_live_inference);
    assert!(batch_result.safety_summary.no_broker_order_account);
    assert!(run_result.paper_only);
    assert!(
        !run_result
            .observer_safety_guard
            .observer_changed_member_opinion
    );
    assert!(
        !run_result
            .observer_safety_guard
            .observer_changed_member_score
    );
    assert!(
        !run_result
            .observer_safety_guard
            .observer_changed_voice_weight
    );
    assert!(
        !run_result
            .observer_safety_guard
            .observer_created_trade_signal
    );
    assert!(!run_result.observer_safety_guard.observer_created_order);
    assert!(
        !run_result
            .observer_safety_guard
            .observer_touched_broker_order_account
    );
    assert!(run_config.smartcore_observer_lane_enabled);
    assert_eq!(
        run_config.smartcore_observer_output_path.as_deref(),
        Some("target/minimal_smartcore_observer_lane.json")
    );
    assert!(run_config.smartcore_observer_compare_member_opinion);
    assert!(run_config.smartcore_observer_compare_chairman);
    assert!(run_config.smartcore_observer_compare_risk_governor);
    assert!(run_config.smartcore_observer_target_coverage_closure_enabled);
    assert!(run_config.smartcore_observer_emit_owner_section);
}

#[test]
fn sprint184_closure_policy_rejects_news_only_and_low_trust_targets() {
    let policy = core::default_observer_target_closure_execution_policy();
    let news_only = core::ObserverAgreementTargetRecord {
        target_id: "news-only".to_string(),
        source_closure_item_id: None,
        source_record_id: None,
        member_id: Some("evidence-kr-short".to_string()),
        canonical_member_id: Some("evidence-kr-short".to_string()),
        symbol: Some("AAPL".to_string()),
        market_scope: Some(MarketScope::UsShortTerm),
        head: core::ObserverAgreementTargetHead::EvidenceNeed,
        target_bucket: core::SmartCoreHeadBucketNormalizedValue::NeedMoreEvidence,
        source_type: core::ObserverAgreementTargetSource::ResearchEvidence,
        source_confidence: core::SourceConfidence::High,
        approval_status: core::ObserverAgreementTargetApprovalStatus::Candidate,
        reason: "news only evidence".to_string(),
        eval_only: true,
        not_input_feature: true,
        paper_only: true,
    };
    let low_trust = core::ObserverAgreementTargetRecord {
        target_id: "low-trust".to_string(),
        source_type: core::ObserverAgreementTargetSource::MemberOpinion,
        source_confidence: core::SourceConfidence::Low,
        head: core::ObserverAgreementTargetHead::Stance,
        target_bucket: core::SmartCoreHeadBucketNormalizedValue::PositiveLike,
        member_id: Some("trend-kr-short".to_string()),
        canonical_member_id: Some("trend-kr-short".to_string()),
        symbol: Some("AAPL".to_string()),
        market_scope: Some(MarketScope::UsShortTerm),
        source_closure_item_id: None,
        source_record_id: None,
        approval_status: core::ObserverAgreementTargetApprovalStatus::Candidate,
        reason: "low trust observer target".to_string(),
        eval_only: true,
        not_input_feature: true,
        paper_only: true,
    };

    let news_validation = core::validate_observer_agreement_target(&news_only, &policy);
    let low_trust_validation = core::validate_observer_agreement_target(&low_trust, &policy);

    assert_eq!(
        news_validation.approval_status,
        core::ObserverAgreementTargetApprovalStatus::Rejected
    );
    assert_eq!(
        low_trust_validation.approval_status,
        core::ObserverAgreementTargetApprovalStatus::NeedsReview
    );
}

#[test]
fn sprint184_target_validation_approves_risk_and_stance_and_rejects_broker_terms() {
    let policy = core::default_observer_target_closure_execution_policy();
    let risk_target = core::ObserverAgreementTargetRecord {
        target_id: "risk-target".to_string(),
        source_closure_item_id: Some("risk-item".to_string()),
        source_record_id: Some("batch".to_string()),
        member_id: Some("risk-kr-short".to_string()),
        canonical_member_id: Some("risk-kr-short".to_string()),
        symbol: Some("AAPL".to_string()),
        market_scope: Some(MarketScope::UsShortTerm),
        head: core::ObserverAgreementTargetHead::Risk,
        target_bucket: core::SmartCoreHeadBucketNormalizedValue::RiskHigh,
        source_type: core::ObserverAgreementTargetSource::RiskGovernorStatus,
        source_confidence: core::SourceConfidence::High,
        approval_status: core::ObserverAgreementTargetApprovalStatus::Candidate,
        reason: "risk target".to_string(),
        eval_only: true,
        not_input_feature: true,
        paper_only: true,
    };
    let stance_target = core::ObserverAgreementTargetRecord {
        target_id: "stance-target".to_string(),
        source_closure_item_id: Some("stance-item".to_string()),
        source_record_id: Some("batch".to_string()),
        member_id: Some("trend-kr-short".to_string()),
        canonical_member_id: Some("trend-kr-short".to_string()),
        symbol: Some("AAPL".to_string()),
        market_scope: Some(MarketScope::UsShortTerm),
        head: core::ObserverAgreementTargetHead::Stance,
        target_bucket: core::SmartCoreHeadBucketNormalizedValue::PositiveLike,
        source_type: core::ObserverAgreementTargetSource::MemberOpinion,
        source_confidence: core::SourceConfidence::High,
        approval_status: core::ObserverAgreementTargetApprovalStatus::Candidate,
        reason: "member stance target".to_string(),
        eval_only: true,
        not_input_feature: true,
        paper_only: true,
    };
    let unsafe_target = core::ObserverAgreementTargetRecord {
        target_id: "unsafe-target".to_string(),
        reason: "broker order account path".to_string(),
        ..stance_target.clone()
    };

    assert_eq!(
        core::validate_observer_agreement_target(&risk_target, &policy).approval_status,
        core::ObserverAgreementTargetApprovalStatus::Approved
    );
    assert_eq!(
        core::validate_observer_agreement_target(&stance_target, &policy).approval_status,
        core::ObserverAgreementTargetApprovalStatus::Approved
    );
    assert_eq!(
        core::validate_observer_agreement_target(&unsafe_target, &policy).approval_status,
        core::ObserverAgreementTargetApprovalStatus::Rejected
    );
}

#[test]
fn sprint184_closure_executor_closes_risk_and_evidence_and_reviews_confidence_gap() {
    let (batch_result, _observer_run_result) = sprint183_observer_lane_fixture();
    let policy = core::default_observer_target_closure_execution_policy();
    let risk_item = core::ObserverTargetCoverageClosureItem {
        item_id: "risk-item".to_string(),
        scenario_id: None,
        member_id: Some("risk-kr-short".to_string()),
        symbol: Some("AAPL".to_string()),
        market_scope: Some(MarketScope::UsShortTerm),
        head: Some(core::SmartCoreShadowHeadKind::Risk),
        target_need: core::ObserverTargetCoverageClosureNeed::MoreRiskTargets,
        priority: core::ObserverTargetCoverageClosurePriority::High,
        reason: "need more risk targets".to_string(),
        paper_only: true,
    };
    let evidence_item = core::ObserverTargetCoverageClosureItem {
        item_id: "evidence-item".to_string(),
        scenario_id: None,
        member_id: Some("evidence-kr-short".to_string()),
        symbol: Some("AAPL".to_string()),
        market_scope: Some(MarketScope::UsShortTerm),
        head: Some(core::SmartCoreShadowHeadKind::EvidenceNeed),
        target_need: core::ObserverTargetCoverageClosureNeed::MoreEvidenceTargets,
        priority: core::ObserverTargetCoverageClosurePriority::High,
        reason: "need more evidence targets".to_string(),
        paper_only: true,
    };
    let confidence_gap_item = core::ObserverTargetCoverageClosureItem {
        item_id: "confidence-gap-item".to_string(),
        scenario_id: None,
        member_id: Some("missing-member".to_string()),
        symbol: Some("AAPL".to_string()),
        market_scope: Some(MarketScope::UsShortTerm),
        head: Some(core::SmartCoreShadowHeadKind::ConfidenceCalibration),
        target_need: core::ObserverTargetCoverageClosureNeed::MoreConfidenceTargets,
        priority: core::ObserverTargetCoverageClosurePriority::Normal,
        reason: "need more confidence targets".to_string(),
        paper_only: true,
    };

    let risk_result = core::execute_observer_target_coverage_closure_item(
        &risk_item,
        &batch_result,
        None,
        None,
        None,
        &policy,
    );
    let evidence_result = core::execute_observer_target_coverage_closure_item(
        &evidence_item,
        &batch_result,
        None,
        None,
        None,
        &policy,
    );
    let confidence_result = core::execute_observer_target_coverage_closure_item(
        &confidence_gap_item,
        &batch_result,
        None,
        None,
        None,
        &policy,
    );

    assert_eq!(
        risk_result.execution_status,
        core::ObserverTargetClosureExecutionStatus::Closed
    );
    assert_eq!(
        evidence_result.execution_status,
        core::ObserverTargetClosureExecutionStatus::Closed
    );
    assert_eq!(
        confidence_result.execution_status,
        core::ObserverTargetClosureExecutionStatus::NeedsReview
    );
}

#[test]
fn sprint184_closure_run_creates_targets_and_dry_run_writes_no_target_output() {
    let (batch_result, observer_run_result) = sprint183_observer_lane_fixture();
    let queue = observer_run_result
        .target_coverage_closure_result
        .as_ref()
        .expect("closure queue")
        .closure_queue
        .clone();
    let target_path = sprint171_temp_json_path("observer-target-dry-run");
    let _ = fs::remove_file(&target_path);

    let result = core::run_observer_target_coverage_closure(
        Some(&queue),
        &batch_result,
        None,
        None,
        None,
        &core::ObserverTargetCoverageClosureRunConfig {
            run_id: "closure-run".to_string(),
            enabled: true,
            closure_queue_input_path: None,
            observer_targets_output_path: None,
            replay_dataset_path: None,
            calibration_dataset_path: None,
            paper_evidence_path: None,
            dry_run: false,
            max_items: 16,
            paper_only: true,
        },
    )
    .expect("closure run");
    let dry_run = core::run_observer_target_coverage_closure(
        Some(&queue),
        &batch_result,
        None,
        None,
        None,
        &core::ObserverTargetCoverageClosureRunConfig {
            run_id: "closure-dry-run".to_string(),
            enabled: true,
            closure_queue_input_path: None,
            observer_targets_output_path: Some(target_path.to_string_lossy().to_string()),
            replay_dataset_path: None,
            calibration_dataset_path: None,
            paper_evidence_path: None,
            dry_run: true,
            max_items: 16,
            paper_only: true,
        },
    )
    .expect("dry run");

    assert!(result.approved_target_count > 0);
    assert!(!result.target_records.is_empty());
    assert_eq!(dry_run.approved_target_count, 0);
    assert!(dry_run.target_records.is_empty());
    assert!(!target_path.exists());
}

#[test]
fn sprint184_target_set_refresh_dedupes_and_rerun_uses_refreshed_targets() {
    let (_batch_result, observer_run_result, closure_result, _target_set, _rerun_result, _, _) =
        sprint184_closure_fixture();
    let duplicate = closure_result
        .target_records
        .iter()
        .find(|target| {
            target.approval_status == core::ObserverAgreementTargetApprovalStatus::Approved
        })
        .expect("approved target")
        .clone();
    let refresh = core::refresh_observer_agreement_target_set(
        &[duplicate.clone()],
        &[duplicate.clone(), duplicate],
    );
    let rerun = core::rerun_observer_comparison_with_refreshed_targets(
        &observer_run_result,
        &refresh.refreshed_target_set,
    );

    assert_eq!(refresh.duplicate_target_count, 2);
    assert_eq!(
        refresh.refresh_status,
        core::ObserverAgreementTargetSetRefreshStatus::NoChange
    );
    assert!(matches!(
        rerun.rerun_status,
        core::ObserverComparisonRerunStatus::Improved
            | core::ObserverComparisonRerunStatus::NoChange
    ));
}

#[test]
fn sprint184_readiness_hardening_reduces_warnings_without_decision_integration() {
    let (
        batch_result,
        _observer_run_result,
        _closure_result,
        _target_set,
        _rerun_result,
        guard,
        gate,
    ) = sprint184_closure_fixture();

    assert_eq!(
        guard.guard_status,
        core::ObserverTargetClosureDecisionIsolationGuardStatus::Preserved
    );
    assert!(batch_result.safety_summary.no_model_training);
    assert!(batch_result.safety_summary.no_weight_update);
    assert!(batch_result.safety_summary.no_checkpoint);
    assert!(batch_result.safety_summary.no_live_inference);
    assert!(batch_result.safety_summary.no_broker_order_account);
    assert!(matches!(
        gate.new_readiness_status,
        core::SmartCoreObserverLaneReadinessStatus::NonVotingObserverReady
            | core::SmartCoreObserverLaneReadinessStatus::NonVotingObserverReadyWithWarnings
    ));
    assert_ne!(
        gate.new_readiness_status,
        core::SmartCoreObserverLaneReadinessStatus::BlockedByDecisionLeak
    );
}

#[test]
fn sprint184_observer_comparison_ledger_appends_and_rejects_remote_path() {
    let (_batch_result, observer_run_result, _closure_result, _target_set, rerun_result, _, gate) =
        sprint184_closure_fixture();
    let mut ledger = core::ObserverComparisonLedger {
        ledger_id: "ledger".to_string(),
        entries: Vec::new(),
        entry_count: 0,
        latest_entry_id: None,
        paper_only: true,
    };
    core::append_observer_comparison_ledger_entry(
        &mut ledger,
        core::ObserverComparisonLedgerEntry {
            entry_id: "entry-1".to_string(),
            run_id: "run-1".to_string(),
            timestamp: None,
            observer_member_count: observer_run_result.observer_members.len(),
            comparison_count: observer_run_result.comparison_summary.comparison_count,
            disagreement_count: rerun_result.new_disagreement_count,
            comparison_summary_status: rerun_result.new_comparison_summary_status,
            readiness_status: gate.new_readiness_status,
            target_coverage_status: rerun_result.new_target_coverage_status,
            safety_status: observer_run_result.observer_safety_guard.guard_status,
            target_count: 2,
            paper_only: true,
        },
    );
    let repeated_entry = ledger.entries[0].clone();
    core::append_observer_comparison_ledger_entry(&mut ledger, repeated_entry);
    ledger.entries.push(ledger.entries[0].clone());
    ledger.entry_count = ledger.entries.len();
    let path = sprint171_temp_json_path("observer-ledger");
    core::save_observer_comparison_ledger_to_local_json(&path, &ledger).expect("save ledger");
    let loaded = core::load_observer_comparison_ledger_from_local_json(&path).expect("load ledger");

    assert_eq!(loaded.entry_count, 1);
    assert_eq!(
        core::latest_observer_comparison_entry(&loaded)
            .expect("latest")
            .entry_id,
        "entry-1"
    );
    assert!(
        core::save_observer_comparison_ledger_to_local_json(
            std::path::Path::new("https://example.com/observer-ledger.json"),
            &ledger
        )
        .is_err()
    );
}

#[test]
fn sprint184_owner_summary_and_decision_isolation_cover_pass_and_input_feature_violation() {
    let (
        batch_result,
        _observer_run_result,
        closure_result,
        _target_set,
        rerun_result,
        guard,
        gate,
    ) = sprint184_closure_fixture();
    let summary =
        core::build_owner_observer_coverage_closure_summary(&gate, &closure_result, &rerun_result);
    let mut violating_closure = closure_result.clone();
    violating_closure.target_records[0].not_input_feature = false;
    violating_closure.target_records[0].reason = "input feature leak".to_string();
    let violated = core::evaluate_observer_target_closure_decision_isolation(
        &violating_closure,
        &batch_result,
        Some(&batch_result),
    );

    assert_eq!(
        guard.guard_status,
        core::ObserverTargetClosureDecisionIsolationGuardStatus::Preserved
    );
    assert!(summary.non_voting);
    assert!(summary.read_only);
    assert!(summary.not_investment_signal);
    assert!(summary.not_committee_opinion);
    assert_eq!(
        violated.guard_status,
        core::ObserverTargetClosureDecisionIsolationGuardStatus::Violated
    );
}

#[test]
fn sprint184_config_loads_new_flags() {
    let config = MinimalAiCommitteeCycleConfig::from_toml_path(std::path::Path::new(
        "examples/soma_minimal_ai_committee_core.toml",
    ))
    .expect("config");
    let run_config = config
        .load_autonomous_paper_run_config()
        .expect("autonomous config");

    assert!(run_config.observer_target_closure_enabled);
    assert!(run_config.observer_target_closure_dry_run);
    assert_eq!(
        run_config.observer_target_closure_output_path.as_deref(),
        Some("target/minimal_observer_target_closure.json")
    );
    assert_eq!(
        run_config.observer_target_set_output_path.as_deref(),
        Some("target/minimal_observer_agreement_targets.json")
    );
    assert_eq!(
        run_config.observer_comparison_ledger_path.as_deref(),
        Some("target/minimal_observer_comparison_ledger.json")
    );
    assert!(run_config.observer_readiness_hardening_enabled);
    assert!(run_config.observer_coverage_closure_emit_owner_summary);
}

#[test]
fn sprint185_apply_policy_requires_explicit_apply_and_rejects_non_approved_targets() {
    let mut invalid_policy = core::default_observer_target_closure_apply_policy();
    invalid_policy.require_explicit_apply = false;
    assert!(core::validate_observer_target_closure_apply_policy(&invalid_policy).is_err());
    let mut invalid_policy = core::default_observer_target_closure_apply_policy();
    invalid_policy.require_read_only_observer = false;
    assert!(core::validate_observer_target_closure_apply_policy(&invalid_policy).is_err());
    let mut invalid_policy = core::default_observer_target_closure_apply_policy();
    invalid_policy.allow_needs_review_targets = true;
    assert!(core::validate_observer_target_closure_apply_policy(&invalid_policy).is_err());
    let mut invalid_policy = core::default_observer_target_closure_apply_policy();
    invalid_policy.reject_input_feature_targets = false;
    assert!(core::validate_observer_target_closure_apply_policy(&invalid_policy).is_err());

    let policy = core::ObserverTargetClosureApplyPolicy {
        allow_apply_approved_targets: true,
        ..core::default_observer_target_closure_apply_policy()
    };
    let closure_result = sprint185_closure_result_with_targets(
        "apply-policy",
        vec![
            sprint185_target(
                "approved",
                core::ObserverAgreementTargetApprovalStatus::Approved,
                core::ObserverAgreementTargetSource::MemberOpinion,
                core::SourceConfidence::High,
                "safe target",
            ),
            sprint185_target(
                "needs-review",
                core::ObserverAgreementTargetApprovalStatus::NeedsReview,
                core::ObserverAgreementTargetSource::MemberOpinion,
                core::SourceConfidence::ReviewRequired,
                "needs review",
            ),
            sprint185_target(
                "rejected",
                core::ObserverAgreementTargetApprovalStatus::Rejected,
                core::ObserverAgreementTargetSource::MemberOpinion,
                core::SourceConfidence::High,
                "rejected",
            ),
            sprint185_target(
                "news-only",
                core::ObserverAgreementTargetApprovalStatus::Approved,
                core::ObserverAgreementTargetSource::ResearchEvidence,
                core::SourceConfidence::High,
                "news only",
            ),
            sprint185_target(
                "broker-order",
                core::ObserverAgreementTargetApprovalStatus::Approved,
                core::ObserverAgreementTargetSource::MemberOpinion,
                core::SourceConfidence::High,
                "broker order account",
            ),
        ],
    );
    let apply_result = core::apply_observer_target_closure_records(
        &closure_result,
        None,
        &core::ObserverTargetClosureApplyConfig {
            run_id: "apply-policy".to_string(),
            closure_result_input_path: None,
            target_set_input_path: None,
            target_set_output_path: None,
            apply_enabled: true,
            dry_run: false,
            paper_only: true,
        },
        &policy,
    )
    .expect("apply");

    assert_eq!(apply_result.applied_count, 1);
    assert_eq!(apply_result.skipped_needs_review_count, 1);
    assert_eq!(apply_result.skipped_rejected_count, 1);
    assert_eq!(apply_result.unsafe_rejected_count, 2);

    let err = core::apply_observer_target_closure_records(
        &closure_result,
        None,
        &core::ObserverTargetClosureApplyConfig {
            run_id: "apply-policy-not-paper".to_string(),
            closure_result_input_path: None,
            target_set_input_path: None,
            target_set_output_path: None,
            apply_enabled: true,
            dry_run: false,
            paper_only: false,
        },
        &policy,
    )
    .expect_err("non-paper apply config must fail");
    assert!(err.contains("paper_only"));
}

#[test]
fn sprint185_dry_run_apply_writes_nothing() {
    let store_path = sprint171_temp_json_path("observer-apply-dry-run");
    let _ = fs::remove_file(&store_path);
    let policy = core::ObserverTargetClosureApplyPolicy {
        allow_apply_approved_targets: true,
        ..core::default_observer_target_closure_apply_policy()
    };
    let closure_result = sprint185_closure_result_with_targets(
        "dry-run",
        vec![sprint185_target(
            "approved",
            core::ObserverAgreementTargetApprovalStatus::Approved,
            core::ObserverAgreementTargetSource::MemberOpinion,
            core::SourceConfidence::High,
            "safe target",
        )],
    );
    let apply_result = core::apply_observer_target_closure_records(
        &closure_result,
        None,
        &core::ObserverTargetClosureApplyConfig {
            run_id: "dry-run".to_string(),
            closure_result_input_path: None,
            target_set_input_path: None,
            target_set_output_path: Some(store_path.to_string_lossy().to_string()),
            apply_enabled: true,
            dry_run: true,
            paper_only: true,
        },
        &policy,
    )
    .expect("apply");

    assert_eq!(
        apply_result.apply_status,
        core::ObserverTargetClosureApplyStatus::DryRunPreview
    );
    assert!(!apply_result.wrote_target_set);
    assert!(!store_path.exists());
}

#[test]
fn sprint185_apply_mode_writes_store_and_target_store_dedupes() {
    let (
        _batch_result,
        _observer_run_result,
        _closure_result,
        run_result,
        store_path,
        _ledger_path,
    ) = sprint185_apply_trend_fixture(true, false);
    let store =
        core::load_observer_agreement_target_store_from_local_json(&store_path).expect("store");
    let query_target = sprint185_target(
        "query-target",
        core::ObserverAgreementTargetApprovalStatus::Approved,
        core::ObserverAgreementTargetSource::MemberOpinion,
        core::SourceConfidence::High,
        "safe target",
    );
    let merged = core::merge_approved_observer_targets(
        &store,
        &[query_target.clone(), query_target.clone()],
    );
    let deduped = core::dedupe_observer_targets(&merged);

    assert!(run_result.apply_result.wrote_target_set);
    assert!(store.approved_count > 0);
    assert_eq!(merged.target_count, deduped.target_count);
    let scoped_target = deduped
        .target_set
        .targets
        .iter()
        .find_map(|target| Some((target.symbol.as_deref()?, target.market_scope?)))
        .expect("persisted target symbol scope");
    assert!(!deduped.targets_by_member("trend-kr-short").is_empty());
    assert!(
        !deduped
            .targets_by_head(core::ObserverAgreementTargetHead::Stance)
            .is_empty()
    );
    assert!(
        !deduped
            .targets_by_symbol_scope(scoped_target.0, scoped_target.1)
            .is_empty()
    );
    let mut invalid_store = store.clone();
    invalid_store.target_set.targets.push(sprint185_target(
        "needs-review-store",
        core::ObserverAgreementTargetApprovalStatus::NeedsReview,
        core::ObserverAgreementTargetSource::MemberOpinion,
        core::SourceConfidence::ReviewRequired,
        "needs review",
    ));
    invalid_store.target_count = invalid_store.target_set.targets.len();
    let invalid_store_path = sprint171_temp_json_path("observer-target-store-invalid");
    let err = core::save_observer_agreement_target_store_to_local_json(
        &invalid_store_path,
        &invalid_store,
    )
    .expect_err("persisted target store must reject NeedsReview targets");
    assert!(err.contains("approved targets only"));
}

#[test]
fn sprint185_comparison_rerun_v2_uses_persisted_target_store() {
    let (
        _batch_result,
        _observer_run_result,
        _closure_result,
        run_result,
        _store_path,
        _ledger_path,
    ) = sprint185_apply_trend_fixture(true, false);

    assert!(run_result.comparison_rerun_result.new_target_count > 0);
    assert!(matches!(
        run_result.comparison_rerun_result.rerun_status,
        core::ObserverComparisonRerunV2Status::Improved
            | core::ObserverComparisonRerunV2Status::NoChange
    ));
}

#[test]
fn sprint185_ledger_normalization_and_trend_cover_history_cases() {
    let one_entry = core::ObserverComparisonLedger {
        ledger_id: "ledger".to_string(),
        entries: vec![core::ObserverComparisonLedgerEntry {
            entry_id: "entry".to_string(),
            run_id: "run-1".to_string(),
            timestamp: None,
            observer_member_count: 3,
            comparison_count: 3,
            disagreement_count: 1,
            comparison_summary_status: core::ObserverVsCommitteeComparisonSummaryStatus::Mixed,
            readiness_status:
                core::SmartCoreObserverLaneReadinessStatus::NonVotingObserverReadyWithWarnings,
            target_coverage_status: core::ObserverTargetCoverageClosureStatus::ClosurePlanned,
            safety_status: core::SmartCoreObserverLaneSafetyGuardStatus::Preserved,
            target_count: 1,
            paper_only: true,
        }],
        entry_count: 1,
        latest_entry_id: Some("entry".to_string()),
        paper_only: true,
    };
    let duplicate_ledger = core::ObserverComparisonLedger {
        entries: vec![
            one_entry.entries[0].clone(),
            one_entry.entries[0].clone(),
            core::ObserverComparisonLedgerEntry {
                entry_id: "entry-3".to_string(),
                run_id: "run-3".to_string(),
                timestamp: None,
                observer_member_count: 3,
                comparison_count: 3,
                disagreement_count: 0,
                comparison_summary_status:
                    core::ObserverVsCommitteeComparisonSummaryStatus::MostlyAgree,
                readiness_status:
                    core::SmartCoreObserverLaneReadinessStatus::NonVotingObserverReady,
                target_coverage_status: core::ObserverTargetCoverageClosureStatus::NoClosureNeeded,
                safety_status: core::SmartCoreObserverLaneSafetyGuardStatus::Preserved,
                target_count: 2,
                paper_only: true,
            },
        ],
        entry_count: 3,
        latest_entry_id: Some("entry-3".to_string()),
        ..one_entry.clone()
    };
    let (normalized, normalization) =
        core::normalize_observer_comparison_ledger_entries(&duplicate_ledger);
    let insufficient = core::compute_observer_comparison_ledger_trend(
        &one_entry,
        core::ObserverComparisonLedgerTrendWindow::All,
    );
    let improving = core::compute_observer_comparison_ledger_trend(
        &normalized,
        core::ObserverComparisonLedgerTrendWindow::All,
    );
    let stable_ledger = core::ObserverComparisonLedger {
        entries: vec![
            core::ObserverComparisonLedgerEntry {
                disagreement_count: 1,
                ..normalized.entries[0].clone()
            },
            core::ObserverComparisonLedgerEntry {
                entry_id: "stable-2".to_string(),
                disagreement_count: 1,
                ..normalized.entries[0].clone()
            },
        ],
        entry_count: 2,
        latest_entry_id: Some("stable-2".to_string()),
        ..normalized.clone()
    };
    let worsening_ledger = core::ObserverComparisonLedger {
        entries: vec![
            core::ObserverComparisonLedgerEntry {
                disagreement_count: 0,
                ..normalized.entries[0].clone()
            },
            core::ObserverComparisonLedgerEntry {
                entry_id: "worse-2".to_string(),
                disagreement_count: 2,
                ..normalized.entries[0].clone()
            },
        ],
        entry_count: 2,
        latest_entry_id: Some("worse-2".to_string()),
        ..normalized.clone()
    };

    assert_eq!(normalization.duplicate_entry_count, 1);
    assert_eq!(normalization.normalized_entry_count, 2);
    assert_eq!(normalized.entry_count, 2);
    assert_eq!(
        normalized
            .entries
            .iter()
            .map(|entry| entry.entry_id.as_str())
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        normalized.entries.len()
    );
    assert_eq!(
        insufficient.trend_status,
        core::ObserverComparisonLedgerTrendStatus::InsufficientHistory
    );
    assert_eq!(
        improving.disagreement_trend,
        core::ObserverTrendDirection::Improving
    );
    assert_eq!(
        core::compute_observer_comparison_ledger_trend(
            &stable_ledger,
            core::ObserverComparisonLedgerTrendWindow::All
        )
        .disagreement_trend,
        core::ObserverTrendDirection::Stable
    );
    assert_eq!(
        core::compute_observer_comparison_ledger_trend(
            &worsening_ledger,
            core::ObserverComparisonLedgerTrendWindow::All
        )
        .disagreement_trend,
        core::ObserverTrendDirection::Worsening
    );
}

#[test]
fn sprint185_warning_reducer_and_readiness_v2_handle_remaining_review_and_clear_case() {
    let (
        _batch_result,
        observer_run_result,
        _closure_result,
        _target_set,
        _rerun_result,
        _guard,
        previous_gate,
    ) = sprint184_closure_fixture();
    let clear_apply = core::ObserverTargetClosureApplyResult {
        run_id: "clear".to_string(),
        approved_input_count: 2,
        needs_review_input_count: 0,
        rejected_input_count: 0,
        applied_count: 2,
        skipped_needs_review_count: 0,
        skipped_rejected_count: 0,
        skipped_duplicate_count: 0,
        unsafe_rejected_count: 0,
        previous_target_count: 0,
        new_target_count: 2,
        wrote_target_set: true,
        output_path: None,
        applied_targets: vec![sprint185_target(
            "approved",
            core::ObserverAgreementTargetApprovalStatus::Approved,
            core::ObserverAgreementTargetSource::MemberOpinion,
            core::SourceConfidence::High,
            "safe target",
        )],
        apply_status: core::ObserverTargetClosureApplyStatus::Applied,
        warnings: Vec::new(),
        paper_only: true,
    };
    let clear_rerun = core::ObserverComparisonRerunV2Result {
        previous_comparison_summary_status: core::ObserverVsCommitteeComparisonSummaryStatus::Mixed,
        new_comparison_summary_status:
            core::ObserverVsCommitteeComparisonSummaryStatus::MostlyAgree,
        previous_disagreement_count: 1,
        new_disagreement_count: 0,
        disagreement_delta: -1,
        previous_target_count: 0,
        new_target_count: 2,
        target_delta: 2,
        target_coverage_improved: true,
        rerun_status: core::ObserverComparisonRerunV2Status::Improved,
        paper_only: true,
    };
    let clear_trend = core::ObserverComparisonLedgerTrend {
        trend_id: "trend".to_string(),
        ledger_id: "ledger".to_string(),
        window: core::ObserverComparisonLedgerTrendWindow::All,
        entry_count: 2,
        disagreement_series: vec![1, 0],
        comparison_count_series: vec![3, 3],
        readiness_status_series: vec![
            core::SmartCoreObserverLaneReadinessStatus::NonVotingObserverReadyWithWarnings,
            core::SmartCoreObserverLaneReadinessStatus::NonVotingObserverReady,
        ],
        target_coverage_status_series: vec![
            core::ObserverTargetCoverageClosureStatus::ClosurePlanned,
            core::ObserverTargetCoverageClosureStatus::NoClosureNeeded,
        ],
        safety_status_series: vec![
            core::SmartCoreObserverLaneSafetyGuardStatus::Preserved,
            core::SmartCoreObserverLaneSafetyGuardStatus::Preserved,
        ],
        disagreement_trend: core::ObserverTrendDirection::Improving,
        readiness_trend: core::ObserverTrendDirection::Improving,
        target_coverage_trend: core::ObserverTrendDirection::Improving,
        trend_status: core::ObserverComparisonLedgerTrendStatus::Useful,
        paper_only: true,
    };
    let clear_reducer = core::reduce_observer_readiness_warnings(
        &previous_gate,
        &clear_apply,
        &clear_rerun,
        &clear_trend,
    );
    let clear_v2 = core::evaluate_smartcore_observer_readiness_v2(
        &previous_gate,
        &clear_apply,
        &clear_rerun,
        &clear_trend,
        &clear_reducer,
        &observer_run_result.observer_safety_guard,
    );

    let review_apply = core::ObserverTargetClosureApplyResult {
        run_id: "review".to_string(),
        skipped_needs_review_count: 1,
        needs_review_input_count: 1,
        apply_status: core::ObserverTargetClosureApplyStatus::AppliedWithWarnings,
        ..clear_apply.clone()
    };
    let review_trend = core::ObserverComparisonLedgerTrend {
        entry_count: 1,
        trend_status: core::ObserverComparisonLedgerTrendStatus::InsufficientHistory,
        disagreement_trend: core::ObserverTrendDirection::InsufficientHistory,
        readiness_trend: core::ObserverTrendDirection::InsufficientHistory,
        target_coverage_trend: core::ObserverTrendDirection::InsufficientHistory,
        ..clear_trend.clone()
    };
    let review_reducer = core::reduce_observer_readiness_warnings(
        &previous_gate,
        &review_apply,
        &clear_rerun,
        &review_trend,
    );
    let review_v2 = core::evaluate_smartcore_observer_readiness_v2(
        &previous_gate,
        &review_apply,
        &clear_rerun,
        &review_trend,
        &review_reducer,
        &observer_run_result.observer_safety_guard,
    );

    assert!(matches!(
        clear_reducer.reduction_status,
        core::ObserverReadinessWarningReductionStatus::Cleared
            | core::ObserverReadinessWarningReductionStatus::Reduced
    ));
    assert!(matches!(
        clear_v2.readiness_status,
        core::SmartCoreObserverReadinessV2Status::NonVotingObserverReady
            | core::SmartCoreObserverReadinessV2Status::NonVotingObserverReadyWithWarnings
    ));
    assert!(
        review_reducer
            .remaining_warnings
            .contains(&core::ObserverReadinessWarningKind::NeedsReviewTargetRemaining)
    );
    assert!(matches!(
        review_v2.readiness_status,
        core::SmartCoreObserverReadinessV2Status::NeedsMoreTargets
            | core::SmartCoreObserverReadinessV2Status::NonVotingObserverReadyWithWarnings
            | core::SmartCoreObserverReadinessV2Status::NeedsMoreHistory
    ));
}

#[test]
fn sprint185_owner_trend_summary_and_apply_decision_isolation_cover_pass_and_fail() {
    let (
        batch_result,
        _observer_run_result,
        _closure_result,
        run_result,
        _store_path,
        _ledger_path,
    ) = sprint185_apply_trend_fixture(true, false);
    let summary = run_result
        .owner_trend_summary
        .as_ref()
        .expect("owner trend summary");
    let mut violating_apply = run_result.apply_result.clone();
    violating_apply.applied_targets[0].not_input_feature = false;
    let violated = core::evaluate_observer_apply_decision_isolation(
        &violating_apply,
        &batch_result,
        Some(&batch_result),
    );

    assert!(summary.non_voting);
    assert!(summary.read_only);
    assert!(summary.not_investment_signal);
    assert!(summary.not_committee_opinion);
    assert_eq!(
        run_result.decision_isolation_guard.guard_status,
        core::ObserverApplyDecisionIsolationGuardStatus::Preserved
    );
    assert_eq!(
        violated.guard_status,
        core::ObserverApplyDecisionIsolationGuardStatus::Violated
    );
}

#[test]
fn sprint185_apply_trend_run_preserves_safety_and_is_deterministic() {
    let (batch_result, _observer_run_result, _closure_result, first, _store_path, _ledger_path) =
        sprint185_apply_trend_fixture(true, false);
    let (
        _batch_result2,
        _observer_run_result2,
        _closure_result2,
        second,
        _store_path2,
        _ledger_path2,
    ) = sprint185_apply_trend_fixture(true, false);

    assert!(batch_result.safety_summary.no_model_training);
    assert!(batch_result.safety_summary.no_weight_update);
    assert!(batch_result.safety_summary.no_checkpoint);
    assert!(batch_result.safety_summary.no_live_inference);
    assert!(batch_result.safety_summary.no_broker_order_account);
    assert_eq!(
        first.apply_result.applied_count,
        second.apply_result.applied_count
    );
    assert_eq!(
        first.comparison_rerun_result.rerun_status,
        second.comparison_rerun_result.rerun_status
    );
    assert_eq!(
        first.readiness_v2.readiness_status,
        second.readiness_v2.readiness_status
    );
}

#[test]
fn sprint185_config_loads_new_flags() {
    let config = MinimalAiCommitteeCycleConfig::from_toml_path(std::path::Path::new(
        "examples/soma_minimal_ai_committee_core.toml",
    ))
    .expect("config");
    let run_config = config
        .load_autonomous_paper_run_config()
        .expect("autonomous config");

    assert!(run_config.observer_target_apply_trend_enabled);
    assert!(run_config.observer_target_apply_dry_run);
    assert!(!run_config.observer_target_apply_targets);
    assert_eq!(
        run_config.observer_target_store_output_path.as_deref(),
        Some("target/minimal_observer_target_store.json")
    );
    assert!(run_config.observer_ledger_trend_enabled);
    assert!(run_config.observer_readiness_v2_enabled);
    assert!(run_config.observer_trend_summary_enabled);
    assert_eq!(
        run_config.observer_apply_trend_output_path.as_deref(),
        Some("target/minimal_observer_apply_trend.json")
    );
    assert!(run_config.observer_seed_apply_trend_enabled);
    assert!(run_config.observer_seed_apply_dry_run);
    assert!(!run_config.observer_seed_apply_targets);
    assert_eq!(
        run_config.observer_seed_target_store_output_path.as_deref(),
        Some("target/minimal_observer_seed_target_store.json")
    );
    assert_eq!(
        run_config.observer_seed_apply_output_path.as_deref(),
        Some("target/minimal_observer_seed_apply_trend.json")
    );
    assert!(run_config.observer_seed_require_approved_target);
    assert!(run_config.observer_seed_rerun_comparison);
    assert!(run_config.observer_seed_compute_ledger_trend);
    assert!(run_config.observer_seed_recheck_readiness);
    assert!(run_config.observer_seed_emit_owner_summary);
    assert!(run_config.observer_approved_apply_governance_enabled);
    assert_eq!(
        run_config.observer_approved_apply_mode,
        core::ObserverExplicitApplyMode::DryRun
    );
    assert!(run_config.observer_approved_apply_dry_run);
    assert_eq!(
        run_config
            .observer_approved_target_store_output_path
            .as_deref(),
        Some("target/minimal_observer_approved_target_store.json")
    );
    assert_eq!(
        run_config.observer_approved_apply_output_path.as_deref(),
        Some("target/minimal_observer_approved_apply_governance.json")
    );
    assert!(run_config.observer_approved_apply_recheck_readiness);
    assert!(run_config.chairman_governance_contract_prepare_enabled);
    assert!(run_config.chairman_governance_readiness_check_enabled);
    assert!(run_config.observer_approved_apply_emit_owner_summary);
    assert!(!run_config.observer_apply_verify_chairman_shadow_enabled);
    assert_eq!(
        run_config.observer_apply_verify_mode,
        core::ObserverExplicitApplyMode::DryRun
    );
    assert!(run_config.observer_apply_verify_dry_run);
    assert_eq!(
        run_config
            .observer_apply_verify_target_store_output_path
            .as_deref(),
        Some("target/minimal_observer_apply_verify_target_store.json")
    );
    assert_eq!(
        run_config.observer_apply_verify_output_path.as_deref(),
        Some("target/minimal_observer_apply_verify_and_shadow.json")
    );
    assert!(run_config.observer_apply_verify_emit_owner_summary);
    assert!(run_config.chairman_shadow_governance_enabled);
}

#[test]
fn sprint188_dedicated_apply_verify_config_uses_non_dry_profile() {
    let config = MinimalAiCommitteeCycleConfig::from_toml_path(std::path::Path::new(
        "examples/soma_minimal_ai_committee_observer_apply_verify.toml",
    ))
    .expect("config");
    let run_config = config
        .load_autonomous_paper_run_config()
        .expect("autonomous config");

    assert!(run_config.observer_apply_verify_chairman_shadow_enabled);
    assert_eq!(
        run_config.observer_apply_verify_mode,
        core::ObserverExplicitApplyMode::ApplyApprovedTargets
    );
    assert!(!run_config.observer_apply_verify_dry_run);
    assert_eq!(
        run_config
            .observer_apply_verify_target_store_output_path
            .as_deref(),
        Some("target/minimal_observer_apply_verify_target_store.verify.json")
    );
    assert_eq!(
        run_config.observer_apply_verify_output_path.as_deref(),
        Some("target/minimal_observer_apply_verify_and_shadow.verify.json")
    );
    assert!(run_config.chairman_shadow_governance_enabled);
}

#[test]
fn sprint185_apply_trend_dry_run_does_not_persist_target_store() {
    let (_batch_result, _observer_run_result, _closure_result, apply, store_path, ledger_path) =
        sprint185_apply_trend_fixture(true, true);

    assert_eq!(
        apply.apply_result.apply_status,
        core::ObserverTargetClosureApplyStatus::DryRunPreview
    );
    assert!(!apply.apply_result.wrote_target_set);
    assert!(!store_path.exists());
    assert!(ledger_path.exists());
}

#[test]
fn sprint186_seed_builder_creates_member_and_risk_seeds() {
    let (batch_result, _) = sprint183_observer_lane_fixture();
    let build = core::build_observer_approved_target_seeds(
        &batch_result,
        None,
        None,
        None,
        &core::ObserverApprovedTargetSeedBuildConfig {
            run_id: "sprint186-seed-build".to_string(),
            include_member_opinion: true,
            include_risk_governor_status: true,
            include_chairman_decision: true,
            include_validated_replay_labels: true,
            include_core_calibration_targets: true,
            include_validated_paper_outcomes: true,
            max_seeds_per_member: 3,
            max_total_seeds: 16,
            paper_only: true,
        },
    );
    assert!(build.seeds.iter().any(|seed| {
        seed.source_type == core::ObserverApprovedTargetSeedSource::MemberOpinion
            && seed.head == core::ObserverAgreementTargetHead::Stance
    }));
    assert!(build.seeds.iter().any(|seed| {
        seed.source_type == core::ObserverApprovedTargetSeedSource::RiskGovernorStatus
            && seed.head == core::ObserverAgreementTargetHead::Risk
    }));
}

#[test]
fn sprint186_seed_builder_uses_validated_replay_and_calibration_sources_only() {
    let (batch_result, _) = sprint183_observer_lane_fixture();
    let mut validated_replay = sprint156_replay_example(
        "sprint186-validated-replay",
        "AAPL",
        MarketScope::UsShortTerm,
    );
    if let Some(labels) = validated_replay.target_labels.as_mut() {
        labels.label_source = ReplayLabelSource::ValidatedPaperLabel;
        labels.label_confidence = ReplayLabelConfidence::High;
    }
    let mut review_replay =
        sprint156_replay_example("sprint186-review-replay", "AAPL", MarketScope::UsShortTerm);
    if let Some(labels) = review_replay.target_labels.as_mut() {
        labels.label_source = ReplayLabelSource::ReviewRequired;
        labels.label_confidence = ReplayLabelConfidence::ReviewRequired;
    }
    let replay_dataset = sprint156_replay_dataset(vec![validated_replay, review_replay]);
    let calibration_dataset = core::CoreCalibrationDataset {
        dataset_id: "sprint186-calibration-dataset".to_string(),
        examples: vec![
            core::CoreCalibrationExample {
                calibration_example_id: "sprint186-calibration-approved".to_string(),
                member_id: "trend-kr-short".to_string(),
                debug_output_id: "debug-approved".to_string(),
                target_id: None,
                symbol: Some("AAPL".to_string()),
                market_scope: Some(MarketScope::UsShortTerm),
                head: core::SmartCoreShadowHeadKind::Risk,
                debug_bucket: "RiskLow".to_string(),
                target_bucket: "RiskLow".to_string(),
                alignment: core::SmartCoreShadowAlignmentStatus::Match,
                mismatch_type: None,
                suggested_data_need: core::SmartCoreMismatchDataNeed::KeepObserving,
                label_source: core::CoreCalibrationLabelSource::RiskGovernorStatus,
                label_confidence: ReplayLabelConfidence::High,
                paper_only: true,
            },
            core::CoreCalibrationExample {
                calibration_example_id: "sprint186-calibration-low".to_string(),
                member_id: "trend-kr-short".to_string(),
                debug_output_id: "debug-low".to_string(),
                target_id: None,
                symbol: Some("AAPL".to_string()),
                market_scope: Some(MarketScope::UsShortTerm),
                head: core::SmartCoreShadowHeadKind::Risk,
                debug_bucket: "RiskHigh".to_string(),
                target_bucket: "RiskHigh".to_string(),
                alignment: core::SmartCoreShadowAlignmentStatus::Mismatch,
                mismatch_type: None,
                suggested_data_need: core::SmartCoreMismatchDataNeed::MoreRiskLabels,
                label_source: core::CoreCalibrationLabelSource::ReplayLabel,
                label_confidence: ReplayLabelConfidence::Low,
                paper_only: true,
            },
        ],
        example_count: 2,
        member_count: 1,
        head_distribution: std::collections::BTreeMap::new(),
        alignment_distribution: std::collections::BTreeMap::new(),
        mismatch_type_distribution: std::collections::BTreeMap::new(),
        paper_only: true,
    };
    let build = core::build_observer_approved_target_seeds(
        &batch_result,
        Some(&replay_dataset),
        Some(&calibration_dataset),
        None,
        &core::ObserverApprovedTargetSeedBuildConfig {
            run_id: "sprint186-seed-build-datasets".to_string(),
            include_member_opinion: false,
            include_risk_governor_status: false,
            include_chairman_decision: false,
            include_validated_replay_labels: true,
            include_core_calibration_targets: true,
            include_validated_paper_outcomes: false,
            max_seeds_per_member: 3,
            max_total_seeds: 16,
            paper_only: true,
        },
    );

    assert!(build.seeds.iter().any(|seed| {
        seed.source_type == core::ObserverApprovedTargetSeedSource::ValidatedReplayLabel
            && seed.source_record_id.as_deref() == Some("sprint186-validated-replay")
    }));
    assert!(build.seeds.iter().any(|seed| {
        seed.source_type == core::ObserverApprovedTargetSeedSource::CoreCalibrationDataset
            && seed.source_record_id.as_deref() == Some("sprint186-calibration-approved")
    }));
    assert!(!build.seeds.iter().any(|seed| {
        seed.source_record_id.as_deref() == Some("sprint186-review-replay")
            || seed.source_record_id.as_deref() == Some("sprint186-calibration-low")
    }));
}

#[test]
fn sprint186_seed_validation_rejects_missing_member_id() {
    let validation = core::validate_observer_approved_target_seed(
        &core::ObserverApprovedTargetSeed {
            seed_id: "seed-missing-member".to_string(),
            source_type: core::ObserverApprovedTargetSeedSource::MemberOpinion,
            source_record_id: Some("record".to_string()),
            member_id: None,
            canonical_member_id: None,
            symbol: Some("AAPL".to_string()),
            market_scope: Some(MarketScope::UsShortTerm),
            head: core::ObserverAgreementTargetHead::Stance,
            target_bucket: core::SmartCoreHeadBucketNormalizedValue::PositiveLike,
            source_confidence: core::SourceConfidence::High,
            seed_status: core::ObserverApprovedTargetSeedStatus::ApprovedSeed,
            eval_only: true,
            not_input_feature: true,
            paper_only: true,
        },
        &core::default_observer_target_closure_apply_policy(),
    );
    assert!(!validation.valid);
    assert_eq!(
        validation.approval_status,
        core::ObserverAgreementTargetApprovalStatus::Rejected
    );
}

#[test]
fn sprint186_seed_validation_rejects_unknown_target_when_forbidden() {
    let validation = core::validate_observer_approved_target_seed(
        &core::ObserverApprovedTargetSeed {
            seed_id: "seed-unknown".to_string(),
            source_type: core::ObserverApprovedTargetSeedSource::ChairmanDecision,
            source_record_id: Some("record".to_string()),
            member_id: Some("trend-kr-short".to_string()),
            canonical_member_id: Some("trend-kr-short".to_string()),
            symbol: Some("AAPL".to_string()),
            market_scope: Some(MarketScope::UsShortTerm),
            head: core::ObserverAgreementTargetHead::Stance,
            target_bucket: core::SmartCoreHeadBucketNormalizedValue::Unknown,
            source_confidence: core::SourceConfidence::High,
            seed_status: core::ObserverApprovedTargetSeedStatus::ApprovedSeed,
            eval_only: true,
            not_input_feature: true,
            paper_only: true,
        },
        &core::default_observer_target_closure_apply_policy(),
    );
    assert!(!validation.valid);
    assert_eq!(
        validation.approval_status,
        core::ObserverAgreementTargetApprovalStatus::Rejected
    );
}

#[test]
fn sprint186_seed_conversion_creates_approved_observer_target() {
    let conversion = core::convert_approved_target_seeds_to_observer_targets(
        &[
            core::ObserverApprovedTargetSeed {
                seed_id: "seed-approved".to_string(),
                source_type: core::ObserverApprovedTargetSeedSource::MemberOpinion,
                source_record_id: Some("record".to_string()),
                member_id: Some("trend-kr-short".to_string()),
                canonical_member_id: Some("trend-kr-short".to_string()),
                symbol: Some("AAPL".to_string()),
                market_scope: Some(MarketScope::UsShortTerm),
                head: core::ObserverAgreementTargetHead::Stance,
                target_bucket: core::SmartCoreHeadBucketNormalizedValue::PositiveLike,
                source_confidence: core::SourceConfidence::High,
                seed_status: core::ObserverApprovedTargetSeedStatus::ApprovedSeed,
                eval_only: true,
                not_input_feature: true,
                paper_only: true,
            },
            core::ObserverApprovedTargetSeed {
                seed_id: "seed-needs-review".to_string(),
                source_type: core::ObserverApprovedTargetSeedSource::MemberOpinion,
                source_record_id: Some("record-review".to_string()),
                member_id: Some("trend-kr-short".to_string()),
                canonical_member_id: Some("trend-kr-short".to_string()),
                symbol: Some("AAPL".to_string()),
                market_scope: Some(MarketScope::UsShortTerm),
                head: core::ObserverAgreementTargetHead::Stance,
                target_bucket: core::SmartCoreHeadBucketNormalizedValue::PositiveLike,
                source_confidence: core::SourceConfidence::High,
                seed_status: core::ObserverApprovedTargetSeedStatus::NeedsReview,
                eval_only: true,
                not_input_feature: true,
                paper_only: true,
            },
            core::ObserverApprovedTargetSeed {
                seed_id: "seed-rejected".to_string(),
                source_type: core::ObserverApprovedTargetSeedSource::MemberOpinion,
                source_record_id: Some("record-rejected".to_string()),
                member_id: Some("trend-kr-short".to_string()),
                canonical_member_id: Some("trend-kr-short".to_string()),
                symbol: Some("AAPL".to_string()),
                market_scope: Some(MarketScope::UsShortTerm),
                head: core::ObserverAgreementTargetHead::Stance,
                target_bucket: core::SmartCoreHeadBucketNormalizedValue::PositiveLike,
                source_confidence: core::SourceConfidence::High,
                seed_status: core::ObserverApprovedTargetSeedStatus::Rejected,
                eval_only: true,
                not_input_feature: true,
                paper_only: true,
            },
        ],
        &core::default_observer_target_closure_apply_policy(),
    );
    assert_eq!(conversion.converted_target_count, 1);
    assert_eq!(conversion.needs_review_count, 1);
    assert_eq!(conversion.rejected_count, 1);
    assert_eq!(
        conversion.target_records[0].approval_status,
        core::ObserverAgreementTargetApprovalStatus::Approved
    );
}

#[test]
fn sprint186_controlled_apply_smoke_requires_approved_target() {
    let (mut batch_result, _) = sprint183_observer_lane_fixture();
    batch_result.member_opinions.clear();
    batch_result.chairman_decisions.clear();
    let smoke = core::run_observer_controlled_apply_smoke(
        &batch_result,
        None,
        None,
        None,
        &core::ObserverControlledApplySmokeConfig {
            run_id: "sprint186-smoke-require-approved".to_string(),
            enabled: true,
            apply_targets: true,
            dry_run: false,
            target_store_output_path: None,
            observer_apply_smoke_output_path: None,
            require_at_least_one_approved_target: true,
            paper_only: true,
        },
    )
    .expect("smoke");
    assert_eq!(
        smoke.smoke_status,
        core::ObserverControlledApplySmokeStatus::Failed
    );
    assert_eq!(
        smoke.apply_result.apply_status,
        core::ObserverTargetClosureApplyStatus::NoApprovedTargets
    );
}

#[test]
fn sprint186_controlled_apply_smoke_dry_run_writes_no_store_and_readiness_needs_apply() {
    let (_batch_result, _observer_run_result, run_result, store_path, ledger_path, output_path) =
        sprint186_seed_apply_fixture(true, true);
    assert_eq!(
        run_result
            .controlled_apply_smoke_result
            .apply_result
            .apply_status,
        core::ObserverTargetClosureApplyStatus::DryRunPreview
    );
    assert!(!run_result.controlled_apply_smoke_result.wrote_target_store);
    assert!(!store_path.exists());
    assert!(ledger_path.exists());
    assert!(output_path.exists());
    assert_eq!(
        run_result.readiness_recheck.readiness_status,
        core::ObserverSeededTargetReadinessStatus::NeedsApply
    );
    assert_eq!(
        run_result.decision_isolation_guard.guard_status,
        core::ObserverSeededTargetDecisionIsolationGuardStatus::Preserved
    );
}

#[test]
fn sprint186_controlled_apply_smoke_non_dry_run_persists_approved_targets_only() {
    let (_batch_result, _observer_run_result, run_result, store_path, _ledger_path, _output_path) =
        sprint186_seed_apply_fixture(true, false);
    assert!(store_path.exists());
    assert!(run_result.controlled_apply_smoke_result.wrote_target_store);
    let store = run_result
        .controlled_apply_smoke_result
        .target_store_after
        .as_ref()
        .expect("target store");
    assert!(store.target_count > 0);
    assert_eq!(store.target_count, store.approved_count);
    assert_eq!(store.needs_review_count, 0);
    assert_eq!(store.rejected_count, 0);
    assert_eq!(
        run_result.readiness_recheck.readiness_status,
        core::ObserverSeededTargetReadinessStatus::NonVotingObserverReadyWithWarnings
    );
}

#[test]
fn sprint186_owner_seeded_summary_stays_non_voting_read_only_eval_only() {
    let (_batch_result, _observer_run_result, run_result, ..) =
        sprint186_seed_apply_fixture(true, false);
    let summary = run_result.owner_summary.expect("owner summary");
    assert!(summary.non_voting);
    assert!(summary.read_only);
    assert!(summary.eval_only);
    assert!(summary.not_investment_signal);
    assert!(summary.not_committee_opinion);
}

#[test]
fn sprint186_seeded_decision_isolation_detects_pass_and_fail() {
    let (batch_result, _observer_run_result, run_result, ..) =
        sprint186_seed_apply_fixture(true, false);
    let preserved = core::evaluate_observer_seeded_target_decision_isolation(
        &run_result.controlled_apply_smoke_result,
        &batch_result,
        Some(&batch_result),
    );
    assert_eq!(
        preserved.guard_status,
        core::ObserverSeededTargetDecisionIsolationGuardStatus::Preserved
    );

    let mut mutated_batch = batch_result.clone();
    mutated_batch.member_opinions.push(MemberOpinion {
        member_id: "trend-us-medium".to_string(),
        symbol: "AAPL".to_string(),
        market_scope: MarketScope::UsShortTerm,
        stance: MemberStance::BuyProposal,
        confidence: 0.95,
        expected_return_hint: 0.05,
        risk_hint: 0.2,
        evidence_notes: vec!["seed leak".to_string()],
        event_triggered: false,
        event_reason: None,
    });
    let violated = core::evaluate_observer_seeded_target_decision_isolation(
        &run_result.controlled_apply_smoke_result,
        &batch_result,
        Some(&mutated_batch),
    );
    assert_eq!(
        violated.guard_status,
        core::ObserverSeededTargetDecisionIsolationGuardStatus::Violated
    );
    assert!(violated.seeded_targets_used_as_member_opinion);
}

#[test]
fn sprint186_seed_apply_trend_is_deterministic() {
    let (_batch_result, _observer_run_result, first, _store_a, _ledger_a, _output_a) =
        sprint186_seed_apply_fixture(true, true);
    let (_batch_result, _observer_run_result, second, _store_b, _ledger_b, _output_b) =
        sprint186_seed_apply_fixture(true, true);
    assert_eq!(
        first
            .controlled_apply_smoke_result
            .seed_build_result
            .approved_seed_count,
        second
            .controlled_apply_smoke_result
            .seed_build_result
            .approved_seed_count
    );
    assert_eq!(
        first
            .controlled_apply_smoke_result
            .seed_conversion_result
            .converted_target_count,
        second
            .controlled_apply_smoke_result
            .seed_conversion_result
            .converted_target_count
    );
    assert_eq!(
        first.readiness_recheck.readiness_status,
        second.readiness_recheck.readiness_status
    );
    assert_eq!(first.run_status, second.run_status);
}

#[test]
fn sprint187_explicit_apply_mode_rejects_dry_run_true() {
    let (_batch_result, _observer_run_result, run_result, store_path, _ledger_path, _output_path) =
        sprint187_apply_governance_fixture(
            core::ObserverExplicitApplyMode::ApplyApprovedTargets,
            true,
        );
    assert_eq!(
        run_result.apply_result.apply_status,
        core::ObserverApprovedTargetApplyV2Status::Blocked
    );
    assert!(!run_result.apply_result.wrote_target_store);
    assert!(!store_path.exists());
}

#[test]
fn sprint187_dry_run_apply_writes_no_store() {
    let (_batch_result, _observer_run_result, run_result, store_path, _ledger_path, output_path) =
        sprint187_apply_governance_fixture(core::ObserverExplicitApplyMode::DryRun, true);
    assert_eq!(
        run_result.apply_result.apply_status,
        core::ObserverApprovedTargetApplyV2Status::DryRunPreview
    );
    assert!(!run_result.apply_result.wrote_target_store);
    assert!(!store_path.exists());
    assert!(!_ledger_path.exists());
    assert!(output_path.exists());
}

#[test]
fn sprint187_apply_mode_writes_approved_target_store_to_temp_path() {
    let (_batch_result, _observer_run_result, run_result, store_path, ledger_path, _output_path) =
        sprint187_apply_governance_fixture(
            core::ObserverExplicitApplyMode::ApplyApprovedTargets,
            false,
        );
    assert!(store_path.exists());
    assert!(ledger_path.exists());
    assert!(matches!(
        run_result.apply_result.apply_status,
        core::ObserverApprovedTargetApplyV2Status::Applied
            | core::ObserverApprovedTargetApplyV2Status::AppliedWithWarnings
    ));
    assert!(run_result.apply_result.wrote_target_store);
    assert!(run_result.apply_result.applied_count > 0);
}

#[test]
fn sprint187_apply_mode_requires_output_path_before_claiming_persisted_store() {
    let (mut batch_result, observer_run_result, seed_run, ..) =
        sprint186_seed_apply_fixture(true, false);
    batch_result.observer_seed_apply_trend_run_result = Some(seed_run.clone());
    batch_result.observer_agreement_target_store = seed_run
        .controlled_apply_smoke_result
        .target_store_after
        .clone();
    let output_path = sprint171_temp_json_path("observer-approved-apply-no-store-output");
    let _ = fs::remove_file(&output_path);
    let run_result = core::run_observer_approved_apply_and_governance_prep(
        &batch_result,
        &seed_run
            .controlled_apply_smoke_result
            .seed_conversion_result
            .target_records,
        &observer_run_result,
        &core::ObserverApprovedApplyAndGovernancePrepRunConfig {
            run_id: "sprint187-approved-apply-no-store-output".to_string(),
            enabled: true,
            apply_mode: core::ObserverExplicitApplyMode::ApplyApprovedTargets,
            dry_run: false,
            target_store_input_path: None,
            target_store_output_path: None,
            observer_ledger_path: None,
            output_path: Some(output_path.to_string_lossy().to_string()),
            recheck_observer_readiness: true,
            prepare_chairman_governance_contract: true,
            emit_owner_summary: true,
            paper_only: true,
        },
    )
    .expect("approved apply governance run");

    assert_eq!(
        run_result.apply_result.apply_status,
        core::ObserverApprovedTargetApplyV2Status::Blocked
    );
    assert_eq!(
        run_result.run_status,
        core::ObserverApprovedApplyAndGovernancePrepRunStatus::Failed
    );
    assert!(!run_result.apply_result.wrote_target_store);
    assert_eq!(run_result.apply_result.applied_count, 0);
    assert!(run_result.apply_result.target_store_path.is_none());
    assert!(
        run_result
            .apply_result
            .warnings
            .iter()
            .any(|warning| warning.contains("target store output path required"))
    );
    assert_eq!(
        run_result.observer_readiness_v3.readiness_status,
        core::SmartCoreObserverReadinessV3Status::NeedsApply
    );
    assert!(output_path.exists());
}

#[test]
fn sprint187_apply_mode_blocks_invalid_store_before_persisting() {
    let store_path = sprint171_temp_json_path("observer-approved-apply-invalid-store");
    let _ = fs::remove_file(&store_path);
    let mut invalid_target = sprint185_target(
        "invalid-existing",
        core::ObserverAgreementTargetApprovalStatus::Approved,
        core::ObserverAgreementTargetSource::MemberOpinion,
        core::SourceConfidence::High,
        "validated paper evidence",
    );
    invalid_target.source_record_id = Some("invalid-existing".to_string());
    invalid_target.target_bucket = core::SmartCoreHeadBucketNormalizedValue::Unknown;
    let invalid_store = core::ObserverAgreementTargetStore {
        store_id: "invalid-existing-store".to_string(),
        target_set: core::ObserverAgreementTargetSet {
            target_set_id: "invalid-existing-target-set".to_string(),
            target_count: 1,
            approved_count: 1,
            needs_review_count: 0,
            rejected_count: 0,
            targets: vec![invalid_target],
            paper_only: true,
        },
        latest_updated_at: None,
        target_count: 1,
        approved_count: 1,
        needs_review_count: 0,
        rejected_count: 0,
        paper_only: true,
    };
    let mut safe_target = sprint185_target(
        "safe-new",
        core::ObserverAgreementTargetApprovalStatus::Approved,
        core::ObserverAgreementTargetSource::MemberOpinion,
        core::SourceConfidence::High,
        "validated paper evidence",
    );
    safe_target.source_record_id = Some("safe-new".to_string());
    let apply_result = core::apply_observer_approved_targets_v2(
        &[safe_target],
        Some(&invalid_store),
        &core::ObserverApprovedTargetApplyV2Config {
            run_id: "sprint187-invalid-store-before-persist".to_string(),
            converted_targets_input_path: None,
            target_store_input_path: None,
            target_store_output_path: Some(store_path.to_string_lossy().to_string()),
            apply_mode: core::ObserverExplicitApplyMode::ApplyApprovedTargets,
            dry_run: false,
            require_at_least_one_approved_target: true,
            paper_only: true,
        },
        &core::default_observer_explicit_apply_policy(),
    )
    .expect("apply result");

    assert_eq!(
        apply_result.apply_status,
        core::ObserverApprovedTargetApplyV2Status::Blocked
    );
    assert!(!apply_result.wrote_target_store);
    assert!(!store_path.exists());
    let accepted_store = apply_result
        .accepted_target_store
        .as_ref()
        .expect("preview store");
    let acceptance = core::check_observer_target_store_acceptance(accepted_store);
    assert_eq!(
        acceptance.acceptance_status,
        core::ObserverTargetStoreAcceptanceStatus::Rejected
    );
}

#[test]
fn sprint187_target_store_acceptance_requires_approved_only_targets() {
    let store = core::ObserverAgreementTargetStore {
        store_id: "accepted-store".to_string(),
        target_set: core::ObserverAgreementTargetSet {
            target_set_id: "accepted-target-set".to_string(),
            target_count: 1,
            approved_count: 1,
            needs_review_count: 0,
            rejected_count: 0,
            targets: vec![sprint185_target(
                "approved",
                core::ObserverAgreementTargetApprovalStatus::Approved,
                core::ObserverAgreementTargetSource::MemberOpinion,
                core::SourceConfidence::High,
                "validated paper evidence",
            )],
            paper_only: true,
        },
        latest_updated_at: None,
        target_count: 1,
        approved_count: 1,
        needs_review_count: 0,
        rejected_count: 0,
        paper_only: true,
    };
    let check = core::check_observer_target_store_acceptance(&store);
    assert_eq!(
        check.acceptance_status,
        core::ObserverTargetStoreAcceptanceStatus::Accepted
    );
    assert_eq!(check.invalid_count, 0);
}

#[test]
fn sprint187_target_store_acceptance_rejects_input_feature_target() {
    let mut invalid = sprint185_target(
        "invalid-input-feature",
        core::ObserverAgreementTargetApprovalStatus::Approved,
        core::ObserverAgreementTargetSource::MemberOpinion,
        core::SourceConfidence::High,
        "validated paper evidence",
    );
    invalid.not_input_feature = false;
    let store = core::ObserverAgreementTargetStore {
        store_id: "rejected-store".to_string(),
        target_set: core::ObserverAgreementTargetSet {
            target_set_id: "rejected-target-set".to_string(),
            target_count: 1,
            approved_count: 1,
            needs_review_count: 0,
            rejected_count: 0,
            targets: vec![invalid],
            paper_only: true,
        },
        latest_updated_at: None,
        target_count: 1,
        approved_count: 1,
        needs_review_count: 0,
        rejected_count: 0,
        paper_only: true,
    };
    let check = core::check_observer_target_store_acceptance(&store);
    assert_eq!(
        check.acceptance_status,
        core::ObserverTargetStoreAcceptanceStatus::Rejected
    );
    assert!(check.invalid_count > 0);
}

#[test]
fn sprint187_comparison_rerun_v3_uses_accepted_target_store() {
    let (_batch_result, observer_run_result, run_result, ..) = sprint187_apply_governance_fixture(
        core::ObserverExplicitApplyMode::ApplyApprovedTargets,
        false,
    );
    let accepted_store = run_result
        .apply_result
        .accepted_target_store
        .as_ref()
        .expect("accepted store")
        .clone();
    let rerun = core::rerun_observer_comparison_with_accepted_store_v3(
        &observer_run_result,
        &accepted_store,
        &core::ObserverComparisonRerunV3Config {
            run_id: "sprint187-rerun-v3".to_string(),
            use_persisted_target_store: true,
            require_target_store_accepted: true,
            compare_member_opinion: true,
            compare_chairman_decision: true,
            compare_risk_governor: true,
            paper_only: true,
        },
    );
    assert!(rerun.target_store_accepted);
    assert!(rerun.new_target_count >= rerun.previous_target_count);
}

#[test]
fn sprint187_readiness_v3_moves_needs_apply_after_apply_and_blocks_invalid_store() {
    let (_batch_result, _observer_run_result, dry_run_result, ..) =
        sprint187_apply_governance_fixture(core::ObserverExplicitApplyMode::DryRun, true);
    assert_eq!(
        dry_run_result.observer_readiness_v3.readiness_status,
        core::SmartCoreObserverReadinessV3Status::NeedsApply
    );

    let (_batch_result, _observer_run_result, apply_result, ..) =
        sprint187_apply_governance_fixture(
            core::ObserverExplicitApplyMode::ApplyApprovedTargets,
            false,
        );
    assert!(matches!(
        apply_result.observer_readiness_v3.readiness_status,
        core::SmartCoreObserverReadinessV3Status::NonVotingObserverReady
            | core::SmartCoreObserverReadinessV3Status::NonVotingObserverReadyWithWarnings
    ));

    let blocked = core::evaluate_smartcore_observer_readiness_v3(
        apply_result.observer_readiness_v3.previous_readiness_status,
        &apply_result.apply_result,
        &core::ObserverTargetStoreAcceptanceCheck {
            acceptance_status: core::ObserverTargetStoreAcceptanceStatus::Rejected,
            blockers: vec!["invalid store".to_string()],
            ..apply_result.target_store_acceptance_check.clone()
        },
        &apply_result.comparison_rerun_v3,
        &apply_result.ledger_trend_v2,
        apply_result.observer_readiness_v3.observer_safety_status,
        &apply_result.decision_isolation_guard_v3,
    );
    assert_eq!(
        blocked.readiness_status,
        core::SmartCoreObserverReadinessV3Status::BlockedByTargetStore
    );
}

#[test]
fn sprint187_ledger_trend_v2_updates_after_appended_entry() {
    let ledger = core::ObserverComparisonLedger {
        ledger_id: "trend-v2-ledger".to_string(),
        entries: vec![
            core::ObserverComparisonLedgerEntry {
                entry_id: "entry-1".to_string(),
                run_id: "run-1".to_string(),
                timestamp: None,
                observer_member_count: 3,
                comparison_count: 3,
                disagreement_count: 2,
                comparison_summary_status: core::ObserverVsCommitteeComparisonSummaryStatus::Mixed,
                readiness_status: core::SmartCoreObserverLaneReadinessStatus::NeedsMoreTargets,
                target_coverage_status: core::ObserverTargetCoverageClosureStatus::ClosurePlanned,
                safety_status: core::SmartCoreObserverLaneSafetyGuardStatus::Preserved,
                target_count: 1,
                paper_only: true,
            },
            core::ObserverComparisonLedgerEntry {
                entry_id: "entry-2".to_string(),
                run_id: "run-2".to_string(),
                timestamp: None,
                observer_member_count: 3,
                comparison_count: 3,
                disagreement_count: 1,
                comparison_summary_status:
                    core::ObserverVsCommitteeComparisonSummaryStatus::MostlyAgree,
                readiness_status:
                    core::SmartCoreObserverLaneReadinessStatus::NonVotingObserverReadyWithWarnings,
                target_coverage_status: core::ObserverTargetCoverageClosureStatus::NoClosureNeeded,
                safety_status: core::SmartCoreObserverLaneSafetyGuardStatus::Preserved,
                target_count: 3,
                paper_only: true,
            },
        ],
        entry_count: 2,
        latest_entry_id: Some("entry-2".to_string()),
        paper_only: true,
    };
    let trend = core::compute_observer_comparison_ledger_trend_v2(&ledger);
    assert_eq!(trend.entry_count, 2);
    assert_eq!(
        trend.target_count_trend,
        core::ObserverTargetCountTrendDirection::Increasing
    );
}

#[test]
fn sprint187_chairman_reward_penalty_contract_is_contract_only_and_no_mutation() {
    let contract = core::default_chairman_reward_penalty_contract();
    assert_eq!(
        contract.status,
        core::ChairmanGovernanceContractStatus::ContractOnly
    );
    assert!(!contract.can_mutate_score);
    assert!(!contract.can_mutate_voice_weight);
    assert!(!contract.can_promote_member);
    assert!(!contract.can_demote_member);
    assert!(!contract.can_override_risk_governor);

    let mut unsafe_contract = contract.clone();
    unsafe_contract.can_override_risk_governor = true;
    let readiness = core::evaluate_chairman_governance_readiness(
        &unsafe_contract,
        None,
        None,
        None,
        true,
        true,
        core::SmartCoreObserverLaneSafetyGuardStatus::Preserved,
    );
    assert_eq!(
        readiness.readiness_status,
        core::ChairmanGovernanceReadinessStatus::BlockedBySafety
    );
    assert!(!readiness.can_start_shadow_reward_evaluation);
}

#[test]
fn sprint187_chairman_governance_readiness_is_shadow_only_and_owner_summary_mentions_inactive() {
    let (batch_result, _observer_run_result, run_result, ..) = sprint187_apply_governance_fixture(
        core::ObserverExplicitApplyMode::ApplyApprovedTargets,
        false,
    );
    assert!(matches!(
        run_result
            .chairman_governance_readiness_check
            .readiness_status,
        core::ChairmanGovernanceReadinessStatus::ShadowGovernanceReady
            | core::ChairmanGovernanceReadinessStatus::ShadowGovernanceReadyWithWarnings
            | core::ChairmanGovernanceReadinessStatus::NeedsMoreObserverHistory
            | core::ChairmanGovernanceReadinessStatus::NeedsMorePaperEvidence
    ));
    let summary = run_result.owner_summary.expect("owner summary");
    assert!(
        summary
            .message
            .contains("Chairman reward/penalty contract is prepared but not active")
    );
    assert_eq!(
        summary.approved_seed_count,
        batch_result
            .observer_seed_apply_trend_run_result
            .as_ref()
            .map(|seed| seed
                .controlled_apply_smoke_result
                .seed_build_result
                .approved_seed_count)
            .unwrap_or(summary.approved_seed_count)
    );
}

#[test]
fn sprint187_decision_isolation_v3_passes_and_fails_on_score_or_member_opinion_mutation() {
    let (batch_result, _observer_run_result, run_result, ..) = sprint187_apply_governance_fixture(
        core::ObserverExplicitApplyMode::ApplyApprovedTargets,
        false,
    );
    assert_eq!(
        run_result.decision_isolation_guard_v3.guard_status,
        core::ObserverApprovedApplyDecisionIsolationGuardV3Status::Preserved
    );

    let mut mutating_contract = run_result.chairman_reward_penalty_contract.clone();
    mutating_contract.can_mutate_score = true;
    let violated_score = core::evaluate_observer_approved_apply_decision_isolation_v3(
        &run_result.apply_result,
        &mutating_contract,
        &batch_result,
        Some(&batch_result),
    );
    assert_eq!(
        violated_score.guard_status,
        core::ObserverApprovedApplyDecisionIsolationGuardV3Status::Violated
    );
    assert!(violated_score.chairman_contract_mutated_score);

    let mut mutating_voice_contract = run_result.chairman_reward_penalty_contract.clone();
    mutating_voice_contract.can_mutate_voice_weight = true;
    let violated_voice = core::evaluate_observer_approved_apply_decision_isolation_v3(
        &run_result.apply_result,
        &mutating_voice_contract,
        &batch_result,
        Some(&batch_result),
    );
    assert_eq!(
        violated_voice.guard_status,
        core::ObserverApprovedApplyDecisionIsolationGuardV3Status::Violated
    );
    assert!(violated_voice.chairman_contract_mutated_voice);

    let mut promoting_contract = run_result.chairman_reward_penalty_contract.clone();
    promoting_contract.can_promote_member = true;
    let violated_promotion = core::evaluate_observer_approved_apply_decision_isolation_v3(
        &run_result.apply_result,
        &promoting_contract,
        &batch_result,
        Some(&batch_result),
    );
    assert_eq!(
        violated_promotion.guard_status,
        core::ObserverApprovedApplyDecisionIsolationGuardV3Status::Violated
    );
    assert!(violated_promotion.chairman_contract_promoted_or_demoted_member);

    let mut mutated_batch = batch_result.clone();
    mutated_batch.member_opinions.push(MemberOpinion {
        member_id: "trend-us-medium".to_string(),
        symbol: "AAPL".to_string(),
        market_scope: MarketScope::UsShortTerm,
        stance: MemberStance::BuyProposal,
        confidence: 0.9,
        expected_return_hint: 0.04,
        risk_hint: 0.2,
        evidence_notes: vec!["leak".to_string()],
        event_triggered: false,
        event_reason: None,
    });
    let violated_opinion = core::evaluate_observer_approved_apply_decision_isolation_v3(
        &run_result.apply_result,
        &run_result.chairman_reward_penalty_contract,
        &batch_result,
        Some(&mutated_batch),
    );
    assert_eq!(
        violated_opinion.guard_status,
        core::ObserverApprovedApplyDecisionIsolationGuardV3Status::Violated
    );
    assert!(violated_opinion.applied_targets_used_as_member_opinion);
}

#[test]
fn sprint187_run_keeps_committee_score_voice_unchanged_and_is_deterministic() {
    let (batch_a, _observer_a, first, ..) =
        sprint187_apply_governance_fixture(core::ObserverExplicitApplyMode::DryRun, true);
    let (_batch_b, _observer_b, second, ..) =
        sprint187_apply_governance_fixture(core::ObserverExplicitApplyMode::DryRun, true);
    assert!(
        !first
            .decision_isolation_guard_v3
            .applied_targets_changed_committee_decision
    );
    assert!(
        !first
            .decision_isolation_guard_v3
            .applied_targets_changed_member_score
    );
    assert!(
        !first
            .decision_isolation_guard_v3
            .applied_targets_changed_voice_weight
    );
    assert!(
        !first
            .decision_isolation_guard_v3
            .applied_targets_used_as_trade_signal
    );
    assert!(
        !first
            .decision_isolation_guard_v3
            .applied_targets_used_as_order
    );
    assert_eq!(
        first.chairman_reward_penalty_contract.status,
        core::ChairmanGovernanceContractStatus::ContractOnly
    );
    assert!(!first.chairman_reward_penalty_contract.can_mutate_score);
    assert!(
        !first
            .chairman_reward_penalty_contract
            .can_mutate_voice_weight
    );
    assert!(batch_a.safety_summary.paper_only);
    assert!(batch_a.safety_summary.no_model_training);
    assert!(batch_a.safety_summary.no_weight_update);
    assert!(batch_a.safety_summary.no_checkpoint);
    assert!(batch_a.safety_summary.no_live_inference);
    assert!(batch_a.safety_summary.no_broker_order_account);
    assert!(batch_a.safety_summary.no_real_order_path);
    assert_eq!(
        first.apply_result.approved_input_count,
        second.apply_result.approved_input_count
    );
    assert_eq!(
        first.apply_result.apply_status,
        second.apply_result.apply_status
    );
    assert_eq!(
        first.observer_readiness_v3.readiness_status,
        second.observer_readiness_v3.readiness_status
    );
}

#[test]
fn sprint188_apply_verification_profile_validation_checks_mode_and_dry_run() {
    let valid = core::validate_observer_apply_verification_profile(
        &core::ObserverApplyVerificationProfile {
            profile_id: "sprint188-valid".to_string(),
            config_path: None,
            apply_mode: core::ObserverExplicitApplyMode::ApplyApprovedTargets,
            dry_run: false,
            target_store_output_path: Some("target/sprint188_valid_store.json".to_string()),
            apply_output_path: Some("target/sprint188_valid_output.json".to_string()),
            main_example_safe_default: false,
            verification_profile: true,
            paper_only: true,
        },
    );
    assert!(valid.valid);
    assert_eq!(
        valid.validation_status,
        core::ObserverApplyVerificationProfileValidationStatus::Valid
    );

    let invalid = core::validate_observer_apply_verification_profile(
        &core::ObserverApplyVerificationProfile {
            profile_id: "sprint188-invalid-main".to_string(),
            config_path: None,
            apply_mode: core::ObserverExplicitApplyMode::ApplyApprovedTargets,
            dry_run: false,
            target_store_output_path: Some("target/sprint188_invalid_store.json".to_string()),
            apply_output_path: None,
            main_example_safe_default: true,
            verification_profile: true,
            paper_only: true,
        },
    );
    assert!(!invalid.valid);
    assert_eq!(
        invalid.validation_status,
        core::ObserverApplyVerificationProfileValidationStatus::Invalid
    );

    let missing_path = core::validate_observer_apply_verification_profile(
        &core::ObserverApplyVerificationProfile {
            profile_id: "sprint188-missing-path".to_string(),
            config_path: None,
            apply_mode: core::ObserverExplicitApplyMode::ApplyApprovedTargets,
            dry_run: false,
            target_store_output_path: None,
            apply_output_path: Some("target/sprint188_missing_path_output.json".to_string()),
            main_example_safe_default: false,
            verification_profile: true,
            paper_only: true,
        },
    );
    assert!(!missing_path.valid);
    assert!(!missing_path.output_path_valid);

    let implicit_profile = core::validate_observer_apply_verification_profile(
        &core::ObserverApplyVerificationProfile {
            profile_id: "sprint188-implicit".to_string(),
            config_path: None,
            apply_mode: core::ObserverExplicitApplyMode::ApplyApprovedTargets,
            dry_run: false,
            target_store_output_path: Some("target/sprint188_implicit_store.json".to_string()),
            apply_output_path: None,
            main_example_safe_default: false,
            verification_profile: false,
            paper_only: true,
        },
    );
    assert!(!implicit_profile.valid);
    assert!(
        implicit_profile
            .blockers
            .iter()
            .any(|blocker| blocker.contains("verification profile flag"))
    );
}

#[test]
fn sprint188_target_store_write_proof_and_readiness_closure_complete_after_non_dry_apply() {
    let (_batch_result, _observer_run_result, run_result, store_path, output_path) =
        sprint188_apply_verify_shadow_fixture(
            core::ObserverExplicitApplyMode::ApplyApprovedTargets,
            false,
            true,
        );
    assert!(store_path.exists());
    assert!(output_path.exists());
    assert!(matches!(
        run_result.target_store_write_proof.proof_status,
        core::ObserverTargetStoreWriteProofStatus::Proven
            | core::ObserverTargetStoreWriteProofStatus::ProvenWithWarnings
    ));
    assert!(matches!(
        run_result.readiness_closure_check.closure_status,
        core::ObserverReadinessV3ClosureStatus::Closed
            | core::ObserverReadinessV3ClosureStatus::ClosedWithWarnings
    ));
    assert!(matches!(
        run_result.run_status,
        core::ObserverApplyVerifyAndChairmanShadowRunStatus::Passed
            | core::ObserverApplyVerifyAndChairmanShadowRunStatus::PassedWithWarnings
    ));

    let mut missing_file_apply = run_result.apply_result.clone();
    let missing_store_path = sprint171_temp_json_path("observer-apply-verify-missing-proof");
    let _ = fs::remove_file(&missing_store_path);
    missing_file_apply.target_store_path = Some(missing_store_path.to_string_lossy().to_string());
    missing_file_apply.wrote_target_store = true;
    let missing_proof = core::prove_observer_target_store_write(
        "sprint188-missing-proof".to_string(),
        &missing_file_apply,
        None,
    );
    assert_eq!(
        missing_proof.proof_status,
        core::ObserverTargetStoreWriteProofStatus::Failed
    );

    let violated_isolation_guard = core::ObserverApprovedApplyDecisionIsolationGuardV3 {
        applied_targets_used_as_input_feature: false,
        applied_targets_used_as_member_opinion: false,
        applied_targets_used_in_committee_session: false,
        applied_targets_used_in_chairman_decision: false,
        applied_targets_used_in_risk_governor: false,
        applied_targets_used_as_trade_signal: false,
        applied_targets_used_as_order: false,
        applied_targets_changed_member_score: true,
        applied_targets_changed_voice_weight: false,
        applied_targets_changed_committee_decision: false,
        chairman_contract_mutated_score: false,
        chairman_contract_mutated_voice: false,
        chairman_contract_promoted_or_demoted_member: false,
        guard_status: core::ObserverApprovedApplyDecisionIsolationGuardV3Status::Violated,
        violations: vec!["injected score mutation".to_string()],
        paper_only: true,
    };
    let blocked_closure = core::check_observer_readiness_v3_closure(
        "sprint188-blocked-closure".to_string(),
        &run_result.observer_readiness_v3,
        &run_result.target_store_write_proof,
        &run_result.target_store_acceptance_check,
        &run_result.comparison_rerun_v3,
        &run_result.ledger_trend_v2,
        &violated_isolation_guard,
    );
    assert_eq!(
        blocked_closure.closure_status,
        core::ObserverReadinessV3ClosureStatus::Blocked
    );
}

#[test]
fn sprint188_invalid_profile_stops_before_target_store_write() {
    let (batch_result, observer_run_result, seed_run, ..) =
        sprint186_seed_apply_fixture(true, false);
    let store_path = sprint171_temp_json_path("observer-apply-verify-invalid-profile-store");
    let output_path = sprint171_temp_json_path("observer-apply-verify-invalid-profile-output");
    let _ = fs::remove_file(&store_path);
    let _ = fs::remove_file(&output_path);
    let err = core::run_observer_apply_verify_and_chairman_shadow(
        &batch_result,
        &seed_run
            .controlled_apply_smoke_result
            .seed_conversion_result
            .target_records,
        &observer_run_result,
        &core::ObserverApplyVerifyAndChairmanShadowRunConfig {
            run_id: "sprint188-invalid-profile-run".to_string(),
            enabled: true,
            apply_verification_config_path: Some("https://example.invalid/config.toml".to_string()),
            apply_mode: core::ObserverExplicitApplyMode::ApplyApprovedTargets,
            dry_run: false,
            target_store_output_path: Some(store_path.to_string_lossy().to_string()),
            output_path: Some(output_path.to_string_lossy().to_string()),
            run_chairman_shadow_governance: true,
            emit_owner_summary: true,
            paper_only: true,
        },
    )
    .expect_err("invalid verification profile must stop before apply");

    assert!(err.contains("profile invalid"));
    assert!(!store_path.exists());
    assert!(!output_path.exists());
}

#[test]
fn sprint188_chairman_shadow_governance_stays_eval_only_and_safety_preserved() {
    let (_batch_result, _observer_run_result, run_result, ..) =
        sprint188_apply_verify_shadow_fixture(
            core::ObserverExplicitApplyMode::ApplyApprovedTargets,
            false,
            true,
        );
    let input_set = run_result
        .chairman_shadow_governance_inputs
        .as_ref()
        .expect("shadow input set");
    let evaluation = run_result
        .chairman_shadow_governance_evaluation
        .as_ref()
        .expect("shadow evaluation");
    let safety = run_result
        .chairman_shadow_governance_safety
        .as_ref()
        .expect("shadow safety");
    assert!(input_set.record_count > 0);
    assert!(evaluation.no_score_mutation);
    assert!(evaluation.no_voice_mutation);
    assert!(evaluation.no_promotion_demotion);
    assert!(evaluation.no_risk_governor_override);
    assert_eq!(
        safety.guard_status,
        core::ChairmanShadowGovernanceSafetyGuardStatus::Preserved
    );
    assert!(safety.violations.is_empty());

    let mut score_mutated = _batch_result.clone();
    score_mutated.score_updates.push(core::MemberScoreUpdate {
        member_id: "trend-kr-short".to_string(),
        previous_score: 0.5,
        new_score: 0.7,
        previous_voice_weight: 0.5,
        new_voice_weight: 0.6,
        update_reason: core::MemberScoreUpdateReason::GoodCall,
        promoted: true,
        demoted: false,
    });
    let violated_score = core::evaluate_chairman_shadow_governance_safety_guard(
        evaluation,
        &_batch_result,
        Some(&score_mutated),
    );
    assert_eq!(
        violated_score.guard_status,
        core::ChairmanShadowGovernanceSafetyGuardStatus::Violated
    );
    assert!(violated_score.score_mutation_detected);
    assert!(violated_score.voice_mutation_detected);
    assert!(violated_score.promotion_detected);

    let mut risk_mutated = _batch_result.clone();
    if let Some(decision) = risk_mutated.chairman_decisions.first_mut() {
        decision.risk_governor_status =
            if decision.risk_governor_status == core::RiskGovernorStatus::Vetoed {
                core::RiskGovernorStatus::Passed
            } else {
                core::RiskGovernorStatus::Vetoed
            };
    }
    let violated_risk = core::evaluate_chairman_shadow_governance_safety_guard(
        evaluation,
        &_batch_result,
        Some(&risk_mutated),
    );
    assert_eq!(
        violated_risk.guard_status,
        core::ChairmanShadowGovernanceSafetyGuardStatus::Violated
    );
    assert!(violated_risk.risk_governor_override_detected);
}

#[test]
fn sprint188_owner_summary_v2_emits_non_voting_message_and_counts() {
    let (_batch_result, _observer_run_result, run_result, ..) =
        sprint188_apply_verify_shadow_fixture(
            core::ObserverExplicitApplyMode::ApplyApprovedTargets,
            false,
            true,
        );
    let summary = run_result.owner_summary.as_ref().expect("owner summary v2");
    assert!(summary.non_voting);
    assert!(summary.read_only);
    assert!(summary.eval_only);
    assert!(summary.chairman_contract_only);
    assert!(summary.not_investment_signal);
    assert!(summary.not_committee_opinion);
    assert!(
        summary
            .message
            .contains("Approved observer targets were applied only to local evaluation store")
    );
    assert!(
        summary
            .message
            .contains("Chairman governance signals are shadow-only candidates")
    );
    assert_eq!(
        summary.reward_candidate_count + summary.penalty_candidate_count,
        run_result
            .chairman_shadow_governance_evaluation
            .as_ref()
            .map(|evaluation| evaluation.reward_candidate_count + evaluation.penalty_candidate_count)
            .unwrap_or(0)
    );
    assert!(summary.message.contains("No score"));
    assert!(summary.message.contains("voice"));
}

#[test]
fn sprint188_main_defaults_do_not_request_apply_verify_run() {
    assert!(
        !core::observer_apply_verify_chairman_shadow_requested_from_flags(
            false,
            core::ObserverExplicitApplyMode::DryRun,
            Some(&"target/minimal_observer_apply_verify_target_store.json".to_string()),
            Some(&"target/minimal_observer_apply_verify_and_shadow.json".to_string()),
            None,
            true,
            true,
            true,
        )
    );
    assert!(
        core::observer_apply_verify_chairman_shadow_requested_from_flags(
            true,
            core::ObserverExplicitApplyMode::DryRun,
            Some(&"target/minimal_observer_apply_verify_target_store.json".to_string()),
            Some(&"target/minimal_observer_apply_verify_and_shadow.json".to_string()),
            None,
            true,
            true,
            true,
        )
    );
    assert!(
        core::observer_apply_verify_chairman_shadow_requested_from_flags(
            false,
            core::ObserverExplicitApplyMode::ApplyApprovedTargets,
            None,
            None,
            None,
            false,
            false,
            false,
        )
    );
}

#[test]
fn paper_evidence_review_required_with_price_promotes_medium_label() {
    let dataset = sprint156_replay_dataset(vec![sprint156_replay_example(
        "replay-review-valid",
        "AAPL",
        MarketScope::UsShortTerm,
    )]);
    let policy = PaperLabelValidationPolicy::default();
    let build = build_validated_replay_dataset_with_paper_evidence(
        &dataset,
        &[sprint156_paper_evidence(
            "review-required-valid-price",
            Some("replay-review-valid"),
            "AAPL",
            MarketScope::UsShortTerm,
            MemberExperienceOutcome::PaperPositive,
            ReplayLabelSource::ReviewRequired,
            ReplayLabelConfidence::ReviewRequired,
            Some(0.03),
        )],
        &policy,
        soma_zero::league::minimal_ai_committee_core::ValidatedReplayDatasetBuildConfig {
            source_sanitized_dataset_id: dataset.dataset_id.clone(),
            label_validation_policy: policy.clone(),
            backtest_label_contract: None,
            require_validated_labels_for_training: true,
            allow_ready_with_warnings_for_medium_confidence: true,
            reject_rejected_labels: true,
            paper_only: true,
        },
    );

    assert_eq!(build.promoted_count, 1);
    assert_eq!(
        build.dataset.examples[0]
            .target_labels
            .as_ref()
            .map(|labels| (labels.label_source, labels.label_confidence)),
        Some((
            ReplayLabelSource::ValidatedPaperLabel,
            ReplayLabelConfidence::Medium
        ))
    );
}

#[test]
fn paper_evidence_promotes_only_valid_labels_and_preserves_inputs() {
    let dataset = sprint156_replay_dataset(vec![
        sprint156_replay_example("replay-positive", "005930.KS", MarketScope::KoreaShortTerm),
        sprint156_replay_example("replay-negative", "MSFT", MarketScope::UsLongTerm),
        sprint156_replay_example("replay-review", "AAPL", MarketScope::UsShortTerm),
        sprint156_replay_example("replay-contradict", "BTCUSDT", MarketScope::CryptoShortTerm),
        sprint156_replay_example("replay-simulated", "ETHUSDT", MarketScope::CryptoLongTerm),
        sprint156_replay_example("replay-backtest", "000660.KS", MarketScope::KoreaLongTerm),
    ]);
    let before_inputs: std::collections::BTreeMap<_, _> = dataset
        .examples
        .iter()
        .map(|example| {
            (
                example.replay_id.clone(),
                serde_json::to_value(&example.input_features).expect("input json"),
            )
        })
        .collect();
    let policy = PaperLabelValidationPolicy::default();
    let evidence_records = vec![
        sprint156_paper_evidence(
            "valid-positive",
            Some("replay-positive"),
            "005930.KS",
            MarketScope::KoreaShortTerm,
            MemberExperienceOutcome::PaperPositive,
            ReplayLabelSource::ManualPaperLabel,
            ReplayLabelConfidence::High,
            Some(0.03),
        ),
        sprint156_paper_evidence(
            "valid-negative",
            Some("replay-negative"),
            "MSFT",
            MarketScope::UsLongTerm,
            MemberExperienceOutcome::PaperNegative,
            ReplayLabelSource::ManualPaperLabel,
            ReplayLabelConfidence::Medium,
            Some(-0.03),
        ),
        sprint156_paper_evidence(
            "missing-price",
            Some("replay-review"),
            "AAPL",
            MarketScope::UsShortTerm,
            MemberExperienceOutcome::PaperPositive,
            ReplayLabelSource::ReviewRequired,
            ReplayLabelConfidence::ReviewRequired,
            None,
        ),
        sprint156_paper_evidence(
            "contradictory",
            Some("replay-contradict"),
            "BTCUSDT",
            MarketScope::CryptoShortTerm,
            MemberExperienceOutcome::PaperPositive,
            ReplayLabelSource::ManualPaperLabel,
            ReplayLabelConfidence::High,
            Some(-0.04),
        ),
        sprint156_paper_evidence(
            "simulated-high",
            Some("replay-simulated"),
            "ETHUSDT",
            MarketScope::CryptoLongTerm,
            MemberExperienceOutcome::PaperPositive,
            ReplayLabelSource::SimulatedFixture,
            ReplayLabelConfidence::High,
            Some(0.04),
        ),
        sprint156_paper_evidence(
            "backtest-deferred",
            Some("replay-backtest"),
            "000660.KS",
            MarketScope::KoreaLongTerm,
            MemberExperienceOutcome::PaperPositive,
            ReplayLabelSource::PaperBacktestDeferred,
            ReplayLabelConfidence::High,
            Some(0.04),
        ),
    ];
    let build = build_validated_replay_dataset_with_paper_evidence(
        &dataset,
        &evidence_records,
        &policy,
        soma_zero::league::minimal_ai_committee_core::ValidatedReplayDatasetBuildConfig {
            source_sanitized_dataset_id: dataset.dataset_id.clone(),
            label_validation_policy: policy.clone(),
            backtest_label_contract: Some(ready_backtest_contract()),
            require_validated_labels_for_training: true,
            allow_ready_with_warnings_for_medium_confidence: true,
            reject_rejected_labels: false,
            paper_only: true,
        },
    );

    assert_eq!(build.promoted_count, 2);
    assert_eq!(build.rejected_count, 1);
    assert!(build.needs_review_count >= 3);
    assert_eq!(build.unmatched_evidence_count, 0);
    assert_eq!(build.ambiguous_match_count, 0);
    assert_eq!(
        promote_labels_with_paper_evidence(&dataset, &evidence_records, &policy),
        promote_labels_with_paper_evidence(&dataset, &evidence_records, &policy)
    );
    let after_inputs: std::collections::BTreeMap<_, _> = build
        .dataset
        .examples
        .iter()
        .map(|example| {
            (
                example.replay_id.clone(),
                serde_json::to_value(&example.input_features).expect("input json"),
            )
        })
        .collect();
    assert_eq!(before_inputs, after_inputs);
    assert!(
        build.paper_evidence_promotion_results.iter().any(
            |result| result.promotion_status == ValidatedPaperEvidencePromotionStatus::Promoted
        )
    );

    let summary = summarize_label_quality(&build);
    assert!(summary.validated_label_ratio > 0.0);
    let coverage = evaluate_replay_coverage_targets(
        &build.dataset,
        ReplayCoverageTargetConfig {
            min_total_examples: 1,
            min_examples_total: 1,
            min_examples_per_member: 1,
            require_non_weak_label_source: false,
            paper_only: true,
            ..ReplayCoverageTargetConfig::default()
        },
    );
    let leakage = check_replay_data_leakage(&build.dataset);
    let recheck = soma_zero::league::minimal_ai_committee_core::recheck_training_readiness_after_paper_evidence(
        &build,
        &coverage,
        &leakage,
    );
    assert_eq!(recheck.promoted_by_paper_evidence_count, 2);
    assert!(recheck.validated_label_ratio_after_evidence > 0.0);

    let all_validated = sprint156_replay_dataset(vec![
        {
            let mut example = sprint156_replay_example(
                "ready-medium-1",
                "005930.KS",
                MarketScope::KoreaShortTerm,
            );
            example.target_labels.as_mut().expect("labels").label_source =
                ReplayLabelSource::ValidatedPaperLabel;
            example
                .target_labels
                .as_mut()
                .expect("labels")
                .label_confidence = ReplayLabelConfidence::Medium;
            example
        },
        {
            let mut example =
                sprint156_replay_example("ready-medium-2", "MSFT", MarketScope::UsLongTerm);
            example.target_labels.as_mut().expect("labels").label_source =
                ReplayLabelSource::ValidatedPaperLabel;
            example
                .target_labels
                .as_mut()
                .expect("labels")
                .label_confidence = ReplayLabelConfidence::Medium;
            example
        },
    ]);
    let ready_gate = evaluate_offline_training_readiness_with_thresholds(
        &all_validated,
        &MemberExperienceStore::new("ready-medium-store", Vec::new()),
        1,
        0,
    );
    assert_eq!(
        ready_gate.readiness_status,
        OfflineTrainingReadinessStatus::ReadyWithWarnings
    );
}

#[test]
fn paper_evidence_quality_summary_recommends_next_actions() {
    let dataset = sprint156_replay_dataset(vec![
        sprint156_replay_example("replay-positive", "005930.KS", MarketScope::KoreaShortTerm),
        sprint156_replay_example("replay-ambiguous-1", "AAPL", MarketScope::UsShortTerm),
        sprint156_replay_example("replay-ambiguous-2", "AAPL", MarketScope::UsShortTerm),
    ]);
    let load = validate_paper_outcome_evidence_file(PaperOutcomeEvidenceFile {
        schema_version: "paper-outcome-evidence.v1".to_string(),
        evidence_file_id: "quality-unit".to_string(),
        created_at: None,
        source_label: "unit".to_string(),
        records: vec![
            sprint156_paper_evidence(
                "valid-positive",
                Some("replay-positive"),
                "005930.KS",
                MarketScope::KoreaShortTerm,
                MemberExperienceOutcome::PaperPositive,
                ReplayLabelSource::ManualPaperLabel,
                ReplayLabelConfidence::High,
                Some(0.02),
            ),
            sprint156_paper_evidence(
                "ambiguous",
                None,
                "AAPL",
                MarketScope::UsShortTerm,
                MemberExperienceOutcome::PaperPositive,
                ReplayLabelSource::ManualPaperLabel,
                ReplayLabelConfidence::Medium,
                None,
            ),
        ],
        paper_only: true,
    });
    let matching = match_paper_outcome_evidence_to_replay(&dataset, &load.records);
    let promotions = promote_labels_with_paper_evidence(
        &dataset,
        &load.records,
        &PaperLabelValidationPolicy::default(),
    );
    let summary = summarize_paper_outcome_evidence_quality(&load, &matching, &promotions);
    assert_eq!(summary.evidence_file_id, "quality-unit");
    assert_eq!(summary.promoted_records, 1);
    assert_eq!(summary.ambiguous_records, 1);
    assert_eq!(
        summary.ambiguous_evidence_ids,
        vec!["ambiguous".to_string()]
    );
    assert_eq!(
        summary.candidate_evidence_ids,
        vec!["ambiguous".to_string()]
    );
    let plan = build_validated_label_ratio_expansion_plan(&dataset, &summary, 0.8);
    assert_eq!(plan.ambiguous_evidence_ids, vec!["ambiguous".to_string()]);
    assert_eq!(plan.candidate_evidence_ids, vec!["ambiguous".to_string()]);
    assert!(
        summary
            .recommended_next_actions
            .contains(&RecommendedPaperEvidenceAction::ResolveAmbiguousMatches)
    );
    assert!(
        summary
            .recommended_next_actions
            .contains(&RecommendedPaperEvidenceAction::AddBacktestEngine)
    );
}

#[test]
fn paper_label_validation_and_promotion_are_explicit_and_safe() {
    let policy = PaperLabelValidationPolicy::default();
    let simulated_high = sample_label_evidence(
        None,
        MemberExperienceOutcome::PaperPositive,
        ReplayLabelSource::SimulatedFixture,
        Some(0.02),
    );
    let result = validate_paper_label(&simulated_high, &policy);
    assert_eq!(
        result.validation_status,
        LabelEvidenceValidationStatus::Rejected
    );
    assert_eq!(
        result.promoted_label_source,
        ReplayLabelSource::RejectedLabel
    );

    let manual_without_evidence = sample_label_evidence(
        None,
        MemberExperienceOutcome::PaperPositive,
        ReplayLabelSource::ManualPaperLabel,
        None,
    );
    let result = validate_paper_label(&manual_without_evidence, &policy);
    assert_eq!(
        result.validation_status,
        LabelEvidenceValidationStatus::NeedsReview
    );
    assert_eq!(
        result.promoted_confidence,
        ReplayLabelConfidence::ReviewRequired
    );

    let validated = sample_label_evidence(
        None,
        MemberExperienceOutcome::PaperPositive,
        ReplayLabelSource::ManualPaperLabel,
        Some(0.03),
    );
    let result = validate_paper_label(&validated, &policy);
    assert_eq!(
        result.validation_status,
        LabelEvidenceValidationStatus::Validated
    );
    assert_eq!(
        result.promoted_label_source,
        ReplayLabelSource::ValidatedPaperLabel
    );

    let contradictory = sample_label_evidence(
        None,
        MemberExperienceOutcome::PaperPositive,
        ReplayLabelSource::ManualPaperLabel,
        Some(-0.03),
    );
    let result = validate_paper_label(&contradictory, &policy);
    assert_eq!(
        result.validation_status,
        LabelEvidenceValidationStatus::Rejected
    );

    let mut unsafe_evidence = validated.clone();
    unsafe_evidence.evidence_items.news_context_evidence =
        Some("broker account order field".to_string());
    let result = validate_paper_label(&unsafe_evidence, &policy);
    assert_eq!(
        result.validation_status,
        LabelEvidenceValidationStatus::Rejected
    );
}

#[test]
fn backtest_contract_check_is_contract_only_and_leakage_guarded() {
    let ready = ready_backtest_contract();
    let first = check_backtest_label_contract(&ready);
    let second = check_backtest_label_contract(&ready);
    assert_eq!(first, second);
    assert_eq!(first.status, BacktestLabelContractStatus::ContractReady);
    assert!(!first.can_generate_validated_backtest_label);

    let mut incomplete = ready.clone();
    incomplete.entry_price_source = BacktestEntryPriceSource::Deferred;
    let checked = check_backtest_label_contract(&incomplete);
    assert_eq!(
        checked.status,
        BacktestLabelContractStatus::ContractIncomplete
    );
    assert!(
        checked
            .missing_fields
            .contains(&"entry_price_source".to_string())
    );

    let mut unsafe_contract = ready;
    unsafe_contract.leakage_guard = vec![BacktestLeakageGuard::NoFutureInput];
    let checked = check_backtest_label_contract(&unsafe_contract);
    assert_eq!(checked.status, BacktestLabelContractStatus::ContractUnsafe);
    assert!(!checked.leakage_safe);
}

#[test]
fn validated_replay_builder_preserves_inputs_and_updates_label_metadata_only() {
    let example = ReplayExample {
        replay_id: "sprint155-replay-1".to_string(),
        member_id: "TrendEntryAI".to_string(),
        symbol: "005930.KS".to_string(),
        market_scope: MarketScope::KoreaShortTerm,
        input_features: ReplayInputFeatures {
            market_data_summary: "pre-decision trend summary".to_string(),
            news_summary: "local paper-only summary".to_string(),
            owner_context_summary: Some("owner context before decision".to_string()),
            memory_state_summary: Some("member memory before decision".to_string()),
        },
        target: ReplayTarget {
            stance: MemberStance::BuyProposal,
            confidence_calibration: MemberLearningLabel::Reinforce,
            risk_label: "risk_passed".to_string(),
            evidence_label: "evidence_sufficient".to_string(),
            outcome_label: MemberExperienceOutcome::PaperPositive,
        },
        sanitized_input_features: None,
        target_labels: Some(ReplayTargetLabels {
            stance_target: MemberStance::BuyProposal,
            confidence_calibration_target: MemberLearningLabel::Reinforce,
            risk_label: "risk_passed".to_string(),
            evidence_label: "evidence_sufficient".to_string(),
            outcome_label: MemberExperienceOutcome::PaperPositive,
            attribution_label: MemberScoreUpdateReason::GoodCall,
            label_source: ReplayLabelSource::ReviewRequired,
            label_confidence: ReplayLabelConfidence::ReviewRequired,
            paper_only: true,
        }),
        post_decision_context: None,
        sample_weight: 1.0,
        paper_only: true,
    };
    let dataset = ReplayDataset {
        dataset_id: "sprint155-source-dataset".to_string(),
        examples: vec![example.clone()],
        member_count: 1,
        example_count: 1,
        generated_from_store_id: "sprint155-source-store".to_string(),
        paper_only: true,
    };
    let evidence = sample_label_evidence(
        Some(example.replay_id.clone()),
        example.target.outcome_label,
        ReplayLabelSource::ManualPaperLabel,
        Some(0.03),
    );
    let gate =
        evaluate_label_promotion(&example, &evidence, &PaperLabelValidationPolicy::default());
    assert_eq!(gate.promotion_status, LabelPromotionStatus::Promoted);
    assert!(gate.promotion_allowed);

    let build = build_validated_replay_dataset(
        &dataset,
        &[evidence],
        soma_zero::league::minimal_ai_committee_core::ValidatedReplayDatasetBuildConfig {
            source_sanitized_dataset_id: dataset.dataset_id.clone(),
            label_validation_policy: PaperLabelValidationPolicy::default(),
            backtest_label_contract: Some(ready_backtest_contract()),
            require_validated_labels_for_training: true,
            allow_ready_with_warnings_for_medium_confidence: true,
            reject_rejected_labels: true,
            paper_only: true,
        },
    );
    assert_eq!(build.promoted_count, 1);
    assert_eq!(
        build.dataset.examples[0].input_features,
        dataset.examples[0].input_features
    );
    assert_eq!(
        build.dataset.examples[0]
            .target_labels
            .as_ref()
            .map(|labels| labels.label_source),
        Some(ReplayLabelSource::ValidatedPaperLabel)
    );
}

#[test]
fn validated_replay_builder_keeps_backtest_labels_deferred_without_engine() {
    let example = ReplayExample {
        replay_id: "sprint155-backtest-deferred-replay-1".to_string(),
        member_id: "TrendEntryAI".to_string(),
        symbol: "005930.KS".to_string(),
        market_scope: MarketScope::KoreaShortTerm,
        input_features: ReplayInputFeatures {
            market_data_summary: "pre-decision trend summary".to_string(),
            news_summary: "local paper-only summary".to_string(),
            owner_context_summary: None,
            memory_state_summary: None,
        },
        target: ReplayTarget {
            stance: MemberStance::BuyProposal,
            confidence_calibration: MemberLearningLabel::Reinforce,
            risk_label: "risk_passed".to_string(),
            evidence_label: "evidence_sufficient".to_string(),
            outcome_label: MemberExperienceOutcome::PaperPositive,
        },
        sanitized_input_features: None,
        target_labels: Some(ReplayTargetLabels {
            stance_target: MemberStance::BuyProposal,
            confidence_calibration_target: MemberLearningLabel::Reinforce,
            risk_label: "risk_passed".to_string(),
            evidence_label: "evidence_sufficient".to_string(),
            outcome_label: MemberExperienceOutcome::PaperPositive,
            attribution_label: MemberScoreUpdateReason::GoodCall,
            label_source: ReplayLabelSource::PaperBacktestDeferred,
            label_confidence: ReplayLabelConfidence::ReviewRequired,
            paper_only: true,
        }),
        post_decision_context: None,
        sample_weight: 1.0,
        paper_only: true,
    };
    let dataset = ReplayDataset {
        dataset_id: "sprint155-backtest-source-dataset".to_string(),
        examples: vec![example.clone()],
        member_count: 1,
        example_count: 1,
        generated_from_store_id: "sprint155-backtest-source-store".to_string(),
        paper_only: true,
    };
    let evidence = sample_label_evidence(
        Some(example.replay_id.clone()),
        example.target.outcome_label,
        ReplayLabelSource::PaperBacktestDeferred,
        Some(0.03),
    );

    let build = build_validated_replay_dataset(
        &dataset,
        &[evidence],
        soma_zero::league::minimal_ai_committee_core::ValidatedReplayDatasetBuildConfig {
            source_sanitized_dataset_id: dataset.dataset_id.clone(),
            label_validation_policy: PaperLabelValidationPolicy::default(),
            backtest_label_contract: Some(ready_backtest_contract()),
            require_validated_labels_for_training: true,
            allow_ready_with_warnings_for_medium_confidence: true,
            reject_rejected_labels: true,
            paper_only: true,
        },
    );

    assert_eq!(build.promoted_count, 0);
    assert_eq!(build.needs_review_count, 1);
    assert_eq!(
        build.promotion_gates[0].promotion_status,
        LabelPromotionStatus::NeedsReview
    );
    assert!(build.promotion_gates[0].reason.contains("engine"));
    assert_eq!(
        build.dataset.examples[0]
            .target_labels
            .as_ref()
            .map(|labels| labels.label_source),
        Some(ReplayLabelSource::ReviewRequired)
    );
}

#[test]
fn label_quality_summary_and_readiness_block_weak_or_rejected_labels() {
    let base_config = std::fs::read_to_string("examples/soma_minimal_ai_committee_core.toml")
        .expect("read base config");
    let config = base_config
        .lines()
        .filter(|line| {
            !line.starts_with("label_validation_with_evidence_enabled")
                && !line.starts_with("paper_outcome_evidence_path")
                && !line.starts_with("paper_outcome_evidence_quality_output_path")
                && !line.starts_with("validated_replay_with_evidence_output_path")
                && !line.starts_with("evidence_backfill_")
                && !line.starts_with("validated_ratio_")
                && !line.starts_with("paper_price_series_path")
                && !line.starts_with("generated_paper_evidence_output_path")
                && !line.starts_with("enriched_evidence_")
                && !line.starts_with("enriched_staging_output_path")
                && !line.starts_with("enriched_approved_evidence_output_path")
                && !line.starts_with("enriched_training_candidate_output_path")
        })
        .collect::<Vec<_>>()
        .join("\n")
        .replace(
            "target/minimal_member_experience_store.json",
            "target/sprint155_label_quality_member_experience_store.json",
        )
        .replace(
            "target/minimal_member_replay_dataset.json",
            "target/sprint155_label_quality_replay_dataset.json",
        )
        .replace(
            "target/minimal_replay_quality_eval.json",
            "target/sprint155_label_quality_replay_quality_eval.json",
        )
        .replace(
            "target/minimal_sanitized_replay_dataset.json",
            "target/sprint155_label_quality_sanitized_replay_dataset.json",
        )
        .replace(
            "target/minimal_paper_scenario_collection_queue.json",
            "target/sprint155_label_quality_collection_queue.json",
        )
        .replace(
            "target/minimal_validated_replay_dataset.json",
            "target/sprint155_label_quality_validated_replay_dataset.json",
        )
        .replace(
            "target/minimal_label_quality_summary.json",
            "target/sprint155_label_quality_summary.json",
        )
        .replace(
            "target/minimal_owner_daily_brief_store.json",
            "target/sprint155_label_quality_owner_daily_brief_store.json",
        )
        .replace(
            "target/minimal_committee_state_snapshot.json",
            "target/sprint155_label_quality_committee_state_snapshot.json",
        )
        .replace(
            "target/minimal_committee_state",
            "target/sprint155_label_quality_committee_state",
        )
        .replace(
            "min_validated_label_ratio_required = 0.5",
            "min_validated_label_ratio_required = 0.75",
        );
    let config_path = std::path::Path::new("target/sprint155_label_quality_config.toml");
    std::fs::create_dir_all("target").expect("target dir");
    std::fs::write(config_path, config).expect("write sprint155 label quality config");

    let stateful =
        run_batch_committee_cycle_with_state_from_config_path(config_path).expect("stateful run");
    let configured_gate = stateful
        .batch_result
        .replay_quality_eval
        .as_ref()
        .expect("configured replay quality eval")
        .offline_training_readiness_gate
        .clone();
    assert_eq!(configured_gate.min_validated_label_ratio_required, 0.75);
    assert!(
        configured_gate
            .blockers
            .contains(&"validated label ratio below 0.75".to_string())
    );
    let store = MemberExperienceStore::new(
        "label-quality-readiness-store",
        stateful.batch_result.member_experience_records.clone(),
    );
    let weak_gate = evaluate_offline_training_readiness_with_thresholds(
        &stateful.batch_result.replay_dataset,
        &store,
        1,
        1,
    );
    assert_eq!(weak_gate.validated_label_count, 0);
    assert_eq!(
        weak_gate.readiness_status,
        OfflineTrainingReadinessStatus::NeedsMoreData
    );
    assert!(!weak_gate.ready_for_offline_training);

    let build = stateful
        .batch_result
        .validated_replay_build
        .as_ref()
        .expect("validated build from config");
    let summary = summarize_label_quality(build);
    assert_eq!(summary.label_quality_status, LabelQualityStatus::Weak);
    assert!(
        summary
            .recommended_next_label_actions
            .contains(&RecommendedLabelAction::AddPaperOutcomeEvidence)
    );
    assert!(stateful.batch_result.safety_summary.no_model_training);
    assert!(stateful.batch_result.safety_summary.no_live_inference);
    assert!(stateful.batch_result.safety_summary.no_broker_order_account);
}

#[test]
fn replay_quality_eval_defaults_to_sanitized_dataset_when_enabled() {
    let base_config = std::fs::read_to_string("examples/soma_minimal_ai_committee_core.toml")
        .expect("read base config");
    let config_without_explicit_sanitization = base_config
        .lines()
        .filter(|line| {
            !line.starts_with("replay_sanitization_enabled")
                && !line.starts_with("sanitized_replay_dataset_output_path")
        })
        .collect::<Vec<_>>()
        .join("\n")
        .replace(
            "target/minimal_member_experience_store.json",
            "target/unit_default_sanitized_member_experience_store.json",
        )
        .replace(
            "target/minimal_member_replay_dataset.json",
            "target/unit_default_sanitized_replay_dataset.json",
        )
        .replace(
            "target/minimal_replay_quality_eval.json",
            "target/unit_default_sanitized_quality_eval.json",
        )
        .replace(
            "target/minimal_committee_state",
            "target/unit_default_sanitized_committee_state",
        );
    let config_path = std::path::Path::new("target/unit_default_sanitized_config.toml");
    std::fs::create_dir_all("target").expect("target dir");
    std::fs::write(config_path, config_without_explicit_sanitization).expect("write config");

    let stateful =
        run_batch_committee_cycle_with_state_from_config_path(config_path).expect("stateful run");
    let sanitized_build = stateful
        .batch_result
        .sanitized_replay_build
        .as_ref()
        .expect("sanitized build should default on for quality eval");
    let leakage = check_replay_data_leakage(&stateful.batch_result.replay_dataset);
    let quality_eval = stateful
        .batch_result
        .replay_quality_eval
        .as_ref()
        .expect("quality eval");

    assert!(sanitized_build.sanitized_count > 0);
    assert_eq!(
        leakage.leakage_status,
        ReplayLeakageStatus::NoLeakageDetected
    );
    assert_eq!(
        quality_eval.leakage_check.leakage_status,
        ReplayLeakageStatus::NoLeakageDetected
    );
}

#[test]
fn member_state_store_accumulates_batch_memory_and_persists_locally() {
    let roster = create_three_member_pilot_roster(MarketScope::KoreaShortTerm);
    let mut store = MemberStateStore::from_members("test-member-state-store", &roster, "unit-test");
    let initial_store = store.clone();
    assert!(store.paper_only);
    assert_eq!(store.members.len(), 3);
    assert!(store.get_member_state("trend-kr-short").is_some());

    let err = MemberStateStore::load_from_local_json(std::path::Path::new(
        "https://example.invalid/member-state.json",
    ))
    .expect_err("remote state input path must fail");
    assert!(err.contains("must be local"));

    let batch_input = MinimalAiCommitteeCycleConfig::from_toml_path(std::path::Path::new(
        "examples/soma_minimal_ai_committee_core.toml",
    ))
    .expect("config")
    .load_batch_input()
    .expect("batch input");
    let result = run_batch_committee_cycle_with_state(BatchCommitteeCycleWithStateInput {
        batch_input,
        member_state_store: Some(store.clone()),
        member_state_output_path: None,
        emit_owner_summary: true,
        emit_owner_console_view: true,
        owner_feedback: Vec::new(),
        emit_reconsideration_view: false,
    })
    .expect("stateful batch cycle");

    store.apply_score_updates(&result.batch_result.score_updates);
    store.apply_memory_updates(&result.batch_result.memory_updates);
    store.append_learning_journal_entries(&result.batch_result.learning_journal_entries);
    let risk_state = store
        .get_member_state("risk-kr-short")
        .expect("risk member state");
    assert!(
        result
            .batch_result
            .score_updates
            .iter()
            .any(|update| update.new_voice_weight != update.previous_voice_weight)
    );
    for state in &store.members {
        let initial_state = initial_store
            .get_member_state(&state.member_id)
            .expect("initial member state");
        let (expected_score, expected_voice_weight) = result
            .batch_result
            .score_updates
            .iter()
            .filter(|update| update.member_id == state.member_id)
            .fold(
                (initial_state.score, initial_state.voice_weight),
                |(score, voice_weight), update| {
                    (
                        (score + update.new_score - update.previous_score).clamp(0.0, 1.0),
                        (voice_weight + update.new_voice_weight - update.previous_voice_weight)
                            .clamp(0.0, 1.0),
                    )
                },
            );
        assert!((state.score - expected_score).abs() < 1e-12);
        assert!((state.voice_weight - expected_voice_weight).abs() < 1e-12);
    }
    assert_eq!(result.state_update.updated_member_states, store.members);
    assert!(risk_state.memory_state.recent_opinion_count > 0);
    assert!(risk_state.memory_state.recent_event_count > 0);
    assert!(
        risk_state.score
            > initial_store
                .get_member_state("risk-kr-short")
                .expect("initial risk state")
                .score
    );
    assert!(
        risk_state.learning_journal_summary.reinforce_count
            + risk_state.learning_journal_summary.penalize_count
            + risk_state.learning_journal_summary.watch_count
            + risk_state.learning_journal_summary.ignore_count
            > 0
    );

    let path = std::path::Path::new("target/sprint132_member_state_store_test.json");
    let _ = std::fs::remove_file(path);
    store
        .save_to_local_json(path)
        .expect("save local member state");
    let loaded = MemberStateStore::load_from_local_json(path).expect("reload local member state");
    assert_eq!(loaded, store);
    let _ = std::fs::remove_file(path);

    let unsafe_path = std::path::Path::new("target/sprint132_unsafe_member_state_store_test.json");
    let unsafe_state = serde_json::json!({
        "store_id": "unsafe-member-state-store",
        "members": [],
        "source_label": "unit-test",
        "paper_only": true,
        "broker_account": "not allowed"
    })
    .to_string();
    std::fs::write(unsafe_path, unsafe_state).expect("write unsafe state fixture");
    let err = MemberStateStore::load_from_local_json(unsafe_path)
        .expect_err("unsafe broker/account state field must fail");
    assert!(err.contains("unsafe field"));
    let _ = std::fs::remove_file(unsafe_path);

    let err = store
        .save_to_local_json(std::path::Path::new(
            "https://example.invalid/member-state.json",
        ))
        .expect_err("remote state output path must fail");
    assert!(err.contains("must be local"));
}

#[test]
fn batch_cycle_with_state_produces_owner_summary_and_is_deterministic() {
    let first = run_batch_committee_cycle_with_state_from_config_path(std::path::Path::new(
        "examples/soma_minimal_ai_committee_core.toml",
    ))
    .expect("first stateful batch cycle");
    let second = run_batch_committee_cycle_with_state_from_config_path(std::path::Path::new(
        "examples/soma_minimal_ai_committee_core.toml",
    ))
    .expect("second stateful batch cycle");
    assert_eq!(first, second);

    assert_eq!(
        first.state_update.cycle_id,
        "offline-batch-sprint130-sample"
    );
    assert_eq!(
        first.state_update.score_updates.len(),
        first.batch_result.score_update_count
    );
    assert!(!first.state_update.memory_updates.is_empty());
    assert_eq!(first.state_update.updated_member_states.len(), 3);
    let summary = first.owner_summary.as_ref().expect("owner summary");
    assert!(summary.symbols_reviewed.contains(&"005930.KS".to_string()));
    assert!(summary.symbols_reviewed.contains(&"BTCUSDT".to_string()));
    assert_eq!(
        summary.event_count,
        first.batch_result.event_queue.event_count
    );
    assert_eq!(summary.risk_veto_count, first.batch_result.risk_veto_count);
    assert!(!summary.member_voice_changes.is_empty());
    assert!(!summary.chairman_actions.is_empty());
    assert!(
        summary
            .risk_warnings
            .iter()
            .any(|warning| warning.contains("high_volatility"))
    );
    assert!(summary.owner_readable_summary.contains("검토 종목"));
    assert!(summary.paper_only_warning.contains("not an order"));
    let console = first
        .owner_console_view
        .as_ref()
        .expect("owner console view");
    assert_eq!(console.cycle_id, first.batch_result.batch_id);
    assert_eq!(console.member_status_rows.len(), 3);
    assert_eq!(console.active_members.len(), 3);
    assert!(!console.event_rows.is_empty());
    assert_eq!(
        console.event_rows.len(),
        first.batch_result.event_queue.event_count
    );
    assert_eq!(
        console.committee_rows.len(),
        first.batch_result.committee_sessions.len()
    );
    assert_eq!(
        console.chairman_decision_rows.len(),
        first.batch_result.chairman_decisions.len()
    );
    assert!(!console.risk_veto_rows.is_empty());
    assert!(!console.voice_change_rows.is_empty());
    assert!(
        console
            .next_action_rows
            .iter()
            .any(|row| row.action_type == NextActionType::RiskBlocked)
    );
    assert!(
        console
            .next_action_rows
            .iter()
            .any(|row| row.action_type == NextActionType::NeedMoreEvidence)
    );
    assert!(
        console
            .next_action_rows
            .iter()
            .any(|row| row.action_type == NextActionType::Watch)
    );
    assert!(console.paper_only_warning.contains("paper-only"));
    let console_json = serde_json::to_string(console).expect("console json");
    for forbidden_field in ["\"broker\"", "\"order\"", "\"account\""] {
        assert!(!console_json.contains(forbidden_field));
    }
    assert!(first.batch_result.safety_summary.no_broker_order_account);
    assert!(first.batch_result.safety_summary.no_model_training);
    assert!(first.batch_result.safety_summary.no_live_inference);
}

#[test]
fn owner_feedback_routes_reconsideration_and_preserves_paper_only_safety() {
    let loaded_feedback = load_owner_feedback_from_local_json(std::path::Path::new(
        "examples/minimal_owner_feedback.sample.json",
    ))
    .expect("load owner feedback sample");
    assert_eq!(loaded_feedback.len(), 7);
    assert!(loaded_feedback.iter().all(|feedback| feedback.paper_only));

    let err = load_owner_feedback_from_local_json(std::path::Path::new(
        "https://example.invalid/owner-feedback.json",
    ))
    .expect_err("remote owner feedback must fail");
    assert!(err.contains("must be local"));

    for (case, text) in [
        ("trade", "execute trade with real money"),
        ("broker", "send this to broker account"),
        ("claim", "use private data for guaranteed return"),
    ] {
        let unsafe_path = std::path::PathBuf::from(format!(
            "target/sprint134_unsafe_owner_feedback_{case}.json"
        ));
        let unsafe_feedback = serde_json::json!([
            {
                "feedback_id": format!("unsafe-owner-feedback-{case}"),
                "symbol": "BTCUSDT",
                "market_scope": "CryptoShortTerm",
                "target_member_id": null,
                "feedback_type": "RiskConcern",
                "text": text,
                "priority": "High",
                "created_at": "2026-05-22T18:04:00Z",
                "paper_only": true
            }
        ])
        .to_string();
        std::fs::write(&unsafe_path, unsafe_feedback).expect("write unsafe owner feedback");
        let err = load_owner_feedback_from_local_json(&unsafe_path)
            .expect_err("unsafe owner feedback text must fail");
        assert!(err.contains("unsafe"));
        let _ = std::fs::remove_file(&unsafe_path);
    }

    let first = run_batch_committee_cycle_with_state_from_config_path(std::path::Path::new(
        "examples/soma_minimal_ai_committee_core.toml",
    ))
    .expect("first feedback reconsideration run");
    let second = run_batch_committee_cycle_with_state_from_config_path(std::path::Path::new(
        "examples/soma_minimal_ai_committee_core.toml",
    ))
    .expect("second feedback reconsideration run");
    assert_eq!(first, second);

    let reconsideration = first
        .owner_feedback_reconsideration
        .as_ref()
        .expect("owner feedback reconsideration");
    assert_eq!(reconsideration.owner_feedback_count, loaded_feedback.len());
    assert!(
        reconsideration
            .paper_only_warning
            .contains("no broker/order/account")
    );
    assert!(
        reconsideration
            .owner_feedback_journal_entries
            .iter()
            .any(|entry| {
                entry.feedback_id == "owner-feedback-comment-samsung"
                    && !entry.reconsideration_opened
                    && entry.outcome == OwnerFeedbackOutcome::LoggedOnly
            })
    );
    assert!(
        reconsideration
            .owner_feedback_journal_entries
            .iter()
            .any(|entry| {
                entry.feedback_id == "owner-feedback-outcome-samsung"
                    && !entry.reconsideration_opened
                    && entry.outcome == OwnerFeedbackOutcome::LoggedOnly
                    && entry.note.contains("paper outcome label")
            })
    );
    assert!(
        reconsideration
            .owner_feedback_journal_entries
            .iter()
            .any(|entry| {
                entry.feedback_id == "owner-feedback-disagree-samsung-trend"
                    && entry.routed_to_members == vec!["trend-kr-short".to_string()]
                    && entry.reconsideration_opened
            })
    );
    assert!(
        reconsideration
            .reconsideration_sessions
            .iter()
            .any(|session| {
                session.owner_feedback.feedback_type == OwnerFeedbackType::RiskConcern
                    && session
                        .risk_flags
                        .contains(&"owner_risk_concern".to_string())
            })
    );
    assert!(
        reconsideration
            .reconsideration_sessions
            .iter()
            .any(|session| {
                session.owner_feedback.feedback_type == OwnerFeedbackType::ReconsiderationRequest
                    && !session.invited_members.is_empty()
            })
    );
    assert!(
        reconsideration
            .routed_feedback_packets
            .iter()
            .any(|packet| {
                packet.feedback.feedback_type == OwnerFeedbackType::EvidenceRequest
                    && packet
                        .related_previous_opinions
                        .iter()
                        .any(|opinion| opinion.member_id.contains("evidence"))
            })
    );
    assert!(
        reconsideration
            .revised_member_opinions
            .iter()
            .any(|opinion| {
                opinion.member_id == "trend-kr-short"
                    && opinion.previous_stance == MemberStance::BuyProposal
                    && opinion.revised_stance == MemberStance::Hold
                    && opinion.changed
            })
    );
    assert!(
        reconsideration
            .revised_member_opinions
            .iter()
            .any(|opinion| {
                opinion.member_id.contains("evidence")
                    && opinion.revised_stance == MemberStance::NeedMoreEvidence
                    && !opinion.evidence_needed.is_empty()
            })
    );
    assert!(
        reconsideration
            .revised_member_opinions
            .iter()
            .any(|opinion| {
                opinion.member_id.contains("risk")
                    && opinion.revised_stance == MemberStance::NoTrade
                    && !opinion.risk_notes.is_empty()
            })
    );
    assert!(
        reconsideration
            .chairman_reconsideration_decisions
            .iter()
            .any(|decision| decision.risk_governor_status == RiskGovernorStatus::Vetoed)
    );
    assert!(
        reconsideration
            .chairman_reconsideration_decisions
            .iter()
            .any(|decision| {
                decision.final_action
                    == soma_zero::league::minimal_ai_committee_core::ChairmanReconsiderationFinalAction::NeedMoreEvidence
            })
    );
    assert!(
        reconsideration
            .chairman_reconsideration_decisions
            .iter()
            .any(|decision| {
                decision.final_action
                    == soma_zero::league::minimal_ai_committee_core::ChairmanReconsiderationFinalAction::PaperHold
            })
    );
    assert!(
        reconsideration
            .updated_owner_console_view
            .next_action_rows
            .iter()
            .any(|row| row.action_type == NextActionType::RiskBlocked)
    );

    let json = serde_json::to_string(reconsideration).expect("reconsideration json");
    for forbidden_field in ["\"broker\"", "\"account\""] {
        assert!(!json.contains(forbidden_field));
    }
    assert!(first.batch_result.safety_summary.no_broker_order_account);
    assert!(first.batch_result.safety_summary.no_model_training);
    assert!(first.batch_result.safety_summary.no_live_inference);
}

#[test]
fn autonomous_paper_loop_runs_cycles_attention_queue_and_archive_safely() {
    let handle = std::thread::Builder::new()
        .name("autonomous-paper-loop-large-stack".to_string())
        .stack_size(8 * 1024 * 1024)
        .spawn(autonomous_paper_loop_runs_cycles_attention_queue_and_archive_safely_inner)
        .expect("spawn large-stack autonomous loop test");
    handle
        .join()
        .expect("large-stack autonomous loop test panicked");
}

fn autonomous_paper_loop_runs_cycles_attention_queue_and_archive_safely_inner() {
    let sample_watchlist = WatchlistCandidateStore::load_from_local_json(std::path::Path::new(
        "examples/minimal_watchlist_candidates.sample.json",
    ))
    .expect("load watchlist sample");
    assert!(sample_watchlist.active_count >= 1);
    assert!(
        sample_watchlist
            .candidates
            .iter()
            .all(|candidate| candidate.paper_only)
    );

    let first = run_autonomous_paper_committee_loop_from_config_path(std::path::Path::new(
        "examples/soma_minimal_ai_committee_core.toml",
    ))
    .expect("first autonomous run");
    let second = run_autonomous_paper_committee_loop_from_config_path(std::path::Path::new(
        "examples/soma_minimal_ai_committee_core.toml",
    ))
    .expect("second autonomous run");
    assert_eq!(first, second);

    assert_eq!(first.run_id, "soma-autonomous-paper-sprint135");
    assert_eq!(first.cycle_count, 1);
    assert_eq!(first.cycles.len(), 1);
    assert!(!first.final_member_states.is_empty());
    assert!(!first.cycles[0].owner_summary.symbols_reviewed.is_empty());
    assert!(first.cycles[0].owner_console_view.is_some());
    assert!(
        first
            .attention_queue
            .items
            .iter()
            .any(|item| item.attention_type == OwnerAttentionType::RiskVeto)
    );
    assert!(
        first
            .attention_queue
            .items
            .iter()
            .any(|item| item.attention_type == OwnerAttentionType::NeedMoreEvidence)
    );
    let mut paper_hold_result = first.cycles[0].batch_result.clone();
    let paper_hold_decision = paper_hold_result
        .chairman_decisions
        .first_mut()
        .expect("paper hold fixture decision");
    paper_hold_decision.final_action = ChairmanFinalAction::PaperHold;
    paper_hold_decision.risk_governor_status = RiskGovernorStatus::Passed;
    let paper_hold_queue = OwnerAttentionQueue::from_batch_cycle_result(
        "paper-hold-watchlist-test",
        0,
        &paper_hold_result,
        None,
        OwnerConfirmationPolicy::Never,
    );
    assert!(
        paper_hold_queue
            .items
            .iter()
            .any(|item| item.attention_type == OwnerAttentionType::WatchlistCandidate)
    );
    assert_eq!(
        first
            .attention_queue
            .items
            .first()
            .map(|item| item.priority),
        Some(OwnerAttentionPriority::High)
    );
    assert_eq!(first.attention_queue.requires_owner_input_count, 0);
    assert!(first.attention_queue.unresolved_items().is_empty());
    assert!(!first.paper_decision_archive.entries.is_empty());
    assert!(first.paper_decision_archive.risk_veto_count() >= 1);
    assert!(
        first
            .paper_decision_archive
            .entries
            .iter()
            .all(|entry| entry.paper_only)
    );
    assert!(
        !first
            .paper_decision_archive
            .decisions_by_symbol("BTCUSDT")
            .is_empty()
    );
    assert!(first.safety_summary.no_broker_order_account);
    assert!(first.safety_summary.no_model_training);
    assert!(first.safety_summary.no_live_inference);
    let triage = first
        .owner_attention_triage
        .as_ref()
        .expect("owner attention triage");
    assert!(triage.inbox.open_count <= triage.inbox.items.len());
    assert!(!triage.action_results.is_empty());
    assert!(!triage.generated_owner_feedback.is_empty());
    assert!(!triage.generated_watchlist_candidates.is_empty());
    let recheck = first.watchlist_recheck.as_ref().expect("watchlist recheck");
    assert_eq!(
        recheck.recheck_id,
        "soma-autonomous-paper-sprint135-watchlist-recheck"
    );
    assert_eq!(recheck.selection.selected_count, 3);
    assert!(
        recheck
            .selection
            .skip_reasons
            .contains(&WatchlistRecheckSkipReason::Archived)
    );
    assert!(
        recheck
            .selection
            .skip_reasons
            .contains(&WatchlistRecheckSkipReason::OverCandidateLimit)
    );
    assert!(
        recheck
            .selection
            .skip_reasons
            .contains(&WatchlistRecheckSkipReason::RiskBlockedExcluded)
    );
    assert!(recheck.batch_result.member_opinions.iter().all(|opinion| {
        recheck.selected_candidates.iter().any(|candidate| {
            candidate.symbol == opinion.symbol && candidate.market_scope == opinion.market_scope
        })
    }));
    assert!(
        recheck
            .lifecycle_events
            .iter()
            .any(|event| event.new_status == WatchlistCandidateStatus::RiskBlocked)
    );
    assert!(
        recheck
            .lifecycle_events
            .iter()
            .any(|event| event.new_status == WatchlistCandidateStatus::NeedsEvidence)
    );
    assert!(
        recheck
            .lifecycle_events
            .iter()
            .all(|event| event.paper_only)
    );
    assert!(!recheck.generated_attention_items.is_empty());
    let brief = recheck
        .owner_daily_brief
        .as_ref()
        .expect("owner daily brief");
    assert!(!brief.reviewed_symbols.is_empty());
    assert!(!brief.risk_vetoes.is_empty());
    assert!(!brief.need_more_evidence_items.is_empty());
    assert!(!brief.next_owner_attention.is_empty());
    assert!(brief.brief_text.contains("paper-only"));
    assert!(recheck.safety_summary.no_broker_order_account);
    assert!(recheck.safety_summary.no_model_training);
    assert!(recheck.safety_summary.no_live_inference);

    let fixed_config_path = std::path::Path::new("target/sprint135_autonomous_fixed.toml");
    std::fs::write(
        fixed_config_path,
        r#"
input_path = "examples/minimal_ai_committee_multi_market_sample.json"
offline_member_output_batch_path = "examples/minimal_offline_member_output_batch.sample.json"
batch_mode = true
autonomous_paper_run = true
run_id = "sprint135-fixed-test"
max_cycles = 2
cycle_mode = "FixedCount"
require_owner_confirmation = "Never"
emit_owner_console_view = true
pilot_roster = "three_member"
paper_outcome = "Positive"
archetype_style_cards_path = "examples/investor_archetype_style_cards.sample.json"
style_mapping_mode = "LocalFixture"
"#,
    )
    .expect("write fixed autonomous config");
    let fixed = run_autonomous_paper_committee_loop_from_config_path(fixed_config_path)
        .expect("fixed autonomous run");
    assert_eq!(fixed.cycle_count, 2);
    assert!(
        fixed
            .attention_queue
            .items
            .iter()
            .all(|item| item.attention_type != OwnerAttentionType::OwnerFeedbackAvailable)
    );
    let first_risk_state = fixed.cycles[0]
        .state_update
        .updated_member_states
        .iter()
        .find(|state| state.member_id == "risk-kr-short")
        .expect("first risk state");
    let second_risk_state = fixed.cycles[1]
        .state_update
        .updated_member_states
        .iter()
        .find(|state| state.member_id == "risk-kr-short")
        .expect("second risk state");
    assert!(
        second_risk_state.memory_state.recent_opinion_count
            >= first_risk_state.memory_state.recent_opinion_count
    );
    assert_eq!(fixed.attention_queue.requires_owner_input_count, 0);
    let _ = std::fs::remove_file(fixed_config_path);
}

#[test]
fn watchlist_recheck_direct_cycle_loads_local_paths() {
    let config = MinimalAiCommitteeCycleConfig::from_toml_path(std::path::Path::new(
        "examples/soma_minimal_ai_committee_core.toml",
    ))
    .expect("config");
    let loaded_batch = config.load_batch_input().expect("batch input");
    std::fs::create_dir_all("target").expect("target dir");
    let market_data_path = std::path::Path::new("target/sprint137_watchlist_market_data.json");
    let news_path = std::path::Path::new("target/sprint137_watchlist_news.json");
    std::fs::write(
        market_data_path,
        serde_json::to_string_pretty(&loaded_batch.market_data).expect("market data json"),
    )
    .expect("write market data");
    std::fs::write(
        news_path,
        serde_json::to_string_pretty(&loaded_batch.news).expect("news json"),
    )
    .expect("write news");

    let mut seed_batch = loaded_batch;
    seed_batch.market_data.clear();
    seed_batch.news.clear();
    seed_batch.offline_output_batch = None;

    let result = run_watchlist_recheck_cycle(WatchlistRecheckConfig {
        recheck_id: "direct-path-watchlist-recheck".to_string(),
        watchlist_input_path: Some("examples/minimal_watchlist_candidates.sample.json".to_string()),
        watchlist_output_path: None,
        member_state_input_path: None,
        member_state_output_path: None,
        market_data_path: Some(market_data_path.to_string_lossy().to_string()),
        news_path: Some(news_path.to_string_lossy().to_string()),
        offline_member_output_batch_path: Some(
            "examples/minimal_offline_member_output_batch.sample.json".to_string(),
        ),
        max_candidates_per_cycle: 3,
        include_risk_blocked: false,
        include_needs_evidence: true,
        emit_owner_daily_brief: true,
        paper_only: true,
        watchlist_store: WatchlistCandidateStore::new("ignored-direct-path-store"),
        batch_input: seed_batch,
        member_state_store: None,
    })
    .expect("direct path watchlist recheck");

    assert_eq!(result.selection.selected_count, 3);
    assert!(
        result
            .selection
            .skip_reasons
            .contains(&WatchlistRecheckSkipReason::Archived)
    );
    assert!(result.batch_result.member_opinions.iter().any(|opinion| {
        opinion
            .evidence_notes
            .iter()
            .any(|note| note == "offline batch opinion")
    }));
    assert!(result.lifecycle_events.iter().any(|event| {
        event.new_status == WatchlistCandidateStatus::RiskBlocked
            || event.new_status == WatchlistCandidateStatus::NeedsEvidence
    }));
    let brief = result.owner_daily_brief.expect("owner daily brief");
    assert!(brief.reviewed_symbols.contains(&"BTCUSDT".to_string()));
    assert!(brief.brief_text.contains("paper-only"));

    let _ = std::fs::remove_file(market_data_path);
    let _ = std::fs::remove_file(news_path);
}

#[test]
fn watchlist_recheck_updates_paper_candidate_without_order_path() {
    let store = WatchlistCandidateStore {
        store_id: "paper-candidate-recheck-store".to_string(),
        candidates: vec![WatchlistCandidate {
            candidate_id: "watchlist-paper-samsung".to_string(),
            symbol: "005930.KS".to_string(),
            market_scope: MarketScope::KoreaShortTerm,
            source_attention_item_id: "attention-paper-candidate".to_string(),
            reason: "Paper-only candidate path test".to_string(),
            status: WatchlistCandidateStatus::Watching,
            created_at: Some("2026-05-23T09:40:00Z".to_string()),
            paper_only: true,
        }],
        active_count: 1,
        risk_blocked_count: 0,
        needs_evidence_count: 0,
        paper_only: true,
    };
    let mut batch_input = MinimalAiCommitteeCycleConfig::from_toml_path(std::path::Path::new(
        "examples/soma_minimal_ai_committee_core.toml",
    ))
    .expect("config")
    .load_batch_input()
    .expect("batch input");
    batch_input
        .market_data
        .retain(|market| market.symbol == "005930.KS");
    batch_input.news.retain(|news| news.symbol == "005930.KS");
    batch_input.offline_output_batch = Some(OfflineMemberOutputBatch {
        batch_id: "paper-candidate-offline-batch".to_string(),
        created_at: "2026-05-23T09:41:00Z".to_string(),
        source_label: "unit-test".to_string(),
        opinions: vec![
            OfflineMemberOpinionFixture {
                member_id: "trend-kr-short".to_string(),
                symbol: "005930.KS".to_string(),
                market_scope: MarketScope::KoreaShortTerm,
                stance: MemberStance::BuyProposal,
                confidence: 0.91,
                expected_return_hint: 0.03,
                risk_hint: 0.02,
                evidence_notes: vec!["paper-only buy candidate".to_string()],
                event_triggered: true,
                event_reason: Some("paper-only candidate".to_string()),
            },
            OfflineMemberOpinionFixture {
                member_id: "risk-kr-short".to_string(),
                symbol: "005930.KS".to_string(),
                market_scope: MarketScope::KoreaShortTerm,
                stance: MemberStance::Hold,
                confidence: 0.2,
                expected_return_hint: 0.0,
                risk_hint: 0.01,
                evidence_notes: vec!["paper-only risk passed".to_string()],
                event_triggered: false,
                event_reason: None,
            },
            OfflineMemberOpinionFixture {
                member_id: "evidence-kr-short".to_string(),
                symbol: "005930.KS".to_string(),
                market_scope: MarketScope::KoreaShortTerm,
                stance: MemberStance::Hold,
                confidence: 0.2,
                expected_return_hint: 0.0,
                risk_hint: 0.01,
                evidence_notes: vec!["paper-only evidence enough".to_string()],
                event_triggered: false,
                event_reason: None,
            },
        ],
    });
    let result = run_watchlist_recheck_cycle(WatchlistRecheckConfig {
        recheck_id: "paper-candidate-recheck".to_string(),
        watchlist_input_path: None,
        watchlist_output_path: None,
        member_state_input_path: None,
        member_state_output_path: None,
        market_data_path: None,
        news_path: None,
        offline_member_output_batch_path: None,
        max_candidates_per_cycle: 1,
        include_risk_blocked: false,
        include_needs_evidence: true,
        emit_owner_daily_brief: true,
        paper_only: true,
        watchlist_store: store,
        batch_input,
        member_state_store: None,
    })
    .expect("paper candidate recheck");
    assert!(
        result
            .lifecycle_events
            .iter()
            .any(|event| event.new_status == WatchlistCandidateStatus::PaperCandidate)
    );
    assert!(
        result
            .updated_watchlist_store
            .candidates
            .iter()
            .all(|candidate| candidate.paper_only)
    );
    let json = serde_json::to_string(&result).expect("watchlist recheck json");
    for forbidden_field in ["\"broker\"", "\"order\"", "\"account\""] {
        assert!(!json.contains(forbidden_field));
    }
}

#[test]
fn owner_attention_inbox_triages_actions_and_watchlist_safely() {
    let handle = std::thread::Builder::new()
        .name("owner-attention-inbox-large-stack".to_string())
        .stack_size(8 * 1024 * 1024)
        .spawn(owner_attention_inbox_triages_actions_and_watchlist_safely_inner)
        .expect("spawn large-stack owner attention inbox test");
    handle
        .join()
        .expect("large-stack owner attention inbox test panicked");
}

fn owner_attention_inbox_triages_actions_and_watchlist_safely_inner() {
    let run = run_autonomous_paper_committee_loop_from_config_path(std::path::Path::new(
        "examples/soma_minimal_ai_committee_core.toml",
    ))
    .expect("autonomous run with attention inbox");
    let mut inbox = OwnerAttentionInbox::from_attention_queue(&run.attention_queue);
    let original_count = inbox.items.len();
    inbox.merge_new_items(&run.attention_queue);
    assert_eq!(inbox.items.len(), original_count);
    assert_eq!(
        inbox
            .high_priority_items()
            .first()
            .map(|item| item.priority),
        Some(OwnerAttentionPriority::High)
    );
    assert!(!inbox.open_items().is_empty());
    assert_eq!(inbox.requires_owner_input_count, 0);
    assert!(inbox.items_requiring_owner_input().is_empty());

    let watch_item = inbox
        .items
        .iter()
        .find(|item| item.symbol.is_some() && item.market_scope.is_some())
        .expect("symbol scoped attention item")
        .clone();
    let actions = vec![
        OwnerAttentionAction {
            action_id: "triage-ack".to_string(),
            item_id: watch_item.item_id.clone(),
            action_type: OwnerAttentionActionType::Acknowledge,
            comment: Some("Paper-only acknowledge".to_string()),
            created_at: Some("2026-05-23T04:30:00Z".to_string()),
            paper_only: true,
        },
        OwnerAttentionAction {
            action_id: "triage-defer".to_string(),
            item_id: watch_item.item_id.clone(),
            action_type: OwnerAttentionActionType::Defer,
            comment: Some("Paper-only defer".to_string()),
            created_at: Some("2026-05-23T04:31:00Z".to_string()),
            paper_only: true,
        },
        OwnerAttentionAction {
            action_id: "triage-dismiss".to_string(),
            item_id: watch_item.item_id.clone(),
            action_type: OwnerAttentionActionType::Dismiss,
            comment: Some("Paper-only dismiss".to_string()),
            created_at: Some("2026-05-23T04:32:00Z".to_string()),
            paper_only: true,
        },
        OwnerAttentionAction {
            action_id: "triage-watch".to_string(),
            item_id: watch_item.item_id.clone(),
            action_type: OwnerAttentionActionType::ConvertToWatchlist,
            comment: Some("Paper-only watchlist candidate".to_string()),
            created_at: Some("2026-05-23T04:33:00Z".to_string()),
            paper_only: true,
        },
        OwnerAttentionAction {
            action_id: "triage-evidence".to_string(),
            item_id: watch_item.item_id.clone(),
            action_type: OwnerAttentionActionType::RequestMoreEvidence,
            comment: Some("Paper-only request more evidence".to_string()),
            created_at: Some("2026-05-23T04:34:00Z".to_string()),
            paper_only: true,
        },
        OwnerAttentionAction {
            action_id: "triage-reconsider".to_string(),
            item_id: watch_item.item_id.clone(),
            action_type: OwnerAttentionActionType::RequestReconsideration,
            comment: Some("Paper-only committee reconsideration".to_string()),
            created_at: Some("2026-05-23T04:35:00Z".to_string()),
            paper_only: true,
        },
        OwnerAttentionAction {
            action_id: "triage-comment".to_string(),
            item_id: watch_item.item_id.clone(),
            action_type: OwnerAttentionActionType::AddComment,
            comment: Some("Paper-only owner comment".to_string()),
            created_at: Some("2026-05-23T04:36:00Z".to_string()),
            paper_only: true,
        },
        OwnerAttentionAction {
            action_id: "triage-unsafe".to_string(),
            item_id: watch_item.item_id.clone(),
            action_type: OwnerAttentionActionType::AddComment,
            comment: Some("execute order with broker account".to_string()),
            created_at: Some("2026-05-23T04:37:00Z".to_string()),
            paper_only: true,
        },
    ];
    let first = run_owner_attention_triage(OwnerAttentionTriageInput {
        previous_run: run.clone(),
        previous_inbox: Some(inbox.clone()),
        owner_actions: actions.clone(),
        watchlist_store: Some(WatchlistCandidateStore::new("triage-test-watchlist")),
    })
    .expect("first triage");
    let second = run_owner_attention_triage(OwnerAttentionTriageInput {
        previous_run: run,
        previous_inbox: Some(inbox),
        owner_actions: actions,
        watchlist_store: Some(WatchlistCandidateStore::new("triage-test-watchlist")),
    })
    .expect("second triage");
    assert_eq!(first, second);

    assert!(first.action_results.iter().any(|result| {
        result.action_id == "triage-ack"
            && result.new_status == OwnerAttentionInboxStatus::Acknowledged
            && result.safety_status == OwnerAttentionActionSafetyStatus::Passed
    }));
    assert!(first.action_results.iter().any(|result| {
        result.action_id == "triage-defer"
            && result.new_status == OwnerAttentionInboxStatus::Deferred
    }));
    assert!(first.action_results.iter().any(|result| {
        result.action_id == "triage-dismiss"
            && result.new_status == OwnerAttentionInboxStatus::Dismissed
    }));
    assert!(first.action_results.iter().any(|result| {
        result.action_id == "triage-unsafe"
            && result.safety_status == OwnerAttentionActionSafetyStatus::Rejected
    }));
    assert!(
        first
            .generated_watchlist_candidates
            .iter()
            .all(|candidate| {
                candidate.paper_only
                    && matches!(
                        candidate.status,
                        WatchlistCandidateStatus::Watching
                            | WatchlistCandidateStatus::NeedsEvidence
                            | WatchlistCandidateStatus::RiskBlocked
                            | WatchlistCandidateStatus::PaperCandidate
                    )
            })
    );
    assert!(first.generated_owner_feedback.iter().any(|feedback| {
        feedback.feedback_type == OwnerFeedbackType::EvidenceRequest && feedback.paper_only
    }));
    assert!(first.generated_owner_feedback.iter().any(|feedback| {
        feedback.feedback_type == OwnerFeedbackType::ReconsiderationRequest && feedback.paper_only
    }));
    assert!(first.generated_owner_feedback.iter().any(|feedback| {
        feedback.feedback_type == OwnerFeedbackType::Comment && feedback.paper_only
    }));
    assert!(first.watchlist_store.active_count >= 1);
    assert!(
        !first
            .watchlist_store
            .candidates_by_symbol(&watch_item.symbol.expect("watch symbol"))
            .is_empty()
    );
    assert!(first.safety_summary.no_broker_order_account);
    assert!(first.safety_summary.no_model_training);
    assert!(first.safety_summary.no_live_inference);

    let inbox_path = std::path::Path::new("target/sprint136_owner_attention_inbox.json");
    let watchlist_path = std::path::Path::new("target/sprint136_watchlist_store.json");
    let _ = std::fs::remove_file(inbox_path);
    let _ = std::fs::remove_file(watchlist_path);
    first
        .inbox
        .save_to_local_json(inbox_path)
        .expect("save inbox");
    let loaded_inbox = OwnerAttentionInbox::load_from_local_json(inbox_path).expect("load inbox");
    assert_eq!(loaded_inbox, first.inbox);
    first
        .watchlist_store
        .save_to_local_json(watchlist_path)
        .expect("save watchlist");
    let loaded_watchlist =
        WatchlistCandidateStore::load_from_local_json(watchlist_path).expect("load watchlist");
    assert_eq!(loaded_watchlist, first.watchlist_store);
    let _ = std::fs::remove_file(inbox_path);
    let _ = std::fs::remove_file(watchlist_path);

    let err = OwnerAttentionInbox::load_from_local_json(std::path::Path::new(
        "https://example.invalid/inbox.json",
    ))
    .expect_err("remote inbox path must fail");
    assert!(err.contains("must be local"));
    let err = WatchlistCandidateStore::load_from_local_json(std::path::Path::new(
        "https://example.invalid/watchlist.json",
    ))
    .expect_err("remote watchlist path must fail");
    assert!(err.contains("must be local"));
    let sample_actions = load_owner_attention_actions_from_local_json(std::path::Path::new(
        "examples/minimal_owner_attention_actions.sample.json",
    ))
    .expect("load owner attention action sample");
    assert!(!sample_actions.is_empty());
}

#[test]
fn owner_daily_brief_store_appends_loads_and_rejects_remote_paths() {
    let handle = std::thread::Builder::new()
        .name("owner-daily-brief-store-large-stack".to_string())
        .stack_size(8 * 1024 * 1024)
        .spawn(owner_daily_brief_store_appends_loads_and_rejects_remote_paths_inner)
        .expect("spawn large-stack owner daily brief store test");
    if let Err(payload) = handle.join() {
        std::panic::resume_unwind(payload);
    }
}

fn owner_daily_brief_store_appends_loads_and_rejects_remote_paths_inner() {
    let run = run_autonomous_paper_committee_loop_from_config_path(std::path::Path::new(
        "examples/soma_minimal_ai_committee_core.toml",
    ))
    .expect("autonomous run with daily brief");
    let brief = run
        .watchlist_recheck
        .as_ref()
        .and_then(|recheck| recheck.owner_daily_brief.clone())
        .expect("owner daily brief");
    let mut store = OwnerDailyBriefStore::new("owner-daily-brief-store-test");
    store.append_brief(brief.clone());
    assert_eq!(
        store.latest().map(|item| &item.brief_id),
        Some(&brief.brief_id)
    );
    let mut earlier_sorting_brief = brief.clone();
    earlier_sorting_brief.brief_id = "brief-0000-appended-last".to_string();
    store.append_brief(earlier_sorting_brief.clone());
    assert_eq!(
        store.latest().map(|item| &item.brief_id),
        Some(&earlier_sorting_brief.brief_id)
    );
    assert!(
        !store
            .briefs_by_symbol(&brief.reviewed_symbols[0])
            .is_empty()
    );
    assert_eq!(
        store.risk_veto_count(),
        brief.risk_vetoes.len() + earlier_sorting_brief.risk_vetoes.len()
    );
    assert_eq!(
        store.need_more_evidence_count(),
        brief.need_more_evidence_items.len() + earlier_sorting_brief.need_more_evidence_items.len()
    );

    let path = std::path::Path::new("target/sprint141_owner_daily_brief_store.json");
    let _ = std::fs::remove_file(path);
    store.save_to_local_json(path).expect("save brief store");
    let loaded = OwnerDailyBriefStore::load_from_local_json(path).expect("load brief store");
    assert_eq!(loaded, store);
    let _ = std::fs::remove_file(path);

    let err = OwnerDailyBriefStore::load_from_local_json(std::path::Path::new(
        "https://example.invalid/brief-store.json",
    ))
    .expect_err("remote brief store path must fail");
    assert!(err.contains("must be local"));

    let mut unsafe_store = OwnerDailyBriefStore::new("unsafe-owner-daily-brief-store-test");
    let mut unsafe_brief = brief;
    unsafe_brief
        .next_owner_attention
        .push("execute order through broker account".to_string());
    unsafe_store.append_brief(unsafe_brief);
    let err = unsafe_store
        .save_to_local_json(path)
        .expect_err("unsafe brief store must fail");
    assert!(err.contains("unsafe instruction"));
}

#[test]
fn committee_state_snapshot_exports_ui_shape_safely_and_deterministically() {
    let handle = std::thread::Builder::new()
        .name("committee-state-snapshot-export-test".to_string())
        .stack_size(8 * 1024 * 1024)
        .spawn(committee_state_snapshot_exports_ui_shape_safely_and_deterministically_inner)
        .expect("spawn committee state snapshot test");
    if let Err(payload) = handle.join() {
        std::panic::resume_unwind(payload);
    }
}

fn committee_state_snapshot_exports_ui_shape_safely_and_deterministically_inner() {
    let run = run_autonomous_paper_committee_loop_from_config_path(std::path::Path::new(
        "examples/soma_minimal_ai_committee_core.toml",
    ))
    .expect("autonomous run with snapshot");
    let triage = run
        .owner_attention_triage
        .as_ref()
        .expect("owner attention triage");
    let recheck = run.watchlist_recheck.as_ref().expect("watchlist recheck");
    let brief = recheck
        .owner_daily_brief
        .clone()
        .expect("owner daily brief");
    let mut brief_store = OwnerDailyBriefStore::new("snapshot-brief-store");
    brief_store.append_brief(brief);
    let input = CommitteeStateExportInput {
        autonomous_run_result: Some(run.clone()),
        watchlist_recheck_result: run.watchlist_recheck.clone(),
        attention_inbox: Some(triage.inbox.clone()),
        watchlist_store: Some(recheck.updated_watchlist_store.clone()),
        member_state_store: Some(MemberStateStore {
            store_id: "snapshot-member-state-store".to_string(),
            members: run.final_member_states.clone(),
            source_label: "unit-test".to_string(),
            paper_only: true,
        }),
        owner_daily_brief_store: Some(brief_store),
        owner_summary: Some(recheck.owner_summary.clone()),
        owner_console_view: recheck.owner_console_view.clone(),
    };
    let first = build_committee_state_snapshot(input.clone()).expect("first snapshot");
    let second = build_committee_state_snapshot(input).expect("second snapshot");
    assert_eq!(
        serde_json::to_string_pretty(&first).expect("first json"),
        serde_json::to_string_pretty(&second).expect("second json")
    );
    assert!(first.paper_only);
    assert!(first.safety_summary.no_broker_order_account);
    assert!(first.safety_summary.no_model_training);
    assert!(first.safety_summary.no_live_inference);
    assert_eq!(first.members.len(), 3);
    assert!(!first.watchlist_candidates.is_empty());
    assert!(!first.attention_items.is_empty());
    assert!(!first.recent_chairman_decisions.is_empty());
    assert!(!first.recent_risk_vetoes.is_empty());
    assert!(!first.next_owner_actions.is_empty());
    assert!(
        first
            .next_owner_actions
            .iter()
            .any(|row| row.action_type == NextOwnerActionType::ReviewRiskVeto)
    );
    assert!(first.latest_owner_daily_brief.is_some());
    let json = serde_json::to_string(&first).expect("snapshot json");
    for forbidden_field in ["\"broker\"", "\"order\"", "\"account\""] {
        assert!(!json.contains(forbidden_field));
    }

    let path = std::path::Path::new("target/sprint141_committee_state_snapshot.json");
    let _ = std::fs::remove_file(path);
    save_committee_state_snapshot(path, &first).expect("save snapshot");
    assert!(path.exists());
    let _ = std::fs::remove_file(path);
    let err = save_committee_state_snapshot(
        std::path::Path::new("https://example.invalid/snapshot.json"),
        &first,
    )
    .expect_err("remote snapshot path must fail");
    assert!(err.contains("must be local"));
}

#[test]
fn committee_state_export_writes_state_contract_and_owner_read_model() {
    let handle = std::thread::Builder::new()
        .name("committee-state-export-large-stack".to_string())
        .stack_size(8 * 1024 * 1024)
        .spawn(committee_state_export_writes_state_contract_and_owner_read_model_inner)
        .expect("spawn large-stack committee state export test");
    if let Err(payload) = handle.join() {
        std::panic::resume_unwind(payload);
    }
}

fn committee_state_export_writes_state_contract_and_owner_read_model_inner() {
    let run = run_autonomous_paper_committee_loop_from_config_path(std::path::Path::new(
        "examples/soma_minimal_ai_committee_core.toml",
    ))
    .expect("autonomous run with snapshot");
    let snapshot = run
        .committee_state_snapshot
        .clone()
        .expect("committee state snapshot");
    let root = std::path::Path::new("target/sprint142_state_export_contract");
    let _ = std::fs::remove_dir_all(root);
    let result = write_committee_state_export(
        CommitteeStateExportConfig {
            export_id: "sprint142-state-export".to_string(),
            export_root_path: root.display().to_string(),
            write_latest_snapshot: true,
            write_history_snapshot: true,
            write_snapshot_index: true,
            write_owner_console_read_model: true,
            schema_version: "committee-state.v1".to_string(),
            max_history_entries: Some(1),
            paper_only: true,
        },
        &snapshot,
        CommitteeStateSnapshotSource::ManualExport,
    )
    .expect("state export writes");
    assert!(result.wrote_latest_snapshot);
    assert!(result.wrote_history_snapshot);
    assert!(result.wrote_snapshot_index);
    assert!(result.wrote_owner_console_read_model);
    assert!(root.join("latest.json").exists());
    assert!(root.join("snapshot_index.json").exists());
    assert!(root.join("owner_console_read_model.json").exists());
    assert!(
        root.join("history")
            .join(format!("{}.json", snapshot.snapshot_id))
            .exists()
    );
    let policy = SnapshotFileNamingPolicy::state_folder_contract();
    assert_eq!(policy.latest_path(root), root.join("latest.json"));
    assert_eq!(
        policy.snapshot_index_path(root),
        root.join("snapshot_index.json")
    );
    assert_eq!(
        policy
            .history_path(root, &snapshot.snapshot_id)
            .expect("history path"),
        root.join("history")
            .join(format!("{}.json", snapshot.snapshot_id))
    );
    assert_eq!(
        policy.owner_console_read_model_path(root),
        root.join("owner_console_read_model.json")
    );

    let deterministic_latest = std::fs::read_to_string(root.join("latest.json")).expect("latest");
    write_committee_state_export(
        CommitteeStateExportConfig {
            export_id: "sprint142-state-export-repeat".to_string(),
            export_root_path: root.display().to_string(),
            write_latest_snapshot: true,
            write_history_snapshot: true,
            write_snapshot_index: true,
            write_owner_console_read_model: true,
            schema_version: "committee-state.v1".to_string(),
            max_history_entries: Some(1),
            paper_only: true,
        },
        &snapshot,
        CommitteeStateSnapshotSource::ManualExport,
    )
    .expect("repeated state export writes");
    assert_eq!(
        deterministic_latest,
        std::fs::read_to_string(root.join("latest.json")).expect("repeated latest")
    );

    let latest: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(root.join("latest.json")).expect("latest json"),
    )
    .expect("latest envelope");
    assert_eq!(latest["schema_version"], "committee-state.v1");
    assert_eq!(latest["snapshot_id"], snapshot.snapshot_id);
    assert_eq!(latest["source"], "manual_export");
    assert_eq!(latest["safety"]["paper_only"], true);
    assert_eq!(latest["safety"]["no_live_trading"], true);

    let read_model = build_owner_console_read_model("committee-state.v1", &snapshot);
    assert_eq!(read_model.snapshot_id, snapshot.snapshot_id);
    assert!(!read_model.member_cards.is_empty());
    assert!(!read_model.watchlist_cards.is_empty());
    assert!(!read_model.attention_cards.is_empty());
    assert!(!read_model.decision_cards.is_empty());
    assert!(!read_model.next_action_cards.is_empty());
    assert!(
        read_model
            .safety_badges
            .iter()
            .any(|badge| badge == "read-only")
    );
    let read_model_json: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(root.join("owner_console_read_model.json"))
            .expect("read model json"),
    )
    .expect("owner console read model");
    assert_eq!(read_model_json["schema_version"], "committee-state.v1");
    assert!(
        read_model_json["header"]["reviewed_symbol_count"]
            .as_u64()
            .expect("reviewed symbol count")
            > 0
    );

    let mut next_snapshot = snapshot.clone();
    next_snapshot.snapshot_id = format!("{}-next", snapshot.snapshot_id);
    next_snapshot.generated_at = Some("zzzz-next-snapshot".to_string());
    write_committee_state_export(
        CommitteeStateExportConfig {
            export_id: "sprint142-state-export-second".to_string(),
            export_root_path: root.display().to_string(),
            write_latest_snapshot: false,
            write_history_snapshot: true,
            write_snapshot_index: true,
            write_owner_console_read_model: false,
            schema_version: "committee-state.v1".to_string(),
            max_history_entries: Some(1),
            paper_only: true,
        },
        &next_snapshot,
        CommitteeStateSnapshotSource::ManualExport,
    )
    .expect("second state export writes");
    let index: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(root.join("snapshot_index.json")).expect("index json"),
    )
    .expect("snapshot index");
    assert_eq!(index["latest_snapshot_id"], next_snapshot.snapshot_id);
    assert_eq!(index["entries"].as_array().expect("entries").len(), 1);
    assert_eq!(
        index["entries"][0]["path"],
        format!("history/{}.json", next_snapshot.snapshot_id)
    );
    let mut older_latest_snapshot = snapshot.clone();
    older_latest_snapshot.snapshot_id = format!("{}-older-latest", snapshot.snapshot_id);
    older_latest_snapshot.generated_at = Some("0000-older-latest".to_string());
    write_committee_state_export(
        CommitteeStateExportConfig {
            export_id: "sprint142-state-export-older-latest".to_string(),
            export_root_path: root.display().to_string(),
            write_latest_snapshot: false,
            write_history_snapshot: true,
            write_snapshot_index: true,
            write_owner_console_read_model: false,
            schema_version: "committee-state.v1".to_string(),
            max_history_entries: Some(1),
            paper_only: true,
        },
        &older_latest_snapshot,
        CommitteeStateSnapshotSource::ManualExport,
    )
    .expect("older latest state export writes");
    let capped_index: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(root.join("snapshot_index.json")).expect("capped index json"),
    )
    .expect("capped snapshot index");
    assert_eq!(
        capped_index["latest_snapshot_id"],
        older_latest_snapshot.snapshot_id
    );
    assert_eq!(
        capped_index["entries"]
            .as_array()
            .expect("capped entries")
            .len(),
        1
    );
    assert_eq!(
        capped_index["entries"][0]["snapshot_id"],
        older_latest_snapshot.snapshot_id
    );
    let export_text =
        std::fs::read_to_string(root.join("latest.json")).expect("safe latest export text");
    for forbidden_field in ["\"broker\"", "\"order\"", "\"account\""] {
        assert!(!export_text.contains(forbidden_field));
    }

    let remote_err = write_committee_state_export(
        CommitteeStateExportConfig {
            export_id: "sprint142-state-export-remote".to_string(),
            export_root_path: "https://example.invalid/state".to_string(),
            write_latest_snapshot: true,
            write_history_snapshot: false,
            write_snapshot_index: false,
            write_owner_console_read_model: false,
            schema_version: "committee-state.v1".to_string(),
            max_history_entries: None,
            paper_only: true,
        },
        &snapshot,
        CommitteeStateSnapshotSource::ManualExport,
    )
    .expect_err("remote export path must fail");
    assert!(remote_err.contains("local path"));
    let traversal_err = write_committee_state_export(
        CommitteeStateExportConfig {
            export_id: "sprint142-state-export-traversal".to_string(),
            export_root_path: "target/../sprint142_bad".to_string(),
            write_latest_snapshot: true,
            write_history_snapshot: false,
            write_snapshot_index: false,
            write_owner_console_read_model: false,
            schema_version: "committee-state.v1".to_string(),
            max_history_entries: None,
            paper_only: true,
        },
        &snapshot,
        CommitteeStateSnapshotSource::ManualExport,
    )
    .expect_err("parent traversal must fail");
    assert!(traversal_err.contains("parent-directory traversal"));
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn owner_console_viewer_loads_and_renders_read_only_terminal_view() {
    let handle = std::thread::Builder::new()
        .name("owner-console-viewer-large-stack".to_string())
        .stack_size(8 * 1024 * 1024)
        .spawn(owner_console_viewer_loads_and_renders_read_only_terminal_view_inner)
        .expect("spawn large-stack owner console viewer test");
    if let Err(payload) = handle.join() {
        std::panic::resume_unwind(payload);
    }
}

fn owner_console_viewer_loads_and_renders_read_only_terminal_view_inner() {
    let run = run_autonomous_paper_committee_loop_from_config_path(std::path::Path::new(
        "examples/soma_minimal_ai_committee_core.toml",
    ))
    .expect("autonomous run with owner read model");
    let snapshot = run
        .committee_state_snapshot
        .clone()
        .expect("committee state snapshot");
    let root = std::path::Path::new("target/sprint143_owner_console_viewer_state");
    let _ = std::fs::remove_dir_all(root);
    write_committee_state_export(
        CommitteeStateExportConfig {
            export_id: "sprint143-owner-console-viewer".to_string(),
            export_root_path: root.display().to_string(),
            write_latest_snapshot: false,
            write_history_snapshot: false,
            write_snapshot_index: false,
            write_owner_console_read_model: true,
            schema_version: "committee-state.v1".to_string(),
            max_history_entries: None,
            paper_only: true,
        },
        &snapshot,
        CommitteeStateSnapshotSource::ManualExport,
    )
    .expect("write read model");
    let read_model_path = root.join("owner_console_read_model.json");
    let before = std::fs::read_to_string(&read_model_path).expect("read model before viewer");
    let (source, read_model) = load_owner_console_read_model_from_local_file(&read_model_path)
        .expect("load local read model");
    assert_eq!(source.schema_version, "committee-state.v1");
    assert_eq!(
        source.snapshot_id.as_deref(),
        Some(snapshot.snapshot_id.as_str())
    );

    let options = OwnerConsoleTerminalOptions {
        max_width: Some(160),
        ..OwnerConsoleTerminalOptions::default()
    };
    let terminal_view = render_owner_console_terminal_view(&read_model, &options);
    assert!(terminal_view.title.contains("Soma Owner Console"));
    assert!(
        terminal_view
            .header_lines
            .iter()
            .any(|line| line.contains("reviewed_symbols="))
    );
    assert_eq!(terminal_view.member_sections.len(), 3);
    assert!(!terminal_view.watchlist_sections.is_empty());
    assert!(!terminal_view.attention_sections.is_empty());
    assert!(!terminal_view.decision_sections.is_empty());
    assert!(!terminal_view.next_action_sections.is_empty());
    assert!(
        terminal_view
            .safety_lines
            .iter()
            .any(|line| line.contains("paper-only"))
    );

    let first = run_owner_console_viewer(&read_model_path, options.clone()).expect("viewer result");
    let second = run_owner_console_viewer(&read_model_path, options).expect("repeat viewer result");
    assert_eq!(first.rendered_text, second.rendered_text);
    assert!(first.rendered_text.contains("Header"));
    assert!(first.rendered_text.contains("Members"));
    assert!(first.rendered_text.contains("Watchlist"));
    assert!(first.rendered_text.contains("Attention"));
    assert!(first.rendered_text.contains("Decisions"));
    assert!(first.rendered_text.contains("Next Actions"));
    assert!(first.rendered_text.contains("Safety"));
    assert!(first.safety_status.contains("read-only"));
    let rendered_json = serde_json::to_string(&first.terminal_view).expect("terminal view json");
    for forbidden_field in ["\"broker\"", "\"order\"", "\"account\""] {
        assert!(!rendered_json.contains(forbidden_field));
    }
    let after = std::fs::read_to_string(&read_model_path).expect("read model after viewer");
    assert_eq!(before, after);

    let remote_err = load_owner_console_read_model_from_local_file(std::path::Path::new(
        "https://example.invalid/owner_console_read_model.json",
    ))
    .expect_err("remote read model path must fail");
    assert!(remote_err.contains("local"));
    let traversal_err = load_owner_console_read_model_from_local_file(std::path::Path::new(
        "target/../owner_console_read_model.json",
    ))
    .expect_err("path traversal must fail");
    assert!(traversal_err.contains("parent-directory traversal"));
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn owner_action_composer_writes_safe_deterministic_action_file() {
    let handle = std::thread::Builder::new()
        .name("owner-action-composer-large-stack".to_string())
        .stack_size(8 * 1024 * 1024)
        .spawn(owner_action_composer_writes_safe_deterministic_action_file_inner)
        .expect("spawn large-stack owner action composer test");
    if let Err(payload) = handle.join() {
        std::panic::resume_unwind(payload);
    }
}

fn owner_action_composer_writes_safe_deterministic_action_file_inner() {
    let run = run_autonomous_paper_committee_loop_from_config_path(std::path::Path::new(
        "examples/soma_minimal_ai_committee_core.toml",
    ))
    .expect("autonomous run with owner read model");
    let snapshot = run
        .committee_state_snapshot
        .clone()
        .expect("committee state snapshot");
    let root = std::path::Path::new("target/sprint144_owner_action_composer");
    let _ = std::fs::remove_dir_all(root);
    write_committee_state_export(
        CommitteeStateExportConfig {
            export_id: "sprint144-owner-action-composer".to_string(),
            export_root_path: root.display().to_string(),
            write_latest_snapshot: false,
            write_history_snapshot: false,
            write_snapshot_index: false,
            write_owner_console_read_model: true,
            schema_version: "committee-state.v1".to_string(),
            max_history_entries: None,
            paper_only: true,
        },
        &snapshot,
        CommitteeStateSnapshotSource::ManualExport,
    )
    .expect("write read model");
    let read_model_path = root.join("owner_console_read_model.json");
    let output_path = root.join("owner_attention_actions.json");
    let (_source, read_model) =
        load_owner_console_read_model_from_local_file(&read_model_path).expect("load read model");
    let target_item_id = read_model
        .attention_cards
        .first()
        .map(|card| card.item_id.clone())
        .expect("attention card target");
    let before = std::fs::read_to_string(&read_model_path).expect("state before compose");
    let dry_run = compose_owner_action_from_read_model(OwnerActionComposerConfig {
        read_model_path: read_model_path.display().to_string(),
        output_actions_path: output_path.display().to_string(),
        target_item_id: Some(target_item_id.clone()),
        action_type: OwnerAttentionActionType::RequestMoreEvidence,
        comment: Some("근거가 부족하니 다시 검토".to_string()),
        dry_run: true,
        paper_only: true,
    })
    .expect("dry-run compose");
    assert!(dry_run.target_item_found);
    assert!(!dry_run.wrote_output);
    assert!(!output_path.exists());
    assert!(dry_run.preview_text.contains("RequestMoreEvidence"));

    let write_result = compose_owner_action_from_read_model(OwnerActionComposerConfig {
        read_model_path: read_model_path.display().to_string(),
        output_actions_path: output_path.display().to_string(),
        target_item_id: Some(target_item_id.clone()),
        action_type: OwnerAttentionActionType::RequestMoreEvidence,
        comment: Some("근거가 부족하니 다시 검토".to_string()),
        dry_run: false,
        paper_only: true,
    })
    .expect("write compose");
    assert!(write_result.wrote_output);
    assert!(output_path.exists());
    let first_text = std::fs::read_to_string(&output_path).expect("action file");
    let action_file: OwnerActionFile = serde_json::from_str(&first_text).expect("action file json");
    assert!(action_file.paper_only);
    assert_eq!(action_file.actions.len(), 1);
    assert_eq!(action_file.actions[0].item_id, target_item_id);
    assert_eq!(
        action_file.actions[0].action_type,
        OwnerAttentionActionType::RequestMoreEvidence
    );
    let loaded_actions =
        load_owner_attention_actions_from_local_json(&output_path).expect("load action file");
    assert_eq!(loaded_actions, action_file.actions);
    for forbidden_field in ["\"broker\"", "\"order\"", "\"account\""] {
        assert!(!first_text.contains(forbidden_field));
    }

    compose_owner_action_from_read_model(OwnerActionComposerConfig {
        read_model_path: read_model_path.display().to_string(),
        output_actions_path: output_path.display().to_string(),
        target_item_id: Some(target_item_id.clone()),
        action_type: OwnerAttentionActionType::RequestMoreEvidence,
        comment: Some("근거가 부족하니 다시 검토".to_string()),
        dry_run: false,
        paper_only: true,
    })
    .expect("repeat compose");
    assert_eq!(
        first_text,
        std::fs::read_to_string(&output_path).expect("repeat action file")
    );
    assert_eq!(
        before,
        std::fs::read_to_string(&read_model_path).expect("state after compose")
    );

    let general_comment_path = root.join("owner_general_comment_actions.json");
    let general_comment = compose_owner_action_from_read_model(OwnerActionComposerConfig {
        read_model_path: read_model_path.display().to_string(),
        output_actions_path: general_comment_path.display().to_string(),
        target_item_id: None,
        action_type: OwnerAttentionActionType::AddComment,
        comment: Some("확인함".to_string()),
        dry_run: false,
        paper_only: true,
    })
    .expect("targetless add-comment compose");
    assert!(!general_comment.target_item_found);
    assert!(general_comment.wrote_output);
    let general_actions = load_owner_attention_actions_from_local_json(&general_comment_path)
        .expect("load targetless add-comment action");
    assert_eq!(general_actions.len(), 1);
    assert_eq!(general_actions[0].item_id, "owner-general-comment");
    assert_eq!(
        general_actions[0].action_type,
        OwnerAttentionActionType::AddComment
    );

    let unknown_err = compose_owner_action_from_read_model(OwnerActionComposerConfig {
        read_model_path: read_model_path.display().to_string(),
        output_actions_path: output_path.display().to_string(),
        target_item_id: Some("missing-attention-item".to_string()),
        action_type: OwnerAttentionActionType::RequestMoreEvidence,
        comment: Some("근거가 부족하니 다시 검토".to_string()),
        dry_run: true,
        paper_only: true,
    })
    .expect_err("unknown target rejected");
    assert!(unknown_err.contains("not found"));
    let remote_state_err = compose_owner_action_from_read_model(OwnerActionComposerConfig {
        read_model_path: "https://example.invalid/owner_console_read_model.json".to_string(),
        output_actions_path: output_path.display().to_string(),
        target_item_id: Some(target_item_id.clone()),
        action_type: OwnerAttentionActionType::RequestMoreEvidence,
        comment: Some("근거가 부족하니 다시 검토".to_string()),
        dry_run: true,
        paper_only: true,
    })
    .expect_err("remote state rejected");
    assert!(remote_state_err.contains("local"));
    let remote_output_err = compose_owner_action_from_read_model(OwnerActionComposerConfig {
        read_model_path: read_model_path.display().to_string(),
        output_actions_path: "https://example.invalid/actions.json".to_string(),
        target_item_id: Some(target_item_id.clone()),
        action_type: OwnerAttentionActionType::RequestMoreEvidence,
        comment: Some("근거가 부족하니 다시 검토".to_string()),
        dry_run: true,
        paper_only: true,
    })
    .expect_err("remote output rejected");
    assert!(remote_output_err.contains("local"));
    let traversal_err = compose_owner_action_from_read_model(OwnerActionComposerConfig {
        read_model_path: read_model_path.display().to_string(),
        output_actions_path: "target/../owner_attention_actions.json".to_string(),
        target_item_id: Some(target_item_id),
        action_type: OwnerAttentionActionType::RequestMoreEvidence,
        comment: Some("근거가 부족하니 다시 검토".to_string()),
        dry_run: true,
        paper_only: true,
    })
    .expect_err("path traversal rejected");
    assert!(traversal_err.contains("parent-directory traversal"));
    let unsafe_order_err = compose_owner_action_from_read_model(OwnerActionComposerConfig {
        read_model_path: read_model_path.display().to_string(),
        output_actions_path: output_path.display().to_string(),
        target_item_id: read_model
            .attention_cards
            .first()
            .map(|card| card.item_id.clone()),
        action_type: OwnerAttentionActionType::RequestMoreEvidence,
        comment: Some("계좌에서 주문 넣어".to_string()),
        dry_run: true,
        paper_only: true,
    })
    .expect_err("unsafe order text rejected");
    assert!(unsafe_order_err.contains("unsafe instruction"));
    let unsafe_leverage_err = compose_owner_action_from_read_model(OwnerActionComposerConfig {
        read_model_path: read_model_path.display().to_string(),
        output_actions_path: output_path.display().to_string(),
        target_item_id: read_model
            .attention_cards
            .first()
            .map(|card| card.item_id.clone()),
        action_type: OwnerAttentionActionType::RequestMoreEvidence,
        comment: Some("레버리지 최대로".to_string()),
        dry_run: true,
        paper_only: true,
    })
    .expect_err("unsafe leverage text rejected");
    assert!(unsafe_leverage_err.contains("unsafe instruction"));
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn owner_action_composer_outputs_feed_existing_triage_actions() {
    let handle = std::thread::Builder::new()
        .name("owner-action-composer-triage-large-stack".to_string())
        .stack_size(8 * 1024 * 1024)
        .spawn(owner_action_composer_outputs_feed_existing_triage_actions_inner)
        .expect("spawn large-stack owner action composer triage test");
    if let Err(payload) = handle.join() {
        std::panic::resume_unwind(payload);
    }
}

fn owner_action_composer_outputs_feed_existing_triage_actions_inner() {
    let run = run_autonomous_paper_committee_loop_from_config_path(std::path::Path::new(
        "examples/soma_minimal_ai_committee_core.toml",
    ))
    .expect("autonomous run with owner attention");
    let snapshot = run
        .committee_state_snapshot
        .clone()
        .expect("committee state snapshot");
    let root = std::path::Path::new("target/sprint144_owner_action_triage_bridge");
    let _ = std::fs::remove_dir_all(root);
    write_committee_state_export(
        CommitteeStateExportConfig {
            export_id: "sprint144-owner-action-triage".to_string(),
            export_root_path: root.display().to_string(),
            write_latest_snapshot: false,
            write_history_snapshot: false,
            write_snapshot_index: false,
            write_owner_console_read_model: true,
            schema_version: "committee-state.v1".to_string(),
            max_history_entries: None,
            paper_only: true,
        },
        &snapshot,
        CommitteeStateSnapshotSource::ManualExport,
    )
    .expect("write read model");
    let read_model_path = root.join("owner_console_read_model.json");
    let (_source, read_model) =
        load_owner_console_read_model_from_local_file(&read_model_path).expect("load read model");
    let attention_ids = read_model
        .attention_cards
        .iter()
        .map(|card| card.item_id.clone())
        .collect::<Vec<_>>();
    assert!(attention_ids.len() >= 4);
    let watch_target_id = run
        .attention_queue
        .items
        .iter()
        .find(|item| {
            item.symbol.is_some()
                && item.market_scope.is_some()
                && attention_ids.iter().any(|id| id == &item.item_id)
        })
        .map(|item| item.item_id.clone())
        .expect("watchlist-convertible attention item");
    let action_specs = [
        (
            OwnerAttentionActionType::RequestMoreEvidence,
            Some(attention_ids[0].clone()),
            "근거가 부족하니 다시 검토",
        ),
        (
            OwnerAttentionActionType::RequestReconsideration,
            Some(attention_ids[1].clone()),
            "리스크 위원 의견을 다시 반영",
        ),
        (
            OwnerAttentionActionType::ConvertToWatchlist,
            Some(watch_target_id),
            "관심종목으로 보류",
        ),
        (
            OwnerAttentionActionType::AddComment,
            Some(attention_ids[3].clone()),
            "확인함",
        ),
        (OwnerAttentionActionType::AddComment, None, "확인함"),
    ];
    let mut owner_actions = Vec::new();
    for (index, (action_type, item_id, comment)) in action_specs.iter().enumerate() {
        let path = root.join(format!("owner_attention_actions_{index}.json"));
        compose_owner_action_from_read_model(OwnerActionComposerConfig {
            read_model_path: read_model_path.display().to_string(),
            output_actions_path: path.display().to_string(),
            target_item_id: item_id.clone(),
            action_type: *action_type,
            comment: Some((*comment).to_string()),
            dry_run: false,
            paper_only: true,
        })
        .expect("compose triage action");
        owner_actions.extend(
            load_owner_attention_actions_from_local_json(&path).expect("load composed action"),
        );
    }
    let triage = run_owner_attention_triage(OwnerAttentionTriageInput {
        previous_run: run,
        previous_inbox: None,
        owner_actions,
        watchlist_store: None,
    })
    .expect("triage consumes composed actions");
    assert!(
        triage
            .action_results
            .iter()
            .all(|result| result.safety_status == OwnerAttentionActionSafetyStatus::Passed)
    );
    assert!(triage.generated_owner_feedback_count >= 4);
    assert!(!triage.generated_watchlist_candidates.is_empty());
    assert!(
        triage
            .generated_owner_feedback
            .iter()
            .any(|feedback| feedback.feedback_type == OwnerFeedbackType::EvidenceRequest)
    );
    assert!(
        triage
            .generated_owner_feedback
            .iter()
            .any(|feedback| feedback.feedback_type == OwnerFeedbackType::ReconsiderationRequest)
    );
    assert!(
        triage
            .generated_owner_feedback
            .iter()
            .any(|feedback| feedback.feedback_type == OwnerFeedbackType::Comment)
    );
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn owner_action_consumption_applies_ledgers_duplicates_and_refreshes_state() {
    let handle = std::thread::Builder::new()
        .name("owner-action-consumption-ledger-test".to_string())
        .stack_size(8 * 1024 * 1024)
        .spawn(owner_action_consumption_applies_ledgers_duplicates_and_refreshes_state_inner)
        .expect("spawn owner action consumption ledger test");
    if let Err(payload) = handle.join() {
        std::panic::resume_unwind(payload);
    }
}

fn owner_action_consumption_applies_ledgers_duplicates_and_refreshes_state_inner() {
    let run = run_autonomous_paper_committee_loop_from_config_path(std::path::Path::new(
        "examples/soma_minimal_ai_committee_core.toml",
    ))
    .expect("autonomous run for owner action consumption");
    let snapshot = run
        .committee_state_snapshot
        .clone()
        .expect("committee state snapshot");
    let root = std::path::Path::new("target/sprint145_owner_action_consumption");
    let _ = std::fs::remove_dir_all(root);
    write_committee_state_export(
        CommitteeStateExportConfig {
            export_id: "sprint145-owner-action-consumption".to_string(),
            export_root_path: root.display().to_string(),
            write_latest_snapshot: true,
            write_history_snapshot: true,
            write_snapshot_index: true,
            write_owner_console_read_model: true,
            schema_version: "committee-state.v1".to_string(),
            max_history_entries: Some(3),
            paper_only: true,
        },
        &snapshot,
        CommitteeStateSnapshotSource::ManualExport,
    )
    .expect("write read model");
    let read_model_path = root.join("owner_console_read_model.json");
    let action_path = root.join("owner_attention_actions.json");
    let ledger_path = root.join("owner_action_processing_ledger.json");
    let inbox_path = root.join("owner_attention_inbox.json");
    let watchlist_path = root.join("owner_watchlist_store.json");
    let member_state_path = root.join("member_state_store.json");
    let (_source, read_model) =
        load_owner_console_read_model_from_local_file(&read_model_path).expect("load read model");
    let target_item_id = read_model
        .attention_cards
        .first()
        .map(|card| card.item_id.clone())
        .expect("attention target");
    compose_owner_action_from_read_model(OwnerActionComposerConfig {
        read_model_path: read_model_path.display().to_string(),
        output_actions_path: action_path.display().to_string(),
        target_item_id: Some(target_item_id),
        action_type: OwnerAttentionActionType::RequestMoreEvidence,
        comment: Some("근거가 부족하니 다시 검토".to_string()),
        dry_run: false,
        paper_only: true,
    })
    .expect("compose owner action");

    let dry_run = consume_owner_action_file_with_previous_run(
        OwnerActionConsumptionConfig {
            action_file_path: action_path.display().to_string(),
            processed_action_ledger_path: Some(ledger_path.display().to_string()),
            inbox_input_path: None,
            inbox_output_path: Some(inbox_path.display().to_string()),
            watchlist_input_path: None,
            watchlist_output_path: Some(watchlist_path.display().to_string()),
            member_state_input_path: None,
            member_state_output_path: Some(member_state_path.display().to_string()),
            committee_state_export_root_path: Some(root.display().to_string()),
            owner_console_read_model_path: Some(read_model_path.display().to_string()),
            apply_mode: OwnerActionApplyMode::DryRun,
            duplicate_policy: OwnerActionDuplicatePolicy::SkipAlreadyProcessed,
            after_apply_file_policy: OwnerActionAfterApplyFilePolicy::KeepActionFile,
            paper_only: true,
        },
        Some(run.clone()),
    )
    .expect("dry-run consumption");
    assert!(dry_run.dry_run);
    assert_eq!(dry_run.applied_action_count, 0);
    assert!(!ledger_path.exists());
    assert!(!inbox_path.exists());

    let apply_config = OwnerActionConsumptionConfig {
        action_file_path: action_path.display().to_string(),
        processed_action_ledger_path: Some(ledger_path.display().to_string()),
        inbox_input_path: None,
        inbox_output_path: Some(inbox_path.display().to_string()),
        watchlist_input_path: None,
        watchlist_output_path: Some(watchlist_path.display().to_string()),
        member_state_input_path: None,
        member_state_output_path: Some(member_state_path.display().to_string()),
        committee_state_export_root_path: Some(root.display().to_string()),
        owner_console_read_model_path: Some(read_model_path.display().to_string()),
        apply_mode: OwnerActionApplyMode::Apply,
        duplicate_policy: OwnerActionDuplicatePolicy::SkipAlreadyProcessed,
        after_apply_file_policy: OwnerActionAfterApplyFilePolicy::KeepActionFile,
        paper_only: true,
    };
    let applied =
        consume_owner_action_file_with_previous_run(apply_config.clone(), Some(run.clone()))
            .expect("apply consumption");
    assert_eq!(applied.loaded_action_count, 1);
    assert_eq!(applied.applied_action_count, 1);
    assert_eq!(applied.rejected_action_count, 0);
    assert!(ledger_path.exists());
    assert!(inbox_path.exists());
    assert!(member_state_path.exists());
    assert!(
        applied
            .generated_owner_feedback
            .iter()
            .any(|feedback| feedback.feedback_type == OwnerFeedbackType::EvidenceRequest)
    );
    assert!(
        applied
            .reconsideration_results
            .iter()
            .any(|result| result.reconsideration_session_count > 0)
    );
    assert!(applied.updated_committee_state_snapshot.is_some());
    assert!(applied.state_export_result.is_some());
    assert!(
        applied
            .processing_ledger
            .processed_actions
            .iter()
            .any(|record| record.processed_status == OwnerActionProcessedStatus::Applied)
    );

    let archive_action_path = root.join("owner_attention_actions_archive.json");
    compose_owner_action_from_read_model(OwnerActionComposerConfig {
        read_model_path: read_model_path.display().to_string(),
        output_actions_path: archive_action_path.display().to_string(),
        target_item_id: None,
        action_type: OwnerAttentionActionType::AddComment,
        comment: Some("확인함".to_string()),
        dry_run: false,
        paper_only: true,
    })
    .expect("compose archive policy action");
    let archived = consume_owner_action_file_with_previous_run(
        OwnerActionConsumptionConfig {
            action_file_path: archive_action_path.display().to_string(),
            inbox_input_path: Some(inbox_path.display().to_string()),
            watchlist_input_path: Some(watchlist_path.display().to_string()),
            member_state_input_path: Some(member_state_path.display().to_string()),
            after_apply_file_policy: OwnerActionAfterApplyFilePolicy::ArchiveActionFile,
            ..apply_config.clone()
        },
        Some(run.clone()),
    )
    .expect("archive action file after apply");
    assert_eq!(archived.applied_action_count, 1);
    assert!(!archive_action_path.exists());
    assert!(
        root.join("owner_attention_actions_archive.processed.json")
            .exists()
    );

    let skipped = consume_owner_action_file_with_previous_run(apply_config.clone(), Some(run))
        .expect("duplicate skip");
    assert_eq!(skipped.applied_action_count, 0);
    assert_eq!(skipped.skipped_duplicate_count, 1);
    let duplicate_reject = consume_owner_action_file(OwnerActionConsumptionConfig {
        duplicate_policy: OwnerActionDuplicatePolicy::RejectAlreadyProcessed,
        ..apply_config
    })
    .expect_err("duplicate reject");
    assert!(duplicate_reject.contains("already processed"));
    let refreshed_read_model =
        std::fs::read_to_string(&read_model_path).expect("refreshed read model exists");
    assert!(refreshed_read_model.contains("committee-state.v1"));
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn owner_action_consumption_updates_inbox_watchlist_and_general_comment() {
    let handle = std::thread::Builder::new()
        .name("owner-action-consumption-behavior-test".to_string())
        .stack_size(8 * 1024 * 1024)
        .spawn(owner_action_consumption_updates_inbox_watchlist_and_general_comment_inner)
        .expect("spawn owner action consumption behavior test");
    if let Err(payload) = handle.join() {
        std::panic::resume_unwind(payload);
    }
}

fn owner_action_consumption_updates_inbox_watchlist_and_general_comment_inner() {
    let run = run_autonomous_paper_committee_loop_from_config_path(std::path::Path::new(
        "examples/soma_minimal_ai_committee_core.toml",
    ))
    .expect("autonomous run for action behavior");
    let root = std::path::Path::new("target/sprint145_owner_action_behavior");
    let _ = std::fs::remove_dir_all(root);
    std::fs::create_dir_all(root).expect("create behavior root");
    let inbox_items = OwnerAttentionInbox::from_attention_queue(&run.attention_queue).items;
    let watch_item = inbox_items
        .iter()
        .find(|item| item.symbol.is_some() && item.market_scope.is_some())
        .expect("watchlist convertible item");
    let distinct_item_ids = inbox_items
        .iter()
        .map(|item| item.item_id.clone())
        .filter(|item_id| item_id != &watch_item.item_id)
        .fold(Vec::<String>::new(), |mut ids, item_id| {
            if !ids.contains(&item_id) {
                ids.push(item_id);
            }
            ids
        });
    assert!(distinct_item_ids.len() >= 4);
    let actions = vec![
        OwnerAttentionAction {
            action_id: "sprint145-ack".to_string(),
            item_id: distinct_item_ids[0].clone(),
            action_type: OwnerAttentionActionType::Acknowledge,
            comment: Some("확인함".to_string()),
            created_at: None,
            paper_only: true,
        },
        OwnerAttentionAction {
            action_id: "sprint145-defer".to_string(),
            item_id: distinct_item_ids[1].clone(),
            action_type: OwnerAttentionActionType::Defer,
            comment: Some("관심종목으로 보류".to_string()),
            created_at: None,
            paper_only: true,
        },
        OwnerAttentionAction {
            action_id: "sprint145-dismiss".to_string(),
            item_id: distinct_item_ids[2].clone(),
            action_type: OwnerAttentionActionType::Dismiss,
            comment: Some("이 항목은 dismiss".to_string()),
            created_at: None,
            paper_only: true,
        },
        OwnerAttentionAction {
            action_id: "sprint145-watch".to_string(),
            item_id: watch_item.item_id.clone(),
            action_type: OwnerAttentionActionType::ConvertToWatchlist,
            comment: Some("관심종목으로 보류".to_string()),
            created_at: None,
            paper_only: true,
        },
        OwnerAttentionAction {
            action_id: "sprint145-reconsider".to_string(),
            item_id: distinct_item_ids[3].clone(),
            action_type: OwnerAttentionActionType::RequestReconsideration,
            comment: Some("리스크 위원 의견을 다시 반영".to_string()),
            created_at: None,
            paper_only: true,
        },
        OwnerAttentionAction {
            action_id: "sprint145-general-comment".to_string(),
            item_id: "owner-general-comment".to_string(),
            action_type: OwnerAttentionActionType::AddComment,
            comment: Some("확인함".to_string()),
            created_at: None,
            paper_only: true,
        },
    ];
    let action_path = root.join("actions.json");
    std::fs::write(
        &action_path,
        serde_json::to_string_pretty(&OwnerActionFile {
            schema_version: "owner-action-file.v1".to_string(),
            source_snapshot_id: Some("sprint145-behavior".to_string()),
            actions,
            paper_only: true,
            safety_notes: vec!["paper-only local triage input".to_string()],
        })
        .expect("action file json"),
    )
    .expect("write action file");
    let result = consume_owner_action_file_with_previous_run(
        OwnerActionConsumptionConfig {
            action_file_path: action_path.display().to_string(),
            processed_action_ledger_path: Some(root.join("ledger.json").display().to_string()),
            inbox_input_path: None,
            inbox_output_path: Some(root.join("inbox.json").display().to_string()),
            watchlist_input_path: None,
            watchlist_output_path: Some(root.join("watchlist.json").display().to_string()),
            member_state_input_path: None,
            member_state_output_path: None,
            committee_state_export_root_path: None,
            owner_console_read_model_path: None,
            apply_mode: OwnerActionApplyMode::Apply,
            duplicate_policy: OwnerActionDuplicatePolicy::SkipAlreadyProcessed,
            after_apply_file_policy: OwnerActionAfterApplyFilePolicy::KeepActionFile,
            paper_only: true,
        },
        Some(run),
    )
    .expect("consume behavior actions");
    assert!(result.action_results.iter().any(|result| {
        result.action_id == "sprint145-ack"
            && result.new_status == OwnerAttentionInboxStatus::Acknowledged
    }));
    assert!(result.action_results.iter().any(|result| {
        result.action_id == "sprint145-defer"
            && result.new_status == OwnerAttentionInboxStatus::Deferred
    }));
    assert!(result.action_results.iter().any(|result| {
        result.action_id == "sprint145-dismiss"
            && result.new_status == OwnerAttentionInboxStatus::Dismissed
    }));
    assert!(result.action_results.iter().any(|result| {
        result.action_id == "sprint145-watch"
            && result.new_status == OwnerAttentionInboxStatus::ConvertedToWatchlist
    }));
    assert!(!result.generated_watchlist_candidates.is_empty());
    assert!(
        result
            .generated_watchlist_candidates
            .iter()
            .all(|candidate| candidate.paper_only)
    );
    assert!(
        result.generated_owner_feedback.iter().any(|feedback| {
            feedback.feedback_type == OwnerFeedbackType::ReconsiderationRequest
        })
    );
    assert!(
        result
            .generated_owner_feedback
            .iter()
            .any(|feedback| feedback.feedback_type == OwnerFeedbackType::Comment)
    );
    assert!(
        result
            .reconsideration_results
            .iter()
            .any(|item| item.chairman_reconsideration_decision_count > 0)
    );
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn owner_action_consumption_rejects_paths_and_unsafe_files() {
    let remote_err = consume_owner_action_file(OwnerActionConsumptionConfig {
        action_file_path: "https://example.invalid/actions.json".to_string(),
        processed_action_ledger_path: None,
        inbox_input_path: None,
        inbox_output_path: None,
        watchlist_input_path: None,
        watchlist_output_path: None,
        member_state_input_path: None,
        member_state_output_path: None,
        committee_state_export_root_path: None,
        owner_console_read_model_path: None,
        apply_mode: OwnerActionApplyMode::DryRun,
        duplicate_policy: OwnerActionDuplicatePolicy::SkipAlreadyProcessed,
        after_apply_file_policy: OwnerActionAfterApplyFilePolicy::KeepActionFile,
        paper_only: true,
    })
    .expect_err("remote action path rejected");
    assert!(remote_err.contains("local"));
    let traversal_err = consume_owner_action_file(OwnerActionConsumptionConfig {
        action_file_path: "target/../actions.json".to_string(),
        processed_action_ledger_path: None,
        inbox_input_path: None,
        inbox_output_path: None,
        watchlist_input_path: None,
        watchlist_output_path: None,
        member_state_input_path: None,
        member_state_output_path: None,
        committee_state_export_root_path: None,
        owner_console_read_model_path: None,
        apply_mode: OwnerActionApplyMode::DryRun,
        duplicate_policy: OwnerActionDuplicatePolicy::SkipAlreadyProcessed,
        after_apply_file_policy: OwnerActionAfterApplyFilePolicy::KeepActionFile,
        paper_only: true,
    })
    .expect_err("traversal action path rejected");
    assert!(traversal_err.contains("parent-directory traversal"));

    let root = std::path::Path::new("target/sprint145_unsafe_action_file");
    let _ = std::fs::remove_dir_all(root);
    std::fs::create_dir_all(root).expect("create unsafe root");
    let unsafe_path = root.join("actions.json");
    std::fs::write(
        &unsafe_path,
        r#"{"schema_version":"owner-action-file.v1","paper_only":true,"order_id":"bad","actions":[]}"#,
    )
    .expect("write unsafe action file");
    let unsafe_err = consume_owner_action_file(OwnerActionConsumptionConfig {
        action_file_path: unsafe_path.display().to_string(),
        processed_action_ledger_path: None,
        inbox_input_path: None,
        inbox_output_path: None,
        watchlist_input_path: None,
        watchlist_output_path: None,
        member_state_input_path: None,
        member_state_output_path: None,
        committee_state_export_root_path: None,
        owner_console_read_model_path: None,
        apply_mode: OwnerActionApplyMode::DryRun,
        duplicate_policy: OwnerActionDuplicatePolicy::SkipAlreadyProcessed,
        after_apply_file_policy: OwnerActionAfterApplyFilePolicy::KeepActionFile,
        paper_only: true,
    })
    .expect_err("unsafe action file rejected");
    assert!(unsafe_err.contains("unsafe field"));
    let unsafe_quantity_path = root.join("quantity_actions.json");
    std::fs::write(
        &unsafe_quantity_path,
        r#"{"schema_version":"owner-action-file.v1","paper_only":true,"quantity":10,"actions":[]}"#,
    )
    .expect("write unsafe quantity action file");
    let unsafe_quantity_err = consume_owner_action_file(OwnerActionConsumptionConfig {
        action_file_path: unsafe_quantity_path.display().to_string(),
        processed_action_ledger_path: None,
        inbox_input_path: None,
        inbox_output_path: None,
        watchlist_input_path: None,
        watchlist_output_path: None,
        member_state_input_path: None,
        member_state_output_path: None,
        committee_state_export_root_path: None,
        owner_console_read_model_path: None,
        apply_mode: OwnerActionApplyMode::DryRun,
        duplicate_policy: OwnerActionDuplicatePolicy::SkipAlreadyProcessed,
        after_apply_file_policy: OwnerActionAfterApplyFilePolicy::KeepActionFile,
        paper_only: true,
    })
    .expect_err("unsafe quantity action file rejected");
    assert!(unsafe_quantity_err.contains("unsafe field"));
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn member_core_contract_remains_deferred_and_viewer_adds_no_web_dependency() {
    let spec = Mamba3GatedDeltaNetCoreSpec::runtime_deferred_for("contract-lock-member");
    assert_eq!(spec.core_family, MemberCoreFamily::Mamba3GatedDeltaNet);
    assert_eq!(spec.sequence_core, SequenceCoreKind::Mamba3Deferred);
    assert_eq!(spec.memory_core, MemoryCoreKind::GatedDeltaNetDeferred);
    assert_eq!(spec.runtime_status, CoreRuntimeStatus::RuntimeDeferred);
    assert!(
        spec.notes
            .iter()
            .any(|note| note.contains("runtime/training/live inference deferred"))
    );
    let policy = mac_mini_local_policy();
    assert!(policy.do_not_run_all_18_cores_concurrently);
    assert!(policy.lazy_activation);
    assert!(
        policy
            .notes
            .iter()
            .any(|note| note.contains("owner console viewer is read-only UI data"))
    );

    let manifest = std::fs::read_to_string("Cargo.toml").expect("Cargo manifest");
    for forbidden in ["tauri", "svelte", "react", "javascript", "typescript"] {
        assert!(!manifest.to_ascii_lowercase().contains(forbidden));
    }
    let mut forbidden_web_ui_files = Vec::new();
    collect_forbidden_web_ui_files(std::path::Path::new("."), &mut forbidden_web_ui_files)
        .expect("scan web UI files");
    assert!(
        forbidden_web_ui_files.is_empty(),
        "web UI files must not be added: {:?}",
        forbidden_web_ui_files
    );
}

fn collect_forbidden_web_ui_files(
    root: &std::path::Path,
    matches: &mut Vec<std::path::PathBuf>,
) -> Result<(), String> {
    for entry in std::fs::read_dir(root).map_err(|err| err.to_string())? {
        let entry = entry.map_err(|err| err.to_string())?;
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if path.is_dir() {
            if matches!(
                name.as_ref(),
                ".git" | "target" | "node_modules" | ".svelte-kit" | "dist"
            ) {
                continue;
            }
            collect_forbidden_web_ui_files(&path, matches)?;
            continue;
        }
        let lower_name = name.to_ascii_lowercase();
        let lower_ext = path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();
        if matches!(
            lower_name.as_str(),
            "package.json"
                | "package-lock.json"
                | "pnpm-lock.yaml"
                | "yarn.lock"
                | "vite.config.js"
                | "vite.config.ts"
                | "svelte.config.js"
                | "tauri.conf.json"
        ) || matches!(lower_ext.as_str(), "js" | "jsx" | "ts" | "tsx" | "svelte")
        {
            matches.push(path);
        }
    }
    Ok(())
}

#[test]
fn daily_brief_storage_update_saves_snapshot_without_running_new_analysis() {
    let handle = std::thread::Builder::new()
        .name("daily-brief-storage-update-test".to_string())
        .stack_size(8 * 1024 * 1024)
        .spawn(daily_brief_storage_update_saves_snapshot_without_running_new_analysis_inner)
        .expect("spawn storage update test");
    if let Err(payload) = handle.join() {
        std::panic::resume_unwind(payload);
    }
}

fn daily_brief_storage_update_saves_snapshot_without_running_new_analysis_inner() {
    std::fs::create_dir_all("target").expect("target dir");
    let config_path = std::path::Path::new("target/sprint141_storage_update_autonomous.toml");
    let config_text = std::fs::read_to_string("examples/soma_minimal_ai_committee_core.toml")
        .expect("read autonomous config");
    let isolated_config = config_text
        .replace("target/minimal_", "target/sprint141_storage_update_")
        .replace(
            "run_id = \"soma-autonomous-paper-sprint135\"",
            "run_id = \"sprint141-storage-update\"",
        );
    std::fs::write(config_path, isolated_config).expect("write isolated autonomous config");
    let run = run_autonomous_paper_committee_loop_from_config_path(config_path)
        .expect("autonomous run for storage update");
    let recheck = run.watchlist_recheck.clone().expect("watchlist recheck");
    let brief = recheck
        .owner_daily_brief
        .clone()
        .expect("owner daily brief");
    let store_path = "target/sprint141_storage_update_briefs.json";
    let snapshot_path = "target/sprint141_storage_update_snapshot.json";
    let _ = std::fs::remove_file(store_path);
    let _ = std::fs::remove_file(snapshot_path);
    let result = run_daily_brief_storage_update(DailyBriefStorageUpdateInput {
        owner_daily_brief: Some(brief.clone()),
        owner_daily_brief_store_input_path: None,
        owner_daily_brief_store_output_path: Some(store_path.to_string()),
        committee_state_snapshot_output_path: Some(snapshot_path.to_string()),
        emit_committee_state_snapshot: true,
        export_input: CommitteeStateExportInput {
            autonomous_run_result: Some(run.clone()),
            watchlist_recheck_result: Some(recheck.clone()),
            attention_inbox: run
                .owner_attention_triage
                .as_ref()
                .map(|triage| triage.inbox.clone()),
            watchlist_store: Some(recheck.updated_watchlist_store.clone()),
            member_state_store: Some(MemberStateStore {
                store_id: "storage-update-member-state-store".to_string(),
                members: run.final_member_states.clone(),
                source_label: "unit-test".to_string(),
                paper_only: true,
            }),
            owner_daily_brief_store: None,
            owner_summary: Some(recheck.owner_summary.clone()),
            owner_console_view: recheck.owner_console_view.clone(),
        },
    })
    .expect("storage update");
    let _ = std::fs::remove_file(config_path);
    assert_eq!(
        result
            .updated_brief_store
            .latest()
            .map(|item| &item.brief_id),
        Some(&brief.brief_id)
    );
    assert!(result.committee_state_snapshot.is_some());
    assert!(result.safety_summary.no_broker_order_account);
    assert!(std::path::Path::new(store_path).exists());
    assert!(std::path::Path::new(snapshot_path).exists());
    let _ = std::fs::remove_file(store_path);
    let _ = std::fs::remove_file(snapshot_path);
}

#[test]
fn autonomous_paper_config_rejects_remote_and_unsafe_fields() {
    let config = MinimalAiCommitteeCycleConfig {
        input_path: Some("examples/minimal_ai_committee_core_sample.json".to_string()),
        offline_member_opinion_path: None,
        offline_member_output_batch_path: None,
        batch_mode: true,
        member_state_input_path: None,
        member_state_output_path: None,
        emit_owner_summary: true,
        emit_owner_console_view: true,
        owner_feedback_path: None,
        owner_comment_text: None,
        owner_comment_path: None,
        owner_intent_policy_path: None,
        emit_reconsideration_view: false,
        member_experience_store_input_path: None,
        member_experience_store_output_path: None,
        replay_dataset_output_path: None,
        emit_learning_summary: false,
        emit_replay_dataset_summary: false,
        replay_quality_eval_enabled: false,
        replay_quality_eval_output_path: None,
        min_replay_examples_required: 10,
        min_examples_per_member_required: 2,
        replay_sanitization_enabled: false,
        sanitized_replay_dataset_output_path: None,
        strict_temporal_boundary: true,
        include_post_decision_context_for_audit: true,
        reject_on_blocking_leakage: true,
        replay_coverage_eval_enabled: false,
        replay_coverage_target_min_total: 10,
        replay_coverage_collection_queue_output_path: None,
        paper_scenario_collection_enabled: false,
        paper_outcome_fixture_path: None,
        scenario_run_output_path: None,
        label_validation_enabled: false,
        validated_replay_dataset_output_path: None,
        label_quality_summary_output_path: None,
        min_validated_label_ratio_required: 0.5,
        paper_label_validation_policy_path: None,
        backtest_label_contract_path: None,
        label_validation_with_evidence_enabled: false,
        paper_outcome_evidence_path: None,
        paper_outcome_evidence_quality_output_path: None,
        validated_replay_with_evidence_output_path: None,
        evidence_backfill_enabled: false,
        evidence_backfill_dry_run: true,
        evidence_backfill_apply_patch: false,
        evidence_backfill_output_path: None,
        evidence_backfill_min_validated_ratio: 0.5,
        evidence_backfill_emit_summary: false,
        validated_ratio_expansion_enabled: false,
        validated_ratio_expansion_dry_run: true,
        paper_price_series_path: None,
        generated_paper_evidence_output_path: None,
        validated_ratio_target: 0.5,
        validated_ratio_expansion_output_path: None,
        weak_label_review_enabled: false,
        weak_label_review_decision_path: None,
        weak_label_review_output_path: None,
        replay_training_inclusion_mask_output_path: None,
        weak_label_review_dry_run: true,
        exclude_weak_labels_from_training_design: true,
        weak_label_closure_enabled: false,
        weak_label_closure_dry_run: true,
        training_candidate_dataset_output_path: None,
        training_split_output_path: None,
        offline_trainer_dry_run_enabled: false,
        offline_trainer_dry_run_output_path: None,
        offline_trainer_v2_enabled: false,
        offline_trainer_v2_batch_size: 8,
        offline_trainer_v2_output_path: None,
        offline_trainer_design_status_output_path: None,
        trainer_readiness_brief_enabled: false,
        trainer_readiness_brief_output_path: None,
        tiny_training_eligibility_gate_enabled: false,
        tiny_training_contract_output_path: None,
        min_tiny_training_examples_required: 8,
        min_tiny_training_members_required: 3,
        tiny_no_weight_loss_simulation_enabled: false,
        tiny_no_weight_loss_simulation_output_path: None,
        tiny_loss_batch_size: 8,
        tiny_loss_enabled_heads: vec![
            soma_zero::league::minimal_ai_committee_core::TrainingTargetHead::Stance,
            soma_zero::league::minimal_ai_committee_core::TrainingTargetHead::ConfidenceCalibration,
            soma_zero::league::minimal_ai_committee_core::TrainingTargetHead::Risk,
            soma_zero::league::minimal_ai_committee_core::TrainingTargetHead::EvidenceNeed,
        ],
        tiny_loss_prediction_policy:
            soma_zero::league::minimal_ai_committee_core::DummyPredictionPolicy::default(),
        no_persistence_training_gate_enabled: false,
        no_persistence_training_simulation_enabled: false,
        no_persistence_training_simulation_output_path: None,
        no_persistence_training_brief_output_path: None,
        no_persistence_max_epochs: 1,
        no_persistence_max_steps: 3,
        smartcore_adapter_skeleton_gate_enabled: false,
        adapter_skeleton_dry_run_enabled: false,
        adapter_skeleton_output_path: None,
        adapter_skeleton_include_sparse_event_attention: true,
        adapter_skeleton_validate_batches: true,
        adapter_skeleton_require_runtime_deferred: true,
        adapter_skeleton_require_training_deferred: true,
        adapter_contract_lock_enabled: false,
        adapter_contract_golden_snapshot_output_path: None,
        adapter_contract_expected_snapshot_path: None,
        adapter_contract_require_schema_version_match: true,
        adapter_contract_fail_on_unmatched_batch: true,
        adapter_contract_fail_on_unknown_member_alias: true,
        adapter_contract_fail_on_output_values: true,
        adapter_contract_lock_v2_enabled: false,
        adapter_expected_golden_baseline_path: None,
        adapter_bootstrap_golden_baseline_path: None,
        adapter_bootstrap_missing_baseline: false,
        adapter_write_golden_baseline_if_missing: false,
        adapter_fail_on_missing_baseline: true,
        adapter_allow_schema_version_bump: false,
        adapter_run_regression_harness: false,
        adapter_contract_acceptance_output_path: None,
        runtime_adapter_entry_gate_enabled: false,
        runtime_entry_audit_output_path: None,
        runtime_entry_requested_capabilities: vec![
            SmartCoreRuntimeCapability::ShapeValidation,
            SmartCoreRuntimeCapability::BuildInputShape,
            SmartCoreRuntimeCapability::BuildOutputShape,
            SmartCoreRuntimeCapability::ValidateAdapterContract,
            SmartCoreRuntimeCapability::ValidateGoldenBaseline,
        ],
        runtime_entry_run_negative_harness: false,
        runtime_entry_fail_on_forbidden_capability: true,
        runtime_entry_fail_on_contract_not_locked: true,
        runtime_entry_fail_on_baseline_drift: true,
        runtime_entry_fail_on_safety_violation: true,
        smartcore_microkernel_v0_enabled: false,
        smartcore_microkernel_lab_mode: false,
        smartcore_microkernel_output_path: None,
        smartcore_microkernel_sequence_len: 4,
        smartcore_microkernel_input_dim: 8,
        smartcore_microkernel_temporal_state_dim: 8,
        smartcore_microkernel_memory_dim: 8,
        smartcore_microkernel_output_dim: 8,
        smartcore_microkernel_use_training_candidates: true,
        smartcore_microkernel_synthetic_fallback: true,
        microkernel_bridge_enabled: false,
        microkernel_bridge_sequence_len: 4,
        microkernel_bridge_input_dim: 8,
        microkernel_bridge_fail_on_warning: false,
        microkernel_bridge_output_path: None,
        smartcore_head_projection_v0_enabled: false,
        smartcore_head_projection_output_path: None,
        smartcore_enable_stance_head: true,
        smartcore_enable_risk_head: true,
        smartcore_enable_evidence_head: true,
        smartcore_enable_confidence_head: true,
        smartcore_enable_uncertainty_head: true,
        smartcore_enable_expected_return_head: false,
        smartcore_shadow_alignment_enabled: false,
        smartcore_shadow_alignment_output_path: None,
        smartcore_shadow_include_batch_member_opinions: true,
        smartcore_shadow_include_replay_targets: true,
        smartcore_shadow_include_risk_governor_targets: true,
        smartcore_emit_owner_debug_cards: true,
        smartcore_mismatch_self_growing_enabled: false,
        smartcore_mismatch_max_tasks_total: 12,
        smartcore_mismatch_max_tasks_per_member: 4,
        smartcore_calibration_dataset_output_path: None,
        smartcore_mismatch_task_output_path: None,
        smartcore_mismatch_emit_owner_debug_summary: true,
        smartcore_mismatch_learning_loop_enabled: false,
        smartcore_mismatch_learning_dry_run: true,
        smartcore_execute_mismatch_research_tasks: false,
        smartcore_approve_calibration_targets: false,
        smartcore_refresh_calibration_dataset: false,
        smartcore_recheck_alignment: false,
        smartcore_calibration_dataset_input_path: None,
        smartcore_mismatch_learning_loop_output_path: None,
        smartcore_recalibration_enabled: false,
        smartcore_recalibration_dry_run: true,
        smartcore_recalibration_rule_table_output_path: None,
        smartcore_calibrated_debug_output_path: None,
        smartcore_recalibration_result_output_path: None,
        smartcore_recalibration_min_support: 2,
        smartcore_recalibration_max_rules_per_member_head: 2,
        smartcore_recalibration_emit_owner_summary: true,
        smartcore_shadow_opinion_enabled: false,
        smartcore_shadow_opinion_output_path: None,
        smartcore_shadow_compare_member_opinion: false,
        smartcore_shadow_target_eval: false,
        smartcore_shadow_emit_owner_debug: true,
        smartcore_shadow_stability_enabled: false,
        smartcore_shadow_stability_repeats: 3,
        smartcore_shadow_stability_output_path: None,
        smartcore_shadow_expand_agreement_targets: false,
        smartcore_shadow_target_collection_queue_output_path: None,
        smartcore_shadow_stability_emit_owner_summary: true,
        smartcore_shadow_scenario_sweep_enabled: false,
        smartcore_shadow_scenario_set_path: None,
        smartcore_shadow_scenario_repeats: 3,
        smartcore_shadow_scenario_max_count: 5,
        smartcore_shadow_scenario_output_path: None,
        smartcore_observer_readiness_gate_enabled: false,
        smartcore_observer_min_scenarios_required: 3,
        smartcore_shadow_scenario_emit_owner_summary: true,
        smartcore_observer_lane_enabled: false,
        smartcore_observer_output_path: None,
        smartcore_observer_compare_member_opinion: true,
        smartcore_observer_compare_chairman: true,
        smartcore_observer_compare_risk_governor: true,
        smartcore_observer_target_coverage_closure_enabled: true,
        smartcore_observer_emit_owner_section: true,
        observer_target_closure_enabled: false,
        observer_target_closure_dry_run: true,
        observer_target_closure_output_path: None,
        observer_target_set_output_path: None,
        observer_comparison_ledger_path: None,
        observer_readiness_hardening_enabled: false,
        observer_coverage_closure_emit_owner_summary: true,
        observer_target_apply_trend_enabled: false,
        observer_target_apply_dry_run: true,
        observer_target_apply_targets: false,
        observer_target_store_input_path: None,
        observer_target_store_output_path: None,
        observer_ledger_trend_enabled: true,
        observer_readiness_v2_enabled: false,
        observer_trend_summary_enabled: false,
        observer_apply_trend_output_path: None,
        observer_seed_apply_trend_enabled: false,
        observer_seed_apply_dry_run: true,
        observer_seed_apply_targets: false,
        observer_seed_target_store_output_path: None,
        observer_seed_apply_output_path: None,
        observer_seed_require_approved_target: true,
        observer_seed_rerun_comparison: true,
        observer_seed_compute_ledger_trend: true,
        observer_seed_recheck_readiness: true,
        observer_seed_emit_owner_summary: true,
        observer_approved_apply_governance_enabled: false,
        observer_approved_apply_mode: core::ObserverExplicitApplyMode::DryRun,
        observer_approved_apply_dry_run: true,
        observer_approved_target_store_input_path: None,
        observer_approved_target_store_output_path: None,
        observer_approved_apply_output_path: None,
        observer_approved_apply_recheck_readiness: true,
        chairman_governance_contract_prepare_enabled: true,
        chairman_governance_readiness_check_enabled: true,
        observer_approved_apply_emit_owner_summary: true,
        observer_apply_verify_chairman_shadow_enabled: false,
        observer_apply_verify_mode: core::ObserverExplicitApplyMode::DryRun,
        observer_apply_verify_dry_run: true,
        observer_apply_verify_target_store_output_path: None,
        observer_apply_verify_output_path: None,
        observer_apply_verify_config_path: None,
        observer_apply_verify_emit_owner_summary: true,
        chairman_shadow_governance_enabled: true,
        training_candidate_min_examples: None,
        self_growing_replay_enabled: false,
        research_source_registry_path: None,
        self_growing_max_tasks: 16,
        self_growing_max_evidence_records: 32,
        self_growing_allow_network_sources: false,
        research_evidence_output_path: None,
        self_growing_replay_output_path: None,
        emit_research_task_summary: false,
        self_growing_evidence_staging_enabled: false,
        self_growing_evidence_promotion_enabled: false,
        self_growing_evidence_promotion_dry_run: true,
        self_growing_evidence_apply_promotions: false,
        self_growing_refresh_training_candidates: false,
        self_growing_staging_store_path: None,
        self_growing_approved_evidence_output_path: None,
        self_growing_training_candidate_output_path: None,
        enriched_evidence_promotion_enabled: false,
        enriched_evidence_promotion_dry_run: true,
        enriched_evidence_apply_patch: false,
        enriched_evidence_apply_promotions: false,
        enriched_evidence_refresh_training_candidates: false,
        enriched_staging_output_path: None,
        enriched_approved_evidence_output_path: None,
        enriched_training_candidate_output_path: None,
        auto_approval_e2e_enabled: false,
        auto_approval_e2e_dry_run: true,
        auto_approval_success_staging_path: None,
        auto_approval_success_price_series_path: None,
        auto_approval_apply_promotions: false,
        auto_approval_refresh_training_candidates: false,
        auto_approval_approved_evidence_output_path: None,
        auto_approval_training_candidate_output_path: None,
        autonomous_paper_run: true,
        run_id: Some("remote-reject".to_string()),
        market_scopes: Vec::new(),
        symbols: Vec::new(),
        max_cycles: 1,
        cycle_mode: AutonomousPaperCycleMode::SingleShot,
        require_owner_confirmation: OwnerConfirmationPolicy::Never,
        local_market_data_path: Some("https://example.invalid/market.json".to_string()),
        local_news_path: None,
        news_collection_enabled: false,
        news_collection_config_path: None,
        news_provider_config_path: None,
        research_run_enabled: false,
        emit_research_run_summary: false,
        emit_research_packet_summary: false,
        research_auto_run_enabled: false,
        news_cache_input_path: None,
        news_cache_output_path: None,
        news_network_mode: NewsProviderRunMode::OfflineOnly,
        news_fetch_policy: None,
        rss_xml_fixture_path: None,
        rss_fetch_pilot_enabled: false,
        rss_fetch_pilot_url: None,
        rss_fetch_allowed_domains: Vec::new(),
        rss_fetch_source_label: None,
        rss_network_enabled: false,
        rss_safe_http_timeout_ms: 3_000,
        rss_safe_http_rate_limit_ms: 1_000,
        rss_safe_http_max_response_bytes: 262_144,
        rss_allowed_content_types: vec![
            "application/rss+xml".to_string(),
            "application/xml".to_string(),
            "text/xml".to_string(),
            "application/atom+xml".to_string(),
        ],
        rss_allow_redirects: false,
        rss_allow_missing_content_type: false,
        run_committee_cycle_from_research_packets: false,
        paper_only: true,
        owner_attention_inbox_input_path: None,
        owner_attention_inbox_output_path: None,
        owner_attention_actions_path: None,
        watchlist_candidate_input_path: None,
        watchlist_candidate_output_path: None,
        emit_owner_attention_inbox: false,
        enable_watchlist_recheck: false,
        watchlist_input_path: None,
        watchlist_output_path: None,
        max_candidates_per_cycle: 3,
        include_risk_blocked: false,
        include_needs_evidence: true,
        emit_owner_daily_brief: false,
        owner_daily_brief_store_input_path: None,
        owner_daily_brief_store_output_path: None,
        committee_state_snapshot_output_path: None,
        emit_committee_state_snapshot: false,
        committee_state_export_root_path: None,
        write_latest_snapshot: false,
        write_history_snapshot: false,
        write_snapshot_index: false,
        write_owner_console_read_model: false,
        committee_state_schema_version: None,
        max_snapshot_history_entries: None,
        inline_offline_member_opinions: Vec::new(),
        inline_input: None,
        pilot_roster: None,
        paper_outcome: None,
        archetype_style_cards_path: None,
        style_mapping_mode: StyleMappingMode::None,
    };
    let err = config
        .validate()
        .expect_err("remote autonomous market path must fail");
    assert!(err.contains("local_market_data_path must be local"));

    let unsafe_config_path = std::path::Path::new("target/sprint135_unsafe_autonomous.toml");
    std::fs::write(
        unsafe_config_path,
        r#"
input_path = "examples/minimal_ai_committee_multi_market_sample.json"
batch_mode = true
autonomous_paper_run = true
run_id = "unsafe-autonomous-test"
max_cycles = 1
broker = "not allowed"
"#,
    )
    .expect("write unsafe autonomous config");
    let err = MinimalAiCommitteeCycleConfig::from_toml_path(unsafe_config_path)
        .expect_err("unsafe autonomous config must fail");
    assert!(err.contains("unsafe field or instruction"));
    let _ = std::fs::remove_file(unsafe_config_path);
}

#[test]
fn event_queue_groups_and_risk_first_ordering_are_deterministic() {
    let opinions = vec![
        OfflineMemberOpinionFixture {
            member_id: "trend-kr-short".to_string(),
            symbol: "005930.KS".to_string(),
            market_scope: MarketScope::KoreaShortTerm,
            stance: MemberStance::BuyProposal,
            confidence: 0.91,
            expected_return_hint: 0.03,
            risk_hint: 0.04,
            evidence_notes: vec!["paper-only event queue test".to_string()],
            event_triggered: true,
            event_reason: Some("entry".to_string()),
        },
        OfflineMemberOpinionFixture {
            member_id: "risk-kr-short".to_string(),
            symbol: "005930.KS".to_string(),
            market_scope: MarketScope::KoreaShortTerm,
            stance: MemberStance::NoTrade,
            confidence: 0.72,
            expected_return_hint: 0.0,
            risk_hint: 0.3,
            evidence_notes: vec!["paper-only event queue test".to_string()],
            event_triggered: true,
            event_reason: Some("risk".to_string()),
        },
    ]
    .into_iter()
    .map(|fixture| OfflineMemberBrainAdapter {
        fixtures: vec![fixture],
    })
    .map(|adapter| {
        let fixture = adapter.fixtures.first().expect("fixture");
        soma_zero::league::minimal_ai_committee_core::MemberOpinion {
            member_id: fixture.member_id.clone(),
            symbol: fixture.symbol.clone(),
            market_scope: fixture.market_scope,
            stance: fixture.stance,
            confidence: fixture.confidence,
            expected_return_hint: fixture.expected_return_hint,
            risk_hint: fixture.risk_hint,
            evidence_notes: fixture.evidence_notes.clone(),
            event_triggered: fixture.event_triggered,
            event_reason: fixture.event_reason.clone(),
        }
    })
    .collect::<Vec<_>>();
    let queue = InvestmentEventQueue::from_member_opinions(&opinions);
    assert_eq!(queue.event_count, 2);
    assert_eq!(
        queue.events.first().expect("risk first").event_type,
        soma_zero::league::minimal_ai_committee_core::InvestmentEventType::RiskWarning
    );
    assert_eq!(
        queue
            .group_by_symbol()
            .get("005930.KS")
            .expect("symbol group")
            .len(),
        2
    );
    assert_eq!(
        queue
            .group_by_market_scope()
            .get(&MarketScope::KoreaShortTerm)
            .expect("scope group")
            .len(),
        2
    );
}

#[test]
fn three_member_pilot_roster_has_independent_roles_memory_and_core_specs() {
    let roster = create_three_member_pilot_roster(MarketScope::KoreaShortTerm);
    assert_eq!(roster.len(), 3);
    assert_eq!(
        roster
            .iter()
            .filter_map(|member| member.role)
            .collect::<Vec<_>>(),
        vec![
            IndependentMemberRole::TrendEntry,
            IndependentMemberRole::RiskGuard,
            IndependentMemberRole::EvidenceRegime
        ]
    );
    for member in roster {
        assert_eq!(member.market_scopes, vec![MarketScope::KoreaShortTerm]);
        let core_spec = member.core_spec.as_ref().expect("core spec");
        assert_eq!(core_spec.core_family, MemberCoreFamily::Mamba3GatedDeltaNet);
        assert_eq!(core_spec.sequence_core, SequenceCoreKind::Mamba3Deferred);
        assert_eq!(core_spec.memory_core, MemoryCoreKind::GatedDeltaNetDeferred);
        let memory = member.memory_state.expect("memory state");
        assert_eq!(memory.member_id, member.member_id);
        assert_eq!(memory.recent_opinion_count, 0);
        assert!(
            memory
                .notes
                .iter()
                .any(|note| note.contains("no model weight update"))
        );
    }

    let roster = create_three_member_pilot_roster(MarketScope::KoreaShortTerm);
    let output = route_data_to_ai_members(DataRouterInput {
        market_data: vec![
            serde_json::from_value(serde_json::json!({
                "symbol": "005930.KS",
                "market_scope": "KoreaShortTerm",
                "timestamp": "2026-05-21T09:00:00+09:00",
                "price": 78000.0,
                "change_pct": 1.0,
                "volume": 1000.0,
                "volatility_hint": 0.04,
                "source_label": "test"
            }))
            .expect("market data"),
        ],
        news: Vec::new(),
        members: roster.clone(),
        owner_context: Some("paper-only".to_string()),
    });
    assert_eq!(output.packets.len(), 3);

    let evidence_member = roster
        .iter()
        .find(|member| member.role == Some(IndependentMemberRole::EvidenceRegime))
        .expect("evidence member")
        .clone();
    let evidence_packet = MemberInputPacket {
        member_id: evidence_member.member_id.clone(),
        market_data: serde_json::from_value(serde_json::json!({
            "symbol": "005930.KS",
            "market_scope": "KoreaShortTerm",
            "timestamp": "2026-05-21T09:00:00+09:00",
            "price": 78000.0,
            "change_pct": 1.0,
            "volume": 1000.0,
            "volatility_hint": 0.04,
            "source_label": "test"
        }))
        .expect("market data"),
        news: vec![
            serde_json::from_value(serde_json::json!({
                "symbol": "005930.KS",
                "headline": "unclear evidence",
                "summary": "unknown regime evidence",
                "sentiment_hint": "unknown",
                "source_label": "test",
                "timestamp": "2026-05-21T09:00:00+09:00"
            }))
            .expect("news"),
        ],
        owner_context: Some("paper-only".to_string()),
        previous_member_score: Some(evidence_member.score),
    };
    let evidence_opinion = DeterministicMockBrain {
        member: evidence_member,
    }
    .produce_opinion(&evidence_packet);
    assert_eq!(evidence_opinion.stance, MemberStance::NeedMoreEvidence);
    assert!(evidence_opinion.event_triggered);
}

#[test]
fn archetype_style_cards_load_validate_and_map_to_three_members() {
    let registry = ArchetypeStyleCardRegistry::load_style_cards_from_local_fixture(
        std::path::Path::new("examples/investor_archetype_style_cards.sample.json"),
    )
    .expect("load style cards");
    assert_eq!(registry.cards.len(), 18);
    assert!(registry.active_count >= 15);
    assert!(registry.review_required_count >= 3);
    registry
        .validate_no_impersonation()
        .expect("no impersonation wording");
    registry
        .validate_do_not_learn_guards()
        .expect("do-not-learn guards");
    assert!(
        registry
            .cards_for_role(IndependentMemberRole::TrendEntry)
            .iter()
            .any(|card| card
                .primary_style_tags
                .iter()
                .any(|tag| format!("{:?}", tag) == "Momentum"))
    );
    assert!(
        registry
            .cards_for_role(IndependentMemberRole::RiskGuard)
            .iter()
            .any(|card| card.risk_bias
                == soma_zero::league::minimal_ai_committee_core::ArchetypeRiskBias::Conservative)
    );
    assert!(
        registry
            .cards_for_role(IndependentMemberRole::EvidenceRegime)
            .iter()
            .any(|card| format!("{:?}", card.evidence_preference) == "Fundamentals")
    );

    let mapping = map_style_cards_to_three_member_pilot(&registry);
    for blend in [
        &mapping.trend_entry_blend,
        &mapping.risk_guard_blend,
        &mapping.evidence_regime_blend,
    ] {
        let weight_sum: f64 = blend
            .archetype_weights
            .iter()
            .map(|weight| weight.weight)
            .sum();
        assert!((weight_sum - 1.0).abs() < 0.0001);
        assert!(
            blend
                .prohibited_claims
                .contains(&"not a real person clone".to_string())
        );
    }
    assert!(
        mapping
            .review_required_archetypes
            .contains(&"archetype-16".to_string())
    );
    assert_eq!(
        mapping.trend_entry_blend.source_confidence_minimum,
        SourceConfidence::ReviewRequired
    );
    assert_eq!(
        mapping.trend_entry_blend.style_status,
        MemberStyleStatus::ReadyWithWarnings
    );
}

#[test]
fn review_required_style_cards_do_not_silently_upgrade_confidence() {
    let guard = vec![
        "private_life_details".to_string(),
        "exact_personality_clone".to_string(),
        "unverified_profit_claims".to_string(),
        "unsourced_quotes".to_string(),
        "illegal_or_private_info".to_string(),
    ];
    let cards = vec![
        InvestorArchetypeStyleCard {
            archetype_id: "safe-high".to_string(),
            display_name: "Safe high confidence public trend card".to_string(),
            public_style_summary:
                "Inspired by public investment philosophy; style influence only; not a real person clone."
                    .to_string(),
            preferred_time_horizon: PreferredTimeHorizon::ShortTerm,
            preferred_market_bias: PreferredMarketBias::Any,
            primary_style_tags: vec![ArchetypeStyleTag::Trend],
            risk_bias: ArchetypeRiskBias::Balanced,
            evidence_preference: EvidencePreference::PriceAction,
            do_not_learn: guard.clone(),
            source_confidence: SourceConfidence::High,
            status: StyleCardStatus::ActiveStyleCard,
        },
        InvestorArchetypeStyleCard {
            archetype_id: "review-required".to_string(),
            display_name: "Review required public momentum card".to_string(),
            public_style_summary:
                "Inspired by public investment philosophy; style influence only; not a real person clone."
                    .to_string(),
            preferred_time_horizon: PreferredTimeHorizon::ShortTerm,
            preferred_market_bias: PreferredMarketBias::Any,
            primary_style_tags: vec![ArchetypeStyleTag::Momentum],
            risk_bias: ArchetypeRiskBias::Aggressive,
            evidence_preference: EvidencePreference::PriceAction,
            do_not_learn: guard,
            source_confidence: SourceConfidence::ReviewRequired,
            status: StyleCardStatus::ReviewRequired,
        },
    ];
    let registry = ArchetypeStyleCardRegistry::from_cards(cards);
    let mapping = map_style_cards_to_three_member_pilot(&registry);
    let weights = &mapping.trend_entry_blend.archetype_weights;
    let review_weight = weights
        .iter()
        .find(|weight| weight.archetype_id == "review-required")
        .expect("review card participates with limited weight");

    assert_eq!(
        mapping.trend_entry_blend.source_confidence_minimum,
        SourceConfidence::ReviewRequired
    );
    assert_eq!(
        mapping.trend_entry_blend.style_status,
        MemberStyleStatus::ReadyWithWarnings
    );
    assert!(review_weight.weight < 0.2);
    assert!((weights.iter().map(|weight| weight.weight).sum::<f64>() - 1.0).abs() < 0.0001);
}

#[test]
fn archetype_style_card_rejects_impersonation_wording() {
    let text = std::fs::read_to_string("examples/investor_archetype_style_cards.sample.json")
        .expect("style fixture");
    let mut cards: Vec<InvestorArchetypeStyleCard> =
        serde_json::from_str(&text).expect("style cards");
    cards[0].display_name = "Warren Buffett AI".to_string();
    let registry = ArchetypeStyleCardRegistry::from_cards(cards);
    let err = registry
        .validate_no_impersonation()
        .expect_err("impersonation wording should fail");
    assert!(err.contains("impersonation"));
}

#[test]
fn real_archetype_intake_rejects_guaranteed_returns_and_preserves_review_status() {
    let text = std::fs::read_to_string("examples/investor_archetype_style_cards.sample.json")
        .expect("style fixture");
    let mut cards: Vec<InvestorArchetypeStyleCard> =
        serde_json::from_str(&text).expect("style cards");
    cards[0].public_style_summary =
        "Inspired by public investment philosophy; guaranteed return; not a real person clone."
            .to_string();
    let registry = ArchetypeStyleCardRegistry::from_cards(cards);
    let policy = RealArchetypeIntakePolicy::default();
    let err = policy
        .validate_registry(&registry)
        .expect_err("guaranteed return claim should fail");
    assert!(err.contains("guaranteed-return"));

    let text = std::fs::read_to_string("examples/investor_archetype_style_cards.sample.json")
        .expect("style fixture");
    let mut cards: Vec<InvestorArchetypeStyleCard> =
        serde_json::from_str(&text).expect("style cards");
    cards[0].public_style_summary =
        "Inspired by public investment philosophy; uses private strategy; not a real person clone."
            .to_string();
    let registry = ArchetypeStyleCardRegistry::from_cards(cards);
    let err = policy
        .validate_registry(&registry)
        .expect_err("private strategy claim should fail");
    assert!(err.contains("private-strategy"));

    let text = std::fs::read_to_string("examples/investor_archetype_style_cards.sample.json")
        .expect("style fixture");
    let mut cards: Vec<InvestorArchetypeStyleCard> =
        serde_json::from_str(&text).expect("style cards");
    let review_card = cards
        .iter_mut()
        .find(|card| card.source_confidence == SourceConfidence::ReviewRequired)
        .expect("review card");
    review_card.status = StyleCardStatus::ActiveStyleCard;
    let registry = ArchetypeStyleCardRegistry::from_cards(cards);
    let err = policy
        .validate_registry(&registry)
        .expect_err("review-required card should stay review-required");
    assert!(err.contains("ReviewRequired status"));
}

#[test]
fn real_archetype_intake_uses_local_json_only() {
    let policy = RealArchetypeIntakePolicy::default();
    let registry = policy
        .load_registry_from_local_json(std::path::Path::new(
            "examples/investor_archetype_style_cards.sample.json",
        ))
        .expect("local style fixture");
    assert_eq!(registry.cards.len(), 18);

    let err = policy
        .load_registry_from_local_json(std::path::Path::new("https://example.invalid/cards.json"))
        .expect_err("remote style cards must fail");
    assert!(err.contains("local JSON"));
}

#[test]
fn member_core_registry_limits_18_members_and_keeps_risk_member() {
    let members: Vec<_> = (0..18)
        .map(|index| {
            let member_id = if index == 17 {
                "risk-member".to_string()
            } else {
                format!("trend-member-{index:02}")
            };
            AICommitteeMember {
                member_id: member_id.clone(),
                display_name: member_id.clone(),
                market_scopes: vec![MarketScope::KoreaShortTerm],
                style_profile: if index == 17 {
                    "risk".to_string()
                } else {
                    "trend".to_string()
                },
                voice_weight: if index == 17 {
                    0.1
                } else {
                    0.95 - index as f64 * 0.02
                },
                score: 0.6,
                status: AICommitteeMemberStatus::Active,
                runtime_mode: AIRuntimeMode::MockLocal,
                core_spec: Some(Mamba3GatedDeltaNetCoreSpec::mock_local_for(&member_id)),
                role: None,
                memory_state: None,
            }
        })
        .collect();
    assert!(members.iter().all(|member| {
        member
            .core_spec
            .as_ref()
            .is_some_and(|spec| spec.runtime_status == CoreRuntimeStatus::MockLocal)
    }));

    let registry = AiMemberCoreRegistry::from_members(&members);
    let policy = MemberActivationPolicy::default();
    let plan =
        registry.select_members_for_cycle(&members, MarketScope::KoreaShortTerm, None, &policy);

    assert_eq!(market_committee_layouts().len(), 6);
    assert!(plan.selected_member_ids.len() <= policy.max_active_members_per_cycle);
    assert!(plan.selected_member_ids.len() <= policy.max_active_members_per_market_scope);
    assert_ne!(plan.selected_member_ids.len(), 18);
    assert!(
        plan.selected_member_ids
            .contains(&"risk-member".to_string())
    );
    assert!(
        plan.skipped_members
            .iter()
            .any(|skip| { skip.reason == MemberSelectionSkipReason::OverActivationLimit })
    );
    assert!(
        plan.policy_notes
            .iter()
            .any(|note| note.contains("do not run 18 AI cores concurrently"))
    );

    let mut mixed_scope_members = members[..4].to_vec();
    mixed_scope_members[0].market_scopes = vec![MarketScope::KoreaLongTerm];
    let mixed_registry = AiMemberCoreRegistry::from_members(&mixed_scope_members);
    let scoped_policy = MemberActivationPolicy {
        max_active_members_per_cycle: 5,
        max_active_members_per_market_scope: 1,
        ..MemberActivationPolicy::default()
    };
    assert_eq!(
        mixed_registry.active_core_count(&scoped_policy, MarketScope::KoreaShortTerm),
        1
    );
    assert_eq!(
        mixed_registry.active_core_count(&scoped_policy, MarketScope::KoreaLongTerm),
        1
    );
    assert_eq!(
        mixed_registry.estimate_cycle_memory_mb(&scoped_policy, MarketScope::UsLongTerm),
        0
    );
}

#[test]
fn core_aware_brain_respects_deferred_offline_and_mock_modes() {
    let member = AICommitteeMember {
        member_id: "core-member".to_string(),
        display_name: "core member".to_string(),
        market_scopes: vec![MarketScope::KoreaShortTerm],
        style_profile: "trend".to_string(),
        voice_weight: 0.7,
        score: 0.6,
        status: AICommitteeMemberStatus::Active,
        runtime_mode: AIRuntimeMode::MockLocal,
        core_spec: None,
        role: Some(IndependentMemberRole::TrendEntry),
        memory_state: None,
    };
    let packet = MemberInputPacket {
        member_id: member.member_id.clone(),
        market_data: serde_json::from_value(serde_json::json!({
            "symbol": "005930.KS",
            "market_scope": "KoreaShortTerm",
            "timestamp": "2026-05-21T09:00:00+09:00",
            "price": 78000.0,
            "change_pct": 4.2,
            "volume": 1000.0,
            "volatility_hint": 0.03,
            "source_label": "test"
        }))
        .expect("market data"),
        news: vec![
            serde_json::from_value(serde_json::json!({
                "symbol": "005930.KS",
                "headline": "positive",
                "summary": "positive",
                "sentiment_hint": "positive",
                "source_label": "test",
                "timestamp": "2026-05-21T09:00:00+09:00"
            }))
            .expect("news"),
        ],
        owner_context: Some("paper-only".to_string()),
        previous_member_score: Some(0.6),
    };
    let offline_adapter = OfflineMemberBrainAdapter {
        fixtures: vec![OfflineMemberOpinionFixture {
            member_id: member.member_id.clone(),
            symbol: packet.market_data.symbol.clone(),
            market_scope: packet.market_data.market_scope,
            stance: MemberStance::BuyProposal,
            confidence: 0.8,
            expected_return_hint: 0.03,
            risk_hint: 0.04,
            evidence_notes: vec!["offline fixture opinion".to_string()],
            event_triggered: true,
            event_reason: Some("offline fixture".to_string()),
        }],
    };

    let offline = CoreAwareMemberBrainAdapter {
        member: member.clone(),
        core_spec: Mamba3GatedDeltaNetCoreSpec::offline_fixture_for(&member.member_id),
        offline_adapter: offline_adapter.clone(),
    }
    .produce_opinion(&packet);
    assert_eq!(offline.stance, MemberStance::BuyProposal);

    let mock = CoreAwareMemberBrainAdapter {
        member: member.clone(),
        core_spec: Mamba3GatedDeltaNetCoreSpec::mock_local_for(&member.member_id),
        offline_adapter: offline_adapter.clone(),
    }
    .produce_opinion(&packet);
    assert_eq!(mock.stance, MemberStance::BuyProposal);
    assert!(
        mock.evidence_notes
            .iter()
            .any(|note| note.contains("deterministic mock"))
    );

    let runtime_deferred = CoreAwareMemberBrainAdapter {
        member: member.clone(),
        core_spec: Mamba3GatedDeltaNetCoreSpec::runtime_deferred_for(&member.member_id),
        offline_adapter: offline_adapter.clone(),
    }
    .produce_opinion(&packet);
    assert_eq!(runtime_deferred.stance, MemberStance::NeedMoreEvidence);
    assert_eq!(
        runtime_deferred.event_reason.as_deref(),
        Some("runtime deferred")
    );

    let training_deferred = CoreAwareMemberBrainAdapter {
        member,
        core_spec: Mamba3GatedDeltaNetCoreSpec::training_deferred_for("core-member"),
        offline_adapter,
    }
    .produce_opinion(&packet);
    assert_eq!(training_deferred.stance, MemberStance::NeedMoreEvidence);
    assert_eq!(
        training_deferred.event_reason.as_deref(),
        Some("training deferred")
    );
}

#[test]
fn data_router_routes_all_market_scopes_without_creating_opinions() {
    let text = std::fs::read_to_string("examples/minimal_ai_committee_multi_market_sample.json")
        .expect("multi-market sample");
    let input: DataRouterInput = serde_json::from_str(&text).expect("data router input");
    let market_count = input.market_data.len();
    let member_count = input.members.len();
    let output = route_data_to_ai_members(input.clone());

    assert_eq!(market_count, 6);
    assert_eq!(member_count, 7);
    assert_eq!(output.unrouted_symbol_count, 0);
    assert_eq!(output.routed_member_count, 6);
    assert_eq!(output.packets.len(), 6);
    assert!(
        output
            .safety_notes
            .iter()
            .any(|note| note.contains("does not create opinions"))
    );
    assert!(
        output
            .safety_notes
            .iter()
            .any(|note| note.contains("AI members judge"))
    );
    assert!(output.safety_notes.iter().any(|note| {
        let note = note.to_ascii_lowercase();
        note.contains("does not create") && note.contains("recommendations")
    }));

    let routed_scopes: std::collections::BTreeSet<_> = output
        .packets
        .iter()
        .map(|packet| format!("{:?}", packet.market_data.market_scope))
        .collect();
    assert_eq!(
        routed_scopes,
        [
            "KoreaShortTerm",
            "KoreaLongTerm",
            "UsShortTerm",
            "UsLongTerm",
            "CryptoShortTerm",
            "CryptoLongTerm",
        ]
        .into_iter()
        .map(str::to_string)
        .collect()
    );
    assert!(
        !output
            .packets
            .iter()
            .any(|packet| packet.member_id == "disabled-all-market")
    );
    for packet in &output.packets {
        let member = input
            .members
            .iter()
            .find(|member| member.member_id == packet.member_id)
            .expect("packet member exists");
        assert!(
            member
                .market_scopes
                .contains(&packet.market_data.market_scope)
        );
        assert!(
            packet
                .news
                .iter()
                .all(|news| news.symbol == packet.market_data.symbol)
        );
    }

    let first_packet = output
        .packets
        .iter()
        .find(|packet| packet.member_id == "kr-short-trend")
        .expect("kr short packet");
    let offline_adapter = OfflineMemberBrainAdapter {
        fixtures: vec![OfflineMemberOpinionFixture {
            member_id: first_packet.member_id.clone(),
            symbol: first_packet.market_data.symbol.clone(),
            market_scope: first_packet.market_data.market_scope,
            stance: MemberStance::BuyProposal,
            confidence: 0.71,
            expected_return_hint: 0.02,
            risk_hint: first_packet.market_data.volatility_hint,
            evidence_notes: vec!["offline fixture opinion".to_string()],
            event_triggered: true,
            event_reason: Some("router-fed offline fixture".to_string()),
        }],
    };
    let offline_opinion = offline_adapter.produce_opinion(first_packet);
    assert_eq!(offline_opinion.member_id, "kr-short-trend");
    assert_eq!(offline_opinion.stance, MemberStance::BuyProposal);
    assert!(offline_opinion.event_triggered);
    assert!(
        offline_opinion
            .evidence_notes
            .contains(&"offline fixture opinion".to_string())
    );

    let member = input
        .members
        .iter()
        .find(|member| member.member_id == first_packet.member_id)
        .expect("matching member")
        .clone();
    let opinion = DeterministicMockBrain { member }.produce_opinion(first_packet);
    assert_eq!(opinion.member_id, "kr-short-trend");
    assert_eq!(opinion.stance, MemberStance::BuyProposal);
    assert!(opinion.event_triggered);
}

#[test]
fn owner_natural_input_parses_to_internal_feedback_and_action_json() {
    let parsed = parse_owner_natural_input(OwnerNaturalInput {
        input_id: "test-owner-natural-evidence".to_string(),
        text: "005930.KS 근거가 부족해 보여. 다시 봐줘".to_string(),
        symbol: Some("005930.KS".to_string()),
        market_scope: Some(MarketScope::KoreaShortTerm),
        target_member_id: Some("value-member".to_string()),
        target_item_id: None,
        source_label: Some("owner-cli".to_string()),
        created_at: Some("2026-05-23T09:00:00+09:00".to_string()),
        paper_only: true,
    })
    .expect("parse natural owner input");

    assert_eq!(parsed.input_id, "test-owner-natural-evidence");
    assert_eq!(
        parsed.detected_intent,
        OwnerNaturalInputIntent::EvidenceRequest
    );
    assert_eq!(parsed.intent, OwnerNaturalInputIntent::EvidenceRequest);
    assert_eq!(
        parsed.safety_status,
        OwnerAttentionActionSafetyStatus::Passed
    );
    assert!(parsed.rejection_reason.is_none());
    assert!(parsed.paper_only);
    assert!(parsed.generated_feedback.is_some());
    assert!(parsed.generated_attention_action.is_some());
    assert_eq!(
        parsed.feedback.feedback_type,
        OwnerFeedbackType::EvidenceRequest
    );
    assert_eq!(parsed.feedback.symbol.as_deref(), Some("005930.KS"));
    assert_eq!(
        parsed.internal_action_file.actions[0].action_type,
        OwnerAttentionActionType::RequestMoreEvidence
    );
    assert_eq!(
        parsed.internal_action_file.actions[0].item_id,
        "owner-general-comment"
    );
    assert!(parsed.internal_action_file.paper_only);
    assert!(
        parsed
            .safety_notes
            .iter()
            .any(|note| note.contains("no LLM"))
    );
    let parse_case = |text: &str, intent: OwnerNaturalInputIntent| {
        let parsed = parse_owner_natural_input(OwnerNaturalInput {
            input_id: format!("case-{intent:?}"),
            text: text.to_string(),
            symbol: Some("005930.KS".to_string()),
            market_scope: Some(MarketScope::KoreaShortTerm),
            target_member_id: None,
            target_item_id: None,
            source_label: Some("unit-test".to_string()),
            created_at: None,
            paper_only: true,
        })
        .expect("parse owner natural input case");
        assert_eq!(parsed.detected_intent, intent);
    };
    parse_case(
        "리스크와 변동성이 걱정돼",
        OwnerNaturalInputIntent::RiskConcern,
    );
    parse_case(
        "관심종목으로 지켜봐",
        OwnerNaturalInputIntent::WatchlistRequest,
    );
    parse_case(
        "위원회 다시 재검토해줘",
        OwnerNaturalInputIntent::ReconsiderationRequest,
    );
    parse_case(
        "paper positive 결과 좋음",
        OwnerNaturalInputIntent::PaperOutcomeLabel,
    );
    parse_case("확인 메모", OwnerNaturalInputIntent::Comment);
    parse_case(
        "분류하기 어려운 일반 문장",
        OwnerNaturalInputIntent::Unknown,
    );
    for unsafe_text in ["계좌에서 주문해", "레버리지 최대로", "수익 보장"] {
        let err = parse_owner_natural_input(OwnerNaturalInput {
            input_id: "unsafe-owner-natural-input".to_string(),
            text: unsafe_text.to_string(),
            symbol: Some("005930.KS".to_string()),
            market_scope: Some(MarketScope::KoreaShortTerm),
            target_member_id: None,
            target_item_id: None,
            source_label: Some("unit-test".to_string()),
            created_at: None,
            paper_only: true,
        })
        .expect_err("unsafe natural owner input rejected");
        assert!(err.contains("owner policy rejected"));
    }

    let output_path = std::path::Path::new("target/sprint146_owner_say_action.json");
    let result = write_owner_natural_input_action_file(
        OwnerNaturalInput {
            input_id: "test-owner-natural-write".to_string(),
            text: "리스크 근거를 더 모아줘".to_string(),
            symbol: Some("005930.KS".to_string()),
            market_scope: Some(MarketScope::KoreaShortTerm),
            target_member_id: None,
            target_item_id: None,
            source_label: Some("owner-cli".to_string()),
            created_at: None,
            paper_only: true,
        },
        output_path,
        false,
    )
    .expect("write internal action file");
    assert!(result.wrote_output);
    let saved: OwnerActionFile = serde_json::from_str(
        &std::fs::read_to_string(output_path).expect("read owner natural action file"),
    )
    .expect("saved action file JSON");
    assert_eq!(saved.actions.len(), 1);
    assert!(
        saved
            .safety_notes
            .iter()
            .any(|note| note.contains("owner did not write JSON"))
    );
    assert!(
        saved
            .safety_notes
            .iter()
            .any(|note| note.contains("internal"))
    );
    let _ = std::fs::remove_file(output_path);
}

#[test]
fn owner_intent_policy_table_loads_prioritizes_and_rejects_safely() {
    let default_load = default_owner_intent_policy_load_result();
    assert!(!default_load.loaded);
    assert!(default_load.rule_count > 0);
    assert!(default_load.safety_rule_count > 0);
    assert!(default_load.policy.paper_only);

    let built_in = built_in_owner_intent_policy();
    for (text, intent) in [
        (
            "삼성전자 뉴스 근거가 부족해 보여. 다시 확인해줘",
            OwnerNaturalInputIntent::EvidenceRequest,
        ),
        (
            "리스크와 변동성이 걱정돼",
            OwnerNaturalInputIntent::RiskConcern,
        ),
        (
            "관심종목으로 지켜봐",
            OwnerNaturalInputIntent::WatchlistRequest,
        ),
        (
            "위원회 다시 재검토해줘",
            OwnerNaturalInputIntent::ReconsiderationRequest,
        ),
    ] {
        let parsed = parse_owner_natural_input_with_policy(
            OwnerNaturalInput {
                input_id: format!("policy-case-{intent:?}"),
                text: text.to_string(),
                symbol: Some("005930.KS".to_string()),
                market_scope: Some(MarketScope::KoreaShortTerm),
                target_member_id: None,
                target_item_id: None,
                source_label: Some("unit-test".to_string()),
                created_at: None,
                paper_only: true,
            },
            &built_in,
        )
        .expect("built-in policy parses");
        assert_eq!(parsed.detected_intent, intent);
        assert!(
            parsed
                .safety_notes
                .iter()
                .any(|note| note.contains("owner intent policy"))
        );
    }

    for unsafe_text in [
        "계좌에서 주문해",
        "레버리지 최대로",
        "수익 보장",
        "API key secret",
        "미공개 정보로 판단해줘",
    ] {
        let err = parse_owner_natural_input_with_policy(
            OwnerNaturalInput {
                input_id: "policy-unsafe".to_string(),
                text: unsafe_text.to_string(),
                symbol: Some("005930.KS".to_string()),
                market_scope: Some(MarketScope::KoreaShortTerm),
                target_member_id: None,
                target_item_id: None,
                source_label: Some("unit-test".to_string()),
                created_at: None,
                paper_only: true,
            },
            &built_in,
        )
        .expect_err("unsafe text rejected by policy");
        assert!(err.contains("owner policy rejected"));
    }

    let policy_path = std::path::Path::new("target/sprint147_owner_intent_policy.json");
    let custom_policy = OwnerIntentPolicy {
        policy_id: "custom-priority-policy".to_string(),
        language: OwnerIntentPolicyLanguage::Mixed,
        intent_rules: vec![
            OwnerIntentRule {
                rule_id: "low-risk".to_string(),
                intent: OwnerNaturalInputIntent::RiskConcern,
                include_terms: vec!["다시".to_string()],
                exclude_terms: Vec::new(),
                priority: 1,
                confidence_hint: 0.51,
            },
            OwnerIntentRule {
                rule_id: "high-evidence".to_string(),
                intent: OwnerNaturalInputIntent::EvidenceRequest,
                include_terms: vec!["다시".to_string()],
                exclude_terms: Vec::new(),
                priority: 10,
                confidence_hint: 0.91,
            },
        ],
        safety_rules: vec![OwnerSafetyRule {
            rule_id: "block-secret".to_string(),
            blocked_category: OwnerSafetyBlockedCategory::SecretCredential,
            blocked_terms: vec!["secret".to_string()],
            severity: OwnerSafetyRuleSeverity::Reject,
            rejection_message: "custom policy rejected secret".to_string(),
        }],
        default_intent: OwnerNaturalInputIntent::Comment,
        paper_only: true,
    };
    std::fs::write(
        policy_path,
        serde_json::to_string_pretty(&custom_policy).expect("serialize custom policy"),
    )
    .expect("write custom owner policy");
    let loaded = load_owner_intent_policy_from_local_file(policy_path).expect("load policy");
    assert!(loaded.loaded);
    assert_eq!(loaded.rule_count, 2);
    let parsed = parse_owner_natural_input_with_policy(
        OwnerNaturalInput {
            input_id: "policy-priority".to_string(),
            text: "다시".to_string(),
            symbol: None,
            market_scope: None,
            target_member_id: None,
            target_item_id: None,
            source_label: Some("unit-test".to_string()),
            created_at: None,
            paper_only: true,
        },
        &loaded.policy,
    )
    .expect("priority policy parses");
    assert_eq!(
        parsed.detected_intent,
        OwnerNaturalInputIntent::EvidenceRequest
    );
    let toml_policy_path = std::path::Path::new("target/sprint147_owner_intent_policy.toml");
    std::fs::write(
        toml_policy_path,
        r#"
policy_id = "custom-toml-policy"
language = "Mixed"
default_intent = "Comment"
paper_only = true

[[intent_rules]]
rule_id = "toml-evidence"
intent = "EvidenceRequest"
include_terms = ["증빙"]
exclude_terms = []
priority = 90
confidence_hint = 0.93

[[safety_rules]]
rule_id = "toml-secret"
blocked_category = "SecretCredential"
blocked_terms = ["token"]
severity = "Reject"
rejection_message = "custom toml policy rejected token"
"#,
    )
    .expect("write toml owner policy");
    let toml_loaded =
        load_owner_intent_policy_from_local_file(toml_policy_path).expect("load toml policy");
    assert!(toml_loaded.loaded);
    let toml_parsed = parse_owner_natural_input_with_policy(
        OwnerNaturalInput {
            input_id: "policy-toml".to_string(),
            text: "증빙을 다시 확인".to_string(),
            symbol: None,
            market_scope: None,
            target_member_id: None,
            target_item_id: None,
            source_label: Some("unit-test".to_string()),
            created_at: None,
            paper_only: true,
        },
        &toml_loaded.policy,
    )
    .expect("toml policy parses");
    assert_eq!(
        toml_parsed.detected_intent,
        OwnerNaturalInputIntent::EvidenceRequest
    );
    let remote_err = load_owner_intent_policy_from_local_file(std::path::Path::new(
        "https://example.invalid/policy.json",
    ))
    .expect_err("remote owner policy path rejected");
    assert!(remote_err.contains("must be local"));
    let traversal_err =
        load_owner_intent_policy_from_local_file(std::path::Path::new("../policy.json"))
            .expect_err("traversal owner policy path rejected");
    assert!(traversal_err.contains("parent-directory traversal"));
    let _ = std::fs::remove_file(policy_path);
    let _ = std::fs::remove_file(toml_policy_path);
}

#[test]
fn automated_news_intake_normalizes_local_fixture_without_network() {
    let fixture_path = std::path::Path::new("target/sprint146_news_fixture.json");
    let items = vec![
        CollectedNewsItem {
            symbol: "005930.KS".to_string(),
            market_scope: Some(MarketScope::KoreaShortTerm),
            headline: "Samsung earnings growth beat".to_string(),
            summary: "Short local summary only; no copied article body.".to_string(),
            sentiment_hint: None,
            source_label: "local-fixture".to_string(),
            timestamp: "2026-05-23T09:01:00+09:00".to_string(),
            url: None,
            license_note: Some("headline and short summary fixture".to_string()),
        },
        CollectedNewsItem {
            symbol: "005930.KS".to_string(),
            market_scope: Some(MarketScope::KoreaShortTerm),
            headline: "Second item should be capped".to_string(),
            summary: "Capped by max_items_per_symbol.".to_string(),
            sentiment_hint: Some("neutral".to_string()),
            source_label: "local-fixture".to_string(),
            timestamp: "2026-05-23T09:02:00+09:00".to_string(),
            url: None,
            license_note: Some("headline and short summary fixture".to_string()),
        },
    ];
    std::fs::write(
        fixture_path,
        serde_json::to_string_pretty(&items).expect("serialize news fixture"),
    )
    .expect("write news fixture");

    let result = collect_news_snapshots(&NewsCollectionConfig {
        source_mode: NewsCollectionSourceMode::LocalFixture,
        local_fixture_path: Some(fixture_path.display().to_string()),
        sources: Vec::new(),
        inline_items: Vec::new(),
        allow_network: false,
        allowed_domains: Vec::new(),
        max_items_per_symbol: 1,
        paper_only: true,
    })
    .expect("collect news snapshots");
    assert_eq!(result.collected_item_count, 2);
    assert_eq!(result.snapshot_count, 1);
    assert_eq!(result.news_snapshots[0].symbol, "005930.KS");
    assert_eq!(result.news_snapshots[0].sentiment_hint, "positive");
    assert!(
        result
            .safety_notes
            .iter()
            .any(|note| note.contains("no network access"))
    );

    let converted = convert_collected_news_to_snapshots(&items, 2);
    assert_eq!(converted.len(), 2);
    let remote_err = collect_news_snapshots(&NewsCollectionConfig {
        source_mode: NewsCollectionSourceMode::RssFeed,
        local_fixture_path: None,
        sources: vec![
            soma_zero::league::minimal_ai_committee_core::NewsSourceDescriptor {
                source_id: "remote-unallowed".to_string(),
                label: Some("remote".to_string()),
                mode: NewsCollectionSourceMode::RssFeed,
                path: None,
                url: Some("https://unapproved.example/feed.xml".to_string()),
                allowed_domain: None,
                paper_only: true,
            },
        ],
        inline_items: Vec::new(),
        allow_network: true,
        allowed_domains: vec!["approved.example".to_string()],
        max_items_per_symbol: 1,
        paper_only: true,
    })
    .expect_err("remote news source outside allowlist rejected");
    assert!(remote_err.contains("allowed domain"));
    let traversal_err = collect_news_snapshots(&NewsCollectionConfig {
        source_mode: NewsCollectionSourceMode::LocalFixture,
        local_fixture_path: Some("target/../bad_news.json".to_string()),
        sources: Vec::new(),
        inline_items: Vec::new(),
        allow_network: false,
        allowed_domains: Vec::new(),
        max_items_per_symbol: 1,
        paper_only: true,
    })
    .expect_err("news fixture traversal rejected");
    assert!(traversal_err.contains("parent-directory traversal"));
    let _ = std::fs::remove_file(fixture_path);
}

#[test]
fn news_provider_layer_collects_local_and_defers_remote_safely() {
    let fixture_path = std::path::Path::new("target/sprint147_provider_news_fixture.json");
    let items = vec![CollectedNewsItem {
        symbol: "005930.KS".to_string(),
        market_scope: Some(MarketScope::KoreaShortTerm),
        headline: "Samsung local headline".to_string(),
        summary: "Short provider summary only.".to_string(),
        sentiment_hint: Some("neutral".to_string()),
        source_label: "provider-fixture".to_string(),
        timestamp: "2026-05-23T09:03:00+09:00".to_string(),
        url: None,
        license_note: Some("headline and summary only".to_string()),
    }];
    std::fs::write(
        fixture_path,
        serde_json::to_string_pretty(&items).expect("serialize provider fixture"),
    )
    .expect("write provider fixture");

    let local = NewsProviderConfig {
        provider_id: "local-provider".to_string(),
        kind: NewsProviderKind::LocalFixture,
        enabled: true,
        source_path_or_url: Some(fixture_path.display().to_string()),
        source_label: "provider-fixture".to_string(),
        allowed_domains: Vec::new(),
        symbols: vec!["005930.KS".to_string()],
        market_scopes: vec![MarketScope::KoreaShortTerm],
        max_items: 3,
        timeout_ms: 100,
        trust_level: NewsProviderTrustLevel::ReviewRequired,
        paper_only: true,
    };
    let disabled_rss = NewsProviderConfig {
        provider_id: "rss-disabled".to_string(),
        kind: NewsProviderKind::RssFeed,
        enabled: false,
        source_path_or_url: Some("https://news.example/feed.xml".to_string()),
        source_label: "rss-disabled".to_string(),
        allowed_domains: vec!["news.example".to_string()],
        symbols: Vec::new(),
        market_scopes: Vec::new(),
        max_items: 3,
        timeout_ms: 100,
        trust_level: NewsProviderTrustLevel::Low,
        paper_only: true,
    };
    let run =
        collect_news_from_providers(&[local.clone(), disabled_rss]).expect("collect provider news");
    assert_eq!(run.collected_news.len(), 1);
    assert_eq!(run.news_snapshots.len(), 1);
    assert!(run.provider_results.iter().any(|result| {
        result.provider_id == "rss-disabled" && result.status == NewsProviderRunStatus::Disabled
    }));

    let allowed_remote = NewsProviderConfig {
        provider_id: "rss-allowed-deferred".to_string(),
        kind: NewsProviderKind::RssFeed,
        enabled: true,
        source_path_or_url: Some("https://news.example/feed.xml".to_string()),
        source_label: "rss-allowed".to_string(),
        allowed_domains: vec!["news.example".to_string()],
        symbols: Vec::new(),
        market_scopes: Vec::new(),
        max_items: 3,
        timeout_ms: 100,
        trust_level: NewsProviderTrustLevel::Low,
        paper_only: true,
    };
    let remote_run = collect_news_from_providers(&[allowed_remote])
        .expect("allowed remote provider safely deferred");
    assert!(remote_run.provider_results.iter().any(|result| {
        result.status == NewsProviderRunStatus::ProviderDeferred
            && result.message.contains("deferred")
    }));

    let allowed_http = NewsProviderConfig {
        provider_id: "http-allowed-deferred".to_string(),
        kind: NewsProviderKind::HttpHeadline,
        enabled: true,
        source_path_or_url: Some("https://headlines.example/top".to_string()),
        source_label: "http-allowed".to_string(),
        allowed_domains: vec!["headlines.example".to_string()],
        symbols: Vec::new(),
        market_scopes: Vec::new(),
        max_items: 3,
        timeout_ms: 100,
        trust_level: NewsProviderTrustLevel::Low,
        paper_only: true,
    };
    let http_run = collect_news_from_providers(&[allowed_http.clone()])
        .expect("allowed HTTP headline provider safely deferred");
    assert!(http_run.provider_results.iter().any(|result| {
        result.status == NewsProviderRunStatus::ProviderDeferred
            && result.message.contains("no browser")
    }));

    let no_allowlist_remote = NewsProviderConfig {
        provider_id: "rss-no-allowlist".to_string(),
        kind: NewsProviderKind::RssFeed,
        enabled: true,
        source_path_or_url: Some("https://news.example/feed.xml".to_string()),
        source_label: "rss-no-allowlist".to_string(),
        allowed_domains: Vec::new(),
        symbols: Vec::new(),
        market_scopes: Vec::new(),
        max_items: 3,
        timeout_ms: 100,
        trust_level: NewsProviderTrustLevel::Low,
        paper_only: true,
    };
    let err = collect_news_from_providers(&[no_allowlist_remote])
        .expect_err("enabled remote provider without allowlist rejected");
    assert!(err.contains("allowed_domains"));

    let disallowed_remote = NewsProviderConfig {
        provider_id: "rss-disallowed".to_string(),
        kind: NewsProviderKind::RssFeed,
        enabled: true,
        source_path_or_url: Some("https://badapproved.example/feed.xml".to_string()),
        source_label: "rss-disallowed".to_string(),
        allowed_domains: vec!["approved.example".to_string()],
        symbols: Vec::new(),
        market_scopes: Vec::new(),
        max_items: 3,
        timeout_ms: 100,
        trust_level: NewsProviderTrustLevel::Low,
        paper_only: true,
    };
    let err = collect_news_from_providers(&[disallowed_remote])
        .expect_err("disallowed remote host rejected by host");
    assert!(err.contains("allowed_domains"));

    let subdomain_remote = NewsProviderConfig {
        provider_id: "rss-subdomain-disallowed".to_string(),
        kind: NewsProviderKind::RssFeed,
        enabled: true,
        source_path_or_url: Some("https://feed.news.example/feed.xml".to_string()),
        source_label: "rss-subdomain-disallowed".to_string(),
        allowed_domains: vec!["news.example".to_string()],
        symbols: Vec::new(),
        market_scopes: Vec::new(),
        max_items: 3,
        timeout_ms: 100,
        trust_level: NewsProviderTrustLevel::Low,
        paper_only: true,
    };
    let err = collect_news_from_providers(&[subdomain_remote])
        .expect_err("allowlist host must be exact, not subdomain or substring");
    assert!(err.contains("allowed_domains"));

    let deterministic_first = collect_news_from_providers(&[allowed_http.clone(), local.clone()])
        .expect("deterministic first");
    let deterministic_second =
        collect_news_from_providers(&[local.clone(), allowed_http]).expect("deterministic second");
    assert_eq!(deterministic_first, deterministic_second);

    let unsafe_fixture_path = std::path::Path::new("target/sprint147_full_article_news.json");
    std::fs::write(
        unsafe_fixture_path,
        r#"[{"symbol":"005930.KS","headline":"h","summary":"s","source_label":"x","timestamp":"t","article_body":"full copy"}]"#,
    )
    .expect("write unsafe news fixture");
    let unsafe_local = NewsProviderConfig {
        source_path_or_url: Some(unsafe_fixture_path.display().to_string()),
        ..local
    };
    let err =
        collect_news_from_providers(&[unsafe_local]).expect_err("full article body field rejected");
    assert!(err.contains("article_body"));
    let _ = std::fs::remove_file(fixture_path);
    let _ = std::fs::remove_file(unsafe_fixture_path);
}

#[test]
fn safe_news_policy_disables_network_and_caps_summaries() {
    let policy = SafeNewsFetchPolicy::default();
    assert!(!policy.network_enabled);
    assert!(!policy.allow_full_article_body);
    assert!(!policy.allow_browser_scraping);
    assert!(!policy.allow_js_execution);

    let rss = NewsProviderConfig {
        provider_id: "rss-offline".to_string(),
        kind: NewsProviderKind::RssFeed,
        enabled: true,
        source_path_or_url: Some("https://news.example/feed.xml".to_string()),
        source_label: "rss-offline".to_string(),
        allowed_domains: vec!["news.example".to_string()],
        symbols: Vec::new(),
        market_scopes: Vec::new(),
        max_items: 3,
        timeout_ms: 100,
        trust_level: NewsProviderTrustLevel::Low,
        paper_only: true,
    };
    let rss_result = run_news_provider(&rss, &policy).expect("rss deferred offline");
    assert_eq!(rss_result.status, NewsProviderStatus::Deferred);
    assert!(!rss_result.network_used);

    let http = NewsProviderConfig {
        provider_id: "http-offline".to_string(),
        kind: NewsProviderKind::HttpHeadline,
        source_label: "http-offline".to_string(),
        ..rss.clone()
    };
    let http_result = run_news_provider(&http, &policy).expect("http deferred offline");
    assert_eq!(http_result.status, NewsProviderStatus::Deferred);
    assert!(!http_result.network_used);

    let mut explicit_policy = policy.clone();
    explicit_policy.network_enabled = true;
    explicit_policy.allowed_domains = vec!["approved.example".to_string()];
    let bypass = NewsProviderConfig {
        provider_id: "substring-bypass".to_string(),
        kind: NewsProviderKind::RssFeed,
        enabled: true,
        source_path_or_url: Some("https://badapproved.example/feed.xml".to_string()),
        source_label: "substring-bypass".to_string(),
        allowed_domains: vec!["approved.example".to_string()],
        symbols: Vec::new(),
        market_scopes: Vec::new(),
        max_items: 3,
        timeout_ms: 100,
        trust_level: NewsProviderTrustLevel::Low,
        paper_only: true,
    };
    let err = run_news_provider(&bypass, &explicit_policy)
        .expect_err("substring host allowlist bypass rejected");
    assert!(err.contains("exact allowed_domains"));

    let policy_allowlist_only = NewsProviderConfig {
        provider_id: "policy-allowlist-only".to_string(),
        kind: NewsProviderKind::RssFeed,
        enabled: true,
        source_path_or_url: Some("https://approved.example/feed.xml".to_string()),
        source_label: "policy-allowlist-only".to_string(),
        allowed_domains: Vec::new(),
        symbols: Vec::new(),
        market_scopes: Vec::new(),
        max_items: 3,
        timeout_ms: 100,
        trust_level: NewsProviderTrustLevel::Low,
        paper_only: true,
    };
    let allowed_result = run_news_provider(&policy_allowlist_only, &explicit_policy)
        .expect("policy allowlist exact host accepted");
    assert_eq!(allowed_result.status, NewsProviderStatus::Deferred);
    assert_eq!(allowed_result.host.as_deref(), Some("approved.example"));
    assert!(!allowed_result.network_used);

    let config_allowlist = NewsProviderConfig {
        allowed_domains: vec!["approved.example".to_string()],
        ..policy_allowlist_only
    };
    let allowed_result =
        run_news_provider(&config_allowlist, &explicit_policy).expect("config exact host accepted");
    assert_eq!(allowed_result.status, NewsProviderStatus::Deferred);

    let fixture_path = std::path::Path::new("target/sprint148_summary_cap_news.json");
    std::fs::write(
        fixture_path,
        serde_json::to_string_pretty(&vec![CollectedNewsItem {
            symbol: "005930.KS".to_string(),
            market_scope: Some(MarketScope::KoreaShortTerm),
            headline: "Summary cap headline".to_string(),
            summary: "1234567890 extra words beyond cap".to_string(),
            sentiment_hint: None,
            source_label: "cap-fixture".to_string(),
            timestamp: "2026-05-23T22:00:00+09:00".to_string(),
            url: None,
            license_note: Some("short summary fixture".to_string()),
        }])
        .expect("serialize cap fixture"),
    )
    .expect("write cap fixture");
    let local = NewsProviderConfig {
        provider_id: "cap-local".to_string(),
        kind: NewsProviderKind::LocalFixture,
        enabled: true,
        source_path_or_url: Some(fixture_path.display().to_string()),
        source_label: "cap-fixture".to_string(),
        allowed_domains: Vec::new(),
        symbols: Vec::new(),
        market_scopes: Vec::new(),
        max_items: 3,
        timeout_ms: 100,
        trust_level: NewsProviderTrustLevel::ReviewRequired,
        paper_only: true,
    };
    let mut capped_policy = SafeNewsFetchPolicy::default();
    capped_policy.max_summary_chars = 10;
    let local_result = run_news_provider(&local, &capped_policy).expect("local cap provider");
    assert_eq!(local_result.status, NewsProviderStatus::LocalFixtureReady);
    assert_eq!(local_result.collected_items[0].summary.chars().count(), 10);
    let _ = std::fs::remove_file(fixture_path);
}

#[test]
fn news_cache_adds_dedupes_persists_and_converts_to_snapshots() {
    let items = vec![
        CollectedNewsItem {
            symbol: "005930.KS".to_string(),
            market_scope: Some(MarketScope::KoreaShortTerm),
            headline: "Cache headline".to_string(),
            summary: "Cache summary only.".to_string(),
            sentiment_hint: Some("neutral".to_string()),
            source_label: "cache-fixture".to_string(),
            timestamp: "2026-05-23T22:01:00+09:00".to_string(),
            url: Some("https://news.example/item".to_string()),
            license_note: Some("headline and summary only".to_string()),
        },
        CollectedNewsItem {
            symbol: "005930.KS".to_string(),
            market_scope: Some(MarketScope::KoreaShortTerm),
            headline: "Cache headline".to_string(),
            summary: "Duplicate summary ignored by fingerprint.".to_string(),
            sentiment_hint: Some("neutral".to_string()),
            source_label: "cache-fixture".to_string(),
            timestamp: "2026-05-23T22:02:00+09:00".to_string(),
            url: Some("https://news.example/item".to_string()),
            license_note: Some("headline and summary only".to_string()),
        },
    ];
    let entries = news_cache_entries_from_collected_items(
        &items,
        "cache-provider",
        NewsProviderTrustLevel::ReviewRequired,
    );
    let mut store = NewsCacheStore::default();
    let update = store.add_entries(entries);
    assert_eq!(update.added_count, 1);
    assert_eq!(update.duplicate_count, 1);
    assert_eq!(store.entries_by_symbol("005930.KS").len(), 1);
    assert_eq!(store.entries_by_scope(MarketScope::KoreaShortTerm).len(), 1);
    assert_eq!(store.latest(1).len(), 1);
    let snapshots = news_cache_entries_to_news_snapshots(&store.entries);
    assert_eq!(snapshots.len(), 1);
    assert_eq!(snapshots[0].symbol, "005930.KS");

    let cache_path = std::path::Path::new("target/sprint148_news_cache.json");
    store.save_to_local_json(cache_path).expect("save cache");
    let loaded = NewsCacheStore::load_from_local_json(cache_path).expect("load cache");
    assert_eq!(loaded.entries.len(), 1);
    let remote_err = NewsCacheStore::load_from_local_json(std::path::Path::new(
        "https://example.invalid/cache.json",
    ))
    .expect_err("remote cache path rejected");
    assert!(remote_err.contains("must be local"));
    let _ = std::fs::remove_file(cache_path);
}

#[test]
fn research_auto_run_builds_packets_from_cache_and_optional_committee_cycle() {
    let fixture_path = std::path::Path::new("target/sprint148_auto_news_fixture.json");
    let provider_path = std::path::Path::new("target/sprint148_auto_news_providers.json");
    let cache_path = std::path::Path::new("target/sprint148_auto_news_cache.json");
    std::fs::write(
        fixture_path,
        serde_json::to_string_pretty(&vec![CollectedNewsItem {
            symbol: "005930.KS".to_string(),
            market_scope: Some(MarketScope::KoreaShortTerm),
            headline: "Auto run headline".to_string(),
            summary: "Auto run summary only.".to_string(),
            sentiment_hint: Some("positive".to_string()),
            source_label: "auto-fixture".to_string(),
            timestamp: "2026-05-23T22:03:00+09:00".to_string(),
            url: None,
            license_note: Some("headline and summary only".to_string()),
        }])
        .expect("serialize auto fixture"),
    )
    .expect("write auto fixture");
    let providers = vec![NewsProviderConfig {
        provider_id: "auto-local".to_string(),
        kind: NewsProviderKind::LocalFixture,
        enabled: true,
        source_path_or_url: Some(fixture_path.display().to_string()),
        source_label: "auto-fixture".to_string(),
        allowed_domains: Vec::new(),
        symbols: vec!["005930.KS".to_string()],
        market_scopes: vec![MarketScope::KoreaShortTerm],
        max_items: 5,
        timeout_ms: 100,
        trust_level: NewsProviderTrustLevel::ReviewRequired,
        paper_only: true,
    }];
    std::fs::write(
        provider_path,
        serde_json::to_string_pretty(&providers).expect("serialize auto providers"),
    )
    .expect("write auto providers");
    let config = ResearchAutoRunConfig {
        run_id: "sprint148-auto-run".to_string(),
        market_data_path: "examples/minimal_ai_committee_multi_market_sample.json".to_string(),
        news_provider_config_path: Some(provider_path.display().to_string()),
        news_cache_input_path: None,
        news_cache_output_path: Some(cache_path.display().to_string()),
        owner_intent_policy_path: None,
        owner_comment_text: Some("뉴스 근거가 부족해 보여. 다시 확인해줘".to_string()),
        owner_comment_path: None,
        member_state_input_path: None,
        offline_member_output_batch_path: None,
        network_mode: ResearchNetworkMode::OfflineOnly,
        news_fetch_policy: None,
        rss_xml_fixture_path: None,
        rss_fetch_pilot_enabled: false,
        rss_fetch_pilot_url: None,
        rss_fetch_allowed_domains: Vec::new(),
        rss_fetch_source_label: None,
        rss_network_enabled: false,
        rss_safe_http_timeout_ms: 3_000,
        rss_safe_http_rate_limit_ms: 1_000,
        rss_safe_http_max_response_bytes: 262_144,
        rss_allowed_content_types: vec![
            "application/rss+xml".to_string(),
            "application/xml".to_string(),
            "text/xml".to_string(),
            "application/atom+xml".to_string(),
        ],
        rss_allow_redirects: false,
        rss_allow_missing_content_type: false,
        run_committee_cycle: true,
        emit_research_summary: true,
        paper_only: true,
    };
    let first = run_research_auto_run(config.clone()).expect("first auto run");
    let second = run_research_auto_run(config).expect("second auto run");
    assert_eq!(first, second);
    assert_eq!(first.news_cache_update.added_count, 1);
    assert!(first.news_cache_update.duplicate_count == 0);
    assert!(first.research_packet_batch.packet_count > 0);
    assert!(
        first
            .research_packet_batch
            .packets
            .iter()
            .any(|packet| !packet.news.is_empty() && packet.owner_context.is_some())
    );
    assert!(first.committee_cycle_result.is_some());
    assert!(first.research_run_result.member_opinion_count > 0);
    assert!(first.safety_summary.no_broker_order_account);
    assert!(first.safety_summary.no_model_training);
    assert!(first.safety_summary.no_live_inference);
    let _ = std::fs::remove_file(fixture_path);
    let _ = std::fs::remove_file(provider_path);
    let _ = std::fs::remove_file(cache_path);
}

#[test]
fn minimal_ai_committee_cli_emits_research_auto_run_statuses() {
    let binary = env!("CARGO_BIN_EXE_soma_experiment");
    let fixture_path = "target/sprint148_cli_news_fixture.json";
    let provider_path = "target/sprint148_cli_news_providers.json";
    let cache_path = "target/sprint148_cli_news_cache.json";
    let config_path = "target/sprint148_cli_research_auto_run.toml";
    std::fs::write(
        fixture_path,
        serde_json::to_string_pretty(&vec![CollectedNewsItem {
            symbol: "005930.KS".to_string(),
            market_scope: Some(MarketScope::KoreaShortTerm),
            headline: "CLI auto run headline".to_string(),
            summary: "CLI auto run summary only.".to_string(),
            sentiment_hint: Some("neutral".to_string()),
            source_label: "cli-auto-fixture".to_string(),
            timestamp: "2026-05-23T22:04:00+09:00".to_string(),
            url: None,
            license_note: Some("headline and summary only".to_string()),
        }])
        .expect("serialize cli fixture"),
    )
    .expect("write cli fixture");
    let providers = vec![NewsProviderConfig {
        provider_id: "cli-auto-local".to_string(),
        kind: NewsProviderKind::LocalFixture,
        enabled: true,
        source_path_or_url: Some(fixture_path.to_string()),
        source_label: "cli-auto-fixture".to_string(),
        allowed_domains: Vec::new(),
        symbols: vec!["005930.KS".to_string()],
        market_scopes: vec![MarketScope::KoreaShortTerm],
        max_items: 5,
        timeout_ms: 100,
        trust_level: NewsProviderTrustLevel::ReviewRequired,
        paper_only: true,
    }];
    std::fs::write(
        provider_path,
        serde_json::to_string_pretty(&providers).expect("serialize cli providers"),
    )
    .expect("write cli providers");
    std::fs::write(
        config_path,
        format!(
            r#"input_path = "examples/minimal_ai_committee_multi_market_sample.json"
batch_mode = true
autonomous_paper_run = false
research_auto_run_enabled = true
run_id = "sprint148-cli-auto-run"
news_provider_config_path = "{provider_path}"
news_cache_output_path = "{cache_path}"
news_network_mode = "OfflineOnly"
run_committee_cycle_from_research_packets = true
owner_comment_text = "뉴스 근거가 부족해 보여. 다시 확인해줘"
paper_only = true
"#
        ),
    )
    .expect("write cli config");
    let output = Command::new(binary)
        .args(["minimal-ai-committee-cycle", "--config", config_path])
        .output()
        .expect("run minimal-ai-committee-cycle research auto-run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"news_provider_results\""));
    assert!(stdout.contains("\"network_used\": false"));
    assert!(stdout.contains("\"news_cache_update\""));
    assert!(stdout.contains("\"research_packet_batch\""));
    assert!(stdout.contains("\"no_broker_order_account\": true"));
    let _ = std::fs::remove_file(fixture_path);
    let _ = std::fs::remove_file(provider_path);
    let _ = std::fs::remove_file(cache_path);
    let _ = std::fs::remove_file(config_path);
}

#[test]
fn rss_xml_fixture_parser_parses_caps_rejects_and_converts() {
    let xml = r#"
<rss version="2.0">
  <channel>
    <item>
      <title>Samsung earnings headline</title>
      <link>https://news.example/samsung-earnings</link>
      <description>Short summary &amp; context only for AI member review.</description>
      <pubDate>Sat, 23 May 2026 10:00:00 +0900</pubDate>
    </item>
    <item>
      <title><![CDATA[Memory demand headline]]></title>
      <link>https://news.example/memory-demand</link>
      <description>1234567890 long description should be capped by the conservative RSS parser.</description>
      <pubDate>Sat, 23 May 2026 10:05:00 +0900</pubDate>
    </item>
    <item>
      <title>Unsafe broker account headline</title>
      <description>broker account and full article body should be rejected</description>
      <pubDate>Sat, 23 May 2026 10:10:00 +0900</pubDate>
    </item>
  </channel>
</rss>
"#;
    let result = parse_rss_xml_fixture(
        xml,
        RssXmlParseConfig {
            parser_id: "rss-parser-test".to_string(),
            source_label: "sample-rss".to_string(),
            symbol_hint: Some("005930.KS".to_string()),
            market_scope: Some(MarketScope::KoreaShortTerm),
            max_items: 5,
            max_summary_chars: 18,
            trust_level: NewsProviderTrustLevel::ReviewRequired,
            paper_only: true,
        },
    );

    assert_eq!(result.parse_status, RssXmlParseStatus::ParsedWithWarnings);
    assert_eq!(result.parsed_count, 2);
    assert_eq!(result.rejected_count, 1);
    assert_eq!(result.parsed_items[0].title, "Samsung earnings headline");
    assert_eq!(
        result.parsed_items[0].link.as_deref(),
        Some("https://news.example/samsung-earnings")
    );
    assert_eq!(
        result.parsed_items[0].pub_date.as_deref(),
        Some("Sat, 23 May 2026 10:00:00 +0900")
    );
    assert_eq!(
        result.parsed_items[1]
            .description
            .as_ref()
            .expect("capped description")
            .chars()
            .count(),
        18
    );
    assert!(
        result
            .safety_notes
            .iter()
            .any(|note| note.contains("no article link is fetched"))
    );

    let collected = convert_rss_items_to_collected_news(&result.parsed_items);
    assert_eq!(collected.len(), 2);
    assert_eq!(collected[0].symbol, "005930.KS");
    assert_eq!(collected[0].sentiment_hint.as_deref(), Some("Unknown"));
    assert_eq!(
        collected[0].url.as_deref(),
        Some("https://news.example/samsung-earnings")
    );
    let snapshots = convert_collected_news_to_snapshots(&collected, 5);
    assert_eq!(snapshots.len(), 2);
    assert_eq!(snapshots[0].symbol, "005930.KS");
    assert!(reject_full_article_like_summary("copied full article body").is_err());
}

#[test]
fn rss_xml_parser_reports_empty_feed_without_network() {
    let result = parse_rss_xml_fixture(
        "<rss><channel><title>empty</title></channel></rss>",
        RssXmlParseConfig {
            parser_id: "empty-rss".to_string(),
            source_label: "sample-rss".to_string(),
            symbol_hint: None,
            market_scope: None,
            max_items: 5,
            max_summary_chars: 80,
            trust_level: NewsProviderTrustLevel::Low,
            paper_only: true,
        },
    );

    assert_eq!(result.parse_status, RssXmlParseStatus::EmptyFeed);
    assert_eq!(result.parsed_count, 0);
    assert!(
        result
            .safety_notes
            .iter()
            .any(|note| note.contains("no <item>"))
    );
}

#[test]
fn news_cache_dedupes_rss_parse_results_and_rejects_unsafe_summary() {
    let xml = r#"
<rss><channel>
  <item>
    <title>Duplicate RSS headline</title>
    <link>https://news.example/duplicate</link>
    <description>Short cache summary.</description>
    <pubDate>Sat, 23 May 2026 11:00:00 +0900</pubDate>
  </item>
  <item>
    <title>Duplicate RSS headline</title>
    <link>https://news.example/duplicate</link>
    <description>Different summary should still dedupe by fingerprint.</description>
    <pubDate>Sat, 23 May 2026 11:05:00 +0900</pubDate>
  </item>
</channel></rss>
"#;
    let parse_result = parse_rss_xml_fixture(
        xml,
        RssXmlParseConfig {
            parser_id: "cache-rss".to_string(),
            source_label: "sample-rss".to_string(),
            symbol_hint: Some("005930.KS".to_string()),
            market_scope: Some(MarketScope::KoreaShortTerm),
            max_items: 5,
            max_summary_chars: 80,
            trust_level: NewsProviderTrustLevel::ReviewRequired,
            paper_only: true,
        },
    );
    let mut store = NewsCacheStore::default();
    let update = store.add_rss_parse_result(&parse_result);
    assert_eq!(update.added_count, 1);
    assert_eq!(update.duplicate_count, 1);
    assert_eq!(store.entries.len(), 1);
    assert_eq!(store.entries_by_symbol("005930.KS").len(), 1);

    let unsafe_update = store.add_entries(vec![
        soma_zero::league::minimal_ai_committee_core::NewsCacheEntry {
            cache_id: "unsafe-cache".to_string(),
            news_id: "unsafe-news".to_string(),
            symbol: Some("005930.KS".to_string()),
            market_scope: Some(MarketScope::KoreaShortTerm),
            headline: "Unsafe summary".to_string(),
            summary: "full article body copied here".to_string(),
            source_label: "sample-rss".to_string(),
            source_url: None,
            collected_at: Some("rss-fixture".to_string()),
            trust_level: NewsProviderTrustLevel::ReviewRequired,
            provider_id: "cache-rss".to_string(),
            fingerprint: "unsafe-fingerprint".to_string(),
            paper_only: true,
        },
    ]);
    assert_eq!(unsafe_update.rejected_count, 1);
}

#[test]
fn single_rss_fetch_pilot_is_disabled_or_deferred_and_exact_allowlisted() {
    let config = RssFetchPilotConfig {
        pilot_id: "rss-pilot".to_string(),
        enabled: false,
        rss_url: "https://approved.example/feed.xml".to_string(),
        allowed_domains: vec!["approved.example".to_string()],
        timeout_ms: 100,
        rate_limit_ms: 100,
        max_items: 5,
        max_summary_chars: 100,
        source_label: "sample-rss".to_string(),
        symbol_hint: Some("005930.KS".to_string()),
        market_scope: Some(MarketScope::KoreaShortTerm),
        paper_only: true,
    };
    let offline_policy = SafeNewsFetchPolicy::default();
    let disabled =
        run_single_allowlisted_rss_fetch_pilot(&config, &offline_policy).expect("disabled pilot");
    assert_eq!(disabled.status, RssFetchPilotStatus::Disabled);
    assert!(!disabled.network_used);

    let enabled = RssFetchPilotConfig {
        enabled: true,
        ..config.clone()
    };
    let deferred =
        run_single_allowlisted_rss_fetch_pilot(&enabled, &offline_policy).expect("deferred pilot");
    assert_eq!(deferred.status, RssFetchPilotStatus::FetchDeferred);
    assert_eq!(deferred.host.as_deref(), Some("approved.example"));
    assert!(deferred.allowed);
    assert!(!deferred.network_used);

    let mut network_policy = offline_policy.clone();
    network_policy.network_enabled = true;
    network_policy.allowed_domains = vec!["approved.example".to_string()];
    let exact =
        run_single_allowlisted_rss_fetch_pilot(&enabled, &network_policy).expect("exact allowed");
    assert_eq!(exact.status, RssFetchPilotStatus::FetchDeferred);
    assert_eq!(exact.host.as_deref(), Some("approved.example"));
    assert!(exact.allowed);
    assert!(!exact.network_used);

    let bypass = RssFetchPilotConfig {
        rss_url: "https://badapproved.example/feed.xml".to_string(),
        ..enabled
    };
    let rejected =
        run_single_allowlisted_rss_fetch_pilot(&bypass, &network_policy).expect("host rejected");
    assert_eq!(rejected.status, RssFetchPilotStatus::HostRejected);
    assert!(!rejected.allowed);
    assert!(!rejected.network_used);

    let offline_rejected = run_single_allowlisted_rss_fetch_pilot(&bypass, &offline_policy)
        .expect("offline host rejected before network gate");
    assert_eq!(offline_rejected.status, RssFetchPilotStatus::HostRejected);
    assert_eq!(
        offline_rejected.host.as_deref(),
        Some("badapproved.example")
    );
    assert!(!offline_rejected.allowed);
    assert!(!offline_rejected.network_used);
}

#[test]
fn safe_http_policy_and_request_guards_fail_closed() {
    let default_policy = SafeHttpClientPolicy::default();
    assert!(!default_policy.network_enabled);
    assert!(default_policy.require_https);
    assert!(!default_policy.allow_redirects);
    assert_eq!(default_policy.max_response_bytes, 262_144);
    assert!(
        default_policy
            .allowed_content_types
            .contains(&"application/rss+xml".to_string())
    );

    let mut policy = SafeHttpClientPolicy {
        network_enabled: true,
        allowed_hosts: vec!["approved.example".to_string()],
        ..SafeHttpClientPolicy::default()
    };
    let exact = SafeHttpRequest {
        request_id: "exact".to_string(),
        method: SafeHttpMethod::Get,
        url: "https://approved.example/rss.xml".to_string(),
        host: "approved.example".to_string(),
        source_label: "safe-http-test".to_string(),
        paper_only: true,
    };
    assert_eq!(
        validate_safe_http_request(&exact, &policy),
        SafeHttpFetchStatus::Fetched
    );

    let http = SafeHttpRequest {
        request_id: "http".to_string(),
        url: "http://approved.example/rss.xml".to_string(),
        ..exact.clone()
    };
    assert_eq!(
        validate_safe_http_request(&http, &policy),
        SafeHttpFetchStatus::SchemeRejected
    );

    let non_allowlisted = SafeHttpRequest {
        request_id: "non-allowlisted".to_string(),
        url: "https://other.example/rss.xml".to_string(),
        host: "other.example".to_string(),
        ..exact.clone()
    };
    assert_eq!(
        validate_safe_http_request(&non_allowlisted, &policy),
        SafeHttpFetchStatus::HostRejected
    );

    let substring_bypass = SafeHttpRequest {
        request_id: "substring".to_string(),
        url: "https://badapproved.example/rss.xml".to_string(),
        host: "badapproved.example".to_string(),
        ..exact.clone()
    };
    assert_eq!(
        validate_safe_http_request(&substring_bypass, &policy),
        SafeHttpFetchStatus::HostRejected
    );

    let post = SafeHttpRequest {
        request_id: "post".to_string(),
        method: SafeHttpMethod::Post,
        ..exact.clone()
    };
    assert_eq!(
        validate_safe_http_request(&post, &policy),
        SafeHttpFetchStatus::MethodRejected
    );

    policy.network_enabled = false;
    assert_eq!(
        validate_safe_http_request(&exact, &policy),
        SafeHttpFetchStatus::FetchDeferred
    );
    let deferred = fetch_safe_http_text(
        exact,
        &policy,
        &MockSafeHttpTransport {
            response: SafeHttpResponse {
                request_id: "unused".to_string(),
                status_code: Some(200),
                content_type: Some("application/rss+xml".to_string()),
                body_bytes: b"<rss><channel></channel></rss>".to_vec(),
                body_text: Some("<rss><channel></channel></rss>".to_string()),
                received_bytes: 29,
                network_used: false,
                paper_only: true,
            },
        },
    );
    assert_eq!(deferred.status, SafeHttpFetchStatus::FetchDeferred);
    assert!(deferred.response.is_none());
}

#[test]
fn safe_http_response_and_rss_content_type_gate_rejects_unsafe_payloads() {
    let policy = SafeHttpClientPolicy {
        network_enabled: true,
        allowed_hosts: vec!["approved.example".to_string()],
        max_response_bytes: 32,
        ..SafeHttpClientPolicy::default()
    };
    let gate = RssContentTypeGate::default();
    assert_eq!(
        validate_rss_content_type(Some("application/rss+xml; charset=utf-8"), &gate),
        RssContentTypeStatus::Allowed
    );
    assert_eq!(
        validate_rss_content_type(Some("text/html"), &gate),
        RssContentTypeStatus::Rejected
    );
    assert_eq!(
        validate_rss_content_type(None, &gate),
        RssContentTypeStatus::MissingRejected
    );

    let base = SafeHttpResponse {
        request_id: "response".to_string(),
        status_code: Some(200),
        content_type: Some("application/rss+xml".to_string()),
        body_bytes: b"<rss><channel></channel></rss>".to_vec(),
        body_text: Some("<rss><channel></channel></rss>".to_string()),
        received_bytes: 29,
        network_used: true,
        paper_only: true,
    };
    assert_eq!(
        validate_safe_http_response(&base, &policy),
        SafeHttpFetchStatus::Fetched
    );
    assert_eq!(
        validate_safe_http_response(
            &SafeHttpResponse {
                status_code: Some(302),
                ..base.clone()
            },
            &policy
        ),
        SafeHttpFetchStatus::RedirectRejected
    );
    assert_eq!(
        validate_safe_http_response(
            &SafeHttpResponse {
                status_code: Some(500),
                ..base.clone()
            },
            &policy
        ),
        SafeHttpFetchStatus::FetchFailed
    );
    assert_eq!(
        validate_safe_http_response(
            &SafeHttpResponse {
                content_type: Some("text/html".to_string()),
                body_text: Some("<html></html>".to_string()),
                body_bytes: b"<html></html>".to_vec(),
                ..base.clone()
            },
            &policy
        ),
        SafeHttpFetchStatus::ContentTypeRejected
    );
    assert_eq!(
        validate_safe_http_response(
            &SafeHttpResponse {
                content_type: None,
                ..base.clone()
            },
            &policy
        ),
        SafeHttpFetchStatus::ContentTypeRejected
    );
    assert_eq!(
        validate_safe_http_response(
            &SafeHttpResponse {
                body_bytes: vec![b'x'; 64],
                body_text: Some("x".repeat(64)),
                received_bytes: 64,
                ..base.clone()
            },
            &policy
        ),
        SafeHttpFetchStatus::ResponseTooLarge
    );
    assert_eq!(
        validate_safe_http_response(
            &SafeHttpResponse {
                body_bytes: Vec::new(),
                body_text: Some("x".repeat(64)),
                received_bytes: 0,
                ..base
            },
            &policy
        ),
        SafeHttpFetchStatus::ResponseTooLarge
    );
}

#[test]
fn rss_fetch_pilot_parses_mock_rss_and_rejects_html_without_real_network() {
    let config = RssFetchPilotConfig {
        pilot_id: "rss-mock-pilot".to_string(),
        enabled: true,
        rss_url: "https://approved.example/feed.xml".to_string(),
        allowed_domains: vec!["approved.example".to_string()],
        timeout_ms: 100,
        rate_limit_ms: 100,
        max_items: 5,
        max_summary_chars: 100,
        source_label: "mock-rss".to_string(),
        symbol_hint: Some("005930.KS".to_string()),
        market_scope: Some(MarketScope::KoreaShortTerm),
        paper_only: true,
    };
    let mut policy = SafeNewsFetchPolicy::default();
    policy.network_enabled = true;
    policy.allowed_domains = vec!["approved.example".to_string()];
    let body = r#"<rss><channel>
  <item>
    <title>Mock fetched RSS headline</title>
    <link>https://approved.example/article-not-fetched</link>
    <description>Mock fetched RSS summary only.</description>
    <pubDate>Sat, 23 May 2026 13:00:00 +0900</pubDate>
  </item>
</channel></rss>"#;
    let fetched = run_single_allowlisted_rss_fetch_pilot_with_transport(
        &config,
        &policy,
        &MockSafeHttpTransport {
            response: SafeHttpResponse {
                request_id: "mock".to_string(),
                status_code: Some(200),
                content_type: Some("application/rss+xml".to_string()),
                body_bytes: body.as_bytes().to_vec(),
                body_text: Some(body.to_string()),
                received_bytes: body.len(),
                network_used: true,
                paper_only: true,
            },
        },
    )
    .expect("mock RSS fetch pilot");
    assert_eq!(fetched.status, RssFetchPilotStatus::FetchedAndParsed);
    assert_eq!(fetched.safe_http_status, Some(SafeHttpFetchStatus::Fetched));
    assert_eq!(
        fetched.content_type_status,
        Some(RssContentTypeStatus::Allowed)
    );
    assert!(fetched.network_used);
    assert_eq!(fetched.collected_news.len(), 1);
    assert_eq!(
        fetched.collected_news[0].url.as_deref(),
        Some("https://approved.example/article-not-fetched")
    );
    assert!(
        fetched
            .safety_notes
            .iter()
            .any(|note| note.contains("RSS item links are not fetched"))
    );

    let html = run_single_allowlisted_rss_fetch_pilot_with_transport(
        &config,
        &policy,
        &MockSafeHttpTransport {
            response: SafeHttpResponse {
                request_id: "html".to_string(),
                status_code: Some(200),
                content_type: Some("text/html".to_string()),
                body_bytes: b"<html><body>article</body></html>".to_vec(),
                body_text: Some("<html><body>article</body></html>".to_string()),
                received_bytes: 33,
                network_used: true,
                paper_only: true,
            },
        },
    )
    .expect("html rejected");
    assert_eq!(html.status, RssFetchPilotStatus::FetchFailed);
    assert_eq!(
        html.safe_http_status,
        Some(SafeHttpFetchStatus::ContentTypeRejected)
    );
    assert_eq!(
        html.content_type_status,
        Some(RssContentTypeStatus::Rejected)
    );
    assert!(html.collected_news.is_empty());
}

#[test]
fn research_auto_run_builds_packets_from_mock_fetched_rss_news() {
    let cache_path = std::path::Path::new("target/sprint150_mock_rss_cache.json");
    let mut policy = SafeNewsFetchPolicy::default();
    policy.network_enabled = true;
    policy.allowed_domains = vec!["approved.example".to_string()];
    let body = r#"<rss><channel>
  <item>
    <title>Mock auto run RSS headline</title>
    <link>https://approved.example/auto-article-not-fetched</link>
    <description>Mock auto run RSS summary only.</description>
    <pubDate>Sat, 23 May 2026 14:00:00 +0900</pubDate>
  </item>
</channel></rss>"#;
    let config = ResearchAutoRunConfig {
        run_id: "sprint150-mock-rss-auto".to_string(),
        market_data_path: "examples/minimal_ai_committee_multi_market_sample.json".to_string(),
        news_provider_config_path: None,
        news_cache_input_path: None,
        news_cache_output_path: Some(cache_path.display().to_string()),
        owner_intent_policy_path: None,
        owner_comment_text: Some("뉴스 근거를 다시 확인해줘".to_string()),
        owner_comment_path: None,
        member_state_input_path: None,
        offline_member_output_batch_path: None,
        network_mode: ResearchNetworkMode::ExplicitNetworkAllowed,
        news_fetch_policy: Some(policy),
        rss_xml_fixture_path: None,
        rss_fetch_pilot_enabled: true,
        rss_fetch_pilot_url: Some("https://approved.example/feed.xml".to_string()),
        rss_fetch_allowed_domains: vec!["approved.example".to_string()],
        rss_fetch_source_label: Some("mock-rss".to_string()),
        rss_network_enabled: true,
        rss_safe_http_timeout_ms: 3_000,
        rss_safe_http_rate_limit_ms: 1_000,
        rss_safe_http_max_response_bytes: 262_144,
        rss_allowed_content_types: vec![
            "application/rss+xml".to_string(),
            "application/xml".to_string(),
            "text/xml".to_string(),
            "application/atom+xml".to_string(),
        ],
        rss_allow_redirects: false,
        rss_allow_missing_content_type: false,
        run_committee_cycle: true,
        emit_research_summary: true,
        paper_only: true,
    };
    let transport = MockSafeHttpTransport {
        response: SafeHttpResponse {
            request_id: "mock-auto".to_string(),
            status_code: Some(200),
            content_type: Some("application/rss+xml".to_string()),
            body_bytes: body.as_bytes().to_vec(),
            body_text: Some(body.to_string()),
            received_bytes: body.len(),
            network_used: true,
            paper_only: true,
        },
    };
    let first =
        run_research_auto_run_with_rss_transport(config.clone(), &transport).expect("first mock");
    let second = run_research_auto_run_with_rss_transport(config, &transport).expect("second mock");
    assert_eq!(first, second);
    assert_eq!(
        first.safe_http_fetch_status,
        Some(SafeHttpFetchStatus::Fetched)
    );
    assert_eq!(
        first.rss_content_type_status,
        Some(RssContentTypeStatus::Allowed)
    );
    assert_eq!(first.network_used_count, 1);
    assert_eq!(first.rss_fetched_count, 1);
    assert_eq!(first.cache_added_count, 1);
    assert!(
        first
            .research_packet_batch
            .packets
            .iter()
            .any(|packet| packet.symbol == "005930.KS" && !packet.news.is_empty())
    );
    assert!(
        first
            .committee_cycle_result
            .as_ref()
            .expect("committee cycle")
            .member_opinion_count
            > 0
    );
    assert!(first.safety_summary.no_broker_order_account);
    assert!(first.safety_summary.no_model_training);
    assert!(first.safety_summary.no_live_inference);
    let _ = std::fs::remove_file(cache_path);
}

#[test]
fn research_auto_run_uses_rss_fixture_for_packets_and_committee_cycle() {
    let rss_path = std::path::Path::new("target/sprint149_auto_rss_fixture.xml");
    let cache_path = std::path::Path::new("target/sprint149_auto_rss_cache.json");
    std::fs::write(
        rss_path,
        r#"<rss><channel>
  <item>
    <title>Auto RSS headline</title>
    <link>https://news.example/auto-rss</link>
    <description>Auto RSS summary only.</description>
    <pubDate>Sat, 23 May 2026 12:00:00 +0900</pubDate>
  </item>
</channel></rss>"#,
    )
    .expect("write rss fixture");
    let config = ResearchAutoRunConfig {
        run_id: "sprint149-auto-rss".to_string(),
        market_data_path: "examples/minimal_ai_committee_multi_market_sample.json".to_string(),
        news_provider_config_path: None,
        news_cache_input_path: None,
        news_cache_output_path: Some(cache_path.display().to_string()),
        owner_intent_policy_path: None,
        owner_comment_text: Some("뉴스 근거를 다시 확인해줘".to_string()),
        owner_comment_path: None,
        member_state_input_path: None,
        offline_member_output_batch_path: None,
        network_mode: ResearchNetworkMode::OfflineOnly,
        news_fetch_policy: None,
        rss_xml_fixture_path: Some(rss_path.display().to_string()),
        rss_fetch_pilot_enabled: false,
        rss_fetch_pilot_url: None,
        rss_fetch_allowed_domains: Vec::new(),
        rss_fetch_source_label: Some("sample-rss".to_string()),
        rss_network_enabled: false,
        rss_safe_http_timeout_ms: 3_000,
        rss_safe_http_rate_limit_ms: 1_000,
        rss_safe_http_max_response_bytes: 262_144,
        rss_allowed_content_types: vec![
            "application/rss+xml".to_string(),
            "application/xml".to_string(),
            "text/xml".to_string(),
            "application/atom+xml".to_string(),
        ],
        rss_allow_redirects: false,
        rss_allow_missing_content_type: false,
        run_committee_cycle: true,
        emit_research_summary: true,
        paper_only: true,
    };

    let first = run_research_auto_run(config.clone()).expect("first rss auto run");
    let second = run_research_auto_run(config).expect("second rss auto run");
    assert_eq!(first, second);
    assert_eq!(first.rss_parse_status, Some(RssXmlParseStatus::Parsed));
    assert_eq!(first.rss_items_parsed, 1);
    assert_eq!(first.cache_added_count, 1);
    assert!(!first.network_used);
    assert!(
        first
            .research_packet_batch
            .packets
            .iter()
            .any(|packet| packet.symbol == "005930.KS" && !packet.news.is_empty())
    );
    assert!(
        first
            .research_packet_batch
            .safety_notes
            .iter()
            .any(|note| note.contains("does not produce opinions"))
    );
    let committee = first
        .committee_cycle_result
        .as_ref()
        .expect("committee cycle result");
    assert!(committee.member_opinion_count > 0);
    assert!(committee.chairman_decisions.iter().all(|decision| matches!(
        decision.risk_governor_status,
        RiskGovernorStatus::Passed | RiskGovernorStatus::Vetoed | RiskGovernorStatus::NeedsReview
    )));
    assert!(first.safety_summary.no_broker_order_account);
    assert!(first.safety_summary.no_model_training);
    assert!(first.safety_summary.no_live_inference);
    let _ = std::fs::remove_file(rss_path);
    let _ = std::fs::remove_file(cache_path);
}

#[test]
fn sprint149_keeps_no_browser_js_tauri_svelte_dependency() {
    let cargo = std::fs::read_to_string("Cargo.toml").expect("Cargo.toml");
    for forbidden in ["tauri", "svelte", "react", "typescript"] {
        assert!(!cargo.to_ascii_lowercase().contains(forbidden));
    }
}

#[test]
fn research_packet_router_combines_market_news_and_owner_context_without_opinions() {
    let text = std::fs::read_to_string("examples/minimal_ai_committee_multi_market_sample.json")
        .expect("multi-market sample");
    let input: DataRouterInput = serde_json::from_str(&text).expect("data router input");
    let batch = build_ai_research_packets(
        input.market_data,
        input.news,
        input.members,
        Some("owner-natural-input: 근거를 더 확인".to_string()),
    );

    assert_eq!(batch.packet_count, 6);
    assert_eq!(batch.routed_member_count, 6);
    assert_eq!(batch.unrouted_symbol_count, 0);
    assert!(batch.packets.iter().all(|packet| packet.paper_only));
    assert!(batch.packets.iter().all(|packet| {
        packet
            .owner_context
            .as_deref()
            .unwrap_or("")
            .contains("owner-natural")
    }));
    assert!(batch.safety_notes.iter().any(|note| {
        note.contains("does not produce opinions") || note.contains("does not create opinions")
    }));
}

#[test]
fn research_run_pipeline_is_deterministic_and_keeps_members_as_judges() {
    let fixture_path = std::path::Path::new("target/sprint147_research_news_fixture.json");
    let provider_path = std::path::Path::new("target/sprint147_news_providers.json");
    let items = vec![CollectedNewsItem {
        symbol: "005930.KS".to_string(),
        market_scope: Some(MarketScope::KoreaShortTerm),
        headline: "Samsung research headline".to_string(),
        summary: "Short local research summary.".to_string(),
        sentiment_hint: Some("positive".to_string()),
        source_label: "research-fixture".to_string(),
        timestamp: "2026-05-23T09:04:00+09:00".to_string(),
        url: None,
        license_note: Some("headline and short summary only".to_string()),
    }];
    std::fs::write(
        fixture_path,
        serde_json::to_string_pretty(&items).expect("serialize research news fixture"),
    )
    .expect("write research news fixture");
    let providers = vec![NewsProviderConfig {
        provider_id: "research-local-provider".to_string(),
        kind: NewsProviderKind::LocalFixture,
        enabled: true,
        source_path_or_url: Some(fixture_path.display().to_string()),
        source_label: "research-fixture".to_string(),
        allowed_domains: Vec::new(),
        symbols: vec!["005930.KS".to_string()],
        market_scopes: vec![MarketScope::KoreaShortTerm],
        max_items: 5,
        timeout_ms: 100,
        trust_level: NewsProviderTrustLevel::ReviewRequired,
        paper_only: true,
    }];
    std::fs::write(
        provider_path,
        serde_json::to_string_pretty(&providers).expect("serialize providers"),
    )
    .expect("write providers");

    let config = ResearchRunConfig {
        research_run_id: "sprint147-research-run".to_string(),
        market_scopes: vec![MarketScope::KoreaShortTerm],
        symbols: vec!["005930.KS".to_string()],
        market_data_path: Some(
            "examples/minimal_ai_committee_multi_market_sample.json".to_string(),
        ),
        news_provider_config_path: Some(provider_path.display().to_string()),
        owner_intent_policy_path: None,
        owner_comment_text: Some("뉴스 근거가 부족해 보여. 다시 확인해줘".to_string()),
        owner_comment_path: None,
        member_state_input_path: None,
        offline_member_output_batch_path: None,
        run_mode: ResearchRunMode::SingleShot,
        max_cycles: 1,
        paper_only: true,
    };
    let first = run_research_packet_pipeline(config.clone()).expect("first research run");
    let second = run_research_packet_pipeline(config).expect("second research run");
    assert_eq!(first, second);
    assert_eq!(first.news_collection_run.collected_news.len(), 1);
    assert!(first.research_packet_batch.packet_count > 0);
    assert!(
        first
            .research_packet_batch
            .packets
            .iter()
            .all(|packet| packet.owner_context.is_some() && packet.paper_only)
    );
    assert!(first.member_opinion_count > 0);
    assert!(first.event_count > 0);
    assert!(first.committee_session_count > 0);
    assert_eq!(
        first.research_packet_summary.packets_generated,
        first.research_packet_batch.packet_count
    );
    assert!(first.safety_summary.no_broker_order_account);
    assert!(first.safety_summary.no_model_training);
    assert!(first.safety_summary.no_live_inference);
    assert!(first.research_packet_batch.safety_notes.iter().any(|note| {
        note.contains("does not produce opinions") || note.contains("AI members judge")
    }));
    assert!(
        first
            .research_packet_batch
            .safety_notes
            .iter()
            .any(|note| note.contains("Risk Governor remains final"))
    );
    let _ = std::fs::remove_file(fixture_path);
    let _ = std::fs::remove_file(provider_path);
}

#[test]
fn owner_say_cli_writes_internal_action_json() {
    let binary = env!("CARGO_BIN_EXE_soma_experiment");
    let output_path = "target/sprint146_owner_say_cli_action.json";
    let output = Command::new(binary)
        .args([
            "owner-say",
            "--text",
            "005930.KS 근거를 더 확인해줘",
            "--symbol",
            "005930.KS",
            "--scope",
            "KoreaShortTerm",
            "--out",
            output_path,
        ])
        .output()
        .expect("run owner-say CLI");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("owner_say_warning"));
    assert!(stdout.contains("wrote_output="));
    let saved: OwnerActionFile =
        serde_json::from_str(&std::fs::read_to_string(output_path).expect("read owner-say action"))
            .expect("owner-say JSON");
    assert_eq!(
        saved.actions[0].action_type,
        OwnerAttentionActionType::RequestMoreEvidence
    );
    assert!(saved.paper_only);

    let custom_policy_path = "target/sprint147_owner_say_policy.json";
    let custom_output_path = "target/sprint147_owner_say_cli_policy_action.json";
    let custom_policy = OwnerIntentPolicy {
        policy_id: "owner-say-cli-custom-policy".to_string(),
        language: OwnerIntentPolicyLanguage::Mixed,
        intent_rules: vec![OwnerIntentRule {
            rule_id: "custom-watch-term".to_string(),
            intent: OwnerNaturalInputIntent::WatchlistRequest,
            include_terms: vec!["보관".to_string()],
            exclude_terms: Vec::new(),
            priority: 100,
            confidence_hint: 0.95,
        }],
        safety_rules: Vec::new(),
        default_intent: OwnerNaturalInputIntent::Comment,
        paper_only: true,
    };
    std::fs::write(
        custom_policy_path,
        serde_json::to_string_pretty(&custom_policy).expect("custom owner-say policy"),
    )
    .expect("write custom owner-say policy");
    let output = Command::new(binary)
        .args([
            "owner-say",
            "--text",
            "이 종목은 보관",
            "--symbol",
            "005930.KS",
            "--scope",
            "KoreaShortTerm",
            "--policy",
            custom_policy_path,
            "--out",
            custom_output_path,
        ])
        .output()
        .expect("run owner-say CLI with policy");
    assert!(output.status.success());
    let custom_saved: OwnerActionFile = serde_json::from_str(
        &std::fs::read_to_string(custom_output_path).expect("read policy owner-say action"),
    )
    .expect("policy owner-say JSON");
    assert_eq!(
        custom_saved.actions[0].action_type,
        OwnerAttentionActionType::ConvertToWatchlist
    );
    assert!(custom_saved.paper_only);
    let _ = std::fs::remove_file(output_path);
    let _ = std::fs::remove_file(custom_policy_path);
    let _ = std::fs::remove_file(custom_output_path);
}

fn sprint161_write_research_registry(
    registry_path: &std::path::Path,
    news_path: &std::path::Path,
    price_path: &std::path::Path,
    rss_path: &std::path::Path,
) {
    let registry = ResearchSourceRegistry {
        registry_id: "sprint161-test-registry".to_string(),
        sources: vec![
            ResearchSourceDescriptor {
                source_id: "local-news".to_string(),
                kind: ResearchSourceKind::LocalNewsFixture,
                label: "local-news".to_string(),
                source_path_or_url: news_path.display().to_string(),
                allowed_host: None,
                market_scopes: vec![MarketScope::UsShortTerm],
                symbols: vec!["AAPL".to_string()],
                trust_level: ResearchSourceTrustLevel::ReviewRequired,
                enabled: true,
                max_items: 4,
                max_summary_chars: 120,
                paper_only: true,
            },
            ResearchSourceDescriptor {
                source_id: "local-price".to_string(),
                kind: ResearchSourceKind::LocalPriceSeries,
                label: "local-price".to_string(),
                source_path_or_url: price_path.display().to_string(),
                allowed_host: None,
                market_scopes: vec![MarketScope::UsShortTerm],
                symbols: vec!["AAPL".to_string()],
                trust_level: ResearchSourceTrustLevel::High,
                enabled: true,
                max_items: 2,
                max_summary_chars: 64,
                paper_only: true,
            },
            ResearchSourceDescriptor {
                source_id: "local-rss".to_string(),
                kind: ResearchSourceKind::LocalRssFixture,
                label: "local-rss".to_string(),
                source_path_or_url: rss_path.display().to_string(),
                allowed_host: None,
                market_scopes: vec![MarketScope::UsShortTerm],
                symbols: vec!["AAPL".to_string()],
                trust_level: ResearchSourceTrustLevel::ReviewRequired,
                enabled: true,
                max_items: 2,
                max_summary_chars: 120,
                paper_only: true,
            },
            ResearchSourceDescriptor {
                source_id: "disabled-network".to_string(),
                kind: ResearchSourceKind::AllowlistedRssFeed,
                label: "disabled-network".to_string(),
                source_path_or_url: "https://example.invalid/feed.xml".to_string(),
                allowed_host: Some("example.invalid".to_string()),
                market_scopes: Vec::new(),
                symbols: Vec::new(),
                trust_level: ResearchSourceTrustLevel::Low,
                enabled: false,
                max_items: 2,
                max_summary_chars: 120,
                paper_only: true,
            },
        ],
        enabled_count: 0,
        network_enabled_count: 0,
        local_source_count: 0,
        paper_only: true,
    };
    std::fs::write(
        registry_path,
        serde_json::to_string_pretty(&registry).expect("serialize research registry"),
    )
    .expect("write research registry");
}

fn sprint161_write_local_research_fixtures(
    prefix: &str,
) -> (
    std::path::PathBuf,
    std::path::PathBuf,
    std::path::PathBuf,
    std::path::PathBuf,
) {
    std::fs::create_dir_all("target").expect("target dir");
    let news_path = std::path::PathBuf::from(format!("target/{prefix}_news.json"));
    let price_path = std::path::PathBuf::from(format!("target/{prefix}_prices.json"));
    let rss_path = std::path::PathBuf::from(format!("target/{prefix}_rss.xml"));
    let registry_path = std::path::PathBuf::from(format!("target/{prefix}_registry.json"));
    let news_items = vec![CollectedNewsItem {
        symbol: "AAPL".to_string(),
        market_scope: Some(MarketScope::UsShortTerm),
        headline: "AAPL local headline".to_string(),
        summary: "Local paper-only summary for AAPL momentum context".to_string(),
        sentiment_hint: Some("positive".to_string()),
        source_label: "unit-local-news".to_string(),
        timestamp: "2026-05-25T09:00:00Z".to_string(),
        url: Some("https://example.invalid/aapl".to_string()),
        license_note: Some("headline-only local fixture".to_string()),
    }];
    std::fs::write(
        &news_path,
        serde_json::to_string_pretty(&news_items).expect("serialize news fixture"),
    )
    .expect("write news fixture");
    save_price_series_store_to_local_json(&sprint158_price_series_store(), &price_path)
        .expect("write price fixture");
    std::fs::write(
        &rss_path,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>Local RSS</title>
    <item>
      <title>AAPL rss headline</title>
      <description>Short rss summary for AAPL research context.</description>
      <link>https://example.invalid/rss-aapl</link>
      <pubDate>Mon, 25 May 2026 09:00:00 GMT</pubDate>
    </item>
  </channel>
</rss>"#,
    )
    .expect("write rss fixture");
    sprint161_write_research_registry(&registry_path, &news_path, &price_path, &rss_path);
    (news_path, price_path, rss_path, registry_path)
}

fn sprint161_research_inventory() -> WeakReplayLabelInventory {
    WeakReplayLabelInventory {
        inventory_id: "sprint161-weak-inventory".to_string(),
        dataset_id: "sprint161-dataset".to_string(),
        total_examples: 3,
        weak_label_count: 3,
        review_required_count: 3,
        low_confidence_count: 0,
        ambiguous_label_count: 0,
        missing_evidence_count: 3,
        weak_items: vec![
            WeakReplayLabelItem {
                item_id: "trend-aapl".to_string(),
                replay_id: "replay-trend-aapl".to_string(),
                experience_id: Some("exp-trend-aapl".to_string()),
                decision_id: Some("decision-trend-aapl".to_string()),
                member_id: "TrendEntryAI".to_string(),
                symbol: "AAPL".to_string(),
                market_scope: MarketScope::UsShortTerm,
                current_label_source: ReplayLabelSource::ReviewRequired,
                current_label_confidence: ReplayLabelConfidence::ReviewRequired,
                current_outcome_label: MemberExperienceOutcome::Unknown,
                weakness_reasons: vec![WeakReplayLabelReason::UnknownOutcome],
                suggested_next_evidence: WeakReplaySuggestedEvidence::AddPriceMoveEvidence,
                priority: WeakReplayLabelPriority::High,
                paper_only: true,
            },
            WeakReplayLabelItem {
                item_id: "risk-msft".to_string(),
                replay_id: "replay-risk-msft".to_string(),
                experience_id: Some("exp-risk-msft".to_string()),
                decision_id: Some("decision-risk-msft".to_string()),
                member_id: "RiskGuardAI".to_string(),
                symbol: "MSFT".to_string(),
                market_scope: MarketScope::UsLongTerm,
                current_label_source: ReplayLabelSource::AmbiguousLabel,
                current_label_confidence: ReplayLabelConfidence::ReviewRequired,
                current_outcome_label: MemberExperienceOutcome::PaperNegative,
                weakness_reasons: vec![WeakReplayLabelReason::ContradictoryEvidence],
                suggested_next_evidence: WeakReplaySuggestedEvidence::RejectLabel,
                priority: WeakReplayLabelPriority::High,
                paper_only: true,
            },
            WeakReplayLabelItem {
                item_id: "evidence-aapl".to_string(),
                replay_id: "replay-evidence-aapl".to_string(),
                experience_id: Some("exp-evidence-aapl".to_string()),
                decision_id: Some("decision-evidence-aapl".to_string()),
                member_id: "EvidenceRegimeAI".to_string(),
                symbol: "AAPL".to_string(),
                market_scope: MarketScope::UsShortTerm,
                current_label_source: ReplayLabelSource::ReviewRequired,
                current_label_confidence: ReplayLabelConfidence::ReviewRequired,
                current_outcome_label: MemberExperienceOutcome::Unknown,
                weakness_reasons: vec![WeakReplayLabelReason::MissingPriceEvidence],
                suggested_next_evidence: WeakReplaySuggestedEvidence::AddPaperOutcomeReview,
                priority: WeakReplayLabelPriority::Normal,
                paper_only: true,
            },
        ],
        paper_only: true,
    }
}

#[test]
fn sprint161_research_source_registry_loads_local_sources_and_rejects_remote_and_wildcard_hosts() {
    let (news_path, price_path, rss_path, registry_path) =
        sprint161_write_local_research_fixtures("sprint161_registry");
    let registry =
        ResearchSourceRegistry::load_from_local_json(&registry_path).expect("load registry");
    assert_eq!(registry.enabled_count, 3);
    assert_eq!(registry.network_enabled_count, 0);
    assert_eq!(registry.local_source_count, 3);
    assert_eq!(registry.enabled_sources().len(), 3);
    assert_eq!(registry.disabled_sources().len(), 1);
    assert_eq!(registry.sources_for_symbol("AAPL").len(), 3);
    assert_eq!(
        registry.sources_for_scope(MarketScope::UsShortTerm).len(),
        3
    );

    let remote_registry_path = std::path::PathBuf::from("target/sprint161_registry_remote.json");
    std::fs::write(
        &remote_registry_path,
        serde_json::json!({
            "registry_id": "remote-registry",
            "sources": [{
                "source_id": "remote-local",
                "kind": "LocalNewsFixture",
                "label": "remote-local",
                "source_path_or_url": "https://example.invalid/news.json",
                "allowed_host": null,
                "market_scopes": [],
                "symbols": [],
                "trust_level": "ReviewRequired",
                "enabled": true,
                "max_items": 2,
                "max_summary_chars": 64,
                "paper_only": true
            }],
            "enabled_count": 1,
            "network_enabled_count": 0,
            "local_source_count": 1,
            "paper_only": true
        })
        .to_string(),
    )
    .expect("write remote registry");
    let remote_err = ResearchSourceRegistry::load_from_local_json(&remote_registry_path)
        .expect_err("remote local source rejected");
    assert!(remote_err.contains("local"));

    let wildcard_registry_path =
        std::path::PathBuf::from("target/sprint161_registry_wildcard.json");
    std::fs::write(
        &wildcard_registry_path,
        serde_json::json!({
            "registry_id": "wildcard-registry",
            "sources": [{
                "source_id": "wildcard-network",
                "kind": "AllowlistedRssFeed",
                "label": "wildcard-network",
                "source_path_or_url": "https://news.example/feed.xml",
                "allowed_host": "*.example",
                "market_scopes": [],
                "symbols": [],
                "trust_level": "Low",
                "enabled": false,
                "max_items": 2,
                "max_summary_chars": 64,
                "paper_only": true
            }],
            "enabled_count": 0,
            "network_enabled_count": 0,
            "local_source_count": 0,
            "paper_only": true
        })
        .to_string(),
    )
    .expect("write wildcard registry");
    let wildcard_err = ResearchSourceRegistry::load_from_local_json(&wildcard_registry_path)
        .expect_err("wildcard host rejected");
    assert!(wildcard_err.contains("non-wildcard host"));

    let substring_registry_path =
        std::path::PathBuf::from("target/sprint161_registry_substring.json");
    std::fs::write(
        &substring_registry_path,
        serde_json::json!({
            "registry_id": "substring-registry",
            "sources": [{
                "source_id": "substring-network",
                "kind": "AllowlistedRssFeed",
                "label": "substring-network",
                "source_path_or_url": "https://news.example.com/feed.xml",
                "allowed_host": "example.com",
                "market_scopes": [],
                "symbols": [],
                "trust_level": "Low",
                "enabled": false,
                "max_items": 2,
                "max_summary_chars": 64,
                "paper_only": true
            }],
            "enabled_count": 0,
            "network_enabled_count": 0,
            "local_source_count": 0,
            "paper_only": true
        })
        .to_string(),
    )
    .expect("write substring registry");
    let substring_err = ResearchSourceRegistry::load_from_local_json(&substring_registry_path)
        .expect_err("substring host rejected");
    assert!(substring_err.contains("URL host must equal allowed_host"));

    let _ = std::fs::remove_file(news_path);
    let _ = std::fs::remove_file(price_path);
    let _ = std::fs::remove_file(rss_path);
    let _ = std::fs::remove_file(registry_path);
    let _ = std::fs::remove_file(remote_registry_path);
    let _ = std::fs::remove_file(wildcard_registry_path);
    let _ = std::fs::remove_file(substring_registry_path);
}

#[test]
fn sprint161_member_research_tasks_route_by_member_role() {
    let queue = build_tasks_from_weak_labels(&sprint161_research_inventory());
    assert_eq!(queue.tasks.len(), 3);
    assert!(queue.tasks.iter().any(|task| {
        task.member_id == "TrendEntryAI"
            && task.task_type
                == soma_zero::league::minimal_ai_committee_core::MemberResearchTaskType::GatherPriceEvidence
    }));
    assert!(queue.tasks.iter().any(|task| {
        task.member_id == "RiskGuardAI"
            && task.task_type
                == soma_zero::league::minimal_ai_committee_core::MemberResearchTaskType::FindContradictoryEvidence
    }));
    assert!(queue.tasks.iter().any(|task| {
        task.member_id == "EvidenceRegimeAI"
            && task.task_type
                == soma_zero::league::minimal_ai_committee_core::MemberResearchTaskType::ReviewNeedMoreEvidenceCase
    }));
    assert!(queue.high_priority_count >= 2);
}

#[test]
fn sprint161_research_evidence_bundle_collects_local_fixture_evidence_and_news_does_not_promote() {
    let (news_path, price_path, rss_path, registry_path) =
        sprint161_write_local_research_fixtures("sprint161_bundle");
    let registry =
        ResearchSourceRegistry::load_from_local_json(&registry_path).expect("load registry");
    let trust = evaluate_research_source_trust(&registry, None);
    assert!(trust.trusted_count >= 1);
    assert!(trust.source_scores.iter().any(|score| {
        score.source_id == "local-news" && score.trust_status == SourceTrustStatus::ReviewRequired
    }));

    let inventory = WeakReplayLabelInventory {
        inventory_id: "bundle-weak".to_string(),
        dataset_id: "bundle-dataset".to_string(),
        total_examples: 1,
        weak_label_count: 1,
        review_required_count: 1,
        low_confidence_count: 0,
        ambiguous_label_count: 0,
        missing_evidence_count: 1,
        weak_items: vec![sprint161_research_inventory().weak_items[0].clone()],
        paper_only: true,
    };
    let queue = build_tasks_from_weak_labels(&inventory);
    let bundle = collect_research_evidence(&queue.tasks, &registry, &trust.source_scores)
        .expect("collect research evidence");
    assert!(bundle.evidence_count >= 2);
    assert!(
        bundle
            .records
            .iter()
            .any(|record| record.kind == ResearchEvidenceKind::NewsHeadline)
    );
    assert!(
        bundle
            .records
            .iter()
            .any(|record| record.kind == ResearchEvidenceKind::PriceMove)
    );
    let serialized = serde_json::to_string(&bundle)
        .expect("serialize bundle")
        .to_ascii_lowercase();
    assert!(!serialized.contains("broker"));
    assert!(!serialized.contains("account"));
    assert!(!serialized.contains("article_body"));
    assert!(!serialized.contains("full_article"));

    let conversion = convert_research_evidence_to_paper_outcome_evidence(
        &bundle,
        &ResearchToPaperEvidenceConversionPolicy::default(),
    );
    assert!(conversion.converted_count >= 1);
    assert!(
        conversion
            .skip_reasons
            .iter()
            .any(|reason| reason.contains("news headline alone"))
    );
    let news_only_records: Vec<_> = bundle
        .records
        .iter()
        .filter(|record| record.kind == ResearchEvidenceKind::NewsHeadline)
        .cloned()
        .collect();
    let news_only_bundle = soma_zero::league::minimal_ai_committee_core::ResearchEvidenceBundle {
        bundle_id: "news-only".to_string(),
        records: news_only_records.clone(),
        source_scores: bundle.source_scores.clone(),
        task_count: bundle.task_count,
        evidence_count: news_only_records.len(),
        paper_only: true,
    };
    let news_only_conversion = convert_research_evidence_to_paper_outcome_evidence(
        &news_only_bundle,
        &ResearchToPaperEvidenceConversionPolicy::default(),
    );
    assert_eq!(news_only_conversion.converted_count, 0);
    let mut low_trust_bundle = bundle.clone();
    for score in &mut low_trust_bundle.source_scores {
        if score.source_id == "local-price" {
            score.base_trust_level = ResearchSourceTrustLevel::Low;
            score.trust_status = SourceTrustStatus::UsableWithWarnings;
            score.final_score = 0.68;
        }
    }
    let low_trust_conversion = convert_research_evidence_to_paper_outcome_evidence(
        &low_trust_bundle,
        &ResearchToPaperEvidenceConversionPolicy::default(),
    );
    assert_eq!(low_trust_conversion.converted_count, 0);
    assert!(
        low_trust_conversion
            .skip_reasons
            .iter()
            .any(|reason| reason.contains("low or review-required trust"))
    );

    let _ = std::fs::remove_file(news_path);
    let _ = std::fs::remove_file(price_path);
    let _ = std::fs::remove_file(rss_path);
    let _ = std::fs::remove_file(registry_path);
}

#[test]
fn sprint161_self_growing_replay_run_is_deterministic_and_paper_only() {
    let (news_path, price_path, rss_path, registry_path) =
        sprint161_write_local_research_fixtures("sprint161_self_growing");
    let dataset_path = std::path::PathBuf::from("target/sprint161_self_growing_dataset.json");
    let dataset = sprint156_replay_dataset(vec![sprint159_replay_example(
        "replay-self-growing-aapl",
        "AAPL",
        MarketScope::UsShortTerm,
        ReplayLabelSource::ReviewRequired,
        ReplayLabelConfidence::ReviewRequired,
        MemberExperienceOutcome::Unknown,
    )]);
    dataset
        .save_to_local_json(&dataset_path)
        .expect("save self-growing dataset");

    let config = SelfGrowingReplayEvidenceConfig {
        run_id: "sprint161-self-growing".to_string(),
        source_registry_path: Some(registry_path.display().to_string()),
        experience_store_path: None,
        sanitized_dataset_path: None,
        validated_dataset_path: Some(dataset_path.display().to_string()),
        weak_label_inventory_path: None,
        coverage_gap_path: None,
        max_tasks: 4,
        max_evidence_records: 8,
        allow_network_sources: false,
        paper_only: true,
    };
    let first = run_self_growing_replay_evidence(config.clone()).expect("first self-growing run");
    let second =
        run_self_growing_replay_evidence(config).expect("repeat deterministic self-growing run");
    assert_eq!(first.research_task_queue, second.research_task_queue);
    assert_eq!(
        first.research_evidence_bundle.records,
        second.research_evidence_bundle.records
    );
    assert_eq!(
        first.generated_paper_evidence_records,
        second.generated_paper_evidence_records
    );
    assert!(!first.generated_paper_evidence_records.is_empty());
    assert!(
        first
            .training_candidate_dataset
            .as_ref()
            .map(|dataset| dataset.example_count)
            .unwrap_or(0)
            >= 1
    );
    let serialized = serde_json::to_string(&(
        &first.research_evidence_bundle,
        &first.generated_paper_evidence_records,
    ))
    .expect("serialize self-growing research evidence")
    .to_ascii_lowercase();
    for forbidden in ["broker", "account", "live inference", "order execution"] {
        assert!(
            !serialized.contains(forbidden),
            "self-growing result leaked forbidden fragment: {forbidden}"
        );
    }
    assert!(first.safety_summary.no_model_training);
    assert!(first.safety_summary.no_live_inference);
    assert!(first.safety_summary.no_real_order_path);
    assert!(first.safety_summary.no_broker_order_account);

    let _ = std::fs::remove_file(news_path);
    let _ = std::fs::remove_file(price_path);
    let _ = std::fs::remove_file(rss_path);
    let _ = std::fs::remove_file(registry_path);
    let _ = std::fs::remove_file(dataset_path);
}

fn sprint162_sample_staging_store() -> SelfGrowingEvidenceStagingStore {
    SelfGrowingEvidenceStagingStore {
        store_id: "sprint162-staging-store".to_string(),
        candidates: vec![
            SelfGrowingEvidenceCandidate {
                candidate_id: "candidate-strong-price".to_string(),
                source_research_evidence_id: "evidence-strong-price".to_string(),
                member_id: "TrendEntryAI".to_string(),
                symbol: Some("AAPL".to_string()),
                market_scope: Some(MarketScope::UsShortTerm),
                related_replay_id: Some("replay-aapl-001".to_string()),
                related_experience_id: Some("exp-aapl-001".to_string()),
                related_decision_id: Some("decision-aapl-001".to_string()),
                evidence_kind: SelfGrowingEvidenceKind::PriceMove,
                source_id: Some("local-price".to_string()),
                source_trust_status: SourceTrustStatus::Trusted,
                evidence_confidence: ReplayLabelConfidence::High,
                candidate_label: Some(MemberExperienceOutcome::PaperPositive),
                label_source: SelfGrowingEvidenceLabelSource::SelfGrowingPriceEvidence,
                reference_price: Some(100.0),
                horizon_price: Some(103.1),
                price_change_pct: Some(0.031),
                evidence_notes: vec!["price_change_pct=0.031000".to_string()],
                status: SelfGrowingEvidenceCandidateStatus::Pending,
                paper_only: true,
            },
            SelfGrowingEvidenceCandidate {
                candidate_id: "candidate-news-only".to_string(),
                source_research_evidence_id: "evidence-news-only".to_string(),
                member_id: "TrendEntryAI".to_string(),
                symbol: Some("AAPL".to_string()),
                market_scope: Some(MarketScope::UsShortTerm),
                related_replay_id: Some("replay-aapl-001".to_string()),
                related_experience_id: None,
                related_decision_id: Some("decision-aapl-001".to_string()),
                evidence_kind: SelfGrowingEvidenceKind::NewsContext,
                source_id: Some("local-news".to_string()),
                source_trust_status: SourceTrustStatus::ReviewRequired,
                evidence_confidence: ReplayLabelConfidence::Low,
                candidate_label: None,
                label_source: SelfGrowingEvidenceLabelSource::SelfGrowingNewsContext,
                reference_price: None,
                horizon_price: None,
                price_change_pct: None,
                evidence_notes: vec!["headline-only context".to_string()],
                status: SelfGrowingEvidenceCandidateStatus::Pending,
                paper_only: true,
            },
            SelfGrowingEvidenceCandidate {
                candidate_id: "candidate-low-trust".to_string(),
                source_research_evidence_id: "evidence-low-trust".to_string(),
                member_id: "EvidenceRegimeAI".to_string(),
                symbol: Some("AAPL".to_string()),
                market_scope: Some(MarketScope::UsShortTerm),
                related_replay_id: Some("replay-aapl-001".to_string()),
                related_experience_id: None,
                related_decision_id: None,
                evidence_kind: SelfGrowingEvidenceKind::SupportingEvidence,
                source_id: Some("local-rss".to_string()),
                source_trust_status: SourceTrustStatus::ReviewRequired,
                evidence_confidence: ReplayLabelConfidence::ReviewRequired,
                candidate_label: Some(MemberExperienceOutcome::Unknown),
                label_source: SelfGrowingEvidenceLabelSource::ReviewRequired,
                reference_price: None,
                horizon_price: None,
                price_change_pct: None,
                evidence_notes: vec!["review-required support".to_string()],
                status: SelfGrowingEvidenceCandidateStatus::Pending,
                paper_only: true,
            },
            SelfGrowingEvidenceCandidate {
                candidate_id: "candidate-ambiguous".to_string(),
                source_research_evidence_id: "evidence-ambiguous".to_string(),
                member_id: "RiskGuardAI".to_string(),
                symbol: Some("AAPL".to_string()),
                market_scope: Some(MarketScope::UsShortTerm),
                related_replay_id: None,
                related_experience_id: Some("exp-aapl-ambiguous".to_string()),
                related_decision_id: Some("decision-aapl-ambiguous".to_string()),
                evidence_kind: SelfGrowingEvidenceKind::SupportingEvidence,
                source_id: Some("local-market".to_string()),
                source_trust_status: SourceTrustStatus::UsableWithWarnings,
                evidence_confidence: ReplayLabelConfidence::Medium,
                candidate_label: Some(MemberExperienceOutcome::Unknown),
                label_source: SelfGrowingEvidenceLabelSource::ReviewRequired,
                reference_price: None,
                horizon_price: None,
                price_change_pct: None,
                evidence_notes: vec!["partial linkage only".to_string()],
                status: SelfGrowingEvidenceCandidateStatus::Pending,
                paper_only: true,
            },
            SelfGrowingEvidenceCandidate {
                candidate_id: "candidate-missing-replay".to_string(),
                source_research_evidence_id: "evidence-missing-replay".to_string(),
                member_id: "TrendEntryAI".to_string(),
                symbol: Some("AAPL".to_string()),
                market_scope: Some(MarketScope::UsShortTerm),
                related_replay_id: None,
                related_experience_id: None,
                related_decision_id: None,
                evidence_kind: SelfGrowingEvidenceKind::PriceMove,
                source_id: Some("local-price".to_string()),
                source_trust_status: SourceTrustStatus::Trusted,
                evidence_confidence: ReplayLabelConfidence::Medium,
                candidate_label: Some(MemberExperienceOutcome::PaperPositive),
                label_source: SelfGrowingEvidenceLabelSource::SelfGrowingPriceEvidence,
                reference_price: None,
                horizon_price: None,
                price_change_pct: Some(0.022),
                evidence_notes: vec!["price_change_pct=0.022000".to_string()],
                status: SelfGrowingEvidenceCandidateStatus::Pending,
                paper_only: true,
            },
        ],
        candidate_count: 5,
        pending_count: 5,
        approved_count: 0,
        rejected_count: 0,
        review_required_count: 0,
        paper_only: true,
    }
}

fn sprint162_source_scores() -> Vec<soma_zero::league::minimal_ai_committee_core::SourceTrustScore>
{
    vec![
        soma_zero::league::minimal_ai_committee_core::SourceTrustScore {
            source_id: "local-price".to_string(),
            base_trust_level: ResearchSourceTrustLevel::High,
            freshness_score: 0.8,
            consistency_score: 0.8,
            coverage_score: 0.8,
            safety_score: 1.0,
            final_score: 0.88,
            trust_status: SourceTrustStatus::Trusted,
            reasons: vec!["trusted local price series".to_string()],
            paper_only: true,
        },
        soma_zero::league::minimal_ai_committee_core::SourceTrustScore {
            source_id: "local-news".to_string(),
            base_trust_level: ResearchSourceTrustLevel::ReviewRequired,
            freshness_score: 0.7,
            consistency_score: 0.6,
            coverage_score: 0.6,
            safety_score: 1.0,
            final_score: 0.35,
            trust_status: SourceTrustStatus::ReviewRequired,
            reasons: vec!["headline context only".to_string()],
            paper_only: true,
        },
        soma_zero::league::minimal_ai_committee_core::SourceTrustScore {
            source_id: "local-rss".to_string(),
            base_trust_level: ResearchSourceTrustLevel::Low,
            freshness_score: 0.7,
            consistency_score: 0.5,
            coverage_score: 0.5,
            safety_score: 1.0,
            final_score: 0.32,
            trust_status: SourceTrustStatus::ReviewRequired,
            reasons: vec!["review required".to_string()],
            paper_only: true,
        },
        soma_zero::league::minimal_ai_committee_core::SourceTrustScore {
            source_id: "local-market".to_string(),
            base_trust_level: ResearchSourceTrustLevel::Medium,
            freshness_score: 0.7,
            consistency_score: 0.7,
            coverage_score: 0.7,
            safety_score: 1.0,
            final_score: 0.72,
            trust_status: SourceTrustStatus::UsableWithWarnings,
            reasons: vec!["local market support".to_string()],
            paper_only: true,
        },
    ]
}

fn sprint163_replay_dataset() -> ReplayDataset {
    let mut trend = sprint159_replay_example(
        "replay-aapl-001",
        "AAPL",
        MarketScope::UsShortTerm,
        ReplayLabelSource::ReviewRequired,
        ReplayLabelConfidence::ReviewRequired,
        MemberExperienceOutcome::Unknown,
    );
    trend.member_id = "TrendEntryAI".to_string();
    let mut evidence = sprint159_replay_example(
        "replay-evidence-aapl-001",
        "AAPL",
        MarketScope::UsShortTerm,
        ReplayLabelSource::ReviewRequired,
        ReplayLabelConfidence::ReviewRequired,
        MemberExperienceOutcome::Unknown,
    );
    evidence.member_id = "EvidenceRegimeAI".to_string();
    let mut risk = sprint159_replay_example(
        "replay-risk-aapl-001",
        "AAPL",
        MarketScope::UsShortTerm,
        ReplayLabelSource::ReviewRequired,
        ReplayLabelConfidence::ReviewRequired,
        MemberExperienceOutcome::Unknown,
    );
    risk.member_id = "RiskGuardAI".to_string();
    ReplayDataset {
        dataset_id: "sprint163-dataset".to_string(),
        example_count: 3,
        member_count: 3,
        examples: vec![trend, evidence, risk],
        generated_from_store_id: "sprint163-store".to_string(),
        paper_only: true,
    }
}

fn sprint163_experience_store() -> MemberExperienceStore {
    MemberExperienceStore::new(
        "sprint163-experience-store",
        vec![
            MemberExperienceRecord {
                experience_id: "aapl-001".to_string(),
                member_id: "TrendEntryAI".to_string(),
                symbol: "AAPL".to_string(),
                market_scope: MarketScope::UsShortTerm,
                cycle_id: None,
                event_id: None,
                session_id: None,
                decision_id: Some("decision-aapl-001".to_string()),
                input_context: MemberExperienceInputContext {
                    market_data_summary: "AAPL market".to_string(),
                    news_summary: "AAPL news".to_string(),
                    owner_context_summary: None,
                    style_blend_summary: None,
                    memory_state_summary: None,
                },
                member_opinion: MemberExperienceOpinionSnapshot {
                    stance: MemberStance::BuyProposal,
                    confidence: 0.7,
                    expected_return_hint: 0.02,
                    risk_hint: 0.2,
                    evidence_notes: vec!["trend trace".to_string()],
                    event_triggered: true,
                },
                committee_context: MemberExperienceCommitteeContext {
                    disagreement_level: 0.0,
                    other_member_stances: Vec::new(),
                    chairman_action: None,
                    risk_governor_status: None,
                    risk_flags: Vec::new(),
                },
                outcome: MemberExperienceOutcome::Unknown,
                attribution: MemberScoreUpdateReason::Neutral,
                learning_label: MemberLearningLabel::Reinforce,
                created_at: Some("2026-05-25T09:00:00Z".to_string()),
                paper_only: true,
            },
            MemberExperienceRecord {
                experience_id: "risk-aapl-001".to_string(),
                member_id: "RiskGuardAI".to_string(),
                symbol: "AAPL".to_string(),
                market_scope: MarketScope::UsShortTerm,
                cycle_id: None,
                event_id: None,
                session_id: None,
                decision_id: Some("decision-risk-aapl-001".to_string()),
                input_context: MemberExperienceInputContext {
                    market_data_summary: "AAPL market".to_string(),
                    news_summary: "AAPL risk".to_string(),
                    owner_context_summary: None,
                    style_blend_summary: None,
                    memory_state_summary: None,
                },
                member_opinion: MemberExperienceOpinionSnapshot {
                    stance: MemberStance::NeedMoreEvidence,
                    confidence: 0.55,
                    expected_return_hint: 0.0,
                    risk_hint: 0.4,
                    evidence_notes: vec!["risk trace".to_string()],
                    event_triggered: false,
                },
                committee_context: MemberExperienceCommitteeContext {
                    disagreement_level: 0.1,
                    other_member_stances: Vec::new(),
                    chairman_action: None,
                    risk_governor_status: None,
                    risk_flags: Vec::new(),
                },
                outcome: MemberExperienceOutcome::Unknown,
                attribution: MemberScoreUpdateReason::Neutral,
                learning_label: MemberLearningLabel::Keep,
                created_at: Some("2026-05-25T09:00:00Z".to_string()),
                paper_only: true,
            },
        ],
    )
}

fn sprint163_price_series_store() -> PaperPriceSeriesStore {
    sprint158_price_series_store()
}

fn sprint163_enrichable_staging_store() -> SelfGrowingEvidenceStagingStore {
    let mut store = sprint162_sample_staging_store();
    store.store_id = "sprint163-enrichable-store".to_string();
    store.candidates[4].related_experience_id = Some("aapl-001".to_string());
    store.candidates[4].related_decision_id = Some("decision-aapl-001".to_string());
    store.candidates[4].candidate_label = None;
    store.candidates[4].reference_price = None;
    store.candidates[4].horizon_price = None;
    store.candidates[4].price_change_pct = None;
    store.candidates[4].evidence_notes = vec!["missing replay link before enrichment".to_string()];
    store
}

#[test]
fn sprint162_staging_store_loads_local_candidates_and_rejects_remote_paths() {
    std::fs::create_dir_all("target").expect("target dir");
    let staging_path = std::path::PathBuf::from("target/sprint162_staging_store.json");
    let store = sprint162_sample_staging_store();
    store
        .save_to_local_json(&staging_path)
        .expect("save staging store");
    let loaded =
        SelfGrowingEvidenceStagingStore::load_from_local_json(&staging_path).expect("load store");
    assert_eq!(loaded.candidate_count, 5);
    assert_eq!(loaded.pending_candidates().len(), 5);
    assert_eq!(loaded.candidates_by_symbol("AAPL").len(), 5);
    let remote_err = SelfGrowingEvidenceStagingStore::load_from_local_json(std::path::Path::new(
        "https://example.invalid/staging.json",
    ))
    .expect_err("remote path rejected");
    assert!(remote_err.contains("local"));
    let mut unsafe_store = store.clone();
    unsafe_store.candidates[0]
        .evidence_notes
        .push("broker account must not be stored".to_string());
    let unsafe_save_err = unsafe_store
        .save_to_local_json(&staging_path)
        .expect_err("unsafe staging candidate rejected on save");
    assert!(unsafe_save_err.contains("unsafe candidate"));
    let _ = std::fs::remove_file(staging_path);
}

#[test]
fn sprint162_candidate_scoring_rewards_exact_replay_link_and_rejects_unsafe_content() {
    let store = sprint162_sample_staging_store();
    let source_scores = sprint162_source_scores();
    let strong_score = score_self_growing_evidence_candidate(&store.candidates[0], &source_scores);
    let low_trust_score =
        score_self_growing_evidence_candidate(&store.candidates[2], &source_scores);
    let missing_link_score =
        score_self_growing_evidence_candidate(&store.candidates[4], &source_scores);
    assert_eq!(
        strong_score.score_status,
        soma_zero::league::minimal_ai_committee_core::EvidenceCandidateScoreStatus::Strong
    );
    assert!(strong_score.replay_link_score > missing_link_score.replay_link_score);
    assert!(strong_score.total_score > low_trust_score.total_score);
    assert_eq!(
        low_trust_score.score_status,
        soma_zero::league::minimal_ai_committee_core::EvidenceCandidateScoreStatus::Weak
    );

    let mut unsafe_candidate = store.candidates[0].clone();
    unsafe_candidate
        .evidence_notes
        .push("broker account should never appear".to_string());
    let unsafe_score = score_self_growing_evidence_candidate(&unsafe_candidate, &source_scores);
    assert_eq!(
        unsafe_score.score_status,
        soma_zero::league::minimal_ai_committee_core::EvidenceCandidateScoreStatus::Rejected
    );

    let mut rejected_source_scores = source_scores.clone();
    rejected_source_scores[0].trust_status = SourceTrustStatus::Rejected;
    rejected_source_scores[0].final_score = 0.99;
    let rejected_source_score =
        score_self_growing_evidence_candidate(&store.candidates[0], &rejected_source_scores);
    assert_eq!(
        rejected_source_score.score_status,
        soma_zero::league::minimal_ai_committee_core::EvidenceCandidateScoreStatus::Rejected
    );
    let rejected_source_decision = evaluate_self_growing_evidence_promotion(
        &store.candidates[0],
        &rejected_source_score,
        &soma_zero::league::minimal_ai_committee_core::SelfGrowingEvidencePromotionPolicy::default(
        ),
    );
    assert_eq!(
        rejected_source_decision.decision,
        SelfGrowingEvidencePromotionDecisionStatus::Rejected
    );
}

#[test]
fn sprint162_promotion_gate_auto_approves_only_exact_price_evidence_and_builds_review_queue() {
    let store = sprint162_sample_staging_store();
    let source_scores = sprint162_source_scores();
    let policy =
        soma_zero::league::minimal_ai_committee_core::SelfGrowingEvidencePromotionPolicy::default();
    let decisions: Vec<_> = store
        .candidates
        .iter()
        .map(|candidate| {
            let score = score_self_growing_evidence_candidate(candidate, &source_scores);
            evaluate_self_growing_evidence_promotion(candidate, &score, &policy)
        })
        .collect();
    assert_eq!(
        decisions[0].decision,
        SelfGrowingEvidencePromotionDecisionStatus::AutoApproved
    );
    assert!(decisions[0].generated_paper_evidence.is_some());
    assert_eq!(
        decisions[1].decision,
        SelfGrowingEvidencePromotionDecisionStatus::NeedsReview
    );
    assert_eq!(
        decisions[3].decision,
        SelfGrowingEvidencePromotionDecisionStatus::NeedsReview
    );
    assert_eq!(
        decisions[4].decision,
        SelfGrowingEvidencePromotionDecisionStatus::NeedsReview
    );

    let review_queue = build_staged_evidence_review_queue(&store.candidates, &decisions);
    assert!(
        review_queue
            .items
            .iter()
            .any(|item| item.review_reason == StagedEvidenceReviewReason::NewsOnlyEvidence)
    );
    assert!(
        review_queue
            .items
            .iter()
            .any(|item| item.review_reason == StagedEvidenceReviewReason::AmbiguousReplayMatch)
    );
    assert!(
        review_queue
            .items
            .iter()
            .any(|item| item.review_reason == StagedEvidenceReviewReason::MissingReplayId)
    );
}

#[test]
fn sprint162_promotion_run_dry_run_is_deterministic_and_writes_nothing() {
    std::fs::create_dir_all("target").expect("target dir");
    let staging_path = std::path::PathBuf::from("target/sprint162_dry_run_staging.json");
    let approved_path = std::path::PathBuf::from("target/sprint162_dry_run_approved.json");
    let training_path = std::path::PathBuf::from("target/sprint162_dry_run_training.json");
    let dataset_path = std::path::PathBuf::from("target/sprint162_dry_run_dataset.json");
    let store = sprint162_sample_staging_store();
    store
        .save_to_local_json(&staging_path)
        .expect("save dry-run staging store");
    let dataset = sprint156_replay_dataset(vec![sprint159_replay_example(
        "replay-aapl-001",
        "AAPL",
        MarketScope::UsShortTerm,
        ReplayLabelSource::ReviewRequired,
        ReplayLabelConfidence::ReviewRequired,
        MemberExperienceOutcome::Unknown,
    )]);
    dataset
        .save_to_local_json(&dataset_path)
        .expect("save dry-run dataset");
    let config = SelfGrowingEvidencePromotionRunConfig {
        run_id: "sprint162-dry-run".to_string(),
        staging_store_input_path: Some(staging_path.display().to_string()),
        staging_store_output_path: Some("target/sprint162_dry_run_staging_after.json".to_string()),
        research_evidence_bundle_path: None,
        source_trust_input_path: None,
        validated_dataset_input_path: Some(dataset_path.display().to_string()),
        sanitized_dataset_input_path: None,
        experience_store_input_path: None,
        approved_paper_evidence_output_path: Some(approved_path.display().to_string()),
        validated_replay_output_path: Some("target/sprint162_dry_run_validated.json".to_string()),
        training_candidate_dataset_output_path: Some(training_path.display().to_string()),
        dry_run: true,
        apply_promotions: true,
        refresh_training_candidates: true,
        paper_only: true,
    };
    let first = run_self_growing_evidence_promotion(config.clone()).expect("first dry-run");
    let second = run_self_growing_evidence_promotion(config).expect("repeat dry-run");
    assert_eq!(first.promotion_decisions, second.promotion_decisions);
    assert_eq!(first.review_queue, second.review_queue);
    assert_eq!(first.auto_approved_count, 1);
    assert!(!approved_path.exists());
    assert!(!training_path.exists());
    let _ = std::fs::remove_file(staging_path);
    let _ = std::fs::remove_file(dataset_path);
}

#[test]
fn sprint162_promotion_run_apply_mode_writes_outputs_and_refreshes_candidates_without_mutating_inputs()
 {
    std::fs::create_dir_all("target").expect("target dir");
    let staging_path = std::path::PathBuf::from("target/sprint162_apply_staging.json");
    let staging_after_path = std::path::PathBuf::from("target/sprint162_apply_staging_after.json");
    let approved_path = std::path::PathBuf::from("target/sprint162_apply_approved.json");
    let training_path = std::path::PathBuf::from("target/sprint162_apply_training.json");
    let validated_path = std::path::PathBuf::from("target/sprint162_apply_validated.json");
    let dataset_path = std::path::PathBuf::from("target/sprint162_apply_dataset.json");
    let store = sprint162_sample_staging_store();
    store
        .save_to_local_json(&staging_path)
        .expect("save apply staging store");
    let dataset = sprint156_replay_dataset(vec![sprint159_replay_example(
        "replay-aapl-001",
        "AAPL",
        MarketScope::UsShortTerm,
        ReplayLabelSource::ReviewRequired,
        ReplayLabelConfidence::ReviewRequired,
        MemberExperienceOutcome::Unknown,
    )]);
    let original_input_features = dataset.examples[0].input_features.clone();
    dataset
        .save_to_local_json(&dataset_path)
        .expect("save apply dataset");

    let run_result = run_self_growing_evidence_promotion(SelfGrowingEvidencePromotionRunConfig {
        run_id: "sprint162-apply".to_string(),
        staging_store_input_path: Some(staging_path.display().to_string()),
        staging_store_output_path: Some(staging_after_path.display().to_string()),
        research_evidence_bundle_path: None,
        source_trust_input_path: None,
        validated_dataset_input_path: Some(dataset_path.display().to_string()),
        sanitized_dataset_input_path: None,
        experience_store_input_path: None,
        approved_paper_evidence_output_path: Some(approved_path.display().to_string()),
        validated_replay_output_path: Some(validated_path.display().to_string()),
        training_candidate_dataset_output_path: Some(training_path.display().to_string()),
        dry_run: false,
        apply_promotions: true,
        refresh_training_candidates: true,
        paper_only: true,
    })
    .expect("apply promotion run");
    assert_eq!(run_result.auto_approved_count, 1);
    assert!(approved_path.exists());
    assert!(training_path.exists());
    assert!(validated_path.exists());
    assert!(run_result.training_candidate_dataset.is_some());
    assert!(run_result.training_inclusion_mask.is_some());
    assert!(run_result.design_gate.is_some());
    let updated_store = SelfGrowingEvidenceStagingStore::load_from_local_json(&staging_after_path)
        .expect("load updated staging store");
    assert!(updated_store.approved_count >= 1);
    assert!(updated_store.review_required_count >= 2);
    let refresh = refresh_training_candidate_dataset_from_promotions(
        &dataset,
        &run_result,
        &TrainingCandidateRefreshPolicy::default(),
    );
    assert_ne!(
        refresh.refresh_status,
        soma_zero::league::minimal_ai_committee_core::TrainingCandidateRefreshStatus::Blocked
    );
    assert_eq!(dataset.examples[0].input_features, original_input_features);

    let _ = std::fs::remove_file(staging_path);
    let _ = std::fs::remove_file(staging_after_path);
    let _ = std::fs::remove_file(approved_path);
    let _ = std::fs::remove_file(training_path);
    let _ = std::fs::remove_file(validated_path);
    let _ = std::fs::remove_file(dataset_path);
}

#[test]
fn sprint163_review_analysis_counts_missing_news_and_low_trust_candidates() {
    let store = sprint162_sample_staging_store();
    let source_scores = sprint162_source_scores();
    let policy =
        soma_zero::league::minimal_ai_committee_core::SelfGrowingEvidencePromotionPolicy::default();
    let decisions: Vec<_> = store
        .candidates
        .iter()
        .map(|candidate| {
            let score = score_self_growing_evidence_candidate(candidate, &source_scores);
            evaluate_self_growing_evidence_promotion(candidate, &score, &policy)
        })
        .collect();
    let analysis = analyze_staged_evidence_review_queue(&store.candidates, &decisions);
    assert_eq!(analysis.auto_approved_count, 1);
    assert_eq!(analysis.missing_replay_link_count, 2);
    assert_eq!(analysis.news_only_count, 1);
    assert_eq!(analysis.low_trust_count, 2);
    assert_eq!(analysis.ambiguous_count, 1);
    assert!(analysis.review_items.iter().any(|item| {
        item.failure_reasons
            .contains(&StagedEvidenceFailureReason::MissingReplayId)
    }));
    assert!(analysis.review_items.iter().any(|item| {
        item.failure_reasons
            .contains(&StagedEvidenceFailureReason::NewsOnlyEvidence)
    }));
}

#[test]
fn sprint163_link_resolution_supports_exact_ids_and_keeps_symbol_scope_only_unsafe() {
    let mut exact_candidate = sprint163_enrichable_staging_store().candidates[4].clone();
    exact_candidate.related_replay_id = None;
    let dataset = sprint163_replay_dataset();
    let experience_store = sprint163_experience_store();
    let exact = resolve_staged_evidence_links(
        &[exact_candidate],
        &dataset,
        &experience_store,
        &StagedEvidenceLinkResolutionPolicy::default(),
    );
    assert_eq!(exact.resolved_count, 1);
    assert!(exact.link_candidates[0].link_safe);
    assert_eq!(
        exact.link_candidates[0].match_type,
        StagedEvidenceLinkMatchType::ExperienceId
    );
    assert_eq!(
        exact.link_candidates[0].suggested_replay_id.as_deref(),
        Some("replay-aapl-001")
    );

    let mut symbol_scope_only = sprint162_sample_staging_store().candidates[4].clone();
    symbol_scope_only.related_experience_id = None;
    symbol_scope_only.related_decision_id = None;
    let symbol_scope = resolve_staged_evidence_links(
        &[symbol_scope_only],
        &dataset,
        &experience_store,
        &StagedEvidenceLinkResolutionPolicy {
            allow_symbol_scope_unique_match: true,
            ..StagedEvidenceLinkResolutionPolicy::default()
        },
    );
    assert_eq!(
        symbol_scope.link_candidates[0].match_type,
        StagedEvidenceLinkMatchType::SymbolScopeOnly
    );
    assert!(!symbol_scope.link_candidates[0].link_safe);
}

#[test]
fn sprint163_price_enrichment_handles_exact_series_mismatch_and_invalid_prices() {
    let mut candidate = sprint163_enrichable_staging_store().candidates[4].clone();
    candidate.related_replay_id = Some("replay-aapl-001".to_string());
    let enriched = enrich_staged_evidence_with_local_price_series(
        &[candidate.clone()],
        &sprint163_price_series_store(),
        &StagedEvidencePriceEnrichmentPolicy::default(),
    );
    assert_eq!(enriched.enriched_count, 1);
    assert_eq!(
        enriched.enrichments[0].enrichment_status,
        StagedEvidencePriceEnrichmentStatus::Enriched
    );
    assert_ne!(
        enriched.enrichments[0].candidate_label,
        MemberExperienceOutcome::Unknown
    );

    let mut mismatch_candidate = candidate.clone();
    mismatch_candidate.market_scope = Some(MarketScope::UsLongTerm);
    let mismatch = enrich_staged_evidence_with_local_price_series(
        &[mismatch_candidate],
        &sprint163_price_series_store(),
        &StagedEvidencePriceEnrichmentPolicy::default(),
    );
    assert_eq!(
        mismatch.enrichments[0].enrichment_status,
        StagedEvidencePriceEnrichmentStatus::SymbolScopeMismatch
    );

    let mut invalid_store = sprint163_price_series_store();
    invalid_store.series[0].bars[0].close = f64::NAN;
    let invalid = enrich_staged_evidence_with_local_price_series(
        &[candidate],
        &invalid_store,
        &StagedEvidencePriceEnrichmentPolicy::default(),
    );
    assert_eq!(
        invalid.enrichments[0].enrichment_status,
        StagedEvidencePriceEnrichmentStatus::InvalidPriceSeries
    );
}

#[test]
fn sprint163_enrichment_patch_adds_links_and_price_evidence_without_mutating_research_text() {
    let store = sprint163_enrichable_staging_store();
    let dataset = sprint163_replay_dataset();
    let experience_store = sprint163_experience_store();
    let original_notes = store.candidates[4].evidence_notes.clone();
    let link_result = resolve_staged_evidence_links(
        &store.candidates,
        &dataset,
        &experience_store,
        &StagedEvidenceLinkResolutionPolicy::default(),
    );
    let mut linked_candidates = store.candidates.clone();
    linked_candidates[4].related_replay_id = Some("replay-aapl-001".to_string());
    linked_candidates[4].related_experience_id = Some("aapl-001".to_string());
    linked_candidates[4].related_decision_id = Some("decision-aapl-001".to_string());
    let price_result = enrich_staged_evidence_with_local_price_series(
        &linked_candidates,
        &sprint163_price_series_store(),
        &StagedEvidencePriceEnrichmentPolicy::default(),
    );
    let patch =
        build_self_growing_candidate_enrichment_patch(&store, link_result, price_result.clone());
    let patched_store = apply_self_growing_candidate_enrichment_patch(&store, &patch);
    let patched = patched_store
        .candidates
        .iter()
        .find(|candidate| candidate.candidate_id == "candidate-missing-replay")
        .expect("patched candidate");
    assert_eq!(store.candidates[4].evidence_notes, original_notes);
    assert_eq!(
        patched.related_replay_id.as_deref(),
        Some("replay-aapl-001")
    );
    assert!(patched.price_change_pct.is_some());
    assert_eq!(
        patched.label_source,
        SelfGrowingEvidenceLabelSource::SelfGrowingPriceEvidence
    );
    assert!(
        patched
            .evidence_notes
            .iter()
            .any(|note| note == "missing replay link before enrichment")
    );
    assert!(
        patch
            .price_enrichments
            .enrichments
            .iter()
            .any(|item| item.enrichment_status == StagedEvidencePriceEnrichmentStatus::Enriched)
    );

    let mut empty_price_store = sprint163_price_series_store();
    empty_price_store.series.clear();
    let link_only_price_result = enrich_staged_evidence_with_local_price_series(
        &linked_candidates,
        &empty_price_store,
        &StagedEvidencePriceEnrichmentPolicy::default(),
    );
    let link_only_patch = build_self_growing_candidate_enrichment_patch(
        &store,
        patch.link_resolutions.clone(),
        link_only_price_result,
    );
    assert!(
        !link_only_patch
            .skipped_candidates
            .contains(&"candidate-missing-replay".to_string())
    );
}

#[test]
fn sprint163_promotion_safety_delta_reports_safe_improvement_and_unsafe_regression() {
    let store = sprint162_sample_staging_store();
    let source_scores = sprint162_source_scores();
    let policy =
        soma_zero::league::minimal_ai_committee_core::SelfGrowingEvidencePromotionPolicy::default();
    let before: Vec<_> = store
        .candidates
        .iter()
        .map(|candidate| {
            let score = score_self_growing_evidence_candidate(candidate, &source_scores);
            evaluate_self_growing_evidence_promotion(candidate, &score, &policy)
        })
        .collect();

    let mut improved_store = sprint163_enrichable_staging_store();
    improved_store.candidates[4].related_replay_id = Some("replay-aapl-001".to_string());
    improved_store.candidates[4].reference_price = Some(100.0);
    improved_store.candidates[4].horizon_price = Some(102.5);
    improved_store.candidates[4].price_change_pct = Some(0.025);
    improved_store.candidates[4].candidate_label = Some(MemberExperienceOutcome::PaperPositive);
    let improved_after: Vec<_> = improved_store
        .candidates
        .iter()
        .map(|candidate| {
            let score = score_self_growing_evidence_candidate(candidate, &source_scores);
            evaluate_self_growing_evidence_promotion(candidate, &score, &policy)
        })
        .collect();
    let improved =
        compute_promotion_safety_delta(&improved_store.candidates, &before, &improved_after);
    assert_eq!(
        improved.delta_status,
        PromotionSafetyDeltaStatus::ImprovedSafely
    );
    assert!(improved.auto_approved_after > improved.auto_approved_before);

    let mut unsafe_after = before.clone();
    unsafe_after[1].decision = SelfGrowingEvidencePromotionDecisionStatus::AutoApproved;
    let unsafe_delta = compute_promotion_safety_delta(&store.candidates, &before, &unsafe_after);
    assert_eq!(
        unsafe_delta.delta_status,
        PromotionSafetyDeltaStatus::UnsafeRegression
    );
    assert!(unsafe_delta.unsafe_approval_count >= 1);

    let mut non_price_store = improved_store.clone();
    non_price_store.candidates[4].evidence_kind = SelfGrowingEvidenceKind::SupportingEvidence;
    let mut non_price_after = improved_after.clone();
    non_price_after[4].decision = SelfGrowingEvidencePromotionDecisionStatus::AutoApproved;
    let non_price_delta =
        compute_promotion_safety_delta(&non_price_store.candidates, &before, &non_price_after);
    assert_eq!(
        non_price_delta.delta_status,
        PromotionSafetyDeltaStatus::UnsafeRegression
    );
}

#[test]
fn sprint163_enriched_promotion_run_is_deterministic_and_preserves_safety_boundaries() {
    std::fs::create_dir_all("target").expect("target dir");
    let staging_path = std::path::PathBuf::from("target/sprint163_enriched_staging_input.json");
    let dataset_path = std::path::PathBuf::from("target/sprint163_enriched_dataset.json");
    let experience_path = std::path::PathBuf::from("target/sprint163_enriched_experience.json");
    let price_path = std::path::PathBuf::from("target/sprint163_enriched_prices.json");
    let enriched_staging_path = std::path::PathBuf::from("target/sprint163_enriched_staging.json");
    let approved_path = std::path::PathBuf::from("target/sprint163_enriched_approved.json");
    let validated_path = std::path::PathBuf::from("target/sprint163_enriched_validated.json");
    let training_path = std::path::PathBuf::from("target/sprint163_enriched_training.json");
    let store = sprint163_enrichable_staging_store();
    store
        .save_to_local_json(&staging_path)
        .expect("save staged evidence");
    let dataset = sprint163_replay_dataset();
    let original_input_features = dataset.examples[0].input_features.clone();
    dataset
        .save_to_local_json(&dataset_path)
        .expect("save replay dataset");
    std::fs::write(
        &experience_path,
        serde_json::to_string_pretty(&sprint163_experience_store()).expect("serialize experience"),
    )
    .expect("write experience store");
    save_price_series_store_to_local_json(&sprint163_price_series_store(), &price_path)
        .expect("save price series");
    let config = EnrichedEvidencePromotionRunConfig {
        run_id: "sprint163-enriched-dry-run".to_string(),
        staging_store_input_path: Some(staging_path.display().to_string()),
        price_series_path: Some(price_path.display().to_string()),
        sanitized_dataset_path: Some(dataset_path.display().to_string()),
        experience_store_path: Some(experience_path.display().to_string()),
        enriched_staging_output_path: Some(enriched_staging_path.display().to_string()),
        approved_evidence_output_path: Some(approved_path.display().to_string()),
        validated_replay_output_path: Some(validated_path.display().to_string()),
        training_candidate_output_path: Some(training_path.display().to_string()),
        dry_run: true,
        apply_enrichment_patch: true,
        apply_promotions: true,
        refresh_training_candidates: true,
        paper_only: true,
    };
    let first = run_enriched_evidence_promotion(config.clone()).expect("first enriched dry-run");
    let second = run_enriched_evidence_promotion(config).expect("second enriched dry-run");
    assert_eq!(first.review_analysis, second.review_analysis);
    assert_eq!(first.link_resolution_result, second.link_resolution_result);
    assert_eq!(
        first.price_enrichment_result,
        second.price_enrichment_result
    );
    assert_eq!(first.safety_delta, second.safety_delta);
    assert!(first.auto_approved_count_after > first.auto_approved_count_before);
    assert_eq!(
        first.safety_delta.delta_status,
        PromotionSafetyDeltaStatus::ImprovedSafely
    );
    assert!(
        first
            .generated_paper_evidence_records
            .iter()
            .all(|record| record.evidence_id != "approved-candidate-news-only")
    );
    assert!(!enriched_staging_path.exists());
    assert!(!approved_path.exists());
    assert!(!validated_path.exists());
    assert!(!training_path.exists());
    assert_eq!(dataset.examples[0].input_features, original_input_features);
    let serialized = serde_json::to_string(&(
        &first.enrichment_patch_result,
        &first.generated_paper_evidence_records,
    ))
    .expect("serialize enriched evidence artifacts");
    for forbidden in ["training run", "broker", "order", "account"] {
        assert!(
            !serialized.to_ascii_lowercase().contains(forbidden),
            "unexpected forbidden fragment: {forbidden}"
        );
    }
    assert!(first.safety_summary.no_model_training);
    assert!(first.safety_summary.no_live_inference);
    assert!(first.safety_summary.no_broker_order_account);
    let _ = std::fs::remove_file(staging_path);
    let _ = std::fs::remove_file(dataset_path);
    let _ = std::fs::remove_file(experience_path);
    let _ = std::fs::remove_file(price_path);
}

#[test]
fn sprint163_enriched_apply_writes_local_outputs_and_refreshes_from_approved_evidence_only() {
    std::fs::create_dir_all("target").expect("target dir");
    let staging_path = std::path::PathBuf::from("target/sprint163_apply_staging_input.json");
    let dataset_path = std::path::PathBuf::from("target/sprint163_apply_dataset.json");
    let experience_path = std::path::PathBuf::from("target/sprint163_apply_experience.json");
    let price_path = std::path::PathBuf::from("target/sprint163_apply_prices.json");
    let enriched_staging_path = std::path::PathBuf::from("target/sprint163_apply_staging.json");
    let approved_path = std::path::PathBuf::from("target/sprint163_apply_approved.json");
    let validated_path = std::path::PathBuf::from("target/sprint163_apply_validated.json");
    let training_path = std::path::PathBuf::from("target/sprint163_apply_training.json");
    sprint163_enrichable_staging_store()
        .save_to_local_json(&staging_path)
        .expect("save staging");
    sprint163_replay_dataset()
        .save_to_local_json(&dataset_path)
        .expect("save dataset");
    std::fs::write(
        &experience_path,
        serde_json::to_string_pretty(&sprint163_experience_store()).expect("serialize experience"),
    )
    .expect("write experience");
    save_price_series_store_to_local_json(&sprint163_price_series_store(), &price_path)
        .expect("save prices");
    let result = run_enriched_evidence_promotion(EnrichedEvidencePromotionRunConfig {
        run_id: "sprint163-enriched-apply".to_string(),
        staging_store_input_path: Some(staging_path.display().to_string()),
        price_series_path: Some(price_path.display().to_string()),
        sanitized_dataset_path: Some(dataset_path.display().to_string()),
        experience_store_path: Some(experience_path.display().to_string()),
        enriched_staging_output_path: Some(enriched_staging_path.display().to_string()),
        approved_evidence_output_path: Some(approved_path.display().to_string()),
        validated_replay_output_path: Some(validated_path.display().to_string()),
        training_candidate_output_path: Some(training_path.display().to_string()),
        dry_run: false,
        apply_enrichment_patch: true,
        apply_promotions: true,
        refresh_training_candidates: true,
        paper_only: true,
    })
    .expect("run enriched apply");
    assert!(enriched_staging_path.exists());
    assert!(approved_path.exists());
    assert!(validated_path.exists());
    assert!(training_path.exists());
    assert!(result.training_candidate_refresh_result.is_some());
    assert!(
        result
            .generated_paper_evidence_records
            .iter()
            .all(|record| record.replay_id.is_some())
    );
    assert_eq!(
        result.safety_delta.delta_status,
        PromotionSafetyDeltaStatus::ImprovedSafely
    );
    let _ = std::fs::remove_file(staging_path);
    let _ = std::fs::remove_file(dataset_path);
    let _ = std::fs::remove_file(experience_path);
    let _ = std::fs::remove_file(price_path);
    let _ = std::fs::remove_file(enriched_staging_path);
    let _ = std::fs::remove_file(approved_path);
    let _ = std::fs::remove_file(validated_path);
    let _ = std::fs::remove_file(training_path);
}

#[test]
fn sprint164_success_samples_load_and_dry_run_proves_first_safe_auto_approval_path() {
    let success_store = SelfGrowingEvidenceStagingStore::load_from_local_json(
        std::path::Path::new("examples/self_growing_evidence_staging.success.sample.json"),
    )
    .expect("load success staging sample");
    let success_prices = load_price_series_store_from_local_json(std::path::Path::new(
        "examples/minimal_self_growing_price_series.success.sample.json",
    ))
    .expect("load success price sample");
    assert_eq!(success_store.candidate_count, 5);
    assert_eq!(success_prices.series.len(), 1);

    std::fs::create_dir_all("target").expect("target dir");
    let dataset_path = std::path::PathBuf::from("target/sprint164_success_dataset.json");
    let experience_path = std::path::PathBuf::from("target/sprint164_success_experience.json");
    sprint163_replay_dataset()
        .save_to_local_json(&dataset_path)
        .expect("save success dataset");
    std::fs::write(
        &experience_path,
        serde_json::to_string_pretty(&sprint163_experience_store()).expect("serialize experience"),
    )
    .expect("write success experience");

    let result = run_auto_approval_end_to_end(AutoApprovalEndToEndRunConfig {
        run_id: "sprint164-success-dry-run".to_string(),
        success_staging_path: "examples/self_growing_evidence_staging.success.sample.json"
            .to_string(),
        success_price_series_path: "examples/minimal_self_growing_price_series.success.sample.json"
            .to_string(),
        sanitized_dataset_path: Some(dataset_path.display().to_string()),
        experience_store_path: Some(experience_path.display().to_string()),
        validated_dataset_path: None,
        approved_evidence_output_path: Some(
            "target/sprint164_success_approved_should_not_exist.json".to_string(),
        ),
        training_candidate_output_path: Some(
            "target/sprint164_success_training_should_not_exist.json".to_string(),
        ),
        dry_run: true,
        apply_promotions: false,
        refresh_training_candidates: false,
        paper_only: true,
    })
    .expect("run sprint164 dry-run");
    assert_eq!(
        result.enriched_promotion_result.auto_approved_count_before,
        0
    );
    assert!(result.enriched_promotion_result.auto_approved_count_after >= 1);
    assert_eq!(
        result.enriched_promotion_result.safety_delta.delta_status,
        PromotionSafetyDeltaStatus::ImprovedSafely
    );
    assert!(
        result
            .enriched_promotion_result
            .generated_paper_evidence_records
            .iter()
            .any(|record| record.evidence_id == "approved-self-grow-price-success-001")
    );
    let success_check = result
        .success_path_checks
        .iter()
        .find(|check| check.candidate_id == "self-grow-price-success-001")
        .expect("success path check");
    assert!(success_check.promotion_allowed);
    assert!(success_check.price_evidence_present);
    assert!(success_check.source_trust_ok);
    assert!(success_check.evidence_kind_ok);
    assert!(success_check.label_ok);
    assert!(success_check.safety_ok);
    assert!(success_check.failure_reasons.is_empty());
    assert_eq!(
        success_check.exact_link_type,
        AutoApprovalSuccessExactLinkType::ReplayId
    );
    for blocked_evidence_id in [
        "approved-self-grow-news-review-001",
        "approved-self-grow-low-trust-review-001",
        "approved-self-grow-ambiguous-review-001",
        "approved-self-grow-missing-link-review-001",
    ] {
        assert!(
            result
                .enriched_promotion_result
                .generated_paper_evidence_records
                .iter()
                .all(|record| record.evidence_id != blocked_evidence_id),
            "blocked evidence was generated: {blocked_evidence_id}"
        );
    }
    for (blocked_id, expected_reason) in [
        (
            "self-grow-news-review-001",
            "evidence_kind is not price evidence",
        ),
        (
            "self-grow-low-trust-review-001",
            "source trust below promotion threshold",
        ),
        (
            "self-grow-ambiguous-review-001",
            "promotion gate still blocks candidate",
        ),
        (
            "self-grow-missing-link-review-001",
            "missing exact replay link",
        ),
    ] {
        let blocked_check = result
            .success_path_checks
            .iter()
            .find(|check| check.candidate_id == blocked_id)
            .expect("blocked candidate check");
        assert!(!blocked_check.promotion_allowed);
        assert!(
            blocked_check
                .failure_reasons
                .iter()
                .any(|reason| reason == expected_reason),
            "missing expected reason {expected_reason} for {blocked_id}: {:?}",
            blocked_check.failure_reasons
        );
    }
    assert!(result.safety_summary.no_model_training);
    assert!(result.safety_summary.no_live_inference);
    assert!(result.safety_summary.no_broker_order_account);
    assert!(
        !std::path::Path::new("target/sprint164_success_approved_should_not_exist.json").exists()
    );
    assert!(
        !std::path::Path::new("target/sprint164_success_training_should_not_exist.json").exists()
    );
    let _ = std::fs::remove_file(dataset_path);
    let _ = std::fs::remove_file(experience_path);
}

#[test]
fn sprint164_apply_approved_evidence_refreshes_outputs_and_keeps_input_features_unchanged() {
    std::fs::create_dir_all("target").expect("target dir");
    let dataset_path = std::path::PathBuf::from("target/sprint164_apply_dataset.json");
    let experience_path = std::path::PathBuf::from("target/sprint164_apply_experience.json");
    let approved_path = std::path::PathBuf::from("target/sprint164_apply_approved.json");
    let training_path = std::path::PathBuf::from("target/sprint164_apply_training.json");
    let dataset = sprint163_replay_dataset();
    let original_input_features = dataset.examples[0].input_features.clone();
    dataset
        .save_to_local_json(&dataset_path)
        .expect("save apply dataset");
    std::fs::write(
        &experience_path,
        serde_json::to_string_pretty(&sprint163_experience_store()).expect("serialize experience"),
    )
    .expect("write apply experience");
    let result = run_auto_approval_end_to_end(AutoApprovalEndToEndRunConfig {
        run_id: "sprint164-success-apply".to_string(),
        success_staging_path: "examples/self_growing_evidence_staging.success.sample.json"
            .to_string(),
        success_price_series_path: "examples/minimal_self_growing_price_series.success.sample.json"
            .to_string(),
        sanitized_dataset_path: Some(dataset_path.display().to_string()),
        experience_store_path: Some(experience_path.display().to_string()),
        validated_dataset_path: None,
        approved_evidence_output_path: Some(approved_path.display().to_string()),
        training_candidate_output_path: Some(training_path.display().to_string()),
        dry_run: false,
        apply_promotions: true,
        refresh_training_candidates: true,
        paper_only: true,
    })
    .expect("run sprint164 apply");
    let apply_result = result
        .approved_evidence_apply_result
        .as_ref()
        .expect("approved evidence apply result");
    assert!(approved_path.exists());
    assert!(training_path.exists());
    assert!(apply_result.validated_replay_refreshed);
    assert!(apply_result.training_candidates_refreshed);
    assert!(apply_result.input_features_unchanged);
    assert_eq!(dataset.examples[0].input_features, original_input_features);
    assert!(
        apply_result.new_training_candidate_count.unwrap_or(0)
            >= apply_result.previous_training_candidate_count.unwrap_or(0)
    );
    let approved_records: Vec<PaperOutcomeEvidenceRecord> = serde_json::from_str(
        &std::fs::read_to_string(&approved_path).expect("read approved evidence"),
    )
    .expect("parse approved evidence");
    assert!(
        approved_records
            .iter()
            .all(|record| record.evidence_id != "approved-self-grow-news-review-001")
    );
    let training_dataset: TrainingCandidateDataset = serde_json::from_str(
        &std::fs::read_to_string(&training_path).expect("read training candidates"),
    )
    .expect("parse training candidates");
    assert!(
        training_dataset.examples.iter().all(|example| matches!(
            example.label_source,
            ReplayLabelSource::ValidatedPaperLabel | ReplayLabelSource::ValidatedBacktestLabel
        )),
        "training refresh must use validated labels only"
    );
    assert!(
        !training_dataset
            .label_source_distribution
            .contains_key("ReviewRequired")
    );
    assert!(
        !training_dataset
            .label_source_distribution
            .contains_key("AmbiguousLabel")
    );
    let _ = std::fs::remove_file(dataset_path);
    let _ = std::fs::remove_file(experience_path);
    let _ = std::fs::remove_file(approved_path);
    let _ = std::fs::remove_file(training_path);
}

#[test]
fn sprint164_apply_function_rejects_unsafe_records_and_success_metrics_keep_blocked_counts() {
    std::fs::create_dir_all("target").expect("target dir");
    let dataset = sprint163_replay_dataset();
    dataset
        .save_to_local_json(std::path::Path::new(
            "target/sprint164_metrics_dataset.json",
        ))
        .expect("save metrics dataset");
    std::fs::write(
        "target/sprint164_metrics_experience.json",
        serde_json::to_string_pretty(&sprint163_experience_store()).expect("serialize experience"),
    )
    .expect("write metrics experience");
    let result = run_auto_approval_end_to_end(AutoApprovalEndToEndRunConfig {
        run_id: "sprint164-metrics-dry-run".to_string(),
        success_staging_path: "examples/self_growing_evidence_staging.success.sample.json"
            .to_string(),
        success_price_series_path: "examples/minimal_self_growing_price_series.success.sample.json"
            .to_string(),
        sanitized_dataset_path: Some("target/sprint164_metrics_dataset.json".to_string()),
        experience_store_path: Some("target/sprint164_metrics_experience.json".to_string()),
        validated_dataset_path: None,
        approved_evidence_output_path: None,
        training_candidate_output_path: None,
        dry_run: true,
        apply_promotions: false,
        refresh_training_candidates: false,
        paper_only: true,
    })
    .expect("run metrics dry-run");
    assert!(result.promotion_success_metrics.auto_approval_delta > 0);
    assert!(
        result
            .promotion_success_metrics
            .generated_paper_evidence_count
            >= 1
    );
    assert!(result.promotion_success_metrics.news_only_blocked_count >= 1);
    assert!(result.promotion_success_metrics.low_trust_blocked_count >= 1);
    assert!(result.promotion_success_metrics.ambiguous_blocked_count >= 1);
    assert!(result.promotion_success_metrics.missing_link_blocked_count >= 1);

    let mut unsafe_records = result
        .enriched_promotion_result
        .generated_paper_evidence_records
        .clone();
    unsafe_records.push(PaperOutcomeEvidenceRecord {
        evidence_id: "approved-unsafe".to_string(),
        symbol: "AAPL".to_string(),
        market_scope: MarketScope::UsShortTerm,
        decision_id: Some("decision-aapl-001".to_string()),
        event_id: None,
        replay_id: Some("replay-aapl-001".to_string()),
        experience_id: Some("aapl-001".to_string()),
        horizon: PaperOutcomeEvidenceHorizon::ShortTerm,
        horizon_bars: Some(2),
        reference_price: Some(100.0),
        horizon_price: Some(103.4),
        price_change_pct: Some(0.034),
        candidate_label: MemberExperienceOutcome::PaperPositive,
        label_source: ReplayLabelSource::ManualPaperLabel,
        label_confidence: ReplayLabelConfidence::High,
        evidence_notes: vec!["broker account should never appear".to_string()],
        validation_hint: Some(PaperOutcomeEvidenceValidationHint::PromoteIfPolicyPasses),
        paper_only: true,
    });
    let apply_result = apply_approved_evidence_to_training_candidates(
        ApprovedEvidenceApplyConfig {
            run_id: "sprint164-unsafe-apply".to_string(),
            approved_evidence_input_path: None,
            approved_evidence_output_path: None,
            validated_replay_input_path: None,
            validated_replay_output_path: None,
            training_candidate_output_path: None,
            apply_mode: ApprovedEvidenceApplyMode::DryRun,
            refresh_validated_replay: true,
            refresh_training_candidates: true,
            paper_only: true,
        },
        &unsafe_records,
        &dataset,
    )
    .expect("apply approved evidence dry-run");
    assert_eq!(
        apply_result.apply_status,
        ApprovedEvidenceApplyStatus::DryRunPreview
    );
    assert!(apply_result.rejected_evidence_count >= 1);
    let _ = std::fs::remove_file("target/sprint164_metrics_dataset.json");
    let _ = std::fs::remove_file("target/sprint164_metrics_experience.json");
}

#[test]
fn sprint164_success_check_matches_manual_candidate_gate() {
    let mut candidate = SelfGrowingEvidenceStagingStore::load_from_local_json(
        std::path::Path::new("examples/self_growing_evidence_staging.success.sample.json"),
    )
    .expect("load success sample")
    .candidates
    .into_iter()
    .find(|candidate| candidate.candidate_id == "self-grow-price-success-001")
    .expect("success candidate");
    candidate.reference_price = Some(100.0);
    candidate.horizon_price = Some(103.4);
    candidate.price_change_pct = Some(0.034);
    candidate.candidate_label = Some(MemberExperienceOutcome::PaperPositive);
    candidate.label_source = SelfGrowingEvidenceLabelSource::SelfGrowingPriceEvidence;
    let enrichment = StagedEvidencePriceEnrichment {
        candidate_id: candidate.candidate_id.clone(),
        replay_id: candidate.related_replay_id.clone(),
        symbol: "AAPL".to_string(),
        market_scope: MarketScope::UsShortTerm,
        reference_price: Some(100.0),
        horizon_price: Some(103.4),
        price_change_pct: Some(0.034),
        candidate_label: MemberExperienceOutcome::PaperPositive,
        enrichment_status: StagedEvidencePriceEnrichmentStatus::Enriched,
        evidence_notes: vec!["deterministic success enrichment".to_string()],
        paper_only: true,
    };
    let score = score_self_growing_evidence_candidate(&candidate, &[]);
    let success = check_auto_approval_success_path(
        &candidate,
        &score,
        Some(&enrichment),
        &soma_zero::league::minimal_ai_committee_core::SelfGrowingEvidencePromotionPolicy::default(
        ),
    );
    assert!(success.promotion_allowed);
    assert_eq!(
        success.exact_link_type,
        AutoApprovalSuccessExactLinkType::ReplayId
    );
    let metrics = compute_promotion_success_metrics(
        0,
        1,
        1,
        None,
        &StagedEvidenceReviewAnalysis {
            analysis_id: "metrics".to_string(),
            candidate_count: 5,
            auto_approved_count: 0,
            needs_review_count: 4,
            rejected_count: 0,
            missing_replay_link_count: 1,
            news_only_count: 1,
            low_trust_count: 1,
            ambiguous_count: 1,
            missing_price_evidence_count: 1,
            unsafe_count: 0,
            review_items: Vec::new(),
            paper_only: true,
        },
        PromotionSafetyDeltaStatus::ImprovedSafely,
    );
    assert_eq!(metrics.auto_approval_delta, 1);
    assert_eq!(
        metrics.safety_delta_status,
        PromotionSafetyDeltaStatus::ImprovedSafely
    );
}
