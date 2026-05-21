use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SourceAwareStorageAudit {
    pub official_artifact_bytes: usize,
    pub yfinance_artifact_bytes: usize,
    pub comparison_report_bytes: usize,
    pub total_bytes: usize,
    pub budget_exceeded: bool,
    pub largest_artifacts: Vec<String>,
    #[serde(default)]
    pub compaction_recommendation: Option<String>,
    pub reason_codes: Vec<ReasonCode>,
}

pub fn build_source_aware_storage_audit(
    official_artifact_bytes: usize,
    yfinance_artifact_bytes: usize,
    comparison_report_bytes: usize,
    largest_artifacts: Vec<String>,
    max_storage_bytes: usize,
) -> SourceAwareStorageAudit {
    let total_bytes = official_artifact_bytes + yfinance_artifact_bytes + comparison_report_bytes;
    let budget_exceeded = total_bytes > max_storage_bytes;
    SourceAwareStorageAudit {
        official_artifact_bytes,
        yfinance_artifact_bytes,
        comparison_report_bytes,
        total_bytes,
        budget_exceeded,
        largest_artifacts,
        compaction_recommendation: budget_exceeded.then(|| {
            "comparison artifacts exceeded storage budget; compact or trim report inputs"
                .to_string()
        }),
        reason_codes: vec![ReasonCode::SourceStorageAuditBuilt],
    }
}
