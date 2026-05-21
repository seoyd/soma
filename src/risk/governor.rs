use serde::{Deserialize, Serialize};

use crate::core::{MarketSnapshot, build_audit_event};
use crate::core::{
    OrderPlan, ReasonCode, Regime, RiskDecision, RiskDecisionKind, RiskSnapshot, Side,
    TradeProposal,
};

use super::gates::{projected_total_exposure, risk_reward_ratio};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct GovernorConfig {
    pub max_daily_loss_pct: f64,
    pub max_consecutive_losses: u32,
    pub min_expected_edge: f64,
    pub min_confidence: f64,
    pub max_spread_bps: f64,
    pub min_data_quality: f64,
    pub min_api_health: f64,
    pub max_allowed_volatility: f64,
    pub min_risk_reward: f64,
    pub max_total_exposure: f64,
    pub max_symbol_exposure: f64,
    pub min_trade_value: f64,
}

impl Default for GovernorConfig {
    fn default() -> Self {
        Self {
            max_daily_loss_pct: 0.03,
            max_consecutive_losses: 3,
            min_expected_edge: 0.001,
            min_confidence: 0.55,
            max_spread_bps: 12.0,
            min_data_quality: 0.80,
            min_api_health: 0.80,
            max_allowed_volatility: 0.04,
            min_risk_reward: 1.5,
            max_total_exposure: 1.0,
            max_symbol_exposure: 0.35,
            min_trade_value: 250_000.0,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct RiskGovernor {
    pub config: GovernorConfig,
}

impl RiskGovernor {
    pub fn evaluate(
        &self,
        market: &MarketSnapshot,
        snapshot: &RiskSnapshot,
        proposal: Option<&TradeProposal>,
        timestamp_ms: u64,
    ) -> RiskDecision {
        let audit =
            |kind: RiskDecisionKind, reasons: Vec<ReasonCode>, approved: Option<OrderPlan>| {
                let summary = format!("{kind:?}:{:?}", reasons);
                let event = build_audit_event(
                    timestamp_ms,
                    crate::core::AuditEventType::RiskEvaluated,
                    &summary,
                    &summary,
                    reasons.clone(),
                    std::collections::BTreeMap::from([
                        ("daily_pnl_pct".to_string(), snapshot.daily_pnl_pct),
                        (
                            "expected_edge".to_string(),
                            proposal.map_or(0.0, |p| p.expected_edge_after_cost),
                        ),
                        ("spread_bps".to_string(), market.spread_bps),
                    ]),
                );
                RiskDecision {
                    kind,
                    approved_order_plan: approved,
                    reason_codes: reasons,
                    audit_id: format!("audit-{:#016x}", event.input_hash),
                }
            };

        if snapshot.daily_pnl_pct <= -self.config.max_daily_loss_pct {
            return audit(
                RiskDecisionKind::EmergencyStop,
                vec![ReasonCode::DailyLossGateBreached],
                None,
            );
        }
        if snapshot.api_health_score < self.config.min_api_health * 0.5 {
            return audit(
                RiskDecisionKind::EmergencyStop,
                vec![ReasonCode::ApiHealthGateBreached],
                None,
            );
        }
        let combined_quality = market.data_quality_score.min(snapshot.data_quality_score);
        if combined_quality < self.config.min_data_quality * 0.5 {
            return audit(
                RiskDecisionKind::EmergencyStop,
                vec![ReasonCode::DataQualityGateBreached],
                None,
            );
        }
        if snapshot.consecutive_losses >= self.config.max_consecutive_losses {
            return audit(
                RiskDecisionKind::Cooldown,
                vec![ReasonCode::ConsecutiveLossGateBreached],
                None,
            );
        }

        let Some(proposal) = proposal else {
            return audit(
                RiskDecisionKind::Deny,
                vec![ReasonCode::DeniedByDefault, ReasonCode::NoTradePreferred],
                None,
            );
        };

        let mut reasons = Vec::new();
        if proposal.side == Side::Short {
            reasons.push(ReasonCode::ShortSellingDisabled);
        }
        if proposal.expected_edge_after_cost <= 0.0 {
            reasons.push(ReasonCode::ExpectedEdgeNonPositive);
        } else if proposal.expected_edge_after_cost <= self.config.min_expected_edge {
            reasons.push(ReasonCode::ExpectedEdgeBelowThreshold);
        }
        if proposal.stop_loss.is_none() {
            reasons.push(ReasonCode::MissingStopLoss);
        }
        if proposal.take_profit.is_none() {
            reasons.push(ReasonCode::MissingTakeProfit);
        }
        if proposal.confidence < self.config.min_confidence {
            reasons.push(ReasonCode::ConfidenceGateBreached);
        }
        if market.spread_bps > self.config.max_spread_bps {
            reasons.push(ReasonCode::SpreadGateBreached);
        }
        if market.trade_value < self.config.min_trade_value {
            reasons.push(ReasonCode::LiquidityGateBreached);
        }
        if combined_quality < self.config.min_data_quality {
            reasons.push(ReasonCode::DataQualityGateBreached);
        }
        if snapshot.api_health_score < self.config.min_api_health {
            reasons.push(ReasonCode::ApiHealthGateBreached);
        }
        if matches!(market.regime, Regime::Unknown) {
            reasons.push(ReasonCode::UnknownRegimeGateBreached);
        }
        if market.volatility > self.config.max_allowed_volatility && proposal.quantity_hint > 0.5 {
            reasons.push(ReasonCode::VolatilityShockGateBreached);
        }
        let Some(risk_reward) = risk_reward_ratio(proposal) else {
            reasons.push(ReasonCode::RiskRewardGateBreached);
            return audit(RiskDecisionKind::Deny, reasons, None);
        };
        if risk_reward < self.config.min_risk_reward {
            reasons.push(ReasonCode::RiskRewardGateBreached);
        }
        if projected_total_exposure(snapshot.total_exposure_pct, proposal.quantity_hint)
            > self.config.max_total_exposure
        {
            reasons.push(ReasonCode::TotalExposureGateBreached);
        }
        if projected_total_exposure(snapshot.symbol_exposure_pct, proposal.quantity_hint)
            > self.config.max_symbol_exposure
        {
            reasons.push(ReasonCode::SymbolExposureGateBreached);
        }

        if !reasons.is_empty() {
            return audit(RiskDecisionKind::Deny, reasons, None);
        }

        let approved = OrderPlan {
            symbol: proposal.symbol.clone(),
            side: proposal.side,
            quantity: proposal.quantity_hint,
            entry_price: proposal.entry_price_hint,
            stop_loss: proposal.stop_loss.expect("checked"),
            take_profit: proposal.take_profit.expect("checked"),
            max_slippage_bps: proposal.max_slippage_bps,
            paper_only: true,
        };
        audit(
            RiskDecisionKind::ApprovePaper,
            vec![ReasonCode::ApprovePaperOnly, ReasonCode::PaperExecutionOnly],
            Some(approved),
        )
    }
}
