use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};

use super::kis_auth_readiness::{
    KIS_APP_KEY_ENV_VAR, KIS_APP_SECRET_ENV_VAR, KIS_BASE_URL_ENV_VAR, KIS_WS_APPROVAL_KEY_ENV_VAR,
};
use super::kis_market_data_activation::KISMarketDataActivationConfig;

fn default_output_root() -> String {
    "target/sprint58/kis_auth_closure".to_string()
}

fn default_true() -> bool {
    true
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KISAuthClosureConfig {
    pub closure_id: String,
    #[serde(default)]
    pub kis_activation_config_paths: Vec<String>,
    #[serde(default)]
    pub provider_readiness_report_paths: Vec<String>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default = "default_true")]
    pub require_app_key: bool,
    #[serde(default = "default_true")]
    pub require_app_secret: bool,
    #[serde(default = "default_true")]
    pub require_base_url: bool,
    #[serde(default)]
    pub require_ws_approval_key_for_realtime: bool,
    #[serde(default = "default_true")]
    pub redact_base_url: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum KISAuthClosureStatus {
    Ready,
    MissingAppKey,
    MissingAppSecret,
    MissingBaseUrl,
    MissingAppKeyAndSecret,
    MissingWebSocketApprovalKeyForRealtime,
    ReadyForDryRunOnly,
    #[default]
    DiagnosticOnly,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct KISAuthClosureReport {
    pub closure_id: String,
    pub app_key_env_var_name: String,
    pub app_key_present: bool,
    pub app_secret_env_var_name: String,
    pub app_secret_present: bool,
    pub base_url_env_var_name: String,
    pub base_url_present: bool,
    pub ws_approval_key_env_var_name: String,
    pub ws_approval_key_present: bool,
    #[serde(default)]
    pub base_url_preview_redacted: Option<String>,
    pub closure_status: KISAuthClosureStatus,
    pub safe_for_rest_market_data_dry_run: bool,
    pub safe_for_rest_market_data_live: bool,
    pub safe_for_realtime_market_data: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct KISAuthClosureRunner;

impl Default for KISAuthClosureConfig {
    fn default() -> Self {
        Self {
            closure_id: "sprint58-kis-auth-close".to_string(),
            kis_activation_config_paths: Vec::new(),
            provider_readiness_report_paths: Vec::new(),
            output_root: default_output_root(),
            require_app_key: true,
            require_app_secret: true,
            require_base_url: true,
            require_ws_approval_key_for_realtime: false,
            redact_base_url: true,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

impl KISAuthClosureConfig {
    pub fn from_toml_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        toml::from_str(&text).map_err(|err| err.to_string())
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.closure_id.trim().is_empty() {
            return Err("kis auth closure id must not be empty".to_string());
        }
        if self
            .kis_activation_config_paths
            .iter()
            .chain(self.provider_readiness_report_paths.iter())
            .chain(std::iter::once(&self.output_root))
            .any(|path| path.contains("://"))
        {
            return Err("kis auth closure paths must be local".to_string());
        }
        Ok(())
    }

    pub fn artifact_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.closure_id)
    }
}

impl KISAuthClosureReport {
    pub fn to_text(&self) -> String {
        [
            "research_only_warning=kis auth closure is market-data-only and research-only"
                .to_string(),
            "secret_safety_warning=only env var names and redacted base-url previews are rendered"
                .to_string(),
            format!("closure_id={}", self.closure_id),
            format!("app_key_env_var_name={}", self.app_key_env_var_name),
            format!("app_key_present={}", self.app_key_present),
            format!("app_secret_env_var_name={}", self.app_secret_env_var_name),
            format!("app_secret_present={}", self.app_secret_present),
            format!("base_url_env_var_name={}", self.base_url_env_var_name),
            format!("base_url_present={}", self.base_url_present),
            format!(
                "ws_approval_key_env_var_name={}",
                self.ws_approval_key_env_var_name
            ),
            format!("ws_approval_key_present={}", self.ws_approval_key_present),
            format!(
                "base_url_preview_redacted={}",
                self.base_url_preview_redacted.clone().unwrap_or_default()
            ),
            format!("closure_status={:?}", self.closure_status),
            format!(
                "safe_for_rest_market_data_dry_run={}",
                self.safe_for_rest_market_data_dry_run
            ),
            format!(
                "safe_for_rest_market_data_live={}",
                self.safe_for_rest_market_data_live
            ),
            format!(
                "safe_for_realtime_market_data={}",
                self.safe_for_realtime_market_data
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

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        let text_path = output_dir.join("kis_auth_closure.txt");
        fs::write(&text_path, self.to_text()).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("kis_auth_closure.json"),
            serde_json::to_string_pretty(self).map_err(|err| err.to_string())?,
        )
        .map_err(|err| err.to_string())?;
        Ok(text_path)
    }
}

impl KISAuthClosureRunner {
    pub fn run(&self, config: &KISAuthClosureConfig) -> Result<KISAuthClosureReport, String> {
        config.validate()?;

        let app_key_present = env_var_present(KIS_APP_KEY_ENV_VAR);
        let app_secret_present = env_var_present(KIS_APP_SECRET_ENV_VAR);
        let base_url_value = env::var(KIS_BASE_URL_ENV_VAR).ok();
        let base_url_present = base_url_value
            .as_deref()
            .is_some_and(|value| !value.trim().is_empty());
        let ws_approval_key_present = env_var_present(KIS_WS_APPROVAL_KEY_ENV_VAR);
        let activation_requires_live = config
            .kis_activation_config_paths
            .iter()
            .filter_map(|path| KISMarketDataActivationConfig::from_toml_path(Path::new(path)).ok())
            .any(|activation| activation.run_live_market_data_collection);
        let require_ws = config.require_ws_approval_key_for_realtime || activation_requires_live;
        let rest_ready = (!config.require_app_key || app_key_present)
            && (!config.require_app_secret || app_secret_present)
            && (!config.require_base_url || base_url_present);
        let safe_for_rest_market_data_dry_run = rest_ready;
        let safe_for_rest_market_data_live = rest_ready && activation_requires_live;
        let safe_for_realtime_market_data = rest_ready && (!require_ws || ws_approval_key_present);

        let closure_status = if !config.require_app_key
            && !config.require_app_secret
            && !config.require_base_url
            && !require_ws
        {
            KISAuthClosureStatus::DiagnosticOnly
        } else if config.require_app_key
            && !app_key_present
            && config.require_app_secret
            && !app_secret_present
        {
            KISAuthClosureStatus::MissingAppKeyAndSecret
        } else if config.require_app_key && !app_key_present {
            KISAuthClosureStatus::MissingAppKey
        } else if config.require_app_secret && !app_secret_present {
            KISAuthClosureStatus::MissingAppSecret
        } else if config.require_base_url && !base_url_present {
            KISAuthClosureStatus::MissingBaseUrl
        } else if require_ws && !ws_approval_key_present {
            KISAuthClosureStatus::MissingWebSocketApprovalKeyForRealtime
        } else if activation_requires_live {
            KISAuthClosureStatus::Ready
        } else {
            KISAuthClosureStatus::ReadyForDryRunOnly
        };

        let mut reason_codes = config.reason_codes.clone();
        reason_codes.push(ReasonCode::KISAuthClosureBuilt);
        if app_key_present {
            reason_codes.push(ReasonCode::AuthConfigValidated);
        } else {
            reason_codes.push(ReasonCode::MissingApiKey);
        }
        if app_secret_present {
            reason_codes.push(ReasonCode::AuthConfigValidated);
        } else {
            reason_codes.push(ReasonCode::MissingAuth);
        }
        if base_url_present {
            reason_codes.push(ReasonCode::KISBaseUrlPreviewRedacted);
        } else {
            reason_codes.push(ReasonCode::MissingEndpointTemplate);
        }
        if require_ws && !ws_approval_key_present {
            reason_codes.push(ReasonCode::MissingApproval);
        }

        let report = KISAuthClosureReport {
            closure_id: config.closure_id.clone(),
            app_key_env_var_name: KIS_APP_KEY_ENV_VAR.to_string(),
            app_key_present,
            app_secret_env_var_name: KIS_APP_SECRET_ENV_VAR.to_string(),
            app_secret_present,
            base_url_env_var_name: KIS_BASE_URL_ENV_VAR.to_string(),
            base_url_present,
            ws_approval_key_env_var_name: KIS_WS_APPROVAL_KEY_ENV_VAR.to_string(),
            ws_approval_key_present,
            base_url_preview_redacted: if config.redact_base_url {
                base_url_value
                    .as_deref()
                    .filter(|value| !value.trim().is_empty())
                    .map(redacted_base_url_preview)
            } else {
                None
            },
            closure_status,
            safe_for_rest_market_data_dry_run,
            safe_for_rest_market_data_live,
            safe_for_realtime_market_data,
            reason_codes: stable_reason_codes(&reason_codes),
        };
        report.write_to_dir(&config.artifact_dir())?;
        Ok(report)
    }
}

fn env_var_present(name: &str) -> bool {
    env::var_os(name)
        .map(|value| !value.is_empty())
        .unwrap_or(false)
}

fn redacted_base_url_preview(value: &str) -> String {
    let normalized = value.trim();
    let has_https = normalized.starts_with("https://");
    let host = normalized
        .split("//")
        .nth(1)
        .unwrap_or(normalized)
        .split('/')
        .next()
        .unwrap_or_default();
    let host_preview = if host.is_empty() {
        "unknown-host".to_string()
    } else {
        let parts = host.split('.').collect::<Vec<_>>();
        if parts.len() <= 2 {
            format!("***.{}", host)
        } else {
            format!("***.{}", parts[1..].join("."))
        }
    };
    format!("configured(redacted;base_url={host_preview};https={has_https})")
}
