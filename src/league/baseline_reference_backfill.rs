use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_hash_string, stable_reason_codes};

use super::comparable_committee_evidence::ComparableCommitteeEvidenceRow;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum BaselineBackfillSource {
    ExistingBaselineArtifact,
    DeterministicNoTradeBaseline,
    DeterministicBaselineApproximation,
    #[default]
    Unavailable,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BaselineReferenceBackfillPlanItem {
    pub row_id: String,
    pub source: BaselineBackfillSource,
    pub can_backfill: bool,
    pub diagnostic_only: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BaselineReferenceBackfillPlan {
    pub plan_id: String,
    pub items: Vec<BaselineReferenceBackfillPlanItem>,
    pub existing_artifact_count: usize,
    pub no_trade_fallback_count: usize,
    pub approximation_count: usize,
    pub unavailable_count: usize,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

pub fn build_baseline_reference_backfill_plan(
    plan_id: impl Into<String>,
    rows: &[ComparableCommitteeEvidenceRow],
) -> BaselineReferenceBackfillPlan {
    let plan_id = plan_id.into();
    let mut items = rows
        .iter()
        .filter(|row| !row.baseline_reference_available)
        .map(build_item)
        .collect::<Vec<_>>();
    items.sort_by(|left, right| left.row_id.cmp(&right.row_id));
    let existing_artifact_count = items
        .iter()
        .filter(|item| item.source == BaselineBackfillSource::ExistingBaselineArtifact)
        .count();
    let no_trade_fallback_count = items
        .iter()
        .filter(|item| item.source == BaselineBackfillSource::DeterministicNoTradeBaseline)
        .count();
    let approximation_count = items
        .iter()
        .filter(|item| item.source == BaselineBackfillSource::DeterministicBaselineApproximation)
        .count();
    let unavailable_count = items
        .iter()
        .filter(|item| item.source == BaselineBackfillSource::Unavailable)
        .count();
    BaselineReferenceBackfillPlan {
        plan_id,
        items,
        existing_artifact_count,
        no_trade_fallback_count,
        approximation_count,
        unavailable_count,
        reason_codes: stable_reason_codes(&[
            ReasonCode::CommitteeReferencePackBuilt,
            ReasonCode::DeterministicPath,
        ]),
    }
}

impl BaselineReferenceBackfillPlan {
    pub fn fingerprint(&self) -> String {
        stable_hash_string(&serde_json::to_string(self).unwrap_or_else(|_| self.plan_id.clone()))
    }

    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("plan_id={}", self.plan_id),
            format!("existing_artifact_count={}", self.existing_artifact_count),
            format!("no_trade_fallback_count={}", self.no_trade_fallback_count),
            format!("approximation_count={}", self.approximation_count),
            format!("unavailable_count={}", self.unavailable_count),
            format!("fingerprint={}", self.fingerprint()),
        ];
        lines.extend(self.items.iter().map(|item| {
            format!(
                "row_id={};source={:?};can_backfill={};diagnostic_only={}",
                item.row_id, item.source, item.can_backfill, item.diagnostic_only,
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
            output_dir.join("baseline_reference_backfill_plan.txt"),
            self.to_text(),
        )
        .map_err(|err| err.to_string())?;
        let json_path = output_dir.join("baseline_reference_backfill_plan.json");
        fs::write(&json_path, self.to_json_string()?).map_err(|err| err.to_string())?;
        Ok(json_path)
    }
}

fn build_item(row: &ComparableCommitteeEvidenceRow) -> BaselineReferenceBackfillPlanItem {
    let (source, can_backfill, diagnostic_only, extra_reason) = if row
        .baseline_action
        .as_ref()
        .is_some_and(|value| !value.trim().is_empty() && !value.eq_ignore_ascii_case("NoTrade"))
    {
        (
            BaselineBackfillSource::ExistingBaselineArtifact,
            true,
            false,
            ReasonCode::CommitteeReferencePackBuilt,
        )
    } else if !row.no_trade_baseline_action.trim().is_empty() {
        (
            BaselineBackfillSource::DeterministicNoTradeBaseline,
            true,
            false,
            ReasonCode::NoTradePreferred,
        )
    } else if !row.committee_final_action.trim().is_empty() {
        (
            BaselineBackfillSource::DeterministicBaselineApproximation,
            true,
            true,
            ReasonCode::ResearchOnlyOverride,
        )
    } else {
        (
            BaselineBackfillSource::Unavailable,
            false,
            false,
            ReasonCode::MissingRealLocalData,
        )
    };
    BaselineReferenceBackfillPlanItem {
        row_id: row.row_id.clone(),
        source,
        can_backfill,
        diagnostic_only,
        reason_codes: stable_reason_codes(
            &row.reason_codes
                .iter()
                .cloned()
                .chain([extra_reason])
                .collect::<Vec<_>>(),
        ),
    }
}
