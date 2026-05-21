use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;
use crate::data::{
    MarketDataProviderCatalog, ProviderCredentialProfile, ProviderCredentialStatus, ProviderKind,
    ProviderMarket, ProviderSelectionPolicy, ProviderSelectionResult,
    ProviderSelectionResultStatus, build_default_provider_catalog,
    default_provider_credential_profiles, default_provider_selection_policies,
    evaluate_provider_credential_profiles, select_provider,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OfficialProviderReadinessStatus {
    ReadyForCryptoEvidence,
    ReadyForKoreanEquityEvidence,
    ReadyForUSEquityEvidence,
    ReadyForMultiVenueEvidence,
    MissingKoreanAuth,
    MissingUSAuth,
    MissingProviderEndpointProfile,
    ResearchOnlyFallback,
    NeedAuthSetup,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfficialProviderReadinessConfig {
    pub report_id: String,
    pub output_dir: String,
    #[serde(default = "default_markets")]
    pub markets: Vec<ProviderMarket>,
    #[serde(default = "default_true")]
    pub allow_research_supplemental: bool,
    #[serde(default = "default_true")]
    pub allow_professional_paid: bool,
    #[serde(default = "default_max_providers_per_market")]
    pub max_providers_per_market: usize,
    #[serde(default)]
    pub credential_profile_overrides: Vec<ProviderCredentialProfile>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfficialProviderReadinessReport {
    pub report_id: String,
    pub catalog: MarketDataProviderCatalog,
    pub credential_statuses: Vec<ProviderCredentialStatus>,
    pub selection_results: Vec<ProviderSelectionResult>,
    pub implemented_providers: Vec<String>,
    pub missing_auth_actions: Vec<String>,
    pub deferred_provider_actions: Vec<String>,
    pub official_ready_markets: Vec<ProviderMarket>,
    pub research_only_markets: Vec<ProviderMarket>,
    pub final_status: OfficialProviderReadinessStatus,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OfficialProviderReadinessRunner;

impl Default for OfficialProviderReadinessConfig {
    fn default() -> Self {
        Self {
            report_id: "provider_readiness".to_string(),
            output_dir: "target/soma_provider_readiness".to_string(),
            markets: default_markets(),
            allow_research_supplemental: true,
            allow_professional_paid: true,
            max_providers_per_market: default_max_providers_per_market(),
            credential_profile_overrides: Vec::new(),
            reason_codes: vec![ReasonCode::ProviderReadinessReportBuilt],
        }
    }
}

impl OfficialProviderReadinessConfig {
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

impl OfficialProviderReadinessRunner {
    pub fn run(&self, config: &OfficialProviderReadinessConfig) -> OfficialProviderReadinessReport {
        let catalog = build_default_provider_catalog();
        let credential_statuses =
            evaluate_provider_credential_profiles(&apply_credential_overrides(config));
        let mut selection_results = default_provider_selection_policies()
            .into_iter()
            .filter(|policy| config.markets.contains(&policy.market))
            .map(|mut policy| {
                apply_policy_overrides(&mut policy, config);
                select_provider(&catalog, &credential_statuses, &policy)
            })
            .collect::<Vec<_>>();
        selection_results.sort_by_key(|result| market_rank(result.market));

        let implemented_providers = catalog
            .providers
            .iter()
            .filter(|entry| {
                !matches!(
                    entry.implemented_status,
                    crate::data::ProviderImplementedStatus::Deferred
                        | crate::data::ProviderImplementedStatus::DocumentedOnly
                )
            })
            .map(|entry| entry.provider_name.clone())
            .collect::<Vec<_>>();

        let official_ready_markets = selection_results
            .iter()
            .filter(|result| result.status == ProviderSelectionResultStatus::Selected)
            .map(|result| result.market)
            .collect::<Vec<_>>();
        let research_only_markets = selection_results
            .iter()
            .filter(|result| result.status == ProviderSelectionResultStatus::ResearchOnlyFallback)
            .map(|result| result.market)
            .collect::<Vec<_>>();

        let missing_auth_actions =
            build_missing_auth_actions(&selection_results, &credential_statuses);
        let deferred_provider_actions = build_deferred_actions(&selection_results);
        let final_status = classify_final_status(&selection_results);

        OfficialProviderReadinessReport {
            report_id: config.report_id.clone(),
            catalog,
            credential_statuses,
            selection_results,
            implemented_providers,
            missing_auth_actions,
            deferred_provider_actions,
            official_ready_markets,
            research_only_markets,
            final_status,
            reason_codes: vec![
                ReasonCode::ProviderReadinessReportBuilt,
                ReasonCode::DeterministicPath,
            ],
        }
    }
}

fn apply_credential_overrides(
    config: &OfficialProviderReadinessConfig,
) -> Vec<ProviderCredentialProfile> {
    let mut profiles = default_provider_credential_profiles();
    for override_profile in &config.credential_profile_overrides {
        if let Some(existing) = profiles
            .iter_mut()
            .find(|profile| profile.provider_kind == override_profile.provider_kind)
        {
            *existing = override_profile.clone();
        } else {
            profiles.push(override_profile.clone());
        }
    }
    profiles
}

impl OfficialProviderReadinessReport {
    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("report_id={}", self.report_id),
            format!("final_status={:?}", self.final_status),
            format!(
                "official_ready_markets={}",
                self.official_ready_markets
                    .iter()
                    .map(|market| format!("{market:?}"))
                    .collect::<Vec<_>>()
                    .join("|")
            ),
            format!(
                "research_only_markets={}",
                self.research_only_markets
                    .iter()
                    .map(|market| format!("{market:?}"))
                    .collect::<Vec<_>>()
                    .join("|")
            ),
        ];
        for result in &self.selection_results {
            lines.push(format!(
                "market={:?};status={:?};selected={};fallback={};missing_auth={};deferred={};documented_only={}",
                result.market,
                result.status,
                result
                    .selected_provider
                    .map(provider_label)
                    .unwrap_or("".to_string()),
                result
                    .fallback_selected
                    .map(provider_label)
                    .unwrap_or("".to_string()),
                result
                    .missing_auth_providers
                    .iter()
                    .copied()
                    .map(provider_label)
                    .collect::<Vec<_>>()
                    .join("|"),
                result
                    .deferred_providers
                    .iter()
                    .copied()
                    .map(provider_label)
                    .collect::<Vec<_>>()
                    .join("|"),
                result
                    .documented_only_providers
                    .iter()
                    .copied()
                    .map(provider_label)
                    .collect::<Vec<_>>()
                    .join("|"),
            ));
        }
        for action in &self.missing_auth_actions {
            lines.push(format!("missing_auth_action={action}"));
        }
        for action in &self.deferred_provider_actions {
            lines.push(format!("deferred_action={action}"));
        }
        lines.join("\n")
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        let json_path = output_dir.join("provider_readiness_report.json");
        fs::write(&json_path, self.to_json_string()?).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("provider_readiness_report.txt"),
            self.to_text(),
        )
        .map_err(|err| err.to_string())?;
        Ok(json_path)
    }
}

fn apply_policy_overrides(
    policy: &mut ProviderSelectionPolicy,
    config: &OfficialProviderReadinessConfig,
) {
    policy.allow_research_supplemental = config.allow_research_supplemental;
    policy.allow_professional_paid = config.allow_professional_paid;
    policy.max_providers_per_market = config.max_providers_per_market;
}

fn build_missing_auth_actions(
    selection_results: &[ProviderSelectionResult],
    credential_statuses: &[ProviderCredentialStatus],
) -> Vec<String> {
    let providers = selection_results
        .iter()
        .flat_map(|result| result.missing_auth_providers.iter().copied())
        .collect::<BTreeSet<_>>();
    providers
        .into_iter()
        .filter_map(|provider_kind| {
            let status = credential_statuses
                .iter()
                .find(|status| status.provider_kind == provider_kind)?;
            let mut required = status.required_env_vars.clone();
            required.extend(status.endpoint_template_env_vars.clone());
            Some(format!(
                "{}: set {} locally; never store or print secret values",
                provider_label(provider_kind),
                required.join(", ")
            ))
        })
        .collect()
}

fn build_deferred_actions(selection_results: &[ProviderSelectionResult]) -> Vec<String> {
    let providers = selection_results
        .iter()
        .flat_map(|result| {
            result
                .deferred_providers
                .iter()
                .chain(result.documented_only_providers.iter())
        })
        .copied()
        .collect::<BTreeSet<_>>();
    providers
        .into_iter()
        .map(|provider_kind| match provider_kind {
            ProviderKind::DataGoKrFscStockPrice => {
                "data-go-kr-fsc-stock-price: confirm approved endpoint profile before live collection"
                    .to_string()
            }
            ProviderKind::PolygonProfessional => {
                "polygon: provider card only; add fixture-backed implementation before collection"
                    .to_string()
            }
            ProviderKind::NasdaqDataLink => {
                "nasdaq-data-link: provider card only; add fixture-backed implementation before collection"
                    .to_string()
            }
            ProviderKind::KoscomProfessional => {
                "koscom: provider card only; confirm commercial onboarding separately".to_string()
            }
            ProviderKind::Binance => "binance: deferred optional crypto fallback".to_string(),
            ProviderKind::Korbit => "korbit: deferred optional crypto fallback".to_string(),
            other => format!("{}: keep bounded, research-only setup until upgraded", provider_label(other)),
        })
        .collect()
}

fn classify_final_status(
    selection_results: &[ProviderSelectionResult],
) -> OfficialProviderReadinessStatus {
    let korean = selection_results
        .iter()
        .find(|result| result.market == ProviderMarket::KoreanEquity);
    let us = selection_results
        .iter()
        .find(|result| result.market == ProviderMarket::USEquity);
    let crypto = selection_results
        .iter()
        .find(|result| result.market == ProviderMarket::Crypto);

    if korean.is_some_and(|result| {
        result.status == ProviderSelectionResultStatus::MissingAuth
            || (result.selected_provider.is_none() && !result.missing_auth_providers.is_empty())
    }) {
        return OfficialProviderReadinessStatus::MissingKoreanAuth;
    }
    if us.is_some_and(|result| {
        result.status == ProviderSelectionResultStatus::MissingAuth
            || (result.selected_provider.is_none() && !result.missing_auth_providers.is_empty())
    }) {
        return OfficialProviderReadinessStatus::MissingUSAuth;
    }
    if selection_results.iter().any(|result| {
        result.selected_provider == Some(ProviderKind::DataGoKrFscStockPrice)
            || result.status == ProviderSelectionResultStatus::Deferred
    }) {
        return OfficialProviderReadinessStatus::MissingProviderEndpointProfile;
    }
    if selection_results
        .iter()
        .any(|result| result.status == ProviderSelectionResultStatus::ResearchOnlyFallback)
    {
        return OfficialProviderReadinessStatus::ResearchOnlyFallback;
    }

    let has_crypto =
        crypto.is_some_and(|result| result.status == ProviderSelectionResultStatus::Selected);
    let has_korean =
        korean.is_some_and(|result| result.status == ProviderSelectionResultStatus::Selected);
    let has_us = us.is_some_and(|result| result.status == ProviderSelectionResultStatus::Selected);
    if has_crypto && has_korean && has_us {
        OfficialProviderReadinessStatus::ReadyForMultiVenueEvidence
    } else if has_korean {
        OfficialProviderReadinessStatus::ReadyForKoreanEquityEvidence
    } else if has_us {
        OfficialProviderReadinessStatus::ReadyForUSEquityEvidence
    } else if has_crypto {
        OfficialProviderReadinessStatus::ReadyForCryptoEvidence
    } else {
        OfficialProviderReadinessStatus::NeedAuthSetup
    }
}

fn default_markets() -> Vec<ProviderMarket> {
    vec![
        ProviderMarket::Crypto,
        ProviderMarket::KoreanEquity,
        ProviderMarket::USEquity,
    ]
}

fn default_true() -> bool {
    true
}

fn default_max_providers_per_market() -> usize {
    4
}

fn market_rank(market: ProviderMarket) -> usize {
    match market {
        ProviderMarket::Crypto => 0,
        ProviderMarket::KoreanEquity => 1,
        ProviderMarket::USEquity => 2,
        ProviderMarket::GlobalEquity => 3,
    }
}

fn provider_label(provider_kind: ProviderKind) -> String {
    match provider_kind {
        ProviderKind::Upbit => "upbit".to_string(),
        ProviderKind::Binance => "binance".to_string(),
        ProviderKind::Korbit => "korbit".to_string(),
        ProviderKind::KrxOpenApi => "krx".to_string(),
        ProviderKind::DataGoKrFscStockPrice => "data-go-kr-fsc-stock-price".to_string(),
        ProviderKind::AlphaVantage => "alphavantage".to_string(),
        ProviderKind::Alpaca => "alpaca".to_string(),
        ProviderKind::KoreaInvestmentMarketData => "kis-market-data-only".to_string(),
        ProviderKind::PolygonProfessional => "polygon".to_string(),
        ProviderKind::NasdaqDataLink => "nasdaq-data-link".to_string(),
        ProviderKind::KoscomProfessional => "koscom".to_string(),
        ProviderKind::MockFixture => "mock-fixture".to_string(),
        ProviderKind::Unknown => "unknown".to_string(),
    }
}
