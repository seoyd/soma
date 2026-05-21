use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};

use super::core_performance_bundle::CorePerformanceScorecardBundle;
use super::core_performance_scorecard::CorePerformanceScorecard;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CorePerformanceRegressionSummary {
    pub scorecard_id: String,
    pub official_row_count: usize,
    pub outcome_linked_rows: usize,
    pub counterfactual_rows: usize,
    #[serde(default)]
    pub brier_score: Option<f64>,
    #[serde(default)]
    pub ece: Option<f64>,
    pub denial_rate: f64,
    pub avoided_loss_total: f64,
    #[serde(default)]
    pub actionability_ratio: Option<f64>,
    pub report_bytes: usize,
    pub fingerprint: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CorePerformanceRegressionConfig {
    #[serde(default)]
    pub previous_scorecard_path: Option<String>,
    #[serde(default)]
    pub current_scorecard_path: Option<String>,
    #[serde(default = "default_official_row_drop")]
    pub max_allowed_official_row_drop: usize,
    #[serde(default = "default_outcome_link_drop")]
    pub max_allowed_outcome_link_drop: usize,
    #[serde(default = "default_counterfactual_drop")]
    pub max_allowed_counterfactual_drop: usize,
    #[serde(default = "default_calibration_worsening")]
    pub max_allowed_calibration_worsening: f64,
    #[serde(default = "default_denial_rate_increase")]
    pub max_allowed_denial_rate_increase: f64,
    #[serde(default = "default_actionability_drop")]
    pub max_allowed_actionability_drop: f64,
    #[serde(default = "default_storage_growth_bytes")]
    pub max_allowed_storage_growth_bytes: usize,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CorePerformanceRegressionReport {
    pub comparable: bool,
    #[serde(default)]
    pub previous_summary: Option<CorePerformanceRegressionSummary>,
    pub current_summary: CorePerformanceRegressionSummary,
    pub regressions: Vec<String>,
    pub improvements: Vec<String>,
    pub regression_detected: bool,
    pub blocking_regression_detected: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for CorePerformanceRegressionConfig {
    fn default() -> Self {
        Self {
            previous_scorecard_path: None,
            current_scorecard_path: None,
            max_allowed_official_row_drop: default_official_row_drop(),
            max_allowed_outcome_link_drop: default_outcome_link_drop(),
            max_allowed_counterfactual_drop: default_counterfactual_drop(),
            max_allowed_calibration_worsening: default_calibration_worsening(),
            max_allowed_denial_rate_increase: default_denial_rate_increase(),
            max_allowed_actionability_drop: default_actionability_drop(),
            max_allowed_storage_growth_bytes: default_storage_growth_bytes(),
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

impl CorePerformanceRegressionConfig {
    pub fn from_toml_str(input: &str) -> Result<Self, String> {
        toml::from_str(input).map_err(|err| err.to_string())
    }

    pub fn from_toml_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        Self::from_toml_str(&text)
    }

    pub fn to_toml_string(&self) -> Result<String, String> {
        toml::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn validate(&self) -> Result<(), String> {
        let paths = self
            .previous_scorecard_path
            .iter()
            .chain(self.current_scorecard_path.iter());
        if paths.clone().any(|path| path.contains("://")) {
            return Err("core-regression paths must be local".to_string());
        }
        if self.max_allowed_calibration_worsening < 0.0
            || self.max_allowed_denial_rate_increase < 0.0
            || self.max_allowed_actionability_drop < 0.0
        {
            return Err("core-regression thresholds must be non-negative".to_string());
        }
        Ok(())
    }
}

pub fn build_core_performance_regression_report(
    config: &CorePerformanceRegressionConfig,
    previous_summary: Option<CorePerformanceRegressionSummary>,
    current_summary: CorePerformanceRegressionSummary,
) -> CorePerformanceRegressionReport {
    let mut regressions = Vec::new();
    let mut improvements = Vec::new();
    let comparable = previous_summary.is_some();

    if let Some(previous) = &previous_summary {
        if previous.official_row_count
            > current_summary
                .official_row_count
                .saturating_add(config.max_allowed_official_row_drop)
        {
            regressions.push(format!(
                "official_row_drop={}=>{}",
                previous.official_row_count, current_summary.official_row_count
            ));
        } else if current_summary.official_row_count > previous.official_row_count {
            improvements.push(format!(
                "official_row_gain={}=>{}",
                previous.official_row_count, current_summary.official_row_count
            ));
        }

        if previous.outcome_linked_rows
            > current_summary
                .outcome_linked_rows
                .saturating_add(config.max_allowed_outcome_link_drop)
        {
            regressions.push(format!(
                "outcome_link_drop={}=>{}",
                previous.outcome_linked_rows, current_summary.outcome_linked_rows
            ));
        }
        if previous.counterfactual_rows
            > current_summary
                .counterfactual_rows
                .saturating_add(config.max_allowed_counterfactual_drop)
        {
            regressions.push(format!(
                "counterfactual_drop={}=>{}",
                previous.counterfactual_rows, current_summary.counterfactual_rows
            ));
        }

        let current_calibration = current_summary
            .ece
            .or(current_summary.brier_score)
            .unwrap_or_default();
        let previous_calibration = previous.ece.or(previous.brier_score).unwrap_or_default();
        if current_calibration - previous_calibration > config.max_allowed_calibration_worsening {
            regressions.push(format!(
                "calibration_worsened={previous_calibration:.6}=>{current_calibration:.6}"
            ));
        } else if previous_calibration - current_calibration > 0.0 {
            improvements.push(format!(
                "calibration_improved={previous_calibration:.6}=>{current_calibration:.6}"
            ));
        }

        if current_summary.denial_rate - previous.denial_rate
            > config.max_allowed_denial_rate_increase
            && current_summary.avoided_loss_total <= previous.avoided_loss_total
        {
            regressions.push(format!(
                "denial_rate_increase={:.6}=>{:.6}",
                previous.denial_rate, current_summary.denial_rate
            ));
        }

        if let (Some(previous_actionability), Some(current_actionability)) = (
            previous.actionability_ratio,
            current_summary.actionability_ratio,
        ) {
            if previous_actionability - current_actionability
                > config.max_allowed_actionability_drop
            {
                regressions.push(format!(
                    "actionability_drop={previous_actionability:.6}=>{current_actionability:.6}"
                ));
            }
        }

        if current_summary
            .report_bytes
            .saturating_sub(previous.report_bytes)
            > config.max_allowed_storage_growth_bytes
        {
            regressions.push(format!(
                "storage_growth={}=>{}",
                previous.report_bytes, current_summary.report_bytes
            ));
        }

        if previous.fingerprint != current_summary.fingerprint && regressions.is_empty() {
            regressions.push("determinism_fingerprint_changed".to_string());
        }
    }

    let regression_detected = !regressions.is_empty();
    let mut reason_codes = config.reason_codes.clone();
    reason_codes.push(ReasonCode::CorePerformanceRegressionReportBuilt);
    if regression_detected {
        reason_codes.push(ReasonCode::RegressionDetected);
    }

    CorePerformanceRegressionReport {
        comparable,
        previous_summary,
        current_summary,
        regressions,
        improvements,
        regression_detected,
        blocking_regression_detected: regression_detected,
        reason_codes: stable_reason_codes(&reason_codes),
    }
}

impl CorePerformanceRegressionReport {
    pub fn from_config(config: &CorePerformanceRegressionConfig) -> Result<Self, String> {
        config.validate()?;
        let current = load_summary_from_path(config.current_scorecard_path.as_deref())?;
        let previous = match config.previous_scorecard_path.as_deref() {
            Some(path) => Some(load_summary_from_path(Some(path))?),
            None => None,
        };
        Ok(build_core_performance_regression_report(
            config, previous, current,
        ))
    }

    pub fn to_text(&self) -> String {
        [
            format!("comparable={}", self.comparable),
            format!(
                "previous_scorecard_id={}",
                self.previous_summary
                    .as_ref()
                    .map(|summary| summary.scorecard_id.clone())
                    .unwrap_or_default()
            ),
            format!("current_scorecard_id={}", self.current_summary.scorecard_id),
            format!("regressions={}", self.regressions.join(" | ")),
            format!("improvements={}", self.improvements.join(" | ")),
            format!("regression_detected={}", self.regression_detected),
            format!(
                "blocking_regression_detected={}",
                self.blocking_regression_detected
            ),
        ]
        .join("\n")
    }
}

pub fn summary_from_scorecard(
    scorecard: &CorePerformanceScorecard,
) -> CorePerformanceRegressionSummary {
    CorePerformanceRegressionSummary {
        scorecard_id: scorecard.scorecard_id.clone(),
        official_row_count: scorecard.signal_quality_report.official_evaluated_rows,
        outcome_linked_rows: scorecard.signal_quality_report.outcome_linked_rows,
        counterfactual_rows: scorecard.no_trade_value_report.no_trade_counterfactuals
            + scorecard
                .risk_governor_value_report
                .risk_denied_counterfactual_count,
        brier_score: scorecard.signal_quality_report.brier_score,
        ece: scorecard.signal_quality_report.ece,
        denial_rate: scorecard.risk_governor_value_report.denial_rate,
        avoided_loss_total: scorecard.risk_governor_value_report.avoided_loss_total
            + scorecard.no_trade_value_report.avoided_loss_value,
        actionability_ratio: actionability_ratio(scorecard),
        report_bytes: scorecard.to_text().len(),
        fingerprint: scorecard.fingerprint(),
    }
}

fn actionability_ratio(scorecard: &CorePerformanceScorecard) -> Option<f64> {
    let counts = &scorecard
        .committee_value_attribution_report
        .committee_action_counts;
    let comparable = scorecard.committee_value_attribution_report.comparable_rows;
    if comparable == 0 {
        return None;
    }
    let actionable = counts.get("PaperApprove").copied().unwrap_or(0)
        + counts.get("PaperReduceSize").copied().unwrap_or(0)
        + counts.get("HumanConfirmRequired").copied().unwrap_or(0);
    Some(actionable as f64 / comparable as f64)
}

fn load_summary_from_path(path: Option<&str>) -> Result<CorePerformanceRegressionSummary, String> {
    let Some(path) = path else {
        return Err("current scorecard path is required".to_string());
    };
    let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
    if let Ok(summary) = serde_json::from_str::<CorePerformanceRegressionSummary>(&text) {
        return Ok(summary);
    }
    if let Ok(scorecard) = serde_json::from_str::<CorePerformanceScorecard>(&text) {
        return Ok(summary_from_scorecard(&scorecard));
    }
    if let Ok(bundle) = serde_json::from_str::<CorePerformanceScorecardBundle>(&text) {
        return Ok(summary_from_scorecard(&bundle.scorecard));
    }
    let scorecard = CorePerformanceScorecard::from_json_path(Path::new(path))?;
    Ok(summary_from_scorecard(&scorecard))
}

impl CorePerformanceRegressionSummary {
    pub fn to_json_path(&self, path: &Path) -> Result<PathBuf, String> {
        fs::write(
            path,
            serde_json::to_string_pretty(self).map_err(|err| err.to_string())?,
        )
        .map_err(|err| err.to_string())?;
        Ok(path.to_path_buf())
    }
}

fn default_official_row_drop() -> usize {
    0
}

fn default_outcome_link_drop() -> usize {
    0
}

fn default_counterfactual_drop() -> usize {
    0
}

fn default_calibration_worsening() -> f64 {
    0.02
}

fn default_denial_rate_increase() -> f64 {
    0.10
}

fn default_actionability_drop() -> f64 {
    0.10
}

fn default_storage_growth_bytes() -> usize {
    128 * 1024
}
