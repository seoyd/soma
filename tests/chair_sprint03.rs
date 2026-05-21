use std::fs;
use std::path::PathBuf;

use soma_zero::chair::ChairEngine;
use soma_zero::core::{
    ChairDecisionKind, ChairInput, MarketSnapshot, ReasonCode, Regime, RiskDecisionKind,
    RiskSnapshot, SignalOutput,
};
use soma_zero::league::{CycleRiskSkeptic, MomentumTrendFast, Persona, ValueQualityFilter};
use soma_zero::risk::RiskGovernor;

fn market() -> MarketSnapshot {
    MarketSnapshot {
        symbol: "EURUSD".to_string(),
        timestamp_ms: 1_715_000_000_000,
        price: 100.0,
        bid: 99.99,
        ask: 100.01,
        spread_bps: 2.0,
        volume: 15_000.0,
        trade_value: 1_500_000.0,
        volatility: 0.012,
        regime: Regime::TrendUp,
        data_quality_score: 0.98,
    }
}

fn signal() -> SignalOutput {
    SignalOutput {
        symbol: "EURUSD".to_string(),
        horizon_bars: 8,
        p_win: 0.65,
        p_stop: 0.28,
        expected_return: 0.015,
        expected_drawdown: 0.005,
        confidence: 0.80,
        no_trade_probability: 0.18,
        source: "test".to_string(),
    }
}

fn risk_snapshot() -> RiskSnapshot {
    RiskSnapshot {
        daily_pnl_pct: 0.0,
        consecutive_losses: 0,
        current_positions_count: 0,
        total_exposure_pct: 0.0,
        symbol_exposure_pct: 0.0,
        api_health_score: 1.0,
        data_quality_score: 0.98,
    }
}

#[test]
fn chair_forces_contrarian_inclusion_when_votes_are_one_sided() {
    let chair = ChairEngine::default();
    let signal_output = signal();
    let mut lead = MomentumTrendFast::default().vote(&market(), &signal_output);
    lead.voice_power = 0.72;
    lead.conviction = 0.88;
    let mut wingman = lead.clone();
    wingman.persona_id = "momentum_shadow".to_string();
    wingman.voice_power = 0.58;
    wingman.conviction = 0.76;
    let mut contrarian = CycleRiskSkeptic::default().vote(&market(), &signal_output);
    contrarian.persona_id = "cycle_shadow".to_string();
    contrarian.voice_power = 0.26;
    contrarian.conviction = 0.42;

    let output = chair.evaluate(&ChairInput {
        market: market(),
        signal: signal_output,
        votes: vec![lead, wingman, contrarian],
        full_auto: false,
    });
    assert!(output.forced_contrarian);
}

#[test]
fn chair_applies_cluster_penalty_and_groupthink_risk() {
    let chair = ChairEngine::default();
    let signal_output = signal();
    let mut clone_vote = MomentumTrendFast::default().vote(&market(), &signal_output);
    clone_vote.persona_id = "momentum_clone".to_string();
    clone_vote.cluster_id = "trend".to_string();
    clone_vote.voice_power = 0.8;
    clone_vote.conviction = 0.9;

    let output = chair.evaluate(&ChairInput {
        market: market(),
        signal: signal_output.clone(),
        votes: vec![
            MomentumTrendFast::default().vote(&market(), &signal_output),
            clone_vote,
            CycleRiskSkeptic::default().vote(&market(), &signal_output),
        ],
        full_auto: false,
    });
    assert!(
        output
            .reason_codes
            .contains(&ReasonCode::ClusterPenaltyApplied)
    );
    assert!(output.groupthink_risk > 0.0);
}

#[test]
fn chair_filters_out_wrong_horizon_personas() {
    let chair = ChairEngine::default();
    let output = chair.evaluate(&ChairInput {
        market: market(),
        signal: signal(),
        votes: vec![ValueQualityFilter::default().vote(&market(), &signal())],
        full_auto: false,
    });
    assert!(output.reason_codes.contains(&ReasonCode::HorizonFiltered));
}

#[test]
fn chair_converts_require_confirm_to_no_trade_in_full_auto() {
    let chair = ChairEngine::default();
    let mut confirm_signal = signal();
    confirm_signal.expected_return = 0.0035;
    confirm_signal.confidence = 0.53;
    confirm_signal.no_trade_probability = 0.24;
    let mut trend_vote = MomentumTrendFast::default().vote(&market(), &confirm_signal);
    trend_vote.voice_power = 0.60;
    trend_vote.conviction = 0.80;
    let mut skeptic_vote = CycleRiskSkeptic { base_voice: 0.10 }.vote(&market(), &confirm_signal);
    skeptic_vote.voice_power = 0.22;
    skeptic_vote.conviction = 0.38;
    skeptic_vote.risk_penalty = 0.12;

    let output = chair.evaluate(&ChairInput {
        market: market(),
        signal: confirm_signal,
        votes: vec![trend_vote, skeptic_vote],
        full_auto: true,
    });
    assert_eq!(output.decision, ChairDecisionKind::NoTrade);
    assert!(
        output
            .reason_codes
            .contains(&ReasonCode::RequireConfirmBlockedInAuto)
    );
}

#[test]
fn risk_governor_still_denies_even_if_chair_approves() {
    let chair = ChairEngine::default();
    let signal = signal();
    let output = chair.evaluate(&ChairInput {
        market: market(),
        signal: signal.clone(),
        votes: vec![
            MomentumTrendFast::default().vote(&market(), &signal),
            CycleRiskSkeptic { base_voice: 0.05 }.vote(&market(), &signal),
        ],
        full_auto: false,
    });
    let mut proposal = chair
        .build_trade_proposal(&market(), &signal, &output)
        .expect("proposal");
    proposal.expected_edge_after_cost = 0.0;

    let governor = RiskGovernor::default();
    let decision = governor.evaluate(
        &market(),
        &risk_snapshot(),
        Some(&proposal),
        market().timestamp_ms,
    );
    assert_eq!(decision.kind, RiskDecisionKind::Deny);
}

#[test]
fn legacy_files_are_not_in_active_live_path() {
    let cargo_toml = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let contents = fs::read_to_string(cargo_toml).expect("cargo toml");
    for crate_name in [
        "soma-ssm",
        "soma-gdn",
        "soma-attn",
        "soma-mor",
        "soma-memory",
        "soma-online",
        "soma-adapt",
    ] {
        assert!(!contents.contains(&format!("\"{crate_name}\"")));
    }
}
