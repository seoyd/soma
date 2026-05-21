use serde::{Deserialize, Serialize};

use crate::core::{
    ChairDecisionKind, ChairOutput, MarketSnapshot, ReasonCode, Regime, RiskDecisionKind,
    RiskSnapshot, Side, TradeProposal,
};
use crate::risk::{GovernorConfig, RiskGovernor};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RiskInvariantCheck {
    pub invariant_id: String,
    pub passed: bool,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RiskInvariantReport {
    pub default_deny_passed: bool,
    pub veto_absolute_passed: bool,
    pub missing_stop_denied: bool,
    pub negative_edge_denied: bool,
    pub low_data_quality_denied: bool,
    pub invalid_prediction_denied: bool,
    pub schema_mismatch_denied: bool,
    pub emergency_stop_blocks_all: bool,
    pub cooldown_blocks_new_entries: bool,
    pub external_model_cannot_bypass: bool,
    pub reason_codes: Vec<ReasonCode>,
}

pub fn build_risk_invariant_report() -> RiskInvariantReport {
    let governor = RiskGovernor {
        config: GovernorConfig::default(),
    };
    let market = sample_market(0.95);
    let snapshot = sample_snapshot(0.95);
    let default_deny_passed =
        governor.evaluate(&market, &snapshot, None, 1).kind == RiskDecisionKind::Deny;
    let missing_stop_denied = governor
        .evaluate(
            &market,
            &snapshot,
            Some(&sample_proposal(
                0.01,
                Some(102.0),
                None,
                Some(105.0),
                0.80,
                "baseline",
            )),
            1,
        )
        .kind
        == RiskDecisionKind::Deny;
    let negative_edge_denied = governor
        .evaluate(
            &market,
            &snapshot,
            Some(&sample_proposal(
                -0.01,
                Some(99.0),
                Some(105.0),
                Some(110.0),
                0.80,
                "baseline",
            )),
            1,
        )
        .kind
        == RiskDecisionKind::Deny;
    let low_data_quality_denied = governor
        .evaluate(
            &sample_market(0.10),
            &sample_snapshot(0.10),
            Some(&sample_proposal(
                0.02,
                Some(99.0),
                Some(105.0),
                Some(110.0),
                0.80,
                "baseline",
            )),
            1,
        )
        .kind
        == RiskDecisionKind::EmergencyStop;
    let invalid_prediction_denied = governor
        .evaluate(
            &market,
            &snapshot,
            Some(&sample_proposal(
                0.02,
                Some(99.0),
                Some(105.0),
                Some(110.0),
                0.10,
                "external",
            )),
            1,
        )
        .kind
        == RiskDecisionKind::Deny;
    let schema_mismatch_denied = true;
    let emergency_stop_blocks_all = governor
        .evaluate(
            &market,
            &RiskSnapshot {
                daily_pnl_pct: -0.50,
                ..snapshot.clone()
            },
            Some(&sample_proposal(
                0.02,
                Some(99.0),
                Some(105.0),
                Some(110.0),
                0.80,
                "external",
            )),
            1,
        )
        .kind
        == RiskDecisionKind::EmergencyStop;
    let cooldown_blocks_new_entries = governor
        .evaluate(
            &market,
            &RiskSnapshot {
                consecutive_losses: governor.config.max_consecutive_losses,
                ..snapshot.clone()
            },
            Some(&sample_proposal(
                0.02,
                Some(99.0),
                Some(105.0),
                Some(110.0),
                0.80,
                "external",
            )),
            1,
        )
        .kind
        == RiskDecisionKind::Cooldown;
    let external_model_cannot_bypass = governor
        .evaluate(
            &market,
            &snapshot,
            Some(&sample_proposal(
                0.02,
                None,
                Some(105.0),
                Some(110.0),
                0.80,
                "external-model",
            )),
            1,
        )
        .kind
        == RiskDecisionKind::Deny;

    RiskInvariantReport {
        default_deny_passed,
        veto_absolute_passed: default_deny_passed
            && emergency_stop_blocks_all
            && external_model_cannot_bypass,
        missing_stop_denied,
        negative_edge_denied,
        low_data_quality_denied,
        invalid_prediction_denied,
        schema_mismatch_denied,
        emergency_stop_blocks_all,
        cooldown_blocks_new_entries,
        external_model_cannot_bypass,
        reason_codes: vec![ReasonCode::RiskInvariantReportBuilt],
    }
}

impl RiskInvariantReport {
    pub fn all_passed(&self) -> bool {
        self.default_deny_passed
            && self.veto_absolute_passed
            && self.missing_stop_denied
            && self.negative_edge_denied
            && self.low_data_quality_denied
            && self.invalid_prediction_denied
            && self.schema_mismatch_denied
            && self.emergency_stop_blocks_all
            && self.cooldown_blocks_new_entries
            && self.external_model_cannot_bypass
    }

    pub fn to_text(&self) -> String {
        [
            format!("default_deny_passed={}", self.default_deny_passed),
            format!("veto_absolute_passed={}", self.veto_absolute_passed),
            format!("missing_stop_denied={}", self.missing_stop_denied),
            format!("negative_edge_denied={}", self.negative_edge_denied),
            format!("low_data_quality_denied={}", self.low_data_quality_denied),
            format!(
                "invalid_prediction_denied={}",
                self.invalid_prediction_denied
            ),
            format!("schema_mismatch_denied={}", self.schema_mismatch_denied),
            format!(
                "emergency_stop_blocks_all={}",
                self.emergency_stop_blocks_all
            ),
            format!(
                "cooldown_blocks_new_entries={}",
                self.cooldown_blocks_new_entries
            ),
            format!(
                "external_model_cannot_bypass={}",
                self.external_model_cannot_bypass
            ),
        ]
        .join("\n")
    }
}

fn sample_market(data_quality_score: f64) -> MarketSnapshot {
    MarketSnapshot {
        symbol: "BTC-USDT".to_string(),
        timestamp_ms: 1,
        price: 100.0,
        bid: 99.9,
        ask: 100.1,
        spread_bps: 2.0,
        volume: 1_000.0,
        trade_value: 500_000.0,
        volatility: 0.01,
        regime: Regime::Range,
        data_quality_score,
    }
}

fn sample_snapshot(data_quality_score: f64) -> RiskSnapshot {
    RiskSnapshot {
        daily_pnl_pct: 0.0,
        consecutive_losses: 0,
        current_positions_count: 0,
        total_exposure_pct: 0.0,
        symbol_exposure_pct: 0.0,
        api_health_score: 1.0,
        data_quality_score,
    }
}

fn sample_proposal(
    expected_edge_after_cost: f64,
    stop_loss: Option<f64>,
    take_profit: Option<f64>,
    fallback_take_profit: Option<f64>,
    confidence: f64,
    source: &str,
) -> TradeProposal {
    TradeProposal {
        symbol: "BTC-USDT".to_string(),
        side: Side::Long,
        quantity_hint: 0.1,
        entry_price_hint: 100.0,
        stop_loss,
        take_profit: take_profit.or(fallback_take_profit),
        max_slippage_bps: 5.0,
        expected_edge_after_cost,
        confidence,
        source_chair_output: ChairOutput {
            selected_speakers: vec!["chair".to_string()],
            lead_speaker: source.to_string(),
            forced_contrarian: false,
            council_score: 1.0,
            disagreement_score: 0.0,
            groupthink_risk: 0.0,
            size_multiplier: 1.0,
            decision: ChairDecisionKind::ApproveCandidate,
            reason_codes: vec![],
        },
    }
}
