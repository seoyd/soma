use serde::{Deserialize, Serialize};

use crate::core::{ArtifactSize, ReasonCode, stable_reason_codes};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoreLatencyBudgetConfig {
    pub max_scorecard_artifacts: usize,
    pub max_rows: usize,
    pub max_report_bytes: usize,
    pub max_artifact_bytes: usize,
    pub max_decision_path_steps: usize,
    #[serde(default)]
    pub target_decision_latency_ms: Option<u64>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CoreLatencyBudgetStatus {
    #[default]
    WithinBudget,
    StorageBudgetExceeded,
    LatencyBudgetExceeded,
    TooManyArtifacts,
    TooManyRows,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoreLatencyBudgetReport {
    pub artifact_count: usize,
    pub row_count: usize,
    pub report_bytes: usize,
    pub artifact_bytes: usize,
    pub decision_path_steps: usize,
    #[serde(default)]
    pub estimated_decision_latency_ms: Option<u64>,
    pub storage_budget_exceeded: bool,
    pub latency_budget_exceeded: bool,
    pub largest_artifacts: Vec<ArtifactSize>,
    pub budget_status: CoreLatencyBudgetStatus,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for CoreLatencyBudgetConfig {
    fn default() -> Self {
        Self {
            max_scorecard_artifacts: 64,
            max_rows: 10_000,
            max_report_bytes: 2 * 1024 * 1024,
            max_artifact_bytes: 5 * 1024 * 1024,
            max_decision_path_steps: 16,
            target_decision_latency_ms: Some(80),
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

pub fn build_core_latency_budget_report(
    config: &CoreLatencyBudgetConfig,
    artifacts: &[ArtifactSize],
    row_count: usize,
    report_bytes: usize,
    decision_path_steps: usize,
) -> CoreLatencyBudgetReport {
    let mut largest_artifacts = artifacts.to_vec();
    largest_artifacts.sort_by(|left, right| {
        right
            .bytes
            .cmp(&left.bytes)
            .then_with(|| left.path.cmp(&right.path))
    });

    let artifact_count = artifacts.len();
    let artifact_bytes = artifacts
        .iter()
        .map(|artifact| artifact.bytes)
        .sum::<usize>();
    let estimated_decision_latency_ms = config
        .target_decision_latency_ms
        .map(|_| (decision_path_steps as u64).saturating_mul(5));
    let storage_budget_exceeded =
        artifact_bytes > config.max_artifact_bytes || report_bytes > config.max_report_bytes;
    let latency_budget_exceeded = decision_path_steps > config.max_decision_path_steps
        || config
            .target_decision_latency_ms
            .zip(estimated_decision_latency_ms)
            .is_some_and(|(target, observed)| observed > target);

    let budget_status = if artifact_count > config.max_scorecard_artifacts {
        CoreLatencyBudgetStatus::TooManyArtifacts
    } else if row_count > config.max_rows {
        CoreLatencyBudgetStatus::TooManyRows
    } else if storage_budget_exceeded {
        CoreLatencyBudgetStatus::StorageBudgetExceeded
    } else if latency_budget_exceeded {
        CoreLatencyBudgetStatus::LatencyBudgetExceeded
    } else {
        CoreLatencyBudgetStatus::WithinBudget
    };

    let mut reason_codes = config.reason_codes.clone();
    reason_codes.push(ReasonCode::CoreLatencyBudgetReportBuilt);
    if artifact_count > config.max_scorecard_artifacts
        || row_count > config.max_rows
        || storage_budget_exceeded
        || latency_budget_exceeded
    {
        reason_codes.push(ReasonCode::BudgetExceeded);
        reason_codes.push(ReasonCode::CompactionRecommended);
    }

    CoreLatencyBudgetReport {
        artifact_count,
        row_count,
        report_bytes,
        artifact_bytes,
        decision_path_steps,
        estimated_decision_latency_ms,
        storage_budget_exceeded,
        latency_budget_exceeded,
        largest_artifacts: largest_artifacts.into_iter().take(10).collect(),
        budget_status,
        reason_codes: stable_reason_codes(&reason_codes),
    }
}

impl CoreLatencyBudgetReport {
    pub fn to_text(&self) -> String {
        let largest = self
            .largest_artifacts
            .iter()
            .map(|artifact| format!("{}:{}", artifact.path, artifact.bytes))
            .collect::<Vec<_>>()
            .join("|");
        [
            format!("artifact_count={}", self.artifact_count),
            format!("row_count={}", self.row_count),
            format!("report_bytes={}", self.report_bytes),
            format!("artifact_bytes={}", self.artifact_bytes),
            format!("decision_path_steps={}", self.decision_path_steps),
            format!(
                "estimated_decision_latency_ms={}",
                self.estimated_decision_latency_ms
                    .map(|value| value.to_string())
                    .unwrap_or_default()
            ),
            format!("storage_budget_exceeded={}", self.storage_budget_exceeded),
            format!("latency_budget_exceeded={}", self.latency_budget_exceeded),
            format!("budget_status={:?}", self.budget_status),
            format!("largest_artifacts={largest}"),
        ]
        .join("\n")
    }
}
