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
        && compatibility.maximum_staleness_ms <= request.max_staleness_ms
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
pub const PERSISTED_LEARNING_INTENT_PROJECTION_VERSION_V1: &str =
    "persisted-agent-learning-intent-projection-v1";
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

const CANONICAL_VIEW_GAP_REPORT_VERSION_V1: &str = "agent-canonical-view-gap-report-v1";
const LEARNING_EVIDENCE_PROVIDER_CONTRACT_VERSION_V1: &str =
    "learning-evidence-provider-contract-v1";
const LEARNING_EVIDENCE_REGISTRATION_VERSION_V1: &str =
    "learning-evidence-acquisition-registration-v1";
const LEARNING_EVIDENCE_RECEIPT_VERSION_V1: &str = "learning-evidence-request-receipt-v1";
const DAILY_CADENCE_MS_V1: u64 = 86_400_000;
const MAX_LEARNING_EVIDENCE_SEGMENTS_V1: usize = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CanonicalViewGapStatusV1 {
    Complete,
    MissingRequiredEvidence,
    MissingOptionalEvidenceOnly,
    ProviderSingleRequestCapacityExceeded,
    SegmentedAcquisitionRequired,
    SegmentedAcquisitionUnsupported,
    ProviderUnavailable,
    ProviderContractUnverified,
    AmbiguousArtifacts,
    IncompatibleCadence,
    IncompatibleMarket,
    IncompatibleSymbol,
    CutoffMismatch,
    IntegrityFailure,
    TrainerUnavailable,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentCanonicalViewGapV1 {
    pub agent_id: String,
    pub intent_digest: String,
    pub market_scopes: Vec<AcquisitionMarketScope>,
    pub symbols: Vec<String>,
    pub cadence: String,
    pub lookback: DataLookback,
    pub information_cutoff_ms: u64,
    pub maximum_staleness_ms: u64,
    pub required_dataset_kinds: Vec<DatasetKind>,
    pub resolved_required_dataset_kinds: Vec<DatasetKind>,
    pub missing_required_dataset_kinds: Vec<DatasetKind>,
    pub optional_dataset_kinds: Vec<DatasetKind>,
    pub resolved_optional_dataset_kinds: Vec<DatasetKind>,
    pub missing_optional_dataset_kinds: Vec<DatasetKind>,
    pub usable_artifact_digests: Vec<String>,
    pub rejected_artifact_digests: Vec<String>,
    pub authorized_provider_ids: Vec<String>,
    pub trainer_available: bool,
    pub status: CanonicalViewGapStatusV1,
    pub gap_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LearningEvidenceProviderContractV1 {
    pub contract_version: String,
    pub provider_id: String,
    pub dataset_kind: DatasetKind,
    pub market_scope: AcquisitionMarketScope,
    pub symbols: Vec<String>,
    pub cadence: String,
    pub maximum_lookback_bars: usize,
    pub earliest_timestamp_ms: u64,
    pub latest_exclusive_timestamp_ms: u64,
    pub maximum_response_bytes: usize,
    pub credential_free: bool,
    pub read_only: bool,
    pub approved_for_network: bool,
    pub all_rows_finalized: bool,
    pub enabled: bool,
    pub contract_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LearningEvidenceSafetyCountersV1 {
    pub active_committee_count: usize,
    pub request_attempts: usize,
    pub retry_count: usize,
    pub transport_constructions: usize,
    pub credential_reads: usize,
    pub prospective_artifact_reads: usize,
    pub prospective_label_reads: usize,
    pub future_evaluation_reads: usize,
    pub active_model_changes: usize,
    pub chair_decisions: usize,
    pub votes: usize,
    pub rewards: usize,
    pub penalties: usize,
    pub voice_changes: usize,
    pub promotions: usize,
    pub executions: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentCanonicalViewGapReportV1 {
    pub report_version: String,
    pub gaps: Vec<AgentCanonicalViewGapV1>,
    pub provider_contract_digests: Vec<String>,
    pub safety_counters: LearningEvidenceSafetyCountersV1,
    pub report_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LearningEvidenceAcquisitionRegistrationV1 {
    pub registration_version: String,
    pub target_agent_ids: Vec<String>,
    pub gap_report_digests: Vec<String>,
    pub provider_id: String,
    pub provider_contract_digest: String,
    pub dataset_kind: DatasetKind,
    pub market_scope: AcquisitionMarketScope,
    pub symbols: Vec<String>,
    pub cadence: String,
    pub lookback: DataLookback,
    pub information_cutoff_ms: u64,
    pub expected_timestamp_ms: Vec<u64>,
    pub protected_registration_digests: Vec<String>,
    pub excluded_timestamp_ms: Vec<u64>,
    pub maximum_requests: usize,
    pub maximum_concurrency: usize,
    pub maximum_retries: usize,
    pub maximum_response_bytes: usize,
    pub credential_free_required: bool,
    pub read_only_required: bool,
    pub prospective_storage_forbidden: bool,
    pub registration_digest: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LearningEvidenceRequestStatusV1 {
    ReadyNotAttempted,
    EvidenceAcquired,
    ProviderRejected,
    TimeoutNoRetry,
    InvalidResponse,
    TechnicalFailure,
    RequestBudgetExhausted,
    RegistrationInvalid,
    GapNoLongerCurrent,
    EquivalentSnapshotExists,
    MissingNetworkConsent,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LearningEvidenceRequestReceiptV1 {
    pub receipt_version: String,
    pub registration_digest: String,
    pub provider_contract_digest: String,
    pub request_attempted: bool,
    pub request_count: usize,
    pub retry_count: usize,
    pub status: LearningEvidenceRequestStatusV1,
    pub http_status_class: Option<String>,
    pub returned_row_count: usize,
    pub verified_row_count: usize,
    pub raw_response_digest: Option<String>,
    pub provenance_manifest_digest: Option<String>,
    pub snapshot_digest: Option<String>,
    pub receipt_digest: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LearningEvidenceTransportResponseV1 {
    pub http_status_class: String,
    pub raw_response: Vec<u8>,
    pub response: ReadOnlyProviderResponse,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LearningEvidenceTransportFailureV1 {
    ProviderRejected {
        http_status_class: Option<String>,
        raw_response: Option<Vec<u8>>,
    },
    TimedOut,
    Technical,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LearningEvidenceAcquisitionResultV1 {
    pub status: LearningEvidenceRequestStatusV1,
    pub receipt: Option<LearningEvidenceRequestReceiptV1>,
    pub raw_response: Option<Vec<u8>>,
    pub provenance_manifest: Option<LearningDataProvenanceManifestV0>,
    pub snapshot: Option<DataSnapshot>,
    pub safety_counters: LearningEvidenceSafetyCountersV1,
}

pub fn seal_learning_evidence_provider_contract_v1(
    mut contract: LearningEvidenceProviderContractV1,
) -> Result<LearningEvidenceProviderContractV1, String> {
    contract.symbols.sort();
    contract.symbols.dedup();
    contract.contract_digest.clear();
    if contract.contract_version != LEARNING_EVIDENCE_PROVIDER_CONTRACT_VERSION_V1
        || contract.provider_id.trim().is_empty()
        || contract.dataset_kind == DatasetKind::Unknown
        || contract.market_scope == AcquisitionMarketScope::Unknown
        || contract.symbols.is_empty()
        || contract.cadence != "1d"
        || contract.maximum_lookback_bars == 0
        || contract.earliest_timestamp_ms >= contract.latest_exclusive_timestamp_ms
        || contract.maximum_response_bytes == 0
        || !contract.credential_free
        || !contract.read_only
        || !contract.approved_for_network
        || !contract.all_rows_finalized
        || (contract.provider_id == "upbit"
            && (contract.dataset_kind != DatasetKind::DailyOhlcv
                || contract.market_scope != AcquisitionMarketScope::BtcCrypto))
    {
        return Err("learning evidence provider contract rejected".into());
    }
    contract.contract_digest = learning_evidence_provider_contract_digest_v1(&contract);
    Ok(contract)
}

fn learning_evidence_provider_contract_digest_v1(
    contract: &LearningEvidenceProviderContractV1,
) -> String {
    stable_hash_string(&format!(
        "{}:{}:{:?}:{:?}:{:?}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}",
        contract.contract_version,
        contract.provider_id,
        contract.dataset_kind,
        contract.market_scope,
        contract.symbols,
        contract.cadence,
        contract.maximum_lookback_bars,
        contract.earliest_timestamp_ms,
        contract.latest_exclusive_timestamp_ms,
        contract.maximum_response_bytes,
        contract.credential_free,
        contract.read_only,
        contract.approved_for_network,
        contract.all_rows_finalized,
        contract.enabled,
    ))
}

fn validate_learning_evidence_provider_contract_v1(
    contract: &LearningEvidenceProviderContractV1,
) -> bool {
    seal_learning_evidence_provider_contract_v1(contract.clone()).as_ref() == Ok(contract)
}

fn stable_learning_strings_v1(values: &[String]) -> Vec<String> {
    let mut values = values.to_vec();
    values.sort();
    values.dedup();
    values
}

fn stable_learning_kinds_v1(values: &[DatasetKind]) -> Vec<DatasetKind> {
    let mut values = values.to_vec();
    values.sort();
    values.dedup();
    values
}

fn stable_learning_markets_v1(values: &[AcquisitionMarketScope]) -> Vec<AcquisitionMarketScope> {
    let mut values = values.to_vec();
    values.sort();
    values.dedup();
    values
}

fn learning_evidence_contract_supports_v1(
    contract: &LearningEvidenceProviderContractV1,
    dataset_kind: DatasetKind,
    market_scope: AcquisitionMarketScope,
    symbols: &[String],
    cadence: &str,
    lookback: &DataLookback,
    information_cutoff_ms: u64,
) -> bool {
    let timestamps = expected_learning_timestamps_v1(lookback);
    validate_learning_evidence_provider_contract_v1(contract)
        && contract.enabled
        && contract.dataset_kind == dataset_kind
        && contract.market_scope == market_scope
        && contract.symbols == stable_learning_strings_v1(symbols)
        && contract.cadence == cadence
        && lookback.bars <= contract.maximum_lookback_bars
        && lookback.end_timestamp_ms == Some(information_cutoff_ms)
        && timestamps.as_ref().is_some_and(|timestamps| {
            timestamps
                .first()
                .is_some_and(|start| *start >= contract.earliest_timestamp_ms)
                && timestamps
                    .last()
                    .is_some_and(|end| *end < contract.latest_exclusive_timestamp_ms)
        })
}

fn learning_evidence_contract_matches_identity_v1(
    contract: &LearningEvidenceProviderContractV1,
    dataset_kind: DatasetKind,
    market_scope: AcquisitionMarketScope,
    symbols: &[String],
) -> bool {
    validate_learning_evidence_provider_contract_v1(contract)
        && contract.dataset_kind == dataset_kind
        && contract.market_scope == market_scope
        && contract.symbols == stable_learning_strings_v1(symbols)
}

fn learning_evidence_contract_matches_range_v1(
    contract: &LearningEvidenceProviderContractV1,
    lookback: &DataLookback,
    information_cutoff_ms: u64,
) -> bool {
    let Some(timestamps) = expected_learning_timestamps_v1(lookback) else {
        return false;
    };
    lookback.end_timestamp_ms == Some(information_cutoff_ms)
        && lookback
            .start_timestamp_ms
            .is_none_or(|start| timestamps.first().copied() == Some(start))
        && timestamps
            .first()
            .is_some_and(|start| *start >= contract.earliest_timestamp_ms)
        && information_cutoff_ms <= contract.latest_exclusive_timestamp_ms
}

fn single_request_capacity_status_v1(
    required_rows: usize,
    maximum_rows: usize,
) -> Option<CanonicalViewGapStatusV1> {
    (required_rows > maximum_rows)
        .then_some(CanonicalViewGapStatusV1::ProviderSingleRequestCapacityExceeded)
}

fn exact_bounded_segment_count_v1(
    lookback: &DataLookback,
    cadence: &str,
    maximum_rows_per_segment: usize,
    approved_segment_cap: usize,
    response_dependent_pagination: bool,
) -> Result<usize, String> {
    if response_dependent_pagination
        || cadence != "1d"
        || lookback.bars == 0
        || maximum_rows_per_segment == 0
        || approved_segment_cap == 0
    {
        return Err("exact bounded learning evidence segmentation unavailable".into());
    }
    let timestamps = expected_learning_timestamps_v1(lookback)
        .ok_or_else(|| "exact bounded learning evidence segmentation unavailable".to_string())?;
    if lookback
        .start_timestamp_ms
        .is_some_and(|start| timestamps.first().copied() != Some(start))
    {
        return Err("exact bounded learning evidence segmentation unavailable".into());
    }
    let segment_count = lookback.bars.div_ceil(maximum_rows_per_segment);
    if segment_count == 0 || segment_count > approved_segment_cap {
        return Err("learning evidence segment cap exceeded".into());
    }
    Ok(segment_count)
}

fn snapshot_integrity_valid_for_gap_v1(snapshot: &DataSnapshot) -> bool {
    snapshot.schema_version == 1
        && (snapshot.provider_id != "upbit"
            || (snapshot.dataset_kind == DatasetKind::DailyOhlcv
                && snapshot.market_scope == AcquisitionMarketScope::BtcCrypto))
        && snapshot.snapshot_id == snapshot_id_from_semantic_digest_v1(&snapshot.content_digest)
        && snapshot.content_digest == canonical_snapshot_semantic_digest_v1(snapshot)
        && snapshot.row_count == snapshot.normalized_dataset.rows.len()
        && snapshot.quality_summary.accepted
        && snapshot.quality_summary.row_count == snapshot.row_count
        && snapshot.sanitized
        && snapshot.read_only
        && snapshot.provenance.sanitized
        && snapshot.provenance.credential_free
        && snapshot.provenance.provider_id == snapshot.provider_id
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
        && snapshot
            .normalized_dataset
            .rows
            .windows(2)
            .all(|pair| pair[0].timestamp_ms < pair[1].timestamp_ms)
        && snapshot.normalized_dataset.rows.iter().all(|row| {
            row.symbol == snapshot.normalized_dataset.symbol
                && row.open.is_finite()
                && row.high.is_finite()
                && row.low.is_finite()
                && row.close.is_finite()
                && row.volume.is_finite()
                && row.trade_value.is_none_or(f64::is_finite)
                && row.open > 0.0
                && row.high > 0.0
                && row.low > 0.0
                && row.close > 0.0
                && row.volume >= 0.0
                && row.trade_value.is_none_or(|value| value >= 0.0)
                && row.high >= row.open.max(row.close)
                && row.low <= row.open.min(row.close)
                && row.high >= row.low
        })
}

#[derive(Clone, Copy, Debug, Default)]
struct GapRejectionFlagsV1 {
    integrity: bool,
    cadence: bool,
    market: bool,
    symbol: bool,
    cutoff: bool,
    ambiguous: bool,
}

fn snapshot_matches_intent_dataset_v1(
    snapshot: &DataSnapshot,
    intent: &AgentLearningIntentV0,
    dataset_kind: DatasetKind,
    flags: &mut GapRejectionFlagsV1,
) -> bool {
    if snapshot.dataset_kind != dataset_kind {
        return false;
    }
    if !snapshot_integrity_valid_for_gap_v1(snapshot) {
        flags.integrity = true;
        return false;
    }
    if !intent.market_scopes.contains(&snapshot.market_scope) {
        flags.market = true;
        return false;
    }
    if stable_learning_strings_v1(&snapshot.symbols) != intent.symbols {
        flags.symbol = true;
        return false;
    }
    let Some(compatibility) = snapshot.compatibility.as_ref() else {
        flags.cadence = true;
        return false;
    };
    if compatibility.cadence != intent.cadence
        || compatibility.adjustment_semantics != adjustment_semantics_v1(dataset_kind)
        || compatibility.source_schema != "application/x-soma-normalized-dataset"
        || !compatibility.all_rows_finalized
    {
        flags.cadence = true;
        return false;
    }
    if snapshot.requested_lookback != intent.lookback
        || compatibility.requested_cutoff_timestamp_ms != Some(intent.information_cutoff_ms)
        || compatibility.maximum_staleness_ms > intent.maximum_staleness_ms
        || snapshot.actual_end_timestamp_ms.is_none_or(|end| {
            end > intent.information_cutoff_ms
                || intent.information_cutoff_ms.saturating_sub(end) > intent.maximum_staleness_ms
        })
    {
        flags.cutoff = true;
        return false;
    }
    true
}

fn canonical_view_gap_digest_v1(gap: &AgentCanonicalViewGapV1) -> String {
    stable_hash_string(&format!(
        "{}:{}:{:?}:{:?}:{}:{:?}:{}:{}:{:?}:{:?}:{:?}:{:?}:{:?}:{:?}:{:?}:{:?}:{}:{:?}",
        gap.agent_id,
        gap.intent_digest,
        gap.market_scopes,
        gap.symbols,
        gap.cadence,
        gap.lookback,
        gap.information_cutoff_ms,
        gap.maximum_staleness_ms,
        gap.required_dataset_kinds,
        gap.resolved_required_dataset_kinds,
        gap.missing_required_dataset_kinds,
        gap.optional_dataset_kinds,
        gap.resolved_optional_dataset_kinds,
        gap.missing_optional_dataset_kinds,
        gap.usable_artifact_digests,
        gap.rejected_artifact_digests,
        gap.trainer_available,
        (gap.status, &gap.authorized_provider_ids),
    ))
}

fn canonical_view_gap_report_digest_v1(report: &AgentCanonicalViewGapReportV1) -> String {
    stable_hash_string(&format!(
        "{}:{:?}:{:?}:{:?}",
        report.report_version,
        report
            .gaps
            .iter()
            .map(|gap| gap.gap_digest.as_str())
            .collect::<Vec<_>>(),
        report.provider_contract_digests,
        report.safety_counters,
    ))
}

fn zero_learning_evidence_safety_counters_v1() -> LearningEvidenceSafetyCountersV1 {
    LearningEvidenceSafetyCountersV1 {
        active_committee_count: 3,
        request_attempts: 0,
        retry_count: 0,
        transport_constructions: 0,
        credential_reads: 0,
        prospective_artifact_reads: 0,
        prospective_label_reads: 0,
        future_evaluation_reads: 0,
        active_model_changes: 0,
        chair_decisions: 0,
        votes: 0,
        rewards: 0,
        penalties: 0,
        voice_changes: 0,
        promotions: 0,
        executions: 0,
    }
}

pub fn derive_agent_canonical_view_gaps_v1(
    intents: &[AgentLearningIntentV0],
    policies: &[AgentDataPolicy],
    snapshots: &[DataSnapshot],
    trainer_capable_agent_ids: &BTreeSet<String>,
    provider_contracts: &[LearningEvidenceProviderContractV1],
) -> Result<AgentCanonicalViewGapReportV1, String> {
    if intents.len() != 3
        || provider_contracts
            .iter()
            .any(|contract| !validate_learning_evidence_provider_contract_v1(contract))
    {
        return Err("canonical view gap inputs rejected".into());
    }
    let mut gaps = Vec::new();
    for intent in intents {
        let policy = policies
            .iter()
            .find(|policy| policy.agent_kind == intent.agent_kind)
            .ok_or_else(|| "canonical view gap policy unavailable".to_string())?;
        let persisted_projection_valid = intent.intent_version
            == PERSISTED_LEARNING_INTENT_PROJECTION_VERSION_V1
            && expected_learning_agent_id_v0(intent.agent_kind) == Some(intent.agent_id.as_str())
            && !intent.intent_digest.is_empty()
            && !intent.market_scopes.is_empty()
            && intent
                .market_scopes
                .iter()
                .all(|market| policy.allowed_markets.contains(market))
            && stable_learning_kinds_v1(&intent.required_datasets)
                == stable_learning_kinds_v1(&policy.required_dataset_kinds)
            && stable_learning_kinds_v1(&intent.optional_datasets)
                == stable_learning_kinds_v1(&policy.optional_dataset_kinds)
            && !intent.cadence.trim().is_empty()
            && intent.lookback.bars > 0
            && intent.lookback.end_timestamp_ms == Some(intent.information_cutoff_ms)
            && intent.information_cutoff_ms > 0
            && intent.maximum_staleness_ms == policy.max_staleness_ms;
        if validate_agent_learning_intent_v0(intent, policy).is_err() && !persisted_projection_valid
        {
            return Err("canonical view gap intent rejected".into());
        }
        let mut resolved_required = Vec::new();
        let mut missing_required = Vec::new();
        let mut resolved_optional = Vec::new();
        let mut missing_optional = Vec::new();
        let mut usable = Vec::new();
        let mut rejected = Vec::new();
        let mut rejection_flags = GapRejectionFlagsV1::default();
        for (kind, required) in intent
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
            let mut matches = snapshots
                .iter()
                .filter(|snapshot| snapshot.dataset_kind == kind)
                .filter(|snapshot| {
                    let accepted = snapshot_matches_intent_dataset_v1(
                        snapshot,
                        intent,
                        kind,
                        &mut rejection_flags,
                    );
                    if !accepted {
                        rejected.push(snapshot.content_digest.clone());
                    }
                    accepted
                })
                .collect::<Vec<_>>();
            matches.sort_by(|left, right| {
                right
                    .fetched_at_ms
                    .cmp(&left.fetched_at_ms)
                    .then_with(|| right.row_count.cmp(&left.row_count))
                    .then_with(|| left.content_digest.cmp(&right.content_digest))
            });
            if matches.len() > 1
                && matches[0].fetched_at_ms == matches[1].fetched_at_ms
                && matches[0].row_count == matches[1].row_count
                && matches[0].content_digest != matches[1].content_digest
            {
                rejection_flags.ambiguous = true;
                rejected.extend(
                    matches
                        .iter()
                        .map(|snapshot| snapshot.content_digest.clone()),
                );
                if required {
                    missing_required.push(kind);
                } else {
                    missing_optional.push(kind);
                }
            } else if let Some(selected) = matches.first() {
                usable.push(selected.content_digest.clone());
                if required {
                    resolved_required.push(kind);
                } else {
                    resolved_optional.push(kind);
                }
            } else if required {
                missing_required.push(kind);
            } else {
                missing_optional.push(kind);
            }
        }
        let trainer_available = trainer_capable_agent_ids.contains(&intent.agent_id);
        let matches_missing_kind = |contract: &LearningEvidenceProviderContractV1| {
            missing_required.contains(&contract.dataset_kind)
        };
        let matches_market = |contract: &LearningEvidenceProviderContractV1| {
            intent.market_scopes.contains(&contract.market_scope)
        };
        let matches_symbols = |contract: &LearningEvidenceProviderContractV1| {
            contract.symbols == stable_learning_strings_v1(&intent.symbols)
        };
        let matches_cadence =
            |contract: &LearningEvidenceProviderContractV1| contract.cadence == intent.cadence;
        let matches_range = |contract: &LearningEvidenceProviderContractV1| {
            learning_evidence_contract_matches_range_v1(
                contract,
                &intent.lookback,
                intent.information_cutoff_ms,
            )
        };
        let is_exact_segment_provider = |contract: &LearningEvidenceProviderContractV1| {
            contract.provider_id == "upbit"
                && contract.dataset_kind == DatasetKind::DailyOhlcv
                && contract.market_scope == AcquisitionMarketScope::BtcCrypto
                && contract.cadence == "1d"
        };
        let exact_identity = |contract: &LearningEvidenceProviderContractV1| {
            intent.market_scopes.iter().any(|market| {
                missing_required.iter().any(|kind| {
                    learning_evidence_contract_matches_identity_v1(
                        contract,
                        *kind,
                        *market,
                        &intent.symbols,
                    )
                })
            })
        };
        let has_single_request_provider = provider_contracts.iter().any(|contract| {
            contract.enabled
                && exact_identity(contract)
                && matches_cadence(contract)
                && matches_range(contract)
                && learning_evidence_contract_supports_v1(
                    contract,
                    contract.dataset_kind,
                    contract.market_scope,
                    &intent.symbols,
                    &intent.cadence,
                    &intent.lookback,
                    intent.information_cutoff_ms,
                )
        });
        let capacity_exceeded_contracts = provider_contracts
            .iter()
            .filter(|contract| {
                contract.enabled
                    && exact_identity(contract)
                    && matches_cadence(contract)
                    && matches_range(contract)
                    && single_request_capacity_status_v1(
                        intent.lookback.bars,
                        contract.maximum_lookback_bars,
                    )
                    .is_some()
            })
            .collect::<Vec<_>>();
        let segmented_acquisition_required = capacity_exceeded_contracts.iter().any(|contract| {
            is_exact_segment_provider(contract)
                && exact_bounded_segment_count_v1(
                    &intent.lookback,
                    &intent.cadence,
                    contract.maximum_lookback_bars,
                    MAX_LEARNING_EVIDENCE_SEGMENTS_V1,
                    false,
                )
                .is_ok_and(|count| count > 1)
        });
        let segmented_acquisition_unsupported =
            capacity_exceeded_contracts.iter().any(|contract| {
                is_exact_segment_provider(contract)
                    && exact_bounded_segment_count_v1(
                        &intent.lookback,
                        &intent.cadence,
                        contract.maximum_lookback_bars,
                        MAX_LEARNING_EVIDENCE_SEGMENTS_V1,
                        false,
                    )
                    .is_err()
            });
        let single_request_capacity_exceeded = capacity_exceeded_contracts
            .iter()
            .any(|contract| !is_exact_segment_provider(contract));
        let mut authorized_provider_ids = provider_contracts
            .iter()
            .filter(|contract| {
                contract.enabled
                    && exact_identity(contract)
                    && matches_cadence(contract)
                    && matches_range(contract)
                    && (intent.lookback.bars <= contract.maximum_lookback_bars
                        || (is_exact_segment_provider(contract)
                            && exact_bounded_segment_count_v1(
                                &intent.lookback,
                                &intent.cadence,
                                contract.maximum_lookback_bars,
                                MAX_LEARNING_EVIDENCE_SEGMENTS_V1,
                                false,
                            )
                            .is_ok_and(|count| count > 1)))
            })
            .map(|contract| contract.provider_id.clone())
            .collect::<Vec<_>>();
        authorized_provider_ids.sort();
        authorized_provider_ids.dedup();
        let has_disabled_exact_provider = provider_contracts.iter().any(|contract| {
            !contract.enabled
                && exact_identity(contract)
                && matches_cadence(contract)
                && matches_range(contract)
        });
        let has_kind_contract = provider_contracts.iter().any(matches_missing_kind);
        let has_market_contract = provider_contracts
            .iter()
            .any(|contract| matches_missing_kind(contract) && matches_market(contract));
        let has_symbol_contract = provider_contracts.iter().any(|contract| {
            matches_missing_kind(contract) && matches_market(contract) && matches_symbols(contract)
        });
        let has_cadence_contract = provider_contracts.iter().any(|contract| {
            matches_missing_kind(contract)
                && matches_market(contract)
                && matches_symbols(contract)
                && matches_cadence(contract)
        });
        let status = if !trainer_available {
            CanonicalViewGapStatusV1::TrainerUnavailable
        } else if missing_required.is_empty() && missing_optional.is_empty() {
            CanonicalViewGapStatusV1::Complete
        } else if missing_required.is_empty() {
            CanonicalViewGapStatusV1::MissingOptionalEvidenceOnly
        } else if rejection_flags.integrity {
            CanonicalViewGapStatusV1::IntegrityFailure
        } else if rejection_flags.ambiguous {
            CanonicalViewGapStatusV1::AmbiguousArtifacts
        } else if has_single_request_provider {
            CanonicalViewGapStatusV1::MissingRequiredEvidence
        } else if segmented_acquisition_required {
            CanonicalViewGapStatusV1::SegmentedAcquisitionRequired
        } else if segmented_acquisition_unsupported {
            CanonicalViewGapStatusV1::SegmentedAcquisitionUnsupported
        } else if single_request_capacity_exceeded {
            CanonicalViewGapStatusV1::ProviderSingleRequestCapacityExceeded
        } else if has_disabled_exact_provider {
            CanonicalViewGapStatusV1::ProviderUnavailable
        } else if has_symbol_contract && !has_cadence_contract {
            CanonicalViewGapStatusV1::IncompatibleCadence
        } else if has_market_contract && !has_symbol_contract {
            CanonicalViewGapStatusV1::IncompatibleSymbol
        } else if has_kind_contract && !has_market_contract {
            CanonicalViewGapStatusV1::IncompatibleMarket
        } else if has_cadence_contract {
            CanonicalViewGapStatusV1::CutoffMismatch
        } else if rejection_flags.cadence {
            CanonicalViewGapStatusV1::IncompatibleCadence
        } else if rejection_flags.market {
            CanonicalViewGapStatusV1::IncompatibleMarket
        } else if rejection_flags.symbol {
            CanonicalViewGapStatusV1::IncompatibleSymbol
        } else if rejection_flags.cutoff {
            CanonicalViewGapStatusV1::CutoffMismatch
        } else if authorized_provider_ids.is_empty() {
            CanonicalViewGapStatusV1::ProviderContractUnverified
        } else {
            CanonicalViewGapStatusV1::MissingRequiredEvidence
        };
        let mut gap = AgentCanonicalViewGapV1 {
            agent_id: intent.agent_id.clone(),
            intent_digest: intent.intent_digest.clone(),
            market_scopes: stable_learning_markets_v1(&intent.market_scopes),
            symbols: stable_learning_strings_v1(&intent.symbols),
            cadence: intent.cadence.clone(),
            lookback: intent.lookback.clone(),
            information_cutoff_ms: intent.information_cutoff_ms,
            maximum_staleness_ms: intent.maximum_staleness_ms,
            required_dataset_kinds: stable_learning_kinds_v1(&intent.required_datasets),
            resolved_required_dataset_kinds: stable_learning_kinds_v1(&resolved_required),
            missing_required_dataset_kinds: stable_learning_kinds_v1(&missing_required),
            optional_dataset_kinds: stable_learning_kinds_v1(&intent.optional_datasets),
            resolved_optional_dataset_kinds: stable_learning_kinds_v1(&resolved_optional),
            missing_optional_dataset_kinds: stable_learning_kinds_v1(&missing_optional),
            usable_artifact_digests: stable_learning_strings_v1(&usable),
            rejected_artifact_digests: stable_learning_strings_v1(&rejected),
            authorized_provider_ids,
            trainer_available,
            status,
            gap_digest: String::new(),
        };
        gap.gap_digest = canonical_view_gap_digest_v1(&gap);
        gaps.push(gap);
    }
    gaps.sort_by(|left, right| left.agent_id.cmp(&right.agent_id));
    let mut provider_contract_digests = provider_contracts
        .iter()
        .map(|contract| contract.contract_digest.clone())
        .collect::<Vec<_>>();
    provider_contract_digests.sort();
    provider_contract_digests.dedup();
    let mut report = AgentCanonicalViewGapReportV1 {
        report_version: CANONICAL_VIEW_GAP_REPORT_VERSION_V1.into(),
        gaps,
        provider_contract_digests,
        safety_counters: zero_learning_evidence_safety_counters_v1(),
        report_digest: String::new(),
    };
    report.report_digest = canonical_view_gap_report_digest_v1(&report);
    Ok(report)
}

fn expected_learning_timestamps_v1(lookback: &DataLookback) -> Option<Vec<u64>> {
    let end = lookback.end_timestamp_ms?;
    let bars = u64::try_from(lookback.bars).ok()?;
    if bars == 0 {
        return None;
    }
    let start = if let Some(start) = lookback.start_timestamp_ms {
        let inclusive_end =
            start.checked_add(bars.checked_sub(1)?.checked_mul(DAILY_CADENCE_MS_V1)?)?;
        let exclusive_end = start.checked_add(bars.checked_mul(DAILY_CADENCE_MS_V1)?)?;
        if inclusive_end != end && exclusive_end != end {
            return None;
        }
        start
    } else {
        end.checked_sub(bars.checked_mul(DAILY_CADENCE_MS_V1)?)?
    };
    (0..bars)
        .map(|offset| start.checked_add(offset.checked_mul(DAILY_CADENCE_MS_V1)?))
        .collect()
}

fn learning_evidence_registration_digest_v1(
    registration: &LearningEvidenceAcquisitionRegistrationV1,
) -> String {
    stable_hash_string(&format!(
        "{:?}",
        (
            (
                registration.registration_version.as_str(),
                &registration.target_agent_ids,
                &registration.gap_report_digests,
                registration.provider_id.as_str(),
                registration.provider_contract_digest.as_str(),
                registration.dataset_kind,
                registration.market_scope,
                &registration.symbols,
                registration.cadence.as_str(),
            ),
            (
                &registration.lookback,
                registration.information_cutoff_ms,
                &registration.expected_timestamp_ms,
                &registration.protected_registration_digests,
                &registration.excluded_timestamp_ms,
                registration.maximum_requests,
                registration.maximum_concurrency,
                registration.maximum_retries,
                registration.maximum_response_bytes,
            ),
            (
                registration.credential_free_required,
                registration.read_only_required,
                registration.prospective_storage_forbidden,
            ),
        )
    ))
}

pub fn validate_learning_evidence_acquisition_registration_v1(
    registration: &LearningEvidenceAcquisitionRegistrationV1,
) -> Result<(), String> {
    let expected_timestamps = expected_learning_timestamps_v1(&registration.lookback);
    if registration.registration_version != LEARNING_EVIDENCE_REGISTRATION_VERSION_V1
        || registration.target_agent_ids.is_empty()
        || registration.target_agent_ids
            != stable_learning_strings_v1(&registration.target_agent_ids)
        || registration.gap_report_digests.len() != registration.target_agent_ids.len()
        || registration.gap_report_digests
            != stable_learning_strings_v1(&registration.gap_report_digests)
        || registration.provider_id.is_empty()
        || registration.provider_contract_digest.is_empty()
        || registration.dataset_kind == DatasetKind::Unknown
        || registration.market_scope == AcquisitionMarketScope::Unknown
        || registration.symbols.is_empty()
        || registration.symbols != stable_learning_strings_v1(&registration.symbols)
        || registration.cadence != "1d"
        || registration.lookback.bars == 0
        || registration.lookback.end_timestamp_ms != Some(registration.information_cutoff_ms)
        || registration
            .lookback
            .start_timestamp_ms
            .is_some_and(|start| {
                Some(start)
                    != expected_timestamps
                        .as_ref()
                        .and_then(|timestamps| timestamps.first().copied())
            })
        || expected_timestamps.as_ref() != Some(&registration.expected_timestamp_ms)
        || registration.expected_timestamp_ms.iter().any(|timestamp| {
            *timestamp > registration.information_cutoff_ms
                || registration.excluded_timestamp_ms.contains(timestamp)
        })
        || registration.protected_registration_digests.is_empty()
        || registration.protected_registration_digests
            != stable_learning_strings_v1(&registration.protected_registration_digests)
        || registration.excluded_timestamp_ms.is_empty()
        || {
            let mut values = registration.excluded_timestamp_ms.clone();
            values.sort();
            values.dedup();
            values != registration.excluded_timestamp_ms
        }
        || registration.maximum_requests != 1
        || registration.maximum_concurrency != 1
        || registration.maximum_retries != 0
        || registration.maximum_response_bytes == 0
        || !registration.credential_free_required
        || !registration.read_only_required
        || !registration.prospective_storage_forbidden
        || registration.registration_digest
            != learning_evidence_registration_digest_v1(registration)
    {
        return Err("learning evidence acquisition registration rejected".into());
    }
    Ok(())
}

#[derive(Clone)]
struct LearningRequestCandidateV1 {
    semantic_key: String,
    contract: LearningEvidenceProviderContractV1,
    dataset_kind: DatasetKind,
    market_scope: AcquisitionMarketScope,
    symbols: Vec<String>,
    cadence: String,
    lookback: DataLookback,
    information_cutoff_ms: u64,
    target_agent_ids: Vec<String>,
    gap_digests: Vec<String>,
}

pub fn select_learning_evidence_acquisition_registration_v1(
    report: &AgentCanonicalViewGapReportV1,
    provider_contracts: &[LearningEvidenceProviderContractV1],
    protected_registration_digests: &[String],
    excluded_timestamp_ms: &[u64],
) -> Result<Option<LearningEvidenceAcquisitionRegistrationV1>, String> {
    if report.report_version != CANONICAL_VIEW_GAP_REPORT_VERSION_V1
        || report.report_digest != canonical_view_gap_report_digest_v1(report)
    {
        return Err("canonical view gap report rejected".into());
    }
    let mut grouped = BTreeMap::<String, LearningRequestCandidateV1>::new();
    for gap in report.gaps.iter().filter(|gap| {
        gap.trainer_available
            && gap.status == CanonicalViewGapStatusV1::MissingRequiredEvidence
            && !gap.missing_required_dataset_kinds.is_empty()
    }) {
        for dataset_kind in &gap.missing_required_dataset_kinds {
            for market_scope in &gap.market_scopes {
                for contract in provider_contracts.iter().filter(|contract| {
                    learning_evidence_contract_supports_v1(
                        contract,
                        *dataset_kind,
                        *market_scope,
                        &gap.symbols,
                        &gap.cadence,
                        &gap.lookback,
                        gap.information_cutoff_ms,
                    )
                }) {
                    let semantic_key = format!(
                        "{}:{:?}:{:?}:{:?}:{}:{:?}:{}",
                        contract.provider_id,
                        dataset_kind,
                        market_scope,
                        gap.symbols,
                        gap.cadence,
                        gap.lookback,
                        gap.information_cutoff_ms,
                    );
                    let entry = grouped.entry(semantic_key.clone()).or_insert_with(|| {
                        LearningRequestCandidateV1 {
                            semantic_key,
                            contract: contract.clone(),
                            dataset_kind: *dataset_kind,
                            market_scope: *market_scope,
                            symbols: gap.symbols.clone(),
                            cadence: gap.cadence.clone(),
                            lookback: gap.lookback.clone(),
                            information_cutoff_ms: gap.information_cutoff_ms,
                            target_agent_ids: Vec::new(),
                            gap_digests: Vec::new(),
                        }
                    });
                    entry.target_agent_ids.push(gap.agent_id.clone());
                    entry.gap_digests.push(gap.gap_digest.clone());
                }
            }
        }
    }
    let mut candidates = grouped.into_values().collect::<Vec<_>>();
    for candidate in &mut candidates {
        candidate.target_agent_ids = stable_learning_strings_v1(&candidate.target_agent_ids);
        candidate.gap_digests = stable_learning_strings_v1(&candidate.gap_digests);
    }
    candidates.sort_by(|left, right| {
        right
            .contract
            .credential_free
            .cmp(&left.contract.credential_free)
            .then_with(|| {
                right
                    .target_agent_ids
                    .len()
                    .cmp(&left.target_agent_ids.len())
            })
            .then_with(|| left.lookback.bars.cmp(&right.lookback.bars))
            .then_with(|| left.semantic_key.cmp(&right.semantic_key))
    });
    let Some(selected) = candidates.into_iter().next() else {
        return Ok(None);
    };
    let lookback = selected.lookback;
    let expected_timestamp_ms = expected_learning_timestamps_v1(&lookback)
        .ok_or_else(|| "learning evidence range unavailable".to_string())?;
    let mut protected_registration_digests = protected_registration_digests.to_vec();
    protected_registration_digests.sort();
    protected_registration_digests.dedup();
    let mut excluded_timestamp_ms = excluded_timestamp_ms.to_vec();
    excluded_timestamp_ms.sort();
    excluded_timestamp_ms.dedup();
    let mut registration = LearningEvidenceAcquisitionRegistrationV1 {
        registration_version: LEARNING_EVIDENCE_REGISTRATION_VERSION_V1.into(),
        target_agent_ids: selected.target_agent_ids,
        gap_report_digests: selected.gap_digests,
        provider_id: selected.contract.provider_id,
        provider_contract_digest: selected.contract.contract_digest,
        dataset_kind: selected.dataset_kind,
        market_scope: selected.market_scope,
        symbols: selected.symbols,
        cadence: selected.cadence,
        lookback,
        information_cutoff_ms: selected.information_cutoff_ms,
        expected_timestamp_ms,
        protected_registration_digests,
        excluded_timestamp_ms,
        maximum_requests: 1,
        maximum_concurrency: 1,
        maximum_retries: 0,
        maximum_response_bytes: selected.contract.maximum_response_bytes,
        credential_free_required: true,
        read_only_required: true,
        prospective_storage_forbidden: true,
        registration_digest: String::new(),
    };
    registration.registration_digest = learning_evidence_registration_digest_v1(&registration);
    validate_learning_evidence_acquisition_registration_v1(&registration)?;
    Ok(Some(registration))
}

fn learning_evidence_receipt_digest_v1(receipt: &LearningEvidenceRequestReceiptV1) -> String {
    stable_hash_string(&format!(
        "{}:{}:{}:{}:{}:{}:{:?}:{:?}:{}:{}:{:?}:{:?}:{:?}:{:?}",
        receipt.receipt_version,
        receipt.registration_digest,
        receipt.provider_contract_digest,
        receipt.request_attempted,
        receipt.request_count,
        receipt.retry_count,
        receipt.status,
        receipt.http_status_class,
        receipt.returned_row_count,
        receipt.verified_row_count,
        receipt.raw_response_digest,
        receipt.provenance_manifest_digest,
        receipt.snapshot_digest,
        LEARNING_EVIDENCE_RECEIPT_VERSION_V1,
    ))
}

pub fn validate_learning_evidence_request_receipt_v1(
    receipt: &LearningEvidenceRequestReceiptV1,
) -> Result<(), String> {
    let success = receipt.status == LearningEvidenceRequestStatusV1::EvidenceAcquired;
    if receipt.receipt_version != LEARNING_EVIDENCE_RECEIPT_VERSION_V1
        || receipt.registration_digest.is_empty()
        || receipt.provider_contract_digest.is_empty()
        || !receipt.request_attempted
        || receipt.request_count != 1
        || receipt.retry_count != 0
        || (success
            && (receipt.returned_row_count == 0
                || receipt.returned_row_count != receipt.verified_row_count
                || receipt.raw_response_digest.is_none()
                || receipt.provenance_manifest_digest.is_none()
                || receipt.snapshot_digest.is_none()))
        || (!success
            && (receipt.verified_row_count != 0
                || receipt.provenance_manifest_digest.is_some()
                || receipt.snapshot_digest.is_some()))
        || receipt.receipt_digest != learning_evidence_receipt_digest_v1(receipt)
    {
        return Err("learning evidence request receipt rejected".into());
    }
    Ok(())
}

fn learning_receipt_after_attempt_v1(
    registration: &LearningEvidenceAcquisitionRegistrationV1,
    status: LearningEvidenceRequestStatusV1,
    http_status_class: Option<String>,
    returned_row_count: usize,
    verified_row_count: usize,
    raw_response_digest: Option<String>,
    provenance_manifest_digest: Option<String>,
    snapshot_digest: Option<String>,
) -> LearningEvidenceRequestReceiptV1 {
    let mut receipt = LearningEvidenceRequestReceiptV1 {
        receipt_version: LEARNING_EVIDENCE_RECEIPT_VERSION_V1.into(),
        registration_digest: registration.registration_digest.clone(),
        provider_contract_digest: registration.provider_contract_digest.clone(),
        request_attempted: true,
        request_count: 1,
        retry_count: 0,
        status,
        http_status_class,
        returned_row_count,
        verified_row_count,
        raw_response_digest,
        provenance_manifest_digest,
        snapshot_digest,
        receipt_digest: String::new(),
    };
    receipt.receipt_digest = learning_evidence_receipt_digest_v1(&receipt);
    receipt
}

fn learning_result_without_attempt_v1(
    status: LearningEvidenceRequestStatusV1,
) -> LearningEvidenceAcquisitionResultV1 {
    LearningEvidenceAcquisitionResultV1 {
        status,
        receipt: None,
        raw_response: None,
        provenance_manifest: None,
        snapshot: None,
        safety_counters: zero_learning_evidence_safety_counters_v1(),
    }
}

fn raw_learning_response_is_sanitized_v1(body: &[u8]) -> bool {
    let Ok(text) = std::str::from_utf8(body) else {
        return false;
    };
    let lowered = text.to_ascii_lowercase();
    !body.is_empty()
        && !body.contains(&0)
        && !lowered.contains("authorization")
        && !lowered.contains("access_key")
        && !lowered.contains("secret_key")
        && !lowered.contains("<html")
}

fn validate_learning_transport_response_v1(
    registration: &LearningEvidenceAcquisitionRegistrationV1,
    response: &LearningEvidenceTransportResponseV1,
) -> bool {
    let normalized = &response.response.normalized_dataset;
    response.http_status_class == "2xx"
        && response.raw_response.len() <= registration.maximum_response_bytes
        && raw_learning_response_is_sanitized_v1(&response.raw_response)
        && response.response.request_id
            == format!("learning-evidence-v1-{}", registration.registration_digest)
        && response.response.provider_id == registration.provider_id
        && response.response.content_type == "application/x-soma-normalized-dataset"
        && response.response.all_rows_finalized
        && response.response.reported_content_bytes == response.raw_response.len()
        && normalized.symbol == registration.symbols.first().cloned().unwrap_or_default()
        && normalized.rows.len() == registration.expected_timestamp_ms.len()
        && normalized
            .rows
            .iter()
            .map(|row| row.timestamp_ms)
            .eq(registration.expected_timestamp_ms.iter().copied())
        && normalized.rows.iter().all(|row| {
            row.symbol == normalized.symbol
                && row.timestamp_ms <= registration.information_cutoff_ms
                && !registration
                    .excluded_timestamp_ms
                    .contains(&row.timestamp_ms)
        })
        && validate_normalized_dataset(normalized).is_ok()
}

pub fn execute_learning_evidence_acquisition_v1<F>(
    registration: &LearningEvidenceAcquisitionRegistrationV1,
    provider_contract: &LearningEvidenceProviderContractV1,
    current_gap_digests: &[String],
    existing_receipt: Option<&LearningEvidenceRequestReceiptV1>,
    existing_snapshots: &[DataSnapshot],
    explicit_network_consent: bool,
    transport: F,
) -> LearningEvidenceAcquisitionResultV1
where
    F: FnOnce(
        &ReadOnlyProviderRequest,
    )
        -> Result<LearningEvidenceTransportResponseV1, LearningEvidenceTransportFailureV1>,
{
    if validate_learning_evidence_acquisition_registration_v1(registration).is_err()
        || !validate_learning_evidence_provider_contract_v1(provider_contract)
        || registration.provider_contract_digest != provider_contract.contract_digest
        || !learning_evidence_contract_supports_v1(
            provider_contract,
            registration.dataset_kind,
            registration.market_scope,
            &registration.symbols,
            &registration.cadence,
            &registration.lookback,
            registration.information_cutoff_ms,
        )
    {
        return learning_result_without_attempt_v1(
            LearningEvidenceRequestStatusV1::RegistrationInvalid,
        );
    }
    if existing_receipt.is_some() {
        return learning_result_without_attempt_v1(
            LearningEvidenceRequestStatusV1::RequestBudgetExhausted,
        );
    }
    if registration
        .gap_report_digests
        .iter()
        .any(|digest| !current_gap_digests.contains(digest))
    {
        return learning_result_without_attempt_v1(
            LearningEvidenceRequestStatusV1::GapNoLongerCurrent,
        );
    }
    if existing_snapshots.iter().any(|snapshot| {
        snapshot_integrity_valid_for_gap_v1(snapshot)
            && snapshot.dataset_kind == registration.dataset_kind
            && snapshot.market_scope == registration.market_scope
            && stable_learning_strings_v1(&snapshot.symbols) == registration.symbols
            && snapshot.requested_lookback == registration.lookback
            && snapshot
                .compatibility
                .as_ref()
                .is_some_and(|compatibility| {
                    compatibility.cadence == registration.cadence
                        && compatibility.adjustment_semantics
                            == adjustment_semantics_v1(registration.dataset_kind)
                        && compatibility.source_schema == "application/x-soma-normalized-dataset"
                        && compatibility.requested_cutoff_timestamp_ms
                            == Some(registration.information_cutoff_ms)
                        && compatibility.maximum_staleness_ms
                            == registration.information_cutoff_ms.saturating_sub(
                                registration
                                    .expected_timestamp_ms
                                    .last()
                                    .copied()
                                    .unwrap_or_default(),
                            )
                        && compatibility.all_rows_finalized
                })
            && snapshot
                .normalized_dataset
                .rows
                .iter()
                .map(|row| row.timestamp_ms)
                .eq(registration.expected_timestamp_ms.iter().copied())
    }) {
        return learning_result_without_attempt_v1(
            LearningEvidenceRequestStatusV1::EquivalentSnapshotExists,
        );
    }
    if !explicit_network_consent {
        return learning_result_without_attempt_v1(
            LearningEvidenceRequestStatusV1::MissingNetworkConsent,
        );
    }
    let request = ReadOnlyProviderRequest {
        request_id: format!("learning-evidence-v1-{}", registration.registration_digest),
        request_key: format!("learning-evidence-v1:{}", registration.registration_digest),
        provider_id: registration.provider_id.clone(),
        dataset_kind: registration.dataset_kind,
        market_scope: registration.market_scope,
        symbols: registration.symbols.clone(),
        lookback: registration.lookback.clone(),
        cadence: registration.cadence.clone(),
        max_staleness_ms: registration.information_cutoff_ms.saturating_sub(
            registration
                .expected_timestamp_ms
                .last()
                .copied()
                .unwrap_or_default(),
        ),
        reason_codes: vec![ReasonCode::AcquisitionRequestPlanned],
    };
    let mut counters = zero_learning_evidence_safety_counters_v1();
    counters.request_attempts = 1;
    counters.transport_constructions = 1;
    let transport_response = match transport(&request) {
        Ok(response) => response,
        Err(failure) => {
            let (status, http_status_class, raw_response) = match failure {
                LearningEvidenceTransportFailureV1::ProviderRejected {
                    http_status_class,
                    raw_response,
                } => (
                    LearningEvidenceRequestStatusV1::ProviderRejected,
                    http_status_class,
                    raw_response,
                ),
                LearningEvidenceTransportFailureV1::TimedOut => {
                    (LearningEvidenceRequestStatusV1::TimeoutNoRetry, None, None)
                }
                LearningEvidenceTransportFailureV1::Technical => (
                    LearningEvidenceRequestStatusV1::TechnicalFailure,
                    None,
                    None,
                ),
            };
            let raw_response = raw_response
                .filter(|raw_response| raw_learning_response_is_sanitized_v1(raw_response));
            let raw_digest = raw_response.as_deref().map(canonical_hash_hex);
            let receipt = learning_receipt_after_attempt_v1(
                registration,
                status,
                http_status_class,
                0,
                0,
                raw_digest,
                None,
                None,
            );
            return LearningEvidenceAcquisitionResultV1 {
                status,
                receipt: Some(receipt),
                raw_response,
                provenance_manifest: None,
                snapshot: None,
                safety_counters: counters,
            };
        }
    };
    let returned_row_count = transport_response.response.normalized_dataset.rows.len();
    let raw_is_sanitized = raw_learning_response_is_sanitized_v1(&transport_response.raw_response);
    let raw_digest = raw_is_sanitized.then(|| canonical_hash_hex(&transport_response.raw_response));
    if !validate_learning_transport_response_v1(registration, &transport_response) {
        let receipt = learning_receipt_after_attempt_v1(
            registration,
            LearningEvidenceRequestStatusV1::InvalidResponse,
            Some(transport_response.http_status_class.clone()),
            returned_row_count,
            0,
            raw_digest,
            None,
            None,
        );
        return LearningEvidenceAcquisitionResultV1 {
            status: LearningEvidenceRequestStatusV1::InvalidResponse,
            receipt: Some(receipt),
            raw_response: raw_is_sanitized.then_some(transport_response.raw_response),
            provenance_manifest: None,
            snapshot: None,
            safety_counters: counters,
        };
    }
    let Some(raw_digest) = raw_digest else {
        let receipt = learning_receipt_after_attempt_v1(
            registration,
            LearningEvidenceRequestStatusV1::InvalidResponse,
            Some(transport_response.http_status_class),
            returned_row_count,
            0,
            None,
            None,
            None,
        );
        return LearningEvidenceAcquisitionResultV1 {
            status: LearningEvidenceRequestStatusV1::InvalidResponse,
            receipt: Some(receipt),
            raw_response: None,
            provenance_manifest: None,
            snapshot: None,
            safety_counters: counters,
        };
    };
    let snapshot = match snapshot_from_response(
        &request,
        transport_response.response,
        AcquisitionMode::ApprovedReadOnlyNetwork,
        registration.information_cutoff_ms,
        registration.maximum_response_bytes,
    ) {
        Ok(snapshot) => snapshot,
        Err(_) => {
            let receipt = learning_receipt_after_attempt_v1(
                registration,
                LearningEvidenceRequestStatusV1::InvalidResponse,
                Some(transport_response.http_status_class.clone()),
                returned_row_count,
                0,
                Some(raw_digest),
                None,
                None,
            );
            return LearningEvidenceAcquisitionResultV1 {
                status: LearningEvidenceRequestStatusV1::InvalidResponse,
                receipt: Some(receipt),
                raw_response: Some(transport_response.raw_response),
                provenance_manifest: None,
                snapshot: None,
                safety_counters: counters,
            };
        }
    };
    if existing_snapshots
        .iter()
        .any(|existing| existing.content_digest == snapshot.content_digest)
    {
        let receipt = learning_receipt_after_attempt_v1(
            registration,
            LearningEvidenceRequestStatusV1::EquivalentSnapshotExists,
            Some(transport_response.http_status_class),
            returned_row_count,
            0,
            Some(raw_digest),
            None,
            None,
        );
        return LearningEvidenceAcquisitionResultV1 {
            status: LearningEvidenceRequestStatusV1::EquivalentSnapshotExists,
            receipt: Some(receipt),
            raw_response: Some(transport_response.raw_response),
            provenance_manifest: None,
            snapshot: None,
            safety_counters: counters,
        };
    }
    let provenance_manifest =
        match seal_learning_data_provenance_manifest_v0(LearningDataProvenanceManifestV0 {
            source_provider_id: registration.provider_id.clone(),
            source_type: "ApprovedReadOnlyProvider".into(),
            acquisition_request_identity: registration.registration_digest.clone(),
            fetch_timestamp_ms: snapshot.fetched_at_ms,
            publication_event_timestamp_ms: snapshot.actual_end_timestamp_ms,
            raw_content_digest: raw_digest.clone(),
            parser_version: "upbit-learning-evidence-parser-v1".into(),
            normalized_artifact_digest: snapshot.content_digest.clone(),
            sanitized: true,
            credential_free: true,
            information_cutoff_ms: registration.information_cutoff_ms,
            usage_classification: LearningDataUsageClassificationV0::ResearchOnlyUnconsumed,
            manifest_digest: String::new(),
        }) {
            Ok(manifest) => manifest,
            Err(_) => {
                let receipt = learning_receipt_after_attempt_v1(
                    registration,
                    LearningEvidenceRequestStatusV1::InvalidResponse,
                    Some(transport_response.http_status_class.clone()),
                    returned_row_count,
                    0,
                    Some(raw_digest),
                    None,
                    None,
                );
                return LearningEvidenceAcquisitionResultV1 {
                    status: LearningEvidenceRequestStatusV1::InvalidResponse,
                    receipt: Some(receipt),
                    raw_response: Some(transport_response.raw_response),
                    provenance_manifest: None,
                    snapshot: None,
                    safety_counters: counters,
                };
            }
        };
    let receipt = learning_receipt_after_attempt_v1(
        registration,
        LearningEvidenceRequestStatusV1::EvidenceAcquired,
        Some(transport_response.http_status_class),
        returned_row_count,
        returned_row_count,
        Some(raw_digest),
        Some(provenance_manifest.manifest_digest.clone()),
        Some(snapshot.content_digest.clone()),
    );
    LearningEvidenceAcquisitionResultV1 {
        status: LearningEvidenceRequestStatusV1::EvidenceAcquired,
        receipt: Some(receipt),
        raw_response: Some(transport_response.raw_response),
        provenance_manifest: Some(provenance_manifest),
        snapshot: Some(snapshot),
        safety_counters: counters,
    }
}

const COMPOSITE_LEARNING_REGISTRATION_VERSION_V1: &str =
    "composite-learning-acquisition-registration-v1";
const LEARNING_SEGMENT_REGISTRATION_VERSION_V1: &str = "learning-evidence-segment-registration-v1";
const LEARNING_SEGMENT_RECEIPT_VERSION_V1: &str = "learning-evidence-segment-receipt-v1";
const LEARNING_SEGMENT_CAPSULE_VERSION_V1: &str = "learning-evidence-segment-capsule-v1";
const LEARNING_EPOCH_RECEIPT_VERSION_V1: &str = "composite-learning-epoch-receipt-v1";
const LEARNING_MERGED_PROVENANCE_VERSION_V1: &str = "composite-learning-provenance-v1";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LearningEvidenceSegmentRegistrationV1 {
    pub segment_index: usize,
    pub expected_timestamps: Vec<u64>,
    pub expected_row_count: usize,
    pub request_to_utc: String,
    pub maximum_requests: usize,
    pub maximum_retries: usize,
    pub segment_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompositeLearningAcquisitionRegistrationV1 {
    pub registration_version: String,
    pub target_agent_ids: Vec<String>,
    pub intent_digest: String,
    pub gap_report_digest: String,
    pub provider_contract_digest: String,
    pub dataset_kind: DatasetKind,
    pub market_scope: AcquisitionMarketScope,
    pub symbols: Vec<String>,
    pub cadence: String,
    pub information_cutoff_ms: u64,
    pub required_row_count: usize,
    pub expected_timestamp_digest: String,
    pub segments: Vec<LearningEvidenceSegmentRegistrationV1>,
    pub maximum_total_requests: usize,
    pub maximum_concurrency: usize,
    pub maximum_retries_per_segment: usize,
    pub protected_registration_digests: Vec<String>,
    pub excluded_timestamp_ms: Vec<u64>,
    pub read_only_required: bool,
    pub credential_free_required: bool,
    pub prospective_storage_forbidden: bool,
    pub registration_digest: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompositeLearningEpochStatusV1 {
    ReadyNotAttempted,
    EvidenceAcquired,
    TerminalSegmentFailure,
    TerminalPartialEvidence,
    AlreadyTerminal,
    RegistrationInvalid,
    GapNoLongerCurrent,
    MissingNetworkConsent,
    IntegrityFailure,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LearningEvidenceSegmentReceiptV1 {
    pub receipt_version: String,
    pub composite_registration_digest: String,
    pub segment_digest: String,
    pub segment_index: usize,
    pub request_attempted: bool,
    pub request_count: usize,
    pub retry_count: usize,
    pub status: LearningEvidenceRequestStatusV1,
    pub http_status_class: Option<String>,
    pub returned_row_count: usize,
    pub verified_row_count: usize,
    pub raw_response_digest: Option<String>,
    pub segment_capsule_digest: Option<String>,
    pub receipt_digest: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LearningEvidenceSegmentCapsuleV1 {
    pub capsule_version: String,
    pub composite_registration_digest: String,
    pub segment_digest: String,
    pub segment_receipt_digest: String,
    pub segment_index: usize,
    pub provider_id: String,
    pub symbol: String,
    pub cadence: String,
    pub expected_timestamps: Vec<u64>,
    pub rows: Vec<HistoricalOhlcvRow>,
    pub segment_semantic_digest: String,
    pub finalized: bool,
    pub read_only: bool,
    pub credential_free: bool,
    pub capsule_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompositeLearningEpochReceiptV1 {
    pub receipt_version: String,
    pub registration_digest: String,
    pub segment_receipt_digests: Vec<String>,
    pub attempted_segment_count: usize,
    pub successful_segment_count: usize,
    pub request_count: usize,
    pub retry_count: usize,
    pub merged_snapshot_digest: Option<String>,
    pub merged_provenance_digest: Option<String>,
    pub status: CompositeLearningEpochStatusV1,
    pub receipt_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompositeLearningMergedProvenanceV1 {
    pub provenance_version: String,
    pub registration_digest: String,
    pub provider_contract_digest: String,
    pub segment_digests: Vec<String>,
    pub segment_receipt_digests: Vec<String>,
    pub segment_capsule_digests: Vec<String>,
    pub canonical_snapshot_digest: String,
    pub expected_timestamp_digest: String,
    pub required_row_count: usize,
    pub read_only: bool,
    pub credential_free: bool,
    pub prospective_storage_used: bool,
    pub provenance_digest: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CompositeLearningAcquisitionResultV1 {
    pub status: CompositeLearningEpochStatusV1,
    pub segment_receipts: Vec<LearningEvidenceSegmentReceiptV1>,
    pub segment_capsules: Vec<LearningEvidenceSegmentCapsuleV1>,
    pub raw_responses: Vec<Vec<u8>>,
    pub epoch_receipt: Option<CompositeLearningEpochReceiptV1>,
    pub merged_provenance: Option<CompositeLearningMergedProvenanceV1>,
    pub snapshot: Option<DataSnapshot>,
    pub safety_counters: LearningEvidenceSafetyCountersV1,
}

fn format_learning_utc_timestamp_v1(timestamp_ms: u64) -> Option<String> {
    let seconds = timestamp_ms / 1_000;
    let days = i64::try_from(seconds / 86_400).ok()?;
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
    let second_of_day = seconds % 86_400;
    Some(format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        second_of_day / 3_600,
        (second_of_day % 3_600) / 60,
        second_of_day % 60
    ))
}

fn learning_segment_registration_digest_v1(
    segment: &LearningEvidenceSegmentRegistrationV1,
) -> String {
    stable_hash_string(&format!(
        "{}:{:?}:{}:{}:{}:{}:{}",
        segment.segment_index,
        segment.expected_timestamps,
        segment.expected_row_count,
        segment.request_to_utc,
        segment.maximum_requests,
        segment.maximum_retries,
        LEARNING_SEGMENT_REGISTRATION_VERSION_V1,
    ))
}

fn composite_learning_registration_digest_v1(
    registration: &CompositeLearningAcquisitionRegistrationV1,
) -> String {
    stable_hash_string(&format!(
        "{:?}",
        (
            (
                registration.registration_version.as_str(),
                &registration.target_agent_ids,
                registration.intent_digest.as_str(),
                registration.gap_report_digest.as_str(),
                registration.provider_contract_digest.as_str(),
                registration.dataset_kind,
                registration.market_scope,
                &registration.symbols,
                registration.cadence.as_str(),
            ),
            (
                registration.information_cutoff_ms,
                registration.required_row_count,
                registration.expected_timestamp_digest.as_str(),
                registration
                    .segments
                    .iter()
                    .map(|segment| segment.segment_digest.as_str())
                    .collect::<Vec<_>>(),
                registration.maximum_total_requests,
                registration.maximum_concurrency,
                registration.maximum_retries_per_segment,
            ),
            (
                &registration.protected_registration_digests,
                &registration.excluded_timestamp_ms,
                registration.read_only_required,
                registration.credential_free_required,
                registration.prospective_storage_forbidden,
            ),
        )
    ))
}

fn derive_learning_segments_v1(
    expected_timestamps: &[u64],
    provider_maximum_rows: usize,
) -> Result<Vec<LearningEvidenceSegmentRegistrationV1>, String> {
    if expected_timestamps.is_empty()
        || provider_maximum_rows == 0
        || expected_timestamps
            .windows(2)
            .any(|pair| pair[1].checked_sub(pair[0]) != Some(DAILY_CADENCE_MS_V1))
    {
        return Err("composite learning timestamp plan rejected".into());
    }
    let segment_count = expected_timestamps.len().div_ceil(provider_maximum_rows);
    if segment_count <= 1 || segment_count > MAX_LEARNING_EVIDENCE_SEGMENTS_V1 {
        return Err("composite learning segment governance rejected".into());
    }
    let mut segments = Vec::with_capacity(segment_count);
    let mut end = expected_timestamps.len();
    for segment_index in 0..segment_count {
        let start = end.saturating_sub(provider_maximum_rows);
        let timestamps = expected_timestamps[start..end].to_vec();
        let request_to_ms = timestamps
            .last()
            .copied()
            .and_then(|timestamp| timestamp.checked_add(DAILY_CADENCE_MS_V1))
            .ok_or("composite learning segment boundary rejected")?;
        let mut segment = LearningEvidenceSegmentRegistrationV1 {
            segment_index,
            expected_row_count: timestamps.len(),
            expected_timestamps: timestamps,
            request_to_utc: format_learning_utc_timestamp_v1(request_to_ms)
                .ok_or("composite learning segment boundary rejected")?,
            maximum_requests: 1,
            maximum_retries: 0,
            segment_digest: String::new(),
        };
        segment.segment_digest = learning_segment_registration_digest_v1(&segment);
        segments.push(segment);
        end = start;
    }
    if end != 0 {
        return Err("composite learning partial timestamp plan rejected".into());
    }
    Ok(segments)
}

pub fn select_composite_learning_acquisition_registration_v1(
    report: &AgentCanonicalViewGapReportV1,
    provider_contracts: &[LearningEvidenceProviderContractV1],
    protected_registration_digests: &[String],
    excluded_timestamp_ms: &[u64],
) -> Result<Option<CompositeLearningAcquisitionRegistrationV1>, String> {
    if report.report_version != CANONICAL_VIEW_GAP_REPORT_VERSION_V1
        || report.report_digest != canonical_view_gap_report_digest_v1(report)
    {
        return Err("composite learning gap report rejected".into());
    }
    let candidates = report
        .gaps
        .iter()
        .filter(|gap| {
            gap.trainer_available
                && gap.status == CanonicalViewGapStatusV1::SegmentedAcquisitionRequired
                && gap.missing_required_dataset_kinds.len() == 1
        })
        .collect::<Vec<_>>();
    if candidates.is_empty() {
        return Ok(None);
    }
    if candidates.len() != 1 {
        return Err("composite learning ambiguous gap rejected".into());
    }
    let gap = candidates[0];
    let dataset_kind = gap.missing_required_dataset_kinds[0];
    let contracts = provider_contracts
        .iter()
        .filter(|contract| {
            validate_learning_evidence_provider_contract_v1(contract)
                && contract.dataset_kind == dataset_kind
                && gap.market_scopes.as_slice() == [contract.market_scope]
                && stable_learning_strings_v1(&gap.symbols)
                    == stable_learning_strings_v1(&contract.symbols)
                && gap.cadence == contract.cadence
                && gap.information_cutoff_ms < contract.latest_exclusive_timestamp_ms
                && gap.lookback.bars > contract.maximum_lookback_bars
        })
        .collect::<Vec<_>>();
    if contracts.len() != 1 {
        return Err("composite learning provider contract rejected".into());
    }
    let contract = contracts[0];
    let expected_timestamps = expected_learning_timestamps_v1(&gap.lookback)
        .ok_or("composite learning expected timestamps unavailable")?;
    if expected_timestamps.len() != gap.lookback.bars
        || expected_timestamps
            .iter()
            .any(|timestamp| *timestamp > gap.information_cutoff_ms)
    {
        return Err("composite learning intent shortening rejected".into());
    }
    let segments =
        derive_learning_segments_v1(&expected_timestamps, contract.maximum_lookback_bars)?;
    let mut protected_registration_digests = protected_registration_digests.to_vec();
    protected_registration_digests.sort();
    protected_registration_digests.dedup();
    let mut excluded_timestamp_ms = excluded_timestamp_ms.to_vec();
    excluded_timestamp_ms.sort();
    excluded_timestamp_ms.dedup();
    if expected_timestamps
        .iter()
        .any(|timestamp| excluded_timestamp_ms.contains(timestamp))
    {
        return Err("composite learning protected timestamp rejected".into());
    }
    let mut registration = CompositeLearningAcquisitionRegistrationV1 {
        registration_version: COMPOSITE_LEARNING_REGISTRATION_VERSION_V1.into(),
        target_agent_ids: vec![gap.agent_id.clone()],
        intent_digest: gap.intent_digest.clone(),
        gap_report_digest: gap.gap_digest.clone(),
        provider_contract_digest: contract.contract_digest.clone(),
        dataset_kind,
        market_scope: contract.market_scope,
        symbols: stable_learning_strings_v1(&gap.symbols),
        cadence: gap.cadence.clone(),
        information_cutoff_ms: gap.information_cutoff_ms,
        required_row_count: expected_timestamps.len(),
        expected_timestamp_digest: stable_hash_string(&format!(
            "composite-learning-expected-timestamps-v1:{expected_timestamps:?}"
        )),
        maximum_total_requests: segments.len(),
        segments,
        maximum_concurrency: 1,
        maximum_retries_per_segment: 0,
        protected_registration_digests,
        excluded_timestamp_ms,
        read_only_required: true,
        credential_free_required: true,
        prospective_storage_forbidden: true,
        registration_digest: String::new(),
    };
    registration.registration_digest = composite_learning_registration_digest_v1(&registration);
    validate_composite_learning_acquisition_registration_v1(&registration, contract)?;
    Ok(Some(registration))
}

pub fn validate_composite_learning_acquisition_registration_v1(
    registration: &CompositeLearningAcquisitionRegistrationV1,
    contract: &LearningEvidenceProviderContractV1,
) -> Result<(), String> {
    if !validate_learning_evidence_provider_contract_v1(contract)
        || registration.registration_version != COMPOSITE_LEARNING_REGISTRATION_VERSION_V1
        || registration.target_agent_ids.len() != 1
        || registration.target_agent_ids
            != stable_learning_strings_v1(&registration.target_agent_ids)
        || registration.intent_digest.is_empty()
        || registration.gap_report_digest.is_empty()
        || registration.provider_contract_digest != contract.contract_digest
        || registration.dataset_kind != contract.dataset_kind
        || registration.market_scope != contract.market_scope
        || registration.symbols != stable_learning_strings_v1(&contract.symbols)
        || registration.cadence != "1d"
        || registration.cadence != contract.cadence
        || registration.required_row_count <= contract.maximum_lookback_bars
        || registration.segments.len() <= 1
        || registration.segments.len() > MAX_LEARNING_EVIDENCE_SEGMENTS_V1
        || registration.maximum_total_requests != registration.segments.len()
        || registration.maximum_concurrency != 1
        || registration.maximum_retries_per_segment != 0
        || registration.protected_registration_digests.is_empty()
        || registration.protected_registration_digests
            != stable_learning_strings_v1(&registration.protected_registration_digests)
        || registration.excluded_timestamp_ms.is_empty()
        || {
            let mut values = registration.excluded_timestamp_ms.clone();
            values.sort();
            values.dedup();
            values != registration.excluded_timestamp_ms
        }
        || !registration.read_only_required
        || !registration.credential_free_required
        || !registration.prospective_storage_forbidden
    {
        return Err("composite learning registration rejected".into());
    }
    let mut expected = registration
        .segments
        .iter()
        .flat_map(|segment| segment.expected_timestamps.iter().copied())
        .collect::<Vec<_>>();
    let unique = expected.iter().copied().collect::<BTreeSet<_>>();
    expected.sort();
    if registration
        .segments
        .iter()
        .enumerate()
        .any(|(index, segment)| {
            segment.segment_index != index
                || segment.expected_timestamps.is_empty()
                || segment.expected_row_count != segment.expected_timestamps.len()
                || segment.expected_row_count > contract.maximum_lookback_bars
                || segment.maximum_requests != 1
                || segment.maximum_retries != 0
                || segment
                    .expected_timestamps
                    .windows(2)
                    .any(|pair| pair[1].checked_sub(pair[0]) != Some(DAILY_CADENCE_MS_V1))
                || segment
                    .expected_timestamps
                    .last()
                    .copied()
                    .and_then(|timestamp| timestamp.checked_add(DAILY_CADENCE_MS_V1))
                    .and_then(format_learning_utc_timestamp_v1)
                    .as_deref()
                    != Some(segment.request_to_utc.as_str())
                || segment.segment_digest != learning_segment_registration_digest_v1(segment)
        })
        || expected.len() != registration.required_row_count
        || unique.len() != registration.required_row_count
        || expected
            .windows(2)
            .any(|pair| pair[1].checked_sub(pair[0]) != Some(DAILY_CADENCE_MS_V1))
        || expected.last().copied().unwrap_or_default() > registration.information_cutoff_ms
        || expected
            .iter()
            .any(|timestamp| registration.excluded_timestamp_ms.contains(timestamp))
        || registration.expected_timestamp_digest
            != stable_hash_string(&format!(
                "composite-learning-expected-timestamps-v1:{expected:?}"
            ))
        || registration.registration_digest
            != composite_learning_registration_digest_v1(registration)
    {
        return Err("composite learning segment partition rejected".into());
    }
    Ok(())
}

pub fn composite_learning_expected_timestamps_v1(
    registration: &CompositeLearningAcquisitionRegistrationV1,
) -> Vec<u64> {
    let mut timestamps = registration
        .segments
        .iter()
        .flat_map(|segment| segment.expected_timestamps.iter().copied())
        .collect::<Vec<_>>();
    timestamps.sort();
    timestamps
}

fn segment_internal_registration_v1(
    composite: &CompositeLearningAcquisitionRegistrationV1,
    contract: &LearningEvidenceProviderContractV1,
    segment: &LearningEvidenceSegmentRegistrationV1,
) -> LearningEvidenceAcquisitionRegistrationV1 {
    let start = segment.expected_timestamps.first().copied();
    let end = segment
        .expected_timestamps
        .last()
        .copied()
        .and_then(|timestamp| timestamp.checked_add(DAILY_CADENCE_MS_V1));
    let mut registration = LearningEvidenceAcquisitionRegistrationV1 {
        registration_version: LEARNING_EVIDENCE_REGISTRATION_VERSION_V1.into(),
        target_agent_ids: composite.target_agent_ids.clone(),
        gap_report_digests: vec![composite.gap_report_digest.clone()],
        provider_id: contract.provider_id.clone(),
        provider_contract_digest: contract.contract_digest.clone(),
        dataset_kind: composite.dataset_kind,
        market_scope: composite.market_scope,
        symbols: composite.symbols.clone(),
        cadence: composite.cadence.clone(),
        lookback: DataLookback {
            bars: segment.expected_row_count,
            start_timestamp_ms: start,
            end_timestamp_ms: end,
        },
        information_cutoff_ms: end.unwrap_or_default(),
        expected_timestamp_ms: segment.expected_timestamps.clone(),
        protected_registration_digests: composite.protected_registration_digests.clone(),
        excluded_timestamp_ms: composite.excluded_timestamp_ms.clone(),
        maximum_requests: 1,
        maximum_concurrency: 1,
        maximum_retries: 0,
        maximum_response_bytes: contract.maximum_response_bytes,
        credential_free_required: true,
        read_only_required: true,
        prospective_storage_forbidden: true,
        registration_digest: String::new(),
    };
    registration.registration_digest = learning_evidence_registration_digest_v1(&registration);
    registration
}

fn segment_receipt_digest_v1(receipt: &LearningEvidenceSegmentReceiptV1) -> String {
    stable_hash_string(&format!(
        "{:?}",
        (
            receipt.receipt_version.as_str(),
            receipt.composite_registration_digest.as_str(),
            receipt.segment_digest.as_str(),
            receipt.segment_index,
            receipt.request_attempted,
            receipt.request_count,
            receipt.retry_count,
            receipt.status,
            &receipt.http_status_class,
            receipt.returned_row_count,
            receipt.verified_row_count,
            &receipt.raw_response_digest,
        )
    ))
}

fn segment_capsule_digest_v1(capsule: &LearningEvidenceSegmentCapsuleV1) -> String {
    stable_hash_string(&format!(
        "{:?}",
        (
            (
                capsule.capsule_version.as_str(),
                capsule.composite_registration_digest.as_str(),
                capsule.segment_digest.as_str(),
                capsule.segment_receipt_digest.as_str(),
                capsule.segment_index,
                capsule.provider_id.as_str(),
                capsule.symbol.as_str(),
                capsule.cadence.as_str(),
            ),
            (
                &capsule.expected_timestamps,
                &capsule.rows,
                capsule.segment_semantic_digest.as_str(),
                capsule.finalized,
                capsule.read_only,
                capsule.credential_free,
            ),
        )
    ))
}

fn epoch_receipt_digest_v1(receipt: &CompositeLearningEpochReceiptV1) -> String {
    stable_hash_string(&format!(
        "{:?}",
        (
            receipt.receipt_version.as_str(),
            receipt.registration_digest.as_str(),
            &receipt.segment_receipt_digests,
            receipt.attempted_segment_count,
            receipt.successful_segment_count,
            receipt.request_count,
            receipt.retry_count,
            &receipt.merged_snapshot_digest,
            &receipt.merged_provenance_digest,
            receipt.status,
        )
    ))
}

fn merged_provenance_digest_v1(provenance: &CompositeLearningMergedProvenanceV1) -> String {
    stable_hash_string(&format!(
        "{:?}",
        (
            (
                provenance.provenance_version.as_str(),
                provenance.registration_digest.as_str(),
                provenance.provider_contract_digest.as_str(),
                &provenance.segment_digests,
                &provenance.segment_receipt_digests,
            ),
            (
                &provenance.segment_capsule_digests,
                provenance.canonical_snapshot_digest.as_str(),
                provenance.expected_timestamp_digest.as_str(),
                provenance.required_row_count,
                provenance.read_only,
                provenance.credential_free,
                provenance.prospective_storage_used,
            ),
        )
    ))
}

pub fn validate_learning_evidence_segment_receipt_v1(
    receipt: &LearningEvidenceSegmentReceiptV1,
) -> Result<(), String> {
    let success = receipt.status == LearningEvidenceRequestStatusV1::EvidenceAcquired;
    if receipt.receipt_version != LEARNING_SEGMENT_RECEIPT_VERSION_V1
        || receipt.composite_registration_digest.is_empty()
        || receipt.segment_digest.is_empty()
        || !receipt.request_attempted
        || receipt.request_count != 1
        || receipt.retry_count != 0
        || (success
            && (receipt.returned_row_count == 0
                || receipt.returned_row_count != receipt.verified_row_count
                || receipt.raw_response_digest.is_none()
                || receipt.segment_capsule_digest.is_none()))
        || (!success
            && (receipt.verified_row_count != 0 || receipt.segment_capsule_digest.is_some()))
        || receipt.receipt_digest != segment_receipt_digest_v1(receipt)
    {
        Err("composite learning segment receipt rejected".into())
    } else {
        Ok(())
    }
}

pub fn validate_learning_evidence_segment_capsule_v1(
    capsule: &LearningEvidenceSegmentCapsuleV1,
    registration: &CompositeLearningAcquisitionRegistrationV1,
    contract: &LearningEvidenceProviderContractV1,
) -> Result<(), String> {
    validate_composite_learning_acquisition_registration_v1(registration, contract)?;
    let segment = registration
        .segments
        .get(capsule.segment_index)
        .ok_or("composite learning segment missing")?;
    let dataset = HistoricalReplayDataset {
        symbol: capsule.symbol.clone(),
        rows: capsule.rows.clone(),
        source: capsule.provider_id.clone(),
        reason_codes: vec![],
    };
    if capsule.capsule_version != LEARNING_SEGMENT_CAPSULE_VERSION_V1
        || capsule.composite_registration_digest != registration.registration_digest
        || capsule.segment_digest != segment.segment_digest
        || capsule.segment_receipt_digest.is_empty()
        || capsule.provider_id != contract.provider_id
        || registration.symbols.as_slice() != [capsule.symbol.clone()]
        || capsule.cadence != registration.cadence
        || capsule.expected_timestamps != segment.expected_timestamps
        || capsule.rows.len() != segment.expected_row_count
        || capsule
            .rows
            .iter()
            .map(|row| row.timestamp_ms)
            .ne(segment.expected_timestamps.iter().copied())
        || capsule.rows.iter().any(|row| {
            row.symbol != capsule.symbol
                || registration
                    .excluded_timestamp_ms
                    .contains(&row.timestamp_ms)
        })
        || validate_normalized_dataset(&dataset).is_err()
        || capsule.segment_semantic_digest != historical_replay_dataset_digest_v0(&dataset)
        || !capsule.finalized
        || !capsule.read_only
        || !capsule.credential_free
        || capsule.capsule_digest != segment_capsule_digest_v1(capsule)
    {
        Err("composite learning segment capsule rejected".into())
    } else {
        Ok(())
    }
}

pub fn validate_composite_learning_epoch_receipt_v1(
    receipt: &CompositeLearningEpochReceiptV1,
    registration: &CompositeLearningAcquisitionRegistrationV1,
) -> Result<(), String> {
    let success = receipt.status == CompositeLearningEpochStatusV1::EvidenceAcquired;
    if receipt.receipt_version != LEARNING_EPOCH_RECEIPT_VERSION_V1
        || receipt.registration_digest != registration.registration_digest
        || receipt.attempted_segment_count != receipt.segment_receipt_digests.len()
        || receipt.request_count != receipt.attempted_segment_count
        || receipt.request_count > registration.maximum_total_requests
        || receipt.retry_count != 0
        || receipt.successful_segment_count > receipt.attempted_segment_count
        || (success
            && (receipt.successful_segment_count != registration.segments.len()
                || receipt.merged_snapshot_digest.is_none()
                || receipt.merged_provenance_digest.is_none()))
        || (!success
            && (receipt.merged_snapshot_digest.is_some()
                || receipt.merged_provenance_digest.is_some()))
        || receipt.receipt_digest != epoch_receipt_digest_v1(receipt)
    {
        Err("composite learning epoch receipt rejected".into())
    } else {
        Ok(())
    }
}

fn terminal_epoch_receipt_v1(
    registration: &CompositeLearningAcquisitionRegistrationV1,
    segment_receipts: &[LearningEvidenceSegmentReceiptV1],
    successful_segment_count: usize,
    status: CompositeLearningEpochStatusV1,
) -> CompositeLearningEpochReceiptV1 {
    let mut receipt = CompositeLearningEpochReceiptV1 {
        receipt_version: LEARNING_EPOCH_RECEIPT_VERSION_V1.into(),
        registration_digest: registration.registration_digest.clone(),
        segment_receipt_digests: segment_receipts
            .iter()
            .map(|receipt| receipt.receipt_digest.clone())
            .collect(),
        attempted_segment_count: segment_receipts.len(),
        successful_segment_count,
        request_count: segment_receipts.len(),
        retry_count: 0,
        merged_snapshot_digest: None,
        merged_provenance_digest: None,
        status,
        receipt_digest: String::new(),
    };
    receipt.receipt_digest = epoch_receipt_digest_v1(&receipt);
    receipt
}

fn empty_composite_result_v1(
    status: CompositeLearningEpochStatusV1,
) -> CompositeLearningAcquisitionResultV1 {
    CompositeLearningAcquisitionResultV1 {
        status,
        segment_receipts: vec![],
        segment_capsules: vec![],
        raw_responses: vec![],
        epoch_receipt: None,
        merged_provenance: None,
        snapshot: None,
        safety_counters: zero_learning_evidence_safety_counters_v1(),
    }
}

pub fn execute_composite_learning_acquisition_v1<F>(
    registration: &CompositeLearningAcquisitionRegistrationV1,
    contract: &LearningEvidenceProviderContractV1,
    current_gap_digests: &[String],
    existing_epoch_receipt: Option<&CompositeLearningEpochReceiptV1>,
    explicit_network_consent: bool,
    mut transport: F,
) -> CompositeLearningAcquisitionResultV1
where
    F: FnMut(
        &LearningEvidenceSegmentRegistrationV1,
        &ReadOnlyProviderRequest,
    )
        -> Result<LearningEvidenceTransportResponseV1, LearningEvidenceTransportFailureV1>,
{
    if validate_composite_learning_acquisition_registration_v1(registration, contract).is_err() {
        return empty_composite_result_v1(CompositeLearningEpochStatusV1::RegistrationInvalid);
    }
    if let Some(existing) = existing_epoch_receipt {
        return if validate_composite_learning_epoch_receipt_v1(existing, registration).is_ok() {
            empty_composite_result_v1(CompositeLearningEpochStatusV1::AlreadyTerminal)
        } else {
            empty_composite_result_v1(CompositeLearningEpochStatusV1::IntegrityFailure)
        };
    }
    if !current_gap_digests.contains(&registration.gap_report_digest) {
        return empty_composite_result_v1(CompositeLearningEpochStatusV1::GapNoLongerCurrent);
    }
    if !explicit_network_consent {
        return empty_composite_result_v1(CompositeLearningEpochStatusV1::MissingNetworkConsent);
    }
    let mut result = empty_composite_result_v1(CompositeLearningEpochStatusV1::ReadyNotAttempted);
    let mut merged_fetched_at_ms = 0;
    let mut merged_normalized_at_ms = 0;
    for segment in &registration.segments {
        let internal = segment_internal_registration_v1(registration, contract, segment);
        if validate_learning_evidence_acquisition_registration_v1(&internal).is_err() {
            result.status = CompositeLearningEpochStatusV1::RegistrationInvalid;
            return result;
        }
        let single = execute_learning_evidence_acquisition_v1(
            &internal,
            contract,
            current_gap_digests,
            None,
            &[],
            true,
            |request| transport(segment, request),
        );
        result.safety_counters.request_attempts += single.safety_counters.request_attempts;
        result.safety_counters.transport_constructions +=
            single.safety_counters.transport_constructions;
        result.safety_counters.retry_count += single.safety_counters.retry_count;
        let Some(single_receipt) = single.receipt else {
            result.status = CompositeLearningEpochStatusV1::IntegrityFailure;
            return result;
        };
        let raw_response_digest = single_receipt.raw_response_digest.clone();
        if let Some(raw) = single.raw_response {
            result.raw_responses.push(raw);
        }
        let mut segment_receipt = LearningEvidenceSegmentReceiptV1 {
            receipt_version: LEARNING_SEGMENT_RECEIPT_VERSION_V1.into(),
            composite_registration_digest: registration.registration_digest.clone(),
            segment_digest: segment.segment_digest.clone(),
            segment_index: segment.segment_index,
            request_attempted: true,
            request_count: 1,
            retry_count: 0,
            status: single_receipt.status,
            http_status_class: single_receipt.http_status_class,
            returned_row_count: single_receipt.returned_row_count,
            verified_row_count: single_receipt.verified_row_count,
            raw_response_digest,
            segment_capsule_digest: None,
            receipt_digest: String::new(),
        };
        if single.status == LearningEvidenceRequestStatusV1::EvidenceAcquired {
            let Some(snapshot) = single.snapshot else {
                result.status = CompositeLearningEpochStatusV1::IntegrityFailure;
                return result;
            };
            merged_fetched_at_ms = merged_fetched_at_ms.max(snapshot.fetched_at_ms);
            merged_normalized_at_ms = merged_normalized_at_ms.max(snapshot.normalized_at_ms);
            let dataset = HistoricalReplayDataset {
                symbol: snapshot.normalized_dataset.symbol.clone(),
                rows: snapshot.normalized_dataset.rows.clone(),
                source: contract.provider_id.clone(),
                reason_codes: vec![],
            };
            let mut capsule = LearningEvidenceSegmentCapsuleV1 {
                capsule_version: LEARNING_SEGMENT_CAPSULE_VERSION_V1.into(),
                composite_registration_digest: registration.registration_digest.clone(),
                segment_digest: segment.segment_digest.clone(),
                segment_receipt_digest: String::new(),
                segment_index: segment.segment_index,
                provider_id: contract.provider_id.clone(),
                symbol: snapshot.normalized_dataset.symbol,
                cadence: registration.cadence.clone(),
                expected_timestamps: segment.expected_timestamps.clone(),
                rows: snapshot.normalized_dataset.rows,
                segment_semantic_digest: historical_replay_dataset_digest_v0(&dataset),
                finalized: true,
                read_only: true,
                credential_free: true,
                capsule_digest: String::new(),
            };
            segment_receipt.receipt_digest = segment_receipt_digest_v1(&segment_receipt);
            capsule.segment_receipt_digest = segment_receipt.receipt_digest.clone();
            capsule.capsule_digest = segment_capsule_digest_v1(&capsule);
            segment_receipt.segment_capsule_digest = Some(capsule.capsule_digest.clone());
            result.segment_capsules.push(capsule);
        } else {
            segment_receipt.receipt_digest = segment_receipt_digest_v1(&segment_receipt);
        }
        if validate_learning_evidence_segment_receipt_v1(&segment_receipt).is_err() {
            result.status = CompositeLearningEpochStatusV1::IntegrityFailure;
            return result;
        }
        let succeeded = segment_receipt.status == LearningEvidenceRequestStatusV1::EvidenceAcquired;
        result.segment_receipts.push(segment_receipt);
        if !succeeded {
            let successful = result.segment_capsules.len();
            let status = if segment.segment_index == 0 {
                CompositeLearningEpochStatusV1::TerminalSegmentFailure
            } else {
                CompositeLearningEpochStatusV1::TerminalPartialEvidence
            };
            let epoch = terminal_epoch_receipt_v1(
                registration,
                &result.segment_receipts,
                successful,
                status,
            );
            result.status = status;
            result.epoch_receipt = Some(epoch);
            return result;
        }
    }
    let mut rows = result
        .segment_capsules
        .iter()
        .flat_map(|capsule| capsule.rows.iter().cloned())
        .collect::<Vec<_>>();
    rows.sort_by_key(|row| row.timestamp_ms);
    let expected = composite_learning_expected_timestamps_v1(registration);
    if rows.len() != registration.required_row_count
        || rows
            .iter()
            .map(|row| row.timestamp_ms)
            .ne(expected.iter().copied())
        || rows.iter().any(|row| {
            registration
                .excluded_timestamp_ms
                .contains(&row.timestamp_ms)
                || registration.symbols.as_slice() != [row.symbol.clone()]
        })
    {
        let epoch = terminal_epoch_receipt_v1(
            registration,
            &result.segment_receipts,
            result.segment_capsules.len(),
            CompositeLearningEpochStatusV1::IntegrityFailure,
        );
        result.status = CompositeLearningEpochStatusV1::IntegrityFailure;
        result.epoch_receipt = Some(epoch);
        return result;
    }
    let dataset = HistoricalReplayDataset {
        symbol: registration.symbols[0].clone(),
        rows,
        source: contract.provider_id.clone(),
        reason_codes: vec![],
    };
    if validate_normalized_dataset(&dataset).is_err() {
        result.status = CompositeLearningEpochStatusV1::IntegrityFailure;
        return result;
    }
    let content_digest = historical_replay_dataset_digest_v0(&dataset);
    let snapshot = DataSnapshot {
        snapshot_id: snapshot_id_from_semantic_digest_v1(&content_digest),
        request_key: format!(
            "composite-learning-evidence-v1:{}",
            registration.registration_digest
        ),
        provider_id: contract.provider_id.clone(),
        dataset_kind: registration.dataset_kind,
        market_scope: registration.market_scope,
        symbols: registration.symbols.clone(),
        requested_lookback: DataLookback {
            bars: registration.required_row_count,
            start_timestamp_ms: expected.first().copied(),
            end_timestamp_ms: Some(registration.information_cutoff_ms),
        },
        actual_start_timestamp_ms: expected.first().copied(),
        actual_end_timestamp_ms: expected.last().copied(),
        fetched_at_ms: merged_fetched_at_ms,
        normalized_at_ms: merged_normalized_at_ms,
        schema_version: 1,
        row_count: dataset.rows.len(),
        quality_summary: SnapshotQualitySummary {
            accepted: true,
            row_count: dataset.rows.len(),
            reason_codes: vec![ReasonCode::CsvLoaded],
        },
        content_digest: content_digest.clone(),
        sanitized: true,
        read_only: true,
        compatibility: Some(SnapshotCompatibilityV1 {
            cadence: registration.cadence.clone(),
            adjustment_semantics: adjustment_semantics_v1(registration.dataset_kind),
            source_schema: "application/x-soma-normalized-dataset".into(),
            requested_cutoff_timestamp_ms: Some(registration.information_cutoff_ms),
            maximum_staleness_ms: registration
                .information_cutoff_ms
                .saturating_sub(expected.last().copied().unwrap_or_default()),
            all_rows_finalized: true,
        }),
        normalized_dataset: dataset,
        provenance: SnapshotProvenance {
            provider_id: contract.provider_id.clone(),
            acquisition_request_id: registration.registration_digest.clone(),
            fetch_receipt_id: stable_hash_string(&format!(
                "composite-learning-receipt-v1:{}",
                registration.registration_digest
            )),
            source_type: SnapshotSourceType::ApprovedReadOnlyProvider,
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
    };
    let mut provenance = CompositeLearningMergedProvenanceV1 {
        provenance_version: LEARNING_MERGED_PROVENANCE_VERSION_V1.into(),
        registration_digest: registration.registration_digest.clone(),
        provider_contract_digest: contract.contract_digest.clone(),
        segment_digests: registration
            .segments
            .iter()
            .map(|segment| segment.segment_digest.clone())
            .collect(),
        segment_receipt_digests: result
            .segment_receipts
            .iter()
            .map(|receipt| receipt.receipt_digest.clone())
            .collect(),
        segment_capsule_digests: result
            .segment_capsules
            .iter()
            .map(|capsule| capsule.capsule_digest.clone())
            .collect(),
        canonical_snapshot_digest: content_digest.clone(),
        expected_timestamp_digest: registration.expected_timestamp_digest.clone(),
        required_row_count: registration.required_row_count,
        read_only: true,
        credential_free: true,
        prospective_storage_used: false,
        provenance_digest: String::new(),
    };
    provenance.provenance_digest = merged_provenance_digest_v1(&provenance);
    let mut epoch = terminal_epoch_receipt_v1(
        registration,
        &result.segment_receipts,
        result.segment_capsules.len(),
        CompositeLearningEpochStatusV1::EvidenceAcquired,
    );
    epoch.merged_snapshot_digest = Some(content_digest);
    epoch.merged_provenance_digest = Some(provenance.provenance_digest.clone());
    epoch.receipt_digest = epoch_receipt_digest_v1(&epoch);
    result.status = CompositeLearningEpochStatusV1::EvidenceAcquired;
    result.epoch_receipt = Some(epoch);
    result.merged_provenance = Some(provenance);
    result.snapshot = Some(snapshot);
    result
}

const LEARNING_GAP_ARTIFACT_KIND_V1: &str = "agent-canonical-view-gap-report-v1";
const LEARNING_REGISTRATION_ARTIFACT_KIND_V1: &str =
    "learning-evidence-acquisition-registration-v1";
const LEARNING_PROVENANCE_ARTIFACT_KIND_V1: &str = "learning-evidence-provenance-manifest-v1";
const LEARNING_RECEIPT_ARTIFACT_KIND_V1: &str = "learning-evidence-request-receipt-v1";

#[derive(Clone, PartialEq, Message)]
struct LearningLookbackProtobufV1 {
    #[prost(uint64, tag = "1")]
    bars: u64,
    #[prost(uint64, optional, tag = "2")]
    start_timestamp_ms: Option<u64>,
    #[prost(uint64, optional, tag = "3")]
    end_timestamp_ms: Option<u64>,
}

#[derive(Clone, PartialEq, Message)]
struct CanonicalViewGapProtobufV1 {
    #[prost(string, tag = "1")]
    agent_id: String,
    #[prost(string, tag = "2")]
    intent_digest: String,
    #[prost(uint32, repeated, tag = "3")]
    market_scopes: Vec<u32>,
    #[prost(string, repeated, tag = "4")]
    symbols: Vec<String>,
    #[prost(string, tag = "5")]
    cadence: String,
    #[prost(message, optional, tag = "6")]
    lookback: Option<LearningLookbackProtobufV1>,
    #[prost(uint64, tag = "7")]
    information_cutoff_ms: u64,
    #[prost(uint64, tag = "8")]
    maximum_staleness_ms: u64,
    #[prost(uint32, repeated, tag = "9")]
    required_dataset_kinds: Vec<u32>,
    #[prost(uint32, repeated, tag = "10")]
    resolved_required_dataset_kinds: Vec<u32>,
    #[prost(uint32, repeated, tag = "11")]
    missing_required_dataset_kinds: Vec<u32>,
    #[prost(uint32, repeated, tag = "12")]
    optional_dataset_kinds: Vec<u32>,
    #[prost(uint32, repeated, tag = "13")]
    resolved_optional_dataset_kinds: Vec<u32>,
    #[prost(uint32, repeated, tag = "14")]
    missing_optional_dataset_kinds: Vec<u32>,
    #[prost(string, repeated, tag = "15")]
    usable_artifact_digests: Vec<String>,
    #[prost(string, repeated, tag = "16")]
    rejected_artifact_digests: Vec<String>,
    #[prost(string, repeated, tag = "17")]
    authorized_provider_ids: Vec<String>,
    #[prost(bool, tag = "18")]
    trainer_available: bool,
    #[prost(uint32, tag = "19")]
    status: u32,
    #[prost(string, tag = "20")]
    gap_digest: String,
}

#[derive(Clone, PartialEq, Message)]
struct LearningEvidenceSafetyCountersProtobufV1 {
    #[prost(uint64, tag = "1")]
    active_committee_count: u64,
    #[prost(uint64, tag = "2")]
    request_attempts: u64,
    #[prost(uint64, tag = "3")]
    retry_count: u64,
    #[prost(uint64, tag = "4")]
    transport_constructions: u64,
    #[prost(uint64, tag = "5")]
    credential_reads: u64,
    #[prost(uint64, tag = "6")]
    prospective_artifact_reads: u64,
    #[prost(uint64, tag = "7")]
    prospective_label_reads: u64,
    #[prost(uint64, tag = "8")]
    future_evaluation_reads: u64,
    #[prost(uint64, tag = "9")]
    active_model_changes: u64,
    #[prost(uint64, tag = "10")]
    chair_decisions: u64,
    #[prost(uint64, tag = "11")]
    votes: u64,
    #[prost(uint64, tag = "12")]
    rewards: u64,
    #[prost(uint64, tag = "13")]
    penalties: u64,
    #[prost(uint64, tag = "14")]
    voice_changes: u64,
    #[prost(uint64, tag = "15")]
    promotions: u64,
    #[prost(uint64, tag = "16")]
    executions: u64,
}

#[derive(Clone, PartialEq, Message)]
struct CanonicalViewGapReportProtobufV1 {
    #[prost(string, tag = "1")]
    report_version: String,
    #[prost(message, repeated, tag = "2")]
    gaps: Vec<CanonicalViewGapProtobufV1>,
    #[prost(string, repeated, tag = "3")]
    provider_contract_digests: Vec<String>,
    #[prost(message, optional, tag = "4")]
    safety_counters: Option<LearningEvidenceSafetyCountersProtobufV1>,
    #[prost(string, tag = "5")]
    report_digest: String,
}

#[derive(Clone, PartialEq, Message)]
struct LearningEvidenceRegistrationProtobufV1 {
    #[prost(string, tag = "1")]
    registration_version: String,
    #[prost(string, repeated, tag = "2")]
    target_agent_ids: Vec<String>,
    #[prost(string, repeated, tag = "3")]
    gap_report_digests: Vec<String>,
    #[prost(string, tag = "4")]
    provider_id: String,
    #[prost(string, tag = "5")]
    provider_contract_digest: String,
    #[prost(uint32, tag = "6")]
    dataset_kind: u32,
    #[prost(uint32, tag = "7")]
    market_scope: u32,
    #[prost(string, repeated, tag = "8")]
    symbols: Vec<String>,
    #[prost(string, tag = "9")]
    cadence: String,
    #[prost(message, optional, tag = "10")]
    lookback: Option<LearningLookbackProtobufV1>,
    #[prost(uint64, tag = "11")]
    information_cutoff_ms: u64,
    #[prost(uint64, repeated, tag = "12")]
    expected_timestamp_ms: Vec<u64>,
    #[prost(string, repeated, tag = "13")]
    protected_registration_digests: Vec<String>,
    #[prost(uint64, repeated, tag = "14")]
    excluded_timestamp_ms: Vec<u64>,
    #[prost(uint64, tag = "15")]
    maximum_requests: u64,
    #[prost(uint64, tag = "16")]
    maximum_concurrency: u64,
    #[prost(uint64, tag = "17")]
    maximum_retries: u64,
    #[prost(uint64, tag = "18")]
    maximum_response_bytes: u64,
    #[prost(bool, tag = "19")]
    credential_free_required: bool,
    #[prost(bool, tag = "20")]
    read_only_required: bool,
    #[prost(bool, tag = "21")]
    prospective_storage_forbidden: bool,
    #[prost(string, tag = "22")]
    registration_digest: String,
}

#[derive(Clone, PartialEq, Message)]
struct LearningEvidenceProvenanceProtobufV1 {
    #[prost(string, tag = "1")]
    source_provider_id: String,
    #[prost(string, tag = "2")]
    source_type: String,
    #[prost(string, tag = "3")]
    acquisition_request_identity: String,
    #[prost(uint64, tag = "4")]
    fetch_timestamp_ms: u64,
    #[prost(uint64, optional, tag = "5")]
    publication_event_timestamp_ms: Option<u64>,
    #[prost(string, tag = "6")]
    raw_content_digest: String,
    #[prost(string, tag = "7")]
    parser_version: String,
    #[prost(string, tag = "8")]
    normalized_artifact_digest: String,
    #[prost(bool, tag = "9")]
    sanitized: bool,
    #[prost(bool, tag = "10")]
    credential_free: bool,
    #[prost(uint64, tag = "11")]
    information_cutoff_ms: u64,
    #[prost(uint32, tag = "12")]
    usage_classification: u32,
    #[prost(string, tag = "13")]
    manifest_digest: String,
}

#[derive(Clone, PartialEq, Message)]
struct LearningEvidenceReceiptProtobufV1 {
    #[prost(string, tag = "1")]
    receipt_version: String,
    #[prost(string, tag = "2")]
    registration_digest: String,
    #[prost(string, tag = "3")]
    provider_contract_digest: String,
    #[prost(bool, tag = "4")]
    request_attempted: bool,
    #[prost(uint64, tag = "5")]
    request_count: u64,
    #[prost(uint64, tag = "6")]
    retry_count: u64,
    #[prost(uint32, tag = "7")]
    status: u32,
    #[prost(string, optional, tag = "8")]
    http_status_class: Option<String>,
    #[prost(uint64, tag = "9")]
    returned_row_count: u64,
    #[prost(uint64, tag = "10")]
    verified_row_count: u64,
    #[prost(string, optional, tag = "11")]
    raw_response_digest: Option<String>,
    #[prost(string, optional, tag = "12")]
    provenance_manifest_digest: Option<String>,
    #[prost(string, optional, tag = "13")]
    snapshot_digest: Option<String>,
    #[prost(string, tag = "14")]
    receipt_digest: String,
}

fn learning_lookback_to_protobuf_v1(
    lookback: &DataLookback,
) -> Result<LearningLookbackProtobufV1, String> {
    Ok(LearningLookbackProtobufV1 {
        bars: u64::try_from(lookback.bars).map_err(|_| "learning lookback overflow".to_string())?,
        start_timestamp_ms: lookback.start_timestamp_ms,
        end_timestamp_ms: lookback.end_timestamp_ms,
    })
}

fn learning_lookback_from_protobuf_v1(
    lookback: Option<LearningLookbackProtobufV1>,
) -> Result<DataLookback, String> {
    let lookback = lookback.ok_or_else(|| "learning lookback missing".to_string())?;
    Ok(DataLookback {
        bars: usize::try_from(lookback.bars)
            .map_err(|_| "learning lookback rejected".to_string())?,
        start_timestamp_ms: lookback.start_timestamp_ms,
        end_timestamp_ms: lookback.end_timestamp_ms,
    })
}

fn market_scope_from_code_v1(code: u32) -> Result<AcquisitionMarketScope, String> {
    match code {
        1 => Ok(AcquisitionMarketScope::UsStocks),
        2 => Ok(AcquisitionMarketScope::KoreanStocks),
        3 => Ok(AcquisitionMarketScope::BtcCrypto),
        _ => Err("learning market scope rejected".into()),
    }
}

fn dataset_kind_from_code_v1(code: u32) -> Result<DatasetKind, String> {
    u16::try_from(code)
        .ok()
        .and_then(dataset_kind_from_code_v0)
        .ok_or_else(|| "learning dataset kind rejected".into())
}

fn gap_status_tag_v1(status: CanonicalViewGapStatusV1) -> u32 {
    match status {
        CanonicalViewGapStatusV1::Complete => 1,
        CanonicalViewGapStatusV1::MissingRequiredEvidence => 2,
        CanonicalViewGapStatusV1::MissingOptionalEvidenceOnly => 3,
        CanonicalViewGapStatusV1::ProviderUnavailable => 4,
        CanonicalViewGapStatusV1::ProviderContractUnverified => 5,
        CanonicalViewGapStatusV1::AmbiguousArtifacts => 6,
        CanonicalViewGapStatusV1::IncompatibleCadence => 7,
        CanonicalViewGapStatusV1::IncompatibleMarket => 8,
        CanonicalViewGapStatusV1::IncompatibleSymbol => 9,
        CanonicalViewGapStatusV1::CutoffMismatch => 10,
        CanonicalViewGapStatusV1::IntegrityFailure => 11,
        CanonicalViewGapStatusV1::TrainerUnavailable => 12,
        CanonicalViewGapStatusV1::ProviderSingleRequestCapacityExceeded => 13,
        CanonicalViewGapStatusV1::SegmentedAcquisitionRequired => 14,
        CanonicalViewGapStatusV1::SegmentedAcquisitionUnsupported => 15,
    }
}

fn gap_status_from_tag_v1(tag: u32) -> Result<CanonicalViewGapStatusV1, String> {
    match tag {
        1 => Ok(CanonicalViewGapStatusV1::Complete),
        2 => Ok(CanonicalViewGapStatusV1::MissingRequiredEvidence),
        3 => Ok(CanonicalViewGapStatusV1::MissingOptionalEvidenceOnly),
        4 => Ok(CanonicalViewGapStatusV1::ProviderUnavailable),
        5 => Ok(CanonicalViewGapStatusV1::ProviderContractUnverified),
        6 => Ok(CanonicalViewGapStatusV1::AmbiguousArtifacts),
        7 => Ok(CanonicalViewGapStatusV1::IncompatibleCadence),
        8 => Ok(CanonicalViewGapStatusV1::IncompatibleMarket),
        9 => Ok(CanonicalViewGapStatusV1::IncompatibleSymbol),
        10 => Ok(CanonicalViewGapStatusV1::CutoffMismatch),
        11 => Ok(CanonicalViewGapStatusV1::IntegrityFailure),
        12 => Ok(CanonicalViewGapStatusV1::TrainerUnavailable),
        13 => Ok(CanonicalViewGapStatusV1::ProviderSingleRequestCapacityExceeded),
        14 => Ok(CanonicalViewGapStatusV1::SegmentedAcquisitionRequired),
        15 => Ok(CanonicalViewGapStatusV1::SegmentedAcquisitionUnsupported),
        _ => Err("canonical gap status rejected".into()),
    }
}

fn receipt_status_tag_v1(status: LearningEvidenceRequestStatusV1) -> u32 {
    match status {
        LearningEvidenceRequestStatusV1::ReadyNotAttempted => 1,
        LearningEvidenceRequestStatusV1::EvidenceAcquired => 2,
        LearningEvidenceRequestStatusV1::ProviderRejected => 3,
        LearningEvidenceRequestStatusV1::TimeoutNoRetry => 4,
        LearningEvidenceRequestStatusV1::InvalidResponse => 5,
        LearningEvidenceRequestStatusV1::TechnicalFailure => 6,
        LearningEvidenceRequestStatusV1::RequestBudgetExhausted => 7,
        LearningEvidenceRequestStatusV1::RegistrationInvalid => 8,
        LearningEvidenceRequestStatusV1::GapNoLongerCurrent => 9,
        LearningEvidenceRequestStatusV1::EquivalentSnapshotExists => 10,
        LearningEvidenceRequestStatusV1::MissingNetworkConsent => 11,
    }
}

fn receipt_status_from_tag_v1(tag: u32) -> Result<LearningEvidenceRequestStatusV1, String> {
    match tag {
        1 => Ok(LearningEvidenceRequestStatusV1::ReadyNotAttempted),
        2 => Ok(LearningEvidenceRequestStatusV1::EvidenceAcquired),
        3 => Ok(LearningEvidenceRequestStatusV1::ProviderRejected),
        4 => Ok(LearningEvidenceRequestStatusV1::TimeoutNoRetry),
        5 => Ok(LearningEvidenceRequestStatusV1::InvalidResponse),
        6 => Ok(LearningEvidenceRequestStatusV1::TechnicalFailure),
        7 => Ok(LearningEvidenceRequestStatusV1::RequestBudgetExhausted),
        8 => Ok(LearningEvidenceRequestStatusV1::RegistrationInvalid),
        9 => Ok(LearningEvidenceRequestStatusV1::GapNoLongerCurrent),
        10 => Ok(LearningEvidenceRequestStatusV1::EquivalentSnapshotExists),
        11 => Ok(LearningEvidenceRequestStatusV1::MissingNetworkConsent),
        _ => Err("learning receipt status rejected".into()),
    }
}

fn safety_to_protobuf_v1(
    counters: &LearningEvidenceSafetyCountersV1,
) -> Result<LearningEvidenceSafetyCountersProtobufV1, String> {
    let convert = |value| u64::try_from(value).map_err(|_| "learning counter overflow".to_string());
    Ok(LearningEvidenceSafetyCountersProtobufV1 {
        active_committee_count: convert(counters.active_committee_count)?,
        request_attempts: convert(counters.request_attempts)?,
        retry_count: convert(counters.retry_count)?,
        transport_constructions: convert(counters.transport_constructions)?,
        credential_reads: convert(counters.credential_reads)?,
        prospective_artifact_reads: convert(counters.prospective_artifact_reads)?,
        prospective_label_reads: convert(counters.prospective_label_reads)?,
        future_evaluation_reads: convert(counters.future_evaluation_reads)?,
        active_model_changes: convert(counters.active_model_changes)?,
        chair_decisions: convert(counters.chair_decisions)?,
        votes: convert(counters.votes)?,
        rewards: convert(counters.rewards)?,
        penalties: convert(counters.penalties)?,
        voice_changes: convert(counters.voice_changes)?,
        promotions: convert(counters.promotions)?,
        executions: convert(counters.executions)?,
    })
}

fn safety_from_protobuf_v1(
    counters: Option<LearningEvidenceSafetyCountersProtobufV1>,
) -> Result<LearningEvidenceSafetyCountersV1, String> {
    let counters = counters.ok_or_else(|| "learning safety counters missing".to_string())?;
    let convert =
        |value| usize::try_from(value).map_err(|_| "learning counter rejected".to_string());
    Ok(LearningEvidenceSafetyCountersV1 {
        active_committee_count: convert(counters.active_committee_count)?,
        request_attempts: convert(counters.request_attempts)?,
        retry_count: convert(counters.retry_count)?,
        transport_constructions: convert(counters.transport_constructions)?,
        credential_reads: convert(counters.credential_reads)?,
        prospective_artifact_reads: convert(counters.prospective_artifact_reads)?,
        prospective_label_reads: convert(counters.prospective_label_reads)?,
        future_evaluation_reads: convert(counters.future_evaluation_reads)?,
        active_model_changes: convert(counters.active_model_changes)?,
        chair_decisions: convert(counters.chair_decisions)?,
        votes: convert(counters.votes)?,
        rewards: convert(counters.rewards)?,
        penalties: convert(counters.penalties)?,
        voice_changes: convert(counters.voice_changes)?,
        promotions: convert(counters.promotions)?,
        executions: convert(counters.executions)?,
    })
}

fn encode_learning_artifact_envelope_v1<M: Message>(
    artifact_kind: &str,
    semantic_digest: &str,
    source_artifact_digests: Vec<String>,
    payload: M,
) -> Result<Vec<u8>, String> {
    if artifact_kind.is_empty() || semantic_digest.is_empty() {
        return Err("learning artifact envelope rejected".into());
    }
    let payload = payload.encode_to_vec();
    Ok(CanonicalLearningArtifactEnvelopeProtobufV0 {
        magic: LEARNING_ENVELOPE_MAGIC_V0.into(),
        envelope_version: 1,
        schema_name: "soma.learning_evidence.v1".into(),
        artifact_kind: artifact_kind.into(),
        semantic_digest: semantic_digest.into(),
        payload_length: u64::try_from(payload.len())
            .map_err(|_| "learning artifact payload too large".to_string())?,
        payload_digest: canonical_hash_hex(&payload),
        payload,
        source_artifact_digests,
    }
    .encode_to_vec())
}

fn decode_learning_artifact_envelope_v1(
    bytes: &[u8],
    expected_kind: &str,
) -> Result<(String, Vec<String>, Vec<u8>), String> {
    let envelope = CanonicalLearningArtifactEnvelopeProtobufV0::decode(bytes)
        .map_err(|_| "learning artifact envelope decode failed".to_string())?;
    if envelope.magic != LEARNING_ENVELOPE_MAGIC_V0
        || envelope.envelope_version != 1
        || envelope.schema_name != "soma.learning_evidence.v1"
        || envelope.artifact_kind != expected_kind
        || envelope.semantic_digest.is_empty()
        || usize::try_from(envelope.payload_length).ok() != Some(envelope.payload.len())
        || envelope.payload_digest != canonical_hash_hex(&envelope.payload)
    {
        return Err("learning artifact envelope rejected".into());
    }
    Ok((
        envelope.semantic_digest,
        envelope.source_artifact_digests,
        envelope.payload,
    ))
}

fn gap_to_protobuf_v1(gap: &AgentCanonicalViewGapV1) -> Result<CanonicalViewGapProtobufV1, String> {
    Ok(CanonicalViewGapProtobufV1 {
        agent_id: gap.agent_id.clone(),
        intent_digest: gap.intent_digest.clone(),
        market_scopes: gap
            .market_scopes
            .iter()
            .map(|market| u32::from(market_scope_code_v0(*market)))
            .collect(),
        symbols: gap.symbols.clone(),
        cadence: gap.cadence.clone(),
        lookback: Some(learning_lookback_to_protobuf_v1(&gap.lookback)?),
        information_cutoff_ms: gap.information_cutoff_ms,
        maximum_staleness_ms: gap.maximum_staleness_ms,
        required_dataset_kinds: gap
            .required_dataset_kinds
            .iter()
            .map(|kind| u32::from(dataset_kind_code_v0(*kind)))
            .collect(),
        resolved_required_dataset_kinds: gap
            .resolved_required_dataset_kinds
            .iter()
            .map(|kind| u32::from(dataset_kind_code_v0(*kind)))
            .collect(),
        missing_required_dataset_kinds: gap
            .missing_required_dataset_kinds
            .iter()
            .map(|kind| u32::from(dataset_kind_code_v0(*kind)))
            .collect(),
        optional_dataset_kinds: gap
            .optional_dataset_kinds
            .iter()
            .map(|kind| u32::from(dataset_kind_code_v0(*kind)))
            .collect(),
        resolved_optional_dataset_kinds: gap
            .resolved_optional_dataset_kinds
            .iter()
            .map(|kind| u32::from(dataset_kind_code_v0(*kind)))
            .collect(),
        missing_optional_dataset_kinds: gap
            .missing_optional_dataset_kinds
            .iter()
            .map(|kind| u32::from(dataset_kind_code_v0(*kind)))
            .collect(),
        usable_artifact_digests: gap.usable_artifact_digests.clone(),
        rejected_artifact_digests: gap.rejected_artifact_digests.clone(),
        authorized_provider_ids: gap.authorized_provider_ids.clone(),
        trainer_available: gap.trainer_available,
        status: gap_status_tag_v1(gap.status),
        gap_digest: gap.gap_digest.clone(),
    })
}

fn gap_from_protobuf_v1(
    gap: CanonicalViewGapProtobufV1,
) -> Result<AgentCanonicalViewGapV1, String> {
    let convert_kinds = |values: Vec<u32>| {
        values
            .into_iter()
            .map(dataset_kind_from_code_v1)
            .collect::<Result<Vec<_>, _>>()
    };
    let gap = AgentCanonicalViewGapV1 {
        agent_id: gap.agent_id,
        intent_digest: gap.intent_digest,
        market_scopes: gap
            .market_scopes
            .into_iter()
            .map(market_scope_from_code_v1)
            .collect::<Result<Vec<_>, _>>()?,
        symbols: gap.symbols,
        cadence: gap.cadence,
        lookback: learning_lookback_from_protobuf_v1(gap.lookback)?,
        information_cutoff_ms: gap.information_cutoff_ms,
        maximum_staleness_ms: gap.maximum_staleness_ms,
        required_dataset_kinds: convert_kinds(gap.required_dataset_kinds)?,
        resolved_required_dataset_kinds: convert_kinds(gap.resolved_required_dataset_kinds)?,
        missing_required_dataset_kinds: convert_kinds(gap.missing_required_dataset_kinds)?,
        optional_dataset_kinds: convert_kinds(gap.optional_dataset_kinds)?,
        resolved_optional_dataset_kinds: convert_kinds(gap.resolved_optional_dataset_kinds)?,
        missing_optional_dataset_kinds: convert_kinds(gap.missing_optional_dataset_kinds)?,
        usable_artifact_digests: gap.usable_artifact_digests,
        rejected_artifact_digests: gap.rejected_artifact_digests,
        authorized_provider_ids: gap.authorized_provider_ids,
        trainer_available: gap.trainer_available,
        status: gap_status_from_tag_v1(gap.status)?,
        gap_digest: gap.gap_digest,
    };
    if gap.gap_digest != canonical_view_gap_digest_v1(&gap) {
        return Err("canonical view gap identity rejected".into());
    }
    Ok(gap)
}

pub fn encode_agent_canonical_view_gap_report_protobuf_v1(
    report: &AgentCanonicalViewGapReportV1,
) -> Result<Vec<u8>, String> {
    if report.report_version != CANONICAL_VIEW_GAP_REPORT_VERSION_V1
        || report.report_digest != canonical_view_gap_report_digest_v1(report)
    {
        return Err("canonical view gap report rejected".into());
    }
    encode_learning_artifact_envelope_v1(
        LEARNING_GAP_ARTIFACT_KIND_V1,
        &report.report_digest,
        report
            .gaps
            .iter()
            .flat_map(|gap| gap.usable_artifact_digests.clone())
            .collect(),
        CanonicalViewGapReportProtobufV1 {
            report_version: report.report_version.clone(),
            gaps: report
                .gaps
                .iter()
                .map(gap_to_protobuf_v1)
                .collect::<Result<Vec<_>, _>>()?,
            provider_contract_digests: report.provider_contract_digests.clone(),
            safety_counters: Some(safety_to_protobuf_v1(&report.safety_counters)?),
            report_digest: report.report_digest.clone(),
        },
    )
}

pub fn decode_agent_canonical_view_gap_report_protobuf_v1(
    bytes: &[u8],
) -> Result<AgentCanonicalViewGapReportV1, String> {
    let (semantic_digest, _, payload) =
        decode_learning_artifact_envelope_v1(bytes, LEARNING_GAP_ARTIFACT_KIND_V1)?;
    let value = CanonicalViewGapReportProtobufV1::decode(payload.as_slice())
        .map_err(|_| "canonical view gap report decode failed".to_string())?;
    let report = AgentCanonicalViewGapReportV1 {
        report_version: value.report_version,
        gaps: value
            .gaps
            .into_iter()
            .map(gap_from_protobuf_v1)
            .collect::<Result<Vec<_>, _>>()?,
        provider_contract_digests: value.provider_contract_digests,
        safety_counters: safety_from_protobuf_v1(value.safety_counters)?,
        report_digest: value.report_digest,
    };
    if report.report_version != CANONICAL_VIEW_GAP_REPORT_VERSION_V1
        || report.report_digest != canonical_view_gap_report_digest_v1(&report)
        || report.report_digest != semantic_digest
    {
        return Err("canonical view gap report identity rejected".into());
    }
    Ok(report)
}

pub fn encode_learning_evidence_registration_protobuf_v1(
    registration: &LearningEvidenceAcquisitionRegistrationV1,
) -> Result<Vec<u8>, String> {
    validate_learning_evidence_acquisition_registration_v1(registration)?;
    encode_learning_artifact_envelope_v1(
        LEARNING_REGISTRATION_ARTIFACT_KIND_V1,
        &registration.registration_digest,
        registration.gap_report_digests.clone(),
        LearningEvidenceRegistrationProtobufV1 {
            registration_version: registration.registration_version.clone(),
            target_agent_ids: registration.target_agent_ids.clone(),
            gap_report_digests: registration.gap_report_digests.clone(),
            provider_id: registration.provider_id.clone(),
            provider_contract_digest: registration.provider_contract_digest.clone(),
            dataset_kind: u32::from(dataset_kind_code_v0(registration.dataset_kind)),
            market_scope: u32::from(market_scope_code_v0(registration.market_scope)),
            symbols: registration.symbols.clone(),
            cadence: registration.cadence.clone(),
            lookback: Some(learning_lookback_to_protobuf_v1(&registration.lookback)?),
            information_cutoff_ms: registration.information_cutoff_ms,
            expected_timestamp_ms: registration.expected_timestamp_ms.clone(),
            protected_registration_digests: registration.protected_registration_digests.clone(),
            excluded_timestamp_ms: registration.excluded_timestamp_ms.clone(),
            maximum_requests: u64::try_from(registration.maximum_requests)
                .map_err(|_| "learning request budget overflow".to_string())?,
            maximum_concurrency: u64::try_from(registration.maximum_concurrency)
                .map_err(|_| "learning concurrency overflow".to_string())?,
            maximum_retries: u64::try_from(registration.maximum_retries)
                .map_err(|_| "learning retry budget overflow".to_string())?,
            maximum_response_bytes: u64::try_from(registration.maximum_response_bytes)
                .map_err(|_| "learning response budget overflow".to_string())?,
            credential_free_required: registration.credential_free_required,
            read_only_required: registration.read_only_required,
            prospective_storage_forbidden: registration.prospective_storage_forbidden,
            registration_digest: registration.registration_digest.clone(),
        },
    )
}

pub fn decode_learning_evidence_registration_protobuf_v1(
    bytes: &[u8],
) -> Result<LearningEvidenceAcquisitionRegistrationV1, String> {
    let (semantic_digest, _, payload) =
        decode_learning_artifact_envelope_v1(bytes, LEARNING_REGISTRATION_ARTIFACT_KIND_V1)?;
    let value = LearningEvidenceRegistrationProtobufV1::decode(payload.as_slice())
        .map_err(|_| "learning evidence registration decode failed".to_string())?;
    let registration = LearningEvidenceAcquisitionRegistrationV1 {
        registration_version: value.registration_version,
        target_agent_ids: value.target_agent_ids,
        gap_report_digests: value.gap_report_digests,
        provider_id: value.provider_id,
        provider_contract_digest: value.provider_contract_digest,
        dataset_kind: dataset_kind_from_code_v1(value.dataset_kind)?,
        market_scope: market_scope_from_code_v1(value.market_scope)?,
        symbols: value.symbols,
        cadence: value.cadence,
        lookback: learning_lookback_from_protobuf_v1(value.lookback)?,
        information_cutoff_ms: value.information_cutoff_ms,
        expected_timestamp_ms: value.expected_timestamp_ms,
        protected_registration_digests: value.protected_registration_digests,
        excluded_timestamp_ms: value.excluded_timestamp_ms,
        maximum_requests: usize::try_from(value.maximum_requests)
            .map_err(|_| "learning request budget rejected".to_string())?,
        maximum_concurrency: usize::try_from(value.maximum_concurrency)
            .map_err(|_| "learning concurrency rejected".to_string())?,
        maximum_retries: usize::try_from(value.maximum_retries)
            .map_err(|_| "learning retry budget rejected".to_string())?,
        maximum_response_bytes: usize::try_from(value.maximum_response_bytes)
            .map_err(|_| "learning response budget rejected".to_string())?,
        credential_free_required: value.credential_free_required,
        read_only_required: value.read_only_required,
        prospective_storage_forbidden: value.prospective_storage_forbidden,
        registration_digest: value.registration_digest,
    };
    validate_learning_evidence_acquisition_registration_v1(&registration)?;
    if registration.registration_digest != semantic_digest {
        return Err("learning evidence registration identity rejected".into());
    }
    Ok(registration)
}

pub fn encode_learning_evidence_provenance_protobuf_v1(
    manifest: &LearningDataProvenanceManifestV0,
) -> Result<Vec<u8>, String> {
    if seal_learning_data_provenance_manifest_v0(manifest.clone()).as_ref() != Ok(manifest) {
        return Err("learning provenance manifest rejected".into());
    }
    encode_learning_artifact_envelope_v1(
        LEARNING_PROVENANCE_ARTIFACT_KIND_V1,
        &manifest.manifest_digest,
        vec![manifest.normalized_artifact_digest.clone()],
        LearningEvidenceProvenanceProtobufV1 {
            source_provider_id: manifest.source_provider_id.clone(),
            source_type: manifest.source_type.clone(),
            acquisition_request_identity: manifest.acquisition_request_identity.clone(),
            fetch_timestamp_ms: manifest.fetch_timestamp_ms,
            publication_event_timestamp_ms: manifest.publication_event_timestamp_ms,
            raw_content_digest: manifest.raw_content_digest.clone(),
            parser_version: manifest.parser_version.clone(),
            normalized_artifact_digest: manifest.normalized_artifact_digest.clone(),
            sanitized: manifest.sanitized,
            credential_free: manifest.credential_free,
            information_cutoff_ms: manifest.information_cutoff_ms,
            usage_classification: 1,
            manifest_digest: manifest.manifest_digest.clone(),
        },
    )
}

pub fn decode_learning_evidence_provenance_protobuf_v1(
    bytes: &[u8],
) -> Result<LearningDataProvenanceManifestV0, String> {
    let (semantic_digest, _, payload) =
        decode_learning_artifact_envelope_v1(bytes, LEARNING_PROVENANCE_ARTIFACT_KIND_V1)?;
    let value = LearningEvidenceProvenanceProtobufV1::decode(payload.as_slice())
        .map_err(|_| "learning provenance manifest decode failed".to_string())?;
    if value.usage_classification != 1 {
        return Err("learning provenance classification rejected".into());
    }
    let manifest = LearningDataProvenanceManifestV0 {
        source_provider_id: value.source_provider_id,
        source_type: value.source_type,
        acquisition_request_identity: value.acquisition_request_identity,
        fetch_timestamp_ms: value.fetch_timestamp_ms,
        publication_event_timestamp_ms: value.publication_event_timestamp_ms,
        raw_content_digest: value.raw_content_digest,
        parser_version: value.parser_version,
        normalized_artifact_digest: value.normalized_artifact_digest,
        sanitized: value.sanitized,
        credential_free: value.credential_free,
        information_cutoff_ms: value.information_cutoff_ms,
        usage_classification: LearningDataUsageClassificationV0::ResearchOnlyUnconsumed,
        manifest_digest: value.manifest_digest,
    };
    if seal_learning_data_provenance_manifest_v0(manifest.clone()).as_ref() != Ok(&manifest)
        || manifest.manifest_digest != semantic_digest
    {
        return Err("learning provenance manifest identity rejected".into());
    }
    Ok(manifest)
}

pub fn encode_learning_evidence_receipt_protobuf_v1(
    receipt: &LearningEvidenceRequestReceiptV1,
) -> Result<Vec<u8>, String> {
    validate_learning_evidence_request_receipt_v1(receipt)?;
    encode_learning_artifact_envelope_v1(
        LEARNING_RECEIPT_ARTIFACT_KIND_V1,
        &receipt.receipt_digest,
        vec![receipt.registration_digest.clone()],
        LearningEvidenceReceiptProtobufV1 {
            receipt_version: receipt.receipt_version.clone(),
            registration_digest: receipt.registration_digest.clone(),
            provider_contract_digest: receipt.provider_contract_digest.clone(),
            request_attempted: receipt.request_attempted,
            request_count: u64::try_from(receipt.request_count)
                .map_err(|_| "learning request count overflow".to_string())?,
            retry_count: u64::try_from(receipt.retry_count)
                .map_err(|_| "learning retry count overflow".to_string())?,
            status: receipt_status_tag_v1(receipt.status),
            http_status_class: receipt.http_status_class.clone(),
            returned_row_count: u64::try_from(receipt.returned_row_count)
                .map_err(|_| "learning row count overflow".to_string())?,
            verified_row_count: u64::try_from(receipt.verified_row_count)
                .map_err(|_| "learning row count overflow".to_string())?,
            raw_response_digest: receipt.raw_response_digest.clone(),
            provenance_manifest_digest: receipt.provenance_manifest_digest.clone(),
            snapshot_digest: receipt.snapshot_digest.clone(),
            receipt_digest: receipt.receipt_digest.clone(),
        },
    )
}

pub fn decode_learning_evidence_receipt_protobuf_v1(
    bytes: &[u8],
) -> Result<LearningEvidenceRequestReceiptV1, String> {
    let (semantic_digest, _, payload) =
        decode_learning_artifact_envelope_v1(bytes, LEARNING_RECEIPT_ARTIFACT_KIND_V1)?;
    let value = LearningEvidenceReceiptProtobufV1::decode(payload.as_slice())
        .map_err(|_| "learning evidence receipt decode failed".to_string())?;
    let receipt = LearningEvidenceRequestReceiptV1 {
        receipt_version: value.receipt_version,
        registration_digest: value.registration_digest,
        provider_contract_digest: value.provider_contract_digest,
        request_attempted: value.request_attempted,
        request_count: usize::try_from(value.request_count)
            .map_err(|_| "learning request count rejected".to_string())?,
        retry_count: usize::try_from(value.retry_count)
            .map_err(|_| "learning retry count rejected".to_string())?,
        status: receipt_status_from_tag_v1(value.status)?,
        http_status_class: value.http_status_class,
        returned_row_count: usize::try_from(value.returned_row_count)
            .map_err(|_| "learning row count rejected".to_string())?,
        verified_row_count: usize::try_from(value.verified_row_count)
            .map_err(|_| "learning row count rejected".to_string())?,
        raw_response_digest: value.raw_response_digest,
        provenance_manifest_digest: value.provenance_manifest_digest,
        snapshot_digest: value.snapshot_digest,
        receipt_digest: value.receipt_digest,
    };
    validate_learning_evidence_request_receipt_v1(&receipt)?;
    if receipt.receipt_digest != semantic_digest {
        return Err("learning evidence receipt identity rejected".into());
    }
    Ok(receipt)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LearningEvidenceArtifactWriteStatusV1 {
    Written,
    DuplicateRejected,
}

fn write_learning_evidence_artifact_v1<F>(
    path: &Path,
    bytes: &[u8],
    expected_digest: &str,
    verify: F,
) -> Result<LearningEvidenceArtifactWriteStatusV1, String>
where
    F: Fn(&[u8]) -> Result<String, String>,
{
    if !safe_learning_data_path_v0(path)
        || path.extension().is_none_or(|extension| extension != "pb")
        || expected_digest.is_empty()
    {
        return Err("learning evidence storage path rejected".into());
    }
    if path.is_file() {
        let existing =
            fs::read(path).map_err(|_| "learning evidence artifact reopen failed".to_string())?;
        return if verify(&existing)? == expected_digest {
            Ok(LearningEvidenceArtifactWriteStatusV1::DuplicateRejected)
        } else {
            Err("learning evidence artifact identity collision".into())
        };
    }
    let parent = path
        .parent()
        .ok_or_else(|| "learning evidence artifact parent missing".to_string())?;
    fs::create_dir_all(parent)
        .map_err(|_| "learning evidence directory unavailable".to_string())?;
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| "learning evidence filename rejected".to_string())?;
    let temporary = parent.join(format!(".{file_name}.{}.tmp", std::process::id()));
    let write_result = (|| {
        let mut file = File::options()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .map_err(|_| "learning evidence temporary create failed".to_string())?;
        file.write_all(bytes)
            .map_err(|_| "learning evidence temporary write failed".to_string())?;
        file.flush()
            .map_err(|_| "learning evidence temporary flush failed".to_string())?;
        file.sync_all()
            .map_err(|_| "learning evidence temporary sync failed".to_string())?;
        drop(file);
        let temporary_bytes = fs::read(&temporary)
            .map_err(|_| "learning evidence temporary reopen failed".to_string())?;
        if verify(&temporary_bytes)? != expected_digest {
            return Err("learning evidence temporary verification failed".into());
        }
        fs::rename(&temporary, path)
            .map_err(|_| "learning evidence atomic rename failed".to_string())?;
        let final_bytes =
            fs::read(path).map_err(|_| "learning evidence final reopen failed".to_string())?;
        if verify(&final_bytes)? != expected_digest {
            return Err("learning evidence final verification failed".into());
        }
        Ok(LearningEvidenceArtifactWriteStatusV1::Written)
    })();
    if write_result.is_err() && temporary.is_file() {
        let _ = fs::remove_file(temporary);
    }
    write_result
}

pub fn write_and_verify_agent_canonical_view_gap_report_v1(
    report: &AgentCanonicalViewGapReportV1,
    root: &Path,
) -> Result<LearningEvidenceArtifactWriteStatusV1, String> {
    let bytes = encode_agent_canonical_view_gap_report_protobuf_v1(report)?;
    let path = root
        .join("acquisition_v1")
        .join("gap_reports")
        .join(format!("{}.pb", report.report_digest));
    write_learning_evidence_artifact_v1(&path, &bytes, &report.report_digest, |bytes| {
        Ok(decode_agent_canonical_view_gap_report_protobuf_v1(bytes)?.report_digest)
    })
}

pub fn write_and_verify_learning_evidence_registration_v1(
    registration: &LearningEvidenceAcquisitionRegistrationV1,
    root: &Path,
) -> Result<LearningEvidenceArtifactWriteStatusV1, String> {
    let bytes = encode_learning_evidence_registration_protobuf_v1(registration)?;
    let path = root
        .join("acquisition_v1")
        .join("registrations")
        .join(format!("{}.pb", registration.registration_digest));
    write_learning_evidence_artifact_v1(&path, &bytes, &registration.registration_digest, |bytes| {
        Ok(decode_learning_evidence_registration_protobuf_v1(bytes)?.registration_digest)
    })
}

pub fn read_learning_evidence_registration_v1(
    registration_digest: &str,
    root: &Path,
) -> Result<LearningEvidenceAcquisitionRegistrationV1, String> {
    let path = root
        .join("acquisition_v1")
        .join("registrations")
        .join(format!("{registration_digest}.pb"));
    if !safe_learning_data_path_v0(&path) {
        return Err("learning registration path rejected".into());
    }
    decode_learning_evidence_registration_protobuf_v1(
        &fs::read(path).map_err(|_| "learning registration unavailable".to_string())?,
    )
}

pub fn write_and_verify_learning_evidence_provenance_v1(
    manifest: &LearningDataProvenanceManifestV0,
    root: &Path,
) -> Result<LearningEvidenceArtifactWriteStatusV1, String> {
    let bytes = encode_learning_evidence_provenance_protobuf_v1(manifest)?;
    let path = root
        .join("acquisition_v1")
        .join("provenance_manifests")
        .join(format!("{}.pb", manifest.manifest_digest));
    write_learning_evidence_artifact_v1(&path, &bytes, &manifest.manifest_digest, |bytes| {
        Ok(decode_learning_evidence_provenance_protobuf_v1(bytes)?.manifest_digest)
    })
}

pub fn write_and_verify_learning_evidence_receipt_v1(
    receipt: &LearningEvidenceRequestReceiptV1,
    root: &Path,
) -> Result<LearningEvidenceArtifactWriteStatusV1, String> {
    let bytes = encode_learning_evidence_receipt_protobuf_v1(receipt)?;
    let path = root
        .join("acquisition_v1")
        .join("receipts")
        .join(format!("{}.pb", receipt.registration_digest));
    write_learning_evidence_artifact_v1(&path, &bytes, &receipt.receipt_digest, |bytes| {
        Ok(decode_learning_evidence_receipt_protobuf_v1(bytes)?.receipt_digest)
    })
}

pub fn read_learning_evidence_receipt_v1(
    registration_digest: &str,
    root: &Path,
) -> Result<Option<LearningEvidenceRequestReceiptV1>, String> {
    let path = root
        .join("acquisition_v1")
        .join("receipts")
        .join(format!("{registration_digest}.pb"));
    if !safe_learning_data_path_v0(&path) {
        return Err("learning receipt path rejected".into());
    }
    path.is_file()
        .then(|| {
            decode_learning_evidence_receipt_protobuf_v1(
                &fs::read(path).map_err(|_| "learning receipt unavailable".to_string())?,
            )
        })
        .transpose()
}

const COMPOSITE_REGISTRATION_ARTIFACT_KIND_V1: &str =
    "composite-learning-acquisition-registration-v1";
const COMPOSITE_SEGMENT_RECEIPT_ARTIFACT_KIND_V1: &str = "learning-evidence-segment-receipt-v1";
const COMPOSITE_SEGMENT_CAPSULE_ARTIFACT_KIND_V1: &str = "learning-evidence-segment-capsule-v1";
const COMPOSITE_EPOCH_RECEIPT_ARTIFACT_KIND_V1: &str = "composite-learning-epoch-receipt-v1";
const COMPOSITE_PROVENANCE_ARTIFACT_KIND_V1: &str = "composite-learning-provenance-v1";

#[derive(Clone, PartialEq, Message)]
struct LearningEvidenceSegmentRegistrationProtobufV1 {
    #[prost(uint64, tag = "1")]
    segment_index: u64,
    #[prost(uint64, repeated, tag = "2")]
    expected_timestamps: Vec<u64>,
    #[prost(uint64, tag = "3")]
    expected_row_count: u64,
    #[prost(string, tag = "4")]
    request_to_utc: String,
    #[prost(uint64, tag = "5")]
    maximum_requests: u64,
    #[prost(uint64, tag = "6")]
    maximum_retries: u64,
    #[prost(string, tag = "7")]
    segment_digest: String,
}

#[derive(Clone, PartialEq, Message)]
struct CompositeLearningRegistrationProtobufV1 {
    #[prost(string, tag = "1")]
    registration_version: String,
    #[prost(string, repeated, tag = "2")]
    target_agent_ids: Vec<String>,
    #[prost(string, tag = "3")]
    intent_digest: String,
    #[prost(string, tag = "4")]
    gap_report_digest: String,
    #[prost(string, tag = "5")]
    provider_contract_digest: String,
    #[prost(uint32, tag = "6")]
    dataset_kind: u32,
    #[prost(uint32, tag = "7")]
    market_scope: u32,
    #[prost(string, repeated, tag = "8")]
    symbols: Vec<String>,
    #[prost(string, tag = "9")]
    cadence: String,
    #[prost(uint64, tag = "10")]
    information_cutoff_ms: u64,
    #[prost(uint64, tag = "11")]
    required_row_count: u64,
    #[prost(string, tag = "12")]
    expected_timestamp_digest: String,
    #[prost(message, repeated, tag = "13")]
    segments: Vec<LearningEvidenceSegmentRegistrationProtobufV1>,
    #[prost(uint64, tag = "14")]
    maximum_total_requests: u64,
    #[prost(uint64, tag = "15")]
    maximum_concurrency: u64,
    #[prost(uint64, tag = "16")]
    maximum_retries_per_segment: u64,
    #[prost(string, repeated, tag = "17")]
    protected_registration_digests: Vec<String>,
    #[prost(uint64, repeated, tag = "18")]
    excluded_timestamp_ms: Vec<u64>,
    #[prost(bool, tag = "19")]
    read_only_required: bool,
    #[prost(bool, tag = "20")]
    credential_free_required: bool,
    #[prost(bool, tag = "21")]
    prospective_storage_forbidden: bool,
    #[prost(string, tag = "22")]
    registration_digest: String,
}

#[derive(Clone, PartialEq, Message)]
struct CompositeSegmentReceiptProtobufV1 {
    #[prost(string, tag = "1")]
    receipt_version: String,
    #[prost(string, tag = "2")]
    composite_registration_digest: String,
    #[prost(string, tag = "3")]
    segment_digest: String,
    #[prost(uint64, tag = "4")]
    segment_index: u64,
    #[prost(bool, tag = "5")]
    request_attempted: bool,
    #[prost(uint64, tag = "6")]
    request_count: u64,
    #[prost(uint64, tag = "7")]
    retry_count: u64,
    #[prost(uint32, tag = "8")]
    status: u32,
    #[prost(string, optional, tag = "9")]
    http_status_class: Option<String>,
    #[prost(uint64, tag = "10")]
    returned_row_count: u64,
    #[prost(uint64, tag = "11")]
    verified_row_count: u64,
    #[prost(string, optional, tag = "12")]
    raw_response_digest: Option<String>,
    #[prost(string, optional, tag = "13")]
    segment_capsule_digest: Option<String>,
    #[prost(string, tag = "14")]
    receipt_digest: String,
}

#[derive(Clone, PartialEq, Message)]
struct CompositeHistoricalRowProtobufV1 {
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
struct CompositeSegmentCapsuleProtobufV1 {
    #[prost(string, tag = "1")]
    capsule_version: String,
    #[prost(string, tag = "2")]
    composite_registration_digest: String,
    #[prost(string, tag = "3")]
    segment_digest: String,
    #[prost(string, tag = "4")]
    segment_receipt_digest: String,
    #[prost(uint64, tag = "5")]
    segment_index: u64,
    #[prost(string, tag = "6")]
    provider_id: String,
    #[prost(string, tag = "7")]
    symbol: String,
    #[prost(string, tag = "8")]
    cadence: String,
    #[prost(uint64, repeated, tag = "9")]
    expected_timestamps: Vec<u64>,
    #[prost(message, repeated, tag = "10")]
    rows: Vec<CompositeHistoricalRowProtobufV1>,
    #[prost(string, tag = "11")]
    segment_semantic_digest: String,
    #[prost(bool, tag = "12")]
    finalized: bool,
    #[prost(bool, tag = "13")]
    read_only: bool,
    #[prost(bool, tag = "14")]
    credential_free: bool,
    #[prost(string, tag = "15")]
    capsule_digest: String,
}

#[derive(Clone, PartialEq, Message)]
struct CompositeEpochReceiptProtobufV1 {
    #[prost(string, tag = "1")]
    receipt_version: String,
    #[prost(string, tag = "2")]
    registration_digest: String,
    #[prost(string, repeated, tag = "3")]
    segment_receipt_digests: Vec<String>,
    #[prost(uint64, tag = "4")]
    attempted_segment_count: u64,
    #[prost(uint64, tag = "5")]
    successful_segment_count: u64,
    #[prost(uint64, tag = "6")]
    request_count: u64,
    #[prost(uint64, tag = "7")]
    retry_count: u64,
    #[prost(string, optional, tag = "8")]
    merged_snapshot_digest: Option<String>,
    #[prost(string, optional, tag = "9")]
    merged_provenance_digest: Option<String>,
    #[prost(uint32, tag = "10")]
    status: u32,
    #[prost(string, tag = "11")]
    receipt_digest: String,
}

#[derive(Clone, PartialEq, Message)]
struct CompositeMergedProvenanceProtobufV1 {
    #[prost(string, tag = "1")]
    provenance_version: String,
    #[prost(string, tag = "2")]
    registration_digest: String,
    #[prost(string, tag = "3")]
    provider_contract_digest: String,
    #[prost(string, repeated, tag = "4")]
    segment_digests: Vec<String>,
    #[prost(string, repeated, tag = "5")]
    segment_receipt_digests: Vec<String>,
    #[prost(string, repeated, tag = "6")]
    segment_capsule_digests: Vec<String>,
    #[prost(string, tag = "7")]
    canonical_snapshot_digest: String,
    #[prost(string, tag = "8")]
    expected_timestamp_digest: String,
    #[prost(uint64, tag = "9")]
    required_row_count: u64,
    #[prost(bool, tag = "10")]
    read_only: bool,
    #[prost(bool, tag = "11")]
    credential_free: bool,
    #[prost(bool, tag = "12")]
    prospective_storage_used: bool,
    #[prost(string, tag = "13")]
    provenance_digest: String,
}

fn composite_status_tag_v1(value: CompositeLearningEpochStatusV1) -> u32 {
    match value {
        CompositeLearningEpochStatusV1::ReadyNotAttempted => 1,
        CompositeLearningEpochStatusV1::EvidenceAcquired => 2,
        CompositeLearningEpochStatusV1::TerminalSegmentFailure => 3,
        CompositeLearningEpochStatusV1::TerminalPartialEvidence => 4,
        CompositeLearningEpochStatusV1::AlreadyTerminal => 5,
        CompositeLearningEpochStatusV1::RegistrationInvalid => 6,
        CompositeLearningEpochStatusV1::GapNoLongerCurrent => 7,
        CompositeLearningEpochStatusV1::MissingNetworkConsent => 8,
        CompositeLearningEpochStatusV1::IntegrityFailure => 9,
    }
}

fn composite_status_from_tag_v1(value: u32) -> Result<CompositeLearningEpochStatusV1, String> {
    match value {
        1 => Ok(CompositeLearningEpochStatusV1::ReadyNotAttempted),
        2 => Ok(CompositeLearningEpochStatusV1::EvidenceAcquired),
        3 => Ok(CompositeLearningEpochStatusV1::TerminalSegmentFailure),
        4 => Ok(CompositeLearningEpochStatusV1::TerminalPartialEvidence),
        5 => Ok(CompositeLearningEpochStatusV1::AlreadyTerminal),
        6 => Ok(CompositeLearningEpochStatusV1::RegistrationInvalid),
        7 => Ok(CompositeLearningEpochStatusV1::GapNoLongerCurrent),
        8 => Ok(CompositeLearningEpochStatusV1::MissingNetworkConsent),
        9 => Ok(CompositeLearningEpochStatusV1::IntegrityFailure),
        _ => Err("composite learning epoch status rejected".into()),
    }
}

fn segment_to_protobuf_v1(
    segment: &LearningEvidenceSegmentRegistrationV1,
) -> LearningEvidenceSegmentRegistrationProtobufV1 {
    LearningEvidenceSegmentRegistrationProtobufV1 {
        segment_index: segment.segment_index as u64,
        expected_timestamps: segment.expected_timestamps.clone(),
        expected_row_count: segment.expected_row_count as u64,
        request_to_utc: segment.request_to_utc.clone(),
        maximum_requests: segment.maximum_requests as u64,
        maximum_retries: segment.maximum_retries as u64,
        segment_digest: segment.segment_digest.clone(),
    }
}

fn segment_from_protobuf_v1(
    segment: LearningEvidenceSegmentRegistrationProtobufV1,
) -> Result<LearningEvidenceSegmentRegistrationV1, String> {
    Ok(LearningEvidenceSegmentRegistrationV1 {
        segment_index: usize::try_from(segment.segment_index)
            .map_err(|_| "composite learning segment index rejected")?,
        expected_timestamps: segment.expected_timestamps,
        expected_row_count: usize::try_from(segment.expected_row_count)
            .map_err(|_| "composite learning row count rejected")?,
        request_to_utc: segment.request_to_utc,
        maximum_requests: usize::try_from(segment.maximum_requests)
            .map_err(|_| "composite learning request count rejected")?,
        maximum_retries: usize::try_from(segment.maximum_retries)
            .map_err(|_| "composite learning retry count rejected")?,
        segment_digest: segment.segment_digest,
    })
}

pub fn encode_composite_learning_registration_protobuf_v1(
    registration: &CompositeLearningAcquisitionRegistrationV1,
    contract: &LearningEvidenceProviderContractV1,
) -> Result<Vec<u8>, String> {
    validate_composite_learning_acquisition_registration_v1(registration, contract)?;
    encode_learning_artifact_envelope_v1(
        COMPOSITE_REGISTRATION_ARTIFACT_KIND_V1,
        &registration.registration_digest,
        registration.protected_registration_digests.clone(),
        CompositeLearningRegistrationProtobufV1 {
            registration_version: registration.registration_version.clone(),
            target_agent_ids: registration.target_agent_ids.clone(),
            intent_digest: registration.intent_digest.clone(),
            gap_report_digest: registration.gap_report_digest.clone(),
            provider_contract_digest: registration.provider_contract_digest.clone(),
            dataset_kind: u32::from(dataset_kind_code_v0(registration.dataset_kind)),
            market_scope: u32::from(market_scope_code_v0(registration.market_scope)),
            symbols: registration.symbols.clone(),
            cadence: registration.cadence.clone(),
            information_cutoff_ms: registration.information_cutoff_ms,
            required_row_count: registration.required_row_count as u64,
            expected_timestamp_digest: registration.expected_timestamp_digest.clone(),
            segments: registration
                .segments
                .iter()
                .map(segment_to_protobuf_v1)
                .collect(),
            maximum_total_requests: registration.maximum_total_requests as u64,
            maximum_concurrency: registration.maximum_concurrency as u64,
            maximum_retries_per_segment: registration.maximum_retries_per_segment as u64,
            protected_registration_digests: registration.protected_registration_digests.clone(),
            excluded_timestamp_ms: registration.excluded_timestamp_ms.clone(),
            read_only_required: registration.read_only_required,
            credential_free_required: registration.credential_free_required,
            prospective_storage_forbidden: registration.prospective_storage_forbidden,
            registration_digest: registration.registration_digest.clone(),
        },
    )
}

pub fn decode_composite_learning_registration_protobuf_v1(
    bytes: &[u8],
) -> Result<CompositeLearningAcquisitionRegistrationV1, String> {
    let (semantic_digest, _, payload) =
        decode_learning_artifact_envelope_v1(bytes, COMPOSITE_REGISTRATION_ARTIFACT_KIND_V1)?;
    let value = CompositeLearningRegistrationProtobufV1::decode(payload.as_slice())
        .map_err(|_| "composite learning registration decode failed")?;
    let registration = CompositeLearningAcquisitionRegistrationV1 {
        registration_version: value.registration_version,
        target_agent_ids: value.target_agent_ids,
        intent_digest: value.intent_digest,
        gap_report_digest: value.gap_report_digest,
        provider_contract_digest: value.provider_contract_digest,
        dataset_kind: dataset_kind_from_code_v1(value.dataset_kind)?,
        market_scope: market_scope_from_code_v1(value.market_scope)?,
        symbols: value.symbols,
        cadence: value.cadence,
        information_cutoff_ms: value.information_cutoff_ms,
        required_row_count: usize::try_from(value.required_row_count)
            .map_err(|_| "composite learning row count rejected")?,
        expected_timestamp_digest: value.expected_timestamp_digest,
        segments: value
            .segments
            .into_iter()
            .map(segment_from_protobuf_v1)
            .collect::<Result<Vec<_>, _>>()?,
        maximum_total_requests: usize::try_from(value.maximum_total_requests)
            .map_err(|_| "composite learning request count rejected")?,
        maximum_concurrency: usize::try_from(value.maximum_concurrency)
            .map_err(|_| "composite learning concurrency rejected")?,
        maximum_retries_per_segment: usize::try_from(value.maximum_retries_per_segment)
            .map_err(|_| "composite learning retry count rejected")?,
        protected_registration_digests: value.protected_registration_digests,
        excluded_timestamp_ms: value.excluded_timestamp_ms,
        read_only_required: value.read_only_required,
        credential_free_required: value.credential_free_required,
        prospective_storage_forbidden: value.prospective_storage_forbidden,
        registration_digest: value.registration_digest,
    };
    if registration.registration_digest != semantic_digest
        || registration.registration_digest
            != composite_learning_registration_digest_v1(&registration)
    {
        return Err("composite learning registration identity rejected".into());
    }
    Ok(registration)
}

fn encode_segment_receipt_protobuf_v1(
    receipt: &LearningEvidenceSegmentReceiptV1,
) -> Result<Vec<u8>, String> {
    validate_learning_evidence_segment_receipt_v1(receipt)?;
    encode_learning_artifact_envelope_v1(
        COMPOSITE_SEGMENT_RECEIPT_ARTIFACT_KIND_V1,
        &receipt.receipt_digest,
        vec![
            receipt.composite_registration_digest.clone(),
            receipt.segment_digest.clone(),
        ],
        CompositeSegmentReceiptProtobufV1 {
            receipt_version: receipt.receipt_version.clone(),
            composite_registration_digest: receipt.composite_registration_digest.clone(),
            segment_digest: receipt.segment_digest.clone(),
            segment_index: receipt.segment_index as u64,
            request_attempted: receipt.request_attempted,
            request_count: receipt.request_count as u64,
            retry_count: receipt.retry_count as u64,
            status: receipt_status_tag_v1(receipt.status),
            http_status_class: receipt.http_status_class.clone(),
            returned_row_count: receipt.returned_row_count as u64,
            verified_row_count: receipt.verified_row_count as u64,
            raw_response_digest: receipt.raw_response_digest.clone(),
            segment_capsule_digest: receipt.segment_capsule_digest.clone(),
            receipt_digest: receipt.receipt_digest.clone(),
        },
    )
}

fn decode_segment_receipt_protobuf_v1(
    bytes: &[u8],
) -> Result<LearningEvidenceSegmentReceiptV1, String> {
    let (semantic_digest, _, payload) =
        decode_learning_artifact_envelope_v1(bytes, COMPOSITE_SEGMENT_RECEIPT_ARTIFACT_KIND_V1)?;
    let value = CompositeSegmentReceiptProtobufV1::decode(payload.as_slice())
        .map_err(|_| "composite segment receipt decode failed")?;
    let receipt = LearningEvidenceSegmentReceiptV1 {
        receipt_version: value.receipt_version,
        composite_registration_digest: value.composite_registration_digest,
        segment_digest: value.segment_digest,
        segment_index: usize::try_from(value.segment_index)
            .map_err(|_| "segment index rejected")?,
        request_attempted: value.request_attempted,
        request_count: usize::try_from(value.request_count)
            .map_err(|_| "request count rejected")?,
        retry_count: usize::try_from(value.retry_count).map_err(|_| "retry count rejected")?,
        status: receipt_status_from_tag_v1(value.status)?,
        http_status_class: value.http_status_class,
        returned_row_count: usize::try_from(value.returned_row_count)
            .map_err(|_| "row count rejected")?,
        verified_row_count: usize::try_from(value.verified_row_count)
            .map_err(|_| "row count rejected")?,
        raw_response_digest: value.raw_response_digest,
        segment_capsule_digest: value.segment_capsule_digest,
        receipt_digest: value.receipt_digest,
    };
    validate_learning_evidence_segment_receipt_v1(&receipt)?;
    if receipt.receipt_digest != semantic_digest {
        return Err("composite segment receipt identity rejected".into());
    }
    Ok(receipt)
}

fn encode_segment_capsule_protobuf_v1(
    capsule: &LearningEvidenceSegmentCapsuleV1,
    registration: &CompositeLearningAcquisitionRegistrationV1,
    contract: &LearningEvidenceProviderContractV1,
) -> Result<Vec<u8>, String> {
    validate_learning_evidence_segment_capsule_v1(capsule, registration, contract)?;
    encode_learning_artifact_envelope_v1(
        COMPOSITE_SEGMENT_CAPSULE_ARTIFACT_KIND_V1,
        &capsule.capsule_digest,
        vec![
            capsule.composite_registration_digest.clone(),
            capsule.segment_receipt_digest.clone(),
        ],
        CompositeSegmentCapsuleProtobufV1 {
            capsule_version: capsule.capsule_version.clone(),
            composite_registration_digest: capsule.composite_registration_digest.clone(),
            segment_digest: capsule.segment_digest.clone(),
            segment_receipt_digest: capsule.segment_receipt_digest.clone(),
            segment_index: capsule.segment_index as u64,
            provider_id: capsule.provider_id.clone(),
            symbol: capsule.symbol.clone(),
            cadence: capsule.cadence.clone(),
            expected_timestamps: capsule.expected_timestamps.clone(),
            rows: capsule
                .rows
                .iter()
                .map(|row| CompositeHistoricalRowProtobufV1 {
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
            segment_semantic_digest: capsule.segment_semantic_digest.clone(),
            finalized: capsule.finalized,
            read_only: capsule.read_only,
            credential_free: capsule.credential_free,
            capsule_digest: capsule.capsule_digest.clone(),
        },
    )
}

fn decode_segment_capsule_protobuf_v1(
    bytes: &[u8],
    registration: &CompositeLearningAcquisitionRegistrationV1,
    contract: &LearningEvidenceProviderContractV1,
) -> Result<LearningEvidenceSegmentCapsuleV1, String> {
    let (semantic_digest, _, payload) =
        decode_learning_artifact_envelope_v1(bytes, COMPOSITE_SEGMENT_CAPSULE_ARTIFACT_KIND_V1)?;
    let value = CompositeSegmentCapsuleProtobufV1::decode(payload.as_slice())
        .map_err(|_| "composite segment capsule decode failed")?;
    let capsule = LearningEvidenceSegmentCapsuleV1 {
        capsule_version: value.capsule_version,
        composite_registration_digest: value.composite_registration_digest,
        segment_digest: value.segment_digest,
        segment_receipt_digest: value.segment_receipt_digest,
        segment_index: usize::try_from(value.segment_index)
            .map_err(|_| "segment index rejected")?,
        provider_id: value.provider_id,
        symbol: value.symbol,
        cadence: value.cadence,
        expected_timestamps: value.expected_timestamps,
        rows: value
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
        segment_semantic_digest: value.segment_semantic_digest,
        finalized: value.finalized,
        read_only: value.read_only,
        credential_free: value.credential_free,
        capsule_digest: value.capsule_digest,
    };
    validate_learning_evidence_segment_capsule_v1(&capsule, registration, contract)?;
    if capsule.capsule_digest != semantic_digest {
        return Err("composite segment capsule identity rejected".into());
    }
    Ok(capsule)
}

fn encode_epoch_receipt_protobuf_v1(
    receipt: &CompositeLearningEpochReceiptV1,
    registration: &CompositeLearningAcquisitionRegistrationV1,
) -> Result<Vec<u8>, String> {
    validate_composite_learning_epoch_receipt_v1(receipt, registration)?;
    encode_learning_artifact_envelope_v1(
        COMPOSITE_EPOCH_RECEIPT_ARTIFACT_KIND_V1,
        &receipt.receipt_digest,
        vec![receipt.registration_digest.clone()],
        CompositeEpochReceiptProtobufV1 {
            receipt_version: receipt.receipt_version.clone(),
            registration_digest: receipt.registration_digest.clone(),
            segment_receipt_digests: receipt.segment_receipt_digests.clone(),
            attempted_segment_count: receipt.attempted_segment_count as u64,
            successful_segment_count: receipt.successful_segment_count as u64,
            request_count: receipt.request_count as u64,
            retry_count: receipt.retry_count as u64,
            merged_snapshot_digest: receipt.merged_snapshot_digest.clone(),
            merged_provenance_digest: receipt.merged_provenance_digest.clone(),
            status: composite_status_tag_v1(receipt.status),
            receipt_digest: receipt.receipt_digest.clone(),
        },
    )
}

fn decode_epoch_receipt_protobuf_v1(
    bytes: &[u8],
    registration: &CompositeLearningAcquisitionRegistrationV1,
) -> Result<CompositeLearningEpochReceiptV1, String> {
    let (semantic_digest, _, payload) =
        decode_learning_artifact_envelope_v1(bytes, COMPOSITE_EPOCH_RECEIPT_ARTIFACT_KIND_V1)?;
    let value = CompositeEpochReceiptProtobufV1::decode(payload.as_slice())
        .map_err(|_| "composite epoch receipt decode failed")?;
    let receipt = CompositeLearningEpochReceiptV1 {
        receipt_version: value.receipt_version,
        registration_digest: value.registration_digest,
        segment_receipt_digests: value.segment_receipt_digests,
        attempted_segment_count: usize::try_from(value.attempted_segment_count)
            .map_err(|_| "attempt count rejected")?,
        successful_segment_count: usize::try_from(value.successful_segment_count)
            .map_err(|_| "success count rejected")?,
        request_count: usize::try_from(value.request_count)
            .map_err(|_| "request count rejected")?,
        retry_count: usize::try_from(value.retry_count).map_err(|_| "retry count rejected")?,
        merged_snapshot_digest: value.merged_snapshot_digest,
        merged_provenance_digest: value.merged_provenance_digest,
        status: composite_status_from_tag_v1(value.status)?,
        receipt_digest: value.receipt_digest,
    };
    validate_composite_learning_epoch_receipt_v1(&receipt, registration)?;
    if receipt.receipt_digest != semantic_digest {
        return Err("composite epoch receipt identity rejected".into());
    }
    Ok(receipt)
}

fn validate_composite_merged_provenance_v1(
    provenance: &CompositeLearningMergedProvenanceV1,
    registration: &CompositeLearningAcquisitionRegistrationV1,
) -> Result<(), String> {
    if provenance.provenance_version != LEARNING_MERGED_PROVENANCE_VERSION_V1
        || provenance.registration_digest != registration.registration_digest
        || provenance.provider_contract_digest != registration.provider_contract_digest
        || provenance.segment_digests
            != registration
                .segments
                .iter()
                .map(|segment| segment.segment_digest.clone())
                .collect::<Vec<_>>()
        || provenance.segment_receipt_digests.len() != registration.segments.len()
        || provenance.segment_capsule_digests.len() != registration.segments.len()
        || provenance.canonical_snapshot_digest.is_empty()
        || provenance.expected_timestamp_digest != registration.expected_timestamp_digest
        || provenance.required_row_count != registration.required_row_count
        || !provenance.read_only
        || !provenance.credential_free
        || provenance.prospective_storage_used
        || provenance.provenance_digest != merged_provenance_digest_v1(provenance)
    {
        Err("composite learning provenance rejected".into())
    } else {
        Ok(())
    }
}

fn encode_merged_provenance_protobuf_v1(
    provenance: &CompositeLearningMergedProvenanceV1,
    registration: &CompositeLearningAcquisitionRegistrationV1,
) -> Result<Vec<u8>, String> {
    validate_composite_merged_provenance_v1(provenance, registration)?;
    encode_learning_artifact_envelope_v1(
        COMPOSITE_PROVENANCE_ARTIFACT_KIND_V1,
        &provenance.provenance_digest,
        provenance.segment_capsule_digests.clone(),
        CompositeMergedProvenanceProtobufV1 {
            provenance_version: provenance.provenance_version.clone(),
            registration_digest: provenance.registration_digest.clone(),
            provider_contract_digest: provenance.provider_contract_digest.clone(),
            segment_digests: provenance.segment_digests.clone(),
            segment_receipt_digests: provenance.segment_receipt_digests.clone(),
            segment_capsule_digests: provenance.segment_capsule_digests.clone(),
            canonical_snapshot_digest: provenance.canonical_snapshot_digest.clone(),
            expected_timestamp_digest: provenance.expected_timestamp_digest.clone(),
            required_row_count: provenance.required_row_count as u64,
            read_only: provenance.read_only,
            credential_free: provenance.credential_free,
            prospective_storage_used: provenance.prospective_storage_used,
            provenance_digest: provenance.provenance_digest.clone(),
        },
    )
}

fn decode_merged_provenance_protobuf_v1(
    bytes: &[u8],
    registration: &CompositeLearningAcquisitionRegistrationV1,
) -> Result<CompositeLearningMergedProvenanceV1, String> {
    let (semantic_digest, _, payload) =
        decode_learning_artifact_envelope_v1(bytes, COMPOSITE_PROVENANCE_ARTIFACT_KIND_V1)?;
    let value = CompositeMergedProvenanceProtobufV1::decode(payload.as_slice())
        .map_err(|_| "composite learning provenance decode failed")?;
    let provenance = CompositeLearningMergedProvenanceV1 {
        provenance_version: value.provenance_version,
        registration_digest: value.registration_digest,
        provider_contract_digest: value.provider_contract_digest,
        segment_digests: value.segment_digests,
        segment_receipt_digests: value.segment_receipt_digests,
        segment_capsule_digests: value.segment_capsule_digests,
        canonical_snapshot_digest: value.canonical_snapshot_digest,
        expected_timestamp_digest: value.expected_timestamp_digest,
        required_row_count: usize::try_from(value.required_row_count)
            .map_err(|_| "row count rejected")?,
        read_only: value.read_only,
        credential_free: value.credential_free,
        prospective_storage_used: value.prospective_storage_used,
        provenance_digest: value.provenance_digest,
    };
    validate_composite_merged_provenance_v1(&provenance, registration)?;
    if provenance.provenance_digest != semantic_digest {
        return Err("composite learning provenance identity rejected".into());
    }
    Ok(provenance)
}

pub fn write_and_verify_composite_learning_registration_v1(
    registration: &CompositeLearningAcquisitionRegistrationV1,
    contract: &LearningEvidenceProviderContractV1,
    root: &Path,
) -> Result<LearningEvidenceArtifactWriteStatusV1, String> {
    let bytes = encode_composite_learning_registration_protobuf_v1(registration, contract)?;
    let path = root.join("acquisition_v1/composite/registration.pb");
    write_learning_evidence_artifact_v1(&path, &bytes, &registration.registration_digest, |bytes| {
        Ok(decode_composite_learning_registration_protobuf_v1(bytes)?.registration_digest)
    })
}

pub fn read_composite_learning_registration_v1(
    root: &Path,
) -> Result<Option<CompositeLearningAcquisitionRegistrationV1>, String> {
    let path = root.join("acquisition_v1/composite/registration.pb");
    path.is_file()
        .then(|| {
            decode_composite_learning_registration_protobuf_v1(
                &fs::read(path).map_err(|_| "composite learning registration unavailable")?,
            )
        })
        .transpose()
}

pub fn write_and_verify_composite_segment_receipt_v1(
    receipt: &LearningEvidenceSegmentReceiptV1,
    root: &Path,
) -> Result<LearningEvidenceArtifactWriteStatusV1, String> {
    let bytes = encode_segment_receipt_protobuf_v1(receipt)?;
    let path = root.join(format!(
        "acquisition_v1/composite/segment-{}-receipt.pb",
        receipt.segment_index
    ));
    write_learning_evidence_artifact_v1(&path, &bytes, &receipt.receipt_digest, |bytes| {
        Ok(decode_segment_receipt_protobuf_v1(bytes)?.receipt_digest)
    })
}

pub fn write_and_verify_composite_segment_capsule_v1(
    capsule: &LearningEvidenceSegmentCapsuleV1,
    registration: &CompositeLearningAcquisitionRegistrationV1,
    contract: &LearningEvidenceProviderContractV1,
    root: &Path,
) -> Result<LearningEvidenceArtifactWriteStatusV1, String> {
    let bytes = encode_segment_capsule_protobuf_v1(capsule, registration, contract)?;
    let path = root.join(format!(
        "acquisition_v1/composite/segment-{}-capsule.pb",
        capsule.segment_index
    ));
    write_learning_evidence_artifact_v1(&path, &bytes, &capsule.capsule_digest, |bytes| {
        Ok(decode_segment_capsule_protobuf_v1(bytes, registration, contract)?.capsule_digest)
    })
}

pub fn write_and_verify_composite_epoch_receipt_v1(
    receipt: &CompositeLearningEpochReceiptV1,
    registration: &CompositeLearningAcquisitionRegistrationV1,
    root: &Path,
) -> Result<LearningEvidenceArtifactWriteStatusV1, String> {
    let bytes = encode_epoch_receipt_protobuf_v1(receipt, registration)?;
    let path = root.join("acquisition_v1/composite/epoch-receipt.pb");
    write_learning_evidence_artifact_v1(&path, &bytes, &receipt.receipt_digest, |bytes| {
        Ok(decode_epoch_receipt_protobuf_v1(bytes, registration)?.receipt_digest)
    })
}

pub fn read_composite_epoch_receipt_v1(
    registration: &CompositeLearningAcquisitionRegistrationV1,
    root: &Path,
) -> Result<Option<CompositeLearningEpochReceiptV1>, String> {
    let path = root.join("acquisition_v1/composite/epoch-receipt.pb");
    path.is_file()
        .then(|| {
            decode_epoch_receipt_protobuf_v1(
                &fs::read(path).map_err(|_| "composite epoch receipt unavailable")?,
                registration,
            )
        })
        .transpose()
}

pub fn write_and_verify_composite_merged_provenance_v1(
    provenance: &CompositeLearningMergedProvenanceV1,
    registration: &CompositeLearningAcquisitionRegistrationV1,
    root: &Path,
) -> Result<LearningEvidenceArtifactWriteStatusV1, String> {
    let bytes = encode_merged_provenance_protobuf_v1(provenance, registration)?;
    let path = root.join("acquisition_v1/composite/merged-provenance.pb");
    write_learning_evidence_artifact_v1(&path, &bytes, &provenance.provenance_digest, |bytes| {
        Ok(decode_merged_provenance_protobuf_v1(bytes, registration)?.provenance_digest)
    })
}

pub fn write_and_verify_learning_raw_response_v1(
    raw_response: &[u8],
    expected_digest: &str,
    root: &Path,
) -> Result<LearningEvidenceArtifactWriteStatusV1, String> {
    let path = root
        .join("acquisition_v1")
        .join("raw")
        .join(format!("{expected_digest}.json"));
    if !safe_learning_data_path_v0(&path)
        || canonical_hash_hex(raw_response) != expected_digest
        || !raw_learning_response_is_sanitized_v1(raw_response)
    {
        return Err("learning raw response rejected".into());
    }
    if path.is_file() {
        return if canonical_hash_hex(
            &fs::read(path).map_err(|_| "learning raw response reopen failed".to_string())?,
        ) == expected_digest
        {
            Ok(LearningEvidenceArtifactWriteStatusV1::DuplicateRejected)
        } else {
            Err("learning raw response identity collision".into())
        };
    }
    let parent = path
        .parent()
        .ok_or_else(|| "learning raw response parent missing".to_string())?;
    fs::create_dir_all(parent)
        .map_err(|_| "learning raw response directory unavailable".to_string())?;
    let temporary = parent.join(format!(".{expected_digest}.{}.tmp", std::process::id()));
    let write_result = (|| {
        let mut file = File::options()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .map_err(|_| "learning raw response temporary create failed".to_string())?;
        file.write_all(raw_response)
            .map_err(|_| "learning raw response write failed".to_string())?;
        file.flush()
            .map_err(|_| "learning raw response flush failed".to_string())?;
        file.sync_all()
            .map_err(|_| "learning raw response sync failed".to_string())?;
        drop(file);
        let temporary_bytes = fs::read(&temporary)
            .map_err(|_| "learning raw response temporary reopen failed".to_string())?;
        if canonical_hash_hex(&temporary_bytes) != expected_digest {
            return Err("learning raw response temporary verification failed".into());
        }
        fs::rename(&temporary, &path)
            .map_err(|_| "learning raw response atomic rename failed".to_string())?;
        let final_bytes =
            fs::read(&path).map_err(|_| "learning raw response final reopen failed".to_string())?;
        if canonical_hash_hex(&final_bytes) != expected_digest {
            return Err("learning raw response final verification failed".into());
        }
        Ok(LearningEvidenceArtifactWriteStatusV1::Written)
    })();
    if write_result.is_err() && temporary.is_file() {
        let _ = fs::remove_file(temporary);
    }
    write_result
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

    const LEARNING_V1_CUTOFF_MS: u64 = 1_780_000_000_000;

    fn learning_v1_intents() -> Vec<AgentLearningIntentV0> {
        let mut intents = derive_active_agent_learning_intents_v0(
            &canonical_current_agent_states(),
            &universe(),
            &default_agent_data_policies(),
            LEARNING_V1_CUTOFF_MS,
        )
        .unwrap();
        let momentum = intents
            .iter_mut()
            .find(|intent| intent.agent_kind == AgentKind::MomentumTrendFast)
            .unwrap();
        momentum.market_scopes = vec![AcquisitionMarketScope::BtcCrypto];
        momentum.symbols = vec!["KRW-BTC".into()];
        stabilize_learning_intent_v0(momentum);
        momentum.intent_digest = agent_learning_intent_digest_v0(momentum);
        intents
    }

    fn learning_v1_contract() -> LearningEvidenceProviderContractV1 {
        seal_learning_evidence_provider_contract_v1(LearningEvidenceProviderContractV1 {
            contract_version: LEARNING_EVIDENCE_PROVIDER_CONTRACT_VERSION_V1.into(),
            provider_id: "upbit".into(),
            dataset_kind: DatasetKind::DailyOhlcv,
            market_scope: AcquisitionMarketScope::BtcCrypto,
            symbols: vec!["KRW-BTC".into()],
            cadence: "1d".into(),
            maximum_lookback_bars: 200,
            earliest_timestamp_ms: LEARNING_V1_CUTOFF_MS - 300 * DAILY_CADENCE_MS_V1,
            latest_exclusive_timestamp_ms: LEARNING_V1_CUTOFF_MS + DAILY_CADENCE_MS_V1,
            maximum_response_bytes: 262_144,
            credential_free: true,
            read_only: true,
            approved_for_network: true,
            all_rows_finalized: true,
            enabled: true,
            contract_digest: String::new(),
        })
        .unwrap()
    }

    fn learning_v1_trainers() -> BTreeSet<String> {
        BTreeSet::from([
            "momentum_trend_fast".to_string(),
            "cycle_risk_skeptic".to_string(),
        ])
    }

    fn learning_v1_gap_report(snapshots: &[DataSnapshot]) -> AgentCanonicalViewGapReportV1 {
        derive_agent_canonical_view_gaps_v1(
            &learning_v1_intents(),
            &default_agent_data_policies(),
            snapshots,
            &learning_v1_trainers(),
            &[learning_v1_contract()],
        )
        .unwrap()
    }

    fn learning_v1_registration() -> LearningEvidenceAcquisitionRegistrationV1 {
        select_learning_evidence_acquisition_registration_v1(
            &learning_v1_gap_report(&[]),
            &[learning_v1_contract()],
            &["opening-registration".into()],
            &[
                LEARNING_V1_CUTOFF_MS + DAILY_CADENCE_MS_V1,
                LEARNING_V1_CUTOFF_MS + 2 * DAILY_CADENCE_MS_V1,
                LEARNING_V1_CUTOFF_MS + 3 * DAILY_CADENCE_MS_V1,
                LEARNING_V1_CUTOFF_MS + 4 * DAILY_CADENCE_MS_V1,
            ],
        )
        .unwrap()
        .unwrap()
    }

    fn learning_v1_snapshot(intent: &AgentLearningIntentV0) -> DataSnapshot {
        let timestamps = expected_learning_timestamps_v1(&intent.lookback).unwrap();
        let rows = timestamps
            .iter()
            .enumerate()
            .map(|(index, timestamp_ms)| {
                let close = 100.0 + index as f64;
                HistoricalOhlcvRow {
                    symbol: "KRW-BTC".into(),
                    timestamp_ms: *timestamp_ms,
                    open: close,
                    high: close + 2.0,
                    low: close - 2.0,
                    close: close + 1.0,
                    volume: 10.0,
                    trade_value: Some(1_000.0),
                }
            })
            .collect::<Vec<_>>();
        let normalized_dataset = HistoricalReplayDataset {
            symbol: "KRW-BTC".into(),
            source: "upbit-approved-readonly-daily".into(),
            rows,
            reason_codes: vec![],
        };
        let content_digest = historical_replay_dataset_digest_v0(&normalized_dataset);
        DataSnapshot {
            snapshot_id: snapshot_id_from_semantic_digest_v1(&content_digest),
            request_key: "learning-v1-fixture".into(),
            provider_id: "upbit".into(),
            dataset_kind: DatasetKind::DailyOhlcv,
            market_scope: AcquisitionMarketScope::BtcCrypto,
            symbols: vec!["KRW-BTC".into()],
            requested_lookback: intent.lookback.clone(),
            actual_start_timestamp_ms: timestamps.first().copied(),
            actual_end_timestamp_ms: timestamps.last().copied(),
            fetched_at_ms: LEARNING_V1_CUTOFF_MS,
            normalized_at_ms: LEARNING_V1_CUTOFF_MS,
            schema_version: 1,
            row_count: normalized_dataset.rows.len(),
            quality_summary: SnapshotQualitySummary {
                accepted: true,
                row_count: normalized_dataset.rows.len(),
                reason_codes: vec![],
            },
            content_digest,
            sanitized: true,
            read_only: true,
            compatibility: Some(SnapshotCompatibilityV1 {
                cadence: "1d".into(),
                adjustment_semantics: SnapshotAdjustmentSemanticsV1::Unadjusted,
                source_schema: "application/x-soma-normalized-dataset".into(),
                requested_cutoff_timestamp_ms: Some(intent.information_cutoff_ms),
                maximum_staleness_ms: intent.maximum_staleness_ms,
                all_rows_finalized: true,
            }),
            normalized_dataset,
            provenance: SnapshotProvenance {
                provider_id: "upbit".into(),
                acquisition_request_id: "learning-v1-fixture".into(),
                fetch_receipt_id: "learning-v1-fixture-receipt".into(),
                source_type: SnapshotSourceType::ApprovedReadOnlyProvider,
                sanitized: true,
                credential_free: true,
                reason_codes: vec![],
            },
            reason_codes: vec![],
        }
    }

    fn learning_v1_transport_response(
        request: &ReadOnlyProviderRequest,
    ) -> LearningEvidenceTransportResponseV1 {
        let momentum = learning_v1_intents()
            .into_iter()
            .find(|intent| intent.agent_kind == AgentKind::MomentumTrendFast)
            .unwrap();
        let snapshot = learning_v1_snapshot(&momentum);
        let raw_response = b"[{\"bounded\":true}]".to_vec();
        LearningEvidenceTransportResponseV1 {
            http_status_class: "2xx".into(),
            raw_response: raw_response.clone(),
            response: ReadOnlyProviderResponse {
                request_id: request.request_id.clone(),
                provider_id: request.provider_id.clone(),
                fetched_at_ms: LEARNING_V1_CUTOFF_MS,
                content_type: "application/x-soma-normalized-dataset".into(),
                all_rows_finalized: true,
                normalized_dataset: snapshot.normalized_dataset,
                reported_content_bytes: raw_response.len(),
                reason_codes: vec![],
            },
        }
    }

    #[test]
    fn v1_gap_report_uses_exact_policy_required_datasets() {
        let report = learning_v1_gap_report(&[]);
        let momentum = report
            .gaps
            .iter()
            .find(|gap| gap.agent_id == "momentum_trend_fast")
            .unwrap();
        let risk = report
            .gaps
            .iter()
            .find(|gap| gap.agent_id == "cycle_risk_skeptic")
            .unwrap();
        assert_eq!(momentum.required_dataset_kinds, [DatasetKind::DailyOhlcv]);
        assert_eq!(
            risk.required_dataset_kinds,
            [DatasetKind::MarketIndexDaily, DatasetKind::VolatilityDaily]
        );
    }

    #[test]
    fn v1_gap_report_distinguishes_optional_from_required() {
        let report = learning_v1_gap_report(&[]);
        for gap in report.gaps {
            assert_eq!(
                gap.missing_required_dataset_kinds,
                gap.required_dataset_kinds
            );
            assert_eq!(
                gap.missing_optional_dataset_kinds,
                gap.optional_dataset_kinds
            );
        }
    }

    #[test]
    fn v1_value_trainer_unavailable_is_not_selected() {
        let report = learning_v1_gap_report(&[]);
        assert_eq!(
            report
                .gaps
                .iter()
                .find(|gap| gap.agent_id == "value_quality_filter")
                .unwrap()
                .status,
            CanonicalViewGapStatusV1::TrainerUnavailable
        );
        assert_eq!(
            learning_v1_registration().target_agent_ids,
            ["momentum_trend_fast"]
        );
    }

    #[test]
    fn v1_cycle_without_exact_provider_contract_is_explicit() {
        let report = learning_v1_gap_report(&[]);
        assert_eq!(
            report
                .gaps
                .iter()
                .find(|gap| gap.agent_id == "cycle_risk_skeptic")
                .unwrap()
                .status,
            CanonicalViewGapStatusV1::ProviderContractUnverified
        );
    }

    #[test]
    fn v1_daily_intent_and_daily_provider_are_not_incompatible_cadence() {
        let report = learning_v1_gap_report(&[]);
        let gap = report
            .gaps
            .iter()
            .find(|gap| gap.agent_id == "momentum_trend_fast")
            .unwrap();
        assert_eq!(
            gap.status,
            CanonicalViewGapStatusV1::MissingRequiredEvidence
        );
        assert_ne!(gap.status, CanonicalViewGapStatusV1::IncompatibleCadence);
    }

    #[test]
    fn v1_312_rows_against_200_row_limit_requires_segmentation() {
        let mut intents = learning_v1_intents();
        let momentum = intents
            .iter_mut()
            .find(|intent| intent.agent_kind == AgentKind::MomentumTrendFast)
            .unwrap();
        momentum.lookback = DataLookback {
            bars: 312,
            start_timestamp_ms: Some(LEARNING_V1_CUTOFF_MS - 311 * DAILY_CADENCE_MS_V1),
            end_timestamp_ms: Some(LEARNING_V1_CUTOFF_MS),
        };
        stabilize_learning_intent_v0(momentum);
        momentum.intent_digest = agent_learning_intent_digest_v0(momentum);
        let mut contract = learning_v1_contract();
        contract.earliest_timestamp_ms = LEARNING_V1_CUTOFF_MS - 400 * DAILY_CADENCE_MS_V1;
        contract.contract_digest.clear();
        let contract = seal_learning_evidence_provider_contract_v1(contract).unwrap();
        let report = derive_agent_canonical_view_gaps_v1(
            &intents,
            &default_agent_data_policies(),
            &[],
            &learning_v1_trainers(),
            &[contract],
        )
        .unwrap();
        let gap = report
            .gaps
            .iter()
            .find(|gap| gap.agent_id == "momentum_trend_fast")
            .unwrap();
        assert_eq!(
            gap.status,
            CanonicalViewGapStatusV1::SegmentedAcquisitionRequired
        );
        assert_ne!(gap.status, CanonicalViewGapStatusV1::IncompatibleCadence);
        assert_eq!(gap.lookback.bars, 312);
        assert_eq!(gap.authorized_provider_ids, ["upbit"]);
        assert_eq!(
            decode_agent_canonical_view_gap_report_protobuf_v1(
                &encode_agent_canonical_view_gap_report_protobuf_v1(&report).unwrap(),
            )
            .unwrap(),
            report
        );
    }

    #[test]
    fn v1_non_segment_provider_reports_single_request_capacity() {
        let mut contract = learning_v1_contract();
        contract.provider_id = "bounded-daily-provider".into();
        contract.maximum_lookback_bars = 50;
        contract.contract_digest.clear();
        let contract = seal_learning_evidence_provider_contract_v1(contract).unwrap();
        let report = derive_agent_canonical_view_gaps_v1(
            &learning_v1_intents(),
            &default_agent_data_policies(),
            &[],
            &learning_v1_trainers(),
            &[contract],
        )
        .unwrap();
        assert_eq!(
            report
                .gaps
                .iter()
                .find(|gap| gap.agent_id == "momentum_trend_fast")
                .unwrap()
                .status,
            CanonicalViewGapStatusV1::ProviderSingleRequestCapacityExceeded
        );
    }

    #[test]
    fn v1_actual_minute_daily_provider_mismatch_is_incompatible_cadence() {
        let mut intents = learning_v1_intents();
        let momentum = intents
            .iter_mut()
            .find(|intent| intent.agent_kind == AgentKind::MomentumTrendFast)
            .unwrap();
        momentum.cadence = "1m".into();
        stabilize_learning_intent_v0(momentum);
        momentum.intent_digest = agent_learning_intent_digest_v0(momentum);
        let report = derive_agent_canonical_view_gaps_v1(
            &intents,
            &default_agent_data_policies(),
            &[],
            &learning_v1_trainers(),
            &[learning_v1_contract()],
        )
        .unwrap();
        assert_eq!(
            report
                .gaps
                .iter()
                .find(|gap| gap.agent_id == "momentum_trend_fast")
                .unwrap()
                .status,
            CanonicalViewGapStatusV1::IncompatibleCadence
        );
    }

    #[test]
    fn v1_unbounded_or_response_dependent_segment_plan_rejects() {
        let unbounded = DataLookback {
            bars: 312,
            start_timestamp_ms: None,
            end_timestamp_ms: None,
        };
        assert!(exact_bounded_segment_count_v1(&unbounded, "1d", 200, 2, false).is_err());
        let bounded = DataLookback {
            bars: 312,
            start_timestamp_ms: None,
            end_timestamp_ms: Some(LEARNING_V1_CUTOFF_MS),
        };
        assert!(exact_bounded_segment_count_v1(&bounded, "1d", 200, 2, true).is_err());
    }

    #[test]
    fn v1_segment_plan_over_approved_cap_is_unsupported() {
        let mut intents = learning_v1_intents();
        let momentum = intents
            .iter_mut()
            .find(|intent| intent.agent_kind == AgentKind::MomentumTrendFast)
            .unwrap();
        momentum.lookback = DataLookback {
            bars: 401,
            start_timestamp_ms: None,
            end_timestamp_ms: Some(LEARNING_V1_CUTOFF_MS),
        };
        stabilize_learning_intent_v0(momentum);
        momentum.intent_digest = agent_learning_intent_digest_v0(momentum);
        let mut contract = learning_v1_contract();
        contract.earliest_timestamp_ms = LEARNING_V1_CUTOFF_MS - 500 * DAILY_CADENCE_MS_V1;
        contract.contract_digest.clear();
        let contract = seal_learning_evidence_provider_contract_v1(contract).unwrap();
        let report = derive_agent_canonical_view_gaps_v1(
            &intents,
            &default_agent_data_policies(),
            &[],
            &learning_v1_trainers(),
            &[contract],
        )
        .unwrap();
        assert_eq!(
            report
                .gaps
                .iter()
                .find(|gap| gap.agent_id == "momentum_trend_fast")
                .unwrap()
                .status,
            CanonicalViewGapStatusV1::SegmentedAcquisitionUnsupported
        );
    }

    #[test]
    fn v1_upbit_ohlcv_cannot_masquerade_as_volatility_or_index() {
        for dataset_kind in [DatasetKind::VolatilityDaily, DatasetKind::MarketIndexDaily] {
            let mut contract = learning_v1_contract();
            contract.dataset_kind = dataset_kind;
            contract.contract_digest.clear();
            assert!(seal_learning_evidence_provider_contract_v1(contract).is_err());
        }
    }

    #[test]
    fn v1_equivalent_gaps_deduplicate_to_one_request() {
        let mut report = learning_v1_gap_report(&[]);
        let mut duplicate = report
            .gaps
            .iter()
            .find(|gap| gap.agent_id == "momentum_trend_fast")
            .unwrap()
            .clone();
        duplicate.agent_id = "momentum_trend_second".into();
        duplicate.intent_digest = stable_hash_string("second-momentum-intent");
        duplicate.gap_digest = canonical_view_gap_digest_v1(&duplicate);
        report.gaps.push(duplicate);
        report
            .gaps
            .sort_by(|left, right| left.agent_id.cmp(&right.agent_id));
        report.report_digest = canonical_view_gap_report_digest_v1(&report);
        let registration = select_learning_evidence_acquisition_registration_v1(
            &report,
            &[learning_v1_contract()],
            &["protected".into()],
            &[LEARNING_V1_CUTOFF_MS + DAILY_CADENCE_MS_V1],
        )
        .unwrap()
        .unwrap();
        assert_eq!(registration.maximum_requests, 1);
        assert_eq!(registration.target_agent_ids.len(), 2);
    }

    #[test]
    fn v1_selected_range_stays_at_or_before_cutoff_and_excludes_prospective() {
        let registration = learning_v1_registration();
        assert!(
            registration
                .expected_timestamp_ms
                .iter()
                .all(|timestamp| *timestamp <= registration.information_cutoff_ms)
        );
        assert!(
            registration
                .expected_timestamp_ms
                .iter()
                .all(|timestamp| { !registration.excluded_timestamp_ms.contains(timestamp) })
        );
    }

    #[test]
    fn v1_registration_freezes_one_request_no_retry_contract() {
        let registration = learning_v1_registration();
        assert_eq!(registration.maximum_requests, 1);
        assert_eq!(registration.maximum_concurrency, 1);
        assert_eq!(registration.maximum_retries, 0);
        assert!(registration.credential_free_required);
        assert!(registration.read_only_required);
        assert!(registration.prospective_storage_forbidden);
    }

    #[test]
    fn v1_missing_consent_constructs_no_transport() {
        let registration = learning_v1_registration();
        let calls = std::cell::Cell::new(0);
        let result = execute_learning_evidence_acquisition_v1(
            &registration,
            &learning_v1_contract(),
            &registration.gap_report_digests,
            None,
            &[],
            false,
            |_| {
                calls.set(calls.get() + 1);
                Err(LearningEvidenceTransportFailureV1::Technical)
            },
        );
        assert_eq!(calls.get(), 0);
        assert_eq!(
            result.status,
            LearningEvidenceRequestStatusV1::MissingNetworkConsent
        );
        assert_eq!(result.safety_counters.transport_constructions, 0);
    }

    #[test]
    fn v1_one_attempt_consumes_budget() {
        let registration = learning_v1_registration();
        let first = execute_learning_evidence_acquisition_v1(
            &registration,
            &learning_v1_contract(),
            &registration.gap_report_digests,
            None,
            &[],
            true,
            |request| Ok(learning_v1_transport_response(request)),
        );
        let receipt = first.receipt.as_ref().unwrap();
        let calls = std::cell::Cell::new(0);
        let replay = execute_learning_evidence_acquisition_v1(
            &registration,
            &learning_v1_contract(),
            &registration.gap_report_digests,
            Some(receipt),
            &[],
            true,
            |_| {
                calls.set(calls.get() + 1);
                Err(LearningEvidenceTransportFailureV1::Technical)
            },
        );
        assert_eq!(calls.get(), 0);
        assert_eq!(
            replay.status,
            LearningEvidenceRequestStatusV1::RequestBudgetExhausted
        );
    }

    #[test]
    fn v1_failure_is_terminal_and_never_retries() {
        let registration = learning_v1_registration();
        let calls = std::cell::Cell::new(0);
        let result = execute_learning_evidence_acquisition_v1(
            &registration,
            &learning_v1_contract(),
            &registration.gap_report_digests,
            None,
            &[],
            true,
            |_| {
                calls.set(calls.get() + 1);
                Err(LearningEvidenceTransportFailureV1::TimedOut)
            },
        );
        assert_eq!(calls.get(), 1);
        assert_eq!(
            result.status,
            LearningEvidenceRequestStatusV1::TimeoutNoRetry
        );
        assert_eq!(result.receipt.as_ref().unwrap().request_count, 1);
        assert_eq!(result.receipt.as_ref().unwrap().retry_count, 0);
    }

    #[test]
    fn v1_valid_response_creates_canonical_snapshot_and_manifest() {
        let registration = learning_v1_registration();
        let result = execute_learning_evidence_acquisition_v1(
            &registration,
            &learning_v1_contract(),
            &registration.gap_report_digests,
            None,
            &[],
            true,
            |request| Ok(learning_v1_transport_response(request)),
        );
        assert_eq!(
            result.status,
            LearningEvidenceRequestStatusV1::EvidenceAcquired
        );
        let snapshot = result.snapshot.unwrap();
        assert_eq!(snapshot.dataset_kind, DatasetKind::DailyOhlcv);
        assert_eq!(snapshot.market_scope, AcquisitionMarketScope::BtcCrypto);
        assert_eq!(snapshot.row_count, registration.lookback.bars);
        assert!(result.provenance_manifest.is_some());
    }

    #[test]
    fn v1_duplicate_snapshot_rejects_before_transport() {
        let registration = learning_v1_registration();
        let momentum = learning_v1_intents()
            .into_iter()
            .find(|intent| intent.agent_kind == AgentKind::MomentumTrendFast)
            .unwrap();
        let snapshot = learning_v1_snapshot(&momentum);
        let calls = std::cell::Cell::new(0);
        let result = execute_learning_evidence_acquisition_v1(
            &registration,
            &learning_v1_contract(),
            &registration.gap_report_digests,
            None,
            &[snapshot],
            true,
            |_| {
                calls.set(calls.get() + 1);
                Err(LearningEvidenceTransportFailureV1::Technical)
            },
        );
        assert_eq!(calls.get(), 0);
        assert_eq!(
            result.status,
            LearningEvidenceRequestStatusV1::EquivalentSnapshotExists
        );
    }

    #[test]
    fn v1_post_transport_duplicate_still_consumes_request_budget() {
        let registration = learning_v1_registration();
        let momentum = learning_v1_intents()
            .into_iter()
            .find(|intent| intent.agent_kind == AgentKind::MomentumTrendFast)
            .unwrap();
        let mut noncanonical_existing = learning_v1_snapshot(&momentum);
        noncanonical_existing.quality_summary.accepted = false;
        let result = execute_learning_evidence_acquisition_v1(
            &registration,
            &learning_v1_contract(),
            &registration.gap_report_digests,
            None,
            &[noncanonical_existing],
            true,
            |request| Ok(learning_v1_transport_response(request)),
        );
        assert_eq!(
            result.status,
            LearningEvidenceRequestStatusV1::EquivalentSnapshotExists
        );
        assert_eq!(result.safety_counters.request_attempts, 1);
        assert_eq!(result.safety_counters.transport_constructions, 1);
        let receipt = result.receipt.unwrap();
        assert_eq!(receipt.request_count, 1);
        assert_eq!(receipt.retry_count, 0);
        validate_learning_evidence_request_receipt_v1(&receipt).unwrap();
    }

    #[test]
    fn v1_unsafe_rejection_body_is_dropped_but_attempt_is_receipted() {
        let registration = learning_v1_registration();
        let result = execute_learning_evidence_acquisition_v1(
            &registration,
            &learning_v1_contract(),
            &registration.gap_report_digests,
            None,
            &[],
            true,
            |_| {
                Err(LearningEvidenceTransportFailureV1::ProviderRejected {
                    http_status_class: Some("4xx".into()),
                    raw_response: Some(b"<html>authorization failed</html>".to_vec()),
                })
            },
        );
        assert_eq!(
            result.status,
            LearningEvidenceRequestStatusV1::ProviderRejected
        );
        assert!(result.raw_response.is_none());
        let receipt = result.receipt.unwrap();
        assert_eq!(receipt.request_count, 1);
        assert!(receipt.raw_response_digest.is_none());
        validate_learning_evidence_request_receipt_v1(&receipt).unwrap();
    }

    #[test]
    fn v1_snapshot_completes_only_authorized_momentum_view() {
        let intents = learning_v1_intents();
        let momentum = intents
            .iter()
            .find(|intent| intent.agent_kind == AgentKind::MomentumTrendFast)
            .unwrap();
        let report = learning_v1_gap_report(&[learning_v1_snapshot(momentum)]);
        assert!(
            report
                .gaps
                .iter()
                .find(|gap| gap.agent_id == "momentum_trend_fast")
                .unwrap()
                .missing_required_dataset_kinds
                .is_empty()
        );
        assert_eq!(
            report
                .gaps
                .iter()
                .find(|gap| gap.agent_id == "cycle_risk_skeptic")
                .unwrap()
                .status,
            CanonicalViewGapStatusV1::ProviderContractUnverified
        );
    }

    #[test]
    fn v1_snapshot_accepts_stricter_staleness_and_rejects_policy_overrun() {
        let intents = learning_v1_intents();
        let momentum = intents
            .iter()
            .find(|intent| intent.agent_kind == AgentKind::MomentumTrendFast)
            .unwrap();
        let mut fresher = learning_v1_snapshot(momentum);
        fresher.compatibility.as_mut().unwrap().maximum_staleness_ms = 0;
        let request = ReadOnlyProviderRequest {
            request_id: "stricter-staleness".into(),
            request_key: "stricter-staleness".into(),
            provider_id: fresher.provider_id.clone(),
            dataset_kind: fresher.dataset_kind,
            market_scope: fresher.market_scope,
            symbols: fresher.symbols.clone(),
            lookback: fresher.requested_lookback.clone(),
            cadence: "1d".into(),
            max_staleness_ms: momentum.maximum_staleness_ms,
            reason_codes: vec![],
        };
        assert!(snapshot_is_compatible_fallback(&fresher, &request));
        let report = learning_v1_gap_report(&[fresher]);
        assert!(
            report
                .gaps
                .iter()
                .find(|gap| gap.agent_id == "momentum_trend_fast")
                .unwrap()
                .missing_required_dataset_kinds
                .is_empty()
        );

        let mut stale = learning_v1_snapshot(momentum);
        stale.compatibility.as_mut().unwrap().maximum_staleness_ms =
            momentum.maximum_staleness_ms + 1;
        assert!(!snapshot_is_compatible_fallback(&stale, &request));
        let report = learning_v1_gap_report(&[stale]);
        assert_eq!(
            report
                .gaps
                .iter()
                .find(|gap| gap.agent_id == "momentum_trend_fast")
                .unwrap()
                .missing_required_dataset_kinds,
            [DatasetKind::DailyOhlcv]
        );
    }

    #[test]
    fn v1_gap_and_registration_protobuf_round_trip_and_corruption_reject() {
        let gap = learning_v1_gap_report(&[]);
        let gap_bytes = encode_agent_canonical_view_gap_report_protobuf_v1(&gap).unwrap();
        assert_eq!(
            decode_agent_canonical_view_gap_report_protobuf_v1(&gap_bytes).unwrap(),
            gap
        );
        let registration = learning_v1_registration();
        let registration_bytes =
            encode_learning_evidence_registration_protobuf_v1(&registration).unwrap();
        assert_eq!(
            decode_learning_evidence_registration_protobuf_v1(&registration_bytes).unwrap(),
            registration
        );
        let mut corrupt = registration_bytes;
        let last = corrupt.len() - 1;
        corrupt[last] ^= 0xff;
        assert!(decode_learning_evidence_registration_protobuf_v1(&corrupt).is_err());
    }

    #[test]
    fn v1_manifest_and_receipt_protobuf_round_trip_and_corruption_reject() {
        let registration = learning_v1_registration();
        let result = execute_learning_evidence_acquisition_v1(
            &registration,
            &learning_v1_contract(),
            &registration.gap_report_digests,
            None,
            &[],
            true,
            |request| Ok(learning_v1_transport_response(request)),
        );
        let manifest = result.provenance_manifest.unwrap();
        let manifest_bytes = encode_learning_evidence_provenance_protobuf_v1(&manifest).unwrap();
        assert_eq!(
            decode_learning_evidence_provenance_protobuf_v1(&manifest_bytes).unwrap(),
            manifest
        );
        let receipt = result.receipt.unwrap();
        let mut receipt_bytes = encode_learning_evidence_receipt_protobuf_v1(&receipt).unwrap();
        assert_eq!(
            decode_learning_evidence_receipt_protobuf_v1(&receipt_bytes).unwrap(),
            receipt
        );
        let last = receipt_bytes.len() - 1;
        receipt_bytes[last] ^= 0xff;
        assert!(decode_learning_evidence_receipt_protobuf_v1(&receipt_bytes).is_err());
    }

    #[test]
    fn v1_prospective_named_storage_path_is_forbidden() {
        let report = learning_v1_gap_report(&[]);
        assert!(
            write_and_verify_agent_canonical_view_gap_report_v1(
                &report,
                Path::new("state/learning_data/prospective")
            )
            .is_err()
        );
    }

    #[test]
    fn v1_atomic_artifacts_reopen_and_duplicate_reject() {
        let root = PathBuf::from(format!(
            "state/learning_data/test-acquisition-v1-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        let report = learning_v1_gap_report(&[]);
        assert_eq!(
            write_and_verify_agent_canonical_view_gap_report_v1(&report, &root).unwrap(),
            LearningEvidenceArtifactWriteStatusV1::Written
        );
        assert_eq!(
            write_and_verify_agent_canonical_view_gap_report_v1(&report, &root).unwrap(),
            LearningEvidenceArtifactWriteStatusV1::DuplicateRejected
        );
        let registration = learning_v1_registration();
        assert_eq!(
            write_and_verify_learning_evidence_registration_v1(&registration, &root).unwrap(),
            LearningEvidenceArtifactWriteStatusV1::Written
        );
        assert_eq!(
            read_learning_evidence_registration_v1(&registration.registration_digest, &root)
                .unwrap(),
            registration
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn v1_gap_digest_changes_when_current_evidence_changes() {
        let empty = learning_v1_gap_report(&[]);
        let momentum = learning_v1_intents()
            .into_iter()
            .find(|intent| intent.agent_kind == AgentKind::MomentumTrendFast)
            .unwrap();
        let filled = learning_v1_gap_report(&[learning_v1_snapshot(&momentum)]);
        assert_ne!(empty.report_digest, filled.report_digest);
    }

    #[test]
    fn v1_response_with_protected_timestamp_rejects() {
        let mut registration = learning_v1_registration();
        registration
            .excluded_timestamp_ms
            .push(registration.expected_timestamp_ms[0]);
        registration.excluded_timestamp_ms.sort();
        registration.excluded_timestamp_ms.dedup();
        registration.registration_digest = learning_evidence_registration_digest_v1(&registration);
        assert!(validate_learning_evidence_acquisition_registration_v1(&registration).is_err());
    }

    #[test]
    fn v1_authority_and_future_read_counters_stay_zero() {
        let registration = learning_v1_registration();
        let result = execute_learning_evidence_acquisition_v1(
            &registration,
            &learning_v1_contract(),
            &registration.gap_report_digests,
            None,
            &[],
            true,
            |request| Ok(learning_v1_transport_response(request)),
        );
        let counters = result.safety_counters;
        assert_eq!(counters.request_attempts, 1);
        assert_eq!(counters.retry_count, 0);
        assert_eq!(counters.credential_reads, 0);
        assert_eq!(counters.prospective_artifact_reads, 0);
        assert_eq!(counters.prospective_label_reads, 0);
        assert_eq!(counters.future_evaluation_reads, 0);
        assert_eq!(counters.active_model_changes, 0);
        assert_eq!(counters.chair_decisions, 0);
        assert_eq!(counters.votes, 0);
        assert_eq!(counters.rewards, 0);
        assert_eq!(counters.penalties, 0);
        assert_eq!(counters.voice_changes, 0);
        assert_eq!(counters.promotions, 0);
        assert_eq!(counters.executions, 0);
        assert_eq!(counters.active_committee_count, 3);
    }

    #[test]
    fn v1_invalid_registration_constructs_no_transport() {
        let mut registration = learning_v1_registration();
        registration.maximum_retries = 1;
        let calls = std::cell::Cell::new(0);
        let result = execute_learning_evidence_acquisition_v1(
            &registration,
            &learning_v1_contract(),
            &registration.gap_report_digests,
            None,
            &[],
            true,
            |_| {
                calls.set(calls.get() + 1);
                Err(LearningEvidenceTransportFailureV1::Technical)
            },
        );
        assert_eq!(calls.get(), 0);
        assert_eq!(
            result.status,
            LearningEvidenceRequestStatusV1::RegistrationInvalid
        );
    }

    #[test]
    fn v1_stale_gap_constructs_no_transport() {
        let registration = learning_v1_registration();
        let calls = std::cell::Cell::new(0);
        let result = execute_learning_evidence_acquisition_v1(
            &registration,
            &learning_v1_contract(),
            &[],
            None,
            &[],
            true,
            |_| {
                calls.set(calls.get() + 1);
                Err(LearningEvidenceTransportFailureV1::Technical)
            },
        );
        assert_eq!(calls.get(), 0);
        assert_eq!(
            result.status,
            LearningEvidenceRequestStatusV1::GapNoLongerCurrent
        );
    }

    #[test]
    fn v1_unmatched_provider_contract_constructs_no_transport() {
        let registration = learning_v1_registration();
        let mut contract = learning_v1_contract();
        contract.maximum_response_bytes /= 2;
        contract.contract_digest.clear();
        let contract = seal_learning_evidence_provider_contract_v1(contract).unwrap();
        let calls = std::cell::Cell::new(0);
        let result = execute_learning_evidence_acquisition_v1(
            &registration,
            &contract,
            &registration.gap_report_digests,
            None,
            &[],
            true,
            |_| {
                calls.set(calls.get() + 1);
                Err(LearningEvidenceTransportFailureV1::Technical)
            },
        );
        assert_eq!(calls.get(), 0);
        assert_eq!(
            result.status,
            LearningEvidenceRequestStatusV1::RegistrationInvalid
        );
    }

    #[test]
    fn v1_invalid_response_consumes_exactly_one_attempt() {
        let registration = learning_v1_registration();
        let calls = std::cell::Cell::new(0);
        let result = execute_learning_evidence_acquisition_v1(
            &registration,
            &learning_v1_contract(),
            &registration.gap_report_digests,
            None,
            &[],
            true,
            |request| {
                calls.set(calls.get() + 1);
                let mut response = learning_v1_transport_response(request);
                response.http_status_class = "4xx".into();
                Ok(response)
            },
        );
        assert_eq!(calls.get(), 1);
        assert_eq!(
            result.status,
            LearningEvidenceRequestStatusV1::InvalidResponse
        );
        let receipt = result.receipt.unwrap();
        assert_eq!(receipt.request_count, 1);
        assert_eq!(receipt.retry_count, 0);
        assert!(result.snapshot.is_none());
    }

    #[test]
    fn v1_resolved_required_with_missing_optional_is_optional_only() {
        let intents = learning_v1_intents();
        let momentum = intents
            .iter()
            .find(|intent| intent.agent_kind == AgentKind::MomentumTrendFast)
            .unwrap();
        let report = learning_v1_gap_report(&[learning_v1_snapshot(momentum)]);
        let gap = report
            .gaps
            .iter()
            .find(|gap| gap.agent_id == "momentum_trend_fast")
            .unwrap();
        assert_eq!(
            gap.status,
            CanonicalViewGapStatusV1::MissingOptionalEvidenceOnly
        );
        assert!(gap.missing_required_dataset_kinds.is_empty());
        assert!(!gap.missing_optional_dataset_kinds.is_empty());
    }

    #[test]
    fn v1_receipt_digest_rejects_semantic_mutation() {
        let registration = learning_v1_registration();
        let result = execute_learning_evidence_acquisition_v1(
            &registration,
            &learning_v1_contract(),
            &registration.gap_report_digests,
            None,
            &[],
            true,
            |request| Ok(learning_v1_transport_response(request)),
        );
        let mut receipt = result.receipt.unwrap();
        receipt.verified_row_count = receipt.verified_row_count.saturating_sub(1);
        assert!(encode_learning_evidence_receipt_protobuf_v1(&receipt).is_err());
    }

    fn composite_learning_contract() -> LearningEvidenceProviderContractV1 {
        let mut contract = learning_v1_contract();
        contract.earliest_timestamp_ms =
            LEARNING_V1_CUTOFF_MS.saturating_sub(400 * DAILY_CADENCE_MS_V1);
        contract.latest_exclusive_timestamp_ms = LEARNING_V1_CUTOFF_MS + 1;
        contract.contract_digest = String::new();
        seal_learning_evidence_provider_contract_v1(contract).unwrap()
    }

    fn composite_learning_report() -> AgentCanonicalViewGapReportV1 {
        let mut intents = learning_v1_intents();
        let momentum = intents
            .iter_mut()
            .find(|intent| intent.agent_kind == AgentKind::MomentumTrendFast)
            .unwrap();
        momentum.lookback = DataLookback {
            bars: 312,
            start_timestamp_ms: None,
            end_timestamp_ms: Some(LEARNING_V1_CUTOFF_MS),
        };
        momentum.information_cutoff_ms = LEARNING_V1_CUTOFF_MS;
        momentum.intent_digest = agent_learning_intent_digest_v0(momentum);
        derive_agent_canonical_view_gaps_v1(
            &intents,
            &default_agent_data_policies(),
            &[],
            &learning_v1_trainers(),
            &[composite_learning_contract()],
        )
        .unwrap()
    }

    fn composite_learning_registration() -> CompositeLearningAcquisitionRegistrationV1 {
        select_composite_learning_acquisition_registration_v1(
            &composite_learning_report(),
            &[composite_learning_contract()],
            &["opening-registration".into()],
            &[
                LEARNING_V1_CUTOFF_MS + DAILY_CADENCE_MS_V1,
                LEARNING_V1_CUTOFF_MS + 2 * DAILY_CADENCE_MS_V1,
            ],
        )
        .unwrap()
        .unwrap()
    }

    fn composite_segment_response(
        request: &ReadOnlyProviderRequest,
    ) -> LearningEvidenceTransportResponseV1 {
        let timestamps = expected_learning_timestamps_v1(&request.lookback).unwrap();
        let rows = timestamps
            .iter()
            .enumerate()
            .map(|(index, timestamp_ms)| {
                let close = 100.0 + index as f64;
                HistoricalOhlcvRow {
                    symbol: "KRW-BTC".into(),
                    timestamp_ms: *timestamp_ms,
                    open: close,
                    high: close + 2.0,
                    low: close - 2.0,
                    close: close + 1.0,
                    volume: 10.0,
                    trade_value: Some(1_000.0),
                }
            })
            .collect::<Vec<_>>();
        let raw_response = format!("[{{\"request\":\"{}\"}}]", request.request_id).into_bytes();
        LearningEvidenceTransportResponseV1 {
            http_status_class: "2xx".into(),
            raw_response: raw_response.clone(),
            response: ReadOnlyProviderResponse {
                request_id: request.request_id.clone(),
                provider_id: request.provider_id.clone(),
                fetched_at_ms: LEARNING_V1_CUTOFF_MS,
                content_type: "application/x-soma-normalized-dataset".into(),
                all_rows_finalized: true,
                normalized_dataset: HistoricalReplayDataset {
                    symbol: "KRW-BTC".into(),
                    rows,
                    source: "upbit".into(),
                    reason_codes: vec![],
                },
                reported_content_bytes: raw_response.len(),
                reason_codes: vec![],
            },
        }
    }

    #[test]
    fn composite_plan_is_exact_deterministic_complete_and_disjoint() {
        let first = composite_learning_registration();
        let second = composite_learning_registration();
        assert_eq!(first, second);
        assert_eq!(first.segments.len(), 2);
        assert_eq!(first.segments[0].expected_row_count, 200);
        assert_eq!(first.segments[1].expected_row_count, 112);
        let expected = composite_learning_expected_timestamps_v1(&first);
        assert_eq!(expected.len(), first.required_row_count);
        assert_eq!(
            expected.iter().copied().collect::<BTreeSet<_>>().len(),
            expected.len()
        );
        let newer = first.segments[0]
            .expected_timestamps
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        let older = first.segments[1]
            .expected_timestamps
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        assert!(newer.is_disjoint(&older));
    }

    #[test]
    fn composite_plan_rejects_shortening_and_segment_cap_overflow() {
        let contract = composite_learning_contract();
        let mut registration = composite_learning_registration();
        registration.required_row_count -= 1;
        registration.registration_digest = composite_learning_registration_digest_v1(&registration);
        assert!(
            validate_composite_learning_acquisition_registration_v1(&registration, &contract)
                .is_err()
        );
        let timestamps = (0..401)
            .map(|index| index as u64 * DAILY_CADENCE_MS_V1)
            .collect::<Vec<_>>();
        assert!(derive_learning_segments_v1(&timestamps, 200).is_err());
    }

    #[test]
    fn composite_first_failure_suppresses_second_and_consumes_one_attempt() {
        let registration = composite_learning_registration();
        let calls = std::cell::Cell::new(0);
        let result = execute_composite_learning_acquisition_v1(
            &registration,
            &composite_learning_contract(),
            &[registration.gap_report_digest.clone()],
            None,
            true,
            |_segment, _request| {
                calls.set(calls.get() + 1);
                Err(LearningEvidenceTransportFailureV1::Technical)
            },
        );
        assert_eq!(calls.get(), 1);
        assert_eq!(
            result.status,
            CompositeLearningEpochStatusV1::TerminalSegmentFailure
        );
        assert_eq!(result.segment_receipts.len(), 1);
        assert_eq!(result.safety_counters.request_attempts, 1);
        assert_eq!(result.safety_counters.retry_count, 0);
        assert!(result.snapshot.is_none());
    }

    #[test]
    fn composite_second_failure_retains_only_forensic_partial_evidence() {
        let registration = composite_learning_registration();
        let calls = std::cell::Cell::new(0);
        let result = execute_composite_learning_acquisition_v1(
            &registration,
            &composite_learning_contract(),
            &[registration.gap_report_digest.clone()],
            None,
            true,
            |_segment, request| {
                calls.set(calls.get() + 1);
                if calls.get() == 1 {
                    Ok(composite_segment_response(request))
                } else {
                    Err(LearningEvidenceTransportFailureV1::TimedOut)
                }
            },
        );
        assert_eq!(calls.get(), 2);
        assert_eq!(
            result.status,
            CompositeLearningEpochStatusV1::TerminalPartialEvidence
        );
        assert_eq!(result.segment_capsules.len(), 1);
        assert!(result.snapshot.is_none());
        assert_eq!(result.safety_counters.retry_count, 0);
    }

    #[test]
    fn composite_success_merges_exact_semantic_dataset_independent_of_capsule_order() {
        let registration = composite_learning_registration();
        let result = execute_composite_learning_acquisition_v1(
            &registration,
            &composite_learning_contract(),
            &[registration.gap_report_digest.clone()],
            None,
            true,
            |_segment, request| Ok(composite_segment_response(request)),
        );
        assert_eq!(
            result.status,
            CompositeLearningEpochStatusV1::EvidenceAcquired
        );
        let snapshot = result.snapshot.as_ref().unwrap();
        assert_eq!(snapshot.row_count, registration.required_row_count);
        assert_eq!(result.segment_receipts.len(), 2);
        assert_eq!(result.safety_counters.request_attempts, 2);
        assert_eq!(result.safety_counters.retry_count, 0);
        let mut reversed = result.segment_capsules.clone();
        reversed.reverse();
        let mut rows = reversed
            .iter()
            .flat_map(|capsule| capsule.rows.clone())
            .collect::<Vec<_>>();
        rows.sort_by_key(|row| row.timestamp_ms);
        let dataset = HistoricalReplayDataset {
            symbol: registration.symbols[0].clone(),
            rows,
            source: composite_learning_contract().provider_id,
            reason_codes: vec![],
        };
        assert_eq!(
            historical_replay_dataset_digest_v0(&dataset),
            snapshot.content_digest
        );
    }

    #[test]
    fn composite_exact_timestamp_validation_rejects_missing_duplicate_and_extra() {
        for mutation in 0..3 {
            let registration = composite_learning_registration();
            let result = execute_composite_learning_acquisition_v1(
                &registration,
                &composite_learning_contract(),
                &[registration.gap_report_digest.clone()],
                None,
                true,
                |_segment, request| {
                    let mut response = composite_segment_response(request);
                    match mutation {
                        0 => {
                            response.response.normalized_dataset.rows.pop();
                        }
                        1 => {
                            let row = response.response.normalized_dataset.rows[0].clone();
                            response.response.normalized_dataset.rows.insert(0, row);
                        }
                        _ => {
                            let mut row = response
                                .response
                                .normalized_dataset
                                .rows
                                .last()
                                .unwrap()
                                .clone();
                            row.timestamp_ms += DAILY_CADENCE_MS_V1;
                            response.response.normalized_dataset.rows.push(row);
                        }
                    }
                    Ok(response)
                },
            );
            assert_eq!(
                result.status,
                CompositeLearningEpochStatusV1::TerminalSegmentFailure
            );
            assert!(result.snapshot.is_none());
        }
    }

    #[test]
    fn composite_protobuf_roundtrip_and_corruption_reject() {
        let registration = composite_learning_registration();
        let contract = composite_learning_contract();
        let bytes =
            encode_composite_learning_registration_protobuf_v1(&registration, &contract).unwrap();
        assert_eq!(
            decode_composite_learning_registration_protobuf_v1(&bytes).unwrap(),
            registration
        );
        let mut corrupt = bytes;
        let digest = registration.registration_digest.as_bytes();
        let offset = corrupt
            .windows(digest.len())
            .position(|window| window == digest)
            .unwrap();
        corrupt[offset] ^= 1;
        assert!(decode_composite_learning_registration_protobuf_v1(&corrupt).is_err());
    }

    #[test]
    fn composite_terminal_receipt_blocks_all_new_transports() {
        let registration = composite_learning_registration();
        let first = execute_composite_learning_acquisition_v1(
            &registration,
            &composite_learning_contract(),
            &[registration.gap_report_digest.clone()],
            None,
            true,
            |_segment, _request| Err(LearningEvidenceTransportFailureV1::Technical),
        );
        let calls = std::cell::Cell::new(0);
        let repeated = execute_composite_learning_acquisition_v1(
            &registration,
            &composite_learning_contract(),
            &[registration.gap_report_digest.clone()],
            first.epoch_receipt.as_ref(),
            true,
            |_segment, _request| {
                calls.set(calls.get() + 1);
                Err(LearningEvidenceTransportFailureV1::Technical)
            },
        );
        assert_eq!(
            repeated.status,
            CompositeLearningEpochStatusV1::AlreadyTerminal
        );
        assert_eq!(calls.get(), 0);
    }
}
