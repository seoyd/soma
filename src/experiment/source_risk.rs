use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;

use super::source_benchmark::SourceBenchmarkSummary;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SourceRiskInteractionComparison {
    #[serde(default)]
    pub official_denial_rate: Option<f64>,
    #[serde(default)]
    pub yfinance_denial_rate: Option<f64>,
    #[serde(default)]
    pub denial_rate_delta: Option<f64>,
    #[serde(default)]
    pub official_defensive_value: Option<f64>,
    #[serde(default)]
    pub yfinance_defensive_value: Option<f64>,
    #[serde(default)]
    pub defensive_value_delta: Option<f64>,
    #[serde(default)]
    pub official_opportunity_cost: Option<f64>,
    #[serde(default)]
    pub yfinance_opportunity_cost: Option<f64>,
    pub risk_behavior_consistent: bool,
    pub warnings: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

pub fn build_source_risk_interaction_comparison(
    official: Option<&SourceBenchmarkSummary>,
    yfinance: Option<&SourceBenchmarkSummary>,
    max_allowed_risk_delta: f64,
) -> SourceRiskInteractionComparison {
    let official_denial_rate = official.and_then(|summary| summary.denial_rate);
    let yfinance_denial_rate = yfinance.and_then(|summary| summary.denial_rate);
    let denial_rate_delta = official_denial_rate
        .zip(yfinance_denial_rate)
        .map(|(left, right)| (left - right).abs());
    let official_defensive_value = official.and_then(|summary| summary.defensive_value);
    let yfinance_defensive_value = yfinance.and_then(|summary| summary.defensive_value);
    let defensive_value_delta = official_defensive_value
        .zip(yfinance_defensive_value)
        .map(|(left, right)| (left - right).abs());
    let official_opportunity_cost = official.and_then(|summary| summary.opportunity_cost);
    let yfinance_opportunity_cost = yfinance.and_then(|summary| summary.opportunity_cost);
    let mut warnings = Vec::new();
    if denial_rate_delta.is_some_and(|delta| delta > max_allowed_risk_delta) {
        warnings.push("denial-rate delta exceeds configured threshold".to_string());
    }
    if let (Some(y_denial), Some(o_denial), Some(y_drawdown), Some(o_drawdown)) = (
        yfinance_denial_rate,
        official_denial_rate,
        yfinance.and_then(|summary| summary.avg_max_drawdown_pct),
        official.and_then(|summary| summary.avg_max_drawdown_pct),
    ) {
        if y_denial < o_denial && y_drawdown > o_drawdown {
            warnings.push(
                "yfinance shows lower denial but worse drawdown; risk behavior may not generalize"
                    .to_string(),
            );
        }
    }
    let risk_behavior_consistent = !warnings
        .iter()
        .any(|warning| warning.contains("denial-rate delta"));
    SourceRiskInteractionComparison {
        official_denial_rate,
        yfinance_denial_rate,
        denial_rate_delta,
        official_defensive_value,
        yfinance_defensive_value,
        defensive_value_delta,
        official_opportunity_cost,
        yfinance_opportunity_cost,
        risk_behavior_consistent,
        warnings,
        reason_codes: vec![ReasonCode::SourceRiskCompared],
    }
}
