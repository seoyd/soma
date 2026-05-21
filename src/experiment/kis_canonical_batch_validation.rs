use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};
use crate::data::{DataManifest, DataProvenance, PreflightFinalStatus, PreflightReport};

use super::kis_symbol_whitelist::{KISDataFreshness, KISMarket};

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
    ManifestMissing,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct KISCanonicalValidationReport {
    pub canonical_csv_path: String,
    pub market: KISMarket,
    #[serde(default)]
    pub provider_symbol: Option<String>,
    #[serde(default)]
    pub normalized_symbol: Option<String>,
    #[serde(default)]
    pub exchange_code: Option<String>,
    pub timeframe: String,
    pub freshness: KISDataFreshness,
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
    pub manifest_available: bool,
    pub official_readiness_eligible: bool,
    pub validation_status: KISCanonicalValidationStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum KISCanonicalBatchValidationStatus {
    BatchValid,
    BatchPartiallyValid,
    BatchInvalid,
    MissingProvenance,
    MissingPreflight,
    DataQualityTooLow,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct KISCanonicalBatchValidationReport {
    pub batch_id: String,
    pub validation_reports: Vec<KISCanonicalValidationReport>,
    pub valid_csv_count: usize,
    pub invalid_csv_count: usize,
    pub domestic_valid_csv_count: usize,
    pub overseas_valid_csv_count: usize,
    pub total_rows: usize,
    pub official_readiness_eligible_csv_count: usize,
    pub missing_provenance_count: usize,
    pub missing_preflight_count: usize,
    pub data_quality_too_low_count: usize,
    pub duplicate_timestamp_count: usize,
    pub gap_heavy_count: usize,
    pub validation_status: KISCanonicalBatchValidationStatus,
    pub reason_codes: Vec<ReasonCode>,
}

impl KISCanonicalValidationReport {
    #[allow(clippy::too_many_arguments)]
    pub fn validate(
        canonical_csv_path: &str,
        market: KISMarket,
        provider_symbol: Option<String>,
        normalized_symbol: Option<String>,
        exchange_code: Option<String>,
        timeframe: String,
        freshness: KISDataFreshness,
        provenance_path: Option<&str>,
        preflight_path: Option<&str>,
        manifest_path: Option<&str>,
        require_provenance: bool,
        require_preflight: bool,
        require_manifest: bool,
    ) -> Self {
        let mut reason_codes = vec![ReasonCode::KISCanonicalValidationBuilt];
        let mut report = Self {
            canonical_csv_path: canonical_csv_path.to_string(),
            market,
            provider_symbol,
            normalized_symbol,
            exchange_code,
            timeframe,
            freshness,
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
            manifest_available: false,
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
        let mut seen = BTreeSet::new();
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
            let bid = parse_f64(&fields, &header_index, "bid");
            let ask = parse_f64(&fields, &header_index, "ask");
            if let Some(timestamp) = timestamp {
                if !seen.insert(timestamp) {
                    report.duplicates_count += 1;
                }
                timestamps.push(timestamp);
            } else {
                report.validation_status = KISCanonicalValidationStatus::BadTimestamp;
                reason_codes.push(ReasonCode::UnsupportedTimestampFormat);
            }
            if [open, high, low, close, bid, ask]
                .iter()
                .any(|value| value.is_none())
            {
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
        let manifest = manifest_path.and_then(load_manifest);
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
        report.manifest_available = manifest
            .as_ref()
            .is_some_and(|manifest| manifest.row_count > 0);

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
        } else if require_manifest && !report.manifest_available {
            reason_codes.push(ReasonCode::DataManifestBuilt);
            KISCanonicalValidationStatus::ManifestMissing
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
            && (!require_preflight || report.preflight_available)
            && (!require_manifest || report.manifest_available);
        report.reason_codes = stable_reason_codes(&reason_codes);
        report
    }

    pub fn to_text(&self) -> String {
        format!(
            "canonical_csv_path={};market={:?};provider_symbol={};normalized_symbol={};exchange_code={};timeframe={};freshness={:?};row_count={};timestamp_start_ms={};timestamp_end_ms={};required_columns_present={};ohlc_valid={};duplicates_count={};gap_count={};quality_score={};provenance_available={};preflight_available={};manifest_available={};official_readiness_eligible={};validation_status={:?};reason_codes={}",
            self.canonical_csv_path,
            self.market,
            self.provider_symbol.clone().unwrap_or_default(),
            self.normalized_symbol.clone().unwrap_or_default(),
            self.exchange_code.clone().unwrap_or_default(),
            self.timeframe,
            self.freshness,
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
            self.manifest_available,
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

impl KISCanonicalBatchValidationReport {
    pub fn build(
        batch_id: &str,
        canonical_csv_paths: &[String],
        require_provenance: bool,
        require_preflight: bool,
        require_manifest: bool,
    ) -> Self {
        let mut validation_reports = canonical_csv_paths
            .iter()
            .map(|path| {
                validate_path(
                    path,
                    require_provenance,
                    require_preflight,
                    require_manifest,
                )
            })
            .collect::<Vec<_>>();
        validation_reports
            .sort_by(|left, right| left.canonical_csv_path.cmp(&right.canonical_csv_path));
        Self::from_validation_reports(batch_id, validation_reports)
    }

    pub fn from_validation_reports(
        batch_id: &str,
        validation_reports: Vec<KISCanonicalValidationReport>,
    ) -> Self {
        let valid_csv_count = validation_reports
            .iter()
            .filter(|report| {
                matches!(
                    report.validation_status,
                    KISCanonicalValidationStatus::Valid
                )
            })
            .count();
        let invalid_csv_count = validation_reports.len().saturating_sub(valid_csv_count);
        let domestic_valid_csv_count = validation_reports
            .iter()
            .filter(|report| {
                report.market == KISMarket::KoreanEquity
                    && matches!(
                        report.validation_status,
                        KISCanonicalValidationStatus::Valid
                    )
            })
            .count();
        let overseas_valid_csv_count = validation_reports
            .iter()
            .filter(|report| {
                report.market != KISMarket::KoreanEquity
                    && matches!(
                        report.validation_status,
                        KISCanonicalValidationStatus::Valid
                    )
            })
            .count();
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
                    KISCanonicalValidationStatus::DataQualityTooLow
                        | KISCanonicalValidationStatus::BadPrice
                        | KISCanonicalValidationStatus::BadVolume
                        | KISCanonicalValidationStatus::OhlcInvariantFailed
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
                    KISCanonicalValidationStatus::GapHeavy
                )
            })
            .count();
        let validation_status = if validation_reports.is_empty() {
            KISCanonicalBatchValidationStatus::DiagnosticOnly
        } else if missing_provenance_count > 0 && valid_csv_count == 0 {
            KISCanonicalBatchValidationStatus::MissingProvenance
        } else if missing_preflight_count > 0 && valid_csv_count == 0 {
            KISCanonicalBatchValidationStatus::MissingPreflight
        } else if data_quality_too_low_count > 0 && valid_csv_count == 0 {
            KISCanonicalBatchValidationStatus::DataQualityTooLow
        } else if valid_csv_count == validation_reports.len() {
            KISCanonicalBatchValidationStatus::BatchValid
        } else if valid_csv_count > 0 {
            KISCanonicalBatchValidationStatus::BatchPartiallyValid
        } else {
            KISCanonicalBatchValidationStatus::BatchInvalid
        };
        let mut reason_codes = vec![ReasonCode::KISCanonicalValidationBuilt];
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
            domestic_valid_csv_count,
            overseas_valid_csv_count,
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
            format!("domestic_valid_csv_count={}", self.domestic_valid_csv_count),
            format!("overseas_valid_csv_count={}", self.overseas_valid_csv_count),
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
                .map(KISCanonicalValidationReport::to_text),
        );
        lines.join("\n")
    }

    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<std::path::PathBuf, String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        let text_path = output_dir.join("kis_canonical_batch_validation.txt");
        fs::write(&text_path, self.to_text()).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("kis_canonical_batch_validation.json"),
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
    require_manifest: bool,
) -> KISCanonicalValidationReport {
    let metadata = infer_metadata_from_path(canonical_csv_path);
    KISCanonicalValidationReport::validate(
        canonical_csv_path,
        metadata.market,
        metadata.provider_symbol,
        metadata.normalized_symbol,
        metadata.exchange_code,
        metadata.timeframe,
        metadata.freshness,
        infer_sidecar_path(canonical_csv_path, "_provenance.json").as_deref(),
        infer_sidecar_path(canonical_csv_path, "_preflight.json").as_deref(),
        infer_sidecar_path(canonical_csv_path, "_manifest.json").as_deref(),
        require_provenance,
        require_preflight,
        require_manifest,
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

#[derive(Clone)]
struct InferredMetadata {
    market: KISMarket,
    provider_symbol: Option<String>,
    normalized_symbol: Option<String>,
    exchange_code: Option<String>,
    timeframe: String,
    freshness: KISDataFreshness,
}

fn infer_metadata_from_path(path: &str) -> InferredMetadata {
    let stem = Path::new(path)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or_default();
    let tokens = stem.split('_').collect::<Vec<_>>();
    match tokens.as_slice() {
        ["kis", "kr", symbol, timeframe, freshness, ..] => InferredMetadata {
            market: KISMarket::KoreanEquity,
            provider_symbol: Some((*symbol).to_string()),
            normalized_symbol: Some((*symbol).to_string()),
            exchange_code: None,
            timeframe: (*timeframe).to_string(),
            freshness: infer_freshness(freshness),
        },
        ["kis", market, exchange, symbol, timeframe, freshness, ..] => InferredMetadata {
            market: infer_market(market),
            provider_symbol: Some((*symbol).to_string()),
            normalized_symbol: Some(symbol.to_ascii_uppercase()),
            exchange_code: Some((*exchange).to_ascii_uppercase()),
            timeframe: (*timeframe).to_string(),
            freshness: infer_freshness(freshness),
        },
        _ => InferredMetadata {
            market: KISMarket::KoreanEquity,
            provider_symbol: None,
            normalized_symbol: None,
            exchange_code: None,
            timeframe: "1d".to_string(),
            freshness: KISDataFreshness::Unknown,
        },
    }
}

fn infer_market(value: &str) -> KISMarket {
    match value {
        "kr" => KISMarket::KoreanEquity,
        "us" => KISMarket::USEquity,
        "jp" => KISMarket::JapaneseEquity,
        "hk" => KISMarket::HongKongEquity,
        _ => KISMarket::OtherOverseasEquity,
    }
}

fn infer_freshness(value: &str) -> KISDataFreshness {
    match value {
        "eod" => KISDataFreshness::Eod,
        "delayed" => KISDataFreshness::Delayed,
        "realtime" => KISDataFreshness::Realtime,
        _ => KISDataFreshness::Unknown,
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

fn load_manifest(path: &str) -> Option<DataManifest> {
    let text = fs::read_to_string(path).ok()?;
    serde_json::from_str(&text).ok()
}
