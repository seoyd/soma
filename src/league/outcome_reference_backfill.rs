use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_hash_string, stable_reason_codes};

use super::comparable_committee_evidence::{
    ComparableCommitteeEvidenceRow, ComparableEvidenceSourceClass,
};
use super::official_candle_coverage_pack::{
    OfficialCandleCoveragePack, OfficialCandleSeriesDescriptor, normalize_symbol,
};
use super::scenario_materialization_v3::{
    ScenarioMaterializationV3Level, ScenarioMaterializationV3Report,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum OutcomeBackfillGapKind {
    MissingTripleBarrierOutcome,
    MissingOutcomeWindow,
    MissingFutureBars,
    MissingCostModel,
    MissingSlippageModel,
    TimestampMismatch,
    HorizonMismatch,
    NoLookaheadViolation,
    SourceIneligible,
    #[default]
    DiagnosticOnly,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum OutcomeBackfillSuggestedAction {
    BuildTripleBarrierOutcome,
    ProvideLongerCandleWindow,
    ProvideCostModel,
    ProvideSlippageModel,
    FixTimestampAlignment,
    FixHorizonAlignment,
    #[default]
    NoSafeAction,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OutcomeReferenceBackfillPlanItem {
    pub row_id: String,
    pub gap_kind: OutcomeBackfillGapKind,
    pub can_build_from_candles: bool,
    pub required_horizon_bars: usize,
    pub required_future_window: usize,
    pub suggested_action: OutcomeBackfillSuggestedAction,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OutcomeReferenceBackfillPlan {
    pub plan_id: String,
    pub items: Vec<OutcomeReferenceBackfillPlanItem>,
    pub buildable_count: usize,
    pub unavailable_count: usize,
    pub missing_future_window_count: usize,
    pub no_lookahead_blocked_count: usize,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

pub fn build_outcome_reference_backfill_plan(
    plan_id: impl Into<String>,
    rows: &[ComparableCommitteeEvidenceRow],
    materialization: Option<&ScenarioMaterializationV3Report>,
    packs: &[OfficialCandleCoveragePack],
) -> OutcomeReferenceBackfillPlan {
    let plan_id = plan_id.into();
    let materialized = materialization_map(materialization);
    let descriptors = descriptor_map(packs);
    let mut items = rows
        .iter()
        .filter(|row| !row.outcome_reference_available)
        .map(|row| build_item(row, materialized.get(&row.row_id), &descriptors))
        .collect::<Vec<_>>();
    items.sort_by(|left, right| left.row_id.cmp(&right.row_id));
    let buildable_count = items
        .iter()
        .filter(|item| item.can_build_from_candles)
        .count();
    let unavailable_count = items.len().saturating_sub(buildable_count);
    let missing_future_window_count = items
        .iter()
        .filter(|item| item.gap_kind == OutcomeBackfillGapKind::MissingFutureBars)
        .count();
    let no_lookahead_blocked_count = items
        .iter()
        .filter(|item| item.gap_kind == OutcomeBackfillGapKind::NoLookaheadViolation)
        .count();
    OutcomeReferenceBackfillPlan {
        plan_id,
        items,
        buildable_count,
        unavailable_count,
        missing_future_window_count,
        no_lookahead_blocked_count,
        reason_codes: stable_reason_codes(&[
            ReasonCode::CommitteeOutcomeReferenceBuilt,
            ReasonCode::DeterministicPath,
        ]),
    }
}

impl OutcomeReferenceBackfillPlan {
    pub fn fingerprint(&self) -> String {
        stable_hash_string(&serde_json::to_string(self).unwrap_or_else(|_| self.plan_id.clone()))
    }

    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("plan_id={}", self.plan_id),
            format!("buildable_count={}", self.buildable_count),
            format!("unavailable_count={}", self.unavailable_count),
            format!(
                "missing_future_window_count={}",
                self.missing_future_window_count
            ),
            format!(
                "no_lookahead_blocked_count={}",
                self.no_lookahead_blocked_count
            ),
            format!("fingerprint={}", self.fingerprint()),
        ];
        lines.extend(self.items.iter().map(|item| {
            format!(
                "row_id={};gap_kind={:?};can_build_from_candles={};required_horizon_bars={};required_future_window={};suggested_action={:?}",
                item.row_id,
                item.gap_kind,
                item.can_build_from_candles,
                item.required_horizon_bars,
                item.required_future_window,
                item.suggested_action,
            )
        }));
        lines.join("\n")
    }

    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn from_json_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        serde_json::from_str(&text).map_err(|err| err.to_string())
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("outcome_reference_backfill_plan.txt"),
            self.to_text(),
        )
        .map_err(|err| err.to_string())?;
        let json_path = output_dir.join("outcome_reference_backfill_plan.json");
        fs::write(&json_path, self.to_json_string()?).map_err(|err| err.to_string())?;
        Ok(json_path)
    }
}

fn build_item(
    row: &ComparableCommitteeEvidenceRow,
    materialized_level: Option<&ScenarioMaterializationV3Level>,
    descriptors: &BTreeMap<String, OfficialCandleSeriesDescriptor>,
) -> OutcomeReferenceBackfillPlanItem {
    let descriptor = row
        .matched_candle_series_id
        .as_ref()
        .and_then(|id| descriptors.get(id))
        .or_else(|| match_descriptor(row, descriptors));
    let mut reason_codes = row.reason_codes.clone();
    let mut gap_kind = OutcomeBackfillGapKind::MissingTripleBarrierOutcome;
    let mut suggested_action = OutcomeBackfillSuggestedAction::BuildTripleBarrierOutcome;
    let mut can_build_from_candles = false;
    if !row.no_lookahead_safe {
        gap_kind = OutcomeBackfillGapKind::NoLookaheadViolation;
        suggested_action = OutcomeBackfillSuggestedAction::NoSafeAction;
        reason_codes.push(ReasonCode::RejectedNoLookaheadReference);
    } else if matches!(
        row.source_class,
        ComparableEvidenceSourceClass::YFinanceResearch
            | ComparableEvidenceSourceClass::FixtureArchitectureTest
            | ComparableEvidenceSourceClass::SyntheticTest
    ) {
        gap_kind = OutcomeBackfillGapKind::SourceIneligible;
        suggested_action = OutcomeBackfillSuggestedAction::NoSafeAction;
        reason_codes.push(ReasonCode::ReadinessEvidenceExcluded);
    } else if row.candle_match_status.as_deref() == Some("TimestampMismatch") {
        gap_kind = OutcomeBackfillGapKind::TimestampMismatch;
        suggested_action = OutcomeBackfillSuggestedAction::FixTimestampAlignment;
        reason_codes.push(ReasonCode::UnsupportedTimestampFormat);
    } else if row.candle_match_status.as_deref() == Some("TimeframeMismatch") {
        gap_kind = OutcomeBackfillGapKind::HorizonMismatch;
        suggested_action = OutcomeBackfillSuggestedAction::FixHorizonAlignment;
        reason_codes.push(ReasonCode::UnsupportedTimeframe);
    } else if row.cost_bps <= 0.0 {
        gap_kind = OutcomeBackfillGapKind::MissingCostModel;
        suggested_action = OutcomeBackfillSuggestedAction::ProvideCostModel;
        reason_codes.push(ReasonCode::CostApplied);
    } else if row.slippage_bps <= 0.0 {
        gap_kind = OutcomeBackfillGapKind::MissingSlippageModel;
        suggested_action = OutcomeBackfillSuggestedAction::ProvideSlippageModel;
        reason_codes.push(ReasonCode::CostApplied);
    } else if materialized_level
        .is_some_and(|level| *level == ScenarioMaterializationV3Level::Rejected)
    {
        gap_kind = OutcomeBackfillGapKind::MissingOutcomeWindow;
        suggested_action = OutcomeBackfillSuggestedAction::NoSafeAction;
        reason_codes.push(ReasonCode::FeatureUnavailable);
    } else if let Some(descriptor) = descriptor {
        if descriptor.row_count <= row.horizon_bars {
            gap_kind = OutcomeBackfillGapKind::MissingFutureBars;
            suggested_action = OutcomeBackfillSuggestedAction::ProvideLongerCandleWindow;
            reason_codes.push(ReasonCode::InsufficientBars);
        } else {
            can_build_from_candles = true;
        }
    } else if row.candle_coverage_available && row.candle_official_ready_match {
        can_build_from_candles = true;
    } else {
        gap_kind = OutcomeBackfillGapKind::MissingOutcomeWindow;
        suggested_action = OutcomeBackfillSuggestedAction::ProvideLongerCandleWindow;
        reason_codes.push(ReasonCode::MissingRealLocalData);
    }
    if row.diagnostic_only && can_build_from_candles {
        gap_kind = OutcomeBackfillGapKind::DiagnosticOnly;
        reason_codes.push(ReasonCode::ControlledOnlyEvidence);
    }
    OutcomeReferenceBackfillPlanItem {
        row_id: row.row_id.clone(),
        gap_kind,
        can_build_from_candles,
        required_horizon_bars: row.horizon_bars,
        required_future_window: row.horizon_bars.saturating_add(1),
        suggested_action,
        reason_codes: stable_reason_codes(&reason_codes),
    }
}

fn materialization_map(
    report: Option<&ScenarioMaterializationV3Report>,
) -> BTreeMap<String, ScenarioMaterializationV3Level> {
    report
        .map(|report| {
            report
                .records
                .iter()
                .map(|record| (record.row_id.clone(), record.materialization_level))
                .collect()
        })
        .unwrap_or_default()
}

fn descriptor_map(
    packs: &[OfficialCandleCoveragePack],
) -> BTreeMap<String, OfficialCandleSeriesDescriptor> {
    let mut map = BTreeMap::new();
    for pack in packs {
        for descriptor in &pack.descriptors {
            map.insert(descriptor.candle_series_id.clone(), descriptor.clone());
        }
    }
    map
}

fn match_descriptor<'a>(
    row: &ComparableCommitteeEvidenceRow,
    descriptors: &'a BTreeMap<String, OfficialCandleSeriesDescriptor>,
) -> Option<&'a OfficialCandleSeriesDescriptor> {
    let normalized = normalize_symbol(&row.symbol);
    descriptors.values().find(|descriptor| {
        descriptor.normalized_symbol == normalized
            && descriptor.timeframe.eq_ignore_ascii_case(&row.timeframe)
    })
}
