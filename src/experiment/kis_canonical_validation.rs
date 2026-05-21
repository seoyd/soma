use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};
use crate::data::{DataProvenance, PreflightFinalStatus, PreflightReport};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum KISCanonicalValidationStatus {
    Valid,
    MissingFile,
    MissingRequiredColumns,
    BadTimestamp,
    OhlcInvariantFailed,
    DuplicateTimestamp,
    GapHeavy,
    BadVolume,
    BadPrice,
    DataQualityTooLow,
    PreflightMissing,
    ProvenanceMissing,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct KISCanonicalValidationReport {
    pub canonical_csv_path: String,
    #[serde(default)]
    pub provider_symbol: Option<String>,
    #[serde(default)]
    pub normalized_symbol: Option<String>,
    pub row_count: usize,
    #[serde(default)]
    pub timestamp_start_ms: Option<u64>,
    #[serde(default)]
    pub timestamp_end_ms: Option<u64>,
    pub required_columns_present: bool,
    pub ohlc_valid: bool,
    pub duplicates_count: usize,
    pub gap_count: usize,
    #[serde(default)]
    pub quality_score: Option<f64>,
    pub provenance_available: bool,
    pub preflight_available: bool,
    pub official_readiness_eligible: bool,
    pub validation_status: KISCanonicalValidationStatus,
    pub reason_codes: Vec<ReasonCode>,
}

impl KISCanonicalValidationReport {
    pub fn validate(
        canonical_csv_path: &str,
        provider_symbol: Option<String>,
        normalized_symbol: Option<String>,
        provenance_path: Option<&str>,
        preflight_path: Option<&str>,
        require_provenance: bool,
        require_preflight: bool,
    ) -> Self {
        let mut reason_codes = vec![ReasonCode::KISCanonicalValidationBuilt];
        let mut report = Self {
            canonical_csv_path: canonical_csv_path.to_string(),
            provider_symbol,
            normalized_symbol,
            row_count: 0,
            timestamp_start_ms: None,
            timestamp_end_ms: None,
            required_columns_present: false,
            ohlc_valid: false,
            duplicates_count: 0,
            gap_count: 0,
            quality_score: None,
            provenance_available: false,
            preflight_available: false,
            official_readiness_eligible: false,
            validation_status: KISCanonicalValidationStatus::DiagnosticOnly,
            reason_codes: Vec::new(),
        };

        let canonical_path = Path::new(canonical_csv_path);
        if !canonical_path.exists() {
            reason_codes.push(ReasonCode::MissingFile);
            report.validation_status = KISCanonicalValidationStatus::MissingFile;
            report.reason_codes = stable_reason_codes(&reason_codes);
            return report;
        }

        let csv_text = match fs::read_to_string(canonical_path) {
            Ok(text) => text,
            Err(_) => {
                reason_codes.push(ReasonCode::MissingFile);
                report.validation_status = KISCanonicalValidationStatus::MissingFile;
                report.reason_codes = stable_reason_codes(&reason_codes);
                return report;
            }
        };
        let mut lines = csv_text.lines();
        let header = lines.next().unwrap_or_default();
        let header_index = header
            .split(',')
            .enumerate()
            .map(|(index, value)| (value.trim().to_string(), index))
            .collect::<BTreeMap<_, _>>();
        let required_columns = [
            "timestamp_ms",
            "open",
            "high",
            "low",
            "close",
            "volume",
            "trade_value",
            "bid",
            "ask",
            "spread_bps",
        ];
        report.required_columns_present = required_columns
            .iter()
            .all(|column| header_index.contains_key(*column));
        if !report.required_columns_present {
            reason_codes.push(ReasonCode::MissingRequiredColumn);
            report.validation_status = KISCanonicalValidationStatus::MissingRequiredColumns;
            report.reason_codes = stable_reason_codes(&reason_codes);
            return report;
        }

        let mut timestamps = Vec::new();
        let mut duplicates = BTreeSet::new();
        let mut ohlc_valid = true;
        let mut bad_volume = false;
        let mut bad_price = false;
        for line in lines.filter(|line| !line.trim().is_empty()) {
            let fields = line.split(',').map(str::trim).collect::<Vec<_>>();
            let timestamp = parse_u64(&fields, &header_index, "timestamp_ms");
            let open = parse_f64(&fields, &header_index, "open");
            let high = parse_f64(&fields, &header_index, "high");
            let low = parse_f64(&fields, &header_index, "low");
            let close = parse_f64(&fields, &header_index, "close");
            let volume = parse_f64(&fields, &header_index, "volume");
            let trade_value = parse_f64(&fields, &header_index, "trade_value");
            if let Some(timestamp) = timestamp {
                if !duplicates.insert(timestamp) {
                    report.duplicates_count += 1;
                }
                timestamps.push(timestamp);
            } else {
                reason_codes.push(ReasonCode::UnsupportedTimestampFormat);
                report.validation_status = KISCanonicalValidationStatus::BadTimestamp;
            }
            if [open, high, low, close].iter().any(|value| value.is_none()) {
                bad_price = true;
            }
            if [volume, trade_value].iter().any(|value| value.is_none()) {
                bad_volume = true;
            }
            if let (Some(open), Some(high), Some(low), Some(close)) = (open, high, low, close) {
                if open < 0.0
                    || high < 0.0
                    || low < 0.0
                    || close < 0.0
                    || high < open
                    || high < close
                    || high < low
                    || low > open
                    || low > close
                {
                    ohlc_valid = false;
                }
            }
            if let Some(volume) = volume {
                if volume < 0.0 {
                    bad_volume = true;
                }
            }
            if let Some(trade_value) = trade_value {
                if trade_value < 0.0 {
                    bad_volume = true;
                }
            }
        }
        timestamps.sort_unstable();
        report.row_count = timestamps.len();
        report.timestamp_start_ms = timestamps.first().copied();
        report.timestamp_end_ms = timestamps.last().copied();
        report.ohlc_valid = ohlc_valid;
        report.gap_count = timestamps
            .windows(2)
            .filter(|pair| pair[1].saturating_sub(pair[0]) > 3 * 86_400_000)
            .count();

        let preflight = preflight_path.and_then(load_preflight_report);
        let provenance = provenance_path.and_then(load_provenance);
        report.preflight_available = preflight.as_ref().is_some_and(|report| {
            !matches!(
                report.final_status,
                PreflightFinalStatus::MissingFile
                    | PreflightFinalStatus::UnsupportedFormat
                    | PreflightFinalStatus::AmbiguousFormat
                    | PreflightFinalStatus::NotRealLocalEligible
            )
        });
        report.provenance_available = provenance.as_ref().is_some_and(|provenance| {
            provenance.official_provider.unwrap_or(false)
                && provenance.readiness_eligible.unwrap_or(true)
                && !provenance
                    .validate_local_only()
                    .contains(&ReasonCode::LocalPathRejected)
        });

        let mut quality_score = 1.0;
        quality_score -= report.duplicates_count as f64 * 0.25;
        quality_score -= report.gap_count as f64 * 0.10;
        if !ohlc_valid {
            quality_score -= 0.30;
        }
        if bad_volume {
            quality_score -= 0.20;
        }
        if bad_price {
            quality_score -= 0.20;
        }
        quality_score = quality_score.clamp(0.0, 1.0);
        report.quality_score = Some(quality_score);

        report.validation_status = if require_provenance && !report.provenance_available {
            reason_codes.push(ReasonCode::MissingOfficialProvenance);
            KISCanonicalValidationStatus::ProvenanceMissing
        } else if require_preflight && !report.preflight_available {
            reason_codes.push(ReasonCode::MissingOfficialPreflight);
            KISCanonicalValidationStatus::PreflightMissing
        } else if report.duplicates_count > 0 {
            reason_codes.push(ReasonCode::DuplicateTimestampDetected);
            KISCanonicalValidationStatus::DuplicateTimestamp
        } else if !ohlc_valid {
            reason_codes.push(ReasonCode::OhlcInvariantViolationDetected);
            KISCanonicalValidationStatus::OhlcInvariantFailed
        } else if bad_price {
            reason_codes.push(ReasonCode::NonPositivePrice);
            KISCanonicalValidationStatus::BadPrice
        } else if bad_volume {
            reason_codes.push(ReasonCode::NegativeVolumeDetected);
            KISCanonicalValidationStatus::BadVolume
        } else if report.gap_count > 3 {
            reason_codes.push(ReasonCode::GapDetected);
            KISCanonicalValidationStatus::GapHeavy
        } else if quality_score < 0.50 {
            reason_codes.push(ReasonCode::DataQualityTooLow);
            KISCanonicalValidationStatus::DataQualityTooLow
        } else if matches!(
            report.validation_status,
            KISCanonicalValidationStatus::BadTimestamp
        ) {
            KISCanonicalValidationStatus::BadTimestamp
        } else {
            KISCanonicalValidationStatus::Valid
        };
        report.official_readiness_eligible = matches!(
            report.validation_status,
            KISCanonicalValidationStatus::Valid
        ) && (!require_provenance
            || report.provenance_available)
            && (!require_preflight || report.preflight_available);
        report.reason_codes = stable_reason_codes(&reason_codes);
        report
    }

    pub fn to_text(&self) -> String {
        format!(
            "canonical_csv_path={};provider_symbol={};normalized_symbol={};row_count={};timestamp_start_ms={};timestamp_end_ms={};required_columns_present={};ohlc_valid={};duplicates_count={};gap_count={};quality_score={};provenance_available={};preflight_available={};official_readiness_eligible={};validation_status={:?};reason_codes={}",
            self.canonical_csv_path,
            self.provider_symbol.clone().unwrap_or_default(),
            self.normalized_symbol.clone().unwrap_or_default(),
            self.row_count,
            self.timestamp_start_ms
                .map(|value| value.to_string())
                .unwrap_or_default(),
            self.timestamp_end_ms
                .map(|value| value.to_string())
                .unwrap_or_default(),
            self.required_columns_present,
            self.ohlc_valid,
            self.duplicates_count,
            self.gap_count,
            self.quality_score
                .map(|value| format!("{value:.4}"))
                .unwrap_or_default(),
            self.provenance_available,
            self.preflight_available,
            self.official_readiness_eligible,
            self.validation_status,
            self.reason_codes
                .iter()
                .map(|reason| format!("{reason:?}"))
                .collect::<Vec<_>>()
                .join("|")
        )
    }
}

fn parse_u64(fields: &[&str], header_index: &BTreeMap<String, usize>, name: &str) -> Option<u64> {
    let index = *header_index.get(name)?;
    fields.get(index)?.parse().ok()
}

fn parse_f64(fields: &[&str], header_index: &BTreeMap<String, usize>, name: &str) -> Option<f64> {
    let index = *header_index.get(name)?;
    fields.get(index)?.parse().ok()
}

fn load_preflight_report(path: &str) -> Option<PreflightReport> {
    let text = fs::read_to_string(path).ok()?;
    serde_json::from_str(&text).ok()
}

fn load_provenance(path: &str) -> Option<DataProvenance> {
    let text = fs::read_to_string(path).ok()?;
    serde_json::from_str(&text).ok()
}
