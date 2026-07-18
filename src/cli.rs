use std::{
    collections::BTreeSet,
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
    pub dry_run: bool,
    #[arg(long, default_value_t = false)]
    pub execute: bool,
    #[arg(long, default_value_t = false)]
    pub confirm_single_public_candle_request: bool,
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
    if args.toss_historical_contract_report {
        return print_toss_historical_contract_report(
            args.toss_kr_historical_manifest,
            args.toss_us_historical_manifest,
        );
    }
    if let Some(config) = args.historical_snapshot_campaign_config {
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
}
