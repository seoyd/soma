use std::collections::{BTreeMap, VecDeque};
use std::env;
use std::fmt;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::backtest::{CandleSeries, Timeframe};
use crate::core::{MarketSnapshot, Regime, RiskSnapshot, Stance};

const REDACTED: &str = "[REDACTED]";
const HEALTH_PATH: &str = "/soma/read-only/health";
const MARKET_SNAPSHOT_PATH: &str = "/soma/read-only/market-snapshot";

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TossApiConfig {
    pub base_url: String,
    pub app_key_env_name: String,
    pub app_secret_env_name: String,
    pub account_id_env_name: Option<String>,
    pub paper_only: bool,
    pub read_only: bool,
    pub timeout_ms: u64,
    pub max_retries: usize,
    pub rate_limit_per_minute: Option<u32>,
}

impl fmt::Debug for TossApiConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TossApiConfig")
            .field("base_url", &redacted_base_url_preview(&self.base_url))
            .field("app_key_env_name", &self.app_key_env_name)
            .field("app_secret_env_name", &self.app_secret_env_name)
            .field("account_id_env_name", &self.account_id_env_name)
            .field("paper_only", &self.paper_only)
            .field("read_only", &self.read_only)
            .field("timeout_ms", &self.timeout_ms)
            .field("max_retries", &self.max_retries)
            .field("rate_limit_per_minute", &self.rate_limit_per_minute)
            .finish()
    }
}

impl Default for TossApiConfig {
    fn default() -> Self {
        Self {
            base_url: "https://replace-me.invalid".to_string(),
            app_key_env_name: "TOSS_APP_KEY".to_string(),
            app_secret_env_name: "TOSS_APP_SECRET".to_string(),
            account_id_env_name: Some("TOSS_ACCOUNT_ID".to_string()),
            paper_only: true,
            read_only: true,
            timeout_ms: 5_000,
            max_retries: 0,
            rate_limit_per_minute: None,
        }
    }
}

impl TossApiConfig {
    pub fn validate(&self) -> Result<(), TossError> {
        let base_url = self.base_url.trim();
        if base_url.is_empty()
            || !base_url.starts_with("https://")
            || base_url.contains('?')
            || base_url.contains('#')
            || base_url.contains('@')
        {
            return Err(TossError::InvalidConfig("base_url"));
        }
        if !valid_env_name(&self.app_key_env_name) {
            return Err(TossError::InvalidConfig("app_key_env_name"));
        }
        if !valid_env_name(&self.app_secret_env_name) {
            return Err(TossError::InvalidConfig("app_secret_env_name"));
        }
        if self
            .account_id_env_name
            .as_deref()
            .is_some_and(|name| !valid_env_name(name))
        {
            return Err(TossError::InvalidConfig("account_id_env_name"));
        }
        if !self.paper_only {
            return Err(TossError::PaperOnlyRequired);
        }
        if !self.read_only {
            return Err(TossError::ReadOnlyRequired);
        }
        if self.timeout_ms == 0 {
            return Err(TossError::InvalidConfig("timeout_ms"));
        }
        if self.rate_limit_per_minute == Some(0) {
            return Err(TossError::InvalidConfig("rate_limit_per_minute"));
        }
        Ok(())
    }
}

fn valid_env_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_')
}

fn redacted_base_url_preview(base_url: &str) -> String {
    let host = base_url
        .strip_prefix("https://")
        .unwrap_or(base_url)
        .split('/')
        .next()
        .unwrap_or_default();
    if host.is_empty() {
        "configured(redacted-host)".to_string()
    } else {
        format!("configured(https;host-length={})", host.len())
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct TossCredentials {
    app_key: String,
    app_secret: String,
    account_id: Option<String>,
}

impl fmt::Debug for TossCredentials {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TossCredentials")
            .field("app_key", &REDACTED)
            .field("app_secret", &REDACTED)
            .field("account_id", &self.account_id.as_ref().map(|_| REDACTED))
            .finish()
    }
}

impl TossCredentials {
    pub fn from_env(config: &TossApiConfig) -> Result<Self, TossError> {
        Self::load_with(config, |name| env::var(name).ok())
    }

    fn load_with(
        config: &TossApiConfig,
        lookup: impl Fn(&str) -> Option<String>,
    ) -> Result<Self, TossError> {
        config.validate()?;
        let app_key = required_env_value(&config.app_key_env_name, &lookup)?;
        let app_secret = required_env_value(&config.app_secret_env_name, &lookup)?;
        let account_id = config
            .account_id_env_name
            .as_deref()
            .and_then(&lookup)
            .filter(|value| !value.trim().is_empty());
        Ok(Self {
            app_key,
            app_secret,
            account_id,
        })
    }

    pub fn redactor(&self, account_id_sensitive: bool) -> SecretRedactor {
        SecretRedactor::from_credentials(self, account_id_sensitive)
    }
}

fn required_env_value(
    name: &str,
    lookup: &impl Fn(&str) -> Option<String>,
) -> Result<String, TossError> {
    lookup(name)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| TossError::MissingCredential {
            env_name: name.to_string(),
        })
}

#[derive(Clone, Default, PartialEq, Eq)]
pub struct SecretRedactor {
    known_values: Vec<String>,
    account_id_sensitive: bool,
}

impl fmt::Debug for SecretRedactor {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SecretRedactor")
            .field("known_value_count", &self.known_values.len())
            .field("account_id_sensitive", &self.account_id_sensitive)
            .finish()
    }
}

impl SecretRedactor {
    pub fn from_credentials(credentials: &TossCredentials, account_id_sensitive: bool) -> Self {
        let mut known_values = vec![credentials.app_key.clone(), credentials.app_secret.clone()];
        if account_id_sensitive {
            if let Some(account_id) = &credentials.account_id {
                known_values.push(account_id.clone());
            }
        }
        known_values.retain(|value| !value.is_empty());
        known_values.sort_by_key(|value| std::cmp::Reverse(value.len()));
        known_values.dedup();
        Self {
            known_values,
            account_id_sensitive,
        }
    }

    pub fn redact_header_value(&self, name: &str, value: &str) -> String {
        if sensitive_header_name(name) || contains_bearer(value) {
            REDACTED.to_string()
        } else {
            self.redact_known_values(value)
        }
    }

    pub fn redact_json_like_text(&self, text: &str) -> String {
        match serde_json::from_str::<Value>(text) {
            Ok(mut value) => {
                self.redact_json_value(None, &mut value);
                serde_json::to_string(&value).unwrap_or_else(|_| REDACTED.to_string())
            }
            Err(_) => self.redact_bearer_tokens(&self.redact_known_values(text)),
        }
    }

    pub fn redact_url_query(&self, url: &str) -> String {
        let Some((base, query_and_fragment)) = url.split_once('?') else {
            return self.redact_known_values(url);
        };
        let (query, fragment) = query_and_fragment
            .split_once('#')
            .map_or((query_and_fragment, None), |(query, fragment)| {
                (query, Some(fragment))
            });
        let redacted_query = query
            .split('&')
            .map(|pair| {
                let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
                let value = if sensitive_key(key, self.account_id_sensitive) {
                    REDACTED.to_string()
                } else {
                    self.redact_known_values(value)
                };
                if pair.contains('=') {
                    format!("{key}={value}")
                } else {
                    key.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("&");
        let mut result = format!("{}?{redacted_query}", self.redact_known_values(base));
        if let Some(fragment) = fragment {
            result.push('#');
            result.push_str(&self.redact_known_values(fragment));
        }
        result
    }

    pub fn safe_debug_string(&self, text: &str) -> String {
        let redacted = self.redact_json_like_text(text);
        self.redact_url_query(&redacted)
    }

    fn redact_json_value(&self, key: Option<&str>, value: &mut Value) {
        if key.is_some_and(|key| sensitive_key(key, self.account_id_sensitive)) {
            *value = Value::String(REDACTED.to_string());
            return;
        }
        match value {
            Value::Object(entries) => {
                for (key, value) in entries {
                    self.redact_json_value(Some(key), value);
                }
            }
            Value::Array(values) => {
                for value in values {
                    self.redact_json_value(None, value);
                }
            }
            Value::String(value) => {
                *value = self.redact_bearer_tokens(&self.redact_known_values(value));
            }
            Value::Null | Value::Bool(_) | Value::Number(_) => {}
        }
    }

    fn redact_known_values(&self, text: &str) -> String {
        self.known_values
            .iter()
            .fold(text.to_string(), |redacted, secret| {
                redacted.replace(secret, REDACTED)
            })
    }

    fn redact_bearer_tokens(&self, text: &str) -> String {
        let lowercase = text.to_ascii_lowercase();
        let mut output = String::with_capacity(text.len());
        let mut cursor = 0;
        while let Some(relative) = lowercase[cursor..].find("bearer ") {
            let start = cursor + relative;
            output.push_str(&text[cursor..start]);
            output.push_str("Bearer ");
            output.push_str(REDACTED);
            let token_start = start + "bearer ".len();
            let token_len = text[token_start..]
                .find(|character: char| {
                    character.is_ascii_whitespace()
                        || matches!(character, '"' | '\'' | ',' | '}' | ']' | '&')
                })
                .unwrap_or(text.len() - token_start);
            cursor = token_start + token_len;
        }
        output.push_str(&text[cursor..]);
        output
    }
}

pub fn redact_header_value(name: &str, value: &str) -> String {
    SecretRedactor::default().redact_header_value(name, value)
}

pub fn redact_json_like_text(text: &str) -> String {
    SecretRedactor::default().redact_json_like_text(text)
}

pub fn redact_url_query(url: &str) -> String {
    SecretRedactor::default().redact_url_query(url)
}

pub fn safe_debug_string(text: &str) -> String {
    SecretRedactor::default().safe_debug_string(text)
}

fn sensitive_header_name(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "authorization"
            | "proxy-authorization"
            | "x-app-key"
            | "x-app-secret"
            | "app-key"
            | "app-secret"
            | "access-token"
    )
}

fn sensitive_key(name: &str, account_id_sensitive: bool) -> bool {
    let normalized = name.to_ascii_lowercase().replace('-', "_");
    matches!(
        normalized.as_str(),
        "app_key"
            | "app_secret"
            | "authorization"
            | "access_token"
            | "bearer_token"
            | "token"
            | "password"
    ) || account_id_sensitive && matches!(normalized.as_str(), "account_id" | "account")
}

fn contains_bearer(value: &str) -> bool {
    value.to_ascii_lowercase().contains("bearer ")
}

pub fn check_fixture_has_no_known_secrets(
    fixture: &str,
    known_secrets: &[&str],
) -> Result<(), TossError> {
    if known_secrets
        .iter()
        .any(|secret| !secret.is_empty() && fixture.contains(secret))
    {
        return Err(TossError::UnsafeFixture("known_secret"));
    }
    Ok(())
}

pub fn check_fixture_has_no_authorization_header(fixture: &str) -> Result<(), TossError> {
    if fixture.to_ascii_lowercase().contains("authorization") {
        return Err(TossError::UnsafeFixture("authorization_header"));
    }
    Ok(())
}

pub fn check_fixture_has_no_secret_field_names(fixture: &str) -> Result<(), TossError> {
    let normalized = fixture.to_ascii_lowercase().replace('-', "_");
    if [
        "app_key",
        "app_secret",
        "access_token",
        "refresh_token",
        "account_id",
        "account_number",
        ".private",
        "private_contract",
    ]
    .iter()
    .any(|marker| normalized.contains(marker))
    {
        return Err(TossError::UnsafeFixture("secret_or_private_field"));
    }
    Ok(())
}

pub fn check_fixture_has_no_bearer_token(fixture: &str) -> Result<(), TossError> {
    if contains_bearer(fixture) {
        return Err(TossError::UnsafeFixture("bearer_token"));
    }
    Ok(())
}

pub fn check_fixture_has_no_private_account_id(fixture: &str) -> Result<(), TossError> {
    if let Ok(account_id) = env::var("TOSS_ACCOUNT_ID") {
        check_fixture_has_no_known_secrets(fixture, &[account_id.as_str()])?;
    }
    Ok(())
}

pub fn check_fixture_has_no_realistic_secret_pattern(fixture: &str) -> Result<(), TossError> {
    let has_secret_like_token = fixture
        .split(|character: char| !character.is_ascii_alphanumeric())
        .any(|token| {
            token.len() >= 32
                && token.bytes().any(|byte| byte.is_ascii_alphabetic())
                && token.bytes().any(|byte| byte.is_ascii_digit())
                && !token.to_ascii_lowercase().starts_with("fake")
                && !token.to_ascii_lowercase().starts_with("sanitized")
                && !token.to_ascii_lowercase().starts_with("replace")
        });
    if has_secret_like_token {
        return Err(TossError::UnsafeFixture("secret_like_pattern"));
    }
    Ok(())
}

pub fn check_fixture_safety(fixture: &str) -> Result<(), TossError> {
    let known_secrets = ["TOSS_APP_KEY", "TOSS_APP_SECRET"]
        .into_iter()
        .filter_map(|name| env::var(name).ok())
        .collect::<Vec<_>>();
    let known_secret_refs = known_secrets.iter().map(String::as_str).collect::<Vec<_>>();
    let private_account_id = env::var("TOSS_ACCOUNT_ID").ok();
    check_fixture_safety_with_known_values(
        fixture,
        &known_secret_refs,
        private_account_id.as_deref(),
    )
}

pub fn check_fixture_safety_with_known_values(
    fixture: &str,
    known_secrets: &[&str],
    private_account_id: Option<&str>,
) -> Result<(), TossError> {
    check_fixture_has_no_known_secrets(fixture, known_secrets)?;
    check_fixture_has_no_authorization_header(fixture)?;
    check_fixture_has_no_secret_field_names(fixture)?;
    check_fixture_has_no_bearer_token(fixture)?;
    if let Some(account_id) = private_account_id {
        check_fixture_has_no_known_secrets(fixture, &[account_id])?;
    }
    check_fixture_has_no_realistic_secret_pattern(fixture)?;
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(i32)]
pub enum TossReasonCode {
    ConfigLoaded = 100,
    MissingCredential = 101,
    PaperOnlyRequired = 102,
    ReadOnlyRequired = 103,
    ConfigInvalid = 104,
    RequestAttempted = 200,
    AuthFailed = 201,
    RateLimited = 202,
    Timeout = 203,
    MalformedResponse = 204,
    TransportFailed = 205,
    UnsupportedEndpoint = 206,
    EndpointDenied = 207,
    ContractValidated = 208,
    ContractInvalid = 209,
    FixtureSafe = 210,
    FixtureUnsafe = 211,
    MarketDataAccepted = 300,
    MarketDataRejected = 301,
    MissingPrice = 302,
    MissingSpread = 303,
    StaleData = 304,
    ConservativeSpread = 305,
    ApiUnavailable = 306,
    MissingTimestamp = 307,
    InvalidBidAsk = 308,
    BadSpread = 309,
    PrivateContractMappingRequired = 310,
    NonPositivePrice = 311,
    SensitiveFieldNameRejected = 312,
    MappingValidationFailed = 313,
    PaperOnlyDecision = 400,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TossAuditKind {
    ConfigLoaded,
    ReadOnlyRequestAttempted,
    ReadOnlyRequestFailed,
    MarketDataAccepted,
    MarketDataRejected,
    PaperOnlyDecisionCreated,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TossAuditEvent {
    pub kind: TossAuditKind,
    pub reason_code: TossReasonCode,
    pub numeric_status: i32,
    pub endpoint: String,
    pub read_only: bool,
    pub paper_only: bool,
}

impl TossAuditEvent {
    fn new(
        kind: TossAuditKind,
        reason_code: TossReasonCode,
        numeric_status: i32,
        endpoint: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            reason_code,
            numeric_status,
            endpoint: endpoint.into(),
            read_only: true,
            paper_only: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TossError {
    InvalidConfig(&'static str),
    MissingCredential { env_name: String },
    PaperOnlyRequired,
    ReadOnlyRequired,
    InvalidSymbol,
    InvalidContract(&'static str),
    MappingValidationFailed(&'static str),
    SensitiveFieldNameRejected,
    UnsafeFixture(&'static str),
    EndpointDenied,
    EndpointUnsupported,
    AuthFailure,
    RateLimited,
    Timeout,
    MalformedResponse,
    MissingPrice,
    MissingTimestamp,
    NonPositivePrice,
    InvalidBidAsk,
    InvalidMarketData,
    TransportFailure,
    HttpStatus(u16),
}

impl TossError {
    pub fn reason_code(&self) -> TossReasonCode {
        match self {
            Self::MissingCredential { .. } => TossReasonCode::MissingCredential,
            Self::PaperOnlyRequired => TossReasonCode::PaperOnlyRequired,
            Self::ReadOnlyRequired => TossReasonCode::ReadOnlyRequired,
            Self::EndpointDenied | Self::InvalidSymbol => TossReasonCode::EndpointDenied,
            Self::EndpointUnsupported => TossReasonCode::UnsupportedEndpoint,
            Self::InvalidContract(_) => TossReasonCode::ContractInvalid,
            Self::MappingValidationFailed(_) => TossReasonCode::MappingValidationFailed,
            Self::SensitiveFieldNameRejected => TossReasonCode::SensitiveFieldNameRejected,
            Self::UnsafeFixture(_) => TossReasonCode::FixtureUnsafe,
            Self::AuthFailure => TossReasonCode::AuthFailed,
            Self::RateLimited => TossReasonCode::RateLimited,
            Self::Timeout => TossReasonCode::Timeout,
            Self::MalformedResponse => TossReasonCode::MalformedResponse,
            Self::MissingPrice => TossReasonCode::MissingPrice,
            Self::MissingTimestamp => TossReasonCode::MissingTimestamp,
            Self::NonPositivePrice => TossReasonCode::NonPositivePrice,
            Self::InvalidBidAsk => TossReasonCode::InvalidBidAsk,
            Self::InvalidMarketData => TossReasonCode::MarketDataRejected,
            Self::InvalidConfig(_) => TossReasonCode::ConfigInvalid,
            Self::TransportFailure | Self::HttpStatus(_) => TossReasonCode::TransportFailed,
        }
    }
}

impl fmt::Display for TossError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidConfig(field) => write!(formatter, "invalid Toss config field: {field}"),
            Self::MissingCredential { env_name } => {
                write!(
                    formatter,
                    "missing Toss credential environment variable: {env_name}"
                )
            }
            Self::InvalidContract(field) => {
                write!(formatter, "invalid Toss endpoint contract field: {field}")
            }
            Self::MappingValidationFailed(field) => {
                write!(formatter, "invalid sanitized Toss field mapping: {field}")
            }
            Self::SensitiveFieldNameRejected => {
                write!(formatter, "sensitive Toss field mapping name rejected")
            }
            Self::UnsafeFixture(reason) => {
                write!(formatter, "unsafe sanitized Toss fixture: {reason}")
            }
            Self::HttpStatus(status) => write!(formatter, "Toss read-only HTTP status: {status}"),
            other => write!(formatter, "Toss read-only error: {:?}", other.reason_code()),
        }
    }
}

impl std::error::Error for TossError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TossMethod {
    Get,
    Head,
    Post,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TossEndpointKind {
    HealthRead,
    QuoteRead,
    CandleRead,
    AccountRead,
    TokenAuth,
    UnsupportedOrder,
    UnsupportedCancel,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TossHistoricalCapabilityV0 {
    KoreanEquityDailyOhlcv,
    UsEquityDailyOhlcv,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TossHistoricalContractStatusV0 {
    Qualified,
    ContractIncomplete,
    RequiresGuessedMapping,
    UnsupportedHistoricalDataset,
    ConfigurationMissing,
    CredentialUnavailable,
    NetworkConsentRequired,
    SmokeFailed,
    SnapshotAccepted,
    SnapshotRejected,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TossHistoricalContractQualificationV0 {
    pub capability: TossHistoricalCapabilityV0,
    pub status: TossHistoricalContractStatusV0,
    pub contract_version: Option<String>,
    pub operation_id: Option<String>,
    pub request_schema_known: bool,
    pub response_schema_known: bool,
    pub timestamp_semantics_known: bool,
    pub adjustment_semantics_known: bool,
    pub pagination_semantics_known: bool,
    pub rate_limit_semantics_known: bool,
    pub authentication_semantics_known: bool,
    pub read_only_verified: bool,
    pub reason_codes: Vec<String>,
}

pub fn qualify_toss_historical_capability_v0(
    capability: TossHistoricalCapabilityV0,
) -> TossHistoricalContractQualificationV0 {
    let contract = TossEndpointContract::candle_read();
    let incomplete = contract.response_schema_name == "deferred_private_contract"
        || !contract.allowed_in_manual_readonly_smoke;
    TossHistoricalContractQualificationV0 {
        capability,
        status: if incomplete {
            TossHistoricalContractStatusV0::ContractIncomplete
        } else {
            TossHistoricalContractStatusV0::RequiresGuessedMapping
        },
        contract_version: None,
        operation_id: None,
        request_schema_known: false,
        response_schema_known: false,
        timestamp_semantics_known: false,
        adjustment_semantics_known: false,
        pagination_semantics_known: false,
        rate_limit_semantics_known: false,
        authentication_semantics_known: false,
        read_only_verified: contract.read_only && contract.method == TossMethod::Get,
        reason_codes: vec!["exact_historical_contract_unavailable".to_string()],
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TossEndpointContract {
    pub kind: TossEndpointKind,
    pub method: TossMethod,
    pub path_template: String,
    pub required_headers: Vec<String>,
    pub required_query_fields: Vec<String>,
    pub required_body_fields: Vec<String>,
    pub response_schema_name: String,
    pub read_only: bool,
    pub requires_account: bool,
    pub allowed_in_tests: bool,
    pub allowed_in_manual_readonly_smoke: bool,
    pub reason_codes: Vec<TossReasonCode>,
}

impl TossEndpointContract {
    pub fn health_read() -> Self {
        Self::read_contract(TossEndpointKind::HealthRead, HEALTH_PATH, "health_v1", &[])
    }

    pub fn quote_read() -> Self {
        let mut contract = Self::read_contract(
            TossEndpointKind::QuoteRead,
            MARKET_SNAPSHOT_PATH,
            "toss_quote_sanitized_v1",
            &["symbol"],
        );
        contract
            .reason_codes
            .push(TossReasonCode::PrivateContractMappingRequired);
        contract
    }

    pub fn candle_read() -> Self {
        let mut contract = Self::read_contract(
            TossEndpointKind::CandleRead,
            "/soma/read-only/candles",
            "deferred_private_contract",
            &["symbol", "timeframe", "limit"],
        );
        contract.allowed_in_tests = false;
        contract
    }

    pub fn account_read() -> Self {
        let mut contract = Self::read_contract(
            TossEndpointKind::AccountRead,
            "/soma/read-only/account",
            "deferred_private_contract",
            &[],
        );
        contract.requires_account = true;
        contract.allowed_in_tests = false;
        contract
    }

    pub fn token_auth() -> Self {
        Self {
            kind: TossEndpointKind::TokenAuth,
            method: TossMethod::Post,
            path_template: "deferred-token-auth".to_string(),
            required_headers: Vec::new(),
            required_query_fields: Vec::new(),
            required_body_fields: Vec::new(),
            response_schema_name: "deferred_private_contract".to_string(),
            read_only: false,
            requires_account: false,
            allowed_in_tests: false,
            allowed_in_manual_readonly_smoke: false,
            reason_codes: vec![
                TossReasonCode::UnsupportedEndpoint,
                TossReasonCode::ContractValidated,
            ],
        }
    }

    pub fn unsupported_order() -> Self {
        Self::disabled(TossEndpointKind::UnsupportedOrder)
    }

    pub fn unsupported_cancel() -> Self {
        Self::disabled(TossEndpointKind::UnsupportedCancel)
    }

    pub fn unknown() -> Self {
        Self::disabled(TossEndpointKind::Unknown)
    }

    pub fn validate(&self) -> Result<(), TossError> {
        if self.path_template.trim().is_empty() {
            return Err(TossError::InvalidContract("path_template"));
        }
        if self.response_schema_name.trim().is_empty() {
            return Err(TossError::InvalidContract("response_schema_name"));
        }
        match self.kind {
            TossEndpointKind::HealthRead
            | TossEndpointKind::QuoteRead
            | TossEndpointKind::CandleRead => {
                if !self.read_only || self.method != TossMethod::Get {
                    return Err(TossError::ReadOnlyRequired);
                }
                if self.requires_account || !self.required_body_fields.is_empty() {
                    return Err(TossError::InvalidContract("market_read_shape"));
                }
            }
            TossEndpointKind::AccountRead => {
                if !self.read_only
                    || self.method != TossMethod::Get
                    || !self.requires_account
                    || !self.required_body_fields.is_empty()
                {
                    return Err(TossError::InvalidContract("account_read_shape"));
                }
            }
            TossEndpointKind::TokenAuth => {
                if self.read_only
                    || self.method != TossMethod::Post
                    || self.allowed_in_tests
                    || self.allowed_in_manual_readonly_smoke
                {
                    return Err(TossError::InvalidContract("token_auth_shape"));
                }
            }
            TossEndpointKind::UnsupportedOrder
            | TossEndpointKind::UnsupportedCancel
            | TossEndpointKind::Unknown => return Err(TossError::EndpointUnsupported),
        }
        Ok(())
    }

    pub fn is_callable_read_only(&self) -> bool {
        self.validate().is_ok()
            && self.read_only
            && self.method == TossMethod::Get
            && self.allowed_in_tests
            && matches!(
                self.kind,
                TossEndpointKind::HealthRead | TossEndpointKind::QuoteRead
            )
    }

    fn read_contract(
        kind: TossEndpointKind,
        path_template: &str,
        response_schema_name: &str,
        required_query_fields: &[&str],
    ) -> Self {
        Self {
            kind,
            method: TossMethod::Get,
            path_template: path_template.to_string(),
            required_headers: Vec::new(),
            required_query_fields: required_query_fields
                .iter()
                .map(|field| (*field).to_string())
                .collect(),
            required_body_fields: Vec::new(),
            response_schema_name: response_schema_name.to_string(),
            read_only: true,
            requires_account: false,
            allowed_in_tests: true,
            allowed_in_manual_readonly_smoke: false,
            reason_codes: vec![
                TossReasonCode::ContractValidated,
                TossReasonCode::ReadOnlyRequired,
            ],
        }
    }

    fn disabled(kind: TossEndpointKind) -> Self {
        Self {
            kind,
            method: TossMethod::Post,
            path_template: "disabled".to_string(),
            required_headers: Vec::new(),
            required_query_fields: Vec::new(),
            required_body_fields: Vec::new(),
            response_schema_name: "unsupported".to_string(),
            read_only: false,
            requires_account: false,
            allowed_in_tests: false,
            allowed_in_manual_readonly_smoke: false,
            reason_codes: vec![
                TossReasonCode::UnsupportedEndpoint,
                TossReasonCode::EndpointDenied,
            ],
        }
    }
}

fn default_endpoint_contracts() -> BTreeMap<TossEndpointKind, TossEndpointContract> {
    [
        TossEndpointContract::health_read(),
        TossEndpointContract::quote_read(),
        TossEndpointContract::candle_read(),
        TossEndpointContract::account_read(),
        TossEndpointContract::token_auth(),
        TossEndpointContract::unsupported_order(),
        TossEndpointContract::unsupported_cancel(),
    ]
    .into_iter()
    .map(|contract| (contract.kind, contract))
    .collect()
}

#[derive(Clone, PartialEq, Eq)]
pub struct TossRequest {
    method: TossMethod,
    path: String,
    headers: BTreeMap<String, String>,
    body: Option<String>,
}

impl TossRequest {
    pub fn method(&self) -> TossMethod {
        self.method
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn headers(&self) -> &BTreeMap<String, String> {
        &self.headers
    }

    pub fn body(&self) -> Option<&str> {
        self.body.as_deref()
    }
}

impl fmt::Debug for TossRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let headers = self
            .headers
            .iter()
            .map(|(name, value)| {
                (
                    name,
                    if sensitive_header_name(name) {
                        REDACTED
                    } else {
                        value.as_str()
                    },
                )
            })
            .collect::<BTreeMap<_, _>>();
        formatter
            .debug_struct("TossRequest")
            .field("method", &self.method)
            .field("path", &redact_url_query(&self.path))
            .field("headers", &headers)
            .field("body", &self.body.as_ref().map(|_| REDACTED))
            .finish()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct TossResponse {
    pub status: u16,
    pub headers: BTreeMap<String, String>,
    pub body: String,
}

impl TossResponse {
    pub fn json(status: u16, body: impl Into<String>) -> Self {
        Self {
            status,
            headers: BTreeMap::new(),
            body: body.into(),
        }
    }
}

impl fmt::Debug for TossResponse {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TossResponse")
            .field("status", &self.status)
            .field("header_count", &self.headers.len())
            .field("body", &REDACTED)
            .finish()
    }
}

pub trait TossTransport {
    fn request(&self, request: &TossRequest) -> Result<TossResponse, TossError>;
}

#[derive(Clone, Default)]
pub struct MockTossTransport {
    responses: Arc<Mutex<VecDeque<Result<TossResponse, TossError>>>>,
    requests: Arc<Mutex<Vec<TossRequest>>>,
}

impl fmt::Debug for MockTossTransport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("MockTossTransport")
            .field("response_count", &self.remaining_response_count())
            .field("request_count", &self.requests().len())
            .finish()
    }
}

impl MockTossTransport {
    pub fn new(responses: impl IntoIterator<Item = Result<TossResponse, TossError>>) -> Self {
        Self {
            responses: Arc::new(Mutex::new(responses.into_iter().collect())),
            requests: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn with_response(response: TossResponse) -> Self {
        Self::new([Ok(response)])
    }

    pub fn requests(&self) -> Vec<TossRequest> {
        self.requests
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    pub fn remaining_response_count(&self) -> usize {
        self.responses
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .len()
    }
}

impl TossTransport for MockTossTransport {
    fn request(&self, request: &TossRequest) -> Result<TossResponse, TossError> {
        self.requests
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push(request.clone());
        self.responses
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .pop_front()
            .unwrap_or(Err(TossError::TransportFailure))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TossCapabilities {
    pub market_data_read: bool,
    pub account_read: bool,
    pub order_execution: bool,
    pub live_execution: bool,
    pub paper_only: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TossQuoteFieldMapping {
    pub symbol_field: String,
    pub timestamp_field: String,
    pub price_field: String,
    pub bid_field: Option<String>,
    pub ask_field: Option<String>,
    pub volume_field: Option<String>,
    pub trade_value_field: Option<String>,
    pub status_field: Option<String>,
}

impl Default for TossQuoteFieldMapping {
    fn default() -> Self {
        Self {
            symbol_field: "symbol".to_string(),
            timestamp_field: "timestamp_ms".to_string(),
            price_field: "price".to_string(),
            bid_field: Some("bid".to_string()),
            ask_field: Some("ask".to_string()),
            volume_field: Some("volume".to_string()),
            trade_value_field: Some("trade_value".to_string()),
            status_field: Some("raw_status".to_string()),
        }
    }
}

impl TossQuoteFieldMapping {
    pub fn validate(&self) -> Result<(), TossError> {
        let required = [
            self.symbol_field.as_str(),
            self.timestamp_field.as_str(),
            self.price_field.as_str(),
        ];
        if required.iter().any(|field| field.trim().is_empty()) {
            return Err(TossError::MappingValidationFailed("required_quote_field"));
        }
        let mut fields = required.to_vec();
        for field in [
            self.bid_field.as_deref(),
            self.ask_field.as_deref(),
            self.volume_field.as_deref(),
            self.trade_value_field.as_deref(),
            self.status_field.as_deref(),
        ]
        .into_iter()
        .flatten()
        {
            if field.trim().is_empty() {
                return Err(TossError::MappingValidationFailed("optional_quote_field"));
            }
            fields.push(field);
        }
        fields.sort_unstable();
        if fields.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(TossError::MappingValidationFailed("duplicate_quote_field"));
        }
        if fields.iter().any(|field| {
            let normalized = field.to_ascii_lowercase();
            sensitive_key(field, true)
                || [
                    "private",
                    "credential",
                    "account",
                    "token",
                    "secret",
                    "authorization",
                ]
                .iter()
                .any(|marker| normalized.contains(marker))
        }) {
            return Err(TossError::SensitiveFieldNameRejected);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TossQuoteResponse {
    pub symbol: String,
    #[serde(default)]
    pub timestamp_ms: u64,
    #[serde(default)]
    pub price: Option<f64>,
    #[serde(default)]
    pub bid: Option<f64>,
    #[serde(default)]
    pub ask: Option<f64>,
    #[serde(default)]
    pub spread_bps: Option<f64>,
    #[serde(default)]
    pub volume: Option<f64>,
    #[serde(default)]
    pub trade_value: Option<f64>,
    #[serde(default)]
    pub volatility: Option<f64>,
    #[serde(default)]
    pub raw_status: Option<String>,
}

pub type TossQuote = TossQuoteResponse;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TossMappedQuote {
    pub snapshot: MarketSnapshot,
    pub reason_codes: Vec<TossReasonCode>,
}

pub fn parse_toss_quote_response(body: &str) -> Result<TossQuoteResponse, TossError> {
    let mapping = TossQuoteFieldMapping::default();
    parse_toss_quote_response_with_mapping(body, &mapping)
}

pub fn parse_toss_quote_response_with_mapping(
    body: &str,
    mapping: &TossQuoteFieldMapping,
) -> Result<TossQuoteResponse, TossError> {
    mapping.validate()?;
    let value = serde_json::from_str::<Value>(body).map_err(|_| TossError::MalformedResponse)?;
    reject_missing_required_fields(&value, mapping)?;
    let object = value.as_object().ok_or(TossError::MalformedResponse)?;
    let quote = TossQuoteResponse {
        symbol: object
            .get(&mapping.symbol_field)
            .and_then(Value::as_str)
            .ok_or(TossError::InvalidMarketData)?
            .to_string(),
        timestamp_ms: object
            .get(&mapping.timestamp_field)
            .and_then(Value::as_u64)
            .ok_or(TossError::MissingTimestamp)?,
        price: object.get(&mapping.price_field).and_then(Value::as_f64),
        bid: optional_f64(object, mapping.bid_field.as_deref())?,
        ask: optional_f64(object, mapping.ask_field.as_deref())?,
        spread_bps: None,
        volume: optional_f64(object, mapping.volume_field.as_deref())?,
        trade_value: optional_f64(object, mapping.trade_value_field.as_deref())?,
        volatility: None,
        raw_status: optional_string(object, mapping.status_field.as_deref())?,
    };
    validate_quote_fields(&quote)?;
    Ok(quote)
}

pub fn reject_missing_required_fields(
    value: &Value,
    mapping: &TossQuoteFieldMapping,
) -> Result<(), TossError> {
    let object = value.as_object().ok_or(TossError::MalformedResponse)?;
    if !object.contains_key(&mapping.timestamp_field) {
        return Err(TossError::MissingTimestamp);
    }
    if !object.contains_key(&mapping.price_field) {
        return Err(TossError::MissingPrice);
    }
    if !object.contains_key(&mapping.symbol_field) {
        return Err(TossError::InvalidMarketData);
    }
    Ok(())
}

pub fn reject_non_positive_price(price: Option<f64>) -> Result<(), TossError> {
    let Some(price) = price else {
        return Err(TossError::MissingPrice);
    };
    if !price.is_finite() || price <= 0.0 {
        return Err(TossError::NonPositivePrice);
    }
    Ok(())
}

pub fn reject_invalid_bid_ask(bid: Option<f64>, ask: Option<f64>) -> Result<(), TossError> {
    if bid.is_some_and(|bid| !bid.is_finite() || bid <= 0.0)
        || ask.is_some_and(|ask| !ask.is_finite() || ask <= 0.0)
        || bid.zip(ask).is_some_and(|(bid, ask)| ask < bid)
    {
        return Err(TossError::InvalidBidAsk);
    }
    Ok(())
}

pub fn validate_quote_fields(quote: &TossQuoteResponse) -> Result<(), TossError> {
    if !valid_symbol(&quote.symbol) {
        return Err(TossError::InvalidMarketData);
    }
    if quote.timestamp_ms == 0 {
        return Err(TossError::MissingTimestamp);
    }
    reject_non_positive_price(quote.price)?;
    reject_invalid_bid_ask(quote.bid, quote.ask)?;
    if quote
        .spread_bps
        .is_some_and(|spread| !spread.is_finite() || spread < 0.0)
    {
        return Err(TossError::InvalidMarketData);
    }
    Ok(())
}

pub fn compute_spread_bps(
    price: f64,
    bid: Option<f64>,
    ask: Option<f64>,
) -> Result<Option<f64>, TossError> {
    reject_invalid_bid_ask(bid, ask)?;
    Ok(bid.zip(ask).map(|(bid, ask)| {
        let midpoint = (bid + ask) / 2.0;
        (ask - bid) / midpoint.max(price.abs() * 1e-9).max(1e-9) * 10_000.0
    }))
}

pub fn compute_data_quality_score(
    quote: &TossQuoteResponse,
    evaluation_timestamp_ms: u64,
    stale_after_ms: u64,
    spread_bps: Option<f64>,
) -> (f64, Vec<TossReasonCode>) {
    let mut quality = 1.0_f64;
    let mut reasons = Vec::new();
    if quote.timestamp_ms > evaluation_timestamp_ms.saturating_add(stale_after_ms)
        || evaluation_timestamp_ms.saturating_sub(quote.timestamp_ms) > stale_after_ms
    {
        quality = quality.min(0.35);
        reasons.push(TossReasonCode::StaleData);
    }
    match spread_bps {
        Some(spread) if spread > 25.0 => {
            quality = quality.min(0.55);
            reasons.push(TossReasonCode::BadSpread);
        }
        Some(_) => {}
        None => {
            quality = quality.min(0.65);
            reasons.push(TossReasonCode::MissingSpread);
            reasons.push(TossReasonCode::ConservativeSpread);
        }
    }
    if quote.volume.is_none() {
        quality = (quality - 0.10).max(0.0);
    }
    (quality.clamp(0.0, 1.0), reasons)
}

pub fn map_quote_response_to_internal_snapshot(
    body: &str,
    evaluation_timestamp_ms: u64,
    stale_after_ms: u64,
) -> Result<TossMappedQuote, TossError> {
    let quote = parse_toss_quote_response(body)?;
    map_quote_to_internal_snapshot(&quote, evaluation_timestamp_ms, stale_after_ms)
}

pub fn map_quote_response_to_internal_snapshot_with_mapping(
    body: &str,
    mapping: &TossQuoteFieldMapping,
    evaluation_timestamp_ms: u64,
    stale_after_ms: u64,
) -> Result<TossMappedQuote, TossError> {
    let quote = parse_toss_quote_response_with_mapping(body, mapping)?;
    map_quote_to_internal_snapshot(&quote, evaluation_timestamp_ms, stale_after_ms)
}

fn map_quote_to_internal_snapshot(
    quote: &TossQuoteResponse,
    evaluation_timestamp_ms: u64,
    stale_after_ms: u64,
) -> Result<TossMappedQuote, TossError> {
    validate_quote_fields(quote)?;
    let price = quote.price.ok_or(TossError::MissingPrice)?;
    let computed_spread = compute_spread_bps(price, quote.bid, quote.ask)?;
    let effective_spread = computed_spread.or(quote.spread_bps);
    let (quality, reason_codes) = compute_data_quality_score(
        quote,
        evaluation_timestamp_ms,
        stale_after_ms,
        effective_spread,
    );
    let spread_bps = effective_spread.unwrap_or(50.0);
    let half_spread = price * spread_bps / 20_000.0;
    let bid = quote.bid.unwrap_or((price - half_spread).max(0.0));
    let ask = quote.ask.unwrap_or(price + half_spread);
    let volume = quote.volume.unwrap_or(0.0);
    let trade_value = quote.trade_value.unwrap_or(price * volume);
    let volatility = quote.volatility.unwrap_or(0.05);
    let snapshot = MarketSnapshot {
        symbol: quote.symbol.clone(),
        timestamp_ms: quote.timestamp_ms,
        price,
        bid,
        ask,
        spread_bps,
        volume,
        trade_value,
        volatility,
        regime: Regime::Unknown,
        data_quality_score: quality,
    };
    if [
        snapshot.price,
        snapshot.bid,
        snapshot.ask,
        snapshot.spread_bps,
        snapshot.volume,
        snapshot.trade_value,
        snapshot.volatility,
        snapshot.data_quality_score,
    ]
    .iter()
    .any(|value| !value.is_finite())
    {
        return Err(TossError::InvalidMarketData);
    }
    Ok(TossMappedQuote {
        snapshot,
        reason_codes,
    })
}

fn optional_f64(
    object: &serde_json::Map<String, Value>,
    field: Option<&str>,
) -> Result<Option<f64>, TossError> {
    let Some(field) = field else {
        return Ok(None);
    };
    match object.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(value) => value.as_f64().map(Some).ok_or(TossError::InvalidMarketData),
    }
}

fn optional_string(
    object: &serde_json::Map<String, Value>,
    field: Option<&str>,
) -> Result<Option<String>, TossError> {
    let Some(field) = field else {
        return Ok(None);
    };
    match object.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(value) => value
            .as_str()
            .map(|value| Some(value.to_string()))
            .ok_or(TossError::InvalidMarketData),
    }
}

#[derive(Deserialize)]
struct HealthResponse {
    ok: bool,
}

pub struct TossClient<T: TossTransport> {
    config: TossApiConfig,
    credentials: TossCredentials,
    transport: T,
    redactor: SecretRedactor,
    contracts: BTreeMap<TossEndpointKind, TossEndpointContract>,
    audit_events: Mutex<Vec<TossAuditEvent>>,
}

impl<T: TossTransport> fmt::Debug for TossClient<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TossClient")
            .field(
                "base_url",
                &redacted_base_url_preview(&self.config.base_url),
            )
            .field("credentials", &self.credentials)
            .field("capabilities", &self.capabilities())
            .finish_non_exhaustive()
    }
}

impl<T: TossTransport> TossClient<T> {
    pub fn new(
        config: TossApiConfig,
        credentials: TossCredentials,
        transport: T,
    ) -> Result<Self, TossError> {
        config.validate()?;
        let redactor = credentials.redactor(true);
        let config_endpoint = redacted_base_url_preview(&config.base_url);
        Ok(Self {
            config,
            credentials,
            transport,
            redactor,
            contracts: default_endpoint_contracts(),
            audit_events: Mutex::new(vec![TossAuditEvent::new(
                TossAuditKind::ConfigLoaded,
                TossReasonCode::ConfigLoaded,
                1,
                config_endpoint,
            )]),
        })
    }

    pub fn capabilities(&self) -> TossCapabilities {
        TossCapabilities {
            market_data_read: true,
            account_read: false,
            order_execution: false,
            live_execution: false,
            paper_only: true,
        }
    }

    pub fn audit_events(&self) -> Vec<TossAuditEvent> {
        self.audit_events
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    pub fn endpoint_contract(&self, kind: TossEndpointKind) -> Option<&TossEndpointContract> {
        self.contracts.get(&kind)
    }

    pub fn health_check(&self) -> Result<bool, TossError> {
        let response = self.perform_contract_get(TossEndpointKind::HealthRead, &BTreeMap::new())?;
        let health = serde_json::from_str::<HealthResponse>(&response.body)
            .map_err(|_| self.record_failure(HEALTH_PATH, TossError::MalformedResponse))?;
        Ok(health.ok)
    }

    pub fn get_market_snapshot(&self, symbol: &str) -> Result<TossQuote, TossError> {
        if !valid_symbol(symbol) {
            return Err(TossError::InvalidSymbol);
        }
        let query = BTreeMap::from([("symbol", symbol)]);
        let response = self.perform_contract_get(TossEndpointKind::QuoteRead, &query)?;
        let path = contract_path(
            self.contracts
                .get(&TossEndpointKind::QuoteRead)
                .ok_or(TossError::EndpointUnsupported)?,
            &query,
        )?;
        let quote = parse_toss_quote_response(&response.body)
            .map_err(|error| self.record_failure(&path, error))?;
        if quote.symbol != symbol {
            return Err(self.record_failure(&path, TossError::InvalidMarketData));
        }
        Ok(quote)
    }

    pub fn get_account_snapshot(&self) -> Result<TossAccountSnapshot, TossError> {
        self.perform_contract_get(TossEndpointKind::AccountRead, &BTreeMap::new())?;
        Err(TossError::EndpointUnsupported)
    }

    fn perform_contract_get(
        &self,
        kind: TossEndpointKind,
        query: &BTreeMap<&str, &str>,
    ) -> Result<TossResponse, TossError> {
        let contract = self
            .contracts
            .get(&kind)
            .ok_or(TossError::EndpointUnsupported)?;
        if !contract.is_callable_read_only() {
            return Err(
                self.record_failure(&contract.path_template, TossError::EndpointUnsupported)
            );
        }
        let path = contract_path(contract, query)?;
        let endpoint = self.redactor.redact_url_query(&path);
        self.push_audit(TossAuditEvent::new(
            TossAuditKind::ReadOnlyRequestAttempted,
            TossReasonCode::RequestAttempted,
            10,
            endpoint,
        ));
        let request = TossRequest {
            method: contract.method,
            path: path.clone(),
            headers: BTreeMap::new(),
            body: None,
        };
        let response = self
            .transport
            .request(&request)
            .map_err(|error| self.record_failure(&path, error))?;
        match response.status {
            200..=299 => Ok(response),
            401 | 403 => Err(self.record_failure(&path, TossError::AuthFailure)),
            408 => Err(self.record_failure(&path, TossError::Timeout)),
            429 => Err(self.record_failure(&path, TossError::RateLimited)),
            status => Err(self.record_failure(&path, TossError::HttpStatus(status))),
        }
    }

    fn record_failure(&self, path: &str, error: TossError) -> TossError {
        self.push_audit(TossAuditEvent::new(
            TossAuditKind::ReadOnlyRequestFailed,
            error.reason_code(),
            -1,
            self.redactor.redact_url_query(path),
        ));
        error
    }

    fn push_audit(&self, event: TossAuditEvent) {
        self.audit_events
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push(event);
    }
}

fn contract_path(
    contract: &TossEndpointContract,
    query: &BTreeMap<&str, &str>,
) -> Result<String, TossError> {
    if contract
        .required_query_fields
        .iter()
        .any(|field| !query.contains_key(field.as_str()))
    {
        return Err(TossError::InvalidContract("required_query_fields"));
    }
    if query.is_empty() {
        return Ok(contract.path_template.clone());
    }
    Ok(format!(
        "{}?{}",
        contract.path_template,
        query
            .iter()
            .map(|(key, value)| format!("{key}={value}"))
            .collect::<Vec<_>>()
            .join("&")
    ))
}

fn valid_symbol(symbol: &str) -> bool {
    !symbol.is_empty()
        && symbol.len() <= 32
        && symbol
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'))
}

#[derive(Clone, PartialEq)]
pub struct TossAccountSnapshot {
    pub account_configured: bool,
    pub timestamp_ms: u64,
}

impl fmt::Debug for TossAccountSnapshot {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TossAccountSnapshot")
            .field("account_configured", &self.account_configured)
            .field("timestamp_ms", &self.timestamp_ms)
            .finish()
    }
}

pub trait MarketDataProvider {
    fn latest_quote(&self, symbol: &str) -> Result<MarketSnapshot, TossError>;

    fn candles(
        &self,
        symbol: &str,
        timeframe: Timeframe,
        limit: usize,
    ) -> Result<CandleSeries, TossError>;
}

pub trait AccountReadProvider {
    fn account_snapshot(&self) -> Result<TossAccountSnapshot, TossError>;
}

pub struct TossReadOnlyAdapter<T: TossTransport> {
    client: TossClient<T>,
    evaluation_timestamp_ms: u64,
    stale_after_ms: u64,
}

impl<T: TossTransport> TossReadOnlyAdapter<T> {
    pub fn new(client: TossClient<T>, evaluation_timestamp_ms: u64) -> Self {
        Self {
            client,
            evaluation_timestamp_ms,
            stale_after_ms: 60_000,
        }
    }

    pub fn with_stale_after_ms(mut self, stale_after_ms: u64) -> Self {
        self.stale_after_ms = stale_after_ms;
        self
    }

    pub fn pipeline_input(&self, symbol: &str) -> TossPipelineInput {
        match self.client.get_market_snapshot(symbol) {
            Ok(quote) => match self.map_quote(&quote) {
                Ok((market, reason_codes)) => {
                    let data_accepted = market.price > 0.0
                        && market.data_quality_score >= 0.80
                        && !reason_codes.contains(&TossReasonCode::StaleData);
                    self.client.push_audit(TossAuditEvent::new(
                        if data_accepted {
                            TossAuditKind::MarketDataAccepted
                        } else {
                            TossAuditKind::MarketDataRejected
                        },
                        if data_accepted {
                            TossReasonCode::MarketDataAccepted
                        } else {
                            TossReasonCode::MarketDataRejected
                        },
                        if data_accepted { 21 } else { -21 },
                        MARKET_SNAPSHOT_PATH,
                    ));
                    let risk = RiskSnapshot {
                        daily_pnl_pct: 0.0,
                        consecutive_losses: 0,
                        current_positions_count: 0,
                        total_exposure_pct: 0.0,
                        symbol_exposure_pct: 0.0,
                        api_health_score: 1.0,
                        data_quality_score: market.data_quality_score,
                    };
                    let mut audit_events = self.client.audit_events();
                    audit_events.push(TossAuditEvent::new(
                        TossAuditKind::PaperOnlyDecisionCreated,
                        TossReasonCode::PaperOnlyDecision,
                        30,
                        MARKET_SNAPSHOT_PATH,
                    ));
                    TossPipelineInput {
                        market,
                        risk,
                        default_action: Stance::NoTrade,
                        read_only: true,
                        paper_only: true,
                        reason_codes,
                        audit_events,
                    }
                }
                Err(error) => self.unavailable_pipeline_input(symbol, error),
            },
            Err(error) => self.unavailable_pipeline_input(symbol, error),
        }
    }

    fn map_quote(
        &self,
        quote: &TossQuote,
    ) -> Result<(MarketSnapshot, Vec<TossReasonCode>), TossError> {
        let mapped = map_quote_to_internal_snapshot(
            quote,
            self.evaluation_timestamp_ms,
            self.stale_after_ms,
        )?;
        Ok((mapped.snapshot, mapped.reason_codes))
    }

    fn unavailable_pipeline_input(&self, symbol: &str, error: TossError) -> TossPipelineInput {
        let mut audit_events = self.client.audit_events();
        audit_events.push(TossAuditEvent::new(
            TossAuditKind::MarketDataRejected,
            error.reason_code(),
            -20,
            MARKET_SNAPSHOT_PATH,
        ));
        audit_events.push(TossAuditEvent::new(
            TossAuditKind::PaperOnlyDecisionCreated,
            TossReasonCode::PaperOnlyDecision,
            30,
            MARKET_SNAPSHOT_PATH,
        ));
        TossPipelineInput {
            market: MarketSnapshot {
                symbol: symbol.to_string(),
                timestamp_ms: self.evaluation_timestamp_ms,
                price: 0.0,
                bid: 0.0,
                ask: 0.0,
                spread_bps: 10_000.0,
                volume: 0.0,
                trade_value: 0.0,
                volatility: 1.0,
                regime: Regime::Unknown,
                data_quality_score: 0.0,
            },
            risk: RiskSnapshot {
                daily_pnl_pct: 0.0,
                consecutive_losses: 0,
                current_positions_count: 0,
                total_exposure_pct: 0.0,
                symbol_exposure_pct: 0.0,
                api_health_score: 0.0,
                data_quality_score: 0.0,
            },
            default_action: Stance::NoTrade,
            read_only: true,
            paper_only: true,
            reason_codes: vec![error.reason_code(), TossReasonCode::ApiUnavailable],
            audit_events,
        }
    }
}

impl<T: TossTransport> MarketDataProvider for TossReadOnlyAdapter<T> {
    fn latest_quote(&self, symbol: &str) -> Result<MarketSnapshot, TossError> {
        self.client
            .get_market_snapshot(symbol)
            .and_then(|quote| self.map_quote(&quote).map(|mapped| mapped.0))
    }

    fn candles(
        &self,
        _symbol: &str,
        _timeframe: Timeframe,
        _limit: usize,
    ) -> Result<CandleSeries, TossError> {
        Err(TossError::EndpointUnsupported)
    }
}

impl<T: TossTransport> AccountReadProvider for TossReadOnlyAdapter<T> {
    fn account_snapshot(&self) -> Result<TossAccountSnapshot, TossError> {
        Err(TossError::EndpointUnsupported)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TossPipelineInput {
    pub market: MarketSnapshot,
    pub risk: RiskSnapshot,
    pub default_action: Stance,
    pub read_only: bool,
    pub paper_only: bool,
    pub reason_codes: Vec<TossReasonCode>,
    pub audit_events: Vec<TossAuditEvent>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{
        ChairDecisionKind, ChairOutput, ReasonCode, RiskDecisionKind, Side, TradeProposal,
    };
    use crate::paper::{Broker, PaperBroker};
    use crate::risk::RiskGovernor;
    use crate::signal::derive_features;

    const NOW: u64 = 1_800_000_000_000;

    fn credentials() -> TossCredentials {
        TossCredentials {
            app_key: "test_app_key_value".to_string(),
            app_secret: "test_app_secret_value".to_string(),
            account_id: Some("test_account_id".to_string()),
        }
    }

    fn quote_body() -> String {
        format!(
            r#"{{
                "symbol":"005930",
                "timestamp_ms":{NOW},
                "price":70000.0,
                "bid":69990.0,
                "ask":70010.0,
                "volume":1000.0,
                "trade_value":70000000.0,
                "volatility":0.01
            }}"#
        )
    }

    fn adapter_with(
        response: Result<TossResponse, TossError>,
    ) -> TossReadOnlyAdapter<MockTossTransport> {
        let client = TossClient::new(
            TossApiConfig::default(),
            credentials(),
            MockTossTransport::new([response]),
        )
        .expect("safe client");
        TossReadOnlyAdapter::new(client, NOW)
    }

    #[test]
    fn missing_credentials_are_structured_errors() {
        let error = TossCredentials::load_with(&TossApiConfig::default(), |_| None)
            .expect_err("credentials must be missing");
        assert_eq!(
            error,
            TossError::MissingCredential {
                env_name: "TOSS_APP_KEY".to_string()
            }
        );
    }

    #[test]
    fn unsafe_config_modes_are_rejected() {
        let not_paper = TossApiConfig {
            paper_only: false,
            ..TossApiConfig::default()
        };
        assert_eq!(not_paper.validate(), Err(TossError::PaperOnlyRequired));

        let not_read_only = TossApiConfig {
            read_only: false,
            ..TossApiConfig::default()
        };
        assert_eq!(not_read_only.validate(), Err(TossError::ReadOnlyRequired));
    }

    #[test]
    fn credentials_load_from_configured_environment_names() {
        let loaded = TossCredentials::load_with(&TossApiConfig::default(), |name| {
            BTreeMap::from([
                ("TOSS_APP_KEY", "key"),
                ("TOSS_APP_SECRET", "secret"),
                ("TOSS_ACCOUNT_ID", "account"),
            ])
            .get(name)
            .map(ToString::to_string)
        })
        .expect("credentials");
        assert_eq!(loaded.app_key, "key");
        assert_eq!(loaded.app_secret, "secret");
        assert_eq!(loaded.account_id.as_deref(), Some("account"));
    }

    #[test]
    fn credential_debug_and_audit_are_redacted() {
        let value = credentials();
        let debug = format!("{value:?}");
        assert!(!debug.contains("test_app_key_value"));
        assert!(!debug.contains("test_app_secret_value"));
        let client = TossClient::new(
            TossApiConfig::default(),
            value,
            MockTossTransport::default(),
        )
        .expect("client");
        let audit = serde_json::to_string(&client.audit_events()).expect("audit json");
        assert!(!audit.contains("test_app_key_value"));
        assert!(!audit.contains("test_app_secret_value"));
        assert!(!audit.contains("test_account_id"));
    }

    #[test]
    fn environment_example_and_gitignore_are_safe() {
        let example = include_str!("../.env.example");
        assert!(example.contains("TOSS_APP_KEY=replace_me"));
        assert!(example.contains("TOSS_APP_SECRET=replace_me"));
        assert!(!example.contains("test_app_key_value"));
        assert!(
            include_str!("../.gitignore")
                .lines()
                .any(|line| line == ".env")
        );
    }

    #[test]
    fn authorization_bearer_known_values_and_account_are_redacted() {
        assert_eq!(
            redact_header_value("Authorization", "Bearer visible"),
            REDACTED
        );
        assert_eq!(redact_header_value("x-other", "Bearer visible"), REDACTED);
        let redactor = credentials().redactor(true);
        let text = redactor.redact_json_like_text(
            r#"{"app_key":"test_app_key_value","note":"test_app_secret_value","account_id":"test_account_id"}"#,
        );
        assert!(!text.contains("test_app_key_value"));
        assert!(!text.contains("test_app_secret_value"));
        assert!(!text.contains("test_account_id"));
    }

    #[test]
    fn url_query_secrets_are_redacted() {
        let redactor = credentials().redactor(true);
        let url = redactor.redact_url_query(
            "https://example.invalid/quote?token=abc&account_id=test_account_id&symbol=005930",
        );
        assert!(!url.contains("abc"));
        assert!(!url.contains("test_account_id"));
        assert!(url.contains("symbol=005930"));
    }

    #[test]
    fn request_and_response_debug_hide_sensitive_payloads() {
        let request = TossRequest {
            method: TossMethod::Get,
            path: "/quote?token=visible".to_string(),
            headers: BTreeMap::from([("Authorization".to_string(), "Bearer visible".to_string())]),
            body: Some("test_app_secret_value".to_string()),
        };
        let response = TossResponse::json(200, "test_app_secret_value");
        assert!(!format!("{request:?}").contains("visible"));
        assert!(!format!("{request:?}").contains("test_app_secret_value"));
        assert!(!format!("{response:?}").contains("test_app_secret_value"));
    }

    #[test]
    fn mock_transport_is_deterministic_and_records_read_only_request() {
        let transport = MockTossTransport::with_response(TossResponse::json(200, r#"{"ok":true}"#));
        let client = TossClient::new(TossApiConfig::default(), credentials(), transport.clone())
            .expect("client");
        assert!(client.health_check().expect("health"));
        let requests = transport.requests();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].method, TossMethod::Get);
        assert!(requests[0].headers.is_empty());
        assert!(requests[0].body.is_none());
    }

    #[test]
    fn mock_transport_simulates_timeout_rate_limit_and_malformed_body() {
        let timeout = adapter_with(Err(TossError::Timeout)).pipeline_input("005930");
        assert_eq!(timeout.default_action, Stance::NoTrade);
        assert_eq!(timeout.risk.api_health_score, 0.0);

        let limited = adapter_with(Ok(TossResponse::json(429, "{}"))).pipeline_input("005930");
        assert!(limited.reason_codes.contains(&TossReasonCode::RateLimited));

        let malformed = adapter_with(Ok(TossResponse::json(200, "{bad"))).pipeline_input("005930");
        assert!(
            malformed
                .reason_codes
                .contains(&TossReasonCode::MalformedResponse)
        );
    }

    #[test]
    fn authentication_failure_produces_no_trade_and_sanitized_failure_audit() {
        let input = adapter_with(Ok(TossResponse::json(401, "{}"))).pipeline_input("005930");
        assert_eq!(input.default_action, Stance::NoTrade);
        assert_eq!(input.risk.api_health_score, 0.0);
        assert!(input.reason_codes.contains(&TossReasonCode::AuthFailed));
        assert!(input.audit_events.iter().any(|event| {
            event.kind == TossAuditKind::ReadOnlyRequestFailed
                && event.reason_code == TossReasonCode::AuthFailed
                && event.numeric_status < 0
        }));
    }

    #[test]
    fn client_has_read_only_capabilities_and_no_account_or_order_capability() {
        let client = TossClient::new(
            TossApiConfig::default(),
            credentials(),
            MockTossTransport::default(),
        )
        .expect("client");
        assert_eq!(
            client.capabilities(),
            TossCapabilities {
                market_data_read: true,
                account_read: false,
                order_execution: false,
                live_execution: false,
                paper_only: true,
            }
        );
        assert_eq!(
            client.get_account_snapshot(),
            Err(TossError::EndpointUnsupported)
        );
    }

    #[test]
    fn malformed_response_is_rejected_with_sanitized_audit() {
        let client = TossClient::new(
            TossApiConfig::default(),
            credentials(),
            MockTossTransport::with_response(TossResponse::json(200, "{bad")),
        )
        .expect("client");
        assert_eq!(
            client.get_market_snapshot("005930"),
            Err(TossError::MalformedResponse)
        );
        let audit = serde_json::to_string(&client.audit_events()).expect("audit");
        assert!(!audit.contains("test_app_secret_value"));
        assert!(client.audit_events().iter().any(|event| {
            event.kind == TossAuditKind::ReadOnlyRequestFailed
                && event.reason_code == TossReasonCode::MalformedResponse
        }));
    }

    #[test]
    fn mock_quote_maps_to_existing_market_and_feature_types() {
        let input =
            adapter_with(Ok(TossResponse::json(200, quote_body()))).pipeline_input("005930");
        assert_eq!(input.market.symbol, "005930");
        assert_eq!(input.market.price, 70_000.0);
        assert!(input.read_only);
        assert!(input.paper_only);
        let features = derive_features(&input.market);
        assert_eq!(features.data_quality_score, input.market.data_quality_score);
        assert!(input.audit_events.iter().any(|event| {
            event.kind == TossAuditKind::MarketDataAccepted
                && event.reason_code == TossReasonCode::MarketDataAccepted
        }));
        assert!(input.audit_events.iter().any(|event| {
            event.kind == TossAuditKind::PaperOnlyDecisionCreated
                && event.reason_code == TossReasonCode::PaperOnlyDecision
        }));
    }

    #[test]
    fn missing_price_stale_data_and_missing_spread_reduce_quality() {
        let missing_price_body = quote_body().replace(r#""price":70000.0"#, r#""price":null"#);
        let missing_price =
            adapter_with(Ok(TossResponse::json(200, missing_price_body))).pipeline_input("005930");
        assert_eq!(missing_price.market.data_quality_score, 0.0);
        assert!(
            missing_price
                .reason_codes
                .contains(&TossReasonCode::MissingPrice)
        );

        let stale_body =
            quote_body().replace(&format!(r#""timestamp_ms":{NOW}"#), r#""timestamp_ms":1"#);
        let stale = adapter_with(Ok(TossResponse::json(200, stale_body))).pipeline_input("005930");
        assert!(stale.market.data_quality_score <= 0.35);
        assert!(stale.reason_codes.contains(&TossReasonCode::StaleData));

        let no_spread_body = quote_body()
            .replace(r#""bid":69990.0"#, r#""bid":null"#)
            .replace(r#""ask":70010.0"#, r#""ask":null"#);
        let no_spread =
            adapter_with(Ok(TossResponse::json(200, no_spread_body))).pipeline_input("005930");
        assert_eq!(no_spread.market.spread_bps, 50.0);
        assert!(no_spread.market.data_quality_score <= 0.65);
    }

    #[test]
    fn bad_spread_is_conservative_and_api_failure_is_risk_denied() {
        let bad_spread_body = quote_body()
            .replace(r#""bid":69990.0"#, r#""bid":69000.0"#)
            .replace(r#""ask":70010.0"#, r#""ask":71000.0"#);
        let bad_spread =
            adapter_with(Ok(TossResponse::json(200, bad_spread_body))).pipeline_input("005930");
        assert!(bad_spread.market.spread_bps > 25.0);
        assert!(bad_spread.market.data_quality_score <= 0.55);

        let unavailable = adapter_with(Err(TossError::Timeout)).pipeline_input("005930");
        let decision =
            RiskGovernor::default().evaluate(&unavailable.market, &unavailable.risk, None, NOW);
        assert!(matches!(
            decision.kind,
            RiskDecisionKind::Deny | RiskDecisionKind::EmergencyStop
        ));
        assert!(decision.reason_codes.iter().any(|reason| matches!(
            reason,
            ReasonCode::ApiHealthGateBreached | ReasonCode::DataQualityGateBreached
        )));
    }

    #[test]
    fn same_mock_input_produces_same_snapshot_and_default_no_trade() {
        let first =
            adapter_with(Ok(TossResponse::json(200, quote_body()))).pipeline_input("005930");
        let second =
            adapter_with(Ok(TossResponse::json(200, quote_body()))).pipeline_input("005930");
        assert_eq!(first, second);
        assert_eq!(first.default_action, Stance::NoTrade);
    }

    #[test]
    fn paper_broker_is_the_only_execution_boundary() {
        let input =
            adapter_with(Ok(TossResponse::json(200, quote_body()))).pipeline_input("005930");
        let mut broker = PaperBroker::default();
        let result = crate::simulate_paper_cycle(
            &input.market,
            &input.risk,
            &crate::MockSignalEngine::default(),
            &crate::ChairEngine::default(),
            &RiskGovernor::default(),
            &mut broker,
            false,
        );
        assert_eq!(result.votes.len(), 3);
        assert!(matches!(
            result.risk_decision.kind,
            RiskDecisionKind::Deny | RiskDecisionKind::EmergencyStop
        ));
        assert!(result.paper_order.is_none());
        assert!(!broker.supports_live_execution());
        assert_eq!(broker.live_call_count(), 0);
    }

    #[test]
    fn unsupported_candle_and_account_endpoints_do_not_call_transport() {
        let transport = MockTossTransport::default();
        let client = TossClient::new(TossApiConfig::default(), credentials(), transport.clone())
            .expect("client");
        let adapter = TossReadOnlyAdapter::new(client, NOW);
        assert_eq!(
            adapter.candles("005930", Timeframe::OneDay, 10),
            Err(TossError::EndpointUnsupported)
        );
        assert_eq!(
            adapter.account_snapshot(),
            Err(TossError::EndpointUnsupported)
        );
        assert!(transport.requests().is_empty());
    }

    #[test]
    fn private_paths_and_environment_files_are_ignored() {
        let ignore = include_str!("../.gitignore");
        for pattern in [
            ".env",
            ".env.*",
            "!.env.example",
            "local_private/",
            "*.private.*",
            "secrets/",
            "credentials/",
            "*.key",
            "*.pem",
        ] {
            assert!(ignore.lines().any(|line| line == pattern));
        }
    }

    #[test]
    fn read_only_contracts_validate_and_deferred_contracts_stay_disabled() {
        assert!(TossEndpointContract::quote_read().validate().is_ok());
        assert!(TossEndpointContract::candle_read().validate().is_ok());
        assert!(TossEndpointContract::account_read().validate().is_ok());
        assert!(TossEndpointContract::token_auth().validate().is_ok());
        assert!(!TossEndpointContract::candle_read().is_callable_read_only());
        assert!(!TossEndpointContract::account_read().is_callable_read_only());
        assert!(!TossEndpointContract::token_auth().is_callable_read_only());
    }

    #[test]
    fn historical_capabilities_fail_closed_without_exact_contract_material() {
        for capability in [
            TossHistoricalCapabilityV0::KoreanEquityDailyOhlcv,
            TossHistoricalCapabilityV0::UsEquityDailyOhlcv,
        ] {
            let qualification = qualify_toss_historical_capability_v0(capability);
            assert_eq!(
                qualification.status,
                TossHistoricalContractStatusV0::ContractIncomplete
            );
            assert!(qualification.read_only_verified);
            assert!(!qualification.request_schema_known);
            assert!(!qualification.response_schema_known);
        }
    }

    #[test]
    fn unsupported_and_unknown_contracts_never_validate_or_call_transport() {
        for contract in [
            TossEndpointContract::unsupported_order(),
            TossEndpointContract::unsupported_cancel(),
            TossEndpointContract::unknown(),
        ] {
            assert_eq!(contract.validate(), Err(TossError::EndpointUnsupported));
        }

        let transport = MockTossTransport::default();
        let client = TossClient::new(TossApiConfig::default(), credentials(), transport.clone())
            .expect("client");
        for kind in [
            TossEndpointKind::UnsupportedOrder,
            TossEndpointKind::UnsupportedCancel,
            TossEndpointKind::Unknown,
        ] {
            assert_eq!(
                client.perform_contract_get(kind, &BTreeMap::new()),
                Err(TossError::EndpointUnsupported)
            );
        }
        assert!(transport.requests().is_empty());
    }

    #[test]
    fn client_rejects_mutated_non_read_only_quote_contract() {
        let transport = MockTossTransport::default();
        let mut client =
            TossClient::new(TossApiConfig::default(), credentials(), transport.clone())
                .expect("client");
        let contract = client
            .contracts
            .get_mut(&TossEndpointKind::QuoteRead)
            .expect("quote contract");
        contract.read_only = false;
        assert_eq!(
            client.get_market_snapshot("FAKE123"),
            Err(TossError::EndpointUnsupported)
        );
        assert!(transport.requests().is_empty());
    }

    #[test]
    fn sanitized_quote_fixtures_pass_safety_scan() {
        for fixture in [
            include_str!("../fixtures/toss/quote_ok.json"),
            include_str!("../fixtures/toss/quote_missing_price.json"),
            include_str!("../fixtures/toss/quote_stale.json"),
            include_str!("../fixtures/toss/quote_bad_spread.json"),
            include_str!("../fixtures/toss/quote_invalid_bid_ask.json"),
            include_str!("../fixtures/toss/malformed_response.txt"),
        ] {
            assert_eq!(
                check_fixture_safety_with_known_values(fixture, &[], None),
                Ok(())
            );
        }
    }

    #[test]
    fn fixture_scanner_rejects_authorization_bearer_and_known_values() {
        assert_eq!(
            check_fixture_has_no_authorization_header(r#"{"Authorization":"fake"}"#),
            Err(TossError::UnsafeFixture("authorization_header"))
        );
        assert_eq!(
            check_fixture_has_no_bearer_token(r#"{"token":"Bearer fake-token"}"#),
            Err(TossError::UnsafeFixture("bearer_token"))
        );
        assert_eq!(
            check_fixture_has_no_secret_field_names(r#"{"access_token":"fake"}"#),
            Err(TossError::UnsafeFixture("secret_or_private_field"))
        );
        assert_eq!(
            check_fixture_has_no_secret_field_names(r#"{"refresh_token":"fake"}"#),
            Err(TossError::UnsafeFixture("secret_or_private_field"))
        );
        assert_eq!(
            check_fixture_has_no_secret_field_names(r#"{"private_contract":"fake"}"#),
            Err(TossError::UnsafeFixture("secret_or_private_field"))
        );
        assert_eq!(
            check_fixture_has_no_known_secrets("prefix fake-secret suffix", &["fake-secret"]),
            Err(TossError::UnsafeFixture("known_secret"))
        );
        assert_eq!(
            check_fixture_safety_with_known_values(
                "prefix injected-env-secret suffix",
                &["injected-env-secret"],
                None,
            ),
            Err(TossError::UnsafeFixture("known_secret"))
        );
        assert_eq!(
            check_fixture_has_no_realistic_secret_pattern("AbCdEfGhIjKlMnOpQrStUvWxYz0123456789"),
            Err(TossError::UnsafeFixture("secret_like_pattern"))
        );
    }

    #[test]
    fn quote_fixture_parses_through_client_and_adapter() {
        let body = include_str!("../fixtures/toss/quote_ok.json");
        let parsed = parse_toss_quote_response(body).expect("sanitized quote");
        assert_eq!(parsed.symbol, "FAKE123");
        assert_eq!(parsed.price, Some(100.0));

        let input = adapter_with(Ok(TossResponse::json(200, body))).pipeline_input("FAKE123");
        assert_eq!(input.market.price, 100.0);
        assert!(input.market.data_quality_score >= 0.80);
        assert_eq!(input.default_action, Stance::NoTrade);
    }

    #[test]
    fn sanitized_field_mapping_is_explicit_deterministic_and_finite() {
        let mapping = TossQuoteFieldMapping::default();
        assert!(mapping.validate().is_ok());
        assert_eq!(mapping.symbol_field, "symbol");
        assert_eq!(mapping.timestamp_field, "timestamp_ms");
        assert_eq!(mapping.price_field, "price");
        assert!(
            TossEndpointContract::quote_read()
                .reason_codes
                .contains(&TossReasonCode::PrivateContractMappingRequired)
        );

        let body = include_str!("../fixtures/toss/quote_ok.json");
        let first =
            map_quote_response_to_internal_snapshot(body, NOW, 60_000).expect("first mapped quote");
        let second = map_quote_response_to_internal_snapshot(body, NOW, 60_000)
            .expect("second mapped quote");
        assert_eq!(first, second);
        for value in [
            first.snapshot.price,
            first.snapshot.bid,
            first.snapshot.ask,
            first.snapshot.spread_bps,
            first.snapshot.volume,
            first.snapshot.trade_value,
            first.snapshot.volatility,
            first.snapshot.data_quality_score,
        ] {
            assert!(value.is_finite());
        }
        assert!(first.snapshot.data_quality_score >= 0.80);
    }

    #[test]
    fn sanitized_custom_field_mapping_maps_fake_keys_only() {
        let mapping = TossQuoteFieldMapping {
            symbol_field: "fake_symbol".to_string(),
            timestamp_field: "fake_timestamp".to_string(),
            price_field: "fake_price".to_string(),
            bid_field: Some("fake_bid".to_string()),
            ask_field: Some("fake_ask".to_string()),
            volume_field: None,
            trade_value_field: None,
            status_field: None,
        };
        let body = r#"{
            "fake_symbol":"FAKE123",
            "fake_timestamp":1800000000000,
            "fake_price":100.0,
            "fake_bid":99.98,
            "fake_ask":100.02
        }"#;
        let mapped =
            map_quote_response_to_internal_snapshot_with_mapping(body, &mapping, NOW, 60_000)
                .expect("custom sanitized mapping");
        assert_eq!(mapped.snapshot.symbol, "FAKE123");
        assert_eq!(mapped.snapshot.price, 100.0);
        assert!(mapped.snapshot.spread_bps > 0.0);
    }

    #[test]
    fn sanitized_mapping_rejects_sensitive_field_names() {
        let mapping = TossQuoteFieldMapping {
            price_field: "access_token".to_string(),
            ..TossQuoteFieldMapping::default()
        };
        assert_eq!(
            mapping.validate(),
            Err(TossError::SensitiveFieldNameRejected)
        );
        assert_eq!(
            mapping
                .validate()
                .expect_err("sensitive field")
                .reason_code(),
            TossReasonCode::SensitiveFieldNameRejected
        );

        let duplicate = TossQuoteFieldMapping {
            price_field: "symbol".to_string(),
            ..TossQuoteFieldMapping::default()
        };
        assert_eq!(
            duplicate
                .validate()
                .expect_err("duplicate field")
                .reason_code(),
            TossReasonCode::MappingValidationFailed
        );
    }

    #[test]
    fn invalid_quote_fixtures_are_rejected_with_structured_errors() {
        assert_eq!(
            parse_toss_quote_response(include_str!("../fixtures/toss/quote_missing_price.json")),
            Err(TossError::MissingPrice)
        );
        assert_eq!(
            parse_toss_quote_response(include_str!("../fixtures/toss/malformed_response.txt")),
            Err(TossError::MalformedResponse)
        );
        let missing_mapped_price =
            quote_body().replace(r#""price":70000.0,"#, r#""other_price":70000.0,"#);
        assert_eq!(
            map_quote_response_to_internal_snapshot(&missing_mapped_price, NOW, 60_000),
            Err(TossError::MissingPrice)
        );
        let missing_mapped_timestamp = quote_body().replace(r#""timestamp_ms":1800000000000,"#, "");
        assert_eq!(
            map_quote_response_to_internal_snapshot(&missing_mapped_timestamp, NOW, 60_000),
            Err(TossError::MissingTimestamp)
        );
        assert_eq!(
            parse_toss_quote_response(include_str!("../fixtures/toss/quote_invalid_bid_ask.json")),
            Err(TossError::InvalidBidAsk)
        );

        let ask_below_bid = quote_body().replace(r#""ask":70010.0"#, r#""ask":69900.0"#);
        assert_eq!(
            parse_toss_quote_response(&ask_below_bid),
            Err(TossError::InvalidBidAsk)
        );
        let non_positive = quote_body().replace(r#""price":70000.0"#, r#""price":0.0"#);
        assert_eq!(
            parse_toss_quote_response(&non_positive),
            Err(TossError::NonPositivePrice)
        );
    }

    #[test]
    fn stale_and_bad_spread_fixtures_remain_risk_conservative() {
        let stale = adapter_with(Ok(TossResponse::json(
            200,
            include_str!("../fixtures/toss/quote_stale.json"),
        )))
        .pipeline_input("FAKE123");
        assert!(stale.market.data_quality_score <= 0.35);
        assert!(stale.reason_codes.contains(&TossReasonCode::StaleData));

        let bad_spread = adapter_with(Ok(TossResponse::json(
            200,
            include_str!("../fixtures/toss/quote_bad_spread.json"),
        )))
        .pipeline_input("FAKE123");
        assert!(bad_spread.market.spread_bps >= 200.0);
        assert!(bad_spread.market.data_quality_score <= 0.55);
        assert!(bad_spread.reason_codes.contains(&TossReasonCode::BadSpread));
    }

    #[test]
    fn unsafe_fixture_inputs_cannot_escape_chair_risk_and_paper_pipeline() {
        for body in [
            include_str!("../fixtures/toss/quote_missing_price.json"),
            include_str!("../fixtures/toss/quote_stale.json"),
            include_str!("../fixtures/toss/quote_bad_spread.json"),
        ] {
            let input = adapter_with(Ok(TossResponse::json(200, body))).pipeline_input("FAKE123");
            let mut broker = PaperBroker::default();
            let result = crate::simulate_paper_cycle(
                &input.market,
                &input.risk,
                &crate::MockSignalEngine::default(),
                &crate::ChairEngine::default(),
                &RiskGovernor::default(),
                &mut broker,
                false,
            );
            assert_eq!(result.votes.len(), 3);
            assert_ne!(result.risk_decision.kind, RiskDecisionKind::ApprovePaper);
            assert!(result.paper_order.is_none());
            assert!(!broker.supports_live_execution());
            assert_eq!(broker.live_call_count(), 0);
        }
    }

    #[test]
    fn risk_governor_vetoes_unsafe_chair_approved_toss_candidate() {
        let input = adapter_with(Ok(TossResponse::json(
            200,
            include_str!("../fixtures/toss/quote_ok.json"),
        )))
        .pipeline_input("FAKE123");
        let proposal = TradeProposal {
            symbol: "FAKE123".to_string(),
            side: Side::Long,
            quantity_hint: 0.1,
            entry_price_hint: input.market.price,
            stop_loss: Some(99.0),
            take_profit: Some(103.0),
            max_slippage_bps: 5.0,
            expected_edge_after_cost: -0.01,
            confidence: 0.20,
            source_chair_output: ChairOutput {
                selected_speakers: vec!["fixture-chair".to_string()],
                lead_speaker: "fixture-chair".to_string(),
                forced_contrarian: false,
                council_score: 1.0,
                disagreement_score: 0.0,
                groupthink_risk: 0.0,
                size_multiplier: 0.1,
                decision: ChairDecisionKind::ApproveCandidate,
                reason_codes: vec![ReasonCode::CandidateApproved],
            },
        };
        let decision =
            RiskGovernor::default().evaluate(&input.market, &input.risk, Some(&proposal), NOW);
        assert_eq!(decision.kind, RiskDecisionKind::Deny);
        assert!(decision.approved_order_plan.is_none());
        assert!(
            decision
                .reason_codes
                .contains(&ReasonCode::ExpectedEdgeNonPositive)
        );
    }

    #[test]
    fn manual_readonly_smoke_remains_document_only_and_disabled() {
        let manifest = include_str!("../Cargo.toml");
        let smoke_doc = include_str!("../docs/TOSS_READONLY_SMOKE_TEST.md");
        assert!(!manifest.contains("toss_live_readonly_smoke"));
        assert!(smoke_doc.contains("No executable smoke binary is implemented"));
        assert!(smoke_doc.contains("Never executed by unit tests or CI"));
    }
}
