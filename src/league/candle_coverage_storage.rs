use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CandleCoverageArtifactSize {
    pub path: String,
    pub bytes: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CandleCoverageStorageReport {
    pub candle_pack_bytes: usize,
    pub backfilled_bundle_bytes: usize,
    pub generated_reference_bytes: usize,
    pub output_report_bytes: usize,
    pub total_bytes: usize,
    pub budget_exceeded: bool,
    pub largest_artifacts: Vec<CandleCoverageArtifactSize>,
    pub compaction_recommendation: String,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

pub fn build_candle_coverage_storage_report(
    candle_pack_bytes: usize,
    backfilled_bundle_bytes: usize,
    generated_reference_bytes: usize,
    output_report_bytes: usize,
    budget_bytes: usize,
    mut artifacts: Vec<CandleCoverageArtifactSize>,
) -> CandleCoverageStorageReport {
    artifacts.sort_by(|left, right| {
        right
            .bytes
            .cmp(&left.bytes)
            .then(left.path.cmp(&right.path))
    });
    let total_bytes = candle_pack_bytes
        .saturating_add(backfilled_bundle_bytes)
        .saturating_add(generated_reference_bytes)
        .saturating_add(output_report_bytes);
    let budget_exceeded = budget_bytes > 0 && total_bytes > budget_bytes;
    CandleCoverageStorageReport {
        candle_pack_bytes,
        backfilled_bundle_bytes,
        generated_reference_bytes,
        output_report_bytes,
        total_bytes,
        budget_exceeded,
        largest_artifacts: artifacts,
        compaction_recommendation: if budget_exceeded {
            "budget exceeded: compact largest artifacts before adding more candle coverage"
                .to_string()
        } else {
            "within budget: no compaction required".to_string()
        },
        reason_codes: stable_reason_codes(
            &[ReasonCode::DeterministicPath]
                .iter()
                .cloned()
                .chain(budget_exceeded.then_some(ReasonCode::BudgetExceeded))
                .chain(budget_exceeded.then_some(ReasonCode::CompactionRecommended))
                .collect::<Vec<_>>(),
        ),
    }
}

impl CandleCoverageStorageReport {
    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("candle_pack_bytes={}", self.candle_pack_bytes),
            format!("backfilled_bundle_bytes={}", self.backfilled_bundle_bytes),
            format!(
                "generated_reference_bytes={}",
                self.generated_reference_bytes
            ),
            format!("output_report_bytes={}", self.output_report_bytes),
            format!("total_bytes={}", self.total_bytes),
            format!("budget_exceeded={}", self.budget_exceeded),
            format!(
                "compaction_recommendation={}",
                self.compaction_recommendation
            ),
        ];
        lines.extend(
            self.largest_artifacts
                .iter()
                .map(|artifact| format!("artifact={};bytes={}", artifact.path, artifact.bytes)),
        );
        lines.join("\n")
    }
}
