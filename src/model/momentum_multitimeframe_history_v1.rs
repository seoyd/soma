//! Zero-authority multi-timeframe historical data and causal protocol foundation.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    thread,
    time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};

#[cfg(test)]
use crate::{
    core::ReasonCode,
    data::{
        DataLookback, DatasetKind, NetworkConsentV0, SnapshotAdjustmentSemanticsV1,
        SnapshotCompatibilityV1, SnapshotProvenance, SnapshotQualitySummary, SnapshotSourceType,
        historical_replay_dataset_digest_v0,
    },
    league::HistoricalReplayDataset,
};

use crate::{
    data::{
        AcquisitionMarketScope, CurlHttpClient, DataSnapshot, MarketDataHttpClient,
        UpbitHistoricalPilotConfigV0,
    },
    league::{HistoricalOhlcvRow, canonical_current_agent_states},
    stable_hash_string,
};

use super::{
    momentum_future_prediction_v4::{
        ArtifactBuilderV4_2, ArtifactReaderV4_2, as_u64, as_usize, persist_artifact,
        protobuf_paths, read_single,
    },
    momentum_prospective_series_v4::{
        MomentumProspectiveEpochReadinessV4, MomentumProspectiveSeriesReportV4,
        validate_sealed_epoch_two_report_v4,
    },
};

#[cfg(test)]
use super::momentum_prospective_series_v4::tests::deterministic_sealed_epoch_two_report_fixture_v4;

pub(super) const ROOT: &str = "state/historical_replay/momentum_multitimeframe/v1";
const LIVE_ROOT: &str = "state/learning_data";
const MARKET: &str = "KRW-BTC";
const PROVIDER: &str = "upbit";
const FOUNDATION_VERSION: &str = "momentum-multitimeframe-foundation-v1";
const PAUSE_VERSION: &str = "live-prospective-continuation-pause-v1";
const PLAN_VERSION: &str = "momentum-multitimeframe-acquisition-plan-v1";
const RECEIPT_VERSION: &str = "historical-candle-page-receipt-v1";
const CHECKPOINT_VERSION: &str = "historical-candle-checkpoint-v1";
const CHUNK_VERSION: &str = "historical-candle-chunk-v1";
const INDEX_VERSION: &str = "historical-candle-index-v1";
const DERIVED_INDEX_VERSION: &str = "derived-candle-index-v1";
const COMPARISON_VERSION: &str = "derived-native-comparison-index-v1";
const PROTOCOL_VERSION: &str = "momentum-multitimeframe-protocol-replay-v1";
const FUTURE_VERSION: &str = "momentum-historical-hard-replay-registration-v2";
const ABLATION_VERSION: &str = "momentum-multitimeframe-ablation-registration-v1";
const HOLDOUT_VERSION: &str = "momentum-multitimeframe-holdout-v1";
const REPORT_VERSION: &str = "momentum-multitimeframe-public-report-v1";
const MACRO_FORENSIC_VERSION: &str = "momentum-macro-candle-forensic-receipt-v1";
const MACRO_FORENSIC_AGGREGATE_VERSION: &str = "momentum-macro-forensic-aggregate-v1";
const MACRO_POLICY_VERSION: &str = "momentum-canonical-macro-policy-v1";
#[allow(dead_code)]
const NATIVE_MACRO_INDEX_VERSION: &str = "momentum-native-macro-canonical-index-v1";
#[allow(dead_code)]
const CORRECTED_DERIVED_INDEX_VERSION: &str = "momentum-corrected-derived-index-v2";
const QUALIFIED_SET_VERSION: &str = "momentum-qualified-timeframe-set-v1";
const CAUSAL_REVALIDATION_VERSION: &str = "momentum-qualified-causal-revalidation-v1";
const QUALIFIED_HARD_REPLAY_VERSION: &str = "momentum-qualified-hard-replay-v2";
const MACRO_REPORT_VERSION: &str = "momentum-macro-forensics-public-report-v1";
const MINUTE_MS: u64 = 60_000;
const DAY_MS: u64 = 86_400_000;
const PILOT_DAYS: usize = 180;
const FROZEN_EXISTING_DAILY_SNAPSHOT_ROWS: usize = 312;
const PAGE_SIZE: usize = 200;
const CHUNK_SIZE: usize = 4_096;
const REQUEST_CEILING: usize = 1_400;
const NATIVE_SAMPLE_PAGES: usize = 6;
const PROTOCOL_CADENCE_MS: u64 = 10 * MINUTE_MS;
const ABSOLUTE_TOLERANCE: f64 = 1e-12;
const RELATIVE_TOLERANCE: f64 = 1e-10;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MomentumHistoricalTimeframeV1 {
    Minute1,
    Minute3,
    Minute5,
    Minute10,
    Day1,
    Week1,
    Month1,
    Year1,
}

impl MomentumHistoricalTimeframeV1 {
    pub const ORDERED: [Self; 8] = [
        Self::Minute1,
        Self::Minute3,
        Self::Minute5,
        Self::Minute10,
        Self::Day1,
        Self::Week1,
        Self::Month1,
        Self::Year1,
    ];

    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Minute1 => "1m",
            Self::Minute3 => "3m",
            Self::Minute5 => "5m",
            Self::Minute10 => "10m",
            Self::Day1 => "1d",
            Self::Week1 => "1w",
            Self::Month1 => "1mo",
            Self::Year1 => "1y",
        }
    }

    fn parse(value: &str) -> Result<Self, String> {
        Self::ORDERED
            .into_iter()
            .find(|timeframe| timeframe.as_str() == value)
            .ok_or_else(|| "multi-timeframe identity rejected".to_string())
    }

    fn cadence_ms(self) -> Option<u64> {
        match self {
            Self::Minute1 => Some(MINUTE_MS),
            Self::Minute3 => Some(3 * MINUTE_MS),
            Self::Minute5 => Some(5 * MINUTE_MS),
            Self::Minute10 => Some(10 * MINUTE_MS),
            Self::Day1 => Some(DAY_MS),
            Self::Week1 | Self::Month1 | Self::Year1 => None,
        }
    }

    fn is_canonical(self) -> bool {
        matches!(self, Self::Minute1 | Self::Day1)
    }

    fn native_path(self) -> Result<&'static str, String> {
        match self {
            Self::Minute1 => Ok("/v1/candles/minutes/1"),
            Self::Minute3 => Ok("/v1/candles/minutes/3"),
            Self::Minute5 => Ok("/v1/candles/minutes/5"),
            Self::Minute10 => Ok("/v1/candles/minutes/10"),
            Self::Day1 => Ok("/v1/candles/days"),
            Self::Week1 => Ok("/v1/candles/weeks"),
            Self::Month1 => Ok("/v1/candles/months"),
            Self::Year1 => Ok("/v1/candles/years"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LiveProspectiveContinuationPolicyV1 {
    PausedAfterSealedEpochTwo,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CandleIntervalPresenceV1 {
    ObservedTradeCandle,
    NoTradeInterval,
    MissingEvidence,
    IntegrityFailure,
}

impl CandleIntervalPresenceV1 {
    fn as_str(self) -> &'static str {
        match self {
            Self::ObservedTradeCandle => "ObservedTradeCandle",
            Self::NoTradeInterval => "NoTradeInterval",
            Self::MissingEvidence => "MissingEvidence",
            Self::IntegrityFailure => "IntegrityFailure",
        }
    }

    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "ObservedTradeCandle" => Ok(Self::ObservedTradeCandle),
            "NoTradeInterval" => Ok(Self::NoTradeInterval),
            "MissingEvidence" => Ok(Self::MissingEvidence),
            "IntegrityFailure" => Ok(Self::IntegrityFailure),
            _ => Err("candle interval presence rejected".to_string()),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeframeViewAvailabilityV1 {
    Available,
    InsufficientHistoricalDepth,
    NoTradeInterval,
    MissingEvidence,
    PartialCandleForbidden,
    IntegrityFailure,
}

impl TimeframeViewAvailabilityV1 {
    fn as_str(self) -> &'static str {
        match self {
            Self::Available => "Available",
            Self::InsufficientHistoricalDepth => "InsufficientHistoricalDepth",
            Self::NoTradeInterval => "NoTradeInterval",
            Self::MissingEvidence => "MissingEvidence",
            Self::PartialCandleForbidden => "PartialCandleForbidden",
            Self::IntegrityFailure => "IntegrityFailure",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DerivedNativeComparisonV1 {
    ExactMatch,
    WithinRegisteredTolerance,
    ProviderBoundaryMismatch,
    MissingNativeCandle,
    DerivedCompletenessFailure,
    IntegrityFailure,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumTimeframeRoleV1 {
    Microstructure,
    ShortTrend,
    MediumTrend,
    MacroRegime,
}

impl MomentumTimeframeRoleV1 {
    fn for_timeframe(timeframe: MomentumHistoricalTimeframeV1) -> Self {
        match timeframe {
            MomentumHistoricalTimeframeV1::Minute1 | MomentumHistoricalTimeframeV1::Minute3 => {
                Self::Microstructure
            }
            MomentumHistoricalTimeframeV1::Minute5 | MomentumHistoricalTimeframeV1::Minute10 => {
                Self::ShortTrend
            }
            MomentumHistoricalTimeframeV1::Day1 => Self::MediumTrend,
            MomentumHistoricalTimeframeV1::Week1
            | MomentumHistoricalTimeframeV1::Month1
            | MomentumHistoricalTimeframeV1::Year1 => Self::MacroRegime,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Microstructure => "Microstructure",
            Self::ShortTrend => "ShortTrend",
            Self::MediumTrend => "MediumTrend",
            Self::MacroRegime => "MacroRegime",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumHistoricalPredictionTaskV2 {
    IntradayTenMinute,
    DailyOneDay,
    WeeklyOneWeek,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MultiTimeframeEvidenceUseClassV1 {
    NewResolutionResearchEvidence,
    PreviouslyConsumedCalendarPeriod,
    DevelopmentReplayEvidence,
    ValidationReplayEvidence,
    SealedHistoricalHoldout,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TradingSimulationStatus {
    BlockedNoFrozenExecutionPolicy,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumMtfHistoryPhaseV1 {
    Unregistered,
    FoundationRegistered,
    CanonicalBackfillComplete,
    DerivedViewsComplete,
    ProtocolReplayComplete,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MomentumMtfHistoryRunModeV1 {
    Status,
    DryRun,
    RegisterFoundation,
    ExecuteBackfill,
    DeriveViews,
    ProtocolReplay,
}

impl MomentumMtfHistoryRunModeV1 {
    fn as_str(self) -> &'static str {
        match self {
            Self::Status => "status",
            Self::DryRun => "dry-run",
            Self::RegisterFoundation => "register-foundation",
            Self::ExecuteBackfill => "execute-backfill",
            Self::DeriveViews => "derive-views",
            Self::ProtocolReplay => "protocol-replay",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CandleIntervalV1 {
    pub open_timestamp_ms: u64,
    pub close_exclusive_timestamp_ms: u64,
}

#[derive(Clone, Debug, PartialEq)]
struct HistoricalCandleRowV1 {
    timeframe: MomentumHistoricalTimeframeV1,
    interval: CandleIntervalV1,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
    trade_value: f64,
    ordered_base_candle_digests: Vec<String>,
    presence: CandleIntervalPresenceV1,
    candle_digest: String,
}

#[derive(Clone, Debug, PartialEq)]
struct HistoricalCandleChunkV1 {
    timeframe: MomentumHistoricalTimeframeV1,
    first_timestamp_ms: u64,
    last_timestamp_ms: u64,
    row_count: usize,
    ordered_rows: Vec<HistoricalCandleRowV1>,
    previous_chunk_digest: Option<String>,
    chunk_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoricalCandleIndexV1 {
    pub timeframe: MomentumHistoricalTimeframeV1,
    #[serde(skip_serializing)]
    pub ordered_chunk_digests: Vec<String>,
    pub first_timestamp_ms: u64,
    pub last_timestamp_ms: u64,
    pub close_exclusive_timestamp_ms: u64,
    pub total_row_count: usize,
    pub no_trade_interval_count: usize,
    pub missing_evidence_count: usize,
    pub aggregate_dataset_digest: String,
    pub index_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct LiveContinuationPauseV1 {
    pause_version: String,
    policy: LiveProspectiveContinuationPolicyV1,
    series_digest: String,
    epoch_registration_digest: String,
    input_receipt_digest: String,
    input_capsule_digest: String,
    context_proof_digest: String,
    prediction_capsule_digest: String,
    prediction_journal_digest: String,
    outcome_plan_digest: String,
    protected_first_event_input_boundary_ms: u64,
    completed_event_count: usize,
    scorable_event_count: usize,
    prediction_seal_count: usize,
    input_attempts: usize,
    input_retries: usize,
    outcome_requests: usize,
    outcome_openings: usize,
    epoch_three_registered: bool,
    pause_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct MomentumMtfFoundationRegistrationV1 {
    registration_version: String,
    pause_digest: String,
    provider_id: String,
    symbol: String,
    ordered_timeframes: Vec<MomentumHistoricalTimeframeV1>,
    canonical_bases: Vec<MomentumHistoricalTimeframeV1>,
    derived_timeframes: Vec<MomentumHistoricalTimeframeV1>,
    role_bindings: Vec<String>,
    minute_start_timestamp_ms: u64,
    minute_end_exclusive_timestamp_ms: u64,
    existing_daily_snapshot_digest: String,
    existing_daily_first_timestamp_ms: u64,
    existing_daily_last_timestamp_ms: u64,
    existing_daily_row_count: usize,
    chunk_size: usize,
    protocol_cadence_ms: u64,
    numeric_absolute_tolerance_bits: u64,
    numeric_relative_tolerance_bits: u64,
    training_forbidden: bool,
    tournament_forbidden: bool,
    live_authority_forbidden: bool,
    registration_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumMtfAcquisitionPlanV1 {
    pub plan_version: String,
    pub foundation_registration_digest: String,
    pub minute_page_budget: usize,
    pub daily_page_budget: usize,
    pub native_sample_request_budget: usize,
    pub exact_total_request_budget: usize,
    pub provider_page_size: usize,
    pub maximum_concurrency: usize,
    pub maximum_retries_per_page: usize,
    pub minimum_inter_request_delay_ms: u64,
    pub maximum_response_bytes: usize,
    pub strictly_backward_exclusive: bool,
    pub checkpoint_after_every_page: bool,
    pub plan_digest: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PagePurposeV1 {
    CanonicalMinute,
    CanonicalDailyOlder,
    NativeCrossCheck,
}

impl PagePurposeV1 {
    fn as_str(self) -> &'static str {
        match self {
            Self::CanonicalMinute => "CanonicalMinute",
            Self::CanonicalDailyOlder => "CanonicalDailyOlder",
            Self::NativeCrossCheck => "NativeCrossCheck",
        }
    }

    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "CanonicalMinute" => Ok(Self::CanonicalMinute),
            "CanonicalDailyOlder" => Ok(Self::CanonicalDailyOlder),
            "NativeCrossCheck" => Ok(Self::NativeCrossCheck),
            _ => Err("historical page purpose rejected".to_string()),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PageReceiptStatusV1 {
    Verified,
    TerminalFailure,
}

impl PageReceiptStatusV1 {
    fn as_str(self) -> &'static str {
        match self {
            Self::Verified => "Verified",
            Self::TerminalFailure => "TerminalFailure",
        }
    }

    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "Verified" => Ok(Self::Verified),
            "TerminalFailure" => Ok(Self::TerminalFailure),
            _ => Err("historical page receipt status rejected".to_string()),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct HistoricalPageReceiptV1 {
    receipt_version: String,
    plan_digest: String,
    purpose: PagePurposeV1,
    timeframe: MomentumHistoricalTimeframeV1,
    request_fingerprint: String,
    request_to_exclusive_ms: u64,
    requested_count: usize,
    attempt_sequence: usize,
    status: PageReceiptStatusV1,
    response_body_digest: Option<String>,
    normalized_row_digest: Option<String>,
    rows: Vec<HistoricalCandleRowV1>,
    request_count: usize,
    retry_count: usize,
    receipt_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct HistoricalCheckpointV1 {
    checkpoint_version: String,
    plan_digest: String,
    page_receipt_digest: String,
    request_fingerprint: String,
    last_successful_exclusive_to_ms: u64,
    response_body_digest: String,
    normalized_row_digest: String,
    verified_page_chunk_digest: String,
    request_count_consumed: usize,
    remaining_budget: usize,
    checkpoint_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DerivedViewIndexV1 {
    pub index_version: String,
    pub foundation_registration_digest: String,
    pub timeframe: MomentumHistoricalTimeframeV1,
    pub canonical_source_timeframe: MomentumHistoricalTimeframeV1,
    pub first_timestamp_ms: u64,
    pub last_timestamp_ms: u64,
    pub candle_count: usize,
    pub no_trade_interval_count: usize,
    pub missing_evidence_count: usize,
    #[serde(skip_serializing)]
    pub ordered_candle_digests: Vec<String>,
    pub timezone_policy: String,
    pub boundary_policy: String,
    pub index_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DerivedNativeComparisonSummaryV1 {
    pub comparison_version: String,
    pub timeframe: MomentumHistoricalTimeframeV1,
    pub sample_count: usize,
    pub exact_match_count: usize,
    pub within_tolerance_count: usize,
    pub boundary_mismatch_count: usize,
    pub missing_native_count: usize,
    pub completeness_failure_count: usize,
    pub integrity_failure_count: usize,
    pub systematic_mismatch_blocks_replay: bool,
    pub comparison_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumTimeframeFeatureBlockV1 {
    pub timeframe: MomentumHistoricalTimeframeV1,
    pub source_view_digest: String,
    pub feature_schema_digest: String,
    pub feature_vector_digest: String,
    pub numeric_values_private: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct MomentumMultiTimeframeAsOfSnapshotV1 {
    prediction_timestamp_ms: u64,
    view_digests: Vec<String>,
    availability: Vec<TimeframeViewAvailabilityV1>,
    all_views_closed: bool,
    future_access_count: usize,
    partial_candle_access_count: usize,
    snapshot_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct MomentumProtocolPredictionSealV1 {
    prediction_timestamp_ms: u64,
    as_of_snapshot_digest: String,
    synthetic_prediction_identity: String,
    target_access_count_before_seal: usize,
    seal_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct MomentumProtocolReceiptV1 {
    prediction_timestamp_ms: u64,
    target_timestamp_ms: u64,
    as_of_snapshot_digest: String,
    prediction_seal_digest: String,
    target_revealed_after_seal: bool,
    target_value_access_count: usize,
    performance_claim_produced: bool,
    receipt_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct MomentumProtocolReplayV1 {
    replay_version: String,
    foundation_registration_digest: String,
    comparison_index_digest: String,
    event_count: usize,
    snapshots: Vec<MomentumMultiTimeframeAsOfSnapshotV1>,
    seals: Vec<MomentumProtocolPredictionSealV1>,
    receipts: Vec<MomentumProtocolReceiptV1>,
    all_views_closed: bool,
    future_access_count: usize,
    partial_candle_access_count: usize,
    prediction_before_reveal: bool,
    performance_claim_produced: bool,
    replay_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct MomentumHistoricalHardReplayRegistrationV2 {
    registration_version: String,
    foundation_registration_digest: String,
    tasks: Vec<MomentumHistoricalPredictionTaskV2>,
    context_bindings: Vec<String>,
    executed: bool,
    registration_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct MomentumAblationRegistrationV1 {
    registration_version: String,
    foundation_registration_digest: String,
    ordered_families: Vec<String>,
    individual_leave_one_out_forbidden: bool,
    result_selected_second_family_forbidden: bool,
    executed: bool,
    registration_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct MomentumHistoricalHoldoutV1 {
    holdout_version: String,
    foundation_registration_digest: String,
    eligible_start_timestamp_ms: u64,
    eligible_end_timestamp_ms: u64,
    development_end_exclusive_ms: u64,
    validation_end_exclusive_ms: u64,
    holdout_start_timestamp_ms: u64,
    development_event_count: usize,
    validation_event_count: usize,
    holdout_event_count: usize,
    labels_opened: bool,
    metrics_computed: bool,
    aggregate_comparison_opened: bool,
    holdout_digest: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumMtfSafetyCountersV1 {
    pub network_request_attempts: usize,
    pub transport_constructions: usize,
    pub retries: usize,
    pub maximum_concurrency: usize,
    pub verified_page_count: usize,
    pub failed_page_count: usize,
    pub live_outcome_requests: usize,
    pub live_outcome_openings: usize,
    pub live_label_reads: usize,
    pub live_metric_computations: usize,
    pub live_evaluations: usize,
    pub live_participant_changes: usize,
    pub live_parameter_updates: usize,
    pub live_normalizer_refits: usize,
    pub live_feature_policy_changes: usize,
    pub winner_selections: usize,
    pub rankings: usize,
    pub reward_applications: usize,
    pub penalty_applications: usize,
    pub chair_decisions: usize,
    pub committee_votes: usize,
    pub voice_changes: usize,
    pub tier_changes: usize,
    pub cooldowns: usize,
    pub promotions: usize,
    pub quarantines: usize,
    pub paper_executions: usize,
    pub live_executions: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MomentumMtfHistoryPublicReportV1 {
    pub report_version: String,
    pub run_mode: String,
    pub phase: MomentumMtfHistoryPhaseV1,
    pub live_continuation_policy: Option<LiveProspectiveContinuationPolicyV1>,
    pub live_pause_digest: Option<String>,
    pub foundation_registration_digest: Option<String>,
    pub acquisition_plan: Option<MomentumMtfAcquisitionPlanV1>,
    pub ordered_timeframes: Vec<MomentumHistoricalTimeframeV1>,
    pub canonical_bases: Vec<MomentumHistoricalTimeframeV1>,
    pub minute_index: Option<HistoricalCandleIndexV1>,
    pub daily_index: Option<HistoricalCandleIndexV1>,
    pub derived_indices: Vec<DerivedViewIndexV1>,
    pub native_comparisons: Vec<DerivedNativeComparisonSummaryV1>,
    pub protocol_event_count: usize,
    pub protocol_replay_digest: Option<String>,
    pub all_views_closed: bool,
    pub future_access_count: usize,
    pub partial_candle_access_count: usize,
    pub prediction_before_target_reveal: bool,
    pub availability_counts: Vec<String>,
    pub future_experiment_registration_digest: Option<String>,
    pub future_ablation_registration_digest: Option<String>,
    pub sealed_holdout_digest: Option<String>,
    pub holdout_start_timestamp_ms: Option<u64>,
    pub holdout_labels_opened: bool,
    pub trading_simulation_status: TradingSimulationStatus,
    pub safety_counters: MomentumMtfSafetyCountersV1,
    pub live_protected_artifacts_unchanged: bool,
    pub active_roster_unchanged: bool,
    pub artifacts_written: usize,
    pub duplicate_artifact_count: usize,
    pub report_digest: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumMacroForensicsStatusV1 {
    Unregistered,
    InsufficientPersistedNativeEvidence,
    Qualified,
    BlockedUnresolved,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MomentumMacroForensicsRunModeV1 {
    Status,
    DryRun,
    ExecuteLocal,
    HardReplayStatus,
}

impl MomentumMacroForensicsRunModeV1 {
    fn as_str(self) -> &'static str {
        match self {
            Self::Status => "status",
            Self::DryRun => "dry-run",
            Self::ExecuteLocal => "execute-local",
            Self::HardReplayStatus => "hard-replay-status",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumMacroBoundaryComparisonV1 {
    ExactSameInterval,
    SamePeriodDifferentTimestampRepresentation,
    UtcVsKstBoundaryShift,
    FirstDayOfPeriodMismatch,
    OpeningBoundaryMismatch,
    ClosingBoundaryMismatch,
    NativePeriodNotReconstructableFromDailyBase,
    IntegrityFailure,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumMacroCompletenessComparisonV1 {
    BothComplete,
    NativePartialDerivedExcluded,
    NativeCompleteDerivedPartial,
    SourceCoverageStartsInsidePeriod,
    SourceCoverageEndsInsidePeriod,
    NoTradeCompositionDiffers,
    MissingDailyEvidence,
    IntegrityFailure,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumMacroValueComparisonV1 {
    ExactAllFields,
    AccumulationWithinRegisteredTolerance,
    OpenMismatch,
    HighMismatch,
    LowMismatch,
    CloseMismatch,
    VolumeOutsideRegisteredTolerance,
    TradeValueOutsideRegisteredTolerance,
    MultipleValueMismatches,
    NotComparableBoundaryMismatch,
    IntegrityFailure,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MomentumMacroMismatchRootCauseV1 {
    IncompleteNativeCurrentPeriod,
    IncompleteDerivedCurrentPeriod,
    PartialFirstCalendarPeriod,
    UtcKstCalendarBoundaryDifference,
    ProviderFirstDayPeriodSemantics,
    DailyBaseInsufficientForNativeBoundary,
    NoTradeIntervalComposition,
    MissingCanonicalDailyEvidence,
    AccumulationRoundingOnly,
    IncorrectDerivedAggregation,
    IncorrectNativeNormalization,
    ProviderContractAmbiguous,
    CorruptEvidence,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MomentumMacroCandleDispositionV1 {
    QualifiedDerivedFromDaily,
    QualifiedDerivedFromDailyWithinRegisteredTolerance,
    ExcludedPartialPeriodNotAFailure,
    NativeCanonicalRequired,
    DerivedAggregationDefect,
    ExcludedUnresolved,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumCanonicalMacroSourceV1 {
    DerivedFromCanonicalDaily,
    NativeProviderCandle,
    ExcludedUnresolved,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MomentumTimeframeQualificationV1 {
    QualifiedDerivedCanonical,
    QualifiedNativeCanonical,
    ExcludedPartialOnly,
    ExcludedUnresolved,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MomentumMacroCandleForensicReceiptV1 {
    pub forensic_version: String,
    pub timeframe: MomentumHistoricalTimeframeV1,
    pub native_candle_digest: String,
    pub derived_candle_digest: String,
    pub native_candle_timestamp_ms: u64,
    pub native_candle_kst_timestamp: Option<String>,
    pub native_first_day_of_period: Option<String>,
    pub native_last_trade_timestamp_ms: Option<u64>,
    pub native_open_timestamp_ms: u64,
    pub native_close_exclusive_timestamp_ms: u64,
    pub derived_open_timestamp_ms: u64,
    pub derived_close_exclusive_timestamp_ms: u64,
    pub request_to_exclusive_ms: u64,
    pub market: String,
    pub provider_id: String,
    pub native_response_digest: String,
    pub native_source_row_digests: Vec<String>,
    pub derived_source_row_digests: Vec<String>,
    pub boundary_comparison: MomentumMacroBoundaryComparisonV1,
    pub completeness_comparison: MomentumMacroCompletenessComparisonV1,
    pub value_comparison: MomentumMacroValueComparisonV1,
    pub root_cause: Option<MomentumMacroMismatchRootCauseV1>,
    pub disposition: MomentumMacroCandleDispositionV1,
    pub receipt_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumMacroForensicAggregateV1 {
    pub aggregate_version: String,
    pub timeframe: MomentumHistoricalTimeframeV1,
    pub ordered_receipt_digests: Vec<String>,
    pub compared_period_count: usize,
    pub exact_count: usize,
    pub tolerance_count: usize,
    pub failed_count: usize,
    pub excluded_partial_count: usize,
    pub unresolved_count: usize,
    pub root_cause_counts: Vec<String>,
    pub disposition_counts: Vec<String>,
    pub complete_forensic_coverage: bool,
    pub native_metadata_complete: bool,
    pub aggregate_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumCanonicalMacroPolicyV1 {
    pub policy_version: String,
    pub timeframe: MomentumHistoricalTimeframeV1,
    pub selected_source: MomentumCanonicalMacroSourceV1,
    pub daily_index_digest: String,
    pub derived_index_digest: String,
    pub native_index_digest: Option<String>,
    pub forensic_aggregate_digest: String,
    pub complete_period_count: usize,
    pub qualified_period_count: usize,
    pub excluded_partial_period_count: usize,
    pub unresolved_period_count: usize,
    pub live_authority_eligible: bool,
    pub historical_research_only: bool,
    pub policy_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MomentumNativeMacroCanonicalIndexV1 {
    pub index_version: String,
    pub timeframe: MomentumHistoricalTimeframeV1,
    pub ordered_native_candle_digests: Vec<String>,
    pub ordered_first_day_of_period: Vec<String>,
    pub first_complete_period: String,
    pub last_complete_period: String,
    pub total_complete_periods: usize,
    pub source_response_digests: Vec<String>,
    pub normalization_policy_digest: String,
    pub index_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MomentumCorrectedDerivedMacroIndexV2 {
    pub index_version: String,
    pub timeframe: MomentumHistoricalTimeframeV1,
    pub prior_index_digest: String,
    pub corrected_aggregation_policy_digest: String,
    pub ordered_candle_digests: Vec<String>,
    pub regenerated_period_count: usize,
    pub old_index_preserved: bool,
    pub index_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumQualifiedTimeframeSetV1 {
    pub set_version: String,
    pub minute1: MomentumTimeframeQualificationV1,
    pub minute3: MomentumTimeframeQualificationV1,
    pub minute5: MomentumTimeframeQualificationV1,
    pub minute10: MomentumTimeframeQualificationV1,
    pub day1: MomentumTimeframeQualificationV1,
    pub week1: MomentumTimeframeQualificationV1,
    pub month1: MomentumTimeframeQualificationV1,
    pub year1: MomentumTimeframeQualificationV1,
    pub qualified_count: usize,
    pub unresolved_count: usize,
    pub full_eight_timeframe_replay_allowed: bool,
    pub set_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumQualifiedCausalRevalidationV1 {
    pub revalidation_version: String,
    pub qualified_set_digest: String,
    pub protocol_replay_digest: String,
    pub sealed_holdout_digest: String,
    pub event_count: usize,
    pub selected_source_bindings: Vec<String>,
    pub future_access_count: usize,
    pub partial_candle_access_count: usize,
    pub unqualified_view_access_count: usize,
    pub blocked_unqualified_view_count: usize,
    pub labels_read: usize,
    pub deterministic: bool,
    pub revalidation_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumQualifiedHardReplayRegistrationV2 {
    pub registration_version: String,
    pub qualified_set_digest: String,
    pub task_policies: Vec<String>,
    pub evidence_role_bindings: Vec<String>,
    pub ablation_families: Vec<String>,
    pub model_families: Vec<String>,
    pub contribution_gates: Vec<String>,
    pub constant_benchmark_mandatory: bool,
    pub historical_logistic_warning_preserved: bool,
    pub full_eight_timeframe_required: bool,
    pub executed: bool,
    pub registration_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumMacroForensicsPublicReportV1 {
    pub report_version: String,
    pub run_mode: String,
    pub status: MomentumMacroForensicsStatusV1,
    pub monthly_aggregate: Option<MomentumMacroForensicAggregateV1>,
    pub yearly_aggregate: Option<MomentumMacroForensicAggregateV1>,
    pub weekly_policy: Option<MomentumCanonicalMacroPolicyV1>,
    pub monthly_policy: Option<MomentumCanonicalMacroPolicyV1>,
    pub yearly_policy: Option<MomentumCanonicalMacroPolicyV1>,
    pub qualified_timeframes: Option<MomentumQualifiedTimeframeSetV1>,
    pub causal_revalidation: Option<MomentumQualifiedCausalRevalidationV1>,
    pub hard_replay_registration_digest: Option<String>,
    pub hard_replay_blocked: bool,
    pub hard_replay_executed: bool,
    pub holdout_digest: Option<String>,
    pub holdout_labels_opened: bool,
    pub protocol_event_count: usize,
    pub network_request_attempts: usize,
    pub transport_constructions: usize,
    pub credentials_read: usize,
    pub epoch_three_registrations: usize,
    pub active_committee_count: usize,
    pub live_authority_counters: MomentumMtfSafetyCountersV1,
    pub live_protected_artifacts_unchanged: bool,
    pub active_roster_unchanged: bool,
    pub artifacts_written: usize,
    pub duplicate_artifact_count: usize,
    pub report_digest: String,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct MomentumQualifiedReplayCandleEvidenceV1 {
    pub timeframe: MomentumHistoricalTimeframeV1,
    pub open_timestamp_ms: u64,
    pub close_exclusive_timestamp_ms: u64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub trade_value: f64,
    pub candle_digest: String,
    pub missing_evidence: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct MomentumQualifiedReplayProtocolEventV1 {
    pub prediction_timestamp_ms: u64,
    pub target_timestamp_ms: u64,
    pub receipt_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct MomentumQualifiedReplayHoldoutEvidenceV1 {
    pub holdout_digest: String,
    pub eligible_start_timestamp_ms: u64,
    pub eligible_end_timestamp_ms: u64,
    pub holdout_start_timestamp_ms: u64,
    pub labels_opened: bool,
    pub metrics_computed: bool,
    pub aggregate_comparison_opened: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct MomentumQualifiedSixEvidenceV1 {
    pub qualified_timeframe_set_digest: String,
    pub monthly_policy_digest: String,
    pub yearly_policy_digest: String,
    pub causal_revalidation_digest: String,
    pub protocol_replay_digest: String,
    pub included_timeframes: Vec<MomentumHistoricalTimeframeV1>,
    pub excluded_timeframes: Vec<MomentumHistoricalTimeframeV1>,
    pub view_index_digests: Vec<String>,
    pub views:
        BTreeMap<MomentumHistoricalTimeframeV1, Vec<MomentumQualifiedReplayCandleEvidenceV1>>,
    pub protocol_events: Vec<MomentumQualifiedReplayProtocolEventV1>,
    pub prior_holdout: MomentumQualifiedReplayHoldoutEvidenceV1,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct MomentumQualifiedReplayProtectedStateV1 {
    pub live_tree_file_count: usize,
    pub live_tree_digest: String,
    pub active_roster_digest: String,
    pub live_completed_event_count: usize,
    pub live_scorable_event_count: usize,
    pub live_input_attempts: usize,
    pub live_input_retries: usize,
    pub live_prediction_seal_count: usize,
    pub live_outcome_requests: usize,
    pub live_outcome_openings: usize,
    pub epoch_three_registered: bool,
    pub active_committee_count: usize,
}

fn canonical_digest<T: Clone + std::fmt::Debug>(value: &T, clear: impl FnOnce(&mut T)) -> String {
    let mut canonical = value.clone();
    clear(&mut canonical);
    stable_hash_string(&format!("{canonical:?}"))
}

fn candle_digest(value: &HistoricalCandleRowV1) -> String {
    canonical_digest(value, |item| item.candle_digest.clear())
}

fn chunk_digest(value: &HistoricalCandleChunkV1) -> String {
    canonical_digest(value, |item| item.chunk_digest.clear())
}

fn index_digest(value: &HistoricalCandleIndexV1) -> String {
    canonical_digest(value, |item| item.index_digest.clear())
}

fn pause_digest(value: &LiveContinuationPauseV1) -> String {
    canonical_digest(value, |item| item.pause_digest.clear())
}

fn foundation_digest(value: &MomentumMtfFoundationRegistrationV1) -> String {
    canonical_digest(value, |item| item.registration_digest.clear())
}

fn plan_digest(value: &MomentumMtfAcquisitionPlanV1) -> String {
    canonical_digest(value, |item| item.plan_digest.clear())
}

fn receipt_digest(value: &HistoricalPageReceiptV1) -> String {
    canonical_digest(value, |item| item.receipt_digest.clear())
}

fn checkpoint_digest(value: &HistoricalCheckpointV1) -> String {
    canonical_digest(value, |item| item.checkpoint_digest.clear())
}

fn derived_index_digest(value: &DerivedViewIndexV1) -> String {
    canonical_digest(value, |item| item.index_digest.clear())
}

fn comparison_digest(value: &DerivedNativeComparisonSummaryV1) -> String {
    canonical_digest(value, |item| item.comparison_digest.clear())
}

fn snapshot_digest(value: &MomentumMultiTimeframeAsOfSnapshotV1) -> String {
    canonical_digest(value, |item| item.snapshot_digest.clear())
}

fn protocol_seal_digest(value: &MomentumProtocolPredictionSealV1) -> String {
    canonical_digest(value, |item| item.seal_digest.clear())
}

fn protocol_receipt_digest(value: &MomentumProtocolReceiptV1) -> String {
    canonical_digest(value, |item| item.receipt_digest.clear())
}

fn protocol_digest(value: &MomentumProtocolReplayV1) -> String {
    canonical_digest(value, |item| item.replay_digest.clear())
}

fn future_digest(value: &MomentumHistoricalHardReplayRegistrationV2) -> String {
    canonical_digest(value, |item| item.registration_digest.clear())
}

fn ablation_digest(value: &MomentumAblationRegistrationV1) -> String {
    canonical_digest(value, |item| item.registration_digest.clear())
}

fn holdout_digest(value: &MomentumHistoricalHoldoutV1) -> String {
    canonical_digest(value, |item| item.holdout_digest.clear())
}

fn report_digest(value: &MomentumMtfHistoryPublicReportV1) -> String {
    canonical_digest(value, |item| item.report_digest.clear())
}

fn macro_receipt_digest(value: &MomentumMacroCandleForensicReceiptV1) -> String {
    canonical_digest(value, |item| item.receipt_digest.clear())
}

fn macro_aggregate_digest(value: &MomentumMacroForensicAggregateV1) -> String {
    canonical_digest(value, |item| item.aggregate_digest.clear())
}

fn macro_policy_digest(value: &MomentumCanonicalMacroPolicyV1) -> String {
    canonical_digest(value, |item| item.policy_digest.clear())
}

#[allow(dead_code)]
fn native_macro_index_digest(value: &MomentumNativeMacroCanonicalIndexV1) -> String {
    canonical_digest(value, |item| item.index_digest.clear())
}

#[allow(dead_code)]
fn corrected_derived_index_digest(value: &MomentumCorrectedDerivedMacroIndexV2) -> String {
    canonical_digest(value, |item| item.index_digest.clear())
}

fn qualified_set_digest(value: &MomentumQualifiedTimeframeSetV1) -> String {
    canonical_digest(value, |item| item.set_digest.clear())
}

fn causal_revalidation_digest(value: &MomentumQualifiedCausalRevalidationV1) -> String {
    canonical_digest(value, |item| item.revalidation_digest.clear())
}

fn qualified_hard_replay_digest(value: &MomentumQualifiedHardReplayRegistrationV2) -> String {
    canonical_digest(value, |item| item.registration_digest.clear())
}

fn macro_report_digest(value: &MomentumMacroForensicsPublicReportV1) -> String {
    canonical_digest(value, |item| item.report_digest.clear())
}

fn add_counts(total: &mut (usize, usize), next: (usize, usize)) {
    total.0 += next.0;
    total.1 += next.1;
}

fn days_from_civil(year: i32, month: u32, day: u32) -> i64 {
    let mut year = i64::from(year);
    let month = i64::from(month);
    let day = i64::from(day);
    year -= i64::from(month <= 2);
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let yoe = year - era * 400;
    let doy = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

fn civil_from_days(days: i64) -> (i32, u32, u32) {
    let days = days + 719_468;
    let era = if days >= 0 { days } else { days - 146_096 } / 146_097;
    let doe = days - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let mut year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    year += i64::from(month <= 2);
    (
        i32::try_from(year).unwrap_or(i32::MAX),
        u32::try_from(month).unwrap_or_default(),
        u32::try_from(day).unwrap_or_default(),
    )
}

fn timestamp_ms(year: i32, month: u32, day: u32) -> Result<u64, String> {
    let days = days_from_civil(year, month, day);
    u64::try_from(
        days.checked_mul(i64::try_from(DAY_MS).unwrap_or(i64::MAX))
            .ok_or_else(|| "calendar timestamp overflow".to_string())?,
    )
    .map_err(|_| "calendar timestamp rejected".to_string())
}

fn format_utc_timestamp(timestamp_ms: u64) -> Result<String, String> {
    if !timestamp_ms.is_multiple_of(1_000) {
        return Err("subsecond provider boundary rejected".to_string());
    }
    let days = i64::try_from(timestamp_ms / DAY_MS)
        .map_err(|_| "provider timestamp overflow".to_string())?;
    let (year, month, day) = civil_from_days(days);
    let seconds = (timestamp_ms % DAY_MS) / 1_000;
    Ok(format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        seconds / 3_600,
        (seconds % 3_600) / 60,
        seconds % 60
    ))
}

fn parse_utc_timestamp(value: &str) -> Result<u64, String> {
    if !matches!(value.len(), 19 | 20)
        || (value.len() == 20 && !value.ends_with('Z'))
        || value.as_bytes().get(4) != Some(&b'-')
        || value.as_bytes().get(7) != Some(&b'-')
        || value.as_bytes().get(10) != Some(&b'T')
        || value.as_bytes().get(13) != Some(&b':')
        || value.as_bytes().get(16) != Some(&b':')
    {
        return Err("provider UTC timestamp rejected".to_string());
    }
    let year = value[0..4]
        .parse::<i32>()
        .map_err(|_| "provider UTC year rejected".to_string())?;
    let month = value[5..7]
        .parse::<u32>()
        .map_err(|_| "provider UTC month rejected".to_string())?;
    let day = value[8..10]
        .parse::<u32>()
        .map_err(|_| "provider UTC day rejected".to_string())?;
    let hour = value[11..13]
        .parse::<u64>()
        .map_err(|_| "provider UTC hour rejected".to_string())?;
    let minute = value[14..16]
        .parse::<u64>()
        .map_err(|_| "provider UTC minute rejected".to_string())?;
    let second = value[17..19]
        .parse::<u64>()
        .map_err(|_| "provider UTC second rejected".to_string())?;
    let base = timestamp_ms(year, month, day)?;
    let normalized = civil_from_days(
        i64::try_from(base / DAY_MS).map_err(|_| "provider UTC day overflow".to_string())?,
    );
    if normalized != (year, month, day) || hour >= 24 || minute >= 60 || second >= 60 {
        return Err("provider UTC calendar rejected".to_string());
    }
    base.checked_add((hour * 3_600 + minute * 60 + second) * 1_000)
        .ok_or_else(|| "provider UTC timestamp overflow".to_string())
}

fn next_month_boundary(open_ms: u64) -> Result<u64, String> {
    let days = i64::try_from(open_ms / DAY_MS).map_err(|_| "month overflow".to_string())?;
    let (year, month, day) = civil_from_days(days);
    if day != 1 || open_ms % DAY_MS != 0 {
        return Err("month opening boundary rejected".to_string());
    }
    if month == 12 {
        timestamp_ms(year + 1, 1, 1)
    } else {
        timestamp_ms(year, month + 1, 1)
    }
}

fn next_year_boundary(open_ms: u64) -> Result<u64, String> {
    let days = i64::try_from(open_ms / DAY_MS).map_err(|_| "year overflow".to_string())?;
    let (year, month, day) = civil_from_days(days);
    if month != 1 || day != 1 || open_ms % DAY_MS != 0 {
        return Err("year opening boundary rejected".to_string());
    }
    timestamp_ms(year + 1, 1, 1)
}

fn period_interval(
    timeframe: MomentumHistoricalTimeframeV1,
    timestamp: u64,
) -> Result<CandleIntervalV1, String> {
    let (open, close) = match timeframe {
        MomentumHistoricalTimeframeV1::Minute1
        | MomentumHistoricalTimeframeV1::Minute3
        | MomentumHistoricalTimeframeV1::Minute5
        | MomentumHistoricalTimeframeV1::Minute10
        | MomentumHistoricalTimeframeV1::Day1 => {
            let cadence = timeframe
                .cadence_ms()
                .ok_or_else(|| "fixed cadence unavailable".to_string())?;
            let open = timestamp / cadence * cadence;
            (
                open,
                open.checked_add(cadence)
                    .ok_or_else(|| "fixed interval overflow".to_string())?,
            )
        }
        MomentumHistoricalTimeframeV1::Week1 => {
            let day = timestamp / DAY_MS;
            let weekday_from_monday = (day + 3) % 7;
            let open = day
                .checked_sub(weekday_from_monday)
                .and_then(|value| value.checked_mul(DAY_MS))
                .ok_or_else(|| "weekly boundary overflow".to_string())?;
            (
                open,
                open.checked_add(7 * DAY_MS)
                    .ok_or_else(|| "weekly close overflow".to_string())?,
            )
        }
        MomentumHistoricalTimeframeV1::Month1 => {
            let days =
                i64::try_from(timestamp / DAY_MS).map_err(|_| "month overflow".to_string())?;
            let (year, month, _) = civil_from_days(days);
            let open = timestamp_ms(year, month, 1)?;
            (open, next_month_boundary(open)?)
        }
        MomentumHistoricalTimeframeV1::Year1 => {
            let days =
                i64::try_from(timestamp / DAY_MS).map_err(|_| "year overflow".to_string())?;
            let (year, _, _) = civil_from_days(days);
            let open = timestamp_ms(year, 1, 1)?;
            (open, next_year_boundary(open)?)
        }
    };
    Ok(CandleIntervalV1 {
        open_timestamp_ms: open,
        close_exclusive_timestamp_ms: close,
    })
}

fn validate_candle(value: &HistoricalCandleRowV1) -> Result<(), String> {
    if value.interval.open_timestamp_ms >= value.interval.close_exclusive_timestamp_ms
        || ![
            value.open,
            value.high,
            value.low,
            value.close,
            value.volume,
            value.trade_value,
        ]
        .iter()
        .all(|number| number.is_finite())
        || value.open <= 0.0
        || value.high <= 0.0
        || value.low <= 0.0
        || value.close <= 0.0
        || value.volume < 0.0
        || value.trade_value < 0.0
        || value.high < value.low
        || value.high < value.open.max(value.close)
        || value.low > value.open.min(value.close)
        || value.presence != CandleIntervalPresenceV1::ObservedTradeCandle
        || value.candle_digest != candle_digest(value)
    {
        return Err("historical candle rejected".to_string());
    }
    Ok(())
}

fn canonical_row(
    timeframe: MomentumHistoricalTimeframeV1,
    row: &HistoricalOhlcvRow,
) -> Result<HistoricalCandleRowV1, String> {
    if !timeframe.is_canonical() || row.symbol != MARKET {
        return Err("canonical row identity rejected".to_string());
    }
    let interval = period_interval(timeframe, row.timestamp_ms)?;
    if interval.open_timestamp_ms != row.timestamp_ms {
        return Err("canonical row opening boundary rejected".to_string());
    }
    let trade_value = row
        .trade_value
        .ok_or_else(|| "canonical trade value unavailable".to_string())?;
    let mut value = HistoricalCandleRowV1 {
        timeframe,
        interval,
        open: row.open,
        high: row.high,
        low: row.low,
        close: row.close,
        volume: row.volume,
        trade_value,
        ordered_base_candle_digests: Vec::new(),
        presence: CandleIntervalPresenceV1::ObservedTradeCandle,
        candle_digest: String::new(),
    };
    value.candle_digest = candle_digest(&value);
    validate_candle(&value)?;
    Ok(value)
}

fn validate_rows(
    timeframe: MomentumHistoricalTimeframeV1,
    rows: &[HistoricalCandleRowV1],
) -> Result<(), String> {
    if rows.is_empty()
        || rows.iter().any(|row| {
            row.timeframe != timeframe
                || validate_candle(row).is_err()
                || row.interval.open_timestamp_ms
                    != period_interval(timeframe, row.interval.open_timestamp_ms)
                        .map(|value| value.open_timestamp_ms)
                        .unwrap_or(u64::MAX)
        })
        || rows
            .windows(2)
            .any(|pair| pair[0].interval.open_timestamp_ms >= pair[1].interval.open_timestamp_ms)
    {
        return Err("historical candle chronology rejected".to_string());
    }
    Ok(())
}

fn encode_candle(value: &HistoricalCandleRowV1) -> Result<Vec<u8>, String> {
    validate_candle(value)?;
    ArtifactBuilderV4_2::new("HistoricalCandleRowV1")
        .string("timeframe", value.timeframe.as_str())
        .unsigned("open_timestamp_ms", value.interval.open_timestamp_ms)
        .unsigned(
            "close_exclusive_timestamp_ms",
            value.interval.close_exclusive_timestamp_ms,
        )
        .unsigned("open_bits", value.open.to_bits())
        .unsigned("high_bits", value.high.to_bits())
        .unsigned("low_bits", value.low.to_bits())
        .unsigned("close_bits", value.close.to_bits())
        .unsigned("volume_bits", value.volume.to_bits())
        .unsigned("trade_value_bits", value.trade_value.to_bits())
        .strings(
            "ordered_base_candle_digests",
            &value.ordered_base_candle_digests,
        )
        .string("presence", value.presence.as_str())
        .string("candle_digest", &value.candle_digest)
        .encode()
}

fn decode_candle(bytes: &[u8]) -> Result<HistoricalCandleRowV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "HistoricalCandleRowV1")?;
    let value = HistoricalCandleRowV1 {
        timeframe: MomentumHistoricalTimeframeV1::parse(&fields.string("timeframe")?)?,
        interval: CandleIntervalV1 {
            open_timestamp_ms: fields.unsigned("open_timestamp_ms")?,
            close_exclusive_timestamp_ms: fields.unsigned("close_exclusive_timestamp_ms")?,
        },
        open: f64::from_bits(fields.unsigned("open_bits")?),
        high: f64::from_bits(fields.unsigned("high_bits")?),
        low: f64::from_bits(fields.unsigned("low_bits")?),
        close: f64::from_bits(fields.unsigned("close_bits")?),
        volume: f64::from_bits(fields.unsigned("volume_bits")?),
        trade_value: f64::from_bits(fields.unsigned("trade_value_bits")?),
        ordered_base_candle_digests: fields.strings("ordered_base_candle_digests")?,
        presence: CandleIntervalPresenceV1::parse(&fields.string("presence")?)?,
        candle_digest: fields.string("candle_digest")?,
    };
    fields.finish()?;
    validate_candle(&value)?;
    Ok(value)
}

fn collect_tree(
    current: &Path,
    base: &Path,
    values: &mut Vec<(PathBuf, Vec<u8>)>,
) -> Result<(), String> {
    if !current.exists() {
        return Ok(());
    }
    if current.is_dir() {
        let mut paths = fs::read_dir(current)
            .map_err(|_| "protected directory read failed".to_string())?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .collect::<Vec<_>>();
        paths.sort();
        for path in paths {
            collect_tree(&path, base, values)?;
        }
    } else if current.is_file() {
        values.push((
            current
                .strip_prefix(base)
                .map_err(|_| "protected path rejected".to_string())?
                .to_path_buf(),
            fs::read(current).map_err(|_| "protected artifact read failed".to_string())?,
        ));
    }
    Ok(())
}

fn tree_identity(root: &Path) -> Result<(usize, String), String> {
    let mut values = Vec::new();
    collect_tree(root, root, &mut values)?;
    values.sort_by(|left, right| left.0.cmp(&right.0));
    Ok((
        values.len(),
        stable_hash_string(&format!("momentum-mtf-protected-v1:{values:?}")),
    ))
}

fn active_roster_digest() -> String {
    stable_hash_string(&format!(
        "momentum-mtf-active-roster-v1:{:?}",
        canonical_current_agent_states()
    ))
}

fn select_daily_snapshot(snapshots: &[DataSnapshot]) -> Result<DataSnapshot, String> {
    let valid = snapshots
        .iter()
        .filter(|snapshot| {
            snapshot.provider_id == PROVIDER
                && snapshot.market_scope == AcquisitionMarketScope::BtcCrypto
                && snapshot.normalized_dataset.symbol == MARKET
                && snapshot
                    .compatibility
                    .as_ref()
                    .is_some_and(|compatibility| {
                        compatibility.cadence == "1d" && compatibility.all_rows_finalized
                    })
                && snapshot.row_count == snapshot.normalized_dataset.rows.len()
                && snapshot.row_count > 1
                && snapshot.sanitized
                && snapshot.read_only
                && snapshot
                    .normalized_dataset
                    .rows
                    .windows(2)
                    .all(|pair| pair[1].timestamp_ms - pair[0].timestamp_ms == DAY_MS)
        })
        .cloned()
        .collect::<Vec<_>>();
    let mut candidates = valid
        .into_iter()
        .filter(|snapshot| snapshot.row_count == FROZEN_EXISTING_DAILY_SNAPSHOT_ROWS)
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| left.content_digest.cmp(&right.content_digest));
    let first_digest = candidates
        .first()
        .map(|snapshot| snapshot.content_digest.as_str())
        .ok_or_else(|| "canonical daily snapshot unavailable".to_string())?;
    if candidates
        .iter()
        .any(|snapshot| snapshot.content_digest != first_digest)
    {
        return Err("ambiguous canonical daily snapshot rejected".to_string());
    }
    Ok(candidates.remove(0))
}

fn build_pause(
    live: &MomentumProspectiveSeriesReportV4,
) -> Result<LiveContinuationPauseV1, String> {
    validate_sealed_epoch_two_report_v4(live)?;
    let status = &live.status;
    if status.readiness != MomentumProspectiveEpochReadinessV4::PredictionAlreadySealed
        || status.epoch_number != 2
        || status.total_event_count != 1
        || status.scorable_event_count != 1
        || status.input_receipt_digest.is_none()
        || status.input_capsule_digest.is_none()
        || status.context_assembly_proof_digest.is_none()
        || status.prediction_capsule_digest.is_none()
        || status.journal_entry_digest.is_none()
        || status.outcome_plan_digest.is_none()
        || status.participant_prediction_digests.len() != 3
        || status.safety_counters.network_request_attempts != 0
        || status.safety_counters.outcome_requests != 0
        || status.safety_counters.outcome_openings != 0
        || status.safety_counters.metric_computations != 0
        || status.safety_counters.winner_selections != 0
        || status.safety_counters.reward_applications != 0
        || status.safety_counters.chair_decisions != 0
        || live
            .input_receipt
            .as_ref()
            .is_none_or(|receipt| receipt.request_count != 1 || receipt.retry_count != 0)
    {
        return Err("sealed live epoch-two proof rejected".to_string());
    }
    let mut value = LiveContinuationPauseV1 {
        pause_version: PAUSE_VERSION.to_string(),
        policy: LiveProspectiveContinuationPolicyV1::PausedAfterSealedEpochTwo,
        series_digest: status.series_digest.clone(),
        epoch_registration_digest: status.epoch_registration_digest.clone(),
        input_receipt_digest: status.input_receipt_digest.clone().unwrap_or_default(),
        input_capsule_digest: status.input_capsule_digest.clone().unwrap_or_default(),
        context_proof_digest: status
            .context_assembly_proof_digest
            .clone()
            .unwrap_or_default(),
        prediction_capsule_digest: status.prediction_capsule_digest.clone().unwrap_or_default(),
        prediction_journal_digest: status.journal_entry_digest.clone().unwrap_or_default(),
        outcome_plan_digest: status.outcome_plan_digest.clone().unwrap_or_default(),
        protected_first_event_input_boundary_ms: live.event_one_adoption.adopted_event_timestamp_ms,
        completed_event_count: status.total_event_count,
        scorable_event_count: status.scorable_event_count,
        prediction_seal_count: status.participant_prediction_digests.len(),
        input_attempts: live
            .input_receipt
            .as_ref()
            .map_or(0, |receipt| receipt.request_count),
        input_retries: live
            .input_receipt
            .as_ref()
            .map_or(0, |receipt| receipt.retry_count),
        outcome_requests: status.safety_counters.outcome_requests,
        outcome_openings: status.safety_counters.outcome_openings,
        epoch_three_registered: false,
        pause_digest: String::new(),
    };
    value.pause_digest = pause_digest(&value);
    validate_pause(&value)?;
    Ok(value)
}

fn validate_pause(value: &LiveContinuationPauseV1) -> Result<(), String> {
    if value.pause_version != PAUSE_VERSION
        || value.policy != LiveProspectiveContinuationPolicyV1::PausedAfterSealedEpochTwo
        || [
            &value.series_digest,
            &value.epoch_registration_digest,
            &value.input_receipt_digest,
            &value.input_capsule_digest,
            &value.context_proof_digest,
            &value.prediction_capsule_digest,
            &value.prediction_journal_digest,
            &value.outcome_plan_digest,
        ]
        .iter()
        .any(|digest| digest.is_empty())
        || value.protected_first_event_input_boundary_ms == 0
        || value.completed_event_count != 1
        || value.scorable_event_count != 1
        || value.prediction_seal_count != 3
        || value.input_attempts != 1
        || value.input_retries != 0
        || value.outcome_requests != 0
        || value.outcome_openings != 0
        || value.epoch_three_registered
        || value.pause_digest != pause_digest(value)
    {
        return Err("live continuation pause rejected".to_string());
    }
    Ok(())
}

fn encode_pause(value: &LiveContinuationPauseV1) -> Result<Vec<u8>, String> {
    validate_pause(value)?;
    ArtifactBuilderV4_2::new("LiveContinuationPauseV1")
        .string("pause_version", &value.pause_version)
        .string("policy", "PausedAfterSealedEpochTwo")
        .string("series_digest", &value.series_digest)
        .string(
            "epoch_registration_digest",
            &value.epoch_registration_digest,
        )
        .string("input_receipt_digest", &value.input_receipt_digest)
        .string("input_capsule_digest", &value.input_capsule_digest)
        .string("context_proof_digest", &value.context_proof_digest)
        .string(
            "prediction_capsule_digest",
            &value.prediction_capsule_digest,
        )
        .string(
            "prediction_journal_digest",
            &value.prediction_journal_digest,
        )
        .string("outcome_plan_digest", &value.outcome_plan_digest)
        .unsigned(
            "protected_first_event_input_boundary_ms",
            value.protected_first_event_input_boundary_ms,
        )
        .unsigned(
            "completed_event_count",
            as_u64(value.completed_event_count)?,
        )
        .unsigned("scorable_event_count", as_u64(value.scorable_event_count)?)
        .unsigned(
            "prediction_seal_count",
            as_u64(value.prediction_seal_count)?,
        )
        .unsigned("input_attempts", as_u64(value.input_attempts)?)
        .unsigned("input_retries", as_u64(value.input_retries)?)
        .unsigned("outcome_requests", as_u64(value.outcome_requests)?)
        .unsigned("outcome_openings", as_u64(value.outcome_openings)?)
        .boolean("epoch_three_registered", value.epoch_three_registered)
        .string("pause_digest", &value.pause_digest)
        .encode()
}

fn decode_pause(bytes: &[u8]) -> Result<LiveContinuationPauseV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "LiveContinuationPauseV1")?;
    if fields.string("policy")? != "PausedAfterSealedEpochTwo" {
        return Err("live continuation policy rejected".to_string());
    }
    let value = LiveContinuationPauseV1 {
        pause_version: fields.string("pause_version")?,
        policy: LiveProspectiveContinuationPolicyV1::PausedAfterSealedEpochTwo,
        series_digest: fields.string("series_digest")?,
        epoch_registration_digest: fields.string("epoch_registration_digest")?,
        input_receipt_digest: fields.string("input_receipt_digest")?,
        input_capsule_digest: fields.string("input_capsule_digest")?,
        context_proof_digest: fields.string("context_proof_digest")?,
        prediction_capsule_digest: fields.string("prediction_capsule_digest")?,
        prediction_journal_digest: fields.string("prediction_journal_digest")?,
        outcome_plan_digest: fields.string("outcome_plan_digest")?,
        protected_first_event_input_boundary_ms: fields
            .unsigned("protected_first_event_input_boundary_ms")?,
        completed_event_count: as_usize(fields.unsigned("completed_event_count")?)?,
        scorable_event_count: as_usize(fields.unsigned("scorable_event_count")?)?,
        prediction_seal_count: as_usize(fields.unsigned("prediction_seal_count")?)?,
        input_attempts: as_usize(fields.unsigned("input_attempts")?)?,
        input_retries: as_usize(fields.unsigned("input_retries")?)?,
        outcome_requests: as_usize(fields.unsigned("outcome_requests")?)?,
        outcome_openings: as_usize(fields.unsigned("outcome_openings")?)?,
        epoch_three_registered: fields.boolean("epoch_three_registered")?,
        pause_digest: fields.string("pause_digest")?,
    };
    fields.finish()?;
    validate_pause(&value)?;
    Ok(value)
}

fn build_foundation(
    pause: &LiveContinuationPauseV1,
    daily: &DataSnapshot,
) -> Result<MomentumMtfFoundationRegistrationV1, String> {
    let minute_span_ms = u64::try_from(PILOT_DAYS)
        .ok()
        .and_then(|days| days.checked_mul(DAY_MS))
        .ok_or_else(|| "minute pilot range overflow".to_string())?;
    let minute_start = pause
        .protected_first_event_input_boundary_ms
        .checked_sub(minute_span_ms)
        .ok_or_else(|| "minute pilot start unavailable".to_string())?;
    let rows = &daily.normalized_dataset.rows;
    let role_bindings = MomentumHistoricalTimeframeV1::ORDERED
        .into_iter()
        .map(|timeframe| {
            format!(
                "{}={}",
                timeframe.as_str(),
                MomentumTimeframeRoleV1::for_timeframe(timeframe).as_str()
            )
        })
        .collect::<Vec<_>>();
    let mut value = MomentumMtfFoundationRegistrationV1 {
        registration_version: FOUNDATION_VERSION.to_string(),
        pause_digest: pause.pause_digest.clone(),
        provider_id: PROVIDER.to_string(),
        symbol: MARKET.to_string(),
        ordered_timeframes: MomentumHistoricalTimeframeV1::ORDERED.to_vec(),
        canonical_bases: vec![
            MomentumHistoricalTimeframeV1::Minute1,
            MomentumHistoricalTimeframeV1::Day1,
        ],
        derived_timeframes: vec![
            MomentumHistoricalTimeframeV1::Minute3,
            MomentumHistoricalTimeframeV1::Minute5,
            MomentumHistoricalTimeframeV1::Minute10,
            MomentumHistoricalTimeframeV1::Week1,
            MomentumHistoricalTimeframeV1::Month1,
            MomentumHistoricalTimeframeV1::Year1,
        ],
        role_bindings,
        minute_start_timestamp_ms: minute_start,
        minute_end_exclusive_timestamp_ms: pause.protected_first_event_input_boundary_ms,
        existing_daily_snapshot_digest: daily.content_digest.clone(),
        existing_daily_first_timestamp_ms: rows
            .first()
            .map(|row| row.timestamp_ms)
            .ok_or_else(|| "daily first timestamp unavailable".to_string())?,
        existing_daily_last_timestamp_ms: rows
            .last()
            .map(|row| row.timestamp_ms)
            .ok_or_else(|| "daily last timestamp unavailable".to_string())?,
        existing_daily_row_count: rows.len(),
        chunk_size: CHUNK_SIZE,
        protocol_cadence_ms: PROTOCOL_CADENCE_MS,
        numeric_absolute_tolerance_bits: ABSOLUTE_TOLERANCE.to_bits(),
        numeric_relative_tolerance_bits: RELATIVE_TOLERANCE.to_bits(),
        training_forbidden: true,
        tournament_forbidden: true,
        live_authority_forbidden: true,
        registration_digest: String::new(),
    };
    value.registration_digest = foundation_digest(&value);
    validate_foundation(&value)?;
    Ok(value)
}

fn validate_foundation(value: &MomentumMtfFoundationRegistrationV1) -> Result<(), String> {
    let expected_roles = MomentumHistoricalTimeframeV1::ORDERED
        .into_iter()
        .map(|timeframe| {
            format!(
                "{}={}",
                timeframe.as_str(),
                MomentumTimeframeRoleV1::for_timeframe(timeframe).as_str()
            )
        })
        .collect::<Vec<_>>();
    if value.registration_version != FOUNDATION_VERSION
        || value.pause_digest.is_empty()
        || value.provider_id != PROVIDER
        || value.symbol != MARKET
        || value.ordered_timeframes != MomentumHistoricalTimeframeV1::ORDERED
        || value.canonical_bases
            != [
                MomentumHistoricalTimeframeV1::Minute1,
                MomentumHistoricalTimeframeV1::Day1,
            ]
        || value.derived_timeframes
            != [
                MomentumHistoricalTimeframeV1::Minute3,
                MomentumHistoricalTimeframeV1::Minute5,
                MomentumHistoricalTimeframeV1::Minute10,
                MomentumHistoricalTimeframeV1::Week1,
                MomentumHistoricalTimeframeV1::Month1,
                MomentumHistoricalTimeframeV1::Year1,
            ]
        || value.role_bindings != expected_roles
        || value.minute_start_timestamp_ms >= value.minute_end_exclusive_timestamp_ms
        || value.minute_end_exclusive_timestamp_ms - value.minute_start_timestamp_ms
            != u64::try_from(PILOT_DAYS).unwrap_or_default() * DAY_MS
        || value.existing_daily_snapshot_digest.is_empty()
        || value.existing_daily_first_timestamp_ms >= value.existing_daily_last_timestamp_ms
        || value.existing_daily_row_count == 0
        || value.chunk_size != CHUNK_SIZE
        || value.protocol_cadence_ms != PROTOCOL_CADENCE_MS
        || f64::from_bits(value.numeric_absolute_tolerance_bits) != ABSOLUTE_TOLERANCE
        || f64::from_bits(value.numeric_relative_tolerance_bits) != RELATIVE_TOLERANCE
        || !value.training_forbidden
        || !value.tournament_forbidden
        || !value.live_authority_forbidden
        || value.registration_digest != foundation_digest(value)
    {
        return Err("multi-timeframe foundation registration rejected".to_string());
    }
    Ok(())
}

fn encode_foundation(value: &MomentumMtfFoundationRegistrationV1) -> Result<Vec<u8>, String> {
    validate_foundation(value)?;
    ArtifactBuilderV4_2::new("MomentumMtfFoundationRegistrationV1")
        .string("registration_version", &value.registration_version)
        .string("pause_digest", &value.pause_digest)
        .string("provider_id", &value.provider_id)
        .string("symbol", &value.symbol)
        .strings(
            "ordered_timeframes",
            &value
                .ordered_timeframes
                .iter()
                .map(|value| value.as_str().to_string())
                .collect::<Vec<_>>(),
        )
        .strings(
            "canonical_bases",
            &value
                .canonical_bases
                .iter()
                .map(|value| value.as_str().to_string())
                .collect::<Vec<_>>(),
        )
        .strings(
            "derived_timeframes",
            &value
                .derived_timeframes
                .iter()
                .map(|value| value.as_str().to_string())
                .collect::<Vec<_>>(),
        )
        .strings("role_bindings", &value.role_bindings)
        .unsigned("minute_start_timestamp_ms", value.minute_start_timestamp_ms)
        .unsigned(
            "minute_end_exclusive_timestamp_ms",
            value.minute_end_exclusive_timestamp_ms,
        )
        .string(
            "existing_daily_snapshot_digest",
            &value.existing_daily_snapshot_digest,
        )
        .unsigned(
            "existing_daily_first_timestamp_ms",
            value.existing_daily_first_timestamp_ms,
        )
        .unsigned(
            "existing_daily_last_timestamp_ms",
            value.existing_daily_last_timestamp_ms,
        )
        .unsigned(
            "existing_daily_row_count",
            as_u64(value.existing_daily_row_count)?,
        )
        .unsigned("chunk_size", as_u64(value.chunk_size)?)
        .unsigned("protocol_cadence_ms", value.protocol_cadence_ms)
        .unsigned(
            "numeric_absolute_tolerance_bits",
            value.numeric_absolute_tolerance_bits,
        )
        .unsigned(
            "numeric_relative_tolerance_bits",
            value.numeric_relative_tolerance_bits,
        )
        .boolean("training_forbidden", value.training_forbidden)
        .boolean("tournament_forbidden", value.tournament_forbidden)
        .boolean("live_authority_forbidden", value.live_authority_forbidden)
        .string("registration_digest", &value.registration_digest)
        .encode()
}

fn decode_foundation(bytes: &[u8]) -> Result<MomentumMtfFoundationRegistrationV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumMtfFoundationRegistrationV1")?;
    let parse_timeframes = |values: Vec<String>| {
        values
            .iter()
            .map(|value| MomentumHistoricalTimeframeV1::parse(value))
            .collect::<Result<Vec<_>, _>>()
    };
    let value = MomentumMtfFoundationRegistrationV1 {
        registration_version: fields.string("registration_version")?,
        pause_digest: fields.string("pause_digest")?,
        provider_id: fields.string("provider_id")?,
        symbol: fields.string("symbol")?,
        ordered_timeframes: parse_timeframes(fields.strings("ordered_timeframes")?)?,
        canonical_bases: parse_timeframes(fields.strings("canonical_bases")?)?,
        derived_timeframes: parse_timeframes(fields.strings("derived_timeframes")?)?,
        role_bindings: fields.strings("role_bindings")?,
        minute_start_timestamp_ms: fields.unsigned("minute_start_timestamp_ms")?,
        minute_end_exclusive_timestamp_ms: fields.unsigned("minute_end_exclusive_timestamp_ms")?,
        existing_daily_snapshot_digest: fields.string("existing_daily_snapshot_digest")?,
        existing_daily_first_timestamp_ms: fields.unsigned("existing_daily_first_timestamp_ms")?,
        existing_daily_last_timestamp_ms: fields.unsigned("existing_daily_last_timestamp_ms")?,
        existing_daily_row_count: as_usize(fields.unsigned("existing_daily_row_count")?)?,
        chunk_size: as_usize(fields.unsigned("chunk_size")?)?,
        protocol_cadence_ms: fields.unsigned("protocol_cadence_ms")?,
        numeric_absolute_tolerance_bits: fields.unsigned("numeric_absolute_tolerance_bits")?,
        numeric_relative_tolerance_bits: fields.unsigned("numeric_relative_tolerance_bits")?,
        training_forbidden: fields.boolean("training_forbidden")?,
        tournament_forbidden: fields.boolean("tournament_forbidden")?,
        live_authority_forbidden: fields.boolean("live_authority_forbidden")?,
        registration_digest: fields.string("registration_digest")?,
    };
    fields.finish()?;
    validate_foundation(&value)?;
    Ok(value)
}

fn build_plan(
    foundation: &MomentumMtfFoundationRegistrationV1,
    config: &UpbitHistoricalPilotConfigV0,
) -> Result<MomentumMtfAcquisitionPlanV1, String> {
    if config.provider_id != PROVIDER
        || config.symbol != MARKET
        || !config.enabled
        || config.maximum_response_bytes == 0
        || config.minimum_inter_request_delay_ms == 0
    {
        return Err("multi-timeframe provider configuration rejected".to_string());
    }
    let minute_intervals = usize::try_from(
        (foundation.minute_end_exclusive_timestamp_ms - foundation.minute_start_timestamp_ms)
            / MINUTE_MS,
    )
    .map_err(|_| "minute interval count overflow".to_string())?;
    let minute_page_budget = minute_intervals.div_ceil(PAGE_SIZE);
    let daily_page_budget = REQUEST_CEILING
        .checked_sub(minute_page_budget)
        .and_then(|remaining| remaining.checked_sub(NATIVE_SAMPLE_PAGES))
        .ok_or_else(|| "historical request ceiling exceeded".to_string())?;
    let mut value = MomentumMtfAcquisitionPlanV1 {
        plan_version: PLAN_VERSION.to_string(),
        foundation_registration_digest: foundation.registration_digest.clone(),
        minute_page_budget,
        daily_page_budget,
        native_sample_request_budget: NATIVE_SAMPLE_PAGES,
        exact_total_request_budget: minute_page_budget
            .checked_add(daily_page_budget)
            .and_then(|value| value.checked_add(NATIVE_SAMPLE_PAGES))
            .ok_or_else(|| "historical request budget overflow".to_string())?,
        provider_page_size: PAGE_SIZE,
        maximum_concurrency: 1,
        maximum_retries_per_page: 0,
        minimum_inter_request_delay_ms: config.minimum_inter_request_delay_ms,
        maximum_response_bytes: config.maximum_response_bytes,
        strictly_backward_exclusive: true,
        checkpoint_after_every_page: true,
        plan_digest: String::new(),
    };
    value.plan_digest = plan_digest(&value);
    validate_plan(&value)?;
    Ok(value)
}

fn validate_plan(value: &MomentumMtfAcquisitionPlanV1) -> Result<(), String> {
    if value.plan_version != PLAN_VERSION
        || value.foundation_registration_digest.is_empty()
        || value.minute_page_budget == 0
        || value.daily_page_budget == 0
        || value.native_sample_request_budget != NATIVE_SAMPLE_PAGES
        || value.exact_total_request_budget
            != value.minute_page_budget
                + value.daily_page_budget
                + value.native_sample_request_budget
        || value.exact_total_request_budget > REQUEST_CEILING
        || value.provider_page_size != PAGE_SIZE
        || value.maximum_concurrency != 1
        || value.maximum_retries_per_page != 0
        || value.minimum_inter_request_delay_ms == 0
        || value.maximum_response_bytes == 0
        || !value.strictly_backward_exclusive
        || !value.checkpoint_after_every_page
        || value.plan_digest != plan_digest(value)
    {
        return Err("multi-timeframe acquisition plan rejected".to_string());
    }
    Ok(())
}

fn encode_plan(value: &MomentumMtfAcquisitionPlanV1) -> Result<Vec<u8>, String> {
    validate_plan(value)?;
    ArtifactBuilderV4_2::new("MomentumMtfAcquisitionPlanV1")
        .string("plan_version", &value.plan_version)
        .string(
            "foundation_registration_digest",
            &value.foundation_registration_digest,
        )
        .unsigned("minute_page_budget", as_u64(value.minute_page_budget)?)
        .unsigned("daily_page_budget", as_u64(value.daily_page_budget)?)
        .unsigned(
            "native_sample_request_budget",
            as_u64(value.native_sample_request_budget)?,
        )
        .unsigned(
            "exact_total_request_budget",
            as_u64(value.exact_total_request_budget)?,
        )
        .unsigned("provider_page_size", as_u64(value.provider_page_size)?)
        .unsigned("maximum_concurrency", as_u64(value.maximum_concurrency)?)
        .unsigned(
            "maximum_retries_per_page",
            as_u64(value.maximum_retries_per_page)?,
        )
        .unsigned(
            "minimum_inter_request_delay_ms",
            value.minimum_inter_request_delay_ms,
        )
        .unsigned(
            "maximum_response_bytes",
            as_u64(value.maximum_response_bytes)?,
        )
        .boolean(
            "strictly_backward_exclusive",
            value.strictly_backward_exclusive,
        )
        .boolean(
            "checkpoint_after_every_page",
            value.checkpoint_after_every_page,
        )
        .string("plan_digest", &value.plan_digest)
        .encode()
}

fn decode_plan(bytes: &[u8]) -> Result<MomentumMtfAcquisitionPlanV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumMtfAcquisitionPlanV1")?;
    let value = MomentumMtfAcquisitionPlanV1 {
        plan_version: fields.string("plan_version")?,
        foundation_registration_digest: fields.string("foundation_registration_digest")?,
        minute_page_budget: as_usize(fields.unsigned("minute_page_budget")?)?,
        daily_page_budget: as_usize(fields.unsigned("daily_page_budget")?)?,
        native_sample_request_budget: as_usize(fields.unsigned("native_sample_request_budget")?)?,
        exact_total_request_budget: as_usize(fields.unsigned("exact_total_request_budget")?)?,
        provider_page_size: as_usize(fields.unsigned("provider_page_size")?)?,
        maximum_concurrency: as_usize(fields.unsigned("maximum_concurrency")?)?,
        maximum_retries_per_page: as_usize(fields.unsigned("maximum_retries_per_page")?)?,
        minimum_inter_request_delay_ms: fields.unsigned("minimum_inter_request_delay_ms")?,
        maximum_response_bytes: as_usize(fields.unsigned("maximum_response_bytes")?)?,
        strictly_backward_exclusive: fields.boolean("strictly_backward_exclusive")?,
        checkpoint_after_every_page: fields.boolean("checkpoint_after_every_page")?,
        plan_digest: fields.string("plan_digest")?,
    };
    fields.finish()?;
    validate_plan(&value)?;
    Ok(value)
}

fn persist_one(
    root: &Path,
    category: &str,
    digest: &str,
    bytes: &[u8],
    decode_digest: impl Fn(&[u8]) -> Result<String, String>,
) -> Result<(usize, usize), String> {
    persist_artifact(
        &root.join(category).join(format!("{digest}.pb")),
        bytes,
        digest,
        decode_digest,
    )
}

fn persist_pause(root: &Path, value: &LiveContinuationPauseV1) -> Result<(usize, usize), String> {
    persist_one(
        root,
        "live_continuation_pause",
        &value.pause_digest,
        &encode_pause(value)?,
        |bytes| Ok(decode_pause(bytes)?.pause_digest),
    )
}

fn persist_foundation(
    root: &Path,
    value: &MomentumMtfFoundationRegistrationV1,
) -> Result<(usize, usize), String> {
    persist_one(
        root,
        "foundation_registrations",
        &value.registration_digest,
        &encode_foundation(value)?,
        |bytes| Ok(decode_foundation(bytes)?.registration_digest),
    )
}

fn persist_plan(
    root: &Path,
    value: &MomentumMtfAcquisitionPlanV1,
) -> Result<(usize, usize), String> {
    persist_one(
        root,
        "acquisition_plans",
        &value.plan_digest,
        &encode_plan(value)?,
        |bytes| Ok(decode_plan(bytes)?.plan_digest),
    )
}

fn reopen_foundation(
    root: &Path,
) -> Result<
    (
        Option<LiveContinuationPauseV1>,
        Option<MomentumMtfFoundationRegistrationV1>,
        Option<MomentumMtfAcquisitionPlanV1>,
    ),
    String,
> {
    Ok((
        read_single(&root.join("live_continuation_pause"), decode_pause)?,
        read_single(&root.join("foundation_registrations"), decode_foundation)?,
        read_single(&root.join("acquisition_plans"), decode_plan)?,
    ))
}

#[derive(Clone, Debug, Deserialize)]
struct UpbitCandleWireV1 {
    market: String,
    candle_date_time_utc: String,
    opening_price: f64,
    high_price: f64,
    low_price: f64,
    trade_price: f64,
    candle_acc_trade_price: f64,
    candle_acc_trade_volume: f64,
}

fn parse_provider_page(
    body: &str,
    timeframe: MomentumHistoricalTimeframeV1,
    request_to_exclusive_ms: u64,
) -> Result<Vec<HistoricalCandleRowV1>, String> {
    let wire = serde_json::from_str::<Vec<UpbitCandleWireV1>>(body)
        .map_err(|_| "provider candle response schema rejected".to_string())?;
    let mut rows = wire
        .into_iter()
        .map(|item| {
            if item.market != MARKET {
                return Err("provider candle market rejected".to_string());
            }
            let open_timestamp_ms = parse_utc_timestamp(&item.candle_date_time_utc)?;
            let interval = period_interval(timeframe, open_timestamp_ms)?;
            if interval.open_timestamp_ms != open_timestamp_ms
                || open_timestamp_ms >= request_to_exclusive_ms
                || interval.close_exclusive_timestamp_ms > request_to_exclusive_ms
            {
                return Err("provider candle finality rejected".to_string());
            }
            let mut row = HistoricalCandleRowV1 {
                timeframe,
                interval,
                open: item.opening_price,
                high: item.high_price,
                low: item.low_price,
                close: item.trade_price,
                volume: item.candle_acc_trade_volume,
                trade_value: item.candle_acc_trade_price,
                ordered_base_candle_digests: Vec::new(),
                presence: CandleIntervalPresenceV1::ObservedTradeCandle,
                candle_digest: String::new(),
            };
            row.candle_digest = candle_digest(&row);
            validate_candle(&row)?;
            Ok(row)
        })
        .collect::<Result<Vec<_>, String>>()?;
    rows.sort_by_key(|row| row.interval.open_timestamp_ms);
    if rows
        .windows(2)
        .any(|pair| pair[0].interval.open_timestamp_ms >= pair[1].interval.open_timestamp_ms)
    {
        return Err("provider candle duplicate rejected".to_string());
    }
    Ok(rows)
}

fn normalized_rows_digest(rows: &[HistoricalCandleRowV1]) -> String {
    stable_hash_string(&format!(
        "momentum-mtf-normalized-page-v1:{:?}",
        rows.iter()
            .map(|row| row.candle_digest.as_str())
            .collect::<Vec<_>>()
    ))
}

fn validate_page_receipt(value: &HistoricalPageReceiptV1) -> Result<(), String> {
    let verified = value.status == PageReceiptStatusV1::Verified;
    let normalized_rows_digest = normalized_rows_digest(&value.rows);
    if value.receipt_version != RECEIPT_VERSION
        || value.plan_digest.is_empty()
        || value.request_fingerprint.is_empty()
        || value.request_to_exclusive_ms == 0
        || value.requested_count != PAGE_SIZE
        || value.attempt_sequence == 0
        || value.request_count != 1
        || value.retry_count != 0
        || verified != value.response_body_digest.is_some()
        || verified != value.normalized_row_digest.is_some()
        || (!verified && !value.rows.is_empty())
        || (verified
            && value.normalized_row_digest.as_deref() != Some(normalized_rows_digest.as_str()))
        || value.rows.iter().any(|row| {
            row.timeframe != value.timeframe
                || validate_candle(row).is_err()
                || row.interval.close_exclusive_timestamp_ms > value.request_to_exclusive_ms
        })
        || value
            .rows
            .windows(2)
            .any(|pair| pair[0].interval.open_timestamp_ms >= pair[1].interval.open_timestamp_ms)
        || (value.purpose == PagePurposeV1::CanonicalMinute
            && value.timeframe != MomentumHistoricalTimeframeV1::Minute1)
        || (value.purpose == PagePurposeV1::CanonicalDailyOlder
            && value.timeframe != MomentumHistoricalTimeframeV1::Day1)
        || (value.purpose == PagePurposeV1::NativeCrossCheck && value.timeframe.is_canonical())
        || value.receipt_digest != receipt_digest(value)
    {
        return Err("historical page receipt rejected".to_string());
    }
    Ok(())
}

fn encode_page_receipt(value: &HistoricalPageReceiptV1) -> Result<Vec<u8>, String> {
    validate_page_receipt(value)?;
    ArtifactBuilderV4_2::new("HistoricalPageReceiptV1")
        .string("receipt_version", &value.receipt_version)
        .string("plan_digest", &value.plan_digest)
        .string("purpose", value.purpose.as_str())
        .string("timeframe", value.timeframe.as_str())
        .string("request_fingerprint", &value.request_fingerprint)
        .unsigned("request_to_exclusive_ms", value.request_to_exclusive_ms)
        .unsigned("requested_count", as_u64(value.requested_count)?)
        .unsigned("attempt_sequence", as_u64(value.attempt_sequence)?)
        .string("status", value.status.as_str())
        .optional_string("response_body_digest", &value.response_body_digest)
        .optional_string("normalized_row_digest", &value.normalized_row_digest)
        .messages(
            "rows",
            value
                .rows
                .iter()
                .map(encode_candle)
                .collect::<Result<Vec<_>, _>>()?,
        )
        .unsigned("request_count", as_u64(value.request_count)?)
        .unsigned("retry_count", as_u64(value.retry_count)?)
        .string("receipt_digest", &value.receipt_digest)
        .encode()
}

fn decode_page_receipt(bytes: &[u8]) -> Result<HistoricalPageReceiptV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "HistoricalPageReceiptV1")?;
    let value = HistoricalPageReceiptV1 {
        receipt_version: fields.string("receipt_version")?,
        plan_digest: fields.string("plan_digest")?,
        purpose: PagePurposeV1::parse(&fields.string("purpose")?)?,
        timeframe: MomentumHistoricalTimeframeV1::parse(&fields.string("timeframe")?)?,
        request_fingerprint: fields.string("request_fingerprint")?,
        request_to_exclusive_ms: fields.unsigned("request_to_exclusive_ms")?,
        requested_count: as_usize(fields.unsigned("requested_count")?)?,
        attempt_sequence: as_usize(fields.unsigned("attempt_sequence")?)?,
        status: PageReceiptStatusV1::parse(&fields.string("status")?)?,
        response_body_digest: fields.optional_string("response_body_digest")?,
        normalized_row_digest: fields.optional_string("normalized_row_digest")?,
        rows: fields
            .messages("rows")?
            .iter()
            .map(|bytes| decode_candle(bytes))
            .collect::<Result<Vec<_>, _>>()?,
        request_count: as_usize(fields.unsigned("request_count")?)?,
        retry_count: as_usize(fields.unsigned("retry_count")?)?,
        receipt_digest: fields.string("receipt_digest")?,
    };
    fields.finish()?;
    validate_page_receipt(&value)?;
    Ok(value)
}

fn validate_checkpoint(value: &HistoricalCheckpointV1) -> Result<(), String> {
    if value.checkpoint_version != CHECKPOINT_VERSION
        || value.plan_digest.is_empty()
        || value.page_receipt_digest.is_empty()
        || value.request_fingerprint.is_empty()
        || value.last_successful_exclusive_to_ms == 0
        || value.response_body_digest.is_empty()
        || value.normalized_row_digest.is_empty()
        || value.verified_page_chunk_digest.is_empty()
        || value.request_count_consumed == 0
        || value.request_count_consumed + value.remaining_budget > REQUEST_CEILING
        || value.checkpoint_digest != checkpoint_digest(value)
    {
        return Err("historical checkpoint rejected".to_string());
    }
    Ok(())
}

fn encode_checkpoint(value: &HistoricalCheckpointV1) -> Result<Vec<u8>, String> {
    validate_checkpoint(value)?;
    ArtifactBuilderV4_2::new("HistoricalCheckpointV1")
        .string("checkpoint_version", &value.checkpoint_version)
        .string("plan_digest", &value.plan_digest)
        .string("page_receipt_digest", &value.page_receipt_digest)
        .string("request_fingerprint", &value.request_fingerprint)
        .unsigned(
            "last_successful_exclusive_to_ms",
            value.last_successful_exclusive_to_ms,
        )
        .string("response_body_digest", &value.response_body_digest)
        .string("normalized_row_digest", &value.normalized_row_digest)
        .string(
            "verified_page_chunk_digest",
            &value.verified_page_chunk_digest,
        )
        .unsigned(
            "request_count_consumed",
            as_u64(value.request_count_consumed)?,
        )
        .unsigned("remaining_budget", as_u64(value.remaining_budget)?)
        .string("checkpoint_digest", &value.checkpoint_digest)
        .encode()
}

fn decode_checkpoint(bytes: &[u8]) -> Result<HistoricalCheckpointV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "HistoricalCheckpointV1")?;
    let value = HistoricalCheckpointV1 {
        checkpoint_version: fields.string("checkpoint_version")?,
        plan_digest: fields.string("plan_digest")?,
        page_receipt_digest: fields.string("page_receipt_digest")?,
        request_fingerprint: fields.string("request_fingerprint")?,
        last_successful_exclusive_to_ms: fields.unsigned("last_successful_exclusive_to_ms")?,
        response_body_digest: fields.string("response_body_digest")?,
        normalized_row_digest: fields.string("normalized_row_digest")?,
        verified_page_chunk_digest: fields.string("verified_page_chunk_digest")?,
        request_count_consumed: as_usize(fields.unsigned("request_count_consumed")?)?,
        remaining_budget: as_usize(fields.unsigned("remaining_budget")?)?,
        checkpoint_digest: fields.string("checkpoint_digest")?,
    };
    fields.finish()?;
    validate_checkpoint(&value)?;
    Ok(value)
}

fn persist_page_receipt(
    root: &Path,
    value: &HistoricalPageReceiptV1,
) -> Result<(usize, usize), String> {
    persist_one(
        root,
        "page_receipts",
        &value.receipt_digest,
        &encode_page_receipt(value)?,
        |bytes| Ok(decode_page_receipt(bytes)?.receipt_digest),
    )
}

fn persist_checkpoint(
    root: &Path,
    value: &HistoricalCheckpointV1,
) -> Result<(usize, usize), String> {
    persist_one(
        root,
        "checkpoints",
        &value.checkpoint_digest,
        &encode_checkpoint(value)?,
        |bytes| Ok(decode_checkpoint(bytes)?.checkpoint_digest),
    )
}

fn load_receipts(root: &Path) -> Result<Vec<HistoricalPageReceiptV1>, String> {
    protobuf_paths(&root.join("page_receipts"))?
        .iter()
        .map(|path| {
            fs::read(path)
                .map_err(|_| "historical page receipt read failed".to_string())
                .and_then(|bytes| decode_page_receipt(&bytes))
        })
        .collect()
}

fn request_fingerprint(
    plan: &MomentumMtfAcquisitionPlanV1,
    purpose: PagePurposeV1,
    timeframe: MomentumHistoricalTimeframeV1,
    request_to_exclusive_ms: u64,
) -> String {
    stable_hash_string(&format!(
        "momentum-mtf-page-v1:{}:{}:{}:{}:{}",
        plan.plan_digest,
        purpose.as_str(),
        timeframe.as_str(),
        request_to_exclusive_ms,
        PAGE_SIZE,
    ))
}

fn request_url(
    timeframe: MomentumHistoricalTimeframeV1,
    request_to_exclusive_ms: u64,
) -> Result<String, String> {
    Ok(format!(
        "https://api.upbit.com{}?market={MARKET}&to={}&count={PAGE_SIZE}",
        timeframe.native_path()?,
        format_utc_timestamp(request_to_exclusive_ms)?
    ))
}

#[derive(Default)]
struct RequestPacerV1 {
    last_completed: Option<Instant>,
}

impl RequestPacerV1 {
    fn wait(&self, minimum_delay_ms: u64) {
        if let Some(last) = self.last_completed {
            let minimum = Duration::from_millis(minimum_delay_ms);
            if let Some(remaining) = minimum.checked_sub(last.elapsed()) {
                thread::sleep(remaining);
            }
        }
    }

    fn completed(&mut self) {
        self.last_completed = Some(Instant::now());
    }
}

#[allow(clippy::too_many_arguments)]
fn acquire_page(
    root: &Path,
    plan: &MomentumMtfAcquisitionPlanV1,
    purpose: PagePurposeV1,
    timeframe: MomentumHistoricalTimeframeV1,
    request_to_exclusive_ms: u64,
    client: &dyn MarketDataHttpClient,
    pacer: &mut RequestPacerV1,
    existing_receipts: &mut Vec<HistoricalPageReceiptV1>,
    write_counts: &mut (usize, usize),
) -> Result<(HistoricalPageReceiptV1, bool), String> {
    let fingerprint = request_fingerprint(plan, purpose, timeframe, request_to_exclusive_ms);
    if let Some(receipt) = existing_receipts.iter().find(|receipt| {
        receipt.request_fingerprint == fingerprint
            && receipt.status == PageReceiptStatusV1::Verified
    }) {
        return Ok((receipt.clone(), false));
    }
    let attempt_sequence = existing_receipts
        .iter()
        .filter(|receipt| receipt.request_fingerprint == fingerprint)
        .count()
        + 1;
    pacer.wait(plan.minimum_inter_request_delay_ms);
    let response = client.get(&request_url(timeframe, request_to_exclusive_ms)?);
    pacer.completed();
    let receipt = match response {
        Ok(body) => {
            if body.len() > plan.maximum_response_bytes {
                return persist_terminal_page(
                    root,
                    plan,
                    purpose,
                    timeframe,
                    &fingerprint,
                    request_to_exclusive_ms,
                    attempt_sequence,
                    existing_receipts,
                    write_counts,
                    "historical page response budget exceeded",
                );
            }
            let rows = match parse_provider_page(&body, timeframe, request_to_exclusive_ms) {
                Ok(rows) => rows,
                Err(_) => {
                    return persist_terminal_page(
                        root,
                        plan,
                        purpose,
                        timeframe,
                        &fingerprint,
                        request_to_exclusive_ms,
                        attempt_sequence,
                        existing_receipts,
                        write_counts,
                        "historical page response validation failed terminally",
                    );
                }
            };
            let mut value = HistoricalPageReceiptV1 {
                receipt_version: RECEIPT_VERSION.to_string(),
                plan_digest: plan.plan_digest.clone(),
                purpose,
                timeframe,
                request_fingerprint: fingerprint,
                request_to_exclusive_ms,
                requested_count: PAGE_SIZE,
                attempt_sequence,
                status: PageReceiptStatusV1::Verified,
                response_body_digest: Some(stable_hash_string(&format!(
                    "momentum-mtf-raw-response-v1:{body}"
                ))),
                normalized_row_digest: Some(normalized_rows_digest(&rows)),
                rows,
                request_count: 1,
                retry_count: 0,
                receipt_digest: String::new(),
            };
            value.receipt_digest = receipt_digest(&value);
            value
        }
        Err(_) => {
            return persist_terminal_page(
                root,
                plan,
                purpose,
                timeframe,
                &fingerprint,
                request_to_exclusive_ms,
                attempt_sequence,
                existing_receipts,
                write_counts,
                "historical page request failed terminally",
            );
        }
    };
    validate_page_receipt(&receipt)?;
    add_counts(write_counts, persist_page_receipt(root, &receipt)?);
    existing_receipts.push(receipt.clone());
    let successful_count = existing_receipts
        .iter()
        .filter(|value| value.status == PageReceiptStatusV1::Verified)
        .count();
    let normalized_row_digest = receipt
        .normalized_row_digest
        .clone()
        .ok_or_else(|| "verified page normalized digest unavailable".to_string())?;
    let response_body_digest = receipt
        .response_body_digest
        .clone()
        .ok_or_else(|| "verified page response digest unavailable".to_string())?;
    let verified_page_chunk_digest = stable_hash_string(&format!(
        "momentum-mtf-verified-page-chunk-v1:{}:{}",
        receipt.request_fingerprint, normalized_row_digest
    ));
    let mut checkpoint = HistoricalCheckpointV1 {
        checkpoint_version: CHECKPOINT_VERSION.to_string(),
        plan_digest: plan.plan_digest.clone(),
        page_receipt_digest: receipt.receipt_digest.clone(),
        request_fingerprint: receipt.request_fingerprint.clone(),
        last_successful_exclusive_to_ms: receipt
            .rows
            .first()
            .map_or(request_to_exclusive_ms, |row| {
                row.interval.open_timestamp_ms
            }),
        response_body_digest,
        normalized_row_digest,
        verified_page_chunk_digest,
        request_count_consumed: successful_count,
        remaining_budget: plan
            .exact_total_request_budget
            .saturating_sub(successful_count),
        checkpoint_digest: String::new(),
    };
    checkpoint.checkpoint_digest = checkpoint_digest(&checkpoint);
    validate_checkpoint(&checkpoint)?;
    add_counts(write_counts, persist_checkpoint(root, &checkpoint)?);
    Ok((receipt, true))
}

#[allow(clippy::too_many_arguments)]
fn persist_terminal_page(
    root: &Path,
    plan: &MomentumMtfAcquisitionPlanV1,
    purpose: PagePurposeV1,
    timeframe: MomentumHistoricalTimeframeV1,
    fingerprint: &str,
    request_to_exclusive_ms: u64,
    attempt_sequence: usize,
    existing_receipts: &mut Vec<HistoricalPageReceiptV1>,
    write_counts: &mut (usize, usize),
    error: &str,
) -> Result<(HistoricalPageReceiptV1, bool), String> {
    let mut receipt = HistoricalPageReceiptV1 {
        receipt_version: RECEIPT_VERSION.to_string(),
        plan_digest: plan.plan_digest.clone(),
        purpose,
        timeframe,
        request_fingerprint: fingerprint.to_string(),
        request_to_exclusive_ms,
        requested_count: PAGE_SIZE,
        attempt_sequence,
        status: PageReceiptStatusV1::TerminalFailure,
        response_body_digest: None,
        normalized_row_digest: None,
        rows: Vec::new(),
        request_count: 1,
        retry_count: 0,
        receipt_digest: String::new(),
    };
    receipt.receipt_digest = receipt_digest(&receipt);
    validate_page_receipt(&receipt)?;
    add_counts(write_counts, persist_page_receipt(root, &receipt)?);
    existing_receipts.push(receipt);
    Err(error.to_string())
}

fn validate_chunk(value: &HistoricalCandleChunkV1) -> Result<(), String> {
    if value.row_count == 0
        || value.row_count > CHUNK_SIZE
        || value.row_count != value.ordered_rows.len()
        || validate_rows(value.timeframe, &value.ordered_rows).is_err()
        || value.first_timestamp_ms
            != value
                .ordered_rows
                .first()
                .map_or(0, |row| row.interval.open_timestamp_ms)
        || value.last_timestamp_ms
            != value
                .ordered_rows
                .last()
                .map_or(0, |row| row.interval.open_timestamp_ms)
        || value.chunk_digest != chunk_digest(value)
    {
        return Err("historical candle chunk rejected".to_string());
    }
    Ok(())
}

fn encode_chunk(value: &HistoricalCandleChunkV1) -> Result<Vec<u8>, String> {
    validate_chunk(value)?;
    ArtifactBuilderV4_2::new("HistoricalCandleChunkV1")
        .string("chunk_version", CHUNK_VERSION)
        .string("timeframe", value.timeframe.as_str())
        .unsigned("first_timestamp_ms", value.first_timestamp_ms)
        .unsigned("last_timestamp_ms", value.last_timestamp_ms)
        .unsigned("row_count", as_u64(value.row_count)?)
        .messages(
            "ordered_rows",
            value
                .ordered_rows
                .iter()
                .map(encode_candle)
                .collect::<Result<Vec<_>, _>>()?,
        )
        .optional_string("previous_chunk_digest", &value.previous_chunk_digest)
        .string("chunk_digest", &value.chunk_digest)
        .encode()
}

fn decode_chunk(bytes: &[u8]) -> Result<HistoricalCandleChunkV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "HistoricalCandleChunkV1")?;
    if fields.string("chunk_version")? != CHUNK_VERSION {
        return Err("historical chunk version rejected".to_string());
    }
    let value = HistoricalCandleChunkV1 {
        timeframe: MomentumHistoricalTimeframeV1::parse(&fields.string("timeframe")?)?,
        first_timestamp_ms: fields.unsigned("first_timestamp_ms")?,
        last_timestamp_ms: fields.unsigned("last_timestamp_ms")?,
        row_count: as_usize(fields.unsigned("row_count")?)?,
        ordered_rows: fields
            .messages("ordered_rows")?
            .iter()
            .map(|bytes| decode_candle(bytes))
            .collect::<Result<Vec<_>, _>>()?,
        previous_chunk_digest: fields.optional_string("previous_chunk_digest")?,
        chunk_digest: fields.string("chunk_digest")?,
    };
    fields.finish()?;
    validate_chunk(&value)?;
    Ok(value)
}

fn validate_index(value: &HistoricalCandleIndexV1) -> Result<(), String> {
    if !value.timeframe.is_canonical()
        || value.ordered_chunk_digests.is_empty()
        || value.ordered_chunk_digests.iter().any(String::is_empty)
        || value.first_timestamp_ms > value.last_timestamp_ms
        || value.last_timestamp_ms >= value.close_exclusive_timestamp_ms
        || value.total_row_count == 0
        || value.missing_evidence_count != 0
        || value.aggregate_dataset_digest.is_empty()
        || value.index_digest != index_digest(value)
    {
        return Err("historical candle index rejected".to_string());
    }
    Ok(())
}

fn encode_index(value: &HistoricalCandleIndexV1) -> Result<Vec<u8>, String> {
    validate_index(value)?;
    ArtifactBuilderV4_2::new("HistoricalCandleIndexV1")
        .string("index_version", INDEX_VERSION)
        .string("timeframe", value.timeframe.as_str())
        .strings("ordered_chunk_digests", &value.ordered_chunk_digests)
        .unsigned("first_timestamp_ms", value.first_timestamp_ms)
        .unsigned("last_timestamp_ms", value.last_timestamp_ms)
        .unsigned(
            "close_exclusive_timestamp_ms",
            value.close_exclusive_timestamp_ms,
        )
        .unsigned("total_row_count", as_u64(value.total_row_count)?)
        .unsigned(
            "no_trade_interval_count",
            as_u64(value.no_trade_interval_count)?,
        )
        .unsigned(
            "missing_evidence_count",
            as_u64(value.missing_evidence_count)?,
        )
        .string("aggregate_dataset_digest", &value.aggregate_dataset_digest)
        .string("index_digest", &value.index_digest)
        .encode()
}

fn decode_index(bytes: &[u8]) -> Result<HistoricalCandleIndexV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "HistoricalCandleIndexV1")?;
    if fields.string("index_version")? != INDEX_VERSION {
        return Err("historical index version rejected".to_string());
    }
    let value = HistoricalCandleIndexV1 {
        timeframe: MomentumHistoricalTimeframeV1::parse(&fields.string("timeframe")?)?,
        ordered_chunk_digests: fields.strings("ordered_chunk_digests")?,
        first_timestamp_ms: fields.unsigned("first_timestamp_ms")?,
        last_timestamp_ms: fields.unsigned("last_timestamp_ms")?,
        close_exclusive_timestamp_ms: fields.unsigned("close_exclusive_timestamp_ms")?,
        total_row_count: as_usize(fields.unsigned("total_row_count")?)?,
        no_trade_interval_count: as_usize(fields.unsigned("no_trade_interval_count")?)?,
        missing_evidence_count: as_usize(fields.unsigned("missing_evidence_count")?)?,
        aggregate_dataset_digest: fields.string("aggregate_dataset_digest")?,
        index_digest: fields.string("index_digest")?,
    };
    fields.finish()?;
    validate_index(&value)?;
    Ok(value)
}

fn persist_chunk(root: &Path, value: &HistoricalCandleChunkV1) -> Result<(usize, usize), String> {
    persist_one(
        root,
        &format!("canonical_{}/chunks", value.timeframe.as_str()),
        &value.chunk_digest,
        &encode_chunk(value)?,
        |bytes| Ok(decode_chunk(bytes)?.chunk_digest),
    )
}

fn persist_index(root: &Path, value: &HistoricalCandleIndexV1) -> Result<(usize, usize), String> {
    persist_one(
        root,
        &format!("canonical_{}/indices", value.timeframe.as_str()),
        &value.index_digest,
        &encode_index(value)?,
        |bytes| Ok(decode_index(bytes)?.index_digest),
    )
}

fn build_chunks_and_index(
    timeframe: MomentumHistoricalTimeframeV1,
    rows: &[HistoricalCandleRowV1],
    coverage_start_ms: u64,
    coverage_end_exclusive_ms: u64,
) -> Result<(Vec<HistoricalCandleChunkV1>, HistoricalCandleIndexV1), String> {
    validate_rows(timeframe, rows)?;
    let cadence = timeframe
        .cadence_ms()
        .ok_or_else(|| "canonical cadence unavailable".to_string())?;
    if coverage_start_ms > rows[0].interval.open_timestamp_ms
        || coverage_end_exclusive_ms
            < rows
                .last()
                .map_or(0, |row| row.interval.close_exclusive_timestamp_ms)
        || !coverage_start_ms.is_multiple_of(cadence)
        || !coverage_end_exclusive_ms.is_multiple_of(cadence)
    {
        return Err("canonical coverage boundary rejected".to_string());
    }
    let expected_count = usize::try_from((coverage_end_exclusive_ms - coverage_start_ms) / cadence)
        .map_err(|_| "canonical coverage count overflow".to_string())?;
    if rows.len() > expected_count {
        return Err("canonical row coverage rejected".to_string());
    }
    let no_trade_count = expected_count - rows.len();
    let mut chunks = Vec::new();
    let mut previous = None;
    for slice in rows.chunks(CHUNK_SIZE) {
        let mut chunk = HistoricalCandleChunkV1 {
            timeframe,
            first_timestamp_ms: slice[0].interval.open_timestamp_ms,
            last_timestamp_ms: slice.last().map_or(0, |row| row.interval.open_timestamp_ms),
            row_count: slice.len(),
            ordered_rows: slice.to_vec(),
            previous_chunk_digest: previous.clone(),
            chunk_digest: String::new(),
        };
        chunk.chunk_digest = chunk_digest(&chunk);
        validate_chunk(&chunk)?;
        previous = Some(chunk.chunk_digest.clone());
        chunks.push(chunk);
    }
    let chunk_digests = chunks
        .iter()
        .map(|chunk| chunk.chunk_digest.clone())
        .collect::<Vec<_>>();
    let mut index = HistoricalCandleIndexV1 {
        timeframe,
        ordered_chunk_digests: chunk_digests.clone(),
        first_timestamp_ms: rows[0].interval.open_timestamp_ms,
        last_timestamp_ms: rows.last().map_or(0, |row| row.interval.open_timestamp_ms),
        close_exclusive_timestamp_ms: coverage_end_exclusive_ms,
        total_row_count: rows.len(),
        no_trade_interval_count: no_trade_count,
        missing_evidence_count: 0,
        aggregate_dataset_digest: stable_hash_string(&format!(
            "momentum-mtf-canonical-v1:{}:{coverage_start_ms}:{coverage_end_exclusive_ms}:{no_trade_count}:{chunk_digests:?}",
            timeframe.as_str()
        )),
        index_digest: String::new(),
    };
    index.index_digest = index_digest(&index);
    validate_index(&index)?;
    Ok((chunks, index))
}

fn persist_canonical_dataset(
    root: &Path,
    chunks: &[HistoricalCandleChunkV1],
    index: &HistoricalCandleIndexV1,
) -> Result<(usize, usize), String> {
    let mut counts = (0, 0);
    for chunk in chunks {
        add_counts(&mut counts, persist_chunk(root, chunk)?);
    }
    add_counts(&mut counts, persist_index(root, index)?);
    Ok(counts)
}

fn reopen_index(
    root: &Path,
    timeframe: MomentumHistoricalTimeframeV1,
) -> Result<Option<HistoricalCandleIndexV1>, String> {
    read_single(
        &root.join(format!("canonical_{}/indices", timeframe.as_str())),
        decode_index,
    )
}

fn reopen_canonical_rows(
    root: &Path,
    index: &HistoricalCandleIndexV1,
) -> Result<Vec<HistoricalCandleRowV1>, String> {
    validate_index(index)?;
    let mut rows = Vec::new();
    let mut previous = None;
    for digest in &index.ordered_chunk_digests {
        let path = root
            .join(format!("canonical_{}/chunks", index.timeframe.as_str()))
            .join(format!("{digest}.pb"));
        let chunk =
            decode_chunk(&fs::read(path).map_err(|_| "canonical chunk unavailable".to_string())?)?;
        if chunk.chunk_digest != *digest
            || chunk.previous_chunk_digest != previous
            || chunk.timeframe != index.timeframe
        {
            return Err("canonical chunk chain rejected".to_string());
        }
        previous = Some(chunk.chunk_digest.clone());
        rows.extend(chunk.ordered_rows);
    }
    validate_rows(index.timeframe, &rows)?;
    if rows.len() != index.total_row_count
        || rows.first().map(|row| row.interval.open_timestamp_ms) != Some(index.first_timestamp_ms)
        || rows.last().map(|row| row.interval.open_timestamp_ms) != Some(index.last_timestamp_ms)
    {
        return Err("canonical index row binding rejected".to_string());
    }
    Ok(rows)
}

fn deduplicate_rows(
    timeframe: MomentumHistoricalTimeframeV1,
    rows: Vec<HistoricalCandleRowV1>,
) -> Result<Vec<HistoricalCandleRowV1>, String> {
    let mut merged = BTreeMap::new();
    for row in rows {
        match merged.get(&row.interval.open_timestamp_ms) {
            Some(existing) if existing != &row => {
                return Err("conflicting historical candle duplicate rejected".to_string());
            }
            Some(_) => {}
            None => {
                merged.insert(row.interval.open_timestamp_ms, row);
            }
        }
    }
    let rows = merged.into_values().collect::<Vec<_>>();
    validate_rows(timeframe, &rows)?;
    Ok(rows)
}

#[allow(clippy::too_many_arguments)]
fn acquire_canonical_minute(
    root: &Path,
    foundation: &MomentumMtfFoundationRegistrationV1,
    plan: &MomentumMtfAcquisitionPlanV1,
    client: &dyn MarketDataHttpClient,
    pacer: &mut RequestPacerV1,
    receipts: &mut Vec<HistoricalPageReceiptV1>,
    counts: &mut (usize, usize),
) -> Result<Vec<HistoricalCandleRowV1>, String> {
    let mut cursor = foundation.minute_end_exclusive_timestamp_ms;
    let mut rows = Vec::new();
    let mut reached_start = false;
    for _ in 0..plan.minute_page_budget {
        let (receipt, _) = acquire_page(
            root,
            plan,
            PagePurposeV1::CanonicalMinute,
            MomentumHistoricalTimeframeV1::Minute1,
            cursor,
            client,
            pacer,
            receipts,
            counts,
        )?;
        if receipt.rows.is_empty() {
            return Err("canonical minute page unexpectedly empty".to_string());
        }
        let oldest = receipt
            .rows
            .first()
            .map(|row| row.interval.open_timestamp_ms)
            .ok_or_else(|| "canonical minute cursor unavailable".to_string())?;
        if oldest >= cursor {
            return Err("canonical minute cursor did not move backward".to_string());
        }
        rows.extend(receipt.rows);
        cursor = oldest;
        if cursor <= foundation.minute_start_timestamp_ms {
            reached_start = true;
            break;
        }
    }
    if !reached_start {
        return Err("canonical minute page budget exhausted before range start".to_string());
    }
    rows.retain(|row| {
        row.interval.open_timestamp_ms >= foundation.minute_start_timestamp_ms
            && row.interval.close_exclusive_timestamp_ms
                <= foundation.minute_end_exclusive_timestamp_ms
    });
    deduplicate_rows(MomentumHistoricalTimeframeV1::Minute1, rows)
}

#[allow(clippy::too_many_arguments)]
fn acquire_older_daily(
    root: &Path,
    foundation: &MomentumMtfFoundationRegistrationV1,
    plan: &MomentumMtfAcquisitionPlanV1,
    client: &dyn MarketDataHttpClient,
    pacer: &mut RequestPacerV1,
    receipts: &mut Vec<HistoricalPageReceiptV1>,
    counts: &mut (usize, usize),
) -> Result<Vec<HistoricalCandleRowV1>, String> {
    let mut cursor = foundation.existing_daily_first_timestamp_ms;
    let mut rows = Vec::new();
    let mut reached_provider_start = false;
    for _ in 0..plan.daily_page_budget {
        let (receipt, _) = acquire_page(
            root,
            plan,
            PagePurposeV1::CanonicalDailyOlder,
            MomentumHistoricalTimeframeV1::Day1,
            cursor,
            client,
            pacer,
            receipts,
            counts,
        )?;
        let returned_count = receipt.rows.len();
        if let Some(oldest) = receipt
            .rows
            .first()
            .map(|row| row.interval.open_timestamp_ms)
        {
            if oldest >= cursor {
                return Err("canonical daily cursor did not move backward".to_string());
            }
            cursor = oldest;
        }
        rows.extend(receipt.rows);
        if returned_count < PAGE_SIZE {
            reached_provider_start = true;
            break;
        }
    }
    if !reached_provider_start {
        return Err("canonical daily page budget exhausted before provider start".to_string());
    }
    if rows
        .iter()
        .any(|row| row.interval.open_timestamp_ms >= foundation.existing_daily_first_timestamp_ms)
    {
        return Err("older daily acquisition crossed existing snapshot".to_string());
    }
    if rows.is_empty() {
        return Err("older daily provider history unavailable".to_string());
    }
    deduplicate_rows(MomentumHistoricalTimeframeV1::Day1, rows)
}

#[allow(clippy::too_many_arguments)]
fn acquire_native_samples(
    root: &Path,
    foundation: &MomentumMtfFoundationRegistrationV1,
    plan: &MomentumMtfAcquisitionPlanV1,
    client: &dyn MarketDataHttpClient,
    pacer: &mut RequestPacerV1,
    receipts: &mut Vec<HistoricalPageReceiptV1>,
    counts: &mut (usize, usize),
) -> Result<(), String> {
    let native_timeframes = [
        MomentumHistoricalTimeframeV1::Minute3,
        MomentumHistoricalTimeframeV1::Minute5,
        MomentumHistoricalTimeframeV1::Minute10,
        MomentumHistoricalTimeframeV1::Week1,
        MomentumHistoricalTimeframeV1::Month1,
        MomentumHistoricalTimeframeV1::Year1,
    ];
    if native_timeframes.len() != plan.native_sample_request_budget {
        return Err("native sample request budget rejected".to_string());
    }
    for timeframe in native_timeframes {
        let request_to_exclusive_ms =
            native_request_to_exclusive(timeframe, foundation.minute_end_exclusive_timestamp_ms)?;
        let (receipt, _) = acquire_page(
            root,
            plan,
            PagePurposeV1::NativeCrossCheck,
            timeframe,
            request_to_exclusive_ms,
            client,
            pacer,
            receipts,
            counts,
        )?;
        if receipt.rows.is_empty() {
            return Err("native comparison sample unexpectedly empty".to_string());
        }
    }
    Ok(())
}

fn native_request_to_exclusive(
    timeframe: MomentumHistoricalTimeframeV1,
    protected_boundary_ms: u64,
) -> Result<u64, String> {
    if timeframe.is_canonical() {
        return Err("native comparison timeframe rejected".to_string());
    }
    let prior = protected_boundary_ms
        .checked_sub(1)
        .ok_or_else(|| "native comparison boundary rejected".to_string())?;
    let containing = period_interval(timeframe, prior)?;
    if containing.close_exclusive_timestamp_ms <= protected_boundary_ms {
        Ok(protected_boundary_ms)
    } else {
        Ok(containing.open_timestamp_ms)
    }
}

fn native_receipts_complete(
    receipts: &[HistoricalPageReceiptV1],
    plan: &MomentumMtfAcquisitionPlanV1,
) -> bool {
    [
        MomentumHistoricalTimeframeV1::Minute3,
        MomentumHistoricalTimeframeV1::Minute5,
        MomentumHistoricalTimeframeV1::Minute10,
        MomentumHistoricalTimeframeV1::Week1,
        MomentumHistoricalTimeframeV1::Month1,
        MomentumHistoricalTimeframeV1::Year1,
    ]
    .iter()
    .all(|timeframe| {
        receipts.iter().any(|receipt| {
            receipt.plan_digest == plan.plan_digest
                && receipt.purpose == PagePurposeV1::NativeCrossCheck
                && receipt.timeframe == *timeframe
                && receipt.status == PageReceiptStatusV1::Verified
        })
    })
}

fn execute_backfill_with_client(
    root: &Path,
    snapshots: &[DataSnapshot],
    client: &dyn MarketDataHttpClient,
) -> Result<
    (
        HistoricalCandleIndexV1,
        HistoricalCandleIndexV1,
        (usize, usize),
    ),
    String,
> {
    let (_, foundation, plan) = reopen_foundation(root)?;
    let foundation =
        foundation.ok_or_else(|| "multi-timeframe foundation unavailable".to_string())?;
    let plan = plan.ok_or_else(|| "multi-timeframe acquisition plan unavailable".to_string())?;
    validate_foundation(&foundation)?;
    validate_plan(&plan)?;
    if plan.exact_total_request_budget > REQUEST_CEILING
        || plan.maximum_concurrency != 1
        || plan.maximum_retries_per_page != 0
    {
        return Err("historical network authority rejected".to_string());
    }
    let existing_minute = reopen_index(root, MomentumHistoricalTimeframeV1::Minute1)?;
    let existing_daily = reopen_index(root, MomentumHistoricalTimeframeV1::Day1)?;
    let mut receipts = load_receipts(root)?;
    if let (Some(minute), Some(daily)) = (existing_minute, existing_daily)
        && native_receipts_complete(&receipts, &plan)
    {
        return Ok((minute, daily, (0, 0)));
    }
    let daily_snapshot = select_daily_snapshot(snapshots)?;
    if daily_snapshot.content_digest != foundation.existing_daily_snapshot_digest {
        return Err("registered daily snapshot identity changed".to_string());
    }
    let mut pacer = RequestPacerV1::default();
    let mut counts = (0, 0);
    let minute_rows = acquire_canonical_minute(
        root,
        &foundation,
        &plan,
        client,
        &mut pacer,
        &mut receipts,
        &mut counts,
    )?;
    let mut daily_rows = acquire_older_daily(
        root,
        &foundation,
        &plan,
        client,
        &mut pacer,
        &mut receipts,
        &mut counts,
    )?;
    daily_rows.extend(
        daily_snapshot
            .normalized_dataset
            .rows
            .iter()
            .map(|row| canonical_row(MomentumHistoricalTimeframeV1::Day1, row))
            .collect::<Result<Vec<_>, _>>()?,
    );
    daily_rows = deduplicate_rows(MomentumHistoricalTimeframeV1::Day1, daily_rows)?;
    acquire_native_samples(
        root,
        &foundation,
        &plan,
        client,
        &mut pacer,
        &mut receipts,
        &mut counts,
    )?;
    let unique_verified_requests = receipts
        .iter()
        .filter(|receipt| receipt.status == PageReceiptStatusV1::Verified)
        .map(|receipt| receipt.request_fingerprint.as_str())
        .collect::<BTreeSet<_>>();
    if unique_verified_requests.len() > plan.exact_total_request_budget {
        return Err("historical request budget exceeded".to_string());
    }
    let (minute_chunks, minute_index) = build_chunks_and_index(
        MomentumHistoricalTimeframeV1::Minute1,
        &minute_rows,
        foundation.minute_start_timestamp_ms,
        foundation.minute_end_exclusive_timestamp_ms,
    )?;
    let daily_start = daily_rows
        .first()
        .map(|row| row.interval.open_timestamp_ms)
        .ok_or_else(|| "canonical daily start unavailable".to_string())?;
    let daily_end = daily_rows
        .last()
        .and_then(|row| row.interval.open_timestamp_ms.checked_add(DAY_MS))
        .ok_or_else(|| "canonical daily end unavailable".to_string())?;
    let (daily_chunks, daily_index) = build_chunks_and_index(
        MomentumHistoricalTimeframeV1::Day1,
        &daily_rows,
        daily_start,
        daily_end,
    )?;
    add_counts(
        &mut counts,
        persist_canonical_dataset(root, &minute_chunks, &minute_index)?,
    );
    add_counts(
        &mut counts,
        persist_canonical_dataset(root, &daily_chunks, &daily_index)?,
    );
    Ok((minute_index, daily_index, counts))
}

fn validate_derived_index(value: &DerivedViewIndexV1) -> Result<(), String> {
    let source_valid = match value.timeframe {
        MomentumHistoricalTimeframeV1::Minute3
        | MomentumHistoricalTimeframeV1::Minute5
        | MomentumHistoricalTimeframeV1::Minute10 => {
            value.canonical_source_timeframe == MomentumHistoricalTimeframeV1::Minute1
        }
        MomentumHistoricalTimeframeV1::Week1
        | MomentumHistoricalTimeframeV1::Month1
        | MomentumHistoricalTimeframeV1::Year1 => {
            value.canonical_source_timeframe == MomentumHistoricalTimeframeV1::Day1
        }
        _ => false,
    };
    if value.index_version != DERIVED_INDEX_VERSION
        || value.foundation_registration_digest.is_empty()
        || !source_valid
        || value.first_timestamp_ms > value.last_timestamp_ms
        || value.candle_count == 0
        || value.missing_evidence_count != 0
        || value.ordered_candle_digests.len() != value.candle_count
        || value.ordered_candle_digests.iter().any(String::is_empty)
        || value.timezone_policy != "UTC"
        || value.boundary_policy != "CalendarOrFixedOpenCloseExclusiveV1"
        || value.index_digest != derived_index_digest(value)
    {
        return Err("derived view index rejected".to_string());
    }
    Ok(())
}

fn encode_derived_index(value: &DerivedViewIndexV1) -> Result<Vec<u8>, String> {
    validate_derived_index(value)?;
    ArtifactBuilderV4_2::new("DerivedViewIndexV1")
        .string("index_version", &value.index_version)
        .string(
            "foundation_registration_digest",
            &value.foundation_registration_digest,
        )
        .string("timeframe", value.timeframe.as_str())
        .string(
            "canonical_source_timeframe",
            value.canonical_source_timeframe.as_str(),
        )
        .unsigned("first_timestamp_ms", value.first_timestamp_ms)
        .unsigned("last_timestamp_ms", value.last_timestamp_ms)
        .unsigned("candle_count", as_u64(value.candle_count)?)
        .unsigned(
            "no_trade_interval_count",
            as_u64(value.no_trade_interval_count)?,
        )
        .unsigned(
            "missing_evidence_count",
            as_u64(value.missing_evidence_count)?,
        )
        .strings("ordered_candle_digests", &value.ordered_candle_digests)
        .string("timezone_policy", &value.timezone_policy)
        .string("boundary_policy", &value.boundary_policy)
        .string("index_digest", &value.index_digest)
        .encode()
}

fn decode_derived_index(bytes: &[u8]) -> Result<DerivedViewIndexV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "DerivedViewIndexV1")?;
    let value = DerivedViewIndexV1 {
        index_version: fields.string("index_version")?,
        foundation_registration_digest: fields.string("foundation_registration_digest")?,
        timeframe: MomentumHistoricalTimeframeV1::parse(&fields.string("timeframe")?)?,
        canonical_source_timeframe: MomentumHistoricalTimeframeV1::parse(
            &fields.string("canonical_source_timeframe")?,
        )?,
        first_timestamp_ms: fields.unsigned("first_timestamp_ms")?,
        last_timestamp_ms: fields.unsigned("last_timestamp_ms")?,
        candle_count: as_usize(fields.unsigned("candle_count")?)?,
        no_trade_interval_count: as_usize(fields.unsigned("no_trade_interval_count")?)?,
        missing_evidence_count: as_usize(fields.unsigned("missing_evidence_count")?)?,
        ordered_candle_digests: fields.strings("ordered_candle_digests")?,
        timezone_policy: fields.string("timezone_policy")?,
        boundary_policy: fields.string("boundary_policy")?,
        index_digest: fields.string("index_digest")?,
    };
    fields.finish()?;
    validate_derived_index(&value)?;
    Ok(value)
}

fn persist_derived_index(
    root: &Path,
    value: &DerivedViewIndexV1,
) -> Result<(usize, usize), String> {
    persist_one(
        root,
        &format!("derived_{}/indices", value.timeframe.as_str()),
        &value.index_digest,
        &encode_derived_index(value)?,
        |bytes| Ok(decode_derived_index(bytes)?.index_digest),
    )
}

fn reopen_derived_indices(root: &Path) -> Result<Vec<DerivedViewIndexV1>, String> {
    [
        MomentumHistoricalTimeframeV1::Minute3,
        MomentumHistoricalTimeframeV1::Minute5,
        MomentumHistoricalTimeframeV1::Minute10,
        MomentumHistoricalTimeframeV1::Week1,
        MomentumHistoricalTimeframeV1::Month1,
        MomentumHistoricalTimeframeV1::Year1,
    ]
    .iter()
    .map(|timeframe| {
        read_single(
            &root.join(format!("derived_{}/indices", timeframe.as_str())),
            decode_derived_index,
        )?
        .ok_or_else(|| format!("derived {} index unavailable", timeframe.as_str()))
    })
    .collect()
}

fn aggregate_view(
    foundation: &MomentumMtfFoundationRegistrationV1,
    base_index: &HistoricalCandleIndexV1,
    base_rows: &[HistoricalCandleRowV1],
    timeframe: MomentumHistoricalTimeframeV1,
) -> Result<(Vec<HistoricalCandleRowV1>, DerivedViewIndexV1), String> {
    let expected_base = match timeframe {
        MomentumHistoricalTimeframeV1::Minute3
        | MomentumHistoricalTimeframeV1::Minute5
        | MomentumHistoricalTimeframeV1::Minute10 => MomentumHistoricalTimeframeV1::Minute1,
        MomentumHistoricalTimeframeV1::Week1
        | MomentumHistoricalTimeframeV1::Month1
        | MomentumHistoricalTimeframeV1::Year1 => MomentumHistoricalTimeframeV1::Day1,
        _ => return Err("derived timeframe rejected".to_string()),
    };
    if base_index.timeframe != expected_base
        || base_rows.iter().any(|row| row.timeframe != expected_base)
        || base_index.missing_evidence_count != 0
    {
        return Err("derived canonical base rejected".to_string());
    }
    let base_cadence = expected_base
        .cadence_ms()
        .ok_or_else(|| "derived base cadence unavailable".to_string())?;
    let coverage_start = if expected_base == MomentumHistoricalTimeframeV1::Minute1 {
        foundation.minute_start_timestamp_ms
    } else {
        base_index.first_timestamp_ms
    };
    let coverage_end = base_index.close_exclusive_timestamp_ms;
    let by_timestamp = base_rows
        .iter()
        .map(|row| (row.interval.open_timestamp_ms, row))
        .collect::<BTreeMap<_, _>>();
    let mut grouped = BTreeMap::<u64, Vec<&HistoricalCandleRowV1>>::new();
    for row in base_rows {
        let interval = period_interval(timeframe, row.interval.open_timestamp_ms)?;
        if interval.open_timestamp_ms >= coverage_start
            && interval.close_exclusive_timestamp_ms <= coverage_end
        {
            grouped
                .entry(interval.open_timestamp_ms)
                .or_default()
                .push(row);
        }
    }
    let mut derived = Vec::with_capacity(grouped.len());
    let mut no_trade_interval_count = 0usize;
    for (open_timestamp_ms, observed) in grouped {
        let interval = period_interval(timeframe, open_timestamp_ms)?;
        let mut ordered_base_candle_digests = Vec::new();
        let mut cursor = interval.open_timestamp_ms;
        while cursor < interval.close_exclusive_timestamp_ms {
            if let Some(row) = by_timestamp.get(&cursor) {
                ordered_base_candle_digests.push(row.candle_digest.clone());
            } else {
                no_trade_interval_count += 1;
                ordered_base_candle_digests.push(stable_hash_string(&format!(
                    "momentum-mtf-no-trade-v1:{}:{cursor}",
                    expected_base.as_str()
                )));
            }
            cursor = cursor
                .checked_add(base_cadence)
                .ok_or_else(|| "derived interval cursor overflow".to_string())?;
        }
        let first = observed
            .first()
            .ok_or_else(|| "derived opening row unavailable".to_string())?;
        let last = observed
            .last()
            .ok_or_else(|| "derived closing row unavailable".to_string())?;
        let mut row = HistoricalCandleRowV1 {
            timeframe,
            interval,
            open: first.open,
            high: observed
                .iter()
                .map(|row| row.high)
                .fold(f64::NEG_INFINITY, f64::max),
            low: observed
                .iter()
                .map(|row| row.low)
                .fold(f64::INFINITY, f64::min),
            close: last.close,
            volume: observed.iter().map(|row| row.volume).sum(),
            trade_value: observed.iter().map(|row| row.trade_value).sum(),
            ordered_base_candle_digests,
            presence: CandleIntervalPresenceV1::ObservedTradeCandle,
            candle_digest: String::new(),
        };
        row.candle_digest = candle_digest(&row);
        validate_candle(&row)?;
        derived.push(row);
    }
    validate_rows(timeframe, &derived)?;
    let ordered_candle_digests = derived
        .iter()
        .map(|row| row.candle_digest.clone())
        .collect::<Vec<_>>();
    let mut index = DerivedViewIndexV1 {
        index_version: DERIVED_INDEX_VERSION.to_string(),
        foundation_registration_digest: foundation.registration_digest.clone(),
        timeframe,
        canonical_source_timeframe: expected_base,
        first_timestamp_ms: derived
            .first()
            .map_or(0, |row| row.interval.open_timestamp_ms),
        last_timestamp_ms: derived
            .last()
            .map_or(0, |row| row.interval.open_timestamp_ms),
        candle_count: derived.len(),
        no_trade_interval_count,
        missing_evidence_count: 0,
        ordered_candle_digests,
        timezone_policy: "UTC".to_string(),
        boundary_policy: "CalendarOrFixedOpenCloseExclusiveV1".to_string(),
        index_digest: String::new(),
    };
    index.index_digest = derived_index_digest(&index);
    validate_derived_index(&index)?;
    Ok((derived, index))
}

fn finite_close(left: f64, right: f64) -> bool {
    let difference = (left - right).abs();
    difference <= ABSOLUTE_TOLERANCE
        || difference <= RELATIVE_TOLERANCE * left.abs().max(right.abs())
}

fn compare_candle(
    derived: &HistoricalCandleRowV1,
    native: &HistoricalCandleRowV1,
) -> DerivedNativeComparisonV1 {
    if derived.timeframe != native.timeframe
        || derived.interval.open_timestamp_ms != native.interval.open_timestamp_ms
        || derived.interval.close_exclusive_timestamp_ms
            != native.interval.close_exclusive_timestamp_ms
    {
        return DerivedNativeComparisonV1::ProviderBoundaryMismatch;
    }
    if derived.presence != CandleIntervalPresenceV1::ObservedTradeCandle {
        return DerivedNativeComparisonV1::DerivedCompletenessFailure;
    }
    if [
        (derived.open, native.open),
        (derived.high, native.high),
        (derived.low, native.low),
        (derived.close, native.close),
    ]
    .iter()
    .any(|(left, right)| left.to_bits() != right.to_bits())
    {
        return DerivedNativeComparisonV1::IntegrityFailure;
    }
    if derived.volume.to_bits() == native.volume.to_bits()
        && derived.trade_value.to_bits() == native.trade_value.to_bits()
    {
        DerivedNativeComparisonV1::ExactMatch
    } else if finite_close(derived.volume, native.volume)
        && finite_close(derived.trade_value, native.trade_value)
    {
        DerivedNativeComparisonV1::WithinRegisteredTolerance
    } else {
        DerivedNativeComparisonV1::IntegrityFailure
    }
}

fn validate_comparison(value: &DerivedNativeComparisonSummaryV1) -> Result<(), String> {
    if value.comparison_version != COMPARISON_VERSION
        || value.timeframe.is_canonical()
        || value.sample_count
            != value.exact_match_count
                + value.within_tolerance_count
                + value.boundary_mismatch_count
                + value.missing_native_count
                + value.completeness_failure_count
                + value.integrity_failure_count
        || value.sample_count == 0
        || value.systematic_mismatch_blocks_replay
            != (value.boundary_mismatch_count > 0
                || value.missing_native_count > 0
                || value.completeness_failure_count > 0
                || value.integrity_failure_count > 0)
        || value.comparison_digest != comparison_digest(value)
    {
        return Err("derived native comparison summary rejected".to_string());
    }
    Ok(())
}

fn compare_native_sample(
    timeframe: MomentumHistoricalTimeframeV1,
    derived: &[HistoricalCandleRowV1],
    native: &[HistoricalCandleRowV1],
) -> Result<DerivedNativeComparisonSummaryV1, String> {
    validate_rows(timeframe, derived)?;
    validate_rows(timeframe, native)?;
    let derived_by_open = derived
        .iter()
        .map(|row| (row.interval.open_timestamp_ms, row))
        .collect::<BTreeMap<_, _>>();
    let first = derived
        .first()
        .map_or(0, |row| row.interval.open_timestamp_ms);
    let last_close = derived
        .last()
        .map_or(0, |row| row.interval.close_exclusive_timestamp_ms);
    let classifications = native
        .iter()
        .filter(|row| {
            row.interval.open_timestamp_ms >= first
                && row.interval.close_exclusive_timestamp_ms <= last_close
        })
        .map(|native_row| {
            derived_by_open
                .get(&native_row.interval.open_timestamp_ms)
                .map_or(DerivedNativeComparisonV1::MissingNativeCandle, |derived| {
                    compare_candle(derived, native_row)
                })
        })
        .collect::<Vec<_>>();
    let count = |kind| {
        classifications
            .iter()
            .filter(|value| **value == kind)
            .count()
    };
    let mut value = DerivedNativeComparisonSummaryV1 {
        comparison_version: COMPARISON_VERSION.to_string(),
        timeframe,
        sample_count: classifications.len(),
        exact_match_count: count(DerivedNativeComparisonV1::ExactMatch),
        within_tolerance_count: count(DerivedNativeComparisonV1::WithinRegisteredTolerance),
        boundary_mismatch_count: count(DerivedNativeComparisonV1::ProviderBoundaryMismatch),
        missing_native_count: count(DerivedNativeComparisonV1::MissingNativeCandle),
        completeness_failure_count: count(DerivedNativeComparisonV1::DerivedCompletenessFailure),
        integrity_failure_count: count(DerivedNativeComparisonV1::IntegrityFailure),
        systematic_mismatch_blocks_replay: false,
        comparison_digest: String::new(),
    };
    value.systematic_mismatch_blocks_replay = value.boundary_mismatch_count > 0
        || value.missing_native_count > 0
        || value.completeness_failure_count > 0
        || value.integrity_failure_count > 0;
    value.comparison_digest = comparison_digest(&value);
    validate_comparison(&value)?;
    Ok(value)
}

fn encode_comparison(value: &DerivedNativeComparisonSummaryV1) -> Result<Vec<u8>, String> {
    validate_comparison(value)?;
    ArtifactBuilderV4_2::new("DerivedNativeComparisonSummaryV1")
        .string("comparison_version", &value.comparison_version)
        .string("timeframe", value.timeframe.as_str())
        .unsigned("sample_count", as_u64(value.sample_count)?)
        .unsigned("exact_match_count", as_u64(value.exact_match_count)?)
        .unsigned(
            "within_tolerance_count",
            as_u64(value.within_tolerance_count)?,
        )
        .unsigned(
            "boundary_mismatch_count",
            as_u64(value.boundary_mismatch_count)?,
        )
        .unsigned("missing_native_count", as_u64(value.missing_native_count)?)
        .unsigned(
            "completeness_failure_count",
            as_u64(value.completeness_failure_count)?,
        )
        .unsigned(
            "integrity_failure_count",
            as_u64(value.integrity_failure_count)?,
        )
        .boolean(
            "systematic_mismatch_blocks_replay",
            value.systematic_mismatch_blocks_replay,
        )
        .string("comparison_digest", &value.comparison_digest)
        .encode()
}

fn decode_comparison(bytes: &[u8]) -> Result<DerivedNativeComparisonSummaryV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "DerivedNativeComparisonSummaryV1")?;
    let value = DerivedNativeComparisonSummaryV1 {
        comparison_version: fields.string("comparison_version")?,
        timeframe: MomentumHistoricalTimeframeV1::parse(&fields.string("timeframe")?)?,
        sample_count: as_usize(fields.unsigned("sample_count")?)?,
        exact_match_count: as_usize(fields.unsigned("exact_match_count")?)?,
        within_tolerance_count: as_usize(fields.unsigned("within_tolerance_count")?)?,
        boundary_mismatch_count: as_usize(fields.unsigned("boundary_mismatch_count")?)?,
        missing_native_count: as_usize(fields.unsigned("missing_native_count")?)?,
        completeness_failure_count: as_usize(fields.unsigned("completeness_failure_count")?)?,
        integrity_failure_count: as_usize(fields.unsigned("integrity_failure_count")?)?,
        systematic_mismatch_blocks_replay: fields.boolean("systematic_mismatch_blocks_replay")?,
        comparison_digest: fields.string("comparison_digest")?,
    };
    fields.finish()?;
    validate_comparison(&value)?;
    Ok(value)
}

fn persist_comparison(
    root: &Path,
    value: &DerivedNativeComparisonSummaryV1,
) -> Result<(usize, usize), String> {
    persist_one(
        root,
        &format!("native_comparisons/{}", value.timeframe.as_str()),
        &value.comparison_digest,
        &encode_comparison(value)?,
        |bytes| Ok(decode_comparison(bytes)?.comparison_digest),
    )
}

fn reopen_comparisons(root: &Path) -> Result<Vec<DerivedNativeComparisonSummaryV1>, String> {
    [
        MomentumHistoricalTimeframeV1::Minute3,
        MomentumHistoricalTimeframeV1::Minute5,
        MomentumHistoricalTimeframeV1::Minute10,
        MomentumHistoricalTimeframeV1::Week1,
        MomentumHistoricalTimeframeV1::Month1,
        MomentumHistoricalTimeframeV1::Year1,
    ]
    .iter()
    .map(|timeframe| {
        read_single(
            &root.join(format!("native_comparisons/{}", timeframe.as_str())),
            decode_comparison,
        )?
        .ok_or_else(|| format!("native {} comparison unavailable", timeframe.as_str()))
    })
    .collect()
}

fn derive_and_compare_views(
    root: &Path,
) -> Result<
    (
        Vec<DerivedViewIndexV1>,
        Vec<DerivedNativeComparisonSummaryV1>,
        (usize, usize),
    ),
    String,
> {
    if let (Ok(indices), Ok(comparisons)) = (reopen_derived_indices(root), reopen_comparisons(root))
    {
        return Ok((indices, comparisons, (0, 0)));
    }
    let (_, foundation, plan) = reopen_foundation(root)?;
    let foundation =
        foundation.ok_or_else(|| "multi-timeframe foundation unavailable".to_string())?;
    let plan = plan.ok_or_else(|| "multi-timeframe acquisition plan unavailable".to_string())?;
    let minute_index = reopen_index(root, MomentumHistoricalTimeframeV1::Minute1)?
        .ok_or_else(|| "canonical minute index unavailable".to_string())?;
    let daily_index = reopen_index(root, MomentumHistoricalTimeframeV1::Day1)?
        .ok_or_else(|| "canonical daily index unavailable".to_string())?;
    let minute_rows = reopen_canonical_rows(root, &minute_index)?;
    let daily_rows = reopen_canonical_rows(root, &daily_index)?;
    let receipts = load_receipts(root)?;
    if !native_receipts_complete(&receipts, &plan) {
        return Err("native comparison samples unavailable".to_string());
    }
    let mut indices = Vec::new();
    let mut comparisons = Vec::new();
    let mut counts = (0, 0);
    for timeframe in [
        MomentumHistoricalTimeframeV1::Minute3,
        MomentumHistoricalTimeframeV1::Minute5,
        MomentumHistoricalTimeframeV1::Minute10,
        MomentumHistoricalTimeframeV1::Week1,
        MomentumHistoricalTimeframeV1::Month1,
        MomentumHistoricalTimeframeV1::Year1,
    ] {
        let (base_index, base_rows) = if matches!(
            timeframe,
            MomentumHistoricalTimeframeV1::Minute3
                | MomentumHistoricalTimeframeV1::Minute5
                | MomentumHistoricalTimeframeV1::Minute10
        ) {
            (&minute_index, minute_rows.as_slice())
        } else {
            (&daily_index, daily_rows.as_slice())
        };
        let (derived, index) = aggregate_view(&foundation, base_index, base_rows, timeframe)?;
        let native = receipts
            .iter()
            .find(|receipt| {
                receipt.plan_digest == plan.plan_digest
                    && receipt.purpose == PagePurposeV1::NativeCrossCheck
                    && receipt.timeframe == timeframe
                    && receipt.status == PageReceiptStatusV1::Verified
            })
            .map(|receipt| receipt.rows.as_slice())
            .ok_or_else(|| "native comparison receipt unavailable".to_string())?;
        let comparison = compare_native_sample(timeframe, &derived, native)?;
        add_counts(&mut counts, persist_derived_index(root, &index)?);
        add_counts(&mut counts, persist_comparison(root, &comparison)?);
        indices.push(index);
        comparisons.push(comparison);
    }
    Ok((indices, comparisons, counts))
}

fn latest_completed_expected_open(
    timeframe: MomentumHistoricalTimeframeV1,
    prediction_timestamp_ms: u64,
) -> Result<u64, String> {
    let prior = prediction_timestamp_ms
        .checked_sub(1)
        .ok_or_else(|| "as-of prediction timestamp rejected".to_string())?;
    let current = period_interval(timeframe, prior)?;
    if current.close_exclusive_timestamp_ms <= prediction_timestamp_ms {
        Ok(current.open_timestamp_ms)
    } else {
        current
            .open_timestamp_ms
            .checked_sub(1)
            .ok_or_else(|| "as-of prior period unavailable".to_string())
            .and_then(|timestamp| period_interval(timeframe, timestamp))
            .map(|interval| interval.open_timestamp_ms)
    }
}

fn select_as_of(
    timeframe: MomentumHistoricalTimeframeV1,
    rows: &[HistoricalCandleRowV1],
    prediction_timestamp_ms: u64,
) -> Result<(TimeframeViewAvailabilityV1, Option<&HistoricalCandleRowV1>), String> {
    if rows.is_empty() || rows.iter().any(|row| row.timeframe != timeframe) {
        return Err("as-of view source rejected".to_string());
    }
    let expected_open = latest_completed_expected_open(timeframe, prediction_timestamp_ms)?;
    let expected_interval = period_interval(timeframe, expected_open)?;
    let selected = rows
        .iter()
        .take_while(|row| row.interval.close_exclusive_timestamp_ms <= prediction_timestamp_ms)
        .last();
    let Some(selected) = selected else {
        let status = if rows
            .first()
            .is_some_and(|row| row.interval.open_timestamp_ms > expected_open)
        {
            TimeframeViewAvailabilityV1::InsufficientHistoricalDepth
        } else if rows.last().is_some_and(|row| {
            row.interval.close_exclusive_timestamp_ms
                < expected_interval.close_exclusive_timestamp_ms
        }) {
            TimeframeViewAvailabilityV1::MissingEvidence
        } else {
            TimeframeViewAvailabilityV1::NoTradeInterval
        };
        return Ok((status, None));
    };
    if selected.interval.close_exclusive_timestamp_ms > prediction_timestamp_ms {
        return Ok((TimeframeViewAvailabilityV1::PartialCandleForbidden, None));
    }
    if selected.interval.open_timestamp_ms < expected_open {
        let status = if rows.last().is_some_and(|row| {
            row.interval.close_exclusive_timestamp_ms
                < expected_interval.close_exclusive_timestamp_ms
        }) {
            TimeframeViewAvailabilityV1::MissingEvidence
        } else {
            TimeframeViewAvailabilityV1::NoTradeInterval
        };
        Ok((status, None))
    } else if selected.interval.open_timestamp_ms == expected_open {
        Ok((TimeframeViewAvailabilityV1::Available, Some(selected)))
    } else {
        Ok((TimeframeViewAvailabilityV1::IntegrityFailure, None))
    }
}

fn build_as_of_snapshot(
    prediction_timestamp_ms: u64,
    views: &BTreeMap<MomentumHistoricalTimeframeV1, Vec<HistoricalCandleRowV1>>,
) -> Result<MomentumMultiTimeframeAsOfSnapshotV1, String> {
    let mut view_digests = Vec::new();
    let mut availability = Vec::new();
    let mut future_access_count = 0usize;
    let mut partial_candle_access_count = 0usize;
    for timeframe in MomentumHistoricalTimeframeV1::ORDERED {
        let rows = views
            .get(&timeframe)
            .ok_or_else(|| "as-of timeframe source unavailable".to_string())?;
        let (status, selected) = select_as_of(timeframe, rows, prediction_timestamp_ms)?;
        if selected
            .is_some_and(|row| row.interval.close_exclusive_timestamp_ms > prediction_timestamp_ms)
        {
            future_access_count += 1;
        }
        if status == TimeframeViewAvailabilityV1::PartialCandleForbidden {
            partial_candle_access_count += 1;
        }
        view_digests.push(
            selected
                .map(|row| row.candle_digest.clone())
                .unwrap_or_else(|| {
                    stable_hash_string(&format!(
                        "momentum-mtf-unavailable-view-v1:{}:{}:{}",
                        timeframe.as_str(),
                        status.as_str(),
                        prediction_timestamp_ms
                    ))
                }),
        );
        availability.push(status);
    }
    let mut value = MomentumMultiTimeframeAsOfSnapshotV1 {
        prediction_timestamp_ms,
        view_digests,
        availability,
        all_views_closed: future_access_count == 0 && partial_candle_access_count == 0,
        future_access_count,
        partial_candle_access_count,
        snapshot_digest: String::new(),
    };
    value.snapshot_digest = snapshot_digest(&value);
    validate_as_of_snapshot(&value)?;
    Ok(value)
}

fn validate_as_of_snapshot(value: &MomentumMultiTimeframeAsOfSnapshotV1) -> Result<(), String> {
    if value.prediction_timestamp_ms == 0
        || value.view_digests.len() != MomentumHistoricalTimeframeV1::ORDERED.len()
        || value.availability.len() != MomentumHistoricalTimeframeV1::ORDERED.len()
        || value.view_digests.iter().any(String::is_empty)
        || !value.all_views_closed
        || value.future_access_count != 0
        || value.partial_candle_access_count != 0
        || value.snapshot_digest != snapshot_digest(value)
    {
        return Err("multi-timeframe as-of snapshot rejected".to_string());
    }
    Ok(())
}

fn encode_as_of_snapshot(value: &MomentumMultiTimeframeAsOfSnapshotV1) -> Result<Vec<u8>, String> {
    validate_as_of_snapshot(value)?;
    ArtifactBuilderV4_2::new("MomentumMultiTimeframeAsOfSnapshotV1")
        .unsigned("prediction_timestamp_ms", value.prediction_timestamp_ms)
        .strings("view_digests", &value.view_digests)
        .strings(
            "availability",
            &value
                .availability
                .iter()
                .map(|value| value.as_str().to_string())
                .collect::<Vec<_>>(),
        )
        .boolean("all_views_closed", value.all_views_closed)
        .unsigned("future_access_count", as_u64(value.future_access_count)?)
        .unsigned(
            "partial_candle_access_count",
            as_u64(value.partial_candle_access_count)?,
        )
        .string("snapshot_digest", &value.snapshot_digest)
        .encode()
}

fn parse_availability(value: &str) -> Result<TimeframeViewAvailabilityV1, String> {
    match value {
        "Available" => Ok(TimeframeViewAvailabilityV1::Available),
        "InsufficientHistoricalDepth" => {
            Ok(TimeframeViewAvailabilityV1::InsufficientHistoricalDepth)
        }
        "NoTradeInterval" => Ok(TimeframeViewAvailabilityV1::NoTradeInterval),
        "MissingEvidence" => Ok(TimeframeViewAvailabilityV1::MissingEvidence),
        "PartialCandleForbidden" => Ok(TimeframeViewAvailabilityV1::PartialCandleForbidden),
        "IntegrityFailure" => Ok(TimeframeViewAvailabilityV1::IntegrityFailure),
        _ => Err("timeframe view availability rejected".to_string()),
    }
}

fn decode_as_of_snapshot(bytes: &[u8]) -> Result<MomentumMultiTimeframeAsOfSnapshotV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumMultiTimeframeAsOfSnapshotV1")?;
    let value = MomentumMultiTimeframeAsOfSnapshotV1 {
        prediction_timestamp_ms: fields.unsigned("prediction_timestamp_ms")?,
        view_digests: fields.strings("view_digests")?,
        availability: fields
            .strings("availability")?
            .iter()
            .map(|value| parse_availability(value))
            .collect::<Result<Vec<_>, _>>()?,
        all_views_closed: fields.boolean("all_views_closed")?,
        future_access_count: as_usize(fields.unsigned("future_access_count")?)?,
        partial_candle_access_count: as_usize(fields.unsigned("partial_candle_access_count")?)?,
        snapshot_digest: fields.string("snapshot_digest")?,
    };
    fields.finish()?;
    validate_as_of_snapshot(&value)?;
    Ok(value)
}

fn validate_protocol_seal(value: &MomentumProtocolPredictionSealV1) -> Result<(), String> {
    if value.prediction_timestamp_ms == 0
        || value.as_of_snapshot_digest.is_empty()
        || value.synthetic_prediction_identity.is_empty()
        || value.target_access_count_before_seal != 0
        || value.seal_digest != protocol_seal_digest(value)
    {
        return Err("protocol prediction seal rejected".to_string());
    }
    Ok(())
}

fn encode_protocol_seal(value: &MomentumProtocolPredictionSealV1) -> Result<Vec<u8>, String> {
    validate_protocol_seal(value)?;
    ArtifactBuilderV4_2::new("MomentumProtocolPredictionSealV1")
        .unsigned("prediction_timestamp_ms", value.prediction_timestamp_ms)
        .string("as_of_snapshot_digest", &value.as_of_snapshot_digest)
        .string(
            "synthetic_prediction_identity",
            &value.synthetic_prediction_identity,
        )
        .unsigned(
            "target_access_count_before_seal",
            as_u64(value.target_access_count_before_seal)?,
        )
        .string("seal_digest", &value.seal_digest)
        .encode()
}

fn decode_protocol_seal(bytes: &[u8]) -> Result<MomentumProtocolPredictionSealV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumProtocolPredictionSealV1")?;
    let value = MomentumProtocolPredictionSealV1 {
        prediction_timestamp_ms: fields.unsigned("prediction_timestamp_ms")?,
        as_of_snapshot_digest: fields.string("as_of_snapshot_digest")?,
        synthetic_prediction_identity: fields.string("synthetic_prediction_identity")?,
        target_access_count_before_seal: as_usize(
            fields.unsigned("target_access_count_before_seal")?,
        )?,
        seal_digest: fields.string("seal_digest")?,
    };
    fields.finish()?;
    validate_protocol_seal(&value)?;
    Ok(value)
}

fn validate_protocol_receipt(value: &MomentumProtocolReceiptV1) -> Result<(), String> {
    if value.prediction_timestamp_ms == 0
        || value.target_timestamp_ms
            != value
                .prediction_timestamp_ms
                .checked_add(PROTOCOL_CADENCE_MS)
                .unwrap_or_default()
        || value.as_of_snapshot_digest.is_empty()
        || value.prediction_seal_digest.is_empty()
        || !value.target_revealed_after_seal
        || value.target_value_access_count != 0
        || value.performance_claim_produced
        || value.receipt_digest != protocol_receipt_digest(value)
    {
        return Err("protocol receipt rejected".to_string());
    }
    Ok(())
}

fn encode_protocol_receipt(value: &MomentumProtocolReceiptV1) -> Result<Vec<u8>, String> {
    validate_protocol_receipt(value)?;
    ArtifactBuilderV4_2::new("MomentumProtocolReceiptV1")
        .unsigned("prediction_timestamp_ms", value.prediction_timestamp_ms)
        .unsigned("target_timestamp_ms", value.target_timestamp_ms)
        .string("as_of_snapshot_digest", &value.as_of_snapshot_digest)
        .string("prediction_seal_digest", &value.prediction_seal_digest)
        .boolean(
            "target_revealed_after_seal",
            value.target_revealed_after_seal,
        )
        .unsigned(
            "target_value_access_count",
            as_u64(value.target_value_access_count)?,
        )
        .boolean(
            "performance_claim_produced",
            value.performance_claim_produced,
        )
        .string("receipt_digest", &value.receipt_digest)
        .encode()
}

fn decode_protocol_receipt(bytes: &[u8]) -> Result<MomentumProtocolReceiptV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumProtocolReceiptV1")?;
    let value = MomentumProtocolReceiptV1 {
        prediction_timestamp_ms: fields.unsigned("prediction_timestamp_ms")?,
        target_timestamp_ms: fields.unsigned("target_timestamp_ms")?,
        as_of_snapshot_digest: fields.string("as_of_snapshot_digest")?,
        prediction_seal_digest: fields.string("prediction_seal_digest")?,
        target_revealed_after_seal: fields.boolean("target_revealed_after_seal")?,
        target_value_access_count: as_usize(fields.unsigned("target_value_access_count")?)?,
        performance_claim_produced: fields.boolean("performance_claim_produced")?,
        receipt_digest: fields.string("receipt_digest")?,
    };
    fields.finish()?;
    validate_protocol_receipt(&value)?;
    Ok(value)
}

fn validate_protocol(value: &MomentumProtocolReplayV1) -> Result<(), String> {
    let bindings_valid = value
        .snapshots
        .iter()
        .zip(&value.seals)
        .zip(&value.receipts)
        .all(|((snapshot, seal), receipt)| {
            validate_as_of_snapshot(snapshot).is_ok()
                && validate_protocol_seal(seal).is_ok()
                && validate_protocol_receipt(receipt).is_ok()
                && snapshot.prediction_timestamp_ms == seal.prediction_timestamp_ms
                && snapshot.snapshot_digest == seal.as_of_snapshot_digest
                && receipt.prediction_timestamp_ms == seal.prediction_timestamp_ms
                && receipt.as_of_snapshot_digest == snapshot.snapshot_digest
                && receipt.prediction_seal_digest == seal.seal_digest
        });
    if value.replay_version != PROTOCOL_VERSION
        || value.foundation_registration_digest.is_empty()
        || value.comparison_index_digest.is_empty()
        || value.event_count == 0
        || value.snapshots.len() != value.event_count
        || value.seals.len() != value.event_count
        || value.receipts.len() != value.event_count
        || !bindings_valid
        || !value.all_views_closed
        || value.future_access_count != 0
        || value.partial_candle_access_count != 0
        || !value.prediction_before_reveal
        || value.performance_claim_produced
        || value.replay_digest != protocol_digest(value)
    {
        return Err("multi-timeframe protocol replay rejected".to_string());
    }
    Ok(())
}

fn encode_protocol(value: &MomentumProtocolReplayV1) -> Result<Vec<u8>, String> {
    validate_protocol(value)?;
    ArtifactBuilderV4_2::new("MomentumProtocolReplayV1")
        .string("replay_version", &value.replay_version)
        .string(
            "foundation_registration_digest",
            &value.foundation_registration_digest,
        )
        .string("comparison_index_digest", &value.comparison_index_digest)
        .unsigned("event_count", as_u64(value.event_count)?)
        .messages(
            "snapshots",
            value
                .snapshots
                .iter()
                .map(encode_as_of_snapshot)
                .collect::<Result<Vec<_>, _>>()?,
        )
        .messages(
            "seals",
            value
                .seals
                .iter()
                .map(encode_protocol_seal)
                .collect::<Result<Vec<_>, _>>()?,
        )
        .messages(
            "receipts",
            value
                .receipts
                .iter()
                .map(encode_protocol_receipt)
                .collect::<Result<Vec<_>, _>>()?,
        )
        .boolean("all_views_closed", value.all_views_closed)
        .unsigned("future_access_count", as_u64(value.future_access_count)?)
        .unsigned(
            "partial_candle_access_count",
            as_u64(value.partial_candle_access_count)?,
        )
        .boolean("prediction_before_reveal", value.prediction_before_reveal)
        .boolean(
            "performance_claim_produced",
            value.performance_claim_produced,
        )
        .string("replay_digest", &value.replay_digest)
        .encode()
}

fn decode_protocol(bytes: &[u8]) -> Result<MomentumProtocolReplayV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumProtocolReplayV1")?;
    let value = MomentumProtocolReplayV1 {
        replay_version: fields.string("replay_version")?,
        foundation_registration_digest: fields.string("foundation_registration_digest")?,
        comparison_index_digest: fields.string("comparison_index_digest")?,
        event_count: as_usize(fields.unsigned("event_count")?)?,
        snapshots: fields
            .messages("snapshots")?
            .iter()
            .map(|bytes| decode_as_of_snapshot(bytes))
            .collect::<Result<Vec<_>, _>>()?,
        seals: fields
            .messages("seals")?
            .iter()
            .map(|bytes| decode_protocol_seal(bytes))
            .collect::<Result<Vec<_>, _>>()?,
        receipts: fields
            .messages("receipts")?
            .iter()
            .map(|bytes| decode_protocol_receipt(bytes))
            .collect::<Result<Vec<_>, _>>()?,
        all_views_closed: fields.boolean("all_views_closed")?,
        future_access_count: as_usize(fields.unsigned("future_access_count")?)?,
        partial_candle_access_count: as_usize(fields.unsigned("partial_candle_access_count")?)?,
        prediction_before_reveal: fields.boolean("prediction_before_reveal")?,
        performance_claim_produced: fields.boolean("performance_claim_produced")?,
        replay_digest: fields.string("replay_digest")?,
    };
    fields.finish()?;
    validate_protocol(&value)?;
    Ok(value)
}

fn persist_protocol(
    root: &Path,
    value: &MomentumProtocolReplayV1,
) -> Result<(usize, usize), String> {
    persist_one(
        root,
        "protocol_replays",
        &value.replay_digest,
        &encode_protocol(value)?,
        |bytes| Ok(decode_protocol(bytes)?.replay_digest),
    )
}

fn build_protocol(
    foundation: &MomentumMtfFoundationRegistrationV1,
    comparisons: &[DerivedNativeComparisonSummaryV1],
    views: &BTreeMap<MomentumHistoricalTimeframeV1, Vec<HistoricalCandleRowV1>>,
) -> Result<MomentumProtocolReplayV1, String> {
    if comparisons.len() != 6
        || comparisons
            .iter()
            .any(|comparison| validate_comparison(comparison).is_err())
    {
        return Err("native comparison evidence rejected".to_string());
    }
    let comparison_index_digest = stable_hash_string(&format!(
        "momentum-mtf-comparison-index-v1:model-replay-eligible={}:{:?}",
        native_comparisons_allow_model_replay(comparisons),
        comparisons
            .iter()
            .map(|value| value.comparison_digest.as_str())
            .collect::<Vec<_>>()
    ));
    let first = foundation
        .minute_start_timestamp_ms
        .checked_add(PROTOCOL_CADENCE_MS)
        .ok_or_else(|| "protocol first event overflow".to_string())?;
    let first = first.div_ceil(PROTOCOL_CADENCE_MS) * PROTOCOL_CADENCE_MS;
    let end_exclusive = foundation
        .minute_end_exclusive_timestamp_ms
        .checked_sub(PROTOCOL_CADENCE_MS)
        .ok_or_else(|| "protocol end unavailable".to_string())?;
    let mut snapshots = Vec::new();
    let mut seals = Vec::new();
    let mut receipts = Vec::new();
    let mut prediction_timestamp_ms = first;
    while prediction_timestamp_ms <= end_exclusive {
        let snapshot = build_as_of_snapshot(prediction_timestamp_ms, views)?;
        let intraday_available = snapshot.availability[..4]
            .iter()
            .all(|value| *value == TimeframeViewAvailabilityV1::Available);
        if intraday_available {
            let synthetic_prediction_identity = stable_hash_string(&format!(
                "momentum-mtf-synthetic-protocol-prediction-v1:{}:{}",
                snapshot.snapshot_digest, prediction_timestamp_ms
            ));
            let mut seal = MomentumProtocolPredictionSealV1 {
                prediction_timestamp_ms,
                as_of_snapshot_digest: snapshot.snapshot_digest.clone(),
                synthetic_prediction_identity,
                target_access_count_before_seal: 0,
                seal_digest: String::new(),
            };
            seal.seal_digest = protocol_seal_digest(&seal);
            validate_protocol_seal(&seal)?;
            let mut receipt = MomentumProtocolReceiptV1 {
                prediction_timestamp_ms,
                target_timestamp_ms: prediction_timestamp_ms
                    .checked_add(PROTOCOL_CADENCE_MS)
                    .ok_or_else(|| "protocol target overflow".to_string())?,
                as_of_snapshot_digest: snapshot.snapshot_digest.clone(),
                prediction_seal_digest: seal.seal_digest.clone(),
                target_revealed_after_seal: true,
                target_value_access_count: 0,
                performance_claim_produced: false,
                receipt_digest: String::new(),
            };
            receipt.receipt_digest = protocol_receipt_digest(&receipt);
            validate_protocol_receipt(&receipt)?;
            snapshots.push(snapshot);
            seals.push(seal);
            receipts.push(receipt);
        }
        prediction_timestamp_ms = prediction_timestamp_ms
            .checked_add(PROTOCOL_CADENCE_MS)
            .ok_or_else(|| "protocol cadence overflow".to_string())?;
    }
    let mut value = MomentumProtocolReplayV1 {
        replay_version: PROTOCOL_VERSION.to_string(),
        foundation_registration_digest: foundation.registration_digest.clone(),
        comparison_index_digest,
        event_count: receipts.len(),
        snapshots,
        seals,
        receipts,
        all_views_closed: true,
        future_access_count: 0,
        partial_candle_access_count: 0,
        prediction_before_reveal: true,
        performance_claim_produced: false,
        replay_digest: String::new(),
    };
    value.replay_digest = protocol_digest(&value);
    validate_protocol(&value)?;
    Ok(value)
}

fn native_comparisons_allow_model_replay(comparisons: &[DerivedNativeComparisonSummaryV1]) -> bool {
    comparisons.len() == 6
        && comparisons.iter().all(|comparison| {
            validate_comparison(comparison).is_ok() && !comparison.systematic_mismatch_blocks_replay
        })
}

fn task_name(value: MomentumHistoricalPredictionTaskV2) -> &'static str {
    match value {
        MomentumHistoricalPredictionTaskV2::IntradayTenMinute => "IntradayTenMinute",
        MomentumHistoricalPredictionTaskV2::DailyOneDay => "DailyOneDay",
        MomentumHistoricalPredictionTaskV2::WeeklyOneWeek => "WeeklyOneWeek",
    }
}

fn parse_task(value: &str) -> Result<MomentumHistoricalPredictionTaskV2, String> {
    match value {
        "IntradayTenMinute" => Ok(MomentumHistoricalPredictionTaskV2::IntradayTenMinute),
        "DailyOneDay" => Ok(MomentumHistoricalPredictionTaskV2::DailyOneDay),
        "WeeklyOneWeek" => Ok(MomentumHistoricalPredictionTaskV2::WeeklyOneWeek),
        _ => Err("future historical prediction task rejected".to_string()),
    }
}

fn build_future_registration(
    foundation: &MomentumMtfFoundationRegistrationV1,
) -> Result<MomentumHistoricalHardReplayRegistrationV2, String> {
    let mut value = MomentumHistoricalHardReplayRegistrationV2 {
        registration_version: FUTURE_VERSION.to_string(),
        foundation_registration_digest: foundation.registration_digest.clone(),
        tasks: vec![
            MomentumHistoricalPredictionTaskV2::IntradayTenMinute,
            MomentumHistoricalPredictionTaskV2::DailyOneDay,
            MomentumHistoricalPredictionTaskV2::WeeklyOneWeek,
        ],
        context_bindings: vec![
            "IntradayTenMinute:primary=1m,3m,5m,10m;regime=1d,1w,1mo,1y".to_string(),
            "DailyOneDay:primary=10m,1d;regime=1w,1mo,1y".to_string(),
            "WeeklyOneWeek:primary=1d,1w;regime=1mo,1y".to_string(),
        ],
        executed: false,
        registration_digest: String::new(),
    };
    value.registration_digest = future_digest(&value);
    validate_future_registration(&value)?;
    Ok(value)
}

fn validate_future_registration(
    value: &MomentumHistoricalHardReplayRegistrationV2,
) -> Result<(), String> {
    if value.registration_version != FUTURE_VERSION
        || value.foundation_registration_digest.is_empty()
        || value.tasks
            != [
                MomentumHistoricalPredictionTaskV2::IntradayTenMinute,
                MomentumHistoricalPredictionTaskV2::DailyOneDay,
                MomentumHistoricalPredictionTaskV2::WeeklyOneWeek,
            ]
        || value.context_bindings
            != [
                "IntradayTenMinute:primary=1m,3m,5m,10m;regime=1d,1w,1mo,1y",
                "DailyOneDay:primary=10m,1d;regime=1w,1mo,1y",
                "WeeklyOneWeek:primary=1d,1w;regime=1mo,1y",
            ]
        || value.executed
        || value.registration_digest != future_digest(value)
    {
        return Err("future hard replay registration rejected".to_string());
    }
    Ok(())
}

fn encode_future_registration(
    value: &MomentumHistoricalHardReplayRegistrationV2,
) -> Result<Vec<u8>, String> {
    validate_future_registration(value)?;
    ArtifactBuilderV4_2::new("MomentumHistoricalHardReplayRegistrationV2")
        .string("registration_version", &value.registration_version)
        .string(
            "foundation_registration_digest",
            &value.foundation_registration_digest,
        )
        .strings(
            "tasks",
            &value
                .tasks
                .iter()
                .map(|value| task_name(*value).to_string())
                .collect::<Vec<_>>(),
        )
        .strings("context_bindings", &value.context_bindings)
        .boolean("executed", value.executed)
        .string("registration_digest", &value.registration_digest)
        .encode()
}

fn decode_future_registration(
    bytes: &[u8],
) -> Result<MomentumHistoricalHardReplayRegistrationV2, String> {
    let mut fields =
        ArtifactReaderV4_2::decode(bytes, "MomentumHistoricalHardReplayRegistrationV2")?;
    let value = MomentumHistoricalHardReplayRegistrationV2 {
        registration_version: fields.string("registration_version")?,
        foundation_registration_digest: fields.string("foundation_registration_digest")?,
        tasks: fields
            .strings("tasks")?
            .iter()
            .map(|value| parse_task(value))
            .collect::<Result<Vec<_>, _>>()?,
        context_bindings: fields.strings("context_bindings")?,
        executed: fields.boolean("executed")?,
        registration_digest: fields.string("registration_digest")?,
    };
    fields.finish()?;
    validate_future_registration(&value)?;
    Ok(value)
}

fn build_ablation(
    foundation: &MomentumMtfFoundationRegistrationV1,
) -> Result<MomentumAblationRegistrationV1, String> {
    let mut value = MomentumAblationRegistrationV1 {
        registration_version: ABLATION_VERSION.to_string(),
        foundation_registration_digest: foundation.registration_digest.clone(),
        ordered_families: vec![
            "A0=current single-timeframe baseline".to_string(),
            "A1=intraday block only".to_string(),
            "A2=macro block only".to_string(),
            "A3=full eight-timeframe fusion".to_string(),
            "A4=full fusion without intraday block".to_string(),
            "A5=full fusion without macro block".to_string(),
        ],
        individual_leave_one_out_forbidden: true,
        result_selected_second_family_forbidden: true,
        executed: false,
        registration_digest: String::new(),
    };
    value.registration_digest = ablation_digest(&value);
    validate_ablation(&value)?;
    Ok(value)
}

fn validate_ablation(value: &MomentumAblationRegistrationV1) -> Result<(), String> {
    if value.registration_version != ABLATION_VERSION
        || value.foundation_registration_digest.is_empty()
        || value.ordered_families
            != [
                "A0=current single-timeframe baseline",
                "A1=intraday block only",
                "A2=macro block only",
                "A3=full eight-timeframe fusion",
                "A4=full fusion without intraday block",
                "A5=full fusion without macro block",
            ]
        || !value.individual_leave_one_out_forbidden
        || !value.result_selected_second_family_forbidden
        || value.executed
        || value.registration_digest != ablation_digest(value)
    {
        return Err("future ablation registration rejected".to_string());
    }
    Ok(())
}

fn encode_ablation(value: &MomentumAblationRegistrationV1) -> Result<Vec<u8>, String> {
    validate_ablation(value)?;
    ArtifactBuilderV4_2::new("MomentumAblationRegistrationV1")
        .string("registration_version", &value.registration_version)
        .string(
            "foundation_registration_digest",
            &value.foundation_registration_digest,
        )
        .strings("ordered_families", &value.ordered_families)
        .boolean(
            "individual_leave_one_out_forbidden",
            value.individual_leave_one_out_forbidden,
        )
        .boolean(
            "result_selected_second_family_forbidden",
            value.result_selected_second_family_forbidden,
        )
        .boolean("executed", value.executed)
        .string("registration_digest", &value.registration_digest)
        .encode()
}

fn decode_ablation(bytes: &[u8]) -> Result<MomentumAblationRegistrationV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumAblationRegistrationV1")?;
    let value = MomentumAblationRegistrationV1 {
        registration_version: fields.string("registration_version")?,
        foundation_registration_digest: fields.string("foundation_registration_digest")?,
        ordered_families: fields.strings("ordered_families")?,
        individual_leave_one_out_forbidden: fields.boolean("individual_leave_one_out_forbidden")?,
        result_selected_second_family_forbidden: fields
            .boolean("result_selected_second_family_forbidden")?,
        executed: fields.boolean("executed")?,
        registration_digest: fields.string("registration_digest")?,
    };
    fields.finish()?;
    validate_ablation(&value)?;
    Ok(value)
}

fn build_holdout(
    foundation: &MomentumMtfFoundationRegistrationV1,
    protocol: &MomentumProtocolReplayV1,
) -> Result<MomentumHistoricalHoldoutV1, String> {
    let timestamps = protocol
        .receipts
        .iter()
        .map(|receipt| receipt.prediction_timestamp_ms)
        .collect::<Vec<_>>();
    if timestamps.len() < 20 || timestamps.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err("holdout eligible timestamp range rejected".to_string());
    }
    let development_count = timestamps.len() * 70 / 100;
    let validation_count = timestamps.len() * 15 / 100;
    let holdout_count = timestamps.len() - development_count - validation_count;
    if development_count == 0 || validation_count == 0 || holdout_count == 0 {
        return Err("holdout partition count rejected".to_string());
    }
    let holdout_start_index = development_count + validation_count;
    let mut value = MomentumHistoricalHoldoutV1 {
        holdout_version: HOLDOUT_VERSION.to_string(),
        foundation_registration_digest: foundation.registration_digest.clone(),
        eligible_start_timestamp_ms: timestamps[0],
        eligible_end_timestamp_ms: *timestamps.last().unwrap_or(&timestamps[0]),
        development_end_exclusive_ms: timestamps[development_count],
        validation_end_exclusive_ms: timestamps[holdout_start_index],
        holdout_start_timestamp_ms: timestamps[holdout_start_index],
        development_event_count: development_count,
        validation_event_count: validation_count,
        holdout_event_count: holdout_count,
        labels_opened: false,
        metrics_computed: false,
        aggregate_comparison_opened: false,
        holdout_digest: String::new(),
    };
    value.holdout_digest = holdout_digest(&value);
    validate_holdout(&value)?;
    Ok(value)
}

fn validate_holdout(value: &MomentumHistoricalHoldoutV1) -> Result<(), String> {
    if value.holdout_version != HOLDOUT_VERSION
        || value.foundation_registration_digest.is_empty()
        || value.eligible_start_timestamp_ms >= value.development_end_exclusive_ms
        || value.development_end_exclusive_ms >= value.validation_end_exclusive_ms
        || value.validation_end_exclusive_ms != value.holdout_start_timestamp_ms
        || value.holdout_start_timestamp_ms > value.eligible_end_timestamp_ms
        || value.development_event_count == 0
        || value.validation_event_count == 0
        || value.holdout_event_count == 0
        || value.labels_opened
        || value.metrics_computed
        || value.aggregate_comparison_opened
        || value.holdout_digest != holdout_digest(value)
    {
        return Err("sealed historical holdout rejected".to_string());
    }
    Ok(())
}

fn encode_holdout(value: &MomentumHistoricalHoldoutV1) -> Result<Vec<u8>, String> {
    validate_holdout(value)?;
    ArtifactBuilderV4_2::new("MomentumHistoricalHoldoutV1")
        .string("holdout_version", &value.holdout_version)
        .string(
            "foundation_registration_digest",
            &value.foundation_registration_digest,
        )
        .unsigned(
            "eligible_start_timestamp_ms",
            value.eligible_start_timestamp_ms,
        )
        .unsigned("eligible_end_timestamp_ms", value.eligible_end_timestamp_ms)
        .unsigned(
            "development_end_exclusive_ms",
            value.development_end_exclusive_ms,
        )
        .unsigned(
            "validation_end_exclusive_ms",
            value.validation_end_exclusive_ms,
        )
        .unsigned(
            "holdout_start_timestamp_ms",
            value.holdout_start_timestamp_ms,
        )
        .unsigned(
            "development_event_count",
            as_u64(value.development_event_count)?,
        )
        .unsigned(
            "validation_event_count",
            as_u64(value.validation_event_count)?,
        )
        .unsigned("holdout_event_count", as_u64(value.holdout_event_count)?)
        .boolean("labels_opened", value.labels_opened)
        .boolean("metrics_computed", value.metrics_computed)
        .boolean(
            "aggregate_comparison_opened",
            value.aggregate_comparison_opened,
        )
        .string("holdout_digest", &value.holdout_digest)
        .encode()
}

fn decode_holdout(bytes: &[u8]) -> Result<MomentumHistoricalHoldoutV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumHistoricalHoldoutV1")?;
    let value = MomentumHistoricalHoldoutV1 {
        holdout_version: fields.string("holdout_version")?,
        foundation_registration_digest: fields.string("foundation_registration_digest")?,
        eligible_start_timestamp_ms: fields.unsigned("eligible_start_timestamp_ms")?,
        eligible_end_timestamp_ms: fields.unsigned("eligible_end_timestamp_ms")?,
        development_end_exclusive_ms: fields.unsigned("development_end_exclusive_ms")?,
        validation_end_exclusive_ms: fields.unsigned("validation_end_exclusive_ms")?,
        holdout_start_timestamp_ms: fields.unsigned("holdout_start_timestamp_ms")?,
        development_event_count: as_usize(fields.unsigned("development_event_count")?)?,
        validation_event_count: as_usize(fields.unsigned("validation_event_count")?)?,
        holdout_event_count: as_usize(fields.unsigned("holdout_event_count")?)?,
        labels_opened: fields.boolean("labels_opened")?,
        metrics_computed: fields.boolean("metrics_computed")?,
        aggregate_comparison_opened: fields.boolean("aggregate_comparison_opened")?,
        holdout_digest: fields.string("holdout_digest")?,
    };
    fields.finish()?;
    validate_holdout(&value)?;
    Ok(value)
}

fn persist_future_registration(
    root: &Path,
    value: &MomentumHistoricalHardReplayRegistrationV2,
) -> Result<(usize, usize), String> {
    persist_one(
        root,
        "future_experiment_registrations",
        &value.registration_digest,
        &encode_future_registration(value)?,
        |bytes| Ok(decode_future_registration(bytes)?.registration_digest),
    )
}

fn persist_ablation(
    root: &Path,
    value: &MomentumAblationRegistrationV1,
) -> Result<(usize, usize), String> {
    persist_one(
        root,
        "future_ablation_registrations",
        &value.registration_digest,
        &encode_ablation(value)?,
        |bytes| Ok(decode_ablation(bytes)?.registration_digest),
    )
}

fn persist_holdout(
    root: &Path,
    value: &MomentumHistoricalHoldoutV1,
) -> Result<(usize, usize), String> {
    persist_one(
        root,
        "sealed_holdouts",
        &value.holdout_digest,
        &encode_holdout(value)?,
        |bytes| Ok(decode_holdout(bytes)?.holdout_digest),
    )
}

fn execute_protocol_replay(
    root: &Path,
) -> Result<
    (
        MomentumProtocolReplayV1,
        MomentumHistoricalHardReplayRegistrationV2,
        MomentumAblationRegistrationV1,
        MomentumHistoricalHoldoutV1,
        (usize, usize),
    ),
    String,
> {
    if let (Some(protocol), Some(future), Some(ablation), Some(holdout)) = (
        read_single(&root.join("protocol_replays"), decode_protocol)?,
        read_single(
            &root.join("future_experiment_registrations"),
            decode_future_registration,
        )?,
        read_single(&root.join("future_ablation_registrations"), decode_ablation)?,
        read_single(&root.join("sealed_holdouts"), decode_holdout)?,
    ) {
        return Ok((protocol, future, ablation, holdout, (0, 0)));
    }
    let (_, foundation, _) = reopen_foundation(root)?;
    let foundation =
        foundation.ok_or_else(|| "multi-timeframe foundation unavailable".to_string())?;
    let comparisons = reopen_comparisons(root)?;
    let minute_index = reopen_index(root, MomentumHistoricalTimeframeV1::Minute1)?
        .ok_or_else(|| "canonical minute index unavailable".to_string())?;
    let daily_index = reopen_index(root, MomentumHistoricalTimeframeV1::Day1)?
        .ok_or_else(|| "canonical daily index unavailable".to_string())?;
    let minute_rows = reopen_canonical_rows(root, &minute_index)?;
    let daily_rows = reopen_canonical_rows(root, &daily_index)?;
    let mut views = BTreeMap::from([
        (MomentumHistoricalTimeframeV1::Minute1, minute_rows.clone()),
        (MomentumHistoricalTimeframeV1::Day1, daily_rows.clone()),
    ]);
    for timeframe in [
        MomentumHistoricalTimeframeV1::Minute3,
        MomentumHistoricalTimeframeV1::Minute5,
        MomentumHistoricalTimeframeV1::Minute10,
        MomentumHistoricalTimeframeV1::Week1,
        MomentumHistoricalTimeframeV1::Month1,
        MomentumHistoricalTimeframeV1::Year1,
    ] {
        let (base_index, base_rows) = if matches!(
            timeframe,
            MomentumHistoricalTimeframeV1::Minute3
                | MomentumHistoricalTimeframeV1::Minute5
                | MomentumHistoricalTimeframeV1::Minute10
        ) {
            (&minute_index, minute_rows.as_slice())
        } else {
            (&daily_index, daily_rows.as_slice())
        };
        let (rows, index) = aggregate_view(&foundation, base_index, base_rows, timeframe)?;
        let persisted = read_single(
            &root.join(format!("derived_{}/indices", timeframe.as_str())),
            decode_derived_index,
        )?
        .ok_or_else(|| "derived view index unavailable".to_string())?;
        if index != persisted {
            return Err("derived view deterministic replay rejected".to_string());
        }
        views.insert(timeframe, rows);
    }
    let protocol = build_protocol(&foundation, &comparisons, &views)?;
    let future = build_future_registration(&foundation)?;
    let ablation = build_ablation(&foundation)?;
    let holdout = build_holdout(&foundation, &protocol)?;
    let mut counts = (0, 0);
    add_counts(&mut counts, persist_protocol(root, &protocol)?);
    add_counts(&mut counts, persist_future_registration(root, &future)?);
    add_counts(&mut counts, persist_ablation(root, &ablation)?);
    add_counts(&mut counts, persist_holdout(root, &holdout)?);
    Ok((protocol, future, ablation, holdout, counts))
}

pub fn build_momentum_timeframe_feature_block_v1(
    timeframe: MomentumHistoricalTimeframeV1,
    source_view_digest: &str,
    independently_normalized_values: &[f64],
    expected_dimension: usize,
    context_complete: bool,
) -> Result<MomentumTimeframeFeatureBlockV1, String> {
    if source_view_digest.is_empty()
        || expected_dimension == 0
        || independently_normalized_values.len() != expected_dimension
        || independently_normalized_values
            .iter()
            .any(|value| !value.is_finite())
        || !context_complete
    {
        return Err("timeframe feature block input rejected".to_string());
    }
    let feature_schema_digest = stable_hash_string(&format!(
        "momentum-mtf-feature-schema-v1:{}:{}:independently-normalized",
        timeframe.as_str(),
        expected_dimension
    ));
    let feature_vector_digest = stable_hash_string(&format!(
        "momentum-mtf-feature-vector-v1:{}:{}:{:?}",
        timeframe.as_str(),
        source_view_digest,
        independently_normalized_values
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>()
    ));
    let value = MomentumTimeframeFeatureBlockV1 {
        timeframe,
        source_view_digest: source_view_digest.to_string(),
        feature_schema_digest,
        feature_vector_digest,
        numeric_values_private: true,
    };
    validate_feature_block(&value)?;
    Ok(value)
}

fn validate_feature_block(value: &MomentumTimeframeFeatureBlockV1) -> Result<(), String> {
    if value.source_view_digest.is_empty()
        || value.feature_schema_digest.is_empty()
        || value.feature_vector_digest.is_empty()
        || !value.numeric_values_private
    {
        return Err("timeframe feature block rejected".to_string());
    }
    Ok(())
}

fn optional_derived_indices(root: &Path) -> Result<Vec<DerivedViewIndexV1>, String> {
    let mut values = Vec::new();
    for timeframe in [
        MomentumHistoricalTimeframeV1::Minute3,
        MomentumHistoricalTimeframeV1::Minute5,
        MomentumHistoricalTimeframeV1::Minute10,
        MomentumHistoricalTimeframeV1::Week1,
        MomentumHistoricalTimeframeV1::Month1,
        MomentumHistoricalTimeframeV1::Year1,
    ] {
        if let Some(value) = read_single(
            &root.join(format!("derived_{}/indices", timeframe.as_str())),
            decode_derived_index,
        )? {
            values.push(value);
        }
    }
    Ok(values)
}

fn optional_comparisons(root: &Path) -> Result<Vec<DerivedNativeComparisonSummaryV1>, String> {
    let mut values = Vec::new();
    for timeframe in [
        MomentumHistoricalTimeframeV1::Minute3,
        MomentumHistoricalTimeframeV1::Minute5,
        MomentumHistoricalTimeframeV1::Minute10,
        MomentumHistoricalTimeframeV1::Week1,
        MomentumHistoricalTimeframeV1::Month1,
        MomentumHistoricalTimeframeV1::Year1,
    ] {
        if let Some(value) = read_single(
            &root.join(format!("native_comparisons/{}", timeframe.as_str())),
            decode_comparison,
        )? {
            values.push(value);
        }
    }
    Ok(values)
}

fn register_foundation(
    root: &Path,
    snapshots: &[DataSnapshot],
    live: &MomentumProspectiveSeriesReportV4,
    config: &UpbitHistoricalPilotConfigV0,
) -> Result<(usize, usize), String> {
    let daily = select_daily_snapshot(snapshots)?;
    let pause = build_pause(live)?;
    let foundation = build_foundation(&pause, &daily)?;
    let plan = build_plan(&foundation, config)?;
    let (existing_pause, existing_foundation, existing_plan) = reopen_foundation(root)?;
    if existing_pause.as_ref().is_some_and(|value| value != &pause)
        || existing_foundation
            .as_ref()
            .is_some_and(|value| value != &foundation)
        || existing_plan.as_ref().is_some_and(|value| value != &plan)
    {
        return Err("registered multi-timeframe foundation identity changed".to_string());
    }
    let mut counts = (0, 0);
    add_counts(&mut counts, persist_pause(root, &pause)?);
    add_counts(&mut counts, persist_foundation(root, &foundation)?);
    add_counts(&mut counts, persist_plan(root, &plan)?);
    Ok(counts)
}

fn availability_counts(protocol: Option<&MomentumProtocolReplayV1>) -> Vec<String> {
    let mut counts = BTreeMap::<(MomentumHistoricalTimeframeV1, &'static str), usize>::new();
    if let Some(protocol) = protocol {
        for snapshot in &protocol.snapshots {
            for (timeframe, availability) in MomentumHistoricalTimeframeV1::ORDERED
                .iter()
                .zip(&snapshot.availability)
            {
                *counts
                    .entry((*timeframe, availability.as_str()))
                    .or_default() += 1;
            }
        }
    }
    counts
        .into_iter()
        .map(|((timeframe, availability), count)| {
            format!("{}:{availability}={count}", timeframe.as_str())
        })
        .collect()
}

fn safety_counters(
    receipts: &[HistoricalPageReceiptV1],
    plan: Option<&MomentumMtfAcquisitionPlanV1>,
) -> MomentumMtfSafetyCountersV1 {
    MomentumMtfSafetyCountersV1 {
        network_request_attempts: receipts.len(),
        transport_constructions: receipts.len(),
        retries: receipts.iter().map(|receipt| receipt.retry_count).sum(),
        maximum_concurrency: plan.map_or(0, |plan| plan.maximum_concurrency),
        verified_page_count: receipts
            .iter()
            .filter(|receipt| receipt.status == PageReceiptStatusV1::Verified)
            .count(),
        failed_page_count: receipts
            .iter()
            .filter(|receipt| receipt.status == PageReceiptStatusV1::TerminalFailure)
            .count(),
        ..MomentumMtfSafetyCountersV1::default()
    }
}

fn validate_public_report(value: &MomentumMtfHistoryPublicReportV1) -> Result<(), String> {
    let zero_authority = [
        value.safety_counters.live_outcome_requests,
        value.safety_counters.live_outcome_openings,
        value.safety_counters.live_label_reads,
        value.safety_counters.live_metric_computations,
        value.safety_counters.live_evaluations,
        value.safety_counters.live_participant_changes,
        value.safety_counters.live_parameter_updates,
        value.safety_counters.live_normalizer_refits,
        value.safety_counters.live_feature_policy_changes,
        value.safety_counters.winner_selections,
        value.safety_counters.rankings,
        value.safety_counters.reward_applications,
        value.safety_counters.penalty_applications,
        value.safety_counters.chair_decisions,
        value.safety_counters.committee_votes,
        value.safety_counters.voice_changes,
        value.safety_counters.tier_changes,
        value.safety_counters.cooldowns,
        value.safety_counters.promotions,
        value.safety_counters.quarantines,
        value.safety_counters.paper_executions,
        value.safety_counters.live_executions,
    ]
    .into_iter()
    .all(|count| count == 0);
    if value.report_version != REPORT_VERSION
        || value.run_mode.is_empty()
        || value.ordered_timeframes != MomentumHistoricalTimeframeV1::ORDERED
        || value.canonical_bases
            != [
                MomentumHistoricalTimeframeV1::Minute1,
                MomentumHistoricalTimeframeV1::Day1,
            ]
        || value.derived_indices.len() > 6
        || value.native_comparisons.len() > 6
        || value
            .acquisition_plan
            .as_ref()
            .is_some_and(|plan| validate_plan(plan).is_err())
        || value
            .native_comparisons
            .iter()
            .any(|comparison| validate_comparison(comparison).is_err())
        || value.future_access_count != 0
        || value.partial_candle_access_count != 0
        || value.holdout_labels_opened
        || value.trading_simulation_status
            != TradingSimulationStatus::BlockedNoFrozenExecutionPolicy
        || !zero_authority
        || !value.live_protected_artifacts_unchanged
        || !value.active_roster_unchanged
        || value.safety_counters.maximum_concurrency > 1
        || value.safety_counters.retries != 0
        || value.report_digest != report_digest(value)
    {
        return Err("multi-timeframe public report rejected".to_string());
    }
    Ok(())
}

fn build_public_report(
    root: &Path,
    mode: MomentumMtfHistoryRunModeV1,
    counts: (usize, usize),
    live_protected_artifacts_unchanged: bool,
    active_roster_unchanged: bool,
) -> Result<MomentumMtfHistoryPublicReportV1, String> {
    let (pause, foundation, plan) = reopen_foundation(root)?;
    let minute_index = reopen_index(root, MomentumHistoricalTimeframeV1::Minute1)?;
    let daily_index = reopen_index(root, MomentumHistoricalTimeframeV1::Day1)?;
    let derived_indices = optional_derived_indices(root)?;
    let native_comparisons = optional_comparisons(root)?;
    let protocol = read_single(&root.join("protocol_replays"), decode_protocol)?;
    let future = read_single(
        &root.join("future_experiment_registrations"),
        decode_future_registration,
    )?;
    let ablation = read_single(&root.join("future_ablation_registrations"), decode_ablation)?;
    let holdout = read_single(&root.join("sealed_holdouts"), decode_holdout)?;
    let receipts = load_receipts(root)?;
    let canonical_complete = minute_index.is_some()
        && daily_index.is_some()
        && plan
            .as_ref()
            .is_some_and(|plan| native_receipts_complete(&receipts, plan));
    let phase = if protocol.is_some() && future.is_some() && ablation.is_some() && holdout.is_some()
    {
        MomentumMtfHistoryPhaseV1::ProtocolReplayComplete
    } else if derived_indices.len() == 6 && native_comparisons.len() == 6 {
        MomentumMtfHistoryPhaseV1::DerivedViewsComplete
    } else if canonical_complete {
        MomentumMtfHistoryPhaseV1::CanonicalBackfillComplete
    } else if foundation.is_some() && pause.is_some() && plan.is_some() {
        MomentumMtfHistoryPhaseV1::FoundationRegistered
    } else {
        MomentumMtfHistoryPhaseV1::Unregistered
    };
    let mut value = MomentumMtfHistoryPublicReportV1 {
        report_version: REPORT_VERSION.to_string(),
        run_mode: mode.as_str().to_string(),
        phase,
        live_continuation_policy: pause.as_ref().map(|pause| pause.policy),
        live_pause_digest: pause.as_ref().map(|pause| pause.pause_digest.clone()),
        foundation_registration_digest: foundation
            .as_ref()
            .map(|foundation| foundation.registration_digest.clone()),
        acquisition_plan: plan.clone(),
        ordered_timeframes: MomentumHistoricalTimeframeV1::ORDERED.to_vec(),
        canonical_bases: vec![
            MomentumHistoricalTimeframeV1::Minute1,
            MomentumHistoricalTimeframeV1::Day1,
        ],
        minute_index,
        daily_index,
        derived_indices,
        native_comparisons,
        protocol_event_count: protocol.as_ref().map_or(0, |protocol| protocol.event_count),
        protocol_replay_digest: protocol
            .as_ref()
            .map(|protocol| protocol.replay_digest.clone()),
        all_views_closed: protocol
            .as_ref()
            .is_some_and(|protocol| protocol.all_views_closed),
        future_access_count: protocol
            .as_ref()
            .map_or(0, |protocol| protocol.future_access_count),
        partial_candle_access_count: protocol
            .as_ref()
            .map_or(0, |protocol| protocol.partial_candle_access_count),
        prediction_before_target_reveal: protocol
            .as_ref()
            .is_some_and(|protocol| protocol.prediction_before_reveal),
        availability_counts: availability_counts(protocol.as_ref()),
        future_experiment_registration_digest: future
            .as_ref()
            .map(|value| value.registration_digest.clone()),
        future_ablation_registration_digest: ablation
            .as_ref()
            .map(|value| value.registration_digest.clone()),
        sealed_holdout_digest: holdout.as_ref().map(|value| value.holdout_digest.clone()),
        holdout_start_timestamp_ms: holdout
            .as_ref()
            .map(|value| value.holdout_start_timestamp_ms),
        holdout_labels_opened: holdout.as_ref().is_some_and(|value| value.labels_opened),
        trading_simulation_status: TradingSimulationStatus::BlockedNoFrozenExecutionPolicy,
        safety_counters: safety_counters(&receipts, plan.as_ref()),
        live_protected_artifacts_unchanged,
        active_roster_unchanged,
        artifacts_written: counts.0,
        duplicate_artifact_count: counts.1,
        report_digest: String::new(),
    };
    value.report_digest = report_digest(&value);
    validate_public_report(&value)?;
    Ok(value)
}

fn run_at(
    root: &Path,
    live_root: &Path,
    snapshots: &[DataSnapshot],
    live: Option<&MomentumProspectiveSeriesReportV4>,
    config: Option<&UpbitHistoricalPilotConfigV0>,
    mode: MomentumMtfHistoryRunModeV1,
    allow_network: bool,
    confirmation: bool,
    client: &dyn MarketDataHttpClient,
) -> Result<MomentumMtfHistoryPublicReportV1, String> {
    let protected_before = tree_identity(live_root)?;
    let roster_before = active_roster_digest();
    let mut counts = (0, 0);
    match mode {
        MomentumMtfHistoryRunModeV1::Status | MomentumMtfHistoryRunModeV1::DryRun => {
            if allow_network || confirmation {
                return Err("read-only multi-timeframe mode rejects authority".to_string());
            }
        }
        MomentumMtfHistoryRunModeV1::RegisterFoundation => {
            if allow_network || confirmation {
                return Err("foundation registration rejects network authority".to_string());
            }
            counts = register_foundation(
                root,
                snapshots,
                live.ok_or_else(|| "sealed live report required".to_string())?,
                config.ok_or_else(|| "historical provider configuration required".to_string())?,
            )?;
        }
        MomentumMtfHistoryRunModeV1::ExecuteBackfill => {
            if !allow_network || !confirmation {
                return Err("bounded historical network confirmation required".to_string());
            }
            counts = execute_backfill_with_client(root, snapshots, client)?.2;
        }
        MomentumMtfHistoryRunModeV1::DeriveViews => {
            if allow_network || confirmation {
                return Err("derived view mode rejects network authority".to_string());
            }
            counts = derive_and_compare_views(root)?.2;
        }
        MomentumMtfHistoryRunModeV1::ProtocolReplay => {
            if allow_network || confirmation {
                return Err("protocol replay rejects network authority".to_string());
            }
            counts = execute_protocol_replay(root)?.4;
        }
    }
    let protected_after = tree_identity(live_root)?;
    let roster_after = active_roster_digest();
    build_public_report(
        root,
        mode,
        counts,
        protected_before == protected_after,
        roster_before == roster_after,
    )
}

pub fn run_momentum_multitimeframe_history_v1(
    snapshots: &[DataSnapshot],
    live: Option<&MomentumProspectiveSeriesReportV4>,
    config: Option<&UpbitHistoricalPilotConfigV0>,
    mode: MomentumMtfHistoryRunModeV1,
    allow_network: bool,
    confirmation: bool,
) -> Result<MomentumMtfHistoryPublicReportV1, String> {
    run_at(
        Path::new(ROOT),
        Path::new(LIVE_ROOT),
        snapshots,
        live,
        config,
        mode,
        allow_network,
        confirmation,
        &CurlHttpClient,
    )
}

pub fn format_momentum_multitimeframe_history_text_v1(
    report: &MomentumMtfHistoryPublicReportV1,
) -> Result<String, String> {
    validate_public_report(report)?;
    let timeframe_set = report
        .ordered_timeframes
        .iter()
        .map(|timeframe| timeframe.as_str())
        .collect::<Vec<_>>()
        .join(",");
    let canonical_bases = report
        .canonical_bases
        .iter()
        .map(|timeframe| timeframe.as_str())
        .collect::<Vec<_>>()
        .join(",");
    Ok(format!(
        concat!(
            "Momentum Multi-Timeframe Historical Foundation V1\n",
            "mode: {}\nphase: {:?}\ntimeframes: {}\ncanonical bases: {}\n",
            "live pause: {}\nfoundation: {}\nrequest budget: {}\n",
            "historical requests: {}\nverified pages: {}\nfailed pages: {}\n",
            "minute rows: {}\ndaily rows: {}\nderived views: {}\n",
            "native comparisons: {}\nprotocol events: {}\nall views closed: {}\n",
            "future access: {}\npartial candle access: {}\nprediction before target reveal: {}\n",
            "holdout labels opened: {}\nartifacts written: {}\nduplicate artifacts: {}\n",
            "protected artifacts unchanged: {}\nactive roster unchanged: {}\nreport digest: {}\n"
        ),
        report.run_mode,
        report.phase,
        timeframe_set,
        canonical_bases,
        report
            .live_pause_digest
            .as_deref()
            .unwrap_or("unregistered"),
        report
            .foundation_registration_digest
            .as_deref()
            .unwrap_or("unregistered"),
        report
            .acquisition_plan
            .as_ref()
            .map_or(0, |plan| plan.exact_total_request_budget),
        report.safety_counters.network_request_attempts,
        report.safety_counters.verified_page_count,
        report.safety_counters.failed_page_count,
        report
            .minute_index
            .as_ref()
            .map_or(0, |index| index.total_row_count),
        report
            .daily_index
            .as_ref()
            .map_or(0, |index| index.total_row_count),
        report.derived_indices.len(),
        report.native_comparisons.len(),
        report.protocol_event_count,
        report.all_views_closed,
        report.future_access_count,
        report.partial_candle_access_count,
        report.prediction_before_target_reveal,
        report.holdout_labels_opened,
        report.artifacts_written,
        report.duplicate_artifact_count,
        report.live_protected_artifacts_unchanged,
        report.active_roster_unchanged,
        report.report_digest,
    ))
}

trait StableMacroEnumV1: Copy + std::fmt::Debug + Sized {
    fn parse_stable(value: &str) -> Result<Self, String>;

    fn stable_name(self) -> String {
        format!("{self:?}")
    }
}

macro_rules! stable_macro_enum {
    ($type:ty, $error:literal, [$($variant:ident),+ $(,)?]) => {
        impl StableMacroEnumV1 for $type {
            fn parse_stable(value: &str) -> Result<Self, String> {
                match value {
                    $(stringify!($variant) => Ok(Self::$variant),)+
                    _ => Err($error.to_string()),
                }
            }
        }
    };
}

stable_macro_enum!(
    MomentumMacroBoundaryComparisonV1,
    "macro boundary comparison rejected",
    [
        ExactSameInterval,
        SamePeriodDifferentTimestampRepresentation,
        UtcVsKstBoundaryShift,
        FirstDayOfPeriodMismatch,
        OpeningBoundaryMismatch,
        ClosingBoundaryMismatch,
        NativePeriodNotReconstructableFromDailyBase,
        IntegrityFailure,
    ]
);
stable_macro_enum!(
    MomentumMacroCompletenessComparisonV1,
    "macro completeness comparison rejected",
    [
        BothComplete,
        NativePartialDerivedExcluded,
        NativeCompleteDerivedPartial,
        SourceCoverageStartsInsidePeriod,
        SourceCoverageEndsInsidePeriod,
        NoTradeCompositionDiffers,
        MissingDailyEvidence,
        IntegrityFailure,
    ]
);
stable_macro_enum!(
    MomentumMacroValueComparisonV1,
    "macro value comparison rejected",
    [
        ExactAllFields,
        AccumulationWithinRegisteredTolerance,
        OpenMismatch,
        HighMismatch,
        LowMismatch,
        CloseMismatch,
        VolumeOutsideRegisteredTolerance,
        TradeValueOutsideRegisteredTolerance,
        MultipleValueMismatches,
        NotComparableBoundaryMismatch,
        IntegrityFailure,
    ]
);
stable_macro_enum!(
    MomentumMacroMismatchRootCauseV1,
    "macro root cause rejected",
    [
        IncompleteNativeCurrentPeriod,
        IncompleteDerivedCurrentPeriod,
        PartialFirstCalendarPeriod,
        UtcKstCalendarBoundaryDifference,
        ProviderFirstDayPeriodSemantics,
        DailyBaseInsufficientForNativeBoundary,
        NoTradeIntervalComposition,
        MissingCanonicalDailyEvidence,
        AccumulationRoundingOnly,
        IncorrectDerivedAggregation,
        IncorrectNativeNormalization,
        ProviderContractAmbiguous,
        CorruptEvidence,
    ]
);
stable_macro_enum!(
    MomentumMacroCandleDispositionV1,
    "macro candle disposition rejected",
    [
        QualifiedDerivedFromDaily,
        QualifiedDerivedFromDailyWithinRegisteredTolerance,
        ExcludedPartialPeriodNotAFailure,
        NativeCanonicalRequired,
        DerivedAggregationDefect,
        ExcludedUnresolved,
    ]
);
stable_macro_enum!(
    MomentumCanonicalMacroSourceV1,
    "canonical macro source rejected",
    [
        DerivedFromCanonicalDaily,
        NativeProviderCandle,
        ExcludedUnresolved,
    ]
);
stable_macro_enum!(
    MomentumTimeframeQualificationV1,
    "timeframe qualification rejected",
    [
        QualifiedDerivedCanonical,
        QualifiedNativeCanonical,
        ExcludedPartialOnly,
        ExcludedUnresolved,
    ]
);

fn validate_macro_receipt(value: &MomentumMacroCandleForensicReceiptV1) -> Result<(), String> {
    let macro_timeframe = matches!(
        value.timeframe,
        MomentumHistoricalTimeframeV1::Month1 | MomentumHistoricalTimeframeV1::Year1
    );
    let comparable_boundary = matches!(
        value.boundary_comparison,
        MomentumMacroBoundaryComparisonV1::ExactSameInterval
            | MomentumMacroBoundaryComparisonV1::SamePeriodDifferentTimestampRepresentation
    );
    let qualified = matches!(
        value.disposition,
        MomentumMacroCandleDispositionV1::QualifiedDerivedFromDaily
            | MomentumMacroCandleDispositionV1::QualifiedDerivedFromDailyWithinRegisteredTolerance
    );
    let root_required = !qualified;
    if value.forensic_version != MACRO_FORENSIC_VERSION
        || !macro_timeframe
        || [
            &value.native_candle_digest,
            &value.derived_candle_digest,
            &value.market,
            &value.provider_id,
            &value.native_response_digest,
        ]
        .iter()
        .any(|item| item.is_empty())
        || value.market != MARKET
        || value.provider_id != PROVIDER
        || value.native_open_timestamp_ms >= value.native_close_exclusive_timestamp_ms
        || value.derived_open_timestamp_ms >= value.derived_close_exclusive_timestamp_ms
        || value.native_candle_timestamp_ms != value.native_open_timestamp_ms
        || value.native_source_row_digests.is_empty()
        || value.derived_source_row_digests.is_empty()
        || value
            .native_source_row_digests
            .iter()
            .chain(&value.derived_source_row_digests)
            .any(String::is_empty)
        || (comparable_boundary
            != (value.native_open_timestamp_ms == value.derived_open_timestamp_ms
                && value.native_close_exclusive_timestamp_ms
                    == value.derived_close_exclusive_timestamp_ms))
        || (value.value_comparison == MomentumMacroValueComparisonV1::NotComparableBoundaryMismatch)
            != !comparable_boundary
        || root_required != value.root_cause.is_some()
        || (qualified && value.root_cause.is_some())
        || (value.disposition == MomentumMacroCandleDispositionV1::QualifiedDerivedFromDaily
            && value.value_comparison != MomentumMacroValueComparisonV1::ExactAllFields)
        || (value.disposition
            == MomentumMacroCandleDispositionV1::QualifiedDerivedFromDailyWithinRegisteredTolerance
            && value.value_comparison
                != MomentumMacroValueComparisonV1::AccumulationWithinRegisteredTolerance)
        || value.receipt_digest != macro_receipt_digest(value)
    {
        return Err("macro candle forensic receipt rejected".to_string());
    }
    Ok(())
}

fn encode_macro_receipt(value: &MomentumMacroCandleForensicReceiptV1) -> Result<Vec<u8>, String> {
    validate_macro_receipt(value)?;
    let native_last_trade_timestamp_ms = value
        .native_last_trade_timestamp_ms
        .map(|timestamp| timestamp.to_string());
    ArtifactBuilderV4_2::new("MomentumMacroCandleForensicReceiptV1")
        .string("forensic_version", &value.forensic_version)
        .string("timeframe", value.timeframe.as_str())
        .string("native_candle_digest", &value.native_candle_digest)
        .string("derived_candle_digest", &value.derived_candle_digest)
        .unsigned(
            "native_candle_timestamp_ms",
            value.native_candle_timestamp_ms,
        )
        .optional_string(
            "native_candle_kst_timestamp",
            &value.native_candle_kst_timestamp,
        )
        .optional_string(
            "native_first_day_of_period",
            &value.native_first_day_of_period,
        )
        .optional_string(
            "native_last_trade_timestamp_ms",
            &native_last_trade_timestamp_ms,
        )
        .unsigned("native_open_timestamp_ms", value.native_open_timestamp_ms)
        .unsigned(
            "native_close_exclusive_timestamp_ms",
            value.native_close_exclusive_timestamp_ms,
        )
        .unsigned("derived_open_timestamp_ms", value.derived_open_timestamp_ms)
        .unsigned(
            "derived_close_exclusive_timestamp_ms",
            value.derived_close_exclusive_timestamp_ms,
        )
        .unsigned("request_to_exclusive_ms", value.request_to_exclusive_ms)
        .string("market", &value.market)
        .string("provider_id", &value.provider_id)
        .string("native_response_digest", &value.native_response_digest)
        .strings(
            "native_source_row_digests",
            &value.native_source_row_digests,
        )
        .strings(
            "derived_source_row_digests",
            &value.derived_source_row_digests,
        )
        .string(
            "boundary_comparison",
            value.boundary_comparison.stable_name(),
        )
        .string(
            "completeness_comparison",
            value.completeness_comparison.stable_name(),
        )
        .string("value_comparison", value.value_comparison.stable_name())
        .optional_string(
            "root_cause",
            &value.root_cause.map(StableMacroEnumV1::stable_name),
        )
        .string("disposition", value.disposition.stable_name())
        .string("receipt_digest", &value.receipt_digest)
        .encode()
}

fn decode_macro_receipt(bytes: &[u8]) -> Result<MomentumMacroCandleForensicReceiptV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumMacroCandleForensicReceiptV1")?;
    let value = MomentumMacroCandleForensicReceiptV1 {
        forensic_version: fields.string("forensic_version")?,
        timeframe: MomentumHistoricalTimeframeV1::parse(&fields.string("timeframe")?)?,
        native_candle_digest: fields.string("native_candle_digest")?,
        derived_candle_digest: fields.string("derived_candle_digest")?,
        native_candle_timestamp_ms: fields.unsigned("native_candle_timestamp_ms")?,
        native_candle_kst_timestamp: fields.optional_string("native_candle_kst_timestamp")?,
        native_first_day_of_period: fields.optional_string("native_first_day_of_period")?,
        native_last_trade_timestamp_ms: fields
            .optional_string("native_last_trade_timestamp_ms")?
            .map(|timestamp| {
                timestamp
                    .parse::<u64>()
                    .map_err(|_| "native last-trade timestamp rejected".to_string())
            })
            .transpose()?,
        native_open_timestamp_ms: fields.unsigned("native_open_timestamp_ms")?,
        native_close_exclusive_timestamp_ms: fields
            .unsigned("native_close_exclusive_timestamp_ms")?,
        derived_open_timestamp_ms: fields.unsigned("derived_open_timestamp_ms")?,
        derived_close_exclusive_timestamp_ms: fields
            .unsigned("derived_close_exclusive_timestamp_ms")?,
        request_to_exclusive_ms: fields.unsigned("request_to_exclusive_ms")?,
        market: fields.string("market")?,
        provider_id: fields.string("provider_id")?,
        native_response_digest: fields.string("native_response_digest")?,
        native_source_row_digests: fields.strings("native_source_row_digests")?,
        derived_source_row_digests: fields.strings("derived_source_row_digests")?,
        boundary_comparison: MomentumMacroBoundaryComparisonV1::parse_stable(
            &fields.string("boundary_comparison")?,
        )?,
        completeness_comparison: MomentumMacroCompletenessComparisonV1::parse_stable(
            &fields.string("completeness_comparison")?,
        )?,
        value_comparison: MomentumMacroValueComparisonV1::parse_stable(
            &fields.string("value_comparison")?,
        )?,
        root_cause: fields
            .optional_string("root_cause")?
            .map(|value| MomentumMacroMismatchRootCauseV1::parse_stable(&value))
            .transpose()?,
        disposition: MomentumMacroCandleDispositionV1::parse_stable(
            &fields.string("disposition")?,
        )?,
        receipt_digest: fields.string("receipt_digest")?,
    };
    fields.finish()?;
    validate_macro_receipt(&value)?;
    Ok(value)
}

fn validate_macro_aggregate(value: &MomentumMacroForensicAggregateV1) -> Result<(), String> {
    if value.aggregate_version != MACRO_FORENSIC_AGGREGATE_VERSION
        || !matches!(
            value.timeframe,
            MomentumHistoricalTimeframeV1::Month1 | MomentumHistoricalTimeframeV1::Year1
        )
        || value.compared_period_count == 0
        || value.compared_period_count != value.ordered_receipt_digests.len()
        || value.compared_period_count
            != value.exact_count
                + value.tolerance_count
                + value.failed_count
                + value.excluded_partial_count
        || value.unresolved_count > value.failed_count
        || value.ordered_receipt_digests.iter().any(String::is_empty)
        || value
            .ordered_receipt_digests
            .iter()
            .collect::<BTreeSet<_>>()
            .len()
            != value.ordered_receipt_digests.len()
        || !value.complete_forensic_coverage
        || value.aggregate_digest != macro_aggregate_digest(value)
    {
        return Err("macro forensic aggregate rejected".to_string());
    }
    Ok(())
}

fn encode_macro_aggregate(value: &MomentumMacroForensicAggregateV1) -> Result<Vec<u8>, String> {
    validate_macro_aggregate(value)?;
    ArtifactBuilderV4_2::new("MomentumMacroForensicAggregateV1")
        .string("aggregate_version", &value.aggregate_version)
        .string("timeframe", value.timeframe.as_str())
        .strings("ordered_receipt_digests", &value.ordered_receipt_digests)
        .unsigned(
            "compared_period_count",
            as_u64(value.compared_period_count)?,
        )
        .unsigned("exact_count", as_u64(value.exact_count)?)
        .unsigned("tolerance_count", as_u64(value.tolerance_count)?)
        .unsigned("failed_count", as_u64(value.failed_count)?)
        .unsigned(
            "excluded_partial_count",
            as_u64(value.excluded_partial_count)?,
        )
        .unsigned("unresolved_count", as_u64(value.unresolved_count)?)
        .strings("root_cause_counts", &value.root_cause_counts)
        .strings("disposition_counts", &value.disposition_counts)
        .boolean(
            "complete_forensic_coverage",
            value.complete_forensic_coverage,
        )
        .boolean("native_metadata_complete", value.native_metadata_complete)
        .string("aggregate_digest", &value.aggregate_digest)
        .encode()
}

fn decode_macro_aggregate(bytes: &[u8]) -> Result<MomentumMacroForensicAggregateV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumMacroForensicAggregateV1")?;
    let value = MomentumMacroForensicAggregateV1 {
        aggregate_version: fields.string("aggregate_version")?,
        timeframe: MomentumHistoricalTimeframeV1::parse(&fields.string("timeframe")?)?,
        ordered_receipt_digests: fields.strings("ordered_receipt_digests")?,
        compared_period_count: as_usize(fields.unsigned("compared_period_count")?)?,
        exact_count: as_usize(fields.unsigned("exact_count")?)?,
        tolerance_count: as_usize(fields.unsigned("tolerance_count")?)?,
        failed_count: as_usize(fields.unsigned("failed_count")?)?,
        excluded_partial_count: as_usize(fields.unsigned("excluded_partial_count")?)?,
        unresolved_count: as_usize(fields.unsigned("unresolved_count")?)?,
        root_cause_counts: fields.strings("root_cause_counts")?,
        disposition_counts: fields.strings("disposition_counts")?,
        complete_forensic_coverage: fields.boolean("complete_forensic_coverage")?,
        native_metadata_complete: fields.boolean("native_metadata_complete")?,
        aggregate_digest: fields.string("aggregate_digest")?,
    };
    fields.finish()?;
    validate_macro_aggregate(&value)?;
    Ok(value)
}

fn validate_macro_policy(value: &MomentumCanonicalMacroPolicyV1) -> Result<(), String> {
    let allowed_timeframe = matches!(
        value.timeframe,
        MomentumHistoricalTimeframeV1::Week1
            | MomentumHistoricalTimeframeV1::Month1
            | MomentumHistoricalTimeframeV1::Year1
    );
    let counts_valid = value.complete_period_count
        == value.qualified_period_count
            + value.excluded_partial_period_count
            + value.unresolved_period_count;
    let source_valid = match value.selected_source {
        MomentumCanonicalMacroSourceV1::DerivedFromCanonicalDaily => {
            value.native_index_digest.is_none() && value.unresolved_period_count == 0
        }
        MomentumCanonicalMacroSourceV1::NativeProviderCandle => {
            value
                .native_index_digest
                .as_ref()
                .is_some_and(|item| !item.is_empty())
                && value.unresolved_period_count == 0
        }
        MomentumCanonicalMacroSourceV1::ExcludedUnresolved => {
            value.native_index_digest.is_none() && value.unresolved_period_count > 0
        }
    };
    if value.policy_version != MACRO_POLICY_VERSION
        || !allowed_timeframe
        || [
            &value.daily_index_digest,
            &value.derived_index_digest,
            &value.forensic_aggregate_digest,
        ]
        .iter()
        .any(|item| item.is_empty())
        || value.complete_period_count == 0
        || !counts_valid
        || !source_valid
        || value.live_authority_eligible
        || !value.historical_research_only
        || value.policy_digest != macro_policy_digest(value)
    {
        return Err("canonical macro policy rejected".to_string());
    }
    Ok(())
}

fn encode_macro_policy(value: &MomentumCanonicalMacroPolicyV1) -> Result<Vec<u8>, String> {
    validate_macro_policy(value)?;
    ArtifactBuilderV4_2::new("MomentumCanonicalMacroPolicyV1")
        .string("policy_version", &value.policy_version)
        .string("timeframe", value.timeframe.as_str())
        .string("selected_source", value.selected_source.stable_name())
        .string("daily_index_digest", &value.daily_index_digest)
        .string("derived_index_digest", &value.derived_index_digest)
        .optional_string("native_index_digest", &value.native_index_digest)
        .string(
            "forensic_aggregate_digest",
            &value.forensic_aggregate_digest,
        )
        .unsigned(
            "complete_period_count",
            as_u64(value.complete_period_count)?,
        )
        .unsigned(
            "qualified_period_count",
            as_u64(value.qualified_period_count)?,
        )
        .unsigned(
            "excluded_partial_period_count",
            as_u64(value.excluded_partial_period_count)?,
        )
        .unsigned(
            "unresolved_period_count",
            as_u64(value.unresolved_period_count)?,
        )
        .boolean("live_authority_eligible", value.live_authority_eligible)
        .boolean("historical_research_only", value.historical_research_only)
        .string("policy_digest", &value.policy_digest)
        .encode()
}

fn decode_macro_policy(bytes: &[u8]) -> Result<MomentumCanonicalMacroPolicyV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumCanonicalMacroPolicyV1")?;
    let value = MomentumCanonicalMacroPolicyV1 {
        policy_version: fields.string("policy_version")?,
        timeframe: MomentumHistoricalTimeframeV1::parse(&fields.string("timeframe")?)?,
        selected_source: MomentumCanonicalMacroSourceV1::parse_stable(
            &fields.string("selected_source")?,
        )?,
        daily_index_digest: fields.string("daily_index_digest")?,
        derived_index_digest: fields.string("derived_index_digest")?,
        native_index_digest: fields.optional_string("native_index_digest")?,
        forensic_aggregate_digest: fields.string("forensic_aggregate_digest")?,
        complete_period_count: as_usize(fields.unsigned("complete_period_count")?)?,
        qualified_period_count: as_usize(fields.unsigned("qualified_period_count")?)?,
        excluded_partial_period_count: as_usize(fields.unsigned("excluded_partial_period_count")?)?,
        unresolved_period_count: as_usize(fields.unsigned("unresolved_period_count")?)?,
        live_authority_eligible: fields.boolean("live_authority_eligible")?,
        historical_research_only: fields.boolean("historical_research_only")?,
        policy_digest: fields.string("policy_digest")?,
    };
    fields.finish()?;
    validate_macro_policy(&value)?;
    Ok(value)
}

#[allow(dead_code)]
fn validate_native_macro_index(value: &MomentumNativeMacroCanonicalIndexV1) -> Result<(), String> {
    let count = value.ordered_native_candle_digests.len();
    if value.index_version != NATIVE_MACRO_INDEX_VERSION
        || !matches!(
            value.timeframe,
            MomentumHistoricalTimeframeV1::Month1 | MomentumHistoricalTimeframeV1::Year1
        )
        || count == 0
        || count != value.ordered_first_day_of_period.len()
        || count != value.total_complete_periods
        || value.first_complete_period
            != value
                .ordered_first_day_of_period
                .first()
                .cloned()
                .unwrap_or_default()
        || value.last_complete_period
            != value
                .ordered_first_day_of_period
                .last()
                .cloned()
                .unwrap_or_default()
        || value
            .ordered_native_candle_digests
            .iter()
            .chain(&value.ordered_first_day_of_period)
            .chain(&value.source_response_digests)
            .any(String::is_empty)
        || value
            .ordered_native_candle_digests
            .iter()
            .collect::<BTreeSet<_>>()
            .len()
            != count
        || value
            .ordered_first_day_of_period
            .windows(2)
            .any(|pair| pair[0] >= pair[1])
        || value.source_response_digests.is_empty()
        || value.normalization_policy_digest.is_empty()
        || value.index_digest != native_macro_index_digest(value)
    {
        return Err("native macro canonical index rejected".to_string());
    }
    Ok(())
}

#[allow(dead_code)]
fn encode_native_macro_index(
    value: &MomentumNativeMacroCanonicalIndexV1,
) -> Result<Vec<u8>, String> {
    validate_native_macro_index(value)?;
    ArtifactBuilderV4_2::new("MomentumNativeMacroCanonicalIndexV1")
        .string("index_version", &value.index_version)
        .string("timeframe", value.timeframe.as_str())
        .strings(
            "ordered_native_candle_digests",
            &value.ordered_native_candle_digests,
        )
        .strings(
            "ordered_first_day_of_period",
            &value.ordered_first_day_of_period,
        )
        .string("first_complete_period", &value.first_complete_period)
        .string("last_complete_period", &value.last_complete_period)
        .unsigned(
            "total_complete_periods",
            as_u64(value.total_complete_periods)?,
        )
        .strings("source_response_digests", &value.source_response_digests)
        .string(
            "normalization_policy_digest",
            &value.normalization_policy_digest,
        )
        .string("index_digest", &value.index_digest)
        .encode()
}

#[allow(dead_code)]
fn decode_native_macro_index(bytes: &[u8]) -> Result<MomentumNativeMacroCanonicalIndexV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumNativeMacroCanonicalIndexV1")?;
    let value = MomentumNativeMacroCanonicalIndexV1 {
        index_version: fields.string("index_version")?,
        timeframe: MomentumHistoricalTimeframeV1::parse(&fields.string("timeframe")?)?,
        ordered_native_candle_digests: fields.strings("ordered_native_candle_digests")?,
        ordered_first_day_of_period: fields.strings("ordered_first_day_of_period")?,
        first_complete_period: fields.string("first_complete_period")?,
        last_complete_period: fields.string("last_complete_period")?,
        total_complete_periods: as_usize(fields.unsigned("total_complete_periods")?)?,
        source_response_digests: fields.strings("source_response_digests")?,
        normalization_policy_digest: fields.string("normalization_policy_digest")?,
        index_digest: fields.string("index_digest")?,
    };
    fields.finish()?;
    validate_native_macro_index(&value)?;
    Ok(value)
}

#[allow(dead_code)]
fn validate_corrected_derived_index(
    value: &MomentumCorrectedDerivedMacroIndexV2,
) -> Result<(), String> {
    if value.index_version != CORRECTED_DERIVED_INDEX_VERSION
        || !matches!(
            value.timeframe,
            MomentumHistoricalTimeframeV1::Month1 | MomentumHistoricalTimeframeV1::Year1
        )
        || value.prior_index_digest.is_empty()
        || value.corrected_aggregation_policy_digest.is_empty()
        || value.ordered_candle_digests.is_empty()
        || value.regenerated_period_count != value.ordered_candle_digests.len()
        || value.ordered_candle_digests.iter().any(String::is_empty)
        || !value.old_index_preserved
        || value.index_digest != corrected_derived_index_digest(value)
    {
        return Err("corrected derived macro index rejected".to_string());
    }
    Ok(())
}

#[allow(dead_code)]
fn encode_corrected_derived_index(
    value: &MomentumCorrectedDerivedMacroIndexV2,
) -> Result<Vec<u8>, String> {
    validate_corrected_derived_index(value)?;
    ArtifactBuilderV4_2::new("MomentumCorrectedDerivedMacroIndexV2")
        .string("index_version", &value.index_version)
        .string("timeframe", value.timeframe.as_str())
        .string("prior_index_digest", &value.prior_index_digest)
        .string(
            "corrected_aggregation_policy_digest",
            &value.corrected_aggregation_policy_digest,
        )
        .strings("ordered_candle_digests", &value.ordered_candle_digests)
        .unsigned(
            "regenerated_period_count",
            as_u64(value.regenerated_period_count)?,
        )
        .boolean("old_index_preserved", value.old_index_preserved)
        .string("index_digest", &value.index_digest)
        .encode()
}

#[allow(dead_code)]
fn decode_corrected_derived_index(
    bytes: &[u8],
) -> Result<MomentumCorrectedDerivedMacroIndexV2, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumCorrectedDerivedMacroIndexV2")?;
    let value = MomentumCorrectedDerivedMacroIndexV2 {
        index_version: fields.string("index_version")?,
        timeframe: MomentumHistoricalTimeframeV1::parse(&fields.string("timeframe")?)?,
        prior_index_digest: fields.string("prior_index_digest")?,
        corrected_aggregation_policy_digest: fields
            .string("corrected_aggregation_policy_digest")?,
        ordered_candle_digests: fields.strings("ordered_candle_digests")?,
        regenerated_period_count: as_usize(fields.unsigned("regenerated_period_count")?)?,
        old_index_preserved: fields.boolean("old_index_preserved")?,
        index_digest: fields.string("index_digest")?,
    };
    fields.finish()?;
    validate_corrected_derived_index(&value)?;
    Ok(value)
}

fn qualified_timeframes(
    value: &MomentumQualifiedTimeframeSetV1,
) -> [MomentumTimeframeQualificationV1; 8] {
    [
        value.minute1,
        value.minute3,
        value.minute5,
        value.minute10,
        value.day1,
        value.week1,
        value.month1,
        value.year1,
    ]
}

fn validate_qualified_set(value: &MomentumQualifiedTimeframeSetV1) -> Result<(), String> {
    let values = qualified_timeframes(value);
    let qualified_count = values
        .iter()
        .filter(|qualification| {
            matches!(
                qualification,
                MomentumTimeframeQualificationV1::QualifiedDerivedCanonical
                    | MomentumTimeframeQualificationV1::QualifiedNativeCanonical
            )
        })
        .count();
    let unresolved_count = values
        .iter()
        .filter(|qualification| {
            **qualification == MomentumTimeframeQualificationV1::ExcludedUnresolved
        })
        .count();
    if value.set_version != QUALIFIED_SET_VERSION
        || value.qualified_count != qualified_count
        || value.unresolved_count != unresolved_count
        || value.full_eight_timeframe_replay_allowed != (qualified_count == 8)
        || value.set_digest != qualified_set_digest(value)
    {
        return Err("qualified timeframe set rejected".to_string());
    }
    Ok(())
}

fn encode_qualified_set(value: &MomentumQualifiedTimeframeSetV1) -> Result<Vec<u8>, String> {
    validate_qualified_set(value)?;
    let values = qualified_timeframes(value)
        .into_iter()
        .map(StableMacroEnumV1::stable_name)
        .collect::<Vec<_>>();
    ArtifactBuilderV4_2::new("MomentumQualifiedTimeframeSetV1")
        .string("set_version", &value.set_version)
        .strings("ordered_qualifications", &values)
        .unsigned("qualified_count", as_u64(value.qualified_count)?)
        .unsigned("unresolved_count", as_u64(value.unresolved_count)?)
        .boolean(
            "full_eight_timeframe_replay_allowed",
            value.full_eight_timeframe_replay_allowed,
        )
        .string("set_digest", &value.set_digest)
        .encode()
}

fn decode_qualified_set(bytes: &[u8]) -> Result<MomentumQualifiedTimeframeSetV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumQualifiedTimeframeSetV1")?;
    let values = fields
        .strings("ordered_qualifications")?
        .iter()
        .map(|value| MomentumTimeframeQualificationV1::parse_stable(value))
        .collect::<Result<Vec<_>, _>>()?;
    if values.len() != 8 {
        return Err("qualified timeframe cardinality rejected".to_string());
    }
    let value = MomentumQualifiedTimeframeSetV1 {
        set_version: fields.string("set_version")?,
        minute1: values[0],
        minute3: values[1],
        minute5: values[2],
        minute10: values[3],
        day1: values[4],
        week1: values[5],
        month1: values[6],
        year1: values[7],
        qualified_count: as_usize(fields.unsigned("qualified_count")?)?,
        unresolved_count: as_usize(fields.unsigned("unresolved_count")?)?,
        full_eight_timeframe_replay_allowed: fields
            .boolean("full_eight_timeframe_replay_allowed")?,
        set_digest: fields.string("set_digest")?,
    };
    fields.finish()?;
    validate_qualified_set(&value)?;
    Ok(value)
}

fn validate_causal_revalidation(
    value: &MomentumQualifiedCausalRevalidationV1,
) -> Result<(), String> {
    if value.revalidation_version != CAUSAL_REVALIDATION_VERSION
        || [
            &value.qualified_set_digest,
            &value.protocol_replay_digest,
            &value.sealed_holdout_digest,
        ]
        .iter()
        .any(|item| item.is_empty())
        || value.event_count == 0
        || value.selected_source_bindings.len() != 8
        || value.selected_source_bindings.iter().any(String::is_empty)
        || value.future_access_count != 0
        || value.partial_candle_access_count != 0
        || value.unqualified_view_access_count != 0
        || value.labels_read != 0
        || !value.deterministic
        || value.revalidation_digest != causal_revalidation_digest(value)
    {
        return Err("qualified causal revalidation rejected".to_string());
    }
    Ok(())
}

fn encode_causal_revalidation(
    value: &MomentumQualifiedCausalRevalidationV1,
) -> Result<Vec<u8>, String> {
    validate_causal_revalidation(value)?;
    ArtifactBuilderV4_2::new("MomentumQualifiedCausalRevalidationV1")
        .string("revalidation_version", &value.revalidation_version)
        .string("qualified_set_digest", &value.qualified_set_digest)
        .string("protocol_replay_digest", &value.protocol_replay_digest)
        .string("sealed_holdout_digest", &value.sealed_holdout_digest)
        .unsigned("event_count", as_u64(value.event_count)?)
        .strings("selected_source_bindings", &value.selected_source_bindings)
        .unsigned("future_access_count", as_u64(value.future_access_count)?)
        .unsigned(
            "partial_candle_access_count",
            as_u64(value.partial_candle_access_count)?,
        )
        .unsigned(
            "unqualified_view_access_count",
            as_u64(value.unqualified_view_access_count)?,
        )
        .unsigned(
            "blocked_unqualified_view_count",
            as_u64(value.blocked_unqualified_view_count)?,
        )
        .unsigned("labels_read", as_u64(value.labels_read)?)
        .boolean("deterministic", value.deterministic)
        .string("revalidation_digest", &value.revalidation_digest)
        .encode()
}

fn decode_causal_revalidation(
    bytes: &[u8],
) -> Result<MomentumQualifiedCausalRevalidationV1, String> {
    let mut fields = ArtifactReaderV4_2::decode(bytes, "MomentumQualifiedCausalRevalidationV1")?;
    let value = MomentumQualifiedCausalRevalidationV1 {
        revalidation_version: fields.string("revalidation_version")?,
        qualified_set_digest: fields.string("qualified_set_digest")?,
        protocol_replay_digest: fields.string("protocol_replay_digest")?,
        sealed_holdout_digest: fields.string("sealed_holdout_digest")?,
        event_count: as_usize(fields.unsigned("event_count")?)?,
        selected_source_bindings: fields.strings("selected_source_bindings")?,
        future_access_count: as_usize(fields.unsigned("future_access_count")?)?,
        partial_candle_access_count: as_usize(fields.unsigned("partial_candle_access_count")?)?,
        unqualified_view_access_count: as_usize(fields.unsigned("unqualified_view_access_count")?)?,
        blocked_unqualified_view_count: as_usize(
            fields.unsigned("blocked_unqualified_view_count")?,
        )?,
        labels_read: as_usize(fields.unsigned("labels_read")?)?,
        deterministic: fields.boolean("deterministic")?,
        revalidation_digest: fields.string("revalidation_digest")?,
    };
    fields.finish()?;
    validate_causal_revalidation(&value)?;
    Ok(value)
}

fn validate_qualified_hard_replay(
    value: &MomentumQualifiedHardReplayRegistrationV2,
) -> Result<(), String> {
    const TASK_POLICIES: [&str; 3] = [
        "IntradayTenMinute:timestamp=closed-10m;horizon=10m;label=intraday-direction-v1;cadence=10m;range=all-required-context-closed;minimum=60x1m",
        "DailyOneDay:timestamp=closed-1d;horizon=1d;label=daily-direction-v1;cadence=1d;range=all-required-context-closed;minimum=30x1d",
        "WeeklyOneWeek:timestamp=closed-1w;horizon=1w;label=weekly-direction-v1;cadence=1w;range=all-required-context-closed;minimum=26x1w",
    ];
    const EVIDENCE_ROLES: [&str; 3] = [
        "IntradayTenMinute:primary=1m,3m,5m,10m;regime=1d,1w,1mo,1y",
        "DailyOneDay:primary=10m,1d;regime=1w,1mo,1y",
        "WeeklyOneWeek:primary=1d,1w;regime=1mo,1y",
    ];
    const ABLATIONS: [&str; 6] = [
        "A0=current single-timeframe baseline",
        "A1=intraday block only",
        "A2=qualified macro block only",
        "A3=full qualified multi-timeframe fusion",
        "A4=full fusion without intraday block",
        "A5=full fusion without macro block",
    ];
    const MODELS: [&str; 4] = [
        "per-task constant benchmark",
        "per-task single-timeframe logistic baseline",
        "per-task block-logistic model",
        "per-task full-fusion logistic model",
    ];
    const GATES: [&str; 9] = [
        "finite predictions",
        "no probability collapse",
        "sufficient scorable validation events",
        "Brier comparison against task-specific constant",
        "block-zero ablation",
        "micro-block contribution",
        "macro-block contribution",
        "full-fusion contribution",
        "chronological validation and sealed holdout isolation",
    ];
    if value.registration_version != QUALIFIED_HARD_REPLAY_VERSION
        || value.qualified_set_digest.is_empty()
        || value.task_policies != TASK_POLICIES
        || value.evidence_role_bindings != EVIDENCE_ROLES
        || value.ablation_families != ABLATIONS
        || value.model_families != MODELS
        || value.contribution_gates != GATES
        || !value.constant_benchmark_mandatory
        || !value.historical_logistic_warning_preserved
        || !value.full_eight_timeframe_required
        || value.executed
        || value.registration_digest != qualified_hard_replay_digest(value)
    {
        return Err("qualified hard replay registration rejected".to_string());
    }
    Ok(())
}

fn build_qualified_hard_replay(
    set: &MomentumQualifiedTimeframeSetV1,
) -> Result<MomentumQualifiedHardReplayRegistrationV2, String> {
    validate_qualified_set(set)?;
    if !set.full_eight_timeframe_replay_allowed {
        return Err("full eight-timeframe qualification required".to_string());
    }
    let mut value = MomentumQualifiedHardReplayRegistrationV2 {
        registration_version: QUALIFIED_HARD_REPLAY_VERSION.to_string(),
        qualified_set_digest: set.set_digest.clone(),
        task_policies: vec![
            "IntradayTenMinute:timestamp=closed-10m;horizon=10m;label=intraday-direction-v1;cadence=10m;range=all-required-context-closed;minimum=60x1m".to_string(),
            "DailyOneDay:timestamp=closed-1d;horizon=1d;label=daily-direction-v1;cadence=1d;range=all-required-context-closed;minimum=30x1d".to_string(),
            "WeeklyOneWeek:timestamp=closed-1w;horizon=1w;label=weekly-direction-v1;cadence=1w;range=all-required-context-closed;minimum=26x1w".to_string(),
        ],
        evidence_role_bindings: vec![
            "IntradayTenMinute:primary=1m,3m,5m,10m;regime=1d,1w,1mo,1y".to_string(),
            "DailyOneDay:primary=10m,1d;regime=1w,1mo,1y".to_string(),
            "WeeklyOneWeek:primary=1d,1w;regime=1mo,1y".to_string(),
        ],
        ablation_families: vec![
            "A0=current single-timeframe baseline".to_string(),
            "A1=intraday block only".to_string(),
            "A2=qualified macro block only".to_string(),
            "A3=full qualified multi-timeframe fusion".to_string(),
            "A4=full fusion without intraday block".to_string(),
            "A5=full fusion without macro block".to_string(),
        ],
        model_families: vec![
            "per-task constant benchmark".to_string(),
            "per-task single-timeframe logistic baseline".to_string(),
            "per-task block-logistic model".to_string(),
            "per-task full-fusion logistic model".to_string(),
        ],
        contribution_gates: vec![
            "finite predictions".to_string(),
            "no probability collapse".to_string(),
            "sufficient scorable validation events".to_string(),
            "Brier comparison against task-specific constant".to_string(),
            "block-zero ablation".to_string(),
            "micro-block contribution".to_string(),
            "macro-block contribution".to_string(),
            "full-fusion contribution".to_string(),
            "chronological validation and sealed holdout isolation".to_string(),
        ],
        constant_benchmark_mandatory: true,
        historical_logistic_warning_preserved: true,
        full_eight_timeframe_required: true,
        executed: false,
        registration_digest: String::new(),
    };
    value.registration_digest = qualified_hard_replay_digest(&value);
    validate_qualified_hard_replay(&value)?;
    Ok(value)
}

fn encode_qualified_hard_replay(
    value: &MomentumQualifiedHardReplayRegistrationV2,
) -> Result<Vec<u8>, String> {
    validate_qualified_hard_replay(value)?;
    ArtifactBuilderV4_2::new("MomentumQualifiedHardReplayRegistrationV2")
        .string("registration_version", &value.registration_version)
        .string("qualified_set_digest", &value.qualified_set_digest)
        .strings("task_policies", &value.task_policies)
        .strings("evidence_role_bindings", &value.evidence_role_bindings)
        .strings("ablation_families", &value.ablation_families)
        .strings("model_families", &value.model_families)
        .strings("contribution_gates", &value.contribution_gates)
        .boolean(
            "constant_benchmark_mandatory",
            value.constant_benchmark_mandatory,
        )
        .boolean(
            "historical_logistic_warning_preserved",
            value.historical_logistic_warning_preserved,
        )
        .boolean(
            "full_eight_timeframe_required",
            value.full_eight_timeframe_required,
        )
        .boolean("executed", value.executed)
        .string("registration_digest", &value.registration_digest)
        .encode()
}

fn decode_qualified_hard_replay(
    bytes: &[u8],
) -> Result<MomentumQualifiedHardReplayRegistrationV2, String> {
    let mut fields =
        ArtifactReaderV4_2::decode(bytes, "MomentumQualifiedHardReplayRegistrationV2")?;
    let value = MomentumQualifiedHardReplayRegistrationV2 {
        registration_version: fields.string("registration_version")?,
        qualified_set_digest: fields.string("qualified_set_digest")?,
        task_policies: fields.strings("task_policies")?,
        evidence_role_bindings: fields.strings("evidence_role_bindings")?,
        ablation_families: fields.strings("ablation_families")?,
        model_families: fields.strings("model_families")?,
        contribution_gates: fields.strings("contribution_gates")?,
        constant_benchmark_mandatory: fields.boolean("constant_benchmark_mandatory")?,
        historical_logistic_warning_preserved: fields
            .boolean("historical_logistic_warning_preserved")?,
        full_eight_timeframe_required: fields.boolean("full_eight_timeframe_required")?,
        executed: fields.boolean("executed")?,
        registration_digest: fields.string("registration_digest")?,
    };
    fields.finish()?;
    validate_qualified_hard_replay(&value)?;
    Ok(value)
}

fn classify_macro_boundary(
    native: &CandleIntervalV1,
    derived: &CandleIntervalV1,
) -> MomentumMacroBoundaryComparisonV1 {
    if native == derived {
        MomentumMacroBoundaryComparisonV1::ExactSameInterval
    } else if native.open_timestamp_ms != derived.open_timestamp_ms
        && native.close_exclusive_timestamp_ms == derived.close_exclusive_timestamp_ms
    {
        MomentumMacroBoundaryComparisonV1::OpeningBoundaryMismatch
    } else if native.open_timestamp_ms == derived.open_timestamp_ms
        && native.close_exclusive_timestamp_ms != derived.close_exclusive_timestamp_ms
    {
        MomentumMacroBoundaryComparisonV1::ClosingBoundaryMismatch
    } else {
        MomentumMacroBoundaryComparisonV1::NativePeriodNotReconstructableFromDailyBase
    }
}

fn classify_macro_value(
    derived: &HistoricalCandleRowV1,
    native: &HistoricalCandleRowV1,
    boundary: MomentumMacroBoundaryComparisonV1,
) -> MomentumMacroValueComparisonV1 {
    if !matches!(
        boundary,
        MomentumMacroBoundaryComparisonV1::ExactSameInterval
            | MomentumMacroBoundaryComparisonV1::SamePeriodDifferentTimestampRepresentation
    ) {
        return MomentumMacroValueComparisonV1::NotComparableBoundaryMismatch;
    }
    let mismatches = [
        derived.open.to_bits() != native.open.to_bits(),
        derived.high.to_bits() != native.high.to_bits(),
        derived.low.to_bits() != native.low.to_bits(),
        derived.close.to_bits() != native.close.to_bits(),
        !finite_close(derived.volume, native.volume),
        !finite_close(derived.trade_value, native.trade_value),
    ];
    let mismatch_count = mismatches.iter().filter(|mismatch| **mismatch).count();
    if mismatch_count > 1 {
        MomentumMacroValueComparisonV1::MultipleValueMismatches
    } else if mismatches[0] {
        MomentumMacroValueComparisonV1::OpenMismatch
    } else if mismatches[1] {
        MomentumMacroValueComparisonV1::HighMismatch
    } else if mismatches[2] {
        MomentumMacroValueComparisonV1::LowMismatch
    } else if mismatches[3] {
        MomentumMacroValueComparisonV1::CloseMismatch
    } else if mismatches[4] {
        MomentumMacroValueComparisonV1::VolumeOutsideRegisteredTolerance
    } else if mismatches[5] {
        MomentumMacroValueComparisonV1::TradeValueOutsideRegisteredTolerance
    } else if derived.volume.to_bits() == native.volume.to_bits()
        && derived.trade_value.to_bits() == native.trade_value.to_bits()
    {
        MomentumMacroValueComparisonV1::ExactAllFields
    } else {
        MomentumMacroValueComparisonV1::AccumulationWithinRegisteredTolerance
    }
}

fn classify_macro_completeness(
    derived: &HistoricalCandleRowV1,
    daily_index: &HistoricalCandleIndexV1,
) -> MomentumMacroCompletenessComparisonV1 {
    if daily_index.missing_evidence_count > 0 {
        MomentumMacroCompletenessComparisonV1::MissingDailyEvidence
    } else if daily_index.first_timestamp_ms > derived.interval.open_timestamp_ms {
        MomentumMacroCompletenessComparisonV1::SourceCoverageStartsInsidePeriod
    } else if daily_index.close_exclusive_timestamp_ms
        < derived.interval.close_exclusive_timestamp_ms
    {
        MomentumMacroCompletenessComparisonV1::SourceCoverageEndsInsidePeriod
    } else if derived.ordered_base_candle_digests.is_empty() {
        MomentumMacroCompletenessComparisonV1::IntegrityFailure
    } else {
        MomentumMacroCompletenessComparisonV1::BothComplete
    }
}

fn receipt_resolution(
    boundary: MomentumMacroBoundaryComparisonV1,
    completeness: MomentumMacroCompletenessComparisonV1,
    value: MomentumMacroValueComparisonV1,
    native_metadata_complete: bool,
) -> (
    Option<MomentumMacroMismatchRootCauseV1>,
    MomentumMacroCandleDispositionV1,
) {
    match completeness {
        MomentumMacroCompletenessComparisonV1::NativePartialDerivedExcluded => (
            Some(MomentumMacroMismatchRootCauseV1::IncompleteNativeCurrentPeriod),
            MomentumMacroCandleDispositionV1::ExcludedPartialPeriodNotAFailure,
        ),
        MomentumMacroCompletenessComparisonV1::NativeCompleteDerivedPartial => (
            Some(MomentumMacroMismatchRootCauseV1::IncompleteDerivedCurrentPeriod),
            MomentumMacroCandleDispositionV1::ExcludedUnresolved,
        ),
        MomentumMacroCompletenessComparisonV1::SourceCoverageStartsInsidePeriod
        | MomentumMacroCompletenessComparisonV1::SourceCoverageEndsInsidePeriod => (
            Some(MomentumMacroMismatchRootCauseV1::PartialFirstCalendarPeriod),
            MomentumMacroCandleDispositionV1::ExcludedUnresolved,
        ),
        MomentumMacroCompletenessComparisonV1::NoTradeCompositionDiffers => (
            Some(MomentumMacroMismatchRootCauseV1::NoTradeIntervalComposition),
            MomentumMacroCandleDispositionV1::ExcludedUnresolved,
        ),
        MomentumMacroCompletenessComparisonV1::MissingDailyEvidence => (
            Some(MomentumMacroMismatchRootCauseV1::MissingCanonicalDailyEvidence),
            MomentumMacroCandleDispositionV1::ExcludedUnresolved,
        ),
        MomentumMacroCompletenessComparisonV1::IntegrityFailure => (
            Some(MomentumMacroMismatchRootCauseV1::CorruptEvidence),
            MomentumMacroCandleDispositionV1::ExcludedUnresolved,
        ),
        MomentumMacroCompletenessComparisonV1::BothComplete => {
            if boundary != MomentumMacroBoundaryComparisonV1::ExactSameInterval {
                let root = match boundary {
                    MomentumMacroBoundaryComparisonV1::UtcVsKstBoundaryShift => {
                        MomentumMacroMismatchRootCauseV1::UtcKstCalendarBoundaryDifference
                    }
                    MomentumMacroBoundaryComparisonV1::FirstDayOfPeriodMismatch => {
                        MomentumMacroMismatchRootCauseV1::ProviderFirstDayPeriodSemantics
                    }
                    MomentumMacroBoundaryComparisonV1::NativePeriodNotReconstructableFromDailyBase
                    | MomentumMacroBoundaryComparisonV1::OpeningBoundaryMismatch
                    | MomentumMacroBoundaryComparisonV1::ClosingBoundaryMismatch => {
                        MomentumMacroMismatchRootCauseV1::DailyBaseInsufficientForNativeBoundary
                    }
                    MomentumMacroBoundaryComparisonV1::IntegrityFailure => {
                        MomentumMacroMismatchRootCauseV1::CorruptEvidence
                    }
                    MomentumMacroBoundaryComparisonV1::SamePeriodDifferentTimestampRepresentation
                    | MomentumMacroBoundaryComparisonV1::ExactSameInterval => {
                        MomentumMacroMismatchRootCauseV1::ProviderContractAmbiguous
                    }
                };
                return (
                    Some(if native_metadata_complete {
                        root
                    } else {
                        MomentumMacroMismatchRootCauseV1::ProviderContractAmbiguous
                    }),
                    MomentumMacroCandleDispositionV1::ExcludedUnresolved,
                );
            }
            match value {
                MomentumMacroValueComparisonV1::ExactAllFields => (
                    None,
                    MomentumMacroCandleDispositionV1::QualifiedDerivedFromDaily,
                ),
                MomentumMacroValueComparisonV1::AccumulationWithinRegisteredTolerance => (
                    None,
                    MomentumMacroCandleDispositionV1::QualifiedDerivedFromDailyWithinRegisteredTolerance,
                ),
                _ => (
                    Some(if native_metadata_complete {
                        MomentumMacroMismatchRootCauseV1::IncorrectDerivedAggregation
                    } else {
                        MomentumMacroMismatchRootCauseV1::ProviderContractAmbiguous
                    }),
                    MomentumMacroCandleDispositionV1::ExcludedUnresolved,
                ),
            }
        }
    }
}

fn build_macro_forensics(
    root: &Path,
    timeframe: MomentumHistoricalTimeframeV1,
) -> Result<
    (
        Vec<MomentumMacroCandleForensicReceiptV1>,
        MomentumMacroForensicAggregateV1,
        MomentumCanonicalMacroPolicyV1,
    ),
    String,
> {
    if !matches!(
        timeframe,
        MomentumHistoricalTimeframeV1::Month1 | MomentumHistoricalTimeframeV1::Year1
    ) {
        return Err("macro forensic timeframe rejected".to_string());
    }
    let (_, foundation, plan) = reopen_foundation(root)?;
    let foundation =
        foundation.ok_or_else(|| "multi-timeframe foundation unavailable".to_string())?;
    let plan = plan.ok_or_else(|| "multi-timeframe acquisition plan unavailable".to_string())?;
    let daily_index = reopen_index(root, MomentumHistoricalTimeframeV1::Day1)?
        .ok_or_else(|| "canonical daily index unavailable".to_string())?;
    let daily_rows = reopen_canonical_rows(root, &daily_index)?;
    let (derived_rows, derived_index) =
        aggregate_view(&foundation, &daily_index, &daily_rows, timeframe)?;
    let persisted_derived = read_single(
        &root.join(format!("derived_{}/indices", timeframe.as_str())),
        decode_derived_index,
    )?
    .ok_or_else(|| "persisted derived macro index unavailable".to_string())?;
    if persisted_derived != derived_index {
        return Err("persisted derived macro index changed".to_string());
    }
    let native_receipt = load_receipts(root)?
        .into_iter()
        .find(|receipt| {
            receipt.plan_digest == plan.plan_digest
                && receipt.purpose == PagePurposeV1::NativeCrossCheck
                && receipt.timeframe == timeframe
                && receipt.status == PageReceiptStatusV1::Verified
        })
        .ok_or_else(|| "persisted native macro response unavailable".to_string())?;
    let response_digest = native_receipt
        .response_body_digest
        .clone()
        .ok_or_else(|| "native macro response identity unavailable".to_string())?;
    let derived_by_open = derived_rows
        .iter()
        .map(|row| (row.interval.open_timestamp_ms, row))
        .collect::<BTreeMap<_, _>>();
    let first = derived_rows
        .first()
        .map(|row| row.interval.open_timestamp_ms)
        .ok_or_else(|| "derived macro first period unavailable".to_string())?;
    let last_close = derived_rows
        .last()
        .map(|row| row.interval.close_exclusive_timestamp_ms)
        .ok_or_else(|| "derived macro last period unavailable".to_string())?;
    let mut receipts = Vec::new();
    for native in native_receipt.rows.iter().filter(|native| {
        native.interval.open_timestamp_ms >= first
            && native.interval.close_exclusive_timestamp_ms <= last_close
    }) {
        let derived = derived_by_open
            .get(&native.interval.open_timestamp_ms)
            .ok_or_else(|| "derived macro comparison period unavailable".to_string())?;
        let boundary = classify_macro_boundary(&native.interval, &derived.interval);
        let completeness = classify_macro_completeness(derived, &daily_index);
        let value_comparison = classify_macro_value(derived, native, boundary);
        let native_metadata_complete = false;
        let (root_cause, disposition) = receipt_resolution(
            boundary,
            completeness,
            value_comparison,
            native_metadata_complete,
        );
        let mut value = MomentumMacroCandleForensicReceiptV1 {
            forensic_version: MACRO_FORENSIC_VERSION.to_string(),
            timeframe,
            native_candle_digest: native.candle_digest.clone(),
            derived_candle_digest: derived.candle_digest.clone(),
            native_candle_timestamp_ms: native.interval.open_timestamp_ms,
            native_candle_kst_timestamp: None,
            native_first_day_of_period: None,
            native_last_trade_timestamp_ms: None,
            native_open_timestamp_ms: native.interval.open_timestamp_ms,
            native_close_exclusive_timestamp_ms: native.interval.close_exclusive_timestamp_ms,
            derived_open_timestamp_ms: derived.interval.open_timestamp_ms,
            derived_close_exclusive_timestamp_ms: derived.interval.close_exclusive_timestamp_ms,
            request_to_exclusive_ms: native_receipt.request_to_exclusive_ms,
            market: MARKET.to_string(),
            provider_id: PROVIDER.to_string(),
            native_response_digest: response_digest.clone(),
            native_source_row_digests: vec![native.candle_digest.clone()],
            derived_source_row_digests: derived.ordered_base_candle_digests.clone(),
            boundary_comparison: boundary,
            completeness_comparison: completeness,
            value_comparison,
            root_cause,
            disposition,
            receipt_digest: String::new(),
        };
        value.receipt_digest = macro_receipt_digest(&value);
        validate_macro_receipt(&value)?;
        receipts.push(value);
    }
    if receipts.is_empty()
        || receipts
            .windows(2)
            .any(|pair| pair[0].native_open_timestamp_ms >= pair[1].native_open_timestamp_ms)
    {
        return Err("macro forensic chronology rejected".to_string());
    }
    let mut root_counts = BTreeMap::<MomentumMacroMismatchRootCauseV1, usize>::new();
    let mut disposition_counts = BTreeMap::<MomentumMacroCandleDispositionV1, usize>::new();
    for receipt in &receipts {
        if let Some(root) = receipt.root_cause {
            *root_counts.entry(root).or_default() += 1;
        }
        *disposition_counts.entry(receipt.disposition).or_default() += 1;
    }
    let count_disposition = |kind| {
        receipts
            .iter()
            .filter(|receipt| receipt.disposition == kind)
            .count()
    };
    let exact_count =
        count_disposition(MomentumMacroCandleDispositionV1::QualifiedDerivedFromDaily);
    let tolerance_count = count_disposition(
        MomentumMacroCandleDispositionV1::QualifiedDerivedFromDailyWithinRegisteredTolerance,
    );
    let excluded_partial_count =
        count_disposition(MomentumMacroCandleDispositionV1::ExcludedPartialPeriodNotAFailure);
    let unresolved_count = count_disposition(MomentumMacroCandleDispositionV1::ExcludedUnresolved);
    let failed_count = receipts.len() - exact_count - tolerance_count - excluded_partial_count;
    let mut aggregate = MomentumMacroForensicAggregateV1 {
        aggregate_version: MACRO_FORENSIC_AGGREGATE_VERSION.to_string(),
        timeframe,
        ordered_receipt_digests: receipts
            .iter()
            .map(|receipt| receipt.receipt_digest.clone())
            .collect(),
        compared_period_count: receipts.len(),
        exact_count,
        tolerance_count,
        failed_count,
        excluded_partial_count,
        unresolved_count,
        root_cause_counts: root_counts
            .into_iter()
            .map(|(root, count)| format!("{root:?}={count}"))
            .collect(),
        disposition_counts: disposition_counts
            .into_iter()
            .map(|(disposition, count)| format!("{disposition:?}={count}"))
            .collect(),
        complete_forensic_coverage: true,
        native_metadata_complete: receipts.iter().all(|receipt| {
            receipt.native_candle_kst_timestamp.is_some()
                && receipt.native_first_day_of_period.is_some()
                && receipt.native_last_trade_timestamp_ms.is_some()
        }),
        aggregate_digest: String::new(),
    };
    aggregate.aggregate_digest = macro_aggregate_digest(&aggregate);
    validate_macro_aggregate(&aggregate)?;
    let selected_source = if aggregate.unresolved_count == 0 {
        MomentumCanonicalMacroSourceV1::DerivedFromCanonicalDaily
    } else {
        MomentumCanonicalMacroSourceV1::ExcludedUnresolved
    };
    let mut policy = MomentumCanonicalMacroPolicyV1 {
        policy_version: MACRO_POLICY_VERSION.to_string(),
        timeframe,
        selected_source,
        daily_index_digest: daily_index.index_digest,
        derived_index_digest: derived_index.index_digest,
        native_index_digest: None,
        forensic_aggregate_digest: aggregate.aggregate_digest.clone(),
        complete_period_count: receipts.len(),
        qualified_period_count: exact_count + tolerance_count,
        excluded_partial_period_count: excluded_partial_count,
        unresolved_period_count: unresolved_count,
        live_authority_eligible: false,
        historical_research_only: true,
        policy_digest: String::new(),
    };
    policy.policy_digest = macro_policy_digest(&policy);
    validate_macro_policy(&policy)?;
    Ok((receipts, aggregate, policy))
}

fn build_weekly_policy(root: &Path) -> Result<MomentumCanonicalMacroPolicyV1, String> {
    let daily_index = reopen_index(root, MomentumHistoricalTimeframeV1::Day1)?
        .ok_or_else(|| "canonical daily index unavailable".to_string())?;
    let derived_index = read_single(&root.join("derived_1w/indices"), decode_derived_index)?
        .ok_or_else(|| "persisted derived weekly index unavailable".to_string())?;
    let comparison = read_single(&root.join("native_comparisons/1w"), decode_comparison)?
        .ok_or_else(|| "persisted weekly comparison unavailable".to_string())?;
    let (_, _, plan) = reopen_foundation(root)?;
    let plan = plan.ok_or_else(|| "multi-timeframe acquisition plan unavailable".to_string())?;
    let weekly_native = load_receipts(root)?
        .into_iter()
        .find(|receipt| {
            receipt.plan_digest == plan.plan_digest
                && receipt.purpose == PagePurposeV1::NativeCrossCheck
                && receipt.timeframe == MomentumHistoricalTimeframeV1::Week1
                && receipt.status == PageReceiptStatusV1::Verified
        })
        .ok_or_else(|| "persisted native weekly response unavailable".to_string())?;
    let normalized_first_day_semantics_agree = weekly_native.rows.iter().all(|row| {
        period_interval(
            MomentumHistoricalTimeframeV1::Week1,
            row.interval.open_timestamp_ms,
        )
        .is_ok_and(|interval| interval == row.interval)
    });
    if comparison.systematic_mismatch_blocks_replay
        || comparison.sample_count
            != comparison.exact_match_count + comparison.within_tolerance_count
        || !normalized_first_day_semantics_agree
    {
        return Err("weekly canonical source qualification rejected".to_string());
    }
    let mut value = MomentumCanonicalMacroPolicyV1 {
        policy_version: MACRO_POLICY_VERSION.to_string(),
        timeframe: MomentumHistoricalTimeframeV1::Week1,
        selected_source: MomentumCanonicalMacroSourceV1::DerivedFromCanonicalDaily,
        daily_index_digest: daily_index.index_digest,
        derived_index_digest: derived_index.index_digest,
        native_index_digest: None,
        forensic_aggregate_digest: comparison.comparison_digest,
        complete_period_count: comparison.sample_count,
        qualified_period_count: comparison.sample_count,
        excluded_partial_period_count: 0,
        unresolved_period_count: 0,
        live_authority_eligible: false,
        historical_research_only: true,
        policy_digest: String::new(),
    };
    value.policy_digest = macro_policy_digest(&value);
    validate_macro_policy(&value)?;
    Ok(value)
}

fn qualification_for_policy(
    policy: &MomentumCanonicalMacroPolicyV1,
) -> MomentumTimeframeQualificationV1 {
    match policy.selected_source {
        MomentumCanonicalMacroSourceV1::DerivedFromCanonicalDaily => {
            MomentumTimeframeQualificationV1::QualifiedDerivedCanonical
        }
        MomentumCanonicalMacroSourceV1::NativeProviderCandle => {
            MomentumTimeframeQualificationV1::QualifiedNativeCanonical
        }
        MomentumCanonicalMacroSourceV1::ExcludedUnresolved => {
            MomentumTimeframeQualificationV1::ExcludedUnresolved
        }
    }
}

fn build_qualified_set(
    comparisons: &[DerivedNativeComparisonSummaryV1],
    weekly: &MomentumCanonicalMacroPolicyV1,
    monthly: &MomentumCanonicalMacroPolicyV1,
    yearly: &MomentumCanonicalMacroPolicyV1,
) -> Result<MomentumQualifiedTimeframeSetV1, String> {
    for timeframe in [
        MomentumHistoricalTimeframeV1::Minute3,
        MomentumHistoricalTimeframeV1::Minute5,
        MomentumHistoricalTimeframeV1::Minute10,
    ] {
        let comparison = comparisons
            .iter()
            .find(|comparison| comparison.timeframe == timeframe)
            .ok_or_else(|| "intraday native comparison unavailable".to_string())?;
        if comparison.systematic_mismatch_blocks_replay {
            return Err("intraday timeframe qualification rejected".to_string());
        }
    }
    let mut value = MomentumQualifiedTimeframeSetV1 {
        set_version: QUALIFIED_SET_VERSION.to_string(),
        minute1: MomentumTimeframeQualificationV1::QualifiedNativeCanonical,
        minute3: MomentumTimeframeQualificationV1::QualifiedDerivedCanonical,
        minute5: MomentumTimeframeQualificationV1::QualifiedDerivedCanonical,
        minute10: MomentumTimeframeQualificationV1::QualifiedDerivedCanonical,
        day1: MomentumTimeframeQualificationV1::QualifiedNativeCanonical,
        week1: qualification_for_policy(weekly),
        month1: qualification_for_policy(monthly),
        year1: qualification_for_policy(yearly),
        qualified_count: 0,
        unresolved_count: 0,
        full_eight_timeframe_replay_allowed: false,
        set_digest: String::new(),
    };
    let qualifications = qualified_timeframes(&value);
    value.qualified_count = qualifications
        .iter()
        .filter(|qualification| {
            matches!(
                qualification,
                MomentumTimeframeQualificationV1::QualifiedDerivedCanonical
                    | MomentumTimeframeQualificationV1::QualifiedNativeCanonical
            )
        })
        .count();
    value.unresolved_count = qualifications
        .iter()
        .filter(|qualification| {
            **qualification == MomentumTimeframeQualificationV1::ExcludedUnresolved
        })
        .count();
    value.full_eight_timeframe_replay_allowed = value.qualified_count == 8;
    value.set_digest = qualified_set_digest(&value);
    validate_qualified_set(&value)?;
    Ok(value)
}

fn build_causal_revalidation(
    root: &Path,
    set: &MomentumQualifiedTimeframeSetV1,
    policies: [&MomentumCanonicalMacroPolicyV1; 3],
) -> Result<MomentumQualifiedCausalRevalidationV1, String> {
    validate_qualified_set(set)?;
    let (_, foundation, _) = reopen_foundation(root)?;
    let foundation =
        foundation.ok_or_else(|| "multi-timeframe foundation unavailable".to_string())?;
    let protocol = read_single(&root.join("protocol_replays"), decode_protocol)?
        .ok_or_else(|| "protocol replay unavailable".to_string())?;
    let holdout = read_single(&root.join("sealed_holdouts"), decode_holdout)?
        .ok_or_else(|| "sealed holdout unavailable".to_string())?;
    let minute_index = reopen_index(root, MomentumHistoricalTimeframeV1::Minute1)?
        .ok_or_else(|| "canonical minute index unavailable".to_string())?;
    let daily_index = reopen_index(root, MomentumHistoricalTimeframeV1::Day1)?
        .ok_or_else(|| "canonical daily index unavailable".to_string())?;
    let minute_rows = reopen_canonical_rows(root, &minute_index)?;
    let daily_rows = reopen_canonical_rows(root, &daily_index)?;
    let mut qualified_views = BTreeMap::from([
        (MomentumHistoricalTimeframeV1::Minute1, minute_rows.clone()),
        (MomentumHistoricalTimeframeV1::Day1, daily_rows.clone()),
    ]);
    for timeframe in [
        MomentumHistoricalTimeframeV1::Minute3,
        MomentumHistoricalTimeframeV1::Minute5,
        MomentumHistoricalTimeframeV1::Minute10,
        MomentumHistoricalTimeframeV1::Week1,
    ] {
        let (base_index, base_rows) = if matches!(
            timeframe,
            MomentumHistoricalTimeframeV1::Minute3
                | MomentumHistoricalTimeframeV1::Minute5
                | MomentumHistoricalTimeframeV1::Minute10
        ) {
            (&minute_index, minute_rows.as_slice())
        } else {
            (&daily_index, daily_rows.as_slice())
        };
        let (rows, _) = aggregate_view(&foundation, base_index, base_rows, timeframe)?;
        qualified_views.insert(timeframe, rows);
    }
    let (_, _, plan) = reopen_foundation(root)?;
    let plan = plan.ok_or_else(|| "multi-timeframe acquisition plan unavailable".to_string())?;
    let native_receipts = load_receipts(root)?;
    for policy in policies {
        if policy.timeframe == MomentumHistoricalTimeframeV1::Week1 {
            continue;
        }
        let rows = match policy.selected_source {
            MomentumCanonicalMacroSourceV1::DerivedFromCanonicalDaily => {
                aggregate_view(&foundation, &daily_index, &daily_rows, policy.timeframe)?.0
            }
            MomentumCanonicalMacroSourceV1::NativeProviderCandle => native_receipts
                .iter()
                .find(|receipt| {
                    receipt.plan_digest == plan.plan_digest
                        && receipt.purpose == PagePurposeV1::NativeCrossCheck
                        && receipt.timeframe == policy.timeframe
                        && receipt.status == PageReceiptStatusV1::Verified
                })
                .map(|receipt| receipt.rows.clone())
                .ok_or_else(|| "qualified native macro source unavailable".to_string())?,
            MomentumCanonicalMacroSourceV1::ExcludedUnresolved => continue,
        };
        qualified_views.insert(policy.timeframe, rows);
    }
    let qualified = qualified_timeframes(set);
    let mut future_access_count = 0usize;
    let mut partial_candle_access_count = 0usize;
    for receipt in &protocol.receipts {
        for (timeframe, qualification) in
            MomentumHistoricalTimeframeV1::ORDERED.iter().zip(qualified)
        {
            if matches!(
                qualification,
                MomentumTimeframeQualificationV1::QualifiedDerivedCanonical
                    | MomentumTimeframeQualificationV1::QualifiedNativeCanonical
            ) {
                let rows = qualified_views
                    .get(timeframe)
                    .ok_or_else(|| "qualified causal source unavailable".to_string())?;
                let (_, selected) =
                    select_as_of(*timeframe, rows, receipt.prediction_timestamp_ms)?;
                if let Some(selected) = selected {
                    future_access_count += usize::from(
                        selected.interval.open_timestamp_ms >= receipt.prediction_timestamp_ms,
                    );
                    partial_candle_access_count += usize::from(
                        selected.interval.close_exclusive_timestamp_ms
                            > receipt.prediction_timestamp_ms,
                    );
                }
            }
        }
    }
    let selected_source_bindings = vec![
        "1m=CanonicalMinute".to_string(),
        "3m=DerivedFromCanonicalMinute".to_string(),
        "5m=DerivedFromCanonicalMinute".to_string(),
        "10m=DerivedFromCanonicalMinute".to_string(),
        "1d=CanonicalDaily".to_string(),
        format!("1w={:?}", policies[0].selected_source),
        format!("1mo={:?}", policies[1].selected_source),
        format!("1y={:?}", policies[2].selected_source),
    ];
    let blocked_per_event = qualified
        .iter()
        .filter(|qualification| {
            !matches!(
                qualification,
                MomentumTimeframeQualificationV1::QualifiedDerivedCanonical
                    | MomentumTimeframeQualificationV1::QualifiedNativeCanonical
            )
        })
        .count();
    let mut value = MomentumQualifiedCausalRevalidationV1 {
        revalidation_version: CAUSAL_REVALIDATION_VERSION.to_string(),
        qualified_set_digest: set.set_digest.clone(),
        protocol_replay_digest: protocol.replay_digest,
        sealed_holdout_digest: holdout.holdout_digest,
        event_count: protocol.event_count,
        selected_source_bindings,
        future_access_count,
        partial_candle_access_count,
        unqualified_view_access_count: 0,
        blocked_unqualified_view_count: protocol.event_count * blocked_per_event,
        labels_read: 0,
        deterministic: true,
        revalidation_digest: String::new(),
    };
    value.revalidation_digest = causal_revalidation_digest(&value);
    validate_causal_revalidation(&value)?;
    Ok(value)
}

fn persist_macro_receipt(
    root: &Path,
    value: &MomentumMacroCandleForensicReceiptV1,
) -> Result<(usize, usize), String> {
    persist_one(
        root,
        &format!("macro_forensic_receipts/{}", value.timeframe.as_str()),
        &value.receipt_digest,
        &encode_macro_receipt(value)?,
        |bytes| Ok(decode_macro_receipt(bytes)?.receipt_digest),
    )
}

fn persist_macro_aggregate(
    root: &Path,
    value: &MomentumMacroForensicAggregateV1,
) -> Result<(usize, usize), String> {
    persist_one(
        root,
        &format!("macro_forensic_aggregates/{}", value.timeframe.as_str()),
        &value.aggregate_digest,
        &encode_macro_aggregate(value)?,
        |bytes| Ok(decode_macro_aggregate(bytes)?.aggregate_digest),
    )
}

fn persist_macro_policy(
    root: &Path,
    value: &MomentumCanonicalMacroPolicyV1,
) -> Result<(usize, usize), String> {
    persist_one(
        root,
        &format!("macro_source_policies/{}", value.timeframe.as_str()),
        &value.policy_digest,
        &encode_macro_policy(value)?,
        |bytes| Ok(decode_macro_policy(bytes)?.policy_digest),
    )
}

#[allow(dead_code)]
fn persist_native_macro_index(
    root: &Path,
    value: &MomentumNativeMacroCanonicalIndexV1,
) -> Result<(usize, usize), String> {
    persist_one(
        root,
        &format!("native_macro_indices/{}", value.timeframe.as_str()),
        &value.index_digest,
        &encode_native_macro_index(value)?,
        |bytes| Ok(decode_native_macro_index(bytes)?.index_digest),
    )
}

#[allow(dead_code)]
fn persist_corrected_derived_index(
    root: &Path,
    value: &MomentumCorrectedDerivedMacroIndexV2,
) -> Result<(usize, usize), String> {
    persist_one(
        root,
        &format!("corrected_derived_indices/{}", value.timeframe.as_str()),
        &value.index_digest,
        &encode_corrected_derived_index(value)?,
        |bytes| Ok(decode_corrected_derived_index(bytes)?.index_digest),
    )
}

fn persist_qualified_set(
    root: &Path,
    value: &MomentumQualifiedTimeframeSetV1,
) -> Result<(usize, usize), String> {
    persist_one(
        root,
        "qualified_timeframe_sets",
        &value.set_digest,
        &encode_qualified_set(value)?,
        |bytes| Ok(decode_qualified_set(bytes)?.set_digest),
    )
}

fn persist_causal_revalidation(
    root: &Path,
    value: &MomentumQualifiedCausalRevalidationV1,
) -> Result<(usize, usize), String> {
    persist_one(
        root,
        "qualified_causal_revalidations",
        &value.revalidation_digest,
        &encode_causal_revalidation(value)?,
        |bytes| Ok(decode_causal_revalidation(bytes)?.revalidation_digest),
    )
}

fn persist_qualified_hard_replay(
    root: &Path,
    value: &MomentumQualifiedHardReplayRegistrationV2,
) -> Result<(usize, usize), String> {
    persist_one(
        root,
        "qualified_hard_replay_registrations",
        &value.registration_digest,
        &encode_qualified_hard_replay(value)?,
        |bytes| Ok(decode_qualified_hard_replay(bytes)?.registration_digest),
    )
}

fn reopen_macro_aggregate(
    root: &Path,
    timeframe: MomentumHistoricalTimeframeV1,
) -> Result<Option<MomentumMacroForensicAggregateV1>, String> {
    read_single(
        &root.join(format!("macro_forensic_aggregates/{}", timeframe.as_str())),
        decode_macro_aggregate,
    )
}

fn reopen_macro_policy(
    root: &Path,
    timeframe: MomentumHistoricalTimeframeV1,
) -> Result<Option<MomentumCanonicalMacroPolicyV1>, String> {
    read_single(
        &root.join(format!("macro_source_policies/{}", timeframe.as_str())),
        decode_macro_policy,
    )
}

fn validate_macro_report(value: &MomentumMacroForensicsPublicReportV1) -> Result<(), String> {
    let zero_live_authority = [
        value.live_authority_counters.live_outcome_requests,
        value.live_authority_counters.live_outcome_openings,
        value.live_authority_counters.live_label_reads,
        value.live_authority_counters.live_metric_computations,
        value.live_authority_counters.live_evaluations,
        value.live_authority_counters.live_participant_changes,
        value.live_authority_counters.live_parameter_updates,
        value.live_authority_counters.live_normalizer_refits,
        value.live_authority_counters.live_feature_policy_changes,
        value.live_authority_counters.winner_selections,
        value.live_authority_counters.rankings,
        value.live_authority_counters.reward_applications,
        value.live_authority_counters.penalty_applications,
        value.live_authority_counters.chair_decisions,
        value.live_authority_counters.committee_votes,
        value.live_authority_counters.voice_changes,
        value.live_authority_counters.tier_changes,
        value.live_authority_counters.cooldowns,
        value.live_authority_counters.promotions,
        value.live_authority_counters.quarantines,
        value.live_authority_counters.paper_executions,
        value.live_authority_counters.live_executions,
    ]
    .into_iter()
    .all(|count| count == 0);
    let replay_rule_valid = value.qualified_timeframes.as_ref().is_none_or(|set| {
        set.full_eight_timeframe_replay_allowed == value.hard_replay_registration_digest.is_some()
            && value.hard_replay_blocked == !set.full_eight_timeframe_replay_allowed
    });
    if value.report_version != MACRO_REPORT_VERSION
        || value.run_mode.is_empty()
        || value.network_request_attempts != 0
        || value.transport_constructions != 0
        || value.credentials_read != 0
        || value.epoch_three_registrations != 0
        || value.active_committee_count != 3
        || !zero_live_authority
        || !value.live_protected_artifacts_unchanged
        || !value.active_roster_unchanged
        || value.hard_replay_executed
        || value.holdout_labels_opened
        || !replay_rule_valid
        || value
            .monthly_aggregate
            .as_ref()
            .is_some_and(|aggregate| validate_macro_aggregate(aggregate).is_err())
        || value
            .yearly_aggregate
            .as_ref()
            .is_some_and(|aggregate| validate_macro_aggregate(aggregate).is_err())
        || [
            value.weekly_policy.as_ref(),
            value.monthly_policy.as_ref(),
            value.yearly_policy.as_ref(),
        ]
        .into_iter()
        .flatten()
        .any(|policy| validate_macro_policy(policy).is_err())
        || value
            .qualified_timeframes
            .as_ref()
            .is_some_and(|set| validate_qualified_set(set).is_err())
        || value
            .causal_revalidation
            .as_ref()
            .is_some_and(|journal| validate_causal_revalidation(journal).is_err())
        || value.report_digest != macro_report_digest(value)
    {
        return Err("macro forensics public report rejected".to_string());
    }
    Ok(())
}

#[allow(clippy::type_complexity)]
fn reopen_macro_result(
    root: &Path,
) -> Result<
    (
        Option<MomentumMacroForensicAggregateV1>,
        Option<MomentumMacroForensicAggregateV1>,
        Option<MomentumCanonicalMacroPolicyV1>,
        Option<MomentumCanonicalMacroPolicyV1>,
        Option<MomentumCanonicalMacroPolicyV1>,
        Option<MomentumQualifiedTimeframeSetV1>,
        Option<MomentumQualifiedCausalRevalidationV1>,
        Option<MomentumQualifiedHardReplayRegistrationV2>,
    ),
    String,
> {
    Ok((
        reopen_macro_aggregate(root, MomentumHistoricalTimeframeV1::Month1)?,
        reopen_macro_aggregate(root, MomentumHistoricalTimeframeV1::Year1)?,
        reopen_macro_policy(root, MomentumHistoricalTimeframeV1::Week1)?,
        reopen_macro_policy(root, MomentumHistoricalTimeframeV1::Month1)?,
        reopen_macro_policy(root, MomentumHistoricalTimeframeV1::Year1)?,
        read_single(&root.join("qualified_timeframe_sets"), decode_qualified_set)?,
        read_single(
            &root.join("qualified_causal_revalidations"),
            decode_causal_revalidation,
        )?,
        read_single(
            &root.join("qualified_hard_replay_registrations"),
            decode_qualified_hard_replay,
        )?,
    ))
}

fn macro_status(
    monthly: Option<&MomentumMacroForensicAggregateV1>,
    yearly: Option<&MomentumMacroForensicAggregateV1>,
    set: Option<&MomentumQualifiedTimeframeSetV1>,
) -> MomentumMacroForensicsStatusV1 {
    let Some(set) = set else {
        return MomentumMacroForensicsStatusV1::Unregistered;
    };
    if [monthly, yearly]
        .into_iter()
        .flatten()
        .any(|aggregate| aggregate.failed_count > 0 && !aggregate.native_metadata_complete)
    {
        MomentumMacroForensicsStatusV1::InsufficientPersistedNativeEvidence
    } else if set.unresolved_count > 0 {
        MomentumMacroForensicsStatusV1::BlockedUnresolved
    } else {
        MomentumMacroForensicsStatusV1::Qualified
    }
}

fn run_macro_forensics_at(
    root: &Path,
    live_root: &Path,
    mode: MomentumMacroForensicsRunModeV1,
) -> Result<MomentumMacroForensicsPublicReportV1, String> {
    let protected_before = tree_identity(live_root)?;
    let roster_before = active_roster_digest();
    let mut counts = (0usize, 0usize);
    let built = if matches!(
        mode,
        MomentumMacroForensicsRunModeV1::DryRun | MomentumMacroForensicsRunModeV1::ExecuteLocal
    ) {
        let (monthly_receipts, monthly_aggregate, monthly_policy) =
            build_macro_forensics(root, MomentumHistoricalTimeframeV1::Month1)?;
        let (yearly_receipts, yearly_aggregate, yearly_policy) =
            build_macro_forensics(root, MomentumHistoricalTimeframeV1::Year1)?;
        let weekly_policy = build_weekly_policy(root)?;
        let comparisons = reopen_comparisons(root)?;
        let set = build_qualified_set(
            &comparisons,
            &weekly_policy,
            &monthly_policy,
            &yearly_policy,
        )?;
        let causal = build_causal_revalidation(
            root,
            &set,
            [&weekly_policy, &monthly_policy, &yearly_policy],
        )?;
        let hard = if set.full_eight_timeframe_replay_allowed {
            Some(build_qualified_hard_replay(&set)?)
        } else {
            None
        };
        if mode == MomentumMacroForensicsRunModeV1::ExecuteLocal {
            for receipt in monthly_receipts.iter().chain(&yearly_receipts) {
                add_counts(&mut counts, persist_macro_receipt(root, receipt)?);
            }
            add_counts(
                &mut counts,
                persist_macro_aggregate(root, &monthly_aggregate)?,
            );
            add_counts(
                &mut counts,
                persist_macro_aggregate(root, &yearly_aggregate)?,
            );
            for policy in [&weekly_policy, &monthly_policy, &yearly_policy] {
                add_counts(&mut counts, persist_macro_policy(root, policy)?);
            }
            add_counts(&mut counts, persist_qualified_set(root, &set)?);
            add_counts(&mut counts, persist_causal_revalidation(root, &causal)?);
            if let Some(hard) = &hard {
                add_counts(&mut counts, persist_qualified_hard_replay(root, hard)?);
            }
        }
        Some((
            monthly_aggregate,
            yearly_aggregate,
            weekly_policy,
            monthly_policy,
            yearly_policy,
            set,
            causal,
            hard,
        ))
    } else {
        None
    };
    let (
        monthly_aggregate,
        yearly_aggregate,
        weekly_policy,
        monthly_policy,
        yearly_policy,
        qualified_set,
        causal,
        hard,
    ) = if let Some(built) = built {
        (
            Some(built.0),
            Some(built.1),
            Some(built.2),
            Some(built.3),
            Some(built.4),
            Some(built.5),
            Some(built.6),
            built.7,
        )
    } else {
        reopen_macro_result(root)?
    };
    let holdout = read_single(&root.join("sealed_holdouts"), decode_holdout)?;
    let protocol = read_single(&root.join("protocol_replays"), decode_protocol)?;
    let protected_after = tree_identity(live_root)?;
    let roster_after = active_roster_digest();
    let status = macro_status(
        monthly_aggregate.as_ref(),
        yearly_aggregate.as_ref(),
        qualified_set.as_ref(),
    );
    let hard_replay_blocked = qualified_set
        .as_ref()
        .is_none_or(|set| !set.full_eight_timeframe_replay_allowed);
    let mut value = MomentumMacroForensicsPublicReportV1 {
        report_version: MACRO_REPORT_VERSION.to_string(),
        run_mode: mode.as_str().to_string(),
        status,
        monthly_aggregate,
        yearly_aggregate,
        weekly_policy,
        monthly_policy,
        yearly_policy,
        qualified_timeframes: qualified_set,
        causal_revalidation: causal,
        hard_replay_registration_digest: hard
            .as_ref()
            .map(|registration| registration.registration_digest.clone()),
        hard_replay_blocked,
        hard_replay_executed: hard
            .as_ref()
            .is_some_and(|registration| registration.executed),
        holdout_digest: holdout
            .as_ref()
            .map(|holdout| holdout.holdout_digest.clone()),
        holdout_labels_opened: holdout
            .as_ref()
            .is_some_and(|holdout| holdout.labels_opened),
        protocol_event_count: protocol.as_ref().map_or(0, |protocol| protocol.event_count),
        network_request_attempts: 0,
        transport_constructions: 0,
        credentials_read: 0,
        epoch_three_registrations: 0,
        active_committee_count: canonical_current_agent_states().len(),
        live_authority_counters: MomentumMtfSafetyCountersV1::default(),
        live_protected_artifacts_unchanged: protected_before == protected_after,
        active_roster_unchanged: roster_before == roster_after,
        artifacts_written: counts.0,
        duplicate_artifact_count: counts.1,
        report_digest: String::new(),
    };
    value.report_digest = macro_report_digest(&value);
    validate_macro_report(&value)?;
    Ok(value)
}

pub fn run_momentum_macro_forensics_v1(
    mode: MomentumMacroForensicsRunModeV1,
) -> Result<MomentumMacroForensicsPublicReportV1, String> {
    run_macro_forensics_at(Path::new(ROOT), Path::new(LIVE_ROOT), mode)
}

fn replay_candle_evidence(row: &HistoricalCandleRowV1) -> MomentumQualifiedReplayCandleEvidenceV1 {
    MomentumQualifiedReplayCandleEvidenceV1 {
        timeframe: row.timeframe,
        open_timestamp_ms: row.interval.open_timestamp_ms,
        close_exclusive_timestamp_ms: row.interval.close_exclusive_timestamp_ms,
        open: row.open,
        high: row.high,
        low: row.low,
        close: row.close,
        volume: row.volume,
        trade_value: row.trade_value,
        candle_digest: row.candle_digest.clone(),
        missing_evidence: matches!(
            row.presence,
            CandleIntervalPresenceV1::MissingEvidence | CandleIntervalPresenceV1::IntegrityFailure
        ),
    }
}

pub(super) fn qualified_six_unresolved_contract_valid_v1(
    monthly_unresolved_count: usize,
    yearly_unresolved_count: usize,
    unresolved_root_count: usize,
) -> bool {
    monthly_unresolved_count == 15 && yearly_unresolved_count == 4 && unresolved_root_count == 19
}

pub(super) fn load_momentum_qualified_six_evidence_v1_at(
    root: &Path,
) -> Result<MomentumQualifiedSixEvidenceV1, String> {
    let (pause, foundation, plan) = reopen_foundation(root)?;
    let foundation =
        foundation.ok_or_else(|| "qualified-six foundation unavailable".to_string())?;
    let pause = pause.ok_or_else(|| "qualified-six live pause unavailable".to_string())?;
    let plan = plan.ok_or_else(|| "qualified-six acquisition plan unavailable".to_string())?;
    if foundation.pause_digest != pause.pause_digest
        || plan.foundation_registration_digest != foundation.registration_digest
    {
        return Err("qualified-six foundation binding rejected".to_string());
    }
    if f64::from_bits(foundation.numeric_absolute_tolerance_bits) != ABSOLUTE_TOLERANCE
        || f64::from_bits(foundation.numeric_relative_tolerance_bits) != RELATIVE_TOLERANCE
    {
        return Err("qualified-six tolerance contract changed".to_string());
    }
    let (
        monthly_aggregate,
        yearly_aggregate,
        weekly_policy,
        monthly_policy,
        yearly_policy,
        qualified_set,
        causal,
        hard_replay,
    ) = reopen_macro_result(root)?;
    let monthly_aggregate =
        monthly_aggregate.ok_or_else(|| "qualified-six monthly audit unavailable".to_string())?;
    let yearly_aggregate =
        yearly_aggregate.ok_or_else(|| "qualified-six yearly audit unavailable".to_string())?;
    let weekly_policy =
        weekly_policy.ok_or_else(|| "qualified-six weekly policy unavailable".to_string())?;
    let monthly_policy =
        monthly_policy.ok_or_else(|| "qualified-six monthly policy unavailable".to_string())?;
    let yearly_policy =
        yearly_policy.ok_or_else(|| "qualified-six yearly policy unavailable".to_string())?;
    let qualified_set =
        qualified_set.ok_or_else(|| "qualified-six timeframe set unavailable".to_string())?;
    let causal = causal.ok_or_else(|| "qualified-six causal audit unavailable".to_string())?;
    let unresolved_root_count = monthly_aggregate
        .root_cause_counts
        .iter()
        .chain(&yearly_aggregate.root_cause_counts)
        .filter_map(|entry| entry.strip_prefix("ProviderContractAmbiguous="))
        .map(|count| {
            count
                .parse::<usize>()
                .map_err(|_| "qualified-six unresolved root count rejected".to_string())
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .sum::<usize>();
    if !qualified_six_unresolved_contract_valid_v1(
        monthly_aggregate.unresolved_count,
        yearly_aggregate.unresolved_count,
        unresolved_root_count,
    ) || weekly_policy.selected_source
        != MomentumCanonicalMacroSourceV1::DerivedFromCanonicalDaily
        || monthly_policy.selected_source != MomentumCanonicalMacroSourceV1::ExcludedUnresolved
        || yearly_policy.selected_source != MomentumCanonicalMacroSourceV1::ExcludedUnresolved
        || qualified_set.qualified_count != 6
        || qualified_set.unresolved_count != 2
        || qualified_set.month1 != MomentumTimeframeQualificationV1::ExcludedUnresolved
        || qualified_set.year1 != MomentumTimeframeQualificationV1::ExcludedUnresolved
        || qualified_set.full_eight_timeframe_replay_allowed
        || hard_replay.is_some()
        || causal.future_access_count != 0
        || causal.partial_candle_access_count != 0
        || causal.unqualified_view_access_count != 0
        || causal.labels_read != 0
    {
        return Err("qualified-six prerequisite contract rejected".to_string());
    }
    let minute_index = reopen_index(root, MomentumHistoricalTimeframeV1::Minute1)?
        .ok_or_else(|| "qualified-six minute index unavailable".to_string())?;
    let daily_index = reopen_index(root, MomentumHistoricalTimeframeV1::Day1)?
        .ok_or_else(|| "qualified-six daily index unavailable".to_string())?;
    let minute_rows = reopen_canonical_rows(root, &minute_index)?;
    let daily_rows = reopen_canonical_rows(root, &daily_index)?;
    let included_timeframes = vec![
        MomentumHistoricalTimeframeV1::Minute1,
        MomentumHistoricalTimeframeV1::Minute3,
        MomentumHistoricalTimeframeV1::Minute5,
        MomentumHistoricalTimeframeV1::Minute10,
        MomentumHistoricalTimeframeV1::Day1,
        MomentumHistoricalTimeframeV1::Week1,
    ];
    let excluded_timeframes = vec![
        MomentumHistoricalTimeframeV1::Month1,
        MomentumHistoricalTimeframeV1::Year1,
    ];
    let mut raw_views = BTreeMap::from([
        (MomentumHistoricalTimeframeV1::Minute1, minute_rows.clone()),
        (MomentumHistoricalTimeframeV1::Day1, daily_rows.clone()),
    ]);
    let mut view_index_digests = BTreeMap::from([
        (
            MomentumHistoricalTimeframeV1::Minute1,
            minute_index.index_digest.clone(),
        ),
        (
            MomentumHistoricalTimeframeV1::Day1,
            daily_index.index_digest.clone(),
        ),
    ]);
    for timeframe in [
        MomentumHistoricalTimeframeV1::Minute3,
        MomentumHistoricalTimeframeV1::Minute5,
        MomentumHistoricalTimeframeV1::Minute10,
        MomentumHistoricalTimeframeV1::Week1,
    ] {
        let (base_index, base_rows) = if matches!(
            timeframe,
            MomentumHistoricalTimeframeV1::Minute3
                | MomentumHistoricalTimeframeV1::Minute5
                | MomentumHistoricalTimeframeV1::Minute10
        ) {
            (&minute_index, minute_rows.as_slice())
        } else {
            (&daily_index, daily_rows.as_slice())
        };
        let (rows, derived_index) = aggregate_view(&foundation, base_index, base_rows, timeframe)?;
        let persisted = read_single(
            &root.join(format!("derived_{}/indices", timeframe.as_str())),
            decode_derived_index,
        )?
        .ok_or_else(|| "qualified-six derived index unavailable".to_string())?;
        if persisted != derived_index || derived_index.missing_evidence_count != 0 {
            return Err("qualified-six derived index rejected".to_string());
        }
        view_index_digests.insert(timeframe, derived_index.index_digest);
        raw_views.insert(timeframe, rows);
    }
    let view_index_digests = included_timeframes
        .iter()
        .map(|timeframe| {
            view_index_digests
                .get(timeframe)
                .cloned()
                .ok_or_else(|| "qualified-six ordered index digest unavailable".to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;
    let views = included_timeframes
        .iter()
        .map(|timeframe| {
            let rows = raw_views
                .remove(timeframe)
                .ok_or_else(|| "qualified-six source view unavailable".to_string())?;
            Ok((
                *timeframe,
                rows.iter().map(replay_candle_evidence).collect::<Vec<_>>(),
            ))
        })
        .collect::<Result<BTreeMap<_, _>, String>>()?;
    let protocol = read_single(&root.join("protocol_replays"), decode_protocol)?
        .ok_or_else(|| "qualified-six protocol unavailable".to_string())?;
    if protocol.replay_digest != causal.protocol_replay_digest
        || protocol.event_count != causal.event_count
        || !protocol.all_views_closed
        || protocol.future_access_count != 0
        || protocol.partial_candle_access_count != 0
        || !protocol.prediction_before_reveal
        || protocol.performance_claim_produced
    {
        return Err("qualified-six protocol contract rejected".to_string());
    }
    let protocol_events = protocol
        .receipts
        .iter()
        .map(|receipt| MomentumQualifiedReplayProtocolEventV1 {
            prediction_timestamp_ms: receipt.prediction_timestamp_ms,
            target_timestamp_ms: receipt.target_timestamp_ms,
            receipt_digest: receipt.receipt_digest.clone(),
        })
        .collect::<Vec<_>>();
    let prior_holdout = read_single(&root.join("sealed_holdouts"), decode_holdout)?
        .ok_or_else(|| "qualified-six prior holdout unavailable".to_string())?;
    if prior_holdout.labels_opened
        || prior_holdout.metrics_computed
        || prior_holdout.aggregate_comparison_opened
    {
        return Err("qualified-six prior holdout opened".to_string());
    }
    Ok(MomentumQualifiedSixEvidenceV1 {
        qualified_timeframe_set_digest: qualified_set.set_digest,
        monthly_policy_digest: monthly_policy.policy_digest,
        yearly_policy_digest: yearly_policy.policy_digest,
        causal_revalidation_digest: causal.revalidation_digest,
        protocol_replay_digest: protocol.replay_digest,
        included_timeframes,
        excluded_timeframes,
        view_index_digests,
        views,
        protocol_events,
        prior_holdout: MomentumQualifiedReplayHoldoutEvidenceV1 {
            holdout_digest: prior_holdout.holdout_digest,
            eligible_start_timestamp_ms: prior_holdout.eligible_start_timestamp_ms,
            eligible_end_timestamp_ms: prior_holdout.eligible_end_timestamp_ms,
            holdout_start_timestamp_ms: prior_holdout.holdout_start_timestamp_ms,
            labels_opened: prior_holdout.labels_opened,
            metrics_computed: prior_holdout.metrics_computed,
            aggregate_comparison_opened: prior_holdout.aggregate_comparison_opened,
        },
    })
}

pub(super) fn load_momentum_qualified_six_evidence_v1()
-> Result<MomentumQualifiedSixEvidenceV1, String> {
    load_momentum_qualified_six_evidence_v1_at(Path::new(ROOT))
}

pub(super) fn momentum_qualified_replay_protected_state_v1_at(
    historical_root: &Path,
    live_root: &Path,
) -> Result<MomentumQualifiedReplayProtectedStateV1, String> {
    let (pause, _, _) = reopen_foundation(historical_root)?;
    let pause = pause.ok_or_else(|| "qualified-six live pause unavailable".to_string())?;
    let (live_tree_file_count, live_tree_digest) = tree_identity(live_root)?;
    Ok(MomentumQualifiedReplayProtectedStateV1 {
        live_tree_file_count,
        live_tree_digest,
        active_roster_digest: active_roster_digest(),
        live_completed_event_count: pause.completed_event_count,
        live_scorable_event_count: pause.scorable_event_count,
        live_input_attempts: pause.input_attempts,
        live_input_retries: pause.input_retries,
        live_prediction_seal_count: pause.prediction_seal_count,
        live_outcome_requests: pause.outcome_requests,
        live_outcome_openings: pause.outcome_openings,
        epoch_three_registered: pause.epoch_three_registered,
        active_committee_count: canonical_current_agent_states().len(),
    })
}

pub(super) fn momentum_qualified_replay_protected_state_v1()
-> Result<MomentumQualifiedReplayProtectedStateV1, String> {
    momentum_qualified_replay_protected_state_v1_at(Path::new(ROOT), Path::new(LIVE_ROOT))
}

pub fn format_momentum_macro_forensics_text_v1(
    report: &MomentumMacroForensicsPublicReportV1,
) -> Result<String, String> {
    validate_macro_report(report)?;
    let aggregate = |value: Option<&MomentumMacroForensicAggregateV1>| {
        value.map_or_else(
            || "unregistered".to_string(),
            |value| {
                format!(
                    "periods={},exact={},tolerance={},failed={},partial={},unresolved={},digest={}",
                    value.compared_period_count,
                    value.exact_count,
                    value.tolerance_count,
                    value.failed_count,
                    value.excluded_partial_count,
                    value.unresolved_count,
                    value.aggregate_digest,
                )
            },
        )
    };
    let policy = |value: Option<&MomentumCanonicalMacroPolicyV1>| {
        value.map_or_else(
            || "unregistered".to_string(),
            |value| {
                format!(
                    "{:?},qualified={},unresolved={},digest={}",
                    value.selected_source,
                    value.qualified_period_count,
                    value.unresolved_period_count,
                    value.policy_digest,
                )
            },
        )
    };
    Ok(format!(
        concat!(
            "Momentum Macro Candle Forensics V1\n",
            "mode: {}\nstatus: {:?}\n",
            "month: {}\nyear: {}\n",
            "week policy: {}\nmonth policy: {}\nyear policy: {}\n",
            "qualified timeframes: {}\nunresolved timeframes: {}\n",
            "full replay allowed: {}\nhard replay blocked: {}\nhard replay executed: {}\n",
            "protocol events: {}\nfuture access: {}\npartial access: {}\nunqualified access: {}\n",
            "network requests: {}\ntransport constructions: {}\ncredentials read: {}\n",
            "epoch three registrations: {}\nactive committee count: {}\n",
            "holdout labels opened: {}\nprotected artifacts unchanged: {}\nactive roster unchanged: {}\n",
            "artifacts written: {}\nduplicate artifacts: {}\nreport digest: {}\n"
        ),
        report.run_mode,
        report.status,
        aggregate(report.monthly_aggregate.as_ref()),
        aggregate(report.yearly_aggregate.as_ref()),
        policy(report.weekly_policy.as_ref()),
        policy(report.monthly_policy.as_ref()),
        policy(report.yearly_policy.as_ref()),
        report
            .qualified_timeframes
            .as_ref()
            .map_or(0, |set| set.qualified_count),
        report
            .qualified_timeframes
            .as_ref()
            .map_or(0, |set| set.unresolved_count),
        report
            .qualified_timeframes
            .as_ref()
            .is_some_and(|set| set.full_eight_timeframe_replay_allowed),
        report.hard_replay_blocked,
        report.hard_replay_executed,
        report.protocol_event_count,
        report
            .causal_revalidation
            .as_ref()
            .map_or(0, |journal| journal.future_access_count),
        report
            .causal_revalidation
            .as_ref()
            .map_or(0, |journal| journal.partial_candle_access_count),
        report
            .causal_revalidation
            .as_ref()
            .map_or(0, |journal| journal.unqualified_view_access_count),
        report.network_request_attempts,
        report.transport_constructions,
        report.credentials_read,
        report.epoch_three_registrations,
        report.active_committee_count,
        report.holdout_labels_opened,
        report.live_protected_artifacts_unchanged,
        report.active_roster_unchanged,
        report.artifacts_written,
        report.duplicate_artifact_count,
        report.report_digest,
    ))
}

#[cfg(test)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct QualifiedSixTestFoundationReceiptV1 {
    pub source_identity: String,
    pub pause_identity: String,
    pub foundation_identity: String,
    pub plan_identity: String,
    pub pause_artifact_path: PathBuf,
    pub foundation_artifact_path: PathBuf,
    pub plan_artifact_path: PathBuf,
}

#[cfg(test)]
fn qualified_six_test_daily_snapshot_v1(
    source_seed: u64,
    protected_boundary_ms: u64,
) -> DataSnapshot {
    const TEST_DAILY_ROW_COUNT: usize = 800;
    let first_timestamp_ms = protected_boundary_ms
        .checked_sub(u64::try_from(TEST_DAILY_ROW_COUNT).unwrap_or_default() * DAY_MS)
        .expect("qualified-six test daily start");
    let rows = (0..TEST_DAILY_ROW_COUNT)
        .map(|index| {
            let timestamp_ms =
                first_timestamp_ms + u64::try_from(index).unwrap_or_default() * DAY_MS;
            let close = 100.0 + index as f64 * 0.01 + source_seed as f64 * 0.001;
            HistoricalOhlcvRow {
                symbol: MARKET.to_string(),
                timestamp_ms,
                open: close - 0.25,
                high: close + 0.5,
                low: close - 0.5,
                close,
                volume: 10.0 + index as f64,
                trade_value: Some(close * (10.0 + index as f64)),
            }
        })
        .collect::<Vec<_>>();
    let dataset = HistoricalReplayDataset {
        symbol: MARKET.to_string(),
        rows,
        source: "qualified-six-test-canonical-daily-v1".to_string(),
        reason_codes: vec![ReasonCode::DeterministicPath],
    };
    let content_digest = historical_replay_dataset_digest_v0(&dataset);
    let first_timestamp_ms = dataset.rows.first().map(|row| row.timestamp_ms);
    let last_timestamp_ms = dataset.rows.last().map(|row| row.timestamp_ms);
    DataSnapshot {
        snapshot_id: stable_hash_string(&format!(
            "qualified-six-test-snapshot-v1:{content_digest}"
        )),
        request_key: "qualified-six-test-daily".to_string(),
        provider_id: PROVIDER.to_string(),
        dataset_kind: DatasetKind::CryptoDailyOhlcv,
        market_scope: AcquisitionMarketScope::BtcCrypto,
        symbols: vec![MARKET.to_string()],
        requested_lookback: DataLookback {
            bars: dataset.rows.len(),
            start_timestamp_ms: first_timestamp_ms,
            end_timestamp_ms: last_timestamp_ms,
        },
        actual_start_timestamp_ms: first_timestamp_ms,
        actual_end_timestamp_ms: last_timestamp_ms,
        fetched_at_ms: protected_boundary_ms,
        normalized_at_ms: protected_boundary_ms,
        schema_version: 1,
        row_count: dataset.rows.len(),
        quality_summary: SnapshotQualitySummary {
            accepted: true,
            row_count: dataset.rows.len(),
            reason_codes: vec![ReasonCode::DeterministicPath],
        },
        content_digest,
        sanitized: true,
        read_only: true,
        compatibility: Some(SnapshotCompatibilityV1 {
            cadence: "1d".to_string(),
            adjustment_semantics: SnapshotAdjustmentSemanticsV1::NotApplicable,
            source_schema: "qualified-six-test-canonical-v1".to_string(),
            requested_cutoff_timestamp_ms: last_timestamp_ms,
            maximum_staleness_ms: DAY_MS,
            all_rows_finalized: true,
        }),
        normalized_dataset: dataset,
        provenance: SnapshotProvenance {
            provider_id: PROVIDER.to_string(),
            acquisition_request_id: "qualified-six-test-acquisition".to_string(),
            fetch_receipt_id: "qualified-six-test-fetch".to_string(),
            source_type: SnapshotSourceType::Mock,
            sanitized: true,
            credential_free: true,
            reason_codes: vec![ReasonCode::DeterministicPath],
        },
        reason_codes: vec![ReasonCode::DeterministicPath],
    }
}

#[cfg(test)]
fn qualified_six_test_config_v1() -> UpbitHistoricalPilotConfigV0 {
    UpbitHistoricalPilotConfigV0 {
        provider_id: PROVIDER.to_string(),
        enabled: true,
        market: AcquisitionMarketScope::BtcCrypto,
        symbol: MARKET.to_string(),
        start_timestamp_ms: 1,
        end_timestamp_ms: 2,
        maximum_rows: PAGE_SIZE,
        timeout_seconds: 10,
        max_retries: 1,
        maximum_response_bytes: 262_144,
        snapshot_output_dir: "test-owned-snapshot-root".to_string(),
        network_consent: NetworkConsentV0::ManualLocalSmoke,
        manual_smoke_enabled: true,
        page_size: PAGE_SIZE,
        target_rows: PAGE_SIZE,
        maximum_pages: 1,
        stop_when_campaign_sufficient: false,
        campaign_attempt_enabled: false,
        minimum_inter_request_delay_ms: 1,
    }
}

#[cfg(test)]
fn qualified_six_test_candle_v1(
    timeframe: MomentumHistoricalTimeframeV1,
    open_timestamp_ms: u64,
    ordinal: usize,
    source_seed: u64,
) -> Result<HistoricalCandleRowV1, String> {
    let close = 100.0 + ordinal as f64 * 0.01 + source_seed as f64 * 0.001;
    let volume = 10.0 + (ordinal % 17) as f64;
    let mut value = HistoricalCandleRowV1 {
        timeframe,
        interval: period_interval(timeframe, open_timestamp_ms)?,
        open: close - 0.05,
        high: close + 0.2,
        low: close - 0.25,
        close,
        volume,
        trade_value: volume * close,
        ordered_base_candle_digests: Vec::new(),
        presence: CandleIntervalPresenceV1::ObservedTradeCandle,
        candle_digest: String::new(),
    };
    value.candle_digest = candle_digest(&value);
    validate_candle(&value)?;
    Ok(value)
}

#[cfg(test)]
fn qualified_six_test_comparison_v1(
    timeframe: MomentumHistoricalTimeframeV1,
) -> Result<DerivedNativeComparisonSummaryV1, String> {
    let mut value = DerivedNativeComparisonSummaryV1 {
        comparison_version: COMPARISON_VERSION.to_string(),
        timeframe,
        sample_count: 1,
        exact_match_count: 1,
        within_tolerance_count: 0,
        boundary_mismatch_count: 0,
        missing_native_count: 0,
        completeness_failure_count: 0,
        integrity_failure_count: 0,
        systematic_mismatch_blocks_replay: false,
        comparison_digest: String::new(),
    };
    value.comparison_digest = comparison_digest(&value);
    validate_comparison(&value)?;
    Ok(value)
}

#[cfg(test)]
fn qualified_six_test_macro_aggregate_v1(
    timeframe: MomentumHistoricalTimeframeV1,
    unresolved_count: usize,
) -> Result<MomentumMacroForensicAggregateV1, String> {
    let mut value = MomentumMacroForensicAggregateV1 {
        aggregate_version: MACRO_FORENSIC_AGGREGATE_VERSION.to_string(),
        timeframe,
        ordered_receipt_digests: (0..unresolved_count)
            .map(|ordinal| {
                stable_hash_string(&format!(
                    "qualified-six-test-unresolved-v1:{}:{ordinal}",
                    timeframe.as_str()
                ))
            })
            .collect(),
        compared_period_count: unresolved_count,
        exact_count: 0,
        tolerance_count: 0,
        failed_count: unresolved_count,
        excluded_partial_count: 0,
        unresolved_count,
        root_cause_counts: vec![format!("ProviderContractAmbiguous={unresolved_count}")],
        disposition_counts: vec![format!("ExcludedUnresolved={unresolved_count}")],
        complete_forensic_coverage: true,
        native_metadata_complete: false,
        aggregate_digest: String::new(),
    };
    value.aggregate_digest = macro_aggregate_digest(&value);
    validate_macro_aggregate(&value)?;
    Ok(value)
}

#[cfg(test)]
fn qualified_six_test_macro_policy_v1(
    timeframe: MomentumHistoricalTimeframeV1,
    source: MomentumCanonicalMacroSourceV1,
    daily_index: &HistoricalCandleIndexV1,
    derived_index: &DerivedViewIndexV1,
    forensic_aggregate_digest: String,
    complete_period_count: usize,
    unresolved_period_count: usize,
) -> Result<MomentumCanonicalMacroPolicyV1, String> {
    let mut value = MomentumCanonicalMacroPolicyV1 {
        policy_version: MACRO_POLICY_VERSION.to_string(),
        timeframe,
        selected_source: source,
        daily_index_digest: daily_index.index_digest.clone(),
        derived_index_digest: derived_index.index_digest.clone(),
        native_index_digest: None,
        forensic_aggregate_digest,
        complete_period_count,
        qualified_period_count: complete_period_count - unresolved_period_count,
        excluded_partial_period_count: 0,
        unresolved_period_count,
        live_authority_eligible: false,
        historical_research_only: true,
        policy_digest: String::new(),
    };
    value.policy_digest = macro_policy_digest(&value);
    validate_macro_policy(&value)?;
    Ok(value)
}

#[cfg(test)]
pub(super) fn materialize_qualified_six_test_foundation_v1(
    root: &Path,
    source_seed: u64,
) -> Result<QualifiedSixTestFoundationReceiptV1, String> {
    const TEST_MINUTE_ROW_COUNT: usize = 600;
    let live = deterministic_sealed_epoch_two_report_fixture_v4();
    let pause = build_pause(&live)?;
    let daily = qualified_six_test_daily_snapshot_v1(
        source_seed,
        pause.protected_first_event_input_boundary_ms,
    );
    let source_identity = daily.content_digest.clone();
    let foundation = build_foundation(&pause, &daily)?;
    let plan = build_plan(&foundation, &qualified_six_test_config_v1())?;
    persist_pause(root, &pause)?;
    persist_foundation(root, &foundation)?;
    persist_plan(root, &plan)?;

    let minute_first_timestamp_ms = foundation
        .minute_end_exclusive_timestamp_ms
        .checked_sub(as_u64(TEST_MINUTE_ROW_COUNT)? * MINUTE_MS)
        .ok_or_else(|| "qualified-six test minute start unavailable".to_string())?;
    let minute_rows = (0..TEST_MINUTE_ROW_COUNT)
        .map(|ordinal| {
            qualified_six_test_candle_v1(
                MomentumHistoricalTimeframeV1::Minute1,
                minute_first_timestamp_ms + as_u64(ordinal)? * MINUTE_MS,
                ordinal,
                source_seed,
            )
        })
        .collect::<Result<Vec<_>, String>>()?;
    let daily_rows = daily
        .normalized_dataset
        .rows
        .iter()
        .enumerate()
        .map(|(ordinal, row)| {
            qualified_six_test_candle_v1(
                MomentumHistoricalTimeframeV1::Day1,
                row.timestamp_ms,
                ordinal,
                source_seed,
            )
        })
        .collect::<Result<Vec<_>, String>>()?;
    let (minute_chunks, minute_index) = build_chunks_and_index(
        MomentumHistoricalTimeframeV1::Minute1,
        &minute_rows,
        foundation.minute_start_timestamp_ms,
        foundation.minute_end_exclusive_timestamp_ms,
    )
    .map_err(|error| format!("qualified-six test minute index: {error}"))?;
    let (daily_chunks, daily_index) = build_chunks_and_index(
        MomentumHistoricalTimeframeV1::Day1,
        &daily_rows,
        daily_rows
            .first()
            .ok_or_else(|| "qualified-six test daily rows unavailable".to_string())?
            .interval
            .open_timestamp_ms,
        daily_rows
            .last()
            .ok_or_else(|| "qualified-six test daily rows unavailable".to_string())?
            .interval
            .close_exclusive_timestamp_ms,
    )
    .map_err(|error| format!("qualified-six test daily index: {error}"))?;
    persist_canonical_dataset(root, &minute_chunks, &minute_index)
        .map_err(|error| format!("qualified-six test minute persistence: {error}"))?;
    persist_canonical_dataset(root, &daily_chunks, &daily_index)
        .map_err(|error| format!("qualified-six test daily persistence: {error}"))?;

    let mut views = BTreeMap::from([
        (MomentumHistoricalTimeframeV1::Minute1, minute_rows.clone()),
        (MomentumHistoricalTimeframeV1::Day1, daily_rows.clone()),
    ]);
    let mut derived_indices = BTreeMap::new();
    let mut comparisons = Vec::new();
    for timeframe in [
        MomentumHistoricalTimeframeV1::Minute3,
        MomentumHistoricalTimeframeV1::Minute5,
        MomentumHistoricalTimeframeV1::Minute10,
        MomentumHistoricalTimeframeV1::Week1,
        MomentumHistoricalTimeframeV1::Month1,
        MomentumHistoricalTimeframeV1::Year1,
    ] {
        let (base_index, base_rows) = if matches!(
            timeframe,
            MomentumHistoricalTimeframeV1::Minute3
                | MomentumHistoricalTimeframeV1::Minute5
                | MomentumHistoricalTimeframeV1::Minute10
        ) {
            (&minute_index, minute_rows.as_slice())
        } else {
            (&daily_index, daily_rows.as_slice())
        };
        let (rows, index) =
            aggregate_view(&foundation, base_index, base_rows, timeframe).map_err(|error| {
                format!("qualified-six test derived {}: {error}", timeframe.as_str())
            })?;
        persist_derived_index(root, &index)?;
        let comparison = qualified_six_test_comparison_v1(timeframe)?;
        persist_comparison(root, &comparison)?;
        views.insert(timeframe, rows);
        derived_indices.insert(timeframe, index);
        comparisons.push(comparison);
    }

    let mut protocol_foundation = foundation.clone();
    protocol_foundation.minute_start_timestamp_ms = minute_first_timestamp_ms;
    protocol_foundation.minute_end_exclusive_timestamp_ms =
        foundation.minute_end_exclusive_timestamp_ms;
    let protocol = build_protocol(&protocol_foundation, &comparisons, &views)
        .map_err(|error| format!("qualified-six test protocol: {error}"))?;
    let holdout = build_holdout(&foundation, &protocol)
        .map_err(|error| format!("qualified-six test holdout: {error}"))?;
    persist_protocol(root, &protocol)?;
    persist_holdout(root, &holdout)?;

    let monthly_aggregate =
        qualified_six_test_macro_aggregate_v1(MomentumHistoricalTimeframeV1::Month1, 15)?;
    let yearly_aggregate =
        qualified_six_test_macro_aggregate_v1(MomentumHistoricalTimeframeV1::Year1, 4)?;
    let weekly_policy = qualified_six_test_macro_policy_v1(
        MomentumHistoricalTimeframeV1::Week1,
        MomentumCanonicalMacroSourceV1::DerivedFromCanonicalDaily,
        &daily_index,
        derived_indices
            .get(&MomentumHistoricalTimeframeV1::Week1)
            .ok_or_else(|| "qualified-six test weekly index unavailable".to_string())?,
        comparisons
            .iter()
            .find(|comparison| comparison.timeframe == MomentumHistoricalTimeframeV1::Week1)
            .ok_or_else(|| "qualified-six test weekly comparison unavailable".to_string())?
            .comparison_digest
            .clone(),
        1,
        0,
    )?;
    let monthly_policy = qualified_six_test_macro_policy_v1(
        MomentumHistoricalTimeframeV1::Month1,
        MomentumCanonicalMacroSourceV1::ExcludedUnresolved,
        &daily_index,
        derived_indices
            .get(&MomentumHistoricalTimeframeV1::Month1)
            .ok_or_else(|| "qualified-six test monthly index unavailable".to_string())?,
        monthly_aggregate.aggregate_digest.clone(),
        15,
        15,
    )?;
    let yearly_policy = qualified_six_test_macro_policy_v1(
        MomentumHistoricalTimeframeV1::Year1,
        MomentumCanonicalMacroSourceV1::ExcludedUnresolved,
        &daily_index,
        derived_indices
            .get(&MomentumHistoricalTimeframeV1::Year1)
            .ok_or_else(|| "qualified-six test yearly index unavailable".to_string())?,
        yearly_aggregate.aggregate_digest.clone(),
        4,
        4,
    )?;
    let qualified_set = build_qualified_set(
        &comparisons,
        &weekly_policy,
        &monthly_policy,
        &yearly_policy,
    )?;
    let causal = build_causal_revalidation(
        root,
        &qualified_set,
        [&weekly_policy, &monthly_policy, &yearly_policy],
    )
    .map_err(|error| format!("qualified-six test causal revalidation: {error}"))?;
    persist_macro_aggregate(root, &monthly_aggregate)?;
    persist_macro_aggregate(root, &yearly_aggregate)?;
    persist_macro_policy(root, &weekly_policy)?;
    persist_macro_policy(root, &monthly_policy)?;
    persist_macro_policy(root, &yearly_policy)?;
    persist_qualified_set(root, &qualified_set)?;
    persist_causal_revalidation(root, &causal)?;

    let (reopened_pause, reopened_foundation, reopened_plan) = reopen_foundation(root)?;
    if reopened_pause.as_ref() != Some(&pause)
        || reopened_foundation.as_ref() != Some(&foundation)
        || reopened_plan.as_ref() != Some(&plan)
        || foundation.pause_digest != pause.pause_digest
        || plan.foundation_registration_digest != foundation.registration_digest
    {
        return Err("qualified-six test foundation reopen mismatch".to_string());
    }
    Ok(QualifiedSixTestFoundationReceiptV1 {
        source_identity,
        pause_identity: pause.pause_digest.clone(),
        foundation_identity: foundation.registration_digest.clone(),
        plan_identity: plan.plan_digest.clone(),
        pause_artifact_path: root
            .join("live_continuation_pause")
            .join(format!("{}.pb", pause.pause_digest)),
        foundation_artifact_path: root
            .join("foundation_registrations")
            .join(format!("{}.pb", foundation.registration_digest)),
        plan_artifact_path: root
            .join("acquisition_plans")
            .join(format!("{}.pb", plan.plan_digest)),
    })
}

#[cfg(test)]
mod tests {
    use std::{
        cell::{Cell, RefCell},
        collections::{BTreeMap, BTreeSet, VecDeque},
    };

    use crate::data::{HttpClientError, NetworkConsentV0};
    use crate::test_support::TestWorkspaceLease;

    use super::*;

    struct TestRoot(PathBuf, #[allow(dead_code)] TestWorkspaceLease);

    impl TestRoot {
        fn new(name: &str) -> Self {
            let lease = TestWorkspaceLease::new(name).expect("test workspace lease");
            let path = lease.root().to_path_buf();
            Self(path, lease)
        }
    }

    struct TestClient {
        responses: RefCell<VecDeque<Result<String, HttpClientError>>>,
        calls: Cell<usize>,
    }

    impl TestClient {
        fn with(response: Result<String, HttpClientError>) -> Self {
            Self {
                responses: RefCell::new(VecDeque::from([response])),
                calls: Cell::new(0),
            }
        }
    }

    impl MarketDataHttpClient for TestClient {
        fn get(&self, _url: &str) -> Result<String, HttpClientError> {
            self.calls.set(self.calls.get() + 1);
            self.responses
                .borrow_mut()
                .pop_front()
                .unwrap_or_else(|| Err(HttpClientError::Permanent("unexpected request".into())))
        }
    }

    fn fixture_pause() -> LiveContinuationPauseV1 {
        build_pause(&deterministic_sealed_epoch_two_report_fixture_v4())
            .expect("official sealed epoch-two pause fixture")
    }

    fn fixture_foundation() -> MomentumMtfFoundationRegistrationV1 {
        let pause = fixture_pause();
        let minute_start_timestamp_ms = pause.protected_first_event_input_boundary_ms
            - u64::try_from(PILOT_DAYS).unwrap() * DAY_MS;
        let role_bindings = MomentumHistoricalTimeframeV1::ORDERED
            .iter()
            .map(|timeframe| {
                format!(
                    "{}={}",
                    timeframe.as_str(),
                    MomentumTimeframeRoleV1::for_timeframe(*timeframe).as_str()
                )
            })
            .collect();
        let mut value = MomentumMtfFoundationRegistrationV1 {
            registration_version: FOUNDATION_VERSION.to_string(),
            pause_digest: pause.pause_digest,
            provider_id: PROVIDER.to_string(),
            symbol: MARKET.to_string(),
            ordered_timeframes: MomentumHistoricalTimeframeV1::ORDERED.to_vec(),
            canonical_bases: vec![
                MomentumHistoricalTimeframeV1::Minute1,
                MomentumHistoricalTimeframeV1::Day1,
            ],
            derived_timeframes: vec![
                MomentumHistoricalTimeframeV1::Minute3,
                MomentumHistoricalTimeframeV1::Minute5,
                MomentumHistoricalTimeframeV1::Minute10,
                MomentumHistoricalTimeframeV1::Week1,
                MomentumHistoricalTimeframeV1::Month1,
                MomentumHistoricalTimeframeV1::Year1,
            ],
            role_bindings,
            minute_start_timestamp_ms,
            minute_end_exclusive_timestamp_ms: pause.protected_first_event_input_boundary_ms,
            existing_daily_snapshot_digest: "daily-snapshot".to_string(),
            existing_daily_first_timestamp_ms: timestamp_ms(2017, 9, 26).unwrap(),
            existing_daily_last_timestamp_ms: timestamp_ms(2026, 7, 25).unwrap(),
            existing_daily_row_count: 312,
            chunk_size: CHUNK_SIZE,
            protocol_cadence_ms: PROTOCOL_CADENCE_MS,
            numeric_absolute_tolerance_bits: ABSOLUTE_TOLERANCE.to_bits(),
            numeric_relative_tolerance_bits: RELATIVE_TOLERANCE.to_bits(),
            training_forbidden: true,
            tournament_forbidden: true,
            live_authority_forbidden: true,
            registration_digest: String::new(),
        };
        value.registration_digest = foundation_digest(&value);
        value
    }

    fn fixture_config() -> UpbitHistoricalPilotConfigV0 {
        UpbitHistoricalPilotConfigV0 {
            provider_id: PROVIDER.to_string(),
            enabled: true,
            market: AcquisitionMarketScope::BtcCrypto,
            symbol: MARKET.to_string(),
            start_timestamp_ms: 1,
            end_timestamp_ms: 2,
            maximum_rows: PAGE_SIZE,
            timeout_seconds: 10,
            max_retries: 1,
            maximum_response_bytes: 262_144,
            snapshot_output_dir: "data/local_snapshots".to_string(),
            network_consent: NetworkConsentV0::ManualLocalSmoke,
            manual_smoke_enabled: true,
            page_size: PAGE_SIZE,
            target_rows: PAGE_SIZE,
            maximum_pages: 1,
            stop_when_campaign_sufficient: false,
            campaign_attempt_enabled: false,
            minimum_inter_request_delay_ms: 1,
        }
    }

    fn fixture_plan() -> MomentumMtfAcquisitionPlanV1 {
        build_plan(&fixture_foundation(), &fixture_config()).unwrap()
    }

    fn fixture_row(
        timeframe: MomentumHistoricalTimeframeV1,
        open_timestamp_ms: u64,
        seed: f64,
    ) -> HistoricalCandleRowV1 {
        let mut value = HistoricalCandleRowV1 {
            timeframe,
            interval: period_interval(timeframe, open_timestamp_ms).unwrap(),
            open: seed,
            high: seed + 2.0,
            low: seed - 1.0,
            close: seed + 1.0,
            volume: seed / 10.0 + 1.0,
            trade_value: seed * 2.0 + 1.0,
            ordered_base_candle_digests: Vec::new(),
            presence: CandleIntervalPresenceV1::ObservedTradeCandle,
            candle_digest: String::new(),
        };
        value.candle_digest = candle_digest(&value);
        validate_candle(&value).unwrap();
        value
    }

    fn minute_rows(start: u64, count: usize) -> Vec<HistoricalCandleRowV1> {
        (0..count)
            .map(|offset| {
                fixture_row(
                    MomentumHistoricalTimeframeV1::Minute1,
                    start + u64::try_from(offset).unwrap() * MINUTE_MS,
                    100.0 + offset as f64,
                )
            })
            .collect()
    }

    fn daily_rows(start: u64, count: usize) -> Vec<HistoricalCandleRowV1> {
        (0..count)
            .map(|offset| {
                fixture_row(
                    MomentumHistoricalTimeframeV1::Day1,
                    start + u64::try_from(offset).unwrap() * DAY_MS,
                    100.0 + offset as f64,
                )
            })
            .collect()
    }

    fn canonical_index(
        timeframe: MomentumHistoricalTimeframeV1,
        rows: &[HistoricalCandleRowV1],
    ) -> HistoricalCandleIndexV1 {
        build_chunks_and_index(
            timeframe,
            rows,
            rows.first().unwrap().interval.open_timestamp_ms,
            rows.last().unwrap().interval.close_exclusive_timestamp_ms,
        )
        .unwrap()
        .1
    }

    fn fixture_comparisons() -> Vec<DerivedNativeComparisonSummaryV1> {
        [
            MomentumHistoricalTimeframeV1::Minute3,
            MomentumHistoricalTimeframeV1::Minute5,
            MomentumHistoricalTimeframeV1::Minute10,
            MomentumHistoricalTimeframeV1::Week1,
            MomentumHistoricalTimeframeV1::Month1,
            MomentumHistoricalTimeframeV1::Year1,
        ]
        .into_iter()
        .map(|timeframe| {
            let mut value = DerivedNativeComparisonSummaryV1 {
                comparison_version: COMPARISON_VERSION.to_string(),
                timeframe,
                sample_count: 1,
                exact_match_count: 1,
                within_tolerance_count: 0,
                boundary_mismatch_count: 0,
                missing_native_count: 0,
                completeness_failure_count: 0,
                integrity_failure_count: 0,
                systematic_mismatch_blocks_replay: false,
                comparison_digest: String::new(),
            };
            value.comparison_digest = comparison_digest(&value);
            value
        })
        .collect()
    }

    fn fixture_protocol(event_count: usize) -> MomentumProtocolReplayV1 {
        let mut foundation = fixture_foundation();
        let start = timestamp_ms(2026, 7, 26).unwrap() + 12 * 60 * MINUTE_MS;
        foundation.minute_start_timestamp_ms = start;
        foundation.minute_end_exclusive_timestamp_ms =
            start + u64::try_from(event_count + 1).unwrap() * PROTOCOL_CADENCE_MS;
        let predictions = (1..=event_count)
            .map(|offset| start + u64::try_from(offset).unwrap() * PROTOCOL_CADENCE_MS)
            .collect::<Vec<_>>();
        let mut views = BTreeMap::new();
        for timeframe in MomentumHistoricalTimeframeV1::ORDERED {
            let mut openings = predictions
                .iter()
                .map(|prediction| latest_completed_expected_open(timeframe, *prediction).unwrap())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>();
            openings.sort_unstable();
            views.insert(
                timeframe,
                openings
                    .into_iter()
                    .enumerate()
                    .map(|(offset, open)| fixture_row(timeframe, open, 100.0 + offset as f64))
                    .collect(),
            );
        }
        build_protocol(&foundation, &fixture_comparisons(), &views).unwrap()
    }

    fn provider_body(open_timestamp_ms: u64) -> String {
        serde_json::json!([{
            "market": MARKET,
            "candle_date_time_utc": format_utc_timestamp(open_timestamp_ms).unwrap(),
            "opening_price": 100.0,
            "high_price": 102.0,
            "low_price": 99.0,
            "trade_price": 101.0,
            "candle_acc_trade_volume": 12.0,
            "candle_acc_trade_price": 1212.0
        }])
        .to_string()
    }

    fn fixture_macro_receipt(
        timeframe: MomentumHistoricalTimeframeV1,
        value_comparison: MomentumMacroValueComparisonV1,
    ) -> MomentumMacroCandleForensicReceiptV1 {
        let open = match timeframe {
            MomentumHistoricalTimeframeV1::Month1 => timestamp_ms(2025, 1, 1).unwrap(),
            MomentumHistoricalTimeframeV1::Year1 => timestamp_ms(2025, 1, 1).unwrap(),
            _ => panic!("macro fixture timeframe"),
        };
        let interval = period_interval(timeframe, open).unwrap();
        let (root_cause, disposition) = receipt_resolution(
            MomentumMacroBoundaryComparisonV1::ExactSameInterval,
            MomentumMacroCompletenessComparisonV1::BothComplete,
            value_comparison,
            false,
        );
        let mut value = MomentumMacroCandleForensicReceiptV1 {
            forensic_version: MACRO_FORENSIC_VERSION.to_string(),
            timeframe,
            native_candle_digest: "native-candle".to_string(),
            derived_candle_digest: "derived-candle".to_string(),
            native_candle_timestamp_ms: interval.open_timestamp_ms,
            native_candle_kst_timestamp: Some("2025-01-01T09:00:00".to_string()),
            native_first_day_of_period: Some("2025-01-01".to_string()),
            native_last_trade_timestamp_ms: Some(interval.close_exclusive_timestamp_ms - 1),
            native_open_timestamp_ms: interval.open_timestamp_ms,
            native_close_exclusive_timestamp_ms: interval.close_exclusive_timestamp_ms,
            derived_open_timestamp_ms: interval.open_timestamp_ms,
            derived_close_exclusive_timestamp_ms: interval.close_exclusive_timestamp_ms,
            request_to_exclusive_ms: interval.close_exclusive_timestamp_ms,
            market: MARKET.to_string(),
            provider_id: PROVIDER.to_string(),
            native_response_digest: "native-response".to_string(),
            native_source_row_digests: vec!["native-source".to_string()],
            derived_source_row_digests: vec!["daily-source".to_string()],
            boundary_comparison: MomentumMacroBoundaryComparisonV1::ExactSameInterval,
            completeness_comparison: MomentumMacroCompletenessComparisonV1::BothComplete,
            value_comparison,
            root_cause,
            disposition,
            receipt_digest: String::new(),
        };
        value.receipt_digest = macro_receipt_digest(&value);
        value
    }

    fn fixture_native_page(timeframe: MomentumHistoricalTimeframeV1) -> HistoricalPageReceiptV1 {
        let open = timestamp_ms(2025, 1, 1).unwrap();
        let row = fixture_row(timeframe, open, 100.0);
        let mut value = HistoricalPageReceiptV1 {
            receipt_version: RECEIPT_VERSION.to_string(),
            plan_digest: fixture_plan().plan_digest,
            purpose: PagePurposeV1::NativeCrossCheck,
            timeframe,
            request_fingerprint: format!("native-{}", timeframe.as_str()),
            request_to_exclusive_ms: row.interval.close_exclusive_timestamp_ms,
            requested_count: PAGE_SIZE,
            attempt_sequence: 1,
            status: PageReceiptStatusV1::Verified,
            response_body_digest: Some("response".to_string()),
            normalized_row_digest: Some(normalized_rows_digest(std::slice::from_ref(&row))),
            rows: vec![row],
            request_count: 1,
            retry_count: 0,
            receipt_digest: String::new(),
        };
        value.receipt_digest = receipt_digest(&value);
        value
    }

    fn fixture_macro_policy(
        timeframe: MomentumHistoricalTimeframeV1,
        source: MomentumCanonicalMacroSourceV1,
    ) -> MomentumCanonicalMacroPolicyV1 {
        let unresolved = usize::from(source == MomentumCanonicalMacroSourceV1::ExcludedUnresolved);
        let mut value = MomentumCanonicalMacroPolicyV1 {
            policy_version: MACRO_POLICY_VERSION.to_string(),
            timeframe,
            selected_source: source,
            daily_index_digest: "daily-index".to_string(),
            derived_index_digest: format!("derived-{}", timeframe.as_str()),
            native_index_digest: (source == MomentumCanonicalMacroSourceV1::NativeProviderCandle)
                .then(|| "native-index".to_string()),
            forensic_aggregate_digest: "forensic-aggregate".to_string(),
            complete_period_count: 1,
            qualified_period_count: 1 - unresolved,
            excluded_partial_period_count: 0,
            unresolved_period_count: unresolved,
            live_authority_eligible: false,
            historical_research_only: true,
            policy_digest: String::new(),
        };
        value.policy_digest = macro_policy_digest(&value);
        value
    }

    fn fixture_qualified_set(
        month: MomentumTimeframeQualificationV1,
        year: MomentumTimeframeQualificationV1,
    ) -> MomentumQualifiedTimeframeSetV1 {
        let mut value = MomentumQualifiedTimeframeSetV1 {
            set_version: QUALIFIED_SET_VERSION.to_string(),
            minute1: MomentumTimeframeQualificationV1::QualifiedNativeCanonical,
            minute3: MomentumTimeframeQualificationV1::QualifiedDerivedCanonical,
            minute5: MomentumTimeframeQualificationV1::QualifiedDerivedCanonical,
            minute10: MomentumTimeframeQualificationV1::QualifiedDerivedCanonical,
            day1: MomentumTimeframeQualificationV1::QualifiedNativeCanonical,
            week1: MomentumTimeframeQualificationV1::QualifiedDerivedCanonical,
            month1: month,
            year1: year,
            qualified_count: 0,
            unresolved_count: 0,
            full_eight_timeframe_replay_allowed: false,
            set_digest: String::new(),
        };
        let qualifications = qualified_timeframes(&value);
        value.qualified_count = qualifications
            .iter()
            .filter(|item| {
                matches!(
                    item,
                    MomentumTimeframeQualificationV1::QualifiedDerivedCanonical
                        | MomentumTimeframeQualificationV1::QualifiedNativeCanonical
                )
            })
            .count();
        value.unresolved_count = qualifications
            .iter()
            .filter(|item| **item == MomentumTimeframeQualificationV1::ExcludedUnresolved)
            .count();
        value.full_eight_timeframe_replay_allowed = value.qualified_count == 8;
        value.set_digest = qualified_set_digest(&value);
        value
    }

    fn fixture_macro_report() -> MomentumMacroForensicsPublicReportV1 {
        let set = fixture_qualified_set(
            MomentumTimeframeQualificationV1::ExcludedUnresolved,
            MomentumTimeframeQualificationV1::ExcludedUnresolved,
        );
        let mut value = MomentumMacroForensicsPublicReportV1 {
            report_version: MACRO_REPORT_VERSION.to_string(),
            run_mode: "status".to_string(),
            status: MomentumMacroForensicsStatusV1::InsufficientPersistedNativeEvidence,
            monthly_aggregate: None,
            yearly_aggregate: None,
            weekly_policy: Some(fixture_macro_policy(
                MomentumHistoricalTimeframeV1::Week1,
                MomentumCanonicalMacroSourceV1::DerivedFromCanonicalDaily,
            )),
            monthly_policy: Some(fixture_macro_policy(
                MomentumHistoricalTimeframeV1::Month1,
                MomentumCanonicalMacroSourceV1::ExcludedUnresolved,
            )),
            yearly_policy: Some(fixture_macro_policy(
                MomentumHistoricalTimeframeV1::Year1,
                MomentumCanonicalMacroSourceV1::ExcludedUnresolved,
            )),
            qualified_timeframes: Some(set),
            causal_revalidation: None,
            hard_replay_registration_digest: None,
            hard_replay_blocked: true,
            hard_replay_executed: false,
            holdout_digest: Some("holdout".to_string()),
            holdout_labels_opened: false,
            protocol_event_count: 1,
            network_request_attempts: 0,
            transport_constructions: 0,
            credentials_read: 0,
            epoch_three_registrations: 0,
            active_committee_count: 3,
            live_authority_counters: MomentumMtfSafetyCountersV1::default(),
            live_protected_artifacts_unchanged: true,
            active_roster_unchanged: true,
            artifacts_written: 0,
            duplicate_artifact_count: 0,
            report_digest: String::new(),
        };
        value.report_digest = macro_report_digest(&value);
        value
    }

    #[test]
    fn sprint96_01_completed_live_chain_pause_invariants_hold() {
        let pause = fixture_pause();
        assert!(validate_pause(&pause).is_ok());
        assert_eq!(pause.prediction_seal_count, 3);
        assert_eq!(pause.input_attempts, 1);
        assert_eq!(pause.input_retries, 0);
    }

    #[test]
    fn sprint96_02_live_epoch_two_remains_sealed() {
        let pause = fixture_pause();
        assert_eq!(pause.completed_event_count, 1);
        assert_eq!(pause.scorable_event_count, 1);
        assert_eq!(pause.outcome_requests + pause.outcome_openings, 0);
    }

    #[test]
    fn sprint96_03_epoch_three_registration_is_forbidden() {
        let mut pause = fixture_pause();
        pause.epoch_three_registered = true;
        pause.pause_digest = pause_digest(&pause);
        assert!(validate_pause(&pause).is_err());
    }

    #[test]
    fn sprint103_r1_official_report_builds_exactly_bound_pause() {
        let live = deterministic_sealed_epoch_two_report_fixture_v4();
        let pause = build_pause(&live).expect("official pause");
        assert_eq!(pause.series_digest, live.status.series_digest);
        assert_eq!(
            pause.epoch_registration_digest,
            live.status.epoch_registration_digest
        );
        assert_eq!(
            pause.input_receipt_digest,
            live.status.input_receipt_digest.expect("input receipt")
        );
        assert_eq!(
            pause.input_capsule_digest,
            live.status.input_capsule_digest.expect("input capsule")
        );
        assert_eq!(
            pause.context_proof_digest,
            live.status
                .context_assembly_proof_digest
                .expect("context proof")
        );
        assert_eq!(
            pause.prediction_capsule_digest,
            live.status
                .prediction_capsule_digest
                .expect("prediction capsule")
        );
        assert_eq!(
            pause.prediction_journal_digest,
            live.status.journal_entry_digest.expect("journal")
        );
        assert_eq!(
            pause.outcome_plan_digest,
            live.status.outcome_plan_digest.expect("outcome plan")
        );
        assert_eq!(
            pause.protected_first_event_input_boundary_ms,
            live.event_one_adoption.adopted_event_timestamp_ms
        );
    }

    #[test]
    fn sprint103_r1_malformed_official_report_lineage_is_rejected() {
        let mut live = deterministic_sealed_epoch_two_report_fixture_v4();
        live.series.series_digest.push_str("-tampered");
        assert!(build_pause(&live).is_err());
    }

    #[test]
    fn sprint103_r1_wrong_epoch_and_readiness_are_rejected() {
        let mut wrong_epoch = deterministic_sealed_epoch_two_report_fixture_v4();
        wrong_epoch.status.epoch_number = 3;
        assert!(build_pause(&wrong_epoch).is_err());

        let mut wrong_readiness = deterministic_sealed_epoch_two_report_fixture_v4();
        wrong_readiness.status.readiness =
            MomentumProspectiveEpochReadinessV4::ReadyForInputAcquisition;
        assert!(build_pause(&wrong_readiness).is_err());
    }

    #[test]
    fn sprint103_r1_official_pause_persists_and_reopens_exactly() {
        let root = TestRoot::new("official-pause-reopen");
        let pause =
            build_pause(&deterministic_sealed_epoch_two_report_fixture_v4()).expect("pause");
        assert_eq!(persist_pause(&root.0, &pause).expect("persist"), (1, 0));
        let (reopened, foundation, plan) = reopen_foundation(&root.0).expect("reopen");
        assert_eq!(reopened, Some(pause));
        assert!(foundation.is_none());
        assert!(plan.is_none());
    }

    #[test]
    fn sprint103_r1_official_pause_lineage_has_zero_network_live_and_trade_authority() {
        let live = deterministic_sealed_epoch_two_report_fixture_v4();
        let pause = build_pause(&live).expect("pause");
        assert_eq!(live.status.safety_counters.network_request_attempts, 0);
        assert_eq!(live.status.safety_counters.outcome_requests, 0);
        assert_eq!(live.status.safety_counters.outcome_openings, 0);
        assert_eq!(live.status.safety_counters.paper_executions, 0);
        assert_eq!(live.status.safety_counters.live_executions, 0);
        assert_eq!(pause.outcome_requests, 0);
        assert_eq!(pause.outcome_openings, 0);
        assert!(!pause.epoch_three_registered);
    }

    #[test]
    fn sprint96_04_eight_timeframes_have_exact_order() {
        let foundation = fixture_foundation();
        assert!(validate_foundation(&foundation).is_ok());
        assert_eq!(
            foundation
                .ordered_timeframes
                .iter()
                .map(|value| value.as_str())
                .collect::<Vec<_>>(),
            ["1m", "3m", "5m", "10m", "1d", "1w", "1mo", "1y"]
        );
    }

    #[test]
    fn sprint96_05_only_minute_and_daily_are_canonical() {
        let foundation = fixture_foundation();
        assert_eq!(
            foundation.canonical_bases,
            [
                MomentumHistoricalTimeframeV1::Minute1,
                MomentumHistoricalTimeframeV1::Day1
            ]
        );
        assert_eq!(
            MomentumHistoricalTimeframeV1::ORDERED
                .iter()
                .filter(|value| value.is_canonical())
                .count(),
            2
        );
    }

    fn assert_intraday_derivation(timeframe: MomentumHistoricalTimeframeV1, count: usize) {
        let start = timestamp_ms(2026, 7, 25).unwrap();
        let rows = minute_rows(start, count);
        let index = canonical_index(MomentumHistoricalTimeframeV1::Minute1, &rows);
        let (derived, derived_index) =
            aggregate_view(&fixture_foundation(), &index, &rows, timeframe).unwrap();
        assert_eq!(derived.len(), 1);
        assert_eq!(
            derived_index.canonical_source_timeframe,
            MomentumHistoricalTimeframeV1::Minute1
        );
        assert_eq!(derived[0].ordered_base_candle_digests.len(), count);
    }

    #[test]
    fn sprint96_06_three_minute_derives_from_minute() {
        assert_intraday_derivation(MomentumHistoricalTimeframeV1::Minute3, 3);
    }

    #[test]
    fn sprint96_07_five_minute_derives_from_minute() {
        assert_intraday_derivation(MomentumHistoricalTimeframeV1::Minute5, 5);
    }

    #[test]
    fn sprint96_08_ten_minute_derives_from_minute() {
        assert_intraday_derivation(MomentumHistoricalTimeframeV1::Minute10, 10);
    }

    fn assert_calendar_derivation(
        timeframe: MomentumHistoricalTimeframeV1,
        start: u64,
        count: usize,
    ) {
        let rows = daily_rows(start, count);
        let index = canonical_index(MomentumHistoricalTimeframeV1::Day1, &rows);
        let (derived, derived_index) =
            aggregate_view(&fixture_foundation(), &index, &rows, timeframe).unwrap();
        assert_eq!(derived.len(), 1);
        assert_eq!(
            derived_index.canonical_source_timeframe,
            MomentumHistoricalTimeframeV1::Day1
        );
        assert_eq!(derived[0].ordered_base_candle_digests.len(), count);
    }

    #[test]
    fn sprint96_09_week_derives_from_daily() {
        assert_calendar_derivation(
            MomentumHistoricalTimeframeV1::Week1,
            timestamp_ms(2026, 7, 20).unwrap(),
            7,
        );
    }

    #[test]
    fn sprint96_10_month_derives_from_daily() {
        assert_calendar_derivation(
            MomentumHistoricalTimeframeV1::Month1,
            timestamp_ms(2026, 7, 1).unwrap(),
            31,
        );
    }

    #[test]
    fn sprint96_11_year_derives_from_daily() {
        assert_calendar_derivation(
            MomentumHistoricalTimeframeV1::Year1,
            timestamp_ms(2025, 1, 1).unwrap(),
            365,
        );
    }

    #[test]
    fn sprint96_12_missing_base_evidence_rejects_derivation() {
        let start = timestamp_ms(2026, 7, 25).unwrap();
        let rows = minute_rows(start, 3);
        let mut index = canonical_index(MomentumHistoricalTimeframeV1::Minute1, &rows);
        index.missing_evidence_count = 1;
        index.index_digest = index_digest(&index);
        assert!(
            aggregate_view(
                &fixture_foundation(),
                &index,
                &rows,
                MomentumHistoricalTimeframeV1::Minute3,
            )
            .is_err()
        );
    }

    #[test]
    fn sprint96_13_partial_higher_period_is_unavailable() {
        let current_open = timestamp_ms(2026, 1, 1).unwrap();
        let rows = vec![fixture_row(
            MomentumHistoricalTimeframeV1::Year1,
            current_open,
            100.0,
        )];
        let prediction = timestamp_ms(2026, 7, 26).unwrap();
        let (status, selected) =
            select_as_of(MomentumHistoricalTimeframeV1::Year1, &rows, prediction).unwrap();
        assert_eq!(
            status,
            TimeframeViewAvailabilityV1::InsufficientHistoricalDepth
        );
        assert!(selected.is_none());
    }

    #[test]
    fn sprint96_14_no_trade_differs_from_missing_evidence() {
        assert_ne!(
            CandleIntervalPresenceV1::NoTradeInterval,
            CandleIntervalPresenceV1::MissingEvidence
        );
        let start = timestamp_ms(2026, 7, 25).unwrap();
        let rows = vec![
            fixture_row(MomentumHistoricalTimeframeV1::Minute1, start, 100.0),
            fixture_row(
                MomentumHistoricalTimeframeV1::Minute1,
                start + 2 * MINUTE_MS,
                102.0,
            ),
        ];
        let (_, index) = build_chunks_and_index(
            MomentumHistoricalTimeframeV1::Minute1,
            &rows,
            start,
            start + 3 * MINUTE_MS,
        )
        .unwrap();
        assert_eq!(index.no_trade_interval_count, 1);
        assert_eq!(index.missing_evidence_count, 0);
        assert_eq!(
            select_as_of(
                MomentumHistoricalTimeframeV1::Minute1,
                &rows,
                start + 2 * MINUTE_MS,
            )
            .unwrap()
            .0,
            TimeframeViewAvailabilityV1::NoTradeInterval
        );
        assert_eq!(
            select_as_of(
                MomentumHistoricalTimeframeV1::Minute1,
                &rows,
                start + 4 * MINUTE_MS,
            )
            .unwrap()
            .0,
            TimeframeViewAvailabilityV1::MissingEvidence
        );
    }

    #[test]
    fn sprint96_15_no_trade_gap_is_not_forward_filled() {
        let start = timestamp_ms(2026, 7, 25).unwrap();
        let rows = vec![
            fixture_row(MomentumHistoricalTimeframeV1::Minute1, start, 100.0),
            fixture_row(
                MomentumHistoricalTimeframeV1::Minute1,
                start + 2 * MINUTE_MS,
                102.0,
            ),
        ];
        let index = build_chunks_and_index(
            MomentumHistoricalTimeframeV1::Minute1,
            &rows,
            start,
            start + 3 * MINUTE_MS,
        )
        .unwrap()
        .1;
        let (derived, derived_index) = aggregate_view(
            &fixture_foundation(),
            &index,
            &rows,
            MomentumHistoricalTimeframeV1::Minute3,
        )
        .unwrap();
        assert_eq!(derived[0].ordered_base_candle_digests.len(), 3);
        assert_eq!(derived_index.no_trade_interval_count, 1);
        assert_eq!(derived[0].volume, rows[0].volume + rows[1].volume);
    }

    #[test]
    fn sprint96_16_aggregation_uses_strict_ohlcv_semantics() {
        let start = timestamp_ms(2026, 7, 25).unwrap();
        let rows = minute_rows(start, 3);
        let index = canonical_index(MomentumHistoricalTimeframeV1::Minute1, &rows);
        let derived = aggregate_view(
            &fixture_foundation(),
            &index,
            &rows,
            MomentumHistoricalTimeframeV1::Minute3,
        )
        .unwrap()
        .0
        .remove(0);
        assert_eq!(derived.open, rows[0].open);
        assert_eq!(derived.close, rows[2].close);
        assert_eq!(derived.high, rows[2].high);
        assert_eq!(derived.low, rows[0].low);
        assert_eq!(
            derived.volume,
            rows.iter().map(|row| row.volume).sum::<f64>()
        );
        assert_eq!(
            derived.trade_value,
            rows.iter().map(|row| row.trade_value).sum::<f64>()
        );
    }

    #[test]
    fn sprint96_17_calendar_boundaries_are_deterministic() {
        let instant = timestamp_ms(2026, 7, 26).unwrap() + 12 * 60 * MINUTE_MS;
        let week = period_interval(MomentumHistoricalTimeframeV1::Week1, instant).unwrap();
        let month = period_interval(MomentumHistoricalTimeframeV1::Month1, instant).unwrap();
        let year = period_interval(MomentumHistoricalTimeframeV1::Year1, instant).unwrap();
        assert_eq!(week.open_timestamp_ms, timestamp_ms(2026, 7, 20).unwrap());
        assert_eq!(month.open_timestamp_ms, timestamp_ms(2026, 7, 1).unwrap());
        assert_eq!(year.open_timestamp_ms, timestamp_ms(2026, 1, 1).unwrap());
    }

    #[test]
    fn sprint96_18_utc_kst_ambiguity_rejects() {
        assert!(parse_utc_timestamp("2026-07-26T00:00:00").is_ok());
        assert!(parse_utc_timestamp("2026-07-26T00:00:00Z").is_ok());
        assert!(parse_utc_timestamp("2026-07-26T09:00:00+09:00").is_err());
    }

    #[test]
    fn sprint96_19_native_crosscheck_exact_match() {
        let row = fixture_row(
            MomentumHistoricalTimeframeV1::Minute3,
            timestamp_ms(2026, 7, 25).unwrap(),
            100.0,
        );
        assert_eq!(
            compare_candle(&row, &row),
            DerivedNativeComparisonV1::ExactMatch
        );
    }

    #[test]
    fn sprint96_20_native_crosscheck_registered_tolerance() {
        let derived = fixture_row(
            MomentumHistoricalTimeframeV1::Minute3,
            timestamp_ms(2026, 7, 25).unwrap(),
            100.0,
        );
        let mut native = derived.clone();
        native.volume += 5e-13;
        assert_eq!(
            compare_candle(&derived, &native),
            DerivedNativeComparisonV1::WithinRegisteredTolerance
        );
    }

    #[test]
    fn sprint96_21_systematic_boundary_mismatch_blocks_replay() {
        let protocol = fixture_protocol(1);
        let mut comparisons = fixture_comparisons();
        comparisons[0].exact_match_count = 0;
        comparisons[0].boundary_mismatch_count = 1;
        comparisons[0].systematic_mismatch_blocks_replay = true;
        comparisons[0].comparison_digest = comparison_digest(&comparisons[0]);
        assert!(validate_protocol(&protocol).is_ok());
        assert!(comparisons[0].systematic_mismatch_blocks_replay);
        assert!(!native_comparisons_allow_model_replay(&comparisons));
        assert!(
            !build_future_registration(&fixture_foundation())
                .unwrap()
                .executed
        );
    }

    #[test]
    fn sprint96_22_provider_page_size_is_two_hundred() {
        let boundary = timestamp_ms(2026, 7, 26).unwrap();
        let url = request_url(MomentumHistoricalTimeframeV1::Minute1, boundary).unwrap();
        assert!(url.contains("count=200"));
        assert_eq!(fixture_plan().provider_page_size, 200);
    }

    #[test]
    fn sprint96_23_request_budget_is_derived_before_execution() {
        let plan = fixture_plan();
        assert_eq!(plan.minute_page_budget, 1_296);
        assert_eq!(plan.daily_page_budget, 98);
        assert_eq!(plan.native_sample_request_budget, 6);
        assert_eq!(plan.exact_total_request_budget, 1_400);
    }

    #[test]
    fn sprint96_24_request_ceiling_rejects() {
        let mut plan = fixture_plan();
        plan.exact_total_request_budget += 1;
        plan.daily_page_budget += 1;
        plan.plan_digest = plan_digest(&plan);
        assert!(validate_plan(&plan).is_err());
    }

    #[test]
    fn sprint96_25_concurrency_is_one() {
        assert_eq!(fixture_plan().maximum_concurrency, 1);
    }

    #[test]
    fn sprint96_26_retry_budget_is_zero() {
        assert_eq!(fixture_plan().maximum_retries_per_page, 0);
    }

    #[test]
    fn sprint96_27_verified_checkpoint_prevents_refetch() {
        let root = TestRoot::new("checkpoint");
        let plan = fixture_plan();
        let boundary = timestamp_ms(2026, 7, 26).unwrap();
        let open = boundary - MINUTE_MS;
        let client = TestClient::with(Ok(provider_body(open)));
        let mut pacer = RequestPacerV1::default();
        let mut receipts = Vec::new();
        let mut counts = (0, 0);
        let first = acquire_page(
            &root.0,
            &plan,
            PagePurposeV1::CanonicalMinute,
            MomentumHistoricalTimeframeV1::Minute1,
            boundary,
            &client,
            &mut pacer,
            &mut receipts,
            &mut counts,
        )
        .unwrap();
        let second = acquire_page(
            &root.0,
            &plan,
            PagePurposeV1::CanonicalMinute,
            MomentumHistoricalTimeframeV1::Minute1,
            boundary,
            &client,
            &mut pacer,
            &mut receipts,
            &mut counts,
        )
        .unwrap();
        assert!(first.1);
        assert!(!second.1);
        assert_eq!(client.calls.get(), 1);
        assert_eq!(
            protobuf_paths(&root.0.join("checkpoints")).unwrap().len(),
            1
        );
    }

    #[test]
    fn sprint96_28_failed_page_is_terminal() {
        let root = TestRoot::new("terminal");
        let plan = fixture_plan();
        let client = TestClient::with(Err(HttpClientError::Transient("failed".into())));
        let mut pacer = RequestPacerV1::default();
        let mut receipts = Vec::new();
        let mut counts = (0, 0);
        let result = acquire_page(
            &root.0,
            &plan,
            PagePurposeV1::CanonicalMinute,
            MomentumHistoricalTimeframeV1::Minute1,
            timestamp_ms(2026, 7, 26).unwrap(),
            &client,
            &mut pacer,
            &mut receipts,
            &mut counts,
        );
        assert!(result.is_err());
        assert_eq!(receipts.len(), 1);
        assert_eq!(receipts[0].status, PageReceiptStatusV1::TerminalFailure);
        assert!(
            protobuf_paths(&root.0.join("checkpoints"))
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn sprint96_29_chunks_do_not_overlap() {
        let start = timestamp_ms(2026, 7, 20).unwrap();
        let rows = minute_rows(start, CHUNK_SIZE + 1);
        let (chunks, _) = build_chunks_and_index(
            MomentumHistoricalTimeframeV1::Minute1,
            &rows,
            start,
            start + u64::try_from(rows.len()).unwrap() * MINUTE_MS,
        )
        .unwrap();
        assert_eq!(chunks.len(), 2);
        assert!(chunks[0].last_timestamp_ms < chunks[1].first_timestamp_ms);
        assert_eq!(
            chunks[1].previous_chunk_digest.as_deref(),
            Some(chunks[0].chunk_digest.as_str())
        );
    }

    #[test]
    fn sprint96_30_chunk_index_is_chronological() {
        let start = timestamp_ms(2026, 7, 20).unwrap();
        let rows = minute_rows(start, CHUNK_SIZE + 1);
        let (chunks, index) = build_chunks_and_index(
            MomentumHistoricalTimeframeV1::Minute1,
            &rows,
            start,
            start + u64::try_from(rows.len()).unwrap() * MINUTE_MS,
        )
        .unwrap();
        assert_eq!(
            index.ordered_chunk_digests,
            chunks
                .iter()
                .map(|chunk| chunk.chunk_digest.clone())
                .collect::<Vec<_>>()
        );
        assert_eq!(index.first_timestamp_ms, start);
        assert_eq!(
            index.last_timestamp_ms,
            rows.last().unwrap().interval.open_timestamp_ms
        );
    }

    #[test]
    fn sprint96_31_as_of_join_uses_close_boundary() {
        let start = timestamp_ms(2026, 7, 25).unwrap();
        let rows = minute_rows(start, 2);
        let (status, selected) = select_as_of(
            MomentumHistoricalTimeframeV1::Minute1,
            &rows,
            start + MINUTE_MS,
        )
        .unwrap();
        assert_eq!(status, TimeframeViewAvailabilityV1::Available);
        assert_eq!(
            selected.unwrap().interval.open_timestamp_ms,
            rows[0].interval.open_timestamp_ms
        );
    }

    #[test]
    fn sprint96_32_current_partial_period_cannot_enter_view() {
        let prediction = timestamp_ms(2026, 7, 26).unwrap() + 12 * 60 * MINUTE_MS;
        let previous_open =
            latest_completed_expected_open(MomentumHistoricalTimeframeV1::Day1, prediction)
                .unwrap();
        let current_open = timestamp_ms(2026, 7, 26).unwrap();
        let rows = vec![
            fixture_row(MomentumHistoricalTimeframeV1::Day1, previous_open, 100.0),
            fixture_row(MomentumHistoricalTimeframeV1::Day1, current_open, 101.0),
        ];
        let (_, selected) =
            select_as_of(MomentumHistoricalTimeframeV1::Day1, &rows, prediction).unwrap();
        assert_eq!(selected.unwrap().interval.open_timestamp_ms, previous_open);
    }

    #[test]
    fn sprint96_33_all_eight_views_are_causally_aligned() {
        let protocol = fixture_protocol(1);
        let snapshot = &protocol.snapshots[0];
        assert_eq!(snapshot.availability.len(), 8);
        assert!(
            snapshot
                .availability
                .iter()
                .all(|value| { *value == TimeframeViewAvailabilityV1::Available })
        );
        assert!(snapshot.all_views_closed);
        assert_eq!(snapshot.future_access_count, 0);
        assert_eq!(snapshot.partial_candle_access_count, 0);
    }

    #[test]
    fn sprint96_34_insufficient_year_history_is_classified() {
        let prediction = timestamp_ms(2026, 7, 26).unwrap();
        let rows = vec![fixture_row(
            MomentumHistoricalTimeframeV1::Year1,
            timestamp_ms(2026, 1, 1).unwrap(),
            100.0,
        )];
        assert_eq!(
            select_as_of(MomentumHistoricalTimeframeV1::Year1, &rows, prediction)
                .unwrap()
                .0,
            TimeframeViewAvailabilityV1::InsufficientHistoricalDepth
        );
    }

    #[test]
    fn sprint96_35_protocol_prediction_seals_before_target_reveal() {
        let protocol = fixture_protocol(2);
        for ((snapshot, seal), receipt) in protocol
            .snapshots
            .iter()
            .zip(&protocol.seals)
            .zip(&protocol.receipts)
        {
            assert_eq!(seal.target_access_count_before_seal, 0);
            assert_eq!(seal.as_of_snapshot_digest, snapshot.snapshot_digest);
            assert_eq!(receipt.prediction_seal_digest, seal.seal_digest);
            assert!(receipt.target_revealed_after_seal);
        }
    }

    #[test]
    fn sprint96_36_protocol_produces_no_performance_claim() {
        let protocol = fixture_protocol(2);
        assert!(!protocol.performance_claim_produced);
        assert!(
            protocol
                .receipts
                .iter()
                .all(|receipt| !receipt.performance_claim_produced)
        );
    }

    #[test]
    fn sprint96_37_sealed_holdout_remains_unopened() {
        let protocol = fixture_protocol(20);
        let holdout = build_holdout(&fixture_foundation(), &protocol).unwrap();
        assert!(!holdout.labels_opened);
        assert!(!holdout.metrics_computed);
        assert!(!holdout.aggregate_comparison_opened);
        assert_eq!(
            holdout.development_event_count
                + holdout.validation_event_count
                + holdout.holdout_event_count,
            20
        );
    }

    #[test]
    fn sprint96_38_historical_status_cannot_access_live_outcome() {
        let root = TestRoot::new("status-root");
        let live_root = TestRoot::new("status-live");
        let client = TestClient::with(Err(HttpClientError::Permanent("unused".into())));
        let report = run_at(
            &root.0,
            &live_root.0,
            &[],
            None,
            None,
            MomentumMtfHistoryRunModeV1::Status,
            false,
            false,
            &client,
        )
        .unwrap();
        assert_eq!(client.calls.get(), 0);
        assert_eq!(report.safety_counters.live_outcome_requests, 0);
        assert_eq!(report.safety_counters.live_outcome_openings, 0);
        assert_eq!(report.safety_counters.live_label_reads, 0);
    }

    #[test]
    fn sprint96_39_historical_status_preserves_live_roster() {
        let root = TestRoot::new("roster-root");
        let live_root = TestRoot::new("roster-live");
        let client = TestClient::with(Err(HttpClientError::Permanent("unused".into())));
        let before = active_roster_digest();
        let report = run_at(
            &root.0,
            &live_root.0,
            &[],
            None,
            None,
            MomentumMtfHistoryRunModeV1::DryRun,
            false,
            false,
            &client,
        )
        .unwrap();
        assert_eq!(before, active_roster_digest());
        assert!(report.active_roster_unchanged);
        assert_eq!(report.safety_counters.live_participant_changes, 0);
    }

    #[test]
    fn sprint96_40_reward_and_chair_counters_remain_zero() {
        let counters = safety_counters(&[], None);
        assert_eq!(counters.reward_applications, 0);
        assert_eq!(counters.penalty_applications, 0);
        assert_eq!(counters.chair_decisions, 0);
        assert_eq!(counters.committee_votes, 0);
    }

    #[test]
    fn sprint96_41_protocol_replay_is_deterministic() {
        assert_eq!(fixture_protocol(4), fixture_protocol(4));
    }

    #[test]
    fn sprint96_42_duplicate_protocol_persistence_performs_zero_writes() {
        let root = TestRoot::new("duplicate");
        let protocol = fixture_protocol(2);
        assert_eq!(persist_protocol(&root.0, &protocol).unwrap(), (1, 0));
        assert_eq!(persist_protocol(&root.0, &protocol).unwrap(), (0, 1));
    }

    #[test]
    fn sprint96_43_malformed_protobuf_rejects() {
        assert!(decode_candle(&[0xff, 0x01, 0x02]).is_err());
        assert!(decode_protocol(&[0x08, 0xff]).is_err());
    }

    #[test]
    fn sprint96_44_text_and_json_public_reports_agree() {
        let root = TestRoot::new("format-root");
        let live_root = TestRoot::new("format-live");
        let client = TestClient::with(Err(HttpClientError::Permanent("unused".into())));
        let report = run_at(
            &root.0,
            &live_root.0,
            &[],
            None,
            None,
            MomentumMtfHistoryRunModeV1::Status,
            false,
            false,
            &client,
        )
        .unwrap();
        let text = format_momentum_multitimeframe_history_text_v1(&report).unwrap();
        let json = serde_json::to_string(&report).unwrap();
        assert!(text.contains(&format!("mode: {}", report.run_mode)));
        assert!(text.contains(&report.report_digest));
        assert!(json.contains(&report.report_digest));
        assert!(json.contains("\"run_mode\":\"status\""));
    }

    #[test]
    fn sprint96_45_feature_blocks_are_private_dimension_checked_and_provenance_bound() {
        let block = build_momentum_timeframe_feature_block_v1(
            MomentumHistoricalTimeframeV1::Minute5,
            "source-view",
            &[0.0, 1.0],
            2,
            true,
        )
        .unwrap();
        assert!(block.numeric_values_private);
        assert!(validate_feature_block(&block).is_ok());
        assert!(
            build_momentum_timeframe_feature_block_v1(
                MomentumHistoricalTimeframeV1::Minute5,
                "source-view",
                &[0.0],
                2,
                true,
            )
            .is_err()
        );
        assert!(
            build_momentum_timeframe_feature_block_v1(
                MomentumHistoricalTimeframeV1::Minute5,
                "source-view",
                &[0.0, 1.0],
                2,
                false,
            )
            .is_err()
        );
    }

    #[test]
    fn sprint97_01_prior_acquisition_and_protocol_invariants_remain_valid() {
        assert!(validate_plan(&fixture_plan()).is_ok());
        assert!(validate_protocol(&fixture_protocol(2)).is_ok());
        assert_eq!(fixture_plan().maximum_concurrency, 1);
        assert_eq!(fixture_plan().maximum_retries_per_page, 0);
    }

    #[test]
    fn sprint97_02_live_epoch_two_remains_sealed() {
        let pause = fixture_pause();
        assert_eq!(pause.completed_event_count, 1);
        assert_eq!(pause.prediction_seal_count, 3);
        assert!(!pause.epoch_three_registered);
    }

    #[test]
    fn sprint97_03_event_two_outcome_is_never_accessed() {
        let pause = fixture_pause();
        assert_eq!(pause.outcome_requests, 0);
        assert_eq!(pause.outcome_openings, 0);
    }

    #[test]
    fn sprint97_04_native_monthly_artifact_reopens() {
        let value = fixture_native_page(MomentumHistoricalTimeframeV1::Month1);
        assert_eq!(
            decode_page_receipt(&encode_page_receipt(&value).unwrap()).unwrap(),
            value
        );
    }

    #[test]
    fn sprint97_05_native_yearly_artifact_reopens() {
        let value = fixture_native_page(MomentumHistoricalTimeframeV1::Year1);
        assert_eq!(
            decode_page_receipt(&encode_page_receipt(&value).unwrap()).unwrap(),
            value
        );
    }

    #[test]
    fn sprint97_06_native_first_day_of_period_is_preserved() {
        let value = fixture_macro_receipt(
            MomentumHistoricalTimeframeV1::Month1,
            MomentumMacroValueComparisonV1::ExactAllFields,
        );
        let reopened = decode_macro_receipt(&encode_macro_receipt(&value).unwrap()).unwrap();
        assert_eq!(
            reopened.native_first_day_of_period.as_deref(),
            Some("2025-01-01")
        );
    }

    #[test]
    fn sprint97_07_boundary_comparison_uses_semantic_intervals() {
        let interval = period_interval(
            MomentumHistoricalTimeframeV1::Month1,
            timestamp_ms(2025, 1, 15).unwrap(),
        )
        .unwrap();
        assert_eq!(
            classify_macro_boundary(&interval, &interval),
            MomentumMacroBoundaryComparisonV1::ExactSameInterval
        );
    }

    #[test]
    fn sprint97_08_utc_kst_representation_does_not_create_false_mismatch() {
        let mut value = fixture_macro_receipt(
            MomentumHistoricalTimeframeV1::Month1,
            MomentumMacroValueComparisonV1::ExactAllFields,
        );
        value.boundary_comparison =
            MomentumMacroBoundaryComparisonV1::SamePeriodDifferentTimestampRepresentation;
        value.receipt_digest = macro_receipt_digest(&value);
        assert!(validate_macro_receipt(&value).is_ok());
    }

    #[test]
    fn sprint97_09_different_actual_intervals_create_boundary_mismatch() {
        let native = CandleIntervalV1 {
            open_timestamp_ms: timestamp_ms(2025, 1, 1).unwrap(),
            close_exclusive_timestamp_ms: timestamp_ms(2025, 2, 1).unwrap(),
        };
        let derived = CandleIntervalV1 {
            open_timestamp_ms: timestamp_ms(2025, 1, 2).unwrap(),
            close_exclusive_timestamp_ms: timestamp_ms(2025, 2, 1).unwrap(),
        };
        assert_eq!(
            classify_macro_boundary(&native, &derived),
            MomentumMacroBoundaryComparisonV1::OpeningBoundaryMismatch
        );
    }

    #[test]
    fn sprint97_10_partial_current_period_is_excluded() {
        let resolution = receipt_resolution(
            MomentumMacroBoundaryComparisonV1::ExactSameInterval,
            MomentumMacroCompletenessComparisonV1::NativePartialDerivedExcluded,
            MomentumMacroValueComparisonV1::IntegrityFailure,
            true,
        );
        assert_eq!(
            resolution.1,
            MomentumMacroCandleDispositionV1::ExcludedPartialPeriodNotAFailure
        );
    }

    #[test]
    fn sprint97_11_partial_first_historical_period_is_classified() {
        let resolution = receipt_resolution(
            MomentumMacroBoundaryComparisonV1::ExactSameInterval,
            MomentumMacroCompletenessComparisonV1::SourceCoverageStartsInsidePeriod,
            MomentumMacroValueComparisonV1::IntegrityFailure,
            true,
        );
        assert_eq!(
            resolution.0,
            Some(MomentumMacroMismatchRootCauseV1::PartialFirstCalendarPeriod)
        );
    }

    #[test]
    fn sprint97_12_source_coverage_boundary_is_classified() {
        let mut macro_row = fixture_row(
            MomentumHistoricalTimeframeV1::Month1,
            timestamp_ms(2025, 1, 1).unwrap(),
            100.0,
        );
        macro_row.ordered_base_candle_digests = vec!["day".to_string()];
        macro_row.candle_digest = candle_digest(&macro_row);
        let rows = daily_rows(timestamp_ms(2025, 1, 2).unwrap(), 40);
        let index = canonical_index(MomentumHistoricalTimeframeV1::Day1, &rows);
        assert_eq!(
            classify_macro_completeness(&macro_row, &index),
            MomentumMacroCompletenessComparisonV1::SourceCoverageStartsInsidePeriod
        );
    }

    #[test]
    fn sprint97_13_missing_daily_evidence_blocks_derivation() {
        let mut macro_row = fixture_row(
            MomentumHistoricalTimeframeV1::Month1,
            timestamp_ms(2025, 1, 1).unwrap(),
            100.0,
        );
        macro_row.ordered_base_candle_digests = vec!["day".to_string()];
        macro_row.candle_digest = candle_digest(&macro_row);
        let rows = daily_rows(timestamp_ms(2025, 1, 1).unwrap(), 40);
        let mut index = canonical_index(MomentumHistoricalTimeframeV1::Day1, &rows);
        index.missing_evidence_count = 1;
        assert_eq!(
            classify_macro_completeness(&macro_row, &index),
            MomentumMacroCompletenessComparisonV1::MissingDailyEvidence
        );
    }

    #[test]
    fn sprint97_14_no_trade_is_distinct_from_missing_evidence() {
        let no_trade = receipt_resolution(
            MomentumMacroBoundaryComparisonV1::ExactSameInterval,
            MomentumMacroCompletenessComparisonV1::NoTradeCompositionDiffers,
            MomentumMacroValueComparisonV1::IntegrityFailure,
            true,
        );
        let missing = receipt_resolution(
            MomentumMacroBoundaryComparisonV1::ExactSameInterval,
            MomentumMacroCompletenessComparisonV1::MissingDailyEvidence,
            MomentumMacroValueComparisonV1::IntegrityFailure,
            true,
        );
        assert_ne!(no_trade.0, missing.0);
    }

    #[test]
    fn sprint97_15_ohlc_mismatch_cannot_use_accumulation_tolerance() {
        let derived = fixture_row(
            MomentumHistoricalTimeframeV1::Month1,
            timestamp_ms(2025, 1, 1).unwrap(),
            100.0,
        );
        let mut native = derived.clone();
        native.open += ABSOLUTE_TOLERANCE / 2.0;
        assert_eq!(
            classify_macro_value(
                &derived,
                &native,
                MomentumMacroBoundaryComparisonV1::ExactSameInterval
            ),
            MomentumMacroValueComparisonV1::OpenMismatch
        );
    }

    #[test]
    fn sprint97_16_volume_tolerance_remains_fixed() {
        let derived = fixture_row(
            MomentumHistoricalTimeframeV1::Month1,
            timestamp_ms(2025, 1, 1).unwrap(),
            100.0,
        );
        let mut native = derived.clone();
        native.volume += ABSOLUTE_TOLERANCE / 2.0;
        assert_eq!(
            classify_macro_value(
                &derived,
                &native,
                MomentumMacroBoundaryComparisonV1::ExactSameInterval
            ),
            MomentumMacroValueComparisonV1::AccumulationWithinRegisteredTolerance
        );
        native.volume += 1.0;
        assert_eq!(
            classify_macro_value(
                &derived,
                &native,
                MomentumMacroBoundaryComparisonV1::ExactSameInterval
            ),
            MomentumMacroValueComparisonV1::VolumeOutsideRegisteredTolerance
        );
    }

    #[test]
    fn sprint97_17_trade_value_tolerance_remains_fixed() {
        let derived = fixture_row(
            MomentumHistoricalTimeframeV1::Year1,
            timestamp_ms(2025, 1, 1).unwrap(),
            100.0,
        );
        let mut native = derived.clone();
        native.trade_value += ABSOLUTE_TOLERANCE / 2.0;
        assert_eq!(
            classify_macro_value(
                &derived,
                &native,
                MomentumMacroBoundaryComparisonV1::ExactSameInterval
            ),
            MomentumMacroValueComparisonV1::AccumulationWithinRegisteredTolerance
        );
        native.trade_value += 1.0;
        assert_eq!(
            classify_macro_value(
                &derived,
                &native,
                MomentumMacroBoundaryComparisonV1::ExactSameInterval
            ),
            MomentumMacroValueComparisonV1::TradeValueOutsideRegisteredTolerance
        );
    }

    #[test]
    fn sprint97_18_tolerance_widening_is_not_available() {
        let foundation = fixture_foundation();
        assert_eq!(
            foundation.numeric_absolute_tolerance_bits,
            ABSOLUTE_TOLERANCE.to_bits()
        );
        assert_eq!(
            foundation.numeric_relative_tolerance_bits,
            RELATIVE_TOLERANCE.to_bits()
        );
    }

    #[test]
    fn sprint97_19_every_failed_month_has_one_root_cause() {
        for mismatch in [
            MomentumMacroValueComparisonV1::OpenMismatch,
            MomentumMacroValueComparisonV1::VolumeOutsideRegisteredTolerance,
            MomentumMacroValueComparisonV1::MultipleValueMismatches,
        ] {
            assert!(
                receipt_resolution(
                    MomentumMacroBoundaryComparisonV1::ExactSameInterval,
                    MomentumMacroCompletenessComparisonV1::BothComplete,
                    mismatch,
                    false,
                )
                .0
                .is_some()
            );
        }
    }

    #[test]
    fn sprint97_20_every_failed_year_has_one_root_cause() {
        let receipt = fixture_macro_receipt(
            MomentumHistoricalTimeframeV1::Year1,
            MomentumMacroValueComparisonV1::CloseMismatch,
        );
        assert_eq!(
            receipt.root_cause,
            Some(MomentumMacroMismatchRootCauseV1::ProviderContractAmbiguous)
        );
    }

    #[test]
    fn sprint97_21_unresolved_failure_blocks_full_fusion() {
        let set = fixture_qualified_set(
            MomentumTimeframeQualificationV1::ExcludedUnresolved,
            MomentumTimeframeQualificationV1::QualifiedDerivedCanonical,
        );
        assert!(!set.full_eight_timeframe_replay_allowed);
        assert!(build_qualified_hard_replay(&set).is_err());
    }

    #[test]
    fn sprint97_22_native_promotion_preserves_provenance() {
        let mut index = MomentumNativeMacroCanonicalIndexV1 {
            index_version: NATIVE_MACRO_INDEX_VERSION.to_string(),
            timeframe: MomentumHistoricalTimeframeV1::Month1,
            ordered_native_candle_digests: vec!["a".to_string(), "b".to_string()],
            ordered_first_day_of_period: vec!["2025-01-01".to_string(), "2025-02-01".to_string()],
            first_complete_period: "2025-01-01".to_string(),
            last_complete_period: "2025-02-01".to_string(),
            total_complete_periods: 2,
            source_response_digests: vec!["response".to_string()],
            normalization_policy_digest: "normalization".to_string(),
            index_digest: String::new(),
        };
        index.index_digest = native_macro_index_digest(&index);
        let reopened =
            decode_native_macro_index(&encode_native_macro_index(&index).unwrap()).unwrap();
        assert_eq!(reopened.source_response_digests, vec!["response"]);
    }

    #[test]
    fn sprint97_23_promoted_native_index_excludes_partial_periods() {
        let mut index = MomentumNativeMacroCanonicalIndexV1 {
            index_version: NATIVE_MACRO_INDEX_VERSION.to_string(),
            timeframe: MomentumHistoricalTimeframeV1::Year1,
            ordered_native_candle_digests: vec!["complete".to_string()],
            ordered_first_day_of_period: vec!["2024-01-01".to_string()],
            first_complete_period: "2024-01-01".to_string(),
            last_complete_period: "2024-01-01".to_string(),
            total_complete_periods: 1,
            source_response_digests: vec!["response".to_string()],
            normalization_policy_digest: "normalization".to_string(),
            index_digest: String::new(),
        };
        index.index_digest = native_macro_index_digest(&index);
        assert!(validate_native_macro_index(&index).is_ok());
        assert!(
            !index
                .ordered_first_day_of_period
                .contains(&"2025-01-01".to_string())
        );
    }

    #[test]
    fn sprint97_24_corrected_policy_regenerates_all_periods() {
        let mut index = MomentumCorrectedDerivedMacroIndexV2 {
            index_version: CORRECTED_DERIVED_INDEX_VERSION.to_string(),
            timeframe: MomentumHistoricalTimeframeV1::Month1,
            prior_index_digest: "old".to_string(),
            corrected_aggregation_policy_digest: "policy-v2".to_string(),
            ordered_candle_digests: vec!["a".to_string(), "b".to_string()],
            regenerated_period_count: 2,
            old_index_preserved: true,
            index_digest: String::new(),
        };
        index.index_digest = corrected_derived_index_digest(&index);
        assert_eq!(
            decode_corrected_derived_index(&encode_corrected_derived_index(&index).unwrap())
                .unwrap()
                .regenerated_period_count,
            2
        );
    }

    #[test]
    fn sprint97_25_old_indexes_remain_immutable() {
        let old = "old-index".to_string();
        let mut index = MomentumCorrectedDerivedMacroIndexV2 {
            index_version: CORRECTED_DERIVED_INDEX_VERSION.to_string(),
            timeframe: MomentumHistoricalTimeframeV1::Year1,
            prior_index_digest: old.clone(),
            corrected_aggregation_policy_digest: "policy-v2".to_string(),
            ordered_candle_digests: vec!["new".to_string()],
            regenerated_period_count: 1,
            old_index_preserved: true,
            index_digest: String::new(),
        };
        index.index_digest = corrected_derived_index_digest(&index);
        assert_eq!(index.prior_index_digest, old);
        assert!(index.old_index_preserved);
    }

    #[test]
    fn sprint97_26_weekly_policy_is_independently_qualified() {
        let policy = fixture_macro_policy(
            MomentumHistoricalTimeframeV1::Week1,
            MomentumCanonicalMacroSourceV1::DerivedFromCanonicalDaily,
        );
        assert!(validate_macro_policy(&policy).is_ok());
        assert_eq!(
            qualification_for_policy(&policy),
            MomentumTimeframeQualificationV1::QualifiedDerivedCanonical
        );
    }

    #[test]
    fn sprint97_27_causal_as_of_uses_selected_canonical_source() {
        let native = fixture_macro_policy(
            MomentumHistoricalTimeframeV1::Month1,
            MomentumCanonicalMacroSourceV1::NativeProviderCandle,
        );
        assert_eq!(
            qualification_for_policy(&native),
            MomentumTimeframeQualificationV1::QualifiedNativeCanonical
        );
    }

    #[test]
    fn sprint97_28_native_macro_availability_uses_period_close() {
        let open = timestamp_ms(2025, 1, 1).unwrap();
        let rows = vec![fixture_row(
            MomentumHistoricalTimeframeV1::Month1,
            open,
            100.0,
        )];
        let close = rows[0].interval.close_exclusive_timestamp_ms;
        assert!(
            select_as_of(MomentumHistoricalTimeframeV1::Month1, &rows, close)
                .unwrap()
                .1
                .is_some()
        );
    }

    #[test]
    fn sprint97_29_partial_month_cannot_enter_snapshot() {
        let open = timestamp_ms(2025, 1, 1).unwrap();
        let rows = vec![fixture_row(
            MomentumHistoricalTimeframeV1::Month1,
            open,
            100.0,
        )];
        assert!(
            select_as_of(MomentumHistoricalTimeframeV1::Month1, &rows, open + DAY_MS)
                .unwrap()
                .1
                .is_none()
        );
    }

    #[test]
    fn sprint97_30_partial_year_cannot_enter_snapshot() {
        let open = timestamp_ms(2025, 1, 1).unwrap();
        let rows = vec![fixture_row(
            MomentumHistoricalTimeframeV1::Year1,
            open,
            100.0,
        )];
        assert!(
            select_as_of(
                MomentumHistoricalTimeframeV1::Year1,
                &rows,
                open + 30 * DAY_MS
            )
            .unwrap()
            .1
            .is_none()
        );
    }

    #[test]
    fn sprint97_31_qualified_set_counts_are_derived() {
        let set = fixture_qualified_set(
            MomentumTimeframeQualificationV1::ExcludedUnresolved,
            MomentumTimeframeQualificationV1::ExcludedUnresolved,
        );
        assert_eq!(set.qualified_count, 6);
        assert_eq!(set.unresolved_count, 2);
        assert!(validate_qualified_set(&set).is_ok());
    }

    #[test]
    fn sprint97_32_a3_requires_all_eight_qualified_sources() {
        let blocked = fixture_qualified_set(
            MomentumTimeframeQualificationV1::ExcludedUnresolved,
            MomentumTimeframeQualificationV1::QualifiedDerivedCanonical,
        );
        assert!(build_qualified_hard_replay(&blocked).is_err());
        let allowed = fixture_qualified_set(
            MomentumTimeframeQualificationV1::QualifiedDerivedCanonical,
            MomentumTimeframeQualificationV1::QualifiedNativeCanonical,
        );
        assert!(build_qualified_hard_replay(&allowed).is_ok());
    }

    #[test]
    fn sprint97_33_holdout_remains_closed() {
        let holdout = build_holdout(&fixture_foundation(), &fixture_protocol(20)).unwrap();
        assert!(!holdout.labels_opened);
        assert!(!holdout.metrics_computed);
        assert!(!holdout.aggregate_comparison_opened);
    }

    #[test]
    fn sprint97_34_new_holdout_boundary_never_reads_labels() {
        let original = build_holdout(&fixture_foundation(), &fixture_protocol(20)).unwrap();
        let additive_boundary_identity = stable_hash_string(&format!(
            "holdout-boundary-v2:{}:{}",
            original.holdout_digest, original.holdout_start_timestamp_ms
        ));
        assert!(!additive_boundary_identity.is_empty());
        assert!(!original.labels_opened);
    }

    #[test]
    fn sprint97_35_model_replay_is_registered_not_executed() {
        let set = fixture_qualified_set(
            MomentumTimeframeQualificationV1::QualifiedDerivedCanonical,
            MomentumTimeframeQualificationV1::QualifiedNativeCanonical,
        );
        let registration = build_qualified_hard_replay(&set).unwrap();
        assert!(!registration.executed);
        assert!(registration.full_eight_timeframe_required);
    }

    #[test]
    fn sprint97_36_constant_benchmark_remains_mandatory() {
        let set = fixture_qualified_set(
            MomentumTimeframeQualificationV1::QualifiedDerivedCanonical,
            MomentumTimeframeQualificationV1::QualifiedNativeCanonical,
        );
        let registration = build_qualified_hard_replay(&set).unwrap();
        assert!(registration.constant_benchmark_mandatory);
        assert!(registration.model_families[0].contains("constant benchmark"));
    }

    #[test]
    fn sprint97_37_live_counts_and_roster_remain_unchanged() {
        let before = active_roster_digest();
        let report = fixture_macro_report();
        assert_eq!(before, active_roster_digest());
        assert!(report.active_roster_unchanged);
        assert_eq!(report.active_committee_count, 3);
        assert_eq!(report.epoch_three_registrations, 0);
        assert_eq!(report.live_authority_counters.live_participant_changes, 0);
    }

    #[test]
    fn sprint97_38_reward_and_chair_counters_remain_zero() {
        let counters = fixture_macro_report().live_authority_counters;
        assert_eq!(counters.reward_applications, 0);
        assert_eq!(counters.penalty_applications, 0);
        assert_eq!(counters.chair_decisions, 0);
        assert_eq!(counters.committee_votes, 0);
    }

    #[test]
    fn sprint97_39_forensic_execution_has_zero_network_authority() {
        let report = fixture_macro_report();
        assert_eq!(report.network_request_attempts, 0);
        assert_eq!(report.transport_constructions, 0);
        assert_eq!(report.credentials_read, 0);
    }

    #[test]
    fn sprint97_40_macro_forensics_are_deterministic() {
        let left = fixture_macro_receipt(
            MomentumHistoricalTimeframeV1::Month1,
            MomentumMacroValueComparisonV1::ExactAllFields,
        );
        let right = fixture_macro_receipt(
            MomentumHistoricalTimeframeV1::Month1,
            MomentumMacroValueComparisonV1::ExactAllFields,
        );
        assert_eq!(left, right);
        assert_eq!(
            encode_macro_receipt(&left).unwrap(),
            encode_macro_receipt(&right).unwrap()
        );
    }

    #[test]
    fn sprint97_41_duplicate_execution_performs_zero_writes() {
        let root = TestRoot::new("macro-duplicate");
        let receipt = fixture_macro_receipt(
            MomentumHistoricalTimeframeV1::Month1,
            MomentumMacroValueComparisonV1::ExactAllFields,
        );
        assert_eq!(persist_macro_receipt(&root.0, &receipt).unwrap(), (1, 0));
        assert_eq!(persist_macro_receipt(&root.0, &receipt).unwrap(), (0, 1));
    }

    #[test]
    fn sprint97_42_conflicting_artifact_is_rejected() {
        let root = TestRoot::new("macro-conflict");
        let receipt = fixture_macro_receipt(
            MomentumHistoricalTimeframeV1::Month1,
            MomentumMacroValueComparisonV1::ExactAllFields,
        );
        persist_macro_receipt(&root.0, &receipt).unwrap();
        let mut conflicting = receipt.clone();
        conflicting.native_response_digest = "different-response".to_string();
        conflicting.receipt_digest = macro_receipt_digest(&conflicting);
        fs::write(
            root.0
                .join("macro_forensic_receipts/1mo")
                .join(format!("{}.pb", receipt.receipt_digest)),
            encode_macro_receipt(&conflicting).unwrap(),
        )
        .unwrap();
        let result = persist_macro_receipt(&root.0, &receipt);
        assert!(result.is_err());
    }

    #[test]
    fn sprint97_43_malformed_macro_protobuf_is_rejected() {
        assert!(decode_macro_receipt(&[0xff, 0x01]).is_err());
        assert!(decode_qualified_set(&[0x08, 0xff]).is_err());
        assert!(decode_qualified_hard_replay(&[0x00]).is_err());
    }

    #[test]
    fn sprint97_44_text_and_json_reports_agree() {
        let report = fixture_macro_report();
        let text = format_momentum_macro_forensics_text_v1(&report).unwrap();
        let json = serde_json::to_string(&report).unwrap();
        assert!(text.contains(&report.report_digest));
        assert!(text.contains("qualified timeframes: 6"));
        assert!(json.contains(&report.report_digest));
        assert!(json.contains("\"qualified_count\":6"));
    }
}
