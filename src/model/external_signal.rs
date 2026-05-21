use serde::{Deserialize, Serialize};

use crate::backtest::Timeframe;
use crate::core::SignalOutput;

use super::prediction::PredictionFrame;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvaluationMode {
    BaselineSignal,
    ExternalPrediction,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExternalPredictionSignalConfig {
    pub strict_validation: bool,
    pub min_confidence: Option<f64>,
    pub fallback_horizon_bars: u32,
}

impl Default for ExternalPredictionSignalConfig {
    fn default() -> Self {
        Self {
            strict_validation: true,
            min_confidence: None,
            fallback_horizon_bars: 8,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExternalPredictionSignalModel {
    pub prediction_frame: PredictionFrame,
    pub config: ExternalPredictionSignalConfig,
}

impl ExternalPredictionSignalModel {
    pub fn signal_for(
        &self,
        symbol: &str,
        timestamp_ms: u64,
        timeframe: Timeframe,
        fold_id: Option<usize>,
        row_id: Option<&str>,
    ) -> SignalOutput {
        if self.config.strict_validation && !self.prediction_frame.schema_validation.valid {
            return conservative_no_trade_signal(
                symbol,
                "external_prediction_invalid_frame",
                self.config.fallback_horizon_bars,
            );
        }

        let row = row_id
            .and_then(|row_id| self.prediction_frame.find_by_row_id(row_id))
            .or_else(|| {
                self.prediction_frame
                    .find_by_key(symbol, timestamp_ms, timeframe, fold_id)
            });

        let Some(row) = row else {
            return conservative_no_trade_signal(
                symbol,
                "external_prediction_missing",
                self.config.fallback_horizon_bars,
            );
        };
        if !row.is_valid()
            || self
                .config
                .min_confidence
                .map(|threshold| row.confidence < threshold)
                .unwrap_or(false)
        {
            return conservative_no_trade_signal(
                symbol,
                "external_prediction_invalid",
                row.horizon_bars.max(self.config.fallback_horizon_bars),
            );
        }

        SignalOutput {
            symbol: row.symbol.clone(),
            horizon_bars: row.horizon_bars,
            p_win: row.p_win,
            p_stop: row.p_stop,
            expected_return: row.expected_return,
            expected_drawdown: row.expected_drawdown,
            confidence: row.confidence,
            no_trade_probability: row.no_trade_probability,
            source: format!("external_prediction:{}", row.model_id),
        }
    }

    pub fn for_fold(&self, fold_id: usize) -> Self {
        Self {
            prediction_frame: self.prediction_frame.for_fold(fold_id),
            config: self.config,
        }
    }
}

pub fn conservative_no_trade_signal(
    symbol: &str,
    source: &str,
    fallback_horizon_bars: u32,
) -> SignalOutput {
    SignalOutput {
        symbol: symbol.to_string(),
        horizon_bars: fallback_horizon_bars,
        p_win: 0.0,
        p_stop: 1.0,
        expected_return: 0.0,
        expected_drawdown: 0.02,
        confidence: 0.0,
        no_trade_probability: 1.0,
        source: source.to_string(),
    }
}
