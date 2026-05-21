use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::core::{ReasonCode, stable_reason_codes};

use super::committee_counterfactual_builder::{horizon_bars_for_row, normalize_symbol};
use super::committee_outcome_reference::{CommitteeBaselineAction, CommitteeBaselineReference};
use super::committee_scenario_loader::CommitteeScenarioRow;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum BaselineReferenceSource {
    ExistingArtifact,
    DeterministicNoTrade,
    DeterministicBaselineSignalApprox,
    #[default]
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BaselineReferencePolicy {
    pub policy_id: String,
    pub source: BaselineReferenceSource,
    #[serde(default)]
    pub allow_approximation: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BaselineGenerationResult {
    pub reference: CommitteeBaselineReference,
    pub source: BaselineReferenceSource,
    pub diagnostic_only: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LoadedBaselineReference {
    pub symbol: String,
    pub timestamp_ms: u64,
    pub horizon_bars: Option<usize>,
    pub reference: CommitteeBaselineReference,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BaselineReferenceGenerator;

impl Default for BaselineReferencePolicy {
    fn default() -> Self {
        Self {
            policy_id: "baseline_reference_policy".to_string(),
            source: BaselineReferenceSource::DeterministicNoTrade,
            allow_approximation: false,
            reason_codes: vec![ReasonCode::CommitteeOutcomeReferenceBuilt],
        }
    }
}

impl BaselineReferenceGenerator {
    pub fn load_existing(paths: &[String]) -> Result<Vec<LoadedBaselineReference>, String> {
        let mut loaded = Vec::new();
        for value in load_values(
            paths,
            &["baselines", "baseline_references", "rows", "records"],
        )? {
            let symbol = value
                .get("symbol")
                .and_then(Value::as_str)
                .ok_or_else(|| "baseline reference is missing symbol".to_string())?;
            let timestamp_ms = value
                .get("timestamp_ms")
                .and_then(Value::as_u64)
                .ok_or_else(|| "baseline reference is missing timestamp_ms".to_string())?;
            loaded.push(LoadedBaselineReference {
                symbol: normalize_symbol(symbol),
                timestamp_ms,
                horizon_bars: value
                    .get("horizon_bars")
                    .and_then(Value::as_u64)
                    .map(|value| value as usize),
                reference: CommitteeBaselineReference {
                    baseline_action: value
                        .get("baseline_action")
                        .and_then(Value::as_str)
                        .map(CommitteeBaselineAction::from_summary)
                        .unwrap_or(CommitteeBaselineAction::NoTrade),
                    baseline_confidence: value.get("baseline_confidence").and_then(Value::as_f64),
                    baseline_expected_edge: value
                        .get("baseline_expected_edge")
                        .and_then(Value::as_f64),
                    baseline_reason_codes: vec![ReasonCode::CommitteeOutcomeReferenceBuilt],
                    reason_codes: vec![ReasonCode::CommitteeOutcomeReferenceBuilt],
                },
            });
        }
        loaded.sort_by(|left, right| {
            left.symbol
                .cmp(&right.symbol)
                .then(left.timestamp_ms.cmp(&right.timestamp_ms))
                .then(left.horizon_bars.cmp(&right.horizon_bars))
        });
        Ok(loaded)
    }

    pub fn find_existing<'a>(
        row: &CommitteeScenarioRow,
        loaded: &'a [LoadedBaselineReference],
        default_horizon_bars: usize,
    ) -> Option<&'a LoadedBaselineReference> {
        let symbol = normalize_symbol(&row.symbol);
        let horizon_bars = horizon_bars_for_row(row, default_horizon_bars);
        loaded.iter().find(|item| {
            item.symbol == symbol
                && item.timestamp_ms == row.timestamp_ms
                && item.horizon_bars.unwrap_or(horizon_bars) == horizon_bars
        })
    }

    pub fn generate(
        &self,
        row: &CommitteeScenarioRow,
        existing: Option<&LoadedBaselineReference>,
        policy: &BaselineReferencePolicy,
    ) -> BaselineGenerationResult {
        if let Some(existing) = existing {
            let mut reason_codes = policy.reason_codes.clone();
            reason_codes.push(ReasonCode::CommitteeOutcomeReferenceBuilt);
            return BaselineGenerationResult {
                reference: existing.reference.clone(),
                source: BaselineReferenceSource::ExistingArtifact,
                diagnostic_only: false,
                reason_codes: stable_reason_codes(&reason_codes),
            };
        }
        match policy.source {
            BaselineReferenceSource::DeterministicBaselineSignalApprox => {
                let summary = row
                    .baseline_signal_summary
                    .as_deref()
                    .unwrap_or(&row.signal_summary);
                let mut reason_codes = policy.reason_codes.clone();
                reason_codes.extend([
                    ReasonCode::CommitteeOutcomeReferenceBuilt,
                    ReasonCode::EvidenceEstimateBuilt,
                ]);
                BaselineGenerationResult {
                    reference: CommitteeBaselineReference {
                        baseline_action: CommitteeBaselineAction::from_summary(summary),
                        baseline_confidence: Some((row.data_quality_score * 0.8).clamp(0.0, 1.0)),
                        baseline_expected_edge: Some(row.expected_edge_after_cost),
                        baseline_reason_codes: stable_reason_codes(&reason_codes),
                        reason_codes: stable_reason_codes(&reason_codes),
                    },
                    source: BaselineReferenceSource::DeterministicBaselineSignalApprox,
                    diagnostic_only: !policy.allow_approximation,
                    reason_codes: stable_reason_codes(&reason_codes),
                }
            }
            BaselineReferenceSource::ExistingArtifact
            | BaselineReferenceSource::DeterministicNoTrade
            | BaselineReferenceSource::Unknown => {
                let mut reason_codes = policy.reason_codes.clone();
                reason_codes.extend([
                    ReasonCode::CommitteeOutcomeReferenceBuilt,
                    ReasonCode::NoTradePreferred,
                    ReasonCode::BaselineSignalNoTradeBias,
                ]);
                BaselineGenerationResult {
                    reference: CommitteeBaselineReference {
                        baseline_action: CommitteeBaselineAction::NoTrade,
                        baseline_confidence: Some(1.0),
                        baseline_expected_edge: Some(0.0),
                        baseline_reason_codes: stable_reason_codes(&reason_codes),
                        reason_codes: stable_reason_codes(&reason_codes),
                    },
                    source: BaselineReferenceSource::DeterministicNoTrade,
                    diagnostic_only: false,
                    reason_codes: stable_reason_codes(&reason_codes),
                }
            }
        }
    }
}

fn load_values(paths: &[String], keys: &[&str]) -> Result<Vec<Value>, String> {
    let mut values = Vec::new();
    for path in paths {
        let text = fs::read_to_string(Path::new(path)).map_err(|err| err.to_string())?;
        let parsed: Value = serde_json::from_str(&text).map_err(|err| err.to_string())?;
        if let Some(items) = parsed.as_array() {
            values.extend(items.iter().cloned());
        } else if let Some(items) = keys
            .iter()
            .find_map(|key| parsed.get(key).and_then(Value::as_array))
        {
            values.extend(items.iter().cloned());
        } else {
            values.push(parsed);
        }
    }
    Ok(values)
}
