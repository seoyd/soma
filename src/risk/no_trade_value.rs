use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct NoTradeValueInputs {
    pub no_trade_decisions: usize,
    pub no_trade_counterfactuals: usize,
    pub avoided_loss_value: f64,
    pub missed_gain_value: f64,
    #[serde(default)]
    pub no_trade_vs_baseline_delta: Option<f64>,
    #[serde(default)]
    pub no_trade_vs_committee_delta: Option<f64>,
    #[serde(default)]
    pub diagnostic_only: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum NoTradeValueStatus {
    NoTradeValuePositive,
    NoTradeTooConservative,
    NoTradeInsufficientCounterfactuals,
    NoTradeDiagnosticOnly,
    #[default]
    NoTradeNotEvaluated,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NoTradeValueReport {
    pub no_trade_decisions: usize,
    pub no_trade_counterfactuals: usize,
    pub avoided_loss_value: f64,
    pub missed_gain_value: f64,
    pub no_trade_value_proxy: f64,
    #[serde(default)]
    pub no_trade_vs_baseline_delta: Option<f64>,
    #[serde(default)]
    pub no_trade_vs_committee_delta: Option<f64>,
    pub status: NoTradeValueStatus,
    pub warnings: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

pub fn build_no_trade_value_report(inputs: &NoTradeValueInputs) -> NoTradeValueReport {
    let no_trade_value_proxy = inputs.avoided_loss_value - inputs.missed_gain_value;
    let mut warnings = Vec::new();
    let status = if inputs.no_trade_decisions == 0 {
        NoTradeValueStatus::NoTradeNotEvaluated
    } else if inputs.diagnostic_only {
        warnings.push("no-trade evidence remains diagnostic-only".to_string());
        NoTradeValueStatus::NoTradeDiagnosticOnly
    } else if inputs.no_trade_counterfactuals == 0 {
        warnings.push("no-trade counterfactual depth is insufficient".to_string());
        NoTradeValueStatus::NoTradeInsufficientCounterfactuals
    } else if inputs.missed_gain_value > inputs.avoided_loss_value {
        warnings.push("no-trade decisions appear more conservative than defensive".to_string());
        NoTradeValueStatus::NoTradeTooConservative
    } else {
        NoTradeValueStatus::NoTradeValuePositive
    };

    let mut reason_codes = inputs.reason_codes.clone();
    reason_codes.push(ReasonCode::NoTradeValueReportBuilt);
    if inputs.no_trade_counterfactuals == 0 {
        reason_codes.push(ReasonCode::NoTradeCounterfactual);
    }

    NoTradeValueReport {
        no_trade_decisions: inputs.no_trade_decisions,
        no_trade_counterfactuals: inputs.no_trade_counterfactuals,
        avoided_loss_value: inputs.avoided_loss_value,
        missed_gain_value: inputs.missed_gain_value,
        no_trade_value_proxy,
        no_trade_vs_baseline_delta: inputs.no_trade_vs_baseline_delta,
        no_trade_vs_committee_delta: inputs.no_trade_vs_committee_delta,
        status,
        warnings,
        reason_codes: stable_reason_codes(&reason_codes),
    }
}

impl NoTradeValueReport {
    pub fn to_text(&self) -> String {
        [
            format!("no_trade_decisions={}", self.no_trade_decisions),
            format!("no_trade_counterfactuals={}", self.no_trade_counterfactuals),
            format!("avoided_loss_value={:.6}", self.avoided_loss_value),
            format!("missed_gain_value={:.6}", self.missed_gain_value),
            format!("no_trade_value_proxy={:.6}", self.no_trade_value_proxy),
            format!(
                "no_trade_vs_baseline_delta={}",
                self.no_trade_vs_baseline_delta
                    .map(|value| format!("{value:.6}"))
                    .unwrap_or_default()
            ),
            format!(
                "no_trade_vs_committee_delta={}",
                self.no_trade_vs_committee_delta
                    .map(|value| format!("{value:.6}"))
                    .unwrap_or_default()
            ),
            format!("status={:?}", self.status),
            format!("warnings={}", self.warnings.join(" | ")),
        ]
        .join("\n")
    }
}
