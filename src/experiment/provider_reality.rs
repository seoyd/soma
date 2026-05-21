use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;
use crate::data::{
    ProviderDataSubject, ProviderEntitlementPreflightConfig, ProviderEntitlementPreflightRunner,
    ProviderEntitlementStatus, ProviderEntitlementStatusKind, ProviderKind,
    default_provider_cost_profiles, default_provider_freshness_profiles,
};

use super::provider_recommendation::{
    BudgetPreference, ProviderRecommendation, ProviderRecommendationRequest,
    ProviderRecommendationStatus, recommend_provider,
};
use super::strategy_compatibility::{
    StrategyDataCompatibilityResult, StrategyUseCase, evaluate_strategy_data_compatibility,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderRealitySummary {
    KRXApprovalPending,
    AlphaVantageEodOnly,
    AlpacaNeededForRealtime,
    PaidProviderNeededForFullMarketRealtime,
    UpbitReady,
    YFinanceResearchOnly,
    NeedProviderAuthSetup,
    ReadyForEodResearch,
    ReadyForCryptoResearch,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StrategyDataCheckRequest {
    pub provider: String,
    pub use_case: StrategyUseCase,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProviderRealityConfig {
    pub report_id: String,
    pub output_dir: String,
    pub entitlement_preflight: ProviderEntitlementPreflightConfig,
    #[serde(default)]
    pub strategy_checks: Vec<StrategyDataCheckRequest>,
    #[serde(default = "default_recommendations")]
    pub recommendation_requests: Vec<ProviderRecommendationRequest>,
    #[serde(default = "default_true")]
    pub include_yfinance_reality: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProviderRealityReport {
    pub report_id: String,
    pub freshness_profiles: Vec<crate::data::ProviderFreshnessProfile>,
    pub cost_profiles: Vec<crate::data::ProviderCostProfile>,
    pub entitlement_statuses: Vec<ProviderEntitlementStatus>,
    pub compatibility_results: Vec<StrategyDataCompatibilityResult>,
    pub recommendations: Vec<ProviderRecommendation>,
    pub operator_actions: Vec<String>,
    pub final_summary: Vec<ProviderRealitySummary>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProviderRealityRunner;

impl Default for ProviderRealityConfig {
    fn default() -> Self {
        Self {
            report_id: "provider_reality".to_string(),
            output_dir: "target/soma_provider_reality".to_string(),
            entitlement_preflight: ProviderEntitlementPreflightConfig::default(),
            strategy_checks: vec![
                StrategyDataCheckRequest {
                    provider: "alphavantage".to_string(),
                    use_case: StrategyUseCase::EodSwing,
                },
                StrategyDataCheckRequest {
                    provider: "yfinance".to_string(),
                    use_case: StrategyUseCase::SourceComparison,
                },
            ],
            recommendation_requests: default_recommendations(),
            include_yfinance_reality: true,
            reason_codes: vec![ReasonCode::ProviderRealityReportBuilt],
        }
    }
}

impl ProviderRealityConfig {
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
}

impl ProviderRealityRunner {
    pub fn run(&self, config: &ProviderRealityConfig) -> Result<ProviderRealityReport, String> {
        let mut entitlement_config = config.entitlement_preflight.clone();
        if config.include_yfinance_reality
            && !entitlement_config
                .providers_to_check
                .contains(&ProviderDataSubject::YFinanceResearch)
        {
            entitlement_config
                .providers_to_check
                .push(ProviderDataSubject::YFinanceResearch);
        }
        let entitlement_statuses =
            ProviderEntitlementPreflightRunner::default().run(&entitlement_config);
        let compatibility_results = config
            .strategy_checks
            .iter()
            .map(|request| {
                let subject = parse_provider_subject(&request.provider)?;
                let entitlement = entitlement_statuses
                    .iter()
                    .find(|status| status.provider_subject == subject);
                Ok(evaluate_strategy_data_compatibility(
                    subject,
                    request.use_case,
                    entitlement,
                ))
            })
            .collect::<Result<Vec<_>, String>>()?;
        let recommendations = config
            .recommendation_requests
            .iter()
            .map(|request| recommend_provider(request, &entitlement_statuses))
            .collect::<Vec<_>>();
        let operator_actions = build_operator_actions(&entitlement_statuses, &recommendations);
        let final_summary = build_final_summary(
            &entitlement_statuses,
            &compatibility_results,
            &recommendations,
        );
        Ok(ProviderRealityReport {
            report_id: config.report_id.clone(),
            freshness_profiles: default_provider_freshness_profiles(),
            cost_profiles: default_provider_cost_profiles(),
            entitlement_statuses,
            compatibility_results,
            recommendations,
            operator_actions,
            final_summary,
            reason_codes: vec![
                ReasonCode::ProviderRealityReportBuilt,
                ReasonCode::DeterministicPath,
            ],
        })
    }
}

impl ProviderRealityReport {
    pub fn from_json_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        serde_json::from_str(&text).map_err(|err| err.to_string())
    }

    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("report_id={}", self.report_id),
            format!(
                "final_summary={}",
                self.final_summary
                    .iter()
                    .map(|value| format!("{value:?}"))
                    .collect::<Vec<_>>()
                    .join("|")
            ),
        ];
        for status in &self.entitlement_statuses {
            lines.push(format!(
                "entitlement={:?};status={:?};auth_ready={};approval_ready={};realtime_entitlement_ready={}",
                status.provider_subject,
                status.status,
                status.auth_ready,
                status.approval_ready,
                status.realtime_entitlement_ready,
            ));
        }
        for compatibility in &self.compatibility_results {
            lines.push(format!(
                "compatibility={:?};provider={:?};compatible={};blockers={};warnings={}",
                compatibility.use_case,
                compatibility.provider_subject,
                compatibility.compatible,
                compatibility.blockers.join("|"),
                compatibility.warnings.join("|"),
            ));
        }
        for recommendation in &self.recommendations {
            lines.push(format!(
                "recommendation={:?};primary={:?};status={:?}",
                recommendation.primary_provider,
                recommendation.fallback_providers,
                recommendation.status,
            ));
        }
        for action in &self.operator_actions {
            lines.push(format!("operator_action={action}"));
        }
        lines.join("\n")
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        let json_path = output_dir.join("provider_reality_report.json");
        fs::write(&json_path, self.to_json_string()?).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("provider_reality_report.txt"),
            self.to_text(),
        )
        .map_err(|err| err.to_string())?;
        Ok(json_path)
    }
}

pub fn parse_provider_subject(value: &str) -> Result<ProviderDataSubject, String> {
    match value {
        "upbit" => Ok(ProviderDataSubject::Provider(ProviderKind::Upbit)),
        "krx" => Ok(ProviderDataSubject::Provider(ProviderKind::KrxOpenApi)),
        "data-go-kr" | "datagokr" => Ok(ProviderDataSubject::Provider(
            ProviderKind::DataGoKrFscStockPrice,
        )),
        "kis" | "kis-market-data" => Ok(ProviderDataSubject::Provider(
            ProviderKind::KoreaInvestmentMarketData,
        )),
        "alphavantage" => Ok(ProviderDataSubject::Provider(ProviderKind::AlphaVantage)),
        "alpaca" => Ok(ProviderDataSubject::Provider(ProviderKind::Alpaca)),
        "polygon" => Ok(ProviderDataSubject::Provider(
            ProviderKind::PolygonProfessional,
        )),
        "nasdaq-data-link" => Ok(ProviderDataSubject::Provider(ProviderKind::NasdaqDataLink)),
        "koscom" => Ok(ProviderDataSubject::Provider(
            ProviderKind::KoscomProfessional,
        )),
        "mock-fixture" => Ok(ProviderDataSubject::Provider(ProviderKind::MockFixture)),
        "yfinance" => Ok(ProviderDataSubject::YFinanceResearch),
        _ => Err(format!("unsupported provider subject: {value}")),
    }
}

fn build_operator_actions(
    entitlement_statuses: &[ProviderEntitlementStatus],
    recommendations: &[ProviderRecommendation],
) -> Vec<String> {
    let mut actions = Vec::new();
    if entitlement_statuses.iter().any(|status| {
        status.provider_subject == ProviderDataSubject::Provider(ProviderKind::KrxOpenApi)
            && status.status == ProviderEntitlementStatusKind::MissingApproval
    }) {
        actions.push(
            "wait for KRX approval before claiming Korean official collection readiness"
                .to_string(),
        );
    }
    if recommendations.iter().any(|recommendation| {
        recommendation.status == ProviderRecommendationStatus::NeedPaidProvider
    }) {
        actions.push(
            "use Alpaca paid SIP or Polygon when full-market realtime US coverage is required"
                .to_string(),
        );
    }
    if recommendations.iter().any(|recommendation| {
        recommendation
            .research_fallbacks
            .iter()
            .any(|fallback| fallback == "yfinance")
    }) {
        actions
            .push("keep yfinance in research-only comparison or prototype workflows".to_string());
    }
    if actions.is_empty() {
        actions
            .push("no additional operator action inferred beyond bounded local setup".to_string());
    }
    actions
}

fn build_final_summary(
    entitlement_statuses: &[ProviderEntitlementStatus],
    compatibility_results: &[StrategyDataCompatibilityResult],
    recommendations: &[ProviderRecommendation],
) -> Vec<ProviderRealitySummary> {
    let mut summary = Vec::new();
    if entitlement_statuses.iter().any(|status| {
        status.provider_subject == ProviderDataSubject::Provider(ProviderKind::KrxOpenApi)
            && status.status == ProviderEntitlementStatusKind::MissingApproval
    }) {
        summary.push(ProviderRealitySummary::KRXApprovalPending);
    }
    if entitlement_statuses.iter().any(|status| {
        status.provider_subject == ProviderDataSubject::Provider(ProviderKind::AlphaVantage)
            && status.status == ProviderEntitlementStatusKind::ReadyForEodResearch
    }) {
        summary.push(ProviderRealitySummary::AlphaVantageEodOnly);
    }
    if recommendations.iter().any(|recommendation| {
        recommendation.primary_provider == Some(ProviderDataSubject::Provider(ProviderKind::Alpaca))
    }) {
        summary.push(ProviderRealitySummary::AlpacaNeededForRealtime);
    }
    if recommendations.iter().any(|recommendation| {
        recommendation.status == ProviderRecommendationStatus::NeedPaidProvider
    }) {
        summary.push(ProviderRealitySummary::PaidProviderNeededForFullMarketRealtime);
    }
    if entitlement_statuses.iter().any(|status| {
        status.provider_subject == ProviderDataSubject::Provider(ProviderKind::Upbit)
            && matches!(
                status.status,
                ProviderEntitlementStatusKind::ReadyForCryptoResearch
                    | ProviderEntitlementStatusKind::ReadyForRealtimeResearch
            )
    }) || compatibility_results.iter().any(|result| {
        result.provider_subject == ProviderDataSubject::Provider(ProviderKind::Upbit)
            && result.compatible
    }) {
        summary.push(ProviderRealitySummary::UpbitReady);
        summary.push(ProviderRealitySummary::ReadyForCryptoResearch);
    }
    if entitlement_statuses
        .iter()
        .any(|status| status.provider_subject == ProviderDataSubject::YFinanceResearch)
    {
        summary.push(ProviderRealitySummary::YFinanceResearchOnly);
    }
    if entitlement_statuses.iter().any(|status| {
        matches!(
            status.status,
            ProviderEntitlementStatusKind::MissingAuth
                | ProviderEntitlementStatusKind::MissingEndpointTemplate
        )
    }) {
        summary.push(ProviderRealitySummary::NeedProviderAuthSetup);
    }
    if entitlement_statuses
        .iter()
        .any(|status| status.status == ProviderEntitlementStatusKind::ReadyForEodResearch)
    {
        summary.push(ProviderRealitySummary::ReadyForEodResearch);
    }
    summary.sort_by_key(|value| *value as u8);
    summary.dedup();
    summary
}

fn default_recommendations() -> Vec<ProviderRecommendationRequest> {
    vec![
        ProviderRecommendationRequest {
            market: crate::data::ProviderMarket::KoreanEquity,
            desired_use_case: StrategyUseCase::EodSwing,
            budget_preference: BudgetPreference::FreeOnly,
            need_realtime: false,
            need_official_readiness: true,
            max_data_size_preference: Some("compact".to_string()),
            reason_codes: vec![ReasonCode::ProviderRecommendationBuilt],
        },
        ProviderRecommendationRequest {
            market: crate::data::ProviderMarket::USEquity,
            desired_use_case: StrategyUseCase::RealtimeScalping,
            budget_preference: BudgetPreference::FreeOnly,
            need_realtime: true,
            need_official_readiness: false,
            max_data_size_preference: Some("compact".to_string()),
            reason_codes: vec![ReasonCode::ProviderRecommendationBuilt],
        },
    ]
}

fn default_true() -> bool {
    true
}
