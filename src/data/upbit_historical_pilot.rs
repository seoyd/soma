//! Explicit, local-only Upbit daily-candle pilot for the acquisition broker.
//!
//! The public quotation endpoint is used only after a local configuration and
//! an explicit command-line network flag agree. This module has no account,
//! order, streaming, or background-polling surface.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File},
    io::Write,
    path::{Component, Path, PathBuf},
    process::Command,
};

use prost::Message;
use serde::{Deserialize, Serialize};

use crate::{
    core::{ReasonCode, stable_hash_string},
    league::{HistoricalOhlcvRow, HistoricalReplayDataset},
};

use super::acquisition::AcquisitionRequest;
use super::{
    AcquisitionMarketScope, AcquisitionMode, AcquisitionPlan, AcquisitionPolicy,
    DataAcquisitionBroker, DataLookback, DataSnapshot, DatasetKind, ProviderCapabilities,
    ProviderFetchFailure, ReadOnlyMarketDataProvider, ReadOnlyProviderRegistry,
    ReadOnlyProviderRequest, ReadOnlyProviderResponse, SnapshotProvenance, SnapshotQualitySummary,
    SnapshotSourceType,
};

const UPBIT_PROVIDER_ID: &str = "upbit";
const UPBIT_DAILY_CANDLES_ENDPOINT: &str = "https://api.upbit.com/v1/candles/days";
const UPBIT_MAX_CANDLES_PER_REQUEST: usize = 200;
const DEFAULT_SNAPSHOT_OUTPUT_DIR: &str = "data/local_snapshots/upbit";
const SNAPSHOT_PROTOBUF_MAGIC_V1: &[u8] = b"SOMA-SNAPSHOT-PB-V1";
const SNAPSHOT_PROTOBUF_SCHEMA_V1: &str = "soma.data_snapshot.v1";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SnapshotStorageFormat {
    JsonLegacyV0,
    ProtobufV1,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SnapshotCodec {
    pub format: SnapshotStorageFormat,
}

impl SnapshotCodec {
    pub const fn protobuf_v1() -> Self {
        Self {
            format: SnapshotStorageFormat::ProtobufV1,
        }
    }

    pub const fn json_legacy_v0() -> Self {
        Self {
            format: SnapshotStorageFormat::JsonLegacyV0,
        }
    }

    pub fn encode(&self, snapshot: &DataSnapshot) -> Result<Vec<u8>, String> {
        match self.format {
            SnapshotStorageFormat::JsonLegacyV0 => serde_json::to_vec(snapshot)
                .map_err(|_| "legacy snapshot serialization failed".to_string()),
            SnapshotStorageFormat::ProtobufV1 => encode_snapshot_protobuf_v1(snapshot),
        }
    }

    pub fn decode(&self, bytes: &[u8]) -> Result<DataSnapshot, String> {
        match self.format {
            SnapshotStorageFormat::JsonLegacyV0 => serde_json::from_slice(bytes)
                .map_err(|_| "legacy snapshot decode failed".to_string()),
            SnapshotStorageFormat::ProtobufV1 => decode_snapshot_protobuf_v1(bytes),
        }
    }
}

#[derive(Clone, PartialEq, Message)]
struct SnapshotEnvelopeProtobufV1 {
    #[prost(bytes = "vec", tag = "1")]
    magic: Vec<u8>,
    #[prost(uint32, tag = "2")]
    version: u32,
    #[prost(string, tag = "3")]
    schema: String,
    #[prost(string, tag = "4")]
    semantic_digest: String,
    #[prost(string, tag = "5")]
    snapshot_id: String,
    #[prost(uint64, tag = "6")]
    payload_length: u64,
    #[prost(string, tag = "7")]
    payload_digest: String,
    #[prost(bytes = "vec", tag = "8")]
    payload: Vec<u8>,
}

#[derive(Clone, PartialEq, Message)]
struct SnapshotPayloadProtobufV1 {
    #[prost(string, tag = "1")]
    snapshot_id: String,
    #[prost(string, tag = "2")]
    request_key: String,
    #[prost(string, tag = "3")]
    provider_id: String,
    #[prost(uint32, tag = "4")]
    dataset_kind: u32,
    #[prost(uint32, tag = "5")]
    market_scope: u32,
    #[prost(string, repeated, tag = "6")]
    symbols: Vec<String>,
    #[prost(message, optional, tag = "7")]
    requested_lookback: Option<LookbackProtobufV1>,
    #[prost(uint64, optional, tag = "8")]
    actual_start_timestamp_ms: Option<u64>,
    #[prost(uint64, optional, tag = "9")]
    actual_end_timestamp_ms: Option<u64>,
    #[prost(uint64, tag = "10")]
    fetched_at_ms: u64,
    #[prost(uint64, tag = "11")]
    normalized_at_ms: u64,
    #[prost(uint32, tag = "12")]
    schema_version: u32,
    #[prost(uint64, tag = "13")]
    row_count: u64,
    #[prost(message, optional, tag = "14")]
    quality_summary: Option<QualityProtobufV1>,
    #[prost(string, tag = "15")]
    content_digest: String,
    #[prost(bool, tag = "16")]
    sanitized: bool,
    #[prost(bool, tag = "17")]
    read_only: bool,
    #[prost(message, optional, tag = "18")]
    normalized_dataset: Option<DatasetProtobufV1>,
    #[prost(message, optional, tag = "19")]
    provenance: Option<ProvenanceProtobufV1>,
    #[prost(string, repeated, tag = "20")]
    reason_codes: Vec<String>,
}

#[derive(Clone, PartialEq, Message)]
struct LookbackProtobufV1 {
    #[prost(uint64, tag = "1")]
    bars: u64,
    #[prost(uint64, optional, tag = "2")]
    start_timestamp_ms: Option<u64>,
    #[prost(uint64, optional, tag = "3")]
    end_timestamp_ms: Option<u64>,
}

#[derive(Clone, PartialEq, Message)]
struct QualityProtobufV1 {
    #[prost(bool, tag = "1")]
    accepted: bool,
    #[prost(uint64, tag = "2")]
    row_count: u64,
    #[prost(string, repeated, tag = "3")]
    reason_codes: Vec<String>,
}

#[derive(Clone, PartialEq, Message)]
struct DatasetProtobufV1 {
    #[prost(string, tag = "1")]
    symbol: String,
    #[prost(message, repeated, tag = "2")]
    rows: Vec<OhlcvProtobufV1>,
    #[prost(string, tag = "3")]
    source: String,
    #[prost(string, repeated, tag = "4")]
    reason_codes: Vec<String>,
}

#[derive(Clone, PartialEq, Message)]
struct OhlcvProtobufV1 {
    #[prost(string, tag = "1")]
    symbol: String,
    #[prost(uint64, tag = "2")]
    timestamp_ms: u64,
    #[prost(fixed64, tag = "3")]
    open_bits: u64,
    #[prost(fixed64, tag = "4")]
    high_bits: u64,
    #[prost(fixed64, tag = "5")]
    low_bits: u64,
    #[prost(fixed64, tag = "6")]
    close_bits: u64,
    #[prost(fixed64, tag = "7")]
    volume_bits: u64,
    #[prost(fixed64, optional, tag = "8")]
    trade_value_bits: Option<u64>,
}

#[derive(Clone, PartialEq, Message)]
struct ProvenanceProtobufV1 {
    #[prost(string, tag = "1")]
    provider_id: String,
    #[prost(string, tag = "2")]
    acquisition_request_id: String,
    #[prost(string, tag = "3")]
    fetch_receipt_id: String,
    #[prost(uint32, tag = "4")]
    source_type: u32,
    #[prost(bool, tag = "5")]
    sanitized: bool,
    #[prost(bool, tag = "6")]
    credential_free: bool,
    #[prost(string, repeated, tag = "7")]
    reason_codes: Vec<String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HistoricalProviderQualificationStatusV0 {
    Qualified,
    Disabled,
    MissingOfficialContract,
    MissingHistoricalCapability,
    UnsupportedMarket,
    UnsafeCapabilitySurface,
    RequiresGuessedMapping,
    ConfigurationMissing,
    #[default]
    Rejected,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoricalProviderQualificationV0 {
    pub provider_id: String,
    pub status: HistoricalProviderQualificationStatusV0,
    pub supports_daily_ohlcv: bool,
    pub supported_markets: Vec<AcquisitionMarketScope>,
    pub requires_credentials: bool,
    pub read_only: bool,
    pub network_approved: bool,
    pub response_schema_known: bool,
    pub timestamp_semantics_known: bool,
    pub pagination_semantics_known: bool,
    pub reason_codes: Vec<String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NetworkConsentV0 {
    #[default]
    Denied,
    ManualLocalSmoke,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HistoricalProviderSelectionStatusV0 {
    Selected,
    NetworkConsentRequired,
    ConfigurationMissing,
    #[default]
    NoQualifiedProvider,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoricalProviderSelectionV0 {
    pub requested_market: AcquisitionMarketScope,
    pub selected_provider: Option<String>,
    pub qualification: Option<HistoricalProviderQualificationV0>,
    pub rejected_candidates: Vec<String>,
    pub status: HistoricalProviderSelectionStatusV0,
    pub reason_codes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpbitHistoricalPilotConfigV0 {
    pub provider_id: String,
    pub enabled: bool,
    pub market: AcquisitionMarketScope,
    pub symbol: String,
    pub start_timestamp_ms: u64,
    pub end_timestamp_ms: u64,
    pub maximum_rows: usize,
    pub timeout_seconds: u64,
    pub max_retries: usize,
    pub maximum_response_bytes: usize,
    pub snapshot_output_dir: String,
    pub network_consent: NetworkConsentV0,
    pub manual_smoke_enabled: bool,
    #[serde(default = "default_page_size")]
    pub page_size: usize,
    #[serde(default = "default_target_rows")]
    pub target_rows: usize,
    #[serde(default = "default_maximum_pages")]
    pub maximum_pages: usize,
    #[serde(default)]
    pub stop_when_campaign_sufficient: bool,
    #[serde(default)]
    pub campaign_attempt_enabled: bool,
}

impl UpbitHistoricalPilotConfigV0 {
    pub fn from_toml_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path)
            .map_err(|_| "local provider config unavailable".to_string())?;
        toml::from_str(&text).map_err(|_| "local provider config is invalid".to_string())
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.provider_id != UPBIT_PROVIDER_ID
            || self.market != AcquisitionMarketScope::BtcCrypto
            || !valid_market_symbol(&self.symbol)
            || self.start_timestamp_ms >= self.end_timestamp_ms
            || self.maximum_rows == 0
            || self.maximum_rows > UPBIT_MAX_CANDLES_PER_REQUEST
            || self.page_size == 0
            || self.page_size > UPBIT_MAX_CANDLES_PER_REQUEST
            || self.target_rows == 0
            || self.maximum_pages == 0
            || self
                .page_size
                .checked_mul(self.maximum_pages)
                .is_none_or(|capacity| self.target_rows > capacity)
            || self.timeout_seconds == 0
            || self.maximum_response_bytes == 0
            || !safe_snapshot_output_dir(Path::new(&self.snapshot_output_dir))
        {
            return Err("local provider config is invalid".to_string());
        }
        Ok(())
    }
}

fn default_page_size() -> usize {
    UPBIT_MAX_CANDLES_PER_REQUEST
}

fn default_target_rows() -> usize {
    128
}

fn default_maximum_pages() -> usize {
    1
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UpbitHistoricalPreflightStatusV0 {
    Ready,
    #[default]
    ConfigurationMissing,
    NetworkConsentRequired,
    InvalidConfiguration,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpbitHistoricalPreflightV0 {
    pub status: UpbitHistoricalPreflightStatusV0,
    pub provider_id: Option<String>,
    pub reason_codes: Vec<String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UpbitHistoricalBackfillStatusV0 {
    RealUpbitSnapshotHarvested,
    RealUpbitBackfillSnapshotHarvested,
    NetworkConsentRequired,
    ConfigurationMissing,
    RealSmokeExecutionBlocked,
    RealSmokeFailed,
    MaximumPagesReachedInsufficient,
    StartBoundaryReached,
    EmptyPageReachedInsufficient,
    CursorStalled,
    ValidationFailure,
    #[default]
    NotAttempted,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpbitHistoricalPageReceiptV0 {
    pub request_id: String,
    pub receipt_id: String,
    pub end_exclusive_timestamp_ms: u64,
    pub row_count: usize,
    pub attempt_count: usize,
    pub snapshot_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpbitHistoricalBackfillResultV0 {
    pub status: UpbitHistoricalBackfillStatusV0,
    pub provider_id: Option<String>,
    pub symbol: Option<String>,
    pub requested_start_timestamp_ms: Option<u64>,
    pub requested_end_timestamp_ms: Option<u64>,
    pub actual_start_timestamp_ms: Option<u64>,
    pub actual_end_timestamp_ms: Option<u64>,
    pub row_count: usize,
    pub page_receipts: Vec<UpbitHistoricalPageReceiptV0>,
    pub snapshot_id: Option<String>,
    pub snapshot_digest: Option<String>,
    pub local_snapshot_path: Option<String>,
    pub reason_codes: Vec<String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FirstHistoricalHarvestStatusV0 {
    RealHistoricalSnapshotHarvested,
    ApprovedProviderReadySmokeNotRun,
    ApprovedProviderSmokeFailed,
    NoQualifyingProviderContract,
    NetworkConsentRequired,
    ConfigurationMissing,
    #[default]
    SnapshotValidationFailed,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FirstHistoricalHarvestResultV0 {
    pub status: FirstHistoricalHarvestStatusV0,
    pub provider_id: Option<String>,
    pub market: Option<AcquisitionMarketScope>,
    pub symbol: Option<String>,
    pub requested_start_timestamp_ms: Option<u64>,
    pub requested_end_timestamp_ms: Option<u64>,
    pub actual_start_timestamp_ms: Option<u64>,
    pub actual_end_timestamp_ms: Option<u64>,
    pub row_count: usize,
    pub snapshot_id: Option<String>,
    pub snapshot_digest: Option<String>,
    pub local_snapshot_path: Option<String>,
    pub reason_codes: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
struct UpbitDailyCandleV0 {
    market: String,
    candle_date_time_utc: String,
    opening_price: f64,
    high_price: f64,
    low_price: f64,
    trade_price: f64,
    candle_acc_trade_volume: f64,
    #[serde(default)]
    candle_acc_trade_price: Option<f64>,
}

pub fn qualify_upbit_historical_provider_v0(
    config: Option<&UpbitHistoricalPilotConfigV0>,
) -> HistoricalProviderQualificationV0 {
    let configured = config.is_some_and(|value| value.validate().is_ok());
    let enabled = config.is_some_and(|value| value.enabled);
    let status = if config.is_none() || !configured {
        HistoricalProviderQualificationStatusV0::ConfigurationMissing
    } else if !enabled {
        HistoricalProviderQualificationStatusV0::Disabled
    } else {
        HistoricalProviderQualificationStatusV0::Qualified
    };
    HistoricalProviderQualificationV0 {
        provider_id: UPBIT_PROVIDER_ID.to_string(),
        status,
        supports_daily_ohlcv: true,
        supported_markets: vec![AcquisitionMarketScope::BtcCrypto],
        requires_credentials: false,
        read_only: true,
        network_approved: true,
        response_schema_known: true,
        timestamp_semantics_known: true,
        pagination_semantics_known: true,
        reason_codes: match status {
            HistoricalProviderQualificationStatusV0::Qualified => {
                vec!["official_upbit_daily_candle_contract".to_string()]
            }
            HistoricalProviderQualificationStatusV0::Disabled => {
                vec!["provider_disabled_by_local_config".to_string()]
            }
            HistoricalProviderQualificationStatusV0::ConfigurationMissing => {
                vec!["provider_configuration_missing_or_invalid".to_string()]
            }
            _ => vec!["provider_not_qualified".to_string()],
        },
    }
}

pub fn select_upbit_historical_provider_v0(
    config: Option<&UpbitHistoricalPilotConfigV0>,
    allow_network: bool,
) -> HistoricalProviderSelectionV0 {
    let qualification = qualify_upbit_historical_provider_v0(config);
    let requested_market = config
        .map(|value| value.market)
        .unwrap_or(AcquisitionMarketScope::Unknown);
    if qualification.status != HistoricalProviderQualificationStatusV0::Qualified {
        return HistoricalProviderSelectionV0 {
            requested_market,
            selected_provider: None,
            qualification: Some(qualification),
            rejected_candidates: vec![UPBIT_PROVIDER_ID.to_string()],
            status: if config.is_none() {
                HistoricalProviderSelectionStatusV0::ConfigurationMissing
            } else {
                HistoricalProviderSelectionStatusV0::NoQualifiedProvider
            },
            reason_codes: vec!["no_qualified_provider_selected".to_string()],
        };
    }
    let consent = config.is_some_and(|value| {
        value.network_consent == NetworkConsentV0::ManualLocalSmoke
            && value.manual_smoke_enabled
            && allow_network
    });
    if !consent {
        return HistoricalProviderSelectionV0 {
            requested_market,
            selected_provider: None,
            qualification: Some(qualification),
            rejected_candidates: vec![],
            status: HistoricalProviderSelectionStatusV0::NetworkConsentRequired,
            reason_codes: vec!["explicit_manual_network_consent_required".to_string()],
        };
    }
    HistoricalProviderSelectionV0 {
        requested_market,
        selected_provider: Some(UPBIT_PROVIDER_ID.to_string()),
        qualification: Some(qualification),
        rejected_candidates: vec![],
        status: HistoricalProviderSelectionStatusV0::Selected,
        reason_codes: vec!["single_readonly_provider_selected".to_string()],
    }
}

pub fn preflight_upbit_historical_backfill_v0(
    config_path: &Path,
    allow_network: bool,
) -> UpbitHistoricalPreflightV0 {
    let config = match UpbitHistoricalPilotConfigV0::from_toml_path(config_path) {
        Ok(config) => config,
        Err(_) => {
            return UpbitHistoricalPreflightV0 {
                status: UpbitHistoricalPreflightStatusV0::ConfigurationMissing,
                provider_id: None,
                reason_codes: vec!["copy_and_enable_the_ignored_local_upbit_config".to_string()],
            };
        }
    };
    if config.validate().is_err() {
        return UpbitHistoricalPreflightV0 {
            status: UpbitHistoricalPreflightStatusV0::InvalidConfiguration,
            provider_id: Some(config.provider_id),
            reason_codes: vec!["local_provider_configuration_invalid".to_string()],
        };
    }
    let selection = select_upbit_historical_provider_v0(Some(&config), allow_network);
    let status = match selection.status {
        HistoricalProviderSelectionStatusV0::Selected => UpbitHistoricalPreflightStatusV0::Ready,
        HistoricalProviderSelectionStatusV0::NetworkConsentRequired => {
            UpbitHistoricalPreflightStatusV0::NetworkConsentRequired
        }
        _ => UpbitHistoricalPreflightStatusV0::InvalidConfiguration,
    };
    UpbitHistoricalPreflightV0 {
        status,
        provider_id: Some(config.provider_id),
        reason_codes: selection.reason_codes,
    }
}

pub struct UpbitDailyOhlcvProviderV0 {
    config: UpbitHistoricalPilotConfigV0,
}

impl UpbitDailyOhlcvProviderV0 {
    pub fn new(config: UpbitHistoricalPilotConfigV0) -> Self {
        Self { config }
    }
}

impl ReadOnlyMarketDataProvider for UpbitDailyOhlcvProviderV0 {
    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            provider_id: UPBIT_PROVIDER_ID.to_string(),
            supported_markets: vec![AcquisitionMarketScope::BtcCrypto],
            supported_dataset_kinds: vec![DatasetKind::DailyOhlcv],
            supported_cadences: vec!["1d".to_string()],
            maximum_lookback_bars: UPBIT_MAX_CANDLES_PER_REQUEST,
            requires_credentials: false,
            read_only: true,
            enabled: self.config.enabled,
            approved_for_network: true,
            mock_only: false,
            reason_codes: vec![],
        }
    }

    fn fetch_readonly(
        &mut self,
        request: &ReadOnlyProviderRequest,
    ) -> Result<ReadOnlyProviderResponse, ProviderFetchFailure> {
        if request.provider_id != UPBIT_PROVIDER_ID
            || request.market_scope != AcquisitionMarketScope::BtcCrypto
            || request.dataset_kind != DatasetKind::DailyOhlcv
            || request.cadence != "1d"
            || request.symbols.as_slice() != [self.config.symbol.clone()]
            || request.lookback.bars == 0
            || request.lookback.bars > UPBIT_MAX_CANDLES_PER_REQUEST
        {
            return Err(ProviderFetchFailure::InvalidResponse);
        }
        let end = request
            .lookback
            .end_timestamp_ms
            .ok_or(ProviderFetchFailure::InvalidResponse)?;
        let url = upbit_daily_candles_url(&self.config.symbol, end, request.lookback.bars)
            .ok_or(ProviderFetchFailure::InvalidResponse)?;
        let output = Command::new("curl")
            .args([
                "--silent",
                "--show-error",
                "--fail",
                "--proto",
                "=https",
                "--proto-redir",
                "=https",
                "--connect-timeout",
                &self.config.timeout_seconds.to_string(),
                "--max-time",
                &self.config.timeout_seconds.to_string(),
                "--max-filesize",
                &self.config.maximum_response_bytes.to_string(),
                "--request",
                "GET",
                "--header",
                "accept: application/json",
                &url,
            ])
            .output()
            .map_err(|_| ProviderFetchFailure::Unavailable)?;
        if !output.status.success() || output.stdout.len() > self.config.maximum_response_bytes {
            return Err(ProviderFetchFailure::Unavailable);
        }
        let body =
            String::from_utf8(output.stdout).map_err(|_| ProviderFetchFailure::InvalidResponse)?;
        let dataset = parse_upbit_daily_ohlcv_v0(&body, &self.config.symbol)
            .map_err(|_| ProviderFetchFailure::InvalidResponse)?;
        if dataset
            .rows
            .iter()
            .any(|row| row.timestamp_ms < self.config.start_timestamp_ms || row.timestamp_ms >= end)
        {
            return Err(ProviderFetchFailure::InvalidResponse);
        }
        Ok(ReadOnlyProviderResponse {
            request_id: request.request_id.clone(),
            provider_id: UPBIT_PROVIDER_ID.to_string(),
            fetched_at_ms: current_time_ms(),
            content_type: "application/x-soma-normalized-dataset".to_string(),
            reported_content_bytes: body.len(),
            normalized_dataset: dataset,
            reason_codes: vec![],
        })
    }
}

pub fn run_manual_upbit_historical_smoke_v0(
    config_path: &Path,
    allow_network: bool,
) -> FirstHistoricalHarvestResultV0 {
    let config = match UpbitHistoricalPilotConfigV0::from_toml_path(config_path) {
        Ok(config) if config.validate().is_ok() => config,
        _ => {
            return harvest_result(
                FirstHistoricalHarvestStatusV0::ConfigurationMissing,
                None,
                vec!["local_provider_configuration_missing_or_invalid".to_string()],
            );
        }
    };
    let selection = select_upbit_historical_provider_v0(Some(&config), allow_network);
    if selection.status != HistoricalProviderSelectionStatusV0::Selected {
        let status = match selection.status {
            HistoricalProviderSelectionStatusV0::NetworkConsentRequired => {
                FirstHistoricalHarvestStatusV0::NetworkConsentRequired
            }
            HistoricalProviderSelectionStatusV0::ConfigurationMissing => {
                FirstHistoricalHarvestStatusV0::ConfigurationMissing
            }
            _ => FirstHistoricalHarvestStatusV0::NoQualifyingProviderContract,
        };
        return harvest_result(status, Some(&config), selection.reason_codes);
    }
    let capabilities = UpbitDailyOhlcvProviderV0::new(config.clone()).capabilities();
    let mut registry = ReadOnlyProviderRegistry::default();
    registry.register(capabilities);
    let mut policy = AcquisitionPolicy::default();
    policy.allow_approved_readonly_network = true;
    policy.max_response_bytes = config.maximum_response_bytes;
    policy.max_retries = config.max_retries;
    policy.max_requests_per_provider = 1;
    let mut broker = DataAcquisitionBroker::new(registry, policy);
    let request = ReadOnlyProviderRequest {
        request_id: format!(
            "upbit-smoke-{}",
            stable_hash_string(&format!("{}:{}", config.symbol, config.end_timestamp_ms))
        ),
        request_key: format!(
            "upbit-daily:{}:{}:{}",
            config.symbol, config.start_timestamp_ms, config.end_timestamp_ms
        ),
        provider_id: UPBIT_PROVIDER_ID.to_string(),
        dataset_kind: DatasetKind::DailyOhlcv,
        market_scope: AcquisitionMarketScope::BtcCrypto,
        symbols: vec![config.symbol.clone()],
        lookback: DataLookback {
            bars: config.maximum_rows,
            start_timestamp_ms: Some(config.start_timestamp_ms),
            end_timestamp_ms: Some(config.end_timestamp_ms),
        },
        cadence: "1d".to_string(),
        max_staleness_ms: u64::MAX,
        reason_codes: vec![],
    };
    let plan = AcquisitionPlan {
        planned_requests: vec![AcquisitionRequest {
            request,
            requested_by_agents: vec![],
            required_by_agents: vec![],
        }],
        rejected_requests: vec![],
        agent_request_mapping: Default::default(),
        deduplicated_request_count: 0,
        reason_codes: vec![],
    };
    let mut provider = UpbitDailyOhlcvProviderV0::new(config.clone());
    let execution = broker.execute_acquisition_plan(
        &plan,
        AcquisitionMode::ApprovedReadOnlyNetwork,
        current_time_ms(),
        Some(&mut provider),
    );
    let Some(snapshot) = execution.new_snapshots.into_iter().next() else {
        return harvest_result(
            FirstHistoricalHarvestStatusV0::ApprovedProviderSmokeFailed,
            Some(&config),
            execution
                .reason_codes
                .iter()
                .map(|code| format!("{code:?}"))
                .collect(),
        );
    };
    match write_and_verify_local_snapshot_v0(&snapshot, Path::new(&config.snapshot_output_dir)) {
        Ok(path) => FirstHistoricalHarvestResultV0 {
            status: FirstHistoricalHarvestStatusV0::RealHistoricalSnapshotHarvested,
            provider_id: Some(UPBIT_PROVIDER_ID.to_string()),
            market: Some(AcquisitionMarketScope::BtcCrypto),
            symbol: Some(config.symbol),
            requested_start_timestamp_ms: Some(config.start_timestamp_ms),
            requested_end_timestamp_ms: Some(config.end_timestamp_ms),
            actual_start_timestamp_ms: snapshot.actual_start_timestamp_ms,
            actual_end_timestamp_ms: snapshot.actual_end_timestamp_ms,
            row_count: snapshot.row_count,
            snapshot_id: Some(snapshot.snapshot_id.clone()),
            snapshot_digest: Some(snapshot.content_digest.clone()),
            local_snapshot_path: Some(path.display().to_string()),
            reason_codes: vec!["manual_readonly_smoke_snapshot_verified".to_string()],
        },
        Err(reason) => harvest_result(
            FirstHistoricalHarvestStatusV0::SnapshotValidationFailed,
            Some(&config),
            vec![reason],
        ),
    }
}

pub fn run_manual_upbit_historical_backfill_v0(
    config_path: &Path,
    allow_network: bool,
    campaign_required_rows: usize,
) -> UpbitHistoricalBackfillResultV0 {
    let config = match UpbitHistoricalPilotConfigV0::from_toml_path(config_path) {
        Ok(config) if config.validate().is_ok() => config,
        _ => {
            return backfill_result(
                UpbitHistoricalBackfillStatusV0::ConfigurationMissing,
                None,
                vec!["local_provider_configuration_missing_or_invalid".to_string()],
            );
        }
    };
    let preflight = preflight_upbit_historical_backfill_v0(config_path, allow_network);
    if preflight.status != UpbitHistoricalPreflightStatusV0::Ready {
        let status = if preflight.status == UpbitHistoricalPreflightStatusV0::NetworkConsentRequired
        {
            UpbitHistoricalBackfillStatusV0::NetworkConsentRequired
        } else {
            UpbitHistoricalBackfillStatusV0::RealSmokeExecutionBlocked
        };
        return backfill_result(status, Some(&config), preflight.reason_codes);
    }
    if campaign_required_rows == 0 {
        return backfill_result(
            UpbitHistoricalBackfillStatusV0::ValidationFailure,
            Some(&config),
            vec!["campaign_required_rows_invalid".to_string()],
        );
    }

    let (first_snapshot, first_receipt) =
        match acquire_upbit_page_v0(&config, config.end_timestamp_ms, config.page_size) {
            Ok(value) => value,
            Err(reason) => {
                return backfill_result(
                    UpbitHistoricalBackfillStatusV0::RealSmokeFailed,
                    Some(&config),
                    vec![reason],
                );
            }
        };
    let first_path = match write_and_verify_local_snapshot_v0(
        &first_snapshot,
        Path::new(&config.snapshot_output_dir),
    ) {
        Ok(path) => path,
        Err(reason) => {
            return backfill_result(
                UpbitHistoricalBackfillStatusV0::RealSmokeFailed,
                Some(&config),
                vec![reason],
            );
        }
    };
    let mut page_receipts = vec![first_receipt];
    let mut pages = vec![first_snapshot.normalized_dataset.clone()];
    let mut cursors = BTreeSet::from([config.end_timestamp_ms]);
    let mut page_digests = BTreeSet::from([dataset_digest(&pages[0])]);
    let mut merged = match merge_upbit_historical_pages_v0(&pages, &config.symbol) {
        Ok((dataset, _)) => dataset,
        Err(reason) => {
            return backfill_result(
                UpbitHistoricalBackfillStatusV0::ValidationFailure,
                Some(&config),
                vec![reason],
            );
        }
    };
    let required_rows = if config.stop_when_campaign_sufficient {
        campaign_required_rows.min(config.target_rows)
    } else {
        config.target_rows
    };
    if merged.rows.len() >= required_rows {
        return completed_backfill_result(
            UpbitHistoricalBackfillStatusV0::RealUpbitSnapshotHarvested,
            &config,
            &first_snapshot,
            first_path,
            page_receipts,
            vec!["single_page_target_reached".to_string()],
        );
    }

    let mut stop_status = UpbitHistoricalBackfillStatusV0::MaximumPagesReachedInsufficient;
    while page_receipts.len() < config.maximum_pages {
        let oldest = merged
            .rows
            .first()
            .map(|row| row.timestamp_ms)
            .unwrap_or_default();
        let page_span_ms = (config.page_size as u64).saturating_mul(86_400_000);
        if oldest <= config.start_timestamp_ms.saturating_add(page_span_ms) {
            stop_status = UpbitHistoricalBackfillStatusV0::StartBoundaryReached;
            break;
        }
        if !cursors.insert(oldest) {
            stop_status = UpbitHistoricalBackfillStatusV0::CursorStalled;
            break;
        }
        let (snapshot, receipt) = match acquire_upbit_page_v0(&config, oldest, config.page_size) {
            Ok(value) => value,
            Err(reason) => {
                return partial_backfill_result(
                    UpbitHistoricalBackfillStatusV0::RealSmokeFailed,
                    &config,
                    &merged,
                    page_receipts,
                    vec![reason],
                );
            }
        };
        let digest = dataset_digest(&snapshot.normalized_dataset);
        if !page_digests.insert(digest) {
            return partial_backfill_result(
                UpbitHistoricalBackfillStatusV0::CursorStalled,
                &config,
                &merged,
                page_receipts,
                vec!["repeated_page_digest".to_string()],
            );
        }
        if snapshot
            .normalized_dataset
            .rows
            .iter()
            .all(|row| row.timestamp_ms >= oldest)
        {
            return partial_backfill_result(
                UpbitHistoricalBackfillStatusV0::CursorStalled,
                &config,
                &merged,
                page_receipts,
                vec!["backfill_cursor_did_not_advance".to_string()],
            );
        }
        page_receipts.push(receipt);
        pages.push(snapshot.normalized_dataset);
        merged = match merge_upbit_historical_pages_v0(&pages, &config.symbol) {
            Ok((dataset, _)) => dataset,
            Err(reason) => {
                return partial_backfill_result(
                    UpbitHistoricalBackfillStatusV0::ValidationFailure,
                    &config,
                    &merged,
                    page_receipts,
                    vec![reason],
                );
            }
        };
        if merged.rows.len() >= required_rows {
            let snapshot = merged_snapshot_v0(&first_snapshot, &config, merged, &page_receipts);
            return match write_and_verify_local_snapshot_v0(
                &snapshot,
                Path::new(&config.snapshot_output_dir),
            ) {
                Ok(path) => completed_backfill_result(
                    UpbitHistoricalBackfillStatusV0::RealUpbitBackfillSnapshotHarvested,
                    &config,
                    &snapshot,
                    path,
                    page_receipts,
                    vec!["bounded_backfill_target_reached".to_string()],
                ),
                Err(reason) => backfill_result(
                    UpbitHistoricalBackfillStatusV0::RealSmokeFailed,
                    Some(&config),
                    vec![reason],
                ),
            };
        }
    }
    partial_backfill_result(
        stop_status,
        &config,
        &merged,
        page_receipts,
        vec!["bounded_backfill_stopped_before_target".to_string()],
    )
}

pub fn merge_upbit_historical_pages_v0(
    pages: &[HistoricalReplayDataset],
    expected_symbol: &str,
) -> Result<(HistoricalReplayDataset, usize), String> {
    let mut rows = BTreeMap::<u64, HistoricalOhlcvRow>::new();
    let mut duplicates = 0;
    for page in pages {
        if page.symbol != expected_symbol {
            return Err("upbit page symbol mismatch".to_string());
        }
        for row in &page.rows {
            if row.symbol != expected_symbol {
                return Err("upbit page row symbol mismatch".to_string());
            }
            match rows.get(&row.timestamp_ms) {
                Some(existing) if existing == row => duplicates += 1,
                Some(_) => return Err("upbit duplicate timestamp conflicts".to_string()),
                None => {
                    rows.insert(row.timestamp_ms, row.clone());
                }
            }
        }
    }
    if rows.is_empty() {
        return Err("upbit backfill has no rows".to_string());
    }
    Ok((
        HistoricalReplayDataset {
            symbol: expected_symbol.to_string(),
            source: "upbit-approved-readonly-daily-backfill".to_string(),
            rows: rows.into_values().collect(),
            reason_codes: vec![ReasonCode::DataSnapshotImmutable],
        },
        duplicates,
    ))
}

fn acquire_upbit_page_v0(
    config: &UpbitHistoricalPilotConfigV0,
    end_timestamp_ms: u64,
    page_size: usize,
) -> Result<(DataSnapshot, UpbitHistoricalPageReceiptV0), String> {
    let capabilities = UpbitDailyOhlcvProviderV0::new(config.clone()).capabilities();
    let mut registry = ReadOnlyProviderRegistry::default();
    registry.register(capabilities);
    let mut policy = AcquisitionPolicy::default();
    policy.allow_approved_readonly_network = true;
    policy.max_response_bytes = config.maximum_response_bytes;
    policy.max_retries = config.max_retries;
    policy.max_requests_per_provider = 1;
    let mut broker = DataAcquisitionBroker::new(registry, policy);
    let request = ReadOnlyProviderRequest {
        request_id: format!(
            "upbit-page-{}",
            stable_hash_string(&format!("{}:{end_timestamp_ms}", config.symbol))
        ),
        request_key: format!(
            "upbit-daily-page:{}:{}:{page_size}",
            config.symbol, end_timestamp_ms
        ),
        provider_id: UPBIT_PROVIDER_ID.to_string(),
        dataset_kind: DatasetKind::DailyOhlcv,
        market_scope: AcquisitionMarketScope::BtcCrypto,
        symbols: vec![config.symbol.clone()],
        lookback: DataLookback {
            bars: page_size,
            start_timestamp_ms: Some(config.start_timestamp_ms),
            end_timestamp_ms: Some(end_timestamp_ms),
        },
        cadence: "1d".to_string(),
        max_staleness_ms: u64::MAX,
        reason_codes: vec![],
    };
    let plan = AcquisitionPlan {
        planned_requests: vec![AcquisitionRequest {
            request,
            requested_by_agents: vec![],
            required_by_agents: vec![],
        }],
        rejected_requests: vec![],
        agent_request_mapping: Default::default(),
        deduplicated_request_count: 0,
        reason_codes: vec![],
    };
    let mut provider = UpbitDailyOhlcvProviderV0::new(config.clone());
    let execution = broker.execute_acquisition_plan(
        &plan,
        AcquisitionMode::ApprovedReadOnlyNetwork,
        current_time_ms(),
        Some(&mut provider),
    );
    let receipt = execution
        .receipts
        .into_iter()
        .next()
        .ok_or_else(|| "upbit page receipt missing".to_string())?;
    let snapshot = execution.new_snapshots.into_iter().next().ok_or_else(|| {
        receipt
            .reason_codes
            .iter()
            .map(|reason| format!("{reason:?}"))
            .collect::<Vec<_>>()
            .join("|")
    })?;
    Ok((
        snapshot.clone(),
        UpbitHistoricalPageReceiptV0 {
            request_id: receipt.request_id,
            receipt_id: receipt.receipt_id,
            end_exclusive_timestamp_ms: end_timestamp_ms,
            row_count: snapshot.row_count,
            attempt_count: receipt.attempt_count,
            snapshot_id: Some(snapshot.snapshot_id),
        },
    ))
}

fn merged_snapshot_v0(
    first: &DataSnapshot,
    config: &UpbitHistoricalPilotConfigV0,
    dataset: HistoricalReplayDataset,
    receipts: &[UpbitHistoricalPageReceiptV0],
) -> DataSnapshot {
    let digest = dataset_digest(&dataset);
    let receipt_material = receipts
        .iter()
        .map(|receipt| receipt.receipt_id.as_str())
        .collect::<Vec<_>>()
        .join("|");
    let request_key = format!(
        "upbit-daily-backfill:{}:{}:{}",
        config.symbol, config.start_timestamp_ms, config.end_timestamp_ms
    );
    let mut snapshot = first.clone();
    snapshot.snapshot_id = super::acquisition::snapshot_id_from_semantic_digest_v1(&digest);
    snapshot.request_key = request_key;
    snapshot.requested_lookback = DataLookback {
        bars: dataset.rows.len(),
        start_timestamp_ms: Some(config.start_timestamp_ms),
        end_timestamp_ms: Some(config.end_timestamp_ms),
    };
    snapshot.actual_start_timestamp_ms = dataset.rows.first().map(|row| row.timestamp_ms);
    snapshot.actual_end_timestamp_ms = dataset.rows.last().map(|row| row.timestamp_ms);
    snapshot.fetched_at_ms = current_time_ms();
    snapshot.normalized_at_ms = snapshot.fetched_at_ms;
    snapshot.row_count = dataset.rows.len();
    snapshot.quality_summary.row_count = snapshot.row_count;
    snapshot.content_digest = digest;
    snapshot.normalized_dataset = dataset;
    snapshot.provenance.acquisition_request_id = format!(
        "upbit-backfill-{}",
        stable_hash_string(&snapshot.request_key)
    );
    snapshot.provenance.fetch_receipt_id =
        format!("backfill-{}", stable_hash_string(&receipt_material));
    snapshot
}

fn dataset_digest(dataset: &HistoricalReplayDataset) -> String {
    super::acquisition::historical_replay_dataset_digest_v0(dataset)
}

fn completed_backfill_result(
    status: UpbitHistoricalBackfillStatusV0,
    config: &UpbitHistoricalPilotConfigV0,
    snapshot: &DataSnapshot,
    path: PathBuf,
    page_receipts: Vec<UpbitHistoricalPageReceiptV0>,
    reason_codes: Vec<String>,
) -> UpbitHistoricalBackfillResultV0 {
    UpbitHistoricalBackfillResultV0 {
        status,
        provider_id: Some(UPBIT_PROVIDER_ID.to_string()),
        symbol: Some(config.symbol.clone()),
        requested_start_timestamp_ms: Some(config.start_timestamp_ms),
        requested_end_timestamp_ms: Some(config.end_timestamp_ms),
        actual_start_timestamp_ms: snapshot.actual_start_timestamp_ms,
        actual_end_timestamp_ms: snapshot.actual_end_timestamp_ms,
        row_count: snapshot.row_count,
        page_receipts,
        snapshot_id: Some(snapshot.snapshot_id.clone()),
        snapshot_digest: Some(snapshot.content_digest.clone()),
        local_snapshot_path: Some(path.display().to_string()),
        reason_codes,
    }
}

fn partial_backfill_result(
    status: UpbitHistoricalBackfillStatusV0,
    config: &UpbitHistoricalPilotConfigV0,
    dataset: &HistoricalReplayDataset,
    page_receipts: Vec<UpbitHistoricalPageReceiptV0>,
    reason_codes: Vec<String>,
) -> UpbitHistoricalBackfillResultV0 {
    UpbitHistoricalBackfillResultV0 {
        status,
        provider_id: Some(UPBIT_PROVIDER_ID.to_string()),
        symbol: Some(config.symbol.clone()),
        requested_start_timestamp_ms: Some(config.start_timestamp_ms),
        requested_end_timestamp_ms: Some(config.end_timestamp_ms),
        actual_start_timestamp_ms: dataset.rows.first().map(|row| row.timestamp_ms),
        actual_end_timestamp_ms: dataset.rows.last().map(|row| row.timestamp_ms),
        row_count: dataset.rows.len(),
        page_receipts,
        snapshot_id: None,
        snapshot_digest: None,
        local_snapshot_path: None,
        reason_codes,
    }
}

fn backfill_result(
    status: UpbitHistoricalBackfillStatusV0,
    config: Option<&UpbitHistoricalPilotConfigV0>,
    reason_codes: Vec<String>,
) -> UpbitHistoricalBackfillResultV0 {
    UpbitHistoricalBackfillResultV0 {
        status,
        provider_id: config.map(|_| UPBIT_PROVIDER_ID.to_string()),
        symbol: config.map(|config| config.symbol.clone()),
        requested_start_timestamp_ms: config.map(|config| config.start_timestamp_ms),
        requested_end_timestamp_ms: config.map(|config| config.end_timestamp_ms),
        actual_start_timestamp_ms: None,
        actual_end_timestamp_ms: None,
        row_count: 0,
        page_receipts: vec![],
        snapshot_id: None,
        snapshot_digest: None,
        local_snapshot_path: None,
        reason_codes,
    }
}

pub fn parse_upbit_daily_ohlcv_v0(
    body: &str,
    expected_symbol: &str,
) -> Result<HistoricalReplayDataset, String> {
    let rows = serde_json::from_str::<Vec<UpbitDailyCandleV0>>(body)
        .map_err(|_| "upbit response schema rejected".to_string())?;
    if rows.is_empty() {
        return Err("upbit response has no daily candles".to_string());
    }
    let mut normalized = rows
        .into_iter()
        .map(|row| {
            if row.market != expected_symbol {
                return Err("upbit response symbol mismatch".to_string());
            }
            let timestamp_ms = parse_upbit_utc_timestamp_ms(&row.candle_date_time_utc)?;
            if !row.opening_price.is_finite()
                || !row.high_price.is_finite()
                || !row.low_price.is_finite()
                || !row.trade_price.is_finite()
                || !row.candle_acc_trade_volume.is_finite()
                || row.opening_price <= 0.0
                || row.low_price <= 0.0
                || row.high_price < row.opening_price.max(row.trade_price)
                || row.low_price > row.opening_price.min(row.trade_price)
                || row.candle_acc_trade_volume < 0.0
                || row
                    .candle_acc_trade_price
                    .is_some_and(|value| !value.is_finite() || value < 0.0)
            {
                return Err("upbit response contains invalid OHLCV".to_string());
            }
            Ok(HistoricalOhlcvRow {
                symbol: expected_symbol.to_string(),
                timestamp_ms,
                open: row.opening_price,
                high: row.high_price,
                low: row.low_price,
                close: row.trade_price,
                volume: row.candle_acc_trade_volume,
                trade_value: row.candle_acc_trade_price,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    normalized.sort_by_key(|row| row.timestamp_ms);
    if normalized
        .windows(2)
        .any(|pair| pair[0].timestamp_ms >= pair[1].timestamp_ms)
    {
        return Err("upbit response has duplicate or non-monotonic timestamps".to_string());
    }
    Ok(HistoricalReplayDataset {
        symbol: expected_symbol.to_string(),
        source: "upbit-approved-readonly-daily".to_string(),
        rows: normalized,
        reason_codes: vec![],
    })
}

pub fn write_and_verify_local_snapshot_v0(
    snapshot: &DataSnapshot,
    output_dir: &Path,
) -> Result<PathBuf, String> {
    if !safe_snapshot_output_dir(output_dir) {
        return Err("local snapshot output path rejected".to_string());
    }
    verify_snapshot_semantic_identity_v1(snapshot)?;
    let serialized = SnapshotCodec::protobuf_v1().encode(snapshot)?;
    fs::create_dir_all(output_dir)
        .map_err(|_| "local snapshot directory unavailable".to_string())?;
    let path = output_dir.join(format!("{}.pb", snapshot.snapshot_id));
    let temporary = output_dir.join(format!(".{}.tmp", snapshot.snapshot_id));
    let mut file =
        File::create(&temporary).map_err(|_| "local snapshot write failed".to_string())?;
    file.write_all(&serialized)
        .map_err(|_| "local snapshot write failed".to_string())?;
    file.sync_all()
        .map_err(|_| "local snapshot sync failed".to_string())?;
    drop(file);
    let temporary_snapshot = read_and_verify_local_snapshot_v0(&temporary, snapshot)?;
    if temporary_snapshot.snapshot_id != snapshot.snapshot_id {
        return Err("local snapshot identifier verification failed".to_string());
    }
    fs::rename(&temporary, &path).map_err(|_| "local snapshot atomic rename failed".to_string())?;
    read_and_verify_local_snapshot_v0(&path, snapshot)?;
    Ok(path)
}

fn read_and_verify_local_snapshot_v0(
    path: &Path,
    expected: &DataSnapshot,
) -> Result<DataSnapshot, String> {
    let stored = SnapshotCodec::protobuf_v1()
        .decode(&fs::read(path).map_err(|_| "local snapshot reread failed".to_string())?)?;
    if stored.snapshot_id != expected.snapshot_id {
        return Err("local snapshot identifier verification failed".to_string());
    }
    if stored.content_digest != expected.content_digest {
        return Err("local snapshot digest verification failed".to_string());
    }
    verify_snapshot_semantic_identity_v1(&stored)?;
    Ok(stored)
}

pub fn read_local_snapshot_protobuf_v1(path: &Path) -> Result<DataSnapshot, String> {
    SnapshotCodec::protobuf_v1()
        .decode(&fs::read(path).map_err(|_| "local snapshot reread failed".to_string())?)
}

pub fn migrate_legacy_json_snapshot_v0(path: &Path) -> Result<PathBuf, String> {
    if path.extension().is_none_or(|extension| extension != "json") {
        return Err("legacy snapshot path must be json".to_string());
    }
    let mut snapshot = SnapshotCodec::json_legacy_v0()
        .decode(&fs::read(path).map_err(|_| "legacy snapshot reread failed".to_string())?)?;
    validate_snapshot_shape_v1(&snapshot)?;
    snapshot.content_digest = dataset_digest(&snapshot.normalized_dataset);
    snapshot.snapshot_id =
        super::acquisition::snapshot_id_from_semantic_digest_v1(&snapshot.content_digest);
    let output_dir = path
        .parent()
        .ok_or_else(|| "legacy snapshot parent unavailable".to_string())?;
    write_and_verify_local_snapshot_v0(&snapshot, output_dir)
}

fn encode_snapshot_protobuf_v1(snapshot: &DataSnapshot) -> Result<Vec<u8>, String> {
    verify_snapshot_semantic_identity_v1(snapshot)?;
    let payload = SnapshotPayloadProtobufV1::from_snapshot(snapshot)?.encode_to_vec();
    let envelope = SnapshotEnvelopeProtobufV1 {
        magic: SNAPSHOT_PROTOBUF_MAGIC_V1.to_vec(),
        version: 1,
        schema: SNAPSHOT_PROTOBUF_SCHEMA_V1.to_string(),
        semantic_digest: snapshot.content_digest.clone(),
        snapshot_id: snapshot.snapshot_id.clone(),
        payload_length: u64::try_from(payload.len())
            .map_err(|_| "snapshot payload too large".to_string())?,
        payload_digest: super::acquisition::canonical_hash_hex(&payload),
        payload,
    };
    Ok(envelope.encode_to_vec())
}

fn decode_snapshot_protobuf_v1(bytes: &[u8]) -> Result<DataSnapshot, String> {
    let envelope = SnapshotEnvelopeProtobufV1::decode(bytes)
        .map_err(|_| "protobuf snapshot envelope decode failed".to_string())?;
    if envelope.magic != SNAPSHOT_PROTOBUF_MAGIC_V1
        || envelope.version != 1
        || envelope.schema != SNAPSHOT_PROTOBUF_SCHEMA_V1
    {
        return Err("protobuf snapshot envelope version rejected".to_string());
    }
    if usize::try_from(envelope.payload_length).ok() != Some(envelope.payload.len())
        || super::acquisition::canonical_hash_hex(&envelope.payload) != envelope.payload_digest
    {
        return Err("protobuf snapshot payload integrity rejected".to_string());
    }
    let snapshot = SnapshotPayloadProtobufV1::decode(envelope.payload.as_slice())
        .map_err(|_| "protobuf snapshot payload decode failed".to_string())?
        .into_snapshot()?;
    if snapshot.snapshot_id != envelope.snapshot_id
        || snapshot.content_digest != envelope.semantic_digest
    {
        return Err("protobuf snapshot identity mismatch".to_string());
    }
    verify_snapshot_semantic_identity_v1(&snapshot)?;
    Ok(snapshot)
}

fn verify_snapshot_semantic_identity_v1(snapshot: &DataSnapshot) -> Result<(), String> {
    validate_snapshot_shape_v1(snapshot)?;
    let digest = dataset_digest(&snapshot.normalized_dataset);
    if snapshot.content_digest != digest {
        return Err("snapshot semantic digest verification failed".to_string());
    }
    if snapshot.snapshot_id != super::acquisition::snapshot_id_from_semantic_digest_v1(&digest) {
        return Err("snapshot semantic identifier verification failed".to_string());
    }
    Ok(())
}

fn validate_snapshot_shape_v1(snapshot: &DataSnapshot) -> Result<(), String> {
    let dataset = &snapshot.normalized_dataset;
    if snapshot.row_count != dataset.rows.len()
        || snapshot.quality_summary.row_count != snapshot.row_count
        || dataset.rows.is_empty()
        || snapshot.symbols.len() != 1
        || snapshot.symbols[0] != dataset.symbol
        || snapshot.actual_start_timestamp_ms != dataset.rows.first().map(|row| row.timestamp_ms)
        || snapshot.actual_end_timestamp_ms != dataset.rows.last().map(|row| row.timestamp_ms)
    {
        return Err("snapshot shape verification failed".to_string());
    }
    for pair in dataset.rows.windows(2) {
        if pair[0].timestamp_ms >= pair[1].timestamp_ms {
            return Err("snapshot chronology verification failed".to_string());
        }
    }
    if dataset.rows.iter().any(|row| {
        row.symbol != dataset.symbol
            || !row.open.is_finite()
            || !row.high.is_finite()
            || !row.low.is_finite()
            || !row.close.is_finite()
            || !row.volume.is_finite()
            || row.trade_value.is_some_and(|value| !value.is_finite())
    }) {
        return Err("snapshot OHLCV verification failed".to_string());
    }
    Ok(())
}

impl SnapshotPayloadProtobufV1 {
    fn from_snapshot(snapshot: &DataSnapshot) -> Result<Self, String> {
        Ok(Self {
            snapshot_id: snapshot.snapshot_id.clone(),
            request_key: snapshot.request_key.clone(),
            provider_id: snapshot.provider_id.clone(),
            dataset_kind: dataset_kind_tag(snapshot.dataset_kind),
            market_scope: market_scope_tag(snapshot.market_scope),
            symbols: sorted_strings(&snapshot.symbols),
            requested_lookback: Some(LookbackProtobufV1 {
                bars: u64::try_from(snapshot.requested_lookback.bars)
                    .map_err(|_| "snapshot lookback too large".to_string())?,
                start_timestamp_ms: snapshot.requested_lookback.start_timestamp_ms,
                end_timestamp_ms: snapshot.requested_lookback.end_timestamp_ms,
            }),
            actual_start_timestamp_ms: snapshot.actual_start_timestamp_ms,
            actual_end_timestamp_ms: snapshot.actual_end_timestamp_ms,
            fetched_at_ms: snapshot.fetched_at_ms,
            normalized_at_ms: snapshot.normalized_at_ms,
            schema_version: snapshot.schema_version,
            row_count: u64::try_from(snapshot.row_count)
                .map_err(|_| "snapshot row count too large".to_string())?,
            quality_summary: Some(QualityProtobufV1 {
                accepted: snapshot.quality_summary.accepted,
                row_count: u64::try_from(snapshot.quality_summary.row_count)
                    .map_err(|_| "snapshot quality row count too large".to_string())?,
                reason_codes: wire_reason_codes(&snapshot.quality_summary.reason_codes)?,
            }),
            content_digest: snapshot.content_digest.clone(),
            sanitized: snapshot.sanitized,
            read_only: snapshot.read_only,
            normalized_dataset: Some(DatasetProtobufV1 {
                symbol: snapshot.normalized_dataset.symbol.clone(),
                rows: snapshot
                    .normalized_dataset
                    .rows
                    .iter()
                    .map(|row| OhlcvProtobufV1 {
                        symbol: row.symbol.clone(),
                        timestamp_ms: row.timestamp_ms,
                        open_bits: row.open.to_bits(),
                        high_bits: row.high.to_bits(),
                        low_bits: row.low.to_bits(),
                        close_bits: row.close.to_bits(),
                        volume_bits: row.volume.to_bits(),
                        trade_value_bits: row.trade_value.map(f64::to_bits),
                    })
                    .collect(),
                source: snapshot.normalized_dataset.source.clone(),
                reason_codes: wire_reason_codes(&snapshot.normalized_dataset.reason_codes)?,
            }),
            provenance: Some(ProvenanceProtobufV1 {
                provider_id: snapshot.provenance.provider_id.clone(),
                acquisition_request_id: snapshot.provenance.acquisition_request_id.clone(),
                fetch_receipt_id: snapshot.provenance.fetch_receipt_id.clone(),
                source_type: source_type_tag(snapshot.provenance.source_type),
                sanitized: snapshot.provenance.sanitized,
                credential_free: snapshot.provenance.credential_free,
                reason_codes: wire_reason_codes(&snapshot.provenance.reason_codes)?,
            }),
            reason_codes: wire_reason_codes(&snapshot.reason_codes)?,
        })
    }

    fn into_snapshot(self) -> Result<DataSnapshot, String> {
        let lookback = self
            .requested_lookback
            .ok_or_else(|| "protobuf snapshot lookback missing".to_string())?;
        let quality = self
            .quality_summary
            .ok_or_else(|| "protobuf snapshot quality missing".to_string())?;
        let dataset = self
            .normalized_dataset
            .ok_or_else(|| "protobuf snapshot dataset missing".to_string())?;
        let provenance = self
            .provenance
            .ok_or_else(|| "protobuf snapshot provenance missing".to_string())?;
        Ok(DataSnapshot {
            snapshot_id: self.snapshot_id,
            request_key: self.request_key,
            provider_id: self.provider_id,
            dataset_kind: dataset_kind_from_tag(self.dataset_kind)?,
            market_scope: market_scope_from_tag(self.market_scope)?,
            symbols: self.symbols,
            requested_lookback: DataLookback {
                bars: usize::try_from(lookback.bars)
                    .map_err(|_| "protobuf snapshot lookback rejected".to_string())?,
                start_timestamp_ms: lookback.start_timestamp_ms,
                end_timestamp_ms: lookback.end_timestamp_ms,
            },
            actual_start_timestamp_ms: self.actual_start_timestamp_ms,
            actual_end_timestamp_ms: self.actual_end_timestamp_ms,
            fetched_at_ms: self.fetched_at_ms,
            normalized_at_ms: self.normalized_at_ms,
            schema_version: self.schema_version,
            row_count: usize::try_from(self.row_count)
                .map_err(|_| "protobuf snapshot row count rejected".to_string())?,
            quality_summary: SnapshotQualitySummary {
                accepted: quality.accepted,
                row_count: usize::try_from(quality.row_count)
                    .map_err(|_| "protobuf snapshot quality row count rejected".to_string())?,
                reason_codes: parse_wire_reason_codes(&quality.reason_codes)?,
            },
            content_digest: self.content_digest,
            sanitized: self.sanitized,
            read_only: self.read_only,
            normalized_dataset: HistoricalReplayDataset {
                symbol: dataset.symbol,
                rows: dataset
                    .rows
                    .into_iter()
                    .map(|row| HistoricalOhlcvRow {
                        symbol: row.symbol,
                        timestamp_ms: row.timestamp_ms,
                        open: f64::from_bits(row.open_bits),
                        high: f64::from_bits(row.high_bits),
                        low: f64::from_bits(row.low_bits),
                        close: f64::from_bits(row.close_bits),
                        volume: f64::from_bits(row.volume_bits),
                        trade_value: row.trade_value_bits.map(f64::from_bits),
                    })
                    .collect(),
                source: dataset.source,
                reason_codes: parse_wire_reason_codes(&dataset.reason_codes)?,
            },
            provenance: SnapshotProvenance {
                provider_id: provenance.provider_id,
                acquisition_request_id: provenance.acquisition_request_id,
                fetch_receipt_id: provenance.fetch_receipt_id,
                source_type: source_type_from_tag(provenance.source_type)?,
                sanitized: provenance.sanitized,
                credential_free: provenance.credential_free,
                reason_codes: parse_wire_reason_codes(&provenance.reason_codes)?,
            },
            reason_codes: parse_wire_reason_codes(&self.reason_codes)?,
        })
    }
}

fn sorted_strings(values: &[String]) -> Vec<String> {
    let mut values = values.to_vec();
    values.sort();
    values.dedup();
    values
}

fn wire_reason_codes(values: &[ReasonCode]) -> Result<Vec<String>, String> {
    let mut values = values
        .iter()
        .map(|value| {
            serde_json::to_value(value).map_err(|_| "reason code serialization failed".to_string())
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .map(|value| {
            value
                .as_str()
                .map(str::to_owned)
                .ok_or_else(|| "reason code wire value rejected".to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;
    values.sort();
    values.dedup();
    Ok(values)
}

fn parse_wire_reason_codes(values: &[String]) -> Result<Vec<ReasonCode>, String> {
    values
        .iter()
        .map(|value| {
            serde_json::from_value(serde_json::Value::String(value.clone()))
                .map_err(|_| "protobuf reason code rejected".to_string())
        })
        .collect()
}

fn dataset_kind_tag(value: DatasetKind) -> u32 {
    match value {
        DatasetKind::DailyOhlcv => 1,
        DatasetKind::AdjustedDailyOhlcv => 2,
        DatasetKind::CorporateActions => 3,
        DatasetKind::QuarterlyFundamentals => 4,
        DatasetKind::ValuationMetrics => 5,
        DatasetKind::MarketIndexDaily => 6,
        DatasetKind::MarketBreadthDaily => 7,
        DatasetKind::VolatilityDaily => 8,
        DatasetKind::LiquidityDaily => 9,
        DatasetKind::CryptoDailyOhlcv => 10,
        DatasetKind::MacroSeries => 11,
        DatasetKind::Unknown => 12,
    }
}

fn dataset_kind_from_tag(value: u32) -> Result<DatasetKind, String> {
    match value {
        1 => Ok(DatasetKind::DailyOhlcv),
        2 => Ok(DatasetKind::AdjustedDailyOhlcv),
        3 => Ok(DatasetKind::CorporateActions),
        4 => Ok(DatasetKind::QuarterlyFundamentals),
        5 => Ok(DatasetKind::ValuationMetrics),
        6 => Ok(DatasetKind::MarketIndexDaily),
        7 => Ok(DatasetKind::MarketBreadthDaily),
        8 => Ok(DatasetKind::VolatilityDaily),
        9 => Ok(DatasetKind::LiquidityDaily),
        10 => Ok(DatasetKind::CryptoDailyOhlcv),
        11 => Ok(DatasetKind::MacroSeries),
        12 => Ok(DatasetKind::Unknown),
        _ => Err("protobuf dataset kind rejected".to_string()),
    }
}

fn market_scope_tag(value: AcquisitionMarketScope) -> u32 {
    match value {
        AcquisitionMarketScope::UsStocks => 1,
        AcquisitionMarketScope::KoreanStocks => 2,
        AcquisitionMarketScope::BtcCrypto => 3,
        AcquisitionMarketScope::Unknown => 4,
    }
}

fn market_scope_from_tag(value: u32) -> Result<AcquisitionMarketScope, String> {
    match value {
        1 => Ok(AcquisitionMarketScope::UsStocks),
        2 => Ok(AcquisitionMarketScope::KoreanStocks),
        3 => Ok(AcquisitionMarketScope::BtcCrypto),
        4 => Ok(AcquisitionMarketScope::Unknown),
        _ => Err("protobuf market scope rejected".to_string()),
    }
}

fn source_type_tag(value: SnapshotSourceType) -> u32 {
    match value {
        SnapshotSourceType::Mock => 1,
        SnapshotSourceType::LocalSnapshotReplay => 2,
        SnapshotSourceType::ApprovedReadOnlyProvider => 3,
    }
}

fn source_type_from_tag(value: u32) -> Result<SnapshotSourceType, String> {
    match value {
        1 => Ok(SnapshotSourceType::Mock),
        2 => Ok(SnapshotSourceType::LocalSnapshotReplay),
        3 => Ok(SnapshotSourceType::ApprovedReadOnlyProvider),
        _ => Err("protobuf snapshot source type rejected".to_string()),
    }
}

fn harvest_result(
    status: FirstHistoricalHarvestStatusV0,
    config: Option<&UpbitHistoricalPilotConfigV0>,
    reason_codes: Vec<String>,
) -> FirstHistoricalHarvestResultV0 {
    FirstHistoricalHarvestResultV0 {
        status,
        provider_id: config.map(|_| UPBIT_PROVIDER_ID.to_string()),
        market: config.map(|value| value.market),
        symbol: config.map(|value| value.symbol.clone()),
        requested_start_timestamp_ms: config.map(|value| value.start_timestamp_ms),
        requested_end_timestamp_ms: config.map(|value| value.end_timestamp_ms),
        actual_start_timestamp_ms: None,
        actual_end_timestamp_ms: None,
        row_count: 0,
        snapshot_id: None,
        snapshot_digest: None,
        local_snapshot_path: None,
        reason_codes,
    }
}

fn upbit_daily_candles_url(symbol: &str, end_timestamp_ms: u64, count: usize) -> Option<String> {
    if !valid_market_symbol(symbol) || count == 0 || count > UPBIT_MAX_CANDLES_PER_REQUEST {
        return None;
    }
    Some(format!(
        "{UPBIT_DAILY_CANDLES_ENDPOINT}?market={symbol}&to={}&count={count}",
        format_utc_timestamp(end_timestamp_ms)?
    ))
}

fn valid_market_symbol(symbol: &str) -> bool {
    !symbol.is_empty()
        && symbol.len() <= 32
        && symbol
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'-')
}

fn safe_snapshot_output_dir(path: &Path) -> bool {
    path.starts_with(DEFAULT_SNAPSHOT_OUTPUT_DIR)
        && path.components().all(|component| {
            !matches!(component, Component::ParentDir | Component::RootDir)
                && component.as_os_str() != ".env"
        })
}

fn parse_upbit_utc_timestamp_ms(value: &str) -> Result<u64, String> {
    let value = value.strip_suffix('Z').unwrap_or(value);
    if value.len() != 19
        || value.as_bytes()[10] != b'T'
        || value.as_bytes()[4] != b'-'
        || value.as_bytes()[7] != b'-'
        || value.as_bytes()[13] != b':'
        || value.as_bytes()[16] != b':'
    {
        return Err("upbit candle timestamp is not UTC ISO-8601".to_string());
    }
    let year = value[0..4]
        .parse::<i32>()
        .map_err(|_| "invalid UTC year".to_string())?;
    let month = value[5..7]
        .parse::<u32>()
        .map_err(|_| "invalid UTC month".to_string())?;
    let day = value[8..10]
        .parse::<u32>()
        .map_err(|_| "invalid UTC day".to_string())?;
    let hour = value[11..13]
        .parse::<u64>()
        .map_err(|_| "invalid UTC hour".to_string())?;
    let minute = value[14..16]
        .parse::<u64>()
        .map_err(|_| "invalid UTC minute".to_string())?;
    let second = value[17..19]
        .parse::<u64>()
        .map_err(|_| "invalid UTC second".to_string())?;
    if !(1..=12).contains(&month)
        || day == 0
        || day > days_in_month(year, month)
        || hour > 23
        || minute > 59
        || second > 59
    {
        return Err("invalid UTC timestamp components".to_string());
    }
    let days = days_from_civil(year, month, day);
    u64::try_from(days.saturating_mul(86_400_000))
        .map_err(|_| "UTC timestamp predates epoch".to_string())?
        .checked_add(hour * 3_600_000 + minute * 60_000 + second * 1_000)
        .ok_or_else(|| "UTC timestamp overflow".to_string())
}

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
        2 => 28,
        _ => 0,
    }
}

fn format_utc_timestamp(timestamp_ms: u64) -> Option<String> {
    let seconds = timestamp_ms / 1_000;
    let days = i64::try_from(seconds / 86_400).ok()?;
    let (year, month, day) = civil_from_days(days);
    let second_of_day = seconds % 86_400;
    Some(format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        second_of_day / 3_600,
        (second_of_day % 3_600) / 60,
        second_of_day % 60
    ))
}

fn days_from_civil(year: i32, month: u32, day: u32) -> i64 {
    let mut year = year as i64;
    let month = month as i64;
    let day = day as i64;
    year -= if month <= 2 { 1 } else { 0 };
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let yoe = year - era * 400;
    let doy = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let days = days + 719_468;
    let era = if days >= 0 { days } else { days - 146_096 } / 146_097;
    let doe = days - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    (year + if month <= 2 { 1 } else { 0 }, month, day)
}

fn current_time_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{SnapshotProvenance, SnapshotQualitySummary, SnapshotSourceType};

    fn config() -> UpbitHistoricalPilotConfigV0 {
        UpbitHistoricalPilotConfigV0 {
            provider_id: UPBIT_PROVIDER_ID.to_string(),
            enabled: true,
            market: AcquisitionMarketScope::BtcCrypto,
            symbol: "KRW-BTC".to_string(),
            start_timestamp_ms: 1_704_067_200_000,
            end_timestamp_ms: 1_704_240_000_000,
            maximum_rows: 2,
            timeout_seconds: 10,
            max_retries: 0,
            maximum_response_bytes: 16_384,
            snapshot_output_dir: DEFAULT_SNAPSHOT_OUTPUT_DIR.to_string(),
            network_consent: NetworkConsentV0::ManualLocalSmoke,
            manual_smoke_enabled: true,
            page_size: 2,
            target_rows: 4,
            maximum_pages: 2,
            stop_when_campaign_sufficient: true,
            campaign_attempt_enabled: false,
        }
    }

    fn local_snapshot(dataset: HistoricalReplayDataset, _snapshot_id: &str) -> DataSnapshot {
        let row_count = dataset.rows.len();
        let digest = dataset_digest(&dataset);
        let symbol = dataset.symbol.clone();
        DataSnapshot {
            snapshot_id: crate::data::snapshot_id_from_semantic_digest_v1(&digest),
            request_key: "upbit-test-local-snapshot".to_string(),
            provider_id: UPBIT_PROVIDER_ID.to_string(),
            dataset_kind: DatasetKind::DailyOhlcv,
            market_scope: AcquisitionMarketScope::BtcCrypto,
            symbols: vec![symbol],
            requested_lookback: DataLookback {
                bars: row_count,
                start_timestamp_ms: Some(1_704_067_200_000),
                end_timestamp_ms: Some(1_704_240_000_000),
            },
            actual_start_timestamp_ms: dataset.rows.first().map(|row| row.timestamp_ms),
            actual_end_timestamp_ms: dataset.rows.last().map(|row| row.timestamp_ms),
            fetched_at_ms: 1_704_240_000_000,
            normalized_at_ms: 1_704_240_000_000,
            schema_version: 1,
            row_count,
            quality_summary: SnapshotQualitySummary {
                accepted: true,
                row_count,
                reason_codes: vec![],
            },
            content_digest: digest,
            sanitized: true,
            read_only: true,
            normalized_dataset: dataset,
            provenance: SnapshotProvenance {
                provider_id: UPBIT_PROVIDER_ID.to_string(),
                acquisition_request_id: "upbit-test-request".to_string(),
                fetch_receipt_id: "upbit-test-receipt".to_string(),
                source_type: SnapshotSourceType::ApprovedReadOnlyProvider,
                sanitized: true,
                credential_free: true,
                reason_codes: vec![],
            },
            reason_codes: vec![ReasonCode::DataSnapshotImmutable],
        }
    }

    #[test]
    fn qualification_and_selection_require_local_consent() {
        let config = config();
        assert_eq!(
            qualify_upbit_historical_provider_v0(Some(&config)).status,
            HistoricalProviderQualificationStatusV0::Qualified
        );
        assert_eq!(
            select_upbit_historical_provider_v0(Some(&config), false).status,
            HistoricalProviderSelectionStatusV0::NetworkConsentRequired
        );
        assert_eq!(
            select_upbit_historical_provider_v0(Some(&config), true).selected_provider,
            Some(UPBIT_PROVIDER_ID.to_string())
        );
    }

    #[test]
    fn parser_normalizes_daily_rows_and_rejects_symbol_mismatch() {
        let body = r#"[
          {"market":"KRW-BTC","candle_date_time_utc":"2024-01-02T00:00:00","opening_price":10.0,"high_price":12.0,"low_price":9.0,"trade_price":11.0,"candle_acc_trade_price":100.0,"candle_acc_trade_volume":5.0},
          {"market":"KRW-BTC","candle_date_time_utc":"2024-01-01T00:00:00","opening_price":8.0,"high_price":10.0,"low_price":7.0,"trade_price":9.0,"candle_acc_trade_price":80.0,"candle_acc_trade_volume":4.0}
        ]"#;
        let dataset = parse_upbit_daily_ohlcv_v0(body, "KRW-BTC").unwrap();
        assert_eq!(dataset.rows.len(), 2);
        assert!(dataset.rows[0].timestamp_ms < dataset.rows[1].timestamp_ms);
        assert!(parse_upbit_daily_ohlcv_v0(body, "KRW-ETH").is_err());
    }

    #[test]
    fn parser_rejects_invalid_prices_and_duplicate_timestamps() {
        let invalid = r#"[{"market":"KRW-BTC","candle_date_time_utc":"2024-01-01T00:00:00","opening_price":0.0,"high_price":1.0,"low_price":1.0,"trade_price":1.0,"candle_acc_trade_volume":1.0}]"#;
        let duplicate = r#"[
          {"market":"KRW-BTC","candle_date_time_utc":"2024-01-01T00:00:00","opening_price":1.0,"high_price":2.0,"low_price":1.0,"trade_price":1.5,"candle_acc_trade_volume":1.0},
          {"market":"KRW-BTC","candle_date_time_utc":"2024-01-01T00:00:00","opening_price":1.0,"high_price":2.0,"low_price":1.0,"trade_price":1.5,"candle_acc_trade_volume":1.0}
        ]"#;
        assert!(parse_upbit_daily_ohlcv_v0(invalid, "KRW-BTC").is_err());
        assert!(parse_upbit_daily_ohlcv_v0(duplicate, "KRW-BTC").is_err());
        assert!(parse_upbit_utc_timestamp_ms("2024-02-30T00:00:00").is_err());
    }

    #[test]
    fn endpoint_is_fixed_https_and_symbol_is_validated() {
        assert!(
            upbit_daily_candles_url("KRW-BTC", 1_704_240_000_000, 2)
                .unwrap()
                .starts_with(UPBIT_DAILY_CANDLES_ENDPOINT)
        );
        assert!(upbit_daily_candles_url("KRW-BTC&x=1", 1_704_240_000_000, 2).is_none());
    }

    #[test]
    fn backfill_config_and_preflight_fail_closed_without_local_consent() {
        let mut config = config();
        config.page_size = 0;
        assert!(config.validate().is_err());
        assert_eq!(
            preflight_upbit_historical_backfill_v0(Path::new("config/local/missing.toml"), true)
                .status,
            UpbitHistoricalPreflightStatusV0::ConfigurationMissing
        );
    }

    #[test]
    fn local_snapshot_write_verifies_before_and_after_atomic_rename() {
        let dataset = parse_upbit_daily_ohlcv_v0(
            r#"[{"market":"KRW-BTC","candle_date_time_utc":"2024-01-01T00:00:00","opening_price":1.0,"high_price":2.0,"low_price":0.5,"trade_price":1.5,"candle_acc_trade_volume":1.0}]"#,
            "KRW-BTC",
        )
        .unwrap();
        let output_dir = Path::new(DEFAULT_SNAPSHOT_OUTPUT_DIR);
        let snapshot = local_snapshot(dataset, "snapshot-upbit-local-write-test");
        let path = output_dir.join(format!("{}.pb", snapshot.snapshot_id));
        let _ = fs::remove_file(&path);
        let written = write_and_verify_local_snapshot_v0(&snapshot, output_dir).unwrap();
        assert_eq!(written, path);
        let stored = read_local_snapshot_protobuf_v1(&path).unwrap();
        assert_eq!(stored.snapshot_id, snapshot.snapshot_id);
        assert_eq!(stored.content_digest, snapshot.content_digest);
        fs::remove_file(&path).unwrap();

        let mut invalid = snapshot;
        invalid.snapshot_id = "snapshot-upbit-local-write-invalid".to_string();
        invalid.content_digest = "invalid".to_string();
        let invalid_path = output_dir.join(format!("{}.pb", invalid.snapshot_id));
        let _ = fs::remove_file(&invalid_path);
        assert!(write_and_verify_local_snapshot_v0(&invalid, output_dir).is_err());
        assert!(!invalid_path.exists());
    }

    #[test]
    fn protobuf_snapshot_round_trip_is_storage_independent_and_detects_corruption() {
        let dataset = parse_upbit_daily_ohlcv_v0(
            r#"[{"market":"KRW-BTC","candle_date_time_utc":"2024-01-01T00:00:00","opening_price":1.0,"high_price":2.0,"low_price":0.5,"trade_price":1.5,"candle_acc_trade_volume":1.0}]"#,
            "KRW-BTC",
        )
        .unwrap();
        let snapshot = local_snapshot(dataset, "unused");
        let protobuf = SnapshotCodec::protobuf_v1().encode(&snapshot).unwrap();
        let json = SnapshotCodec::json_legacy_v0().encode(&snapshot).unwrap();
        assert_ne!(protobuf, json);
        assert_eq!(
            SnapshotCodec::protobuf_v1().decode(&protobuf).unwrap(),
            snapshot
        );
        let mut corrupt = protobuf;
        let last = corrupt.len() - 1;
        corrupt[last] ^= 1;
        assert!(SnapshotCodec::protobuf_v1().decode(&corrupt).is_err());
    }

    #[test]
    fn legacy_json_migration_writes_verified_protobuf_sidecar_without_overwrite() {
        let dataset = parse_upbit_daily_ohlcv_v0(
            r#"[{"market":"KRW-BTC","candle_date_time_utc":"2024-01-01T00:00:00","opening_price":1.0,"high_price":2.0,"low_price":0.5,"trade_price":1.5,"candle_acc_trade_volume":1.0}]"#,
            "KRW-BTC",
        )
        .unwrap();
        let snapshot = local_snapshot(dataset, "unused");
        let output_dir = Path::new(DEFAULT_SNAPSHOT_OUTPUT_DIR)
            .join(format!("legacy-migration-{}", std::process::id()));
        fs::create_dir_all(&output_dir).unwrap();
        let legacy_path = output_dir.join("snapshot-upbit-legacy-migration.json");
        let _ = fs::remove_file(&legacy_path);
        let protobuf_path = output_dir.join(format!("{}.pb", snapshot.snapshot_id));
        let _ = fs::remove_file(&protobuf_path);
        fs::write(
            &legacy_path,
            SnapshotCodec::json_legacy_v0().encode(&snapshot).unwrap(),
        )
        .unwrap();
        let migrated = migrate_legacy_json_snapshot_v0(&legacy_path).unwrap();
        assert_eq!(migrated, protobuf_path);
        assert!(legacy_path.exists());
        assert_eq!(
            read_local_snapshot_protobuf_v1(&migrated).unwrap(),
            snapshot
        );
        fs::remove_file(&legacy_path).unwrap();
        fs::remove_file(&migrated).unwrap();
        fs::remove_dir(&output_dir).unwrap();
    }

    #[test]
    fn canonical_identity_normalizes_negative_zero_but_preserves_nonzero_float_bits() {
        let mut negative_zero = parse_upbit_daily_ohlcv_v0(
            r#"[{"market":"KRW-BTC","candle_date_time_utc":"2024-01-01T00:00:00","opening_price":1.0,"high_price":2.0,"low_price":0.5,"trade_price":1.5,"candle_acc_trade_volume":1.0}]"#,
            "KRW-BTC",
        )
        .unwrap();
        let positive = negative_zero.clone();
        negative_zero.rows[0].trade_value = Some(-0.0);
        let mut positive_zero = positive;
        positive_zero.rows[0].trade_value = Some(0.0);
        assert_eq!(
            dataset_digest(&negative_zero),
            dataset_digest(&positive_zero)
        );
        positive_zero.rows[0].close = f64::from_bits(positive_zero.rows[0].close.to_bits() + 1);
        assert_ne!(
            dataset_digest(&negative_zero),
            dataset_digest(&positive_zero)
        );
    }

    #[test]
    fn protobuf_storage_measurement_uses_sanitized_synthetic_rows() {
        fn measured_snapshot(rows: usize) -> DataSnapshot {
            let dataset = HistoricalReplayDataset {
                symbol: "SYNTH".to_string(),
                source: "sanitized-synthetic-measurement".to_string(),
                rows: (0..rows)
                    .map(|index| {
                        let base = 100.0 + index as f64;
                        HistoricalOhlcvRow {
                            symbol: "SYNTH".to_string(),
                            timestamp_ms: 1_700_000_000_000 + index as u64 * 86_400_000,
                            open: base,
                            high: base + 1.0,
                            low: base - 1.0,
                            close: base + 0.5,
                            volume: 10.0 + index as f64,
                            trade_value: Some(base * 10.0),
                        }
                    })
                    .collect(),
                reason_codes: vec![],
            };
            local_snapshot(dataset, "unused")
        }

        for rows in [16, 128, 1_024] {
            let snapshot = measured_snapshot(rows);
            let json = SnapshotCodec::json_legacy_v0().encode(&snapshot).unwrap();
            let protobuf = SnapshotCodec::protobuf_v1().encode(&snapshot).unwrap();
            let mut json_samples = Vec::new();
            let mut protobuf_samples = Vec::new();
            for iteration in 0..10 {
                let start = std::time::Instant::now();
                let _ = SnapshotCodec::json_legacy_v0().encode(&snapshot).unwrap();
                let json_elapsed = start.elapsed().as_nanos();
                let start = std::time::Instant::now();
                let _ = SnapshotCodec::protobuf_v1().encode(&snapshot).unwrap();
                let protobuf_elapsed = start.elapsed().as_nanos();
                if iteration >= 2 {
                    json_samples.push(json_elapsed);
                    protobuf_samples.push(protobuf_elapsed);
                }
            }
            json_samples.sort_unstable();
            protobuf_samples.sort_unstable();
            println!(
                "rows={rows} json_bytes={} protobuf_bytes={} json_median_ns={} protobuf_median_ns={}",
                json.len(),
                protobuf.len(),
                json_samples[json_samples.len() / 2],
                protobuf_samples[protobuf_samples.len() / 2],
            );
            assert!(protobuf.len() < json.len());
        }
    }

    #[test]
    fn page_merge_is_chronological_deduplicated_and_conflicts_fail() {
        let first = parse_upbit_daily_ohlcv_v0(
            r#"[
              {"market":"KRW-BTC","candle_date_time_utc":"2024-01-03T00:00:00","opening_price":3.0,"high_price":4.0,"low_price":2.0,"trade_price":3.5,"candle_acc_trade_volume":1.0},
              {"market":"KRW-BTC","candle_date_time_utc":"2024-01-02T00:00:00","opening_price":2.0,"high_price":3.0,"low_price":1.0,"trade_price":2.5,"candle_acc_trade_volume":1.0}
            ]"#,
            "KRW-BTC",
        )
        .unwrap();
        let second = parse_upbit_daily_ohlcv_v0(
            r#"[
              {"market":"KRW-BTC","candle_date_time_utc":"2024-01-02T00:00:00","opening_price":2.0,"high_price":3.0,"low_price":1.0,"trade_price":2.5,"candle_acc_trade_volume":1.0},
              {"market":"KRW-BTC","candle_date_time_utc":"2024-01-01T00:00:00","opening_price":1.0,"high_price":2.0,"low_price":0.5,"trade_price":1.5,"candle_acc_trade_volume":1.0}
            ]"#,
            "KRW-BTC",
        )
        .unwrap();
        let (merged, duplicates) =
            merge_upbit_historical_pages_v0(&[first.clone(), second], "KRW-BTC").unwrap();
        assert_eq!(merged.rows.len(), 3);
        assert_eq!(duplicates, 1);
        assert!(
            merged
                .rows
                .windows(2)
                .all(|pair| pair[0].timestamp_ms < pair[1].timestamp_ms)
        );

        let mut conflicting = first;
        conflicting.rows[0].close = 99.0;
        assert!(merge_upbit_historical_pages_v0(&[merged, conflicting], "KRW-BTC").is_err());
    }
}
