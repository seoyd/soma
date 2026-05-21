use std::collections::{BTreeMap, BTreeSet};

use crate::core::ReasonCode;
use crate::data::{
    ProviderDataSubject, ProviderEntitlementStatus, ProviderEntitlementStatusKind, ProviderKind,
    ProviderMarket, build_default_provider_catalog, provider_cost_profile,
    provider_freshness_profile,
};

use super::evidence_lane::{
    EvidenceCollectionPolicy, EvidenceLane, EvidenceLaneKind, EvidenceLaneStatus,
};
use super::executable_evidence_plan::{
    ExecutableEvidencePlan, ExecutableEvidencePlanConfig, ExplicitEvidenceLaneConfig,
};
use super::lane_storage::{
    build_lane_storage_budget_report, build_provider_reality_storage_report,
    default_lane_storage_budget,
};
use super::provider_reality::{ProviderRealityReport, parse_provider_subject};
use super::strategy_compatibility::{StrategyUseCase, evaluate_strategy_data_compatibility};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EvidencePlanBuilder;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct CandidateLaneSpec {
    lane_kind: EvidenceLaneKind,
    provider_subject: ProviderDataSubject,
    market: ProviderMarket,
    use_case: StrategyUseCase,
    symbols: Vec<String>,
    timeframe: String,
    enabled: bool,
    output_subdir: Option<String>,
    max_rows: Option<usize>,
    max_requests: Option<usize>,
    allow_full_history: bool,
    allow_all_symbols: bool,
}

impl EvidencePlanBuilder {
    pub fn from_provider_reality(
        &self,
        report: &ProviderRealityReport,
        config: &ExecutableEvidencePlanConfig,
    ) -> Result<ExecutableEvidencePlan, String> {
        config.validate()?;

        let mut subjects = BTreeSet::new();
        let mut operators = report.operator_actions.clone();
        for status in &report.entitlement_statuses {
            subjects.insert(status.provider_subject);
        }
        for recommendation in &report.recommendations {
            if let Some(primary) = recommendation.primary_provider {
                subjects.insert(primary);
            }
            for fallback in &recommendation.fallback_providers {
                subjects.insert(*fallback);
            }
            operators.extend(recommendation.required_operator_actions.iter().cloned());
        }
        if config.allow_yfinance_research {
            subjects.insert(ProviderDataSubject::YFinanceResearch);
        }

        let entitlement_map = report
            .entitlement_statuses
            .iter()
            .map(|status| (status.provider_subject, status.clone()))
            .collect::<BTreeMap<_, _>>();

        let mut candidates = default_candidates()
            .into_iter()
            .filter(|candidate| subjects.contains(&candidate.provider_subject))
            .collect::<Vec<_>>();
        candidates.sort();

        self.build_plan_from_candidates(candidates, &entitlement_map, config, operators)
    }

    pub fn from_explicit_lanes(
        &self,
        config: &ExecutableEvidencePlanConfig,
    ) -> Result<ExecutableEvidencePlan, String> {
        config.validate()?;
        let mut candidates = config
            .explicit_lanes
            .iter()
            .map(explicit_to_candidate)
            .collect::<Result<Vec<_>, String>>()?;
        candidates.sort();
        self.build_plan_from_candidates(candidates, &BTreeMap::new(), config, Vec::new())
    }

    fn build_plan_from_candidates(
        &self,
        candidates: Vec<CandidateLaneSpec>,
        entitlement_map: &BTreeMap<ProviderDataSubject, ProviderEntitlementStatus>,
        config: &ExecutableEvidencePlanConfig,
        mut operator_actions: Vec<String>,
    ) -> Result<ExecutableEvidencePlan, String> {
        let catalog = build_default_provider_catalog();
        let mut lanes = Vec::new();
        let mut total_estimated = 0usize;

        for (index, candidate) in candidates.iter().enumerate() {
            let entitlement = entitlement_map
                .get(&candidate.provider_subject)
                .cloned()
                .unwrap_or_else(|| synthetic_entitlement(candidate.provider_subject));
            let mut lane = build_lane(candidate, &entitlement, config, &catalog);
            if index >= config.max_lanes
                || lane.collection_policy.symbols.len() > config.max_symbols
                || lane.collection_policy.max_rows > config.max_rows_per_lane
                || lane.collection_policy.max_requests > config.max_requests_per_lane
                || total_estimated.saturating_add(lane.storage_budget.estimated_bytes)
                    > config.max_total_bytes
            {
                lane.lane_status = EvidenceLaneStatus::SkippedBudgetExceeded;
                lane.enabled = false;
                lane.reason_codes.push(ReasonCode::CollectionBudgetExceeded);
                lane.reason_codes.push(ReasonCode::LaneSkippedBudget);
            } else if matches!(lane.market, ProviderMarket::Crypto) && !config.allow_crypto_only {
                lane.lane_status = EvidenceLaneStatus::SkippedIncompatibleFreshness;
                lane.enabled = false;
                lane.warnings
                    .push("crypto-only evidence is disabled by plan config".to_string());
                lane.reason_codes.push(ReasonCode::DeniedByDefault);
            }
            if lane.is_runnable() {
                total_estimated =
                    total_estimated.saturating_add(lane.storage_budget.estimated_bytes);
            }
            operator_actions.extend(actions_for_lane(&lane));
            lanes.push(lane);
        }

        let storage_budget_summary = build_provider_reality_storage_report(
            lanes
                .iter()
                .map(|lane| build_lane_storage_budget_report(lane, None))
                .collect(),
            config.max_total_bytes,
        );
        Ok(ExecutableEvidencePlan::new(
            config.plan_id.clone(),
            lanes,
            operator_actions,
            storage_budget_summary,
        ))
    }
}

fn explicit_to_candidate(
    explicit: &ExplicitEvidenceLaneConfig,
) -> Result<CandidateLaneSpec, String> {
    let subject = parse_provider_subject(&explicit.provider)?;
    let mut candidate = candidate_from_parts(explicit.lane_kind, subject);
    if !explicit.symbols.is_empty() {
        candidate.symbols = explicit.symbols.clone();
    }
    candidate.enabled = explicit.enabled;
    candidate.output_subdir = explicit.output_subdir.clone();
    candidate.max_rows = explicit.max_rows;
    candidate.max_requests = explicit.max_requests;
    candidate.allow_full_history = explicit.allow_full_history;
    candidate.allow_all_symbols = explicit.allow_all_symbols;
    Ok(candidate)
}

fn build_lane(
    candidate: &CandidateLaneSpec,
    entitlement: &ProviderEntitlementStatus,
    config: &ExecutableEvidencePlanConfig,
    catalog: &crate::data::MarketDataProviderCatalog,
) -> EvidenceLane {
    let compatibility = evaluate_strategy_data_compatibility(
        candidate.provider_subject,
        candidate.use_case,
        Some(entitlement),
    );
    let freshness = provider_freshness_profile(candidate.provider_subject);
    let cost = provider_cost_profile(candidate.provider_subject);
    let provider_kind = candidate.provider_subject.provider_kind();
    let source_kind = provider_kind
        .and_then(|kind| catalog.entry(kind).map(|entry| entry.evidence_source_kind))
        .unwrap_or_else(|| match candidate.provider_subject {
            ProviderDataSubject::YFinanceResearch => {
                crate::data::EvidenceSourceKind::YFinanceResearch
            }
            ProviderDataSubject::Provider(_) => {
                crate::data::EvidenceSourceKind::OfficialApiCollected
            }
        });
    let auth_requirement = provider_kind
        .and_then(|kind| {
            catalog
                .entry(kind)
                .map(|entry| entry.auth_requirement.clone())
        })
        .unwrap_or_else(|| "research-only".to_string());
    let output_subdir = format!(
        "{}-{}",
        lane_kind_slug(candidate.lane_kind),
        provider_slug(candidate.provider_subject)
    );
    let collection_policy = EvidenceCollectionPolicy {
        symbols: candidate.symbols.clone(),
        timeframe: candidate.timeframe.clone(),
        output_subdir: candidate.output_subdir.clone().unwrap_or(output_subdir),
        max_rows: candidate
            .max_rows
            .unwrap_or(config.max_rows_per_lane.min(500)),
        max_requests: candidate
            .max_requests
            .unwrap_or(config.max_requests_per_lane.min(10)),
        allow_full_history: candidate.allow_full_history,
        allow_all_symbols: candidate.allow_all_symbols,
        reason_codes: vec![ReasonCode::EvidenceLaneBuilt],
    };
    let storage_budget = default_lane_storage_budget(
        source_kind,
        collection_policy.max_rows,
        collection_policy.max_requests,
        config.max_total_bytes,
    );
    let lane_status = classify_lane_status(
        candidate,
        entitlement,
        &compatibility,
        config.allow_yfinance_research,
    );
    EvidenceLane {
        lane_id: format!(
            "{}-{}",
            lane_kind_slug(candidate.lane_kind),
            provider_slug(candidate.provider_subject)
        ),
        lane_kind: candidate.lane_kind,
        lane_status,
        market: candidate.market,
        desired_use_case: candidate.use_case,
        provider_subject: candidate.provider_subject,
        provider_kind,
        source_kind,
        freshness_tier: freshness.default_freshness,
        cost_tier: cost.cost_tier,
        auth_requirement,
        strategy_compatibility: compatibility,
        collection_policy,
        storage_budget,
        enabled: candidate.enabled && lane_status == EvidenceLaneStatus::ReadyToRun,
        warnings: build_lane_warnings(candidate, entitlement),
        simulate_collection_failure: false,
        simulate_preflight_failure: false,
        simulate_core_block: false,
        reason_codes: vec![ReasonCode::EvidenceLaneBuilt],
    }
}

fn classify_lane_status(
    candidate: &CandidateLaneSpec,
    entitlement: &ProviderEntitlementStatus,
    compatibility: &crate::experiment::StrategyDataCompatibilityResult,
    allow_yfinance_research: bool,
) -> EvidenceLaneStatus {
    if matches!(
        candidate.provider_subject,
        ProviderDataSubject::YFinanceResearch
    ) && !allow_yfinance_research
    {
        return EvidenceLaneStatus::SkippedResearchOnlyNotOfficial;
    }
    if !compatibility.compatible {
        return EvidenceLaneStatus::SkippedIncompatibleFreshness;
    }
    match entitlement.status {
        ProviderEntitlementStatusKind::ReadyForEodResearch
        | ProviderEntitlementStatusKind::ReadyForIntradayResearch
        | ProviderEntitlementStatusKind::ReadyForRealtimeResearch
        | ProviderEntitlementStatusKind::ReadyForRealtimeResearchIexOnly
        | ProviderEntitlementStatusKind::ReadyForCryptoResearch => EvidenceLaneStatus::ReadyToRun,
        ProviderEntitlementStatusKind::MissingAuth => EvidenceLaneStatus::SkippedMissingAuth,
        ProviderEntitlementStatusKind::MissingApproval => {
            EvidenceLaneStatus::SkippedMissingApproval
        }
        ProviderEntitlementStatusKind::MissingEndpointTemplate => {
            EvidenceLaneStatus::SkippedMissingEndpointTemplate
        }
        ProviderEntitlementStatusKind::MissingPremiumEntitlement
        | ProviderEntitlementStatusKind::NotSuitableForUseCase
        | ProviderEntitlementStatusKind::Deferred => EvidenceLaneStatus::SkippedMissingEntitlement,
        ProviderEntitlementStatusKind::ResearchOnlyFallback => {
            if matches!(
                candidate.lane_kind,
                EvidenceLaneKind::YFinanceResearchFallback
            ) {
                EvidenceLaneStatus::ReadyToRun
            } else {
                EvidenceLaneStatus::SkippedResearchOnlyNotOfficial
            }
        }
    }
}

fn build_lane_warnings(
    candidate: &CandidateLaneSpec,
    entitlement: &ProviderEntitlementStatus,
) -> Vec<String> {
    let mut warnings = Vec::new();
    if matches!(
        entitlement.status,
        ProviderEntitlementStatusKind::ReadyForRealtimeResearchIexOnly
    ) {
        warnings.push("IEX-limited realtime coverage".to_string());
    }
    if matches!(
        candidate.provider_subject,
        ProviderDataSubject::YFinanceResearch
    ) {
        warnings.push("research-only supplemental lane".to_string());
    }
    warnings
}

fn actions_for_lane(lane: &EvidenceLane) -> Vec<String> {
    let mut actions = Vec::new();
    match (lane.provider_kind, lane.lane_status) {
        (Some(ProviderKind::KrxOpenApi), EvidenceLaneStatus::SkippedMissingAuth) => {
            actions.push("SetKrxAuth".to_string())
        }
        (Some(ProviderKind::KrxOpenApi), EvidenceLaneStatus::SkippedMissingApproval) => {
            actions.push("WaitForKrxApproval".to_string())
        }
        (Some(ProviderKind::KrxOpenApi), EvidenceLaneStatus::SkippedMissingEndpointTemplate) => {
            actions.push("SetKrxEndpointTemplate".to_string())
        }
        (Some(ProviderKind::DataGoKrFscStockPrice), EvidenceLaneStatus::SkippedMissingAuth) => {
            actions.push("SetDataGoKrAuth".to_string())
        }
        (Some(ProviderKind::AlphaVantage), EvidenceLaneStatus::SkippedMissingAuth) => {
            actions.push("SetAlphaVantageAuth".to_string())
        }
        (Some(ProviderKind::Alpaca), EvidenceLaneStatus::SkippedMissingAuth) => {
            actions.push("SetAlpacaAuth".to_string())
        }
        (Some(ProviderKind::Alpaca), EvidenceLaneStatus::SkippedMissingEntitlement)
        | (
            Some(ProviderKind::PolygonProfessional),
            EvidenceLaneStatus::SkippedMissingEntitlement,
        ) => actions.push("BuyOrConfigureRealtimeEntitlement".to_string()),
        _ => {}
    }
    if matches!(lane.provider_subject, ProviderDataSubject::YFinanceResearch) {
        actions.push("UseYFinanceResearchOnly".to_string());
    }
    if lane.is_runnable() && matches!(lane.market, ProviderMarket::Crypto) {
        actions.push("UseUpbitCryptoOnly".to_string());
    }
    actions
}

fn default_candidates() -> Vec<CandidateLaneSpec> {
    vec![
        candidate_from_parts(
            EvidenceLaneKind::CryptoIntradayEvidence,
            ProviderDataSubject::Provider(ProviderKind::Upbit),
        ),
        candidate_from_parts(
            EvidenceLaneKind::CryptoEodEvidence,
            ProviderDataSubject::Provider(ProviderKind::Upbit),
        ),
        candidate_from_parts(
            EvidenceLaneKind::KoreanEquityEodEvidence,
            ProviderDataSubject::Provider(ProviderKind::KrxOpenApi),
        ),
        candidate_from_parts(
            EvidenceLaneKind::KoreanEquityEodEvidence,
            ProviderDataSubject::Provider(ProviderKind::DataGoKrFscStockPrice),
        ),
        candidate_from_parts(
            EvidenceLaneKind::KoreanEquityIntradayResearch,
            ProviderDataSubject::Provider(ProviderKind::KoreaInvestmentMarketData),
        ),
        candidate_from_parts(
            EvidenceLaneKind::USEquityEodEvidence,
            ProviderDataSubject::Provider(ProviderKind::AlphaVantage),
        ),
        candidate_from_parts(
            EvidenceLaneKind::USEquityRealtimeResearch,
            ProviderDataSubject::Provider(ProviderKind::Alpaca),
        ),
        candidate_from_parts(
            EvidenceLaneKind::USEquityFullMarketRealtimeResearch,
            ProviderDataSubject::Provider(ProviderKind::PolygonProfessional),
        ),
        candidate_from_parts(
            EvidenceLaneKind::YFinanceResearchFallback,
            ProviderDataSubject::YFinanceResearch,
        ),
    ]
}

fn candidate_from_parts(
    lane_kind: EvidenceLaneKind,
    provider_subject: ProviderDataSubject,
) -> CandidateLaneSpec {
    match lane_kind {
        EvidenceLaneKind::CryptoIntradayEvidence => CandidateLaneSpec {
            lane_kind,
            provider_subject,
            market: ProviderMarket::Crypto,
            use_case: StrategyUseCase::IntradaySwing,
            symbols: vec!["BTC-KRW".to_string()],
            timeframe: "1m".to_string(),
            enabled: true,
            output_subdir: None,
            max_rows: None,
            max_requests: None,
            allow_full_history: false,
            allow_all_symbols: false,
        },
        EvidenceLaneKind::CryptoEodEvidence => CandidateLaneSpec {
            lane_kind,
            provider_subject,
            market: ProviderMarket::Crypto,
            use_case: StrategyUseCase::EodSwing,
            symbols: vec!["BTC-KRW".to_string()],
            timeframe: "1d".to_string(),
            enabled: true,
            output_subdir: None,
            max_rows: None,
            max_requests: None,
            allow_full_history: false,
            allow_all_symbols: false,
        },
        EvidenceLaneKind::KoreanEquityEodEvidence => CandidateLaneSpec {
            lane_kind,
            provider_subject,
            market: ProviderMarket::KoreanEquity,
            use_case: StrategyUseCase::EodSwing,
            symbols: vec!["005930.KS".to_string()],
            timeframe: "1d".to_string(),
            enabled: true,
            output_subdir: None,
            max_rows: None,
            max_requests: None,
            allow_full_history: false,
            allow_all_symbols: false,
        },
        EvidenceLaneKind::KoreanEquityIntradayResearch => CandidateLaneSpec {
            lane_kind,
            provider_subject,
            market: ProviderMarket::KoreanEquity,
            use_case: StrategyUseCase::IntradaySwing,
            symbols: vec!["005930.KS".to_string()],
            timeframe: "1m".to_string(),
            enabled: true,
            output_subdir: None,
            max_rows: None,
            max_requests: None,
            allow_full_history: false,
            allow_all_symbols: false,
        },
        EvidenceLaneKind::USEquityEodEvidence => CandidateLaneSpec {
            lane_kind,
            provider_subject,
            market: ProviderMarket::USEquity,
            use_case: StrategyUseCase::EodSwing,
            symbols: vec!["AAPL".to_string()],
            timeframe: "1d".to_string(),
            enabled: true,
            output_subdir: None,
            max_rows: None,
            max_requests: None,
            allow_full_history: false,
            allow_all_symbols: false,
        },
        EvidenceLaneKind::USEquityRealtimeResearch => CandidateLaneSpec {
            lane_kind,
            provider_subject,
            market: ProviderMarket::USEquity,
            use_case: StrategyUseCase::RealtimeScalping,
            symbols: vec!["AAPL".to_string()],
            timeframe: "1m".to_string(),
            enabled: true,
            output_subdir: None,
            max_rows: None,
            max_requests: None,
            allow_full_history: false,
            allow_all_symbols: false,
        },
        EvidenceLaneKind::USEquityFullMarketRealtimeResearch => CandidateLaneSpec {
            lane_kind,
            provider_subject,
            market: ProviderMarket::USEquity,
            use_case: StrategyUseCase::RealtimeExecutionSimulation,
            symbols: vec!["AAPL".to_string()],
            timeframe: "1m".to_string(),
            enabled: true,
            output_subdir: None,
            max_rows: None,
            max_requests: None,
            allow_full_history: false,
            allow_all_symbols: false,
        },
        EvidenceLaneKind::YFinanceResearchFallback => CandidateLaneSpec {
            lane_kind,
            provider_subject,
            market: ProviderMarket::USEquity,
            use_case: StrategyUseCase::SourceComparison,
            symbols: vec!["AAPL".to_string()],
            timeframe: "1d".to_string(),
            enabled: true,
            output_subdir: None,
            max_rows: None,
            max_requests: None,
            allow_full_history: false,
            allow_all_symbols: false,
        },
        EvidenceLaneKind::DiagnosticsOnly => CandidateLaneSpec {
            lane_kind,
            provider_subject,
            market: ProviderMarket::USEquity,
            use_case: StrategyUseCase::ModelPrototypeResearch,
            symbols: vec!["AAPL".to_string()],
            timeframe: "1d".to_string(),
            enabled: true,
            output_subdir: None,
            max_rows: None,
            max_requests: None,
            allow_full_history: false,
            allow_all_symbols: false,
        },
    }
}

fn synthetic_entitlement(subject: ProviderDataSubject) -> ProviderEntitlementStatus {
    let freshness = provider_freshness_profile(subject);
    let cost = provider_cost_profile(subject);
    let status = match subject {
        ProviderDataSubject::Provider(ProviderKind::Upbit) => {
            ProviderEntitlementStatusKind::ReadyForCryptoResearch
        }
        ProviderDataSubject::YFinanceResearch => {
            ProviderEntitlementStatusKind::ResearchOnlyFallback
        }
        ProviderDataSubject::Provider(ProviderKind::PolygonProfessional) => {
            ProviderEntitlementStatusKind::MissingPremiumEntitlement
        }
        _ => ProviderEntitlementStatusKind::MissingAuth,
    };
    ProviderEntitlementStatus {
        provider_subject: subject,
        freshness_available: freshness.available_freshness_tiers,
        cost_tier: cost.cost_tier,
        auth_ready: matches!(
            status,
            ProviderEntitlementStatusKind::ReadyForCryptoResearch
        ),
        approval_ready: !matches!(
            subject,
            ProviderDataSubject::Provider(ProviderKind::KrxOpenApi)
        ),
        endpoint_template_ready: !matches!(
            subject,
            ProviderDataSubject::Provider(ProviderKind::KrxOpenApi)
        ),
        realtime_entitlement_ready: matches!(
            status,
            ProviderEntitlementStatusKind::ReadyForRealtimeResearch
                | ProviderEntitlementStatusKind::ReadyForRealtimeResearchIexOnly
                | ProviderEntitlementStatusKind::ReadyForCryptoResearch
        ),
        delayed_entitlement_ready: matches!(
            status,
            ProviderEntitlementStatusKind::ReadyForIntradayResearch
                | ProviderEntitlementStatusKind::ReadyForRealtimeResearch
                | ProviderEntitlementStatusKind::ReadyForRealtimeResearchIexOnly
                | ProviderEntitlementStatusKind::ReadyForCryptoResearch
        ),
        official_readiness_eligible: !matches!(subject, ProviderDataSubject::YFinanceResearch),
        research_only: matches!(subject, ProviderDataSubject::YFinanceResearch),
        status,
        reason_codes: vec![ReasonCode::ProviderEntitlementPreflightBuilt],
    }
}

fn lane_kind_slug(kind: EvidenceLaneKind) -> &'static str {
    match kind {
        EvidenceLaneKind::CryptoIntradayEvidence => "crypto-intraday",
        EvidenceLaneKind::CryptoEodEvidence => "crypto-eod",
        EvidenceLaneKind::KoreanEquityEodEvidence => "kr-eod",
        EvidenceLaneKind::KoreanEquityIntradayResearch => "kr-intraday",
        EvidenceLaneKind::USEquityEodEvidence => "us-eod",
        EvidenceLaneKind::USEquityRealtimeResearch => "us-realtime",
        EvidenceLaneKind::USEquityFullMarketRealtimeResearch => "us-full-realtime",
        EvidenceLaneKind::YFinanceResearchFallback => "yfinance-fallback",
        EvidenceLaneKind::DiagnosticsOnly => "diagnostics",
    }
}

fn provider_slug(subject: ProviderDataSubject) -> &'static str {
    match subject {
        ProviderDataSubject::Provider(ProviderKind::Upbit) => "upbit",
        ProviderDataSubject::Provider(ProviderKind::KrxOpenApi) => "krx",
        ProviderDataSubject::Provider(ProviderKind::DataGoKrFscStockPrice) => "data-go-kr",
        ProviderDataSubject::Provider(ProviderKind::KoreaInvestmentMarketData) => "kis",
        ProviderDataSubject::Provider(ProviderKind::AlphaVantage) => "alphavantage",
        ProviderDataSubject::Provider(ProviderKind::Alpaca) => "alpaca",
        ProviderDataSubject::Provider(ProviderKind::PolygonProfessional) => "polygon",
        ProviderDataSubject::Provider(ProviderKind::NasdaqDataLink) => "nasdaq-data-link",
        ProviderDataSubject::Provider(ProviderKind::KoscomProfessional) => "koscom",
        ProviderDataSubject::Provider(ProviderKind::Binance) => "binance",
        ProviderDataSubject::Provider(ProviderKind::Korbit) => "korbit",
        ProviderDataSubject::Provider(ProviderKind::MockFixture) => "mock-fixture",
        ProviderDataSubject::Provider(ProviderKind::Unknown) => "unknown",
        ProviderDataSubject::YFinanceResearch => "yfinance",
    }
}
