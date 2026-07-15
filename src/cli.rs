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
    pub toss_historical_contract_report: bool,
    #[arg(long, default_value = "text", value_parser = ["text", "json"])]
    pub output_format: String,
    #[arg(long, default_value_t = false)]
    pub allow_network: bool,
}

pub fn run() -> Result<(), String> {
    let args = CliArgs::parse();
    if args.toss_historical_contract_report {
        return print_toss_historical_contract_report();
    }
    if let Some(config) = args.historical_snapshot_campaign_config {
        return run_local_historical_snapshot_campaign(
            &config,
            args.momentum_temporal_diagnostics || args.momentum_cross_market_report,
            &args.output_format,
            args.momentum_cross_market_report,
        );
    }
    if args.momentum_temporal_diagnostics || args.momentum_cross_market_report {
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
    if snapshot_paths.len() != 1 {
        return Err("local historical campaign requires exactly one snapshot".to_string());
    }
    let snapshot = crate::data::read_local_snapshot_protobuf_v1(&snapshot_paths[0])?;
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

fn print_toss_historical_contract_report() -> Result<(), String> {
    let korean = crate::toss::qualify_toss_historical_capability_v0(
        crate::toss::TossHistoricalCapabilityV0::KoreanEquityDailyOhlcv,
    );
    let us = crate::toss::qualify_toss_historical_capability_v0(
        crate::toss::TossHistoricalCapabilityV0::UsEquityDailyOhlcv,
    );
    println!(
        "{}",
        serde_json::json!({
            "report_version": "toss-historical-contract-qualification-v1",
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
