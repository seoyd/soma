use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;
use crate::data::format_detect::CsvFormatDetectionConfidence;
use crate::eval::WalkForwardSplit;

use super::{
    CandleCsvLoader, CandleLoadFailure, CsvFormatDetectionResult, CsvFormatDetector, DataManifest,
    DataQualityReport, DataQualitySeverity, DataValidationConfig, EvidenceTargetEstimate,
    LocalDataOnboardingConfig, TimeframeSpec, estimate_evidence_targets,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PreflightCheck {
    SourcePathLocal,
    FileExists,
    FormatDetected,
    RequiredColumnsPresent,
    ParseRows,
    CandleInvariants,
    TimestampOrdering,
    DuplicateTimestamps,
    GapAnalysis,
    DataQuality,
    SufficientRows,
    SufficientTimeCoverage,
    WalkForwardFeasible,
    TripleBarrierFeasible,
    OutcomeTargetFeasible,
    ComparableVariantFeasible,
    RealLocalEligible,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PreflightCheckStatus {
    Passed,
    Warning,
    Failed,
    Skipped,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PreflightCheckResult {
    pub check: PreflightCheck,
    pub status: PreflightCheckStatus,
    pub summary: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PreflightFinalStatus {
    ReadyForRealEvidence,
    NeedsColumnMapping,
    NeedsMoreRows,
    DataQualityTooLow,
    MissingFile,
    UnsupportedFormat,
    AmbiguousFormat,
    NotRealLocalEligible,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PreflightReport {
    pub onboarding_id: String,
    pub input_path: String,
    #[serde(default)]
    pub detected_format: Option<super::CandleCsvFormat>,
    pub symbol: String,
    pub timeframe: crate::backtest::Timeframe,
    pub provenance: super::DataProvenance,
    #[serde(default)]
    pub data_quality_report: Option<DataQualityReport>,
    #[serde(default)]
    pub data_manifest_preview: Option<DataManifest>,
    #[serde(default)]
    pub evidence_target_estimate: Option<EvidenceTargetEstimate>,
    pub row_count: usize,
    pub usable_row_count: usize,
    pub estimated_walk_forward_folds: usize,
    pub estimated_outcome_records: usize,
    pub estimated_comparable_variants: usize,
    pub checks: Vec<PreflightCheckResult>,
    pub final_status: PreflightFinalStatus,
    pub blockers: Vec<String>,
    pub warnings: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

impl PreflightReport {
    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn to_text(&self) -> String {
        let detected_format = self
            .detected_format
            .as_ref()
            .map(|format| format!("{format:?}"))
            .unwrap_or_else(|| "None".to_string());
        let checks = self
            .checks
            .iter()
            .map(|check| format!("{:?}={:?} ({})", check.check, check.status, check.summary))
            .collect::<Vec<_>>()
            .join("\n");
        [
            format!("onboarding_id={}", self.onboarding_id),
            format!("input_path={}", self.input_path),
            format!("detected_format={detected_format}"),
            format!("symbol={}", self.symbol),
            format!("timeframe={:?}", self.timeframe),
            format!("final_status={:?}", self.final_status),
            format!("row_count={}", self.row_count),
            format!("usable_row_count={}", self.usable_row_count),
            format!(
                "estimated_walk_forward_folds={}",
                self.estimated_walk_forward_folds
            ),
            format!(
                "estimated_outcome_records={}",
                self.estimated_outcome_records
            ),
            format!(
                "estimated_comparable_variants={}",
                self.estimated_comparable_variants
            ),
            format!("blockers={}", self.blockers.join(" | ")),
            format!("warnings={}", self.warnings.join(" | ")),
            "checks:".to_string(),
            checks,
        ]
        .join("\n")
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<(), String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("preflight_report.json"),
            self.to_json_string()?,
        )
        .map_err(|err| err.to_string())?;
        fs::write(output_dir.join("preflight_report.txt"), self.to_text())
            .map_err(|err| err.to_string())?;
        Ok(())
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PreflightValidator {
    pub detector: CsvFormatDetector,
}

impl PreflightValidator {
    pub fn run(&self, config: &LocalDataOnboardingConfig) -> PreflightReport {
        let symbol = config.resolved_symbol();
        let timeframe = config.resolved_timeframe();
        let provenance = config.build_provenance();
        let mut checks = Vec::new();
        let mut blockers = Vec::new();
        let mut warnings = Vec::new();
        let mut reason_codes = config.validate_local_paths();

        let path_local = !config.input_path.contains("://");
        checks.push(check_result(
            PreflightCheck::SourcePathLocal,
            if path_local {
                PreflightCheckStatus::Passed
            } else {
                PreflightCheckStatus::Failed
            },
            if path_local {
                "input path is local".to_string()
            } else {
                "remote URL-like input path is rejected".to_string()
            },
            if path_local {
                vec![ReasonCode::LocalFileOnly]
            } else {
                vec![ReasonCode::LocalPathRejected]
            },
        ));
        if !path_local {
            blockers.push("input_path must be local".to_string());
            reason_codes.push(ReasonCode::LocalPathRejected);
            return finalize_report(
                config,
                symbol,
                timeframe,
                provenance,
                None,
                None,
                None,
                checks,
                PreflightFinalStatus::NotRealLocalEligible,
                blockers,
                warnings,
                reason_codes,
            );
        }

        let path = Path::new(&config.input_path);
        let file_exists = path.exists();
        checks.push(check_result(
            PreflightCheck::FileExists,
            if file_exists {
                PreflightCheckStatus::Passed
            } else {
                PreflightCheckStatus::Failed
            },
            if file_exists {
                "input file exists".to_string()
            } else {
                "input file is missing".to_string()
            },
            if file_exists {
                vec![ReasonCode::LocalFileOnly]
            } else {
                vec![ReasonCode::MissingFile]
            },
        ));
        if !file_exists {
            blockers.push(format!("missing input file: {}", config.input_path));
            reason_codes.push(ReasonCode::MissingFile);
            return finalize_report(
                config,
                symbol,
                timeframe,
                provenance,
                None,
                None,
                None,
                checks,
                PreflightFinalStatus::MissingFile,
                blockers,
                warnings,
                reason_codes,
            );
        }

        let detection = resolve_detection(&self.detector, config, path);
        checks.extend(detection_checks(config, &detection));
        reason_codes.extend(detection.reason_codes.iter().cloned());
        if matches!(
            detection.confidence,
            CsvFormatDetectionConfidence::Unsupported
        ) {
            blockers.push("csv format is unsupported".to_string());
            return finalize_report(
                config,
                symbol,
                timeframe,
                provenance,
                None,
                None,
                Some(detection),
                checks,
                PreflightFinalStatus::UnsupportedFormat,
                blockers,
                warnings,
                reason_codes,
            );
        }
        if matches!(
            detection.confidence,
            CsvFormatDetectionConfidence::Ambiguous
        ) || (config.strict && matches!(detection.confidence, CsvFormatDetectionConfidence::Low))
        {
            blockers.push("csv format is ambiguous and needs an explicit mapping".to_string());
            return finalize_report(
                config,
                symbol,
                timeframe,
                provenance,
                None,
                None,
                Some(detection),
                checks,
                PreflightFinalStatus::AmbiguousFormat,
                blockers,
                warnings,
                reason_codes,
            );
        }
        let Some(format) = detection.detected_format.clone() else {
            blockers.push("csv needs an explicit format hint or custom column map".to_string());
            return finalize_report(
                config,
                symbol,
                timeframe,
                provenance,
                None,
                None,
                Some(detection),
                checks,
                PreflightFinalStatus::NeedsColumnMapping,
                blockers,
                warnings,
                reason_codes,
            );
        };

        let csv_config = config.build_csv_config(format, detection.header_present);
        let loader = CandleCsvLoader {
            validation: DataValidationConfig {
                strict: config.strict,
                allow_sort_repair: config.allow_sort_repair,
                allow_duplicate_drop: config.allow_duplicate_drop,
                allow_gap: true,
                max_gap_count: usize::MAX,
                max_gap_ratio: 1.0,
                max_invalid_ratio: 1.0,
                expected_step_ms: Some(TimeframeSpec::from_timeframe(timeframe).expected_ms_step),
                reason_codes: vec![ReasonCode::DeterministicPath],
            },
            ..CandleCsvLoader::default()
        };

        match loader.load_from_path(path, &csv_config) {
            Ok(loaded) => {
                let estimate = estimate_evidence_targets(config, &loaded);
                let quality = &loaded.quality_report;
                let triple_feasible =
                    loaded.series.len() > config.resolved_triple_barrier_config().horizon_bars;
                let coverage_ok = loaded
                    .manifest
                    .last_timestamp_ms
                    .saturating_sub(loaded.manifest.first_timestamp_ms)
                    >= TimeframeSpec::from_timeframe(timeframe)
                        .expected_ms_step
                        .saturating_mul(config.min_rows_for_preflight.saturating_sub(1) as u64);
                let walk_forward_split = WalkForwardSplit::generate(
                    &loaded.series,
                    config.resolved_walk_forward_config(),
                );
                let source_kind = config
                    .source_kind
                    .unwrap_or(super::EvidenceSourceKind::RealLocal);
                let real_local_eligible = source_kind.readiness_eligible()
                    && (config.user_supplied
                        || source_kind == super::EvidenceSourceKind::OfficialApiCollected);
                checks.extend(success_checks(
                    config,
                    quality,
                    loaded.series.len(),
                    coverage_ok,
                    walk_forward_split.folds.len(),
                    triple_feasible,
                    real_local_eligible,
                    &estimate,
                ));
                let final_status = if !real_local_eligible {
                    blockers
                        .push("user_supplied must be true for real-local readiness".to_string());
                    reason_codes.push(ReasonCode::PreflightNotRealLocalEligible);
                    PreflightFinalStatus::NotRealLocalEligible
                } else if matches!(
                    quality.severity,
                    DataQualitySeverity::Bad | DataQualitySeverity::Unusable
                ) {
                    blockers.push("data quality is too low for real evidence".to_string());
                    PreflightFinalStatus::DataQualityTooLow
                } else if loaded.series.len() < config.min_rows_for_preflight
                    || !coverage_ok
                    || walk_forward_split.folds.is_empty()
                    || !triple_feasible
                    || !estimate.enough_for_minimum_real_evidence
                {
                    blockers.push("dataset needs more usable rows or better coverage".to_string());
                    PreflightFinalStatus::NeedsMoreRows
                } else {
                    reason_codes.push(ReasonCode::PreflightReadyForRealEvidence);
                    PreflightFinalStatus::ReadyForRealEvidence
                };
                if quality.gap_count > 0 {
                    warnings.push(format!("detected {} temporal gaps", quality.gap_count));
                }
                if quality.repaired_row_count > 0 {
                    warnings.push(format!(
                        "repaired {} rows through sort/duplicate repair",
                        quality.repaired_row_count
                    ));
                }
                finalize_report(
                    config,
                    symbol,
                    timeframe,
                    provenance,
                    Some(loaded.quality_report),
                    Some(loaded.manifest),
                    Some(detection),
                    checks,
                    final_status,
                    blockers,
                    warnings,
                    [
                        reason_codes,
                        loaded.reason_codes,
                        estimate.reason_codes.clone(),
                    ]
                    .concat(),
                )
                .with_estimate(estimate)
            }
            Err(failure) => {
                checks.extend(failure_checks(&failure));
                let final_status = classify_failure_status(&failure);
                blockers.extend(failure_blockers(&failure));
                reason_codes.extend(failure.reason_codes.iter().cloned());
                finalize_report(
                    config,
                    symbol,
                    timeframe,
                    provenance,
                    failure.quality_report,
                    None,
                    Some(detection),
                    checks,
                    final_status,
                    blockers,
                    warnings,
                    reason_codes,
                )
            }
        }
    }
}

impl PreflightReport {
    fn with_estimate(mut self, estimate: EvidenceTargetEstimate) -> Self {
        self.estimated_walk_forward_folds = estimate.estimated_walk_forward_folds;
        self.estimated_outcome_records = estimate.estimated_outcome_records;
        self.estimated_comparable_variants = estimate.estimated_comparable_variants;
        self.evidence_target_estimate = Some(estimate);
        self
    }
}

fn resolve_detection(
    detector: &CsvFormatDetector,
    config: &LocalDataOnboardingConfig,
    path: &Path,
) -> CsvFormatDetectionResult {
    if let Some(column_map) = &config.custom_column_map {
        return detector
            .detect_from_path(path, Some(column_map))
            .unwrap_or_else(|_| unsupported_detection());
    }
    if let Some(format) = &config.csv_format_hint {
        return detector
            .validate_format_from_path(path, format.clone())
            .unwrap_or_else(|_| unsupported_detection());
    }
    if config.allow_format_autodetect {
        detector
            .detect_from_path(path, None)
            .unwrap_or_else(|_| unsupported_detection())
    } else {
        CsvFormatDetectionResult {
            detected_format: None,
            confidence: CsvFormatDetectionConfidence::Low,
            header_present: true,
            detected_columns: Vec::new(),
            missing_required_columns: Vec::new(),
            candidate_mappings: Vec::new(),
            reason_codes: vec![ReasonCode::PreflightNeedsColumnMapping],
        }
    }
}

fn unsupported_detection() -> CsvFormatDetectionResult {
    CsvFormatDetectionResult {
        detected_format: None,
        confidence: CsvFormatDetectionConfidence::Unsupported,
        header_present: true,
        detected_columns: Vec::new(),
        missing_required_columns: Vec::new(),
        candidate_mappings: Vec::new(),
        reason_codes: vec![ReasonCode::UnsupportedCsvFormat],
    }
}

fn detection_checks(
    config: &LocalDataOnboardingConfig,
    detection: &CsvFormatDetectionResult,
) -> Vec<PreflightCheckResult> {
    let format_status = match detection.confidence {
        CsvFormatDetectionConfidence::High | CsvFormatDetectionConfidence::Medium => {
            PreflightCheckStatus::Passed
        }
        CsvFormatDetectionConfidence::Low => {
            if config.strict {
                PreflightCheckStatus::Failed
            } else {
                PreflightCheckStatus::Warning
            }
        }
        CsvFormatDetectionConfidence::Ambiguous | CsvFormatDetectionConfidence::Unsupported => {
            PreflightCheckStatus::Failed
        }
    };
    let required_columns_status = if matches!(
        detection.confidence,
        CsvFormatDetectionConfidence::Unsupported
    ) && detection.detected_format.is_none()
    {
        PreflightCheckStatus::Skipped
    } else if detection.missing_required_columns.is_empty() {
        PreflightCheckStatus::Passed
    } else {
        PreflightCheckStatus::Failed
    };
    vec![
        check_result(
            PreflightCheck::FormatDetected,
            format_status,
            format!("format detection confidence: {:?}", detection.confidence),
            detection.reason_codes.clone(),
        ),
        check_result(
            PreflightCheck::RequiredColumnsPresent,
            required_columns_status,
            if matches!(required_columns_status, PreflightCheckStatus::Skipped) {
                "required-column validation skipped because no supported format was selected"
                    .to_string()
            } else if detection.missing_required_columns.is_empty() {
                "required columns are present".to_string()
            } else {
                format!(
                    "missing required columns: {}",
                    detection.missing_required_columns.join(", ")
                )
            },
            if matches!(required_columns_status, PreflightCheckStatus::Skipped) {
                vec![ReasonCode::UnsupportedCsvFormat]
            } else if detection.missing_required_columns.is_empty() {
                vec![ReasonCode::CsvFormatDetected]
            } else {
                vec![ReasonCode::MissingRequiredColumn]
            },
        ),
    ]
}

fn success_checks(
    config: &LocalDataOnboardingConfig,
    quality: &DataQualityReport,
    usable_row_count: usize,
    coverage_ok: bool,
    folds: usize,
    triple_feasible: bool,
    real_local_eligible: bool,
    estimate: &EvidenceTargetEstimate,
) -> Vec<PreflightCheckResult> {
    vec![
        check_result(
            PreflightCheck::ParseRows,
            if quality.invalid_row_count == 0 {
                PreflightCheckStatus::Passed
            } else {
                PreflightCheckStatus::Warning
            },
            format!(
                "valid rows: {}, invalid rows: {}",
                quality.valid_row_count, quality.invalid_row_count
            ),
            quality.reason_codes.clone(),
        ),
        check_result(
            PreflightCheck::CandleInvariants,
            if quality.ohlc_invariant_violation_count == 0 {
                PreflightCheckStatus::Passed
            } else {
                PreflightCheckStatus::Failed
            },
            format!(
                "OHLC invariant violations: {}",
                quality.ohlc_invariant_violation_count
            ),
            quality.reason_codes.clone(),
        ),
        check_result(
            PreflightCheck::TimestampOrdering,
            if quality.out_of_order_count == 0 {
                PreflightCheckStatus::Passed
            } else if config.allow_sort_repair {
                PreflightCheckStatus::Warning
            } else {
                PreflightCheckStatus::Failed
            },
            format!("out-of-order rows: {}", quality.out_of_order_count),
            quality.reason_codes.clone(),
        ),
        check_result(
            PreflightCheck::DuplicateTimestamps,
            if quality.duplicate_timestamp_count == 0 {
                PreflightCheckStatus::Passed
            } else if config.allow_duplicate_drop {
                PreflightCheckStatus::Warning
            } else {
                PreflightCheckStatus::Failed
            },
            format!(
                "duplicate timestamps: {}",
                quality.duplicate_timestamp_count
            ),
            quality.reason_codes.clone(),
        ),
        check_result(
            PreflightCheck::GapAnalysis,
            if quality.gap_count == 0 {
                PreflightCheckStatus::Passed
            } else {
                PreflightCheckStatus::Warning
            },
            format!("detected gaps: {}", quality.gap_count),
            quality.reason_codes.clone(),
        ),
        check_result(
            PreflightCheck::DataQuality,
            match quality.severity {
                DataQualitySeverity::Good => PreflightCheckStatus::Passed,
                DataQualitySeverity::Warning => PreflightCheckStatus::Warning,
                DataQualitySeverity::Bad | DataQualitySeverity::Unusable => {
                    PreflightCheckStatus::Failed
                }
            },
            format!(
                "data quality {:?} ({:.4})",
                quality.severity, quality.data_quality_score
            ),
            quality.reason_codes.clone(),
        ),
        check_result(
            PreflightCheck::SufficientRows,
            if usable_row_count >= config.min_rows_for_preflight {
                PreflightCheckStatus::Passed
            } else {
                PreflightCheckStatus::Failed
            },
            format!(
                "usable rows: {} / required minimum: {}",
                usable_row_count, config.min_rows_for_preflight
            ),
            if usable_row_count >= config.min_rows_for_preflight {
                vec![ReasonCode::CsvLoaded]
            } else {
                vec![ReasonCode::PreflightNeedsMoreRows]
            },
        ),
        check_result(
            PreflightCheck::SufficientTimeCoverage,
            if coverage_ok {
                PreflightCheckStatus::Passed
            } else {
                PreflightCheckStatus::Failed
            },
            if coverage_ok {
                "time coverage is sufficient".to_string()
            } else {
                "time coverage is insufficient".to_string()
            },
            if coverage_ok {
                vec![ReasonCode::CsvLoaded]
            } else {
                vec![ReasonCode::PreflightNeedsMoreRows]
            },
        ),
        check_result(
            PreflightCheck::WalkForwardFeasible,
            if folds > 0 {
                PreflightCheckStatus::Passed
            } else {
                PreflightCheckStatus::Failed
            },
            format!("estimated walk-forward folds: {folds}"),
            if folds > 0 {
                vec![ReasonCode::WalkForwardFoldGenerated]
            } else {
                vec![ReasonCode::WalkForwardInsufficientData]
            },
        ),
        check_result(
            PreflightCheck::TripleBarrierFeasible,
            if triple_feasible {
                PreflightCheckStatus::Passed
            } else {
                PreflightCheckStatus::Failed
            },
            if triple_feasible {
                "triple-barrier horizon is feasible".to_string()
            } else {
                "triple-barrier horizon is not feasible".to_string()
            },
            if triple_feasible {
                vec![ReasonCode::CsvLoaded]
            } else {
                vec![ReasonCode::PreflightNeedsMoreRows]
            },
        ),
        check_result(
            PreflightCheck::OutcomeTargetFeasible,
            if estimate.estimated_outcome_records >= config.target_min_outcomes {
                PreflightCheckStatus::Passed
            } else {
                PreflightCheckStatus::Failed
            },
            format!(
                "estimated outcome records: {} / target: {}",
                estimate.estimated_outcome_records, config.target_min_outcomes
            ),
            estimate.reason_codes.clone(),
        ),
        check_result(
            PreflightCheck::ComparableVariantFeasible,
            if estimate.estimated_comparable_variants >= config.target_min_comparable_variants {
                PreflightCheckStatus::Passed
            } else {
                PreflightCheckStatus::Failed
            },
            format!(
                "estimated comparable variants: {} / target: {}",
                estimate.estimated_comparable_variants, config.target_min_comparable_variants
            ),
            estimate.reason_codes.clone(),
        ),
        check_result(
            PreflightCheck::RealLocalEligible,
            if real_local_eligible {
                PreflightCheckStatus::Passed
            } else {
                PreflightCheckStatus::Failed
            },
            format!(
                "real-local readiness eligibility: {}",
                if real_local_eligible {
                    "eligible"
                } else {
                    "blocked"
                }
            ),
            if real_local_eligible {
                vec![ReasonCode::PreflightReadyForRealEvidence]
            } else {
                vec![ReasonCode::PreflightNotRealLocalEligible]
            },
        ),
    ]
}

fn failure_checks(failure: &CandleLoadFailure) -> Vec<PreflightCheckResult> {
    vec![
        check_result(
            PreflightCheck::ParseRows,
            PreflightCheckStatus::Failed,
            "csv parsing failed".to_string(),
            failure.reason_codes.clone(),
        ),
        check_result(
            PreflightCheck::CandleInvariants,
            if failure
                .reason_codes
                .contains(&ReasonCode::OhlcInvariantViolationDetected)
            {
                PreflightCheckStatus::Failed
            } else {
                PreflightCheckStatus::Skipped
            },
            "candle invariant validation did not pass".to_string(),
            failure.reason_codes.clone(),
        ),
        check_result(
            PreflightCheck::TimestampOrdering,
            if failure
                .reason_codes
                .contains(&ReasonCode::OutOfOrderTimestampDetected)
            {
                PreflightCheckStatus::Failed
            } else {
                PreflightCheckStatus::Skipped
            },
            "timestamp ordering failed or was not reached".to_string(),
            failure.reason_codes.clone(),
        ),
        check_result(
            PreflightCheck::DuplicateTimestamps,
            if failure
                .reason_codes
                .contains(&ReasonCode::DuplicateTimestampDetected)
            {
                PreflightCheckStatus::Failed
            } else {
                PreflightCheckStatus::Skipped
            },
            "duplicate timestamp validation failed or was not reached".to_string(),
            failure.reason_codes.clone(),
        ),
        check_result(
            PreflightCheck::GapAnalysis,
            if failure.reason_codes.contains(&ReasonCode::GapDetected) {
                PreflightCheckStatus::Warning
            } else {
                PreflightCheckStatus::Skipped
            },
            "gap analysis incomplete".to_string(),
            failure.reason_codes.clone(),
        ),
        check_result(
            PreflightCheck::DataQuality,
            PreflightCheckStatus::Failed,
            "data quality could not reach a ready state".to_string(),
            failure.reason_codes.clone(),
        ),
        check_result(
            PreflightCheck::SufficientRows,
            PreflightCheckStatus::Skipped,
            "row sufficiency not available".to_string(),
            failure.reason_codes.clone(),
        ),
        check_result(
            PreflightCheck::SufficientTimeCoverage,
            PreflightCheckStatus::Skipped,
            "time coverage not available".to_string(),
            failure.reason_codes.clone(),
        ),
        check_result(
            PreflightCheck::WalkForwardFeasible,
            PreflightCheckStatus::Skipped,
            "walk-forward feasibility not available".to_string(),
            failure.reason_codes.clone(),
        ),
        check_result(
            PreflightCheck::TripleBarrierFeasible,
            PreflightCheckStatus::Skipped,
            "triple-barrier feasibility not available".to_string(),
            failure.reason_codes.clone(),
        ),
        check_result(
            PreflightCheck::OutcomeTargetFeasible,
            PreflightCheckStatus::Skipped,
            "outcome estimate unavailable".to_string(),
            failure.reason_codes.clone(),
        ),
        check_result(
            PreflightCheck::ComparableVariantFeasible,
            PreflightCheckStatus::Skipped,
            "variant estimate unavailable".to_string(),
            failure.reason_codes.clone(),
        ),
        check_result(
            PreflightCheck::RealLocalEligible,
            PreflightCheckStatus::Skipped,
            "real-local eligibility not reached".to_string(),
            failure.reason_codes.clone(),
        ),
    ]
}

fn classify_failure_status(failure: &CandleLoadFailure) -> PreflightFinalStatus {
    if failure
        .reason_codes
        .contains(&ReasonCode::UnsupportedCsvFormat)
        || failure
            .reason_codes
            .contains(&ReasonCode::UnsupportedTimestampFormat)
    {
        PreflightFinalStatus::UnsupportedFormat
    } else if failure
        .reason_codes
        .contains(&ReasonCode::MissingRequiredColumn)
    {
        PreflightFinalStatus::NeedsColumnMapping
    } else {
        PreflightFinalStatus::DataQualityTooLow
    }
}

fn failure_blockers(failure: &CandleLoadFailure) -> Vec<String> {
    if failure
        .reason_codes
        .contains(&ReasonCode::MissingRequiredColumn)
    {
        vec!["required csv columns are missing".to_string()]
    } else if failure
        .reason_codes
        .contains(&ReasonCode::DuplicateTimestampDetected)
    {
        vec!["duplicate timestamps must be repaired or dropped".to_string()]
    } else if failure
        .reason_codes
        .contains(&ReasonCode::OutOfOrderTimestampDetected)
    {
        vec!["timestamps are out of order".to_string()]
    } else if failure
        .reason_codes
        .contains(&ReasonCode::OhlcInvariantViolationDetected)
    {
        vec!["OHLC invariants are violated".to_string()]
    } else {
        vec!["csv preflight failed before readiness checks".to_string()]
    }
}

fn finalize_report(
    config: &LocalDataOnboardingConfig,
    symbol: String,
    timeframe: crate::backtest::Timeframe,
    provenance: super::DataProvenance,
    data_quality_report: Option<DataQualityReport>,
    data_manifest_preview: Option<DataManifest>,
    detection: Option<CsvFormatDetectionResult>,
    checks: Vec<PreflightCheckResult>,
    final_status: PreflightFinalStatus,
    blockers: Vec<String>,
    warnings: Vec<String>,
    reason_codes: Vec<ReasonCode>,
) -> PreflightReport {
    let row_count = data_manifest_preview
        .as_ref()
        .map(|manifest| manifest.row_count)
        .or_else(|| {
            data_quality_report
                .as_ref()
                .map(|quality| quality.row_count)
        })
        .unwrap_or(0);
    let usable_row_count = data_quality_report
        .as_ref()
        .map(|quality| quality.valid_row_count)
        .unwrap_or(0);
    PreflightReport {
        onboarding_id: config.onboarding_id.clone(),
        input_path: config.input_path.clone(),
        detected_format: detection.and_then(|detected| detected.detected_format),
        symbol,
        timeframe,
        provenance,
        data_quality_report,
        data_manifest_preview,
        evidence_target_estimate: None,
        row_count,
        usable_row_count,
        estimated_walk_forward_folds: 0,
        estimated_outcome_records: 0,
        estimated_comparable_variants: 0,
        checks,
        final_status,
        blockers,
        warnings,
        reason_codes: dedupe_reasons({
            let mut reasons = reason_codes;
            reasons.push(ReasonCode::PreflightReportBuilt);
            reasons
        }),
    }
}

fn check_result(
    check: PreflightCheck,
    status: PreflightCheckStatus,
    summary: String,
    reason_codes: Vec<ReasonCode>,
) -> PreflightCheckResult {
    PreflightCheckResult {
        check,
        status,
        summary,
        reason_codes: dedupe_reasons(reason_codes),
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
