use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;
use crate::data::{
    DataFreshnessTier, EvidenceSourceKind, ProviderCostTier, ProviderDataSubject, ProviderKind,
    ProviderMarket,
};

use super::lane_storage::LaneStorageBudget;
use super::{StrategyDataCompatibilityResult, StrategyUseCase};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EvidenceLaneKind {
    CryptoIntradayEvidence,
    CryptoEodEvidence,
    KoreanEquityEodEvidence,
    KoreanEquityIntradayResearch,
    USEquityEodEvidence,
    USEquityRealtimeResearch,
    USEquityFullMarketRealtimeResearch,
    YFinanceResearchFallback,
    DiagnosticsOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EvidenceLaneStatus {
    ReadyToRun,
    RanSuccessfully,
    SkippedMissingAuth,
    SkippedMissingApproval,
    SkippedMissingEndpointTemplate,
    SkippedMissingEntitlement,
    SkippedIncompatibleFreshness,
    SkippedResearchOnlyNotOfficial,
    SkippedBudgetExceeded,
    SkippedCoreBlocked,
    FailedCollection,
    FailedPreflight,
    FailedBenchmark,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceCollectionPolicy {
    pub symbols: Vec<String>,
    pub timeframe: String,
    pub output_subdir: String,
    pub max_rows: usize,
    pub max_requests: usize,
    #[serde(default)]
    pub allow_full_history: bool,
    #[serde(default)]
    pub allow_all_symbols: bool,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EvidenceLane {
    pub lane_id: String,
    pub lane_kind: EvidenceLaneKind,
    pub lane_status: EvidenceLaneStatus,
    pub market: ProviderMarket,
    pub desired_use_case: StrategyUseCase,
    pub provider_subject: ProviderDataSubject,
    #[serde(default)]
    pub provider_kind: Option<ProviderKind>,
    pub source_kind: EvidenceSourceKind,
    pub freshness_tier: DataFreshnessTier,
    pub cost_tier: ProviderCostTier,
    pub auth_requirement: String,
    pub strategy_compatibility: StrategyDataCompatibilityResult,
    pub collection_policy: EvidenceCollectionPolicy,
    pub storage_budget: LaneStorageBudget,
    pub enabled: bool,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub simulate_collection_failure: bool,
    #[serde(default)]
    pub simulate_preflight_failure: bool,
    #[serde(default)]
    pub simulate_core_block: bool,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceLaneCollectionReport {
    pub attempted: bool,
    pub records_collected: usize,
    pub request_count: usize,
    #[serde(default)]
    pub output_path: Option<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceLanePreflightReport {
    pub attempted: bool,
    pub passed: bool,
    pub outcome_records: usize,
    pub warnings: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceLaneBenchmarkReport {
    pub attempted: bool,
    pub core_check_passed: bool,
    pub benchmark_ran: bool,
    pub outcome_records: usize,
    #[serde(default)]
    pub calibration_summary: Option<String>,
    #[serde(default)]
    pub risk_summary: Option<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceLaneYFinanceReport {
    pub attempted: bool,
    #[serde(default)]
    pub manifest_path: Option<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EvidenceLaneRunReport {
    pub lane_id: String,
    pub lane_status: EvidenceLaneStatus,
    #[serde(default)]
    pub provider_kind: Option<ProviderKind>,
    pub source_kind: EvidenceSourceKind,
    #[serde(default)]
    pub collection_report: Option<EvidenceLaneCollectionReport>,
    #[serde(default)]
    pub preflight_report: Option<EvidenceLanePreflightReport>,
    #[serde(default)]
    pub benchmark_report: Option<EvidenceLaneBenchmarkReport>,
    #[serde(default)]
    pub yfinance_report: Option<EvidenceLaneYFinanceReport>,
    pub outcome_records: usize,
    #[serde(default)]
    pub calibration_summary: Option<String>,
    #[serde(default)]
    pub risk_summary: Option<String>,
    pub storage_bytes: usize,
    pub reason_codes: Vec<ReasonCode>,
}

impl EvidenceLane {
    pub fn is_runnable(&self) -> bool {
        self.enabled && self.lane_status == EvidenceLaneStatus::ReadyToRun
    }

    pub fn benchmark_eligible(&self) -> bool {
        self.is_runnable()
            && !matches!(
                self.lane_kind,
                EvidenceLaneKind::DiagnosticsOnly | EvidenceLaneKind::YFinanceResearchFallback
            )
            && self.strategy_compatibility.compatible
    }

    pub fn official_readiness_eligible(&self) -> bool {
        self.source_kind.readiness_eligible()
            && !matches!(self.provider_subject, ProviderDataSubject::YFinanceResearch)
    }
}

impl EvidenceLaneRunReport {
    pub fn is_success(&self) -> bool {
        matches!(
            self.lane_status,
            EvidenceLaneStatus::RanSuccessfully | EvidenceLaneStatus::DiagnosticOnly
        )
    }
}
