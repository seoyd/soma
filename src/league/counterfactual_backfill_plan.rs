use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_hash_string, stable_reason_codes};

use super::comparable_committee_evidence::{
    ComparableCommitteeEvidenceRow, ComparableEvidenceSourceClass,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CounterfactualBackfillGapKind {
    MissingNoTradeCounterfactual,
    MissingRiskDeniedCounterfactual,
    MissingCandleWindow,
    MissingRiskDecision,
    MissingCommitteeDecision,
    NoLookaheadViolation,
    SourceIneligible,
    #[default]
    DiagnosticOnly,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CounterfactualBackfillSuggestedAction {
    BuildNoTradeCounterfactual,
    BuildRiskDeniedCounterfactual,
    ProvideRiskDecision,
    ProvideCommitteeDecision,
    ProvideLongerCandleWindow,
    #[default]
    NoSafeAction,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CounterfactualBackfillPlanItem {
    pub row_id: String,
    pub gap_kind: CounterfactualBackfillGapKind,
    pub can_build_no_trade: bool,
    pub can_build_risk_denied: bool,
    pub suggested_action: CounterfactualBackfillSuggestedAction,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CounterfactualBackfillPlan {
    pub plan_id: String,
    pub items: Vec<CounterfactualBackfillPlanItem>,
    pub no_trade_buildable_count: usize,
    pub risk_denied_buildable_count: usize,
    pub unavailable_count: usize,
    pub no_lookahead_blocked_count: usize,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

pub fn build_counterfactual_backfill_plan(
    plan_id: impl Into<String>,
    rows: &[ComparableCommitteeEvidenceRow],
) -> CounterfactualBackfillPlan {
    let plan_id = plan_id.into();
    let mut items = rows
        .iter()
        .filter(|row| {
            !row.no_trade_counterfactual_available || !row.risk_denied_counterfactual_available
        })
        .map(build_item)
        .collect::<Vec<_>>();
    items.sort_by(|left, right| left.row_id.cmp(&right.row_id));
    let no_trade_buildable_count = items.iter().filter(|item| item.can_build_no_trade).count();
    let risk_denied_buildable_count = items
        .iter()
        .filter(|item| item.can_build_risk_denied)
        .count();
    let unavailable_count = items
        .iter()
        .filter(|item| !item.can_build_no_trade && !item.can_build_risk_denied)
        .count();
    let no_lookahead_blocked_count = items
        .iter()
        .filter(|item| item.gap_kind == CounterfactualBackfillGapKind::NoLookaheadViolation)
        .count();
    CounterfactualBackfillPlan {
        plan_id,
        items,
        no_trade_buildable_count,
        risk_denied_buildable_count,
        unavailable_count,
        no_lookahead_blocked_count,
        reason_codes: stable_reason_codes(&[
            ReasonCode::CounterfactualEvaluated,
            ReasonCode::DeterministicPath,
        ]),
    }
}

impl CounterfactualBackfillPlan {
    pub fn fingerprint(&self) -> String {
        stable_hash_string(&serde_json::to_string(self).unwrap_or_else(|_| self.plan_id.clone()))
    }

    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("plan_id={}", self.plan_id),
            format!("no_trade_buildable_count={}", self.no_trade_buildable_count),
            format!(
                "risk_denied_buildable_count={}",
                self.risk_denied_buildable_count
            ),
            format!("unavailable_count={}", self.unavailable_count),
            format!(
                "no_lookahead_blocked_count={}",
                self.no_lookahead_blocked_count
            ),
            format!("fingerprint={}", self.fingerprint()),
        ];
        lines.extend(self.items.iter().map(|item| {
            format!(
                "row_id={};gap_kind={:?};can_build_no_trade={};can_build_risk_denied={};suggested_action={:?}",
                item.row_id,
                item.gap_kind,
                item.can_build_no_trade,
                item.can_build_risk_denied,
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
            output_dir.join("counterfactual_backfill_plan.txt"),
            self.to_text(),
        )
        .map_err(|err| err.to_string())?;
        let json_path = output_dir.join("counterfactual_backfill_plan.json");
        fs::write(&json_path, self.to_json_string()?).map_err(|err| err.to_string())?;
        Ok(json_path)
    }
}

fn build_item(row: &ComparableCommitteeEvidenceRow) -> CounterfactualBackfillPlanItem {
    let mut reason_codes = row.reason_codes.clone();
    let source_ineligible = matches!(
        row.source_class,
        ComparableEvidenceSourceClass::YFinanceResearch
            | ComparableEvidenceSourceClass::FixtureArchitectureTest
            | ComparableEvidenceSourceClass::SyntheticTest
    );
    let can_build_no_trade = !row.no_trade_counterfactual_available
        && row.no_lookahead_safe
        && !source_ineligible
        && row.candle_coverage_available
        && row.outcome_reference_available;
    let can_build_risk_denied = !row.risk_denied_counterfactual_available
        && row.no_lookahead_safe
        && !source_ineligible
        && row.candle_coverage_available
        && row.outcome_reference_available
        && row
            .risk_governor_decision
            .as_ref()
            .is_some_and(|value| !value.trim().is_empty())
        && !row.committee_final_action.trim().is_empty();
    let (gap_kind, suggested_action, extra_reason) = if !row.no_lookahead_safe {
        (
            CounterfactualBackfillGapKind::NoLookaheadViolation,
            CounterfactualBackfillSuggestedAction::NoSafeAction,
            ReasonCode::RejectedNoLookaheadReference,
        )
    } else if source_ineligible {
        (
            CounterfactualBackfillGapKind::SourceIneligible,
            CounterfactualBackfillSuggestedAction::NoSafeAction,
            ReasonCode::ReadinessEvidenceExcluded,
        )
    } else if !row.candle_coverage_available {
        (
            CounterfactualBackfillGapKind::MissingCandleWindow,
            CounterfactualBackfillSuggestedAction::ProvideLongerCandleWindow,
            ReasonCode::MissingRealLocalData,
        )
    } else if row
        .risk_governor_decision
        .as_ref()
        .is_none_or(|value| value.trim().is_empty())
        && !row.risk_denied_counterfactual_available
    {
        (
            CounterfactualBackfillGapKind::MissingRiskDecision,
            CounterfactualBackfillSuggestedAction::ProvideRiskDecision,
            ReasonCode::RiskDeniedCounterfactual,
        )
    } else if row.committee_final_action.trim().is_empty()
        && !row.risk_denied_counterfactual_available
    {
        (
            CounterfactualBackfillGapKind::MissingCommitteeDecision,
            CounterfactualBackfillSuggestedAction::ProvideCommitteeDecision,
            ReasonCode::CounterfactualEvaluated,
        )
    } else if !row.no_trade_counterfactual_available && can_build_no_trade {
        (
            CounterfactualBackfillGapKind::MissingNoTradeCounterfactual,
            CounterfactualBackfillSuggestedAction::BuildNoTradeCounterfactual,
            ReasonCode::NoTradeCounterfactual,
        )
    } else if !row.risk_denied_counterfactual_available && can_build_risk_denied {
        (
            CounterfactualBackfillGapKind::MissingRiskDeniedCounterfactual,
            CounterfactualBackfillSuggestedAction::BuildRiskDeniedCounterfactual,
            ReasonCode::RiskDeniedCounterfactual,
        )
    } else {
        (
            CounterfactualBackfillGapKind::DiagnosticOnly,
            CounterfactualBackfillSuggestedAction::NoSafeAction,
            ReasonCode::CounterfactualEvaluated,
        )
    };
    reason_codes.push(extra_reason);
    CounterfactualBackfillPlanItem {
        row_id: row.row_id.clone(),
        gap_kind,
        can_build_no_trade,
        can_build_risk_denied,
        suggested_action,
        reason_codes: stable_reason_codes(&reason_codes),
    }
}
