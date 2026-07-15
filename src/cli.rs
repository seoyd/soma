use std::{fs, path::PathBuf};

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
    #[arg(long, default_value_t = false)]
    pub allow_network: bool,
}

pub fn run() -> Result<(), String> {
    let args = CliArgs::parse();
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
            let snapshot: crate::data::DataSnapshot = serde_json::from_slice(
                &fs::read(path).map_err(|_| "local snapshot reread failed".to_string())?,
            )
            .map_err(|_| "local snapshot decode failed".to_string())?;
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
            let sufficiency = crate::model::assess_momentum_campaign_sufficiency_v0(
                snapshot.row_count,
                &campaign_config,
            )
            .map_err(|_| "momentum campaign sufficiency calculation failed".to_string())?;
            println!("campaign_sufficient={}", sufficiency.sufficient);
            println!("campaign_possible_windows={}", sufficiency.possible_windows);
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
