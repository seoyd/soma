use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};

use super::kis_auth_closure::{
    KISAuthClosureConfig, KISAuthClosureReport, KISAuthClosureRunner, KISAuthClosureStatus,
};
use super::kis_endpoint_policy::{KISEndpointCategory, KISEndpointPolicy, KISEndpointPolicyStatus};
use super::kis_symbol_whitelist::{KISSymbolWhitelist, KISSymbolWhitelistConfig};

fn default_output_root() -> String {
    "target/sprint58/kis_market_data_dry_run".to_string()
}

fn default_max_symbols() -> usize {
    5
}

fn default_max_requests() -> usize {
    10
}

fn default_max_rows_per_symbol() -> usize {
    300
}

fn default_max_days() -> usize {
    365
}

fn default_max_bytes() -> usize {
    5_000_000
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KISMarketDataDryRunConfig {
    pub dry_run_id: String,
    #[serde(default)]
    pub kis_auth_closure_config_path: Option<String>,
    #[serde(default)]
    pub endpoint_policy_path: Option<String>,
    #[serde(default)]
    pub domestic_symbol_whitelist_path: Option<String>,
    #[serde(default)]
    pub overseas_symbol_whitelist_path: Option<String>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default = "default_max_symbols")]
    pub max_symbols: usize,
    #[serde(default = "default_max_requests")]
    pub max_requests: usize,
    #[serde(default = "default_max_rows_per_symbol")]
    pub max_rows_per_symbol: usize,
    #[serde(default = "default_max_days")]
    pub max_days: usize,
    #[serde(default = "default_max_bytes")]
    pub max_bytes: usize,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum KISMarketDataDryRunStatus {
    Ready,
    MissingAuth,
    MissingBaseUrl,
    EndpointPolicyBlocked,
    ScopeTooBroad,
    BudgetExceeded,
    NoSymbols,
    #[default]
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KISMarketDataDryRunReport {
    pub dry_run_id: String,
    pub auth_status: KISAuthClosureStatus,
    pub endpoint_policy_status: KISEndpointPolicyStatus,
    pub planned_domestic_symbols: usize,
    pub planned_overseas_symbols: usize,
    pub planned_requests: usize,
    pub planned_rows: usize,
    pub planned_days: usize,
    pub planned_bytes: usize,
    pub dry_run_status: KISMarketDataDryRunStatus,
    pub safe_to_run_operator_live_collection: bool,
    #[serde(default)]
    pub blocked_reasons: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct KISMarketDataDryRunRunner;

impl Default for KISMarketDataDryRunConfig {
    fn default() -> Self {
        Self {
            dry_run_id: "sprint58-kis-market-data-dry-run".to_string(),
            kis_auth_closure_config_path: None,
            endpoint_policy_path: None,
            domestic_symbol_whitelist_path: None,
            overseas_symbol_whitelist_path: None,
            output_root: default_output_root(),
            max_symbols: default_max_symbols(),
            max_requests: default_max_requests(),
            max_rows_per_symbol: default_max_rows_per_symbol(),
            max_days: default_max_days(),
            max_bytes: default_max_bytes(),
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

impl KISMarketDataDryRunConfig {
    pub fn from_toml_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        toml::from_str(&text).map_err(|err| err.to_string())
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.dry_run_id.trim().is_empty() {
            return Err("kis market-data dry-run id must not be empty".to_string());
        }
        if [
            self.kis_auth_closure_config_path.as_deref(),
            self.endpoint_policy_path.as_deref(),
            self.domestic_symbol_whitelist_path.as_deref(),
            self.overseas_symbol_whitelist_path.as_deref(),
            Some(self.output_root.as_str()),
        ]
        .into_iter()
        .flatten()
        .any(|path| path.contains("://"))
        {
            return Err("kis market-data dry-run paths must be local".to_string());
        }
        if self.max_symbols == 0 || self.max_symbols > default_max_symbols() {
            return Err("kis market-data dry-run max_symbols must be between 1 and 5".to_string());
        }
        if self.max_requests == 0 || self.max_requests > default_max_requests() {
            return Err(
                "kis market-data dry-run max_requests must be between 1 and 10".to_string(),
            );
        }
        if self.max_rows_per_symbol == 0 || self.max_rows_per_symbol > default_max_rows_per_symbol()
        {
            return Err(
                "kis market-data dry-run max_rows_per_symbol must be between 1 and 300".to_string(),
            );
        }
        if self.max_days == 0 || self.max_days > default_max_days() {
            return Err("kis market-data dry-run max_days must be between 1 and 365".to_string());
        }
        if self.max_bytes == 0 {
            return Err("kis market-data dry-run max_bytes must be positive".to_string());
        }
        Ok(())
    }

    pub fn artifact_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.dry_run_id)
    }
}

impl KISMarketDataDryRunReport {
    pub fn to_text(&self) -> String {
        [
            "market_data_only_warning=kis market-data dry-run never calls network".to_string(),
            format!("dry_run_id={}", self.dry_run_id),
            format!("auth_status={:?}", self.auth_status),
            format!("endpoint_policy_status={:?}", self.endpoint_policy_status),
            format!("planned_domestic_symbols={}", self.planned_domestic_symbols),
            format!("planned_overseas_symbols={}", self.planned_overseas_symbols),
            format!("planned_requests={}", self.planned_requests),
            format!("planned_rows={}", self.planned_rows),
            format!("planned_days={}", self.planned_days),
            format!("planned_bytes={}", self.planned_bytes),
            format!("dry_run_status={:?}", self.dry_run_status),
            format!(
                "safe_to_run_operator_live_collection={}",
                self.safe_to_run_operator_live_collection
            ),
            format!("blocked_reasons={}", self.blocked_reasons.join("|")),
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

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        let text_path = output_dir.join("kis_market_data_dry_run.txt");
        fs::write(&text_path, self.to_text()).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("kis_market_data_dry_run.json"),
            serde_json::to_string_pretty(self).map_err(|err| err.to_string())?,
        )
        .map_err(|err| err.to_string())?;
        Ok(text_path)
    }
}

impl KISMarketDataDryRunRunner {
    pub fn run(
        &self,
        config: &KISMarketDataDryRunConfig,
    ) -> Result<KISMarketDataDryRunReport, String> {
        config.validate()?;
        let auth_report = if let Some(path) = &config.kis_auth_closure_config_path {
            let auth_config = KISAuthClosureConfig::from_toml_path(Path::new(path))?;
            KISAuthClosureRunner::default().run(&auth_config)?
        } else {
            KISAuthClosureRunner::default().run(&KISAuthClosureConfig::default())?
        };
        self.run_with_auth(config, &auth_report)
    }

    pub fn run_with_auth(
        &self,
        config: &KISMarketDataDryRunConfig,
        auth_report: &KISAuthClosureReport,
    ) -> Result<KISMarketDataDryRunReport, String> {
        config.validate()?;
        let endpoint_policy = if let Some(path) = &config.endpoint_policy_path {
            KISEndpointPolicy::from_toml_path(Path::new(path))?
        } else {
            KISEndpointPolicy::default()
        };
        let endpoint_policy_status = endpoint_policy
            .report_for_categories(&[
                KISEndpointCategory::DomesticStockPeriodPrice,
                KISEndpointCategory::OverseasStockPeriodPrice,
                KISEndpointCategory::DomesticStockRealtimeQuote,
                KISEndpointCategory::OverseasStockRealtimeQuote,
            ])
            .policy_status;
        let domestic = load_whitelist(config.domestic_symbol_whitelist_path.as_deref())?;
        let overseas = load_whitelist(config.overseas_symbol_whitelist_path.as_deref())?;
        let planned_domestic_symbols = domestic
            .as_ref()
            .map(|whitelist| whitelist.enabled_entries.len())
            .unwrap_or_default()
            .min(config.max_symbols);
        let planned_overseas_symbols = overseas
            .as_ref()
            .map(|whitelist| whitelist.enabled_entries.len())
            .unwrap_or_default()
            .min(config.max_symbols.saturating_sub(planned_domestic_symbols));
        let total_symbols = planned_domestic_symbols + planned_overseas_symbols;
        let planned_rows = total_symbols.saturating_mul(config.max_rows_per_symbol);
        let planned_requests = total_symbols.min(config.max_requests);
        let planned_days = if total_symbols == 0 {
            0
        } else {
            config.max_days
        };
        let planned_bytes = planned_rows.saturating_mul(96);
        let scope_too_broad = domestic
            .as_ref()
            .map(|whitelist| whitelist.enabled_entries.len())
            .unwrap_or_default()
            + overseas
                .as_ref()
                .map(|whitelist| whitelist.enabled_entries.len())
                .unwrap_or_default()
            > config.max_symbols;
        let no_symbols = total_symbols == 0;
        let budget_exceeded = planned_bytes > config.max_bytes;
        let auth_missing = matches!(
            auth_report.closure_status,
            KISAuthClosureStatus::MissingAppKey
                | KISAuthClosureStatus::MissingAppSecret
                | KISAuthClosureStatus::MissingAppKeyAndSecret
        );
        let base_url_missing = matches!(
            auth_report.closure_status,
            KISAuthClosureStatus::MissingBaseUrl
        );
        let endpoint_blocked = !matches!(
            endpoint_policy_status,
            KISEndpointPolicyStatus::MarketDataOnly | KISEndpointPolicyStatus::DiagnosticOnly
        );
        let dry_run_status = if matches!(
            auth_report.closure_status,
            KISAuthClosureStatus::DiagnosticOnly
        ) && config.domestic_symbol_whitelist_path.is_none()
            && config.overseas_symbol_whitelist_path.is_none()
        {
            KISMarketDataDryRunStatus::DiagnosticOnly
        } else if auth_missing {
            KISMarketDataDryRunStatus::MissingAuth
        } else if base_url_missing {
            KISMarketDataDryRunStatus::MissingBaseUrl
        } else if endpoint_blocked {
            KISMarketDataDryRunStatus::EndpointPolicyBlocked
        } else if scope_too_broad {
            KISMarketDataDryRunStatus::ScopeTooBroad
        } else if budget_exceeded {
            KISMarketDataDryRunStatus::BudgetExceeded
        } else if no_symbols {
            KISMarketDataDryRunStatus::NoSymbols
        } else {
            KISMarketDataDryRunStatus::Ready
        };
        let mut blocked_reasons = Vec::new();
        if auth_missing {
            blocked_reasons.push("app key or app secret is missing".to_string());
        }
        if base_url_missing {
            blocked_reasons.push("base url is missing".to_string());
        }
        if endpoint_blocked {
            blocked_reasons.push("endpoint policy blocked non-market-data categories".to_string());
        }
        if scope_too_broad {
            blocked_reasons.push("requested symbol scope exceeds bounded defaults".to_string());
        }
        if budget_exceeded {
            blocked_reasons.push("planned bytes exceed max_bytes".to_string());
        }
        if no_symbols {
            blocked_reasons.push("no enabled domestic or overseas symbols were found".to_string());
        }
        let mut reason_codes = config.reason_codes.clone();
        reason_codes.push(ReasonCode::KISMarketDataDryRunBuilt);
        if budget_exceeded {
            reason_codes.push(ReasonCode::BudgetExceeded);
        }
        if scope_too_broad {
            reason_codes.push(ReasonCode::DeniedByDefault);
        }
        if endpoint_blocked {
            reason_codes.push(ReasonCode::KISEndpointDenied);
        }
        let report = KISMarketDataDryRunReport {
            dry_run_id: config.dry_run_id.clone(),
            auth_status: auth_report.closure_status,
            endpoint_policy_status,
            planned_domestic_symbols,
            planned_overseas_symbols,
            planned_requests,
            planned_rows,
            planned_days,
            planned_bytes,
            dry_run_status,
            safe_to_run_operator_live_collection: matches!(
                dry_run_status,
                KISMarketDataDryRunStatus::Ready
            ) && matches!(
                auth_report.closure_status,
                KISAuthClosureStatus::Ready
            ),
            blocked_reasons,
            reason_codes: stable_reason_codes(&reason_codes),
        };
        report.write_to_dir(&config.artifact_dir())?;
        Ok(report)
    }
}

fn load_whitelist(path: Option<&str>) -> Result<Option<KISSymbolWhitelist>, String> {
    path.map(|path| {
        let config = KISSymbolWhitelistConfig::from_toml_path(Path::new(path))?;
        config.validate()?;
        Ok(config.build())
    })
    .transpose()
}
