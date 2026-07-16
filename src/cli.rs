use std::{
    fs,
    path::{Path, PathBuf},
};

use clap::Parser;

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
            args.allow_network,
        );
    }
    if args.momentum_temporal_diagnostics
        || args.momentum_cross_market_report
        || args.btc_multi_regime_report
        || args.btc_cross_regime_diagnostics
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
