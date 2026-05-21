use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};

use super::krx_canonical_validation::{KRXCanonicalValidationReport, KRXCanonicalValidationStatus};
use super::krx_collection_smoke::infer_symbol_from_path;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum KRXCanonicalBatchValidationStatus {
    BatchValid,
    BatchPartiallyValid,
    BatchInvalid,
    MissingProvenance,
    MissingPreflight,
    DataQualityTooLow,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct KRXCanonicalBatchValidationReport {
    pub batch_id: String,
    pub validation_reports: Vec<KRXCanonicalValidationReport>,
    pub valid_csv_count: usize,
    pub invalid_csv_count: usize,
    pub total_rows: usize,
    pub official_readiness_eligible_csv_count: usize,
    pub missing_provenance_count: usize,
    pub missing_preflight_count: usize,
    pub data_quality_too_low_count: usize,
    pub duplicate_timestamp_count: usize,
    pub gap_heavy_count: usize,
    pub validation_status: KRXCanonicalBatchValidationStatus,
    pub reason_codes: Vec<ReasonCode>,
}

impl KRXCanonicalBatchValidationReport {
    pub fn build(
        batch_id: &str,
        canonical_csv_paths: &[String],
        require_provenance: bool,
        require_preflight: bool,
    ) -> Self {
        let mut validation_reports = canonical_csv_paths
            .iter()
            .map(|path| validate_path(path, require_provenance, require_preflight))
            .collect::<Vec<_>>();
        validation_reports
            .sort_by(|left, right| left.canonical_csv_path.cmp(&right.canonical_csv_path));
        Self::from_validation_reports(batch_id, validation_reports)
    }

    pub fn from_validation_reports(
        batch_id: &str,
        validation_reports: Vec<KRXCanonicalValidationReport>,
    ) -> Self {
        let valid_csv_count = validation_reports
            .iter()
            .filter(|report| {
                matches!(
                    report.validation_status,
                    KRXCanonicalValidationStatus::Valid
                )
            })
            .count();
        let invalid_csv_count = validation_reports.len().saturating_sub(valid_csv_count);
        let total_rows = validation_reports
            .iter()
            .map(|report| report.row_count)
            .sum();
        let official_readiness_eligible_csv_count = validation_reports
            .iter()
            .filter(|report| report.official_readiness_eligible)
            .count();
        let missing_provenance_count = validation_reports
            .iter()
            .filter(|report| !report.provenance_available)
            .count();
        let missing_preflight_count = validation_reports
            .iter()
            .filter(|report| !report.preflight_available)
            .count();
        let data_quality_too_low_count = validation_reports
            .iter()
            .filter(|report| {
                matches!(
                    report.validation_status,
                    KRXCanonicalValidationStatus::DataQualityTooLow
                        | KRXCanonicalValidationStatus::BadPrice
                        | KRXCanonicalValidationStatus::BadVolume
                        | KRXCanonicalValidationStatus::OhlcInvariantFailed
                )
            })
            .count();
        let duplicate_timestamp_count = validation_reports
            .iter()
            .map(|report| report.duplicates_count)
            .sum();
        let gap_heavy_count = validation_reports
            .iter()
            .filter(|report| {
                matches!(
                    report.validation_status,
                    KRXCanonicalValidationStatus::GapHeavy
                )
            })
            .count();
        let validation_status = if validation_reports.is_empty() {
            KRXCanonicalBatchValidationStatus::DiagnosticOnly
        } else if missing_provenance_count > 0 && valid_csv_count == 0 {
            KRXCanonicalBatchValidationStatus::MissingProvenance
        } else if missing_preflight_count > 0 && valid_csv_count == 0 {
            KRXCanonicalBatchValidationStatus::MissingPreflight
        } else if data_quality_too_low_count > 0 && valid_csv_count == 0 {
            KRXCanonicalBatchValidationStatus::DataQualityTooLow
        } else if valid_csv_count == validation_reports.len() {
            KRXCanonicalBatchValidationStatus::BatchValid
        } else if valid_csv_count > 0 {
            KRXCanonicalBatchValidationStatus::BatchPartiallyValid
        } else {
            KRXCanonicalBatchValidationStatus::BatchInvalid
        };
        let mut reason_codes = vec![ReasonCode::KRXCanonicalValidationBuilt];
        if missing_provenance_count > 0 {
            reason_codes.push(ReasonCode::MissingOfficialProvenance);
        }
        if missing_preflight_count > 0 {
            reason_codes.push(ReasonCode::MissingOfficialPreflight);
        }
        if data_quality_too_low_count > 0 {
            reason_codes.push(ReasonCode::DataQualityTooLow);
        }
        if duplicate_timestamp_count > 0 {
            reason_codes.push(ReasonCode::DuplicateTimestampDetected);
        }
        if gap_heavy_count > 0 {
            reason_codes.push(ReasonCode::GapDetected);
        }
        Self {
            batch_id: batch_id.to_string(),
            validation_reports,
            valid_csv_count,
            invalid_csv_count,
            total_rows,
            official_readiness_eligible_csv_count,
            missing_provenance_count,
            missing_preflight_count,
            data_quality_too_low_count,
            duplicate_timestamp_count,
            gap_heavy_count,
            validation_status,
            reason_codes: stable_reason_codes(&reason_codes),
        }
    }

    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("batch_id={}", self.batch_id),
            format!("valid_csv_count={}", self.valid_csv_count),
            format!("invalid_csv_count={}", self.invalid_csv_count),
            format!("total_rows={}", self.total_rows),
            format!(
                "official_readiness_eligible_csv_count={}",
                self.official_readiness_eligible_csv_count
            ),
            format!("missing_provenance_count={}", self.missing_provenance_count),
            format!("missing_preflight_count={}", self.missing_preflight_count),
            format!(
                "data_quality_too_low_count={}",
                self.data_quality_too_low_count
            ),
            format!(
                "duplicate_timestamp_count={}",
                self.duplicate_timestamp_count
            ),
            format!("gap_heavy_count={}", self.gap_heavy_count),
            format!("validation_status={:?}", self.validation_status),
            format!(
                "reason_codes={}",
                self.reason_codes
                    .iter()
                    .map(|reason| format!("{reason:?}"))
                    .collect::<Vec<_>>()
                    .join("|")
            ),
        ];
        lines.extend(
            self.validation_reports
                .iter()
                .map(KRXCanonicalValidationReport::to_text),
        );
        lines.join("\n")
    }

    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        let text_path = output_dir.join("krx_canonical_batch_validation.txt");
        fs::write(&text_path, self.to_text()).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("krx_canonical_batch_validation.json"),
            self.to_json_string()?,
        )
        .map_err(|err| err.to_string())?;
        Ok(text_path)
    }
}

pub(crate) fn validate_path(
    canonical_csv_path: &str,
    require_provenance: bool,
    require_preflight: bool,
) -> KRXCanonicalValidationReport {
    let symbol = infer_symbol_from_path(canonical_csv_path);
    KRXCanonicalValidationReport::validate(
        canonical_csv_path,
        symbol.as_ref().map(|entry| entry.provider_symbol.clone()),
        symbol.as_ref().map(|entry| entry.normalized_symbol.clone()),
        infer_sidecar_path(canonical_csv_path, "_provenance.json").as_deref(),
        infer_sidecar_path(canonical_csv_path, "_preflight.json").as_deref(),
        require_provenance,
        require_preflight,
    )
}

pub(crate) fn infer_sidecar_path(canonical_csv_path: &str, suffix: &str) -> Option<String> {
    let path = Path::new(canonical_csv_path);
    let stem = path.file_stem()?.to_string_lossy();
    let candidate = path.with_file_name(format!("{stem}{suffix}"));
    if candidate.exists() {
        Some(candidate.display().to_string())
    } else {
        None
    }
}
