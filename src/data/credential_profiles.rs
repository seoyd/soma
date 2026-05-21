use std::env;

use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;

use super::ProviderKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderSecretValuePolicy {
    EnvVarNameOnly,
    NeverPersistSecret,
    NeverPrintSecret,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderAuthCheckMode {
    PresenceOnly,
    EndpointTemplateRequired,
    Deferred,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderCredentialStatusKind {
    Ready,
    MissingAuth,
    MissingEndpointTemplate,
    NotRequired,
    Deferred,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderCredentialProfile {
    pub provider_kind: ProviderKind,
    pub required_env_vars: Vec<String>,
    pub optional_env_vars: Vec<String>,
    pub endpoint_template_env_vars: Vec<String>,
    pub secret_value_policy: Vec<ProviderSecretValuePolicy>,
    pub auth_check_mode: ProviderAuthCheckMode,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderCredentialStatus {
    pub provider_kind: ProviderKind,
    pub required_env_vars: Vec<String>,
    pub optional_env_vars: Vec<String>,
    pub endpoint_template_env_vars: Vec<String>,
    pub missing_required_env_vars: Vec<String>,
    pub missing_endpoint_template_env_vars: Vec<String>,
    pub status: ProviderCredentialStatusKind,
    pub reason_codes: Vec<ReasonCode>,
}

pub fn default_provider_credential_profiles() -> Vec<ProviderCredentialProfile> {
    vec![
        profile(
            ProviderKind::Upbit,
            &[],
            &[],
            &[],
            ProviderAuthCheckMode::PresenceOnly,
        ),
        profile(
            ProviderKind::KrxOpenApi,
            &["KRX_API_KEY"],
            &[],
            &["KRX_ENDPOINT_TEMPLATE"],
            ProviderAuthCheckMode::EndpointTemplateRequired,
        ),
        profile(
            ProviderKind::DataGoKrFscStockPrice,
            &["DATA_GO_KR_SERVICE_KEY"],
            &[],
            &[],
            ProviderAuthCheckMode::PresenceOnly,
        ),
        profile(
            ProviderKind::KoreaInvestmentMarketData,
            &["KIS_APP_KEY", "KIS_APP_SECRET"],
            &["KIS_BASE_URL"],
            &[],
            ProviderAuthCheckMode::PresenceOnly,
        ),
        profile(
            ProviderKind::AlphaVantage,
            &["ALPHAVANTAGE_API_KEY"],
            &[],
            &[],
            ProviderAuthCheckMode::PresenceOnly,
        ),
        profile(
            ProviderKind::Alpaca,
            &["ALPACA_API_KEY_ID", "ALPACA_API_SECRET_KEY"],
            &[],
            &[],
            ProviderAuthCheckMode::PresenceOnly,
        ),
        profile(
            ProviderKind::PolygonProfessional,
            &["POLYGON_API_KEY"],
            &[],
            &[],
            ProviderAuthCheckMode::PresenceOnly,
        ),
        profile(
            ProviderKind::NasdaqDataLink,
            &["NASDAQ_DATA_LINK_API_KEY"],
            &[],
            &[],
            ProviderAuthCheckMode::PresenceOnly,
        ),
        profile(
            ProviderKind::KoscomProfessional,
            &["KOSCOM_API_KEY"],
            &[],
            &[],
            ProviderAuthCheckMode::Deferred,
        ),
        profile(
            ProviderKind::Binance,
            &[],
            &[],
            &[],
            ProviderAuthCheckMode::Deferred,
        ),
        profile(
            ProviderKind::Korbit,
            &[],
            &[],
            &[],
            ProviderAuthCheckMode::Deferred,
        ),
        profile(
            ProviderKind::MockFixture,
            &[],
            &[],
            &[],
            ProviderAuthCheckMode::PresenceOnly,
        ),
    ]
}

pub fn evaluate_provider_credential_profiles(
    profiles: &[ProviderCredentialProfile],
) -> Vec<ProviderCredentialStatus> {
    let mut statuses = profiles
        .iter()
        .map(evaluate_provider_credential_profile)
        .collect::<Vec<_>>();
    statuses.sort_by_key(|status| provider_rank(status.provider_kind));
    statuses
}

pub fn evaluate_provider_credential_profile(
    profile: &ProviderCredentialProfile,
) -> ProviderCredentialStatus {
    let missing_required_env_vars = profile
        .required_env_vars
        .iter()
        .filter(|name| !env_var_present(name))
        .cloned()
        .collect::<Vec<_>>();
    let missing_endpoint_template_env_vars = profile
        .endpoint_template_env_vars
        .iter()
        .filter(|name| !env_var_present(name))
        .cloned()
        .collect::<Vec<_>>();
    let status = if profile.required_env_vars.is_empty()
        && profile.optional_env_vars.is_empty()
        && profile.endpoint_template_env_vars.is_empty()
        && matches!(
            profile.provider_kind,
            ProviderKind::Upbit | ProviderKind::MockFixture
        ) {
        ProviderCredentialStatusKind::NotRequired
    } else if profile.auth_check_mode == ProviderAuthCheckMode::Deferred {
        ProviderCredentialStatusKind::Deferred
    } else if !missing_endpoint_template_env_vars.is_empty()
        && profile.auth_check_mode == ProviderAuthCheckMode::EndpointTemplateRequired
    {
        ProviderCredentialStatusKind::MissingEndpointTemplate
    } else if !missing_required_env_vars.is_empty() {
        ProviderCredentialStatusKind::MissingAuth
    } else {
        ProviderCredentialStatusKind::Ready
    };

    let mut reason_codes = vec![ReasonCode::ProviderCredentialProfileBuilt];
    match status {
        ProviderCredentialStatusKind::Ready | ProviderCredentialStatusKind::NotRequired => {}
        ProviderCredentialStatusKind::MissingAuth => reason_codes.push(ReasonCode::MissingAuth),
        ProviderCredentialStatusKind::MissingEndpointTemplate => {
            reason_codes.push(ReasonCode::MissingEndpointTemplate)
        }
        ProviderCredentialStatusKind::Deferred => {
            reason_codes.push(ReasonCode::ProviderAuthOptionalMissing)
        }
    }

    ProviderCredentialStatus {
        provider_kind: profile.provider_kind,
        required_env_vars: profile.required_env_vars.clone(),
        optional_env_vars: profile.optional_env_vars.clone(),
        endpoint_template_env_vars: profile.endpoint_template_env_vars.clone(),
        missing_required_env_vars,
        missing_endpoint_template_env_vars,
        status,
        reason_codes,
    }
}

fn profile(
    provider_kind: ProviderKind,
    required_env_vars: &[&str],
    optional_env_vars: &[&str],
    endpoint_template_env_vars: &[&str],
    auth_check_mode: ProviderAuthCheckMode,
) -> ProviderCredentialProfile {
    ProviderCredentialProfile {
        provider_kind,
        required_env_vars: required_env_vars
            .iter()
            .map(|value| value.to_string())
            .collect(),
        optional_env_vars: optional_env_vars
            .iter()
            .map(|value| value.to_string())
            .collect(),
        endpoint_template_env_vars: endpoint_template_env_vars
            .iter()
            .map(|value| value.to_string())
            .collect(),
        secret_value_policy: vec![
            ProviderSecretValuePolicy::EnvVarNameOnly,
            ProviderSecretValuePolicy::NeverPersistSecret,
            ProviderSecretValuePolicy::NeverPrintSecret,
        ],
        auth_check_mode,
        reason_codes: vec![ReasonCode::ProviderCredentialProfileBuilt],
    }
}

fn env_var_present(name: &str) -> bool {
    env::var_os(name)
        .map(|value| !value.is_empty())
        .unwrap_or(false)
}

fn provider_rank(provider_kind: ProviderKind) -> usize {
    match provider_kind {
        ProviderKind::Upbit => 0,
        ProviderKind::Binance => 1,
        ProviderKind::Korbit => 2,
        ProviderKind::KrxOpenApi => 3,
        ProviderKind::DataGoKrFscStockPrice => 4,
        ProviderKind::KoreaInvestmentMarketData => 5,
        ProviderKind::AlphaVantage => 6,
        ProviderKind::Alpaca => 7,
        ProviderKind::PolygonProfessional => 8,
        ProviderKind::NasdaqDataLink => 9,
        ProviderKind::KoscomProfessional => 10,
        ProviderKind::MockFixture => 11,
        ProviderKind::Unknown => 12,
    }
}
