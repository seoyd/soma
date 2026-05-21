use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_hash_string};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactSize {
    pub path: String,
    pub bytes: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CorePerformanceBudget {
    pub max_dataset_rows: usize,
    pub max_feature_rows: usize,
    pub max_prediction_rows: usize,
    pub max_report_bytes: usize,
    pub max_artifact_bytes: usize,
    pub max_collection_requests: usize,
    pub max_collection_rows: usize,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CorePerformanceBudgetReport {
    pub dataset_rows: usize,
    pub feature_rows: usize,
    pub prediction_rows: usize,
    pub report_bytes: usize,
    pub artifact_bytes: usize,
    pub collection_requests: usize,
    pub collection_rows: usize,
    pub budget_exceeded: bool,
    pub largest_artifacts: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for CorePerformanceBudget {
    fn default() -> Self {
        Self {
            max_dataset_rows: 50_000,
            max_feature_rows: 50_000,
            max_prediction_rows: 50_000,
            max_report_bytes: 2 * 1024 * 1024,
            max_artifact_bytes: 8 * 1024 * 1024,
            max_collection_requests: 1_000,
            max_collection_rows: 500_000,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn measure_performance_budget(
    budget: &CorePerformanceBudget,
    dataset_rows: usize,
    feature_rows: usize,
    prediction_rows: usize,
    report_bytes: usize,
    artifact_bytes: usize,
    collection_requests: usize,
    collection_rows: usize,
    artifacts: &[ArtifactSize],
) -> CorePerformanceBudgetReport {
    let mut largest = artifacts.to_vec();
    largest.sort_by(|left, right| {
        right
            .bytes
            .cmp(&left.bytes)
            .then_with(|| left.path.cmp(&right.path))
    });
    let budget_exceeded = dataset_rows > budget.max_dataset_rows
        || feature_rows > budget.max_feature_rows
        || prediction_rows > budget.max_prediction_rows
        || report_bytes > budget.max_report_bytes
        || artifact_bytes > budget.max_artifact_bytes
        || collection_requests > budget.max_collection_requests
        || collection_rows > budget.max_collection_rows;
    let mut reason_codes = vec![ReasonCode::CorePerformanceBudgetBuilt];
    if budget_exceeded {
        reason_codes.push(ReasonCode::BudgetExceeded);
        reason_codes.push(ReasonCode::CompactionRecommended);
    }
    CorePerformanceBudgetReport {
        dataset_rows,
        feature_rows,
        prediction_rows,
        report_bytes,
        artifact_bytes,
        collection_requests,
        collection_rows,
        budget_exceeded,
        largest_artifacts: largest
            .into_iter()
            .take(10)
            .map(|item| format!("{}:{}", item.path, item.bytes))
            .collect(),
        reason_codes,
    }
}

impl CorePerformanceBudgetReport {
    pub fn fingerprint(&self) -> String {
        stable_hash_string(&format!(
            "{}|{}|{}|{}|{}|{}|{}|{}",
            self.dataset_rows,
            self.feature_rows,
            self.prediction_rows,
            self.report_bytes,
            self.artifact_bytes,
            self.collection_requests,
            self.collection_rows,
            self.largest_artifacts.join("|")
        ))
    }

    pub fn to_text(&self) -> String {
        [
            format!("dataset_rows={}", self.dataset_rows),
            format!("feature_rows={}", self.feature_rows),
            format!("prediction_rows={}", self.prediction_rows),
            format!("report_bytes={}", self.report_bytes),
            format!("artifact_bytes={}", self.artifact_bytes),
            format!("collection_requests={}", self.collection_requests),
            format!("collection_rows={}", self.collection_rows),
            format!("budget_exceeded={}", self.budget_exceeded),
            format!("largest_artifacts={}", self.largest_artifacts.join("|")),
            format!("fingerprint={}", self.fingerprint()),
        ]
        .join("\n")
    }
}
