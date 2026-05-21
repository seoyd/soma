use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelUsefulnessGate {
    SchemaValid,
    EnoughOutcomes,
    CalibrationAcceptable,
    DrawdownNotWorse,
    NetReturnNotWorse,
    ProfitFactorAcceptable,
    RiskGovernorStable,
    NoLeakageWarnings,
    DataQualityAcceptable,
    StorageBudgetAcceptable,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ModelUsefulnessGateConfig {
    pub min_outcomes: usize,
    pub max_drawdown_worsening_pct: f64,
    pub max_ece: f64,
    pub max_brier_score: f64,
    #[serde(default)]
    pub min_profit_factor: Option<f64>,
    #[serde(default = "default_true")]
    pub require_not_worse_than_baseline: bool,
    #[serde(default = "default_true")]
    pub require_risk_stability: bool,
    #[serde(default = "default_true")]
    pub require_schema_valid: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for ModelUsefulnessGateConfig {
    fn default() -> Self {
        Self {
            min_outcomes: 20,
            max_drawdown_worsening_pct: 0.02,
            max_ece: 0.10,
            max_brier_score: 0.30,
            min_profit_factor: None,
            require_not_worse_than_baseline: true,
            require_risk_stability: true,
            require_schema_valid: true,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ModelUsefulnessGateInputs {
    pub schema_valid: bool,
    pub outcome_count: usize,
    pub calibration_count: usize,
    pub brier_score: f64,
    pub expected_calibration_error: f64,
    pub selected_profit_factor: Option<f64>,
    pub delta_max_drawdown_pct: Option<f64>,
    pub delta_net_return_pct: Option<f64>,
    pub denial_rate: f64,
    pub approval_rate: f64,
    pub emergency_stop_count: usize,
    pub leakage_detected: bool,
    pub data_quality_score: f64,
    pub budget_exceeded: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ModelUsefulnessGateResult {
    pub passed: bool,
    pub failed_gates: Vec<ModelUsefulnessGate>,
    pub warnings: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

impl ModelUsefulnessGateResult {
    pub fn evaluate(
        config: &ModelUsefulnessGateConfig,
        inputs: &ModelUsefulnessGateInputs,
    ) -> Self {
        let mut failed_gates = Vec::new();
        let mut warnings = Vec::new();
        if config.require_schema_valid && !inputs.schema_valid {
            failed_gates.push(ModelUsefulnessGate::SchemaValid);
        }
        if inputs.outcome_count < config.min_outcomes {
            failed_gates.push(ModelUsefulnessGate::EnoughOutcomes);
        }
        if inputs.calibration_count < config.min_outcomes
            || inputs.expected_calibration_error > config.max_ece
            || inputs.brier_score > config.max_brier_score
        {
            failed_gates.push(ModelUsefulnessGate::CalibrationAcceptable);
        }
        if inputs
            .delta_max_drawdown_pct
            .is_some_and(|delta| delta > config.max_drawdown_worsening_pct)
        {
            failed_gates.push(ModelUsefulnessGate::DrawdownNotWorse);
        }
        if config.require_not_worse_than_baseline
            && inputs.delta_net_return_pct.is_some_and(|delta| delta < 0.0)
        {
            failed_gates.push(ModelUsefulnessGate::NetReturnNotWorse);
        }
        if config
            .min_profit_factor
            .is_some_and(|minimum| inputs.selected_profit_factor.unwrap_or(0.0) < minimum)
        {
            failed_gates.push(ModelUsefulnessGate::ProfitFactorAcceptable);
        }
        if config.require_risk_stability
            && (inputs.denial_rate > 0.98
                || (inputs.approval_rate < 0.01 && inputs.denial_rate > 0.90)
                || inputs.emergency_stop_count > 0)
        {
            failed_gates.push(ModelUsefulnessGate::RiskGovernorStable);
        }
        if inputs.leakage_detected {
            failed_gates.push(ModelUsefulnessGate::NoLeakageWarnings);
        }
        if inputs.data_quality_score < 0.80 {
            failed_gates.push(ModelUsefulnessGate::DataQualityAcceptable);
        }
        if inputs.budget_exceeded {
            failed_gates.push(ModelUsefulnessGate::StorageBudgetAcceptable);
        }
        if inputs.expected_calibration_error > config.max_ece * 0.8 {
            warnings.push("calibration is close to the configured ECE limit".to_string());
        }
        if inputs.denial_rate > 0.90 {
            warnings.push("risk governor denial rate is very high".to_string());
        }

        let passed = failed_gates.is_empty();
        let reason_codes = if passed {
            vec![ReasonCode::ModelUsefulnessGatePassed]
        } else {
            vec![ReasonCode::ModelUsefulnessGateFailed]
        };
        Self {
            passed,
            failed_gates,
            warnings,
            reason_codes,
        }
    }

    pub fn to_text(&self) -> String {
        [
            format!("passed={}", self.passed),
            format!(
                "failed_gates={}",
                self.failed_gates
                    .iter()
                    .map(|gate| format!("{gate:?}"))
                    .collect::<Vec<_>>()
                    .join("|")
            ),
            format!("warnings={}", self.warnings.join(" | ")),
            format!(
                "reason_codes={}",
                self.reason_codes
                    .iter()
                    .map(|reason| format!("{reason:?}"))
                    .collect::<Vec<_>>()
                    .join("|")
            ),
        ]
        .join("\n")
    }
}

fn default_true() -> bool {
    true
}
