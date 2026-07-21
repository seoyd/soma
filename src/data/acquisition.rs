use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File},
    io::{Read, Write},
    path::{Component, Path, PathBuf},
};

use prost::Message;
use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, Stance, stable_hash_string, stable_reason_codes};
use crate::league::{
    AgentKind, AgentProposal, AgentStatus, CanonicalAgentState, HistoricalOhlcvRow,
    HistoricalReplayDataset,
};

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub enum DatasetKind {
    DailyOhlcv,
    AdjustedDailyOhlcv,
    CorporateActions,
    QuarterlyFundamentals,
    ValuationMetrics,
    MarketIndexDaily,
    MarketBreadthDaily,
    VolatilityDaily,
    LiquidityDaily,
    CryptoDailyOhlcv,
    MacroSeries,
    #[default]
    Unknown,
}

impl DatasetKind {
    pub fn is_read_only(self) -> bool {
        self != Self::Unknown
    }
}

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub enum AcquisitionMarketScope {
    UsStocks,
    KoreanStocks,
    BtcCrypto,
    #[default]
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DataLookback {
    pub bars: usize,
    pub start_timestamp_ms: Option<u64>,
    pub end_timestamp_ms: Option<u64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataPriority {
    Required,
    Preferred,
    Optional,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentDataIntent {
    pub agent_id: String,
    pub agent_kind: AgentKind,
    pub market_scope: AcquisitionMarketScope,
    pub symbols: Vec<String>,
    pub required_datasets: Vec<DatasetKind>,
    pub optional_datasets: Vec<DatasetKind>,
    pub lookback: DataLookback,
    pub target_cadence: String,
    pub max_staleness_ms: u64,
    pub priority: DataPriority,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentDataPolicy {
    pub agent_kind: AgentKind,
    pub allowed_markets: Vec<AcquisitionMarketScope>,
    pub allowed_dataset_kinds: Vec<DatasetKind>,
    pub required_dataset_kinds: Vec<DatasetKind>,
    pub optional_dataset_kinds: Vec<DatasetKind>,
    pub default_lookback: DataLookback,
    pub max_staleness_ms: u64,
    pub request_budget: usize,
    pub abstain_when_required_missing: bool,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfiguredUniverse {
    pub symbols_by_market: BTreeMap<AcquisitionMarketScope, Vec<String>>,
}

impl ConfiguredUniverse {
    pub fn symbols_for(&self, market: AcquisitionMarketScope) -> Vec<String> {
        self.symbols_by_market
            .get(&market)
            .cloned()
            .unwrap_or_default()
    }
}

pub fn default_agent_data_policies() -> Vec<AgentDataPolicy> {
    vec![
        AgentDataPolicy {
            agent_kind: AgentKind::MomentumTrendFast,
            allowed_markets: vec![
                AcquisitionMarketScope::UsStocks,
                AcquisitionMarketScope::KoreanStocks,
                AcquisitionMarketScope::BtcCrypto,
            ],
            allowed_dataset_kinds: vec![
                DatasetKind::DailyOhlcv,
                DatasetKind::AdjustedDailyOhlcv,
                DatasetKind::VolatilityDaily,
                DatasetKind::LiquidityDaily,
                DatasetKind::CryptoDailyOhlcv,
            ],
            required_dataset_kinds: vec![DatasetKind::DailyOhlcv],
            optional_dataset_kinds: vec![
                DatasetKind::AdjustedDailyOhlcv,
                DatasetKind::VolatilityDaily,
                DatasetKind::LiquidityDaily,
            ],
            default_lookback: DataLookback {
                bars: 90,
                start_timestamp_ms: None,
                end_timestamp_ms: None,
            },
            max_staleness_ms: 86_400_000,
            request_budget: 4,
            abstain_when_required_missing: true,
            reason_codes: vec![
                ReasonCode::AgentDataPolicyApplied,
                ReasonCode::AgentInformationSetDistinct,
            ],
        },
        AgentDataPolicy {
            agent_kind: AgentKind::ValueQualityFilter,
            allowed_markets: vec![
                AcquisitionMarketScope::UsStocks,
                AcquisitionMarketScope::KoreanStocks,
            ],
            allowed_dataset_kinds: vec![
                DatasetKind::AdjustedDailyOhlcv,
                DatasetKind::QuarterlyFundamentals,
                DatasetKind::ValuationMetrics,
                DatasetKind::CorporateActions,
            ],
            required_dataset_kinds: vec![
                DatasetKind::AdjustedDailyOhlcv,
                DatasetKind::QuarterlyFundamentals,
            ],
            optional_dataset_kinds: vec![
                DatasetKind::ValuationMetrics,
                DatasetKind::CorporateActions,
            ],
            default_lookback: DataLookback {
                bars: 252,
                start_timestamp_ms: None,
                end_timestamp_ms: None,
            },
            max_staleness_ms: 7 * 86_400_000,
            request_budget: 4,
            abstain_when_required_missing: true,
            reason_codes: vec![
                ReasonCode::AgentDataPolicyApplied,
                ReasonCode::AgentInformationSetDistinct,
            ],
        },
        AgentDataPolicy {
            agent_kind: AgentKind::CycleRiskSkeptic,
            allowed_markets: vec![
                AcquisitionMarketScope::UsStocks,
                AcquisitionMarketScope::KoreanStocks,
                AcquisitionMarketScope::BtcCrypto,
            ],
            allowed_dataset_kinds: vec![
                DatasetKind::MarketIndexDaily,
                DatasetKind::VolatilityDaily,
                DatasetKind::MarketBreadthDaily,
                DatasetKind::LiquidityDaily,
                DatasetKind::CryptoDailyOhlcv,
                DatasetKind::MacroSeries,
            ],
            required_dataset_kinds: vec![
                DatasetKind::MarketIndexDaily,
                DatasetKind::VolatilityDaily,
            ],
            optional_dataset_kinds: vec![
                DatasetKind::MarketBreadthDaily,
                DatasetKind::LiquidityDaily,
                DatasetKind::MacroSeries,
            ],
            default_lookback: DataLookback {
                bars: 126,
                start_timestamp_ms: None,
                end_timestamp_ms: None,
            },
            max_staleness_ms: 86_400_000,
            request_budget: 5,
            abstain_when_required_missing: true,
            reason_codes: vec![
                ReasonCode::AgentDataPolicyApplied,
                ReasonCode::AgentInformationSetDistinct,
            ],
        },
    ]
}

pub fn plan_agent_data_intent(
    agent_id: impl Into<String>,
    agent_kind: AgentKind,
    configured_universe: &ConfiguredUniverse,
    policy: &AgentDataPolicy,
    now_ms: u64,
) -> AgentDataIntent {
    let market_scope = policy
        .allowed_markets
        .iter()
        .copied()
        .find(|market| !configured_universe.symbols_for(*market).is_empty())
        .unwrap_or(AcquisitionMarketScope::Unknown);
    let mut symbols = configured_universe.symbols_for(market_scope);
    symbols.sort();
    symbols.dedup();
    symbols.truncate(policy.request_budget.max(1));
    let mut lookback = policy.default_lookback.clone();
    lookback.end_timestamp_ms = Some(now_ms);
    AgentDataIntent {
        agent_id: agent_id.into(),
        agent_kind,
        market_scope,
        symbols,
        required_datasets: policy.required_dataset_kinds.clone(),
        optional_datasets: policy.optional_dataset_kinds.clone(),
        lookback,
        target_cadence: "1d".to_string(),
        max_staleness_ms: policy.max_staleness_ms,
        priority: DataPriority::Required,
        reason_codes: stable_reason_codes(&policy.reason_codes),
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum AcquisitionMode {
    #[default]
    Disabled,
    Mock,
    LocalSnapshotReplay,
    ApprovedReadOnlyNetwork,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum StaleDataPolicy {
    Reject,
    UseLastKnownGoodWithinTolerance,
    AbstainAgent,
    ForceNoTrade,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcquisitionPolicy {
    pub max_requests_per_cycle: usize,
    pub max_symbols_per_request: usize,
    pub max_response_bytes: usize,
    pub max_retries: usize,
    pub max_requests_per_provider: usize,
    pub stale_data_policy: StaleDataPolicy,
    pub last_known_good_tolerance_ms: u64,
    pub allow_approved_readonly_network: bool,
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for AcquisitionPolicy {
    fn default() -> Self {
        Self {
            max_requests_per_cycle: 12,
            max_symbols_per_request: 8,
            max_response_bytes: 2 * 1024 * 1024,
            max_retries: 1,
            max_requests_per_provider: 8,
            stale_data_policy: StaleDataPolicy::AbstainAgent,
            last_known_good_tolerance_ms: 7 * 86_400_000,
            allow_approved_readonly_network: false,
            reason_codes: vec![
                ReasonCode::DatasetKindReadOnly,
                ReasonCode::AcquisitionFailedClosed,
                ReasonCode::PaperExecutionOnly,
            ],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderCapabilities {
    pub provider_id: String,
    pub supported_markets: Vec<AcquisitionMarketScope>,
    pub supported_dataset_kinds: Vec<DatasetKind>,
    pub supported_cadences: Vec<String>,
    pub maximum_lookback_bars: usize,
    pub requires_credentials: bool,
    pub read_only: bool,
    pub enabled: bool,
    pub approved_for_network: bool,
    pub mock_only: bool,
    pub reason_codes: Vec<ReasonCode>,
}

impl ProviderCapabilities {
    fn supports(&self, request: &ReadOnlyProviderRequest, mode: AcquisitionMode) -> bool {
        self.enabled
            && self.read_only
            && self.supported_markets.contains(&request.market_scope)
            && self.supported_dataset_kinds.contains(&request.dataset_kind)
            && self.supported_cadences.contains(&request.cadence)
            && request.lookback.bars <= self.maximum_lookback_bars
            && match mode {
                AcquisitionMode::Mock => self.mock_only,
                AcquisitionMode::ApprovedReadOnlyNetwork => {
                    self.approved_for_network && !self.mock_only
                }
                AcquisitionMode::Disabled | AcquisitionMode::LocalSnapshotReplay => false,
            }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadOnlyProviderRegistry {
    pub providers: BTreeMap<String, ProviderCapabilities>,
    pub reason_codes: Vec<ReasonCode>,
}

impl ReadOnlyProviderRegistry {
    pub fn register(&mut self, capabilities: ProviderCapabilities) {
        self.providers
            .insert(capabilities.provider_id.clone(), capabilities);
    }

    fn select(&self, request: &ReadOnlyProviderRequest, mode: AcquisitionMode) -> Option<String> {
        self.providers
            .values()
            .find(|capabilities| capabilities.supports(request, mode))
            .map(|capabilities| capabilities.provider_id.clone())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadOnlyProviderRequest {
    pub request_id: String,
    pub request_key: String,
    pub provider_id: String,
    pub dataset_kind: DatasetKind,
    pub market_scope: AcquisitionMarketScope,
    pub symbols: Vec<String>,
    pub lookback: DataLookback,
    pub cadence: String,
    pub max_staleness_ms: u64,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReadOnlyProviderResponse {
    pub request_id: String,
    pub provider_id: String,
    pub fetched_at_ms: u64,
    pub content_type: String,
    pub all_rows_finalized: bool,
    pub normalized_dataset: HistoricalReplayDataset,
    pub reported_content_bytes: usize,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderFetchFailure {
    Unavailable,
    RateLimited,
    PermissionDenied,
    TimedOut,
    InvalidResponse,
}

pub trait ReadOnlyMarketDataProvider {
    fn capabilities(&self) -> ProviderCapabilities;
    fn fetch_readonly(
        &mut self,
        request: &ReadOnlyProviderRequest,
    ) -> Result<ReadOnlyProviderResponse, ProviderFetchFailure>;
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MockReadOnlyProvider {
    pub capabilities: ProviderCapabilities,
    pub default_response: Option<ReadOnlyProviderResponse>,
    pub default_failure: Option<ProviderFetchFailure>,
    pub requests: Vec<ReadOnlyProviderRequest>,
}

impl ReadOnlyMarketDataProvider for MockReadOnlyProvider {
    fn capabilities(&self) -> ProviderCapabilities {
        self.capabilities.clone()
    }

    fn fetch_readonly(
        &mut self,
        request: &ReadOnlyProviderRequest,
    ) -> Result<ReadOnlyProviderResponse, ProviderFetchFailure> {
        self.requests.push(request.clone());
        if let Some(failure) = &self.default_failure {
            return Err(failure.clone());
        }
        let mut response = self
            .default_response
            .clone()
            .ok_or(ProviderFetchFailure::Unavailable)?;
        response.request_id = request.request_id.clone();
        response.provider_id = request.provider_id.clone();
        Ok(response)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcquisitionRequest {
    pub request: ReadOnlyProviderRequest,
    pub requested_by_agents: Vec<String>,
    pub required_by_agents: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RejectedAcquisitionRequest {
    pub request_key: String,
    pub agent_ids: Vec<String>,
    pub dataset_kind: DatasetKind,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcquisitionPlan {
    pub planned_requests: Vec<AcquisitionRequest>,
    pub rejected_requests: Vec<RejectedAcquisitionRequest>,
    pub agent_request_mapping: BTreeMap<String, Vec<String>>,
    pub deduplicated_request_count: usize,
    pub reason_codes: Vec<ReasonCode>,
}

pub fn build_acquisition_plan(
    agent_intents: &[AgentDataIntent],
    provider_registry: &ReadOnlyProviderRegistry,
    mode: AcquisitionMode,
    policy: &AcquisitionPolicy,
) -> AcquisitionPlan {
    let mut logical =
        BTreeMap::<String, (ReadOnlyProviderRequest, BTreeSet<String>, BTreeSet<String>)>::new();
    let mut rejected_requests = Vec::new();
    for intent in agent_intents {
        for (dataset_kind, required) in intent
            .required_datasets
            .iter()
            .copied()
            .map(|kind| (kind, true))
            .chain(
                intent
                    .optional_datasets
                    .iter()
                    .copied()
                    .map(|kind| (kind, false)),
            )
        {
            let request_key = acquisition_request_key(dataset_kind, intent);
            if dataset_kind == DatasetKind::Unknown || !dataset_kind.is_read_only() {
                rejected_requests.push(RejectedAcquisitionRequest {
                    request_key,
                    agent_ids: vec![intent.agent_id.clone()],
                    dataset_kind,
                    reason_codes: vec![
                        ReasonCode::DatasetKindUnknown,
                        ReasonCode::AcquisitionRequestRejected,
                    ],
                });
                continue;
            }
            if intent.market_scope == AcquisitionMarketScope::Unknown || intent.symbols.is_empty() {
                rejected_requests.push(RejectedAcquisitionRequest {
                    request_key,
                    agent_ids: vec![intent.agent_id.clone()],
                    dataset_kind,
                    reason_codes: vec![
                        ReasonCode::EvidenceMissing,
                        ReasonCode::AcquisitionRequestRejected,
                    ],
                });
                continue;
            }
            let entry = logical.entry(request_key.clone()).or_insert_with(|| {
                let mut symbols = intent.symbols.clone();
                symbols.sort();
                symbols.dedup();
                (
                    ReadOnlyProviderRequest {
                        request_id: format!("acq-{}", stable_hash_string(&request_key)),
                        request_key,
                        provider_id: String::new(),
                        dataset_kind,
                        market_scope: intent.market_scope,
                        symbols,
                        lookback: intent.lookback.clone(),
                        cadence: intent.target_cadence.clone(),
                        max_staleness_ms: intent.max_staleness_ms,
                        reason_codes: vec![ReasonCode::AcquisitionRequestPlanned],
                    },
                    BTreeSet::new(),
                    BTreeSet::new(),
                )
            });
            entry.1.insert(intent.agent_id.clone());
            if required {
                entry.2.insert(intent.agent_id.clone());
            }
        }
    }

    let logical_count = logical.len();
    let mut planned_requests = Vec::new();
    let mut agent_request_mapping = BTreeMap::new();
    for (_, (mut request, agents, required_agents)) in logical {
        let agents = agents.into_iter().collect::<Vec<_>>();
        if request.symbols.len() > policy.max_symbols_per_request
            || planned_requests.len() >= policy.max_requests_per_cycle
        {
            rejected_requests.push(RejectedAcquisitionRequest {
                request_key: request.request_key,
                agent_ids: agents,
                dataset_kind: request.dataset_kind,
                reason_codes: vec![
                    ReasonCode::AcquisitionBudgetExceeded,
                    ReasonCode::AcquisitionRequestRejected,
                ],
            });
            continue;
        }
        if mode != AcquisitionMode::LocalSnapshotReplay {
            let Some(provider_id) = provider_registry.select(&request, mode) else {
                let reason = if mode == AcquisitionMode::ApprovedReadOnlyNetwork {
                    ReasonCode::NoApprovedReadOnlyProviderConfigured
                } else {
                    ReasonCode::DatasetKindProviderUnavailable
                };
                rejected_requests.push(RejectedAcquisitionRequest {
                    request_key: request.request_key,
                    agent_ids: agents,
                    dataset_kind: request.dataset_kind,
                    reason_codes: vec![reason, ReasonCode::AcquisitionProviderUnavailable],
                });
                continue;
            };
            request.provider_id = provider_id;
        }
        agent_request_mapping.insert(request.request_id.clone(), agents.clone());
        planned_requests.push(AcquisitionRequest {
            request,
            requested_by_agents: agents,
            required_by_agents: required_agents.into_iter().collect(),
        });
    }
    let mut reason_codes = policy.reason_codes.clone();
    if logical_count > planned_requests.len() {
        reason_codes.push(ReasonCode::AcquisitionRequestRejected);
    }
    if logical_count > planned_requests.len() + rejected_requests.len() {
        reason_codes.push(ReasonCode::AcquisitionRequestDeduplicated);
    }
    if mode == AcquisitionMode::ApprovedReadOnlyNetwork
        && planned_requests
            .iter()
            .all(|request| request.request.provider_id.is_empty())
    {
        reason_codes.push(ReasonCode::NoApprovedReadOnlyProviderConfigured);
    }
    AcquisitionPlan {
        planned_requests,
        rejected_requests,
        agent_request_mapping,
        deduplicated_request_count: agent_intents
            .iter()
            .map(|intent| intent.required_datasets.len() + intent.optional_datasets.len())
            .sum::<usize>()
            .saturating_sub(logical_count),
        reason_codes: stable_reason_codes(&reason_codes),
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotProvenance {
    pub provider_id: String,
    pub acquisition_request_id: String,
    pub fetch_receipt_id: String,
    pub source_type: SnapshotSourceType,
    pub sanitized: bool,
    pub credential_free: bool,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SnapshotSourceType {
    Mock,
    LocalSnapshotReplay,
    ApprovedReadOnlyProvider,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotQualitySummary {
    pub accepted: bool,
    pub row_count: usize,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SnapshotAdjustmentSemanticsV1 {
    Unadjusted,
    Adjusted,
    NotApplicable,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotCompatibilityV1 {
    pub cadence: String,
    pub adjustment_semantics: SnapshotAdjustmentSemanticsV1,
    pub source_schema: String,
    pub requested_cutoff_timestamp_ms: Option<u64>,
    pub maximum_staleness_ms: u64,
    pub all_rows_finalized: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DataSnapshot {
    pub snapshot_id: String,
    pub request_key: String,
    pub provider_id: String,
    pub dataset_kind: DatasetKind,
    pub market_scope: AcquisitionMarketScope,
    pub symbols: Vec<String>,
    pub requested_lookback: DataLookback,
    pub actual_start_timestamp_ms: Option<u64>,
    pub actual_end_timestamp_ms: Option<u64>,
    pub fetched_at_ms: u64,
    pub normalized_at_ms: u64,
    pub schema_version: u32,
    pub row_count: usize,
    pub quality_summary: SnapshotQualitySummary,
    pub content_digest: String,
    pub sanitized: bool,
    pub read_only: bool,
    #[serde(default)]
    pub compatibility: Option<SnapshotCompatibilityV1>,
    pub normalized_dataset: HistoricalReplayDataset,
    pub provenance: SnapshotProvenance,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct InMemorySnapshotStore {
    snapshots: BTreeMap<String, DataSnapshot>,
    latest_by_request_key: BTreeMap<String, String>,
}

impl InMemorySnapshotStore {
    pub fn put(&mut self, snapshot: DataSnapshot) -> Result<(), ReasonCode> {
        if !self.verify_snapshot(&snapshot) {
            return Err(ReasonCode::DataSnapshotDigestMismatch);
        }
        if self.snapshots.contains_key(&snapshot.snapshot_id) {
            return Err(ReasonCode::DataSnapshotImmutable);
        }
        self.latest_by_request_key
            .insert(snapshot.request_key.clone(), snapshot.snapshot_id.clone());
        self.snapshots
            .insert(snapshot.snapshot_id.clone(), snapshot);
        Ok(())
    }

    pub fn get(&self, snapshot_id: &str) -> Option<DataSnapshot> {
        self.snapshots.get(snapshot_id).cloned()
    }

    pub fn find_latest(&self, request_key: &str) -> Option<DataSnapshot> {
        self.latest_by_request_key
            .get(request_key)
            .and_then(|snapshot_id| self.get(snapshot_id))
    }

    fn find_latest_compatible(&self, request: &ReadOnlyProviderRequest) -> Option<DataSnapshot> {
        self.snapshots
            .values()
            .filter(|snapshot| snapshot_is_compatible_fallback(snapshot, request))
            .max_by_key(|snapshot| snapshot.fetched_at_ms)
            .cloned()
    }

    pub fn verify_digest(&self, snapshot_id: &str) -> bool {
        self.get(snapshot_id)
            .is_some_and(|snapshot| self.verify_snapshot(&snapshot))
    }

    fn verify_snapshot(&self, snapshot: &DataSnapshot) -> bool {
        snapshot.content_digest == snapshot_digest(&snapshot.normalized_dataset)
    }
}

fn snapshot_is_compatible_fallback(
    snapshot: &DataSnapshot,
    request: &ReadOnlyProviderRequest,
) -> bool {
    let Some(compatibility) = snapshot.compatibility.as_ref() else {
        return false;
    };
    snapshot.dataset_kind == request.dataset_kind
        && snapshot.market_scope == request.market_scope
        && snapshot.symbols == request.symbols
        && snapshot.requested_lookback == request.lookback
        && compatibility.cadence == request.cadence
        && compatibility.adjustment_semantics == adjustment_semantics_v1(request.dataset_kind)
        && compatibility.source_schema == "application/x-soma-normalized-dataset"
        && compatibility.requested_cutoff_timestamp_ms == request.lookback.end_timestamp_ms
        && compatibility.maximum_staleness_ms == request.max_staleness_ms
        && compatibility.all_rows_finalized
        && snapshot.schema_version == 1
        && snapshot.quality_summary.accepted
        && snapshot.row_count == snapshot.normalized_dataset.rows.len()
        && snapshot.quality_summary.row_count == snapshot.row_count
        && snapshot.sanitized
        && snapshot.read_only
        && snapshot.provenance.provider_id == snapshot.provider_id
        && snapshot.provenance.sanitized
        && snapshot.provenance.credential_free
        && snapshot.provenance.source_type != SnapshotSourceType::LocalSnapshotReplay
        && snapshot.actual_start_timestamp_ms
            == snapshot
                .normalized_dataset
                .rows
                .first()
                .map(|row| row.timestamp_ms)
        && snapshot.actual_end_timestamp_ms
            == snapshot
                .normalized_dataset
                .rows
                .last()
                .map(|row| row.timestamp_ms)
        && snapshot.actual_end_timestamp_ms.is_some_and(|actual_end| {
            request
                .lookback
                .end_timestamp_ms
                .is_some_and(|requested_end| actual_end <= requested_end)
        })
        && snapshot
            .normalized_dataset
            .rows
            .iter()
            .all(|row| row.symbol == snapshot.normalized_dataset.symbol)
        && validate_normalized_dataset(&snapshot.normalized_dataset).is_ok()
        && snapshot.content_digest == snapshot_digest(&snapshot.normalized_dataset)
}

fn adjustment_semantics_v1(dataset_kind: DatasetKind) -> SnapshotAdjustmentSemanticsV1 {
    match dataset_kind {
        DatasetKind::DailyOhlcv | DatasetKind::CryptoDailyOhlcv => {
            SnapshotAdjustmentSemanticsV1::Unadjusted
        }
        DatasetKind::AdjustedDailyOhlcv => SnapshotAdjustmentSemanticsV1::Adjusted,
        DatasetKind::CorporateActions
        | DatasetKind::QuarterlyFundamentals
        | DatasetKind::ValuationMetrics
        | DatasetKind::MarketIndexDaily
        | DatasetKind::MarketBreadthDaily
        | DatasetKind::VolatilityDaily
        | DatasetKind::LiquidityDaily
        | DatasetKind::MacroSeries
        | DatasetKind::Unknown => SnapshotAdjustmentSemanticsV1::NotApplicable,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AcquisitionReceiptStatus {
    Acquired,
    ReusedSnapshot,
    Rejected,
    Failed,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcquisitionReceipt {
    pub receipt_id: String,
    pub request_id: String,
    pub provider_id: Option<String>,
    pub status: AcquisitionReceiptStatus,
    pub snapshot_id: Option<String>,
    pub attempt_count: usize,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct BrokerExecutionResult {
    pub receipts: Vec<AcquisitionReceipt>,
    pub new_snapshots: Vec<DataSnapshot>,
    pub reused_snapshots: Vec<DataSnapshot>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DataAcquisitionBroker {
    pub provider_registry: ReadOnlyProviderRegistry,
    pub acquisition_policy: AcquisitionPolicy,
    pub snapshot_store: InMemorySnapshotStore,
    pub reason_codes: Vec<ReasonCode>,
}

impl DataAcquisitionBroker {
    pub fn new(
        provider_registry: ReadOnlyProviderRegistry,
        acquisition_policy: AcquisitionPolicy,
    ) -> Self {
        Self {
            provider_registry,
            acquisition_policy,
            snapshot_store: InMemorySnapshotStore::default(),
            reason_codes: vec![
                ReasonCode::DatasetKindReadOnly,
                ReasonCode::AcquisitionFailedClosed,
            ],
        }
    }

    pub fn execute_acquisition_plan(
        &mut self,
        plan: &AcquisitionPlan,
        mode: AcquisitionMode,
        now_ms: u64,
        mut provider: Option<&mut dyn ReadOnlyMarketDataProvider>,
    ) -> BrokerExecutionResult {
        let mut result = BrokerExecutionResult::default();
        for rejected in &plan.rejected_requests {
            result.receipts.push(AcquisitionReceipt {
                receipt_id: format!("receipt-{}", stable_hash_string(&rejected.request_key)),
                request_id: rejected.request_key.clone(),
                provider_id: None,
                status: AcquisitionReceiptStatus::Rejected,
                snapshot_id: None,
                attempt_count: 0,
                reason_codes: rejected.reason_codes.clone(),
            });
        }
        let mut calls_by_provider = BTreeMap::<String, usize>::new();
        for planned in &plan.planned_requests {
            let request = &planned.request;
            if mode == AcquisitionMode::LocalSnapshotReplay {
                self.replay_snapshot(request, now_ms, &mut result);
                continue;
            }
            if mode == AcquisitionMode::Disabled {
                result.receipts.push(failed_receipt(
                    request,
                    None,
                    0,
                    vec![
                        ReasonCode::EvidenceMissing,
                        ReasonCode::AcquisitionFailedClosed,
                    ],
                ));
                continue;
            }
            if mode == AcquisitionMode::ApprovedReadOnlyNetwork
                && !self.acquisition_policy.allow_approved_readonly_network
            {
                result.receipts.push(failed_receipt(
                    request,
                    Some(request.provider_id.clone()),
                    0,
                    vec![
                        ReasonCode::ApprovedProviderPilotDisabled,
                        ReasonCode::NoApprovedReadOnlyProviderConfigured,
                        ReasonCode::AcquisitionFailedClosed,
                    ],
                ));
                continue;
            }
            let count = calls_by_provider
                .entry(request.provider_id.clone())
                .or_default();
            if *count >= self.acquisition_policy.max_requests_per_provider {
                result.receipts.push(failed_receipt(
                    request,
                    Some(request.provider_id.clone()),
                    0,
                    vec![
                        ReasonCode::AcquisitionRateLimited,
                        ReasonCode::AcquisitionFailedClosed,
                    ],
                ));
                continue;
            }
            *count += 1;
            let Some(provider) = provider.as_deref_mut() else {
                result.receipts.push(failed_receipt(
                    request,
                    Some(request.provider_id.clone()),
                    0,
                    vec![
                        ReasonCode::AcquisitionProviderUnavailable,
                        ReasonCode::AcquisitionFailedClosed,
                    ],
                ));
                continue;
            };
            if provider.capabilities().provider_id != request.provider_id {
                result.receipts.push(failed_receipt(
                    request,
                    Some(request.provider_id.clone()),
                    0,
                    vec![
                        ReasonCode::AcquisitionProviderUnavailable,
                        ReasonCode::AcquisitionFailedClosed,
                    ],
                ));
                continue;
            }
            self.fetch_snapshot(request, mode, now_ms, provider, &mut result);
        }
        result.reason_codes = stable_reason_codes(
            &result
                .receipts
                .iter()
                .flat_map(|receipt| receipt.reason_codes.iter().cloned())
                .chain([ReasonCode::AcquisitionFailedClosed])
                .collect::<Vec<_>>(),
        );
        result
    }

    fn fetch_snapshot(
        &mut self,
        request: &ReadOnlyProviderRequest,
        mode: AcquisitionMode,
        now_ms: u64,
        provider: &mut dyn ReadOnlyMarketDataProvider,
        result: &mut BrokerExecutionResult,
    ) {
        let mut attempts = 0;
        let response = loop {
            attempts += 1;
            match provider.fetch_readonly(request) {
                Ok(response) => break Ok(response),
                Err(ProviderFetchFailure::Unavailable | ProviderFetchFailure::TimedOut)
                    if attempts <= self.acquisition_policy.max_retries =>
                {
                    continue;
                }
                Err(error) => break Err(error),
            }
        };
        let response = match response {
            Ok(response) => response,
            Err(error) => {
                let reason = match error {
                    ProviderFetchFailure::RateLimited => ReasonCode::AcquisitionRateLimited,
                    ProviderFetchFailure::PermissionDenied => {
                        ReasonCode::AcquisitionPermissionDenied
                    }
                    ProviderFetchFailure::TimedOut => ReasonCode::AcquisitionTimedOut,
                    ProviderFetchFailure::Unavailable | ProviderFetchFailure::InvalidResponse => {
                        ReasonCode::AcquisitionProviderUnavailable
                    }
                };
                result.receipts.push(failed_receipt(
                    request,
                    Some(request.provider_id.clone()),
                    attempts,
                    vec![reason, ReasonCode::AcquisitionFailedClosed],
                ));
                return;
            }
        };
        let snapshot = match snapshot_from_response(
            request,
            response,
            mode,
            now_ms,
            self.acquisition_policy.max_response_bytes,
        ) {
            Ok(snapshot) => snapshot,
            Err(reason) => {
                result.receipts.push(failed_receipt(
                    request,
                    Some(request.provider_id.clone()),
                    attempts,
                    vec![reason, ReasonCode::AcquisitionFailedClosed],
                ));
                return;
            }
        };
        match self.snapshot_store.put(snapshot.clone()) {
            Ok(()) => {
                result.receipts.push(AcquisitionReceipt {
                    receipt_id: format!("receipt-{}", stable_hash_string(&request.request_id)),
                    request_id: request.request_id.clone(),
                    provider_id: Some(request.provider_id.clone()),
                    status: AcquisitionReceiptStatus::Acquired,
                    snapshot_id: Some(snapshot.snapshot_id.clone()),
                    attempt_count: attempts,
                    reason_codes: vec![
                        ReasonCode::AcquisitionProviderSelected,
                        ReasonCode::DataSnapshotCreated,
                        ReasonCode::EvidenceFresh,
                    ],
                });
                result.new_snapshots.push(snapshot);
            }
            Err(reason) => result.receipts.push(failed_receipt(
                request,
                Some(request.provider_id.clone()),
                attempts,
                vec![reason, ReasonCode::AcquisitionFailedClosed],
            )),
        }
    }

    fn replay_snapshot(
        &self,
        request: &ReadOnlyProviderRequest,
        now_ms: u64,
        result: &mut BrokerExecutionResult,
    ) {
        let Some(snapshot) = self
            .snapshot_store
            .find_latest(&request.request_key)
            .or_else(|| self.snapshot_store.find_latest_compatible(request))
        else {
            result.receipts.push(failed_receipt(
                request,
                None,
                0,
                vec![
                    ReasonCode::EvidenceMissing,
                    ReasonCode::AcquisitionFailedClosed,
                ],
            ));
            return;
        };
        if !self.snapshot_store.verify_digest(&snapshot.snapshot_id) {
            result.receipts.push(failed_receipt(
                request,
                Some(snapshot.provider_id),
                0,
                vec![
                    ReasonCode::DataSnapshotDigestMismatch,
                    ReasonCode::AcquisitionFailedClosed,
                ],
            ));
            return;
        }
        let freshness = assess_freshness(
            &snapshot,
            now_ms,
            request.max_staleness_ms,
            &self.acquisition_policy,
        );
        if freshness == EvidenceFreshnessStatus::StaleRejected {
            result.receipts.push(failed_receipt(
                request,
                Some(snapshot.provider_id),
                0,
                vec![
                    ReasonCode::EvidenceStaleRejected,
                    ReasonCode::AcquisitionFailedClosed,
                ],
            ));
            return;
        }
        let reason = if freshness == EvidenceFreshnessStatus::Fresh {
            ReasonCode::EvidenceFresh
        } else {
            ReasonCode::EvidenceLastKnownGoodUsed
        };
        result.receipts.push(AcquisitionReceipt {
            receipt_id: format!("receipt-{}", stable_hash_string(&request.request_id)),
            request_id: request.request_id.clone(),
            provider_id: Some(snapshot.provider_id.clone()),
            status: AcquisitionReceiptStatus::ReusedSnapshot,
            snapshot_id: Some(snapshot.snapshot_id.clone()),
            attempt_count: 0,
            reason_codes: if freshness == EvidenceFreshnessStatus::Fresh {
                vec![ReasonCode::DataSnapshotReused, reason]
            } else {
                vec![
                    ReasonCode::DataSnapshotReused,
                    reason,
                    ReasonCode::EvidenceStaleWithinTolerance,
                ]
            },
        });
        result.reused_snapshots.push(snapshot);
    }
}

fn failed_receipt(
    request: &ReadOnlyProviderRequest,
    provider_id: Option<String>,
    attempt_count: usize,
    reason_codes: Vec<ReasonCode>,
) -> AcquisitionReceipt {
    AcquisitionReceipt {
        receipt_id: format!("receipt-{}", stable_hash_string(&request.request_id)),
        request_id: request.request_id.clone(),
        provider_id,
        status: AcquisitionReceiptStatus::Failed,
        snapshot_id: None,
        attempt_count,
        reason_codes: stable_reason_codes(&reason_codes),
    }
}

fn snapshot_from_response(
    request: &ReadOnlyProviderRequest,
    response: ReadOnlyProviderResponse,
    mode: AcquisitionMode,
    normalized_at_ms: u64,
    max_response_bytes: usize,
) -> Result<DataSnapshot, ReasonCode> {
    if response.request_id != request.request_id
        || response.provider_id != request.provider_id
        || response.content_type != "application/x-soma-normalized-dataset"
        || !response.all_rows_finalized
    {
        return Err(ReasonCode::DataSnapshotUnsafeContentRejected);
    }
    let serialized = serde_json::to_string(&response.normalized_dataset)
        .map_err(|_| ReasonCode::DataSnapshotUnsafeContentRejected)?;
    if response.reported_content_bytes > max_response_bytes || serialized.len() > max_response_bytes
    {
        return Err(ReasonCode::AcquisitionResponseTooLarge);
    }
    validate_normalized_dataset(&response.normalized_dataset)?;
    let digest = historical_replay_dataset_digest_v0(&response.normalized_dataset);
    let snapshot_id = snapshot_id_from_semantic_digest_v1(&digest);
    let timestamps = response
        .normalized_dataset
        .rows
        .iter()
        .map(|row| row.timestamp_ms)
        .collect::<Vec<_>>();
    Ok(DataSnapshot {
        snapshot_id,
        request_key: request.request_key.clone(),
        provider_id: request.provider_id.clone(),
        dataset_kind: request.dataset_kind,
        market_scope: request.market_scope,
        symbols: request.symbols.clone(),
        requested_lookback: request.lookback.clone(),
        actual_start_timestamp_ms: timestamps.iter().min().copied(),
        actual_end_timestamp_ms: timestamps.iter().max().copied(),
        fetched_at_ms: response.fetched_at_ms,
        normalized_at_ms,
        schema_version: 1,
        row_count: response.normalized_dataset.rows.len(),
        quality_summary: SnapshotQualitySummary {
            accepted: true,
            row_count: response.normalized_dataset.rows.len(),
            reason_codes: vec![ReasonCode::CsvLoaded],
        },
        content_digest: digest,
        sanitized: true,
        read_only: true,
        compatibility: Some(SnapshotCompatibilityV1 {
            cadence: request.cadence.clone(),
            adjustment_semantics: adjustment_semantics_v1(request.dataset_kind),
            source_schema: response.content_type,
            requested_cutoff_timestamp_ms: request.lookback.end_timestamp_ms,
            maximum_staleness_ms: request.max_staleness_ms,
            all_rows_finalized: response.all_rows_finalized,
        }),
        normalized_dataset: response.normalized_dataset,
        provenance: SnapshotProvenance {
            provider_id: request.provider_id.clone(),
            acquisition_request_id: request.request_id.clone(),
            fetch_receipt_id: format!("receipt-{}", stable_hash_string(&request.request_id)),
            source_type: match mode {
                AcquisitionMode::Mock => SnapshotSourceType::Mock,
                AcquisitionMode::ApprovedReadOnlyNetwork => {
                    SnapshotSourceType::ApprovedReadOnlyProvider
                }
                AcquisitionMode::Disabled | AcquisitionMode::LocalSnapshotReplay => {
                    return Err(ReasonCode::DataSnapshotUnsafeContentRejected);
                }
            },
            sanitized: true,
            credential_free: true,
            reason_codes: vec![
                ReasonCode::DatasetKindReadOnly,
                ReasonCode::DataSnapshotCreated,
            ],
        },
        reason_codes: vec![
            ReasonCode::DataSnapshotCreated,
            ReasonCode::DataSnapshotImmutable,
            ReasonCode::DatasetKindReadOnly,
        ],
    })
}

fn validate_normalized_dataset(dataset: &HistoricalReplayDataset) -> Result<(), ReasonCode> {
    if snapshot_text_is_unsafe(&format!("{} {}", dataset.symbol, dataset.source)) {
        return Err(ReasonCode::DataSnapshotUnsafeContentRejected);
    }
    if dataset.rows.is_empty() {
        return Err(ReasonCode::EvidenceMissing);
    }
    let mut previous_timestamp = None;
    for row in &dataset.rows {
        if snapshot_text_is_unsafe(&row.symbol) || !valid_ohlcv(row) {
            return Err(ReasonCode::DataSnapshotUnsafeContentRejected);
        }
        if previous_timestamp.is_some_and(|previous| row.timestamp_ms <= previous) {
            return Err(ReasonCode::DataSnapshotUnsafeContentRejected);
        }
        previous_timestamp = Some(row.timestamp_ms);
    }
    Ok(())
}

fn snapshot_text_is_unsafe(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    [
        "authorization",
        "bearer",
        "account_id",
        "order_id",
        "api_key",
        "app_secret",
        "access_token",
        "refresh_token",
        "private_key",
        "wallet_private_key",
        "raw_response",
        "local_private",
        ".env",
        "http://",
        "https://",
        "ws://",
        "wss://",
        "ftp://",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
}

fn valid_ohlcv(row: &HistoricalOhlcvRow) -> bool {
    row.open.is_finite()
        && row.high.is_finite()
        && row.low.is_finite()
        && row.close.is_finite()
        && row.volume.is_finite()
        && row.open > 0.0
        && row.high >= row.open.max(row.close)
        && row.low <= row.open.min(row.close)
        && row.low > 0.0
        && row.volume >= 0.0
        && row
            .trade_value
            .is_none_or(|value| value.is_finite() && value >= 0.0)
}

pub fn historical_replay_dataset_digest_v0(dataset: &HistoricalReplayDataset) -> String {
    let mut material = Vec::with_capacity(dataset.rows.len().saturating_mul(80));
    material.extend_from_slice(b"SOMA-HISTORICAL-DATASET-SEMANTIC-V1");
    canonical_string(&mut material, 1, &dataset.symbol);
    canonical_string(&mut material, 2, &dataset.source);
    let mut reasons = dataset
        .reason_codes
        .iter()
        .map(|reason| *reason as u16)
        .collect::<Vec<_>>();
    reasons.sort_unstable();
    reasons.dedup();
    canonical_u32(&mut material, 3, reasons.len() as u32);
    for reason in reasons {
        canonical_u16(&mut material, 4, reason);
    }
    canonical_u32(&mut material, 5, dataset.rows.len() as u32);
    for row in &dataset.rows {
        canonical_string(&mut material, 6, &row.symbol);
        canonical_u64(&mut material, 7, row.timestamp_ms);
        canonical_f64(&mut material, 8, row.open);
        canonical_f64(&mut material, 9, row.high);
        canonical_f64(&mut material, 10, row.low);
        canonical_f64(&mut material, 11, row.close);
        canonical_f64(&mut material, 12, row.volume);
        match row.trade_value {
            Some(value) => {
                canonical_u8(&mut material, 13, 1);
                canonical_f64(&mut material, 14, value);
            }
            None => canonical_u8(&mut material, 13, 0),
        }
    }
    canonical_hash_hex(&material)
}

/// The semantic identity is the normalized, validated historical dataset.  It deliberately
/// excludes the storage container and the derived snapshot identifier.
pub fn canonical_snapshot_semantic_digest_v1(snapshot: &DataSnapshot) -> String {
    historical_replay_dataset_digest_v0(&snapshot.normalized_dataset)
}

pub fn snapshot_id_from_semantic_digest_v1(semantic_digest: &str) -> String {
    format!("snapshot-{semantic_digest}")
}

fn canonical_field(material: &mut Vec<u8>, tag: u8, bytes: &[u8]) {
    material.push(tag);
    material.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
    material.extend_from_slice(bytes);
}

fn canonical_string(material: &mut Vec<u8>, tag: u8, value: &str) {
    canonical_field(material, tag, value.as_bytes());
}

fn canonical_u8(material: &mut Vec<u8>, tag: u8, value: u8) {
    canonical_field(material, tag, &[value]);
}

fn canonical_u16(material: &mut Vec<u8>, tag: u8, value: u16) {
    canonical_field(material, tag, &value.to_be_bytes());
}

fn canonical_u32(material: &mut Vec<u8>, tag: u8, value: u32) {
    canonical_field(material, tag, &value.to_be_bytes());
}

fn canonical_u64(material: &mut Vec<u8>, tag: u8, value: u64) {
    canonical_field(material, tag, &value.to_be_bytes());
}

fn canonical_f64(material: &mut Vec<u8>, tag: u8, value: f64) {
    let bits = if value == 0.0 {
        0.0f64.to_bits()
    } else {
        value.to_bits()
    };
    canonical_field(material, tag, &bits.to_be_bytes());
}

pub(crate) fn canonical_hash_hex(bytes: &[u8]) -> String {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{hash:016x}")
}

const LEARNING_INTENT_VERSION_V0: &str = "agent-learning-intent-v0";
const LEARNING_VIEW_VERSION_V0: &str = "agent-learning-data-view-v0";
const LEARNING_ENVELOPE_MAGIC_V0: &str = "SOMA-LEARNING-PB-V0";
const LEARNING_ENVELOPE_SCHEMA_V0: &str = "soma.agent_learning_data_view.v0";
const LEARNING_VIEW_ARTIFACT_KIND_V0: &str = "agent-learning-data-view-v0";
const LEARNING_DATA_NAMESPACE_V0: &str = "state/learning_data";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LearningDataVisibilityV0 {
    SharedCanonicalRaw,
    AgentAuthorizedRaw,
    AgentPrivateDerived,
    CommitteeVisibleSummary,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LearningDataCallerV0 {
    Agent(String),
    NeutralBroker,
    Chair,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LearningDataAuthorityActionV0 {
    CreateIntent,
    CallBroker,
    SelectProvider,
    ModifyView,
    ChangeCutoff,
    SelectLabel,
    ReadPrivateArtifact,
}

pub fn authorize_learning_data_action_v0(
    caller: &LearningDataCallerV0,
    action: LearningDataAuthorityActionV0,
) -> bool {
    match caller {
        LearningDataCallerV0::Agent(agent_id) => {
            active_learning_agent_id_v0(agent_id)
                && matches!(
                    action,
                    LearningDataAuthorityActionV0::CreateIntent
                        | LearningDataAuthorityActionV0::ModifyView
                        | LearningDataAuthorityActionV0::ChangeCutoff
                        | LearningDataAuthorityActionV0::SelectLabel
                        | LearningDataAuthorityActionV0::ReadPrivateArtifact
                )
        }
        LearningDataCallerV0::NeutralBroker => matches!(
            action,
            LearningDataAuthorityActionV0::CallBroker
                | LearningDataAuthorityActionV0::SelectProvider
        ),
        LearningDataCallerV0::Chair => false,
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentLearningIntentV0 {
    pub intent_version: String,
    pub agent_id: String,
    pub agent_kind: AgentKind,
    pub market_scopes: Vec<AcquisitionMarketScope>,
    pub symbols: Vec<String>,
    pub required_datasets: Vec<DatasetKind>,
    pub optional_datasets: Vec<DatasetKind>,
    pub cadence: String,
    pub lookback: DataLookback,
    pub information_cutoff_ms: u64,
    pub maximum_staleness_ms: u64,
    pub source_policy_digest: String,
    pub feature_policy_digest: String,
    pub label_policy_digest: String,
    pub curriculum_policy_digest: String,
    pub intent_digest: String,
}

pub fn create_agent_learning_intent_v0(
    caller: &LearningDataCallerV0,
    data_intent: &AgentDataIntent,
    policy: &AgentDataPolicy,
    information_cutoff_ms: u64,
) -> Result<AgentLearningIntentV0, String> {
    if caller != &LearningDataCallerV0::Agent(data_intent.agent_id.clone())
        || !authorize_learning_data_action_v0(caller, LearningDataAuthorityActionV0::CreateIntent)
    {
        return Err("learning intent caller lacks agent authority".into());
    }
    let mut intent = AgentLearningIntentV0 {
        intent_version: LEARNING_INTENT_VERSION_V0.into(),
        agent_id: data_intent.agent_id.clone(),
        agent_kind: data_intent.agent_kind,
        market_scopes: vec![data_intent.market_scope],
        symbols: data_intent.symbols.clone(),
        required_datasets: data_intent.required_datasets.clone(),
        optional_datasets: data_intent.optional_datasets.clone(),
        cadence: data_intent.target_cadence.clone(),
        lookback: data_intent.lookback.clone(),
        information_cutoff_ms,
        maximum_staleness_ms: data_intent.max_staleness_ms,
        source_policy_digest: learning_policy_digest_v0("source", policy),
        feature_policy_digest: learning_policy_digest_v0("feature", policy),
        label_policy_digest: learning_policy_digest_v0("label", policy),
        curriculum_policy_digest: learning_policy_digest_v0("curriculum", policy),
        intent_digest: String::new(),
    };
    stabilize_learning_intent_v0(&mut intent);
    intent.intent_digest = agent_learning_intent_digest_v0(&intent);
    validate_agent_learning_intent_v0(&intent, policy)?;
    Ok(intent)
}

pub fn derive_active_agent_learning_intents_v0(
    active_agent_states: &[CanonicalAgentState],
    configured_universe: &ConfiguredUniverse,
    policies: &[AgentDataPolicy],
    information_cutoff_ms: u64,
) -> Result<Vec<AgentLearningIntentV0>, String> {
    if active_agent_states.len() != 3 {
        return Err("learning data plane requires exactly three active agents".into());
    }
    let mut intents = active_agent_states
        .iter()
        .map(|state| {
            let policy = policies
                .iter()
                .find(|policy| policy.agent_kind == state.kind)
                .ok_or_else(|| "active agent learning policy unavailable".to_string())?;
            let base = plan_agent_data_intent(
                state.agent_id.clone(),
                state.kind,
                configured_universe,
                policy,
                information_cutoff_ms,
            );
            create_agent_learning_intent_v0(
                &LearningDataCallerV0::Agent(state.agent_id.clone()),
                &base,
                policy,
                information_cutoff_ms,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    intents.sort_by(|left, right| left.agent_id.cmp(&right.agent_id));
    if intents
        .iter()
        .map(|intent| intent.intent_digest.as_str())
        .collect::<BTreeSet<_>>()
        .len()
        != intents.len()
    {
        return Err("active agent learning intents are not independent".into());
    }
    Ok(intents)
}

pub fn validate_agent_learning_intent_v0(
    intent: &AgentLearningIntentV0,
    policy: &AgentDataPolicy,
) -> Result<(), String> {
    let mut stabilized = intent.clone();
    stabilize_learning_intent_v0(&mut stabilized);
    if &stabilized != intent
        || intent.intent_version != LEARNING_INTENT_VERSION_V0
        || expected_learning_agent_id_v0(intent.agent_kind) != Some(intent.agent_id.as_str())
        || policy.agent_kind != intent.agent_kind
        || intent.market_scopes.is_empty()
        || intent
            .market_scopes
            .iter()
            .any(|market| *market == AcquisitionMarketScope::Unknown)
        || intent.symbols.is_empty()
        || intent.required_datasets.is_empty()
        || intent
            .required_datasets
            .iter()
            .chain(intent.optional_datasets.iter())
            .any(|dataset| *dataset == DatasetKind::Unknown || !dataset.is_read_only())
        || intent
            .market_scopes
            .iter()
            .any(|market| !policy.allowed_markets.contains(market))
        || intent
            .required_datasets
            .iter()
            .chain(intent.optional_datasets.iter())
            .any(|dataset| !policy.allowed_dataset_kinds.contains(dataset))
        || intent.cadence.trim().is_empty()
        || intent.lookback.bars == 0
        || intent.information_cutoff_ms == 0
        || intent
            .lookback
            .end_timestamp_ms
            .is_some_and(|end| end > intent.information_cutoff_ms)
        || intent.maximum_staleness_ms != policy.max_staleness_ms
        || intent.source_policy_digest != learning_policy_digest_v0("source", policy)
        || intent.feature_policy_digest != learning_policy_digest_v0("feature", policy)
        || intent.label_policy_digest != learning_policy_digest_v0("label", policy)
        || intent.curriculum_policy_digest != learning_policy_digest_v0("curriculum", policy)
        || intent.intent_digest != agent_learning_intent_digest_v0(intent)
    {
        return Err("agent learning intent validation failed".into());
    }
    Ok(())
}

pub fn build_learning_acquisition_plan_v0(
    intents: &[AgentLearningIntentV0],
    policies: &[AgentDataPolicy],
    provider_registry: &ReadOnlyProviderRegistry,
    mode: AcquisitionMode,
    acquisition_policy: &AcquisitionPolicy,
) -> Result<AcquisitionPlan, String> {
    if !authorize_learning_data_action_v0(
        &LearningDataCallerV0::NeutralBroker,
        LearningDataAuthorityActionV0::CallBroker,
    ) {
        return Err("neutral learning broker authority unavailable".into());
    }
    let mut acquisition_intents = Vec::new();
    for intent in intents {
        let policy = policies
            .iter()
            .find(|policy| policy.agent_kind == intent.agent_kind)
            .ok_or_else(|| "learning broker policy unavailable".to_string())?;
        validate_agent_learning_intent_v0(intent, policy)?;
        for market_scope in &intent.market_scopes {
            acquisition_intents.push(AgentDataIntent {
                agent_id: intent.agent_id.clone(),
                agent_kind: intent.agent_kind,
                market_scope: *market_scope,
                symbols: intent.symbols.clone(),
                required_datasets: intent.required_datasets.clone(),
                optional_datasets: intent.optional_datasets.clone(),
                lookback: intent.lookback.clone(),
                target_cadence: intent.cadence.clone(),
                max_staleness_ms: intent.maximum_staleness_ms,
                priority: DataPriority::Required,
                reason_codes: vec![ReasonCode::AgentDataPolicyApplied],
            });
        }
    }
    Ok(build_acquisition_plan(
        &acquisition_intents,
        provider_registry,
        mode,
        acquisition_policy,
    ))
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LearningDataArtifactRefV0 {
    pub artifact_digest: String,
    pub dataset_kind: DatasetKind,
    pub visibility: LearningDataVisibilityV0,
    pub owner_agent_id: Option<String>,
    pub maximum_event_timestamp_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentPrivateLearningStateV0 {
    pub agent_id: String,
    pub private_namespace_digest: String,
    pub training_ledger_digest: String,
}

pub fn derive_agent_private_learning_state_v0(
    intent: &AgentLearningIntentV0,
) -> AgentPrivateLearningStateV0 {
    AgentPrivateLearningStateV0 {
        agent_id: intent.agent_id.clone(),
        private_namespace_digest: stable_hash_string(&format!(
            "SOMA-AGENT-PRIVATE-NAMESPACE-V0:{}:{}",
            intent.agent_id, intent.intent_digest
        )),
        training_ledger_digest: stable_hash_string(&format!(
            "SOMA-AGENT-TRAINING-LEDGER-V0:{}:{}:{}",
            intent.agent_id, intent.feature_policy_digest, intent.curriculum_policy_digest
        )),
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentLearningDataViewV0 {
    pub view_version: String,
    pub agent_id: String,
    pub source_artifact_digests: Vec<String>,
    pub visible_dataset_kinds: Vec<DatasetKind>,
    pub information_cutoff_ms: u64,
    pub feature_policy_digest: String,
    pub label_policy_digest: String,
    pub curriculum_policy_digest: String,
    pub private_namespace_digest: String,
    pub training_ledger_digest: String,
    pub shared_raw_count: usize,
    pub private_artifact_count: usize,
    pub missing_required_datasets: Vec<DatasetKind>,
    pub decision_gate: EvidenceDecisionGate,
    pub view_digest: String,
}

pub fn build_agent_learning_data_view_v0(
    intent: &AgentLearningIntentV0,
    policy: &AgentDataPolicy,
    artifacts: &[LearningDataArtifactRefV0],
    private_state: &AgentPrivateLearningStateV0,
) -> Result<AgentLearningDataViewV0, String> {
    validate_agent_learning_intent_v0(intent, policy)?;
    if private_state.agent_id != intent.agent_id
        || !valid_learning_digest_v0(&private_state.private_namespace_digest)
        || !valid_learning_digest_v0(&private_state.training_ledger_digest)
    {
        return Err("agent private learning state mismatch".into());
    }
    let authorized = intent
        .required_datasets
        .iter()
        .chain(intent.optional_datasets.iter())
        .copied()
        .collect::<BTreeSet<_>>();
    let mut sources = BTreeSet::new();
    let mut visible = BTreeSet::new();
    let mut available = BTreeSet::new();
    let mut shared_raw = BTreeSet::new();
    let mut private_artifacts = BTreeSet::new();
    for artifact in artifacts {
        if !valid_learning_digest_v0(&artifact.artifact_digest) {
            return Err("learning artifact identity invalid".into());
        }
        if artifact.maximum_event_timestamp_ms > intent.information_cutoff_ms {
            return Err("learning artifact exceeds information cutoff".into());
        }
        if !authorized.contains(&artifact.dataset_kind) {
            return Err("learning artifact dataset is unauthorized".into());
        }
        match artifact.visibility {
            LearningDataVisibilityV0::SharedCanonicalRaw => {
                if artifact.owner_agent_id.is_some() {
                    return Err("shared learning artifact has a private owner".into());
                }
                sources.insert(artifact.artifact_digest.clone());
                shared_raw.insert(artifact.artifact_digest.clone());
                visible.insert(artifact.dataset_kind);
                available.insert(artifact.dataset_kind);
            }
            LearningDataVisibilityV0::AgentAuthorizedRaw => {
                if artifact.owner_agent_id.as_deref() != Some(intent.agent_id.as_str()) {
                    return Err("agent-authorized learning artifact crossed agents".into());
                }
                sources.insert(artifact.artifact_digest.clone());
                visible.insert(artifact.dataset_kind);
                available.insert(artifact.dataset_kind);
            }
            LearningDataVisibilityV0::AgentPrivateDerived => {
                if artifact.owner_agent_id.as_deref() != Some(intent.agent_id.as_str()) {
                    return Err("private learning artifact crossed agents".into());
                }
                private_artifacts.insert(artifact.artifact_digest.clone());
            }
            LearningDataVisibilityV0::CommitteeVisibleSummary => {
                return Err("committee summary cannot enter an agent learning view".into());
            }
        }
    }
    let missing_required_datasets = intent
        .required_datasets
        .iter()
        .filter(|dataset| !available.contains(dataset))
        .copied()
        .collect::<Vec<_>>();
    let decision_gate = if missing_required_datasets.is_empty() {
        EvidenceDecisionGate::Ready
    } else {
        EvidenceDecisionGate::Abstain
    };
    let mut view = AgentLearningDataViewV0 {
        view_version: LEARNING_VIEW_VERSION_V0.into(),
        agent_id: intent.agent_id.clone(),
        source_artifact_digests: sources.into_iter().collect(),
        visible_dataset_kinds: visible.into_iter().collect(),
        information_cutoff_ms: intent.information_cutoff_ms,
        feature_policy_digest: intent.feature_policy_digest.clone(),
        label_policy_digest: intent.label_policy_digest.clone(),
        curriculum_policy_digest: intent.curriculum_policy_digest.clone(),
        private_namespace_digest: private_state.private_namespace_digest.clone(),
        training_ledger_digest: private_state.training_ledger_digest.clone(),
        shared_raw_count: shared_raw.len(),
        private_artifact_count: private_artifacts.len(),
        missing_required_datasets,
        decision_gate,
        view_digest: String::new(),
    };
    view.view_digest = agent_learning_data_view_digest_v0(&view);
    validate_agent_learning_data_view_v0(&view)?;
    Ok(view)
}

pub fn validate_agent_learning_data_view_v0(view: &AgentLearningDataViewV0) -> Result<(), String> {
    let sources = view
        .source_artifact_digests
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let visible = view
        .visible_dataset_kinds
        .iter()
        .copied()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let missing = view
        .missing_required_datasets
        .iter()
        .copied()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    if view.view_version != LEARNING_VIEW_VERSION_V0
        || !active_learning_agent_id_v0(&view.agent_id)
        || view.source_artifact_digests != sources
        || view.visible_dataset_kinds != visible
        || view.missing_required_datasets != missing
        || view
            .source_artifact_digests
            .iter()
            .any(|digest| !valid_learning_digest_v0(digest))
        || view
            .visible_dataset_kinds
            .iter()
            .chain(view.missing_required_datasets.iter())
            .any(|dataset| *dataset == DatasetKind::Unknown)
        || ![
            &view.feature_policy_digest,
            &view.label_policy_digest,
            &view.curriculum_policy_digest,
            &view.private_namespace_digest,
            &view.training_ledger_digest,
        ]
        .into_iter()
        .all(|digest| valid_learning_digest_v0(digest))
        || view.shared_raw_count > view.source_artifact_digests.len()
        || (view.missing_required_datasets.is_empty()
            != (view.decision_gate == EvidenceDecisionGate::Ready))
        || view.view_digest != agent_learning_data_view_digest_v0(view)
    {
        return Err("agent learning data view validation failed".into());
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LearningDataChairFirewallProofV0 {
    pub chair_cannot_create_intent: bool,
    pub chair_cannot_call_broker: bool,
    pub chair_cannot_select_provider: bool,
    pub chair_cannot_modify_view: bool,
    pub chair_cannot_change_cutoff: bool,
    pub chair_cannot_select_label: bool,
    pub chair_cannot_read_private_artifact: bool,
    pub all_invariants_pass: bool,
    pub proof_digest: String,
}

pub fn learning_data_chair_firewall_proof_v0() -> LearningDataChairFirewallProofV0 {
    let chair = LearningDataCallerV0::Chair;
    let mut proof = LearningDataChairFirewallProofV0 {
        chair_cannot_create_intent: !authorize_learning_data_action_v0(
            &chair,
            LearningDataAuthorityActionV0::CreateIntent,
        ),
        chair_cannot_call_broker: !authorize_learning_data_action_v0(
            &chair,
            LearningDataAuthorityActionV0::CallBroker,
        ),
        chair_cannot_select_provider: !authorize_learning_data_action_v0(
            &chair,
            LearningDataAuthorityActionV0::SelectProvider,
        ),
        chair_cannot_modify_view: !authorize_learning_data_action_v0(
            &chair,
            LearningDataAuthorityActionV0::ModifyView,
        ),
        chair_cannot_change_cutoff: !authorize_learning_data_action_v0(
            &chair,
            LearningDataAuthorityActionV0::ChangeCutoff,
        ),
        chair_cannot_select_label: !authorize_learning_data_action_v0(
            &chair,
            LearningDataAuthorityActionV0::SelectLabel,
        ),
        chair_cannot_read_private_artifact: !authorize_learning_data_action_v0(
            &chair,
            LearningDataAuthorityActionV0::ReadPrivateArtifact,
        ),
        all_invariants_pass: false,
        proof_digest: String::new(),
    };
    proof.all_invariants_pass = proof.chair_cannot_create_intent
        && proof.chair_cannot_call_broker
        && proof.chair_cannot_select_provider
        && proof.chair_cannot_modify_view
        && proof.chair_cannot_change_cutoff
        && proof.chair_cannot_select_label
        && proof.chair_cannot_read_private_artifact;
    proof.proof_digest = chair_firewall_proof_digest_v0(&proof);
    proof
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentLearningIndependenceProofV0 {
    pub agent_ids: Vec<String>,
    pub distinct_intent_digests: bool,
    pub distinct_view_digests: bool,
    pub distinct_feature_policies: bool,
    pub distinct_label_policies: bool,
    pub distinct_private_namespaces: bool,
    pub distinct_training_ledgers: bool,
    pub shared_raw_does_not_imply_shared_learning: bool,
    pub private_artifact_isolation: bool,
    pub chair_data_authority_absent: bool,
    pub all_invariants_pass: bool,
    pub proof_digest: String,
}

pub fn agent_learning_independence_proof_v0(
    intents: &[AgentLearningIntentV0],
    views: &[AgentLearningDataViewV0],
    firewall: &LearningDataChairFirewallProofV0,
) -> AgentLearningIndependenceProofV0 {
    let mut agent_ids = intents
        .iter()
        .map(|intent| intent.agent_id.clone())
        .collect::<Vec<_>>();
    agent_ids.sort();
    agent_ids.dedup();
    let matching_agents = agent_ids.len() == 3
        && views.len() == 3
        && views
            .iter()
            .map(|view| view.agent_id.as_str())
            .collect::<BTreeSet<_>>()
            == agent_ids
                .iter()
                .map(String::as_str)
                .collect::<BTreeSet<_>>();
    let distinct_intent_digests =
        distinct_count_v0(intents.iter().map(|intent| intent.intent_digest.as_str()))
            == intents.len();
    let distinct_view_digests =
        distinct_count_v0(views.iter().map(|view| view.view_digest.as_str())) == views.len();
    let distinct_feature_policies =
        distinct_count_v0(views.iter().map(|view| view.feature_policy_digest.as_str()))
            == views.len();
    let distinct_label_policies =
        distinct_count_v0(views.iter().map(|view| view.label_policy_digest.as_str()))
            == views.len();
    let distinct_private_namespaces = distinct_count_v0(
        views
            .iter()
            .map(|view| view.private_namespace_digest.as_str()),
    ) == views.len();
    let distinct_training_ledgers = distinct_count_v0(
        views
            .iter()
            .map(|view| view.training_ledger_digest.as_str()),
    ) == views.len();
    let shared_source_exists = views.iter().enumerate().any(|(index, left)| {
        views.iter().skip(index + 1).any(|right| {
            left.source_artifact_digests
                .iter()
                .any(|digest| right.source_artifact_digests.contains(digest))
        })
    });
    let shared_raw_does_not_imply_shared_learning = shared_source_exists
        && distinct_view_digests
        && distinct_feature_policies
        && distinct_label_policies
        && distinct_private_namespaces
        && distinct_training_ledgers;
    let private_artifact_isolation = matching_agents
        && distinct_private_namespaces
        && distinct_training_ledgers
        && views
            .iter()
            .all(|view| validate_agent_learning_data_view_v0(view).is_ok());
    let chair_data_authority_absent = firewall.all_invariants_pass
        && firewall.proof_digest == chair_firewall_proof_digest_v0(firewall);
    let mut proof = AgentLearningIndependenceProofV0 {
        agent_ids,
        distinct_intent_digests,
        distinct_view_digests,
        distinct_feature_policies,
        distinct_label_policies,
        distinct_private_namespaces,
        distinct_training_ledgers,
        shared_raw_does_not_imply_shared_learning,
        private_artifact_isolation,
        chair_data_authority_absent,
        all_invariants_pass: false,
        proof_digest: String::new(),
    };
    proof.all_invariants_pass = matching_agents
        && proof.distinct_intent_digests
        && proof.distinct_view_digests
        && proof.distinct_feature_policies
        && proof.distinct_label_policies
        && proof.distinct_private_namespaces
        && proof.distinct_training_ledgers
        && proof.shared_raw_does_not_imply_shared_learning
        && proof.private_artifact_isolation
        && proof.chair_data_authority_absent;
    proof.proof_digest = independence_proof_digest_v0(&proof);
    proof
}

fn stabilize_learning_intent_v0(intent: &mut AgentLearningIntentV0) {
    intent.market_scopes.sort();
    intent.market_scopes.dedup();
    intent.symbols.sort();
    intent.symbols.dedup();
    intent.required_datasets.sort();
    intent.required_datasets.dedup();
    intent.optional_datasets.sort();
    intent.optional_datasets.dedup();
    intent
        .optional_datasets
        .retain(|dataset| !intent.required_datasets.contains(dataset));
}

fn learning_policy_digest_v0(domain: &str, policy: &AgentDataPolicy) -> String {
    let mut material = Vec::new();
    canonical_string(&mut material, 1, "SOMA-LEARNING-POLICY-V0");
    canonical_string(&mut material, 2, domain);
    canonical_u8(&mut material, 3, agent_kind_code_v0(policy.agent_kind));
    let mut markets = policy.allowed_markets.clone();
    markets.sort();
    markets.dedup();
    for market in markets {
        canonical_u8(&mut material, 4, market_scope_code_v0(market));
    }
    let mut allowed = policy.allowed_dataset_kinds.clone();
    allowed.sort();
    allowed.dedup();
    for dataset in allowed {
        canonical_u16(&mut material, 5, dataset_kind_code_v0(dataset));
    }
    let mut required = policy.required_dataset_kinds.clone();
    required.sort();
    required.dedup();
    for dataset in required {
        canonical_u16(&mut material, 6, dataset_kind_code_v0(dataset));
    }
    let mut optional = policy.optional_dataset_kinds.clone();
    optional.sort();
    optional.dedup();
    for dataset in optional {
        canonical_u16(&mut material, 7, dataset_kind_code_v0(dataset));
    }
    canonical_u64(
        &mut material,
        8,
        u64::try_from(policy.default_lookback.bars).unwrap_or(u64::MAX),
    );
    canonical_u64(&mut material, 9, policy.max_staleness_ms);
    canonical_u64(
        &mut material,
        10,
        u64::try_from(policy.request_budget).unwrap_or(u64::MAX),
    );
    canonical_u8(
        &mut material,
        11,
        u8::from(policy.abstain_when_required_missing),
    );
    canonical_hash_hex(&material)
}

fn agent_learning_intent_digest_v0(intent: &AgentLearningIntentV0) -> String {
    let mut material = Vec::new();
    canonical_string(&mut material, 1, &intent.intent_version);
    canonical_string(&mut material, 2, &intent.agent_id);
    canonical_u8(&mut material, 3, agent_kind_code_v0(intent.agent_kind));
    for market in &intent.market_scopes {
        canonical_u8(&mut material, 4, market_scope_code_v0(*market));
    }
    for symbol in &intent.symbols {
        canonical_string(&mut material, 5, symbol);
    }
    for dataset in &intent.required_datasets {
        canonical_u16(&mut material, 6, dataset_kind_code_v0(*dataset));
    }
    for dataset in &intent.optional_datasets {
        canonical_u16(&mut material, 7, dataset_kind_code_v0(*dataset));
    }
    canonical_string(&mut material, 8, &intent.cadence);
    canonical_u64(
        &mut material,
        9,
        u64::try_from(intent.lookback.bars).unwrap_or(u64::MAX),
    );
    canonical_u64(
        &mut material,
        10,
        intent.lookback.start_timestamp_ms.unwrap_or_default(),
    );
    canonical_u64(
        &mut material,
        11,
        intent.lookback.end_timestamp_ms.unwrap_or_default(),
    );
    canonical_u64(&mut material, 12, intent.information_cutoff_ms);
    canonical_u64(&mut material, 13, intent.maximum_staleness_ms);
    canonical_string(&mut material, 14, &intent.source_policy_digest);
    canonical_string(&mut material, 15, &intent.feature_policy_digest);
    canonical_string(&mut material, 16, &intent.label_policy_digest);
    canonical_string(&mut material, 17, &intent.curriculum_policy_digest);
    canonical_hash_hex(&material)
}

fn agent_learning_data_view_digest_v0(view: &AgentLearningDataViewV0) -> String {
    let mut material = Vec::new();
    canonical_string(&mut material, 1, &view.view_version);
    canonical_string(&mut material, 2, &view.agent_id);
    for digest in &view.source_artifact_digests {
        canonical_string(&mut material, 3, digest);
    }
    for dataset in &view.visible_dataset_kinds {
        canonical_u16(&mut material, 4, dataset_kind_code_v0(*dataset));
    }
    canonical_u64(&mut material, 5, view.information_cutoff_ms);
    canonical_string(&mut material, 6, &view.feature_policy_digest);
    canonical_string(&mut material, 7, &view.label_policy_digest);
    canonical_string(&mut material, 8, &view.curriculum_policy_digest);
    canonical_string(&mut material, 9, &view.private_namespace_digest);
    canonical_string(&mut material, 10, &view.training_ledger_digest);
    canonical_u64(
        &mut material,
        11,
        u64::try_from(view.shared_raw_count).unwrap_or(u64::MAX),
    );
    canonical_u64(
        &mut material,
        12,
        u64::try_from(view.private_artifact_count).unwrap_or(u64::MAX),
    );
    for dataset in &view.missing_required_datasets {
        canonical_u16(&mut material, 13, dataset_kind_code_v0(*dataset));
    }
    canonical_u8(
        &mut material,
        14,
        evidence_decision_gate_code_v0(view.decision_gate),
    );
    canonical_hash_hex(&material)
}

fn chair_firewall_proof_digest_v0(proof: &LearningDataChairFirewallProofV0) -> String {
    stable_hash_string(&format!(
        "SOMA-LEARNING-CHAIR-FIREWALL-V0:{}:{}:{}:{}:{}:{}:{}:{}",
        proof.chair_cannot_create_intent,
        proof.chair_cannot_call_broker,
        proof.chair_cannot_select_provider,
        proof.chair_cannot_modify_view,
        proof.chair_cannot_change_cutoff,
        proof.chair_cannot_select_label,
        proof.chair_cannot_read_private_artifact,
        proof.all_invariants_pass,
    ))
}

fn independence_proof_digest_v0(proof: &AgentLearningIndependenceProofV0) -> String {
    stable_hash_string(&format!(
        "SOMA-LEARNING-INDEPENDENCE-V0:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}",
        proof.agent_ids.join("|"),
        proof.distinct_intent_digests,
        proof.distinct_view_digests,
        proof.distinct_feature_policies,
        proof.distinct_label_policies,
        proof.distinct_private_namespaces,
        proof.distinct_training_ledgers,
        proof.shared_raw_does_not_imply_shared_learning,
        proof.private_artifact_isolation,
        proof.chair_data_authority_absent,
        proof.all_invariants_pass,
    ))
}

fn distinct_count_v0<'a>(values: impl Iterator<Item = &'a str>) -> usize {
    values.collect::<BTreeSet<_>>().len()
}

fn expected_learning_agent_id_v0(kind: AgentKind) -> Option<&'static str> {
    match kind {
        AgentKind::MomentumTrendFast => Some("momentum_trend_fast"),
        AgentKind::ValueQualityFilter => Some("value_quality_filter"),
        AgentKind::CycleRiskSkeptic => Some("cycle_risk_skeptic"),
        AgentKind::Future8AgentPlaceholder => None,
    }
}

fn active_learning_agent_id_v0(agent_id: &str) -> bool {
    [
        "momentum_trend_fast",
        "value_quality_filter",
        "cycle_risk_skeptic",
    ]
    .contains(&agent_id)
}

fn valid_learning_digest_v0(digest: &str) -> bool {
    digest.len() == 16 && digest.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn agent_kind_code_v0(kind: AgentKind) -> u8 {
    match kind {
        AgentKind::MomentumTrendFast => 1,
        AgentKind::ValueQualityFilter => 2,
        AgentKind::CycleRiskSkeptic => 3,
        AgentKind::Future8AgentPlaceholder => 0,
    }
}

fn market_scope_code_v0(scope: AcquisitionMarketScope) -> u8 {
    match scope {
        AcquisitionMarketScope::UsStocks => 1,
        AcquisitionMarketScope::KoreanStocks => 2,
        AcquisitionMarketScope::BtcCrypto => 3,
        AcquisitionMarketScope::Unknown => 0,
    }
}

fn dataset_kind_code_v0(kind: DatasetKind) -> u16 {
    match kind {
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
        DatasetKind::Unknown => 0,
    }
}

fn dataset_kind_from_code_v0(code: u16) -> Option<DatasetKind> {
    match code {
        1 => Some(DatasetKind::DailyOhlcv),
        2 => Some(DatasetKind::AdjustedDailyOhlcv),
        3 => Some(DatasetKind::CorporateActions),
        4 => Some(DatasetKind::QuarterlyFundamentals),
        5 => Some(DatasetKind::ValuationMetrics),
        6 => Some(DatasetKind::MarketIndexDaily),
        7 => Some(DatasetKind::MarketBreadthDaily),
        8 => Some(DatasetKind::VolatilityDaily),
        9 => Some(DatasetKind::LiquidityDaily),
        10 => Some(DatasetKind::CryptoDailyOhlcv),
        11 => Some(DatasetKind::MacroSeries),
        _ => None,
    }
}

fn evidence_decision_gate_code_v0(gate: EvidenceDecisionGate) -> u8 {
    match gate {
        EvidenceDecisionGate::Ready => 1,
        EvidenceDecisionGate::Abstain => 2,
        EvidenceDecisionGate::ForceNoTrade => 3,
    }
}

fn evidence_decision_gate_from_code_v0(code: u32) -> Option<EvidenceDecisionGate> {
    match code {
        1 => Some(EvidenceDecisionGate::Ready),
        2 => Some(EvidenceDecisionGate::Abstain),
        3 => Some(EvidenceDecisionGate::ForceNoTrade),
        _ => None,
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalLearningArtifactEnvelopeV0 {
    pub magic: String,
    pub envelope_version: u32,
    pub schema_name: String,
    pub artifact_kind: String,
    pub semantic_digest: String,
    pub payload_length: u64,
    pub payload_digest: String,
    pub payload: Vec<u8>,
    pub source_artifact_digests: Vec<String>,
}

#[derive(Clone, PartialEq, Message)]
struct CanonicalLearningArtifactEnvelopeProtobufV0 {
    #[prost(string, tag = "1")]
    magic: String,
    #[prost(uint32, tag = "2")]
    envelope_version: u32,
    #[prost(string, tag = "3")]
    schema_name: String,
    #[prost(string, tag = "4")]
    artifact_kind: String,
    #[prost(string, tag = "5")]
    semantic_digest: String,
    #[prost(uint64, tag = "6")]
    payload_length: u64,
    #[prost(string, tag = "7")]
    payload_digest: String,
    #[prost(bytes = "vec", tag = "8")]
    payload: Vec<u8>,
    #[prost(string, repeated, tag = "9")]
    source_artifact_digests: Vec<String>,
}

#[derive(Clone, PartialEq, Message)]
struct AgentLearningDataViewProtobufV0 {
    #[prost(string, tag = "1")]
    view_version: String,
    #[prost(string, tag = "2")]
    agent_id: String,
    #[prost(string, repeated, tag = "3")]
    source_artifact_digests: Vec<String>,
    #[prost(uint32, repeated, tag = "4")]
    visible_dataset_kinds: Vec<u32>,
    #[prost(uint64, tag = "5")]
    information_cutoff_ms: u64,
    #[prost(string, tag = "6")]
    feature_policy_digest: String,
    #[prost(string, tag = "7")]
    label_policy_digest: String,
    #[prost(string, tag = "8")]
    curriculum_policy_digest: String,
    #[prost(string, tag = "9")]
    private_namespace_digest: String,
    #[prost(string, tag = "10")]
    training_ledger_digest: String,
    #[prost(uint64, tag = "11")]
    shared_raw_count: u64,
    #[prost(uint64, tag = "12")]
    private_artifact_count: u64,
    #[prost(uint32, repeated, tag = "13")]
    missing_required_datasets: Vec<u32>,
    #[prost(uint32, tag = "14")]
    decision_gate: u32,
    #[prost(string, tag = "15")]
    view_digest: String,
}

pub fn encode_agent_learning_data_view_protobuf_v0(
    view: &AgentLearningDataViewV0,
) -> Result<Vec<u8>, String> {
    validate_agent_learning_data_view_v0(view)?;
    let payload = learning_view_to_protobuf_v0(view)?.encode_to_vec();
    let envelope = CanonicalLearningArtifactEnvelopeProtobufV0 {
        magic: LEARNING_ENVELOPE_MAGIC_V0.into(),
        envelope_version: 0,
        schema_name: LEARNING_ENVELOPE_SCHEMA_V0.into(),
        artifact_kind: LEARNING_VIEW_ARTIFACT_KIND_V0.into(),
        semantic_digest: view.view_digest.clone(),
        payload_length: u64::try_from(payload.len())
            .map_err(|_| "learning artifact payload too large".to_string())?,
        payload_digest: canonical_hash_hex(&payload),
        payload,
        source_artifact_digests: view.source_artifact_digests.clone(),
    };
    Ok(envelope.encode_to_vec())
}

pub fn decode_agent_learning_data_view_protobuf_v0(
    bytes: &[u8],
) -> Result<(CanonicalLearningArtifactEnvelopeV0, AgentLearningDataViewV0), String> {
    let envelope = CanonicalLearningArtifactEnvelopeProtobufV0::decode(bytes)
        .map_err(|_| "learning artifact envelope decode failed".to_string())?;
    if envelope.magic != LEARNING_ENVELOPE_MAGIC_V0 {
        return Err("learning artifact magic rejected".into());
    }
    if envelope.envelope_version != 0 {
        return Err("learning artifact major version rejected".into());
    }
    if envelope.schema_name != LEARNING_ENVELOPE_SCHEMA_V0 {
        return Err("learning artifact schema rejected".into());
    }
    if envelope.artifact_kind != LEARNING_VIEW_ARTIFACT_KIND_V0 {
        return Err("learning artifact kind rejected".into());
    }
    if usize::try_from(envelope.payload_length).ok() != Some(envelope.payload.len()) {
        return Err("learning artifact payload length rejected".into());
    }
    if envelope.payload_digest != canonical_hash_hex(&envelope.payload) {
        return Err("learning artifact payload digest rejected".into());
    }
    let view = learning_view_from_protobuf_v0(
        AgentLearningDataViewProtobufV0::decode(envelope.payload.as_slice())
            .map_err(|_| "learning artifact payload decode failed".to_string())?,
    )?;
    validate_agent_learning_data_view_v0(&view)?;
    if envelope.semantic_digest != view.view_digest {
        return Err("learning artifact semantic digest rejected".into());
    }
    if envelope.source_artifact_digests != view.source_artifact_digests
        || envelope
            .source_artifact_digests
            .iter()
            .any(|digest| !valid_learning_digest_v0(digest))
    {
        return Err("learning artifact source identity rejected".into());
    }
    Ok((
        CanonicalLearningArtifactEnvelopeV0 {
            magic: envelope.magic,
            envelope_version: envelope.envelope_version,
            schema_name: envelope.schema_name,
            artifact_kind: envelope.artifact_kind,
            semantic_digest: envelope.semantic_digest,
            payload_length: envelope.payload_length,
            payload_digest: envelope.payload_digest,
            payload: envelope.payload,
            source_artifact_digests: envelope.source_artifact_digests,
        },
        view,
    ))
}

pub fn write_and_verify_agent_learning_data_view_v0(
    view: &AgentLearningDataViewV0,
    output_dir: &Path,
) -> Result<PathBuf, String> {
    if !safe_learning_data_path_v0(output_dir) {
        return Err("learning data output namespace rejected".into());
    }
    fs::create_dir_all(output_dir)
        .map_err(|_| "learning data output directory unavailable".to_string())?;
    let path = output_dir.join(format!("agent-view-{}.pb", view.view_digest));
    write_and_verify_learning_view_at_path_v0(view, &path)?;
    Ok(path)
}

pub fn read_and_verify_agent_learning_data_view_v0(
    path: &Path,
) -> Result<AgentLearningDataViewV0, String> {
    if !safe_learning_data_path_v0(path) {
        return Err("learning data input namespace rejected".into());
    }
    let mut file = File::open(path).map_err(|_| "learning data reopen failed".to_string())?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
        .map_err(|_| "learning data reopen failed".to_string())?;
    decode_agent_learning_data_view_protobuf_v0(&bytes).map(|(_, view)| view)
}

pub fn migrate_legacy_learning_view_json_v0(path: &Path) -> Result<PathBuf, String> {
    if !safe_learning_data_path_v0(path)
        || path.extension().is_none_or(|extension| extension != "json")
    {
        return Err("legacy learning view path rejected".into());
    }
    let original = fs::read(path).map_err(|_| "legacy learning view unavailable".to_string())?;
    let view: AgentLearningDataViewV0 = serde_json::from_slice(&original)
        .map_err(|_| "legacy learning view decode failed".to_string())?;
    validate_agent_learning_data_view_v0(&view)?;
    let sidecar = path.with_extension("pb");
    write_and_verify_learning_view_at_path_v0(&view, &sidecar)?;
    if fs::read(path).map_err(|_| "legacy learning view unavailable".to_string())? != original {
        return Err("legacy learning view changed during migration".into());
    }
    Ok(sidecar)
}

fn write_and_verify_learning_view_at_path_v0(
    view: &AgentLearningDataViewV0,
    path: &Path,
) -> Result<(), String> {
    validate_agent_learning_data_view_v0(view)?;
    if !safe_learning_data_path_v0(path)
        || path.extension().is_none_or(|extension| extension != "pb")
    {
        return Err("learning data storage path rejected".into());
    }
    if path.is_file() {
        return if read_and_verify_agent_learning_data_view_v0(path)? == *view {
            Ok(())
        } else {
            Err("learning data artifact is immutable".into())
        };
    }
    let parent = path
        .parent()
        .ok_or_else(|| "learning data parent unavailable".to_string())?;
    fs::create_dir_all(parent)
        .map_err(|_| "learning data output directory unavailable".to_string())?;
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| "learning data filename rejected".to_string())?;
    let temporary = parent.join(format!(".{file_name}.{}.tmp", std::process::id()));
    let serialized = encode_agent_learning_data_view_protobuf_v0(view)?;
    let write_result = (|| {
        let mut file = File::options()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .map_err(|_| "learning data temporary write failed".to_string())?;
        file.write_all(&serialized)
            .map_err(|_| "learning data temporary write failed".to_string())?;
        file.flush()
            .map_err(|_| "learning data temporary flush failed".to_string())?;
        file.sync_all()
            .map_err(|_| "learning data temporary sync failed".to_string())?;
        drop(file);
        if read_and_verify_agent_learning_data_view_v0(&temporary)? != *view {
            return Err("learning data temporary verification failed".into());
        }
        fs::rename(&temporary, path)
            .map_err(|_| "learning data atomic rename failed".to_string())?;
        if read_and_verify_agent_learning_data_view_v0(path)? != *view {
            return Err("learning data final verification failed".into());
        }
        Ok(())
    })();
    if write_result.is_err() && temporary.is_file() {
        let _ = fs::remove_file(&temporary);
    }
    write_result
}

fn safe_learning_data_path_v0(path: &Path) -> bool {
    path.starts_with(LEARNING_DATA_NAMESPACE_V0)
        && !path.as_os_str().is_empty()
        && !path.to_string_lossy().contains("prospective")
        && path.components().all(|component| {
            !matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            ) && component.as_os_str() != ".env"
        })
}

fn learning_view_to_protobuf_v0(
    view: &AgentLearningDataViewV0,
) -> Result<AgentLearningDataViewProtobufV0, String> {
    Ok(AgentLearningDataViewProtobufV0 {
        view_version: view.view_version.clone(),
        agent_id: view.agent_id.clone(),
        source_artifact_digests: view.source_artifact_digests.clone(),
        visible_dataset_kinds: view
            .visible_dataset_kinds
            .iter()
            .map(|kind| u32::from(dataset_kind_code_v0(*kind)))
            .collect(),
        information_cutoff_ms: view.information_cutoff_ms,
        feature_policy_digest: view.feature_policy_digest.clone(),
        label_policy_digest: view.label_policy_digest.clone(),
        curriculum_policy_digest: view.curriculum_policy_digest.clone(),
        private_namespace_digest: view.private_namespace_digest.clone(),
        training_ledger_digest: view.training_ledger_digest.clone(),
        shared_raw_count: u64::try_from(view.shared_raw_count)
            .map_err(|_| "learning shared raw count overflow".to_string())?,
        private_artifact_count: u64::try_from(view.private_artifact_count)
            .map_err(|_| "learning private artifact count overflow".to_string())?,
        missing_required_datasets: view
            .missing_required_datasets
            .iter()
            .map(|kind| u32::from(dataset_kind_code_v0(*kind)))
            .collect(),
        decision_gate: u32::from(evidence_decision_gate_code_v0(view.decision_gate)),
        view_digest: view.view_digest.clone(),
    })
}

fn learning_view_from_protobuf_v0(
    value: AgentLearningDataViewProtobufV0,
) -> Result<AgentLearningDataViewV0, String> {
    Ok(AgentLearningDataViewV0 {
        view_version: value.view_version,
        agent_id: value.agent_id,
        source_artifact_digests: value.source_artifact_digests,
        visible_dataset_kinds: value
            .visible_dataset_kinds
            .into_iter()
            .map(|code| {
                u16::try_from(code)
                    .ok()
                    .and_then(dataset_kind_from_code_v0)
                    .ok_or_else(|| "learning view dataset kind rejected".to_string())
            })
            .collect::<Result<Vec<_>, _>>()?,
        information_cutoff_ms: value.information_cutoff_ms,
        feature_policy_digest: value.feature_policy_digest,
        label_policy_digest: value.label_policy_digest,
        curriculum_policy_digest: value.curriculum_policy_digest,
        private_namespace_digest: value.private_namespace_digest,
        training_ledger_digest: value.training_ledger_digest,
        shared_raw_count: usize::try_from(value.shared_raw_count)
            .map_err(|_| "learning shared raw count rejected".to_string())?,
        private_artifact_count: usize::try_from(value.private_artifact_count)
            .map_err(|_| "learning private artifact count rejected".to_string())?,
        missing_required_datasets: value
            .missing_required_datasets
            .into_iter()
            .map(|code| {
                u16::try_from(code)
                    .ok()
                    .and_then(dataset_kind_from_code_v0)
                    .ok_or_else(|| "learning missing dataset kind rejected".to_string())
            })
            .collect::<Result<Vec<_>, _>>()?,
        decision_gate: evidence_decision_gate_from_code_v0(value.decision_gate)
            .ok_or_else(|| "learning decision gate rejected".to_string())?,
        view_digest: value.view_digest,
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LearningDataUsageClassificationV0 {
    ResearchOnlyUnconsumed,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LearningDataProvenanceManifestV0 {
    pub source_provider_id: String,
    pub source_type: String,
    pub acquisition_request_identity: String,
    pub fetch_timestamp_ms: u64,
    pub publication_event_timestamp_ms: Option<u64>,
    pub raw_content_digest: String,
    pub parser_version: String,
    pub normalized_artifact_digest: String,
    pub sanitized: bool,
    pub credential_free: bool,
    pub information_cutoff_ms: u64,
    pub usage_classification: LearningDataUsageClassificationV0,
    pub manifest_digest: String,
}

pub fn seal_learning_data_provenance_manifest_v0(
    mut manifest: LearningDataProvenanceManifestV0,
) -> Result<LearningDataProvenanceManifestV0, String> {
    manifest.manifest_digest.clear();
    if manifest.source_provider_id.trim().is_empty()
        || manifest.source_type.trim().is_empty()
        || manifest.acquisition_request_identity.trim().is_empty()
        || manifest.fetch_timestamp_ms == 0
        || manifest.parser_version.trim().is_empty()
        || !valid_learning_digest_v0(&manifest.raw_content_digest)
        || !valid_learning_digest_v0(&manifest.normalized_artifact_digest)
        || !manifest.sanitized
        || !manifest.credential_free
        || manifest.information_cutoff_ms == 0
        || manifest
            .publication_event_timestamp_ms
            .is_some_and(|timestamp| timestamp > manifest.information_cutoff_ms)
    {
        return Err("learning data provenance manifest rejected".into());
    }
    manifest.manifest_digest = learning_data_provenance_manifest_digest_v0(&manifest);
    Ok(manifest)
}

fn learning_data_provenance_manifest_digest_v0(
    manifest: &LearningDataProvenanceManifestV0,
) -> String {
    stable_hash_string(&format!(
        "SOMA-LEARNING-PROVENANCE-V0:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{:?}",
        manifest.source_provider_id,
        manifest.source_type,
        manifest.acquisition_request_identity,
        manifest.fetch_timestamp_ms,
        manifest.publication_event_timestamp_ms.unwrap_or_default(),
        manifest.raw_content_digest,
        manifest.parser_version,
        manifest.normalized_artifact_digest,
        manifest.sanitized,
        manifest.credential_free,
        manifest.information_cutoff_ms,
        manifest.usage_classification,
    ))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LearningNetworkPilotStatusV0 {
    ReadyForExplicitRequest,
    DeferredToProtectProspectiveEvaluation,
    Rejected,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LearningNetworkPilotInputV0 {
    pub explicit_network_consent: bool,
    pub non_overlapping_request_proven: bool,
    pub provider_approved_read_only: bool,
    pub credential_scope_approved: bool,
    pub bounded_response: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LearningDataPlaneSafetyCountersV0 {
    pub active_committee_count: usize,
    pub network_requests: usize,
    pub credential_reads: usize,
    pub prospective_artifact_mutations: usize,
    pub prospective_label_reads: usize,
    pub chair_decisions: usize,
    pub votes: usize,
    pub rewards: usize,
    pub penalties: usize,
    pub voice_changes: usize,
    pub executions: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LearningNetworkPilotPlanV0 {
    pub status: LearningNetworkPilotStatusV0,
    pub storage_namespace: String,
    pub usage_classification: LearningDataUsageClassificationV0,
    pub maximum_requests: usize,
    pub maximum_concurrency: usize,
    pub maximum_retries: usize,
    pub safety_counters: LearningDataPlaneSafetyCountersV0,
}

pub fn plan_learning_network_pilot_v0(
    input: &LearningNetworkPilotInputV0,
) -> LearningNetworkPilotPlanV0 {
    let status = if !input.non_overlapping_request_proven {
        LearningNetworkPilotStatusV0::DeferredToProtectProspectiveEvaluation
    } else if input.explicit_network_consent
        && input.provider_approved_read_only
        && input.credential_scope_approved
        && input.bounded_response
    {
        LearningNetworkPilotStatusV0::ReadyForExplicitRequest
    } else {
        LearningNetworkPilotStatusV0::Rejected
    };
    LearningNetworkPilotPlanV0 {
        status,
        storage_namespace: format!("{LEARNING_DATA_NAMESPACE_V0}/network_pilot"),
        usage_classification: LearningDataUsageClassificationV0::ResearchOnlyUnconsumed,
        maximum_requests: 1,
        maximum_concurrency: 1,
        maximum_retries: 0,
        safety_counters: LearningDataPlaneSafetyCountersV0 {
            active_committee_count: 3,
            network_requests: 0,
            credential_reads: 0,
            prospective_artifact_mutations: 0,
            prospective_label_reads: 0,
            chair_decisions: 0,
            votes: 0,
            rewards: 0,
            penalties: 0,
            voice_changes: 0,
            executions: 0,
        },
    }
}

fn snapshot_digest(dataset: &HistoricalReplayDataset) -> String {
    historical_replay_dataset_digest_v0(dataset)
}

fn acquisition_request_key(dataset_kind: DatasetKind, intent: &AgentDataIntent) -> String {
    let mut symbols = intent.symbols.clone();
    symbols.sort();
    symbols.dedup();
    format!(
        "{:?}:{:?}:{}:{}:{}:{}:{}:{}",
        dataset_kind,
        intent.market_scope,
        symbols.join(","),
        intent.target_cadence,
        intent.lookback.bars,
        intent.lookback.start_timestamp_ms.unwrap_or_default(),
        intent.lookback.end_timestamp_ms.unwrap_or_default(),
        intent.max_staleness_ms,
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvidenceFreshnessStatus {
    Fresh,
    StaleWithinTolerance,
    StaleRejected,
}

fn assess_freshness(
    snapshot: &DataSnapshot,
    now_ms: u64,
    max_staleness_ms: u64,
    policy: &AcquisitionPolicy,
) -> EvidenceFreshnessStatus {
    let age = now_ms.saturating_sub(snapshot.fetched_at_ms);
    if age <= max_staleness_ms {
        EvidenceFreshnessStatus::Fresh
    } else if age <= policy.last_known_good_tolerance_ms
        && policy.stale_data_policy == StaleDataPolicy::UseLastKnownGoodWithinTolerance
    {
        EvidenceFreshnessStatus::StaleWithinTolerance
    } else {
        EvidenceFreshnessStatus::StaleRejected
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvidenceDecisionGate {
    Ready,
    Abstain,
    ForceNoTrade,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentEvidenceBundle {
    pub agent_id: String,
    pub requested_datasets: Vec<DatasetKind>,
    pub required_snapshot_ids: Vec<String>,
    pub optional_snapshot_ids: Vec<String>,
    pub missing_required_datasets: Vec<DatasetKind>,
    pub missing_optional_datasets: Vec<DatasetKind>,
    pub freshness_status: EvidenceFreshnessStatus,
    pub provenance_receipts: Vec<String>,
    pub frozen: bool,
    pub decision_gate: EvidenceDecisionGate,
    pub reason_codes: Vec<ReasonCode>,
}

pub fn build_agent_evidence_bundles(
    intents: &[AgentDataIntent],
    plan: &AcquisitionPlan,
    execution: &BrokerExecutionResult,
    policy: &AcquisitionPolicy,
) -> Vec<AgentEvidenceBundle> {
    let receipt_by_request = execution
        .receipts
        .iter()
        .map(|receipt| (receipt.request_id.clone(), receipt))
        .collect::<BTreeMap<_, _>>();
    intents
        .iter()
        .map(|intent| {
            let mut required_snapshot_ids = Vec::new();
            let mut optional_snapshot_ids = Vec::new();
            let mut receipts = Vec::new();
            let mut missing_required = BTreeSet::new();
            let mut missing_optional = BTreeSet::new();
            for dataset in &intent.required_datasets {
                collect_bundle_dataset(
                    intent,
                    *dataset,
                    true,
                    plan,
                    &receipt_by_request,
                    &mut required_snapshot_ids,
                    &mut optional_snapshot_ids,
                    &mut receipts,
                    &mut missing_required,
                    &mut missing_optional,
                );
            }
            for dataset in &intent.optional_datasets {
                collect_bundle_dataset(
                    intent,
                    *dataset,
                    false,
                    plan,
                    &receipt_by_request,
                    &mut required_snapshot_ids,
                    &mut optional_snapshot_ids,
                    &mut receipts,
                    &mut missing_required,
                    &mut missing_optional,
                );
            }
            let decision_gate = if missing_required.is_empty() {
                EvidenceDecisionGate::Ready
            } else if policy.stale_data_policy == StaleDataPolicy::ForceNoTrade {
                EvidenceDecisionGate::ForceNoTrade
            } else {
                EvidenceDecisionGate::Abstain
            };
            let mut reason_codes = intent.reason_codes.clone();
            if !missing_required.is_empty() {
                reason_codes.push(ReasonCode::AgentRequiredDatasetMissing);
                reason_codes.push(match decision_gate {
                    EvidenceDecisionGate::ForceNoTrade => ReasonCode::EvidenceForcedNoTrade,
                    EvidenceDecisionGate::Ready | EvidenceDecisionGate::Abstain => {
                        ReasonCode::EvidenceAgentAbstained
                    }
                });
            }
            if !missing_optional.is_empty() {
                reason_codes.push(ReasonCode::AgentOptionalDatasetMissing);
            }
            let freshness_status = if missing_required.is_empty() {
                bundle_freshness_status(intent, plan, &receipt_by_request)
            } else {
                EvidenceFreshnessStatus::StaleRejected
            };
            reason_codes.push(match freshness_status {
                EvidenceFreshnessStatus::Fresh => ReasonCode::EvidenceFresh,
                EvidenceFreshnessStatus::StaleWithinTolerance => {
                    ReasonCode::EvidenceStaleWithinTolerance
                }
                EvidenceFreshnessStatus::StaleRejected => ReasonCode::EvidenceStaleRejected,
            });
            AgentEvidenceBundle {
                agent_id: intent.agent_id.clone(),
                requested_datasets: intent
                    .required_datasets
                    .iter()
                    .chain(intent.optional_datasets.iter())
                    .copied()
                    .collect(),
                required_snapshot_ids,
                optional_snapshot_ids,
                missing_required_datasets: missing_required.into_iter().collect(),
                missing_optional_datasets: missing_optional.into_iter().collect(),
                freshness_status,
                provenance_receipts: receipts,
                frozen: false,
                decision_gate,
                reason_codes: stable_reason_codes(&reason_codes),
            }
        })
        .collect()
}

fn bundle_freshness_status(
    intent: &AgentDataIntent,
    plan: &AcquisitionPlan,
    receipts: &BTreeMap<String, &AcquisitionReceipt>,
) -> EvidenceFreshnessStatus {
    let mut status = EvidenceFreshnessStatus::Fresh;
    for request in &plan.planned_requests {
        if !request.requested_by_agents.contains(&intent.agent_id) {
            continue;
        }
        let Some(receipt) = receipts.get(&request.request.request_id) else {
            return EvidenceFreshnessStatus::StaleRejected;
        };
        if receipt
            .reason_codes
            .contains(&ReasonCode::EvidenceStaleRejected)
        {
            return EvidenceFreshnessStatus::StaleRejected;
        }
        if receipt
            .reason_codes
            .contains(&ReasonCode::EvidenceStaleWithinTolerance)
        {
            status = EvidenceFreshnessStatus::StaleWithinTolerance;
        }
    }
    status
}

#[allow(clippy::too_many_arguments)]
fn collect_bundle_dataset(
    intent: &AgentDataIntent,
    dataset_kind: DatasetKind,
    required: bool,
    plan: &AcquisitionPlan,
    receipts: &BTreeMap<String, &AcquisitionReceipt>,
    required_snapshot_ids: &mut Vec<String>,
    optional_snapshot_ids: &mut Vec<String>,
    provenance_receipts: &mut Vec<String>,
    missing_required: &mut BTreeSet<DatasetKind>,
    missing_optional: &mut BTreeSet<DatasetKind>,
) {
    let request_key = acquisition_request_key(dataset_kind, intent);
    let request = plan
        .planned_requests
        .iter()
        .find(|request| request.request.request_key == request_key);
    let snapshot_id = request
        .and_then(|request| receipts.get(&request.request.request_id))
        .and_then(|receipt| receipt.snapshot_id.clone());
    if let Some(snapshot_id) = snapshot_id {
        if required {
            required_snapshot_ids.push(snapshot_id);
        } else {
            optional_snapshot_ids.push(snapshot_id);
        }
        if let Some(request) = request {
            if let Some(receipt) = receipts.get(&request.request.request_id) {
                provenance_receipts.push(receipt.receipt_id.clone());
            }
        }
    } else if required {
        missing_required.insert(dataset_kind);
    } else {
        missing_optional.insert(dataset_kind);
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrozenSnapshotSet {
    pub cycle_id: String,
    pub snapshot_ids: Vec<String>,
    pub agent_ids: Vec<String>,
    pub frozen: bool,
    pub reason_codes: Vec<ReasonCode>,
}

pub fn freeze_decision_snapshot_set(
    cycle_id: impl Into<String>,
    bundles: &mut [AgentEvidenceBundle],
) -> FrozenSnapshotSet {
    let mut snapshot_ids = BTreeSet::new();
    let mut agent_ids = BTreeSet::new();
    for bundle in bundles {
        bundle.frozen = true;
        bundle.reason_codes = stable_reason_codes(
            &bundle
                .reason_codes
                .iter()
                .cloned()
                .chain([ReasonCode::DataSnapshotFrozen])
                .collect::<Vec<_>>(),
        );
        snapshot_ids.extend(bundle.required_snapshot_ids.iter().cloned());
        snapshot_ids.extend(bundle.optional_snapshot_ids.iter().cloned());
        agent_ids.insert(bundle.agent_id.clone());
    }
    FrozenSnapshotSet {
        cycle_id: cycle_id.into(),
        snapshot_ids: snapshot_ids.into_iter().collect(),
        agent_ids: agent_ids.into_iter().collect(),
        frozen: true,
        reason_codes: vec![
            ReasonCode::DataSnapshotFrozen,
            ReasonCode::DataSnapshotImmutable,
        ],
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AgentProposalEvidenceBinding {
    pub proposal: AgentProposal,
    pub snapshot_ids: Vec<String>,
    pub evidence_gate: EvidenceDecisionGate,
    pub frozen: bool,
    pub reason_codes: Vec<ReasonCode>,
}

pub fn bind_proposal_to_frozen_evidence(
    mut proposal: AgentProposal,
    bundle: &AgentEvidenceBundle,
) -> AgentProposalEvidenceBinding {
    if bundle.decision_gate != EvidenceDecisionGate::Ready || !bundle.frozen {
        proposal.stance = if bundle.decision_gate == EvidenceDecisionGate::ForceNoTrade {
            Stance::NoTrade
        } else {
            Stance::Abstain
        };
        proposal.confidence = 0.0;
        proposal.expected_edge = 0.0;
        proposal.reason_codes = stable_reason_codes(
            &proposal
                .reason_codes
                .iter()
                .cloned()
                .chain(bundle.reason_codes.iter().cloned())
                .chain([ReasonCode::AgentDataIntentAbstained])
                .collect::<Vec<_>>(),
        );
    }
    let snapshot_ids = bundle
        .required_snapshot_ids
        .iter()
        .chain(bundle.optional_snapshot_ids.iter())
        .cloned()
        .collect();
    AgentProposalEvidenceBinding {
        proposal,
        snapshot_ids,
        evidence_gate: bundle.decision_gate,
        frozen: bundle.frozen,
        reason_codes: bundle.reason_codes.clone(),
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AutonomousDataCycleInput {
    pub cycle_id: String,
    pub now_ms: u64,
    pub active_agent_states: Vec<CanonicalAgentState>,
    pub configured_universe: ConfiguredUniverse,
    pub acquisition_mode: AcquisitionMode,
    pub agent_data_policies: Vec<AgentDataPolicy>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AutonomousDataCyclePlan {
    pub agent_intents: Vec<AgentDataIntent>,
    pub acquisition_plan: AcquisitionPlan,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AutonomousDataCycleResult {
    pub agent_intents: Vec<AgentDataIntent>,
    pub acquisition_plan: AcquisitionPlan,
    pub acquisition_receipts: Vec<AcquisitionReceipt>,
    pub new_snapshots: Vec<DataSnapshot>,
    pub reused_snapshots: Vec<DataSnapshot>,
    pub rejected_requests: Vec<RejectedAcquisitionRequest>,
    pub agent_evidence_bundles: Vec<AgentEvidenceBundle>,
    pub frozen_snapshot_set: FrozenSnapshotSet,
    pub reason_codes: Vec<ReasonCode>,
}

pub fn plan_autonomous_data_cycle(
    input: &AutonomousDataCycleInput,
    provider_registry: &ReadOnlyProviderRegistry,
    policy: &AcquisitionPolicy,
) -> AutonomousDataCyclePlan {
    let mut planned_kinds = Vec::new();
    let mut agent_intents = Vec::new();
    for state in &input.active_agent_states {
        if state.status != AgentStatus::Active
            || state.kind == AgentKind::Future8AgentPlaceholder
            || planned_kinds.contains(&state.kind)
        {
            continue;
        }
        let Some(policy) = input
            .agent_data_policies
            .iter()
            .find(|policy| policy.agent_kind == state.kind)
        else {
            continue;
        };
        planned_kinds.push(state.kind);
        agent_intents.push(plan_agent_data_intent(
            state.agent_id.clone(),
            state.kind,
            &input.configured_universe,
            policy,
            input.now_ms,
        ));
    }
    let acquisition_plan = build_acquisition_plan(
        &agent_intents,
        provider_registry,
        input.acquisition_mode,
        policy,
    );
    AutonomousDataCyclePlan {
        agent_intents,
        reason_codes: stable_reason_codes(
            &input
                .reason_codes
                .iter()
                .chain(acquisition_plan.reason_codes.iter())
                .cloned()
                .collect::<Vec<_>>(),
        ),
        acquisition_plan,
    }
}

pub fn execute_autonomous_data_cycle(
    input: &AutonomousDataCycleInput,
    broker: &mut DataAcquisitionBroker,
    provider: Option<&mut dyn ReadOnlyMarketDataProvider>,
) -> AutonomousDataCycleResult {
    let planned =
        plan_autonomous_data_cycle(input, &broker.provider_registry, &broker.acquisition_policy);
    let execution = broker.execute_acquisition_plan(
        &planned.acquisition_plan,
        input.acquisition_mode,
        input.now_ms,
        provider,
    );
    let mut bundles = build_agent_evidence_bundles(
        &planned.agent_intents,
        &planned.acquisition_plan,
        &execution,
        &broker.acquisition_policy,
    );
    let frozen_snapshot_set = freeze_decision_snapshot_set(input.cycle_id.clone(), &mut bundles);
    AutonomousDataCycleResult {
        agent_intents: planned.agent_intents,
        acquisition_plan: planned.acquisition_plan.clone(),
        acquisition_receipts: execution.receipts,
        new_snapshots: execution.new_snapshots,
        reused_snapshots: execution.reused_snapshots,
        rejected_requests: planned.acquisition_plan.rejected_requests,
        agent_evidence_bundles: bundles,
        frozen_snapshot_set,
        reason_codes: stable_reason_codes(
            &planned
                .reason_codes
                .iter()
                .chain(execution.reason_codes.iter())
                .cloned()
                .chain([ReasonCode::DataSnapshotFrozen])
                .collect::<Vec<_>>(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::league::canonical_current_agent_states;

    fn universe() -> ConfiguredUniverse {
        ConfiguredUniverse {
            symbols_by_market: BTreeMap::from([
                (
                    AcquisitionMarketScope::UsStocks,
                    vec!["AAA".to_string(), "BBB".to_string()],
                ),
                (
                    AcquisitionMarketScope::KoreanStocks,
                    vec!["005930.KS".to_string()],
                ),
                (
                    AcquisitionMarketScope::BtcCrypto,
                    vec!["BTC-USD".to_string()],
                ),
            ]),
        }
    }

    fn mock_capabilities() -> ProviderCapabilities {
        ProviderCapabilities {
            provider_id: "mock-readonly".to_string(),
            supported_markets: vec![
                AcquisitionMarketScope::UsStocks,
                AcquisitionMarketScope::KoreanStocks,
                AcquisitionMarketScope::BtcCrypto,
            ],
            supported_dataset_kinds: vec![
                DatasetKind::DailyOhlcv,
                DatasetKind::AdjustedDailyOhlcv,
                DatasetKind::QuarterlyFundamentals,
                DatasetKind::ValuationMetrics,
                DatasetKind::CorporateActions,
                DatasetKind::MarketIndexDaily,
                DatasetKind::MarketBreadthDaily,
                DatasetKind::VolatilityDaily,
                DatasetKind::LiquidityDaily,
                DatasetKind::MacroSeries,
            ],
            supported_cadences: vec!["1d".to_string()],
            maximum_lookback_bars: 500,
            requires_credentials: false,
            read_only: true,
            enabled: true,
            approved_for_network: false,
            mock_only: true,
            reason_codes: vec![ReasonCode::DatasetKindReadOnly],
        }
    }

    fn mock_dataset() -> HistoricalReplayDataset {
        HistoricalReplayDataset {
            symbol: "AAA".to_string(),
            source: "normalized-mock".to_string(),
            rows: vec![
                HistoricalOhlcvRow {
                    symbol: "AAA".to_string(),
                    timestamp_ms: 1,
                    open: 10.0,
                    high: 11.0,
                    low: 9.0,
                    close: 10.5,
                    volume: 100.0,
                    trade_value: None,
                },
                HistoricalOhlcvRow {
                    symbol: "AAA".to_string(),
                    timestamp_ms: 2,
                    open: 10.5,
                    high: 12.0,
                    low: 10.0,
                    close: 11.5,
                    volume: 120.0,
                    trade_value: None,
                },
            ],
            reason_codes: vec![],
        }
    }

    fn mock_provider(now_ms: u64) -> MockReadOnlyProvider {
        MockReadOnlyProvider {
            capabilities: mock_capabilities(),
            default_response: Some(ReadOnlyProviderResponse {
                request_id: String::new(),
                provider_id: String::new(),
                fetched_at_ms: now_ms,
                content_type: "application/x-soma-normalized-dataset".to_string(),
                all_rows_finalized: true,
                normalized_dataset: mock_dataset(),
                reported_content_bytes: 512,
                reason_codes: vec![],
            }),
            default_failure: None,
            requests: Vec::new(),
        }
    }

    fn fallback_request(request_key: &str) -> ReadOnlyProviderRequest {
        ReadOnlyProviderRequest {
            request_id: format!("request-{request_key}"),
            request_key: request_key.to_string(),
            provider_id: "mock-readonly".to_string(),
            dataset_kind: DatasetKind::DailyOhlcv,
            market_scope: AcquisitionMarketScope::UsStocks,
            symbols: vec!["AAA".to_string()],
            lookback: DataLookback {
                bars: 2,
                start_timestamp_ms: None,
                end_timestamp_ms: Some(10),
            },
            cadence: "1d".to_string(),
            max_staleness_ms: 20,
            reason_codes: vec![],
        }
    }

    fn fallback_snapshot(request: &ReadOnlyProviderRequest, fetched_at_ms: u64) -> DataSnapshot {
        let mut provider = mock_provider(fetched_at_ms);
        let response = provider.fetch_readonly(request).unwrap();
        snapshot_from_response(
            request,
            response,
            AcquisitionMode::Mock,
            fetched_at_ms,
            1_024,
        )
        .unwrap()
    }

    fn input(mode: AcquisitionMode, now_ms: u64) -> AutonomousDataCycleInput {
        AutonomousDataCycleInput {
            cycle_id: "cycle-1".to_string(),
            now_ms,
            active_agent_states: canonical_current_agent_states(),
            configured_universe: universe(),
            acquisition_mode: mode,
            agent_data_policies: default_agent_data_policies(),
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }

    fn learning_intents() -> Vec<AgentLearningIntentV0> {
        derive_active_agent_learning_intents_v0(
            &canonical_current_agent_states(),
            &universe(),
            &default_agent_data_policies(),
            100,
        )
        .unwrap()
    }

    fn learning_policy(intent: &AgentLearningIntentV0) -> AgentDataPolicy {
        default_agent_data_policies()
            .into_iter()
            .find(|policy| policy.agent_kind == intent.agent_kind)
            .unwrap()
    }

    fn ready_learning_views() -> (Vec<AgentLearningIntentV0>, Vec<AgentLearningDataViewV0>) {
        let intents = learning_intents();
        let views = intents
            .iter()
            .map(|intent| {
                let mut datasets = intent.required_datasets.clone();
                if intent.agent_kind == AgentKind::MomentumTrendFast {
                    datasets.push(DatasetKind::VolatilityDaily);
                }
                datasets.sort();
                datasets.dedup();
                let mut artifacts = datasets
                    .into_iter()
                    .map(|dataset_kind| LearningDataArtifactRefV0 {
                        artifact_digest: stable_hash_string(&format!("shared:{dataset_kind:?}")),
                        dataset_kind,
                        visibility: LearningDataVisibilityV0::SharedCanonicalRaw,
                        owner_agent_id: None,
                        maximum_event_timestamp_ms: 100,
                    })
                    .collect::<Vec<_>>();
                artifacts.push(LearningDataArtifactRefV0 {
                    artifact_digest: stable_hash_string(&format!(
                        "private:{}:{:?}",
                        intent.agent_id, intent.required_datasets[0]
                    )),
                    dataset_kind: intent.required_datasets[0],
                    visibility: LearningDataVisibilityV0::AgentPrivateDerived,
                    owner_agent_id: Some(intent.agent_id.clone()),
                    maximum_event_timestamp_ms: 100,
                });
                build_agent_learning_data_view_v0(
                    intent,
                    &learning_policy(intent),
                    &artifacts,
                    &derive_agent_private_learning_state_v0(intent),
                )
                .unwrap()
            })
            .collect();
        (intents, views)
    }

    #[test]
    fn learning_intents_are_agent_owned_distinct_and_policy_validated() {
        let intents = learning_intents();
        assert_eq!(intents.len(), 3);
        assert_eq!(
            intents
                .iter()
                .map(|intent| intent.intent_digest.as_str())
                .collect::<BTreeSet<_>>()
                .len(),
            3
        );
        assert!(intents.iter().all(|intent| {
            validate_agent_learning_intent_v0(intent, &learning_policy(intent)).is_ok()
        }));

        let mut invalid = intents[0].clone();
        invalid.market_scopes = vec![AcquisitionMarketScope::Unknown];
        assert!(validate_agent_learning_intent_v0(&invalid, &learning_policy(&invalid)).is_err());
        let base = plan_agent_data_intent(
            "momentum_trend_fast",
            AgentKind::MomentumTrendFast,
            &universe(),
            &learning_policy(&intents[0]),
            100,
        );
        assert!(
            create_agent_learning_intent_v0(
                &LearningDataCallerV0::Chair,
                &base,
                &learning_policy(&intents[0]),
                100,
            )
            .is_err()
        );
    }

    #[test]
    fn learning_broker_reuses_semantic_dedup_and_preserves_intents() {
        let policies = default_agent_data_policies();
        let mut intents = learning_intents();
        let momentum = intents
            .iter_mut()
            .find(|intent| intent.agent_kind == AgentKind::MomentumTrendFast)
            .unwrap();
        momentum.required_datasets = vec![DatasetKind::VolatilityDaily];
        momentum.optional_datasets.clear();
        stabilize_learning_intent_v0(momentum);
        momentum.intent_digest = agent_learning_intent_digest_v0(momentum);
        let momentum_request = momentum.clone();
        let risk = intents
            .iter_mut()
            .find(|intent| intent.agent_kind == AgentKind::CycleRiskSkeptic)
            .unwrap();
        risk.market_scopes = momentum_request.market_scopes.clone();
        risk.symbols = momentum_request.symbols.clone();
        risk.required_datasets = momentum_request.required_datasets.clone();
        risk.optional_datasets.clear();
        risk.cadence = momentum_request.cadence.clone();
        risk.lookback = momentum_request.lookback.clone();
        risk.maximum_staleness_ms = momentum_request.maximum_staleness_ms;
        stabilize_learning_intent_v0(risk);
        risk.intent_digest = agent_learning_intent_digest_v0(risk);
        intents.retain(|intent| intent.agent_kind != AgentKind::ValueQualityFilter);
        let before = intents.clone();
        let plan = build_learning_acquisition_plan_v0(
            &intents,
            &policies,
            &ReadOnlyProviderRegistry::default(),
            AcquisitionMode::LocalSnapshotReplay,
            &AcquisitionPolicy::default(),
        )
        .unwrap();
        assert_eq!(plan.planned_requests.len(), 1);
        assert_eq!(plan.deduplicated_request_count, 1);
        assert_eq!(plan.planned_requests[0].requested_by_agents.len(), 2);
        assert_eq!(intents, before);

        let risk = intents
            .iter_mut()
            .find(|intent| intent.agent_kind == AgentKind::CycleRiskSkeptic)
            .unwrap();
        risk.cadence = "1h".into();
        risk.intent_digest = agent_learning_intent_digest_v0(risk);
        let separate = build_learning_acquisition_plan_v0(
            &intents,
            &policies,
            &ReadOnlyProviderRegistry::default(),
            AcquisitionMode::LocalSnapshotReplay,
            &AcquisitionPolicy::default(),
        )
        .unwrap();
        assert_eq!(separate.planned_requests.len(), 2);
        assert_eq!(separate.deduplicated_request_count, 0);
    }

    #[test]
    fn learning_views_fan_out_shared_raw_without_sharing_learning_state() {
        let (intents, views) = ready_learning_views();
        assert!(
            views
                .iter()
                .all(|view| view.decision_gate == EvidenceDecisionGate::Ready)
        );
        assert_eq!(
            views
                .iter()
                .map(|view| view.view_digest.as_str())
                .collect::<BTreeSet<_>>()
                .len(),
            3
        );
        assert_eq!(
            views
                .iter()
                .map(|view| view.private_namespace_digest.as_str())
                .collect::<BTreeSet<_>>()
                .len(),
            3
        );
        assert_eq!(
            views
                .iter()
                .map(|view| view.training_ledger_digest.as_str())
                .collect::<BTreeSet<_>>()
                .len(),
            3
        );
        let shared_volatility = stable_hash_string("shared:VolatilityDaily");
        assert_eq!(
            views
                .iter()
                .filter(|view| view.source_artifact_digests.contains(&shared_volatility))
                .count(),
            2
        );
        let firewall = learning_data_chair_firewall_proof_v0();
        let proof = agent_learning_independence_proof_v0(&intents, &views, &firewall);
        assert!(proof.all_invariants_pass);
        assert!(proof.shared_raw_does_not_imply_shared_learning);
        assert!(proof.private_artifact_isolation);
    }

    #[test]
    fn learning_views_reject_private_crossing_cutoff_leakage_and_unauthorized_data() {
        let intent = learning_intents().remove(0);
        let policy = learning_policy(&intent);
        let private_state = derive_agent_private_learning_state_v0(&intent);
        let base = LearningDataArtifactRefV0 {
            artifact_digest: stable_hash_string("private-crossing"),
            dataset_kind: intent.required_datasets[0],
            visibility: LearningDataVisibilityV0::AgentPrivateDerived,
            owner_agent_id: Some("value_quality_filter".into()),
            maximum_event_timestamp_ms: 100,
        };
        assert!(
            build_agent_learning_data_view_v0(&intent, &policy, &[base.clone()], &private_state)
                .is_err()
        );
        let mut leaked = base.clone();
        leaked.visibility = LearningDataVisibilityV0::SharedCanonicalRaw;
        leaked.owner_agent_id = None;
        leaked.maximum_event_timestamp_ms = 101;
        assert!(
            build_agent_learning_data_view_v0(&intent, &policy, &[leaked], &private_state).is_err()
        );
        let mut unauthorized = base;
        unauthorized.visibility = LearningDataVisibilityV0::SharedCanonicalRaw;
        unauthorized.owner_agent_id = None;
        unauthorized.dataset_kind = DatasetKind::QuarterlyFundamentals;
        assert!(
            build_agent_learning_data_view_v0(&intent, &policy, &[unauthorized], &private_state)
                .is_err()
        );
    }

    #[test]
    fn missing_required_learning_evidence_abstains_without_fabrication() {
        let intent = learning_intents().remove(0);
        let view = build_agent_learning_data_view_v0(
            &intent,
            &learning_policy(&intent),
            &[],
            &derive_agent_private_learning_state_v0(&intent),
        )
        .unwrap();
        assert_eq!(view.decision_gate, EvidenceDecisionGate::Abstain);
        assert_eq!(view.missing_required_datasets, intent.required_datasets);
        assert!(view.source_artifact_digests.is_empty());
    }

    #[test]
    fn chair_firewall_denies_every_learning_data_authority() {
        let proof = learning_data_chair_firewall_proof_v0();
        assert!(proof.all_invariants_pass);
        assert!(!authorize_learning_data_action_v0(
            &LearningDataCallerV0::Chair,
            LearningDataAuthorityActionV0::CallBroker,
        ));
        assert!(authorize_learning_data_action_v0(
            &LearningDataCallerV0::NeutralBroker,
            LearningDataAuthorityActionV0::CallBroker,
        ));
        assert_eq!(proof.proof_digest, chair_firewall_proof_digest_v0(&proof));
    }

    #[test]
    fn learning_view_protobuf_round_trip_and_wire_identity_are_semantic() {
        let (_, views) = ready_learning_views();
        let view = &views[0];
        let encoded = encode_agent_learning_data_view_protobuf_v0(view).unwrap();
        let (envelope, decoded) = decode_agent_learning_data_view_protobuf_v0(&encoded).unwrap();
        assert_eq!(&decoded, view);
        assert_eq!(envelope.semantic_digest, view.view_digest);

        let mut alternate =
            CanonicalLearningArtifactEnvelopeProtobufV0::decode(encoded.as_slice()).unwrap();
        alternate.payload.extend_from_slice(&[0xf8, 0x01, 0x01]);
        alternate.payload_length = alternate.payload.len() as u64;
        alternate.payload_digest = canonical_hash_hex(&alternate.payload);
        let alternate_bytes = alternate.encode_to_vec();
        assert_ne!(encoded, alternate_bytes);
        let (_, alternate_view) =
            decode_agent_learning_data_view_protobuf_v0(&alternate_bytes).unwrap();
        assert_eq!(alternate_view.view_digest, view.view_digest);
        assert_eq!(alternate_view, *view);
    }

    #[test]
    fn learning_view_protobuf_rejects_every_envelope_and_payload_corruption() {
        let (_, views) = ready_learning_views();
        let encoded = encode_agent_learning_data_view_protobuf_v0(&views[0]).unwrap();
        let original =
            CanonicalLearningArtifactEnvelopeProtobufV0::decode(encoded.as_slice()).unwrap();

        let mut wrong = original.clone();
        wrong.magic = "wrong".into();
        assert!(decode_agent_learning_data_view_protobuf_v0(&wrong.encode_to_vec()).is_err());
        wrong = original.clone();
        wrong.envelope_version = 1;
        assert!(decode_agent_learning_data_view_protobuf_v0(&wrong.encode_to_vec()).is_err());
        wrong = original.clone();
        wrong.schema_name = "wrong".into();
        assert!(decode_agent_learning_data_view_protobuf_v0(&wrong.encode_to_vec()).is_err());
        wrong = original.clone();
        wrong.payload_length += 1;
        assert!(decode_agent_learning_data_view_protobuf_v0(&wrong.encode_to_vec()).is_err());
        wrong = original.clone();
        wrong.payload_digest = "0000000000000000".into();
        assert!(decode_agent_learning_data_view_protobuf_v0(&wrong.encode_to_vec()).is_err());
        wrong = original.clone();
        wrong.semantic_digest = "0000000000000000".into();
        assert!(decode_agent_learning_data_view_protobuf_v0(&wrong.encode_to_vec()).is_err());
        wrong = original.clone();
        wrong.source_artifact_digests = vec!["invalid".into()];
        assert!(decode_agent_learning_data_view_protobuf_v0(&wrong.encode_to_vec()).is_err());
        wrong = original.clone();
        let mut wrong_agent =
            AgentLearningDataViewProtobufV0::decode(wrong.payload.as_slice()).unwrap();
        wrong_agent.agent_id = "chair".into();
        wrong.payload = wrong_agent.encode_to_vec();
        wrong.payload_length = wrong.payload.len() as u64;
        wrong.payload_digest = canonical_hash_hex(&wrong.payload);
        assert!(decode_agent_learning_data_view_protobuf_v0(&wrong.encode_to_vec()).is_err());
        wrong = original;
        wrong.payload = vec![0xff];
        wrong.payload_length = 1;
        wrong.payload_digest = canonical_hash_hex(&wrong.payload);
        assert!(decode_agent_learning_data_view_protobuf_v0(&wrong.encode_to_vec()).is_err());
    }

    #[test]
    fn learning_view_atomic_storage_and_json_sidecar_reopen_verified() {
        let (_, views) = ready_learning_views();
        let view = &views[0];
        let output_dir = Path::new(LEARNING_DATA_NAMESPACE_V0)
            .join(format!("acquisition-test-{}", std::process::id()));
        if output_dir.is_dir() {
            fs::remove_dir_all(&output_dir).unwrap();
        }
        let path = write_and_verify_agent_learning_data_view_v0(view, &output_dir).unwrap();
        assert_eq!(
            read_and_verify_agent_learning_data_view_v0(&path).unwrap(),
            *view
        );

        let legacy = output_dir.join("legacy-agent-view.json");
        let original = serde_json::to_vec(view).unwrap();
        fs::write(&legacy, &original).unwrap();
        let sidecar = migrate_legacy_learning_view_json_v0(&legacy).unwrap();
        assert_eq!(fs::read(&legacy).unwrap(), original);
        assert_eq!(
            read_and_verify_agent_learning_data_view_v0(&sidecar).unwrap(),
            *view
        );
        fs::remove_dir_all(&output_dir).unwrap();
    }

    #[test]
    fn learning_network_pilot_is_deferred_and_isolated_with_zero_authority() {
        let protected_paths = [
            "config/local/prospective_shadow_challenge_v0.json",
            "config/local/cycle_risk_prospective_local_state_v0.json",
            "config/local/prospective_external_row_admission_registration_v0.json",
            "config/local/prospective_external_row_capsule_v0.json",
            "config/local/prospective_public_export_acquisition_registration_v0.json",
            "config/local/prospective_public_export_acquisition_receipt_v0.json",
            "config/local/prospective_network_export_capsule_v0.json",
            "config/local/prospective_one_time_opening_registration_v0.json",
        ];
        let before = protected_paths
            .iter()
            .map(fs::read)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        let plan = plan_learning_network_pilot_v0(&LearningNetworkPilotInputV0 {
            explicit_network_consent: false,
            non_overlapping_request_proven: false,
            provider_approved_read_only: false,
            credential_scope_approved: false,
            bounded_response: true,
        });
        assert_eq!(
            plan.status,
            LearningNetworkPilotStatusV0::DeferredToProtectProspectiveEvaluation
        );
        assert!(safe_learning_data_path_v0(Path::new(
            &plan.storage_namespace
        )));
        assert!(!plan.storage_namespace.contains("prospective"));
        assert_eq!(
            (
                plan.maximum_requests,
                plan.maximum_concurrency,
                plan.maximum_retries
            ),
            (1, 1, 0)
        );
        assert_eq!(plan.safety_counters.active_committee_count, 3);
        assert_eq!(plan.safety_counters.network_requests, 0);
        assert_eq!(plan.safety_counters.credential_reads, 0);
        assert_eq!(plan.safety_counters.prospective_artifact_mutations, 0);
        assert_eq!(plan.safety_counters.prospective_label_reads, 0);
        assert_eq!(plan.safety_counters.chair_decisions, 0);
        assert_eq!(plan.safety_counters.votes, 0);
        assert_eq!(plan.safety_counters.rewards, 0);
        assert_eq!(plan.safety_counters.penalties, 0);
        assert_eq!(plan.safety_counters.voice_changes, 0);
        assert_eq!(plan.safety_counters.executions, 0);
        let after = protected_paths
            .iter()
            .map(fs::read)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(before, after);
    }

    #[test]
    fn learning_internet_manifest_binds_raw_normalized_and_cutoff_identity() {
        let manifest =
            seal_learning_data_provenance_manifest_v0(LearningDataProvenanceManifestV0 {
                source_provider_id: "mock-readonly".into(),
                source_type: "local-fixture".into(),
                acquisition_request_identity: stable_hash_string("request"),
                fetch_timestamp_ms: 100,
                publication_event_timestamp_ms: Some(90),
                raw_content_digest: stable_hash_string("raw"),
                parser_version: "parser-v0".into(),
                normalized_artifact_digest: stable_hash_string("normalized"),
                sanitized: true,
                credential_free: true,
                information_cutoff_ms: 100,
                usage_classification: LearningDataUsageClassificationV0::ResearchOnlyUnconsumed,
                manifest_digest: String::new(),
            })
            .unwrap();
        assert!(valid_learning_digest_v0(&manifest.manifest_digest));
        let mut leaked = manifest;
        leaked.publication_event_timestamp_ms = Some(101);
        assert!(seal_learning_data_provenance_manifest_v0(leaked).is_err());
    }

    #[test]
    fn three_agents_plan_distinct_data_intents_without_provider_access() {
        let plans = default_agent_data_policies();
        let intents = canonical_current_agent_states()
            .iter()
            .filter_map(|state| {
                plans
                    .iter()
                    .find(|policy| policy.agent_kind == state.kind)
                    .map(|policy| {
                        plan_agent_data_intent(
                            state.agent_id.clone(),
                            state.kind,
                            &universe(),
                            policy,
                            100,
                        )
                    })
            })
            .collect::<Vec<_>>();
        assert_eq!(intents.len(), 3);
        assert_ne!(intents[0].required_datasets, intents[1].required_datasets);
        assert_ne!(intents[1].required_datasets, intents[2].required_datasets);
        assert!(intents.iter().all(|intent| !intent.symbols.is_empty()));
    }

    #[test]
    fn broker_deduplicates_requests_and_preserves_agent_mapping() {
        let policy = default_agent_data_policies()
            .into_iter()
            .find(|policy| policy.agent_kind == AgentKind::MomentumTrendFast)
            .expect("momentum policy");
        let first = plan_agent_data_intent(
            "one",
            AgentKind::MomentumTrendFast,
            &universe(),
            &policy,
            10,
        );
        let second = plan_agent_data_intent(
            "two",
            AgentKind::MomentumTrendFast,
            &universe(),
            &policy,
            10,
        );
        let mut registry = ReadOnlyProviderRegistry::default();
        registry.register(mock_capabilities());
        let plan = build_acquisition_plan(
            &[first, second],
            &registry,
            AcquisitionMode::Mock,
            &AcquisitionPolicy::default(),
        );
        assert!(plan.deduplicated_request_count > 0);
        assert!(
            plan.agent_request_mapping
                .values()
                .any(|agents| agents == &vec!["one".to_string(), "two".to_string()])
        );
    }

    #[test]
    fn no_approved_provider_fails_closed_without_synthetic_fallback() {
        let policy = AcquisitionPolicy::default();
        let cycle = plan_autonomous_data_cycle(
            &input(AcquisitionMode::ApprovedReadOnlyNetwork, 100),
            &ReadOnlyProviderRegistry::default(),
            &policy,
        );
        assert!(cycle.acquisition_plan.planned_requests.is_empty());
        assert!(
            cycle
                .acquisition_plan
                .rejected_requests
                .iter()
                .all(|request| request
                    .reason_codes
                    .contains(&ReasonCode::NoApprovedReadOnlyProviderConfigured))
        );
    }

    #[test]
    fn mock_cycle_creates_frozen_immutable_snapshots_and_bundles() {
        let mut registry = ReadOnlyProviderRegistry::default();
        registry.register(mock_capabilities());
        let mut broker = DataAcquisitionBroker::new(registry, AcquisitionPolicy::default());
        let mut provider = mock_provider(100);
        let result = execute_autonomous_data_cycle(
            &input(AcquisitionMode::Mock, 100),
            &mut broker,
            Some(&mut provider),
        );
        assert!(!result.new_snapshots.is_empty());
        assert!(result.frozen_snapshot_set.frozen);
        assert!(
            result
                .agent_evidence_bundles
                .iter()
                .all(|bundle| bundle.frozen)
        );
        assert!(
            result
                .new_snapshots
                .iter()
                .all(|snapshot| broker.snapshot_store.verify_digest(&snapshot.snapshot_id))
        );
        let snapshot = result.new_snapshots[0].clone();
        assert_eq!(
            broker.snapshot_store.put(snapshot.clone()),
            Err(ReasonCode::DataSnapshotImmutable)
        );
        let mut corrupted = snapshot;
        corrupted.content_digest = "corrupted".to_string();
        let mut store = InMemorySnapshotStore::default();
        assert_eq!(
            store.put(corrupted),
            Err(ReasonCode::DataSnapshotDigestMismatch)
        );
    }

    #[test]
    fn exact_request_key_reuse_does_not_require_fallback_metadata() {
        let request = fallback_request("exact-key");
        let mut snapshot = fallback_snapshot(&request, 10);
        snapshot.compatibility = None;
        let mut broker = DataAcquisitionBroker::new(
            ReadOnlyProviderRegistry::default(),
            AcquisitionPolicy::default(),
        );
        broker.snapshot_store.put(snapshot).unwrap();
        let mut result = BrokerExecutionResult::default();
        broker.replay_snapshot(&request, 10, &mut result);
        assert_eq!(result.reused_snapshots.len(), 1);
        assert_eq!(
            result.receipts[0].status,
            AcquisitionReceiptStatus::ReusedSnapshot
        );
    }

    #[test]
    fn compatible_daily_fallback_requires_explicit_semantics() {
        let source = fallback_request("source-key");
        let target = fallback_request("target-key");
        let snapshot = fallback_snapshot(&source, 10);
        let mut store = InMemorySnapshotStore::default();
        store.put(snapshot.clone()).unwrap();
        assert_eq!(store.find_latest_compatible(&target), Some(snapshot));
    }

    #[test]
    fn compatible_fallback_rejects_different_cadence() {
        let source = fallback_request("source-key");
        let mut target = fallback_request("target-key");
        target.cadence = "1h".to_string();
        let mut store = InMemorySnapshotStore::default();
        store.put(fallback_snapshot(&source, 10)).unwrap();
        assert!(store.find_latest_compatible(&target).is_none());
    }

    #[test]
    fn compatible_fallback_rejects_different_dataset() {
        let source = fallback_request("source-key");
        let mut target = fallback_request("target-key");
        target.dataset_kind = DatasetKind::AdjustedDailyOhlcv;
        let mut store = InMemorySnapshotStore::default();
        store.put(fallback_snapshot(&source, 10)).unwrap();
        assert!(store.find_latest_compatible(&target).is_none());
    }

    #[test]
    fn compatible_fallback_rejects_different_cutoff() {
        let source = fallback_request("source-key");
        let mut target = fallback_request("target-key");
        target.lookback.end_timestamp_ms = Some(11);
        let mut store = InMemorySnapshotStore::default();
        store.put(fallback_snapshot(&source, 10)).unwrap();
        assert!(store.find_latest_compatible(&target).is_none());
    }

    #[test]
    fn compatible_fallback_is_rejected_when_stale() {
        let source = fallback_request("source-key");
        let target = fallback_request("target-key");
        let mut broker = DataAcquisitionBroker::new(
            ReadOnlyProviderRegistry::default(),
            AcquisitionPolicy::default(),
        );
        broker
            .snapshot_store
            .put(fallback_snapshot(&source, 10))
            .unwrap();
        let mut result = BrokerExecutionResult::default();
        broker.replay_snapshot(&target, 1_000, &mut result);
        assert!(result.reused_snapshots.is_empty());
        assert!(
            result.receipts[0]
                .reason_codes
                .contains(&ReasonCode::EvidenceStaleRejected)
        );
    }

    #[test]
    fn compatible_fallback_rejects_failed_quality() {
        let source = fallback_request("source-key");
        let target = fallback_request("target-key");
        let mut snapshot = fallback_snapshot(&source, 10);
        snapshot.quality_summary.accepted = false;
        let mut store = InMemorySnapshotStore::default();
        store.put(snapshot).unwrap();
        assert!(store.find_latest_compatible(&target).is_none());
    }

    #[test]
    fn compatible_fallback_rejects_digest_corruption() {
        let source = fallback_request("source-key");
        let target = fallback_request("target-key");
        let mut snapshot = fallback_snapshot(&source, 10);
        snapshot.content_digest = "corrupted".to_string();
        let mut store = InMemorySnapshotStore::default();
        store
            .snapshots
            .insert(snapshot.snapshot_id.clone(), snapshot);
        assert!(store.find_latest_compatible(&target).is_none());
    }

    #[test]
    fn invalid_or_unsafe_provider_data_never_creates_snapshot() {
        let mut registry = ReadOnlyProviderRegistry::default();
        registry.register(mock_capabilities());
        let mut broker = DataAcquisitionBroker::new(registry, AcquisitionPolicy::default());
        let mut provider = mock_provider(100);
        provider
            .default_response
            .as_mut()
            .expect("response")
            .normalized_dataset
            .source = "Authorization secret".to_string();
        let result = execute_autonomous_data_cycle(
            &input(AcquisitionMode::Mock, 100),
            &mut broker,
            Some(&mut provider),
        );
        assert!(result.new_snapshots.is_empty());
        assert!(result.acquisition_receipts.iter().any(|receipt| {
            receipt
                .reason_codes
                .contains(&ReasonCode::DataSnapshotUnsafeContentRejected)
        }));
    }

    #[test]
    fn broker_rejects_unknown_dataset_oversize_and_provider_failures() {
        let unknown = AgentDataIntent {
            agent_id: "unknown".to_string(),
            agent_kind: AgentKind::MomentumTrendFast,
            market_scope: AcquisitionMarketScope::UsStocks,
            symbols: vec!["AAA".to_string()],
            required_datasets: vec![DatasetKind::Unknown],
            optional_datasets: vec![],
            lookback: DataLookback {
                bars: 2,
                start_timestamp_ms: None,
                end_timestamp_ms: Some(1),
            },
            target_cadence: "1d".to_string(),
            max_staleness_ms: 10,
            priority: DataPriority::Required,
            reason_codes: vec![],
        };
        let plan = build_acquisition_plan(
            &[unknown],
            &ReadOnlyProviderRegistry::default(),
            AcquisitionMode::Mock,
            &AcquisitionPolicy::default(),
        );
        assert!(
            plan.rejected_requests[0]
                .reason_codes
                .contains(&ReasonCode::DatasetKindUnknown)
        );

        let mut registry = ReadOnlyProviderRegistry::default();
        registry.register(mock_capabilities());
        let mut policy = AcquisitionPolicy::default();
        policy.max_response_bytes = 1;
        let mut broker = DataAcquisitionBroker::new(registry, policy);
        let mut provider = mock_provider(10);
        let oversized = execute_autonomous_data_cycle(
            &input(AcquisitionMode::Mock, 10),
            &mut broker,
            Some(&mut provider),
        );
        assert!(oversized.acquisition_receipts.iter().any(|receipt| {
            receipt
                .reason_codes
                .contains(&ReasonCode::AcquisitionResponseTooLarge)
        }));

        let mut registry = ReadOnlyProviderRegistry::default();
        registry.register(mock_capabilities());
        let mut broker = DataAcquisitionBroker::new(registry, AcquisitionPolicy::default());
        let mut provider = mock_provider(10);
        provider.default_failure = Some(ProviderFetchFailure::TimedOut);
        let timed_out = execute_autonomous_data_cycle(
            &input(AcquisitionMode::Mock, 10),
            &mut broker,
            Some(&mut provider),
        );
        assert!(timed_out.acquisition_receipts.iter().any(|receipt| {
            receipt
                .reason_codes
                .contains(&ReasonCode::AcquisitionTimedOut)
        }));
        assert!(timed_out.new_snapshots.is_empty());
    }

    #[test]
    fn rate_limits_and_permission_errors_are_not_retried() {
        for failure in [
            ProviderFetchFailure::RateLimited,
            ProviderFetchFailure::PermissionDenied,
        ] {
            let mut registry = ReadOnlyProviderRegistry::default();
            registry.register(mock_capabilities());
            let mut policy = AcquisitionPolicy::default();
            policy.max_retries = 3;
            let mut broker = DataAcquisitionBroker::new(registry, policy);
            let mut provider = mock_provider(10);
            provider.default_failure = Some(failure.clone());
            let result = execute_autonomous_data_cycle(
                &input(AcquisitionMode::Mock, 10),
                &mut broker,
                Some(&mut provider),
            );
            let attempted = result
                .acquisition_receipts
                .iter()
                .filter(|receipt| {
                    receipt.provider_id.as_deref() == Some("mock-readonly")
                        && receipt.attempt_count > 0
                })
                .collect::<Vec<_>>();
            assert!(!attempted.is_empty());
            assert!(attempted.iter().all(|receipt| receipt.attempt_count == 1));
        }
    }

    #[test]
    fn missing_required_evidence_binds_proposal_to_abstain() {
        let mut broker = DataAcquisitionBroker::new(
            ReadOnlyProviderRegistry::default(),
            AcquisitionPolicy::default(),
        );
        let result = execute_autonomous_data_cycle(
            &input(AcquisitionMode::Disabled, 100),
            &mut broker,
            None,
        );
        let bundle = result.agent_evidence_bundles.first().expect("bundle");
        let proposal = AgentProposal {
            proposal_id: "proposal".to_string(),
            agent_id: bundle.agent_id.clone(),
            stance: Stance::Buy,
            confidence: 0.9,
            expected_edge: 0.1,
            expected_drawdown: 0.1,
            no_trade_probability: 0.0,
            horizon: crate::league::Horizon::Swing,
            market: "US".to_string(),
            symbol: "AAA".to_string(),
            reason_codes: vec![],
        };
        let bound = bind_proposal_to_frozen_evidence(proposal, bundle);
        assert_eq!(bound.proposal.stance, Stance::Abstain);
        assert!(bound.frozen);
    }
}
