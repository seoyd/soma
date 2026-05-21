use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};
use crate::league::load_barrier_profile_registry_from_path_or_config;

use super::krx_canonical_batch_validation::{KRXCanonicalBatchValidationReport, validate_path};
use super::krx_canonical_validation::{KRXCanonicalValidationReport, KRXCanonicalValidationStatus};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum KRXCandleSufficiencyStatus {
    HealthyKRXCandles,
    MissingOfficialCandles,
    MissingFutureWindows,
    TimestampAlignmentWeak,
    TimeframeMismatch,
    SymbolMismatch,
    MissingPreflight,
    MissingProvenance,
    DataQualityTooLow,
    InsufficientRows,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KRXCandleSufficiencyItem {
    pub provider_symbol: String,
    pub normalized_symbol: String,
    pub timeframe: String,
    pub row_count: usize,
    #[serde(default)]
    pub timestamp_start_ms: Option<u64>,
    #[serde(default)]
    pub timestamp_end_ms: Option<u64>,
    #[serde(default)]
    pub required_future_bars: Option<usize>,
    #[serde(default)]
    pub available_future_bars: Option<usize>,
    #[serde(default)]
    pub missing_future_bars: Option<usize>,
    pub official_ready: bool,
    pub benchmark_ready: bool,
    pub no_lookahead_safe: bool,
    pub status: KRXCandleSufficiencyStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct KRXCandleSufficiencyReport {
    pub report_id: String,
    pub items: Vec<KRXCandleSufficiencyItem>,
    pub total_series: usize,
    pub official_ready_series: usize,
    pub benchmark_ready_series: usize,
    pub series_with_sufficient_future_window: usize,
    pub series_missing_future_window: usize,
    pub sufficiency_status: KRXCandleSufficiencyStatus,
    pub reason_codes: Vec<ReasonCode>,
}

impl KRXCandleSufficiencyReport {
    pub fn build(
        report_id: &str,
        canonical_csv_paths: &[String],
        barrier_profile_registry_path: Option<&str>,
    ) -> Self {
        let validation_reports = canonical_csv_paths
            .iter()
            .map(|path| validate_path(path, true, true))
            .collect::<Vec<_>>();
        Self::from_validation_reports(
            report_id,
            &validation_reports,
            barrier_profile_registry_path,
        )
    }

    pub fn from_batch_validation(
        batch_report: &KRXCanonicalBatchValidationReport,
        report_id: &str,
        barrier_profile_registry_path: Option<&str>,
    ) -> Self {
        Self::from_validation_reports(
            report_id,
            &batch_report.validation_reports,
            barrier_profile_registry_path,
        )
    }

    pub fn from_validation_reports(
        report_id: &str,
        validation_reports: &[KRXCanonicalValidationReport],
        barrier_profile_registry_path: Option<&str>,
    ) -> Self {
        let required_future_bars = required_future_bars(barrier_profile_registry_path);
        let mut items = validation_reports
            .iter()
            .map(|report| item_from_report(report, required_future_bars))
            .collect::<Vec<_>>();
        items.sort_by(|left, right| left.normalized_symbol.cmp(&right.normalized_symbol));
        let total_series = items.len();
        let official_ready_series = items.iter().filter(|item| item.official_ready).count();
        let benchmark_ready_series = items.iter().filter(|item| item.benchmark_ready).count();
        let series_with_sufficient_future_window = items
            .iter()
            .filter(|item| item.missing_future_bars.unwrap_or(0) == 0)
            .count();
        let series_missing_future_window = items
            .iter()
            .filter(|item| item.missing_future_bars.unwrap_or(0) > 0)
            .count();
        let sufficiency_status = if items.is_empty() {
            KRXCandleSufficiencyStatus::DiagnosticOnly
        } else if items
            .iter()
            .any(|item| matches!(item.status, KRXCandleSufficiencyStatus::MissingProvenance))
        {
            KRXCandleSufficiencyStatus::MissingProvenance
        } else if items
            .iter()
            .any(|item| matches!(item.status, KRXCandleSufficiencyStatus::MissingPreflight))
        {
            KRXCandleSufficiencyStatus::MissingPreflight
        } else if official_ready_series == 0 {
            KRXCandleSufficiencyStatus::MissingOfficialCandles
        } else if series_missing_future_window > 0 {
            KRXCandleSufficiencyStatus::MissingFutureWindows
        } else if items.iter().any(|item| {
            matches!(
                item.status,
                KRXCandleSufficiencyStatus::TimestampAlignmentWeak
            )
        }) {
            KRXCandleSufficiencyStatus::TimestampAlignmentWeak
        } else if items
            .iter()
            .any(|item| matches!(item.status, KRXCandleSufficiencyStatus::TimeframeMismatch))
        {
            KRXCandleSufficiencyStatus::TimeframeMismatch
        } else if items
            .iter()
            .any(|item| matches!(item.status, KRXCandleSufficiencyStatus::SymbolMismatch))
        {
            KRXCandleSufficiencyStatus::SymbolMismatch
        } else if items
            .iter()
            .any(|item| matches!(item.status, KRXCandleSufficiencyStatus::DataQualityTooLow))
        {
            KRXCandleSufficiencyStatus::DataQualityTooLow
        } else if items
            .iter()
            .any(|item| matches!(item.status, KRXCandleSufficiencyStatus::InsufficientRows))
        {
            KRXCandleSufficiencyStatus::InsufficientRows
        } else {
            KRXCandleSufficiencyStatus::HealthyKRXCandles
        };
        let mut reason_codes = vec![ReasonCode::KRXCanonicalValidationBuilt];
        if official_ready_series == 0 {
            reason_codes.push(ReasonCode::EvidenceStillInsufficient);
        }
        if series_missing_future_window > 0 {
            reason_codes.push(ReasonCode::InsufficientBars);
        }
        Self {
            report_id: report_id.to_string(),
            items,
            total_series,
            official_ready_series,
            benchmark_ready_series,
            series_with_sufficient_future_window,
            series_missing_future_window,
            sufficiency_status,
            reason_codes: stable_reason_codes(&reason_codes),
        }
    }

    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("report_id={}", self.report_id),
            format!("total_series={}", self.total_series),
            format!("official_ready_series={}", self.official_ready_series),
            format!("benchmark_ready_series={}", self.benchmark_ready_series),
            format!(
                "series_with_sufficient_future_window={}",
                self.series_with_sufficient_future_window
            ),
            format!(
                "series_missing_future_window={}",
                self.series_missing_future_window
            ),
            format!("sufficiency_status={:?}", self.sufficiency_status),
            format!(
                "reason_codes={}",
                self.reason_codes
                    .iter()
                    .map(|reason| format!("{reason:?}"))
                    .collect::<Vec<_>>()
                    .join("|")
            ),
        ];
        lines.extend(self.items.iter().map(|item| {
            format!(
                "provider_symbol={};normalized_symbol={};timeframe={};row_count={};timestamp_start_ms={};timestamp_end_ms={};required_future_bars={};available_future_bars={};missing_future_bars={};official_ready={};benchmark_ready={};no_lookahead_safe={};status={:?};reason_codes={}",
                item.provider_symbol,
                item.normalized_symbol,
                item.timeframe,
                item.row_count,
                item.timestamp_start_ms.map(|value| value.to_string()).unwrap_or_default(),
                item.timestamp_end_ms.map(|value| value.to_string()).unwrap_or_default(),
                item.required_future_bars.map(|value| value.to_string()).unwrap_or_default(),
                item.available_future_bars.map(|value| value.to_string()).unwrap_or_default(),
                item.missing_future_bars.map(|value| value.to_string()).unwrap_or_default(),
                item.official_ready,
                item.benchmark_ready,
                item.no_lookahead_safe,
                item.status,
                item.reason_codes.iter().map(|reason| format!("{reason:?}")).collect::<Vec<_>>().join("|")
            )
        }));
        lines.join("\n")
    }

    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn from_json_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        serde_json::from_str(&text).map_err(|err| err.to_string())
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        let text_path = output_dir.join("krx_candle_sufficiency.txt");
        fs::write(&text_path, self.to_text()).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("krx_candle_sufficiency.json"),
            self.to_json_string()?,
        )
        .map_err(|err| err.to_string())?;
        Ok(text_path)
    }
}

fn item_from_report(
    report: &KRXCanonicalValidationReport,
    required_future_bars: usize,
) -> KRXCandleSufficiencyItem {
    let available_future_bars = report.row_count.saturating_sub(1);
    let missing_future_bars = required_future_bars.saturating_sub(available_future_bars);
    let official_ready = report.official_readiness_eligible;
    let no_lookahead_safe = report.duplicates_count == 0
        && !matches!(
            report.validation_status,
            KRXCanonicalValidationStatus::BadTimestamp
        )
        && report.row_count > required_future_bars;
    let timeframe_mismatch = !report.canonical_csv_path.contains("1d") || report.row_count == 0;
    let mut reason_codes = report.reason_codes.clone();
    let status = if !report.provenance_available {
        reason_codes.push(ReasonCode::MissingOfficialProvenance);
        KRXCandleSufficiencyStatus::MissingProvenance
    } else if !report.preflight_available {
        reason_codes.push(ReasonCode::MissingOfficialPreflight);
        KRXCandleSufficiencyStatus::MissingPreflight
    } else if matches!(
        report.validation_status,
        KRXCanonicalValidationStatus::DataQualityTooLow
            | KRXCanonicalValidationStatus::BadPrice
            | KRXCanonicalValidationStatus::BadVolume
            | KRXCanonicalValidationStatus::OhlcInvariantFailed
    ) {
        reason_codes.push(ReasonCode::DataQualityTooLow);
        KRXCandleSufficiencyStatus::DataQualityTooLow
    } else if timeframe_mismatch {
        reason_codes.push(ReasonCode::UnsupportedTimeframe);
        KRXCandleSufficiencyStatus::TimeframeMismatch
    } else if report
        .provider_symbol
        .as_deref()
        .unwrap_or_default()
        .is_empty()
        || report
            .normalized_symbol
            .as_deref()
            .unwrap_or_default()
            .is_empty()
    {
        reason_codes.push(ReasonCode::InvalidSymbol);
        KRXCandleSufficiencyStatus::SymbolMismatch
    } else if report.row_count < 3 {
        reason_codes.push(ReasonCode::InsufficientBars);
        KRXCandleSufficiencyStatus::InsufficientRows
    } else if missing_future_bars > 0 {
        reason_codes.push(ReasonCode::InsufficientBars);
        KRXCandleSufficiencyStatus::MissingFutureWindows
    } else if report.gap_count > 1 {
        reason_codes.push(ReasonCode::GapDetected);
        KRXCandleSufficiencyStatus::TimestampAlignmentWeak
    } else if !official_ready {
        reason_codes.push(ReasonCode::EvidenceStillInsufficient);
        KRXCandleSufficiencyStatus::MissingOfficialCandles
    } else {
        KRXCandleSufficiencyStatus::HealthyKRXCandles
    };
    let benchmark_ready = official_ready
        && missing_future_bars == 0
        && no_lookahead_safe
        && matches!(status, KRXCandleSufficiencyStatus::HealthyKRXCandles);
    KRXCandleSufficiencyItem {
        provider_symbol: report.provider_symbol.clone().unwrap_or_default(),
        normalized_symbol: report.normalized_symbol.clone().unwrap_or_default(),
        timeframe: "1d".to_string(),
        row_count: report.row_count,
        timestamp_start_ms: report.timestamp_start_ms,
        timestamp_end_ms: report.timestamp_end_ms,
        required_future_bars: Some(required_future_bars),
        available_future_bars: Some(available_future_bars),
        missing_future_bars: Some(missing_future_bars),
        official_ready,
        benchmark_ready,
        no_lookahead_safe,
        status,
        reason_codes: stable_reason_codes(&reason_codes),
    }
}

fn required_future_bars(path: Option<&str>) -> usize {
    path.and_then(|path| load_barrier_profile_registry_from_path_or_config(path).ok())
        .map(|registry| {
            registry
                .official_sufficiency_eligible_profiles
                .iter()
                .map(|profile| profile.horizon_bars)
                .max()
                .unwrap_or(3)
        })
        .unwrap_or(3)
}
