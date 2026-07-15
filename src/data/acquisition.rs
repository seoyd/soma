use std::collections::{BTreeMap, BTreeSet};

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
    pub normalized_dataset: HistoricalReplayDataset,
    pub reported_content_bytes: usize,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderFetchFailure {
    Unavailable,
    RateLimited,
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

    pub fn verify_digest(&self, snapshot_id: &str) -> bool {
        self.get(snapshot_id)
            .is_some_and(|snapshot| self.verify_snapshot(&snapshot))
    }

    fn verify_snapshot(&self, snapshot: &DataSnapshot) -> bool {
        snapshot.content_digest == snapshot_digest(&snapshot.normalized_dataset)
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
                Err(_) if attempts <= self.acquisition_policy.max_retries => continue,
                Err(error) => break Err(error),
            }
        };
        let response = match response {
            Ok(response) => response,
            Err(error) => {
                let reason = match error {
                    ProviderFetchFailure::RateLimited => ReasonCode::AcquisitionRateLimited,
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
        let Some(snapshot) = self.snapshot_store.find_latest(&request.request_key) else {
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
    let snapshot_id = format!(
        "snapshot-{}",
        stable_hash_string(&format!("{}:{}", request.request_key, digest))
    );
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
        concat!("work", ".", "md"),
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
    let mut material = format!("{:?}|{:?}|", dataset.symbol, dataset.source);
    for reason in &dataset.reason_codes {
        material.push_str(&format!("{reason:?}|"));
    }
    for row in &dataset.rows {
        material.push_str(&format!(
            "{:?}|{}|{:016x}|{:016x}|{:016x}|{:016x}|{:016x}|{:?}|",
            row.symbol,
            row.timestamp_ms,
            row.open.to_bits(),
            row.high.to_bits(),
            row.low.to_bits(),
            row.close.to_bits(),
            row.volume.to_bits(),
            row.trade_value.map(f64::to_bits),
        ));
    }
    stable_hash_string(&material)
}

fn snapshot_digest(dataset: &HistoricalReplayDataset) -> String {
    historical_replay_dataset_digest_v0(dataset)
}

fn acquisition_request_key(dataset_kind: DatasetKind, intent: &AgentDataIntent) -> String {
    let mut symbols = intent.symbols.clone();
    symbols.sort();
    format!(
        "{:?}:{:?}:{}:{}:{}",
        dataset_kind,
        intent.market_scope,
        symbols.join(","),
        intent.lookback.bars,
        intent.lookback.start_timestamp_ms.unwrap_or_default(),
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
                normalized_dataset: mock_dataset(),
                reported_content_bytes: 512,
                reason_codes: vec![],
            }),
            default_failure: None,
            requests: Vec::new(),
        }
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
    fn replay_uses_last_known_good_only_within_tolerance() {
        let mut registry = ReadOnlyProviderRegistry::default();
        registry.register(mock_capabilities());
        let mut policy = AcquisitionPolicy::default();
        policy.stale_data_policy = StaleDataPolicy::UseLastKnownGoodWithinTolerance;
        policy.last_known_good_tolerance_ms = 100;
        let mut broker = DataAcquisitionBroker::new(registry, policy);
        let mut provider = mock_provider(10);
        let mut stale_input = input(AcquisitionMode::Mock, 10);
        for policy in &mut stale_input.agent_data_policies {
            policy.max_staleness_ms = 20;
        }
        let initial = execute_autonomous_data_cycle(&stale_input, &mut broker, Some(&mut provider));
        assert!(!initial.new_snapshots.is_empty());
        stale_input.acquisition_mode = AcquisitionMode::LocalSnapshotReplay;
        stale_input.now_ms = 50;
        let replay = execute_autonomous_data_cycle(&stale_input, &mut broker, None);
        assert!(!replay.reused_snapshots.is_empty());
        stale_input.now_ms = 1_000;
        let rejected = execute_autonomous_data_cycle(&stale_input, &mut broker, None);
        assert!(rejected.acquisition_receipts.iter().any(|receipt| {
            receipt
                .reason_codes
                .contains(&ReasonCode::EvidenceStaleRejected)
        }));
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
