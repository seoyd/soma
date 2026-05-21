use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};
use crate::data::ProviderMarket;

use super::krx_symbol_whitelist::{
    KRXSymbolEntry, KRXSymbolWhitelist, KRXSymbolWhitelistConfig, normalize_symbol,
};

pub const DEFAULT_KRX_COLLECTION_OUTPUT_ROOT: &str = "target/soma_krx_collection_closure";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KRXBoundedCollectionSmokeConfig {
    pub smoke_id: String,
    #[serde(default)]
    pub activation_config_path: Option<String>,
    #[serde(default)]
    pub symbol_whitelist_path: Option<String>,
    #[serde(default)]
    pub barrier_profile_registry_path: Option<String>,
    #[serde(default)]
    pub local_fixture_response_paths: Vec<String>,
    #[serde(default)]
    pub local_canonical_csv_paths: Vec<String>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default = "default_max_symbols")]
    pub max_symbols: usize,
    #[serde(default = "default_max_rows_per_symbol")]
    pub max_rows_per_symbol: usize,
    #[serde(default = "default_max_requests")]
    pub max_requests: usize,
    #[serde(default = "default_max_days")]
    pub max_days: usize,
    #[serde(default = "default_max_raw_bytes")]
    pub max_raw_bytes: usize,
    #[serde(default = "default_max_canonical_bytes")]
    pub max_canonical_bytes: usize,
    #[serde(default = "default_max_total_bytes")]
    pub max_total_bytes: usize,
    #[serde(default = "default_true")]
    pub require_krx_api_key: bool,
    #[serde(default = "default_true")]
    pub require_krx_endpoint_template: bool,
    #[serde(default = "default_true")]
    pub run_dry_run: bool,
    #[serde(default)]
    pub run_live_collection: bool,
    #[serde(default = "default_true")]
    pub run_fixture_replay: bool,
    #[serde(default = "default_true")]
    pub run_local_import: bool,
    #[serde(default = "default_true")]
    pub run_preflight: bool,
    #[serde(default)]
    pub run_downstream_reruns: bool,
    #[serde(default = "default_true")]
    pub redact_endpoint_preview: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum KRXCollectionDryRunStatus {
    ReadyToCollect,
    MissingApiKey,
    MissingEndpointTemplate,
    MissingApiKeyAndEndpointTemplate,
    ScopeTooBroad,
    BudgetExceeded,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KRXCollectionDryRunReport {
    pub api_key_env_var_name: String,
    pub api_key_present: bool,
    pub endpoint_template_env_var_name: String,
    pub endpoint_template_present: bool,
    #[serde(default)]
    pub endpoint_preview_redacted: Option<String>,
    pub planned_symbols: usize,
    pub planned_requests: usize,
    pub planned_rows: usize,
    pub planned_days: usize,
    pub planned_bytes: usize,
    pub dry_run_status: KRXCollectionDryRunStatus,
    pub safe_to_run_live_collection: bool,
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for KRXBoundedCollectionSmokeConfig {
    fn default() -> Self {
        Self {
            smoke_id: "krx_bounded_collection_smoke".to_string(),
            activation_config_path: None,
            symbol_whitelist_path: None,
            barrier_profile_registry_path: None,
            local_fixture_response_paths: Vec::new(),
            local_canonical_csv_paths: Vec::new(),
            output_root: default_output_root(),
            max_symbols: default_max_symbols(),
            max_rows_per_symbol: default_max_rows_per_symbol(),
            max_requests: default_max_requests(),
            max_days: default_max_days(),
            max_raw_bytes: default_max_raw_bytes(),
            max_canonical_bytes: default_max_canonical_bytes(),
            max_total_bytes: default_max_total_bytes(),
            require_krx_api_key: true,
            require_krx_endpoint_template: true,
            run_dry_run: true,
            run_live_collection: false,
            run_fixture_replay: true,
            run_local_import: true,
            run_preflight: true,
            run_downstream_reruns: false,
            redact_endpoint_preview: true,
            reason_codes: vec![
                ReasonCode::DeterministicPath,
                ReasonCode::KRXCollectionDisabledByDefault,
                ReasonCode::KRXLocalImportPreferred,
            ],
        }
    }
}

impl KRXBoundedCollectionSmokeConfig {
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

    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.smoke_id)
    }

    pub fn validate_local_paths(&self) -> Vec<ReasonCode> {
        let mut reasons = Vec::new();
        for path in [
            Some(self.output_root.as_str()),
            self.activation_config_path.as_deref(),
            self.symbol_whitelist_path.as_deref(),
            self.barrier_profile_registry_path.as_deref(),
        ]
        .into_iter()
        .flatten()
        .chain(self.local_fixture_response_paths.iter().map(String::as_str))
        .chain(self.local_canonical_csv_paths.iter().map(String::as_str))
        {
            if is_remote_path(path) {
                reasons.push(ReasonCode::LocalPathRejected);
            }
        }
        stable_reason_codes(&reasons)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.smoke_id.trim().is_empty() {
            return Err("krx smoke_id must not be empty".to_string());
        }
        if !self.validate_local_paths().is_empty() {
            return Err("krx bounded collection smoke paths must be local".to_string());
        }
        if self.max_symbols == 0 || self.max_symbols > default_max_symbols() {
            return Err("krx max_symbols must be between 1 and 5".to_string());
        }
        if self.max_rows_per_symbol == 0 || self.max_rows_per_symbol > default_max_rows_per_symbol()
        {
            return Err("krx max_rows_per_symbol must be between 1 and 300".to_string());
        }
        if self.max_requests == 0 || self.max_requests > default_max_requests() {
            return Err("krx max_requests must be between 1 and 10".to_string());
        }
        if self.max_days == 0 || self.max_days > default_max_days() {
            return Err("krx max_days must be between 1 and 365".to_string());
        }
        if self.max_raw_bytes == 0 || self.max_canonical_bytes == 0 || self.max_total_bytes == 0 {
            return Err("krx byte budgets must be positive".to_string());
        }
        if self.max_total_bytes < self.max_raw_bytes
            || self.max_total_bytes < self.max_canonical_bytes
        {
            return Err("krx max_total_bytes must cover raw and canonical budgets".to_string());
        }
        Ok(())
    }

    pub fn load_whitelist(&self) -> Result<KRXSymbolWhitelist, String> {
        if let Some(path) = self.symbol_whitelist_path.as_deref() {
            let config = KRXSymbolWhitelistConfig::from_toml_path(Path::new(path))?;
            config.validate()?;
            return Ok(config.build());
        }
        let mut symbols = self
            .local_fixture_response_paths
            .iter()
            .chain(self.local_canonical_csv_paths.iter())
            .filter_map(|path| infer_symbol_from_path(path))
            .collect::<Vec<_>>();
        symbols.sort_by(|left, right| left.normalized_symbol.cmp(&right.normalized_symbol));
        symbols.dedup_by(|left, right| left.normalized_symbol == right.normalized_symbol);
        let config = KRXSymbolWhitelistConfig {
            whitelist_id: format!("{}-derived", self.smoke_id),
            symbols,
            output_root: self.output_root.clone(),
            max_symbols: self.max_symbols,
            require_market: true,
            require_provider_symbol: true,
            reason_codes: vec![ReasonCode::DeterministicPath],
        };
        config.validate()?;
        Ok(config.build())
    }

    pub fn build_dry_run_report(
        &self,
        whitelist: &KRXSymbolWhitelist,
    ) -> KRXCollectionDryRunReport {
        KRXCollectionDryRunReport::from_config(self, whitelist)
    }
}

impl KRXCollectionDryRunReport {
    pub fn from_config(
        config: &KRXBoundedCollectionSmokeConfig,
        whitelist: &KRXSymbolWhitelist,
    ) -> Self {
        let api_key_present = env_var_present(super::krx_auth_readiness::KRX_API_KEY_ENV_VAR);
        let endpoint_value =
            env::var(super::krx_auth_readiness::KRX_ENDPOINT_TEMPLATE_ENV_VAR).ok();
        let endpoint_template_present = endpoint_value
            .as_deref()
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false);
        let planned_symbols = whitelist.enabled_entries.len();
        let planned_rows = whitelist
            .entries
            .iter()
            .filter(|entry| entry.enabled)
            .take(config.max_symbols)
            .map(|entry| entry.max_rows.unwrap_or(config.max_rows_per_symbol))
            .sum::<usize>();
        let planned_requests = planned_symbols.min(config.max_requests);
        let planned_days = config.max_days;
        let estimated_raw_bytes = planned_rows.saturating_mul(96);
        let estimated_canonical_bytes = planned_rows.saturating_mul(72);
        let planned_bytes = estimated_raw_bytes + estimated_canonical_bytes;
        let scope_too_broad = planned_symbols > config.max_symbols
            || whitelist
                .entries
                .iter()
                .any(|entry| entry.provider_symbol.eq_ignore_ascii_case("ALL"));
        let budget_exceeded = planned_bytes > config.max_total_bytes
            || estimated_raw_bytes > config.max_raw_bytes
            || estimated_canonical_bytes > config.max_canonical_bytes;
        let dry_run_status = if !config.require_krx_api_key && !config.require_krx_endpoint_template
        {
            KRXCollectionDryRunStatus::DiagnosticOnly
        } else if scope_too_broad {
            KRXCollectionDryRunStatus::ScopeTooBroad
        } else if budget_exceeded {
            KRXCollectionDryRunStatus::BudgetExceeded
        } else if config.require_krx_api_key
            && !api_key_present
            && config.require_krx_endpoint_template
            && !endpoint_template_present
        {
            KRXCollectionDryRunStatus::MissingApiKeyAndEndpointTemplate
        } else if config.require_krx_api_key && !api_key_present {
            KRXCollectionDryRunStatus::MissingApiKey
        } else if config.require_krx_endpoint_template && !endpoint_template_present {
            KRXCollectionDryRunStatus::MissingEndpointTemplate
        } else {
            KRXCollectionDryRunStatus::ReadyToCollect
        };
        let mut reason_codes = vec![
            ReasonCode::KRXAuthReadinessBuilt,
            ReasonCode::CollectionBudgetReportBuilt,
        ];
        if api_key_present {
            reason_codes.push(ReasonCode::AuthConfigValidated);
        } else {
            reason_codes.push(ReasonCode::MissingApiKey);
        }
        if endpoint_template_present {
            reason_codes.push(ReasonCode::KRXEndpointPreviewRedacted);
        } else {
            reason_codes.push(ReasonCode::MissingEndpointTemplate);
        }
        if scope_too_broad {
            reason_codes.push(ReasonCode::DeniedByDefault);
        }
        if budget_exceeded {
            reason_codes.push(ReasonCode::BudgetExceeded);
        }
        Self {
            api_key_env_var_name: super::krx_auth_readiness::KRX_API_KEY_ENV_VAR.to_string(),
            api_key_present,
            endpoint_template_env_var_name:
                super::krx_auth_readiness::KRX_ENDPOINT_TEMPLATE_ENV_VAR.to_string(),
            endpoint_template_present,
            endpoint_preview_redacted: endpoint_value
                .as_deref()
                .filter(|value| !value.trim().is_empty() && config.redact_endpoint_preview)
                .map(redacted_endpoint_preview),
            planned_symbols,
            planned_requests,
            planned_rows,
            planned_days,
            planned_bytes,
            dry_run_status,
            safe_to_run_live_collection: config.run_live_collection
                && matches!(dry_run_status, KRXCollectionDryRunStatus::ReadyToCollect),
            reason_codes: stable_reason_codes(&reason_codes),
        }
    }

    pub fn to_text(&self) -> String {
        [
            "research_only_warning=krx bounded collection dry run is market-data-only and never implies live trading".to_string(),
            "market_data_only_warning=bounded KRX planning excludes broker, order, and account APIs".to_string(),
            "secret_safety_warning=env-var names and redacted endpoint previews only; no secret values are rendered".to_string(),
            format!("api_key_env_var_name={}", self.api_key_env_var_name),
            format!("api_key_present={}", self.api_key_present),
            format!(
                "endpoint_template_env_var_name={}",
                self.endpoint_template_env_var_name
            ),
            format!("endpoint_template_present={}", self.endpoint_template_present),
            format!(
                "endpoint_preview_redacted={}",
                self.endpoint_preview_redacted.clone().unwrap_or_default()
            ),
            format!("planned_symbols={}", self.planned_symbols),
            format!("planned_requests={}", self.planned_requests),
            format!("planned_rows={}", self.planned_rows),
            format!("planned_days={}", self.planned_days),
            format!("planned_bytes={}", self.planned_bytes),
            format!("dry_run_status={:?}", self.dry_run_status),
            format!(
                "safe_to_run_live_collection={}",
                self.safe_to_run_live_collection
            ),
            format!(
                "reason_codes={}",
                self.reason_codes
                    .iter()
                    .map(|reason| format!("{reason:?}"))
                    .collect::<Vec<_>>()
                    .join("|")
            ),
        ]
        .join("\n")
    }

    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<(), String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        fs::write(output_dir.join("krx_auth_dry_run.txt"), self.to_text())
            .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("krx_auth_dry_run.json"),
            self.to_json_string()?,
        )
        .map_err(|err| err.to_string())?;
        Ok(())
    }
}

pub(crate) fn infer_symbol_from_path(path: &str) -> Option<KRXSymbolEntry> {
    let normalized = path
        .split(|character: char| !character.is_ascii_alphanumeric())
        .find(|part| part.len() == 6 && part.chars().all(|character| character.is_ascii_digit()))?
        .to_string();
    Some(KRXSymbolEntry {
        provider_symbol: normalized.clone(),
        normalized_symbol: normalize_symbol(&normalized),
        market: ProviderMarket::KoreanEquity,
        venue: Some("KRX".to_string()),
        display_name: None,
        enabled: true,
        max_rows: None,
        timeframe: "1d".to_string(),
        reason_codes: Vec::new(),
    })
}

pub(crate) fn is_remote_path(path: &str) -> bool {
    path.contains("://")
}

fn env_var_present(name: &str) -> bool {
    env::var_os(name)
        .map(|value| !value.is_empty())
        .unwrap_or(false)
}

fn redacted_endpoint_preview(value: &str) -> String {
    let normalized = value.trim();
    let has_symbol_placeholder =
        normalized.contains("{symbol}") || normalized.contains("${symbol}");
    let has_date_placeholder = normalized.contains("{date}") || normalized.contains("${date}");
    let has_query = normalized.contains('?');
    format!(
        "configured(redacted;symbol_placeholder={has_symbol_placeholder};date_placeholder={has_date_placeholder};query={has_query})"
    )
}

fn default_output_root() -> String {
    DEFAULT_KRX_COLLECTION_OUTPUT_ROOT.to_string()
}

fn default_max_symbols() -> usize {
    5
}

fn default_max_rows_per_symbol() -> usize {
    300
}

fn default_max_requests() -> usize {
    10
}

fn default_max_days() -> usize {
    365
}

fn default_max_raw_bytes() -> usize {
    256_000
}

fn default_max_canonical_bytes() -> usize {
    256_000
}

fn default_max_total_bytes() -> usize {
    512_000
}

fn default_true() -> bool {
    true
}
