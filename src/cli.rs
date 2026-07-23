use std::{
    collections::BTreeSet,
    fmt::Write as _,
    fs,
    path::{Path, PathBuf},
};

use clap::Parser;
use serde::Serialize;

use crate::{
    ChairEngine, MarketSnapshot, MockSignalEngine, PaperBroker, Regime, RiskGovernor, RiskSnapshot,
    simulate_paper_cycle,
};

#[derive(Debug, Parser)]
#[command(name = "soma-zero", about = "Paper-only deterministic trading OS MVP")]
pub struct CliArgs {
    #[arg(long, default_value = "BTCUSDT")]
    pub symbol: String,
    #[arg(long, default_value_t = false)]
    pub full_auto: bool,
    #[arg(long)]
    pub historical_provider_smoke_config: Option<PathBuf>,
    #[arg(long)]
    pub historical_snapshot_campaign_config: Option<PathBuf>,
    #[arg(long, default_value_t = false)]
    pub momentum_temporal_diagnostics: bool,
    #[arg(long, default_value_t = false)]
    pub momentum_cross_market_report: bool,
    #[arg(long, default_value_t = false)]
    pub btc_multi_regime_report: bool,
    #[arg(long, default_value_t = false)]
    pub btc_cross_regime_diagnostics: bool,
    #[arg(long, default_value_t = false)]
    pub btc_cycle_risk_shadow_report: bool,
    #[arg(long, default_value_t = false)]
    pub learned_agent_shadow_deliberation: bool,
    #[arg(long, default_value_t = false)]
    pub learned_agent_scope_alignment: bool,
    #[arg(long, default_value_t = false)]
    pub joint_canonical_scope_replay: bool,
    #[arg(long, default_value_t = false)]
    pub joint_momentum_failure_forensics: bool,
    #[arg(long, default_value_t = false)]
    pub joint_canonical_scope_replay_v2: bool,
    #[arg(long, default_value_t = false)]
    pub joint_momentum_closure_forensics_v3: bool,
    #[arg(long, default_value_t = false)]
    pub joint_canonical_scope_registration_v3: bool,
    #[arg(long, default_value_t = false)]
    pub joint_canonical_scope_replay_v3: bool,
    #[arg(long, default_value_t = false)]
    pub chair_shadow_observation_inbox: bool,
    #[arg(long, default_value_t = false)]
    pub chair_shadow_owner_advisory_review: bool,
    #[arg(long, default_value_t = false)]
    pub learned_reward_eligibility: bool,
    #[arg(long, default_value_t = false)]
    pub prospective_external_row_admission: bool,
    #[arg(long, default_value_t = false)]
    pub acquire_one_upbit_prospective_candle: bool,
    #[arg(long, default_value_t = false)]
    pub register_prospective_outcome_opening: bool,
    #[arg(long, default_value_t = false)]
    pub prospective_event_maturity_preflight: bool,
    #[arg(long, default_value_t = false)]
    pub prospective_outcome_acquisition: bool,
    #[arg(long, default_value_t = false)]
    pub prospective_outcome_opening: bool,
    #[arg(long, default_value_t = false)]
    pub agent_private_learning_sessions: bool,
    #[arg(long, default_value_t = false)]
    pub agent_candidate_evidence_audit: bool,
    #[arg(long, default_value_t = false)]
    pub register_agent_candidate_evaluation: bool,
    #[arg(long, default_value_t = false)]
    pub agent_private_learning_candidates_v1: bool,
    #[arg(long, default_value_t = false)]
    pub register_agent_candidate_evaluation_v1: bool,
    #[arg(long, default_value_t = false)]
    pub agent_canonical_view_gap_v1: bool,
    #[arg(long, default_value_t = false)]
    pub migrate_persisted_learning_intent_v1: bool,
    #[arg(long, default_value_t = false)]
    pub momentum_mamba_repair_v2: bool,
    #[arg(long, default_value_t = false)]
    pub momentum_mamba_representation_v3: bool,
    #[arg(long, default_value_t = false)]
    pub momentum_raw_feature_v4: bool,
    #[arg(long, default_value_t = false)]
    pub status: bool,
    #[arg(long, default_value_t = false)]
    pub dry_run: bool,
    #[arg(long, default_value_t = false)]
    pub execute_local: bool,
    #[arg(long, default_value_t = false)]
    pub execute: bool,
    #[arg(long, default_value_t = false)]
    pub confirm_single_public_candle_request: bool,
    #[arg(long, default_value_t = false)]
    pub confirm_one_time_outcome_request: bool,
    #[arg(long, default_value_t = false)]
    pub confirm_one_time_prospective_opening: bool,
    #[arg(long, default_value_t = false)]
    pub confirm_one_time_learning_evidence_request: bool,
    #[arg(long, default_value_t = false)]
    pub confirm_composite_learning_evidence_epoch: bool,
    #[arg(long, default_value_t = false)]
    pub btc_prospective_challenge_create: bool,
    #[arg(long, default_value_t = false)]
    pub btc_prospective_challenge_status: bool,
    #[arg(long, default_value_t = false)]
    pub btc_prospective_challenge_confirm_preregistration: bool,
    #[arg(long, default_value_t = false)]
    pub btc_prospective_registry_close: bool,
    #[arg(long, default_value_t = false)]
    pub btc_prospective_accumulate: bool,
    #[arg(long, default_value_t = false)]
    pub btc_prospective_evaluate: bool,
    #[arg(long, default_value_t = false)]
    pub toss_historical_contract_report: bool,
    #[arg(long)]
    pub toss_kr_historical_manifest: Option<PathBuf>,
    #[arg(long)]
    pub toss_us_historical_manifest: Option<PathBuf>,
    #[arg(long, default_value = "text", value_parser = ["text", "json"])]
    pub output_format: String,
    #[arg(long, default_value_t = false)]
    pub allow_network: bool,
}

pub fn run() -> Result<(), String> {
    let args = CliArgs::parse();
    if args.momentum_raw_feature_v4 {
        if args.execute
            || args.confirm_single_public_candle_request
            || args.confirm_one_time_outcome_request
            || args.confirm_one_time_prospective_opening
            || args.confirm_one_time_learning_evidence_request
            || args.confirm_composite_learning_evidence_epoch
        {
            return Err("Momentum raw-feature V4 rejects network authority flags".into());
        }
        let config = args
            .historical_snapshot_campaign_config
            .as_deref()
            .ok_or_else(|| {
                "Momentum raw-feature V4 requires a local historical provider config".to_string()
            })?;
        return run_momentum_raw_feature_cli_v4(
            config,
            &args.output_format,
            args.status,
            args.dry_run,
            args.execute_local,
            args.allow_network,
        );
    }
    if args.momentum_mamba_representation_v3 {
        if args.execute
            || args.confirm_single_public_candle_request
            || args.confirm_one_time_outcome_request
            || args.confirm_one_time_prospective_opening
            || args.confirm_one_time_learning_evidence_request
            || args.confirm_composite_learning_evidence_epoch
        {
            return Err("Momentum Mamba representation V3 rejects network authority flags".into());
        }
        let config = args
            .historical_snapshot_campaign_config
            .as_deref()
            .ok_or_else(|| {
                "Momentum Mamba representation V3 requires a local historical provider config"
                    .to_string()
            })?;
        return run_momentum_mamba_representation_cli_v3(
            config,
            &args.output_format,
            args.status,
            args.dry_run,
            args.execute_local,
            args.allow_network,
        );
    }
    if args.momentum_mamba_repair_v2 {
        if args.execute
            || args.confirm_single_public_candle_request
            || args.confirm_one_time_outcome_request
            || args.confirm_one_time_prospective_opening
            || args.confirm_one_time_learning_evidence_request
            || args.confirm_composite_learning_evidence_epoch
        {
            return Err("Momentum Mamba repair rejects network authority flags".into());
        }
        let config = args
            .historical_snapshot_campaign_config
            .as_deref()
            .ok_or_else(|| {
                "Momentum Mamba repair requires a local historical provider config".to_string()
            })?;
        return run_momentum_mamba_repair_cli_v2(
            config,
            &args.output_format,
            args.status,
            args.dry_run,
            args.execute_local,
            args.allow_network,
        );
    }
    if args.migrate_persisted_learning_intent_v1 {
        if args.execute
            || args.confirm_single_public_candle_request
            || args.confirm_one_time_outcome_request
            || args.confirm_one_time_prospective_opening
            || args.confirm_one_time_learning_evidence_request
            || args.confirm_composite_learning_evidence_epoch
        {
            return Err(
                "persisted learning intent migration rejects network authority flags".into(),
            );
        }
        let config = args
            .historical_snapshot_campaign_config
            .as_deref()
            .ok_or_else(|| {
                "persisted learning intent migration requires a local historical provider config"
                    .to_string()
            })?;
        return run_persisted_learning_intent_migration_cli_v1(
            config,
            &args.output_format,
            args.status,
            args.dry_run,
            args.execute_local,
            args.allow_network,
        );
    }
    if args.agent_canonical_view_gap_v1 {
        let config = args
            .historical_snapshot_campaign_config
            .as_deref()
            .ok_or_else(|| {
                "canonical view gap audit requires a local historical provider config".to_string()
            })?;
        return run_agent_canonical_view_gap_cli_v1(
            config,
            &args.output_format,
            args.status,
            args.dry_run,
            args.execute_local,
            args.allow_network,
            args.confirm_one_time_learning_evidence_request,
            args.confirm_composite_learning_evidence_epoch,
        );
    }
    if args.agent_private_learning_candidates_v1 && args.register_agent_candidate_evaluation_v1 {
        return Err("select exactly one V1 candidate or registration command".to_string());
    }
    if args.agent_private_learning_candidates_v1 || args.register_agent_candidate_evaluation_v1 {
        return run_agent_candidate_family_cli_v1(
            &args.output_format,
            args.status,
            args.dry_run,
            args.execute_local,
            args.allow_network,
            args.register_agent_candidate_evaluation_v1,
        );
    }
    if args.agent_candidate_evidence_audit && args.register_agent_candidate_evaluation {
        return Err("select exactly one candidate audit or registration command".to_string());
    }
    if args.agent_candidate_evidence_audit || args.register_agent_candidate_evaluation {
        return run_agent_candidate_evaluation_cli_v0(
            &args.output_format,
            args.status,
            args.dry_run,
            args.execute_local,
            args.allow_network,
            args.register_agent_candidate_evaluation,
        );
    }
    if args.agent_private_learning_sessions {
        return run_agent_private_learning_sessions_cli_v0(
            &args.output_format,
            args.status,
            args.dry_run,
            args.execute_local,
            args.allow_network,
        );
    }
    if args.prospective_outcome_opening {
        let config = args
            .historical_snapshot_campaign_config
            .as_deref()
            .ok_or_else(|| {
                "prospective outcome opening requires a local historical provider config"
                    .to_string()
            })?;
        return run_prospective_outcome_opening_v0(
            config,
            &args.output_format,
            args.status,
            args.dry_run,
            args.execute_local,
            args.allow_network,
            args.confirm_one_time_prospective_opening,
        );
    }
    if args.confirm_one_time_prospective_opening {
        return Err(
            "prospective opening confirmation requires --prospective-outcome-opening".into(),
        );
    }
    if args.execute_local {
        return Err(
            "--execute-local requires an offline learning audit or session command".to_string(),
        );
    }
    if (args.status || args.confirm_one_time_outcome_request)
        && !args.prospective_outcome_acquisition
    {
        return Err(
            "prospective outcome mode flags require --prospective-outcome-acquisition".into(),
        );
    }
    if args.confirm_one_time_learning_evidence_request {
        return Err("learning evidence confirmation requires --agent-canonical-view-gap-v1".into());
    }
    if args.confirm_composite_learning_evidence_epoch {
        return Err(
            "composite learning confirmation requires --agent-canonical-view-gap-v1".into(),
        );
    }
    if args.toss_historical_contract_report {
        return print_toss_historical_contract_report(
            args.toss_kr_historical_manifest,
            args.toss_us_historical_manifest,
        );
    }
    if args.register_prospective_outcome_opening && args.prospective_event_maturity_preflight {
        return Err("select exactly one prospective outcome-opening action".to_string());
    }
    if let Some(config) = args.historical_snapshot_campaign_config {
        if args.prospective_outcome_acquisition {
            return run_prospective_outcome_acquisition(
                &config,
                &args.output_format,
                args.status,
                args.dry_run,
                args.execute,
                args.allow_network,
                args.confirm_one_time_outcome_request,
            );
        }
        if args.register_prospective_outcome_opening {
            return run_prospective_outcome_opening_registration(
                &config,
                &args.output_format,
                args.allow_network,
            );
        }
        if args.prospective_event_maturity_preflight {
            return run_prospective_event_maturity_preflight(
                &config,
                &args.output_format,
                args.allow_network,
            );
        }
        return run_local_historical_snapshot_campaign(
            &config,
            args.momentum_temporal_diagnostics || args.momentum_cross_market_report,
            &args.output_format,
            args.momentum_cross_market_report,
            args.btc_multi_regime_report,
            args.btc_cross_regime_diagnostics,
            args.btc_cycle_risk_shadow_report,
            args.learned_agent_shadow_deliberation,
            args.learned_agent_scope_alignment,
            args.joint_canonical_scope_replay,
            args.joint_momentum_failure_forensics,
            args.joint_canonical_scope_replay_v2,
            args.joint_momentum_closure_forensics_v3,
            args.joint_canonical_scope_registration_v3,
            args.joint_canonical_scope_replay_v3,
            args.chair_shadow_observation_inbox,
            args.chair_shadow_owner_advisory_review,
            args.learned_reward_eligibility,
            args.prospective_external_row_admission,
            args.acquire_one_upbit_prospective_candle,
            args.dry_run,
            args.execute,
            args.confirm_single_public_candle_request,
            args.btc_prospective_challenge_create,
            args.btc_prospective_challenge_status,
            args.btc_prospective_challenge_confirm_preregistration,
            args.btc_prospective_registry_close,
            args.btc_prospective_accumulate,
            args.btc_prospective_evaluate,
            args.allow_network,
        );
    }
    if args.momentum_temporal_diagnostics
        || args.momentum_cross_market_report
        || args.btc_multi_regime_report
        || args.btc_cross_regime_diagnostics
        || args.btc_cycle_risk_shadow_report
        || args.learned_agent_shadow_deliberation
        || args.learned_agent_scope_alignment
        || args.joint_canonical_scope_replay
        || args.joint_momentum_failure_forensics
        || args.joint_canonical_scope_replay_v2
        || args.joint_momentum_closure_forensics_v3
        || args.joint_canonical_scope_registration_v3
        || args.joint_canonical_scope_replay_v3
        || args.chair_shadow_observation_inbox
        || args.chair_shadow_owner_advisory_review
        || args.learned_reward_eligibility
        || args.prospective_external_row_admission
        || args.acquire_one_upbit_prospective_candle
        || args.register_prospective_outcome_opening
        || args.prospective_event_maturity_preflight
        || args.prospective_outcome_acquisition
        || args.status
        || args.btc_prospective_challenge_create
        || args.btc_prospective_challenge_status
        || args.btc_prospective_challenge_confirm_preregistration
        || args.btc_prospective_registry_close
        || args.btc_prospective_accumulate
        || args.btc_prospective_evaluate
    {
        return Err(
            "temporal diagnostics require a local historical snapshot campaign config".to_string(),
        );
    }
    if let Some(config) = args.historical_provider_smoke_config {
        if !args.allow_network {
            return Err("historical provider smoke requires --allow-network".to_string());
        }
        let campaign_config = crate::model::MomentumLearningCampaignConfigV0::default();
        let campaign_required_rows =
            crate::model::assess_momentum_campaign_sufficiency_v0(0, &campaign_config)
                .map_err(|_| "momentum campaign sufficiency configuration invalid".to_string())?
                .required_minimum_rows;
        let result = crate::data::run_manual_upbit_historical_backfill_v0(
            &config,
            true,
            campaign_required_rows,
        );
        println!("historical_provider_status={:?}", result.status);
        println!("provider={}", result.provider_id.unwrap_or_default());
        println!("rows={}", result.row_count);
        println!("snapshot_id={}", result.snapshot_id.unwrap_or_default());
        println!("pages={}", result.page_receipts.len());
        println!(
            "snapshot_digest_prefix={}",
            result
                .snapshot_digest
                .as_deref()
                .unwrap_or_default()
                .chars()
                .take(12)
                .collect::<String>()
        );
        println!("reasons={}", result.reason_codes.join("|"));
        if let Some(path) = result.local_snapshot_path {
            let snapshot = crate::data::read_local_snapshot_protobuf_v1(Path::new(&path))?;
            let inventory = crate::model::inventory_historical_snapshots_v0(
                std::slice::from_ref(&snapshot),
                &crate::model::HistoricalEvidencePolicyV0::default(),
            )
            .map_err(|_| "historical snapshot inventory failed".to_string())?;
            println!(
                "inventory_accepted_series={}",
                inventory.accepted_series.len()
            );
            println!(
                "inventory_rejected_snapshots={}",
                inventory.rejected_snapshots.len()
            );
            println!(
                "inventory_rejection_statuses={}",
                inventory
                    .rejected_snapshots
                    .iter()
                    .map(|rejected| format!("{:?}", rejected.status))
                    .collect::<Vec<_>>()
                    .join("|")
            );
            let sufficiency = crate::model::assess_momentum_campaign_sufficiency_v0(
                snapshot.row_count,
                &campaign_config,
            )
            .map_err(|_| "momentum campaign sufficiency calculation failed".to_string())?;
            println!("campaign_sufficient={}", sufficiency.sufficient);
            println!("campaign_possible_windows={}", sufficiency.possible_windows);
            run_momentum_campaign_if_enabled(
                &config,
                &snapshot,
                &campaign_config,
                &sufficiency,
                false,
                "text",
                false,
            )?;
        }
        return Ok(());
    }
    let market = MarketSnapshot {
        symbol: args.symbol,
        timestamp_ms: 1_715_000_000_000,
        price: 100.0,
        bid: 99.98,
        ask: 100.02,
        spread_bps: 4.0,
        volume: 10_000.0,
        trade_value: 1_000_000.0,
        volatility: 0.015,
        regime: Regime::TrendUp,
        data_quality_score: 0.98,
    };
    let risk = RiskSnapshot {
        daily_pnl_pct: 0.0,
        consecutive_losses: 0,
        current_positions_count: 0,
        total_exposure_pct: 0.0,
        symbol_exposure_pct: 0.0,
        api_health_score: 1.0,
        data_quality_score: 0.98,
    };

    let signal_engine = MockSignalEngine::default();
    let chair = ChairEngine::default();
    let governor = RiskGovernor::default();
    let mut broker = PaperBroker::default();

    let result = simulate_paper_cycle(
        &market,
        &risk,
        &signal_engine,
        &chair,
        &governor,
        &mut broker,
        args.full_auto,
    );

    println!("decision: {:?}", result.risk_decision.kind);
    println!("chair: {:?}", result.chair_output.decision);
    println!("reasons: {:?}", result.risk_decision.reason_codes);
    if let Some(order) = result.paper_order {
        println!("paper_order: {}", order.order_id);
    } else {
        println!("paper_order: none");
    }
    Ok(())
}

#[derive(Serialize)]
struct AgentCanonicalViewGapCliReportV1 {
    report_version: &'static str,
    mode: String,
    offline: bool,
    gaps: Vec<crate::data::AgentCanonicalViewGapV1>,
    post_acquisition_gaps: Vec<crate::data::AgentCanonicalViewGapV1>,
    gap_report_digest: String,
    post_gap_report_digest: String,
    provider_contract_digests: Vec<String>,
    selected_target_agent_ids: Vec<String>,
    selected_dataset_kind: Option<crate::data::DatasetKind>,
    registration_digest: Option<String>,
    registration_reopened_and_verified: bool,
    request_status: String,
    segment_count: usize,
    segment_digests: Vec<String>,
    segment_statuses: Vec<String>,
    request_count: usize,
    retry_count: usize,
    transport_constructions: usize,
    http_status_class: Option<String>,
    returned_row_count: usize,
    verified_row_count: usize,
    receipt_present: bool,
    receipt_digest: Option<String>,
    raw_response_present: bool,
    provenance_manifest_present: bool,
    provenance_manifest_digest: Option<String>,
    canonical_snapshot_present: bool,
    canonical_snapshot_digest: Option<String>,
    candidate_families: Vec<crate::model::AgentCandidateFamilyPublicSummaryV1>,
    evaluation_registrations: Vec<crate::model::AgentCandidateEvaluationPublicSummaryV1>,
    safety_counters: crate::data::LearningEvidenceSafetyCountersV1,
    prospective_storage_writes: usize,
}

#[derive(Clone, Debug, Serialize)]
struct MigratedCandidateParticipantCliV1 {
    participant_id: String,
    participant_digest: String,
    model_kind: String,
    qualification_status: Option<crate::model::ValidationQualificationStatusV1>,
}

#[derive(Clone, Debug, Serialize)]
struct MigratedCandidateFamilyCliV1 {
    agent_id: String,
    status: crate::model::AgentLearningSessionStatusV1,
    evidence_status: Option<crate::data::CanonicalViewGapStatusV1>,
    blocker_code: Option<String>,
    session_digest: Option<String>,
    view_digest: Option<String>,
    projection_digest: Option<String>,
    family_digest: Option<String>,
    participants: Vec<MigratedCandidateParticipantCliV1>,
    winner_selected: bool,
    historical_test_access_count: usize,
    eligible_for_active_committee: bool,
    eligible_for_promotion: bool,
    eligible_for_reward: bool,
}

#[derive(Clone, Debug, Serialize)]
struct MigratedEvaluationRegistrationCliV1 {
    agent_id: String,
    status: crate::model::CandidateEvaluationRegistrationStatusV1,
    blocker_code: Option<String>,
    registration_digest: Option<String>,
    exclusion_digest: Option<String>,
    minimum_accepted_timestamp_ms: Option<u64>,
    participant_count: usize,
    historical_test_access_count: usize,
    maximum_requests: usize,
    maximum_concurrency: usize,
    maximum_retries: usize,
    labels_hidden_until_opening: bool,
    probabilities_hidden_until_opening: bool,
    one_time_opening_required: bool,
    winner_selection_forbidden_before_opening: bool,
    active_promotion_forbidden: bool,
    reward_application_forbidden: bool,
}

#[derive(Clone, Debug, Serialize)]
struct PersistedRewardEligibilityReplayCliV1 {
    opening_status: crate::model::ProspectiveOutcomeOpeningStatusV0,
    opening_attempt_count: usize,
    opened_event_count: usize,
    outcome_digests: Vec<String>,
    attribution_classes: Vec<crate::model::LearnedAbstentionAttributionV0>,
    eligibility_statuses: Vec<crate::model::LearnedRewardEligibilityStatusV0>,
    eligibility_digests: Vec<String>,
    reward_candidate_count: usize,
    reward_apply_count: usize,
    penalty_apply_count: usize,
    voice_mutation_count: usize,
    authority_action_count: usize,
    replay_matches_persisted: bool,
}

#[derive(Clone, Debug, Serialize)]
struct PersistedLearningIntentMigrationCliReportV1 {
    report_version: &'static str,
    mode: String,
    offline: bool,
    migration: crate::model::PersistedLearningIntentMigrationReportV1,
    candidate_families: Vec<MigratedCandidateFamilyCliV1>,
    evaluation_registrations: Vec<MigratedEvaluationRegistrationCliV1>,
    reward_eligibility_replay: PersistedRewardEligibilityReplayCliV1,
    new_network_requests: usize,
    transport_constructions: usize,
    new_credential_reads: usize,
    new_prospective_row_reads: usize,
    new_prospective_label_openings: usize,
    new_future_evaluation_reads: usize,
    historical_test_reads_v1: usize,
    active_committee_count: usize,
    active_model_changes: usize,
    chair_decisions: usize,
    votes: usize,
    reward_applications: usize,
    penalty_applications: usize,
    voice_changes: usize,
    cooldowns_started: usize,
    promotions: usize,
    quarantines: usize,
    executions: usize,
}

#[derive(Clone, Debug, Serialize)]
struct MomentumMambaRepairParticipantCliV2 {
    participant_digest: String,
    model_kind: String,
    participant_role: crate::model::ParticipantQualificationRoleV2,
    qualification_status: crate::model::ValidationQualificationStatusV2,
}

#[derive(Clone, Debug, Serialize)]
struct MomentumMambaRepairCliReportV2 {
    report_version: &'static str,
    mode: String,
    offline: bool,
    status: crate::model::MomentumMambaRepairExecutionStatusV2,
    collapse_root_causes: Vec<crate::model::MomentumMambaCollapseRootCauseV2>,
    collapse_audit_digest: Option<String>,
    representation_diagnostic_digest: Option<String>,
    optimization_diagnostic_digest: Option<String>,
    probability_diagnostic_digest: Option<String>,
    class_balance_diagnostic_digest: Option<String>,
    repair_capability_status: Option<crate::model::MomentumMambaRepairCapabilityStatusV2>,
    repair_split_digest: Option<String>,
    repair_registration_digest: Option<String>,
    registered_variant_count: usize,
    participants: Vec<MomentumMambaRepairParticipantCliV2>,
    qualified_learned_participant_count: usize,
    qualified_comparator_count: usize,
    family_digest: Option<String>,
    winner_selected: bool,
    historical_test_accessed: bool,
    roster_status: crate::model::MomentumFutureEvaluationRosterStatusV2,
    roster_digest: Option<String>,
    evaluation_registration_status: crate::model::MomentumFutureEvaluationRegistrationStatusV2,
    evaluation_registration_digest: Option<String>,
    minimum_accepted_timestamp_ms: Option<u64>,
    cycle_risk_evidence_status: Option<crate::data::CanonicalViewGapStatusV1>,
    value_quality_evidence_status: Option<crate::data::CanonicalViewGapStatusV1>,
    reward_eligibility_replay: PersistedRewardEligibilityReplayCliV1,
    artifacts_written: usize,
    duplicate_artifact_count: usize,
    storage_failure_count: usize,
    protected_artifacts_unchanged: bool,
    active_state_unchanged: bool,
    safety_counters: crate::model::MomentumMambaRepairSafetyCountersV2,
    report_digest: String,
}

#[derive(Clone, Debug, Serialize)]
struct MomentumRepresentationProbeCliV3 {
    probe_kind: crate::model::MomentumRepresentationProbeKindV3,
    status: crate::model::MomentumRepresentationProbeStatusV3,
    representation_diagnostic_digest: String,
    probe_digest: String,
}

#[derive(Clone, Debug, Serialize)]
struct MomentumRepresentationParticipantCliV3 {
    participant_digest: String,
    model_kind: String,
    input_kind: String,
    qualification_status: crate::model::MomentumRepresentationQualificationStatusV3,
    contribution_status: Option<crate::model::MambaContributionStatusV3>,
}

#[derive(Clone, Debug, Serialize)]
struct MomentumMambaRepresentationCliReportV3 {
    report_version: &'static str,
    mode: String,
    offline: bool,
    status: crate::model::MomentumRepresentationExecutionStatusV3,
    repair_stage: crate::model::MomentumFrozenMambaRepairStageV3,
    probes: Vec<MomentumRepresentationProbeCliV3>,
    representation_audit_digest: Option<String>,
    split_digest: Option<String>,
    final_reserved_range_digest: Option<String>,
    registration_digest: Option<String>,
    registered_variant_count: usize,
    participants: Vec<MomentumRepresentationParticipantCliV3>,
    qualified_genuine_mamba_count: usize,
    qualified_raw_fallback_count: usize,
    qualified_comparator_count: usize,
    route_decision: Option<crate::model::MomentumRepresentationRouteDecisionV3>,
    decision_digest: Option<String>,
    family_digest: Option<String>,
    roster_status: crate::model::MomentumRepresentationRosterStatusV3,
    roster_digest: Option<String>,
    evaluation_registration_status: crate::model::MomentumRepresentationEvaluationStatusV3,
    evaluation_registration_digest: Option<String>,
    minimum_accepted_timestamp_ms: Option<u64>,
    cycle_risk_evidence_status: Option<crate::data::CanonicalViewGapStatusV1>,
    value_quality_evidence_status: Option<crate::data::CanonicalViewGapStatusV1>,
    reward_eligibility_replay: PersistedRewardEligibilityReplayCliV1,
    artifacts_written: usize,
    duplicate_artifact_count: usize,
    storage_failure_count: usize,
    protected_artifacts_unchanged: bool,
    active_state_unchanged: bool,
    safety_counters: crate::model::MomentumRepresentationSafetyCountersV3,
    report_digest: String,
}

#[derive(Clone, Debug, Serialize)]
struct MomentumRawFeatureParticipantCliV4 {
    participant_id: String,
    participant_role: crate::model::MomentumRawFeatureRoleV4,
    model_kind: crate::model::MomentumRawFeatureModelKindV4,
    qualification_status: crate::model::MomentumRawFeatureQualificationStatusV4,
}

#[derive(Clone, Debug, Serialize)]
struct MomentumRawFeatureCliReportV4 {
    report_version: &'static str,
    mode: String,
    offline: bool,
    status: crate::model::MomentumRawFeatureExecutionStatusV4,
    frozen_mamba_closure_status: Option<crate::model::MomentumFrozenMambaClosureDecisionV4>,
    frozen_mamba_closure_digest: Option<String>,
    split_digest: Option<String>,
    registration_digest: Option<String>,
    participants: Vec<MomentumRawFeatureParticipantCliV4>,
    interaction_contribution_status: Option<crate::model::InteractionContributionStatusV4>,
    qualified_learned_count: usize,
    qualified_benchmark_count: usize,
    family_digest: Option<String>,
    path_decision: Option<crate::model::MomentumRawFeaturePathDecisionV4>,
    decision_digest: Option<String>,
    roster_status: crate::model::MomentumRawFeatureRosterStatusV4,
    roster_digest: Option<String>,
    evaluation_registration_status: crate::model::MomentumRawFeatureEvaluationStatusV4,
    evaluation_registration_digest: Option<String>,
    minimum_accepted_timestamp_ms: Option<u64>,
    cycle_risk_evidence_status: Option<crate::data::CanonicalViewGapStatusV1>,
    value_quality_evidence_status: Option<crate::data::CanonicalViewGapStatusV1>,
    reward_eligibility_replay: PersistedRewardEligibilityReplayCliV1,
    artifacts_written: usize,
    duplicate_artifact_count: usize,
    storage_failure_count: usize,
    protected_artifacts_unchanged: bool,
    active_state_unchanged: bool,
    safety_counters: crate::model::MomentumRawFeatureSafetyCountersV4,
    report_digest: String,
}

fn load_persisted_learning_registrations_v1(
    root: &Path,
) -> Result<Vec<crate::data::LearningEvidenceAcquisitionRegistrationV1>, String> {
    let directory = root.join("acquisition_v1").join("registrations");
    if !directory.is_dir() {
        return Ok(Vec::new());
    }
    let mut registrations = fs::read_dir(directory)
        .map_err(|_| "learning registration directory unavailable".to_string())?
        .map(|entry| {
            let path = entry
                .map_err(|_| "learning registration directory rejected".to_string())?
                .path();
            if path.extension().is_none_or(|extension| extension != "pb") {
                return Ok(None);
            }
            crate::data::decode_learning_evidence_registration_protobuf_v1(
                &fs::read(path).map_err(|_| "learning registration unavailable".to_string())?,
            )
            .map(Some)
        })
        .collect::<Result<Vec<_>, String>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    registrations.sort_by(|left, right| left.registration_digest.cmp(&right.registration_digest));
    Ok(registrations)
}

fn print_agent_canonical_view_gap_report_v1(
    report: &AgentCanonicalViewGapCliReportV1,
    output_format: &str,
) -> Result<(), String> {
    if output_format == "json" {
        println!(
            "{}",
            serde_json::to_string(report)
                .map_err(|_| "canonical view gap report serialization failed")?
        );
        return Ok(());
    }
    if output_format != "text" {
        return Err("unsupported canonical view gap output format".into());
    }
    println!("report_version={}", report.report_version);
    println!("mode={}", report.mode);
    println!("offline={}", report.offline);
    for gap in &report.gaps {
        println!(
            "agent={};intent_digest={};required={:?};resolved_required={:?};missing_required={:?};optional={:?};resolved_optional={:?};missing_optional={:?};authorized_providers={};status={:?};gap_digest={}",
            gap.agent_id,
            gap.intent_digest,
            gap.required_dataset_kinds,
            gap.resolved_required_dataset_kinds,
            gap.missing_required_dataset_kinds,
            gap.optional_dataset_kinds,
            gap.resolved_optional_dataset_kinds,
            gap.missing_optional_dataset_kinds,
            gap.authorized_provider_ids.join("|"),
            gap.status,
            gap.gap_digest,
        );
    }
    println!("gap_report_digest={}", report.gap_report_digest);
    println!(
        "provider_contract_digests={}",
        report.provider_contract_digests.join("|")
    );
    println!(
        "selected_target_agent_ids={}",
        report.selected_target_agent_ids.join("|")
    );
    println!(
        "selected_dataset_kind={}",
        report
            .selected_dataset_kind
            .map(|kind| format!("{kind:?}"))
            .unwrap_or_default()
    );
    println!(
        "registration_digest={}",
        report.registration_digest.as_deref().unwrap_or_default()
    );
    println!(
        "registration_reopened_and_verified={}",
        report.registration_reopened_and_verified
    );
    println!("request_status={}", report.request_status);
    println!("segment_count={}", report.segment_count);
    println!("segment_digests={}", report.segment_digests.join("|"));
    println!("segment_statuses={}", report.segment_statuses.join("|"));
    println!("request_count={}", report.request_count);
    println!("retry_count={}", report.retry_count);
    println!("transport_constructions={}", report.transport_constructions);
    println!(
        "http_status_class={}",
        report.http_status_class.as_deref().unwrap_or_default()
    );
    println!("returned_row_count={}", report.returned_row_count);
    println!("verified_row_count={}", report.verified_row_count);
    println!("receipt_present={}", report.receipt_present);
    println!(
        "receipt_digest={}",
        report.receipt_digest.as_deref().unwrap_or_default()
    );
    println!("raw_response_present={}", report.raw_response_present);
    println!(
        "provenance_manifest_present={}",
        report.provenance_manifest_present
    );
    println!(
        "provenance_manifest_digest={}",
        report
            .provenance_manifest_digest
            .as_deref()
            .unwrap_or_default()
    );
    println!(
        "canonical_snapshot_present={}",
        report.canonical_snapshot_present
    );
    println!(
        "canonical_snapshot_digest={}",
        report
            .canonical_snapshot_digest
            .as_deref()
            .unwrap_or_default()
    );
    for family in &report.candidate_families {
        println!(
            "candidate_agent={};status={:?};participant_count={};historical_test_access_count={}",
            family.agent_id,
            family.status,
            family.participant_count,
            family.historical_test_access_count,
        );
    }
    for registration in &report.evaluation_registrations {
        println!(
            "evaluation_agent={};status={:?};participant_count={};historical_test_access_count={}",
            registration.agent_id,
            registration.registration_status,
            registration.participant_count,
            registration.historical_test_access_count,
        );
    }
    println!("post_gap_report_digest={}", report.post_gap_report_digest);
    println!("safety_counters={:?}", report.safety_counters);
    println!(
        "prospective_storage_writes={}",
        report.prospective_storage_writes
    );
    Ok(())
}

fn run_agent_canonical_view_gap_cli_v1(
    config_path: &Path,
    output_format: &str,
    status: bool,
    dry_run: bool,
    execute_local: bool,
    allow_network: bool,
    confirm_one_time_learning_evidence_request: bool,
    confirm_composite_learning_evidence_epoch: bool,
) -> Result<(), String> {
    if usize::from(status) + usize::from(dry_run) + usize::from(execute_local) != 1 {
        return Err(
            "select exactly one canonical view gap mode: --status, --dry-run, or --execute-local"
                .into(),
        );
    }
    if confirm_one_time_learning_evidence_request && confirm_composite_learning_evidence_epoch {
        return Err("select exactly one learning evidence confirmation".into());
    }
    let network_confirmed =
        confirm_one_time_learning_evidence_request || confirm_composite_learning_evidence_epoch;
    if allow_network != network_confirmed {
        return Err(
            "learning evidence request requires network access and exactly one matching confirmation"
                .into(),
        );
    }
    if (allow_network || network_confirmed) && !execute_local {
        return Err("canonical view gap status and dry-run modes are offline-only".into());
    }
    let mode = if status {
        "status"
    } else if dry_run {
        "dry-run"
    } else {
        "execute-local"
    };
    let root = crate::model::default_private_learning_root_v0();
    let snapshot_root = Path::new("data/local_snapshots");
    let snapshots = crate::model::load_local_learning_snapshots_v0(snapshot_root)?;
    let policies = crate::data::default_agent_data_policies();
    let intents = crate::model::load_persisted_agent_learning_intents_v0(root, &snapshots)?;
    let information_cutoff_ms = intents
        .iter()
        .map(|intent| intent.information_cutoff_ms)
        .max()
        .ok_or_else(|| "canonical learning information cutoff unavailable".to_string())?;
    let trainer_capable_agent_ids = crate::model::agent_trainer_capability_registry_v0()
        .capabilities
        .into_iter()
        .filter(|capability| capability.supports_training)
        .map(|capability| capability.agent_id)
        .collect::<BTreeSet<_>>();
    let provider_config = crate::data::UpbitHistoricalPilotConfigV0::from_toml_path(config_path)?;
    let provider_contracts =
        crate::data::upbit_learning_evidence_provider_contract_v1(&provider_config)
            .map(|contract| vec![contract])
            .unwrap_or_default();
    let gap_report = crate::data::derive_agent_canonical_view_gaps_v1(
        &intents,
        &policies,
        &snapshots,
        &trainer_capable_agent_ids,
        &provider_contracts,
    )?;
    let reservation = crate::model::load_protected_evaluation_reservation_v1(
        config_path
            .parent()
            .ok_or_else(|| "protected reservation directory unavailable".to_string())?,
    )?;
    let selected_registration = crate::data::select_learning_evidence_acquisition_registration_v1(
        &gap_report,
        &provider_contracts,
        &reservation.protected_registration_digests,
        &reservation.reserved_timestamp_ms,
    )?;
    let selected_composite = crate::data::select_composite_learning_acquisition_registration_v1(
        &gap_report,
        &provider_contracts,
        &reservation.protected_registration_digests,
        &reservation.reserved_timestamp_ms,
    )?;
    let persisted_registrations = load_persisted_learning_registrations_v1(root)?;
    if persisted_registrations.len() > 1 {
        return Err("multiple learning evidence registrations rejected".into());
    }
    let registration = selected_registration
        .clone()
        .or_else(|| persisted_registrations.first().cloned());
    let persisted_composite = crate::data::read_composite_learning_registration_v1(root)?;
    let composite_registration = selected_composite
        .clone()
        .or_else(|| persisted_composite.clone());
    if registration.is_some() && composite_registration.is_some() {
        return Err("single and composite learning registrations conflict".into());
    }
    let mut registration_reopened_and_verified = false;
    if dry_run {
        let decoded = crate::data::decode_agent_canonical_view_gap_report_protobuf_v1(
            &crate::data::encode_agent_canonical_view_gap_report_protobuf_v1(&gap_report)?,
        )?;
        if decoded != gap_report {
            return Err("canonical view gap Protobuf round trip rejected".into());
        }
        if let Some(registration) = selected_registration.as_ref() {
            let decoded = crate::data::decode_learning_evidence_registration_protobuf_v1(
                &crate::data::encode_learning_evidence_registration_protobuf_v1(registration)?,
            )?;
            if &decoded != registration {
                return Err("learning evidence registration Protobuf round trip rejected".into());
            }
        }
        if let Some(registration) = selected_composite.as_ref() {
            let contract = provider_contracts
                .iter()
                .find(|contract| contract.contract_digest == registration.provider_contract_digest)
                .ok_or("composite learning provider contract unavailable")?;
            let decoded = crate::data::decode_composite_learning_registration_protobuf_v1(
                &crate::data::encode_composite_learning_registration_protobuf_v1(
                    registration,
                    contract,
                )?,
            )?;
            if &decoded != registration {
                return Err("composite learning registration Protobuf round trip rejected".into());
            }
        }
    }
    if execute_local {
        crate::data::write_and_verify_agent_canonical_view_gap_report_v1(&gap_report, root)?;
        if let Some(selected) = selected_registration.as_ref() {
            crate::data::write_and_verify_learning_evidence_registration_v1(selected, root)?;
            registration_reopened_and_verified =
                crate::data::read_learning_evidence_registration_v1(
                    &selected.registration_digest,
                    root,
                )? == *selected;
            if !registration_reopened_and_verified {
                return Err("learning evidence registration reopen rejected".into());
            }
        } else if let Some(existing) = registration.as_ref() {
            registration_reopened_and_verified =
                crate::data::read_learning_evidence_registration_v1(
                    &existing.registration_digest,
                    root,
                )? == *existing;
        }
        if let Some(selected) = selected_composite.as_ref() {
            let contract = provider_contracts
                .iter()
                .find(|contract| contract.contract_digest == selected.provider_contract_digest)
                .ok_or("composite learning provider contract unavailable")?;
            crate::data::write_and_verify_composite_learning_registration_v1(
                selected, contract, root,
            )?;
            registration_reopened_and_verified =
                crate::data::read_composite_learning_registration_v1(root)?.as_ref()
                    == Some(selected);
            if !registration_reopened_and_verified {
                return Err("composite learning registration reopen rejected".into());
            }
        } else if let Some(existing) = composite_registration.as_ref() {
            registration_reopened_and_verified =
                crate::data::read_composite_learning_registration_v1(root)?.as_ref()
                    == Some(existing);
        }
    }
    let existing_receipt = registration
        .as_ref()
        .map(|registration| {
            crate::data::read_learning_evidence_receipt_v1(&registration.registration_digest, root)
        })
        .transpose()?
        .flatten();
    let current_gap_digests = gap_report
        .gaps
        .iter()
        .map(|gap| gap.gap_digest.clone())
        .collect::<Vec<_>>();
    let mut acquisition_result = None;
    let existing_epoch_receipt = composite_registration
        .as_ref()
        .map(|registration| crate::data::read_composite_epoch_receipt_v1(registration, root))
        .transpose()?
        .flatten();
    let mut composite_result = None;
    if allow_network && confirm_one_time_learning_evidence_request {
        let registration = registration
            .as_ref()
            .ok_or_else(|| "learning evidence registration unavailable".to_string())?;
        let contract = provider_contracts
            .iter()
            .find(|contract| contract.contract_digest == registration.provider_contract_digest)
            .ok_or_else(|| "learning evidence provider contract unavailable".to_string())?;
        let result = crate::data::execute_learning_evidence_acquisition_v1(
            registration,
            contract,
            &current_gap_digests,
            existing_receipt.as_ref(),
            &snapshots,
            true,
            |request| crate::data::fetch_upbit_learning_evidence_once_v1(&provider_config, request),
        );
        if let (Some(raw), Some(receipt)) = (&result.raw_response, &result.receipt) {
            if let Some(raw_digest) = receipt.raw_response_digest.as_deref() {
                crate::data::write_and_verify_learning_raw_response_v1(raw, raw_digest, root)?;
            }
        }
        if let Some(manifest) = &result.provenance_manifest {
            crate::data::write_and_verify_learning_evidence_provenance_v1(manifest, root)?;
        }
        if let Some(snapshot) = &result.snapshot {
            crate::data::write_and_verify_local_snapshot_v0(
                snapshot,
                Path::new(&provider_config.snapshot_output_dir),
            )?;
            let stored = crate::data::read_local_snapshot_protobuf_v1(
                &Path::new(&provider_config.snapshot_output_dir)
                    .join(format!("{}.pb", snapshot.snapshot_id)),
            )?;
            if stored.content_digest != snapshot.content_digest {
                return Err("canonical learning snapshot reopen rejected".into());
            }
        }
        if let Some(receipt) = &result.receipt {
            crate::data::write_and_verify_learning_evidence_receipt_v1(receipt, root)?;
            if crate::data::read_learning_evidence_receipt_v1(&receipt.registration_digest, root)?
                .as_ref()
                != Some(receipt)
            {
                return Err("learning evidence receipt reopen rejected".into());
            }
        }
        acquisition_result = Some(result);
    }
    if allow_network && confirm_composite_learning_evidence_epoch {
        let registration = composite_registration
            .as_ref()
            .ok_or("composite learning registration unavailable")?;
        let contract = provider_contracts
            .iter()
            .find(|contract| contract.contract_digest == registration.provider_contract_digest)
            .ok_or("composite learning provider contract unavailable")?;
        let result = crate::data::execute_composite_learning_acquisition_v1(
            registration,
            contract,
            &current_gap_digests,
            existing_epoch_receipt.as_ref(),
            true,
            |_segment, request| {
                crate::data::fetch_upbit_learning_evidence_once_v1(&provider_config, request)
            },
        );
        let raw_receipts = result
            .segment_receipts
            .iter()
            .filter(|receipt| receipt.raw_response_digest.is_some())
            .collect::<Vec<_>>();
        if raw_receipts.len() != result.raw_responses.len() {
            return Err("composite learning raw response chain rejected".into());
        }
        for (raw, receipt) in result.raw_responses.iter().zip(raw_receipts) {
            crate::data::write_and_verify_learning_raw_response_v1(
                raw,
                receipt.raw_response_digest.as_deref().unwrap_or_default(),
                root,
            )?;
        }
        for receipt in &result.segment_receipts {
            crate::data::write_and_verify_composite_segment_receipt_v1(receipt, root)?;
        }
        for capsule in &result.segment_capsules {
            crate::data::write_and_verify_composite_segment_capsule_v1(
                capsule,
                registration,
                contract,
                root,
            )?;
        }
        if let Some(provenance) = &result.merged_provenance {
            crate::data::write_and_verify_composite_merged_provenance_v1(
                provenance,
                registration,
                root,
            )?;
        }
        if let Some(snapshot) = &result.snapshot {
            crate::data::write_and_verify_local_snapshot_v0(
                snapshot,
                Path::new(&provider_config.snapshot_output_dir),
            )?;
            let stored = crate::data::read_local_snapshot_protobuf_v1(
                &Path::new(&provider_config.snapshot_output_dir)
                    .join(format!("{}.pb", snapshot.snapshot_id)),
            )?;
            if stored.content_digest != snapshot.content_digest
                || stored.row_count != registration.required_row_count
            {
                return Err("composite canonical snapshot reopen rejected".into());
            }
        }
        if let Some(receipt) = &result.epoch_receipt {
            crate::data::write_and_verify_composite_epoch_receipt_v1(receipt, registration, root)?;
            if crate::data::read_composite_epoch_receipt_v1(registration, root)?.as_ref()
                != Some(receipt)
            {
                return Err("composite epoch receipt reopen rejected".into());
            }
        }
        composite_result = Some(result);
    }
    let replayed_receipt = if let Some(result) = acquisition_result.as_ref() {
        result.receipt.clone().or(existing_receipt.clone())
    } else {
        existing_receipt.clone()
    };
    let replayed_epoch_receipt = composite_result
        .as_ref()
        .and_then(|result| result.epoch_receipt.clone())
        .or(existing_epoch_receipt.clone());
    let refreshed_snapshots = crate::model::load_local_learning_snapshots_v0(snapshot_root)?;
    let post_gap_report = crate::data::derive_agent_canonical_view_gaps_v1(
        &intents,
        &policies,
        &refreshed_snapshots,
        &trainer_capable_agent_ids,
        &provider_contracts,
    )?;
    if execute_local {
        crate::data::write_and_verify_agent_canonical_view_gap_report_v1(&post_gap_report, root)?;
    }
    let mut candidate_families = Vec::new();
    let mut evaluation_registrations = Vec::new();
    if execute_local {
        let inputs = crate::model::build_agent_private_learning_inputs_v1(
            &refreshed_snapshots,
            information_cutoff_ms,
            root,
            crate::model::AgentPrivateLearningRunModeV0::ExecuteLocal,
        );
        let mut families = crate::model::run_agent_private_learning_candidates_v1(
            &inputs,
            crate::model::AgentPrivateLearningRunModeV0::ExecuteLocal,
        );
        crate::model::persist_agent_candidate_families_report_v1(&mut families, root);
        let mut evaluations = crate::model::run_agent_candidate_evaluations_v1(
            &families,
            &inputs,
            &reservation,
            crate::model::AgentPrivateLearningRunModeV0::ExecuteLocal,
        );
        crate::model::persist_agent_candidate_evaluations_report_v1(&mut evaluations, root);
        if families.storage_failure_count != 0 || evaluations.storage_failure_count != 0 {
            return Err("offline V1 rerun persistence rejected".into());
        }
        candidate_families = crate::model::public_candidate_family_summaries_v1(&families);
        evaluation_registrations =
            crate::model::public_candidate_evaluation_summaries_v1(&evaluations);
    }
    let request_status = composite_result
        .as_ref()
        .map(|result| format!("{:?}", result.status))
        .or_else(|| {
            replayed_epoch_receipt
                .as_ref()
                .map(|receipt| format!("{:?}", receipt.status))
        })
        .or_else(|| {
            acquisition_result
                .as_ref()
                .map(|result| format!("{:?}", result.status))
        })
        .or_else(|| {
            replayed_receipt
                .as_ref()
                .map(|receipt| format!("{:?}", receipt.status))
        })
        .unwrap_or_else(|| "ReadyNotAttempted".into());
    let mut safety_counters = composite_result
        .as_ref()
        .map(|result| result.safety_counters.clone())
        .or_else(|| {
            acquisition_result
                .as_ref()
                .map(|result| result.safety_counters.clone())
        })
        .unwrap_or_else(|| gap_report.safety_counters.clone());
    if acquisition_result.is_none() && composite_result.is_none() {
        safety_counters.request_attempts = replayed_receipt
            .as_ref()
            .map_or(0, |receipt| receipt.request_count);
        safety_counters.request_attempts = safety_counters.request_attempts.max(
            replayed_epoch_receipt
                .as_ref()
                .map_or(0, |receipt| receipt.request_count),
        );
        safety_counters.retry_count = replayed_receipt
            .as_ref()
            .map_or(0, |receipt| receipt.retry_count);
        safety_counters.retry_count = safety_counters.retry_count.max(
            replayed_epoch_receipt
                .as_ref()
                .map_or(0, |receipt| receipt.retry_count),
        );
    }
    print_agent_canonical_view_gap_report_v1(
        &AgentCanonicalViewGapCliReportV1 {
            report_version: "agent-canonical-view-gap-cli-report-v1",
            mode: mode.into(),
            offline: acquisition_result.is_none() && composite_result.is_none(),
            gaps: gap_report.gaps.clone(),
            post_acquisition_gaps: post_gap_report.gaps,
            gap_report_digest: gap_report.report_digest,
            post_gap_report_digest: post_gap_report.report_digest,
            provider_contract_digests: gap_report.provider_contract_digests,
            selected_target_agent_ids: registration
                .as_ref()
                .map(|registration| registration.target_agent_ids.clone())
                .or_else(|| {
                    composite_registration
                        .as_ref()
                        .map(|registration| registration.target_agent_ids.clone())
                })
                .unwrap_or_default(),
            selected_dataset_kind: registration
                .as_ref()
                .map(|registration| registration.dataset_kind)
                .or_else(|| {
                    composite_registration
                        .as_ref()
                        .map(|registration| registration.dataset_kind)
                }),
            registration_digest: registration
                .as_ref()
                .map(|registration| registration.registration_digest.clone())
                .or_else(|| {
                    composite_registration
                        .as_ref()
                        .map(|registration| registration.registration_digest.clone())
                }),
            registration_reopened_and_verified,
            request_status,
            segment_count: composite_registration
                .as_ref()
                .map_or(0, |registration| registration.segments.len()),
            segment_digests: composite_registration
                .as_ref()
                .map(|registration| {
                    registration
                        .segments
                        .iter()
                        .map(|segment| segment.segment_digest.clone())
                        .collect()
                })
                .unwrap_or_default(),
            segment_statuses: composite_result
                .as_ref()
                .map(|result| {
                    result
                        .segment_receipts
                        .iter()
                        .map(|receipt| format!("{:?}", receipt.status))
                        .collect()
                })
                .unwrap_or_default(),
            request_count: replayed_epoch_receipt
                .as_ref()
                .map(|receipt| receipt.request_count)
                .or_else(|| {
                    replayed_receipt
                        .as_ref()
                        .map(|receipt| receipt.request_count)
                })
                .unwrap_or_default(),
            retry_count: replayed_epoch_receipt
                .as_ref()
                .map(|receipt| receipt.retry_count)
                .or_else(|| replayed_receipt.as_ref().map(|receipt| receipt.retry_count))
                .unwrap_or_default(),
            transport_constructions: composite_result
                .as_ref()
                .map(|result| result.safety_counters.transport_constructions)
                .or_else(|| {
                    acquisition_result
                        .as_ref()
                        .map(|result| result.safety_counters.transport_constructions)
                })
                .unwrap_or_default(),
            http_status_class: composite_result
                .as_ref()
                .and_then(|result| result.segment_receipts.last())
                .and_then(|receipt| receipt.http_status_class.clone())
                .or_else(|| {
                    replayed_receipt
                        .as_ref()
                        .and_then(|receipt| receipt.http_status_class.clone())
                }),
            returned_row_count: composite_result
                .as_ref()
                .map(|result| {
                    result
                        .segment_receipts
                        .iter()
                        .map(|receipt| receipt.returned_row_count)
                        .sum()
                })
                .or_else(|| {
                    replayed_receipt
                        .as_ref()
                        .map(|receipt| receipt.returned_row_count)
                })
                .unwrap_or_default(),
            verified_row_count: composite_result
                .as_ref()
                .map(|result| {
                    result
                        .segment_receipts
                        .iter()
                        .map(|receipt| receipt.verified_row_count)
                        .sum()
                })
                .or_else(|| {
                    replayed_receipt
                        .as_ref()
                        .map(|receipt| receipt.verified_row_count)
                })
                .unwrap_or_default(),
            receipt_present: replayed_epoch_receipt.is_some() || replayed_receipt.is_some(),
            receipt_digest: replayed_epoch_receipt
                .as_ref()
                .map(|receipt| receipt.receipt_digest.clone())
                .or_else(|| {
                    replayed_receipt
                        .as_ref()
                        .map(|receipt| receipt.receipt_digest.clone())
                }),
            raw_response_present: composite_result
                .as_ref()
                .is_some_and(|result| !result.raw_responses.is_empty())
                || replayed_receipt
                    .as_ref()
                    .is_some_and(|receipt| receipt.raw_response_digest.is_some()),
            provenance_manifest_present: replayed_epoch_receipt
                .as_ref()
                .is_some_and(|receipt| receipt.merged_provenance_digest.is_some())
                || replayed_receipt
                    .as_ref()
                    .is_some_and(|receipt| receipt.provenance_manifest_digest.is_some()),
            provenance_manifest_digest: replayed_epoch_receipt
                .as_ref()
                .and_then(|receipt| receipt.merged_provenance_digest.clone())
                .or_else(|| {
                    replayed_receipt
                        .as_ref()
                        .and_then(|receipt| receipt.provenance_manifest_digest.clone())
                }),
            canonical_snapshot_present: replayed_epoch_receipt
                .as_ref()
                .is_some_and(|receipt| receipt.merged_snapshot_digest.is_some())
                || replayed_receipt
                    .as_ref()
                    .is_some_and(|receipt| receipt.snapshot_digest.is_some()),
            canonical_snapshot_digest: replayed_epoch_receipt
                .as_ref()
                .and_then(|receipt| receipt.merged_snapshot_digest.clone())
                .or_else(|| {
                    replayed_receipt
                        .as_ref()
                        .and_then(|receipt| receipt.snapshot_digest.clone())
                }),
            candidate_families,
            evaluation_registrations,
            safety_counters,
            prospective_storage_writes: 0,
        },
        output_format,
    )
}

fn migrated_candidate_family_cli_v1(
    result: &crate::model::AgentCandidateFamilyResultV1,
    evidence_status: Option<crate::data::CanonicalViewGapStatusV1>,
) -> MigratedCandidateFamilyCliV1 {
    let participants = result
        .family
        .as_ref()
        .map(|family| {
            family
                .participants
                .iter()
                .map(|participant| MigratedCandidateParticipantCliV1 {
                    participant_id: participant.participant_id.clone(),
                    participant_digest: participant.participant_digest.clone(),
                    model_kind: participant.model_kind.clone(),
                    qualification_status: result
                        .qualification_receipts
                        .iter()
                        .find(|receipt| {
                            receipt.participant_digest == participant.participant_digest
                        })
                        .map(|receipt| receipt.qualification_status),
                })
                .collect()
        })
        .unwrap_or_default();
    MigratedCandidateFamilyCliV1 {
        agent_id: result.agent_id.clone(),
        status: result.status,
        evidence_status,
        blocker_code: result.sanitized_error_code.clone(),
        session_digest: result
            .session
            .as_ref()
            .map(|session| session.session_digest.clone()),
        view_digest: result
            .session
            .as_ref()
            .map(|session| session.view_digest.clone()),
        projection_digest: result
            .projection
            .as_ref()
            .map(|projection| projection.projection_digest.clone()),
        family_digest: result
            .family
            .as_ref()
            .map(|family| family.family_digest.clone()),
        participants,
        winner_selected: result
            .family
            .as_ref()
            .is_some_and(|family| family.winner_selected),
        historical_test_access_count: result.usage_ledger.as_ref().map_or(0, |ledger| {
            ledger.historical_test_row_reads
                + ledger.historical_test_label_reads
                + ledger.historical_test_inference_count
                + ledger.historical_test_metric_count
                + ledger.historical_test_checkpoint_selection_count
        }),
        eligible_for_active_committee: result
            .family
            .as_ref()
            .is_some_and(|family| family.eligible_for_active_committee),
        eligible_for_promotion: result
            .family
            .as_ref()
            .is_some_and(|family| family.eligible_for_promotion),
        eligible_for_reward: result
            .family
            .as_ref()
            .is_some_and(|family| family.eligible_for_reward),
    }
}

fn migrated_evaluation_registration_cli_v1(
    result: &crate::model::AgentCandidateEvaluationResultV1,
) -> MigratedEvaluationRegistrationCliV1 {
    MigratedEvaluationRegistrationCliV1 {
        agent_id: result.agent_id.clone(),
        status: result.status,
        blocker_code: result.sanitized_error_code.clone(),
        registration_digest: result
            .registration
            .as_ref()
            .map(|registration| registration.registration_digest.clone()),
        exclusion_digest: result
            .exclusion
            .as_ref()
            .map(|exclusion| exclusion.exclusion_digest.clone()),
        minimum_accepted_timestamp_ms: result
            .registration
            .as_ref()
            .map(|registration| registration.minimum_accepted_timestamp_ms),
        participant_count: result
            .registration
            .as_ref()
            .map_or(0, |registration| registration.participant_digests.len()),
        historical_test_access_count: 0,
        maximum_requests: result
            .registration
            .as_ref()
            .map_or(0, |registration| registration.maximum_requests),
        maximum_concurrency: result
            .registration
            .as_ref()
            .map_or(0, |registration| registration.maximum_concurrency),
        maximum_retries: result
            .registration
            .as_ref()
            .map_or(0, |registration| registration.maximum_retries),
        labels_hidden_until_opening: result
            .registration
            .as_ref()
            .is_some_and(|registration| registration.labels_hidden_until_opening),
        probabilities_hidden_until_opening: result
            .registration
            .as_ref()
            .is_some_and(|registration| registration.probabilities_hidden_until_opening),
        one_time_opening_required: result
            .registration
            .as_ref()
            .is_some_and(|registration| registration.one_time_opening_required),
        winner_selection_forbidden_before_opening: result
            .registration
            .as_ref()
            .is_some_and(|registration| registration.winner_selection_forbidden_before_opening),
        active_promotion_forbidden: result
            .registration
            .as_ref()
            .is_some_and(|registration| registration.active_promotion_forbidden),
        reward_application_forbidden: result
            .registration
            .as_ref()
            .is_some_and(|registration| registration.reward_application_forbidden),
    }
}

fn format_persisted_intent_migration_text_v1(
    report: &PersistedLearningIntentMigrationCliReportV1,
) -> String {
    let mut output = String::new();
    let migration = &report.migration;
    let _ = writeln!(output, "report_version={}", report.report_version);
    let _ = writeln!(output, "mode={}", report.mode);
    let _ = writeln!(output, "offline={}", report.offline);
    let _ = writeln!(output, "agent_id={}", migration.agent_id);
    let _ = writeln!(output, "migration_status={:?}", migration.status);
    let _ = writeln!(output, "migration_blocker={:?}", migration.blocker);
    let _ = writeln!(
        output,
        "first_failing_invariant={}",
        migration
            .first_failing_invariant
            .as_deref()
            .unwrap_or_default()
    );
    for (name, value) in [
        ("legacy_session_digest", &migration.legacy_session_digest),
        ("legacy_intent_digest", &migration.legacy_intent_digest),
        ("canonical_gap_digest", &migration.canonical_gap_digest),
        (
            "composite_registration_digest",
            &migration.composite_registration_digest,
        ),
        (
            "canonical_snapshot_digest",
            &migration.canonical_snapshot_digest,
        ),
        (
            "canonical_intent_digest",
            &migration.canonical_intent_digest,
        ),
        ("canonical_view_digest", &migration.canonical_view_digest),
        (
            "policy_compatibility_proof_digest",
            &migration.policy_compatibility_proof_digest,
        ),
        ("migration_proof_digest", &migration.migration_proof_digest),
        (
            "migration_journal_digest",
            &migration.migration_journal_digest,
        ),
    ] {
        let _ = writeln!(output, "{name}={}", value.as_deref().unwrap_or_default());
    }
    let _ = writeln!(
        output,
        "field_provenance_count={}",
        migration.field_provenance_count
    );
    let _ = writeln!(
        output,
        "required_evidence_complete={}",
        migration.required_evidence_complete
    );
    let _ = writeln!(
        output,
        "optional_evidence_unavailable={}",
        migration.optional_evidence_unavailable
    );
    let _ = writeln!(
        output,
        "normal_validator_passed={}",
        migration.normal_validator_passed
    );
    let _ = writeln!(
        output,
        "normal_view_builder_passed={}",
        migration.normal_view_builder_passed
    );
    for family in &report.candidate_families {
        let _ = writeln!(
            output,
            "candidate_agent={};status={:?};evidence_status={};blocker_code={};session_digest={};view_digest={};projection_digest={};family_digest={};participant_count={};winner_selected={};historical_test_access_count={};active_eligible={};promotion_eligible={};reward_eligible={}",
            family.agent_id,
            family.status,
            family
                .evidence_status
                .map(|status| format!("{status:?}"))
                .unwrap_or_default(),
            family.blocker_code.as_deref().unwrap_or_default(),
            family.session_digest.as_deref().unwrap_or_default(),
            family.view_digest.as_deref().unwrap_or_default(),
            family.projection_digest.as_deref().unwrap_or_default(),
            family.family_digest.as_deref().unwrap_or_default(),
            family.participants.len(),
            family.winner_selected,
            family.historical_test_access_count,
            family.eligible_for_active_committee,
            family.eligible_for_promotion,
            family.eligible_for_reward,
        );
        for participant in &family.participants {
            let _ = writeln!(
                output,
                "participant_id={};participant_digest={};model_kind={};qualification_status={}",
                participant.participant_id,
                participant.participant_digest,
                participant.model_kind,
                participant
                    .qualification_status
                    .map(|value| format!("{value:?}"))
                    .unwrap_or_default(),
            );
        }
    }
    for registration in &report.evaluation_registrations {
        let _ = writeln!(
            output,
            "evaluation_agent={};status={:?};blocker_code={};registration_digest={};exclusion_digest={};minimum_accepted_timestamp_ms={};participant_count={};historical_test_access_count={};maximum_requests={};maximum_concurrency={};maximum_retries={};labels_hidden={};probabilities_hidden={};one_time_opening_required={};winner_selection_forbidden={};active_promotion_forbidden={};reward_application_forbidden={}",
            registration.agent_id,
            registration.status,
            registration.blocker_code.as_deref().unwrap_or_default(),
            registration
                .registration_digest
                .as_deref()
                .unwrap_or_default(),
            registration.exclusion_digest.as_deref().unwrap_or_default(),
            registration
                .minimum_accepted_timestamp_ms
                .map(|value| value.to_string())
                .unwrap_or_default(),
            registration.participant_count,
            registration.historical_test_access_count,
            registration.maximum_requests,
            registration.maximum_concurrency,
            registration.maximum_retries,
            registration.labels_hidden_until_opening,
            registration.probabilities_hidden_until_opening,
            registration.one_time_opening_required,
            registration.winner_selection_forbidden_before_opening,
            registration.active_promotion_forbidden,
            registration.reward_application_forbidden,
        );
    }
    let reward = &report.reward_eligibility_replay;
    let _ = writeln!(output, "opening_status={:?}", reward.opening_status);
    let _ = writeln!(
        output,
        "opening_attempt_count={}",
        reward.opening_attempt_count
    );
    let _ = writeln!(output, "opened_event_count={}", reward.opened_event_count);
    let _ = writeln!(output, "outcome_digests={:?}", reward.outcome_digests);
    let _ = writeln!(
        output,
        "attribution_classes={:?}",
        reward.attribution_classes
    );
    let _ = writeln!(
        output,
        "eligibility_statuses={:?}",
        reward.eligibility_statuses
    );
    let _ = writeln!(
        output,
        "eligibility_digests={:?}",
        reward.eligibility_digests
    );
    let _ = writeln!(
        output,
        "reward_candidate_count={}",
        reward.reward_candidate_count
    );
    let _ = writeln!(output, "reward_apply_count={}", reward.reward_apply_count);
    let _ = writeln!(output, "penalty_apply_count={}", reward.penalty_apply_count);
    let _ = writeln!(
        output,
        "voice_mutation_count={}",
        reward.voice_mutation_count
    );
    let _ = writeln!(
        output,
        "authority_action_count={}",
        reward.authority_action_count
    );
    let _ = writeln!(
        output,
        "reward_replay_matches_persisted={}",
        reward.replay_matches_persisted
    );
    for (name, value) in [
        ("new_network_requests", report.new_network_requests),
        ("transport_constructions", report.transport_constructions),
        ("new_credential_reads", report.new_credential_reads),
        (
            "new_prospective_row_reads",
            report.new_prospective_row_reads,
        ),
        (
            "new_prospective_label_openings",
            report.new_prospective_label_openings,
        ),
        (
            "new_future_evaluation_reads",
            report.new_future_evaluation_reads,
        ),
        ("historical_test_reads_v1", report.historical_test_reads_v1),
        ("active_committee_count", report.active_committee_count),
        ("active_model_changes", report.active_model_changes),
        ("chair_decisions", report.chair_decisions),
        ("votes", report.votes),
        ("reward_applications", report.reward_applications),
        ("penalty_applications", report.penalty_applications),
        ("voice_changes", report.voice_changes),
        ("cooldowns_started", report.cooldowns_started),
        ("promotions", report.promotions),
        ("quarantines", report.quarantines),
        ("executions", report.executions),
    ] {
        let _ = writeln!(output, "{name}={value}");
    }
    let _ = writeln!(
        output,
        "protected_artifacts_unchanged={}",
        migration.protected_artifacts_unchanged
    );
    let _ = writeln!(
        output,
        "active_state_unchanged={}",
        migration.active_state_unchanged
    );
    let _ = writeln!(output, "artifacts_written={}", migration.artifacts_written);
    let _ = writeln!(
        output,
        "duplicate_artifact_count={}",
        migration.duplicate_artifact_count
    );
    let _ = writeln!(
        output,
        "storage_failure_count={}",
        migration.storage_failure_count
    );
    let _ = writeln!(output, "report_digest={}", migration.report_digest);
    output
}

fn format_momentum_mamba_repair_text_v2(report: &MomentumMambaRepairCliReportV2) -> String {
    let mut output = String::new();
    let _ = writeln!(output, "report_version={}", report.report_version);
    let _ = writeln!(output, "mode={}", report.mode);
    let _ = writeln!(output, "offline={}", report.offline);
    let _ = writeln!(output, "status={:?}", report.status);
    let _ = writeln!(
        output,
        "collapse_root_causes={:?}",
        report.collapse_root_causes
    );
    let _ = writeln!(
        output,
        "collapse_audit_digest={}",
        report.collapse_audit_digest.as_deref().unwrap_or_default()
    );
    for (name, value) in [
        (
            "representation_diagnostic_digest",
            &report.representation_diagnostic_digest,
        ),
        (
            "optimization_diagnostic_digest",
            &report.optimization_diagnostic_digest,
        ),
        (
            "probability_diagnostic_digest",
            &report.probability_diagnostic_digest,
        ),
        (
            "class_balance_diagnostic_digest",
            &report.class_balance_diagnostic_digest,
        ),
    ] {
        let _ = writeln!(output, "{name}={}", value.as_deref().unwrap_or_default());
    }
    let _ = writeln!(
        output,
        "repair_capability_status={}",
        report
            .repair_capability_status
            .map(|value| format!("{value:?}"))
            .unwrap_or_default()
    );
    let _ = writeln!(
        output,
        "repair_split_digest={}",
        report.repair_split_digest.as_deref().unwrap_or_default()
    );
    let _ = writeln!(
        output,
        "repair_registration_digest={}",
        report
            .repair_registration_digest
            .as_deref()
            .unwrap_or_default()
    );
    let _ = writeln!(
        output,
        "registered_variant_count={}",
        report.registered_variant_count
    );
    for participant in &report.participants {
        let _ = writeln!(
            output,
            "participant_digest={};model_kind={};participant_role={:?};qualification_status={:?}",
            participant.participant_digest,
            participant.model_kind,
            participant.participant_role,
            participant.qualification_status,
        );
    }
    let _ = writeln!(
        output,
        "qualified_learned_participant_count={}",
        report.qualified_learned_participant_count
    );
    let _ = writeln!(
        output,
        "qualified_comparator_count={}",
        report.qualified_comparator_count
    );
    let _ = writeln!(
        output,
        "family_digest={}",
        report.family_digest.as_deref().unwrap_or_default()
    );
    let _ = writeln!(output, "winner_selected={}", report.winner_selected);
    let _ = writeln!(
        output,
        "historical_test_accessed={}",
        report.historical_test_accessed
    );
    let _ = writeln!(output, "roster_status={:?}", report.roster_status);
    let _ = writeln!(
        output,
        "roster_digest={}",
        report.roster_digest.as_deref().unwrap_or_default()
    );
    let _ = writeln!(
        output,
        "evaluation_registration_status={:?}",
        report.evaluation_registration_status
    );
    let _ = writeln!(
        output,
        "evaluation_registration_digest={}",
        report
            .evaluation_registration_digest
            .as_deref()
            .unwrap_or_default()
    );
    let _ = writeln!(
        output,
        "minimum_accepted_timestamp_ms={}",
        report
            .minimum_accepted_timestamp_ms
            .map(|value| value.to_string())
            .unwrap_or_default()
    );
    let _ = writeln!(
        output,
        "cycle_risk_evidence_status={}",
        report
            .cycle_risk_evidence_status
            .map(|value| format!("{value:?}"))
            .unwrap_or_default()
    );
    let _ = writeln!(
        output,
        "value_quality_evidence_status={}",
        report
            .value_quality_evidence_status
            .map(|value| format!("{value:?}"))
            .unwrap_or_default()
    );
    let reward = &report.reward_eligibility_replay;
    let _ = writeln!(output, "opening_status={:?}", reward.opening_status);
    let _ = writeln!(
        output,
        "attribution_classes={:?}",
        reward.attribution_classes
    );
    let _ = writeln!(
        output,
        "reward_eligibility_statuses={:?}",
        reward.eligibility_statuses
    );
    let _ = writeln!(output, "reward_apply_count={}", reward.reward_apply_count);
    let _ = writeln!(output, "penalty_apply_count={}", reward.penalty_apply_count);
    for (name, value) in [
        ("network_requests", report.safety_counters.network_requests),
        (
            "transport_constructions",
            report.safety_counters.transport_constructions,
        ),
        ("credential_reads", report.safety_counters.credential_reads),
        (
            "prospective_row_reads",
            report.safety_counters.prospective_row_reads,
        ),
        (
            "prospective_label_openings",
            report.safety_counters.prospective_label_openings,
        ),
        (
            "future_evaluation_reads",
            report.safety_counters.future_evaluation_reads,
        ),
        (
            "historical_test_reads",
            report.safety_counters.historical_test_reads,
        ),
        (
            "active_model_changes",
            report.safety_counters.active_model_changes,
        ),
        ("chair_decisions", report.safety_counters.chair_decisions),
        ("votes", report.safety_counters.votes),
        (
            "reward_applications",
            report.safety_counters.reward_applications,
        ),
        (
            "penalty_applications",
            report.safety_counters.penalty_applications,
        ),
        ("voice_changes", report.safety_counters.voice_changes),
        (
            "cooldowns_started",
            report.safety_counters.cooldowns_started,
        ),
        ("promotions", report.safety_counters.promotions),
        ("quarantines", report.safety_counters.quarantines),
        ("executions", report.safety_counters.executions),
        (
            "active_committee_count",
            report.safety_counters.active_committee_count,
        ),
    ] {
        let _ = writeln!(output, "{name}={value}");
    }
    let _ = writeln!(output, "artifacts_written={}", report.artifacts_written);
    let _ = writeln!(
        output,
        "duplicate_artifact_count={}",
        report.duplicate_artifact_count
    );
    let _ = writeln!(
        output,
        "storage_failure_count={}",
        report.storage_failure_count
    );
    let _ = writeln!(
        output,
        "protected_artifacts_unchanged={}",
        report.protected_artifacts_unchanged
    );
    let _ = writeln!(
        output,
        "active_state_unchanged={}",
        report.active_state_unchanged
    );
    let _ = writeln!(output, "report_digest={}", report.report_digest);
    output
}

fn format_momentum_mamba_representation_text_v3(
    report: &MomentumMambaRepresentationCliReportV3,
) -> String {
    let mut output = String::new();
    let _ = writeln!(output, "report_version={}", report.report_version);
    let _ = writeln!(output, "mode={}", report.mode);
    let _ = writeln!(output, "offline={}", report.offline);
    let _ = writeln!(output, "status={:?}", report.status);
    let _ = writeln!(output, "repair_stage={:?}", report.repair_stage);
    for probe in &report.probes {
        let _ = writeln!(
            output,
            "probe_kind={:?};status={:?};representation_diagnostic_digest={};probe_digest={}",
            probe.probe_kind,
            probe.status,
            probe.representation_diagnostic_digest,
            probe.probe_digest,
        );
    }
    for (name, value) in [
        (
            "representation_audit_digest",
            &report.representation_audit_digest,
        ),
        ("split_digest", &report.split_digest),
        (
            "final_reserved_range_digest",
            &report.final_reserved_range_digest,
        ),
        ("registration_digest", &report.registration_digest),
        ("decision_digest", &report.decision_digest),
        ("family_digest", &report.family_digest),
        ("roster_digest", &report.roster_digest),
        (
            "evaluation_registration_digest",
            &report.evaluation_registration_digest,
        ),
    ] {
        let _ = writeln!(output, "{name}={}", value.as_deref().unwrap_or_default());
    }
    let _ = writeln!(
        output,
        "registered_variant_count={}",
        report.registered_variant_count
    );
    for participant in &report.participants {
        let _ = writeln!(
            output,
            "participant_digest={};model_kind={};input_kind={};qualification_status={:?};contribution_status={}",
            participant.participant_digest,
            participant.model_kind,
            participant.input_kind,
            participant.qualification_status,
            participant
                .contribution_status
                .map(|value| format!("{value:?}"))
                .unwrap_or_default(),
        );
    }
    let _ = writeln!(
        output,
        "qualified_genuine_mamba_count={}",
        report.qualified_genuine_mamba_count
    );
    let _ = writeln!(
        output,
        "qualified_raw_fallback_count={}",
        report.qualified_raw_fallback_count
    );
    let _ = writeln!(
        output,
        "qualified_comparator_count={}",
        report.qualified_comparator_count
    );
    let _ = writeln!(
        output,
        "route_decision={}",
        report
            .route_decision
            .map(|value| format!("{value:?}"))
            .unwrap_or_default()
    );
    let _ = writeln!(output, "roster_status={:?}", report.roster_status);
    let _ = writeln!(
        output,
        "evaluation_registration_status={:?}",
        report.evaluation_registration_status
    );
    let _ = writeln!(
        output,
        "minimum_accepted_timestamp_ms={}",
        report
            .minimum_accepted_timestamp_ms
            .map(|value| value.to_string())
            .unwrap_or_default()
    );
    let _ = writeln!(
        output,
        "cycle_risk_evidence_status={}",
        report
            .cycle_risk_evidence_status
            .map(|value| format!("{value:?}"))
            .unwrap_or_default()
    );
    let _ = writeln!(
        output,
        "value_quality_evidence_status={}",
        report
            .value_quality_evidence_status
            .map(|value| format!("{value:?}"))
            .unwrap_or_default()
    );
    let reward = &report.reward_eligibility_replay;
    let _ = writeln!(output, "opening_status={:?}", reward.opening_status);
    let _ = writeln!(
        output,
        "attribution_classes={:?}",
        reward.attribution_classes
    );
    let _ = writeln!(
        output,
        "reward_eligibility_statuses={:?}",
        reward.eligibility_statuses
    );
    let _ = writeln!(output, "reward_apply_count={}", reward.reward_apply_count);
    let _ = writeln!(output, "penalty_apply_count={}", reward.penalty_apply_count);
    for (name, value) in [
        ("network_requests", report.safety_counters.network_requests),
        (
            "transport_constructions",
            report.safety_counters.transport_constructions,
        ),
        ("credential_reads", report.safety_counters.credential_reads),
        (
            "prospective_row_reads",
            report.safety_counters.prospective_row_reads,
        ),
        (
            "prospective_label_openings",
            report.safety_counters.prospective_label_openings,
        ),
        (
            "historical_test_reads",
            report.safety_counters.historical_test_reads,
        ),
        (
            "future_evaluation_reads",
            report.safety_counters.future_evaluation_reads,
        ),
        (
            "active_model_changes",
            report.safety_counters.active_model_changes,
        ),
        ("chair_decisions", report.safety_counters.chair_decisions),
        ("votes", report.safety_counters.votes),
        (
            "reward_applications",
            report.safety_counters.reward_applications,
        ),
        (
            "penalty_applications",
            report.safety_counters.penalty_applications,
        ),
        ("voice_changes", report.safety_counters.voice_changes),
        (
            "cooldowns_started",
            report.safety_counters.cooldowns_started,
        ),
        ("promotions", report.safety_counters.promotions),
        ("quarantines", report.safety_counters.quarantines),
        ("executions", report.safety_counters.executions),
        (
            "active_committee_count",
            report.safety_counters.active_committee_count,
        ),
    ] {
        let _ = writeln!(output, "{name}={value}");
    }
    let _ = writeln!(output, "artifacts_written={}", report.artifacts_written);
    let _ = writeln!(
        output,
        "duplicate_artifact_count={}",
        report.duplicate_artifact_count
    );
    let _ = writeln!(
        output,
        "storage_failure_count={}",
        report.storage_failure_count
    );
    let _ = writeln!(
        output,
        "protected_artifacts_unchanged={}",
        report.protected_artifacts_unchanged
    );
    let _ = writeln!(
        output,
        "active_state_unchanged={}",
        report.active_state_unchanged
    );
    let _ = writeln!(output, "report_digest={}", report.report_digest);
    output
}

fn format_momentum_raw_feature_text_v4(report: &MomentumRawFeatureCliReportV4) -> String {
    let mut output = String::new();
    let _ = writeln!(output, "report_version={}", report.report_version);
    let _ = writeln!(output, "mode={}", report.mode);
    let _ = writeln!(output, "offline={}", report.offline);
    let _ = writeln!(output, "status={:?}", report.status);
    let _ = writeln!(
        output,
        "frozen_mamba_closure_status={}",
        report
            .frozen_mamba_closure_status
            .map(|value| format!("{value:?}"))
            .unwrap_or_default()
    );
    for (name, value) in [
        (
            "frozen_mamba_closure_digest",
            &report.frozen_mamba_closure_digest,
        ),
        ("split_digest", &report.split_digest),
        ("registration_digest", &report.registration_digest),
        ("family_digest", &report.family_digest),
        ("decision_digest", &report.decision_digest),
        ("roster_digest", &report.roster_digest),
        (
            "evaluation_registration_digest",
            &report.evaluation_registration_digest,
        ),
    ] {
        let _ = writeln!(output, "{name}={}", value.as_deref().unwrap_or_default());
    }
    for participant in &report.participants {
        let _ = writeln!(
            output,
            "participant_id={};participant_role={:?};model_kind={:?};qualification_status={:?}",
            participant.participant_id,
            participant.participant_role,
            participant.model_kind,
            participant.qualification_status,
        );
    }
    let _ = writeln!(
        output,
        "interaction_contribution_status={}",
        report
            .interaction_contribution_status
            .map(|value| format!("{value:?}"))
            .unwrap_or_default()
    );
    let _ = writeln!(
        output,
        "qualified_learned_count={}",
        report.qualified_learned_count
    );
    let _ = writeln!(
        output,
        "qualified_benchmark_count={}",
        report.qualified_benchmark_count
    );
    let _ = writeln!(
        output,
        "path_decision={}",
        report
            .path_decision
            .map(|value| format!("{value:?}"))
            .unwrap_or_default()
    );
    let _ = writeln!(output, "roster_status={:?}", report.roster_status);
    let _ = writeln!(
        output,
        "evaluation_registration_status={:?}",
        report.evaluation_registration_status
    );
    let _ = writeln!(
        output,
        "minimum_accepted_timestamp_ms={}",
        report
            .minimum_accepted_timestamp_ms
            .map(|value| value.to_string())
            .unwrap_or_default()
    );
    let _ = writeln!(
        output,
        "cycle_risk_evidence_status={}",
        report
            .cycle_risk_evidence_status
            .map(|value| format!("{value:?}"))
            .unwrap_or_default()
    );
    let _ = writeln!(
        output,
        "value_quality_evidence_status={}",
        report
            .value_quality_evidence_status
            .map(|value| format!("{value:?}"))
            .unwrap_or_default()
    );
    let reward = &report.reward_eligibility_replay;
    let _ = writeln!(output, "opening_status={:?}", reward.opening_status);
    let _ = writeln!(
        output,
        "attribution_classes={:?}",
        reward.attribution_classes
    );
    let _ = writeln!(
        output,
        "reward_eligibility_statuses={:?}",
        reward.eligibility_statuses
    );
    let _ = writeln!(output, "reward_apply_count={}", reward.reward_apply_count);
    let _ = writeln!(output, "penalty_apply_count={}", reward.penalty_apply_count);
    for (name, value) in [
        ("network_requests", report.safety_counters.network_requests),
        (
            "transport_constructions",
            report.safety_counters.transport_constructions,
        ),
        ("credential_reads", report.safety_counters.credential_reads),
        (
            "prospective_row_reads",
            report.safety_counters.prospective_row_reads,
        ),
        (
            "prospective_label_openings",
            report.safety_counters.prospective_label_openings,
        ),
        (
            "historical_test_reads",
            report.safety_counters.historical_test_reads,
        ),
        (
            "future_evaluation_reads",
            report.safety_counters.future_evaluation_reads,
        ),
        (
            "final_reserve_row_reads",
            report.safety_counters.final_reserve_row_reads,
        ),
        (
            "final_reserve_label_reads",
            report.safety_counters.final_reserve_label_reads,
        ),
        (
            "active_model_changes",
            report.safety_counters.active_model_changes,
        ),
        ("chair_decisions", report.safety_counters.chair_decisions),
        ("votes", report.safety_counters.votes),
        (
            "reward_applications",
            report.safety_counters.reward_applications,
        ),
        (
            "penalty_applications",
            report.safety_counters.penalty_applications,
        ),
        ("voice_changes", report.safety_counters.voice_changes),
        (
            "cooldowns_started",
            report.safety_counters.cooldowns_started,
        ),
        ("promotions", report.safety_counters.promotions),
        ("quarantines", report.safety_counters.quarantines),
        ("executions", report.safety_counters.executions),
        (
            "active_committee_count",
            report.safety_counters.active_committee_count,
        ),
    ] {
        let _ = writeln!(output, "{name}={value}");
    }
    let _ = writeln!(output, "artifacts_written={}", report.artifacts_written);
    let _ = writeln!(
        output,
        "duplicate_artifact_count={}",
        report.duplicate_artifact_count
    );
    let _ = writeln!(
        output,
        "storage_failure_count={}",
        report.storage_failure_count
    );
    let _ = writeln!(
        output,
        "protected_artifacts_unchanged={}",
        report.protected_artifacts_unchanged
    );
    let _ = writeln!(
        output,
        "active_state_unchanged={}",
        report.active_state_unchanged
    );
    let _ = writeln!(output, "report_digest={}", report.report_digest);
    output
}

fn run_momentum_raw_feature_cli_v4(
    config_path: &Path,
    output_format: &str,
    status: bool,
    dry_run: bool,
    execute_local: bool,
    allow_network: bool,
) -> Result<(), String> {
    if usize::from(status) + usize::from(dry_run) + usize::from(execute_local) != 1 {
        return Err("select exactly one Momentum raw-feature V4 mode".to_string());
    }
    if allow_network {
        return Err("Momentum raw-feature V4 is offline-only".to_string());
    }
    if output_format != "text" && output_format != "json" {
        return Err("unsupported Momentum raw-feature V4 output format".to_string());
    }
    let prior = build_persisted_learning_intent_migration_cli_report_v1(
        config_path,
        true,
        false,
        false,
        false,
    )?;
    let mode = if status {
        crate::model::AgentPrivateLearningRunModeV0::Status
    } else if dry_run {
        crate::model::AgentPrivateLearningRunModeV0::DryRun
    } else {
        crate::model::AgentPrivateLearningRunModeV0::ExecuteLocal
    };
    let snapshots =
        crate::model::load_local_learning_snapshots_v0(Path::new("data/local_snapshots"))?;
    let root = crate::model::default_private_learning_root_v0();
    let reservation = crate::model::load_protected_evaluation_reservation_v1(
        config_path
            .parent()
            .ok_or("Momentum raw-feature reservation directory unavailable")?,
    )?;
    let result = crate::model::run_momentum_raw_feature_v4(root, &snapshots, &reservation, mode);
    let participants = result
        .family
        .as_ref()
        .map(|family| {
            family
                .participants
                .iter()
                .filter_map(|participant| {
                    let receipt = family.qualification_receipts.iter().find(|receipt| {
                        receipt.participant_digest == participant.participant_digest
                    })?;
                    Some(MomentumRawFeatureParticipantCliV4 {
                        participant_id: participant.participant_id.clone(),
                        participant_role: participant.participant_role,
                        model_kind: participant.model_kind,
                        qualification_status: receipt.status,
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let evidence_status = |agent_id: &str| {
        prior
            .candidate_families
            .iter()
            .find(|family| family.agent_id == agent_id)
            .and_then(|family| family.evidence_status)
    };
    let report = MomentumRawFeatureCliReportV4 {
        report_version: "momentum-raw-feature-cli-report-v4",
        mode: if status {
            "status"
        } else if dry_run {
            "dry-run"
        } else {
            "execute-local"
        }
        .to_string(),
        offline: true,
        status: result.status,
        frozen_mamba_closure_status: result.closure.as_ref().map(|value| value.decision),
        frozen_mamba_closure_digest: result
            .closure
            .as_ref()
            .map(|value| value.closure_digest.clone()),
        split_digest: result
            .split
            .as_ref()
            .map(|value| value.split_digest.clone()),
        registration_digest: result
            .registration
            .as_ref()
            .map(|value| value.registration_digest.clone()),
        participants,
        interaction_contribution_status: result
            .family
            .as_ref()
            .and_then(|family| family.interaction_contribution_audit.as_ref())
            .map(|audit| audit.contribution_status),
        qualified_learned_count: result
            .family
            .as_ref()
            .map_or(0, |family| family.qualified_learned_count),
        qualified_benchmark_count: result
            .family
            .as_ref()
            .map_or(0, |family| family.qualified_benchmark_count),
        family_digest: result
            .family
            .as_ref()
            .map(|value| value.family_digest.clone()),
        path_decision: result.decision.as_ref().map(|value| value.decision),
        decision_digest: result
            .decision
            .as_ref()
            .map(|value| value.decision_digest.clone()),
        roster_status: result.roster_status,
        roster_digest: result
            .roster
            .as_ref()
            .map(|value| value.roster_digest.clone()),
        evaluation_registration_status: result.evaluation_registration_status,
        evaluation_registration_digest: result
            .evaluation_registration
            .as_ref()
            .map(|value| value.registration_digest.clone()),
        minimum_accepted_timestamp_ms: result
            .evaluation_registration
            .as_ref()
            .map(|value| value.minimum_accepted_timestamp_ms),
        cycle_risk_evidence_status: evidence_status("cycle_risk_skeptic"),
        value_quality_evidence_status: evidence_status("value_quality_filter"),
        reward_eligibility_replay: prior.reward_eligibility_replay,
        artifacts_written: result.artifacts_written,
        duplicate_artifact_count: result.duplicate_artifact_count,
        storage_failure_count: result.storage_failure_count,
        protected_artifacts_unchanged: result.protected_artifacts_unchanged,
        active_state_unchanged: result.active_state_unchanged,
        safety_counters: result.safety_counters,
        report_digest: result.report_digest,
    };
    if output_format == "json" {
        println!(
            "{}",
            serde_json::to_string(&report)
                .map_err(|_| "Momentum raw-feature V4 report encoding failed")?
        );
    } else {
        print!("{}", format_momentum_raw_feature_text_v4(&report));
    }
    if report.storage_failure_count > 0
        || !report.protected_artifacts_unchanged
        || !report.active_state_unchanged
        || report.status == crate::model::MomentumRawFeatureExecutionStatusV4::TechnicalFailure
    {
        return Err("Momentum raw-feature V4 verification failed".to_string());
    }
    Ok(())
}

fn run_momentum_mamba_representation_cli_v3(
    config_path: &Path,
    output_format: &str,
    status: bool,
    dry_run: bool,
    execute_local: bool,
    allow_network: bool,
) -> Result<(), String> {
    if usize::from(status) + usize::from(dry_run) + usize::from(execute_local) != 1 {
        return Err("select exactly one Momentum Mamba representation V3 mode".to_string());
    }
    if allow_network {
        return Err("Momentum Mamba representation V3 is offline-only".to_string());
    }
    if output_format != "text" && output_format != "json" {
        return Err("unsupported Momentum Mamba representation V3 output format".to_string());
    }
    let prior = build_persisted_learning_intent_migration_cli_report_v1(
        config_path,
        true,
        false,
        false,
        false,
    )?;
    let mode = if status {
        crate::model::AgentPrivateLearningRunModeV0::Status
    } else if dry_run {
        crate::model::AgentPrivateLearningRunModeV0::DryRun
    } else {
        crate::model::AgentPrivateLearningRunModeV0::ExecuteLocal
    };
    let snapshots =
        crate::model::load_local_learning_snapshots_v0(Path::new("data/local_snapshots"))?;
    let root = crate::model::default_private_learning_root_v0();
    let reservation = crate::model::load_protected_evaluation_reservation_v1(
        config_path
            .parent()
            .ok_or("Momentum representation reservation directory unavailable")?,
    )?;
    let result =
        crate::model::run_momentum_mamba_representation_v3(root, &snapshots, &reservation, mode);
    let probes = result
        .representation_audit
        .as_ref()
        .map(|audit| {
            audit
                .probes
                .iter()
                .map(|probe| MomentumRepresentationProbeCliV3 {
                    probe_kind: probe.probe_kind,
                    status: probe.status,
                    representation_diagnostic_digest: probe
                        .representation_diagnostic_digest
                        .clone(),
                    probe_digest: probe.probe_digest.clone(),
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let participants = result
        .family
        .as_ref()
        .map(|family| {
            family
                .participants
                .iter()
                .filter_map(|participant| {
                    let receipt = family.qualification_receipts.iter().find(|receipt| {
                        receipt.participant_digest == participant.participant_digest
                    })?;
                    let contribution_status = family
                        .contribution_audits
                        .iter()
                        .find(|audit| audit.participant_digest == participant.participant_digest)
                        .map(|audit| audit.contribution_status);
                    Some(MomentumRepresentationParticipantCliV3 {
                        participant_digest: participant.participant_digest.clone(),
                        model_kind: participant.model_kind.clone(),
                        input_kind: participant.input_kind.clone(),
                        qualification_status: receipt.status,
                        contribution_status,
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let evidence_status = |agent_id: &str| {
        prior
            .candidate_families
            .iter()
            .find(|family| family.agent_id == agent_id)
            .and_then(|family| family.evidence_status)
    };
    let report = MomentumMambaRepresentationCliReportV3 {
        report_version: "momentum-mamba-representation-cli-report-v3",
        mode: if status {
            "status"
        } else if dry_run {
            "dry-run"
        } else {
            "execute-local"
        }
        .to_string(),
        offline: true,
        status: result.status,
        repair_stage: result.repair_stage,
        probes,
        representation_audit_digest: result
            .representation_audit
            .as_ref()
            .map(|audit| audit.audit_digest.clone()),
        split_digest: result
            .split
            .as_ref()
            .map(|split| split.split_digest.clone()),
        final_reserved_range_digest: result.split.as_ref().map(|split| {
            crate::core::stable_hash_string(&format!(
                "momentum-representation-final-reserve-v3:{}:{}",
                split.final_reserved_range.start, split.final_reserved_range.end
            ))
        }),
        registration_digest: result
            .registration
            .as_ref()
            .map(|registration| registration.registration_digest.clone()),
        registered_variant_count: result
            .registration
            .as_ref()
            .map_or(0, |registration| registration.variants.len()),
        participants,
        qualified_genuine_mamba_count: result.family.as_ref().map_or(0, |family| {
            family.qualified_mamba_only_count + family.qualified_mamba_hybrid_count
        }),
        qualified_raw_fallback_count: result
            .family
            .as_ref()
            .map_or(0, |family| family.qualified_raw_fallback_count),
        qualified_comparator_count: result
            .family
            .as_ref()
            .map_or(0, |family| family.qualified_comparator_count),
        route_decision: result.decision.as_ref().map(|decision| decision.decision),
        decision_digest: result
            .decision
            .as_ref()
            .map(|decision| decision.decision_digest.clone()),
        family_digest: result
            .family
            .as_ref()
            .map(|family| family.family_digest.clone()),
        roster_status: result.roster_status,
        roster_digest: result
            .roster
            .as_ref()
            .map(|roster| roster.roster_digest.clone()),
        evaluation_registration_status: result.evaluation_registration_status,
        evaluation_registration_digest: result
            .evaluation_registration
            .as_ref()
            .map(|registration| registration.registration_digest.clone()),
        minimum_accepted_timestamp_ms: result
            .evaluation_registration
            .as_ref()
            .map(|registration| registration.minimum_accepted_timestamp_ms),
        cycle_risk_evidence_status: evidence_status("cycle_risk_skeptic"),
        value_quality_evidence_status: evidence_status("value_quality_filter"),
        reward_eligibility_replay: prior.reward_eligibility_replay,
        artifacts_written: result.artifacts_written,
        duplicate_artifact_count: result.duplicate_artifact_count,
        storage_failure_count: result.storage_failure_count,
        protected_artifacts_unchanged: result.protected_artifacts_unchanged,
        active_state_unchanged: result.active_state_unchanged,
        safety_counters: result.safety_counters,
        report_digest: result.report_digest,
    };
    if output_format == "json" {
        println!(
            "{}",
            serde_json::to_string(&report)
                .map_err(|_| "Momentum Mamba representation V3 report encoding failed")?
        );
    } else {
        print!("{}", format_momentum_mamba_representation_text_v3(&report));
    }
    if report.storage_failure_count > 0
        || !report.protected_artifacts_unchanged
        || !report.active_state_unchanged
        || report.status == crate::model::MomentumRepresentationExecutionStatusV3::TechnicalFailure
    {
        return Err("Momentum Mamba representation V3 verification failed".to_string());
    }
    Ok(())
}

fn run_momentum_mamba_repair_cli_v2(
    config_path: &Path,
    output_format: &str,
    status: bool,
    dry_run: bool,
    execute_local: bool,
    allow_network: bool,
) -> Result<(), String> {
    if usize::from(status) + usize::from(dry_run) + usize::from(execute_local) != 1 {
        return Err("select exactly one Momentum Mamba repair mode".to_string());
    }
    if allow_network {
        return Err("Momentum Mamba repair is offline-only".to_string());
    }
    if output_format != "text" && output_format != "json" {
        return Err("unsupported Momentum Mamba repair output format".to_string());
    }
    let prior = build_persisted_learning_intent_migration_cli_report_v1(
        config_path,
        status,
        dry_run,
        execute_local,
        false,
    )?;
    let mode = if status {
        crate::model::AgentPrivateLearningRunModeV0::Status
    } else if dry_run {
        crate::model::AgentPrivateLearningRunModeV0::DryRun
    } else {
        crate::model::AgentPrivateLearningRunModeV0::ExecuteLocal
    };
    let snapshots =
        crate::model::load_local_learning_snapshots_v0(Path::new("data/local_snapshots"))?;
    let root = crate::model::default_private_learning_root_v0();
    let reservation = crate::model::load_protected_evaluation_reservation_v1(
        config_path
            .parent()
            .ok_or("Momentum repair reservation directory unavailable")?,
    )?;
    let repair = crate::model::run_momentum_mamba_repair_v2(root, &snapshots, &reservation, mode);
    let participants = repair
        .family
        .as_ref()
        .map(|family| {
            family
                .participants
                .iter()
                .filter_map(|participant| {
                    family
                        .qualification_receipts
                        .iter()
                        .find(|receipt| {
                            receipt.participant_digest == participant.participant_digest
                        })
                        .map(|receipt| MomentumMambaRepairParticipantCliV2 {
                            participant_digest: participant.participant_digest.clone(),
                            model_kind: participant.model_kind.clone(),
                            participant_role: participant.participant_role,
                            qualification_status: receipt.qualification_status,
                        })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let evidence_status = |agent_id: &str| {
        prior
            .candidate_families
            .iter()
            .find(|family| family.agent_id == agent_id)
            .and_then(|family| family.evidence_status)
    };
    let report = MomentumMambaRepairCliReportV2 {
        report_version: "momentum-mamba-repair-cli-report-v2",
        mode: if status {
            "status"
        } else if dry_run {
            "dry-run"
        } else {
            "execute-local"
        }
        .to_string(),
        offline: true,
        status: repair.status,
        collapse_root_causes: repair
            .collapse_audit
            .as_ref()
            .map(|audit| audit.root_causes.clone())
            .unwrap_or_default(),
        collapse_audit_digest: repair
            .collapse_audit
            .as_ref()
            .map(|audit| audit.audit_digest.clone()),
        representation_diagnostic_digest: repair
            .collapse_audit
            .as_ref()
            .map(|audit| audit.representation_diagnostic_digest.clone()),
        optimization_diagnostic_digest: repair
            .collapse_audit
            .as_ref()
            .map(|audit| audit.optimization_diagnostic_digest.clone()),
        probability_diagnostic_digest: repair
            .collapse_audit
            .as_ref()
            .map(|audit| audit.probability_diagnostic_digest.clone()),
        class_balance_diagnostic_digest: repair
            .collapse_audit
            .as_ref()
            .map(|audit| audit.class_balance_diagnostic_digest.clone()),
        repair_capability_status: repair
            .collapse_audit
            .as_ref()
            .map(|audit| audit.repair_capability_status),
        repair_split_digest: repair
            .repair_split
            .as_ref()
            .map(|split| split.split_digest.clone()),
        repair_registration_digest: repair
            .repair_registration
            .as_ref()
            .map(|registration| registration.registration_digest.clone()),
        registered_variant_count: repair
            .repair_registration
            .as_ref()
            .map_or(0, |registration| registration.allowed_variant_configs.len()),
        participants,
        qualified_learned_participant_count: repair
            .family
            .as_ref()
            .map_or(0, |family| family.qualified_learned_participant_count),
        qualified_comparator_count: repair
            .family
            .as_ref()
            .map_or(0, |family| family.qualified_comparator_count),
        family_digest: repair
            .family
            .as_ref()
            .map(|family| family.family_digest.clone()),
        winner_selected: repair
            .family
            .as_ref()
            .is_some_and(|family| family.winner_selected),
        historical_test_accessed: repair
            .family
            .as_ref()
            .is_some_and(|family| family.historical_test_accessed),
        roster_status: repair.roster_status,
        roster_digest: repair
            .roster
            .as_ref()
            .map(|roster| roster.roster_digest.clone()),
        evaluation_registration_status: repair.evaluation_registration_status,
        evaluation_registration_digest: repair
            .evaluation_registration
            .as_ref()
            .map(|registration| registration.registration_digest.clone()),
        minimum_accepted_timestamp_ms: repair
            .evaluation_registration
            .as_ref()
            .map(|registration| registration.minimum_accepted_timestamp_ms),
        cycle_risk_evidence_status: evidence_status("cycle_risk_skeptic"),
        value_quality_evidence_status: evidence_status("value_quality_filter"),
        reward_eligibility_replay: prior.reward_eligibility_replay,
        artifacts_written: repair.artifacts_written,
        duplicate_artifact_count: repair.duplicate_artifact_count,
        storage_failure_count: repair.storage_failure_count,
        protected_artifacts_unchanged: repair.protected_artifacts_unchanged,
        active_state_unchanged: repair.active_state_unchanged,
        safety_counters: repair.safety_counters,
        report_digest: repair.report_digest,
    };
    if output_format == "json" {
        println!(
            "{}",
            serde_json::to_string(&report)
                .map_err(|_| "Momentum Mamba repair report encoding failed")?
        );
    } else {
        print!("{}", format_momentum_mamba_repair_text_v2(&report));
    }
    if report.storage_failure_count > 0
        || !report.protected_artifacts_unchanged
        || !report.active_state_unchanged
        || report.status == crate::model::MomentumMambaRepairExecutionStatusV2::TechnicalFailure
    {
        return Err("Momentum Mamba repair verification failed".to_string());
    }
    Ok(())
}

fn build_persisted_learning_intent_migration_cli_report_v1(
    config_path: &Path,
    status: bool,
    dry_run: bool,
    execute_local: bool,
    allow_network: bool,
) -> Result<PersistedLearningIntentMigrationCliReportV1, String> {
    if usize::from(status) + usize::from(dry_run) + usize::from(execute_local) != 1 {
        return Err("select exactly one persisted intent migration mode".into());
    }
    if allow_network {
        return Err("persisted learning intent migration is offline-only".into());
    }
    let mode = if status {
        crate::model::AgentPrivateLearningRunModeV0::Status
    } else if dry_run {
        crate::model::AgentPrivateLearningRunModeV0::DryRun
    } else {
        crate::model::AgentPrivateLearningRunModeV0::ExecuteLocal
    };
    let snapshots =
        crate::model::load_local_learning_snapshots_v0(Path::new("data/local_snapshots"))?;
    let root = crate::model::default_private_learning_root_v0();
    let gap_statuses =
        crate::model::load_latest_agent_canonical_view_gap_statuses_v1(root).unwrap_or_default();
    let migration =
        crate::model::run_persisted_learning_intent_migration_v1(root, &snapshots, mode);
    let inputs = if execute_local && migration.report.storage_failure_count == 0 {
        let cutoff = migration
            .canonical_input
            .as_ref()
            .map(|input| input.input.intent.information_cutoff_ms)
            .unwrap_or_default();
        crate::model::build_agent_private_learning_inputs_v1(&snapshots, cutoff, root, mode)
    } else {
        migration
            .canonical_input
            .clone()
            .into_iter()
            .collect::<Vec<_>>()
    };
    let mut families = crate::model::run_agent_private_learning_candidates_v1(&inputs, mode);
    if execute_local && migration.report.storage_failure_count == 0 {
        crate::model::persist_agent_candidate_families_report_v1(&mut families, root);
    }
    let reservation = crate::model::load_protected_evaluation_reservation_v1(
        config_path
            .parent()
            .ok_or("persisted intent migration reservation directory unavailable")?,
    )?;
    let mut evaluations =
        crate::model::run_agent_candidate_evaluations_v1(&families, &inputs, &reservation, mode);
    if execute_local
        && migration.report.storage_failure_count == 0
        && families.storage_failure_count == 0
    {
        crate::model::persist_agent_candidate_evaluations_report_v1(&mut evaluations, root);
    }
    let reward_eligibility_replay = replay_persisted_reward_eligibility_v1(config_path)?;
    let active_committee_count = migration.report.safety_counters.active_committee_count;
    let report = PersistedLearningIntentMigrationCliReportV1 {
        report_version: "persisted-learning-intent-migration-cli-report-v1",
        mode: if status {
            "status"
        } else if dry_run {
            "dry-run"
        } else {
            "execute-local"
        }
        .to_string(),
        offline: true,
        migration: migration.report,
        candidate_families: families
            .results
            .iter()
            .map(|result| {
                migrated_candidate_family_cli_v1(
                    result,
                    gap_statuses.get(&result.agent_id).copied(),
                )
            })
            .collect(),
        evaluation_registrations: evaluations
            .results
            .iter()
            .map(migrated_evaluation_registration_cli_v1)
            .collect(),
        reward_eligibility_replay,
        new_network_requests: 0,
        transport_constructions: 0,
        new_credential_reads: 0,
        new_prospective_row_reads: 0,
        new_prospective_label_openings: 0,
        new_future_evaluation_reads: 0,
        historical_test_reads_v1: families.safety_counters.historical_test_reads_v1
            + evaluations.safety_counters.historical_test_reads_v1,
        active_committee_count,
        active_model_changes: families.safety_counters.active_model_changes
            + evaluations.safety_counters.active_model_changes,
        chair_decisions: families.safety_counters.chair_decisions
            + evaluations.safety_counters.chair_decisions,
        votes: families.safety_counters.votes + evaluations.safety_counters.votes,
        reward_applications: families.safety_counters.rewards + evaluations.safety_counters.rewards,
        penalty_applications: families.safety_counters.penalties
            + evaluations.safety_counters.penalties,
        voice_changes: families.safety_counters.voice_changes
            + evaluations.safety_counters.voice_changes,
        cooldowns_started: 0,
        promotions: families.safety_counters.promotions + evaluations.safety_counters.promotions,
        quarantines: 0,
        executions: families.safety_counters.executions + evaluations.safety_counters.executions,
    };
    if report.migration.storage_failure_count > 0
        || families.storage_failure_count > 0
        || evaluations.storage_failure_count > 0
        || !report.migration.protected_artifacts_unchanged
        || !report.migration.active_state_unchanged
    {
        return Err("persisted learning intent migration verification failed".into());
    }
    Ok(report)
}

fn run_persisted_learning_intent_migration_cli_v1(
    config_path: &Path,
    output_format: &str,
    status: bool,
    dry_run: bool,
    execute_local: bool,
    allow_network: bool,
) -> Result<(), String> {
    if output_format != "text" && output_format != "json" {
        return Err("unsupported persisted intent migration output format".into());
    }
    let report = build_persisted_learning_intent_migration_cli_report_v1(
        config_path,
        status,
        dry_run,
        execute_local,
        allow_network,
    )?;
    if output_format == "json" {
        println!(
            "{}",
            serde_json::to_string(&report)
                .map_err(|_| "persisted intent migration report encoding failed")?
        );
    } else {
        print!("{}", format_persisted_intent_migration_text_v1(&report));
    }
    Ok(())
}

fn run_agent_candidate_family_cli_v1(
    output_format: &str,
    status: bool,
    dry_run: bool,
    execute_local: bool,
    allow_network: bool,
    registration_requested: bool,
) -> Result<(), String> {
    if usize::from(status) + usize::from(dry_run) + usize::from(execute_local) != 1 {
        return Err(
            "select exactly one V1 learning mode: --status, --dry-run, or --execute-local"
                .to_string(),
        );
    }
    if allow_network {
        return Err("V1 candidate generation and registration are offline-only".to_string());
    }
    let mode = if status {
        crate::model::AgentPrivateLearningRunModeV0::Status
    } else if dry_run {
        crate::model::AgentPrivateLearningRunModeV0::DryRun
    } else {
        crate::model::AgentPrivateLearningRunModeV0::ExecuteLocal
    };
    let snapshots =
        crate::model::load_local_learning_snapshots_v0(Path::new("data/local_snapshots"))?;
    let cutoff = snapshots
        .iter()
        .filter_map(|snapshot| snapshot.actual_end_timestamp_ms)
        .max()
        .unwrap_or(1);
    let root = crate::model::default_private_learning_root_v0();
    let inputs =
        crate::model::build_agent_private_learning_inputs_v1(&snapshots, cutoff, root, mode);
    let mut families = crate::model::run_agent_private_learning_candidates_v1(&inputs, mode);
    if execute_local {
        crate::model::persist_agent_candidate_families_report_v1(&mut families, root);
    }
    if registration_requested {
        let reservation =
            crate::model::load_protected_evaluation_reservation_v1(Path::new("config/local"))?;
        let mut report = crate::model::run_agent_candidate_evaluations_v1(
            &families,
            &inputs,
            &reservation,
            mode,
        );
        if execute_local && families.storage_failure_count == 0 {
            crate::model::persist_agent_candidate_evaluations_report_v1(&mut report, root);
        }
        let summaries = crate::model::public_candidate_evaluation_summaries_v1(&report);
        if output_format == "json" {
            println!(
                "{}",
                serde_json::json!({
                    "mode": format!("{:?}", report.mode),
                    "registrations": summaries,
                    "active_state_unchanged": report.active_state_unchanged,
                    "duplicate_artifact_count": report.duplicate_artifact_count,
                    "storage_failure_count": report.storage_failure_count,
                    "safety_counters": report.safety_counters,
                    "report_digest": report.report_digest,
                })
            );
        } else {
            println!("mode={:?}", report.mode);
            for summary in summaries {
                println!(
                    "agent={};session_digest={};family_digest={};participant_count={};historical_test_access_count={};minimum_accepted_timestamp_ms={};exclusion_digest={};registration_status={:?}",
                    summary.agent_id,
                    summary.session_digest.unwrap_or_default(),
                    summary.family_digest.unwrap_or_default(),
                    summary.participant_count,
                    summary.historical_test_access_count,
                    summary.minimum_accepted_timestamp_ms.unwrap_or_default(),
                    summary.exclusion_digest.unwrap_or_default(),
                    summary.registration_status,
                );
            }
            print_v1_safety_counters(&report.safety_counters);
            println!("duplicate_artifacts={}", report.duplicate_artifact_count);
            println!("storage_failures={}", report.storage_failure_count);
            println!("report_digest={}", report.report_digest);
        }
        if families.storage_failure_count > 0 || report.storage_failure_count > 0 {
            return Err("one or more V1 evaluation artifacts failed verification".to_string());
        }
        return Ok(());
    }
    let summaries = crate::model::public_candidate_family_summaries_v1(&families);
    if output_format == "json" {
        println!(
            "{}",
            serde_json::json!({
                "mode": format!("{:?}", families.mode),
                "candidate_families": summaries,
                "active_state_unchanged": families.active_state_unchanged,
                "duplicate_artifact_count": families.duplicate_artifact_count,
                "storage_failure_count": families.storage_failure_count,
                "safety_counters": families.safety_counters,
                "report_digest": families.report_digest,
            })
        );
    } else {
        println!("mode={:?}", families.mode);
        for summary in summaries {
            println!(
                "agent={};session_digest={};view_digest={};projection_digest={};family_digest={};participant_count={};historical_test_access_count={};status={:?}",
                summary.agent_id,
                summary.session_digest.unwrap_or_default(),
                summary.view_digest.unwrap_or_default(),
                summary.projection_digest.unwrap_or_default(),
                summary.family_digest.unwrap_or_default(),
                summary.participant_count,
                summary.historical_test_access_count,
                summary.status,
            );
        }
        print_v1_safety_counters(&families.safety_counters);
        println!("duplicate_artifacts={}", families.duplicate_artifact_count);
        println!("storage_failures={}", families.storage_failure_count);
        println!("report_digest={}", families.report_digest);
    }
    if families.storage_failure_count > 0 {
        return Err("one or more V1 candidate artifacts failed verification".to_string());
    }
    Ok(())
}

fn print_v1_safety_counters(counters: &crate::model::AgentLearningSafetyCountersV1) {
    println!("active_committee_count={}", counters.active_committee_count);
    println!("network_requests={}", counters.network_requests);
    println!("credential_reads={}", counters.credential_reads);
    println!("prospective_row_reads={}", counters.prospective_row_reads);
    println!(
        "prospective_label_reads={}",
        counters.prospective_label_reads
    );
    println!("prospective_mutations={}", counters.prospective_mutations);
    println!(
        "historical_test_reads_v1={}",
        counters.historical_test_reads_v1
    );
    println!("active_model_changes={}", counters.active_model_changes);
    println!("chair_decisions={}", counters.chair_decisions);
    println!("votes={}", counters.votes);
    println!("rewards={}", counters.rewards);
    println!("penalties={}", counters.penalties);
    println!("voice_changes={}", counters.voice_changes);
    println!("promotions={}", counters.promotions);
    println!("executions={}", counters.executions);
}

fn run_agent_candidate_evaluation_cli_v0(
    output_format: &str,
    status: bool,
    dry_run: bool,
    execute_local: bool,
    allow_network: bool,
    registration_requested: bool,
) -> Result<(), String> {
    if usize::from(status) + usize::from(dry_run) + usize::from(execute_local) != 1 {
        return Err(
            "select exactly one candidate audit mode: --status, --dry-run, or --execute-local"
                .to_string(),
        );
    }
    if allow_network {
        return Err("candidate evidence audit and registration are offline-only".to_string());
    }
    let mode = if status {
        crate::model::AgentPrivateLearningRunModeV0::Status
    } else if dry_run {
        crate::model::AgentPrivateLearningRunModeV0::DryRun
    } else {
        crate::model::AgentPrivateLearningRunModeV0::ExecuteLocal
    };
    let report = crate::model::run_agent_candidate_evaluation_v0(
        crate::model::default_private_learning_root_v0(),
        mode,
        registration_requested,
        None,
    );
    let summaries = crate::model::public_candidate_evaluation_summaries_v0(&report);
    if output_format == "json" {
        println!(
            "{}",
            serde_json::json!({
                "mode": format!("{:?}", report.mode),
                "registration_requested": report.registration_requested,
                "candidates": summaries,
                "active_state_unchanged": report.active_state_unchanged,
                "duplicate_artifact_count": report.duplicate_artifact_count,
                "storage_failure_count": report.storage_failure_count,
                "safety_counters": report.safety_counters,
                "report_digest": report.report_digest,
            })
        );
    } else {
        println!("mode={:?}", report.mode);
        println!("registration_requested={registration_requested}");
        for summary in summaries {
            println!(
                "agent={};candidate_digest={};session_digest={};view_digest={};projection_digest={};historical_test_status={:?};evidence_usage_ledger_digest={};identity_audit_digest={};evaluation_cutoff_exclusive_ms={};registration_status={:?};comparator_count={}",
                summary.agent_id,
                summary.candidate_digest.unwrap_or_default(),
                summary.session_digest.unwrap_or_default(),
                summary.view_digest.unwrap_or_default(),
                summary.projection_digest.unwrap_or_default(),
                summary.historical_test_status,
                summary.evidence_usage_ledger_digest.unwrap_or_default(),
                summary.identity_audit_digest.unwrap_or_default(),
                summary.evaluation_cutoff_exclusive_ms.unwrap_or_default(),
                summary.registration_status,
                summary.comparator_count,
            );
        }
        println!(
            "network_requests={}",
            report.safety_counters.network_requests
        );
        println!(
            "prospective_row_reads={}",
            report.safety_counters.prospective_row_reads
        );
        println!(
            "prospective_label_reads={}",
            report.safety_counters.prospective_label_reads
        );
        println!(
            "active_model_changes={}",
            report.safety_counters.active_model_changes
        );
        println!("chair_decisions={}", report.safety_counters.chair_decisions);
        println!("votes={}", report.safety_counters.votes);
        println!("rewards={}", report.safety_counters.rewards);
        println!("promotions={}", report.safety_counters.promotions);
        println!("executions={}", report.safety_counters.executions);
        println!("duplicate_artifacts={}", report.duplicate_artifact_count);
        println!("storage_failures={}", report.storage_failure_count);
        println!("report_digest={}", report.report_digest);
    }
    if report.storage_failure_count > 0 {
        return Err("one or more candidate audit artifacts failed verification".to_string());
    }
    Ok(())
}

fn run_agent_private_learning_sessions_cli_v0(
    output_format: &str,
    status: bool,
    dry_run: bool,
    execute_local: bool,
    allow_network: bool,
) -> Result<(), String> {
    if usize::from(status) + usize::from(dry_run) + usize::from(execute_local) != 1 {
        return Err(
            "select exactly one private learning mode: --status, --dry-run, or --execute-local"
                .to_string(),
        );
    }
    if allow_network {
        return Err("agent-private learning sessions are offline-only".to_string());
    }
    let evidence_root = Path::new("data/local_snapshots");
    let snapshots = crate::model::load_local_learning_snapshots_v0(evidence_root)?;
    let cutoff = snapshots
        .iter()
        .filter_map(|snapshot| snapshot.actual_end_timestamp_ms)
        .max()
        .unwrap_or(1);
    let inputs = crate::model::build_agent_private_learning_inputs_v0(&snapshots, cutoff)?;
    let mode = if status {
        crate::model::AgentPrivateLearningRunModeV0::Status
    } else if dry_run {
        crate::model::AgentPrivateLearningRunModeV0::DryRun
    } else {
        crate::model::AgentPrivateLearningRunModeV0::ExecuteLocal
    };
    let mut report = crate::model::run_agent_private_learning_sessions_v0(&inputs, mode);
    if execute_local {
        crate::model::persist_agent_private_learning_report_v0(
            &mut report,
            crate::model::default_private_learning_root_v0(),
        );
    }
    let summaries = crate::model::public_session_summaries_v0(&report);
    if output_format == "json" {
        println!(
            "{}",
            serde_json::json!({
                "mode": format!("{:?}", report.mode),
                "capability_registry_digest": report.capability_registry.registry_digest,
                "sessions": summaries,
                "active_state_unchanged": report.active_state_unchanged,
                "duplicate_artifact_count": report.duplicate_artifact_count,
                "storage_failure_count": report.storage_failure_count,
                "authority_counters": {
                    "network_requests": report.safety_counters.network_requests,
                    "credential_reads": report.safety_counters.credential_reads,
                    "prospective_artifact_mutations": report.safety_counters.prospective_artifact_mutations,
                    "prospective_label_reads": report.safety_counters.prospective_label_reads,
                    "chair_decisions": report.safety_counters.chair_decisions,
                    "votes": report.safety_counters.votes,
                    "rewards": report.safety_counters.rewards,
                    "penalties": report.safety_counters.penalties,
                    "voice_changes": report.safety_counters.voice_changes,
                    "executions": report.safety_counters.executions,
                },
                "report_digest": report.report_digest,
            })
        );
    } else {
        println!("mode={:?}", report.mode);
        println!(
            "capability_registry_digest={}",
            report.capability_registry.registry_digest
        );
        for summary in summaries {
            println!(
                "agent={};intent_digest={};view_digest={};projection_digest={};session_digest={};trainer={:?};view_resolution={:?};sources={};status={:?};candidate_present={};candidate_digest={}",
                summary.agent_id,
                summary.intent_digest,
                summary.data_view_digest,
                summary.trainer_projection_digest.unwrap_or_default(),
                summary.session_digest,
                summary.trainer_kind,
                summary.view_resolution_status,
                summary.source_count,
                summary.session_status,
                summary.candidate_present,
                summary.candidate_digest.unwrap_or_default(),
            );
        }
        println!("active_state_unchanged={}", report.active_state_unchanged);
        println!(
            "network_requests={}",
            report.safety_counters.network_requests
        );
        println!(
            "prospective_artifact_mutations={}",
            report.safety_counters.prospective_artifact_mutations
        );
        println!(
            "prospective_label_reads={}",
            report.safety_counters.prospective_label_reads
        );
        println!("chair_decisions={}", report.safety_counters.chair_decisions);
        println!("votes={}", report.safety_counters.votes);
        println!("rewards={}", report.safety_counters.rewards);
        println!("executions={}", report.safety_counters.executions);
        println!("duplicate_artifacts={}", report.duplicate_artifact_count);
        println!("storage_failures={}", report.storage_failure_count);
        println!("report_digest={}", report.report_digest);
    }
    if report.storage_failure_count > 0 {
        return Err("one or more private learning artifacts failed verification".to_string());
    }
    Ok(())
}

fn run_local_historical_snapshot_campaign(
    config_path: &Path,
    temporal_diagnostics: bool,
    output_format: &str,
    cross_market_report: bool,
    btc_multi_regime_report: bool,
    btc_cross_regime_diagnostics: bool,
    btc_cycle_risk_shadow_report: bool,
    learned_agent_shadow_deliberation: bool,
    learned_agent_scope_alignment: bool,
    joint_canonical_scope_replay: bool,
    joint_momentum_failure_forensics: bool,
    joint_canonical_scope_replay_v2: bool,
    joint_momentum_closure_forensics_v3: bool,
    joint_canonical_scope_registration_v3: bool,
    joint_canonical_scope_replay_v3: bool,
    chair_shadow_observation_inbox: bool,
    chair_shadow_owner_advisory_review: bool,
    learned_reward_eligibility: bool,
    prospective_external_row_admission: bool,
    acquire_one_upbit_prospective_candle: bool,
    dry_run: bool,
    execute: bool,
    confirm_single_public_candle_request: bool,
    btc_prospective_challenge_create: bool,
    btc_prospective_challenge_status: bool,
    btc_prospective_challenge_confirm_preregistration: bool,
    btc_prospective_registry_close: bool,
    btc_prospective_accumulate: bool,
    btc_prospective_evaluate: bool,
    allow_network: bool,
) -> Result<(), String> {
    let config = crate::data::UpbitHistoricalPilotConfigV0::from_toml_path(config_path)
        .map_err(|_| "local provider config unavailable".to_string())?;
    config
        .validate()
        .map_err(|_| "local provider config is invalid".to_string())?;
    let mut snapshot_paths = fs::read_dir(&config.snapshot_output_dir)
        .map_err(|_| "local snapshot directory unavailable".to_string())?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "pb"))
        .collect::<Vec<_>>();
    snapshot_paths.sort();
    if snapshot_paths.is_empty() {
        return Err("local historical campaign requires a snapshot".to_string());
    }
    let mut snapshots = snapshot_paths
        .iter()
        .map(|path| crate::data::read_local_snapshot_protobuf_v1(path))
        .collect::<Result<Vec<_>, _>>()?;
    snapshots.sort_by(|left, right| {
        historical_snapshot_selection_rank(right)
            .cmp(&historical_snapshot_selection_rank(left))
            .then_with(|| right.row_count.cmp(&left.row_count))
            .then_with(|| left.fetched_at_ms.cmp(&right.fetched_at_ms))
            .then_with(|| left.snapshot_id.cmp(&right.snapshot_id))
    });
    let snapshot = snapshots.remove(0);
    let campaign_config = crate::model::MomentumLearningCampaignConfigV0::default();
    let inventory = crate::model::inventory_historical_snapshots_v0(
        std::slice::from_ref(&snapshot),
        &crate::model::HistoricalEvidencePolicyV0::default(),
    )
    .map_err(|_| "historical snapshot inventory failed".to_string())?;
    let sufficiency =
        crate::model::assess_momentum_campaign_sufficiency_v0(snapshot.row_count, &campaign_config)
            .map_err(|_| "momentum campaign sufficiency calculation failed".to_string())?;
    let reloaded_digest =
        crate::data::historical_replay_dataset_digest_v0(&snapshot.normalized_dataset);
    let prospective_action_count = [
        btc_prospective_challenge_create,
        btc_prospective_challenge_status,
        btc_prospective_challenge_confirm_preregistration,
        btc_prospective_registry_close,
        btc_prospective_accumulate,
        btc_prospective_evaluate,
    ]
    .into_iter()
    .filter(|selected| *selected)
    .count();
    if prospective_action_count > 1 {
        return Err("select exactly one BTC prospective challenge action".to_string());
    }
    if prospective_action_count == 1 {
        return run_btc_prospective_challenge_command(
            config_path,
            &snapshot,
            &campaign_config,
            &sufficiency,
            btc_prospective_challenge_create,
            btc_prospective_challenge_status,
            btc_prospective_challenge_confirm_preregistration,
            btc_prospective_registry_close,
            btc_prospective_accumulate,
            btc_prospective_evaluate,
            output_format,
            allow_network,
        );
    }
    if learned_reward_eligibility {
        if allow_network {
            return Err("learned reward eligibility report is offline-only".to_string());
        }
        return run_learned_reward_eligibility_report(config_path, &snapshot, output_format);
    }
    if acquire_one_upbit_prospective_candle {
        return run_one_upbit_prospective_public_export(
            config_path,
            &snapshot,
            output_format,
            dry_run,
            execute,
            allow_network,
            confirm_single_public_candle_request,
        );
    }
    if prospective_external_row_admission {
        if allow_network {
            return Err("prospective external row admission is offline-only".to_string());
        }
        return run_prospective_external_row_admission_report(
            config_path,
            &snapshot,
            output_format,
        );
    }
    if btc_multi_regime_report {
        return run_btc_multi_regime_evidence_report(
            config_path,
            &snapshot,
            &snapshots,
            &campaign_config,
            &inventory,
            &sufficiency,
            reloaded_digest == snapshot.content_digest,
            output_format,
            allow_network,
        );
    }
    if btc_cross_regime_diagnostics {
        if allow_network {
            return Err("BTC cross-regime diagnostics are offline-only".to_string());
        }
        return run_btc_cross_regime_diagnostics(
            &snapshot,
            &campaign_config,
            &sufficiency,
            output_format,
        );
    }
    if btc_cycle_risk_shadow_report {
        if allow_network {
            return Err("BTC cycle/risk shadow report is offline-only".to_string());
        }
        let report = crate::model::run_cycle_risk_shadow_v0(
            &snapshot,
            &crate::model::CycleRiskShadowConfigV0::default(),
        )
        .map_err(|_| "offline BTC cycle/risk shadow execution failed".to_string())?;
        if output_format == "json" {
            println!(
                "cycle_risk_shadow_snapshot_id={} cycle_risk_shadow_snapshot_digest={} cycle_risk_shadow_journal_digest={} cycle_risk_shadow_verdict={:?}",
                report.snapshot_id,
                report.snapshot_digest,
                report.journal.digest,
                report.aggregate_verdict
            );
        } else {
            println!("cycle_risk_shadow_agent={}", report.agent_id);
            println!("cycle_risk_shadow_snapshot_id={}", report.snapshot_id);
            println!(
                "cycle_risk_shadow_snapshot_digest={}",
                report.snapshot_digest
            );
            println!("cycle_risk_shadow_regimes={}", report.regimes.len());
            println!("cycle_risk_shadow_verdict={:?}", report.aggregate_verdict);
            println!(
                "cycle_risk_shadow_network_requests={}",
                report.network_requests
            );
            println!("cycle_risk_shadow_journal_digest={}", report.journal.digest);
            for regime in &report.regimes {
                println!(
                    "cycle_risk_shadow_regime={} verdict={:?} threshold={:.8} r0_brier={:.8} r1_brier={:.8} r2_brier={:.8} r2_auc={:?} r2_false_negatives={} r2_collapse={}",
                    regime.regime_id,
                    regime.verdict,
                    regime.checkpoint.threshold,
                    regime.checkpoint.r0.brier,
                    regime.checkpoint.r1.brier,
                    regime.checkpoint.r2.brier,
                    regime.checkpoint.r2.rank_auc,
                    regime.checkpoint.r2.high_confidence_false_negatives,
                    regime.checkpoint.r2.probability_collapse,
                );
            }
        }
        return Ok(());
    }
    if learned_agent_shadow_deliberation {
        if allow_network {
            return Err("learned-agent shadow deliberation is offline-only".to_string());
        }
        let first = crate::model::replay_btc_shadow_deliberations_v0(&snapshot, &campaign_config)
            .map_err(|_| "offline learned-agent shadow deliberation failed".to_string())?;
        let second = crate::model::replay_btc_shadow_deliberations_v0(&snapshot, &campaign_config)
            .map_err(|_| "offline learned-agent shadow deliberation replay failed".to_string())?;
        if first != second {
            return Err("learned-agent shadow deliberation is nondeterministic".to_string());
        }
        let mut ledger = crate::model::new_shadow_deliberation_ledger_v0();
        for replay in &first {
            crate::model::append_shadow_deliberation_v0(&mut ledger, replay)
                .map_err(|_| "learned-agent shadow ledger append failed".to_string())?;
        }
        if output_format == "json" {
            println!(
                "{}",
                serde_json::json!({"report_version":"learned-agent-shadow-deliberation-v0","offline":true,"provider_calls":0,"transport_constructions":0,"network_consent_reads":0,"deliberation_count":first.len(),"relationships":first.iter().map(|value| format!("{:?}", value.relationship.relationship)).collect::<Vec<_>>(),"transcript_digests":first.iter().map(|value| value.transcript.transcript_digest.clone()).collect::<Vec<_>>(),"ledger_digest":ledger.ledger_digest,"chair_observed":false,"vote_created":false,"execution_created":false})
            );
        } else {
            println!("report_version=learned-agent-shadow-deliberation-v0");
            println!("offline=true");
            println!("provider_calls=0");
            println!("transport_constructions=0");
            println!("network_consent_reads=0");
            println!("deliberation_count={}", first.len());
            for value in &first {
                println!(
                    "relationship={:?} transcript_digest={}",
                    value.relationship.relationship, value.transcript.transcript_digest
                );
            }
            println!("ledger_digest={}", ledger.ledger_digest);
            println!("chair_observed=false");
            println!("vote_created=false");
            println!("execution_created=false");
        }
        return Ok(());
    }
    if joint_momentum_failure_forensics {
        if allow_network {
            return Err("joint Momentum forensics is offline-only".to_string());
        }
        let registration =
            crate::model::joint_canonical_scope_registration_v1(&snapshot, &campaign_config)
                .map_err(|_| "joint forensic parent registration failed".to_string())?;
        let (proof, scopes) =
            crate::model::issue_joint_canonical_scopes_v1(&snapshot, &registration)
                .map_err(|_| "joint forensic scope selection failed".to_string())?;
        if !proof.all_invariants_pass {
            return Err("joint forensic scopes are unavailable".to_string());
        }
        let mut reports = Vec::new();
        for scope in &scopes {
            let first =
                crate::model::forensic_joint_momentum_scope_v2(&snapshot, scope, &campaign_config)
                    .map_err(|error| format!("joint forensic replay failed: {error}"))?;
            let second =
                crate::model::forensic_joint_momentum_scope_v2(&snapshot, scope, &campaign_config)
                    .map_err(|error| {
                        format!("joint forensic determinism replay failed: {error}")
                    })?;
            if first != second {
                return Err("joint forensic replay is nondeterministic".to_string());
            }
            reports.push(first);
        }
        if output_format == "json" {
            println!(
                "{}",
                serde_json::json!({
                    "report_version":"joint-momentum-failure-forensic-v2",
                    "offline":true,
                    "parent_registration_digest_v1":registration.registration_digest_v1,
                    "scope_ids":reports.iter().map(|report| report.joint_scope_id.clone()).collect::<Vec<_>>(),
                    "root_causes":reports.iter().map(|report| format!("{:?}", report.root_cause)).collect::<Vec<_>>(),
                    "first_failed_stages":reports.iter().map(|report| report.execution_trace.first_failed_stage.map(|stage| format!("{:?}", stage))).collect::<Vec<_>>(),
                    "sanitized_errors":reports.iter().map(|report| report.execution_trace.stages.iter().find(|stage| stage.status == crate::model::JointExecutionStageStatusV2::Failed).and_then(|stage| stage.sanitized_error_code.clone())).collect::<Vec<_>>(),
                    "accepted_series_counts":reports.iter().map(|report| report.accepted_series_count).collect::<Vec<_>>(),
                    "pack_series_counts":reports.iter().map(|report| report.pack_series_count).collect::<Vec<_>>(),
                    "trace_digests":reports.iter().map(|report| report.execution_trace.trace_digest_v2.clone()).collect::<Vec<_>>(),
                    "forensic_digests":reports.iter().map(|report| report.forensic_digest_v2.clone()).collect::<Vec<_>>(),
                    "provider_calls":0,"transport_constructions":0,"network_consent_reads":0,"credential_reads":0,
                    "chair_observed":false,"vote_created":false,"execution_created":false
                })
            );
        } else {
            println!("report_version=joint-momentum-failure-forensic-v2");
            println!("offline=true");
            println!(
                "parent_registration_digest_v1={}",
                registration.registration_digest_v1
            );
            println!("scope_count={}", reports.len());
            for report in &reports {
                println!("scope_id={}", report.joint_scope_id);
                println!("root_cause={:?}", report.root_cause);
                println!(
                    "first_failed_stage={:?}",
                    report.execution_trace.first_failed_stage
                );
                println!(
                    "sanitized_error={}",
                    report
                        .execution_trace
                        .stages
                        .iter()
                        .find(|stage| stage.status
                            == crate::model::JointExecutionStageStatusV2::Failed)
                        .and_then(|stage| stage.sanitized_error_code.as_deref())
                        .unwrap_or("")
                );
                println!("accepted_series_count={}", report.accepted_series_count);
                println!("pack_series_count={}", report.pack_series_count);
                println!("trace_digest_v2={}", report.execution_trace.trace_digest_v2);
                println!("forensic_digest_v2={}", report.forensic_digest_v2);
            }
            println!("provider_calls=0");
            println!("transport_constructions=0");
            println!("network_consent_reads=0");
            println!("credential_reads=0");
            println!("chair_observed=false");
            println!("vote_created=false");
            println!("execution_created=false");
        }
        return Ok(());
    }
    if joint_canonical_scope_replay_v2 {
        if allow_network {
            return Err("joint canonical scope replay V2 is offline-only".to_string());
        }
        let registration =
            crate::model::joint_canonical_scope_registration_v2(&snapshot, &campaign_config)
                .map_err(|error| format!("joint V2 registration failed: {error}"))?;
        let scopes = crate::model::validate_joint_canonical_scope_registration_v2(
            &snapshot,
            &campaign_config,
            &registration,
        )
        .map_err(|error| format!("joint V2 registration verification failed: {error}"))?;
        let mut results = Vec::new();
        for scope in &scopes {
            results.push(
                crate::model::replay_joint_scope_results_v2(
                    &snapshot,
                    scope,
                    &registration,
                    &campaign_config,
                )
                .map_err(|error| format!("joint V2 scope replay failed: {error}"))?,
            );
        }
        let (aggregate, ledger) =
            crate::model::aggregate_joint_scope_replays_v2(&registration, &results)
                .map_err(|error| format!("joint V2 aggregate failed: {error}"))?;
        crate::model::validate_joint_scope_replay_ledger_v2(&ledger)
            .map_err(|error| format!("joint V2 ledger verification failed: {error}"))?;
        let mut interpretations = Vec::new();
        for (scope, result) in scopes.iter().zip(&results) {
            let forensic =
                crate::model::forensic_joint_momentum_scope_v2(&snapshot, scope, &campaign_config)
                    .map_err(|error| format!("joint V2 interpretation forensic failed: {error}"))?;
            interpretations.push(
                crate::model::interpret_sprint57_momentum_outcome_v2(
                    scope,
                    &forensic,
                    &result.momentum,
                )
                .map_err(|error| format!("joint V2 interpretation failed: {error}"))?,
            );
        }
        if output_format == "json" {
            println!(
                "{}",
                serde_json::json!({
                    "report_version":"joint-canonical-scope-replay-v2","offline":true,
                    "parent_registration_digest_v1":registration.parent_registration_digest_v1,
                    "registration_digest_v2":registration.registration_digest_v2,
                    "scope_ids":results.iter().map(|result| result.joint_scope_id.clone()).collect::<Vec<_>>(),
                    "scope_digests":results.iter().map(|result| result.joint_scope_digest.clone()).collect::<Vec<_>>(),
                    "momentum_execution_health":results.iter().map(|result| format!("{:?}", result.momentum.execution_trace.execution_health)).collect::<Vec<_>>(),
                    "momentum_model_outcomes":results.iter().map(|result| format!("{:?}", result.momentum.execution_trace.model_evidence_outcome)).collect::<Vec<_>>(),
                    "momentum_operational_results":results.iter().map(|result| format!("{:?}", result.momentum.execution_trace.operational_shadow_result)).collect::<Vec<_>>(),
                    "momentum_anchor_statuses":results.iter().map(|result| format!("{:?}", result.momentum.anchor_status)).collect::<Vec<_>>(),
                    "risk_execution_health":results.iter().map(|result| format!("{:?}", result.risk.execution_trace.execution_health)).collect::<Vec<_>>(),
                    "risk_model_outcomes":results.iter().map(|result| format!("{:?}", result.risk.execution_trace.model_evidence_outcome)).collect::<Vec<_>>(),
                    "risk_operational_results":results.iter().map(|result| format!("{:?}", result.risk.execution_trace.operational_shadow_result)).collect::<Vec<_>>(),
                    "opinion_count":results.iter().flat_map(|result| [&result.momentum, &result.risk]).filter(|result| result.opinion_id.is_some()).count(),
                    "pair_count":aggregate.completed_pair_count,
                    "deliberation_count":aggregate.deliberation_count,
                    "aggregate_composed":aggregate.full_aggregate_composed,
                    "aggregate_digest_v2":aggregate.aggregate_digest_v2,
                    "ledger_digest_v2":ledger.ledger_digest_v2,
                    "forensic_root_causes":interpretations.iter().map(|value| format!("{:?}", value.forensic_root_cause)).collect::<Vec<_>>(),
                    "trace_digests":results.iter().flat_map(|result| [result.momentum.execution_trace.trace_digest_v2.clone(), result.risk.execution_trace.trace_digest_v2.clone()]).collect::<Vec<_>>(),
                    "provider_calls":0,"transport_constructions":0,"network_consent_reads":0,"credential_reads":0,
                    "chair_observed":false,"vote_created":false,"execution_created":false
                })
            );
        } else {
            println!("report_version=joint-canonical-scope-replay-v2");
            println!("offline=true");
            println!(
                "parent_registration_digest_v1={}",
                registration.parent_registration_digest_v1
            );
            println!(
                "registration_digest_v2={}",
                registration.registration_digest_v2
            );
            println!("scope_count={}", results.len());
            for result in &results {
                println!(
                    "scope_id={} scope_digest={}",
                    result.joint_scope_id, result.joint_scope_digest
                );
                println!(
                    "momentum_execution_health={:?}",
                    result.momentum.execution_trace.execution_health
                );
                println!(
                    "momentum_model_outcome={:?}",
                    result.momentum.execution_trace.model_evidence_outcome
                );
                println!(
                    "momentum_operational_result={:?}",
                    result.momentum.execution_trace.operational_shadow_result
                );
                println!("momentum_anchor_status={:?}", result.momentum.anchor_status);
                println!(
                    "risk_execution_health={:?}",
                    result.risk.execution_trace.execution_health
                );
                println!(
                    "risk_model_outcome={:?}",
                    result.risk.execution_trace.model_evidence_outcome
                );
                println!(
                    "risk_operational_result={:?}",
                    result.risk.execution_trace.operational_shadow_result
                );
                println!(
                    "momentum_trace_digest={}",
                    result.momentum.execution_trace.trace_digest_v2
                );
                println!(
                    "risk_trace_digest={}",
                    result.risk.execution_trace.trace_digest_v2
                );
            }
            println!(
                "opinion_count={}",
                results
                    .iter()
                    .flat_map(|result| [&result.momentum, &result.risk])
                    .filter(|result| result.opinion_id.is_some())
                    .count()
            );
            println!("pair_count={}", aggregate.completed_pair_count);
            println!("deliberation_count={}", aggregate.deliberation_count);
            println!("aggregate_composed={}", aggregate.full_aggregate_composed);
            println!("aggregate_digest_v2={}", aggregate.aggregate_digest_v2);
            println!("ledger_digest_v2={}", ledger.ledger_digest_v2);
            println!("provider_calls=0");
            println!("transport_constructions=0");
            println!("network_consent_reads=0");
            println!("credential_reads=0");
            println!("chair_observed=false");
            println!("vote_created=false");
            println!("execution_created=false");
        }
        return Ok(());
    }
    if joint_momentum_closure_forensics_v3 {
        if allow_network {
            return Err("joint Momentum closure forensics is offline-only".to_string());
        }
        let registration =
            crate::model::joint_canonical_scope_registration_v2(&snapshot, &campaign_config)
                .map_err(|error| format!("joint V3 closure registration failed: {error}"))?;
        let scopes = crate::model::validate_joint_canonical_scope_registration_v2(
            &snapshot,
            &campaign_config,
            &registration,
        )
        .map_err(|error| format!("joint V3 closure registration verification failed: {error}"))?;
        let mut audits = Vec::new();
        for scope in &scopes {
            let first = crate::model::audit_joint_scope_momentum_closure_v3(
                &snapshot,
                scope,
                &registration,
                &campaign_config,
            )
            .map_err(|error| format!("joint V3 closure audit failed: {error}"))?;
            let second = crate::model::audit_joint_scope_momentum_closure_v3(
                &snapshot,
                scope,
                &registration,
                &campaign_config,
            )
            .map_err(|error| format!("joint V3 closure audit replay failed: {error}"))?;
            if first != second {
                return Err("joint V3 closure audit is nondeterministic".to_string());
            }
            audits.push(first);
        }
        if output_format == "json" {
            println!(
                "{}",
                serde_json::json!({
                    "report_version":"joint-momentum-closure-forensic-v3",
                    "offline":true,
                    "parent_registration_digest_v1":registration.parent_registration_digest_v1,
                    "registration_digest_v2":registration.registration_digest_v2,
                    "scope_ids":audits.iter().map(|audit| audit.joint_scope_id.clone()).collect::<Vec<_>>(),
                    "audit_digests":audits.iter().map(|audit| audit.audit_digest_v3.clone()).collect::<Vec<_>>(),
                    "preclosure_digests":audits.iter().map(|audit| audit.preclosure.preclosure_digest_v3.clone()).collect::<Vec<_>>(),
                    "open_result_digests":audits.iter().map(|audit| audit.open_result_digest.clone()).collect::<Vec<_>>(),
                    "closed_result_digests":audits.iter().map(|audit| audit.closed_result_digest.clone()).collect::<Vec<_>>(),
                    "preclosure_campaign_window_counts":audits.iter().map(|audit| audit.preclosure.campaign_window_count).collect::<Vec<_>>(),
                    "preclosure_final_verdicts":audits.iter().map(|audit| audit.preclosure.final_verdict.clone()).collect::<Vec<_>>(),
                    "preclosure_no_signal_window_counts":audits.iter().map(|audit| audit.preclosure.no_signal_window_count).collect::<Vec<_>>(),
                    "preclosure_selected_checkpoint_counts":audits.iter().map(|audit| audit.preclosure.selected_checkpoint_count).collect::<Vec<_>>(),
                    "preclosure_support_counts":audits.iter().map(|audit| audit.preclosure.support_counts.clone()).collect::<Vec<_>>(),
                    "first_failed_invariants":audits.iter().map(|audit| audit.first_failed_invariant.map(|value| format!("{value:?}"))).collect::<Vec<_>>(),
                    "failure_classes":audits.iter().map(|audit| format!("{:?}", audit.failure_class)).collect::<Vec<_>>(),
                    "validator_errors":audits.iter().map(|audit| audit.validator_error.clone()).collect::<Vec<_>>(),
                    "all_invariants_pass":audits.iter().map(|audit| audit.all_invariants_pass).collect::<Vec<_>>(),
                    "provider_calls":0,"transport_constructions":0,"network_consent_reads":0,"credential_reads":0,
                    "chair_observed":false,"vote_created":false,"execution_created":false
                })
            );
        } else {
            println!("report_version=joint-momentum-closure-forensic-v3");
            println!("offline=true");
            println!(
                "parent_registration_digest_v1={}",
                registration.parent_registration_digest_v1
            );
            println!(
                "registration_digest_v2={}",
                registration.registration_digest_v2
            );
            println!("scope_count={}", audits.len());
            for audit in &audits {
                println!("scope_id={}", audit.joint_scope_id);
                println!("open_result_digest={}", audit.open_result_digest);
                println!("closed_result_digest={}", audit.closed_result_digest);
                println!("regime_reference_digest={}", audit.regime_reference_digest);
                println!(
                    "preclosure_digest_v3={}",
                    audit.preclosure.preclosure_digest_v3
                );
                println!(
                    "preclosure_campaign_window_count={}",
                    audit.preclosure.campaign_window_count
                );
                println!(
                    "preclosure_final_verdict={}",
                    audit.preclosure.final_verdict
                );
                println!(
                    "preclosure_no_signal_window_count={}",
                    audit.preclosure.no_signal_window_count
                );
                println!(
                    "preclosure_selected_checkpoint_count={}",
                    audit.preclosure.selected_checkpoint_count
                );
                println!(
                    "preclosure_support_counts={}",
                    audit
                        .preclosure
                        .support_counts
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(":")
                );
                println!("first_failed_invariant={:?}", audit.first_failed_invariant);
                println!("failure_class={:?}", audit.failure_class);
                println!(
                    "validator_error={}",
                    audit.validator_error.as_deref().unwrap_or("")
                );
                if let Some(failed) = audit.invariant_results.iter().find(|value| !value.passed) {
                    println!("expected_semantic_value={}", failed.expected_semantic_value);
                    println!("actual_semantic_value={}", failed.actual_semantic_value);
                    println!("reason_code={}", failed.reason_code);
                }
                println!("all_invariants_pass={}", audit.all_invariants_pass);
                println!("audit_digest_v3={}", audit.audit_digest_v3);
            }
            println!("provider_calls=0");
            println!("transport_constructions=0");
            println!("network_consent_reads=0");
            println!("credential_reads=0");
            println!("chair_observed=false");
            println!("vote_created=false");
            println!("execution_created=false");
        }
        return Ok(());
    }
    if joint_canonical_scope_registration_v3
        || joint_canonical_scope_replay_v3
        || chair_shadow_observation_inbox
        || chair_shadow_owner_advisory_review
    {
        if allow_network {
            return Err("joint canonical scope replay V3 is offline-only".to_string());
        }
        let parent_registration =
            crate::model::joint_canonical_scope_registration_v2(&snapshot, &campaign_config)
                .map_err(|error| format!("joint V3 parent registration failed: {error}"))?;
        let scopes = crate::model::validate_joint_canonical_scope_registration_v2(
            &snapshot,
            &campaign_config,
            &parent_registration,
        )
        .map_err(|error| format!("joint V3 parent registration verification failed: {error}"))?;
        let mut audits = Vec::new();
        for scope in &scopes {
            let first = crate::model::audit_joint_scope_momentum_closure_v3(
                &snapshot,
                scope,
                &parent_registration,
                &campaign_config,
            )
            .map_err(|error| format!("joint V3 registration audit failed: {error}"))?;
            let second = crate::model::audit_joint_scope_momentum_closure_v3(
                &snapshot,
                scope,
                &parent_registration,
                &campaign_config,
            )
            .map_err(|error| format!("joint V3 registration audit replay failed: {error}"))?;
            if first != second {
                return Err("joint V3 registration audit is nondeterministic".to_string());
            }
            audits.push(first);
        }
        let registration = crate::model::joint_canonical_scope_registration_v3(
            &snapshot,
            &campaign_config,
            &audits,
        )
        .map_err(|error| format!("joint V3 registration failed: {error}"))?;
        crate::model::validate_joint_canonical_scope_registration_v3(
            &snapshot,
            &campaign_config,
            &registration,
        )
        .map_err(|error| format!("joint V3 registration verification failed: {error}"))?;
        if joint_canonical_scope_registration_v3 {
            if output_format == "json" {
                println!(
                    "{}",
                    serde_json::json!({
                        "report_version":"joint-canonical-scope-registration-v3",
                        "offline":true,
                        "parent_registration_digest_v2":registration.parent_registration_digest_v2,
                        "registration_digest_v3":registration.registration_digest_v3,
                        "scope_ids":registration.joint_scope_ids,
                        "scope_digests":registration.joint_scope_digests,
                        "preclosure_result_digests":registration.preclosure_result_digests,
                        "correction_failure_class":format!("{:?}", registration.correction_failure_class),
                        "scope_ranges_unchanged":registration.scope_ranges_unchanged,
                        "participant_configs_unchanged":registration.participant_configs_unchanged,
                        "preclosure_results_unchanged":registration.preclosure_results_unchanged,
                        "scope0_non_regression_required":registration.scope0_non_regression_required,
                        "result_dependent_model_changes_forbidden":registration.result_dependent_model_changes_forbidden,
                        "provider_calls":0,"transport_constructions":0,"network_consent_reads":0,"credential_reads":0,
                        "active_committee_count":3,"chair_observed":false,"chair_decision_created":false,
                        "reward_created":false,"penalty_created":false,"speaking_right_changed":false,
                        "vote_created":false,"promotion_created":false,"execution_created":false
                    })
                );
            } else {
                println!("report_version=joint-canonical-scope-registration-v3");
                println!("offline=true");
                println!(
                    "parent_registration_digest_v2={}",
                    registration.parent_registration_digest_v2
                );
                println!(
                    "registration_digest_v3={}",
                    registration.registration_digest_v3
                );
                println!("scope_ids={}", registration.joint_scope_ids.join(":"));
                println!(
                    "scope_digests={}",
                    registration.joint_scope_digests.join(":")
                );
                println!(
                    "preclosure_result_digests={}",
                    registration.preclosure_result_digests.join(":")
                );
                println!(
                    "correction_failure_class={:?}",
                    registration.correction_failure_class
                );
                println!(
                    "scope_ranges_unchanged={}",
                    registration.scope_ranges_unchanged
                );
                println!(
                    "participant_configs_unchanged={}",
                    registration.participant_configs_unchanged
                );
                println!(
                    "preclosure_results_unchanged={}",
                    registration.preclosure_results_unchanged
                );
                println!(
                    "scope0_non_regression_required={}",
                    registration.scope0_non_regression_required
                );
                println!(
                    "result_dependent_model_changes_forbidden={}",
                    registration.result_dependent_model_changes_forbidden
                );
                println!("provider_calls=0");
                println!("transport_constructions=0");
                println!("network_consent_reads=0");
                println!("credential_reads=0");
                println!("active_committee_count=3");
                println!("chair_observed=false");
                println!("chair_decision_created=false");
                println!("reward_created=false");
                println!("penalty_created=false");
                println!("speaking_right_changed=false");
                println!("vote_created=false");
                println!("promotion_created=false");
                println!("execution_created=false");
            }
            return Ok(());
        }
        let mut results = Vec::new();
        for scope in &scopes {
            results.push(
                crate::model::replay_joint_scope_results_v3(
                    &snapshot,
                    scope,
                    &registration,
                    &campaign_config,
                )
                .map_err(|error| format!("joint V3 scope replay failed: {error}"))?,
            );
        }
        let mut replay_repeat = Vec::new();
        for scope in &scopes {
            replay_repeat.push(
                crate::model::replay_joint_scope_results_v3(
                    &snapshot,
                    scope,
                    &registration,
                    &campaign_config,
                )
                .map_err(|error| format!("joint V3 scope replay repeat failed: {error}"))?,
            );
        }
        if results != replay_repeat {
            return Err("joint V3 scope replay is nondeterministic".to_string());
        }
        let (aggregate, ledger) =
            crate::model::aggregate_joint_scope_replays_v3(&registration, &results)
                .map_err(|error| format!("joint V3 aggregate failed: {error}"))?;
        crate::model::validate_joint_scope_replay_ledger_v3(&ledger)
            .map_err(|error| format!("joint V3 ledger verification failed: {error}"))?;
        if chair_shadow_observation_inbox || chair_shadow_owner_advisory_review {
            let evidence = crate::model::chair_shadow_observation_evidence_v0(
                registration.clone(),
                results.clone(),
                aggregate.clone(),
                ledger.clone(),
            )
            .map_err(|error| format!("chair shadow observation evidence failed: {error}"))?;
            let first = crate::model::observe_chair_shadow_observation_v0(&evidence)
                .map_err(|error| format!("chair shadow observation failed: {error}"))?;
            let second = crate::model::observe_chair_shadow_observation_v0(&evidence)
                .map_err(|error| format!("chair shadow observation replay failed: {error}"))?;
            if first != second {
                return Err("chair shadow observation is nondeterministic".to_string());
            }
            if chair_shadow_owner_advisory_review {
                let firewall = crate::owner::owner_advisory_decision_firewall_proof_v0();
                if !firewall.all_invariants_pass {
                    return Err("owner advisory decision firewall failed".to_string());
                }
                let ledger_path =
                    Path::new("target/chair-shadow-owner-advisory-review-ledger-v0.json");
                let mut reviews = Vec::new();
                let mut ledger = None;
                for owner_input in crate::owner::chair_shadow_owner_advisory_fixture_inputs_v0() {
                    let review_input = crate::owner::chair_shadow_owner_advisory_review_input_v0(
                        &first,
                        owner_input,
                    );
                    let review =
                        crate::owner::review_chair_shadow_owner_advisory_v0(&first, &review_input);
                    let replay =
                        crate::owner::review_chair_shadow_owner_advisory_v0(&first, &review_input);
                    if review != replay {
                        return Err("owner advisory review is nondeterministic".to_string());
                    }
                    ledger = Some(
                        crate::owner::append_chair_shadow_owner_review_ledger_v0(
                            ledger_path,
                            &review,
                        )
                        .map_err(|error| {
                            format!("owner advisory review storage failed: {error}")
                        })?,
                    );
                    reviews.push(review);
                }
                let ledger =
                    ledger.ok_or_else(|| "owner advisory fixtures unavailable".to_string())?;
                let reopened = crate::owner::read_chair_shadow_owner_review_ledger_v0(ledger_path)
                    .map_err(|error| {
                        format!("owner advisory review storage reopen failed: {error}")
                    })?;
                if reopened != ledger {
                    return Err("owner advisory review storage reopen mismatch".to_string());
                }
                if output_format == "json" {
                    let review_output = reviews
                        .iter()
                        .map(|review| {
                            serde_json::json!({
                                "owner_input_fingerprint":review.owner_input_fingerprint,
                                "owner_policy_allowed":review.owner_policy_allowed,
                                "owner_policy_diagnostic_only":review.owner_policy_diagnostic_only,
                                "status":format!("{:?}", review.status),
                                "reason_codes":review.reason_codes,
                                "explanation":review.explanation,
                                "review_digest":review.review_digest,
                                "changed_observation":review.changed_observation,
                                "changed_model":review.changed_model,
                                "changed_decision":review.changed_decision,
                                "changed_risk_policy":review.changed_risk_policy,
                                "vote_created":review.vote_created,
                                "reward_created":review.reward_created,
                                "penalty_created":review.penalty_created,
                                "speaking_right_changed":review.speaking_right_changed,
                                "risk_handoff_created":review.risk_handoff_created,
                                "paper_action_created":review.paper_action_created,
                                "execution_created":review.execution_created
                            })
                        })
                        .collect::<Vec<_>>();
                    println!(
                        "{}",
                        serde_json::json!({
                            "report_version":"chair-shadow-owner-advisory-review-v0",
                            "offline":true,
                            "fixture_inputs_only":true,
                            "reviews":review_output,
                            "ledger_digest":ledger.ledger_digest,
                            "firewall_digest":firewall.proof_digest,
                            "ledger_reopen_verified":true,
                            "provider_calls":0,"transport_constructions":0,
                            "network_consent_reads":0,"credential_reads":0,
                            "active_committee_count":first.active_committee_count,
                            "chair_engine_invocations":0,"owner_trade_review_invocations":0,
                            "risk_governor_invocations":0,"paper_broker_invocations":0,
                            "votes_created":0,"rewards_created":0,"penalties_created":0,
                            "speaking_right_changes":0,"paper_actions_created":0,"executions_created":0
                        })
                    );
                } else {
                    println!("report_version=chair-shadow-owner-advisory-review-v0");
                    println!("offline=true");
                    println!("fixture_inputs_only=true");
                    for review in &reviews {
                        println!("owner_input_fingerprint={}", review.owner_input_fingerprint);
                        println!("owner_policy_allowed={}", review.owner_policy_allowed);
                        println!(
                            "owner_policy_diagnostic_only={}",
                            review.owner_policy_diagnostic_only
                        );
                        println!("status={:?}", review.status);
                        println!("reason_codes={}", review.reason_codes.join(":"));
                        println!("explanation={}", review.explanation);
                        println!("review_digest={}", review.review_digest);
                        println!("changed_observation={}", review.changed_observation);
                        println!("changed_model={}", review.changed_model);
                        println!("changed_decision={}", review.changed_decision);
                        println!("changed_risk_policy={}", review.changed_risk_policy);
                        println!("vote_created={}", review.vote_created);
                        println!("reward_created={}", review.reward_created);
                        println!("penalty_created={}", review.penalty_created);
                        println!("speaking_right_changed={}", review.speaking_right_changed);
                        println!("risk_handoff_created={}", review.risk_handoff_created);
                        println!("paper_action_created={}", review.paper_action_created);
                        println!("execution_created={}", review.execution_created);
                    }
                    println!("ledger_digest={}", ledger.ledger_digest);
                    println!("firewall_digest={}", firewall.proof_digest);
                    println!("ledger_reopen_verified=true");
                    println!("provider_calls=0");
                    println!("transport_constructions=0");
                    println!("network_consent_reads=0");
                    println!("credential_reads=0");
                    println!("active_committee_count={}", first.active_committee_count);
                    println!("chair_engine_invocations=0");
                    println!("owner_trade_review_invocations=0");
                    println!("risk_governor_invocations=0");
                    println!("paper_broker_invocations=0");
                    println!("votes_created=0");
                    println!("rewards_created=0");
                    println!("penalties_created=0");
                    println!("speaking_right_changes=0");
                    println!("paper_actions_created=0");
                    println!("executions_created=0");
                }
                return Ok(());
            }
            let storage = crate::model::append_chair_shadow_observation_storage_v0(
                Path::new("target/chair-shadow-observation-inbox-v0.json"),
                &first,
            )
            .map_err(|error| format!("chair shadow observation storage failed: {error}"))?;
            if output_format == "json" {
                println!(
                    "{}",
                    serde_json::json!({
                        "report_version":"chair-shadow-observation-inbox-v0",
                        "offline":true,
                        "observation":first,
                        "storage_digest":storage.storage_digest,
                        "storage_reopen_verified":true,
                        "provider_calls":0,"transport_constructions":0,
                        "network_consent_reads":0,"credential_reads":0,
                        "active_committee_count":3
                    })
                );
            } else {
                println!("report_version=chair-shadow-observation-inbox-v0");
                println!("offline=true");
                println!("packet_status={:?}", first.receipt.status);
                println!("evidence_class={:?}", first.packet.evidence_class);
                println!("scope_count={}", first.receipt.observed_scope_count);
                println!("opinion_count={}", first.receipt.observed_opinion_count);
                println!(
                    "abstention_count={}",
                    first.receipt.observed_abstention_count
                );
                println!(
                    "relationship_summary={}",
                    first
                        .receipt
                        .relationship_summary
                        .iter()
                        .map(|item| format!("{:?}:{}", item.category, item.count))
                        .collect::<Vec<_>>()
                        .join(":")
                );
                println!(
                    "uncertainty_flags={}",
                    first
                        .receipt
                        .uncertainty_flags
                        .iter()
                        .map(|item| format!("{:?}:{}", item.category, item.present))
                        .collect::<Vec<_>>()
                        .join(":")
                );
                println!("inbox_digest={}", first.inbox.inbox_digest);
                println!("receipt_digest={}", first.receipt.receipt_digest);
                println!("firewall_digest={}", first.firewall_proof.proof_digest);
                println!("storage_digest={}", storage.storage_digest);
                println!("storage_reopen_verified=true");
                println!("chair_runtime_invocations=0");
                println!("chair_decisions_created=0");
                println!("votes_created=0");
                println!("rewards_created=0");
                println!("penalties_created=0");
                println!("speaking_right_changes=0");
                println!("risk_handoffs=0");
                println!("executions_created=0");
                println!("provider_calls=0");
                println!("transport_constructions=0");
                println!("network_consent_reads=0");
                println!("credential_reads=0");
                println!("active_committee_count=3");
            }
            return Ok(());
        }
        if output_format == "json" {
            let scope_values = serde_json::json!({
                "ids":results.iter().map(|result| result.joint_scope_id.clone()).collect::<Vec<_>>(),
                "digests":results.iter().map(|result| result.joint_scope_digest.clone()).collect::<Vec<_>>(),
                "preclosure_digests_v3":results.iter().map(|result| result.preclosure_digest_v3.clone()).collect::<Vec<_>>(),
                "closure_audit_digests_v3":results.iter().map(|result| result.closure_audit_digest_v3.clone()).collect::<Vec<_>>()
            });
            let momentum_values = serde_json::json!({
                "execution_health":results.iter().map(|result| format!("{:?}", result.replay_result_v2.momentum.execution_trace.execution_health)).collect::<Vec<_>>(),
                "model_outcomes":results.iter().map(|result| format!("{:?}", result.replay_result_v2.momentum.execution_trace.model_evidence_outcome)).collect::<Vec<_>>(),
                "operational_results":results.iter().map(|result| format!("{:?}", result.replay_result_v2.momentum.execution_trace.operational_shadow_result)).collect::<Vec<_>>(),
                "anchor_statuses":results.iter().map(|result| format!("{:?}", result.replay_result_v2.momentum.anchor_status)).collect::<Vec<_>>(),
                "opinion_digests":results.iter().map(|result| result.replay_result_v2.momentum.sealed_opinion.as_ref().map(|pair| pair.0.opinion_digest_v1.clone())).collect::<Vec<_>>(),
                "seal_digests":results.iter().map(|result| result.replay_result_v2.momentum.seal_digest.clone()).collect::<Vec<_>>()
            });
            let risk_values = serde_json::json!({
                "execution_health":results.iter().map(|result| format!("{:?}", result.replay_result_v2.risk.execution_trace.execution_health)).collect::<Vec<_>>(),
                "model_outcomes":results.iter().map(|result| format!("{:?}", result.replay_result_v2.risk.execution_trace.model_evidence_outcome)).collect::<Vec<_>>(),
                "operational_results":results.iter().map(|result| format!("{:?}", result.replay_result_v2.risk.execution_trace.operational_shadow_result)).collect::<Vec<_>>(),
                "opinion_digests":results.iter().map(|result| result.replay_result_v2.risk.sealed_opinion.as_ref().map(|pair| pair.0.opinion_digest_v1.clone())).collect::<Vec<_>>(),
                "seal_digests":results.iter().map(|result| result.replay_result_v2.risk.seal_digest.clone()).collect::<Vec<_>>()
            });
            let aggregate_values = serde_json::json!({
                "pair_eligible":results.iter().map(|result| result.replay_result_v2.pair_eligible).collect::<Vec<_>>(),
                "opinion_count":results.iter().flat_map(|result| [&result.replay_result_v2.momentum, &result.replay_result_v2.risk]).filter(|result| result.opinion_id.is_some()).count(),
                "pair_count":aggregate.replay_aggregate_v2.completed_pair_count,
                "deliberation_count":aggregate.replay_aggregate_v2.deliberation_count,
                "composed":aggregate.replay_aggregate_v2.full_aggregate_composed,
                "relationships":aggregate.replay_aggregate_v2.relationships.iter().map(|value| format!("{value:?}")).collect::<Vec<_>>(),
                "transcript_digests":aggregate.replay_aggregate_v2.transcript_digests,
                "aggregate_digest_v3":aggregate.aggregate_digest_v3,
                "ledger_digest_v3":ledger.ledger_digest_v3
            });
            println!(
                "{}",
                serde_json::json!({
                    "report_version":"joint-canonical-scope-replay-v3",
                    "offline":true,
                    "parent_registration_digest_v2":registration.parent_registration_digest_v2,
                    "registration_digest_v3":registration.registration_digest_v3,
                    "scope":scope_values,
                    "momentum":momentum_values,
                    "risk":risk_values,
                    "aggregate":aggregate_values,
                    "replay_deterministic":true,
                    "authority":{
                        "provider_calls":0,"transport_constructions":0,"network_consent_reads":0,"credential_reads":0,
                        "active_committee_count":3,"chair_observed":false,"chair_decision_created":false,
                        "reward_created":false,"penalty_created":false,"speaking_right_changed":false,
                        "vote_created":false,"promotion_created":false,"execution_created":false
                    }
                })
            );
        } else {
            println!("report_version=joint-canonical-scope-replay-v3");
            println!("offline=true");
            println!(
                "parent_registration_digest_v2={}",
                registration.parent_registration_digest_v2
            );
            println!(
                "registration_digest_v3={}",
                registration.registration_digest_v3
            );
            println!("scope_count={}", results.len());
            for result in &results {
                println!(
                    "scope_id={} scope_digest={}",
                    result.joint_scope_id, result.joint_scope_digest
                );
                println!("preclosure_digest_v3={}", result.preclosure_digest_v3);
                println!("closure_audit_digest_v3={}", result.closure_audit_digest_v3);
                println!(
                    "momentum_execution_health={:?}",
                    result
                        .replay_result_v2
                        .momentum
                        .execution_trace
                        .execution_health
                );
                println!(
                    "momentum_model_outcome={:?}",
                    result
                        .replay_result_v2
                        .momentum
                        .execution_trace
                        .model_evidence_outcome
                );
                println!(
                    "momentum_operational_result={:?}",
                    result
                        .replay_result_v2
                        .momentum
                        .execution_trace
                        .operational_shadow_result
                );
                println!(
                    "momentum_anchor_status={:?}",
                    result.replay_result_v2.momentum.anchor_status
                );
                println!(
                    "momentum_opinion_digest={}",
                    result
                        .replay_result_v2
                        .momentum
                        .sealed_opinion
                        .as_ref()
                        .map(|pair| pair.0.opinion_digest_v1.as_str())
                        .unwrap_or("")
                );
                println!(
                    "momentum_seal_digest={}",
                    result
                        .replay_result_v2
                        .momentum
                        .seal_digest
                        .as_deref()
                        .unwrap_or("")
                );
                println!(
                    "risk_execution_health={:?}",
                    result
                        .replay_result_v2
                        .risk
                        .execution_trace
                        .execution_health
                );
                println!(
                    "risk_model_outcome={:?}",
                    result
                        .replay_result_v2
                        .risk
                        .execution_trace
                        .model_evidence_outcome
                );
                println!(
                    "risk_operational_result={:?}",
                    result
                        .replay_result_v2
                        .risk
                        .execution_trace
                        .operational_shadow_result
                );
                println!(
                    "risk_opinion_digest={}",
                    result
                        .replay_result_v2
                        .risk
                        .sealed_opinion
                        .as_ref()
                        .map(|pair| pair.0.opinion_digest_v1.as_str())
                        .unwrap_or("")
                );
                println!(
                    "risk_seal_digest={}",
                    result
                        .replay_result_v2
                        .risk
                        .seal_digest
                        .as_deref()
                        .unwrap_or("")
                );
                println!("pair_eligible={}", result.replay_result_v2.pair_eligible);
                println!(
                    "momentum_trace_digest={}",
                    result
                        .replay_result_v2
                        .momentum
                        .execution_trace
                        .trace_digest_v2
                );
                println!(
                    "risk_trace_digest={}",
                    result.replay_result_v2.risk.execution_trace.trace_digest_v2
                );
            }
            println!(
                "opinion_count={}",
                results
                    .iter()
                    .flat_map(|result| [
                        &result.replay_result_v2.momentum,
                        &result.replay_result_v2.risk
                    ])
                    .filter(|result| result.opinion_id.is_some())
                    .count()
            );
            println!(
                "pair_count={}",
                aggregate.replay_aggregate_v2.completed_pair_count
            );
            println!(
                "deliberation_count={}",
                aggregate.replay_aggregate_v2.deliberation_count
            );
            println!(
                "aggregate_composed={}",
                aggregate.replay_aggregate_v2.full_aggregate_composed
            );
            println!(
                "relationships={}",
                aggregate
                    .replay_aggregate_v2
                    .relationships
                    .iter()
                    .map(|value| format!("{value:?}"))
                    .collect::<Vec<_>>()
                    .join(":")
            );
            println!(
                "transcript_digests={}",
                aggregate.replay_aggregate_v2.transcript_digests.join(":")
            );
            println!("aggregate_digest_v3={}", aggregate.aggregate_digest_v3);
            println!("ledger_digest_v3={}", ledger.ledger_digest_v3);
            println!("replay_deterministic=true");
            println!("provider_calls=0");
            println!("transport_constructions=0");
            println!("network_consent_reads=0");
            println!("credential_reads=0");
            println!("active_committee_count=3");
            println!("chair_observed=false");
            println!("chair_decision_created=false");
            println!("reward_created=false");
            println!("penalty_created=false");
            println!("speaking_right_changed=false");
            println!("vote_created=false");
            println!("promotion_created=false");
            println!("execution_created=false");
        }
        return Ok(());
    }
    if joint_canonical_scope_replay {
        if allow_network {
            return Err("joint canonical scope replay is offline-only".to_string());
        }
        let registration =
            crate::model::joint_canonical_scope_registration_v1(&snapshot, &campaign_config)
                .map_err(|_| "joint canonical scope registration failed".to_string())?;
        let (proof, scopes) =
            crate::model::issue_joint_canonical_scopes_v1(&snapshot, &registration)
                .map_err(|_| "joint canonical scope selection failed".to_string())?;
        let results = if proof.all_invariants_pass {
            scopes
                .iter()
                .map(|scope| {
                    crate::model::replay_joint_scope_results_v1(&snapshot, scope, &campaign_config)
                        .map_err(|error| format!("joint canonical scope replay failed: {error}"))
                })
                .collect::<Result<Vec<_>, _>>()?
        } else {
            vec![]
        };
        if output_format == "json" {
            println!(
                "{}",
                serde_json::json!({
                    "report_version":"joint-canonical-scope-replay-v1", "offline":true,
                    "registration_digest_v1":registration.registration_digest_v1,
                    "joint_minimum_scope_rows":registration.joint_minimum_scope_rows,
                    "feasible":proof.all_invariants_pass,
                    "joint_scope_count":scopes.len(),
                    "joint_scope_ids":scopes.iter().map(|scope|scope.joint_scope_id.clone()).collect::<Vec<_>>(),
                    "joint_scope_digests":scopes.iter().map(|scope|scope.scope_digest_v1.clone()).collect::<Vec<_>>(),
                    "participant_result_count":results.len() * 2,
                    "momentum_statuses":results.iter().map(|result|format!("{:?}", result.momentum_status)).collect::<Vec<_>>(),
                    "risk_statuses":results.iter().map(|result|format!("{:?}", result.risk_status)).collect::<Vec<_>>(),
                    "provider_calls":0,"transport_constructions":0,"network_consent_reads":0,
                    "chair_observed":false,"vote_created":false,"execution_created":false
                })
            );
        } else {
            println!("report_version=joint-canonical-scope-replay-v1");
            println!("offline=true");
            println!(
                "registration_digest_v1={}",
                registration.registration_digest_v1
            );
            println!(
                "joint_minimum_scope_rows={}",
                registration.joint_minimum_scope_rows
            );
            println!("feasible={}", proof.all_invariants_pass);
            println!("joint_scope_count={}", scopes.len());
            println!("participant_result_count={}", results.len() * 2);
            println!(
                "momentum_completed_count={}",
                results
                    .iter()
                    .filter(|result| matches!(
                        result.momentum_status,
                        crate::model::JointScopeParticipantReplayStatusV1::Completed
                    ))
                    .count()
            );
            println!(
                "risk_completed_count={}",
                results
                    .iter()
                    .filter(|result| matches!(
                        result.risk_status,
                        crate::model::JointScopeParticipantReplayStatusV1::Completed
                    ))
                    .count()
            );
            println!("provider_calls=0");
            println!("transport_constructions=0");
            println!("network_consent_reads=0");
            println!("chair_observed=false");
            println!("vote_created=false");
            println!("execution_created=false");
        }
        return Ok(());
    }
    if learned_agent_scope_alignment {
        if allow_network {
            return Err("learned-agent scope alignment is offline-only".to_string());
        }
        let first = crate::model::replay_btc_scope_alignment_v1(&snapshot, &campaign_config)
            .map_err(|_| "offline learned-agent scope alignment failed".to_string())?;
        let second = crate::model::replay_btc_scope_alignment_v1(&snapshot, &campaign_config)
            .map_err(|_| "offline learned-agent scope alignment replay failed".to_string())?;
        if first != second {
            return Err("learned-agent scope alignment is nondeterministic".to_string());
        }
        let registration = crate::model::SourceBoundOpinionProtocolRegistrationV1::pre_registered();
        registration
            .validate()
            .map_err(|_| "source-bound registration invalid".to_string())?;
        let momentum =
            crate::model::replay_source_bound_momentum_opinions_v1(&snapshot, &campaign_config)
                .map_err(|_| "source-bound momentum replay failed".to_string())?;
        let risk = crate::model::replay_source_bound_cycle_risk_opinions_v1(&snapshot)
            .map_err(|_| "source-bound risk replay failed".to_string())?;
        let mapping = crate::model::map_source_bound_opinions_v1(&momentum, &risk)
            .map_err(|_| "source-bound mapping failed".to_string())?;
        let mut ledger = crate::model::new_source_bound_shadow_ledger_v1(
            &registration,
            first.legacy.registry.registry_digest.clone(),
        )
        .map_err(|_| "source-bound ledger initialization failed".to_string())?;
        for (opinion, seal) in momentum.iter().chain(risk.iter()) {
            crate::model::append_source_bound_opinion_v1(
                &mut ledger,
                opinion.clone(),
                seal.clone(),
            )
            .map_err(|_| "source-bound ledger append failed".to_string())?;
        }
        if output_format == "json" {
            println!(
                "{}",
                serde_json::json!({
                    "report_version": first.report_version,
                    "offline": true,
                    "provider_calls": 0,
                    "transport_constructions": 0,
                    "network_consent_reads": 0,
                    "legacy_report_version": first.legacy.report_version,
                    "legacy_mapping_status": format!("{:?}", first.legacy.registry.mapping_status),
                    "legacy_registry_digest": first.legacy.registry.registry_digest,
                    "risk_range_plan_digest": first.range_plan.plan_digest,
                    "risk_range_count": first.range_plan.ranges.len(),
                    "risk_provenance_status": format!("{:?}", first.provenance.status),
                    "risk_result_identity_count": first.provenance.result_identities.len(),
                    "risk_witness_count": first.provenance.witnesses.len(),
                    "risk_unmatched_opinion_count": first.provenance.unmatched_opinion_ids.len(),
                    "risk_multiple_match_count": first.provenance.multiply_matched_opinion_ids.len(),
                    "risk_provenance_registry_digest_v1": first.provenance.registry_digest_v1,
                    "risk_anchor_scope_count": first.risk_anchor_scopes.len(),
                    "risk_anchor_counts": first.risk_anchor_scopes.iter().map(|value| value.effective_anchor_count).collect::<Vec<_>>(),
                    "report_digest_v1": first.report_digest_v1,
                    "source_bound_registration_digest_v1": registration.policy_digest_v1,
                    "source_bound_momentum_opinion_count": momentum.len(),
                    "source_bound_risk_opinion_count": risk.len(),
                    "source_bound_mapping_status": format!("{:?}", mapping.mapping_status),
                    "source_bound_mapping_digest_v1": mapping.registry_digest_v1,
                    "source_bound_ledger_digest_v1": ledger.ledger_digest_v1,
                    "chair_observed": false,
                    "vote_created": false,
                    "execution_created": false,
                })
            );
        } else {
            println!("report_version={}", first.report_version);
            println!("offline=true");
            println!("provider_calls=0");
            println!("transport_constructions=0");
            println!("network_consent_reads=0");
            println!("legacy_report_version={}", first.legacy.report_version);
            println!(
                "legacy_mapping_status={:?}",
                first.legacy.registry.mapping_status
            );
            println!(
                "legacy_registry_digest={}",
                first.legacy.registry.registry_digest
            );
            println!("risk_range_plan_digest={}", first.range_plan.plan_digest);
            println!("risk_range_count={}", first.range_plan.ranges.len());
            println!("risk_provenance_status={:?}", first.provenance.status);
            println!(
                "risk_result_identity_count={}",
                first.provenance.result_identities.len()
            );
            println!("risk_witness_count={}", first.provenance.witnesses.len());
            println!(
                "risk_unmatched_opinion_count={}",
                first.provenance.unmatched_opinion_ids.len()
            );
            println!(
                "risk_multiple_match_count={}",
                first.provenance.multiply_matched_opinion_ids.len()
            );
            println!(
                "risk_provenance_registry_digest_v1={}",
                first.provenance.registry_digest_v1
            );
            println!("risk_anchor_scope_count={}", first.risk_anchor_scopes.len());
            println!("report_digest_v1={}", first.report_digest_v1);
            println!(
                "source_bound_registration_digest_v1={}",
                registration.policy_digest_v1
            );
            println!("source_bound_momentum_opinion_count={}", momentum.len());
            println!("source_bound_risk_opinion_count={}", risk.len());
            println!("source_bound_mapping_status={:?}", mapping.mapping_status);
            println!(
                "source_bound_mapping_digest_v1={}",
                mapping.registry_digest_v1
            );
            println!("source_bound_ledger_digest_v1={}", ledger.ledger_digest_v1);
            println!("chair_observed=false");
            println!("vote_created=false");
            println!("execution_created=false");
        }
        return Ok(());
    }
    if !temporal_diagnostics {
        println!(
            "inventory_accepted_series={}",
            inventory.accepted_series.len()
        );
        println!(
            "snapshot_digest_matches={}",
            reloaded_digest == snapshot.content_digest
        );
        println!(
            "snapshot_reloaded_digest_prefix={}",
            reloaded_digest.chars().take(12).collect::<String>()
        );
        println!(
            "inventory_rejection_statuses={}",
            inventory
                .rejected_snapshots
                .iter()
                .map(|rejected| format!("{:?}", rejected.status))
                .collect::<Vec<_>>()
                .join("|")
        );
        println!("campaign_sufficient={}", sufficiency.sufficient);
        println!("campaign_possible_windows={}", sufficiency.possible_windows);
    }
    if reloaded_digest != snapshot.content_digest || inventory.accepted_series.is_empty() {
        return Err("local snapshot integrity or evidence inventory failed".to_string());
    }
    run_momentum_campaign_if_enabled(
        config_path,
        &snapshot,
        &campaign_config,
        &sufficiency,
        temporal_diagnostics,
        output_format,
        cross_market_report,
    )
}

fn run_btc_cross_regime_diagnostics(
    snapshot: &crate::data::DataSnapshot,
    campaign_config: &crate::model::MomentumLearningCampaignConfigV0,
    sufficiency: &crate::model::MomentumCampaignSufficiencyV0,
    output_format: &str,
) -> Result<(), String> {
    let snapshot_digest =
        crate::data::historical_replay_dataset_digest_v0(&snapshot.normalized_dataset);
    if snapshot_digest != snapshot.content_digest {
        return Err("expanded snapshot digest verification failed".to_string());
    }
    let inventory = crate::model::inventory_historical_snapshots_v0(
        std::slice::from_ref(snapshot),
        &crate::model::HistoricalEvidencePolicyV0::default(),
    )
    .map_err(|_| "expanded snapshot inventory verification failed".to_string())?;
    if inventory.accepted_series.is_empty() || !sufficiency.sufficient {
        return Err("expanded snapshot is not sufficient for offline regime replay".to_string());
    }
    let regime_config = crate::model::BtcHistoricalRegimeConfigV0 {
        minimum_regimes: 2,
        regime_rows: sufficiency.required_minimum_rows,
        inter_regime_gap_rows: campaign_config.purge_gap_rows,
        minimum_campaign_windows_per_regime: campaign_config.minimum_evaluated_windows,
        segmentation_policy:
            crate::model::TemporalRegimeSegmentationPolicyV0::EqualLengthChronological,
    };
    let segmentation = crate::model::segment_btc_historical_regimes_v0(snapshot, &regime_config)
        .map_err(|_| "BTC regime segmentation failed".to_string())?;
    if segmentation.status != crate::model::BtcRegimeSegmentationStatusV0::Ready
        || segmentation.regimes.len() != regime_config.minimum_regimes
    {
        return Err("offline BTC regime segmentation is incomplete".to_string());
    }
    let packs = crate::model::freeze_btc_historical_regime_packs_v0(
        snapshot,
        &segmentation,
        &crate::model::HistoricalEvidencePolicyV0::default(),
    )
    .map_err(|_| "BTC regime pack freeze failed".to_string())?;
    if packs.len() != segmentation.regimes.len() {
        return Err("offline BTC regime pack count mismatch".to_string());
    }
    for (_, pack) in &packs {
        crate::model::verify_momentum_historical_evidence_pack_v0(pack)
            .map_err(|_| "BTC regime pack verification failed".to_string())?;
    }
    let encoder = crate::model::frozen_mamba3_encoder_from_seed_v0(
        &campaign_config.feature_config,
        campaign_config.campaign_seed,
        campaign_config.backend_preference,
        campaign_config.fallback_policy,
    )
    .map_err(|_| "frozen momentum encoder unavailable".to_string())?;
    let first =
        crate::model::run_btc_historical_regime_campaigns_v0(&packs, campaign_config, &encoder)
            .map_err(|_| "offline BTC regime campaign execution failed".to_string())?;
    let second =
        crate::model::run_btc_historical_regime_campaigns_v0(&packs, campaign_config, &encoder)
            .map_err(|_| "offline BTC regime replay failed".to_string())?;
    if first != second {
        return Err("offline BTC regime replay is nondeterministic".to_string());
    }
    let freeze_proof = crate::model::build_cross_regime_model_freeze_proof_v0(&first);
    let mut chronological = packs
        .iter()
        .map(|(regime, pack)| (regime, pack))
        .collect::<Vec<_>>();
    chronological.sort_by(|(left, _), (right, _)| {
        left.start_timestamp_ms
            .cmp(&right.start_timestamp_ms)
            .then_with(|| left.end_timestamp_ms.cmp(&right.end_timestamp_ms))
            .then_with(|| left.regime_id.cmp(&right.regime_id))
    });
    let closed = chronological
        .iter()
        .enumerate()
        .map(|(rank, (regime, pack))| {
            let raw = first
                .iter()
                .find(|result| result.regime_id == regime.regime_id)
                .ok_or_else(|| "offline regime report missing".to_string())?;
            let reference = crate::model::BtcTemporalRegimeRefV0 {
                regime_id: regime.regime_id.clone(),
                chronological_rank: rank,
                row_count: regime.row_count,
                range_digest: crate::core::stable_hash_string(&format!(
                    "{}:{}:{}",
                    regime.start_timestamp_ms, regime.end_timestamp_ms, regime.row_count
                )),
                pack_digest: pack.digest.clone(),
            };
            let closed = crate::model::close_btc_temporal_regime_result_v0(raw, reference);
            crate::model::validate_btc_temporal_regime_closed_result_v0(&closed)
                .map_err(|_| "offline regime report invariant failed".to_string())?;
            Ok(closed)
        })
        .collect::<Result<Vec<_>, String>>()?;
    let aggregate = crate::model::aggregate_btc_cross_regime_closed_evidence_v0(
        &closed,
        &regime_config,
        &freeze_proof,
    );
    let support_aggregate = crate::model::aggregate_btc_cross_regime_support_evidence_v0(
        &closed,
        &regime_config,
        aggregate.status,
    );
    let ledger = crate::model::build_historical_evidence_usage_ledger_v0(snapshot, &[])
        .map_err(|_| "offline evidence ledger verification failed".to_string())?;
    let holdout = crate::model::seal_prospective_holdout_v0(
        &ledger,
        &crate::model::ProspectiveHoldoutPolicyConfigV0 {
            minimum_future_rows: regime_config.regime_rows,
            required_future_windows: regime_config.minimum_campaign_windows_per_regime,
        },
        &[],
    )
    .map_err(|_| "offline prospective holdout seal failed".to_string())?;
    let regime_values = closed
        .iter()
        .map(|result| {
            let support_traces = result
                .support_traces
                .iter()
                .map(|trace| {
                    serde_json::json!({
                        "window_id": trace.envelope.window_id,
                        "envelope_construction_status": format!("{:?}", trace.envelope.construction_status),
                        "gate_applicability": format!("{:?}", trace.validation.gate_applicability),
                        "validation_support_decision": format!("{:?}", trace.validation.support_decision),
                        "test_support_decision": format!("{:?}", trace.test_support_decision),
                        "first_breach_metric": trace.validation.first_breach_metric.map(|value| format!("{:?}", value)),
                        "train_history_audit": {
                            "fixed_chronological_fold_count": trace.train_history_audit.fixed_chronological_fold_count,
                            "in_support_fold_count": trace.train_history_audit.in_support_fold_count,
                            "out_of_support_fold_count": trace.train_history_audit.out_of_support_fold_count,
                            "insufficient_evidence_fold_count": trace.train_history_audit.insufficient_evidence_fold_count,
                            "unavailable_fold_count": trace.train_history_audit.unavailable_fold_count,
                            "first_breach_metric": trace.train_history_audit.first_breach_metric.map(|value| format!("{:?}", value)),
                            "status": format!("{:?}", trace.train_history_audit.status),
                            "digest": trace.train_history_audit.digest,
                        },
                        "metrics": trace.metrics.iter().map(|metric| serde_json::json!({
                            "metric_id": format!("{:?}", metric.metric_id),
                            "measured_value": metric.measured_value,
                            "configured_threshold": metric.configured_threshold,
                            "decision": format!("{:?}", metric.decision),
                            "required": metric.required,
                        })).collect::<Vec<_>>(),
                    })
                })
                .collect::<Vec<_>>();
            serde_json::json!({
                "regime_id": result.regime.regime_id,
                "chronological_rank": result.regime.chronological_rank,
                "row_count": result.regime.row_count,
                "execution_health": format!("{:?}", result.execution_health),
                "diagnostic_completeness": format!("{:?}", result.diagnostic_completeness),
                "model_evidence_outcome": format!("{:?}", result.model_evidence_outcome),
                "operational_shadow_result": format!("{:?}", result.operational_shadow_result),
                "campaign_window_count": result.campaign_window_count,
                "no_signal_windows": result.no_signal_windows,
                "selected_checkpoint_windows": result.selected_checkpoint_windows,
                "in_support_windows": result.in_support_windows,
                "out_of_support_windows": result.out_of_support_windows,
                "support_unavailable_windows": result.support_unavailable_windows,
                "validation_in_support_windows": result.validation_in_support_windows,
                "validation_out_of_support_windows": result.validation_out_of_support_windows,
                "support_insufficient_windows": result.support_insufficient_windows,
                "support_gate_unavailable_windows": result.support_gate_unavailable_windows,
                "test_in_support_windows": result.test_in_support_windows,
                "test_out_of_support_windows": result.test_out_of_support_windows,
                "dominant_support_outcome": format!("{:?}", result.dominant_support_outcome),
                "first_breach_metric": result.first_breach_metric,
                "support_traces": support_traces,
                "accepted_predictive_versions": result.accepted_predictive_versions,
                "reason_codes": result.reason_codes,
                "execution_trace_digest": result.execution_trace.trace_digest,
                "report_digest": result.report_digest,
            })
        })
        .collect::<Vec<_>>();
    let rendered = serde_json::json!({
        "report_version": "btc-cross-regime-diagnostics-v0",
        "offline": true,
        "provider_calls": 0,
        "provider_calls_after_freeze": 0,
        "transport_construction_count": 0,
        "network_consent_reads": 0,
        "snapshot_digest_verified": true,
        "segmentation_digest": segmentation.segmentation_config_digest,
        "regime_count": segmentation.regimes.len(),
        "model_freeze_proof_digest": freeze_proof.proof_digest,
        "model_freeze_all_equal": freeze_proof.all_equal,
        "regimes": regime_values,
        "representation_status": format!("{:?}", aggregate.status),
        "support_status": format!("{:?}", support_aggregate.support_status),
        "support_aggregate": {
            "validation_in_support_windows": support_aggregate.validation_in_support_windows,
            "validation_out_of_support_windows": support_aggregate.validation_out_of_support_windows,
            "support_insufficient_windows": support_aggregate.support_insufficient_windows,
            "support_gate_unavailable_windows": support_aggregate.support_gate_unavailable_windows,
            "test_in_support_windows": support_aggregate.test_in_support_windows,
            "test_out_of_support_windows": support_aggregate.test_out_of_support_windows,
            "accepted_predictive_versions": support_aggregate.accepted_predictive_versions,
        },
        "diagnostic_failure_root_cause": aggregate.diagnostic_failure_root_cause.map(|value| format!("{:?}", value)),
        "cross_regime_report_digest": aggregate.report_digest,
        "usage_ledger_digest": ledger.ledger_digest,
        "maximum_consumed_timestamp_ms": ledger.maximum_consumed_timestamp_ms,
        "prospective_holdout_status": format!("{:?}", holdout.status),
        "prospective_holdout_opened": holdout.opened,
        "prospective_holdout_labels_accessed": holdout.labels_accessed,
    });
    match output_format {
        "json" => println!("{rendered}"),
        "text" => {
            println!("report_version=btc-cross-regime-diagnostics-v0");
            println!("offline=true");
            println!("provider_calls=0");
            println!("regime_count={}", closed.len());
            for result in &closed {
                println!(
                    "regime={} rank={} execution_health={:?} diagnostic_completeness={:?} model_evidence_outcome={:?} operational_shadow_result={:?} dominant_support_outcome={:?} validation_in_support_windows={} validation_out_of_support_windows={} test_in_support_windows={} test_out_of_support_windows={} support_gate_unavailable_windows={} accepted_predictive_versions={} report_digest={}",
                    result.regime.regime_id,
                    result.regime.chronological_rank,
                    result.execution_health,
                    result.diagnostic_completeness,
                    result.model_evidence_outcome,
                    result.operational_shadow_result,
                    result.dominant_support_outcome,
                    result.validation_in_support_windows,
                    result.validation_out_of_support_windows,
                    result.test_in_support_windows,
                    result.test_out_of_support_windows,
                    result.support_gate_unavailable_windows,
                    result.accepted_predictive_versions,
                    result.report_digest,
                );
                for trace in &result.support_traces {
                    println!(
                        "support_window={} envelope={:?} applicability={:?} validation={:?} test={:?} first_breach={:?} train_history_audit={:?} train_history_folds={} train_history_in_support={} train_history_out_of_support={} train_history_first_breach={:?}",
                        trace.envelope.window_id,
                        trace.envelope.construction_status,
                        trace.validation.gate_applicability,
                        trace.validation.support_decision,
                        trace.test_support_decision,
                        trace.validation.first_breach_metric,
                        trace.train_history_audit.status,
                        trace.train_history_audit.fixed_chronological_fold_count,
                        trace.train_history_audit.in_support_fold_count,
                        trace.train_history_audit.out_of_support_fold_count,
                        trace.train_history_audit.first_breach_metric,
                    );
                    for metric in &trace.metrics {
                        println!(
                            "support_metric={:?} measured={:?} threshold={:?} decision={:?} required={}",
                            metric.metric_id,
                            metric.measured_value,
                            metric.configured_threshold,
                            metric.decision,
                            metric.required,
                        );
                    }
                }
            }
            println!("representation_status={:?}", aggregate.status);
            println!("support_status={:?}", support_aggregate.support_status);
            println!("support_report_digest={}", support_aggregate.report_digest);
            println!("prospective_holdout_status={:?}", holdout.status);
        }
        _ => return Err("unsupported BTC cross-regime diagnostics output format".to_string()),
    }
    if aggregate.status == crate::model::BtcCrossRegimeRepresentationStatusV0::DiagnosticFailure
        || support_aggregate.support_status
            == crate::model::BtcCrossRegimeSupportStatusV0::DiagnosticFailure
    {
        Err("offline BTC cross-regime diagnostics found a technical failure".to_string())
    } else {
        Ok(())
    }
}

#[derive(Serialize)]
struct ProspectiveExternalAdmissionReportV0 {
    report_version: &'static str,
    offline: bool,
    compatibility: String,
    registration_digest: String,
    source_classification: String,
    candidate_input_status: String,
    admission_status: String,
    admitted_row_count: usize,
    shared_raw_evidence_reference_count: usize,
    shared_raw_evidence_digest: Option<String>,
    momentum_independently_valid: bool,
    risk_independently_valid: bool,
    momentum_event_count: usize,
    risk_event_count: usize,
    momentum_abstention_count: usize,
    risk_abstention_count: usize,
    maturity_status: String,
    reward_eligibility: String,
    reward_candidate_count: usize,
    reward_apply_count: usize,
    provider_calls: usize,
    transport_constructions: usize,
    network_consent_reads: usize,
    credential_reads: usize,
    label_reads: usize,
    chair_decision_count: usize,
    reward_applied_count: usize,
    penalty_applied_count: usize,
    voice_mutation_count: usize,
    cooldown_mutation_count: usize,
    promotion_mutation_count: usize,
    quarantine_mutation_count: usize,
    execution_count: usize,
}

#[derive(Serialize)]
struct ProspectivePublicExportReportV0 {
    report_version: &'static str,
    mode: &'static str,
    registration_digest: String,
    registration_reopened_and_verified: bool,
    request_fingerprint: String,
    request_to_utc: String,
    explicit_single_request_consent: bool,
    request_attempted: bool,
    request_count: usize,
    retry_count: usize,
    http_status_class: Option<String>,
    returned_item_count: usize,
    acquisition_outcome: String,
    acquisition_receipt_digest: Option<String>,
    network_capsule_created: bool,
    network_capsule_digest: Option<String>,
    admission_status: String,
    shared_raw_evidence_reference_count: usize,
    momentum_event_count: usize,
    risk_event_count: usize,
    maturity_status: String,
    reward_eligibility: String,
    prospective_label_reads: usize,
    mature_outcomes: usize,
    interim_metrics: usize,
    reward_candidate_count: usize,
    reward_apply_count: usize,
    network_request_count: usize,
    authority_action_count: usize,
    legacy_blind_receipt_unchanged: bool,
    legacy_request_registry_unchanged: bool,
}

fn print_prospective_public_export_report(
    report: &ProspectivePublicExportReportV0,
    output_format: &str,
) -> Result<(), String> {
    match output_format {
        "json" => println!(
            "{}",
            serde_json::to_string(report)
                .map_err(|_| "prospective public export report serialization failed")?
        ),
        "text" => {
            println!("report_version={}", report.report_version);
            println!("mode={}", report.mode);
            println!("registration_digest={}", report.registration_digest);
            println!(
                "registration_reopened_and_verified={}",
                report.registration_reopened_and_verified
            );
            println!("request_fingerprint={}", report.request_fingerprint);
            println!("request_to_utc={}", report.request_to_utc);
            println!(
                "explicit_single_request_consent={}",
                report.explicit_single_request_consent
            );
            println!("request_attempted={}", report.request_attempted);
            println!("request_count={}", report.request_count);
            println!("retry_count={}", report.retry_count);
            println!(
                "http_status_class={}",
                report.http_status_class.as_deref().unwrap_or_default()
            );
            println!("returned_item_count={}", report.returned_item_count);
            println!("acquisition_outcome={}", report.acquisition_outcome);
            println!(
                "acquisition_receipt_digest={}",
                report
                    .acquisition_receipt_digest
                    .as_deref()
                    .unwrap_or_default()
            );
            println!("network_capsule_created={}", report.network_capsule_created);
            println!(
                "network_capsule_digest={}",
                report.network_capsule_digest.as_deref().unwrap_or_default()
            );
            println!("admission_status={}", report.admission_status);
            println!(
                "shared_raw_evidence_reference_count={}",
                report.shared_raw_evidence_reference_count
            );
            println!("momentum_event_count={}", report.momentum_event_count);
            println!("risk_event_count={}", report.risk_event_count);
            println!("maturity_status={}", report.maturity_status);
            println!("reward_eligibility={}", report.reward_eligibility);
            println!("prospective_label_reads={}", report.prospective_label_reads);
            println!("mature_outcomes={}", report.mature_outcomes);
            println!("interim_metrics={}", report.interim_metrics);
            println!("reward_candidate_count={}", report.reward_candidate_count);
            println!("reward_apply_count={}", report.reward_apply_count);
            println!("network_request_count={}", report.network_request_count);
            println!("authority_action_count={}", report.authority_action_count);
            println!(
                "legacy_blind_receipt_unchanged={}",
                report.legacy_blind_receipt_unchanged
            );
            println!(
                "legacy_request_registry_unchanged={}",
                report.legacy_request_registry_unchanged
            );
        }
        _ => return Err("unsupported prospective public export output format".into()),
    }
    Ok(())
}

fn print_prospective_external_admission_report(
    report: &ProspectiveExternalAdmissionReportV0,
    output_format: &str,
) -> Result<(), String> {
    match output_format {
        "json" => println!(
            "{}",
            serde_json::to_string(report)
                .map_err(|_| "prospective external admission report serialization failed")?
        ),
        "text" => {
            println!("report_version={}", report.report_version);
            println!("offline={}", report.offline);
            println!("compatibility={}", report.compatibility);
            println!("registration_digest={}", report.registration_digest);
            println!("source_classification={}", report.source_classification);
            println!("candidate_input_status={}", report.candidate_input_status);
            println!("admission_status={}", report.admission_status);
            println!("admitted_row_count={}", report.admitted_row_count);
            println!(
                "shared_raw_evidence_reference_count={}",
                report.shared_raw_evidence_reference_count
            );
            println!(
                "shared_raw_evidence_digest={}",
                report
                    .shared_raw_evidence_digest
                    .as_deref()
                    .unwrap_or_default()
            );
            println!(
                "momentum_independently_valid={}",
                report.momentum_independently_valid
            );
            println!(
                "risk_independently_valid={}",
                report.risk_independently_valid
            );
            println!("momentum_event_count={}", report.momentum_event_count);
            println!("risk_event_count={}", report.risk_event_count);
            println!(
                "momentum_abstention_count={}",
                report.momentum_abstention_count
            );
            println!("risk_abstention_count={}", report.risk_abstention_count);
            println!("maturity_status={}", report.maturity_status);
            println!("reward_eligibility={}", report.reward_eligibility);
            println!("reward_candidate_count={}", report.reward_candidate_count);
            println!("reward_apply_count={}", report.reward_apply_count);
            println!("provider_calls={}", report.provider_calls);
            println!("transport_constructions={}", report.transport_constructions);
            println!("network_consent_reads={}", report.network_consent_reads);
            println!("credential_reads={}", report.credential_reads);
            println!("label_reads={}", report.label_reads);
            println!("chair_decision_count={}", report.chair_decision_count);
            println!("reward_applied_count={}", report.reward_applied_count);
            println!("penalty_applied_count={}", report.penalty_applied_count);
            println!("voice_mutation_count={}", report.voice_mutation_count);
            println!("cooldown_mutation_count={}", report.cooldown_mutation_count);
            println!(
                "promotion_mutation_count={}",
                report.promotion_mutation_count
            );
            println!(
                "quarantine_mutation_count={}",
                report.quarantine_mutation_count
            );
            println!("execution_count={}", report.execution_count);
        }
        _ => return Err("unsupported prospective external admission output format".into()),
    }
    Ok(())
}

fn prospective_external_admission_report_v0(
    registration: &crate::model::ProspectiveExternalAdmissionRegistrationV0,
    compatibility: crate::model::ExternalAdmissionCompatibilityV0,
    source_classification: String,
    candidate_input_status: &str,
    admission_status: crate::model::ProspectiveRowAdmissionStatusV0,
    shared: Option<&crate::model::SharedProspectiveRawEvidenceV0>,
    momentum_valid: bool,
    risk_valid: bool,
    momentum_events: usize,
    risk_events: usize,
) -> ProspectiveExternalAdmissionReportV0 {
    let sealed_events = momentum_events.saturating_add(risk_events);
    ProspectiveExternalAdmissionReportV0 {
        report_version: "prospective-external-row-admission-v0",
        offline: true,
        compatibility: format!("{compatibility:?}"),
        registration_digest: registration.registration_digest.clone(),
        source_classification,
        candidate_input_status: candidate_input_status.into(),
        admission_status: format!("{admission_status:?}"),
        admitted_row_count: usize::from(shared.is_some()),
        shared_raw_evidence_reference_count: usize::from(shared.is_some()),
        shared_raw_evidence_digest: shared.map(|value| value.reference_digest.clone()),
        momentum_independently_valid: momentum_valid,
        risk_independently_valid: risk_valid,
        momentum_event_count: momentum_events,
        risk_event_count: risk_events,
        momentum_abstention_count: momentum_events,
        risk_abstention_count: risk_events,
        maturity_status: if sealed_events == 0 {
            "NoSealedEvents".into()
        } else {
            "AwaitingMaturity".into()
        },
        reward_eligibility: format!(
            "{:?}",
            crate::model::external_admission_reward_eligibility_status_v0(sealed_events)
        ),
        reward_candidate_count: 0,
        reward_apply_count: 0,
        provider_calls: 0,
        transport_constructions: 0,
        network_consent_reads: 0,
        credential_reads: 0,
        label_reads: 0,
        chair_decision_count: 0,
        reward_applied_count: 0,
        penalty_applied_count: 0,
        voice_mutation_count: 0,
        cooldown_mutation_count: 0,
        promotion_mutation_count: 0,
        quarantine_mutation_count: 0,
        execution_count: 0,
    }
}

fn run_prospective_external_row_admission_report(
    config_path: &Path,
    snapshot: &crate::data::DataSnapshot,
    output_format: &str,
) -> Result<(), String> {
    let report = build_prospective_external_row_admission_report(config_path, snapshot)?;
    print_prospective_external_admission_report(&report, output_format)
}

fn build_prospective_external_row_admission_report(
    config_path: &Path,
    snapshot: &crate::data::DataSnapshot,
) -> Result<ProspectiveExternalAdmissionReportV0, String> {
    let local_dir = config_path
        .parent()
        .ok_or("local prospective external admission directory unavailable")?;
    let momentum_path = local_dir.join("prospective_shadow_challenge_v0.json");
    let momentum = crate::model::read_prospective_challenge_local_state_v0(&momentum_path)
        .map_err(|_| "local prospective external admission momentum state unavailable")?;
    let risk_config = crate::model::CycleRiskShadowConfigV0::default();
    let risk_report = crate::model::run_cycle_risk_shadow_v0(snapshot, &risk_config)
        .map_err(|_| "offline Cycle/Risk prospective contract unavailable")?;
    let risk_capsule = crate::model::prepare_cycle_risk_prospective_tournament_v0(
        snapshot,
        &risk_report,
        &risk_config,
    )
    .map_err(|_| "offline Cycle/Risk prospective contract unavailable")?;
    let compatibility =
        crate::model::audit_external_admission_compatibility_v0(&momentum, &risk_capsule);
    if !matches!(
        compatibility,
        crate::model::ExternalAdmissionCompatibilityV0::PermittedWithExternalAdmissionRegistration
            | crate::model::ExternalAdmissionCompatibilityV0::PermittedByExistingContracts
    ) {
        return Err(format!(
            "prospective external admission contract incompatible: {compatibility:?}"
        ));
    }
    let maximum_consumed_timestamp = snapshot
        .normalized_dataset
        .rows
        .iter()
        .map(|row| row.timestamp_ms)
        .max()
        .ok_or("prospective external admission snapshot empty")?;
    let expected_registration = crate::model::pre_register_prospective_external_row_admission_v0(
        &momentum,
        &risk_capsule,
        maximum_consumed_timestamp,
    )?;
    let registration_path =
        local_dir.join("prospective_external_row_admission_registration_v0.json");
    let registration = if registration_path.exists() {
        let existing =
            crate::model::read_prospective_external_admission_registration_v0(&registration_path)?;
        crate::model::validate_prospective_external_admission_registration_v0(
            &existing,
            &momentum,
            &risk_capsule,
        )?;
        if existing != expected_registration {
            return Err("prospective external admission registration mismatch".into());
        }
        existing
    } else {
        crate::model::write_prospective_external_admission_registration_v0(
            &registration_path,
            &expected_registration,
            &momentum,
            &risk_capsule,
        )?;
        let reopened =
            crate::model::read_prospective_external_admission_registration_v0(&registration_path)?;
        crate::model::validate_prospective_external_admission_registration_v0(
            &reopened,
            &momentum,
            &risk_capsule,
        )?;
        reopened
    };
    let risk_state_path = local_dir.join("cycle_risk_prospective_local_state_v0.json");
    let persisted_risk_event_count = if risk_state_path.is_file() {
        crate::model::read_cycle_risk_prospective_local_state_v0(&risk_state_path)
            .map_err(|_| "local prospective external admission risk state unavailable")?
            .journal
            .event_count
    } else {
        0
    };
    let intake_path = local_dir.join("prospective_external_row_capsule_v0.json");
    if !intake_path.is_file() {
        return Ok(prospective_external_admission_report_v0(
            &registration,
            compatibility,
            "AwaitingQualifiedExternalRow".into(),
            "NoQualifiedExternalCapsuleDiscovered",
            crate::model::ProspectiveRowAdmissionStatusV0::AwaitingQualifiedExternalRow,
            None,
            false,
            false,
            momentum.journal.events.len(),
            persisted_risk_event_count,
        ));
    }
    let capsule = crate::model::read_prospective_external_row_capsule_v0(&intake_path)?;
    let mut existing_timestamps = momentum
        .vault
        .finalized_rows
        .iter()
        .map(|row| row.timestamp_ms)
        .collect::<BTreeSet<_>>();
    let mut existing_digests = momentum
        .vault
        .finalized_rows
        .iter()
        .map(|row| row.canonical_row_digest.clone())
        .collect::<BTreeSet<_>>();
    let mut latest_admitted_timestamp = momentum.vault.last_timestamp_ms;
    let existing_risk = if risk_state_path.is_file() {
        let state = crate::model::read_cycle_risk_prospective_local_state_v0(&risk_state_path)
            .map_err(|_| "local prospective external admission risk state unavailable")?;
        if state.capsule.capsule_digest != risk_capsule.capsule_digest {
            return Err("local prospective external admission risk contract mismatch".into());
        }
        for timestamp in &state.vault.admitted_row_timestamps {
            existing_timestamps.insert(*timestamp);
            latest_admitted_timestamp =
                Some(latest_admitted_timestamp.unwrap_or(0).max(*timestamp));
        }
        for digest in &state.vault.admitted_row_digests {
            existing_digests.insert(digest.clone());
        }
        Some(state)
    } else {
        None
    };
    let context = crate::model::ProspectiveExternalAdmissionContextV0 {
        existing_row_timestamps: existing_timestamps,
        existing_canonical_row_digests: existing_digests,
        latest_admitted_timestamp,
    };
    let admission_status = crate::model::prospective_external_row_admission_status_v0(
        &registration,
        &momentum,
        &risk_capsule,
        &capsule,
        &context,
    );
    if admission_status != crate::model::ProspectiveRowAdmissionStatusV0::Admitted {
        return Ok(prospective_external_admission_report_v0(
            &registration,
            compatibility,
            format!("{:?}", capsule.source_class),
            "QualifiedCapsuleRejected",
            admission_status,
            None,
            false,
            false,
            momentum.journal.events.len(),
            existing_risk
                .as_ref()
                .map(|state| state.journal.event_count)
                .unwrap_or(0),
        ));
    }
    let shared = crate::model::build_shared_prospective_raw_evidence_v0(
        &registration,
        &capsule,
        admission_status,
    )?;
    let mut risk = match existing_risk {
        Some(state) => state,
        None => {
            let mut state =
                crate::model::new_cycle_risk_prospective_local_state_v0(risk_capsule.clone())
                    .map_err(|_| "prospective external admission risk local state unavailable")?;
            crate::model::commit_cycle_risk_pre_registration_v0(&mut state)
                .map_err(|_| "prospective external admission risk pre-registration failed")?;
            state
        }
    };
    let momentum_validation = crate::model::validate_momentum_shared_prospective_reference_v0(
        &registration,
        &momentum,
        &shared,
    );
    let risk_validation =
        crate::model::validate_risk_shared_prospective_reference_v0(&registration, &risk, &shared);
    let momentum_event = momentum_validation
        .independently_valid
        .then(|| {
            crate::model::seal_external_prospective_event_v0(
                &momentum_validation,
                &shared,
                &momentum,
                &risk,
                crate::model::ProspectiveOperationalOutcomeV0::ShadowAbstentionSupportUnavailable,
                Some("frozen_external_inference_support_unavailable".into()),
            )
        })
        .transpose()?;
    let risk_event = risk_validation
        .independently_valid
        .then(|| {
            crate::model::seal_external_prospective_event_v0(
                &risk_validation,
                &shared,
                &momentum,
                &risk,
                crate::model::ProspectiveOperationalOutcomeV0::ShadowAbstentionSupportUnavailable,
                Some("frozen_external_inference_support_unavailable".into()),
            )
        })
        .transpose()?;
    let mut updated_momentum = momentum.clone();
    if momentum_event.is_some() || risk_event.is_some() {
        crate::model::append_external_admission_to_local_stores_v0(
            &mut updated_momentum,
            &mut risk,
            &shared,
            momentum_event.as_ref(),
            risk_event.as_ref(),
        )?;
    }
    if momentum_event.is_some() {
        crate::model::write_prospective_challenge_local_state_v0(&momentum_path, &updated_momentum)
            .map_err(|_| "prospective external admission momentum local write failed")?;
        crate::model::read_prospective_challenge_local_state_v0(&momentum_path)
            .map_err(|_| "prospective external admission momentum local reread failed")?;
    }
    if risk_event.is_some() {
        crate::model::write_cycle_risk_prospective_local_state_v0(&risk_state_path, &risk)
            .map_err(|_| "prospective external admission risk local write failed")?;
        crate::model::read_cycle_risk_prospective_local_state_v0(&risk_state_path)
            .map_err(|_| "prospective external admission risk local reread failed")?;
    }
    Ok(prospective_external_admission_report_v0(
        &registration,
        compatibility,
        format!("{:?}", capsule.source_class),
        "QualifiedCapsuleValidated",
        admission_status,
        Some(&shared),
        momentum_validation.independently_valid,
        risk_validation.independently_valid,
        updated_momentum.journal.events.len(),
        risk.journal.event_count,
    ))
}

fn reopen_external_admission_registration_for_public_export(
    config_path: &Path,
    snapshot: &crate::data::DataSnapshot,
) -> Result<
    (
        crate::model::ProspectiveExternalAdmissionRegistrationV0,
        Vec<crate::data::ProspectiveBlindAcquisitionReceiptV0>,
    ),
    String,
> {
    let local_dir = config_path
        .parent()
        .ok_or("local prospective public export directory unavailable")?;
    let momentum_path = local_dir.join("prospective_shadow_challenge_v0.json");
    let momentum = crate::model::read_prospective_challenge_local_state_v0(&momentum_path)
        .map_err(|_| "local prospective public export momentum state unavailable")?;
    let risk_config = crate::model::CycleRiskShadowConfigV0::default();
    let risk_report = crate::model::run_cycle_risk_shadow_v0(snapshot, &risk_config)
        .map_err(|_| "offline Cycle/Risk prospective contract unavailable")?;
    let risk_capsule = crate::model::prepare_cycle_risk_prospective_tournament_v0(
        snapshot,
        &risk_report,
        &risk_config,
    )
    .map_err(|_| "offline Cycle/Risk prospective contract unavailable")?;
    if !matches!(
        crate::model::audit_external_admission_compatibility_v0(&momentum, &risk_capsule),
        crate::model::ExternalAdmissionCompatibilityV0::PermittedWithExternalAdmissionRegistration
            | crate::model::ExternalAdmissionCompatibilityV0::PermittedByExistingContracts
    ) {
        return Err("prospective public export admission contract incompatible".into());
    }
    let maximum_consumed_timestamp = snapshot
        .normalized_dataset
        .rows
        .iter()
        .map(|row| row.timestamp_ms)
        .max()
        .ok_or("prospective public export snapshot empty")?;
    let expected = crate::model::pre_register_prospective_external_row_admission_v0(
        &momentum,
        &risk_capsule,
        maximum_consumed_timestamp,
    )?;
    let registration_path =
        local_dir.join("prospective_external_row_admission_registration_v0.json");
    let registration =
        crate::model::read_prospective_external_admission_registration_v0(&registration_path)
            .map_err(|_| "prospective public export admission registration unavailable")?;
    crate::model::validate_prospective_external_admission_registration_v0(
        &registration,
        &momentum,
        &risk_capsule,
    )?;
    if registration != expected {
        return Err("prospective public export admission registration mismatch".into());
    }
    Ok((registration, momentum.blind_acquisition_receipts.clone()))
}

fn write_ignored_local_bytes(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or("prospective public export local storage unavailable")?;
    fs::create_dir_all(parent)
        .map_err(|_| "prospective public export local storage unavailable")?;
    let temporary = path.with_extension("tmp");
    fs::write(&temporary, bytes).map_err(|_| "prospective public export local storage failed")?;
    fs::rename(temporary, path)
        .map_err(|_| "prospective public export local storage failed".to_string())
}

fn public_export_report_v0(
    mode: &'static str,
    registration: &crate::data::ProspectivePublicExportAcquisitionRegistrationV0,
    registration_reopened_and_verified: bool,
    receipt: Option<&crate::data::ProspectivePublicExportAcquisitionReceiptV0>,
    capsule: Option<&crate::data::ProspectiveNetworkExportCapsuleV0>,
    admission: Option<&ProspectiveExternalAdmissionReportV0>,
    legacy_blind_receipt_unchanged: bool,
) -> ProspectivePublicExportReportV0 {
    let receipt = receipt.cloned();
    let admission_status = admission
        .map(|value| value.admission_status.clone())
        .unwrap_or_else(|| "NotAttempted".into());
    ProspectivePublicExportReportV0 {
        report_version: "prospective-public-export-acquisition-v0",
        mode,
        registration_digest: registration.registration_digest.clone(),
        registration_reopened_and_verified,
        request_fingerprint: receipt
            .as_ref()
            .map(|value| value.request_fingerprint.clone())
            .unwrap_or_default(),
        request_to_utc: receipt
            .as_ref()
            .map(|value| value.request_to_utc.clone())
            .unwrap_or_default(),
        explicit_single_request_consent: receipt
            .as_ref()
            .is_some_and(|value| value.request_attempted),
        request_attempted: receipt
            .as_ref()
            .is_some_and(|value| value.request_attempted),
        request_count: receipt.as_ref().map_or(0, |value| value.request_count),
        retry_count: receipt.as_ref().map_or(0, |value| value.retry_count),
        http_status_class: receipt
            .as_ref()
            .and_then(|value| value.http_status_class.clone()),
        returned_item_count: receipt
            .as_ref()
            .map_or(0, |value| value.returned_item_count),
        acquisition_outcome: receipt
            .as_ref()
            .map(|value| format!("{:?}", value.outcome))
            .unwrap_or_else(|| "DryRunReady".into()),
        acquisition_receipt_digest: receipt.as_ref().map(|value| value.receipt_digest.clone()),
        network_capsule_created: capsule.is_some(),
        network_capsule_digest: capsule.map(|value| value.capsule_digest.clone()),
        admission_status,
        shared_raw_evidence_reference_count: admission
            .map(|value| value.shared_raw_evidence_reference_count)
            .unwrap_or(0),
        momentum_event_count: admission
            .map(|value| value.momentum_event_count)
            .unwrap_or(0),
        risk_event_count: admission.map(|value| value.risk_event_count).unwrap_or(0),
        maturity_status: admission
            .map(|value| value.maturity_status.clone())
            .unwrap_or_else(|| "NoSealedEvents".into()),
        reward_eligibility: admission
            .map(|value| value.reward_eligibility.clone())
            .unwrap_or_else(|| "IneligibleMinimumSamples".into()),
        prospective_label_reads: 0,
        mature_outcomes: 0,
        interim_metrics: 0,
        reward_candidate_count: 0,
        reward_apply_count: 0,
        network_request_count: receipt.as_ref().map_or(0, |value| value.request_count),
        authority_action_count: 0,
        legacy_blind_receipt_unchanged,
        legacy_request_registry_unchanged: legacy_blind_receipt_unchanged,
    }
}

fn run_one_upbit_prospective_public_export(
    config_path: &Path,
    snapshot: &crate::data::DataSnapshot,
    output_format: &str,
    dry_run: bool,
    execute: bool,
    allow_network: bool,
    confirm_single_public_candle_request: bool,
) -> Result<(), String> {
    if output_format != "text" && output_format != "json" {
        return Err("unsupported prospective public export output format".into());
    }
    if dry_run == execute {
        return Err("select exactly one prospective public export mode".into());
    }
    let config = crate::data::UpbitHistoricalPilotConfigV0::from_toml_path(config_path)
        .map_err(|_| "local provider config unavailable")?;
    config.validate()?;
    let local_dir = config_path
        .parent()
        .ok_or("local prospective public export directory unavailable")?;
    let (admission_registration, old_blind_receipts) =
        reopen_external_admission_registration_for_public_export(config_path, snapshot)?;
    let expected_registration =
        crate::data::pre_register_prospective_public_export_acquisition_v0(&config)?;
    let registration_path =
        local_dir.join("prospective_public_export_acquisition_registration_v0.json");
    if dry_run {
        let registration = if registration_path.is_file() {
            let existing = crate::data::read_prospective_public_export_acquisition_registration_v0(
                &registration_path,
            )?;
            crate::data::validate_prospective_public_export_acquisition_registration_v0(&existing)?;
            if existing != expected_registration {
                return Err("prospective public export registration mismatch".into());
            }
            existing
        } else {
            crate::data::write_prospective_public_export_acquisition_registration_v0(
                &registration_path,
                &expected_registration,
            )?;
            crate::data::read_prospective_public_export_acquisition_registration_v0(
                &registration_path,
            )?
        };
        crate::data::validate_prospective_public_export_acquisition_registration_v0(&registration)?;
        if registration != expected_registration {
            return Err("prospective public export registration reread mismatch".into());
        }
        let plan = crate::data::prospective_public_export_request_plan_v0(
            &registration,
            current_utc_timestamp_ms(),
        )?;
        let dry_receipt = crate::data::ProspectivePublicExportAcquisitionReceiptV0 {
            receipt_version: "prospective-public-export-acquisition-receipt-v0".into(),
            registration_digest: registration.registration_digest.clone(),
            request_attempted: false,
            request_count: 0,
            retry_count: 0,
            request_fingerprint: plan.request_fingerprint,
            request_to_utc: plan.request_to_utc,
            http_status_class: None,
            response_body_digest: None,
            returned_item_count: 0,
            outcome: crate::data::ProspectivePublicExportAcquisitionOutcomeV0::ConsentMissing,
            capsule_digest: None,
            legacy_receipt_unchanged: true,
            receipt_digest: String::new(),
        };
        let mut report = public_export_report_v0(
            "dry-run",
            &registration,
            true,
            Some(&dry_receipt),
            None,
            None,
            true,
        );
        report.acquisition_outcome = "DryRunReady".into();
        report.acquisition_receipt_digest = None;
        return print_prospective_public_export_report(&report, output_format);
    }
    let (registration, registration_verified) = if registration_path.is_file() {
        match crate::data::read_prospective_public_export_acquisition_registration_v0(
            &registration_path,
        ) {
            Ok(existing)
                if crate::data::validate_prospective_public_export_acquisition_registration_v0(
                    &existing,
                )
                .is_ok()
                    && existing == expected_registration =>
            {
                (existing, true)
            }
            _ => (expected_registration.clone(), false),
        }
    } else {
        (expected_registration.clone(), false)
    };
    let receipt_path = local_dir.join("prospective_public_export_acquisition_receipt_v0.json");
    let (existing_receipt, receipt_storage_valid) = if receipt_path.is_file() {
        match crate::data::read_prospective_public_export_acquisition_receipt_v0(&receipt_path) {
            Ok(receipt)
                if crate::data::verify_prospective_public_export_acquisition_receipt_v0(
                    &receipt,
                ) =>
            {
                (Some(receipt), true)
            }
            _ => (None, false),
        }
    } else {
        (None, true)
    };
    let registration_verified = registration_verified && receipt_storage_valid;
    if let Some(receipt) = existing_receipt
        .as_ref()
        .filter(|receipt| receipt.request_attempted)
    {
        let network_capsule_path = local_dir.join("prospective_network_export_capsule_v0.json");
        let network_capsule =
            crate::data::read_prospective_network_export_capsule_v0(&network_capsule_path)
                .map_err(
                    |_| "prospective public export capsule unavailable after recorded request",
                )?;
        if !crate::data::verify_prospective_network_export_capsule_v0(&network_capsule) {
            return Err("prospective public export capsule invalid after recorded request".into());
        }
        let admission = build_prospective_external_row_admission_report(config_path, snapshot)?;
        let momentum_path = local_dir.join("prospective_shadow_challenge_v0.json");
        let reread_momentum =
            crate::model::read_prospective_challenge_local_state_v0(&momentum_path)
                .map_err(|_| "prospective public export momentum reread unavailable")?;
        let legacy_blind_receipt_unchanged =
            reread_momentum.blind_acquisition_receipts == old_blind_receipts;
        if !legacy_blind_receipt_unchanged {
            return Err("legacy blind acquisition receipt changed".into());
        }
        return print_prospective_public_export_report(
            &public_export_report_v0(
                "execute-status",
                &registration,
                registration_verified,
                Some(receipt),
                Some(&network_capsule),
                Some(&admission),
                legacy_blind_receipt_unchanged,
            ),
            output_format,
        );
    }
    let intake_path = local_dir.join("prospective_external_row_capsule_v0.json");
    if registration_verified && !intake_path.is_file() {
        // A single response can be admitted only into an empty, pre-registered
        // Sprint 64 intake.  Refusing before transport preserves the one-call budget.
    } else if registration_verified && intake_path.is_file() {
        return Err("prospective public export intake already exists; no request attempted".into());
    }
    let acquisition = crate::data::execute_prospective_public_export_acquisition_v0(
        &registration,
        registration_verified,
        existing_receipt.as_ref(),
        allow_network,
        confirm_single_public_candle_request,
        current_utc_timestamp_ms(),
        |plan| crate::data::fetch_one_prospective_public_export_v0(&registration, plan),
    );
    if acquisition.receipt.request_attempted {
        crate::data::write_prospective_public_export_acquisition_receipt_v0(
            &receipt_path,
            &acquisition.receipt,
        )?;
        let reread =
            crate::data::read_prospective_public_export_acquisition_receipt_v0(&receipt_path)?;
        if reread != acquisition.receipt
            || !crate::data::verify_prospective_public_export_acquisition_receipt_v0(&reread)
        {
            return Err("prospective public export receipt reread mismatch".into());
        }
    }
    let mut admission = None;
    if let Some(network_capsule) = acquisition.capsule.as_ref() {
        let raw_response_path = local_dir.join("prospective_public_export_response_v0.json");
        write_ignored_local_bytes(&raw_response_path, &network_capsule.raw_response)?;
        let network_capsule_path = local_dir.join("prospective_network_export_capsule_v0.json");
        crate::data::write_prospective_network_export_capsule_v0(
            &network_capsule_path,
            network_capsule,
        )?;
        let reread_network =
            crate::data::read_prospective_network_export_capsule_v0(&network_capsule_path)?;
        if !crate::data::verify_prospective_network_export_capsule_v0(&reread_network) {
            return Err("prospective network export capsule reread mismatch".into());
        }
        let intake = crate::data::convert_prospective_network_export_to_external_row_capsule_v0(
            &reread_network,
            &admission_registration,
        )?;
        write_ignored_local_bytes(
            &intake_path,
            &serde_json::to_vec(&intake)
                .map_err(|_| "prospective external intake serialization failed")?,
        )?;
        admission = Some(build_prospective_external_row_admission_report(
            config_path,
            snapshot,
        )?);
    }
    let momentum_path = local_dir.join("prospective_shadow_challenge_v0.json");
    let reread_momentum = crate::model::read_prospective_challenge_local_state_v0(&momentum_path)
        .map_err(|_| "prospective public export momentum reread unavailable")?;
    let legacy_blind_receipt_unchanged =
        reread_momentum.blind_acquisition_receipts == old_blind_receipts;
    if !legacy_blind_receipt_unchanged {
        return Err("legacy blind acquisition receipt changed".into());
    }
    print_prospective_public_export_report(
        &public_export_report_v0(
            "execute",
            &registration,
            registration_verified,
            Some(&acquisition.receipt),
            acquisition.capsule.as_ref(),
            admission.as_ref(),
            legacy_blind_receipt_unchanged,
        ),
        output_format,
    )
}

fn current_utc_timestamp_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or_default()
}

struct ProspectiveOpeningContextV0 {
    registration: crate::model::ProspectiveOneTimeOpeningRegistrationV0,
    plans: Vec<crate::model::ProspectiveEventMaturityPlanV0>,
    public_registration: crate::data::ProspectivePublicExportAcquisitionRegistrationV0,
    momentum: crate::model::ProspectiveChallengeLocalStateV0,
    risk: crate::model::CycleRiskProspectiveLocalStateV0,
    external_capsule: crate::model::ProspectiveExternalRowCapsuleV0,
    event_audit: crate::model::ProspectiveSealedEventAuditV0,
    outcome_receipt: crate::data::ProspectiveOutcomeAcquisitionReceiptV0,
    outcome_capsule: crate::data::ProspectiveOutcomeEvidenceCapsuleV0,
    event_integrity_valid: bool,
    challenge_valid: bool,
    expected_series_id: String,
    label_open_count: usize,
}

#[derive(Serialize)]
struct ProspectiveOpeningRegistrationReportV0 {
    report_version: &'static str,
    offline: bool,
    opening_registration_digest: String,
    opening_registration_reopened_and_verified: bool,
    event_count: usize,
    event_integrity_valid: bool,
    required_outcome_row_count: usize,
    maximum_future_requests: usize,
    maximum_concurrency: usize,
    maximum_retries: usize,
    maximum_response_rows: usize,
    explicit_opening_authorization_required: bool,
    network_execution_allowed_this_sprint: bool,
    label_access_allowed_this_sprint: bool,
    reward_application_allowed: bool,
    provider_calls: usize,
    transport_constructions: usize,
    network_consent_reads: usize,
    credential_reads: usize,
    outcome_row_reads: usize,
    prospective_label_reads: usize,
    metric_computations: usize,
    reward_candidate_count: usize,
    reward_apply_count: usize,
    authority_action_count: usize,
    sprint65_artifacts_unchanged: bool,
}

#[derive(Serialize)]
struct ProspectiveEventMaturityPreflightReportV0 {
    report_version: &'static str,
    offline: bool,
    opening_registration_digest: String,
    opening_registration_reopened_and_verified: bool,
    event_count: usize,
    event_integrity_valid: bool,
    momentum_required_outcome_row_count: usize,
    risk_required_outcome_row_count: usize,
    required_outcome_row_count: usize,
    momentum_time_boundary_reached: bool,
    risk_time_boundary_reached: bool,
    momentum_opening_readiness: String,
    risk_opening_readiness: String,
    outcome_evidence_status: String,
    opening_readiness: String,
    label_open_count: usize,
    reward_eligibility: String,
    provider_calls: usize,
    transport_constructions: usize,
    network_consent_reads: usize,
    credential_reads: usize,
    outcome_row_reads: usize,
    prospective_label_reads: usize,
    metric_computations: usize,
    reward_candidate_count: usize,
    reward_apply_count: usize,
    penalties_applied: usize,
    chair_observed: bool,
    chair_decisions_created: usize,
    votes_created: usize,
    voice_changes: usize,
    cooldowns_started: usize,
    promotions_created: usize,
    quarantines_created: usize,
    risk_handoffs: usize,
    executions_created: usize,
    sprint65_artifacts_unchanged: bool,
}

fn prospective_opening_artifact_bytes(local_dir: &Path) -> Result<Vec<Vec<u8>>, String> {
    [
        "prospective_shadow_challenge_v0.json",
        "cycle_risk_prospective_local_state_v0.json",
        "prospective_external_row_admission_registration_v0.json",
        "prospective_external_row_capsule_v0.json",
        "prospective_public_export_acquisition_registration_v0.json",
        "prospective_public_export_acquisition_receipt_v0.json",
        "prospective_network_export_capsule_v0.json",
    ]
    .iter()
    .map(|name| {
        fs::read(local_dir.join(name))
            .map_err(|_| "prospective maturity immutable artifact unavailable".to_string())
    })
    .collect()
}

fn prospective_opening_context_v0(local_dir: &Path) -> Result<ProspectiveOpeningContextV0, String> {
    let momentum = crate::model::read_prospective_challenge_local_state_v0(
        &local_dir.join("prospective_shadow_challenge_v0.json"),
    )
    .map_err(|_| "prospective maturity Momentum journal unavailable")?;
    let risk = crate::model::read_cycle_risk_prospective_local_state_v0(
        &local_dir.join("cycle_risk_prospective_local_state_v0.json"),
    )
    .map_err(|_| "prospective maturity Cycle/Risk journal unavailable")?;
    let admission_registration = crate::model::read_prospective_external_admission_registration_v0(
        &local_dir.join("prospective_external_row_admission_registration_v0.json"),
    )
    .map_err(|_| "prospective maturity admission registration unavailable")?;
    let external_capsule = crate::model::read_prospective_external_row_capsule_v0(
        &local_dir.join("prospective_external_row_capsule_v0.json"),
    )
    .map_err(|_| "prospective maturity admitted evidence unavailable")?;
    let public_registration =
        crate::data::read_prospective_public_export_acquisition_registration_v0(
            &local_dir.join("prospective_public_export_acquisition_registration_v0.json"),
        )?;
    crate::data::validate_prospective_public_export_acquisition_registration_v0(
        &public_registration,
    )?;
    let receipt = crate::data::read_prospective_public_export_acquisition_receipt_v0(
        &local_dir.join("prospective_public_export_acquisition_receipt_v0.json"),
    )?;
    let network_capsule = crate::data::read_prospective_network_export_capsule_v0(
        &local_dir.join("prospective_network_export_capsule_v0.json"),
    )?;
    if !crate::data::verify_prospective_public_export_acquisition_receipt_v0(&receipt)
        || !crate::data::verify_prospective_network_export_capsule_v0(&network_capsule)
        || !receipt.request_attempted
        || receipt.request_count != 1
        || receipt.retry_count != 0
        || receipt.registration_digest != public_registration.registration_digest
        || receipt.capsule_digest.as_deref() != Some(network_capsule.capsule_digest.as_str())
        || network_capsule.acquisition_registration_digest
            != public_registration.registration_digest
        || network_capsule.acquisition_receipt_digest != receipt.receipt_digest
        || external_capsule.source_export_digest != network_capsule.capsule_digest
        || crate::data::convert_prospective_network_export_to_external_row_capsule_v0(
            &network_capsule,
            &admission_registration,
        )? != external_capsule
    {
        return Err("prospective maturity Sprint 65 acquisition chain invalid".into());
    }
    let audit = crate::model::audit_sealed_prospective_events_v0(
        &admission_registration,
        &external_capsule,
        &momentum,
        &risk,
    )?;
    let outcome_source_policy_digest = crate::core::stable_hash_string(&format!(
        "prospective-one-time-outcome-source-v0:{}:{}:{}:{}:{}:{}:{}:{}:{}",
        public_registration.registration_digest,
        public_registration.provider_id,
        public_registration.endpoint_origin,
        public_registration.endpoint_path,
        public_registration.configured_market,
        public_registration.cadence,
        public_registration.maximum_requests,
        public_registration.maximum_concurrency,
        public_registration.retry_count,
    ));
    let finalization_policy_digest = crate::core::stable_hash_string(
        "prospective-one-time-outcome-finalization-v0:utc-daily:exclusive-to:complete:contiguous:no-duplicates:no-extra-later-rows",
    );
    let metric_policy_digests = vec![
        crate::core::stable_hash_string(&format!(
            "prospective-opening-metric-policy-v0:{:?}",
            momentum.capsule.evaluation_policy
        )),
        crate::core::stable_hash_string(&format!(
            "prospective-opening-metric-policy-v0:{:?}",
            risk.capsule.evaluation_policy
        )),
    ];
    let (registration, plans) = crate::model::pre_register_prospective_one_time_opening_v0(
        &audit.momentum_event,
        &audit.risk_event,
        momentum.capsule.prediction_horizon,
        risk.capsule.prediction_horizon,
        &momentum.capsule.label_policy_digest,
        &risk.capsule.label_policy_digest,
        &outcome_source_policy_digest,
        &finalization_policy_digest,
        metric_policy_digests,
    )?;
    let challenge_valid = momentum
        .registry
        .challenges
        .first()
        .is_some_and(|entry| !entry.evaluation_opened)
        && risk
            .registry
            .entries
            .first()
            .is_some_and(|entry| !entry.evaluation_opened && !entry.invalidated);
    let label_open_count = momentum
        .journal
        .events
        .iter()
        .filter(|event| {
            event.label_status == crate::model::ProspectiveLabelStatusV0::OpenedForOneTimeEvaluation
        })
        .count()
        + usize::from(risk.vault.opened || risk.journal.evaluation_performed);
    Ok(ProspectiveOpeningContextV0 {
        registration,
        plans,
        public_registration,
        momentum,
        risk,
        external_capsule,
        event_audit: audit,
        outcome_receipt: crate::data::read_prospective_outcome_acquisition_receipt_v0(
            &local_dir.join("prospective_outcome_acquisition_receipt_v0.json"),
        )?,
        outcome_capsule: crate::data::read_prospective_outcome_evidence_capsule_v0(
            &local_dir.join("prospective_outcome_evidence_capsule_v0.json"),
        )?,
        event_integrity_valid: true,
        challenge_valid,
        expected_series_id: admission_registration.canonical_series_id,
        label_open_count,
    })
}

#[derive(Serialize)]
struct ProspectiveOutcomeOpeningCliReportV0 {
    report_version: &'static str,
    mode: String,
    offline: bool,
    status: String,
    readiness: String,
    evidence_status: String,
    opening_registration_digest: String,
    authorization_digest: Option<String>,
    receipt_digest: Option<String>,
    outcome_capsule_digest: String,
    opening_attempt_count: usize,
    opened_event_count: usize,
    outcome_digests: Vec<String>,
    attribution_classes: Vec<String>,
    reward_eligibility: Vec<String>,
    reward_candidate_present: bool,
    duplicate_execution_rejected: bool,
    provider_calls: usize,
    transport_constructions: usize,
    network_consent_reads: usize,
    credential_reads: usize,
    prospective_label_reads: usize,
    reward_apply_count: usize,
    penalty_apply_count: usize,
    voice_mutation_count: usize,
    chair_decisions: usize,
    votes: usize,
    promotions: usize,
    executions: usize,
    protected_artifacts_unchanged: bool,
}

fn print_prospective_outcome_opening_report_v0(
    report: &ProspectiveOutcomeOpeningCliReportV0,
    output_format: &str,
) -> Result<(), String> {
    match output_format {
        "json" => println!(
            "{}",
            serde_json::to_string(report)
                .map_err(|_| "prospective opening report serialization failed")?
        ),
        "text" => {
            println!("report_version={}", report.report_version);
            println!("mode={}", report.mode);
            println!("offline={}", report.offline);
            println!("status={}", report.status);
            println!("readiness={}", report.readiness);
            println!("evidence_status={}", report.evidence_status);
            println!(
                "opening_registration_digest={}",
                report.opening_registration_digest
            );
            println!(
                "authorization_digest={}",
                report.authorization_digest.as_deref().unwrap_or_default()
            );
            println!(
                "receipt_digest={}",
                report.receipt_digest.as_deref().unwrap_or_default()
            );
            println!("outcome_capsule_digest={}", report.outcome_capsule_digest);
            println!("opening_attempt_count={}", report.opening_attempt_count);
            println!("opened_event_count={}", report.opened_event_count);
            println!("outcome_digests={}", report.outcome_digests.join(":"));
            println!(
                "attribution_classes={}",
                report.attribution_classes.join(":")
            );
            println!("reward_eligibility={}", report.reward_eligibility.join(":"));
            println!(
                "reward_candidate_present={}",
                report.reward_candidate_present
            );
            println!(
                "duplicate_execution_rejected={}",
                report.duplicate_execution_rejected
            );
            println!("provider_calls={}", report.provider_calls);
            println!("transport_constructions={}", report.transport_constructions);
            println!("network_consent_reads={}", report.network_consent_reads);
            println!("credential_reads={}", report.credential_reads);
            println!("prospective_label_reads={}", report.prospective_label_reads);
            println!("reward_apply_count={}", report.reward_apply_count);
            println!("penalty_apply_count={}", report.penalty_apply_count);
            println!("voice_mutation_count={}", report.voice_mutation_count);
            println!("chair_decisions={}", report.chair_decisions);
            println!("votes={}", report.votes);
            println!("promotions={}", report.promotions);
            println!("executions={}", report.executions);
            println!(
                "protected_artifacts_unchanged={}",
                report.protected_artifacts_unchanged
            );
        }
        _ => return Err("unsupported prospective opening output format".into()),
    }
    Ok(())
}

fn load_selected_historical_snapshot_v0(
    config_path: &Path,
) -> Result<crate::data::DataSnapshot, String> {
    let config = crate::data::UpbitHistoricalPilotConfigV0::from_toml_path(config_path)?;
    config.validate()?;
    let mut snapshots = fs::read_dir(&config.snapshot_output_dir)
        .map_err(|_| "local snapshot directory unavailable")?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "pb"))
        .map(|path| crate::data::read_local_snapshot_protobuf_v1(&path))
        .collect::<Result<Vec<_>, _>>()?;
    snapshots.sort_by(|left, right| {
        historical_snapshot_selection_rank(right)
            .cmp(&historical_snapshot_selection_rank(left))
            .then_with(|| right.row_count.cmp(&left.row_count))
            .then_with(|| left.fetched_at_ms.cmp(&right.fetched_at_ms))
            .then_with(|| left.snapshot_id.cmp(&right.snapshot_id))
    });
    snapshots
        .into_iter()
        .next()
        .ok_or_else(|| "local historical campaign requires a snapshot".into())
}

fn prospective_opening_reward_contracts_v0(
    context: &ProspectiveOpeningContextV0,
    risk_report: &crate::model::CycleRiskShadowReportV0,
) -> Result<
    (
        crate::model::LearnedProspectiveContractV0,
        crate::model::LearnedProspectiveContractV0,
        crate::model::LearnedRewardSampleGateV0,
        crate::model::LearnedRewardEligibilityRegistrationV0,
    ),
    String,
> {
    let momentum_horizon = crate::core::stable_hash_string(&format!(
        "momentum-horizon-v0:{}:{}",
        context.momentum.capsule.prediction_horizon, context.momentum.capsule.label_policy_digest
    ));
    let risk_horizon = crate::core::stable_hash_string(&format!(
        "cycle-risk-horizon-v0:{}:{}",
        context.risk.capsule.prediction_horizon, context.risk.capsule.label_policy_digest
    ));
    let momentum_contract = crate::model::new_learned_prospective_contract_v0(
        crate::model::LearnedAgentObjectiveV0::DirectionalMomentum,
        context.momentum.capsule.capsule_digest.clone(),
        context.momentum.capsule.candidate.artifact_digest.clone(),
        momentum_horizon,
        context
            .momentum
            .capsule
            .prospective_cutoff_exclusive_timestamp_ms,
    )?;
    let risk_contract = crate::model::new_learned_prospective_contract_v0(
        crate::model::LearnedAgentObjectiveV0::DownsideRisk,
        context.risk.capsule.capsule_digest.clone(),
        context
            .risk
            .capsule
            .historical_champion
            .artifact_digest
            .clone(),
        risk_horizon,
        context.risk.capsule.cutoff_exclusive_timestamp_ms,
    )?;
    let gate = crate::model::new_learned_reward_sample_gate_v0(
        context
            .momentum
            .capsule
            .evaluation_policy
            .minimum_mature_events
            .max(context.risk.capsule.evaluation_policy.minimum_mature_events),
        context
            .momentum
            .capsule
            .evaluation_policy
            .minimum_support_qualified_events
            .max(
                context
                    .risk
                    .capsule
                    .evaluation_policy
                    .minimum_support_qualified_events,
            ),
        risk_report.regimes.len(),
    )?;
    let registration = crate::model::pre_register_learned_reward_eligibility_v0(
        &crate::model::LearnedRewardEligibilityRegistrationInputV0 {
            momentum: momentum_contract.clone(),
            cycle_risk: risk_contract.clone(),
            attribution_policy_digest: crate::core::stable_hash_string(
                "learned-prospective-attribution-policy-v0",
            ),
            maturity_policy_digest: crate::core::stable_hash_string(&format!(
                "learned-outcome-maturity-policy-v0:{}:{}:{}:{}",
                context
                    .momentum
                    .capsule
                    .opening_policy
                    .minimum_mature_events,
                context
                    .momentum
                    .capsule
                    .opening_policy
                    .minimum_support_qualified_events,
                context.risk.capsule.opening_policy.minimum_mature_events,
                context
                    .risk
                    .capsule
                    .opening_policy
                    .minimum_support_qualified_events,
            )),
            sample_gate_policy_digest: gate.gate_digest.clone(),
            objective_mapping_policy_digest: crate::core::stable_hash_string(
                "learned-objective-mapping-policy-v0",
            ),
            integrity_policy_digest: crate::core::stable_hash_string(
                "learned-prospective-integrity-policy-v0",
            ),
        },
    )?;
    Ok((momentum_contract, risk_contract, gate, registration))
}

fn prospective_opening_protected_bytes_v0(local_dir: &Path) -> Result<Vec<Vec<u8>>, String> {
    let mut bytes = prospective_outcome_acquisition_protected_bytes(local_dir)?;
    for name in [
        "prospective_outcome_acquisition_receipt_v0.json",
        "prospective_outcome_evidence_capsule_v0.json",
    ] {
        bytes.push(
            fs::read(local_dir.join(name))
                .map_err(|_| "prospective opening protected artifact unavailable")?,
        );
    }
    Ok(bytes)
}

fn replay_persisted_reward_eligibility_v1(
    config_path: &Path,
) -> Result<PersistedRewardEligibilityReplayCliV1, String> {
    let local_dir = config_path
        .parent()
        .ok_or("reward eligibility replay local directory unavailable")?;
    let context = prospective_opening_context_v0(local_dir)?;
    let registration = crate::model::read_prospective_one_time_opening_registration_v0(
        &local_dir.join("prospective_one_time_opening_registration_v0.json"),
    )?;
    crate::model::validate_prospective_one_time_opening_registration_v0(
        &registration,
        &context.plans,
    )?;
    if registration != context.registration {
        return Err("reward eligibility replay registration mismatch".into());
    }
    let acquisition_plan = crate::data::build_prospective_outcome_acquisition_plan_v0(
        &registration,
        &context.plans,
        &context.public_registration,
        crate::model::ProspectiveOutcomeRequestReadinessV0::ReadyForExplicitRequest,
    )?;
    crate::data::validate_prospective_outcome_evidence_capsule_for_plan_v0(
        &context.outcome_capsule,
        &acquisition_plan,
        &context.expected_series_id,
    )?;
    let snapshot = load_selected_historical_snapshot_v0(config_path)?;
    let risk_config = crate::model::CycleRiskShadowConfigV0::default();
    let risk_report = crate::model::run_cycle_risk_shadow_v0(&snapshot, &risk_config)
        .map_err(|_| "reward eligibility replay risk policy rebuild failed")?;
    let momentum_config = crate::model::MomentumLearningCampaignConfigV0::default();
    let sequence = &momentum_config.sequence_config;
    let momentum_label_policy_digest = crate::core::stable_hash_string(&format!(
        "{}:{}:{}:{}:{}",
        sequence.sequence_length,
        sequence.prediction_horizon,
        sequence.label_dead_zone.to_bits(),
        sequence.stride,
        sequence.include_neutral_labels,
    ));
    if momentum_label_policy_digest != context.momentum.capsule.label_policy_digest {
        return Err("reward eligibility replay momentum policy mismatch".into());
    }
    let risk_threshold_bits = risk_report
        .regimes
        .iter()
        .map(|regime| regime.checkpoint.threshold.to_bits())
        .collect::<Vec<_>>();
    let (momentum_contract, risk_contract, reward_gate, reward_registration) =
        prospective_opening_reward_contracts_v0(&context, &risk_report)?;
    let bundle_path = crate::model::default_private_learning_root_v0()
        .join("prospective_opening_v0/opening-bundle-v0.pb");
    let persisted = crate::model::read_prospective_outcome_opening_bundle_v0(&bundle_path)?;
    crate::model::validate_prospective_outcome_opening_bundle_v0(&persisted)?;
    let authorization = persisted
        .authorization
        .as_ref()
        .ok_or("reward eligibility replay authorization unavailable")?;
    let input = crate::model::ProspectiveOutcomeOpeningInputV0 {
        registration: &registration,
        plans: &context.plans,
        acquisition_receipt: &context.outcome_receipt,
        outcome_capsule: &context.outcome_capsule,
        expected_series_id: &context.expected_series_id,
        momentum_event: &context.event_audit.momentum_event,
        risk_event: &context.event_audit.risk_event,
        prospective_source_row: &context.external_capsule.row,
        momentum_contract: &momentum_contract,
        risk_contract: &risk_contract,
        reward_registration: &reward_registration,
        reward_gate: &reward_gate,
        momentum_label_dead_zone_bits: sequence.label_dead_zone.to_bits(),
        risk_threshold_bits: &risk_threshold_bits,
        observed_timestamp: current_utc_timestamp_ms(),
        challenge_valid: context.challenge_valid,
        opening_attempt_count_before: 0,
        opened_event_count_before: context.label_open_count,
    };
    let replayed =
        crate::model::derive_prospective_outcome_opening_bundle_v0(&input, Some(authorization));
    crate::model::validate_prospective_outcome_opening_bundle_v0(&replayed)?;
    if replayed != persisted {
        return Err("reward eligibility replay differs from persisted opening".into());
    }
    Ok(PersistedRewardEligibilityReplayCliV1 {
        opening_status: replayed.receipt.status,
        opening_attempt_count: replayed.receipt.opening_attempt_count,
        opened_event_count: replayed.receipt.opened_event_count,
        outcome_digests: replayed
            .outcomes
            .iter()
            .map(|outcome| outcome.outcome_digest.clone())
            .collect(),
        attribution_classes: replayed
            .outcomes
            .iter()
            .map(|outcome| outcome.attribution_class)
            .collect(),
        eligibility_statuses: replayed
            .outcomes
            .iter()
            .map(|outcome| outcome.eligibility_status)
            .collect(),
        eligibility_digests: replayed
            .outcomes
            .iter()
            .map(|outcome| outcome.eligibility_digest.clone())
            .collect(),
        reward_candidate_count: replayed.reward_candidate_count,
        reward_apply_count: replayed.reward_apply_count,
        penalty_apply_count: replayed.penalty_apply_count,
        voice_mutation_count: replayed.voice_mutation_count,
        authority_action_count: replayed.authority_action_count,
        replay_matches_persisted: true,
    })
}

fn run_prospective_outcome_opening_v0(
    config_path: &Path,
    output_format: &str,
    status_mode: bool,
    dry_run: bool,
    execute_local: bool,
    allow_network: bool,
    confirm_one_time_prospective_opening: bool,
) -> Result<(), String> {
    if usize::from(status_mode) + usize::from(dry_run) + usize::from(execute_local) != 1 {
        return Err("select exactly one prospective opening mode".into());
    }
    if allow_network {
        return Err("prospective outcome opening is offline-only".into());
    }
    if confirm_one_time_prospective_opening != execute_local {
        return Err(
            "execute-local prospective opening requires its one-time confirmation only".into(),
        );
    }
    if output_format != "text" && output_format != "json" {
        return Err("unsupported prospective opening output format".into());
    }
    let mode = if status_mode {
        "status"
    } else if dry_run {
        "dry-run"
    } else {
        "execute-local"
    };
    let local_dir = config_path
        .parent()
        .ok_or("prospective opening local directory unavailable")?;
    let protected_before = prospective_opening_protected_bytes_v0(local_dir)?;
    let context = prospective_opening_context_v0(local_dir)?;
    let registration = crate::model::read_prospective_one_time_opening_registration_v0(
        &local_dir.join("prospective_one_time_opening_registration_v0.json"),
    )?;
    crate::model::validate_prospective_one_time_opening_registration_v0(
        &registration,
        &context.plans,
    )?;
    if registration != context.registration {
        return Err("prospective opening registration mismatch".into());
    }
    let acquisition_plan = crate::data::build_prospective_outcome_acquisition_plan_v0(
        &registration,
        &context.plans,
        &context.public_registration,
        crate::model::ProspectiveOutcomeRequestReadinessV0::ReadyForExplicitRequest,
    )?;
    crate::data::validate_prospective_outcome_evidence_capsule_for_plan_v0(
        &context.outcome_capsule,
        &acquisition_plan,
        &context.expected_series_id,
    )?;
    let snapshot = load_selected_historical_snapshot_v0(config_path)?;
    let risk_config = crate::model::CycleRiskShadowConfigV0::default();
    let risk_report = crate::model::run_cycle_risk_shadow_v0(&snapshot, &risk_config)
        .map_err(|_| "prospective opening frozen risk policy rebuild failed")?;
    let rebuilt_risk_capsule = crate::model::prepare_cycle_risk_prospective_tournament_v0(
        &snapshot,
        &risk_report,
        &risk_config,
    )
    .map_err(|_| "prospective opening frozen risk capsule rebuild failed")?;
    if rebuilt_risk_capsule != context.risk.capsule
        || risk_config.label.digest() != context.risk.capsule.label_policy_digest
    {
        return Err("prospective opening frozen risk policy mismatch".into());
    }
    let momentum_config = crate::model::MomentumLearningCampaignConfigV0::default();
    let sequence = &momentum_config.sequence_config;
    let momentum_label_policy_digest = crate::core::stable_hash_string(&format!(
        "{}:{}:{}:{}:{}",
        sequence.sequence_length,
        sequence.prediction_horizon,
        sequence.label_dead_zone.to_bits(),
        sequence.stride,
        sequence.include_neutral_labels,
    ));
    if momentum_label_policy_digest != context.momentum.capsule.label_policy_digest {
        return Err("prospective opening frozen momentum policy mismatch".into());
    }
    let risk_threshold_bits = risk_report
        .regimes
        .iter()
        .map(|regime| regime.checkpoint.threshold.to_bits())
        .collect::<Vec<_>>();
    let (momentum_contract, risk_contract, reward_gate, reward_registration) =
        prospective_opening_reward_contracts_v0(&context, &risk_report)?;
    let opening_root =
        crate::model::default_private_learning_root_v0().join("prospective_opening_v0");
    let authorization_path = opening_root.join("opening-authorization-v0.pb");
    let bundle_path = opening_root.join("opening-bundle-v0.pb");
    let existing_bundle = if bundle_path.is_file() {
        Some(crate::model::read_prospective_outcome_opening_bundle_v0(
            &bundle_path,
        )?)
    } else {
        None
    };
    let input = crate::model::ProspectiveOutcomeOpeningInputV0 {
        registration: &registration,
        plans: &context.plans,
        acquisition_receipt: &context.outcome_receipt,
        outcome_capsule: &context.outcome_capsule,
        expected_series_id: &context.expected_series_id,
        momentum_event: &context.event_audit.momentum_event,
        risk_event: &context.event_audit.risk_event,
        prospective_source_row: &context.external_capsule.row,
        momentum_contract: &momentum_contract,
        risk_contract: &risk_contract,
        reward_registration: &reward_registration,
        reward_gate: &reward_gate,
        momentum_label_dead_zone_bits: sequence.label_dead_zone.to_bits(),
        risk_threshold_bits: &risk_threshold_bits,
        observed_timestamp: current_utc_timestamp_ms(),
        challenge_valid: context.challenge_valid,
        opening_attempt_count_before: 0,
        opened_event_count_before: context.label_open_count,
    };
    let preflight = crate::model::derive_prospective_outcome_opening_preflight_v0(&input)?;
    let mut duplicate_execution_rejected = false;
    let bundle = if let Some(existing) = existing_bundle {
        if existing.receipt.opening_registration_digest != registration.registration_digest
            || existing.receipt.outcome_capsule_digest != context.outcome_capsule.capsule_digest
        {
            return Err("prospective opening stored bundle identity mismatch".into());
        }
        duplicate_execution_rejected = execute_local;
        Some(existing)
    } else if execute_local {
        let authorization = crate::model::authorize_prospective_outcome_opening_v0(&input, true)?;
        crate::model::write_and_verify_prospective_outcome_opening_authorization_v0(
            &authorization_path,
            &authorization,
        )?;
        let reopened =
            crate::model::read_prospective_outcome_opening_authorization_v0(&authorization_path)?;
        crate::model::validate_prospective_outcome_opening_authorization_v0(&reopened, &input)?;
        let derived =
            crate::model::derive_prospective_outcome_opening_bundle_v0(&input, Some(&reopened));
        crate::model::write_and_verify_prospective_outcome_opening_bundle_v0(
            &bundle_path,
            &derived,
        )?;
        let reopened_bundle =
            crate::model::read_prospective_outcome_opening_bundle_v0(&bundle_path)?;
        if reopened_bundle != derived {
            return Err("prospective opening bundle reopen mismatch".into());
        }
        Some(reopened_bundle)
    } else {
        None
    };
    let protected_artifacts_unchanged =
        protected_before == prospective_opening_protected_bytes_v0(local_dir)?;
    if !protected_artifacts_unchanged {
        return Err("prospective opening protected artifact mismatch".into());
    }
    let (
        status,
        authorization_digest,
        receipt_digest,
        opening_attempt_count,
        opened_event_count,
        outcome_digests,
        attribution_classes,
        reward_eligibility,
        reward_candidate_present,
        reward_apply_count,
        penalty_apply_count,
        voice_mutation_count,
        authority_action_count,
    ) = if let Some(bundle) = bundle.as_ref() {
        (
            if duplicate_execution_rejected {
                format!(
                    "{:?}",
                    crate::model::ProspectiveOutcomeOpeningStatusV0::AlreadyOpened
                )
            } else {
                format!("{:?}", bundle.receipt.status)
            },
            bundle
                .authorization
                .as_ref()
                .map(|value| value.authorization_digest.clone()),
            Some(bundle.receipt.receipt_digest.clone()),
            bundle.receipt.opening_attempt_count,
            bundle.receipt.opened_event_count,
            bundle
                .outcomes
                .iter()
                .map(|outcome| outcome.outcome_digest.clone())
                .collect(),
            bundle
                .outcomes
                .iter()
                .map(|outcome| format!("{:?}", outcome.attribution_class))
                .collect(),
            bundle
                .outcomes
                .iter()
                .map(|outcome| format!("{:?}", outcome.eligibility_status))
                .collect(),
            bundle.reward_candidate_count > 0,
            bundle.reward_apply_count,
            bundle.penalty_apply_count,
            bundle.voice_mutation_count,
            bundle.authority_action_count,
        )
    } else {
        (
            "ReadyForExplicitOpening".into(),
            None,
            None,
            0,
            0,
            vec![],
            vec![],
            vec![],
            false,
            0,
            0,
            0,
            0,
        )
    };
    print_prospective_outcome_opening_report_v0(
        &ProspectiveOutcomeOpeningCliReportV0 {
            report_version: "prospective-outcome-opening-cli-report-v0",
            mode: mode.into(),
            offline: true,
            status,
            readiness: format!("{:?}", preflight.readiness),
            evidence_status: format!("{:?}", preflight.evidence_status),
            opening_registration_digest: registration.registration_digest,
            authorization_digest,
            receipt_digest,
            outcome_capsule_digest: context.outcome_capsule.capsule_digest,
            opening_attempt_count,
            opened_event_count,
            outcome_digests,
            attribution_classes,
            reward_eligibility,
            reward_candidate_present,
            duplicate_execution_rejected,
            provider_calls: 0,
            transport_constructions: 0,
            network_consent_reads: 0,
            credential_reads: 0,
            prospective_label_reads: opened_event_count,
            reward_apply_count,
            penalty_apply_count,
            voice_mutation_count,
            chair_decisions: 0,
            votes: 0,
            promotions: 0,
            executions: 0,
            protected_artifacts_unchanged,
        },
        output_format,
    )?;
    if authority_action_count != 0 {
        return Err("prospective opening authority mutation rejected".into());
    }
    Ok(())
}

fn print_prospective_opening_registration_report(
    report: &ProspectiveOpeningRegistrationReportV0,
    output_format: &str,
) -> Result<(), String> {
    match output_format {
        "json" => println!(
            "{}",
            serde_json::to_string(report)
                .map_err(|_| "prospective opening registration report serialization failed")?
        ),
        "text" => {
            println!("report_version={}", report.report_version);
            println!("offline={}", report.offline);
            println!(
                "opening_registration_digest={}",
                report.opening_registration_digest
            );
            println!(
                "opening_registration_reopened_and_verified={}",
                report.opening_registration_reopened_and_verified
            );
            println!("event_count={}", report.event_count);
            println!("event_integrity_valid={}", report.event_integrity_valid);
            println!(
                "required_outcome_row_count={}",
                report.required_outcome_row_count
            );
            println!("maximum_future_requests={}", report.maximum_future_requests);
            println!("maximum_concurrency={}", report.maximum_concurrency);
            println!("maximum_retries={}", report.maximum_retries);
            println!("maximum_response_rows={}", report.maximum_response_rows);
            println!(
                "explicit_opening_authorization_required={}",
                report.explicit_opening_authorization_required
            );
            println!(
                "network_execution_allowed_this_sprint={}",
                report.network_execution_allowed_this_sprint
            );
            println!(
                "label_access_allowed_this_sprint={}",
                report.label_access_allowed_this_sprint
            );
            println!(
                "reward_application_allowed={}",
                report.reward_application_allowed
            );
            println!("provider_calls={}", report.provider_calls);
            println!("transport_constructions={}", report.transport_constructions);
            println!("network_consent_reads={}", report.network_consent_reads);
            println!("credential_reads={}", report.credential_reads);
            println!("outcome_row_reads={}", report.outcome_row_reads);
            println!("prospective_label_reads={}", report.prospective_label_reads);
            println!("metric_computations={}", report.metric_computations);
            println!("reward_candidate_count={}", report.reward_candidate_count);
            println!("reward_apply_count={}", report.reward_apply_count);
            println!("authority_action_count={}", report.authority_action_count);
            println!(
                "sprint65_artifacts_unchanged={}",
                report.sprint65_artifacts_unchanged
            );
        }
        _ => return Err("unsupported prospective opening registration output format".into()),
    }
    Ok(())
}

fn print_prospective_event_maturity_preflight_report(
    report: &ProspectiveEventMaturityPreflightReportV0,
    output_format: &str,
) -> Result<(), String> {
    match output_format {
        "json" => println!(
            "{}",
            serde_json::to_string(report)
                .map_err(|_| "prospective maturity preflight report serialization failed")?
        ),
        "text" => {
            println!("report_version={}", report.report_version);
            println!("offline={}", report.offline);
            println!(
                "opening_registration_digest={}",
                report.opening_registration_digest
            );
            println!(
                "opening_registration_reopened_and_verified={}",
                report.opening_registration_reopened_and_verified
            );
            println!("event_count={}", report.event_count);
            println!("event_integrity_valid={}", report.event_integrity_valid);
            println!(
                "momentum_required_outcome_row_count={}",
                report.momentum_required_outcome_row_count
            );
            println!(
                "risk_required_outcome_row_count={}",
                report.risk_required_outcome_row_count
            );
            println!(
                "required_outcome_row_count={}",
                report.required_outcome_row_count
            );
            println!(
                "momentum_time_boundary_reached={}",
                report.momentum_time_boundary_reached
            );
            println!(
                "risk_time_boundary_reached={}",
                report.risk_time_boundary_reached
            );
            println!(
                "momentum_opening_readiness={}",
                report.momentum_opening_readiness
            );
            println!("risk_opening_readiness={}", report.risk_opening_readiness);
            println!("outcome_evidence_status={}", report.outcome_evidence_status);
            println!("opening_readiness={}", report.opening_readiness);
            println!("label_open_count={}", report.label_open_count);
            println!("reward_eligibility={}", report.reward_eligibility);
            println!("provider_calls={}", report.provider_calls);
            println!("transport_constructions={}", report.transport_constructions);
            println!("network_consent_reads={}", report.network_consent_reads);
            println!("credential_reads={}", report.credential_reads);
            println!("outcome_row_reads={}", report.outcome_row_reads);
            println!("prospective_label_reads={}", report.prospective_label_reads);
            println!("metric_computations={}", report.metric_computations);
            println!("reward_candidate_count={}", report.reward_candidate_count);
            println!("reward_apply_count={}", report.reward_apply_count);
            println!("penalties_applied={}", report.penalties_applied);
            println!("chair_observed={}", report.chair_observed);
            println!("chair_decisions_created={}", report.chair_decisions_created);
            println!("votes_created={}", report.votes_created);
            println!("voice_changes={}", report.voice_changes);
            println!("cooldowns_started={}", report.cooldowns_started);
            println!("promotions_created={}", report.promotions_created);
            println!("quarantines_created={}", report.quarantines_created);
            println!("risk_handoffs={}", report.risk_handoffs);
            println!("executions_created={}", report.executions_created);
            println!(
                "sprint65_artifacts_unchanged={}",
                report.sprint65_artifacts_unchanged
            );
        }
        _ => return Err("unsupported prospective maturity preflight output format".into()),
    }
    Ok(())
}

fn run_prospective_outcome_opening_registration(
    config_path: &Path,
    output_format: &str,
    allow_network: bool,
) -> Result<(), String> {
    if allow_network {
        return Err("prospective outcome opening registration is offline-only".into());
    }
    let local_dir = config_path
        .parent()
        .ok_or("prospective opening registration directory unavailable")?;
    let before = prospective_opening_artifact_bytes(local_dir)?;
    let context = prospective_opening_context_v0(local_dir)?;
    let path = local_dir.join("prospective_one_time_opening_registration_v0.json");
    let registration = if path.is_file() {
        let existing = crate::model::read_prospective_one_time_opening_registration_v0(&path)?;
        crate::model::validate_prospective_one_time_opening_registration_v0(
            &existing,
            &context.plans,
        )?;
        if existing != context.registration {
            return Err("prospective opening registration mismatch".into());
        }
        existing
    } else {
        crate::model::write_prospective_one_time_opening_registration_v0(
            &path,
            &context.registration,
            &context.plans,
        )?;
        crate::model::read_prospective_one_time_opening_registration_v0(&path)?
    };
    crate::model::validate_prospective_one_time_opening_registration_v0(
        &registration,
        &context.plans,
    )?;
    if registration != context.registration
        || before != prospective_opening_artifact_bytes(local_dir)?
    {
        return Err("prospective opening registration immutable artifact mismatch".into());
    }
    print_prospective_opening_registration_report(
        &ProspectiveOpeningRegistrationReportV0 {
            report_version: "prospective-one-time-opening-registration-v0",
            offline: true,
            opening_registration_digest: registration.registration_digest,
            opening_registration_reopened_and_verified: true,
            event_count: context.plans.len(),
            event_integrity_valid: context.event_integrity_valid,
            required_outcome_row_count: registration.maximum_response_rows,
            maximum_future_requests: registration.maximum_future_requests,
            maximum_concurrency: registration.maximum_concurrency,
            maximum_retries: registration.maximum_retries,
            maximum_response_rows: registration.maximum_response_rows,
            explicit_opening_authorization_required: registration
                .explicit_opening_authorization_required,
            network_execution_allowed_this_sprint: registration
                .network_execution_allowed_this_sprint,
            label_access_allowed_this_sprint: registration.label_access_allowed_this_sprint,
            reward_application_allowed: registration.reward_application_allowed,
            provider_calls: 0,
            transport_constructions: 0,
            network_consent_reads: 0,
            credential_reads: 0,
            outcome_row_reads: 0,
            prospective_label_reads: 0,
            metric_computations: 0,
            reward_candidate_count: 0,
            reward_apply_count: 0,
            authority_action_count: 0,
            sprint65_artifacts_unchanged: true,
        },
        output_format,
    )
}

fn run_prospective_event_maturity_preflight(
    config_path: &Path,
    output_format: &str,
    allow_network: bool,
) -> Result<(), String> {
    if allow_network {
        return Err("prospective event maturity preflight is offline-only".into());
    }
    let local_dir = config_path
        .parent()
        .ok_or("prospective maturity preflight directory unavailable")?;
    let before = prospective_opening_artifact_bytes(local_dir)?;
    let context = prospective_opening_context_v0(local_dir)?;
    let registration = crate::model::read_prospective_one_time_opening_registration_v0(
        &local_dir.join("prospective_one_time_opening_registration_v0.json"),
    )?;
    crate::model::validate_prospective_one_time_opening_registration_v0(
        &registration,
        &context.plans,
    )?;
    if registration != context.registration {
        return Err("prospective maturity opening registration mismatch".into());
    }
    let evidence = crate::model::assess_prospective_outcome_evidence_v0(
        &context.plans,
        &context.expected_series_id,
        &[],
    );
    let observed_timestamp = current_utc_timestamp_ms();
    let readiness = context
        .plans
        .iter()
        .map(|plan| {
            crate::model::prospective_opening_readiness_v0(
                plan,
                observed_timestamp,
                evidence.status,
                context.event_integrity_valid,
                context.challenge_valid,
                context.label_open_count,
            )
        })
        .collect::<Vec<_>>();
    let opening_readiness = crate::model::aggregate_prospective_opening_readiness_v0(&readiness);
    if before != prospective_opening_artifact_bytes(local_dir)? {
        return Err("prospective maturity preflight immutable artifact mismatch".into());
    }
    print_prospective_event_maturity_preflight_report(
        &ProspectiveEventMaturityPreflightReportV0 {
            report_version: "prospective-event-maturity-preflight-v0",
            offline: true,
            opening_registration_digest: registration.registration_digest,
            opening_registration_reopened_and_verified: true,
            event_count: context.plans.len(),
            event_integrity_valid: context.event_integrity_valid,
            momentum_required_outcome_row_count: context.plans[0].required_finalized_row_count,
            risk_required_outcome_row_count: context.plans[1].required_finalized_row_count,
            required_outcome_row_count: evidence.required_finalized_row_count,
            momentum_time_boundary_reached: observed_timestamp
                >= context.plans[0].maturity_timestamp,
            risk_time_boundary_reached: observed_timestamp >= context.plans[1].maturity_timestamp,
            momentum_opening_readiness: format!("{:?}", readiness[0]),
            risk_opening_readiness: format!("{:?}", readiness[1]),
            outcome_evidence_status: format!("{:?}", evidence.status),
            opening_readiness: format!("{opening_readiness:?}"),
            label_open_count: context.label_open_count,
            reward_eligibility: format!(
                "{:?}",
                crate::model::external_admission_reward_eligibility_status_v0(context.plans.len())
            ),
            provider_calls: 0,
            transport_constructions: 0,
            network_consent_reads: 0,
            credential_reads: 0,
            outcome_row_reads: 0,
            prospective_label_reads: 0,
            metric_computations: 0,
            reward_candidate_count: 0,
            reward_apply_count: 0,
            penalties_applied: 0,
            chair_observed: false,
            chair_decisions_created: 0,
            votes_created: 0,
            voice_changes: 0,
            cooldowns_started: 0,
            promotions_created: 0,
            quarantines_created: 0,
            risk_handoffs: 0,
            executions_created: 0,
            sprint65_artifacts_unchanged: true,
        },
        output_format,
    )
}

#[derive(Serialize)]
struct ProspectiveOutcomeAcquisitionReportV0 {
    report_version: &'static str,
    mode: String,
    offline: bool,
    opening_registration_digest: String,
    opening_registration_reopened_and_verified: bool,
    momentum_event_digest: String,
    risk_event_digest: String,
    maturity_plan_digests: Vec<String>,
    event_integrity_valid: bool,
    readiness: String,
    request_readiness: String,
    status: String,
    blocked_event: Option<String>,
    required_timestamps: Vec<u64>,
    required_row_count: usize,
    request_to_utc: String,
    request_row_count: usize,
    request_count: usize,
    retry_count: usize,
    http_status_class: Option<String>,
    returned_row_count: usize,
    verified_row_count: usize,
    provider_id: String,
    market: String,
    cadence: String,
    maximum_requests: usize,
    maximum_retries: usize,
    maximum_concurrency: usize,
    plan_digest: String,
    request_fingerprint: Option<String>,
    prior_request_attempted: bool,
    receipt_created: bool,
    receipt_present: bool,
    outcome_evidence_present: bool,
    outcome_capsule_created: bool,
    outcome_capsule_present: bool,
    evidence_status: String,
    receipt_digest: Option<String>,
    outcome_capsule_digest: Option<String>,
    opening_readiness: String,
    network_requests: usize,
    network_request_count: usize,
    provider_calls: usize,
    transport_constructions: usize,
    network_consent_reads: usize,
    credential_reads: usize,
    outcome_row_reads: usize,
    outcome_rows_admitted: usize,
    prospective_label_reads: usize,
    labels_opened: usize,
    label_open_count: usize,
    metrics_computed: usize,
    metric_computations: usize,
    reward_candidates: usize,
    reward_candidates_created: usize,
    rewards_applied: usize,
    penalties_applied: usize,
    voice_changes: usize,
    cooldowns_started: usize,
    promotions_created: usize,
    quarantines_created: usize,
    chair_decisions_created: usize,
    votes_created: usize,
    risk_handoffs: usize,
    executions_created: usize,
    protected_artifacts_unchanged: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ProspectiveOutcomeReceiptReplayV0 {
    plan_digest: String,
    request_fingerprint: Option<String>,
    acquisition_status: crate::data::ProspectiveOutcomeAcquisitionStatusV0,
    request_count: usize,
    retry_count: usize,
    http_status_class: Option<String>,
    returned_row_count: usize,
    verified_row_count: usize,
}

fn prospective_outcome_receipt_replay_v0(
    plan: &crate::data::ProspectiveOutcomeAcquisitionPlanV0,
    status_mode: bool,
    fallback_status: crate::data::ProspectiveOutcomeAcquisitionStatusV0,
    receipt: Option<&crate::data::ProspectiveOutcomeAcquisitionReceiptV0>,
) -> Result<ProspectiveOutcomeReceiptReplayV0, String> {
    let request_fingerprint = if let Some(receipt) = receipt {
        Some(receipt.request_fingerprint.clone())
    } else if status_mode {
        None
    } else {
        Some(crate::data::prospective_outcome_request_fingerprint_v0(
            plan,
        )?)
    };
    Ok(ProspectiveOutcomeReceiptReplayV0 {
        plan_digest: receipt
            .map(|value| value.plan_digest.clone())
            .unwrap_or_else(|| plan.plan_digest.clone()),
        request_fingerprint,
        acquisition_status: receipt.map(|value| value.status).unwrap_or(fallback_status),
        request_count: receipt.map(|value| value.request_count).unwrap_or_default(),
        retry_count: receipt.map(|value| value.retry_count).unwrap_or_default(),
        http_status_class: receipt.and_then(|value| value.http_status_class.clone()),
        returned_row_count: receipt
            .map(|value| value.returned_row_count)
            .unwrap_or_default(),
        verified_row_count: receipt
            .map(|value| value.verified_row_count)
            .unwrap_or_default(),
    })
}

fn prospective_outcome_stored_result_chain_valid_v0(
    receipt: Option<&crate::data::ProspectiveOutcomeAcquisitionReceiptV0>,
    capsule: Option<&crate::data::ProspectiveOutcomeEvidenceCapsuleV0>,
) -> bool {
    match (receipt, capsule) {
        (None, None) => true,
        (Some(receipt), None) => receipt.outcome_capsule_digest.is_none(),
        (None, Some(_)) => false,
        (Some(receipt), Some(capsule)) => {
            receipt.status == crate::data::ProspectiveOutcomeAcquisitionStatusV0::EvidenceAcquired
                && receipt.outcome_capsule_digest.as_deref()
                    == Some(capsule.capsule_digest.as_str())
                && capsule.acquisition_receipt_digest == receipt.receipt_digest
                && receipt.returned_row_count == capsule.canonical_rows.len()
                && receipt.verified_row_count == capsule.canonical_rows.len()
        }
    }
}

fn prospective_outcome_acquisition_protected_bytes(
    local_dir: &Path,
) -> Result<Vec<Vec<u8>>, String> {
    let mut bytes = prospective_opening_artifact_bytes(local_dir)?;
    bytes.push(
        fs::read(local_dir.join("prospective_one_time_opening_registration_v0.json"))
            .map_err(|_| "prospective outcome opening registration unavailable")?,
    );
    Ok(bytes)
}

fn prospective_outcome_request_readiness_label(
    readiness: crate::model::ProspectiveOutcomeRequestReadinessV0,
) -> &'static str {
    use crate::model::ProspectiveOutcomeRequestReadinessV0 as Readiness;
    match readiness {
        Readiness::AwaitingMomentumTimeMaturity => "BlockedAwaitingMomentumTimeMaturity",
        Readiness::AwaitingRiskTimeMaturity => "BlockedAwaitingRiskTimeMaturity",
        Readiness::AwaitingBothTimeMaturities => "BlockedAwaitingBothTimeMaturities",
        Readiness::ReadyForExplicitRequest => "ReadyForExplicitRequest",
        Readiness::RequestAlreadyAttempted => "BlockedRequestAlreadyAttempted",
        Readiness::OutcomeEvidenceAlreadyPresent => "BlockedOutcomeEvidenceAlreadyPresent",
        Readiness::RegistrationInvalid => "BlockedRegistrationInvalid",
        Readiness::EventIntegrityInvalid => "BlockedEventIntegrityInvalid",
        Readiness::TechnicalFailure => "BlockedTechnicalFailure",
    }
}

fn prospective_outcome_status_from_readiness(
    readiness: crate::model::ProspectiveOutcomeRequestReadinessV0,
) -> crate::data::ProspectiveOutcomeAcquisitionStatusV0 {
    use crate::data::ProspectiveOutcomeAcquisitionStatusV0 as Status;
    use crate::model::ProspectiveOutcomeRequestReadinessV0 as Readiness;
    match readiness {
        Readiness::AwaitingMomentumTimeMaturity
        | Readiness::AwaitingRiskTimeMaturity
        | Readiness::AwaitingBothTimeMaturities => Status::NotAttemptedNotMature,
        Readiness::ReadyForExplicitRequest => Status::ReadyNotAttempted,
        Readiness::RequestAlreadyAttempted => Status::RequestBudgetExhausted,
        Readiness::OutcomeEvidenceAlreadyPresent => Status::EvidenceAcquired,
        Readiness::RegistrationInvalid | Readiness::EventIntegrityInvalid => {
            Status::IntegrityFailure
        }
        Readiness::TechnicalFailure => Status::TechnicalFailure,
    }
}

fn prospective_outcome_blocked_event(
    readiness: crate::model::ProspectiveOutcomeRequestReadinessV0,
) -> Option<String> {
    use crate::model::ProspectiveOutcomeRequestReadinessV0 as Readiness;
    match readiness {
        Readiness::AwaitingMomentumTimeMaturity => Some("Momentum".into()),
        Readiness::AwaitingRiskTimeMaturity => Some("Cycle/Risk".into()),
        Readiness::AwaitingBothTimeMaturities => Some("Momentum,Cycle/Risk".into()),
        _ => None,
    }
}

fn print_prospective_outcome_acquisition_report(
    report: &ProspectiveOutcomeAcquisitionReportV0,
    output_format: &str,
) -> Result<(), String> {
    match output_format {
        "json" => println!(
            "{}",
            serde_json::to_string(report)
                .map_err(|_| "prospective outcome report serialization failed")?
        ),
        "text" => {
            println!("report_version={}", report.report_version);
            println!("mode={}", report.mode);
            println!("offline={}", report.offline);
            println!(
                "opening_registration_digest={}",
                report.opening_registration_digest
            );
            println!(
                "opening_registration_reopened_and_verified={}",
                report.opening_registration_reopened_and_verified
            );
            println!("momentum_event_digest={}", report.momentum_event_digest);
            println!("risk_event_digest={}", report.risk_event_digest);
            println!(
                "maturity_plan_digests={}",
                report.maturity_plan_digests.join("|")
            );
            println!("event_integrity_valid={}", report.event_integrity_valid);
            println!("readiness={}", report.readiness);
            println!("request_readiness={}", report.request_readiness);
            println!("status={}", report.status);
            println!(
                "blocked_event={}",
                report.blocked_event.as_deref().unwrap_or_default()
            );
            println!(
                "required_timestamps={}",
                report
                    .required_timestamps
                    .iter()
                    .map(u64::to_string)
                    .collect::<Vec<_>>()
                    .join("|")
            );
            println!("required_row_count={}", report.required_row_count);
            println!("request_to_utc={}", report.request_to_utc);
            println!("request_row_count={}", report.request_row_count);
            println!("request_count={}", report.request_count);
            println!("retry_count={}", report.retry_count);
            println!(
                "http_status_class={}",
                report.http_status_class.as_deref().unwrap_or_default()
            );
            println!("returned_row_count={}", report.returned_row_count);
            println!("verified_row_count={}", report.verified_row_count);
            println!("provider_id={}", report.provider_id);
            println!("market={}", report.market);
            println!("cadence={}", report.cadence);
            println!("maximum_requests={}", report.maximum_requests);
            println!("maximum_retries={}", report.maximum_retries);
            println!("maximum_concurrency={}", report.maximum_concurrency);
            println!("plan_digest={}", report.plan_digest);
            println!(
                "request_fingerprint={}",
                report.request_fingerprint.as_deref().unwrap_or_default()
            );
            println!("prior_request_attempted={}", report.prior_request_attempted);
            println!("receipt_created={}", report.receipt_created);
            println!("receipt_present={}", report.receipt_present);
            println!(
                "outcome_evidence_present={}",
                report.outcome_evidence_present
            );
            println!("outcome_capsule_created={}", report.outcome_capsule_created);
            println!("outcome_capsule_present={}", report.outcome_capsule_present);
            println!("evidence_status={}", report.evidence_status);
            println!(
                "receipt_digest={}",
                report.receipt_digest.as_deref().unwrap_or_default()
            );
            println!(
                "outcome_capsule_digest={}",
                report.outcome_capsule_digest.as_deref().unwrap_or_default()
            );
            println!("opening_readiness={}", report.opening_readiness);
            println!("network_requests={}", report.network_requests);
            println!("network_request_count={}", report.network_request_count);
            println!("provider_calls={}", report.provider_calls);
            println!("transport_constructions={}", report.transport_constructions);
            println!("network_consent_reads={}", report.network_consent_reads);
            println!("credential_reads={}", report.credential_reads);
            println!("outcome_row_reads={}", report.outcome_row_reads);
            println!("outcome_rows_admitted={}", report.outcome_rows_admitted);
            println!("prospective_label_reads={}", report.prospective_label_reads);
            println!("labels_opened={}", report.labels_opened);
            println!("label_open_count={}", report.label_open_count);
            println!("metrics_computed={}", report.metrics_computed);
            println!("metric_computations={}", report.metric_computations);
            println!("reward_candidates={}", report.reward_candidates);
            println!(
                "reward_candidates_created={}",
                report.reward_candidates_created
            );
            println!("rewards_applied={}", report.rewards_applied);
            println!("penalties_applied={}", report.penalties_applied);
            println!("voice_changes={}", report.voice_changes);
            println!("cooldowns_started={}", report.cooldowns_started);
            println!("promotions_created={}", report.promotions_created);
            println!("quarantines_created={}", report.quarantines_created);
            println!("chair_decisions_created={}", report.chair_decisions_created);
            println!("votes_created={}", report.votes_created);
            println!("risk_handoffs={}", report.risk_handoffs);
            println!("executions_created={}", report.executions_created);
            println!(
                "protected_artifacts_unchanged={}",
                report.protected_artifacts_unchanged
            );
        }
        _ => return Err("unsupported prospective outcome acquisition output format".into()),
    }
    Ok(())
}

fn run_prospective_outcome_acquisition(
    config_path: &Path,
    output_format: &str,
    status_mode: bool,
    dry_run: bool,
    execute: bool,
    allow_network: bool,
    confirm_one_time_outcome_request: bool,
) -> Result<(), String> {
    if usize::from(status_mode) + usize::from(dry_run) + usize::from(execute) != 1 {
        return Err("select exactly one prospective outcome acquisition mode".into());
    }
    if !execute && (allow_network || confirm_one_time_outcome_request) {
        return Err("status and dry-run prospective outcome modes are offline-only".into());
    }
    let mode = if status_mode {
        "status"
    } else if dry_run {
        "dry-run"
    } else {
        "execute"
    };
    let local_dir = config_path
        .parent()
        .ok_or("prospective outcome acquisition directory unavailable")?;
    let protected_before = prospective_outcome_acquisition_protected_bytes(local_dir)?;
    let context = prospective_opening_context_v0(local_dir)?;
    let registration = crate::model::read_prospective_one_time_opening_registration_v0(
        &local_dir.join("prospective_one_time_opening_registration_v0.json"),
    )?;
    crate::model::validate_prospective_one_time_opening_registration_v0(
        &registration,
        &context.plans,
    )?;
    if registration != context.registration {
        return Err("prospective outcome opening registration mismatch".into());
    }
    let receipt_path = local_dir.join("prospective_outcome_acquisition_receipt_v0.json");
    let capsule_path = local_dir.join("prospective_outcome_evidence_capsule_v0.json");
    let raw_response_path = local_dir.join("prospective_outcome_response_v0.json");
    let existing_receipt = receipt_path
        .is_file()
        .then(|| crate::data::read_prospective_outcome_acquisition_receipt_v0(&receipt_path))
        .transpose()?;
    let existing_capsule = capsule_path
        .is_file()
        .then(|| crate::data::read_prospective_outcome_evidence_capsule_v0(&capsule_path))
        .transpose()?;
    let request_contract_plan = crate::data::build_prospective_outcome_acquisition_plan_v0(
        &registration,
        &context.plans,
        &context.public_registration,
        crate::model::ProspectiveOutcomeRequestReadinessV0::ReadyForExplicitRequest,
    )?;
    if existing_receipt.as_ref().is_some_and(|receipt| {
        receipt.opening_registration_digest != registration.registration_digest
            || receipt.plan_digest != request_contract_plan.plan_digest
            || crate::data::prospective_outcome_request_fingerprint_v0(&request_contract_plan)
                .ok()
                .as_deref()
                != Some(receipt.request_fingerprint.as_str())
    }) {
        return Err("prospective outcome stored receipt chain invalid".into());
    }
    if let Some(existing_capsule) = existing_capsule.as_ref() {
        crate::data::validate_prospective_outcome_evidence_capsule_for_plan_v0(
            existing_capsule,
            &request_contract_plan,
            &context.expected_series_id,
        )?;
    }
    if !prospective_outcome_stored_result_chain_valid_v0(
        existing_receipt.as_ref(),
        existing_capsule.as_ref(),
    ) {
        return Err("prospective outcome stored evidence chain invalid".into());
    }
    let observed_timestamp = current_utc_timestamp_ms();
    let derive_readiness = || {
        crate::model::prospective_outcome_request_readiness_v0(
            &registration,
            &context.plans,
            observed_timestamp,
            context.event_integrity_valid && context.challenge_valid,
            existing_receipt.is_some(),
            existing_capsule.is_some(),
        )
    };
    let readiness = derive_readiness();
    if readiness != derive_readiness() {
        return Err("prospective outcome readiness is nondeterministic".into());
    }
    let plan = crate::data::build_prospective_outcome_acquisition_plan_v0(
        &registration,
        &context.plans,
        &context.public_registration,
        readiness,
    )?;
    let mut acquisition_status = prospective_outcome_status_from_readiness(readiness);
    let mut attempted_this_run = false;
    let mut capsule_created_this_run = false;
    let mut receipt = existing_receipt.clone();
    let mut capsule = existing_capsule.clone();
    if execute {
        let result = crate::data::execute_prospective_outcome_acquisition_v0(
            &plan,
            existing_receipt.as_ref(),
            existing_capsule.as_ref(),
            &context.expected_series_id,
            allow_network,
            confirm_one_time_outcome_request,
            |request_plan| {
                crate::data::fetch_prospective_outcome_acquisition_v0(
                    &context.public_registration,
                    request_plan,
                )
            },
        );
        acquisition_status = result.status;
        attempted_this_run = result.receipt.is_some();
        if let Some(new_receipt) = result.receipt {
            crate::data::write_prospective_outcome_acquisition_receipt_v0(
                &receipt_path,
                &new_receipt,
            )?;
            receipt =
                Some(crate::data::read_prospective_outcome_acquisition_receipt_v0(&receipt_path)?);
        }
        if let Some(raw_response) = result.raw_response {
            let temporary = raw_response_path.with_extension("tmp");
            fs::write(&temporary, raw_response)
                .map_err(|_| "prospective outcome raw response storage failed")?;
            fs::rename(temporary, &raw_response_path)
                .map_err(|_| "prospective outcome raw response storage failed")?;
        }
        if let Some(new_capsule) = result.capsule {
            crate::data::write_prospective_outcome_evidence_capsule_v0(
                &capsule_path,
                &new_capsule,
            )?;
            capsule = Some(crate::data::read_prospective_outcome_evidence_capsule_v0(
                &capsule_path,
            )?);
            capsule_created_this_run = true;
        }
    }
    if let Some(stored_capsule) = capsule.as_ref() {
        crate::data::validate_prospective_outcome_evidence_capsule_for_plan_v0(
            stored_capsule,
            &request_contract_plan,
            &context.expected_series_id,
        )?;
    }
    if !prospective_outcome_stored_result_chain_valid_v0(receipt.as_ref(), capsule.as_ref()) {
        return Err("prospective outcome stored evidence chain invalid".into());
    }
    let receipt_replay = prospective_outcome_receipt_replay_v0(
        &plan,
        status_mode,
        acquisition_status,
        receipt.as_ref(),
    )?;
    acquisition_status = receipt_replay.acquisition_status;
    let protected_artifacts_unchanged =
        protected_before == prospective_outcome_acquisition_protected_bytes(local_dir)?;
    if !protected_artifacts_unchanged {
        return Err("prospective outcome protected artifact mismatch".into());
    }
    print_prospective_outcome_acquisition_report(
        &ProspectiveOutcomeAcquisitionReportV0 {
            report_version: "prospective-outcome-acquisition-report-v0",
            mode: mode.into(),
            offline: !attempted_this_run,
            opening_registration_digest: registration.registration_digest.clone(),
            opening_registration_reopened_and_verified: true,
            momentum_event_digest: registration.momentum_event_digest.clone(),
            risk_event_digest: registration.risk_event_digest.clone(),
            maturity_plan_digests: registration.maturity_plan_digests.clone(),
            event_integrity_valid: context.event_integrity_valid,
            readiness: format!("{readiness:?}"),
            request_readiness: prospective_outcome_request_readiness_label(readiness).into(),
            status: format!("{acquisition_status:?}"),
            blocked_event: prospective_outcome_blocked_event(readiness),
            required_timestamps: plan.required_timestamps,
            required_row_count: plan.required_row_count,
            request_to_utc: plan.request_to_utc,
            request_row_count: plan.request_count,
            request_count: receipt_replay.request_count,
            retry_count: receipt_replay.retry_count,
            http_status_class: receipt_replay.http_status_class,
            returned_row_count: receipt_replay.returned_row_count,
            verified_row_count: receipt_replay.verified_row_count,
            provider_id: plan.provider_id,
            market: plan.market,
            cadence: plan.cadence,
            maximum_requests: plan.maximum_requests,
            maximum_retries: plan.maximum_retries,
            maximum_concurrency: plan.maximum_concurrency,
            plan_digest: receipt_replay.plan_digest,
            request_fingerprint: receipt_replay.request_fingerprint,
            prior_request_attempted: existing_receipt.is_some(),
            receipt_created: attempted_this_run,
            receipt_present: receipt.is_some(),
            outcome_evidence_present: capsule.is_some(),
            outcome_capsule_created: capsule_created_this_run,
            outcome_capsule_present: capsule.is_some(),
            evidence_status: if capsule.is_some() {
                "CompleteVerified".into()
            } else {
                "NoOutcomeRows".into()
            },
            receipt_digest: receipt.as_ref().map(|value| value.receipt_digest.clone()),
            outcome_capsule_digest: capsule.as_ref().map(|value| value.capsule_digest.clone()),
            opening_readiness: if capsule.is_some() {
                "ReadyForExplicitOpening".into()
            } else {
                "NotReadyForExplicitOpening".into()
            },
            network_requests: usize::from(attempted_this_run),
            network_request_count: usize::from(attempted_this_run),
            provider_calls: usize::from(attempted_this_run),
            transport_constructions: usize::from(attempted_this_run),
            network_consent_reads: usize::from(attempted_this_run),
            credential_reads: 0,
            outcome_row_reads: receipt
                .as_ref()
                .filter(|_| attempted_this_run)
                .map(|value| value.returned_row_count)
                .unwrap_or_default(),
            outcome_rows_admitted: receipt
                .as_ref()
                .filter(|_| attempted_this_run)
                .map(|value| value.verified_row_count)
                .unwrap_or_default(),
            prospective_label_reads: 0,
            labels_opened: context.label_open_count,
            label_open_count: context.label_open_count,
            metrics_computed: 0,
            metric_computations: 0,
            reward_candidates: 0,
            reward_candidates_created: 0,
            rewards_applied: 0,
            penalties_applied: 0,
            voice_changes: 0,
            cooldowns_started: 0,
            promotions_created: 0,
            quarantines_created: 0,
            chair_decisions_created: 0,
            votes_created: 0,
            risk_handoffs: 0,
            executions_created: 0,
            protected_artifacts_unchanged,
        },
        output_format,
    )
}

#[allow(clippy::too_many_arguments)]
fn run_learned_reward_eligibility_report(
    config_path: &Path,
    snapshot: &crate::data::DataSnapshot,
    output_format: &str,
) -> Result<(), String> {
    let state_path = config_path
        .parent()
        .ok_or("local learned reward state unavailable")?
        .join("prospective_shadow_challenge_v0.json");
    let momentum = crate::model::read_prospective_challenge_local_state_v0(&state_path)
        .map_err(|_| "local learned reward state unavailable".to_string())?;
    let risk_config = crate::model::CycleRiskShadowConfigV0::default();
    let risk_report = crate::model::run_cycle_risk_shadow_v0(snapshot, &risk_config)
        .map_err(|_| "offline Cycle/Risk prospective contract unavailable".to_string())?;
    let risk = crate::model::prepare_cycle_risk_prospective_tournament_v0(
        snapshot,
        &risk_report,
        &risk_config,
    )
    .map_err(|_| "offline Cycle/Risk prospective contract unavailable".to_string())?;
    let momentum_horizon = crate::core::stable_hash_string(&format!(
        "momentum-horizon-v0:{}:{}",
        momentum.capsule.prediction_horizon, momentum.capsule.label_policy_digest
    ));
    let risk_horizon = crate::core::stable_hash_string(&format!(
        "cycle-risk-horizon-v0:{}:{}",
        risk.prediction_horizon, risk.label_policy_digest
    ));
    let momentum_contract = crate::model::new_learned_prospective_contract_v0(
        crate::model::LearnedAgentObjectiveV0::DirectionalMomentum,
        momentum.capsule.capsule_digest.clone(),
        momentum.capsule.candidate.artifact_digest.clone(),
        momentum_horizon,
        momentum.capsule.prospective_cutoff_exclusive_timestamp_ms,
    )?;
    let risk_contract = crate::model::new_learned_prospective_contract_v0(
        crate::model::LearnedAgentObjectiveV0::DownsideRisk,
        risk.capsule_digest.clone(),
        risk.historical_champion.artifact_digest.clone(),
        risk_horizon,
        risk.cutoff_exclusive_timestamp_ms,
    )?;
    let gate = crate::model::new_learned_reward_sample_gate_v0(
        momentum
            .capsule
            .evaluation_policy
            .minimum_mature_events
            .max(risk.evaluation_policy.minimum_mature_events),
        momentum
            .capsule
            .evaluation_policy
            .minimum_support_qualified_events
            .max(risk.evaluation_policy.minimum_support_qualified_events),
        risk_report.regimes.len(),
    )?;
    let registration = crate::model::pre_register_learned_reward_eligibility_v0(
        &crate::model::LearnedRewardEligibilityRegistrationInputV0 {
            momentum: momentum_contract,
            cycle_risk: risk_contract,
            attribution_policy_digest: crate::core::stable_hash_string(
                "learned-prospective-attribution-policy-v0",
            ),
            maturity_policy_digest: crate::core::stable_hash_string(&format!(
                "learned-outcome-maturity-policy-v0:{}:{}:{}:{}",
                momentum.capsule.opening_policy.minimum_mature_events,
                momentum
                    .capsule
                    .opening_policy
                    .minimum_support_qualified_events,
                risk.opening_policy.minimum_mature_events,
                risk.opening_policy.minimum_support_qualified_events,
            )),
            sample_gate_policy_digest: gate.gate_digest.clone(),
            objective_mapping_policy_digest: crate::core::stable_hash_string(
                "learned-objective-mapping-policy-v0",
            ),
            integrity_policy_digest: crate::core::stable_hash_string(
                "learned-prospective-integrity-policy-v0",
            ),
        },
    )?;
    let ledger = crate::model::new_learned_prospective_outcome_ledger_v0(&registration)?;
    let momentum_eligibility = crate::model::derive_learned_reward_eligibility_v0(
        &registration,
        &gate,
        &ledger,
        crate::model::LearnedAgentObjectiveV0::DirectionalMomentum,
    )?;
    let risk_eligibility = crate::model::derive_learned_reward_eligibility_v0(
        &registration,
        &gate,
        &ledger,
        crate::model::LearnedAgentObjectiveV0::DownsideRisk,
    )?;
    let status = format!("{:?}", momentum_eligibility.eligibility_status);
    if momentum_eligibility.eligibility_status != risk_eligibility.eligibility_status {
        return Err("learned reward eligibility objective status mismatch".to_string());
    }
    if output_format == "json" {
        println!(
            "{}",
            serde_json::json!({
                "report_version": "learned-reward-eligibility-v0",
                "offline": true,
                "registration_digest": registration.registration_digest,
                "momentum_challenge_status": format!("{:?}", momentum.capsule.status),
                "momentum_registry_status": format!("{:?}", momentum.registry.challenges[0].status),
                "risk_tournament_status": format!("{:?}", risk.status),
                "prospective_event_count": ledger.event_attributions.len(),
                "mature_outcome_count": ledger.matured_outcomes.len(),
                "reward_candidate_count": ledger.reward_candidate_count,
                "reward_apply_count": ledger.reward_apply_count,
                "minimum_mature_events": gate.minimum_mature_events,
                "minimum_support_qualified_events": gate.minimum_support_qualified_events,
                "minimum_regime_coverage": gate.minimum_regime_coverage,
                "eligibility": status,
                "ledger_digest": ledger.ledger_digest,
                "provider_calls": 0,
                "transport_constructions": 0,
                "network_consent_reads": 0,
                "credential_reads": 0,
                "label_reads": 0,
                "chair_decision_count": 0,
                "reward_applied_count": 0,
                "penalty_applied_count": 0,
                "voice_mutation_count": 0,
                "cooldown_mutation_count": 0,
                "promotion_mutation_count": 0,
                "quarantine_mutation_count": 0,
                "execution_count": 0
            })
        );
    } else {
        println!("report_version=learned-reward-eligibility-v0");
        println!("offline=true");
        println!("registration_digest={}", registration.registration_digest);
        println!("momentum_challenge_status={:?}", momentum.capsule.status);
        println!(
            "momentum_registry_status={:?}",
            momentum.registry.challenges[0].status
        );
        println!("risk_tournament_status={:?}", risk.status);
        println!(
            "prospective_event_count={}",
            ledger.event_attributions.len()
        );
        println!("mature_outcome_count={}", ledger.matured_outcomes.len());
        println!("reward_candidate_count={}", ledger.reward_candidate_count);
        println!("reward_apply_count={}", ledger.reward_apply_count);
        println!("minimum_mature_events={}", gate.minimum_mature_events);
        println!(
            "minimum_support_qualified_events={}",
            gate.minimum_support_qualified_events
        );
        println!("minimum_regime_coverage={}", gate.minimum_regime_coverage);
        println!("eligibility={status}");
        println!("ledger_digest={}", ledger.ledger_digest);
        println!("provider_calls=0");
        println!("transport_constructions=0");
        println!("network_consent_reads=0");
        println!("credential_reads=0");
        println!("label_reads=0");
        println!("chair_decision_count=0");
        println!("reward_applied_count=0");
        println!("penalty_applied_count=0");
        println!("voice_mutation_count=0");
        println!("cooldown_mutation_count=0");
        println!("promotion_mutation_count=0");
        println!("quarantine_mutation_count=0");
        println!("execution_count=0");
    }
    Ok(())
}

fn run_btc_prospective_challenge_command(
    config_path: &Path,
    snapshot: &crate::data::DataSnapshot,
    campaign_config: &crate::model::MomentumLearningCampaignConfigV0,
    sufficiency: &crate::model::MomentumCampaignSufficiencyV0,
    create: bool,
    status: bool,
    confirm_preregistration: bool,
    close_registry: bool,
    accumulate: bool,
    evaluate: bool,
    output_format: &str,
    allow_network: bool,
) -> Result<(), String> {
    if output_format != "text" && output_format != "json" {
        return Err("unsupported BTC prospective challenge output format".to_string());
    }
    if !sufficiency.sufficient
        || crate::data::historical_replay_dataset_digest_v0(&snapshot.normalized_dataset)
            != snapshot.content_digest
    {
        return Err(
            "prospective challenge requires verified sufficient historical evidence".to_string(),
        );
    }
    let state_path = config_path
        .parent()
        .ok_or_else(|| "local prospective state directory unavailable".to_string())?
        .join("prospective_shadow_challenge_v0.json");
    if create {
        if state_path.exists() {
            return Err("local prospective challenge already exists; use status".to_string());
        }
        let preparation =
            crate::model::prepare_btc_prospective_challenge_v0(snapshot, campaign_config).map_err(
                |error| format!("prospective candidate or capsule resolution failed: {error:?}"),
            )?;
        if preparation.holdout.status
            != crate::model::ProspectiveHoldoutStatusV0::PolicySealedNoFutureRows
        {
            return Err("prospective challenge requires an unopened existing cutoff".to_string());
        }
        let state = crate::model::new_prospective_challenge_local_state_v0(preparation.capsule)
            .map_err(|_| "prospective challenge local state construction failed".to_string())?;
        crate::model::write_prospective_challenge_local_state_v0(&state_path, &state)
            .map_err(|_| "prospective challenge local state write failed".to_string())?;
        let reloaded = crate::model::read_prospective_challenge_local_state_v0(&state_path)
            .map_err(|_| "prospective challenge local state reread failed".to_string())?;
        if reloaded.capsule.capsule_digest != state.capsule.capsule_digest {
            return Err("prospective challenge capsule reread digest mismatch".to_string());
        }
        return render_blind_prospective_status(
            &reloaded,
            output_format,
            "sealed_awaiting_pre_registration_commit",
            preparation.ledger_digest,
            preparation.holdout.manifest_digest,
            0,
        );
    }
    let mut state = crate::model::read_prospective_challenge_local_state_v0(&state_path)
        .map_err(|_| "local prospective challenge is unavailable or invalid".to_string())?;
    if confirm_preregistration {
        crate::model::confirm_prospective_pre_registration_v0(&mut state).map_err(|_| {
            "prospective challenge is not eligible for pre-registration confirmation".to_string()
        })?;
        crate::model::write_prospective_challenge_local_state_v0(&state_path, &state)
            .map_err(|_| "prospective challenge local state write failed".to_string())?;
        return render_blind_prospective_status(
            &state,
            output_format,
            "pre_registration_committed",
            String::new(),
            String::new(),
            0,
        );
    }
    if close_registry {
        let preparation =
            crate::model::prepare_btc_prospective_challenge_v0(snapshot, campaign_config)
                .map_err(|_| "prospective challenge immutable revalidation failed".to_string())?;
        let provenance = crate::model::seal_missing_pre_registration_transition_v0(
            &mut state,
            &preparation.capsule,
        )
        .map_err(|_| "registry provenance closure failed".to_string())?;
        if provenance.status != crate::model::RegistryDigestProvenanceStatusV0::ValidTransitionChain
        {
            return Err("registry provenance is not a valid transition chain".to_string());
        }
        crate::model::write_prospective_challenge_local_state_v0(&state_path, &state)
            .map_err(|_| "prospective challenge local state write failed".to_string())?;
        return render_blind_prospective_status(
            &state,
            output_format,
            "registry_provenance_closed",
            String::new(),
            String::new(),
            0,
        );
    }
    if accumulate {
        let preparation =
            crate::model::prepare_btc_prospective_challenge_v0(snapshot, campaign_config)
                .map_err(|_| "prospective challenge immutable revalidation failed".to_string())?;
        let provenance = crate::model::compute_prospective_registry_digest_provenance_v0(&state)
            .map_err(|_| "registry provenance verification failed".to_string())?;
        let freeze =
            crate::model::build_prospective_challenge_freeze_proof_v0(&state, &preparation.capsule);
        if provenance.status != crate::model::RegistryDigestProvenanceStatusV0::ValidTransitionChain
            || !freeze.all_equal
        {
            return Err("prospective accumulation requires closed registry provenance".to_string());
        }
        let existing_timestamps = state
            .vault
            .finalized_rows
            .iter()
            .map(|row| row.timestamp_ms)
            .collect::<BTreeSet<_>>();
        let acquisition = crate::data::acquire_one_blind_upbit_daily_row_v0(
            config_path,
            &state.capsule.challenge_id,
            state.capsule.prospective_cutoff_exclusive_timestamp_ms,
            &existing_timestamps,
            allow_network,
        );
        if acquisition.receipt.request_attempted {
            crate::model::record_prospective_blind_acquisition_receipt_v0(
                &mut state,
                acquisition.receipt.clone(),
            )
            .map_err(|_| "prospective acquisition receipt rejected".to_string())?;
        }
        for (timestamp_ms, canonical_row_digest) in acquisition.admitted_rows {
            crate::model::append_prospective_vault_row_v0(
                &mut state,
                crate::model::ProspectiveEvidenceRowRefV0 {
                    timestamp_ms,
                    canonical_row_digest,
                    finalized: true,
                },
            )
            .map_err(|_| "prospective finalized row rejected".to_string())?;
            let bridge = crate::model::build_prospective_causal_context_bridge_v0(
                &state,
                snapshot.snapshot_id.clone(),
                snapshot.content_digest.clone(),
                snapshot.row_count,
                1,
                timestamp_ms,
            );
            if let Ok(bridge) = bridge {
                let comparator_artifact_digests = state
                    .capsule
                    .comparators
                    .iter()
                    .map(|comparator| comparator.artifact_digest.clone())
                    .collect::<Vec<_>>();
                let event_id = format!(
                    "sealed-abstention-{}",
                    crate::core::stable_hash_string(&format!(
                        "{}:{}:{}",
                        state.capsule.challenge_id, timestamp_ms, bridge.bridge_digest
                    ))
                );
                let event = crate::model::ProspectivePredictionEventV0 {
                    challenge_id: state.capsule.challenge_id.clone(),
                    event_id,
                    prediction_timestamp_ms: timestamp_ms,
                    required_label_maturity_timestamp_ms: timestamp_ms
                        .saturating_add(state.capsule.prediction_horizon as u64 * 86_400_000),
                    input_evidence_digest: bridge.bridge_digest,
                    candidate_artifact_digest: state.capsule.candidate.artifact_digest.clone(),
                    comparator_artifact_digests,
                    support_applicability: "unavailable".to_string(),
                    support_decision: "gate_unavailable".to_string(),
                    candidate_prediction: None,
                    comparator_predictions: vec![],
                    operational_outcome:
                        crate::model::ProspectiveShadowOutcomeV0::ShadowAbstainSupportUnavailable,
                    label_status: crate::model::ProspectiveLabelStatusV0::AwaitingFutureRows,
                    event_digest: String::new(),
                };
                crate::model::append_prospective_prediction_event_v0(&mut state, event)
                    .map_err(|_| "prospective sealed abstention rejected".to_string())?;
            }
        }
        if acquisition.receipt.request_attempted && acquisition.receipt.admitted_row_count == 0 {
            crate::model::mark_prospective_awaiting_future_rows_v0(&mut state)
                .map_err(|_| "prospective awaiting-future transition rejected".to_string())?;
        }
        crate::model::write_prospective_challenge_local_state_v0(&state_path, &state)
            .map_err(|_| "prospective challenge local state write failed".to_string())?;
        let acquisition_status = match acquisition.receipt.stop_reason {
            crate::data::ProspectiveBlindAcquisitionStopReasonV0::AwaitingNetworkConsent => {
                "awaiting_explicit_network_consent"
            }
            crate::data::ProspectiveBlindAcquisitionStopReasonV0::NoFinalizedDailyBoundary => {
                "no_finalized_daily_boundary"
            }
            crate::data::ProspectiveBlindAcquisitionStopReasonV0::ProviderUnavailable => {
                "provider_unavailable"
            }
            crate::data::ProspectiveBlindAcquisitionStopReasonV0::RateLimited => "rate_limited",
            crate::data::ProspectiveBlindAcquisitionStopReasonV0::PermissionDenied => {
                "permission_denied"
            }
            crate::data::ProspectiveBlindAcquisitionStopReasonV0::InvalidProviderResponse => {
                "provider_response_rejected"
            }
            crate::data::ProspectiveBlindAcquisitionStopReasonV0::NoAdmissibleFinalizedRow => {
                "no_admissible_finalized_row"
            }
            crate::data::ProspectiveBlindAcquisitionStopReasonV0::RowAdmitted => "row_admitted",
        };
        return render_blind_prospective_status(
            &state,
            output_format,
            acquisition_status,
            String::new(),
            String::new(),
            acquisition.receipt.request_count,
        );
    }
    if evaluate {
        return Err(
            "one-time prospective evaluation requires a later explicit authorization".to_string(),
        );
    }
    if status {
        return render_blind_prospective_status(
            &state,
            output_format,
            "offline_status",
            String::new(),
            String::new(),
            0,
        );
    }
    Err("prospective challenge action missing".to_string())
}

fn render_blind_prospective_status(
    state: &crate::model::ProspectiveChallengeLocalStateV0,
    output_format: &str,
    acquisition_status: &str,
    ledger_digest: String,
    holdout_manifest_digest: String,
    provider_calls: usize,
) -> Result<(), String> {
    let status = crate::model::blind_prospective_challenge_status_v0(state)
        .map_err(|_| "prospective challenge integrity verification failed".to_string())?;
    let provenance = crate::model::compute_prospective_registry_digest_provenance_v0(state)
        .map_err(|_| "prospective registry provenance verification failed".to_string())?;
    let challenge_status = format!("{:?}", status.challenge_status);
    let provenance_status = format!("{:?}", provenance.status);
    let provider_calls = provider_calls.max(
        state
            .blind_acquisition_receipts
            .iter()
            .map(|receipt| receipt.request_count)
            .sum(),
    );
    if output_format == "json" {
        println!(
            "{}",
            serde_json::json!({
                "report_version": "btc-prospective-shadow-challenge-v0",
                "offline": provider_calls == 0,
                "provider_calls": provider_calls,
                "transport_constructions": provider_calls,
                "acquisition_status": acquisition_status,
                "challenge_status": challenge_status,
                "finalized_row_count": status.finalized_row_count,
                "eligible_prediction_event_count": status.eligible_prediction_event_count,
                "support_qualified_prediction_count": status.support_qualified_prediction_count,
                "abstention_count": status.abstention_count,
                "awaiting_label_maturity_count": status.awaiting_label_maturity_count,
                "mature_but_sealed_label_count": status.mature_but_sealed_label_count,
                "capsule_digest": status.capsule_digest,
                "vault_digest": status.vault_digest,
                "journal_digest": status.journal_digest,
                "registry_digest": status.registry_digest,
                "registry_provenance_status": provenance_status,
                "registry_provenance_digest": provenance.provenance_digest,
                "registry_transition_record_count": provenance.transition_record_count,
                "blind_acquisition_receipt_digest": state
                    .blind_acquisition_receipts
                    .last()
                    .map(|receipt| receipt.receipt_digest.clone())
                    .unwrap_or_default(),
                "ledger_digest": ledger_digest,
                "holdout_manifest_digest": holdout_manifest_digest,
            })
        );
    } else {
        println!("report_version=btc-prospective-shadow-challenge-v0");
        println!("offline={}", provider_calls == 0);
        println!("provider_calls={provider_calls}");
        println!("transport_constructions={provider_calls}");
        println!("acquisition_status={acquisition_status}");
        println!("challenge_status={challenge_status}");
        println!("finalized_row_count={}", status.finalized_row_count);
        println!(
            "eligible_prediction_event_count={}",
            status.eligible_prediction_event_count
        );
        println!(
            "support_qualified_prediction_count={}",
            status.support_qualified_prediction_count
        );
        println!("abstention_count={}", status.abstention_count);
        println!(
            "awaiting_label_maturity_count={}",
            status.awaiting_label_maturity_count
        );
        println!(
            "mature_but_sealed_label_count={}",
            status.mature_but_sealed_label_count
        );
        println!("capsule_digest={}", status.capsule_digest);
        println!("vault_digest={}", status.vault_digest);
        println!("journal_digest={}", status.journal_digest);
        println!("registry_digest={}", status.registry_digest);
        println!("registry_provenance_status={provenance_status}");
        println!(
            "registry_provenance_digest={}",
            provenance.provenance_digest
        );
        println!(
            "registry_transition_record_count={}",
            provenance.transition_record_count
        );
        println!(
            "blind_acquisition_receipt_digest={}",
            state
                .blind_acquisition_receipts
                .last()
                .map(|receipt| receipt.receipt_digest.as_str())
                .unwrap_or_default()
        );
        if !ledger_digest.is_empty() {
            println!("ledger_digest={ledger_digest}");
        }
        if !holdout_manifest_digest.is_empty() {
            println!("holdout_manifest_digest={holdout_manifest_digest}");
        }
    }
    Ok(())
}

fn run_btc_multi_regime_evidence_report(
    config_path: &Path,
    snapshot: &crate::data::DataSnapshot,
    local_snapshots: &[crate::data::DataSnapshot],
    campaign_config: &crate::model::MomentumLearningCampaignConfigV0,
    inventory: &crate::model::HistoricalSnapshotInventoryV0,
    sufficiency: &crate::model::MomentumCampaignSufficiencyV0,
    snapshot_digest_matches: bool,
    output_format: &str,
    allow_network: bool,
) -> Result<(), String> {
    if output_format != "text" {
        return Err("BTC multi-regime report supports text output only".to_string());
    }
    let local_config = crate::data::UpbitHistoricalPilotConfigV0::from_toml_path(config_path)
        .map_err(|_| "local provider config unavailable".to_string())?;
    let preflight = crate::data::preflight_upbit_historical_backfill_v0(config_path, allow_network);
    let regime_config = crate::model::BtcHistoricalRegimeConfigV0 {
        minimum_regimes: 2,
        regime_rows: sufficiency.required_minimum_rows,
        inter_regime_gap_rows: campaign_config.purge_gap_rows,
        minimum_campaign_windows_per_regime: campaign_config.minimum_evaluated_windows,
        segmentation_policy:
            crate::model::TemporalRegimeSegmentationPolicyV0::EqualLengthChronological,
    };
    let request_budget = crate::data::ethical_upbit_request_budget_v0(&local_config);
    let evidence_requirement = crate::data::plan_btc_regime_backfill_v0(
        snapshot.row_count,
        regime_config.minimum_regimes,
        regime_config.regime_rows,
        regime_config.inter_regime_gap_rows,
        0,
        &local_config,
        &request_budget,
    );
    let dry_run =
        crate::data::sanitized_upbit_backfill_dry_run_v0(&evidence_requirement, &request_budget);
    let forensic_reference = local_snapshots
        .iter()
        .min_by_key(|candidate| candidate.fetched_at_ms)
        .unwrap_or(snapshot);
    let conflict_report = local_snapshots
        .iter()
        .filter(|candidate| {
            candidate.provider_id == "upbit"
                && candidate.normalized_dataset.symbol == snapshot.normalized_dataset.symbol
                && candidate.request_key.starts_with("upbit-daily-page:")
        })
        .filter_map(|candidate| {
            crate::data::inspect_upbit_duplicate_conflict_v0(forensic_reference, candidate)
                .ok()
                .map(|report| (candidate.fetched_at_ms, report))
        })
        .filter(|(_, report)| report.conflicting_duplicate_count > 0)
        .max_by_key(|(fetched_at_ms, _)| *fetched_at_ms)
        .map(|(_, report)| report);
    let strict_cursor_proof = crate::data::build_strict_older_cursor_proof_v0(
        snapshot,
        evidence_requirement.additional_rows_required,
        &local_config,
    );
    let mut expansion_status = if allow_network {
        crate::model::BtcHistoricalExpansionStatusV0::BackfillPreflightBlocked
    } else {
        crate::model::BtcHistoricalExpansionStatusV0::BackfillNotAuthorized
    };
    let mut backfill_rows = 0usize;
    let mut backfill_pages = 0usize;
    let mut expansion_reason = "not_attempted".to_string();
    let mut evidence_snapshot = snapshot.clone();
    if allow_network
        && preflight.status == crate::data::UpbitHistoricalPreflightStatusV0::Ready
        && evidence_requirement.plan_status
            == crate::data::BackfillRequestPlanStatusV0::RequestBudgetRejected
    {
        expansion_status =
            crate::model::BtcHistoricalExpansionStatusV0::BackfillRequestBudgetRejected;
    }
    if allow_network
        && preflight.status == crate::data::UpbitHistoricalPreflightStatusV0::Ready
        && evidence_requirement.plan_status == crate::data::BackfillRequestPlanStatusV0::Ready
        && strict_cursor_proof.proof_status
            == crate::data::StrictHistoricalRequestPlanStatusV0::ReadyZeroOverlap
    {
        let backfill = crate::data::run_manual_upbit_historical_backfill_at_end_v0(
            config_path,
            true,
            strict_cursor_proof.requested_count,
            Some(strict_cursor_proof.requested_exclusive_end),
        );
        backfill_rows = backfill.row_count;
        backfill_pages = backfill.page_receipts.len();
        if let Some(path) = backfill.local_snapshot_path {
            let harvested = crate::data::read_local_snapshot_protobuf_v1(Path::new(&path))?;
            let validation = crate::data::validate_strictly_older_upbit_page_v0(
                snapshot,
                &harvested,
                strict_cursor_proof.requested_count,
            );
            if validation.status == crate::data::StrictOlderPageExecutionStatusV0::OlderPageAccepted
            {
                match crate::data::merge_existing_upbit_snapshot_v0(snapshot, &harvested).and_then(
                    |(merged, _)| {
                        crate::data::write_and_verify_local_snapshot_v0(
                            &merged,
                            Path::new(&local_config.snapshot_output_dir),
                        )?;
                        Ok(merged)
                    },
                ) {
                    Ok(merged) => {
                        evidence_snapshot = merged;
                        expansion_status =
                            crate::model::BtcHistoricalExpansionStatusV0::ExpandedSnapshotAccepted;
                        expansion_reason = "strict_older_page_merged".to_string();
                    }
                    Err(reason) => {
                        expansion_status =
                            crate::model::BtcHistoricalExpansionStatusV0::ExpandedSnapshotRejected;
                        expansion_reason = reason;
                    }
                }
            } else {
                expansion_status =
                    crate::model::BtcHistoricalExpansionStatusV0::ExpandedSnapshotRejected;
                expansion_reason = format!("strict_page_rejected:{:?}", validation.status);
            }
        } else {
            expansion_status = crate::model::BtcHistoricalExpansionStatusV0::BackfillFailed;
            expansion_reason = "backfill_did_not_produce_a_verified_snapshot".to_string();
        }
    }
    let (_, pack) = crate::model::freeze_momentum_historical_evidence_pack_v0(
        std::slice::from_ref(&evidence_snapshot),
        &crate::model::HistoricalEvidencePolicyV0::default(),
    )
    .map_err(|_| "historical evidence freeze failed".to_string())?;
    crate::model::verify_momentum_historical_evidence_pack_v0(&pack)
        .map_err(|_| "historical evidence pack verification failed".to_string())?;
    let active_sufficiency = crate::model::assess_momentum_campaign_sufficiency_v0(
        evidence_snapshot.row_count,
        campaign_config,
    )
    .map_err(|_| "momentum campaign sufficiency calculation failed".to_string())?;
    let mut campaigns = Vec::new();
    let mut reproduced_report_digest = None;
    if local_config.campaign_attempt_enabled && active_sufficiency.sufficient {
        let encoder = crate::model::frozen_mamba3_encoder_from_seed_v0(
            &campaign_config.feature_config,
            campaign_config.campaign_seed,
            campaign_config.backend_preference,
            campaign_config.fallback_policy,
        )
        .map_err(|_| "frozen momentum encoder unavailable".to_string())?;
        let results =
            crate::model::run_momentum_series_campaigns_v0(&pack, campaign_config, &encoder)
                .map_err(|_| "momentum campaign execution failed".to_string())?;
        for result in results {
            let report = crate::model::build_momentum_temporal_diagnostic_report_v0(
                &result.campaign,
                evidence_snapshot.row_count,
                &pack.digest,
            );
            reproduced_report_digest = Some(report.report_digest);
            campaigns.push(result.campaign);
        }
    }
    let ledger =
        crate::model::build_historical_evidence_usage_ledger_v0(&evidence_snapshot, &campaigns)
            .map_err(|_| "historical evidence usage ledger failed".to_string())?;
    let regime_config = crate::model::BtcHistoricalRegimeConfigV0 {
        regime_rows: active_sufficiency.required_minimum_rows,
        ..regime_config
    };
    let segmentation =
        crate::model::segment_btc_historical_regimes_v0(&evidence_snapshot, &regime_config)
            .map_err(|_| "BTC regime segmentation failed".to_string())?;
    let packs = crate::model::freeze_btc_historical_regime_packs_v0(
        &evidence_snapshot,
        &segmentation,
        &crate::model::HistoricalEvidencePolicyV0::default(),
    )
    .map_err(|_| "BTC regime pack freeze failed".to_string())?;
    let regime_results = if packs.is_empty() {
        vec![]
    } else {
        let encoder = crate::model::frozen_mamba3_encoder_from_seed_v0(
            &campaign_config.feature_config,
            campaign_config.campaign_seed,
            campaign_config.backend_preference,
            campaign_config.fallback_policy,
        )
        .map_err(|_| "frozen momentum encoder unavailable".to_string())?;
        crate::model::run_btc_historical_regime_campaigns_v0(&packs, campaign_config, &encoder)
            .map_err(|_| "BTC regime campaign execution failed".to_string())?
    };
    let aggregate =
        crate::model::aggregate_btc_cross_regime_evidence_v0(&regime_results, &regime_config)
            .map_err(|_| "BTC cross-regime aggregation failed".to_string())?;
    let holdout = crate::model::seal_prospective_holdout_v0(
        &ledger,
        &crate::model::ProspectiveHoldoutPolicyConfigV0 {
            minimum_future_rows: regime_config.regime_rows,
            required_future_windows: regime_config.minimum_campaign_windows_per_regime,
        },
        &[],
    )
    .map_err(|_| "prospective holdout seal failed".to_string())?;
    println!("report_version=btc-multi-regime-evidence-v0");
    println!("original_snapshot_digest_matches={snapshot_digest_matches}");
    println!(
        "inventory_accepted_series={}",
        inventory.accepted_series.len()
    );
    println!("original_evidence_pack_verified={}", pack.frozen);
    println!(
        "existing_temporal_report_digest={}",
        reproduced_report_digest.unwrap_or_default()
    );
    println!("usage_ledger_digest={}", ledger.ledger_digest);
    println!("usage_record_count={}", ledger.usages.len());
    println!(
        "maximum_consumed_timestamp_ms={}",
        ledger.maximum_consumed_timestamp_ms
    );
    println!("upbit_preflight={:?}", preflight.status);
    println!(
        "conflict_forensics_status={}",
        conflict_report
            .as_ref()
            .map(|report| format!("{:?}", report.forensic_status))
            .unwrap_or_else(|| "ConflictArtifactUnavailable".to_string())
    );
    println!(
        "conflict_overlap_count={}",
        conflict_report
            .as_ref()
            .map(|report| report.overlapping_timestamp_count)
            .unwrap_or_default()
    );
    println!(
        "conflict_count={}",
        conflict_report
            .as_ref()
            .map(|report| report.conflicting_duplicate_count)
            .unwrap_or_default()
    );
    println!(
        "conflict_identical_count={}",
        conflict_report
            .as_ref()
            .map(|report| report.identical_duplicate_count)
            .unwrap_or_default()
    );
    println!(
        "conflict_first_field={}",
        conflict_report
            .as_ref()
            .and_then(|report| report.first_conflicting_field)
            .map(|field| format!("{field:?}"))
            .unwrap_or_default()
    );
    println!(
        "conflict_finalized_count={}",
        conflict_report
            .as_ref()
            .map(|report| report.finalized_conflict_count)
            .unwrap_or_default()
    );
    println!(
        "conflict_potentially_open_count={}",
        conflict_report
            .as_ref()
            .map(|report| report.potentially_open_conflict_count)
            .unwrap_or_default()
    );
    println!(
        "previous_request_cursor_class={}",
        conflict_report
            .as_ref()
            .map(|report| report.previous_request_cursor_class.as_str())
            .unwrap_or_default()
    );
    println!(
        "conflict_root_cause={}",
        conflict_report
            .as_ref()
            .map(|report| format!("{:?}", report.root_cause))
            .unwrap_or_else(|| "Unknown".to_string())
    );
    println!(
        "conflict_report_digest={}",
        conflict_report
            .as_ref()
            .map(|report| report.report_digest.as_str())
            .unwrap_or_default()
    );
    println!(
        "strict_cursor_proof_status={:?}",
        strict_cursor_proof.proof_status
    );
    println!(
        "strict_cursor_requested_count={}",
        strict_cursor_proof.requested_count
    );
    println!(
        "strict_cursor_expected_overlap={}",
        strict_cursor_proof.expected_overlap_rows
    );
    println!(
        "strict_cursor_proof_digest={}",
        strict_cursor_proof.proof_digest
    );
    println!(
        "backfill_plan_status={:?}",
        evidence_requirement.plan_status
    );
    println!(
        "backfill_required_total_rows={}",
        evidence_requirement.required_total_rows
    );
    println!(
        "backfill_additional_rows_required={}",
        evidence_requirement.additional_rows_required
    );
    println!(
        "backfill_estimated_minimum_requests={}",
        evidence_requirement.estimated_minimum_pages
    );
    println!(
        "backfill_budget_maximum_requests={}",
        request_budget.maximum_requests
    );
    println!(
        "backfill_minimum_inter_request_delay_ms={}",
        request_budget.minimum_inter_request_delay_ms
    );
    println!("backfill_dry_run_digest={}", dry_run.plan_digest);
    println!("historical_expansion_status={expansion_status:?}");
    println!("historical_expansion_reason={expansion_reason}");
    println!("backfill_rows={backfill_rows}");
    println!("backfill_page_receipts={backfill_pages}");
    println!("regime_config_digest={}", regime_config.digest());
    println!("regime_segmentation_status={:?}", segmentation.status);
    println!("regime_count={}", segmentation.regimes.len());
    println!("regime_pack_count={}", packs.len());
    println!("cross_regime_status={:?}", aggregate.status);
    println!("cross_regime_report_digest={}", aggregate.report_digest);
    println!(
        "prospective_holdout_cutoff_ms={}",
        holdout.cutoff_exclusive_timestamp_ms
    );
    println!("prospective_holdout_status={:?}", holdout.status);
    println!("prospective_holdout_opened={}", holdout.opened);
    println!(
        "prospective_holdout_labels_accessed={}",
        holdout.labels_accessed
    );
    println!("network_calls_after_pack_freeze=0");
    println!("model_campaign_config_digest={}", campaign_config.digest());
    Ok(())
}

fn historical_snapshot_selection_rank(snapshot: &crate::data::DataSnapshot) -> u8 {
    if snapshot.request_key.starts_with("upbit-daily-expanded:") {
        3
    } else if snapshot.request_key.starts_with("upbit-daily-page:") {
        1
    } else {
        2
    }
}

fn run_momentum_campaign_if_enabled(
    config_path: &Path,
    snapshot: &crate::data::DataSnapshot,
    campaign_config: &crate::model::MomentumLearningCampaignConfigV0,
    sufficiency: &crate::model::MomentumCampaignSufficiencyV0,
    temporal_diagnostics: bool,
    output_format: &str,
    cross_market_report: bool,
) -> Result<(), String> {
    let local_config = crate::data::UpbitHistoricalPilotConfigV0::from_toml_path(config_path)
        .map_err(|_| "local provider config unavailable after smoke".to_string())?;
    if !local_config.campaign_attempt_enabled || !sufficiency.sufficient {
        return Ok(());
    }
    let inventory = crate::model::inventory_historical_snapshots_v0(
        std::slice::from_ref(snapshot),
        &crate::model::HistoricalEvidencePolicyV0::default(),
    )
    .map_err(|_| "historical snapshot inventory failed".to_string())?;
    if inventory.accepted_series.is_empty() {
        return Err("historical evidence inventory has no accepted series".to_string());
    }
    let (_, pack) = crate::model::freeze_momentum_historical_evidence_pack_v0(
        std::slice::from_ref(snapshot),
        &crate::model::HistoricalEvidencePolicyV0::default(),
    )
    .map_err(|_| "historical evidence freeze failed".to_string())?;
    crate::model::verify_momentum_historical_evidence_pack_v0(&pack)
        .map_err(|_| "historical evidence pack verification failed".to_string())?;
    if !temporal_diagnostics {
        println!("evidence_pack_frozen={}", pack.frozen);
        println!(
            "evidence_pack_digest_prefix={}",
            pack.digest.chars().take(12).collect::<String>()
        );
    }
    let encoder = crate::model::frozen_mamba3_encoder_from_seed_v0(
        &campaign_config.feature_config,
        campaign_config.campaign_seed,
        campaign_config.backend_preference,
        campaign_config.fallback_policy,
    )
    .map_err(|_| "frozen momentum encoder unavailable".to_string())?;
    let results = crate::model::run_momentum_series_campaigns_v0(&pack, campaign_config, &encoder)
        .map_err(|_| "momentum campaign execution failed".to_string())?;
    for result in results {
        if temporal_diagnostics {
            let report = crate::model::build_momentum_temporal_diagnostic_report_v0(
                &result.campaign,
                snapshot.row_count,
                &pack.digest,
            );
            let korean = crate::toss::qualify_toss_historical_capability_v0(
                crate::toss::TossHistoricalCapabilityV0::KoreanEquityDailyOhlcv,
            );
            let us = crate::toss::qualify_toss_historical_capability_v0(
                crate::toss::TossHistoricalCapabilityV0::UsEquityDailyOhlcv,
            );
            let qualifications = [&korean, &us];
            let configured_markets = 1 + qualifications.len();
            let accepted_markets = 1 + qualifications
                .iter()
                .filter(|qualification| {
                    qualification.status
                        == crate::toss::TossHistoricalContractStatusV0::SnapshotAccepted
                })
                .count();
            let cross_market_status = if accepted_markets == configured_markets {
                "complete"
            } else {
                "contract_blocked"
            };
            let reason_codes = vec![
                format!(
                    "toss_kr_{}",
                    toss_historical_contract_status_code(korean.status)
                ),
                format!(
                    "toss_us_{}",
                    toss_historical_contract_status_code(us.status)
                ),
            ];
            let rendered = match output_format {
                "json" if cross_market_report => {
                    let btc = serde_json::from_str::<serde_json::Value>(
                        &crate::model::momentum_temporal_diagnostic_report_json_v0(&report),
                    )
                    .map_err(|_| "temporal report serialization failed".to_string())?;
                    let cross_market_evidence = build_cross_market_evidence_v0(&btc, &korean, &us);
                    let report_digest = cross_market_evidence["report_digest"]
                        .as_str()
                        .unwrap_or_default()
                        .to_string();
                    let permissions = btc["permissions"].clone();
                    serde_json::json!({
                        "report_version": "three-market-momentum-evidence-v1",
                        "btc": btc,
                        "korean_equity": korean,
                        "us_equity": us,
                        "cross_market_evidence": cross_market_evidence,
                        "cross_market_status": cross_market_status,
                        "permissions": permissions,
                        "reason_codes": reason_codes,
                        "report_digest": report_digest,
                    })
                    .to_string()
                }
                "json" => crate::model::momentum_temporal_diagnostic_report_json_v0(&report),
                "text" if cross_market_report => format!(
                    "cross_market_status={cross_market_status}\nconfigured_markets={configured_markets}\naccepted_markets={accepted_markets}\nkorean_equity_status={}\nus_equity_status={}\nreason_codes={}\n{}",
                    toss_historical_contract_status_code(korean.status),
                    toss_historical_contract_status_code(us.status),
                    reason_codes.join(","),
                    crate::model::momentum_temporal_diagnostic_report_text_v0(&report),
                ),
                "text" => crate::model::momentum_temporal_diagnostic_report_text_v0(&report),
                _ => return Err("unsupported temporal diagnostic output format".to_string()),
            };
            println!("{rendered}");
            continue;
        }
        println!(
            "campaign_series_status={:?};windows={};drift={:?};versions={}",
            result.campaign.status,
            result.campaign.windows.len(),
            result.campaign.aggregate_drift,
            result.campaign.generated_versions.len()
        );
        if let (Some(gate), Some(reason_code)) = (
            result.campaign.safety_trace.first_rejecting_gate,
            result.campaign.safety_trace.first_reason_code.as_deref(),
        ) {
            println!("campaign_safety_first_rejection={gate:?};reason_code={reason_code}");
        }
        let eligibility = &result.campaign.safety_trace.eligibility;
        println!(
            "campaign_layered_eligibility=offline_shadow_learning:{};promotion:{};voting:{};execution:{}",
            eligibility.offline_shadow_learning,
            eligibility.promotion,
            eligibility.voting,
            eligibility.execution,
        );
        for forensics in &result.campaign.collapse_forensics {
            println!(
                "campaign_collapse_forensics=status={:?};root_cause={:?};selected_candidate={:?};test_opened_once={}",
                forensics.diagnostic_status,
                forensics.root_cause,
                forensics.selected_candidate,
                forensics.test_partition_opened_once,
            );
        }
        for window in result.campaign.windows {
            for path in window.paths {
                let baselines = path.baselines;
                println!(
                    "campaign_window={};path={:?};constant_brier={:.6};linear_brier={:.6};mamba_brier={:.6};mamba_minus_linear={:.6};test_samples={};high_confidence_errors={};drift={:?}",
                    window.window.window_id,
                    path.path,
                    baselines.constant_probability.brier_score,
                    baselines.linear_momentum.brier_score,
                    baselines.frozen_mamba.brier_score,
                    baselines.frozen_mamba.brier_score - baselines.linear_momentum.brier_score,
                    baselines.frozen_mamba.sample_count,
                    baselines.frozen_mamba.high_confidence_error_count,
                    window.drift_status,
                );
            }
        }
    }
    Ok(())
}

fn print_toss_historical_contract_report(
    kr_manifest_path: Option<PathBuf>,
    us_manifest_path: Option<PathBuf>,
) -> Result<(), String> {
    let kr_manifest_path = kr_manifest_path.or_else(|| {
        crate::toss::toss_historical_manifest_path_from_env(
            crate::toss::TossHistoricalCapabilityV0::KoreanEquityDailyOhlcv,
        )
    });
    let us_manifest_path = us_manifest_path.or_else(|| {
        crate::toss::toss_historical_manifest_path_from_env(
            crate::toss::TossHistoricalCapabilityV0::UsEquityDailyOhlcv,
        )
    });
    let kr_intake = crate::toss::inspect_toss_historical_manifest_v1(kr_manifest_path.as_deref());
    let us_intake = crate::toss::inspect_toss_historical_manifest_v1(us_manifest_path.as_deref());
    let kr_manifest = kr_manifest_path
        .as_deref()
        .and_then(|path| crate::toss::TossHistoricalContractManifestV1::from_toml_path(path).ok());
    let us_manifest = us_manifest_path
        .as_deref()
        .and_then(|path| crate::toss::TossHistoricalContractManifestV1::from_toml_path(path).ok());
    let kr_qualification = crate::toss::qualify_toss_historical_manifest_v1(
        crate::toss::TossHistoricalCapabilityV0::KoreanEquityDailyOhlcv,
        kr_manifest.as_ref(),
    );
    let us_qualification = crate::toss::qualify_toss_historical_manifest_v1(
        crate::toss::TossHistoricalCapabilityV0::UsEquityDailyOhlcv,
        us_manifest.as_ref(),
    );
    let selection = crate::toss::select_toss_historical_capability_v1(
        crate::toss::TossHistoricalCapabilityV0::KoreanEquityDailyOhlcv,
        false,
        kr_qualification,
        us_qualification,
    );
    let korean = crate::toss::qualify_toss_historical_capability_v0(
        crate::toss::TossHistoricalCapabilityV0::KoreanEquityDailyOhlcv,
    );
    let us = crate::toss::qualify_toss_historical_capability_v0(
        crate::toss::TossHistoricalCapabilityV0::UsEquityDailyOhlcv,
    );
    println!(
        "{}",
        serde_json::json!({
            "report_version": "toss-historical-contract-intake-v1",
            "kr_intake": kr_intake,
            "us_intake": us_intake,
            "selection": selection,
            "korean_equity": korean,
            "us_equity": us,
            "network_calls": 0,
        })
    );
    Ok(())
}

fn toss_historical_contract_status_code(
    status: crate::toss::TossHistoricalContractStatusV0,
) -> &'static str {
    match status {
        crate::toss::TossHistoricalContractStatusV0::Qualified => "qualified",
        crate::toss::TossHistoricalContractStatusV0::ContractIncomplete => "contract_incomplete",
        crate::toss::TossHistoricalContractStatusV0::ContractMaterialUnavailable => {
            "contract_material_unavailable"
        }
        crate::toss::TossHistoricalContractStatusV0::RequiresGuessedMapping => {
            "requires_guessed_mapping"
        }
        crate::toss::TossHistoricalContractStatusV0::UnsupportedHistoricalDataset => {
            "unsupported_historical_dataset"
        }
        crate::toss::TossHistoricalContractStatusV0::ConfigurationMissing => {
            "configuration_missing"
        }
        crate::toss::TossHistoricalContractStatusV0::CredentialUnavailable => {
            "credential_unavailable"
        }
        crate::toss::TossHistoricalContractStatusV0::NetworkConsentRequired => {
            "network_consent_required"
        }
        crate::toss::TossHistoricalContractStatusV0::SmokeFailed => "smoke_failed",
        crate::toss::TossHistoricalContractStatusV0::SnapshotAccepted => "snapshot_accepted",
        crate::toss::TossHistoricalContractStatusV0::SnapshotRejected => "snapshot_rejected",
    }
}

fn build_cross_market_evidence_v0(
    btc: &serde_json::Value,
    korean: &crate::toss::TossHistoricalContractQualificationV0,
    us: &crate::toss::TossHistoricalContractQualificationV0,
) -> serde_json::Value {
    let aggregate = &btc["aggregate"];
    let number = |field: &str| {
        aggregate[field]
            .as_u64()
            .and_then(|value| usize::try_from(value).ok())
            .unwrap_or(0)
    };
    let btc_windows = btc["reproduction"]["total_windows"].as_u64().unwrap_or(0);
    let btc_no_signal_windows = btc["reproduction"]["no_signal_windows"]
        .as_u64()
        .unwrap_or(0);
    let btc_rows = btc["evidence"]["row_count"].as_u64().unwrap_or(0);
    let btc_representation_breach = number("representation_shift_windows") > 0;
    let qualifications = [korean, us];
    let configured_markets = 1 + qualifications.len();
    let contract_qualified_markets = qualifications
        .iter()
        .filter(|qualification| {
            matches!(
                qualification.status,
                crate::toss::TossHistoricalContractStatusV0::Qualified
                    | crate::toss::TossHistoricalContractStatusV0::SnapshotAccepted
            )
        })
        .count();
    let acquired_markets = qualifications
        .iter()
        .filter(|qualification| {
            qualification.status == crate::toss::TossHistoricalContractStatusV0::SnapshotAccepted
        })
        .count();
    let accepted_markets = 1 + acquired_markets;
    let status = if accepted_markets < 2 {
        "insufficient_markets"
    } else {
        "pending_multi_market_evaluation"
    };
    let btc_report_digest = btc["report_digest"].as_str().unwrap_or_default();
    let report_digest = stable_cross_market_report_digest(&[
        btc_report_digest,
        toss_historical_contract_status_code(korean.status),
        toss_historical_contract_status_code(us.status),
        status,
    ]);
    let blocked_row =
        |market: &str, qualification: &crate::toss::TossHistoricalContractQualificationV0| {
            serde_json::json!({
                "provider": "toss",
                "market": market,
                "contract_status": toss_historical_contract_status_code(qualification.status),
                "snapshot_accepted": false,
                "row_count": 0,
                "valid_windows": 0,
                "no_signal_windows": 0,
                "selected_checkpoint_windows": 0,
                "in_support_windows": 0,
                "out_of_support_windows": 0,
                "dominant_earliest_shift_stage": null,
                "dominant_root_cause": null,
                "representation_breach_count": 0,
                "warm_start_status": null,
                "operational_abstentions": 0,
                "accepted_predictive_versions": 0,
                "per_series_verdict": "contract_blocked",
            })
        };
    serde_json::json!({
        "configured_markets": configured_markets,
        "contract_qualified_markets": contract_qualified_markets,
        "acquired_markets": acquired_markets,
        "accepted_markets": accepted_markets,
        "evaluated_markets": 1,
        "total_series": configured_markets,
        "total_windows": btc_windows,
        "no_signal_windows": btc_no_signal_windows,
        "in_support_windows": number("in_support_windows"),
        "out_of_support_windows": number("out_of_support_windows"),
        "frozen_representation_shift_markets": btc_representation_breach as usize,
        "feature_shift_markets": (number("normalized_feature_shift_windows") > 0) as usize,
        "sequence_shift_markets": (number("sequence_shift_windows") > 0) as usize,
        "logit_shift_markets": (number("logit_shift_windows") > 0) as usize,
        "probability_shift_markets": (number("probability_shift_windows") > 0) as usize,
        "stable_markets": 0,
        "warm_lock_in_markets": (number("warm_lock_in_windows") > 0) as usize,
        "abstention_count": number("operational_abstentions"),
        "accepted_predictive_versions": number("accepted_predictive_versions"),
        "status": status,
        "markets": [
            {
                "provider": "upbit",
                "market": "btc_crypto",
                "contract_status": "not_applicable",
                "snapshot_accepted": true,
                "row_count": btc_rows,
                "valid_windows": btc_windows,
                "no_signal_windows": btc["reproduction"]["no_signal_windows"],
                "selected_checkpoint_windows": btc["reproduction"]["selected_checkpoint_windows"],
                "in_support_windows": number("in_support_windows"),
                "out_of_support_windows": number("out_of_support_windows"),
                "dominant_earliest_shift_stage": btc["shift"]["earliest_stage"],
                "dominant_root_cause": btc["shift"]["root_cause"],
                "representation_breach_count": number("representation_shift_windows"),
                "warm_start_status": btc["warm_start"]["status"],
                "operational_abstentions": number("operational_abstentions"),
                "accepted_predictive_versions": number("accepted_predictive_versions"),
                "per_series_verdict": btc["final_verdict"],
            },
            blocked_row("korean_equity", korean),
            blocked_row("us_equity", us),
        ],
        "report_digest": report_digest,
    })
}

fn stable_cross_market_report_digest(parts: &[&str]) -> String {
    let mut hash = 0xcbf29ce484222325_u64;
    for part in parts {
        for byte in part.as_bytes().iter().chain(std::iter::once(&0)) {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x100000001b3);
        }
    }
    format!("{hash:016x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn momentum_mamba_repair_cli_fixture_v2() -> MomentumMambaRepairCliReportV2 {
        MomentumMambaRepairCliReportV2 {
            report_version: "momentum-mamba-repair-cli-report-v2",
            mode: "execute-local".to_string(),
            offline: true,
            status: crate::model::MomentumMambaRepairExecutionStatusV2::Executed,
            collapse_root_causes: vec![
                crate::model::MomentumMambaCollapseRootCauseV2::ProbabilitySingleSided,
            ],
            collapse_audit_digest: Some("audit".to_string()),
            representation_diagnostic_digest: Some("representation".to_string()),
            optimization_diagnostic_digest: Some("optimization".to_string()),
            probability_diagnostic_digest: Some("probability".to_string()),
            class_balance_diagnostic_digest: Some("class-balance".to_string()),
            repair_capability_status: Some(
                crate::model::MomentumMambaRepairCapabilityStatusV2::RepairableWithBoundedHeadRegularization,
            ),
            repair_split_digest: Some("split".to_string()),
            repair_registration_digest: Some("repair-registration".to_string()),
            registered_variant_count: 3,
            participants: vec![MomentumMambaRepairParticipantCliV2 {
                participant_digest: "participant".to_string(),
                model_kind: "FrozenMambaHeadV2/control".to_string(),
                participant_role: crate::model::ParticipantQualificationRoleV2::LearnedCandidate,
                qualification_status:
                    crate::model::ValidationQualificationStatusV2::RejectedProbabilityCollapse,
            }],
            qualified_learned_participant_count: 0,
            qualified_comparator_count: 2,
            family_digest: Some("family".to_string()),
            winner_selected: false,
            historical_test_accessed: false,
            roster_status:
                crate::model::MomentumFutureEvaluationRosterStatusV2::NoQualifiedLearnedParticipant,
            roster_digest: None,
            evaluation_registration_status:
                crate::model::MomentumFutureEvaluationRegistrationStatusV2::NoQualifiedLearnedParticipant,
            evaluation_registration_digest: None,
            minimum_accepted_timestamp_ms: None,
            cycle_risk_evidence_status: Some(
                crate::data::CanonicalViewGapStatusV1::ProviderContractUnverified,
            ),
            value_quality_evidence_status: Some(
                crate::data::CanonicalViewGapStatusV1::TrainerUnavailable,
            ),
            reward_eligibility_replay: persisted_intent_migration_cli_fixture_v1()
                .reward_eligibility_replay,
            artifacts_written: 15,
            duplicate_artifact_count: 0,
            storage_failure_count: 0,
            protected_artifacts_unchanged: true,
            active_state_unchanged: true,
            safety_counters: crate::model::MomentumMambaRepairSafetyCountersV2 {
                network_requests: 0,
                transport_constructions: 0,
                credential_reads: 0,
                prospective_row_reads: 0,
                prospective_label_openings: 0,
                future_evaluation_reads: 0,
                historical_test_reads: 0,
                active_model_changes: 0,
                chair_decisions: 0,
                votes: 0,
                reward_applications: 0,
                penalty_applications: 0,
                voice_changes: 0,
                cooldowns_started: 0,
                promotions: 0,
                quarantines: 0,
                executions: 0,
                active_committee_count: 3,
            },
            report_digest: "report".to_string(),
        }
    }

    #[test]
    fn momentum_mamba_repair_text_and_json_public_fields_agree() {
        let report = momentum_mamba_repair_cli_fixture_v2();
        let text = format_momentum_mamba_repair_text_v2(&report);
        let json = serde_json::to_value(&report).unwrap();
        for field in [
            "network_requests",
            "transport_constructions",
            "credential_reads",
            "prospective_row_reads",
            "prospective_label_openings",
            "future_evaluation_reads",
            "historical_test_reads",
            "active_model_changes",
            "chair_decisions",
            "votes",
            "reward_applications",
            "penalty_applications",
            "voice_changes",
            "cooldowns_started",
            "promotions",
            "quarantines",
            "executions",
            "active_committee_count",
        ] {
            assert!(text.contains(&format!("{field}={}", json["safety_counters"][field])));
        }
        assert!(text.contains("collapse_root_causes=[ProbabilitySingleSided]"));
        assert!(text.contains("registered_variant_count=3"));
        assert!(text.contains("minimum_accepted_timestamp_ms="));
        assert!(json["minimum_accepted_timestamp_ms"].is_null());
        assert!(text.contains("reward_apply_count=0"));
        assert!(text.contains("penalty_apply_count=0"));
    }

    #[test]
    fn momentum_mamba_repair_rejects_network_permission_before_io() {
        let error = run_momentum_mamba_repair_cli_v2(
            Path::new("not-used"),
            "json",
            true,
            false,
            false,
            true,
        )
        .unwrap_err();
        assert_eq!(error, "Momentum Mamba repair is offline-only");
    }

    fn momentum_mamba_representation_cli_fixture_v3() -> MomentumMambaRepresentationCliReportV3 {
        MomentumMambaRepresentationCliReportV3 {
            report_version: "momentum-mamba-representation-cli-report-v3",
            mode: "execute-local".to_string(),
            offline: true,
            status: crate::model::MomentumRepresentationExecutionStatusV3::Executed,
            repair_stage:
                crate::model::MomentumFrozenMambaRepairStageV3::V3MambaContributionAbsent,
            probes: vec![MomentumRepresentationProbeCliV3 {
                probe_kind: crate::model::MomentumRepresentationProbeKindV3::MambaMeanOutputProbe,
                status:
                    crate::model::MomentumRepresentationProbeStatusV3::NonCollapsedPrediction,
                representation_diagnostic_digest: "representation-diagnostic".to_string(),
                probe_digest: "probe".to_string(),
            }],
            representation_audit_digest: Some("audit".to_string()),
            split_digest: Some("split".to_string()),
            final_reserved_range_digest: Some("reserve".to_string()),
            registration_digest: Some("registration".to_string()),
            registered_variant_count: 4,
            participants: vec![MomentumRepresentationParticipantCliV3 {
                participant_digest: "participant".to_string(),
                model_kind: "FrozenMambaRepresentationV3/mean".to_string(),
                input_kind: "MambaMeanOutput".to_string(),
                qualification_status:
                    crate::model::MomentumRepresentationQualificationStatusV3::RejectedProbabilityCollapse,
                contribution_status: Some(
                    crate::model::MambaContributionStatusV3::NoDetectableContribution,
                ),
            }],
            qualified_genuine_mamba_count: 0,
            qualified_raw_fallback_count: 1,
            qualified_comparator_count: 2,
            route_decision: Some(
                crate::model::MomentumRepresentationRouteDecisionV3::RawFeatureFallbackOnly,
            ),
            decision_digest: Some("decision".to_string()),
            family_digest: Some("family".to_string()),
            roster_status:
                crate::model::MomentumRepresentationRosterStatusV3::FrozenMambaRepresentationPathRejected,
            roster_digest: None,
            evaluation_registration_status:
                crate::model::MomentumRepresentationEvaluationStatusV3::FrozenMambaRepresentationPathRejected,
            evaluation_registration_digest: None,
            minimum_accepted_timestamp_ms: None,
            cycle_risk_evidence_status: Some(
                crate::data::CanonicalViewGapStatusV1::ProviderContractUnverified,
            ),
            value_quality_evidence_status: Some(
                crate::data::CanonicalViewGapStatusV1::TrainerUnavailable,
            ),
            reward_eligibility_replay: persisted_intent_migration_cli_fixture_v1()
                .reward_eligibility_replay,
            artifacts_written: 26,
            duplicate_artifact_count: 0,
            storage_failure_count: 0,
            protected_artifacts_unchanged: true,
            active_state_unchanged: true,
            safety_counters: crate::model::MomentumRepresentationSafetyCountersV3 {
                network_requests: 0,
                transport_constructions: 0,
                credential_reads: 0,
                prospective_row_reads: 0,
                prospective_label_openings: 0,
                historical_test_reads: 0,
                future_evaluation_reads: 0,
                active_model_changes: 0,
                chair_decisions: 0,
                votes: 0,
                reward_applications: 0,
                penalty_applications: 0,
                voice_changes: 0,
                cooldowns_started: 0,
                promotions: 0,
                quarantines: 0,
                executions: 0,
                active_committee_count: 3,
            },
            report_digest: "report".to_string(),
        }
    }

    #[test]
    fn momentum_mamba_representation_text_and_json_public_fields_agree() {
        let report = momentum_mamba_representation_cli_fixture_v3();
        let text = format_momentum_mamba_representation_text_v3(&report);
        let json = serde_json::to_value(&report).unwrap();
        for field in [
            "network_requests",
            "transport_constructions",
            "credential_reads",
            "prospective_row_reads",
            "prospective_label_openings",
            "historical_test_reads",
            "future_evaluation_reads",
            "active_model_changes",
            "chair_decisions",
            "votes",
            "reward_applications",
            "penalty_applications",
            "voice_changes",
            "cooldowns_started",
            "promotions",
            "quarantines",
            "executions",
            "active_committee_count",
        ] {
            assert!(text.contains(&format!("{field}={}", json["safety_counters"][field])));
        }
        assert!(text.contains("registered_variant_count=4"));
        assert!(text.contains("qualified_genuine_mamba_count=0"));
        assert!(text.contains("route_decision=RawFeatureFallbackOnly"));
        assert!(json["minimum_accepted_timestamp_ms"].is_null());
        let rendered_json = serde_json::to_string(&report).unwrap();
        for forbidden in [
            "raw_rows",
            "raw_features",
            "private_probe_metric_digest",
            "private_metric_digest",
            "logits",
            "probabilities",
            "labels",
            "weights",
            "gradients",
            "artifact_path",
        ] {
            assert!(!text.contains(forbidden));
            assert!(!rendered_json.contains(forbidden));
        }
    }

    #[test]
    fn momentum_mamba_representation_rejects_network_permission_before_io() {
        let error = run_momentum_mamba_representation_cli_v3(
            Path::new("not-used"),
            "json",
            true,
            false,
            false,
            true,
        )
        .unwrap_err();
        assert_eq!(error, "Momentum Mamba representation V3 is offline-only");
    }

    fn momentum_raw_feature_cli_fixture_v4() -> MomentumRawFeatureCliReportV4 {
        MomentumRawFeatureCliReportV4 {
            report_version: "momentum-raw-feature-cli-report-v4",
            mode: "execute-local".to_string(),
            offline: true,
            status: crate::model::MomentumRawFeatureExecutionStatusV4::Executed,
            frozen_mamba_closure_status: Some(
                crate::model::MomentumFrozenMambaClosureDecisionV4::ClosedForCurrentEvidenceAndPolicy,
            ),
            frozen_mamba_closure_digest: Some("closure".to_string()),
            split_digest: Some("split".to_string()),
            registration_digest: Some("registration".to_string()),
            participants: vec![MomentumRawFeatureParticipantCliV4 {
                participant_id: "RawFeatureLogisticV4".to_string(),
                participant_role: crate::model::MomentumRawFeatureRoleV4::LearnedRawLogistic,
                model_kind: crate::model::MomentumRawFeatureModelKindV4::RawFeatureLogistic,
                qualification_status:
                    crate::model::MomentumRawFeatureQualificationStatusV4::QualifiedLearned,
            }],
            interaction_contribution_status: Some(
                crate::model::InteractionContributionStatusV4::LinearEquivalent,
            ),
            qualified_learned_count: 1,
            qualified_benchmark_count: 1,
            family_digest: Some("family".to_string()),
            path_decision: Some(
                crate::model::MomentumRawFeaturePathDecisionV4::OnlyLinearRawPathViable,
            ),
            decision_digest: Some("decision".to_string()),
            roster_status: crate::model::MomentumRawFeatureRosterStatusV4::Ready,
            roster_digest: Some("roster".to_string()),
            evaluation_registration_status:
                crate::model::MomentumRawFeatureEvaluationStatusV4::Registered,
            evaluation_registration_digest: Some("evaluation".to_string()),
            minimum_accepted_timestamp_ms: Some(105),
            cycle_risk_evidence_status: Some(
                crate::data::CanonicalViewGapStatusV1::ProviderContractUnverified,
            ),
            value_quality_evidence_status: Some(
                crate::data::CanonicalViewGapStatusV1::TrainerUnavailable,
            ),
            reward_eligibility_replay: persisted_intent_migration_cli_fixture_v1()
                .reward_eligibility_replay,
            artifacts_written: 15,
            duplicate_artifact_count: 0,
            storage_failure_count: 0,
            protected_artifacts_unchanged: true,
            active_state_unchanged: true,
            safety_counters: crate::model::MomentumRawFeatureSafetyCountersV4 {
                network_requests: 0,
                transport_constructions: 0,
                credential_reads: 0,
                prospective_row_reads: 0,
                prospective_label_openings: 0,
                historical_test_reads: 0,
                future_evaluation_reads: 0,
                final_reserve_row_reads: 0,
                final_reserve_label_reads: 0,
                active_model_changes: 0,
                chair_decisions: 0,
                votes: 0,
                reward_applications: 0,
                penalty_applications: 0,
                voice_changes: 0,
                cooldowns_started: 0,
                promotions: 0,
                quarantines: 0,
                executions: 0,
                active_committee_count: 3,
            },
            report_digest: "report".to_string(),
        }
    }

    #[test]
    fn momentum_raw_feature_text_and_json_public_fields_agree() {
        let report = momentum_raw_feature_cli_fixture_v4();
        let text = format_momentum_raw_feature_text_v4(&report);
        let json = serde_json::to_value(&report).unwrap();
        for field in [
            "network_requests",
            "transport_constructions",
            "credential_reads",
            "prospective_row_reads",
            "prospective_label_openings",
            "historical_test_reads",
            "future_evaluation_reads",
            "final_reserve_row_reads",
            "final_reserve_label_reads",
            "active_model_changes",
            "chair_decisions",
            "votes",
            "reward_applications",
            "penalty_applications",
            "voice_changes",
            "cooldowns_started",
            "promotions",
            "quarantines",
            "executions",
            "active_committee_count",
        ] {
            assert!(text.contains(&format!("{field}={}", json["safety_counters"][field])));
        }
        assert!(text.contains("participant_id=RawFeatureLogisticV4"));
        assert!(text.contains("path_decision=OnlyLinearRawPathViable"));
        assert_eq!(json["minimum_accepted_timestamp_ms"], 105);
        let rendered_json = serde_json::to_string(&report).unwrap();
        for forbidden in [
            "raw_rows",
            "raw_features",
            "expanded_feature_values",
            "probabilities",
            "labels",
            "private_metric_digest",
            "parameters",
            "gradients",
            "local_paths",
            "artifact_path",
        ] {
            assert!(!text.contains(forbidden));
            assert!(!rendered_json.contains(forbidden));
        }
    }

    #[test]
    fn momentum_raw_feature_rejects_network_permission_before_io() {
        let error = run_momentum_raw_feature_cli_v4(
            Path::new("not-used"),
            "json",
            true,
            false,
            false,
            true,
        )
        .unwrap_err();
        assert_eq!(error, "Momentum raw-feature V4 is offline-only");
    }

    fn persisted_intent_migration_cli_fixture_v1() -> PersistedLearningIntentMigrationCliReportV1 {
        PersistedLearningIntentMigrationCliReportV1 {
            report_version: "persisted-learning-intent-migration-cli-report-v1",
            mode: "dry-run".to_string(),
            offline: true,
            migration: crate::model::PersistedLearningIntentMigrationReportV1 {
                report_version: "persisted-learning-intent-migration-report-v1".to_string(),
                mode: crate::model::AgentPrivateLearningRunModeV0::DryRun,
                agent_id: "momentum_trend_fast".to_string(),
                blocker:
                    crate::model::PersistedIntentMigrationBlockerV1::LegacySessionNotSelfDescribing,
                first_failing_invariant: Some("intent_version".to_string()),
                status: crate::model::PersistedLearningIntentMigrationStatusV1::Migrated,
                legacy_session_digest: Some("legacy-session".to_string()),
                legacy_intent_digest: Some("legacy-intent".to_string()),
                canonical_gap_digest: Some("gap".to_string()),
                composite_registration_digest: Some("composite".to_string()),
                canonical_snapshot_digest: Some("snapshot".to_string()),
                canonical_intent_digest: Some("intent".to_string()),
                canonical_view_digest: Some("view".to_string()),
                policy_compatibility_proof_digest: Some("policy-proof".to_string()),
                migration_proof_digest: Some("migration-proof".to_string()),
                migration_journal_digest: Some("journal".to_string()),
                field_provenance_count: 16,
                required_evidence_complete: true,
                optional_evidence_unavailable: true,
                normal_validator_passed: true,
                normal_view_builder_passed: true,
                artifacts_written: 0,
                duplicate_artifact_count: 0,
                storage_failure_count: 0,
                protected_artifacts_unchanged: true,
                active_state_unchanged: true,
                safety_counters: crate::model::PersistedIntentMigrationSafetyCountersV1 {
                    active_committee_count: 3,
                    network_requests: 0,
                    transport_constructions: 0,
                    credential_reads: 0,
                    prospective_artifact_reads: 0,
                    prospective_label_reads: 0,
                    future_evaluation_reads: 0,
                    active_model_changes: 0,
                    chair_decisions: 0,
                    votes: 0,
                    rewards: 0,
                    penalties: 0,
                    voice_changes: 0,
                    promotions: 0,
                    executions: 0,
                },
                report_digest: "report".to_string(),
            },
            candidate_families: Vec::new(),
            evaluation_registrations: vec![MigratedEvaluationRegistrationCliV1 {
                agent_id: "momentum_trend_fast".to_string(),
                status: crate::model::CandidateEvaluationRegistrationStatusV1::QualificationBlocked,
                blocker_code: Some("validation_qualification_invalid".to_string()),
                registration_digest: None,
                exclusion_digest: None,
                minimum_accepted_timestamp_ms: None,
                participant_count: 0,
                historical_test_access_count: 0,
                maximum_requests: 0,
                maximum_concurrency: 0,
                maximum_retries: 0,
                labels_hidden_until_opening: false,
                probabilities_hidden_until_opening: false,
                one_time_opening_required: false,
                winner_selection_forbidden_before_opening: false,
                active_promotion_forbidden: false,
                reward_application_forbidden: false,
            }],
            reward_eligibility_replay: PersistedRewardEligibilityReplayCliV1 {
                opening_status: crate::model::ProspectiveOutcomeOpeningStatusV0::Opened,
                opening_attempt_count: 1,
                opened_event_count: 2,
                outcome_digests: vec!["outcome-a".to_string(), "outcome-b".to_string()],
                attribution_classes: vec![
                    crate::model::LearnedAbstentionAttributionV0::MissedMaterialOpportunity,
                    crate::model::LearnedAbstentionAttributionV0::CorrectUncertainty,
                ],
                eligibility_statuses: vec![
                    crate::model::LearnedRewardEligibilityStatusV0::IneligibleMinimumSamples,
                    crate::model::LearnedRewardEligibilityStatusV0::IneligibleMinimumSamples,
                ],
                eligibility_digests: vec!["eligibility-a".to_string(), "eligibility-b".to_string()],
                reward_candidate_count: 0,
                reward_apply_count: 0,
                penalty_apply_count: 0,
                voice_mutation_count: 0,
                authority_action_count: 0,
                replay_matches_persisted: true,
            },
            new_network_requests: 0,
            transport_constructions: 0,
            new_credential_reads: 0,
            new_prospective_row_reads: 0,
            new_prospective_label_openings: 0,
            new_future_evaluation_reads: 0,
            historical_test_reads_v1: 0,
            active_committee_count: 3,
            active_model_changes: 0,
            chair_decisions: 0,
            votes: 0,
            reward_applications: 0,
            penalty_applications: 0,
            voice_changes: 0,
            cooldowns_started: 0,
            promotions: 0,
            quarantines: 0,
            executions: 0,
        }
    }

    #[test]
    fn persisted_intent_migration_text_and_json_public_fields_agree() {
        let report = persisted_intent_migration_cli_fixture_v1();
        let text = format_persisted_intent_migration_text_v1(&report);
        let json = serde_json::to_value(&report).unwrap();
        for field in [
            "new_network_requests",
            "transport_constructions",
            "new_credential_reads",
            "new_prospective_row_reads",
            "new_prospective_label_openings",
            "new_future_evaluation_reads",
            "historical_test_reads_v1",
            "active_committee_count",
            "active_model_changes",
            "chair_decisions",
            "votes",
            "reward_applications",
            "penalty_applications",
            "voice_changes",
            "cooldowns_started",
            "promotions",
            "quarantines",
            "executions",
        ] {
            assert!(text.contains(&format!("{field}={}", json[field])));
        }
        for field in [
            "legacy_session_digest",
            "legacy_intent_digest",
            "canonical_gap_digest",
            "composite_registration_digest",
            "canonical_snapshot_digest",
            "canonical_intent_digest",
            "canonical_view_digest",
            "policy_compatibility_proof_digest",
            "migration_proof_digest",
            "migration_journal_digest",
        ] {
            let value = json["migration"][field].as_str().unwrap();
            assert!(text.contains(&format!("{field}={value}")));
        }
        assert!(text.contains("migration_status=Migrated"));
        assert!(text.contains("migration_blocker=LegacySessionNotSelfDescribing"));
        assert!(text.contains("opening_attempt_count=1"));
        assert!(text.contains("opened_event_count=2"));
        assert!(text.contains("minimum_accepted_timestamp_ms=;"));
        assert!(json["evaluation_registrations"][0]["minimum_accepted_timestamp_ms"].is_null());
        assert!(text.contains("reward_apply_count=0"));
        assert!(text.contains("penalty_apply_count=0"));
    }

    #[test]
    fn persisted_intent_migration_rejects_network_permission_before_io() {
        let error = run_persisted_learning_intent_migration_cli_v1(
            Path::new("not-used"),
            "json",
            true,
            false,
            false,
            true,
        )
        .unwrap_err();
        assert_eq!(error, "persisted learning intent migration is offline-only");
    }

    #[test]
    fn prospective_external_admission_text_and_json_use_the_same_public_fields() {
        let report = ProspectiveExternalAdmissionReportV0 {
            report_version: "prospective-external-row-admission-v0",
            offline: true,
            compatibility: "PermittedWithExternalAdmissionRegistration".into(),
            registration_digest: "registration".into(),
            source_classification: "AwaitingQualifiedExternalRow".into(),
            candidate_input_status: "NoQualifiedExternalCapsuleDiscovered".into(),
            admission_status: "AwaitingQualifiedExternalRow".into(),
            admitted_row_count: 0,
            shared_raw_evidence_reference_count: 0,
            shared_raw_evidence_digest: None,
            momentum_independently_valid: false,
            risk_independently_valid: false,
            momentum_event_count: 0,
            risk_event_count: 0,
            momentum_abstention_count: 0,
            risk_abstention_count: 0,
            maturity_status: "NoSealedEvents".into(),
            reward_eligibility: "IneligibleNoProspectiveOutcomes".into(),
            reward_candidate_count: 0,
            reward_apply_count: 0,
            provider_calls: 0,
            transport_constructions: 0,
            network_consent_reads: 0,
            credential_reads: 0,
            label_reads: 0,
            chair_decision_count: 0,
            reward_applied_count: 0,
            penalty_applied_count: 0,
            voice_mutation_count: 0,
            cooldown_mutation_count: 0,
            promotion_mutation_count: 0,
            quarantine_mutation_count: 0,
            execution_count: 0,
        };
        let json = serde_json::to_value(&report).unwrap();
        assert_eq!(
            json["admission_status"].as_str(),
            Some(report.admission_status.as_str())
        );
        assert_eq!(
            json["registration_digest"].as_str(),
            Some(report.registration_digest.as_str())
        );
        assert_eq!(
            json["provider_calls"].as_u64(),
            Some(report.provider_calls as u64)
        );
        assert_eq!(
            json["execution_count"].as_u64(),
            Some(report.execution_count as u64)
        );
    }

    #[test]
    fn prospective_public_export_status_keeps_network_authority_and_label_counters_explicit() {
        let report = ProspectivePublicExportReportV0 {
            report_version: "prospective-public-export-acquisition-v0",
            mode: "dry-run",
            registration_digest: "registration".into(),
            registration_reopened_and_verified: true,
            request_fingerprint: "fingerprint".into(),
            request_to_utc: "2024-01-03T00:00:00Z".into(),
            explicit_single_request_consent: false,
            request_attempted: false,
            request_count: 0,
            retry_count: 0,
            http_status_class: None,
            returned_item_count: 0,
            acquisition_outcome: "DryRunReady".into(),
            acquisition_receipt_digest: None,
            network_capsule_created: false,
            network_capsule_digest: None,
            admission_status: "NotAttempted".into(),
            shared_raw_evidence_reference_count: 0,
            momentum_event_count: 0,
            risk_event_count: 0,
            maturity_status: "NoSealedEvents".into(),
            reward_eligibility: "IneligibleMinimumSamples".into(),
            prospective_label_reads: 0,
            mature_outcomes: 0,
            interim_metrics: 0,
            reward_candidate_count: 0,
            reward_apply_count: 0,
            network_request_count: 0,
            authority_action_count: 0,
            legacy_blind_receipt_unchanged: true,
            legacy_request_registry_unchanged: true,
        };
        let json = serde_json::to_value(&report).unwrap();
        assert_eq!(json["request_count"].as_u64(), Some(0));
        assert_eq!(json["network_request_count"].as_u64(), Some(0));
        assert_eq!(json["prospective_label_reads"].as_u64(), Some(0));
        assert_eq!(json["authority_action_count"].as_u64(), Some(0));
        assert_eq!(
            json["registration_reopened_and_verified"].as_bool(),
            Some(true)
        );
    }

    #[test]
    fn prospective_maturity_preflight_text_and_json_share_zero_counter_fields() {
        let report = ProspectiveEventMaturityPreflightReportV0 {
            report_version: "prospective-event-maturity-preflight-v0",
            offline: true,
            opening_registration_digest: "registration".into(),
            opening_registration_reopened_and_verified: true,
            event_count: 2,
            event_integrity_valid: true,
            momentum_required_outcome_row_count: 1,
            risk_required_outcome_row_count: 4,
            required_outcome_row_count: 4,
            momentum_time_boundary_reached: false,
            risk_time_boundary_reached: false,
            momentum_opening_readiness: "AwaitingTimeMaturity".into(),
            risk_opening_readiness: "AwaitingTimeMaturity".into(),
            outcome_evidence_status: "NoOutcomeRows".into(),
            opening_readiness: "AwaitingTimeMaturity".into(),
            label_open_count: 0,
            reward_eligibility: "IneligibleAwaitingMaturity".into(),
            provider_calls: 0,
            transport_constructions: 0,
            network_consent_reads: 0,
            credential_reads: 0,
            outcome_row_reads: 0,
            prospective_label_reads: 0,
            metric_computations: 0,
            reward_candidate_count: 0,
            reward_apply_count: 0,
            penalties_applied: 0,
            chair_observed: false,
            chair_decisions_created: 0,
            votes_created: 0,
            voice_changes: 0,
            cooldowns_started: 0,
            promotions_created: 0,
            quarantines_created: 0,
            risk_handoffs: 0,
            executions_created: 0,
            sprint65_artifacts_unchanged: true,
        };
        let json = serde_json::to_value(&report).unwrap();
        assert_eq!(json["event_count"].as_u64(), Some(2));
        assert_eq!(json["required_outcome_row_count"].as_u64(), Some(4));
        assert_eq!(json["provider_calls"].as_u64(), Some(0));
        assert_eq!(json["prospective_label_reads"].as_u64(), Some(0));
        assert_eq!(json["metric_computations"].as_u64(), Some(0));
        assert_eq!(json["executions_created"].as_u64(), Some(0));
        assert_eq!(json["sprint65_artifacts_unchanged"].as_bool(), Some(true));
    }

    #[test]
    fn prospective_outcome_status_replays_persisted_receipt_fields_without_network() {
        let plan = crate::data::ProspectiveOutcomeAcquisitionPlanV0 {
            plan_version: "prospective-outcome-acquisition-plan-v0".into(),
            opening_registration_digest: "registration".into(),
            momentum_event_digest: "momentum".into(),
            risk_event_digest: "risk".into(),
            required_timestamps: vec![1, 2, 3, 4],
            required_row_count: 4,
            request_to_utc: "2026-07-22T00:00:00Z".into(),
            request_count: 4,
            provider_id: "upbit".into(),
            market: "KRW-BTC".into(),
            cadence: "1d".into(),
            maximum_requests: 1,
            maximum_retries: 0,
            maximum_concurrency: 1,
            readiness: crate::model::ProspectiveOutcomeRequestReadinessV0::RequestAlreadyAttempted,
            plan_digest: "current-plan".into(),
        };
        let receipt = crate::data::ProspectiveOutcomeAcquisitionReceiptV0 {
            receipt_version: "prospective-outcome-acquisition-receipt-v0".into(),
            opening_registration_digest: "registration".into(),
            plan_digest: "request-plan".into(),
            request_fingerprint: "request-fingerprint".into(),
            request_attempted: true,
            request_count: 1,
            retry_count: 0,
            readiness_before_request:
                crate::model::ProspectiveOutcomeRequestReadinessV0::ReadyForExplicitRequest,
            status: crate::data::ProspectiveOutcomeAcquisitionStatusV0::EvidenceAcquired,
            http_status_class: Some("2xx".into()),
            returned_row_count: 4,
            verified_row_count: 4,
            outcome_capsule_digest: Some("capsule".into()),
            receipt_digest: "receipt".into(),
        };

        let replay = prospective_outcome_receipt_replay_v0(
            &plan,
            true,
            crate::data::ProspectiveOutcomeAcquisitionStatusV0::RequestBudgetExhausted,
            Some(&receipt),
        )
        .unwrap();

        assert_eq!(replay.plan_digest, receipt.plan_digest);
        assert_eq!(
            replay.request_fingerprint.as_deref(),
            Some(receipt.request_fingerprint.as_str())
        );
        assert_eq!(replay.acquisition_status, receipt.status);
        assert_eq!(replay.request_count, 1);
        assert_eq!(replay.retry_count, 0);
        assert_eq!(replay.http_status_class.as_deref(), Some("2xx"));
        assert_eq!(replay.returned_row_count, 4);
        assert_eq!(replay.verified_row_count, 4);
        assert_eq!(plan.request_count, 4);
        let row = crate::model::CanonicalHistoricalRowIdentityV1 {
            provider_id: "upbit".into(),
            series_id: "BtcCrypto:KRW-BTC".into(),
            timestamp_ms: 1,
            open_bits: 1.0_f64.to_bits(),
            high_bits: 1.0_f64.to_bits(),
            low_bits: 1.0_f64.to_bits(),
            close_bits: 1.0_f64.to_bits(),
            volume_bits: 0.0_f64.to_bits(),
            trade_value_bits: Some(0.0_f64.to_bits()),
            row_digest_v1: "row".into(),
        };
        let capsule = crate::data::ProspectiveOutcomeEvidenceCapsuleV0 {
            capsule_version: "prospective-outcome-evidence-capsule-v0".into(),
            opening_registration_digest: "registration".into(),
            acquisition_receipt_digest: "receipt".into(),
            provider_id: "upbit".into(),
            market: "KRW-BTC".into(),
            cadence: "1d".into(),
            canonical_rows: vec![row; 4],
            canonical_row_digests: vec!["row".into(); 4],
            first_timestamp: 1,
            last_timestamp: 4,
            complete_registered_range: true,
            finalized: true,
            read_only: true,
            sanitized: true,
            credential_free: true,
            labels_opened: false,
            capsule_digest: "capsule".into(),
        };
        assert!(prospective_outcome_stored_result_chain_valid_v0(
            Some(&receipt),
            Some(&capsule)
        ));
        assert!(!prospective_outcome_stored_result_chain_valid_v0(
            Some(&receipt),
            None
        ));
        assert!(!prospective_outcome_stored_result_chain_valid_v0(
            None,
            Some(&capsule)
        ));
        let mut mismatched_capsule_digest = capsule.clone();
        mismatched_capsule_digest.capsule_digest = "other-capsule".into();
        assert!(!prospective_outcome_stored_result_chain_valid_v0(
            Some(&receipt),
            Some(&mismatched_capsule_digest)
        ));
        let mut mismatched_capsule = capsule;
        mismatched_capsule.acquisition_receipt_digest = "other-receipt".into();
        assert!(!prospective_outcome_stored_result_chain_valid_v0(
            Some(&receipt),
            Some(&mismatched_capsule)
        ));
        let mut failed_receipt = receipt;
        failed_receipt.status = crate::data::ProspectiveOutcomeAcquisitionStatusV0::TimeoutNoRetry;
        failed_receipt.http_status_class = None;
        failed_receipt.returned_row_count = 0;
        failed_receipt.verified_row_count = 0;
        failed_receipt.outcome_capsule_digest = None;
        assert!(prospective_outcome_stored_result_chain_valid_v0(
            Some(&failed_receipt),
            None
        ));
    }
}
