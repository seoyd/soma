use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;

use super::ProviderKind;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderAuthEnvRequirement {
    pub provider_kind: ProviderKind,
    #[serde(default)]
    pub api_key_env_var: Option<String>,
    #[serde(default)]
    pub api_secret_env_var: Option<String>,
    #[serde(default)]
    pub endpoint_template_env_var: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProviderAuthPreflightConfig {
    pub check_id: String,
    pub providers_to_check: Vec<ProviderKind>,
    #[serde(default)]
    pub required_env_vars: Vec<ProviderAuthEnvRequirement>,
    #[serde(default = "default_true")]
    pub allow_missing_optional_auth: bool,
    #[serde(default)]
    pub fail_on_missing_required_auth: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderAuthStatusKind {
    Ready,
    MissingAuth,
    MissingEndpointTemplate,
    NotRequired,
    Deferred,
    UnsafeSecretExposure,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProviderAuthStatus {
    pub provider_kind: ProviderKind,
    pub auth_required: bool,
    pub env_var_names: Vec<String>,
    pub env_vars_present: bool,
    pub endpoint_template_present: bool,
    pub secret_values_exposed: bool,
    pub status: ProviderAuthStatusKind,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProviderAuthPreflightReport {
    pub check_id: String,
    pub statuses: Vec<ProviderAuthStatus>,
    pub ready_providers: Vec<String>,
    pub missing_auth_providers: Vec<String>,
    pub missing_endpoint_providers: Vec<String>,
    pub deferred_providers: Vec<String>,
    pub safe_to_collect: bool,
    pub warnings: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProviderAuthPreflightRunner;

impl Default for ProviderAuthPreflightConfig {
    fn default() -> Self {
        Self {
            check_id: "provider_auth_preflight".to_string(),
            providers_to_check: vec![
                ProviderKind::Upbit,
                ProviderKind::KrxOpenApi,
                ProviderKind::DataGoKrFscStockPrice,
                ProviderKind::AlphaVantage,
                ProviderKind::Alpaca,
                ProviderKind::KoreaInvestmentMarketData,
            ],
            required_env_vars: vec![],
            allow_missing_optional_auth: true,
            fail_on_missing_required_auth: false,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

impl ProviderAuthPreflightConfig {
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

impl ProviderAuthPreflightRunner {
    pub fn run(&self, config: &ProviderAuthPreflightConfig) -> ProviderAuthPreflightReport {
        let mut statuses = config
            .providers_to_check
            .iter()
            .copied()
            .map(|provider_kind| evaluate_provider(config, provider_kind))
            .collect::<Vec<_>>();
        statuses.sort_by(|left, right| {
            provider_name(left.provider_kind).cmp(provider_name(right.provider_kind))
        });

        let mut ready_providers = Vec::new();
        let mut missing_auth_providers = Vec::new();
        let mut missing_endpoint_providers = Vec::new();
        let mut deferred_providers = Vec::new();
        let mut warnings = Vec::new();
        let mut reason_codes = vec![ReasonCode::ProviderAuthPreflightBuilt];
        let mut safe_to_collect = true;

        for status in &statuses {
            let provider = provider_name(status.provider_kind).to_string();
            match status.status {
                ProviderAuthStatusKind::Ready | ProviderAuthStatusKind::NotRequired => {
                    ready_providers.push(provider);
                }
                ProviderAuthStatusKind::MissingAuth => {
                    missing_auth_providers.push(provider.clone());
                    if config.fail_on_missing_required_auth && status.auth_required {
                        safe_to_collect = false;
                    }
                    if !config.allow_missing_optional_auth || status.auth_required {
                        warnings.push(format!("{provider} auth is missing"));
                    } else {
                        reason_codes.push(ReasonCode::ProviderAuthOptionalMissing);
                    }
                }
                ProviderAuthStatusKind::MissingEndpointTemplate => {
                    missing_endpoint_providers.push(provider.clone());
                    if config.fail_on_missing_required_auth && status.auth_required {
                        safe_to_collect = false;
                    }
                    warnings.push(format!("{provider} endpoint template is missing"));
                }
                ProviderAuthStatusKind::Deferred => {
                    deferred_providers.push(provider);
                }
                ProviderAuthStatusKind::UnsafeSecretExposure => {
                    safe_to_collect = false;
                    warnings.push(format!(
                        "{provider} auth config looks like a raw secret value"
                    ));
                }
            }
            reason_codes.extend(status.reason_codes.iter().cloned());
        }

        ProviderAuthPreflightReport {
            check_id: config.check_id.clone(),
            statuses,
            ready_providers,
            missing_auth_providers,
            missing_endpoint_providers,
            deferred_providers,
            safe_to_collect,
            warnings,
            reason_codes: dedupe_reasons(reason_codes),
        }
    }
}

impl ProviderAuthPreflightReport {
    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("check_id={}", self.check_id),
            format!("ready_providers={}", self.ready_providers.join("|")),
            format!(
                "missing_auth_providers={}",
                self.missing_auth_providers.join("|")
            ),
            format!(
                "missing_endpoint_providers={}",
                self.missing_endpoint_providers.join("|")
            ),
            format!("deferred_providers={}", self.deferred_providers.join("|")),
            format!("safe_to_collect={}", self.safe_to_collect),
            format!("warnings={}", self.warnings.join(" | ")),
        ];
        for status in &self.statuses {
            lines.push(format!(
                "provider={};status={:?};env_var_names={};env_vars_present={};endpoint_template_present={};secret_values_exposed={}",
                provider_name(status.provider_kind),
                status.status,
                status.env_var_names.join("|"),
                status.env_vars_present,
                status.endpoint_template_present,
                status.secret_values_exposed,
            ));
        }
        lines.join("\n")
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        let json_path = output_dir.join("provider_auth_preflight_report.json");
        fs::write(&json_path, self.to_json_string()?).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("provider_auth_preflight_report.txt"),
            self.to_text(),
        )
        .map_err(|err| err.to_string())?;
        Ok(json_path)
    }
}

fn evaluate_provider(
    config: &ProviderAuthPreflightConfig,
    provider_kind: ProviderKind,
) -> ProviderAuthStatus {
    let requirement = config
        .required_env_vars
        .iter()
        .find(|requirement| requirement.provider_kind == provider_kind)
        .cloned()
        .unwrap_or_else(|| default_requirement(provider_kind));
    let env_var_names = [
        requirement.api_key_env_var.clone(),
        requirement.api_secret_env_var.clone(),
        requirement.endpoint_template_env_var.clone(),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();
    let secret_values_exposed = env_var_names
        .iter()
        .any(|value| !looks_like_env_var_name(value));
    let auth_required = !matches!(
        provider_kind,
        ProviderKind::Upbit | ProviderKind::MockFixture | ProviderKind::Binance
    );
    let endpoint_template_present = requirement
        .endpoint_template_env_var
        .as_deref()
        .map(env_var_present)
        .unwrap_or(!endpoint_template_required(provider_kind));
    let env_vars_present = required_env_var_names(&requirement, provider_kind)
        .iter()
        .all(|env_var| env_var_present(env_var));

    let status = if secret_values_exposed {
        ProviderAuthStatusKind::UnsafeSecretExposure
    } else if matches!(
        provider_kind,
        ProviderKind::Upbit | ProviderKind::MockFixture | ProviderKind::Binance
    ) {
        ProviderAuthStatusKind::NotRequired
    } else if matches!(
        provider_kind,
        ProviderKind::Alpaca
            | ProviderKind::KoreaInvestmentMarketData
            | ProviderKind::PolygonProfessional
            | ProviderKind::NasdaqDataLink
            | ProviderKind::KoscomProfessional
    ) && !env_vars_present
        && config.allow_missing_optional_auth
    {
        ProviderAuthStatusKind::Deferred
    } else if endpoint_template_required(provider_kind) && !endpoint_template_present {
        ProviderAuthStatusKind::MissingEndpointTemplate
    } else if !env_vars_present {
        ProviderAuthStatusKind::MissingAuth
    } else {
        ProviderAuthStatusKind::Ready
    };

    let mut reason_codes = vec![ReasonCode::ProviderAuthPreflightBuilt];
    match status {
        ProviderAuthStatusKind::Ready
        | ProviderAuthStatusKind::NotRequired
        | ProviderAuthStatusKind::Deferred => {}
        ProviderAuthStatusKind::MissingAuth => reason_codes.push(ReasonCode::MissingAuth),
        ProviderAuthStatusKind::MissingEndpointTemplate => {
            reason_codes.push(ReasonCode::MissingEndpointTemplate)
        }
        ProviderAuthStatusKind::UnsafeSecretExposure => {
            reason_codes.push(ReasonCode::UnsafeSecretExposure)
        }
    }

    ProviderAuthStatus {
        provider_kind,
        auth_required,
        env_var_names,
        env_vars_present,
        endpoint_template_present,
        secret_values_exposed,
        status,
        reason_codes: dedupe_reasons(reason_codes),
    }
}

fn required_env_var_names(
    requirement: &ProviderAuthEnvRequirement,
    provider_kind: ProviderKind,
) -> Vec<String> {
    let mut env_vars = Vec::new();
    match provider_kind {
        ProviderKind::Upbit
        | ProviderKind::MockFixture
        | ProviderKind::Binance
        | ProviderKind::Korbit => {}
        ProviderKind::KrxOpenApi
        | ProviderKind::DataGoKrFscStockPrice
        | ProviderKind::AlphaVantage
        | ProviderKind::PolygonProfessional
        | ProviderKind::NasdaqDataLink
        | ProviderKind::KoscomProfessional
        | ProviderKind::KoreaInvestmentMarketData => {
            if let Some(value) = requirement.api_key_env_var.clone() {
                env_vars.push(value);
            }
        }
        ProviderKind::Alpaca => {
            if let Some(value) = requirement.api_key_env_var.clone() {
                env_vars.push(value);
            }
            if let Some(value) = requirement.api_secret_env_var.clone() {
                env_vars.push(value);
            }
        }
        ProviderKind::Unknown => {}
    }
    env_vars
}

fn default_requirement(provider_kind: ProviderKind) -> ProviderAuthEnvRequirement {
    match provider_kind {
        ProviderKind::Upbit
        | ProviderKind::Binance
        | ProviderKind::Korbit
        | ProviderKind::MockFixture
        | ProviderKind::Unknown => ProviderAuthEnvRequirement {
            provider_kind,
            api_key_env_var: None,
            api_secret_env_var: None,
            endpoint_template_env_var: None,
        },
        ProviderKind::KrxOpenApi => ProviderAuthEnvRequirement {
            provider_kind,
            api_key_env_var: Some("KRX_API_KEY".to_string()),
            api_secret_env_var: None,
            endpoint_template_env_var: Some("KRX_ENDPOINT_TEMPLATE".to_string()),
        },
        ProviderKind::DataGoKrFscStockPrice => ProviderAuthEnvRequirement {
            provider_kind,
            api_key_env_var: Some("DATA_GO_KR_SERVICE_KEY".to_string()),
            api_secret_env_var: None,
            endpoint_template_env_var: None,
        },
        ProviderKind::AlphaVantage => ProviderAuthEnvRequirement {
            provider_kind,
            api_key_env_var: Some("ALPHAVANTAGE_API_KEY".to_string()),
            api_secret_env_var: None,
            endpoint_template_env_var: None,
        },
        ProviderKind::Alpaca => ProviderAuthEnvRequirement {
            provider_kind,
            api_key_env_var: Some("ALPACA_API_KEY_ID".to_string()),
            api_secret_env_var: Some("ALPACA_API_SECRET_KEY".to_string()),
            endpoint_template_env_var: None,
        },
        ProviderKind::KoreaInvestmentMarketData => ProviderAuthEnvRequirement {
            provider_kind,
            api_key_env_var: Some("KIS_APP_KEY".to_string()),
            api_secret_env_var: Some("KIS_APP_SECRET".to_string()),
            endpoint_template_env_var: None,
        },
        ProviderKind::PolygonProfessional => ProviderAuthEnvRequirement {
            provider_kind,
            api_key_env_var: Some("POLYGON_API_KEY".to_string()),
            api_secret_env_var: None,
            endpoint_template_env_var: None,
        },
        ProviderKind::NasdaqDataLink => ProviderAuthEnvRequirement {
            provider_kind,
            api_key_env_var: Some("NASDAQ_DATA_LINK_API_KEY".to_string()),
            api_secret_env_var: None,
            endpoint_template_env_var: None,
        },
        ProviderKind::KoscomProfessional => ProviderAuthEnvRequirement {
            provider_kind,
            api_key_env_var: Some("KOSCOM_API_KEY".to_string()),
            api_secret_env_var: None,
            endpoint_template_env_var: None,
        },
    }
}

fn endpoint_template_required(provider_kind: ProviderKind) -> bool {
    matches!(provider_kind, ProviderKind::KrxOpenApi)
}

fn env_var_present(name: &str) -> bool {
    env::var_os(name)
        .map(|value| !value.is_empty())
        .unwrap_or(false)
}

fn looks_like_env_var_name(value: &str) -> bool {
    !value.is_empty()
        && value.chars().all(|character| {
            character.is_ascii_uppercase() || character.is_ascii_digit() || character == '_'
        })
}

fn provider_name(provider_kind: ProviderKind) -> &'static str {
    match provider_kind {
        ProviderKind::Upbit => "upbit",
        ProviderKind::Binance => "binance",
        ProviderKind::Korbit => "korbit",
        ProviderKind::KrxOpenApi => "krx",
        ProviderKind::DataGoKrFscStockPrice => "data-go-kr-fsc-stock-price",
        ProviderKind::AlphaVantage => "alphavantage",
        ProviderKind::Alpaca => "alpaca",
        ProviderKind::KoreaInvestmentMarketData => "kis-market-data",
        ProviderKind::PolygonProfessional => "polygon",
        ProviderKind::NasdaqDataLink => "nasdaq-data-link",
        ProviderKind::KoscomProfessional => "koscom",
        ProviderKind::MockFixture => "mock-fixture",
        ProviderKind::Unknown => "unknown",
    }
}

fn default_true() -> bool {
    true
}

fn dedupe_reasons(values: Vec<ReasonCode>) -> Vec<ReasonCode> {
    let mut deduped = Vec::new();
    for value in values {
        if !deduped.contains(&value) {
            deduped.push(value);
        }
    }
    deduped
}
