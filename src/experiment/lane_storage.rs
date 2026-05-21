use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;
use crate::data::EvidenceSourceKind;

use super::evidence_lane::EvidenceLane;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LaneStorageBudget {
    pub estimated_bytes: usize,
    pub max_total_bytes: usize,
    pub max_rows: usize,
    pub max_requests: usize,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LaneStorageBudgetReport {
    pub lane_id: String,
    pub estimated_bytes: usize,
    #[serde(default)]
    pub actual_bytes: Option<usize>,
    pub budget_ok: bool,
    pub largest_artifacts: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderRealityStorageReport {
    pub lane_reports: Vec<LaneStorageBudgetReport>,
    pub total_estimated_bytes: usize,
    pub total_actual_bytes: usize,
    pub budget_exceeded: bool,
    pub compaction_recommendation: String,
    pub reason_codes: Vec<ReasonCode>,
}

pub fn default_lane_storage_budget(
    source_kind: EvidenceSourceKind,
    max_rows: usize,
    max_requests: usize,
    max_total_bytes: usize,
) -> LaneStorageBudget {
    let bytes_per_row = match source_kind {
        EvidenceSourceKind::OfficialApiCollected | EvidenceSourceKind::RealLocal => 96,
        EvidenceSourceKind::YFinanceResearch => 72,
        EvidenceSourceKind::SyntheticFixture
        | EvidenceSourceKind::TestFixture
        | EvidenceSourceKind::GeneratedSynthetic => 48,
        EvidenceSourceKind::ExternalPredictionOnly | EvidenceSourceKind::Unknown => 64,
    };
    LaneStorageBudget {
        estimated_bytes: max_rows
            .saturating_mul(bytes_per_row)
            .saturating_add(max_requests.saturating_mul(256)),
        max_total_bytes,
        max_rows,
        max_requests,
        reason_codes: vec![ReasonCode::LaneStorageBudgetBuilt],
    }
}

pub fn build_lane_storage_budget_report(
    lane: &EvidenceLane,
    actual_bytes: Option<usize>,
) -> LaneStorageBudgetReport {
    let actual = actual_bytes.unwrap_or(lane.storage_budget.estimated_bytes);
    LaneStorageBudgetReport {
        lane_id: lane.lane_id.clone(),
        estimated_bytes: lane.storage_budget.estimated_bytes,
        actual_bytes: Some(actual),
        budget_ok: actual <= lane.storage_budget.max_total_bytes,
        largest_artifacts: vec![
            format!("{}/manifest.json", lane.collection_policy.output_subdir),
            format!("{}/dataset.csv", lane.collection_policy.output_subdir),
        ],
        reason_codes: vec![ReasonCode::LaneStorageBudgetBuilt],
    }
}

pub fn build_provider_reality_storage_report(
    mut lane_reports: Vec<LaneStorageBudgetReport>,
    max_total_bytes: usize,
) -> ProviderRealityStorageReport {
    lane_reports.sort_by(|left, right| left.lane_id.cmp(&right.lane_id));
    let total_estimated_bytes: usize = lane_reports
        .iter()
        .map(|report| report.estimated_bytes)
        .sum();
    let total_actual_bytes: usize = lane_reports
        .iter()
        .map(|report| report.actual_bytes.unwrap_or(report.estimated_bytes))
        .sum();
    let budget_exceeded =
        total_estimated_bytes > max_total_bytes || total_actual_bytes > max_total_bytes;
    let compaction_recommendation = if budget_exceeded {
        "ReduceCollectionScope".to_string()
    } else if total_estimated_bytes.saturating_mul(100) >= max_total_bytes.saturating_mul(85) {
        "Near budget limit; keep lane count and row count compact".to_string()
    } else {
        "Budget healthy for bounded research-only evidence".to_string()
    };
    ProviderRealityStorageReport {
        lane_reports,
        total_estimated_bytes,
        total_actual_bytes,
        budget_exceeded,
        compaction_recommendation,
        reason_codes: vec![ReasonCode::ProviderRealityStorageReportBuilt],
    }
}
