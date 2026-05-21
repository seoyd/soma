use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};

use super::comparable_committee_evidence::ComparableCommitteeEvidenceBundle;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ScenarioMaterializationWeakClosureStatus {
    MaterializationImproved,
    #[default]
    MaterializationStillWeak,
    MaterializationBlockedByMissingArtifacts,
    MaterializationBlockedByMissingOfficialData,
    MaterializationBlockedBySourceIneligible,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ScenarioMaterializationWeakClosureReport {
    #[serde(default)]
    pub row_level_before: Option<f64>,
    pub row_level_after: f64,
    #[serde(default)]
    pub summary_derived_before: Option<f64>,
    pub summary_derived_after: f64,
    #[serde(default)]
    pub comparable_rows_before: Option<usize>,
    pub comparable_rows_after: usize,
    #[serde(default)]
    pub complete_rows_before: Option<usize>,
    pub complete_rows_after: usize,
    #[serde(default)]
    pub official_complete_rows_before: Option<usize>,
    pub official_complete_rows_after: usize,
    pub remaining_summary_derived_gaps: usize,
    pub remaining_materialization_gaps: usize,
    pub status: ScenarioMaterializationWeakClosureStatus,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

pub fn build_scenario_materialization_weak_closure_report(
    before: Option<&ComparableCommitteeEvidenceBundle>,
    after: &ComparableCommitteeEvidenceBundle,
) -> ScenarioMaterializationWeakClosureReport {
    let row_level_before = before.map(|bundle| ratio(bundle.row_level_count, bundle.rows.len()));
    let row_level_after = ratio(after.row_level_count, after.rows.len());
    let summary_derived_before =
        before.map(|bundle| ratio(bundle.summary_derived_count, bundle.rows.len()));
    let summary_derived_after = ratio(after.summary_derived_count, after.rows.len());
    let comparable_rows_before = before.map(|bundle| bundle.rows.len());
    let comparable_rows_after = after.rows.len();
    let complete_rows_before = before.map(|bundle| bundle.complete_rows);
    let complete_rows_after = after.complete_rows;
    let official_complete_rows_before =
        before.map(|bundle| bundle.non_crypto_official_rows.min(bundle.complete_rows));
    let official_complete_rows_after = after
        .rows
        .iter()
        .filter(|row| row.official_readiness_eligible && row.no_lookahead_safe)
        .count();
    let remaining_summary_derived_gaps = after.summary_derived_count;
    let remaining_materialization_gaps = after.incomplete_rows + after.summary_derived_count;

    let status = if before.is_some()
        && row_level_after > row_level_before.unwrap_or_default()
        && summary_derived_after <= summary_derived_before.unwrap_or(1.0)
    {
        ScenarioMaterializationWeakClosureStatus::MaterializationImproved
    } else if after.rows.is_empty() {
        ScenarioMaterializationWeakClosureStatus::MaterializationBlockedByMissingArtifacts
    } else if after.non_crypto_official_rows == 0
        && after.rows.iter().any(|row| row.diagnostic_only)
        && !after.rows.iter().all(|row| row.diagnostic_only)
    {
        ScenarioMaterializationWeakClosureStatus::MaterializationBlockedByMissingOfficialData
    } else if after.rows.iter().all(|row| row.diagnostic_only) {
        ScenarioMaterializationWeakClosureStatus::MaterializationBlockedBySourceIneligible
    } else if after.outcome_reference_count == 0 || after.baseline_reference_count == 0 {
        ScenarioMaterializationWeakClosureStatus::MaterializationBlockedByMissingArtifacts
    } else {
        ScenarioMaterializationWeakClosureStatus::MaterializationStillWeak
    };

    ScenarioMaterializationWeakClosureReport {
        row_level_before,
        row_level_after,
        summary_derived_before,
        summary_derived_after,
        comparable_rows_before,
        comparable_rows_after,
        complete_rows_before,
        complete_rows_after,
        official_complete_rows_before,
        official_complete_rows_after,
        remaining_summary_derived_gaps,
        remaining_materialization_gaps,
        status,
        reason_codes: stable_reason_codes(&[
            ReasonCode::SummaryDerived,
            ReasonCode::DeterministicPath,
        ]),
    }
}

impl ScenarioMaterializationWeakClosureReport {
    pub fn to_text(&self) -> String {
        [
            format!(
                "row_level_before={:.6}",
                self.row_level_before.unwrap_or_default()
            ),
            format!("row_level_after={:.6}", self.row_level_after),
            format!(
                "summary_derived_before={:.6}",
                self.summary_derived_before.unwrap_or_default()
            ),
            format!("summary_derived_after={:.6}", self.summary_derived_after),
            format!(
                "comparable_rows_before={}",
                self.comparable_rows_before.unwrap_or_default()
            ),
            format!("comparable_rows_after={}", self.comparable_rows_after),
            format!(
                "complete_rows_before={}",
                self.complete_rows_before.unwrap_or_default()
            ),
            format!("complete_rows_after={}", self.complete_rows_after),
            format!(
                "official_complete_rows_before={}",
                self.official_complete_rows_before.unwrap_or_default()
            ),
            format!(
                "official_complete_rows_after={}",
                self.official_complete_rows_after
            ),
            format!(
                "remaining_summary_derived_gaps={}",
                self.remaining_summary_derived_gaps
            ),
            format!(
                "remaining_materialization_gaps={}",
                self.remaining_materialization_gaps
            ),
            format!("status={:?}", self.status),
        ]
        .join("\n")
    }
}

fn ratio(count: usize, total: usize) -> f64 {
    count as f64 / total.max(1) as f64
}
