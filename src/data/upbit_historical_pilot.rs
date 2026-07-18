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
    thread,
    time::{Duration, Instant},
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
    #[serde(default = "default_minimum_inter_request_delay_ms")]
    pub minimum_inter_request_delay_ms: u64,
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
            || self.minimum_inter_request_delay_ms == 0
            || self.minimum_inter_request_delay_ms > 60_000
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

fn default_minimum_inter_request_delay_ms() -> u64 {
    250
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BackfillRequestPlanStatusV0 {
    Ready,
    ExistingEvidenceSufficient,
    #[default]
    RequestBudgetRejected,
    InvalidConfiguration,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EthicalExternalRequestBudgetV0 {
    pub maximum_requests: usize,
    pub maximum_pages: usize,
    pub maximum_total_rows: usize,
    pub maximum_total_response_bytes: usize,
    pub maximum_wall_clock_seconds: u64,
    pub minimum_inter_request_delay_ms: u64,
    pub maximum_transient_retries_per_request: usize,
    pub stop_on_rate_limit: bool,
    pub stop_on_permission_error: bool,
}

impl EthicalExternalRequestBudgetV0 {
    pub fn validate(&self) -> bool {
        self.maximum_requests > 0
            && self.maximum_pages > 0
            && self.maximum_total_rows > 0
            && self.maximum_total_response_bytes > 0
            && self.maximum_wall_clock_seconds > 0
            && self.minimum_inter_request_delay_ms > 0
            && self.minimum_inter_request_delay_ms <= 60_000
            && self.maximum_transient_retries_per_request <= 3
            && self.stop_on_rate_limit
            && self.stop_on_permission_error
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BtcRegimeEvidenceRequirementV0 {
    pub existing_rows: usize,
    pub minimum_regimes: usize,
    pub rows_per_regime: usize,
    pub inter_regime_gap_rows: usize,
    pub edge_allowance_rows: usize,
    pub required_total_rows: usize,
    pub additional_rows_required: usize,
    pub configured_page_size: usize,
    pub estimated_minimum_pages: usize,
    pub configured_maximum_pages: usize,
    pub configured_maximum_requests: usize,
    pub plan_status: BackfillRequestPlanStatusV0,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SanitizedUpbitBackfillDryRunV0 {
    pub endpoint_category: String,
    pub sequential_only: bool,
    pub rate_limit_stops_immediately: bool,
    pub existing_rows: usize,
    pub required_total_rows: usize,
    pub additional_rows_required: usize,
    pub estimated_minimum_requests: usize,
    pub maximum_permitted_requests: usize,
    pub page_size: usize,
    pub minimum_inter_request_delay_ms: u64,
    pub projected_regime_count: usize,
    pub plan_status: BackfillRequestPlanStatusV0,
    pub plan_digest: String,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HistoricalConflictForensicsStatusV0 {
    ConflictArtifactUnavailable,
    ConflictReproduced,
    FullPageOverlap,
    BoundaryOverlapOnly,
    IncompleteBarMutation,
    CompletedBarProviderRevision,
    CursorPlanningBug,
    TimestampBoundaryBug,
    CanonicalNormalizationMismatch,
    ExistingSnapshotDefect,
    MixedConflictCauses,
    ConflictRootCauseIdentified,
    #[default]
    Unknown,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HistoricalMergeConflictRootCauseV0 {
    RequestCursorOverlappedExistingRange,
    CurrentOrIncompleteDailyBarChanged,
    ProviderRevisedFinalizedBar,
    TimestampConversionMismatch,
    CanonicalNormalizationVersionMismatch,
    SymbolOrMarketMismatch,
    ExistingSnapshotCorruption,
    FetchedPageCorruption,
    DuplicatePolicyBug,
    MultipleCauses,
    #[default]
    Unknown,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DailyBarFinalityStatusV0 {
    Finalized,
    PotentiallyOpen,
    ContractBoundaryAmbiguous,
    #[default]
    InsufficientMetadata,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StrictHistoricalRequestPlanStatusV0 {
    ReadyZeroOverlap,
    ExistingEvidenceAlreadySufficient,
    InvalidExistingRange,
    CursorNotStrictlyOlder,
    ExpectedOverlapNonZero,
    RequestCountInvalid,
    RequestBudgetRejected,
    #[default]
    ContractSemanticsUnavailable,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StrictOlderPageExecutionStatusV0 {
    #[default]
    NotAttempted,
    NetworkConsentRequired,
    PreflightBlocked,
    RequestExecuted,
    RateLimitedStopped,
    PermissionDeniedStopped,
    TransportFailure,
    ProviderFailure,
    ParseFailure,
    UnexpectedOverlap,
    ReturnedRangeNotOlder,
    ValidationFailure,
    OlderPageAccepted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HistoricalConflictFieldV0 {
    Symbol,
    Timestamp,
    Open,
    High,
    Low,
    Close,
    Volume,
    TradeValue,
    MarketScope,
    DatasetKind,
    SchemaVersion,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoricalFieldConflictCountV0 {
    pub field: HistoricalConflictFieldV0,
    pub count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoricalDuplicateConflictReportV0 {
    pub accepted_row_count: usize,
    pub fetched_row_count: usize,
    pub overlapping_timestamp_count: usize,
    pub identical_duplicate_count: usize,
    pub conflicting_duplicate_count: usize,
    pub first_conflict_timestamp_class: String,
    pub first_conflicting_field: Option<HistoricalConflictFieldV0>,
    pub conflicting_field_counts: Vec<HistoricalFieldConflictCountV0>,
    pub finalized_conflict_count: usize,
    pub potentially_open_conflict_count: usize,
    pub previous_request_cursor_class: String,
    pub root_cause: HistoricalMergeConflictRootCauseV0,
    pub forensic_status: HistoricalConflictForensicsStatusV0,
    pub report_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoricalCursorProofV0 {
    pub existing_oldest_timestamp: u64,
    pub requested_exclusive_end: u64,
    pub expected_relation: String,
    pub expected_overlap_rows: usize,
    pub requested_count: usize,
    pub additional_rows_required: usize,
    pub provider_count_limit: usize,
    pub proof_status: StrictHistoricalRequestPlanStatusV0,
    pub proof_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StrictOlderPageValidationV0 {
    pub status: StrictOlderPageExecutionStatusV0,
    pub returned_row_count: usize,
    pub overlapping_timestamp_count: usize,
    pub returned_range_relation: String,
}

pub fn ethical_upbit_request_budget_v0(
    config: &UpbitHistoricalPilotConfigV0,
) -> EthicalExternalRequestBudgetV0 {
    EthicalExternalRequestBudgetV0 {
        maximum_requests: config.maximum_pages,
        maximum_pages: config.maximum_pages,
        maximum_total_rows: config.page_size.saturating_mul(config.maximum_pages),
        maximum_total_response_bytes: config
            .maximum_response_bytes
            .saturating_mul(config.maximum_pages),
        maximum_wall_clock_seconds: config
            .timeout_seconds
            .saturating_mul(config.maximum_pages as u64)
            .saturating_mul(config.max_retries.saturating_add(1) as u64),
        minimum_inter_request_delay_ms: config.minimum_inter_request_delay_ms,
        maximum_transient_retries_per_request: config.max_retries,
        stop_on_rate_limit: true,
        stop_on_permission_error: true,
    }
}

pub fn plan_btc_regime_backfill_v0(
    existing_rows: usize,
    minimum_regimes: usize,
    rows_per_regime: usize,
    inter_regime_gap_rows: usize,
    edge_allowance_rows: usize,
    config: &UpbitHistoricalPilotConfigV0,
    budget: &EthicalExternalRequestBudgetV0,
) -> BtcRegimeEvidenceRequirementV0 {
    let required_total_rows = minimum_regimes
        .saturating_mul(rows_per_regime)
        .saturating_add(
            minimum_regimes
                .saturating_sub(1)
                .saturating_mul(inter_regime_gap_rows),
        )
        .saturating_add(edge_allowance_rows);
    let additional_rows_required = required_total_rows.saturating_sub(existing_rows);
    let estimated_minimum_pages = if additional_rows_required == 0 || config.page_size == 0 {
        0
    } else {
        additional_rows_required.div_ceil(config.page_size)
    };
    let plan_status = if config.validate().is_err()
        || !budget.validate()
        || minimum_regimes == 0
        || rows_per_regime == 0
    {
        BackfillRequestPlanStatusV0::InvalidConfiguration
    } else if additional_rows_required == 0 {
        BackfillRequestPlanStatusV0::ExistingEvidenceSufficient
    } else if estimated_minimum_pages > config.maximum_pages
        || estimated_minimum_pages > budget.maximum_pages
        || estimated_minimum_pages > budget.maximum_requests
        || additional_rows_required > budget.maximum_total_rows
    {
        BackfillRequestPlanStatusV0::RequestBudgetRejected
    } else {
        BackfillRequestPlanStatusV0::Ready
    };
    BtcRegimeEvidenceRequirementV0 {
        existing_rows,
        minimum_regimes,
        rows_per_regime,
        inter_regime_gap_rows,
        edge_allowance_rows,
        required_total_rows,
        additional_rows_required,
        configured_page_size: config.page_size,
        estimated_minimum_pages,
        configured_maximum_pages: config.maximum_pages,
        configured_maximum_requests: budget.maximum_requests,
        plan_status,
    }
}

pub fn sanitized_upbit_backfill_dry_run_v0(
    requirement: &BtcRegimeEvidenceRequirementV0,
    budget: &EthicalExternalRequestBudgetV0,
) -> SanitizedUpbitBackfillDryRunV0 {
    let material = format!(
        "{}:{}:{}:{}:{}:{}:{}:{:?}",
        requirement.existing_rows,
        requirement.required_total_rows,
        requirement.additional_rows_required,
        requirement.estimated_minimum_pages,
        budget.maximum_requests,
        requirement.configured_page_size,
        budget.minimum_inter_request_delay_ms,
        requirement.plan_status,
    );
    SanitizedUpbitBackfillDryRunV0 {
        endpoint_category: "upbit_public_daily_btc".to_string(),
        sequential_only: true,
        rate_limit_stops_immediately: budget.stop_on_rate_limit,
        existing_rows: requirement.existing_rows,
        required_total_rows: requirement.required_total_rows,
        additional_rows_required: requirement.additional_rows_required,
        estimated_minimum_requests: requirement.estimated_minimum_pages,
        maximum_permitted_requests: budget.maximum_requests,
        page_size: requirement.configured_page_size,
        minimum_inter_request_delay_ms: budget.minimum_inter_request_delay_ms,
        projected_regime_count: requirement.minimum_regimes,
        plan_status: requirement.plan_status,
        plan_digest: stable_hash_string(&material),
    }
}

pub fn inspect_upbit_duplicate_conflict_v0(
    accepted: &DataSnapshot,
    fetched: &DataSnapshot,
) -> Result<HistoricalDuplicateConflictReportV0, String> {
    if accepted.normalized_dataset.symbol != fetched.normalized_dataset.symbol
        || accepted.market_scope != fetched.market_scope
        || accepted.dataset_kind != fetched.dataset_kind
    {
        return Err("upbit conflict artifacts are not comparable".to_string());
    }
    let accepted_rows = accepted
        .normalized_dataset
        .rows
        .iter()
        .map(|row| (row.timestamp_ms, row))
        .collect::<BTreeMap<_, _>>();
    let mut fields = BTreeMap::<HistoricalConflictFieldV0, usize>::new();
    let mut overlap = 0usize;
    let mut identical = 0usize;
    let mut conflicting = 0usize;
    let mut finalized = 0usize;
    let mut potentially_open = 0usize;
    let mut first_field = None;
    for fetched_row in &fetched.normalized_dataset.rows {
        let Some(accepted_row) = accepted_rows.get(&fetched_row.timestamp_ms) else {
            continue;
        };
        overlap += 1;
        let differences = canonical_row_differences_v0(accepted_row, fetched_row);
        if differences.is_empty() {
            identical += 1;
            continue;
        }
        conflicting += 1;
        for field in differences {
            *fields.entry(field).or_default() += 1;
            if first_field.is_none() {
                first_field = Some(field);
            }
        }
        let accepted_finality =
            daily_bar_finality_v0(accepted_row.timestamp_ms, accepted.fetched_at_ms);
        let fetched_finality =
            daily_bar_finality_v0(fetched_row.timestamp_ms, fetched.fetched_at_ms);
        if accepted_finality == DailyBarFinalityStatusV0::Finalized
            && fetched_finality == DailyBarFinalityStatusV0::Finalized
        {
            finalized += 1;
        } else {
            potentially_open += 1;
        }
    }
    let existing_oldest = accepted
        .normalized_dataset
        .rows
        .first()
        .map(|row| row.timestamp_ms)
        .ok_or_else(|| "accepted snapshot has no rows".to_string())?;
    let previous_cursor = fetched.requested_lookback.end_timestamp_ms;
    let cursor_overlapped = previous_cursor.is_some_and(|cursor| cursor > existing_oldest);
    let root_cause = if conflicting == 0 {
        HistoricalMergeConflictRootCauseV0::Unknown
    } else if cursor_overlapped {
        HistoricalMergeConflictRootCauseV0::RequestCursorOverlappedExistingRange
    } else if potentially_open > 0 {
        HistoricalMergeConflictRootCauseV0::CurrentOrIncompleteDailyBarChanged
    } else {
        HistoricalMergeConflictRootCauseV0::ProviderRevisedFinalizedBar
    };
    let forensic_status = if conflicting == 0 {
        HistoricalConflictForensicsStatusV0::ConflictReproduced
    } else if cursor_overlapped {
        HistoricalConflictForensicsStatusV0::CursorPlanningBug
    } else if potentially_open > 0 {
        HistoricalConflictForensicsStatusV0::IncompleteBarMutation
    } else {
        HistoricalConflictForensicsStatusV0::CompletedBarProviderRevision
    };
    let field_counts = fields
        .into_iter()
        .map(|(field, count)| HistoricalFieldConflictCountV0 { field, count })
        .collect::<Vec<_>>();
    let cursor_class = if cursor_overlapped {
        "at_or_after_existing_oldest"
    } else if previous_cursor == Some(existing_oldest) {
        "exclusive_existing_oldest"
    } else {
        "unavailable_or_older"
    };
    let material = format!(
        "{}:{}:{}:{}:{}:{:?}:{:?}:{}:{}:{}:{:?}:{:?}",
        accepted.row_count,
        fetched.row_count,
        overlap,
        identical,
        conflicting,
        first_field,
        field_counts,
        finalized,
        potentially_open,
        cursor_class,
        root_cause,
        forensic_status,
    );
    Ok(HistoricalDuplicateConflictReportV0 {
        accepted_row_count: accepted.row_count,
        fetched_row_count: fetched.row_count,
        overlapping_timestamp_count: overlap,
        identical_duplicate_count: identical,
        conflicting_duplicate_count: conflicting,
        first_conflict_timestamp_class: if conflicting == 0 {
            "none".to_string()
        } else {
            "daily_utc_bucket".to_string()
        },
        first_conflicting_field: first_field,
        conflicting_field_counts: field_counts,
        finalized_conflict_count: finalized,
        potentially_open_conflict_count: potentially_open,
        previous_request_cursor_class: cursor_class.to_string(),
        root_cause,
        forensic_status,
        report_digest: stable_hash_string(&material),
    })
}

pub fn build_strict_older_cursor_proof_v0(
    existing: &DataSnapshot,
    additional_rows_required: usize,
    config: &UpbitHistoricalPilotConfigV0,
) -> HistoricalCursorProofV0 {
    let existing_oldest_timestamp = existing
        .normalized_dataset
        .rows
        .first()
        .map(|row| row.timestamp_ms)
        .unwrap_or_default();
    let requested_exclusive_end = existing_oldest_timestamp;
    let provider_count_limit = UPBIT_MAX_CANDLES_PER_REQUEST.min(config.page_size);
    let requested_count = additional_rows_required.min(provider_count_limit);
    let proof_status = if existing_oldest_timestamp == 0
        || existing
            .normalized_dataset
            .rows
            .windows(2)
            .any(|pair| pair[0].timestamp_ms >= pair[1].timestamp_ms)
    {
        StrictHistoricalRequestPlanStatusV0::InvalidExistingRange
    } else if additional_rows_required == 0 {
        StrictHistoricalRequestPlanStatusV0::ExistingEvidenceAlreadySufficient
    } else if requested_exclusive_end <= config.start_timestamp_ms {
        StrictHistoricalRequestPlanStatusV0::CursorNotStrictlyOlder
    } else if requested_count == 0 || requested_count != additional_rows_required {
        StrictHistoricalRequestPlanStatusV0::RequestCountInvalid
    } else if config.maximum_pages != 1 {
        StrictHistoricalRequestPlanStatusV0::RequestBudgetRejected
    } else {
        StrictHistoricalRequestPlanStatusV0::ReadyZeroOverlap
    };
    let material = format!(
        "{}:{}:{}:{}:{}:{}:{:?}",
        existing_oldest_timestamp,
        requested_exclusive_end,
        additional_rows_required,
        requested_count,
        provider_count_limit,
        0,
        proof_status,
    );
    HistoricalCursorProofV0 {
        existing_oldest_timestamp,
        requested_exclusive_end,
        expected_relation: "all_returned_rows_strictly_older".to_string(),
        expected_overlap_rows: 0,
        requested_count,
        additional_rows_required,
        provider_count_limit,
        proof_status,
        proof_digest: stable_hash_string(&material),
    }
}

pub fn validate_strictly_older_upbit_page_v0(
    existing: &DataSnapshot,
    fetched: &DataSnapshot,
    expected_count: usize,
) -> StrictOlderPageValidationV0 {
    let Some(existing_oldest) = existing
        .normalized_dataset
        .rows
        .first()
        .map(|row| row.timestamp_ms)
    else {
        return strict_page_validation(
            StrictOlderPageExecutionStatusV0::ValidationFailure,
            fetched.row_count,
            0,
        );
    };
    let overlap = fetched
        .normalized_dataset
        .rows
        .iter()
        .filter(|row| row.timestamp_ms == existing_oldest)
        .count();
    if fetched.row_count != expected_count {
        return strict_page_validation(
            StrictOlderPageExecutionStatusV0::ValidationFailure,
            fetched.row_count,
            overlap,
        );
    }
    if overlap > 0 {
        return strict_page_validation(
            StrictOlderPageExecutionStatusV0::UnexpectedOverlap,
            fetched.row_count,
            overlap,
        );
    }
    if fetched
        .normalized_dataset
        .rows
        .iter()
        .any(|row| row.timestamp_ms >= existing_oldest)
    {
        return strict_page_validation(
            StrictOlderPageExecutionStatusV0::ReturnedRangeNotOlder,
            fetched.row_count,
            0,
        );
    }
    strict_page_validation(
        StrictOlderPageExecutionStatusV0::OlderPageAccepted,
        fetched.row_count,
        0,
    )
}

fn canonical_row_differences_v0(
    accepted: &HistoricalOhlcvRow,
    fetched: &HistoricalOhlcvRow,
) -> Vec<HistoricalConflictFieldV0> {
    let mut fields = Vec::new();
    if accepted.symbol != fetched.symbol {
        fields.push(HistoricalConflictFieldV0::Symbol);
    }
    if accepted.timestamp_ms != fetched.timestamp_ms {
        fields.push(HistoricalConflictFieldV0::Timestamp);
    }
    if accepted.open.to_bits() != fetched.open.to_bits() {
        fields.push(HistoricalConflictFieldV0::Open);
    }
    if accepted.high.to_bits() != fetched.high.to_bits() {
        fields.push(HistoricalConflictFieldV0::High);
    }
    if accepted.low.to_bits() != fetched.low.to_bits() {
        fields.push(HistoricalConflictFieldV0::Low);
    }
    if accepted.close.to_bits() != fetched.close.to_bits() {
        fields.push(HistoricalConflictFieldV0::Close);
    }
    if accepted.volume.to_bits() != fetched.volume.to_bits() {
        fields.push(HistoricalConflictFieldV0::Volume);
    }
    if accepted.trade_value.map(f64::to_bits) != fetched.trade_value.map(f64::to_bits) {
        fields.push(HistoricalConflictFieldV0::TradeValue);
    }
    fields
}

fn daily_bar_finality_v0(timestamp_ms: u64, acquired_at_ms: u64) -> DailyBarFinalityStatusV0 {
    const DAILY_MS: u64 = 86_400_000;
    if timestamp_ms % DAILY_MS != 0 {
        DailyBarFinalityStatusV0::ContractBoundaryAmbiguous
    } else if acquired_at_ms >= timestamp_ms.saturating_add(DAILY_MS) {
        DailyBarFinalityStatusV0::Finalized
    } else {
        DailyBarFinalityStatusV0::PotentiallyOpen
    }
}

fn strict_page_validation(
    status: StrictOlderPageExecutionStatusV0,
    returned_row_count: usize,
    overlapping_timestamp_count: usize,
) -> StrictOlderPageValidationV0 {
    StrictOlderPageValidationV0 {
        status,
        returned_row_count,
        overlapping_timestamp_count,
        returned_range_relation: match status {
            StrictOlderPageExecutionStatusV0::OlderPageAccepted => "strictly_older".to_string(),
            StrictOlderPageExecutionStatusV0::UnexpectedOverlap => {
                "equal_timestamp_overlap".to_string()
            }
            StrictOlderPageExecutionStatusV0::ReturnedRangeNotOlder => {
                "at_or_after_existing_oldest".to_string()
            }
            _ => "invalid".to_string(),
        },
    }
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
    RateLimitedStopped,
    PermissionDeniedStopped,
    RequestBudgetRejected,
    TransientFailureStopped,
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
                "--write-out",
                "\n%{http_code}",
                &url,
            ])
            .output()
            .map_err(|_| ProviderFetchFailure::Unavailable)?;
        if !output.status.success() || output.stdout.len() > self.config.maximum_response_bytes {
            return Err(ProviderFetchFailure::Unavailable);
        }
        let output =
            String::from_utf8(output.stdout).map_err(|_| ProviderFetchFailure::InvalidResponse)?;
        let (body, status) = output
            .rsplit_once('\n')
            .ok_or(ProviderFetchFailure::InvalidResponse)?;
        if let Some(failure) = upbit_http_failure_v0(status.trim().parse::<u16>().ok()) {
            return Err(failure);
        }
        if body.len() > self.config.maximum_response_bytes {
            return Err(ProviderFetchFailure::Unavailable);
        }
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

fn upbit_http_failure_v0(status: Option<u16>) -> Option<ProviderFetchFailure> {
    match status {
        Some(200..=299) => None,
        Some(429) => Some(ProviderFetchFailure::RateLimited),
        Some(401 | 403) => Some(ProviderFetchFailure::PermissionDenied),
        _ => Some(ProviderFetchFailure::Unavailable),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProspectiveBlindAcquisitionStopReasonV0 {
    AwaitingNetworkConsent,
    NoFinalizedDailyBoundary,
    ProviderUnavailable,
    RateLimited,
    PermissionDenied,
    InvalidProviderResponse,
    NoAdmissibleFinalizedRow,
    RowAdmitted,
}

/// Sanitized reconstruction stages for a completed prospective provider attempt.
///
/// Receipts created before this schema intentionally contain only aggregate outcome
/// fields.  In that case the classifier fails closed instead of inventing a more
/// specific transport or provider diagnosis.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProspectiveProviderPipelineStageV0 {
    RequestPlan,
    ConsentPreflight,
    TransportConstruction,
    DnsResolution,
    TlsHandshake,
    HttpRequest,
    HttpStatusValidation,
    ContentTypeValidation,
    ResponseSizeValidation,
    ResponseParsing,
    ProviderSemanticValidation,
    CanonicalNormalization,
    DailyFinalityValidation,
    CutoffValidation,
    DuplicateValidation,
    VaultAdmission,
    EvidenceInsufficient,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProspectiveProviderRejectionRootCauseV0 {
    NoActualProviderRequest,
    RequestContractMismatch,
    InvalidRequestParameter,
    InvalidCursorOrTimeBoundary,
    UnsupportedCount,
    UnexpectedSymbolOrMarket,
    HttpAuthenticationFailure,
    HttpPermissionDenied,
    HttpRateLimited,
    HttpClientErrorOther,
    HttpServerError,
    DnsFailure,
    TlsFailure,
    Timeout,
    InvalidContentType,
    OversizedResponse,
    ParserSchemaMismatch,
    ProviderSemanticError,
    EmptyFinalizedRange,
    OpenCandleOnly,
    CutoffOrEarlierRowsOnly,
    DuplicateRowsOnly,
    UnexpectedFutureRange,
    CanonicalValidationFailure,
    VaultAdmissionFailure,
    LocalImplementationDefect,
    MultipleCauses,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SharedEpochEligibilityV0 {
    EligibleWithoutCodeChange,
    EligibleAfterCommittedLocalFix,
    EligibleAfterDocumentedProviderCooldown,
    NoFinalizedRowExpectedYet,
    PermanentlyBlockedByPermission,
    BlockedByRateLimitPolicy,
    BlockedByContractUncertainty,
    BlockedByParserUncertainty,
    BlockedByIntegrityFailure,
    BlockedByUnknownCause,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SanitizedProviderStatusClassV0 {
    NoRequest,
    Success,
    FailureWithoutStage,
    InvalidReceipt,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PriorProspectiveRejectionForensicsV0 {
    pub receipt_digest: String,
    pub receipt_version: String,
    pub request_attempted: bool,
    pub request_count: usize,
    pub retry_count: usize,
    pub first_rejected_stage: ProspectiveProviderPipelineStageV0,
    pub status_class: SanitizedProviderStatusClassV0,
    pub parser_invoked: bool,
    pub normalized_row_count: usize,
    pub admitted_row_count: usize,
    pub root_cause: ProspectiveProviderRejectionRootCauseV0,
    pub eligibility: SharedEpochEligibilityV0,
    pub sufficient_for_classification: bool,
    pub reason_codes: Vec<String>,
    pub forensic_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProspectiveBlindAcquisitionReceiptV0 {
    pub challenge_id: String,
    pub request_attempted: bool,
    pub request_count: usize,
    pub response_status_class: Option<String>,
    pub normalized_row_count: usize,
    pub finalized_row_count: usize,
    pub admitted_row_count: usize,
    pub rejected_open_row_count: usize,
    pub rejected_cutoff_row_count: usize,
    pub rejected_duplicate_row_count: usize,
    pub stop_reason: ProspectiveBlindAcquisitionStopReasonV0,
    pub receipt_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProspectiveBlindAcquisitionResultV0 {
    pub receipt: ProspectiveBlindAcquisitionReceiptV0,
    pub admitted_rows: Vec<(u64, String)>,
}

fn prospective_receipt_digest_v0(receipt: &ProspectiveBlindAcquisitionReceiptV0) -> String {
    stable_hash_string(&format!(
        "{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}",
        receipt.challenge_id,
        receipt.request_attempted,
        receipt.request_count,
        receipt.response_status_class.as_deref().unwrap_or_default(),
        receipt.normalized_row_count,
        receipt.finalized_row_count,
        receipt.admitted_row_count,
        receipt.rejected_open_row_count,
        receipt.rejected_cutoff_row_count,
        receipt.rejected_duplicate_row_count,
        format!("{:?}", receipt.stop_reason),
    ))
}

pub fn verify_prospective_blind_acquisition_receipt_v0(
    receipt: &ProspectiveBlindAcquisitionReceiptV0,
) -> bool {
    receipt.request_count <= 1
        && (!receipt.request_attempted || receipt.request_count == 1)
        && receipt.admitted_row_count <= receipt.finalized_row_count
        && receipt.receipt_digest == prospective_receipt_digest_v0(receipt)
}

pub fn classify_prior_prospective_rejection_v0(
    receipt: &ProspectiveBlindAcquisitionReceiptV0,
) -> PriorProspectiveRejectionForensicsV0 {
    let valid = verify_prospective_blind_acquisition_receipt_v0(receipt);
    let (status_class, first_rejected_stage, root_cause, eligibility, sufficient, reason_codes) =
        if !valid {
            (
                SanitizedProviderStatusClassV0::InvalidReceipt,
                ProspectiveProviderPipelineStageV0::EvidenceInsufficient,
                ProspectiveProviderRejectionRootCauseV0::Unknown,
                SharedEpochEligibilityV0::BlockedByIntegrityFailure,
                false,
                vec!["receipt_integrity_invalid".to_string()],
            )
        } else if !receipt.request_attempted {
            (
                SanitizedProviderStatusClassV0::NoRequest,
                ProspectiveProviderPipelineStageV0::ConsentPreflight,
                ProspectiveProviderRejectionRootCauseV0::NoActualProviderRequest,
                SharedEpochEligibilityV0::BlockedByUnknownCause,
                false,
                vec!["no_actual_provider_request".to_string()],
            )
        } else if receipt.stop_reason == ProspectiveBlindAcquisitionStopReasonV0::RateLimited {
            (
                SanitizedProviderStatusClassV0::FailureWithoutStage,
                ProspectiveProviderPipelineStageV0::HttpStatusValidation,
                ProspectiveProviderRejectionRootCauseV0::HttpRateLimited,
                SharedEpochEligibilityV0::BlockedByRateLimitPolicy,
                true,
                vec!["rate_limit_receipt".to_string()],
            )
        } else if receipt.stop_reason == ProspectiveBlindAcquisitionStopReasonV0::PermissionDenied {
            (
                SanitizedProviderStatusClassV0::FailureWithoutStage,
                ProspectiveProviderPipelineStageV0::HttpStatusValidation,
                ProspectiveProviderRejectionRootCauseV0::HttpPermissionDenied,
                SharedEpochEligibilityV0::PermanentlyBlockedByPermission,
                true,
                vec!["permission_denied_receipt".to_string()],
            )
        } else if receipt.stop_reason
            == ProspectiveBlindAcquisitionStopReasonV0::NoAdmissibleFinalizedRow
            && receipt.normalized_row_count > 0
            && receipt.rejected_open_row_count == receipt.normalized_row_count
        {
            (
                SanitizedProviderStatusClassV0::Success,
                ProspectiveProviderPipelineStageV0::DailyFinalityValidation,
                ProspectiveProviderRejectionRootCauseV0::OpenCandleOnly,
                SharedEpochEligibilityV0::NoFinalizedRowExpectedYet,
                true,
                vec!["open_candle_only".to_string()],
            )
        } else {
            (
                if receipt.response_status_class.as_deref() == Some("success") {
                    SanitizedProviderStatusClassV0::Success
                } else {
                    SanitizedProviderStatusClassV0::FailureWithoutStage
                },
                ProspectiveProviderPipelineStageV0::EvidenceInsufficient,
                ProspectiveProviderRejectionRootCauseV0::Unknown,
                SharedEpochEligibilityV0::BlockedByUnknownCause,
                false,
                vec!["legacy_receipt_lacks_pipeline_stage".to_string()],
            )
        };
    let mut report = PriorProspectiveRejectionForensicsV0 {
        receipt_digest: receipt.receipt_digest.clone(),
        receipt_version: "prospective-blind-acquisition-receipt-v0".to_string(),
        request_attempted: receipt.request_attempted,
        request_count: receipt.request_count,
        retry_count: 0,
        first_rejected_stage,
        status_class,
        parser_invoked: receipt.normalized_row_count > 0,
        normalized_row_count: receipt.normalized_row_count,
        admitted_row_count: receipt.admitted_row_count,
        root_cause,
        eligibility,
        sufficient_for_classification: sufficient,
        reason_codes,
        forensic_digest: String::new(),
    };
    report.forensic_digest = stable_hash_string(&format!(
        "{:?}{:?}",
        (
            &report.receipt_digest,
            &report.receipt_version,
            report.request_attempted,
            report.request_count,
            report.retry_count,
            report.first_rejected_stage,
            report.status_class,
        ),
        (
            report.parser_invoked,
            report.normalized_row_count,
            report.admitted_row_count,
            report.root_cause,
            report.eligibility,
            report.sufficient_for_classification,
            &report.reason_codes,
        )
    ));
    report
}

fn prospective_blind_receipt_v0(
    challenge_id: &str,
    request_attempted: bool,
    request_count: usize,
    response_status_class: Option<String>,
    normalized_row_count: usize,
    finalized_row_count: usize,
    admitted_row_count: usize,
    rejected_open_row_count: usize,
    rejected_cutoff_row_count: usize,
    rejected_duplicate_row_count: usize,
    stop_reason: ProspectiveBlindAcquisitionStopReasonV0,
) -> ProspectiveBlindAcquisitionReceiptV0 {
    let mut receipt = ProspectiveBlindAcquisitionReceiptV0 {
        challenge_id: challenge_id.to_string(),
        request_attempted,
        request_count,
        response_status_class,
        normalized_row_count,
        finalized_row_count,
        admitted_row_count,
        rejected_open_row_count,
        rejected_cutoff_row_count,
        rejected_duplicate_row_count,
        stop_reason,
        receipt_digest: String::new(),
    };
    receipt.receipt_digest = prospective_receipt_digest_v0(&receipt);
    receipt
}

fn canonical_prospective_row_digest_v0(row: &HistoricalOhlcvRow) -> String {
    stable_hash_string(&format!(
        "{}:{}:{}:{}:{}:{}:{}:{}",
        row.symbol,
        row.timestamp_ms,
        row.open.to_bits(),
        row.high.to_bits(),
        row.low.to_bits(),
        row.close.to_bits(),
        row.volume.to_bits(),
        row.trade_value.map(f64::to_bits).unwrap_or_default(),
    ))
}

pub fn acquire_one_blind_upbit_daily_row_v0(
    config_path: &Path,
    challenge_id: &str,
    cutoff_exclusive_timestamp_ms: u64,
    existing_timestamps: &BTreeSet<u64>,
    allow_network: bool,
) -> ProspectiveBlindAcquisitionResultV0 {
    let blocked = |reason| ProspectiveBlindAcquisitionResultV0 {
        receipt: prospective_blind_receipt_v0(
            challenge_id,
            false,
            0,
            None,
            0,
            0,
            0,
            0,
            0,
            0,
            reason,
        ),
        admitted_rows: vec![],
    };
    if !allow_network {
        return blocked(ProspectiveBlindAcquisitionStopReasonV0::AwaitingNetworkConsent);
    }
    let preflight = preflight_upbit_historical_backfill_v0(config_path, true);
    if preflight.status != UpbitHistoricalPreflightStatusV0::Ready {
        return blocked(ProspectiveBlindAcquisitionStopReasonV0::AwaitingNetworkConsent);
    }
    let mut config = match UpbitHistoricalPilotConfigV0::from_toml_path(config_path) {
        Ok(config) if config.validate().is_ok() => config,
        _ => return blocked(ProspectiveBlindAcquisitionStopReasonV0::AwaitingNetworkConsent),
    };
    const DAILY_MS: u64 = 86_400_000;
    let finality_boundary = current_time_ms() / DAILY_MS * DAILY_MS;
    if finality_boundary <= cutoff_exclusive_timestamp_ms.saturating_add(1) {
        return blocked(ProspectiveBlindAcquisitionStopReasonV0::NoFinalizedDailyBoundary);
    }
    config.start_timestamp_ms = cutoff_exclusive_timestamp_ms.saturating_add(1);
    config.end_timestamp_ms = finality_boundary;
    config.max_retries = 0;
    let request = ReadOnlyProviderRequest {
        request_id: format!(
            "prospective-daily-{}",
            stable_hash_string(&format!("{}:{}", challenge_id, finality_boundary))
        ),
        request_key: format!("prospective-daily:{}:{}", config.symbol, finality_boundary),
        provider_id: UPBIT_PROVIDER_ID.to_string(),
        dataset_kind: DatasetKind::DailyOhlcv,
        market_scope: AcquisitionMarketScope::BtcCrypto,
        symbols: vec![config.symbol.clone()],
        lookback: DataLookback {
            bars: 1,
            start_timestamp_ms: Some(config.start_timestamp_ms),
            end_timestamp_ms: Some(finality_boundary),
        },
        cadence: "1d".to_string(),
        max_staleness_ms: u64::MAX,
        reason_codes: vec![],
    };
    let response = UpbitDailyOhlcvProviderV0::new(config).fetch_readonly(&request);
    let response = match response {
        Ok(response) => response,
        Err(error) => {
            let reason = match error {
                ProviderFetchFailure::RateLimited => {
                    ProspectiveBlindAcquisitionStopReasonV0::RateLimited
                }
                ProviderFetchFailure::PermissionDenied => {
                    ProspectiveBlindAcquisitionStopReasonV0::PermissionDenied
                }
                ProviderFetchFailure::InvalidResponse => {
                    ProspectiveBlindAcquisitionStopReasonV0::InvalidProviderResponse
                }
                ProviderFetchFailure::Unavailable | ProviderFetchFailure::TimedOut => {
                    ProspectiveBlindAcquisitionStopReasonV0::ProviderUnavailable
                }
            };
            return ProspectiveBlindAcquisitionResultV0 {
                receipt: prospective_blind_receipt_v0(
                    challenge_id,
                    true,
                    1,
                    Some("failure".to_string()),
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    reason,
                ),
                admitted_rows: vec![],
            };
        }
    };
    let mut finalized = 0usize;
    let mut rejected_open = 0usize;
    let mut rejected_cutoff = 0usize;
    let mut rejected_duplicate = 0usize;
    let mut admitted_rows = Vec::new();
    for row in response.normalized_dataset.rows {
        if row.timestamp_ms <= cutoff_exclusive_timestamp_ms {
            rejected_cutoff += 1;
        } else if existing_timestamps.contains(&row.timestamp_ms) {
            rejected_duplicate += 1;
        } else if daily_bar_finality_v0(row.timestamp_ms, response.fetched_at_ms)
            != DailyBarFinalityStatusV0::Finalized
        {
            rejected_open += 1;
        } else {
            finalized += 1;
            admitted_rows.push((row.timestamp_ms, canonical_prospective_row_digest_v0(&row)));
        }
    }
    admitted_rows.truncate(1);
    let stop_reason = if admitted_rows.is_empty() {
        ProspectiveBlindAcquisitionStopReasonV0::NoAdmissibleFinalizedRow
    } else {
        ProspectiveBlindAcquisitionStopReasonV0::RowAdmitted
    };
    ProspectiveBlindAcquisitionResultV0 {
        receipt: prospective_blind_receipt_v0(
            challenge_id,
            true,
            1,
            Some("success".to_string()),
            finalized
                .saturating_add(rejected_open)
                .saturating_add(rejected_cutoff)
                .saturating_add(rejected_duplicate),
            finalized,
            admitted_rows.len(),
            rejected_open,
            rejected_cutoff,
            rejected_duplicate,
            stop_reason,
        ),
        admitted_rows,
    }
}

// This protocol is intentionally separate from the older blind-acquisition
// receipt.  It has one public read-only request budget and never updates the
// historical backfill machinery or its receipts.
const PROSPECTIVE_PUBLIC_EXPORT_REGISTRATION_VERSION_V0: &str =
    "prospective-public-export-acquisition-registration-v0";
const PROSPECTIVE_PUBLIC_EXPORT_RECEIPT_VERSION_V0: &str =
    "prospective-public-export-acquisition-receipt-v0";
const PROSPECTIVE_NETWORK_EXPORT_CAPSULE_VERSION_V0: &str = "prospective-network-export-capsule-v0";
const PROSPECTIVE_PUBLIC_EXPORT_MAX_TIMEOUT_SECONDS_V0: u64 = 60;
const PROSPECTIVE_PUBLIC_EXPORT_MAX_RESPONSE_BYTES_V0: usize = 1_048_576;
const DAILY_INTERVAL_MS_V0: u64 = 86_400_000;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProspectivePublicExportAcquisitionRegistrationV0 {
    pub registration_version: String,
    pub provider_id: String,
    pub endpoint_origin: String,
    pub endpoint_path: String,
    pub configured_market: String,
    pub cadence: String,
    pub maximum_requests: usize,
    pub maximum_concurrency: usize,
    pub retry_count: usize,
    pub response_candle_count: usize,
    pub timeout_seconds: u64,
    pub maximum_response_bytes: usize,
    pub public_read_only: bool,
    pub credential_free: bool,
    pub api_key_free: bool,
    pub authorization_header_forbidden: bool,
    pub cookies_forbidden: bool,
    pub legacy_blind_receipt_immutable: bool,
    pub legacy_request_registry_immutable: bool,
    pub registration_digest: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProspectivePublicExportAcquisitionOutcomeV0 {
    CapsuleCreated,
    RequestCompletedNoCandle,
    RequestRejectedByProvider,
    RequestTimedOutNoRetry,
    ResponseTooLarge,
    InvalidContentType,
    InvalidJson,
    InvalidResponseShape,
    ReturnedMultipleCandles,
    RegistrationMismatch,
    ConsentMissing,
    RequestBudgetExhausted,
    TechnicalFailure,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProspectivePublicExportAcquisitionReceiptV0 {
    pub receipt_version: String,
    pub registration_digest: String,
    pub request_attempted: bool,
    pub request_count: usize,
    pub retry_count: usize,
    pub request_fingerprint: String,
    pub request_to_utc: String,
    pub http_status_class: Option<String>,
    pub response_body_digest: Option<String>,
    pub returned_item_count: usize,
    pub outcome: ProspectivePublicExportAcquisitionOutcomeV0,
    pub capsule_digest: Option<String>,
    pub legacy_receipt_unchanged: bool,
    pub receipt_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProspectiveNetworkExportCapsuleV0 {
    pub capsule_version: String,
    pub acquisition_registration_digest: String,
    pub acquisition_receipt_digest: String,
    pub provider_id: String,
    pub market: String,
    pub cadence: String,
    pub request_to_utc: String,
    pub canonical_row: crate::model::CanonicalHistoricalRowIdentityV1,
    pub source_response_digest: String,
    pub source_class: crate::model::ProspectiveExternalSourceClassV0,
    pub finalized: bool,
    pub read_only: bool,
    pub sanitized: bool,
    pub credential_free: bool,
    pub acquired_without_model_output_access: bool,
    pub acquired_without_label_access: bool,
    pub capsule_digest: String,
    #[serde(skip)]
    pub raw_response: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProspectivePublicExportRequestPlanV0 {
    pub request_to_utc: String,
    pub request_to_timestamp_ms: u64,
    pub request_fingerprint: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProspectivePublicHttpResponseV0 {
    pub status: u16,
    pub content_type: Option<String>,
    pub body: Vec<u8>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProspectivePublicHttpFailureV0 {
    TimedOut,
    Technical,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProspectivePublicExportAcquisitionResultV0 {
    pub receipt: ProspectivePublicExportAcquisitionReceiptV0,
    pub capsule: Option<ProspectiveNetworkExportCapsuleV0>,
}

fn prospective_public_export_registration_digest_v0(
    registration: &ProspectivePublicExportAcquisitionRegistrationV0,
) -> String {
    stable_hash_string(&format!(
        "{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}",
        registration.registration_version,
        registration.provider_id,
        registration.endpoint_origin,
        registration.endpoint_path,
        registration.configured_market,
        registration.cadence,
        registration.maximum_requests,
        registration.maximum_concurrency,
        registration.retry_count,
        registration.response_candle_count,
        registration.timeout_seconds,
        registration.maximum_response_bytes,
        registration.public_read_only,
        registration.credential_free,
        registration.api_key_free,
        registration.authorization_header_forbidden,
        registration.cookies_forbidden,
        registration.legacy_blind_receipt_immutable,
        registration.legacy_request_registry_immutable,
    ))
}

fn prospective_public_export_receipt_digest_v0(
    receipt: &ProspectivePublicExportAcquisitionReceiptV0,
) -> String {
    stable_hash_string(&format!(
        "{}:{}:{}:{}:{}:{}:{}:{}:{:?}:{}:{}",
        receipt.receipt_version,
        receipt.registration_digest,
        receipt.request_attempted,
        receipt.request_count,
        receipt.retry_count,
        receipt.request_fingerprint,
        receipt.request_to_utc,
        receipt.http_status_class.as_deref().unwrap_or_default(),
        receipt.outcome,
        receipt.response_body_digest.as_deref().unwrap_or_default(),
        receipt.returned_item_count,
    ))
}

fn prospective_network_export_capsule_digest_v0(
    capsule: &ProspectiveNetworkExportCapsuleV0,
) -> String {
    stable_hash_string(&format!(
        "{}:{}:{}:{}:{}:{}:{:?}:{}:{:?}:{}:{}:{}:{}:{}:{}:{}",
        capsule.capsule_version,
        capsule.acquisition_registration_digest,
        capsule.acquisition_receipt_digest,
        capsule.provider_id,
        capsule.market,
        capsule.cadence,
        capsule.canonical_row,
        capsule.request_to_utc,
        capsule.source_class,
        capsule.source_response_digest,
        capsule.finalized,
        capsule.read_only,
        capsule.sanitized,
        capsule.credential_free,
        capsule.acquired_without_model_output_access,
        capsule.acquired_without_label_access,
    ))
}

fn digest_bytes_v0(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len().saturating_mul(2));
    for byte in bytes {
        encoded.push(HEX[(byte >> 4) as usize] as char);
        encoded.push(HEX[(byte & 0x0f) as usize] as char);
    }
    stable_hash_string(&encoded)
}

pub fn pre_register_prospective_public_export_acquisition_v0(
    config: &UpbitHistoricalPilotConfigV0,
) -> Result<ProspectivePublicExportAcquisitionRegistrationV0, String> {
    config.validate()?;
    if config.provider_id != UPBIT_PROVIDER_ID || !valid_market_symbol(&config.symbol) {
        return Err("prospective_public_export_configuration_invalid".into());
    }
    let timeout_seconds = config
        .timeout_seconds
        .min(PROSPECTIVE_PUBLIC_EXPORT_MAX_TIMEOUT_SECONDS_V0);
    let maximum_response_bytes = config
        .maximum_response_bytes
        .min(PROSPECTIVE_PUBLIC_EXPORT_MAX_RESPONSE_BYTES_V0);
    if timeout_seconds == 0 || maximum_response_bytes == 0 {
        return Err("prospective_public_export_configuration_invalid".into());
    }
    let mut registration = ProspectivePublicExportAcquisitionRegistrationV0 {
        registration_version: PROSPECTIVE_PUBLIC_EXPORT_REGISTRATION_VERSION_V0.into(),
        provider_id: UPBIT_PROVIDER_ID.into(),
        endpoint_origin: "https://api.upbit.com".into(),
        endpoint_path: "/v1/candles/days".into(),
        configured_market: config.symbol.clone(),
        cadence: "1d".into(),
        maximum_requests: 1,
        maximum_concurrency: 1,
        retry_count: 0,
        response_candle_count: 1,
        timeout_seconds,
        maximum_response_bytes,
        public_read_only: true,
        credential_free: true,
        api_key_free: true,
        authorization_header_forbidden: true,
        cookies_forbidden: true,
        legacy_blind_receipt_immutable: true,
        legacy_request_registry_immutable: true,
        registration_digest: String::new(),
    };
    registration.registration_digest =
        prospective_public_export_registration_digest_v0(&registration);
    validate_prospective_public_export_acquisition_registration_v0(&registration)?;
    Ok(registration)
}

pub fn validate_prospective_public_export_acquisition_registration_v0(
    registration: &ProspectivePublicExportAcquisitionRegistrationV0,
) -> Result<(), String> {
    if registration.registration_version != PROSPECTIVE_PUBLIC_EXPORT_REGISTRATION_VERSION_V0
        || registration.provider_id != UPBIT_PROVIDER_ID
        || registration.endpoint_origin != "https://api.upbit.com"
        || registration.endpoint_path != "/v1/candles/days"
        || !valid_market_symbol(&registration.configured_market)
        || registration.cadence != "1d"
        || registration.maximum_requests != 1
        || registration.maximum_concurrency != 1
        || registration.retry_count != 0
        || registration.response_candle_count != 1
        || registration.timeout_seconds == 0
        || registration.timeout_seconds > PROSPECTIVE_PUBLIC_EXPORT_MAX_TIMEOUT_SECONDS_V0
        || registration.maximum_response_bytes == 0
        || registration.maximum_response_bytes > PROSPECTIVE_PUBLIC_EXPORT_MAX_RESPONSE_BYTES_V0
        || !registration.public_read_only
        || !registration.credential_free
        || !registration.api_key_free
        || !registration.authorization_header_forbidden
        || !registration.cookies_forbidden
        || !registration.legacy_blind_receipt_immutable
        || !registration.legacy_request_registry_immutable
        || registration.registration_digest
            != prospective_public_export_registration_digest_v0(registration)
    {
        Err("prospective_public_export_registration_invalid".into())
    } else {
        Ok(())
    }
}

pub fn write_prospective_public_export_acquisition_registration_v0(
    path: &Path,
    registration: &ProspectivePublicExportAcquisitionRegistrationV0,
) -> Result<(), String> {
    validate_prospective_public_export_acquisition_registration_v0(registration)?;
    let parent = path
        .parent()
        .ok_or("prospective_public_export_registration_storage_unavailable")?;
    fs::create_dir_all(parent)
        .map_err(|_| "prospective_public_export_registration_storage_unavailable")?;
    let encoded = serde_json::to_vec(registration)
        .map_err(|_| "prospective_public_export_registration_serialization_failed")?;
    let temporary = path.with_extension("tmp");
    fs::write(&temporary, encoded)
        .map_err(|_| "prospective_public_export_registration_storage_failed")?;
    fs::rename(temporary, path)
        .map_err(|_| "prospective_public_export_registration_storage_failed".to_string())
}

pub fn read_prospective_public_export_acquisition_registration_v0(
    path: &Path,
) -> Result<ProspectivePublicExportAcquisitionRegistrationV0, String> {
    serde_json::from_slice(
        &fs::read(path).map_err(|_| "prospective_public_export_registration_unavailable")?,
    )
    .map_err(|_| "prospective_public_export_registration_invalid".into())
}

pub fn write_prospective_public_export_acquisition_receipt_v0(
    path: &Path,
    receipt: &ProspectivePublicExportAcquisitionReceiptV0,
) -> Result<(), String> {
    if !verify_prospective_public_export_acquisition_receipt_v0(receipt) {
        return Err("prospective_public_export_receipt_invalid".into());
    }
    let parent = path
        .parent()
        .ok_or("prospective_public_export_receipt_storage_unavailable")?;
    fs::create_dir_all(parent)
        .map_err(|_| "prospective_public_export_receipt_storage_unavailable")?;
    let encoded = serde_json::to_vec(receipt)
        .map_err(|_| "prospective_public_export_receipt_serialization_failed")?;
    let temporary = path.with_extension("tmp");
    fs::write(&temporary, encoded)
        .map_err(|_| "prospective_public_export_receipt_storage_failed")?;
    fs::rename(temporary, path)
        .map_err(|_| "prospective_public_export_receipt_storage_failed".to_string())
}

pub fn read_prospective_public_export_acquisition_receipt_v0(
    path: &Path,
) -> Result<ProspectivePublicExportAcquisitionReceiptV0, String> {
    serde_json::from_slice(
        &fs::read(path).map_err(|_| "prospective_public_export_receipt_unavailable")?,
    )
    .map_err(|_| "prospective_public_export_receipt_invalid".into())
}

pub fn write_prospective_network_export_capsule_v0(
    path: &Path,
    capsule: &ProspectiveNetworkExportCapsuleV0,
) -> Result<(), String> {
    if !verify_prospective_network_export_capsule_v0(capsule) {
        return Err("prospective_network_export_capsule_invalid".into());
    }
    let parent = path
        .parent()
        .ok_or("prospective_network_export_capsule_storage_unavailable")?;
    fs::create_dir_all(parent)
        .map_err(|_| "prospective_network_export_capsule_storage_unavailable")?;
    let encoded = serde_json::to_vec(capsule)
        .map_err(|_| "prospective_network_export_capsule_serialization_failed")?;
    let temporary = path.with_extension("tmp");
    fs::write(&temporary, encoded)
        .map_err(|_| "prospective_network_export_capsule_storage_failed")?;
    fs::rename(temporary, path)
        .map_err(|_| "prospective_network_export_capsule_storage_failed".to_string())
}

pub fn read_prospective_network_export_capsule_v0(
    path: &Path,
) -> Result<ProspectiveNetworkExportCapsuleV0, String> {
    serde_json::from_slice(
        &fs::read(path).map_err(|_| "prospective_network_export_capsule_unavailable")?,
    )
    .map_err(|_| "prospective_network_export_capsule_invalid".into())
}

pub fn verify_prospective_public_export_acquisition_receipt_v0(
    receipt: &ProspectivePublicExportAcquisitionReceiptV0,
) -> bool {
    receipt.receipt_version == PROSPECTIVE_PUBLIC_EXPORT_RECEIPT_VERSION_V0
        && receipt.request_count <= 1
        && receipt.retry_count == 0
        && (!receipt.request_attempted || receipt.request_count == 1)
        && (receipt.request_attempted || receipt.request_count == 0)
        && receipt.legacy_receipt_unchanged
        && receipt.receipt_digest == prospective_public_export_receipt_digest_v0(receipt)
}

pub fn verify_prospective_network_export_capsule_v0(
    capsule: &ProspectiveNetworkExportCapsuleV0,
) -> bool {
    capsule.capsule_version == PROSPECTIVE_NETWORK_EXPORT_CAPSULE_VERSION_V0
        && capsule.provider_id == UPBIT_PROVIDER_ID
        && valid_market_symbol(&capsule.market)
        && capsule.cadence == "1d"
        && capsule.source_class
            == crate::model::ProspectiveExternalSourceClassV0::ApprovedCredentialFreeProviderExport
        && capsule.finalized
        && capsule.read_only
        && capsule.sanitized
        && capsule.credential_free
        && capsule.acquired_without_model_output_access
        && capsule.acquired_without_label_access
        && !capsule.source_response_digest.is_empty()
        && capsule.canonical_row.row_digest_v1
            == crate::model::canonical_semantic_digest_v1(&capsule.canonical_row)
        && capsule.capsule_digest == prospective_network_export_capsule_digest_v0(capsule)
}

pub fn prospective_public_export_request_plan_v0(
    registration: &ProspectivePublicExportAcquisitionRegistrationV0,
    current_timestamp_ms: u64,
) -> Result<ProspectivePublicExportRequestPlanV0, String> {
    validate_prospective_public_export_acquisition_registration_v0(registration)?;
    let request_to_timestamp_ms =
        current_timestamp_ms / DAILY_INTERVAL_MS_V0 * DAILY_INTERVAL_MS_V0;
    let request_to_utc = format_utc_timestamp(request_to_timestamp_ms)
        .ok_or("prospective_public_export_request_boundary_invalid")?;
    if parse_upbit_utc_timestamp_ms(&request_to_utc)? != request_to_timestamp_ms
        || request_to_timestamp_ms > current_timestamp_ms
        || request_to_timestamp_ms % DAILY_INTERVAL_MS_V0 != 0
    {
        return Err("prospective_public_export_request_boundary_invalid".into());
    }
    let request_fingerprint = stable_hash_string(&format!(
        "{}:{}:{}:{}:{}",
        registration.registration_digest,
        registration.provider_id,
        registration.configured_market,
        registration.cadence,
        request_to_utc,
    ));
    Ok(ProspectivePublicExportRequestPlanV0 {
        request_to_utc,
        request_to_timestamp_ms,
        request_fingerprint,
    })
}

fn prospective_public_export_receipt_v0(
    registration: &ProspectivePublicExportAcquisitionRegistrationV0,
    plan: Option<&ProspectivePublicExportRequestPlanV0>,
    request_attempted: bool,
    http_status_class: Option<String>,
    response_body_digest: Option<String>,
    returned_item_count: usize,
    outcome: ProspectivePublicExportAcquisitionOutcomeV0,
    capsule_digest: Option<String>,
    legacy_receipt_unchanged: bool,
) -> ProspectivePublicExportAcquisitionReceiptV0 {
    let mut receipt = ProspectivePublicExportAcquisitionReceiptV0 {
        receipt_version: PROSPECTIVE_PUBLIC_EXPORT_RECEIPT_VERSION_V0.into(),
        registration_digest: registration.registration_digest.clone(),
        request_attempted,
        request_count: usize::from(request_attempted),
        retry_count: 0,
        request_fingerprint: plan
            .map(|value| value.request_fingerprint.clone())
            .unwrap_or_default(),
        request_to_utc: plan
            .map(|value| value.request_to_utc.clone())
            .unwrap_or_default(),
        http_status_class,
        response_body_digest,
        returned_item_count,
        outcome,
        capsule_digest,
        legacy_receipt_unchanged,
        receipt_digest: String::new(),
    };
    receipt.receipt_digest = prospective_public_export_receipt_digest_v0(&receipt);
    receipt
}

fn http_status_class_v0(status: u16) -> String {
    format!("{}xx", status / 100)
}

#[derive(Clone, Deserialize)]
struct UpbitProspectiveDailyCandleV0 {
    market: String,
    candle_date_time_utc: String,
    opening_price: f64,
    high_price: f64,
    low_price: f64,
    trade_price: f64,
    candle_acc_trade_volume: f64,
    candle_acc_trade_price: f64,
    timestamp: u64,
}

fn create_prospective_network_export_capsule_v0(
    registration: &ProspectivePublicExportAcquisitionRegistrationV0,
    receipt_digest: &str,
    plan: &ProspectivePublicExportRequestPlanV0,
    response_digest: &str,
    raw_response: Vec<u8>,
    candle: UpbitProspectiveDailyCandleV0,
) -> Result<ProspectiveNetworkExportCapsuleV0, ProspectivePublicExportAcquisitionOutcomeV0> {
    if candle.market != registration.configured_market {
        return Err(ProspectivePublicExportAcquisitionOutcomeV0::InvalidResponseShape);
    }
    let timestamp_ms = parse_upbit_utc_timestamp_ms(&candle.candle_date_time_utc)
        .map_err(|_| ProspectivePublicExportAcquisitionOutcomeV0::InvalidResponseShape)?;
    let candle_end = timestamp_ms
        .checked_add(DAILY_INTERVAL_MS_V0)
        .ok_or(ProspectivePublicExportAcquisitionOutcomeV0::InvalidResponseShape)?;
    if candle_end > plan.request_to_timestamp_ms
        || candle.timestamp < timestamp_ms
        || candle.timestamp >= candle_end
        || ![
            candle.opening_price,
            candle.high_price,
            candle.low_price,
            candle.trade_price,
            candle.candle_acc_trade_volume,
            candle.candle_acc_trade_price,
        ]
        .iter()
        .all(|value| value.is_finite())
        || candle.opening_price <= 0.0
        || candle.high_price <= 0.0
        || candle.low_price <= 0.0
        || candle.trade_price <= 0.0
        || candle.candle_acc_trade_volume < 0.0
        || candle.candle_acc_trade_price < 0.0
        || candle.high_price < candle.opening_price.max(candle.trade_price)
        || candle.low_price > candle.opening_price.min(candle.trade_price)
        || candle.high_price < candle.low_price
    {
        return Err(ProspectivePublicExportAcquisitionOutcomeV0::InvalidResponseShape);
    }
    let mut canonical_row = crate::model::CanonicalHistoricalRowIdentityV1 {
        provider_id: UPBIT_PROVIDER_ID.into(),
        series_id: registration.configured_market.clone(),
        timestamp_ms,
        open_bits: candle.opening_price.to_bits(),
        high_bits: candle.high_price.to_bits(),
        low_bits: candle.low_price.to_bits(),
        close_bits: candle.trade_price.to_bits(),
        volume_bits: candle.candle_acc_trade_volume.to_bits(),
        trade_value_bits: Some(candle.candle_acc_trade_price.to_bits()),
        row_digest_v1: String::new(),
    };
    canonical_row.row_digest_v1 = crate::model::canonical_semantic_digest_v1(&canonical_row);
    let mut capsule = ProspectiveNetworkExportCapsuleV0 {
        capsule_version: PROSPECTIVE_NETWORK_EXPORT_CAPSULE_VERSION_V0.into(),
        acquisition_registration_digest: registration.registration_digest.clone(),
        acquisition_receipt_digest: receipt_digest.into(),
        provider_id: UPBIT_PROVIDER_ID.into(),
        market: registration.configured_market.clone(),
        cadence: registration.cadence.clone(),
        request_to_utc: plan.request_to_utc.clone(),
        canonical_row,
        source_response_digest: response_digest.into(),
        source_class:
            crate::model::ProspectiveExternalSourceClassV0::ApprovedCredentialFreeProviderExport,
        finalized: true,
        read_only: true,
        sanitized: true,
        credential_free: true,
        acquired_without_model_output_access: true,
        acquired_without_label_access: true,
        capsule_digest: String::new(),
        raw_response,
    };
    capsule.capsule_digest = prospective_network_export_capsule_digest_v0(&capsule);
    Ok(capsule)
}

pub fn execute_prospective_public_export_acquisition_v0<F>(
    registration: &ProspectivePublicExportAcquisitionRegistrationV0,
    registration_reopened_and_verified: bool,
    existing_receipt: Option<&ProspectivePublicExportAcquisitionReceiptV0>,
    allow_network: bool,
    confirm_single_public_candle_request: bool,
    current_timestamp_ms: u64,
    transport: F,
) -> ProspectivePublicExportAcquisitionResultV0
where
    F: FnOnce(
        &ProspectivePublicExportRequestPlanV0,
    ) -> Result<ProspectivePublicHttpResponseV0, ProspectivePublicHttpFailureV0>,
{
    let plan = prospective_public_export_request_plan_v0(registration, current_timestamp_ms).ok();
    if plan.is_none() || !registration_reopened_and_verified {
        return ProspectivePublicExportAcquisitionResultV0 {
            receipt: prospective_public_export_receipt_v0(
                registration,
                plan.as_ref(),
                false,
                None,
                None,
                0,
                ProspectivePublicExportAcquisitionOutcomeV0::RegistrationMismatch,
                None,
                true,
            ),
            capsule: None,
        };
    }
    let plan = plan.expect("checked above");
    if existing_receipt.is_some_and(|receipt| {
        !verify_prospective_public_export_acquisition_receipt_v0(receipt)
            || receipt.registration_digest != registration.registration_digest
            || receipt.request_attempted
    }) {
        return ProspectivePublicExportAcquisitionResultV0 {
            receipt: prospective_public_export_receipt_v0(
                registration,
                Some(&plan),
                false,
                None,
                None,
                0,
                ProspectivePublicExportAcquisitionOutcomeV0::RequestBudgetExhausted,
                None,
                true,
            ),
            capsule: None,
        };
    }
    if !allow_network || !confirm_single_public_candle_request {
        return ProspectivePublicExportAcquisitionResultV0 {
            receipt: prospective_public_export_receipt_v0(
                registration,
                Some(&plan),
                false,
                None,
                None,
                0,
                ProspectivePublicExportAcquisitionOutcomeV0::ConsentMissing,
                None,
                true,
            ),
            capsule: None,
        };
    }
    let response = match transport(&plan) {
        Ok(response) => response,
        Err(ProspectivePublicHttpFailureV0::TimedOut) => {
            return ProspectivePublicExportAcquisitionResultV0 {
                receipt: prospective_public_export_receipt_v0(
                    registration,
                    Some(&plan),
                    true,
                    None,
                    None,
                    0,
                    ProspectivePublicExportAcquisitionOutcomeV0::RequestTimedOutNoRetry,
                    None,
                    true,
                ),
                capsule: None,
            };
        }
        Err(ProspectivePublicHttpFailureV0::Technical) => {
            return ProspectivePublicExportAcquisitionResultV0 {
                receipt: prospective_public_export_receipt_v0(
                    registration,
                    Some(&plan),
                    true,
                    None,
                    None,
                    0,
                    ProspectivePublicExportAcquisitionOutcomeV0::TechnicalFailure,
                    None,
                    true,
                ),
                capsule: None,
            };
        }
    };
    let status_class = Some(http_status_class_v0(response.status));
    let response_digest = digest_bytes_v0(&response.body);
    if response.body.len() > registration.maximum_response_bytes {
        return ProspectivePublicExportAcquisitionResultV0 {
            receipt: prospective_public_export_receipt_v0(
                registration,
                Some(&plan),
                true,
                status_class,
                Some(response_digest),
                0,
                ProspectivePublicExportAcquisitionOutcomeV0::ResponseTooLarge,
                None,
                true,
            ),
            capsule: None,
        };
    }
    if response.status != 200 {
        return ProspectivePublicExportAcquisitionResultV0 {
            receipt: prospective_public_export_receipt_v0(
                registration,
                Some(&plan),
                true,
                status_class,
                Some(response_digest),
                0,
                ProspectivePublicExportAcquisitionOutcomeV0::RequestRejectedByProvider,
                None,
                true,
            ),
            capsule: None,
        };
    }
    if !response.content_type.as_deref().is_some_and(|value| {
        value
            .split(';')
            .next()
            .is_some_and(|mime| mime.trim().eq_ignore_ascii_case("application/json"))
    }) {
        return ProspectivePublicExportAcquisitionResultV0 {
            receipt: prospective_public_export_receipt_v0(
                registration,
                Some(&plan),
                true,
                status_class,
                Some(response_digest),
                0,
                ProspectivePublicExportAcquisitionOutcomeV0::InvalidContentType,
                None,
                true,
            ),
            capsule: None,
        };
    }
    let value: serde_json::Value = match serde_json::from_slice(&response.body) {
        Ok(value) => value,
        Err(_) => {
            return ProspectivePublicExportAcquisitionResultV0 {
                receipt: prospective_public_export_receipt_v0(
                    registration,
                    Some(&plan),
                    true,
                    status_class,
                    Some(response_digest),
                    0,
                    ProspectivePublicExportAcquisitionOutcomeV0::InvalidJson,
                    None,
                    true,
                ),
                capsule: None,
            };
        }
    };
    let returned_item_count = match value.as_array() {
        Some(candles) => candles.len(),
        None => {
            return ProspectivePublicExportAcquisitionResultV0 {
                receipt: prospective_public_export_receipt_v0(
                    registration,
                    Some(&plan),
                    true,
                    status_class,
                    Some(response_digest),
                    0,
                    ProspectivePublicExportAcquisitionOutcomeV0::InvalidResponseShape,
                    None,
                    true,
                ),
                capsule: None,
            };
        }
    };
    if returned_item_count == 0 {
        return ProspectivePublicExportAcquisitionResultV0 {
            receipt: prospective_public_export_receipt_v0(
                registration,
                Some(&plan),
                true,
                status_class,
                Some(response_digest),
                0,
                ProspectivePublicExportAcquisitionOutcomeV0::RequestCompletedNoCandle,
                None,
                true,
            ),
            capsule: None,
        };
    }
    if returned_item_count != registration.response_candle_count {
        return ProspectivePublicExportAcquisitionResultV0 {
            receipt: prospective_public_export_receipt_v0(
                registration,
                Some(&plan),
                true,
                status_class,
                Some(response_digest),
                returned_item_count,
                ProspectivePublicExportAcquisitionOutcomeV0::ReturnedMultipleCandles,
                None,
                true,
            ),
            capsule: None,
        };
    }
    let candle = match serde_json::from_value::<UpbitProspectiveDailyCandleV0>(
        value
            .as_array()
            .and_then(|values| values.first())
            .cloned()
            .expect("one item checked above"),
    ) {
        Ok(candle) => candle,
        Err(_) => {
            return ProspectivePublicExportAcquisitionResultV0 {
                receipt: prospective_public_export_receipt_v0(
                    registration,
                    Some(&plan),
                    true,
                    status_class,
                    Some(response_digest),
                    returned_item_count,
                    ProspectivePublicExportAcquisitionOutcomeV0::InvalidResponseShape,
                    None,
                    true,
                ),
                capsule: None,
            };
        }
    };
    let provisional_receipt = prospective_public_export_receipt_v0(
        registration,
        Some(&plan),
        true,
        status_class,
        Some(response_digest.clone()),
        returned_item_count,
        ProspectivePublicExportAcquisitionOutcomeV0::CapsuleCreated,
        None,
        true,
    );
    let capsule = match create_prospective_network_export_capsule_v0(
        registration,
        &provisional_receipt.receipt_digest,
        &plan,
        &response_digest,
        response.body.clone(),
        candle.clone(),
    ) {
        Ok(capsule) => capsule,
        Err(outcome) => {
            return ProspectivePublicExportAcquisitionResultV0 {
                receipt: prospective_public_export_receipt_v0(
                    registration,
                    Some(&plan),
                    true,
                    provisional_receipt.http_status_class,
                    provisional_receipt.response_body_digest,
                    returned_item_count,
                    outcome,
                    None,
                    true,
                ),
                capsule: None,
            };
        }
    };
    let receipt = prospective_public_export_receipt_v0(
        registration,
        Some(&plan),
        true,
        provisional_receipt.http_status_class,
        provisional_receipt.response_body_digest,
        returned_item_count,
        ProspectivePublicExportAcquisitionOutcomeV0::CapsuleCreated,
        Some(capsule.capsule_digest.clone()),
        true,
    );
    // The receipt digest is part of the capsule identity.  Build it once more
    // after the capsule digest is known, then seal the capsule against it.
    let capsule = create_prospective_network_export_capsule_v0(
        registration,
        &receipt.receipt_digest,
        &plan,
        receipt.response_body_digest.as_deref().unwrap_or_default(),
        response.body.clone(),
        candle,
    )
    .expect("validated response remains valid");
    let receipt = prospective_public_export_receipt_v0(
        registration,
        Some(&plan),
        true,
        receipt.http_status_class,
        receipt.response_body_digest,
        returned_item_count,
        ProspectivePublicExportAcquisitionOutcomeV0::CapsuleCreated,
        Some(capsule.capsule_digest.clone()),
        true,
    );
    ProspectivePublicExportAcquisitionResultV0 {
        receipt,
        capsule: Some(capsule),
    }
}

fn prospective_public_export_url_v0(
    registration: &ProspectivePublicExportAcquisitionRegistrationV0,
    plan: &ProspectivePublicExportRequestPlanV0,
) -> Result<String, String> {
    validate_prospective_public_export_acquisition_registration_v0(registration)?;
    if parse_upbit_utc_timestamp_ms(&plan.request_to_utc)? != plan.request_to_timestamp_ms
        || plan.request_to_timestamp_ms % DAILY_INTERVAL_MS_V0 != 0
        || plan.request_to_utc.is_empty()
    {
        return Err("prospective_public_export_request_boundary_invalid".into());
    }
    let encoded_to = plan.request_to_utc.replace(':', "%3A");
    Ok(format!(
        "{}{}?market={}&to={encoded_to}&count=1",
        registration.endpoint_origin, registration.endpoint_path, registration.configured_market
    ))
}

pub fn fetch_one_prospective_public_export_v0(
    registration: &ProspectivePublicExportAcquisitionRegistrationV0,
    plan: &ProspectivePublicExportRequestPlanV0,
) -> Result<ProspectivePublicHttpResponseV0, ProspectivePublicHttpFailureV0> {
    let url = prospective_public_export_url_v0(registration, plan)
        .map_err(|_| ProspectivePublicHttpFailureV0::Technical)?;
    let output = Command::new("curl")
        .args([
            "--disable",
            "--silent",
            "--show-error",
            "--proto",
            "=https",
            "--proto-redir",
            "=https",
            "--max-redirs",
            "0",
            "--connect-timeout",
            &registration.timeout_seconds.to_string(),
            "--max-time",
            &registration.timeout_seconds.to_string(),
            "--max-filesize",
            &registration.maximum_response_bytes.to_string(),
            "--request",
            "GET",
            "--header",
            "accept: application/json",
            "--write-out",
            "\n%{http_code}\n%{content_type}",
            &url,
        ])
        .output()
        .map_err(|_| ProspectivePublicHttpFailureV0::Technical)?;
    if !output.status.success() {
        return if output.status.code() == Some(28) {
            Err(ProspectivePublicHttpFailureV0::TimedOut)
        } else {
            Err(ProspectivePublicHttpFailureV0::Technical)
        };
    }
    let output =
        String::from_utf8(output.stdout).map_err(|_| ProspectivePublicHttpFailureV0::Technical)?;
    let (body_and_status, content_type) = output
        .rsplit_once('\n')
        .ok_or(ProspectivePublicHttpFailureV0::Technical)?;
    let (body, status) = body_and_status
        .rsplit_once('\n')
        .ok_or(ProspectivePublicHttpFailureV0::Technical)?;
    let status = status
        .trim()
        .parse::<u16>()
        .map_err(|_| ProspectivePublicHttpFailureV0::Technical)?;
    Ok(ProspectivePublicHttpResponseV0 {
        status,
        content_type: (!content_type.trim().is_empty()).then(|| content_type.trim().into()),
        body: body.as_bytes().to_vec(),
    })
}

pub fn convert_prospective_network_export_to_external_row_capsule_v0(
    network_capsule: &ProspectiveNetworkExportCapsuleV0,
    admission_registration: &crate::model::ProspectiveExternalAdmissionRegistrationV0,
) -> Result<crate::model::ProspectiveExternalRowCapsuleV0, String> {
    if !verify_prospective_network_export_capsule_v0(network_capsule)
        || network_capsule.market != admission_registration.symbol
        || network_capsule.cadence != admission_registration.cadence
    {
        return Err("prospective_network_export_admission_mapping_invalid".into());
    }
    let mut row = network_capsule.canonical_row.clone();
    row.provider_id = admission_registration.canonical_provider_id.clone();
    row.series_id = admission_registration.canonical_series_id.clone();
    row.row_digest_v1 = crate::model::canonical_semantic_digest_v1(&row);
    Ok(crate::model::seal_prospective_external_row_capsule_v0(
        crate::model::ProspectiveExternalRowCapsuleV0 {
            capsule_version: "prospective-external-row-capsule-v0".into(),
            provider_id: admission_registration.canonical_provider_id.clone(),
            market: admission_registration.market.clone(),
            symbol: admission_registration.symbol.clone(),
            cadence: admission_registration.cadence.clone(),
            row,
            source_export_digest: network_capsule.capsule_digest.clone(),
            source_class: network_capsule.source_class,
            finalized: true,
            read_only: true,
            sanitized: true,
            credential_free: true,
            acquired_without_model_output_access: true,
            acquired_without_label_access: true,
            candidate_row_count: 1,
            contains_unexplained_later_rows: false,
            used_in_consumed_evidence: false,
            contains_label_or_outcome: false,
            model_configuration_digest: admission_registration
                .frozen_model_configuration_digest
                .clone(),
            capsule_digest: String::new(),
        },
    ))
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
    run_manual_upbit_historical_backfill_at_end_v0(
        config_path,
        allow_network,
        campaign_required_rows,
        None,
    )
}

pub fn run_manual_upbit_historical_backfill_at_end_v0(
    config_path: &Path,
    allow_network: bool,
    campaign_required_rows: usize,
    end_exclusive_timestamp_ms: Option<u64>,
) -> UpbitHistoricalBackfillResultV0 {
    let mut config = match UpbitHistoricalPilotConfigV0::from_toml_path(config_path) {
        Ok(config) if config.validate().is_ok() => config,
        _ => {
            return backfill_result(
                UpbitHistoricalBackfillStatusV0::ConfigurationMissing,
                None,
                vec!["local_provider_configuration_missing_or_invalid".to_string()],
            );
        }
    };
    if let Some(end_timestamp_ms) = end_exclusive_timestamp_ms {
        if select_backfill_end_cursor_v0(&config, Some(end_timestamp_ms)).is_err() {
            return backfill_result(
                UpbitHistoricalBackfillStatusV0::ValidationFailure,
                Some(&config),
                vec!["backfill_end_cursor_invalid".to_string()],
            );
        }
        config.end_timestamp_ms = end_timestamp_ms;
    }
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
    let budget = ethical_upbit_request_budget_v0(&config);
    if !budget.validate() {
        return backfill_result(
            UpbitHistoricalBackfillStatusV0::RequestBudgetRejected,
            Some(&config),
            vec!["ethical_request_budget_invalid".to_string()],
        );
    }
    config.max_retries = 0;
    let required_rows = campaign_required_rows.min(config.target_rows);
    let requested_count = required_rows.min(config.page_size);
    let required_pages = required_rows.div_ceil(config.page_size);
    if required_pages == 0 || required_pages > budget.maximum_requests {
        return backfill_result(
            UpbitHistoricalBackfillStatusV0::RequestBudgetRejected,
            Some(&config),
            vec!["ethical_request_budget_rejected".to_string()],
        );
    }
    let execution_started = Instant::now();

    let (first_snapshot, first_receipt) =
        match acquire_upbit_page_v0(&config, config.end_timestamp_ms, requested_count) {
            Ok(value) => value,
            Err(reason) => {
                return backfill_result(
                    backfill_status_from_page_failure(&reason),
                    Some(&config),
                    vec![page_failure_reason(&reason)],
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
    if merged.rows.len() >= required_rows {
        return match write_and_verify_local_snapshot_v0(
            &first_snapshot,
            Path::new(&config.snapshot_output_dir),
        ) {
            Ok(path) => completed_backfill_result(
                UpbitHistoricalBackfillStatusV0::RealUpbitSnapshotHarvested,
                &config,
                &first_snapshot,
                path,
                page_receipts,
                vec!["single_page_target_reached".to_string()],
            ),
            Err(reason) => backfill_result(
                UpbitHistoricalBackfillStatusV0::RealSmokeFailed,
                Some(&config),
                vec![reason],
            ),
        };
    }

    let mut stop_status = UpbitHistoricalBackfillStatusV0::MaximumPagesReachedInsufficient;
    while page_receipts.len() < required_pages {
        if execution_started.elapsed().as_secs() >= budget.maximum_wall_clock_seconds {
            return partial_backfill_result(
                UpbitHistoricalBackfillStatusV0::TransientFailureStopped,
                &config,
                &merged,
                page_receipts,
                vec!["ethical_wall_clock_budget_exhausted".to_string()],
            );
        }
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
        thread::sleep(Duration::from_millis(budget.minimum_inter_request_delay_ms));
        let (snapshot, receipt) = match acquire_upbit_page_v0(&config, oldest, config.page_size) {
            Ok(value) => value,
            Err(reason) => {
                return partial_backfill_result(
                    backfill_status_from_page_failure(&reason),
                    &config,
                    &merged,
                    page_receipts,
                    vec![page_failure_reason(&reason)],
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

fn select_backfill_end_cursor_v0(
    config: &UpbitHistoricalPilotConfigV0,
    requested_end_exclusive_timestamp_ms: Option<u64>,
) -> Result<u64, String> {
    let end_timestamp_ms = requested_end_exclusive_timestamp_ms.unwrap_or(config.end_timestamp_ms);
    if end_timestamp_ms <= config.start_timestamp_ms || end_timestamp_ms > config.end_timestamp_ms {
        Err("backfill end cursor invalid".to_string())
    } else {
        Ok(end_timestamp_ms)
    }
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

/// Produces a new immutable snapshot from an already accepted BTC snapshot and
/// an independently verified bounded Upbit harvest.  It never writes or mutates
/// either input; callers may persist the returned value through the existing
/// Protobuf V1 write-and-verify path.
pub fn merge_existing_upbit_snapshot_v0(
    existing: &DataSnapshot,
    harvested: &DataSnapshot,
) -> Result<(DataSnapshot, usize), String> {
    if existing.provider_id != UPBIT_PROVIDER_ID
        || harvested.provider_id != UPBIT_PROVIDER_ID
        || existing.market_scope != AcquisitionMarketScope::BtcCrypto
        || harvested.market_scope != AcquisitionMarketScope::BtcCrypto
        || existing.dataset_kind != DatasetKind::DailyOhlcv
        || harvested.dataset_kind != DatasetKind::DailyOhlcv
        || existing.normalized_dataset.symbol != harvested.normalized_dataset.symbol
        || !existing.read_only
        || !harvested.read_only
        || dataset_digest(&existing.normalized_dataset) != existing.content_digest
        || dataset_digest(&harvested.normalized_dataset) != harvested.content_digest
    {
        return Err("upbit immutable snapshot merge rejected".to_string());
    }
    let (dataset, duplicates) = merge_upbit_historical_pages_v0(
        &[
            existing.normalized_dataset.clone(),
            harvested.normalized_dataset.clone(),
        ],
        &existing.normalized_dataset.symbol,
    )?;
    let digest = dataset_digest(&dataset);
    let mut merged = existing.clone();
    merged.snapshot_id = super::acquisition::snapshot_id_from_semantic_digest_v1(&digest);
    merged.request_key = format!(
        "upbit-daily-expanded:{}:{}:{}",
        existing.normalized_dataset.symbol, existing.snapshot_id, harvested.snapshot_id
    );
    merged.requested_lookback = DataLookback {
        bars: dataset.rows.len(),
        start_timestamp_ms: dataset.rows.first().map(|row| row.timestamp_ms),
        end_timestamp_ms: dataset.rows.last().map(|row| row.timestamp_ms),
    };
    merged.actual_start_timestamp_ms = dataset.rows.first().map(|row| row.timestamp_ms);
    merged.actual_end_timestamp_ms = dataset.rows.last().map(|row| row.timestamp_ms);
    merged.fetched_at_ms = existing.fetched_at_ms.max(harvested.fetched_at_ms);
    merged.normalized_at_ms = existing.normalized_at_ms.max(harvested.normalized_at_ms);
    merged.row_count = dataset.rows.len();
    merged.quality_summary.row_count = merged.row_count;
    merged.content_digest = digest;
    merged.normalized_dataset = dataset;
    merged.provenance.acquisition_request_id = format!(
        "upbit-expanded-{}",
        stable_hash_string(&format!(
            "{}:{}",
            existing.snapshot_id, harvested.snapshot_id
        ))
    );
    merged.provenance.fetch_receipt_id = format!(
        "expanded-{}",
        stable_hash_string(&format!(
            "{}:{}",
            existing.content_digest, harvested.content_digest
        ))
    );
    verify_snapshot_semantic_identity_v1(&merged)?;
    Ok((merged, duplicates))
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum UpbitPageAcquireFailureV0 {
    RateLimited,
    PermissionDenied,
    Transient(String),
}

fn backfill_status_from_page_failure(
    failure: &UpbitPageAcquireFailureV0,
) -> UpbitHistoricalBackfillStatusV0 {
    match failure {
        UpbitPageAcquireFailureV0::RateLimited => {
            UpbitHistoricalBackfillStatusV0::RateLimitedStopped
        }
        UpbitPageAcquireFailureV0::PermissionDenied => {
            UpbitHistoricalBackfillStatusV0::PermissionDeniedStopped
        }
        UpbitPageAcquireFailureV0::Transient(_) => {
            UpbitHistoricalBackfillStatusV0::TransientFailureStopped
        }
    }
}

fn page_failure_reason(failure: &UpbitPageAcquireFailureV0) -> String {
    match failure {
        UpbitPageAcquireFailureV0::RateLimited => {
            "upbit_rate_limit_stopped_without_retry".to_string()
        }
        UpbitPageAcquireFailureV0::PermissionDenied => {
            "upbit_permission_denied_stopped_without_retry".to_string()
        }
        UpbitPageAcquireFailureV0::Transient(reason) => reason.clone(),
    }
}

fn acquire_upbit_page_v0(
    config: &UpbitHistoricalPilotConfigV0,
    end_timestamp_ms: u64,
    page_size: usize,
) -> Result<(DataSnapshot, UpbitHistoricalPageReceiptV0), UpbitPageAcquireFailureV0> {
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
    let receipt = execution.receipts.into_iter().next().ok_or_else(|| {
        UpbitPageAcquireFailureV0::Transient("upbit_page_receipt_missing".to_string())
    })?;
    let snapshot = execution.new_snapshots.into_iter().next().ok_or_else(|| {
        if receipt
            .reason_codes
            .contains(&ReasonCode::AcquisitionRateLimited)
        {
            UpbitPageAcquireFailureV0::RateLimited
        } else if receipt
            .reason_codes
            .contains(&ReasonCode::AcquisitionPermissionDenied)
        {
            UpbitPageAcquireFailureV0::PermissionDenied
        } else {
            UpbitPageAcquireFailureV0::Transient(
                receipt
                    .reason_codes
                    .iter()
                    .map(|reason| format!("{reason:?}"))
                    .collect::<Vec<_>>()
                    .join("|"),
            )
        }
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

    #[test]
    fn prospective_blind_acquisition_requires_network_gate_without_transport() {
        let result = acquire_one_blind_upbit_daily_row_v0(
            Path::new("config/local/not-present.toml"),
            "challenge",
            1,
            &BTreeSet::new(),
            false,
        );
        assert!(!result.receipt.request_attempted);
        assert_eq!(result.receipt.request_count, 0);
        assert_eq!(
            result.receipt.stop_reason,
            ProspectiveBlindAcquisitionStopReasonV0::AwaitingNetworkConsent
        );
        assert!(verify_prospective_blind_acquisition_receipt_v0(
            &result.receipt
        ));
        assert!(result.admitted_rows.is_empty());
    }

    #[test]
    fn legacy_failure_receipt_fails_closed_without_a_pipeline_stage() {
        let receipt = prospective_blind_receipt_v0(
            "challenge",
            true,
            1,
            Some("failure".to_string()),
            0,
            0,
            0,
            0,
            0,
            0,
            ProspectiveBlindAcquisitionStopReasonV0::InvalidProviderResponse,
        );
        let first = classify_prior_prospective_rejection_v0(&receipt);
        let second = classify_prior_prospective_rejection_v0(&receipt);
        assert_eq!(first, second);
        assert_eq!(
            first.first_rejected_stage,
            ProspectiveProviderPipelineStageV0::EvidenceInsufficient
        );
        assert_eq!(
            first.root_cause,
            ProspectiveProviderRejectionRootCauseV0::Unknown
        );
        assert_eq!(
            first.eligibility,
            SharedEpochEligibilityV0::BlockedByUnknownCause
        );
        assert!(!first.sufficient_for_classification);
    }

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
            minimum_inter_request_delay_ms: 1,
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
    fn http_statuses_fail_closed_without_reclassifying_rate_or_permission_errors() {
        assert_eq!(upbit_http_failure_v0(Some(200)), None);
        assert_eq!(
            upbit_http_failure_v0(Some(429)),
            Some(ProviderFetchFailure::RateLimited)
        );
        assert_eq!(
            upbit_http_failure_v0(Some(403)),
            Some(ProviderFetchFailure::PermissionDenied)
        );
        assert_eq!(
            upbit_http_failure_v0(Some(500)),
            Some(ProviderFetchFailure::Unavailable)
        );
    }

    #[test]
    fn ethical_backfill_plan_is_exact_and_rejects_excess_request_budget() {
        let config = config();
        let budget = ethical_upbit_request_budget_v0(&config);
        let plan = plan_btc_regime_backfill_v0(2, 2, 3, 1, 0, &config, &budget);
        assert_eq!(plan.required_total_rows, 7);
        assert_eq!(plan.additional_rows_required, 5);
        assert_eq!(plan.estimated_minimum_pages, 3);
        assert_eq!(
            plan.plan_status,
            BackfillRequestPlanStatusV0::RequestBudgetRejected
        );

        let ready = plan_btc_regime_backfill_v0(2, 2, 2, 0, 0, &config, &budget);
        assert_eq!(ready.additional_rows_required, 2);
        assert_eq!(ready.estimated_minimum_pages, 1);
        assert_eq!(ready.plan_status, BackfillRequestPlanStatusV0::Ready);
        assert!(ethical_upbit_request_budget_v0(&config).validate());
    }

    #[test]
    fn backfill_cursor_must_stay_within_the_configured_historical_range() {
        let config = config();
        assert_eq!(
            select_backfill_end_cursor_v0(&config, None).unwrap(),
            config.end_timestamp_ms
        );
        assert_eq!(
            select_backfill_end_cursor_v0(&config, Some(config.end_timestamp_ms - 1)).unwrap(),
            config.end_timestamp_ms - 1
        );
        assert!(select_backfill_end_cursor_v0(&config, Some(config.start_timestamp_ms)).is_err());
        assert!(select_backfill_end_cursor_v0(&config, Some(config.end_timestamp_ms + 1)).is_err());
    }

    #[test]
    fn duplicate_forensics_identify_cursor_overlap_and_bit_exact_field_difference() {
        let accepted = local_snapshot(
            parse_upbit_daily_ohlcv_v0(
                r#"[
                  {"market":"KRW-BTC","candle_date_time_utc":"2024-01-01T00:00:00","opening_price":1.0,"high_price":2.0,"low_price":0.5,"trade_price":1.5,"candle_acc_trade_volume":1.0},
                  {"market":"KRW-BTC","candle_date_time_utc":"2024-01-02T00:00:00","opening_price":2.0,"high_price":3.0,"low_price":1.0,"trade_price":2.5,"candle_acc_trade_volume":1.0}
                ]"#,
                "KRW-BTC",
            )
            .unwrap(),
            "unused",
        );
        let mut fetched = accepted.clone();
        fetched.normalized_dataset.rows[0].close = 9.0;
        fetched.content_digest = dataset_digest(&fetched.normalized_dataset);
        fetched.snapshot_id =
            crate::data::snapshot_id_from_semantic_digest_v1(&fetched.content_digest);
        fetched.requested_lookback.end_timestamp_ms =
            Some(accepted.normalized_dataset.rows[1].timestamp_ms + 86_400_000);
        let report = inspect_upbit_duplicate_conflict_v0(&accepted, &fetched).unwrap();
        assert_eq!(report.overlapping_timestamp_count, 2);
        assert_eq!(report.identical_duplicate_count, 1);
        assert_eq!(report.conflicting_duplicate_count, 1);
        assert_eq!(
            report.first_conflicting_field,
            Some(HistoricalConflictFieldV0::Close)
        );
        assert_eq!(report.finalized_conflict_count, 1);
        assert_eq!(
            report.root_cause,
            HistoricalMergeConflictRootCauseV0::RequestCursorOverlappedExistingRange
        );
    }

    #[test]
    fn strict_cursor_proof_uses_oldest_row_and_exact_missing_count() {
        let dataset = parse_upbit_daily_ohlcv_v0(
            r#"[
              {"market":"KRW-BTC","candle_date_time_utc":"2024-01-01T00:00:00","opening_price":1.0,"high_price":2.0,"low_price":0.5,"trade_price":1.5,"candle_acc_trade_volume":1.0},
              {"market":"KRW-BTC","candle_date_time_utc":"2024-01-02T00:00:00","opening_price":2.0,"high_price":3.0,"low_price":1.0,"trade_price":2.5,"candle_acc_trade_volume":1.0}
            ]"#,
            "KRW-BTC",
        )
        .unwrap();
        let existing = local_snapshot(dataset, "unused");
        let mut strict_config = config();
        strict_config.start_timestamp_ms -= 86_400_000;
        strict_config.maximum_pages = 1;
        strict_config.target_rows = 2;
        let proof = build_strict_older_cursor_proof_v0(&existing, 2, &strict_config);
        assert_eq!(
            proof.requested_exclusive_end,
            existing.normalized_dataset.rows[0].timestamp_ms
        );
        assert_eq!(proof.requested_count, 2);
        assert_eq!(proof.expected_overlap_rows, 0);
        assert_eq!(
            proof.proof_status,
            StrictHistoricalRequestPlanStatusV0::ReadyZeroOverlap
        );
    }

    #[test]
    fn strict_page_validation_accepts_only_non_overlapping_older_rows() {
        let existing = local_snapshot(
            parse_upbit_daily_ohlcv_v0(
                r#"[
                  {"market":"KRW-BTC","candle_date_time_utc":"2024-01-03T00:00:00","opening_price":3.0,"high_price":4.0,"low_price":2.0,"trade_price":3.5,"candle_acc_trade_volume":1.0},
                  {"market":"KRW-BTC","candle_date_time_utc":"2024-01-04T00:00:00","opening_price":4.0,"high_price":5.0,"low_price":3.0,"trade_price":4.5,"candle_acc_trade_volume":1.0}
                ]"#,
                "KRW-BTC",
            )
            .unwrap(),
            "unused",
        );
        let older = local_snapshot(
            parse_upbit_daily_ohlcv_v0(
                r#"[
                  {"market":"KRW-BTC","candle_date_time_utc":"2024-01-01T00:00:00","opening_price":1.0,"high_price":2.0,"low_price":0.5,"trade_price":1.5,"candle_acc_trade_volume":1.0},
                  {"market":"KRW-BTC","candle_date_time_utc":"2024-01-02T00:00:00","opening_price":2.0,"high_price":3.0,"low_price":1.0,"trade_price":2.5,"candle_acc_trade_volume":1.0}
                ]"#,
                "KRW-BTC",
            )
            .unwrap(),
            "unused",
        );
        assert_eq!(
            validate_strictly_older_upbit_page_v0(&existing, &older, 2).status,
            StrictOlderPageExecutionStatusV0::OlderPageAccepted
        );
        assert_eq!(
            validate_strictly_older_upbit_page_v0(&existing, &existing, 2).status,
            StrictOlderPageExecutionStatusV0::UnexpectedOverlap
        );
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

    #[test]
    fn existing_snapshot_merge_creates_a_new_verified_identity_without_mutation() {
        let existing = local_snapshot(
            parse_upbit_daily_ohlcv_v0(
                r#"[
                  {"market":"KRW-BTC","candle_date_time_utc":"2024-01-01T00:00:00","opening_price":1.0,"high_price":2.0,"low_price":0.5,"trade_price":1.5,"candle_acc_trade_volume":1.0},
                  {"market":"KRW-BTC","candle_date_time_utc":"2024-01-02T00:00:00","opening_price":2.0,"high_price":3.0,"low_price":1.0,"trade_price":2.5,"candle_acc_trade_volume":1.0}
                ]"#,
                "KRW-BTC",
            )
            .unwrap(),
            "unused",
        );
        let harvested = local_snapshot(
            parse_upbit_daily_ohlcv_v0(
                r#"[
                  {"market":"KRW-BTC","candle_date_time_utc":"2024-01-02T00:00:00","opening_price":2.0,"high_price":3.0,"low_price":1.0,"trade_price":2.5,"candle_acc_trade_volume":1.0},
                  {"market":"KRW-BTC","candle_date_time_utc":"2024-01-03T00:00:00","opening_price":3.0,"high_price":4.0,"low_price":2.0,"trade_price":3.5,"candle_acc_trade_volume":1.0}
                ]"#,
                "KRW-BTC",
            )
            .unwrap(),
            "unused",
        );
        let original = existing.clone();
        let (merged, duplicates) = merge_existing_upbit_snapshot_v0(&existing, &harvested).unwrap();
        assert_eq!(existing, original);
        assert_eq!(duplicates, 1);
        assert_eq!(merged.row_count, 3);
        assert_ne!(merged.snapshot_id, existing.snapshot_id);
        assert_eq!(
            merged.content_digest,
            dataset_digest(&merged.normalized_dataset)
        );
    }

    fn prospective_registration() -> ProspectivePublicExportAcquisitionRegistrationV0 {
        pre_register_prospective_public_export_acquisition_v0(&config()).unwrap()
    }

    fn prospective_response(body: &str) -> ProspectivePublicHttpResponseV0 {
        ProspectivePublicHttpResponseV0 {
            status: 200,
            content_type: Some("application/json; charset=utf-8".into()),
            body: body.as_bytes().to_vec(),
        }
    }

    fn valid_prospective_body() -> &'static str {
        r#"[{"market":"KRW-BTC","candle_date_time_utc":"2024-01-02T00:00:00","opening_price":10.0,"high_price":12.0,"low_price":9.0,"trade_price":11.0,"candle_acc_trade_volume":5.0,"candle_acc_trade_price":50.0,"timestamp":1704153600100}]"#
    }

    fn prospective_now() -> u64 {
        1_704_240_123_456
    }

    #[test]
    fn public_export_dry_run_plan_uses_one_url_encoded_utc_boundary_without_auth() {
        let registration = prospective_registration();
        let plan =
            prospective_public_export_request_plan_v0(&registration, prospective_now()).unwrap();
        assert_eq!(plan.request_to_utc, "2024-01-03T00:00:00Z");
        let url = prospective_public_export_url_v0(&registration, &plan).unwrap();
        assert!(url.starts_with("https://api.upbit.com/v1/candles/days?market=KRW-BTC"));
        assert!(url.contains("to=2024-01-03T00%3A00%3A00Z&count=1"));
        assert!(!url.contains("authorization"));
        assert!(
            prospective_public_export_url_v0(
                &registration,
                &ProspectivePublicExportRequestPlanV0 {
                    request_to_utc: String::new(),
                    request_to_timestamp_ms: plan.request_to_timestamp_ms,
                    request_fingerprint: plan.request_fingerprint,
                },
            )
            .is_err()
        );
    }

    #[test]
    fn public_export_missing_or_unverified_consent_never_constructs_transport() {
        let registration = prospective_registration();
        let mut calls = 0;
        let missing = execute_prospective_public_export_acquisition_v0(
            &registration,
            true,
            None,
            true,
            false,
            prospective_now(),
            |_| {
                calls += 1;
                Ok(prospective_response(valid_prospective_body()))
            },
        );
        assert_eq!(calls, 0);
        assert_eq!(
            missing.receipt.outcome,
            ProspectivePublicExportAcquisitionOutcomeV0::ConsentMissing
        );
        let unverified = execute_prospective_public_export_acquisition_v0(
            &registration,
            false,
            None,
            true,
            true,
            prospective_now(),
            |_| {
                calls += 1;
                Ok(prospective_response(valid_prospective_body()))
            },
        );
        assert_eq!(calls, 0);
        assert_eq!(
            unverified.receipt.outcome,
            ProspectivePublicExportAcquisitionOutcomeV0::RegistrationMismatch
        );
    }

    #[test]
    fn public_export_one_attempt_budget_is_consumed_by_timeout_429_and_5xx_without_retry() {
        let registration = prospective_registration();
        let timeout = execute_prospective_public_export_acquisition_v0(
            &registration,
            true,
            None,
            true,
            true,
            prospective_now(),
            |_| Err(ProspectivePublicHttpFailureV0::TimedOut),
        );
        assert_eq!(timeout.receipt.request_count, 1);
        assert_eq!(timeout.receipt.retry_count, 0);
        assert_eq!(
            timeout.receipt.outcome,
            ProspectivePublicExportAcquisitionOutcomeV0::RequestTimedOutNoRetry
        );
        let mut budget_calls = 0;
        let exhausted = execute_prospective_public_export_acquisition_v0(
            &registration,
            true,
            Some(&timeout.receipt),
            true,
            true,
            prospective_now(),
            |_| {
                budget_calls += 1;
                Ok(prospective_response(valid_prospective_body()))
            },
        );
        assert_eq!(budget_calls, 0);
        assert_eq!(
            exhausted.receipt.outcome,
            ProspectivePublicExportAcquisitionOutcomeV0::RequestBudgetExhausted
        );
        for status in [429, 500] {
            let rejected = execute_prospective_public_export_acquisition_v0(
                &registration,
                true,
                None,
                true,
                true,
                prospective_now(),
                |_| {
                    Ok(ProspectivePublicHttpResponseV0 {
                        status,
                        content_type: Some("application/json".into()),
                        body: b"[]".to_vec(),
                    })
                },
            );
            assert_eq!(rejected.receipt.request_count, 1);
            assert_eq!(rejected.receipt.retry_count, 0);
            assert_eq!(
                rejected.receipt.outcome,
                ProspectivePublicExportAcquisitionOutcomeV0::RequestRejectedByProvider
            );
        }
    }

    #[test]
    fn public_export_rejects_response_shape_finality_and_ohlcv_failures() {
        let registration = prospective_registration();
        let cases = [
            (
                "[]",
                ProspectivePublicExportAcquisitionOutcomeV0::RequestCompletedNoCandle,
            ),
            (
                r#"[{"market":"KRW-BTC"},{"market":"KRW-BTC"}]"#,
                ProspectivePublicExportAcquisitionOutcomeV0::ReturnedMultipleCandles,
            ),
            (
                "{",
                ProspectivePublicExportAcquisitionOutcomeV0::InvalidJson,
            ),
            (
                r#"[{"market":"KRW-ETH","candle_date_time_utc":"2024-01-02T00:00:00","opening_price":10.0,"high_price":12.0,"low_price":9.0,"trade_price":11.0,"candle_acc_trade_volume":5.0,"candle_acc_trade_price":50.0,"timestamp":1704153600100}]"#,
                ProspectivePublicExportAcquisitionOutcomeV0::InvalidResponseShape,
            ),
            (
                r#"[{"market":"KRW-BTC","candle_date_time_utc":"2024-01-03T00:00:00","opening_price":10.0,"high_price":12.0,"low_price":9.0,"trade_price":11.0,"candle_acc_trade_volume":5.0,"candle_acc_trade_price":50.0,"timestamp":1704240000100}]"#,
                ProspectivePublicExportAcquisitionOutcomeV0::InvalidResponseShape,
            ),
            (
                r#"[{"market":"KRW-BTC","candle_date_time_utc":"2024-01-02T00:00:00+09:00","opening_price":10.0,"high_price":12.0,"low_price":9.0,"trade_price":11.0,"candle_acc_trade_volume":5.0,"candle_acc_trade_price":50.0,"timestamp":1704153600100}]"#,
                ProspectivePublicExportAcquisitionOutcomeV0::InvalidResponseShape,
            ),
            (
                r#"[{"market":"KRW-BTC","candle_date_time_utc":"2024-01-02T00:00:00","opening_price":10.0,"high_price":8.0,"low_price":9.0,"trade_price":11.0,"candle_acc_trade_volume":-1.0,"candle_acc_trade_price":50.0,"timestamp":1704153600100}]"#,
                ProspectivePublicExportAcquisitionOutcomeV0::InvalidResponseShape,
            ),
        ];
        for (body, expected) in cases {
            let result = execute_prospective_public_export_acquisition_v0(
                &registration,
                true,
                None,
                true,
                true,
                prospective_now(),
                |_| Ok(prospective_response(body)),
            );
            assert_eq!(result.receipt.request_count, 1);
            assert_eq!(result.receipt.outcome, expected);
            assert!(result.capsule.is_none());
        }
        let wrong_content_type = execute_prospective_public_export_acquisition_v0(
            &registration,
            true,
            None,
            true,
            true,
            prospective_now(),
            |_| {
                Ok(ProspectivePublicHttpResponseV0 {
                    status: 200,
                    content_type: Some("text/plain".into()),
                    body: valid_prospective_body().as_bytes().to_vec(),
                })
            },
        );
        assert_eq!(
            wrong_content_type.receipt.outcome,
            ProspectivePublicExportAcquisitionOutcomeV0::InvalidContentType
        );
    }

    #[test]
    fn public_export_valid_response_seals_one_deterministic_credential_free_capsule() {
        let registration = prospective_registration();
        let first = execute_prospective_public_export_acquisition_v0(
            &registration,
            true,
            None,
            true,
            true,
            prospective_now(),
            |_| Ok(prospective_response(valid_prospective_body())),
        );
        let second = execute_prospective_public_export_acquisition_v0(
            &registration,
            true,
            None,
            true,
            true,
            prospective_now(),
            |_| Ok(prospective_response(valid_prospective_body())),
        );
        assert_eq!(
            first.receipt.outcome,
            ProspectivePublicExportAcquisitionOutcomeV0::CapsuleCreated
        );
        assert!(verify_prospective_public_export_acquisition_receipt_v0(
            &first.receipt
        ));
        let capsule = first.capsule.as_ref().unwrap();
        assert!(verify_prospective_network_export_capsule_v0(capsule));
        assert_eq!(capsule.raw_response, valid_prospective_body().as_bytes());
        assert_eq!(
            capsule.acquisition_receipt_digest,
            first.receipt.receipt_digest
        );
        assert_eq!(
            first.receipt.response_body_digest,
            second.receipt.response_body_digest
        );
        assert_eq!(
            capsule.capsule_digest,
            second.capsule.unwrap().capsule_digest
        );
        assert!(first.receipt.legacy_receipt_unchanged);
    }
}
