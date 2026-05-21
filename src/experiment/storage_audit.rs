use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;
use crate::data::OfficialCollectionReport;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BenchmarkStorageAudit {
    pub collection_bytes: usize,
    pub dataset_export_bytes: usize,
    pub prediction_bytes: usize,
    pub report_bytes: usize,
    pub raw_archive_bytes: usize,
    pub canonical_bytes: usize,
    pub budget_exceeded: bool,
    pub largest_files: Vec<String>,
    pub retention_actions: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

impl BenchmarkStorageAudit {
    pub fn build(
        collection_report: &OfficialCollectionReport,
        bundle_dirs: &[PathBuf],
        benchmark_root: &Path,
    ) -> Self {
        let mut dataset_export_bytes = 0usize;
        let mut prediction_bytes = 0usize;
        let mut report_bytes = 0usize;
        let mut largest_files = Vec::new();

        for dir in bundle_dirs {
            visit_files(dir, &mut |path, bytes| {
                let name = path
                    .file_name()
                    .and_then(|value| value.to_str())
                    .unwrap_or_default();
                if name == "dataset.csv" {
                    dataset_export_bytes = dataset_export_bytes.saturating_add(bytes);
                } else if name == "predictions.csv" {
                    prediction_bytes = prediction_bytes.saturating_add(bytes);
                } else {
                    report_bytes = report_bytes.saturating_add(bytes);
                }
                largest_files.push((path.display().to_string(), bytes));
            });
        }
        visit_files(benchmark_root, &mut |path, bytes| {
            let name = path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or_default();
            if name != "dataset.csv" && name != "predictions.csv" {
                report_bytes = report_bytes.saturating_add(bytes);
            }
            largest_files.push((path.display().to_string(), bytes));
        });
        largest_files
            .sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));

        let mut reason_codes = vec![ReasonCode::BenchmarkStorageAuditBuilt];
        if collection_report.storage_budget_report.budget_exceeded {
            reason_codes.push(ReasonCode::CollectionBudgetExceeded);
        }

        Self {
            collection_bytes: collection_report.storage_budget_report.total_bytes,
            dataset_export_bytes,
            prediction_bytes,
            report_bytes,
            raw_archive_bytes: collection_report.storage_budget_report.raw_bytes,
            canonical_bytes: collection_report.storage_budget_report.canonical_bytes,
            budget_exceeded: collection_report.storage_budget_report.budget_exceeded,
            largest_files: largest_files
                .into_iter()
                .take(10)
                .map(|(path, bytes)| format!("{path}:{bytes}"))
                .collect(),
            retention_actions: collection_report
                .storage_budget_report
                .retention_actions
                .clone(),
            reason_codes: dedupe_reasons(reason_codes),
        }
    }

    pub fn to_text(&self) -> String {
        [
            format!("collection_bytes={}", self.collection_bytes),
            format!("dataset_export_bytes={}", self.dataset_export_bytes),
            format!("prediction_bytes={}", self.prediction_bytes),
            format!("report_bytes={}", self.report_bytes),
            format!("raw_archive_bytes={}", self.raw_archive_bytes),
            format!("canonical_bytes={}", self.canonical_bytes),
            format!("budget_exceeded={}", self.budget_exceeded),
            format!("largest_files={}", self.largest_files.join(" | ")),
            format!("retention_actions={}", self.retention_actions.join(" | ")),
            format!(
                "reason_codes={}",
                self.reason_codes
                    .iter()
                    .map(|reason| format!("{reason:?}"))
                    .collect::<Vec<_>>()
                    .join("|")
            ),
        ]
        .join("\n")
    }
}

fn visit_files(path: &Path, visitor: &mut dyn FnMut(&Path, usize)) {
    if !path.exists() {
        return;
    }
    if path.is_file() {
        let bytes = path.metadata().map(|meta| meta.len() as usize).unwrap_or(0);
        visitor(path, bytes);
        return;
    }
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let child = entry.path();
            if child.is_dir() {
                visit_files(&child, visitor);
            } else if let Ok(metadata) = child.metadata() {
                visitor(&child, metadata.len() as usize);
            }
        }
    }
}

fn dedupe_reasons(values: Vec<ReasonCode>) -> Vec<ReasonCode> {
    let mut deduped = Vec::new();
    for value in values {
        if !deduped.contains(&value) {
            deduped.push(value);
        }
    }
    deduped
}
