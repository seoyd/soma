use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KISActivationStorageReport {
    pub raw_archive_bytes: usize,
    pub canonical_csv_bytes: usize,
    pub provenance_bytes: usize,
    pub manifest_bytes: usize,
    pub preflight_bytes: usize,
    pub candle_pack_bytes: usize,
    pub outcome_linkage_bytes: usize,
    pub counterfactual_bytes: usize,
    pub downstream_bundle_bytes: usize,
    pub report_bytes: usize,
    pub total_bytes: usize,
    pub budget_exceeded: bool,
    pub largest_artifacts: Vec<String>,
    pub compaction_recommendation: String,
    pub reason_codes: Vec<ReasonCode>,
}

impl KISActivationStorageReport {
    #[allow(clippy::too_many_arguments)]
    pub fn build(
        raw_archive_paths: &[String],
        canonical_paths: &[String],
        provenance_paths: &[String],
        manifest_paths: &[String],
        preflight_paths: &[String],
        candle_pack_paths: &[String],
        outcome_linkage_paths: &[String],
        counterfactual_paths: &[String],
        downstream_paths: &[String],
        report_paths: &[String],
        max_bytes: usize,
    ) -> Self {
        let raw_archive_bytes = total_bytes(raw_archive_paths);
        let canonical_csv_bytes = total_bytes(canonical_paths);
        let provenance_bytes = total_bytes(provenance_paths);
        let manifest_bytes = total_bytes(manifest_paths);
        let preflight_bytes = total_bytes(preflight_paths);
        let candle_pack_bytes = total_bytes(candle_pack_paths);
        let outcome_linkage_bytes = total_bytes(outcome_linkage_paths);
        let counterfactual_bytes = total_bytes(counterfactual_paths);
        let downstream_bundle_bytes = total_bytes(downstream_paths);
        let report_bytes = total_bytes(report_paths);
        let total_bytes = raw_archive_bytes
            + canonical_csv_bytes
            + provenance_bytes
            + manifest_bytes
            + preflight_bytes
            + candle_pack_bytes
            + outcome_linkage_bytes
            + counterfactual_bytes
            + downstream_bundle_bytes
            + report_bytes;
        let mut largest_artifacts = raw_archive_paths
            .iter()
            .chain(canonical_paths.iter())
            .chain(provenance_paths.iter())
            .chain(manifest_paths.iter())
            .chain(preflight_paths.iter())
            .chain(candle_pack_paths.iter())
            .chain(outcome_linkage_paths.iter())
            .chain(counterfactual_paths.iter())
            .chain(downstream_paths.iter())
            .chain(report_paths.iter())
            .filter_map(|path| artifact_line(path))
            .collect::<Vec<_>>();
        largest_artifacts.sort_by(|left, right| {
            parse_line(right)
                .cmp(&parse_line(left))
                .then(left.cmp(right))
        });
        let largest_artifacts = largest_artifacts.into_iter().take(10).collect::<Vec<_>>();
        let budget_exceeded = total_bytes > max_bytes;
        let mut reason_codes = vec![ReasonCode::KISActivationStorageReportBuilt];
        if budget_exceeded {
            reason_codes.push(ReasonCode::BudgetExceeded);
            reason_codes.push(ReasonCode::CompactionRecommended);
        } else {
            reason_codes.push(ReasonCode::StorageBudgetReportBuilt);
        }
        Self {
            raw_archive_bytes,
            canonical_csv_bytes,
            provenance_bytes,
            manifest_bytes,
            preflight_bytes,
            candle_pack_bytes,
            outcome_linkage_bytes,
            counterfactual_bytes,
            downstream_bundle_bytes,
            report_bytes,
            total_bytes,
            budget_exceeded,
            largest_artifacts,
            compaction_recommendation: if budget_exceeded {
                "reduce scope or archive non-essential downstream reports; canonical evidence is retained".to_string()
            } else {
                "within budget".to_string()
            },
            reason_codes: stable_reason_codes(&reason_codes),
        }
    }

    pub fn to_text(&self) -> String {
        [
            format!("raw_archive_bytes={}", self.raw_archive_bytes),
            format!("canonical_csv_bytes={}", self.canonical_csv_bytes),
            format!("provenance_bytes={}", self.provenance_bytes),
            format!("manifest_bytes={}", self.manifest_bytes),
            format!("preflight_bytes={}", self.preflight_bytes),
            format!("candle_pack_bytes={}", self.candle_pack_bytes),
            format!("outcome_linkage_bytes={}", self.outcome_linkage_bytes),
            format!("counterfactual_bytes={}", self.counterfactual_bytes),
            format!("downstream_bundle_bytes={}", self.downstream_bundle_bytes),
            format!("report_bytes={}", self.report_bytes),
            format!("total_bytes={}", self.total_bytes),
            format!("budget_exceeded={}", self.budget_exceeded),
            format!("largest_artifacts={}", self.largest_artifacts.join("|")),
            format!(
                "compaction_recommendation={}",
                self.compaction_recommendation
            ),
        ]
        .join("\n")
    }

    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<(), String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        fs::write(output_dir.join("storage_report.txt"), self.to_text())
            .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("storage_report.json"),
            self.to_json_string()?,
        )
        .map_err(|err| err.to_string())?;
        Ok(())
    }
}

fn total_bytes(paths: &[String]) -> usize {
    paths
        .iter()
        .map(|path| {
            fs::metadata(path)
                .map(|metadata| metadata.len() as usize)
                .unwrap_or(0)
        })
        .sum()
}

fn artifact_line(path: &str) -> Option<String> {
    let bytes = fs::metadata(path).ok()?.len() as usize;
    Some(format!("{}={}", path, bytes))
}

fn parse_line(value: &str) -> usize {
    value
        .rsplit('=')
        .next()
        .unwrap_or_default()
        .parse()
        .unwrap_or(0)
}
