use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_hash_string, stable_reason_codes};
use crate::data::{
    DataManifest, DataProvenance, EvidenceSourceKind, PreflightFinalStatus, PreflightReport,
    ProviderKind, ProviderMarket, infer_source_kind_from_path,
};

use super::comparable_committee_evidence::infer_market_from_symbol;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OfficialCandleCoveragePackConfig {
    pub pack_id: String,
    #[serde(default)]
    pub canonical_csv_paths: Vec<String>,
    #[serde(default)]
    pub provenance_paths: Vec<String>,
    #[serde(default)]
    pub preflight_report_paths: Vec<String>,
    #[serde(default)]
    pub manifest_paths: Vec<String>,
    #[serde(default)]
    pub official_replication_report_paths: Vec<String>,
    #[serde(default)]
    pub comparable_evidence_bundle_paths: Vec<String>,
    #[serde(default)]
    pub scenario_pack_paths: Vec<String>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default = "default_max_rows")]
    pub max_rows: usize,
    #[serde(default = "default_max_symbols")]
    pub max_symbols: usize,
    #[serde(default = "default_max_timeframes")]
    pub max_timeframes: usize,
    #[serde(default = "default_max_bytes")]
    pub max_bytes: usize,
    #[serde(default = "default_true")]
    pub require_official_source: bool,
    #[serde(default = "default_true")]
    pub require_provenance: bool,
    #[serde(default = "default_true")]
    pub require_preflight: bool,
    #[serde(default)]
    pub require_manifest: bool,
    #[serde(default = "default_true")]
    pub allow_crypto_only: bool,
    #[serde(default)]
    pub allow_controlled_fixture: bool,
    #[serde(default)]
    pub allow_yfinance_research: bool,
    #[serde(default)]
    pub allow_fixture: bool,
    #[serde(default)]
    pub allow_synthetic_test: bool,
    #[serde(default)]
    pub allow_timeframe_aggregation: bool,
    #[serde(default = "default_true")]
    pub allow_timestamp_tolerance: bool,
    #[serde(default = "default_timestamp_tolerance_ms")]
    pub timestamp_tolerance_ms: u64,
    #[serde(default = "default_true")]
    pub require_no_lookahead_safe: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum OfficialCandleSeriesSourceClass {
    OfficialNonCrypto,
    OfficialCryptoOnly,
    ControlledDiagnostic,
    YFinanceResearch,
    FixtureArchitectureTest,
    SyntheticTest,
    #[default]
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfficialCandleSeriesDescriptor {
    pub candle_series_id: String,
    pub path: String,
    #[serde(default)]
    pub provider_kind: Option<ProviderKind>,
    pub source_kind: EvidenceSourceKind,
    pub source_class: OfficialCandleSeriesSourceClass,
    pub market: ProviderMarket,
    #[serde(default)]
    pub venue: Option<String>,
    pub symbol: String,
    pub normalized_symbol: String,
    pub timeframe: String,
    pub row_count: usize,
    pub timestamp_start_ms: u64,
    pub timestamp_end_ms: u64,
    pub has_duplicates: bool,
    pub has_gaps: bool,
    #[serde(default)]
    pub data_quality_score: Option<f64>,
    pub provenance_available: bool,
    pub preflight_ready: bool,
    pub manifest_available: bool,
    #[serde(default)]
    pub timestamp_policy: Option<String>,
    #[serde(default)]
    pub adjusted_price_policy: Option<String>,
    pub official_readiness_eligible: bool,
    pub benchmark_eligible: bool,
    pub diagnostic_only: bool,
    #[serde(default)]
    pub storage_bytes: usize,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfficialCandleCoveragePack {
    pub pack_id: String,
    pub descriptors: Vec<OfficialCandleSeriesDescriptor>,
    pub official_non_crypto_series: Vec<OfficialCandleSeriesDescriptor>,
    pub official_crypto_series: Vec<OfficialCandleSeriesDescriptor>,
    pub controlled_series: Vec<OfficialCandleSeriesDescriptor>,
    pub yfinance_series: Vec<OfficialCandleSeriesDescriptor>,
    pub fixture_series: Vec<OfficialCandleSeriesDescriptor>,
    pub unknown_series: Vec<OfficialCandleSeriesDescriptor>,
    pub total_rows: usize,
    pub total_symbols: usize,
    pub total_timeframes: usize,
    pub storage_bytes: usize,
    pub readiness_eligible_series_count: usize,
    pub benchmark_eligible_series_count: usize,
    pub warnings: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CandleCsvTimestampSeries {
    pub symbol: String,
    pub normalized_symbol: String,
    pub timeframe: String,
    pub timestamps: Vec<u64>,
    pub timestamp_policy: Option<String>,
}

impl Default for OfficialCandleCoveragePackConfig {
    fn default() -> Self {
        Self {
            pack_id: "official-candle-coverage-pack".to_string(),
            canonical_csv_paths: Vec::new(),
            provenance_paths: Vec::new(),
            preflight_report_paths: Vec::new(),
            manifest_paths: Vec::new(),
            official_replication_report_paths: Vec::new(),
            comparable_evidence_bundle_paths: Vec::new(),
            scenario_pack_paths: Vec::new(),
            output_root: default_output_root(),
            max_rows: default_max_rows(),
            max_symbols: default_max_symbols(),
            max_timeframes: default_max_timeframes(),
            max_bytes: default_max_bytes(),
            require_official_source: true,
            require_provenance: true,
            require_preflight: true,
            require_manifest: false,
            allow_crypto_only: true,
            allow_controlled_fixture: false,
            allow_yfinance_research: false,
            allow_fixture: false,
            allow_synthetic_test: false,
            allow_timeframe_aggregation: false,
            allow_timestamp_tolerance: true,
            timestamp_tolerance_ms: default_timestamp_tolerance_ms(),
            require_no_lookahead_safe: true,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

impl OfficialCandleCoveragePackConfig {
    pub fn from_toml_str(input: &str) -> Result<Self, String> {
        toml::from_str(input).map_err(|err| err.to_string())
    }

    pub fn from_toml_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        Self::from_toml_str(&text)
    }

    pub fn to_toml_string(&self) -> Result<String, String> {
        toml::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.pack_id.trim().is_empty() {
            return Err("official candle coverage pack id must not be empty".to_string());
        }
        if self
            .all_paths()
            .iter()
            .chain(std::iter::once(&self.output_root))
            .any(|path| is_remote_path(path))
        {
            return Err("official candle coverage pack paths must be local".to_string());
        }
        if self.max_rows == 0 || self.max_rows > default_max_rows() {
            return Err(
                "official candle coverage pack max_rows must be between 1 and 1000".to_string(),
            );
        }
        if self.max_symbols == 0 || self.max_symbols > default_max_symbols() {
            return Err(
                "official candle coverage pack max_symbols must be between 1 and 5".to_string(),
            );
        }
        if self.max_timeframes == 0 || self.max_timeframes > default_max_timeframes() {
            return Err(
                "official candle coverage pack max_timeframes must be between 1 and 5".to_string(),
            );
        }
        if self.max_bytes == 0 || self.max_bytes > default_max_bytes() {
            return Err(
                "official candle coverage pack max_bytes must be between 1 and 5000000".to_string(),
            );
        }
        if self.timestamp_tolerance_ms > 86_400_000 {
            return Err(
                "official candle coverage pack timestamp_tolerance_ms must be bounded to one day"
                    .to_string(),
            );
        }
        Ok(())
    }

    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.pack_id)
    }

    pub fn all_paths(&self) -> Vec<String> {
        self.canonical_csv_paths
            .iter()
            .chain(self.provenance_paths.iter())
            .chain(self.preflight_report_paths.iter())
            .chain(self.manifest_paths.iter())
            .chain(self.official_replication_report_paths.iter())
            .chain(self.comparable_evidence_bundle_paths.iter())
            .chain(self.scenario_pack_paths.iter())
            .cloned()
            .collect()
    }
}

impl OfficialCandleCoveragePack {
    pub fn build(config: &OfficialCandleCoveragePackConfig) -> Result<Self, String> {
        config.validate()?;
        let provenance_lookup = build_sidecar_lookup(&config.provenance_paths);
        let preflight_lookup = build_sidecar_lookup(&config.preflight_report_paths);
        let manifest_lookup = build_sidecar_lookup(&config.manifest_paths);

        let mut warnings = Vec::new();
        let mut descriptors = config
            .canonical_csv_paths
            .iter()
            .map(|path| {
                describe_csv(
                    path,
                    config,
                    &provenance_lookup,
                    &preflight_lookup,
                    &manifest_lookup,
                )
            })
            .collect::<Vec<_>>();
        descriptors
            .sort_by(|left, right| descriptor_sort_key(left).cmp(&descriptor_sort_key(right)));

        let mut bounded = Vec::new();
        let mut total_rows = 0usize;
        let mut total_bytes = 0usize;
        let mut symbols = BTreeSet::new();
        let mut timeframes = BTreeSet::new();

        for descriptor in descriptors {
            if !descriptor_allowed_by_config(&descriptor, config) {
                warnings.push(format!(
                    "skipped_path={};reason=source-boundary-blocked",
                    descriptor.path
                ));
                continue;
            }
            let next_symbol_count = symbols
                .iter()
                .cloned()
                .chain([descriptor.normalized_symbol.clone()])
                .collect::<BTreeSet<_>>()
                .len();
            let next_timeframe_count = timeframes
                .iter()
                .cloned()
                .chain([descriptor.timeframe.clone()])
                .collect::<BTreeSet<_>>()
                .len();
            if total_rows.saturating_add(descriptor.row_count) > config.max_rows
                || next_symbol_count > config.max_symbols
                || next_timeframe_count > config.max_timeframes
                || total_bytes.saturating_add(descriptor.storage_bytes) > config.max_bytes
            {
                warnings.push(format!(
                    "skipped_path={};reason=bounded-budget",
                    descriptor.path
                ));
                continue;
            }
            total_rows += descriptor.row_count;
            total_bytes += descriptor.storage_bytes;
            symbols.insert(descriptor.normalized_symbol.clone());
            timeframes.insert(descriptor.timeframe.clone());
            bounded.push(descriptor);
        }

        let official_non_crypto_series =
            filter_by_class(&bounded, OfficialCandleSeriesSourceClass::OfficialNonCrypto);
        let official_crypto_series = filter_by_class(
            &bounded,
            OfficialCandleSeriesSourceClass::OfficialCryptoOnly,
        );
        let controlled_series = filter_by_class(
            &bounded,
            OfficialCandleSeriesSourceClass::ControlledDiagnostic,
        );
        let yfinance_series =
            filter_by_class(&bounded, OfficialCandleSeriesSourceClass::YFinanceResearch);
        let fixture_series = bounded
            .iter()
            .filter(|descriptor| {
                matches!(
                    descriptor.source_class,
                    OfficialCandleSeriesSourceClass::FixtureArchitectureTest
                        | OfficialCandleSeriesSourceClass::SyntheticTest
                )
            })
            .cloned()
            .collect::<Vec<_>>();
        let unknown_series = filter_by_class(&bounded, OfficialCandleSeriesSourceClass::Unknown);
        let readiness_eligible_series_count = bounded
            .iter()
            .filter(|descriptor| descriptor.official_readiness_eligible)
            .count();
        let benchmark_eligible_series_count = bounded
            .iter()
            .filter(|descriptor| descriptor.benchmark_eligible)
            .count();

        Ok(Self {
            pack_id: config.pack_id.clone(),
            descriptors: bounded,
            official_non_crypto_series,
            official_crypto_series,
            controlled_series,
            yfinance_series,
            fixture_series,
            unknown_series,
            total_rows,
            total_symbols: symbols.len(),
            total_timeframes: timeframes.len(),
            storage_bytes: total_bytes,
            readiness_eligible_series_count,
            benchmark_eligible_series_count,
            warnings,
            reason_codes: stable_reason_codes(
                &config
                    .reason_codes
                    .iter()
                    .cloned()
                    .chain([
                        ReasonCode::OfficialCandleCoverageBuilt,
                        ReasonCode::DeterministicPath,
                    ])
                    .collect::<Vec<_>>(),
            ),
        })
    }

    pub fn from_json_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        serde_json::from_str(&text).map_err(|err| err.to_string())
    }

    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn fingerprint(&self) -> String {
        stable_hash_string(
            &serde_json::to_string(self)
                .unwrap_or_else(|_| format!("{}:{}", self.pack_id, self.descriptors.len())),
        )
    }

    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("pack_id={}", self.pack_id),
            format!("descriptor_count={}", self.descriptors.len()),
            format!(
                "official_non_crypto_series={}",
                self.official_non_crypto_series.len()
            ),
            format!(
                "official_crypto_series={}",
                self.official_crypto_series.len()
            ),
            format!("controlled_series={}", self.controlled_series.len()),
            format!("yfinance_series={}", self.yfinance_series.len()),
            format!("fixture_series={}", self.fixture_series.len()),
            format!("unknown_series={}", self.unknown_series.len()),
            format!("total_rows={}", self.total_rows),
            format!("total_symbols={}", self.total_symbols),
            format!("total_timeframes={}", self.total_timeframes),
            format!("storage_bytes={}", self.storage_bytes),
            format!(
                "readiness_eligible_series_count={}",
                self.readiness_eligible_series_count
            ),
            format!(
                "benchmark_eligible_series_count={}",
                self.benchmark_eligible_series_count
            ),
            format!("warnings={}", self.warnings.join(" | ")),
            format!("fingerprint={}", self.fingerprint()),
        ];
        lines.extend(self.descriptors.iter().map(|descriptor| {
            format!(
                "series_id={};source_class={:?};source_kind={:?};market={:?};symbol={};timeframe={};path={};rows={};duplicates={};gaps={};provenance_available={};preflight_ready={};manifest_available={};official_ready={};benchmark_ready={};diagnostic_only={}",
                descriptor.candle_series_id,
                descriptor.source_class,
                descriptor.source_kind,
                descriptor.market,
                descriptor.symbol,
                descriptor.timeframe,
                descriptor.path,
                descriptor.row_count,
                descriptor.has_duplicates,
                descriptor.has_gaps,
                descriptor.provenance_available,
                descriptor.preflight_ready,
                descriptor.manifest_available,
                descriptor.official_readiness_eligible,
                descriptor.benchmark_eligible,
                descriptor.diagnostic_only,
            )
        }));
        lines.join("\n")
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("official_candle_coverage_pack.txt"),
            self.to_text(),
        )
        .map_err(|err| err.to_string())?;
        let json_path = output_dir.join("official_candle_coverage_pack.json");
        fs::write(&json_path, self.to_json_string()?).map_err(|err| err.to_string())?;
        Ok(json_path)
    }
}

pub fn load_pack_from_path_or_config(path: &str) -> Result<OfficialCandleCoveragePack, String> {
    if path.ends_with(".toml") {
        let config = OfficialCandleCoveragePackConfig::from_toml_path(Path::new(path))?;
        OfficialCandleCoveragePack::build(&config)
    } else {
        OfficialCandleCoveragePack::from_json_path(Path::new(path))
    }
}

pub fn load_candle_csv_timestamp_series(path: &Path) -> Result<CandleCsvTimestampSeries, String> {
    if path.extension().and_then(|value| value.to_str()) != Some("csv") {
        return Err("unsupported candle format".to_string());
    }
    let bytes = fs::read(path).map_err(|err| err.to_string())?;
    let text = String::from_utf8_lossy(&bytes);
    let mut lines = text.lines().filter(|line| !line.trim().is_empty());
    let header = lines
        .next()
        .ok_or_else(|| "empty candle csv".to_string())?
        .split(',')
        .map(|value| value.trim().to_ascii_lowercase())
        .collect::<Vec<_>>();
    let timestamp_index = header
        .iter()
        .position(|value| value == "timestamp" || value == "timestamp_ms")
        .ok_or_else(|| "missing timestamp column".to_string())?;
    let symbol_index = header.iter().position(|value| value == "symbol");
    let timeframe_index = header.iter().position(|value| value == "timeframe");
    let mut raw_symbol = None;
    let mut raw_timeframe = None;
    let mut timestamps = Vec::new();
    let mut saw_seconds = false;
    for line in lines {
        let columns = line
            .split(',')
            .map(|value| value.trim())
            .collect::<Vec<_>>();
        let Some(raw_timestamp) = columns.get(timestamp_index).copied() else {
            continue;
        };
        if let Ok(parsed) = raw_timestamp.parse::<u64>() {
            if parsed < 1_000_000_000_000 {
                saw_seconds = true;
            }
            timestamps.push(normalize_timestamp(parsed));
            if raw_symbol.is_none() {
                raw_symbol = symbol_index
                    .and_then(|index| columns.get(index))
                    .map(|value| value.to_string())
                    .filter(|value| !value.trim().is_empty());
            }
            if raw_timeframe.is_none() {
                raw_timeframe = timeframe_index
                    .and_then(|index| columns.get(index))
                    .map(|value| value.to_string())
                    .filter(|value| !value.trim().is_empty());
            }
        }
    }
    if timestamps.is_empty() {
        return Err("no valid candle timestamps".to_string());
    }
    timestamps.sort_unstable();
    let symbol = raw_symbol.unwrap_or_else(|| symbol_from_path(path));
    let normalized_symbol = normalize_symbol(&symbol);
    let timeframe =
        infer_timeframe_label(path, raw_timeframe.as_deref(), detect_min_step(&timestamps));
    Ok(CandleCsvTimestampSeries {
        symbol,
        normalized_symbol,
        timeframe,
        timestamps,
        timestamp_policy: Some(if saw_seconds {
            "SecondsCoercedToMillis".to_string()
        } else {
            "MillisecondsUtc".to_string()
        }),
    })
}

pub fn normalize_symbol(value: &str) -> String {
    let normalized = value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .collect::<String>()
        .to_ascii_uppercase();
    if normalized.is_empty() {
        value.trim().to_ascii_uppercase()
    } else {
        normalized
    }
}

pub fn normalize_timeframe_label(value: &str) -> String {
    let lowered = value.trim().to_ascii_lowercase();
    match lowered.as_str() {
        "oneminute" | "1m" | "1min" | "minute" => "1m".to_string(),
        "fiveminute" | "5m" | "5min" => "5m".to_string(),
        "fifteenminute" | "15m" | "15min" => "15m".to_string(),
        "onehour" | "1h" | "60m" | "hour" => "1h".to_string(),
        "oneday" | "1d" | "day" | "daily" => "1d".to_string(),
        "intraday" | "swing" | "multiday" | "longterm" | "summary" | "unknown" => lowered,
        _ => lowered,
    }
}

pub fn timeframe_seconds(value: &str) -> Option<u64> {
    match normalize_timeframe_label(value).as_str() {
        "1m" => Some(60),
        "5m" => Some(300),
        "15m" => Some(900),
        "1h" => Some(3_600),
        "1d" => Some(86_400),
        _ => None,
    }
}

fn describe_csv(
    path: &str,
    config: &OfficialCandleCoveragePackConfig,
    provenance_lookup: &BTreeMap<String, Vec<String>>,
    preflight_lookup: &BTreeMap<String, Vec<String>>,
    manifest_lookup: &BTreeMap<String, Vec<String>>,
) -> OfficialCandleSeriesDescriptor {
    let path_ref = Path::new(path);
    let storage_bytes = fs::metadata(path_ref)
        .map(|meta| meta.len() as usize)
        .unwrap_or_default();
    let provenance = find_sidecar(path_ref, provenance_lookup)
        .and_then(|matched| read_json::<DataProvenance>(&matched).ok());
    let preflight = find_sidecar(path_ref, preflight_lookup)
        .and_then(|matched| read_json::<PreflightReport>(&matched).ok());
    let manifest = find_sidecar(path_ref, manifest_lookup)
        .and_then(|matched| read_json::<DataManifest>(&matched).ok());
    let provenance_available = provenance.as_ref().is_some_and(|record| {
        !record
            .validate_local_only()
            .contains(&ReasonCode::LocalPathRejected)
    });
    let preflight_ready = preflight
        .as_ref()
        .is_some_and(|report| report.final_status == PreflightFinalStatus::ReadyForRealEvidence);
    let manifest_available = manifest.is_some();

    let mut reason_codes = Vec::new();
    let mut provider_kind = None;
    let mut venue = None;
    let mut adjusted_price_policy = None;
    let mut data_quality_score = None;
    if let Some(report) = &preflight {
        data_quality_score = report
            .data_quality_report
            .as_ref()
            .map(|report| report.data_quality_score);
        reason_codes.extend(report.reason_codes.iter().cloned());
        if let Some(preview) = &report.data_manifest_preview {
            adjusted_price_policy = preview.adjusted_price_policy_summary.clone();
            venue = Some(format!("{:?}", preview.venue));
        }
    }
    if let Some(record) = &provenance {
        reason_codes.extend(record.validate_local_only());
        provider_kind = provider_kind.or_else(|| {
            record
                .provider_label
                .as_deref()
                .and_then(parse_provider_kind)
        });
    }
    if let Some(record) = &manifest {
        reason_codes.extend(record.reason_codes.iter().cloned());
        adjusted_price_policy =
            adjusted_price_policy.or_else(|| record.adjusted_price_policy_summary.clone());
        venue = venue.or_else(|| Some(format!("{:?}", record.venue)));
        data_quality_score = data_quality_score.or(Some(record.data_quality_score));
    }

    match load_candle_csv_timestamp_series(path_ref) {
        Ok(series) => {
            let has_duplicates = series
                .timestamps
                .windows(2)
                .any(|window| window[0] == window[1]);
            let step_ms = detect_min_step(&series.timestamps);
            let has_gaps = has_gap_windows(&series.timestamps, step_ms);
            let source_kind = provenance
                .as_ref()
                .map(|record| record.source_kind)
                .unwrap_or_else(|| downgrade_only_source_kind(path_ref));
            let market = manifest
                .as_ref()
                .map(|record| match record.asset_class {
                    crate::data::AssetClass::Crypto => ProviderMarket::Crypto,
                    crate::data::AssetClass::Equity
                    | crate::data::AssetClass::Stock
                    | crate::data::AssetClass::Etf => infer_market_from_symbol(&series.symbol),
                    crate::data::AssetClass::Spot
                    | crate::data::AssetClass::Forex
                    | crate::data::AssetClass::Futures
                    | crate::data::AssetClass::Unknown => infer_market_from_symbol(&series.symbol),
                })
                .unwrap_or_else(|| infer_market_from_symbol(&series.symbol));
            let source_class = classify_source_class(path_ref, source_kind, market);
            let official_readiness_eligible = source_class
                == OfficialCandleSeriesSourceClass::OfficialNonCrypto
                && provenance_available
                && preflight_ready
                && (!config.require_manifest || manifest_available)
                && !has_duplicates
                && !has_gaps;
            let benchmark_eligible = matches!(
                source_class,
                OfficialCandleSeriesSourceClass::OfficialNonCrypto
                    | OfficialCandleSeriesSourceClass::OfficialCryptoOnly
            ) && provenance_available
                && preflight_ready
                && !has_duplicates
                && !has_gaps;
            let diagnostic_only = matches!(
                source_class,
                OfficialCandleSeriesSourceClass::ControlledDiagnostic
                    | OfficialCandleSeriesSourceClass::YFinanceResearch
                    | OfficialCandleSeriesSourceClass::FixtureArchitectureTest
                    | OfficialCandleSeriesSourceClass::SyntheticTest
                    | OfficialCandleSeriesSourceClass::Unknown
            ) || !benchmark_eligible;
            if !provenance_available {
                reason_codes.push(ReasonCode::MissingOfficialProvenance);
            }
            if !preflight_ready {
                reason_codes.push(ReasonCode::MissingOfficialPreflight);
            }
            if has_duplicates {
                reason_codes.push(ReasonCode::DuplicateTimestampDetected);
            }
            if has_gaps {
                reason_codes.push(ReasonCode::GapDetected);
            }
            OfficialCandleSeriesDescriptor {
                candle_series_id: format!(
                    "{}:{}:{}",
                    series.normalized_symbol,
                    series.timeframe,
                    stable_hash_string(path)
                ),
                path: path.to_string(),
                provider_kind,
                source_kind,
                source_class,
                market,
                venue,
                symbol: series.symbol.clone(),
                normalized_symbol: series.normalized_symbol,
                timeframe: series.timeframe,
                row_count: series.timestamps.len(),
                timestamp_start_ms: *series.timestamps.first().unwrap_or(&0),
                timestamp_end_ms: *series.timestamps.last().unwrap_or(&0),
                has_duplicates,
                has_gaps,
                data_quality_score,
                provenance_available,
                preflight_ready,
                manifest_available,
                timestamp_policy: series.timestamp_policy,
                adjusted_price_policy,
                official_readiness_eligible,
                benchmark_eligible,
                diagnostic_only,
                storage_bytes,
                reason_codes: stable_reason_codes(&reason_codes),
            }
        }
        Err(error) => {
            reason_codes.push(ReasonCode::UnsupportedCsvFormat);
            if !path_ref.exists() {
                reason_codes.push(ReasonCode::MissingFile);
            }
            OfficialCandleSeriesDescriptor {
                candle_series_id: format!("invalid:{}", stable_hash_string(path)),
                path: path.to_string(),
                provider_kind,
                source_kind: downgrade_only_source_kind(path_ref),
                source_class: classify_source_class(
                    path_ref,
                    downgrade_only_source_kind(path_ref),
                    infer_market_from_symbol(&symbol_from_path(path_ref)),
                ),
                market: infer_market_from_symbol(&symbol_from_path(path_ref)),
                venue,
                symbol: symbol_from_path(path_ref),
                normalized_symbol: normalize_symbol(&symbol_from_path(path_ref)),
                timeframe: infer_timeframe_label(path_ref, None, None),
                row_count: 0,
                timestamp_start_ms: 0,
                timestamp_end_ms: 0,
                has_duplicates: false,
                has_gaps: false,
                data_quality_score,
                provenance_available,
                preflight_ready,
                manifest_available,
                timestamp_policy: None,
                adjusted_price_policy,
                official_readiness_eligible: false,
                benchmark_eligible: false,
                diagnostic_only: true,
                storage_bytes,
                reason_codes: stable_reason_codes(
                    &reason_codes
                        .into_iter()
                        .chain([ReasonCode::DataLoadFailed])
                        .collect::<Vec<_>>(),
                ),
            }
            .with_error_note(error)
        }
    }
}

impl OfficialCandleSeriesDescriptor {
    fn with_error_note(mut self, error: String) -> Self {
        if !error.is_empty() {
            self.reason_codes = stable_reason_codes(
                &self
                    .reason_codes
                    .into_iter()
                    .chain([ReasonCode::DataValidationFailed])
                    .collect::<Vec<_>>(),
            );
        }
        self
    }
}

fn descriptor_allowed_by_config(
    descriptor: &OfficialCandleSeriesDescriptor,
    config: &OfficialCandleCoveragePackConfig,
) -> bool {
    match descriptor.source_class {
        OfficialCandleSeriesSourceClass::OfficialNonCrypto => true,
        OfficialCandleSeriesSourceClass::OfficialCryptoOnly => config.allow_crypto_only,
        OfficialCandleSeriesSourceClass::ControlledDiagnostic => {
            !config.require_official_source && config.allow_controlled_fixture
        }
        OfficialCandleSeriesSourceClass::YFinanceResearch => {
            !config.require_official_source && config.allow_yfinance_research
        }
        OfficialCandleSeriesSourceClass::FixtureArchitectureTest => {
            !config.require_official_source && config.allow_fixture
        }
        OfficialCandleSeriesSourceClass::SyntheticTest => {
            !config.require_official_source && config.allow_synthetic_test
        }
        OfficialCandleSeriesSourceClass::Unknown => !config.require_official_source,
    }
}

fn filter_by_class(
    descriptors: &[OfficialCandleSeriesDescriptor],
    class: OfficialCandleSeriesSourceClass,
) -> Vec<OfficialCandleSeriesDescriptor> {
    descriptors
        .iter()
        .filter(|descriptor| descriptor.source_class == class)
        .cloned()
        .collect()
}

fn descriptor_sort_key(
    descriptor: &OfficialCandleSeriesDescriptor,
) -> (
    OfficialCandleSeriesSourceClass,
    ProviderMarket,
    String,
    String,
    String,
) {
    (
        descriptor.source_class,
        descriptor.market,
        descriptor.normalized_symbol.clone(),
        descriptor.timeframe.clone(),
        descriptor.path.clone(),
    )
}

fn build_sidecar_lookup(paths: &[String]) -> BTreeMap<String, Vec<String>> {
    let mut lookup = BTreeMap::new();
    let mut ordered = paths.to_vec();
    ordered.sort();
    for path in ordered {
        lookup
            .entry(sidecar_key(Path::new(&path)))
            .or_insert_with(Vec::new)
            .push(path);
    }
    lookup
}

fn find_sidecar(path: &Path, lookup: &BTreeMap<String, Vec<String>>) -> Option<String> {
    let key = sidecar_key(path);
    lookup
        .get(&key)
        .and_then(|paths| paths.first().cloned())
        .or_else(|| {
            lookup
                .iter()
                .find(|(candidate, _)| candidate.contains(&key) || key.contains(*candidate))
                .and_then(|(_, paths)| paths.first().cloned())
        })
}

fn sidecar_key(path: &Path) -> String {
    path.file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
        .replace("_preflight_report", "")
        .replace("_preflight", "")
        .replace("_provenance", "")
        .replace("_manifest", "")
        .replace("_bundle", "")
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &str) -> Result<T, String> {
    let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
    serde_json::from_str(&text).map_err(|err| err.to_string())
}

fn classify_source_class(
    path: &Path,
    source_kind: EvidenceSourceKind,
    market: ProviderMarket,
) -> OfficialCandleSeriesSourceClass {
    match source_kind {
        EvidenceSourceKind::OfficialApiCollected => {
            if market == ProviderMarket::Crypto {
                OfficialCandleSeriesSourceClass::OfficialCryptoOnly
            } else {
                OfficialCandleSeriesSourceClass::OfficialNonCrypto
            }
        }
        EvidenceSourceKind::YFinanceResearch => OfficialCandleSeriesSourceClass::YFinanceResearch,
        EvidenceSourceKind::SyntheticFixture | EvidenceSourceKind::TestFixture => {
            OfficialCandleSeriesSourceClass::FixtureArchitectureTest
        }
        EvidenceSourceKind::GeneratedSynthetic => OfficialCandleSeriesSourceClass::SyntheticTest,
        EvidenceSourceKind::RealLocal => OfficialCandleSeriesSourceClass::ControlledDiagnostic,
        EvidenceSourceKind::ExternalPredictionOnly | EvidenceSourceKind::Unknown => {
            let lowered = path.display().to_string().to_ascii_lowercase();
            if lowered.contains("yfinance") || lowered.contains("yahoo") {
                OfficialCandleSeriesSourceClass::YFinanceResearch
            } else if lowered.contains("synthetic") {
                OfficialCandleSeriesSourceClass::SyntheticTest
            } else if lowered.contains("fixture") || lowered.contains("mock") {
                OfficialCandleSeriesSourceClass::FixtureArchitectureTest
            } else if lowered.contains("controlled") {
                OfficialCandleSeriesSourceClass::ControlledDiagnostic
            } else {
                OfficialCandleSeriesSourceClass::Unknown
            }
        }
    }
}

fn downgrade_only_source_kind(path: &Path) -> EvidenceSourceKind {
    match infer_source_kind_from_path(Some(path)) {
        EvidenceSourceKind::YFinanceResearch => EvidenceSourceKind::YFinanceResearch,
        EvidenceSourceKind::SyntheticFixture => EvidenceSourceKind::SyntheticFixture,
        EvidenceSourceKind::TestFixture => EvidenceSourceKind::TestFixture,
        EvidenceSourceKind::GeneratedSynthetic => EvidenceSourceKind::GeneratedSynthetic,
        _ => EvidenceSourceKind::Unknown,
    }
}

fn detect_min_step(timestamps: &[u64]) -> Option<u64> {
    timestamps
        .windows(2)
        .filter_map(|window| {
            let step = window[1].saturating_sub(window[0]);
            (step > 0).then_some(step)
        })
        .min()
}

fn has_gap_windows(timestamps: &[u64], step_ms: Option<u64>) -> bool {
    let Some(step_ms) = step_ms else {
        return false;
    };
    timestamps
        .windows(2)
        .any(|window| window[1].saturating_sub(window[0]) > step_ms)
}

fn symbol_from_path(path: &Path) -> String {
    path.file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("UNKNOWN")
        .split(['_', '-', '.'])
        .find(|value| !value.is_empty() && !value.chars().all(|ch| ch.is_ascii_digit()))
        .unwrap_or("UNKNOWN")
        .to_string()
}

fn infer_timeframe_label(path: &Path, explicit: Option<&str>, step_ms: Option<u64>) -> String {
    if let Some(value) = explicit {
        return normalize_timeframe_label(value);
    }
    let lowered = path.display().to_string().to_ascii_lowercase();
    for (needle, value) in [
        ("15m", "15m"),
        ("5m", "5m"),
        ("1m", "1m"),
        ("1h", "1h"),
        ("1d", "1d"),
        ("oneday", "1d"),
        ("onehour", "1h"),
    ] {
        if lowered.contains(needle) {
            return value.to_string();
        }
    }
    match step_ms {
        Some(60_000) => "1m".to_string(),
        Some(300_000) => "5m".to_string(),
        Some(900_000) => "15m".to_string(),
        Some(3_600_000) => "1h".to_string(),
        Some(86_400_000) => "1d".to_string(),
        _ => "unknown".to_string(),
    }
}

fn normalize_timestamp(value: u64) -> u64 {
    if value < 1_000_000_000_000 {
        value.saturating_mul(1000)
    } else {
        value
    }
}

fn parse_provider_kind(value: &str) -> Option<ProviderKind> {
    match value.trim().to_ascii_lowercase().as_str() {
        "upbit" => Some(ProviderKind::Upbit),
        "binance" => Some(ProviderKind::Binance),
        "krxopenapi" | "krx_open_api" | "krx" => Some(ProviderKind::KrxOpenApi),
        "alphavantage" | "alpha_vantage" => Some(ProviderKind::AlphaVantage),
        "alpaca" => Some(ProviderKind::Alpaca),
        "mockfixture" | "mock_fixture" => Some(ProviderKind::MockFixture),
        _ => None,
    }
}

fn is_remote_path(value: &str) -> bool {
    value.contains("://")
}

fn default_output_root() -> String {
    "target/soma_official_candle_coverage_pack".to_string()
}

fn default_max_rows() -> usize {
    1_000
}

fn default_max_symbols() -> usize {
    5
}

fn default_max_timeframes() -> usize {
    5
}

fn default_max_bytes() -> usize {
    5_000_000
}

fn default_timestamp_tolerance_ms() -> u64 {
    60_000
}

fn default_true() -> bool {
    true
}
