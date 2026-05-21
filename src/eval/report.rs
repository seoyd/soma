use serde::{Deserialize, Serialize};

use crate::backtest::Timeframe;
use crate::core::ReasonCode;

use super::feature_schema::FeatureSchema;
use super::leakage::LeakageReport;
use super::metrics::{
    CalibrationMetrics, ChairMetrics, DecisionMetrics, NoTradeMetrics, PersonaFoldMetrics,
    RegimeMetrics, RiskGovernorMetrics, TradeMetrics,
};
use super::walk_forward::{WalkForwardConfig, WalkForwardFold};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FoldReport {
    pub fold_id: usize,
    pub fold: WalkForwardFold,
    pub train_rows: usize,
    pub validation_rows: usize,
    pub test_rows: usize,
    pub leakage_report: LeakageReport,
    pub test_trade_metrics: TradeMetrics,
    pub test_decision_metrics: DecisionMetrics,
    pub test_no_trade_metrics: NoTradeMetrics,
    pub test_risk_metrics: RiskGovernorMetrics,
    pub calibration_metrics: CalibrationMetrics,
    pub regime_metrics: Vec<RegimeMetrics>,
    pub persona_metrics: Vec<PersonaFoldMetrics>,
    pub chair_metrics: ChairMetrics,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WalkForwardAggregateMetrics {
    pub trade_metrics: TradeMetrics,
    pub decision_metrics: DecisionMetrics,
    pub no_trade_metrics: NoTradeMetrics,
    pub risk_metrics: RiskGovernorMetrics,
    pub calibration_metrics: CalibrationMetrics,
    pub regime_metrics: Vec<RegimeMetrics>,
    pub persona_metrics: Vec<PersonaFoldMetrics>,
    pub chair_metrics: ChairMetrics,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WalkForwardReport {
    pub symbol: String,
    pub timeframe: Timeframe,
    pub config: WalkForwardConfig,
    pub folds: Vec<FoldReport>,
    pub aggregate_metrics: WalkForwardAggregateMetrics,
    pub feature_schema: FeatureSchema,
    pub reason_codes: Vec<ReasonCode>,
}
