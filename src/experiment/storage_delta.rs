use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;
use crate::data::OfficialCollectionReport;
use crate::experiment::CoreCheckedBenchmarkReport;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfficialStorageDelta {
    pub previous_total_bytes: Option<usize>,
    pub current_total_bytes: usize,
    pub added_bytes: isize,
    pub added_raw_bytes: isize,
    pub added_canonical_bytes: isize,
    pub added_report_bytes: isize,
    pub budget_exceeded: bool,
    pub largest_new_artifacts: Vec<String>,
    pub compaction_recommendation: String,
    pub reason_codes: Vec<ReasonCode>,
}

impl OfficialStorageDelta {
    pub fn to_text(&self) -> String {
        [
            format!(
                "previous_total_bytes={}",
                self.previous_total_bytes
                    .map(|value| value.to_string())
                    .unwrap_or_default()
            ),
            format!("current_total_bytes={}", self.current_total_bytes),
            format!("added_bytes={}", self.added_bytes),
            format!("added_raw_bytes={}", self.added_raw_bytes),
            format!("added_canonical_bytes={}", self.added_canonical_bytes),
            format!("added_report_bytes={}", self.added_report_bytes),
            format!("budget_exceeded={}", self.budget_exceeded),
            format!(
                "largest_new_artifacts={}",
                self.largest_new_artifacts.join("|")
            ),
            format!(
                "compaction_recommendation={}",
                self.compaction_recommendation
            ),
        ]
        .join("\n")
    }
}

pub fn build_official_storage_delta(
    previous_report: Option<&CoreCheckedBenchmarkReport>,
    current_report: Option<&CoreCheckedBenchmarkReport>,
    collection_report: Option<&OfficialCollectionReport>,
    max_storage_bytes: usize,
) -> OfficialStorageDelta {
    let previous_total_bytes = previous_report.map(total_bytes_from_core_report);
    let current_total_bytes = current_report
        .map(total_bytes_from_core_report)
        .or_else(|| collection_report.map(|report| report.storage_budget_report.total_bytes))
        .unwrap_or_default();
    let previous_raw_bytes = previous_report
        .map(|report| report.storage_audit.raw_archive_bytes)
        .unwrap_or_default();
    let current_raw_bytes = current_report
        .map(|report| report.storage_audit.raw_archive_bytes)
        .or_else(|| collection_report.map(|report| report.storage_budget_report.raw_bytes))
        .unwrap_or_default();
    let previous_canonical_bytes = previous_report
        .map(|report| report.storage_audit.canonical_bytes)
        .unwrap_or_default();
    let current_canonical_bytes = current_report
        .map(|report| report.storage_audit.canonical_bytes)
        .or_else(|| collection_report.map(|report| report.storage_budget_report.canonical_bytes))
        .unwrap_or_default();
    let previous_report_bytes = previous_report
        .map(|report| report.storage_audit.report_bytes)
        .unwrap_or_default();
    let current_report_bytes = current_report
        .map(|report| report.storage_audit.report_bytes)
        .unwrap_or_default();
    let budget_exceeded = current_total_bytes > max_storage_bytes
        || current_report.is_some_and(|report| report.storage_audit.budget_exceeded);
    let largest_new_artifacts = current_report
        .map(|report| {
            let mut files = report.storage_audit.largest_files.clone();
            files.sort();
            files
        })
        .unwrap_or_default();
    let compaction_recommendation = if budget_exceeded {
        "Stop expansion and compact bounded artifacts before collecting more.".to_string()
    } else if current_total_bytes > (max_storage_bytes.saturating_mul(9) / 10) {
        "Approaching storage budget; keep compact outputs and avoid extra venues until reviewed."
            .to_string()
    } else {
        "Storage remains within bounded research budget.".to_string()
    };

    let mut reason_codes = vec![ReasonCode::OfficialStorageDeltaBuilt];
    if budget_exceeded {
        reason_codes.push(ReasonCode::BudgetExceeded);
    } else if current_total_bytes > (max_storage_bytes.saturating_mul(9) / 10) {
        reason_codes.push(ReasonCode::CompactionRecommended);
    }

    OfficialStorageDelta {
        previous_total_bytes,
        current_total_bytes,
        added_bytes: current_total_bytes as isize
            - previous_total_bytes.unwrap_or_default() as isize,
        added_raw_bytes: current_raw_bytes as isize - previous_raw_bytes as isize,
        added_canonical_bytes: current_canonical_bytes as isize - previous_canonical_bytes as isize,
        added_report_bytes: current_report_bytes as isize - previous_report_bytes as isize,
        budget_exceeded,
        largest_new_artifacts,
        compaction_recommendation,
        reason_codes,
    }
}

fn total_bytes_from_core_report(report: &CoreCheckedBenchmarkReport) -> usize {
    report.storage_audit.collection_bytes
        + report.storage_audit.dataset_export_bytes
        + report.storage_audit.prediction_bytes
        + report.storage_audit.report_bytes
}
