use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};

use super::kis_market_data_activation::KISMarketDataActivationConfig;

pub const KIS_APP_KEY_ENV_VAR: &str = "KIS_APP_KEY";
pub const KIS_APP_SECRET_ENV_VAR: &str = "KIS_APP_SECRET";
pub const KIS_BASE_URL_ENV_VAR: &str = "KIS_BASE_URL";
pub const KIS_WS_APPROVAL_KEY_ENV_VAR: &str = "KIS_WS_APPROVAL_KEY";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum KISAuthReadinessStatus {
    Ready,
    MissingAppKey,
    MissingAppSecret,
    MissingBaseUrl,
    MissingAppKeyAndSecret,
    MissingWebSocketApprovalKey,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KISAuthReadinessReport {
    pub app_key_env_var_name: String,
    pub app_key_present: bool,
    pub app_secret_env_var_name: String,
    pub app_secret_present: bool,
    pub base_url_env_var_name: String,
    pub base_url_present: bool,
    pub websocket_approval_key_env_var_name: String,
    pub websocket_approval_key_present: bool,
    #[serde(default)]
    pub base_url_preview_redacted: Option<String>,
    pub readiness_status: KISAuthReadinessStatus,
    pub safe_to_collect_rest_market_data: bool,
    pub safe_to_collect_realtime_market_data: bool,
    pub reason_codes: Vec<ReasonCode>,
}

impl KISAuthReadinessReport {
    pub fn from_config(config: &KISMarketDataActivationConfig) -> Self {
        let app_key_present = env_var_present(KIS_APP_KEY_ENV_VAR);
        let app_secret_present = env_var_present(KIS_APP_SECRET_ENV_VAR);
        let base_url_value = env::var(KIS_BASE_URL_ENV_VAR).ok();
        let base_url_present = base_url_value
            .as_deref()
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false);
        let websocket_approval_key_present = env_var_present(KIS_WS_APPROVAL_KEY_ENV_VAR);

        let rest_required = config.require_kis_app_key
            || config.require_kis_app_secret
            || config.require_kis_base_url;
        let rest_ready = (!config.require_kis_app_key || app_key_present)
            && (!config.require_kis_app_secret || app_secret_present)
            && (!config.require_kis_base_url || base_url_present);
        let realtime_requested = config.run_live_market_data_collection
            && config.run_collection_dry_run
            && config
                .requested_endpoint_categories()
                .iter()
                .any(|category| category.requires_websocket_approval());
        let realtime_ready = rest_ready && (!realtime_requested || websocket_approval_key_present);

        let readiness_status = if !rest_required && !realtime_requested {
            KISAuthReadinessStatus::DiagnosticOnly
        } else if rest_ready && realtime_requested && !websocket_approval_key_present {
            KISAuthReadinessStatus::MissingWebSocketApprovalKey
        } else if config.require_kis_app_key
            && !app_key_present
            && config.require_kis_app_secret
            && !app_secret_present
        {
            KISAuthReadinessStatus::MissingAppKeyAndSecret
        } else if config.require_kis_app_key && !app_key_present {
            KISAuthReadinessStatus::MissingAppKey
        } else if config.require_kis_app_secret && !app_secret_present {
            KISAuthReadinessStatus::MissingAppSecret
        } else if config.require_kis_base_url && !base_url_present {
            KISAuthReadinessStatus::MissingBaseUrl
        } else {
            KISAuthReadinessStatus::Ready
        };

        let mut reason_codes = vec![ReasonCode::KISAuthReadinessBuilt];
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
        if realtime_requested && !websocket_approval_key_present {
            reason_codes.push(ReasonCode::MissingApproval);
        }

        Self {
            app_key_env_var_name: KIS_APP_KEY_ENV_VAR.to_string(),
            app_key_present,
            app_secret_env_var_name: KIS_APP_SECRET_ENV_VAR.to_string(),
            app_secret_present,
            base_url_env_var_name: KIS_BASE_URL_ENV_VAR.to_string(),
            base_url_present,
            websocket_approval_key_env_var_name: KIS_WS_APPROVAL_KEY_ENV_VAR.to_string(),
            websocket_approval_key_present,
            base_url_preview_redacted: base_url_value
                .as_deref()
                .filter(|value| !value.trim().is_empty())
                .map(redacted_base_url_preview),
            readiness_status,
            safe_to_collect_rest_market_data: rest_ready,
            safe_to_collect_realtime_market_data: realtime_ready,
            reason_codes: stable_reason_codes(&reason_codes),
        }
    }

    pub fn to_text(&self) -> String {
        [
            "research_only_warning=kis auth readiness is research-only and market-data-only".to_string(),
            "market_data_only_warning=no broker order or account surface is enabled".to_string(),
            "secret_safety_warning=reports expose env-var names and redacted base-url previews only".to_string(),
            format!("app_key_env_var_name={}", self.app_key_env_var_name),
            format!("app_key_present={}", self.app_key_present),
            format!("app_secret_env_var_name={}", self.app_secret_env_var_name),
            format!("app_secret_present={}", self.app_secret_present),
            format!("base_url_env_var_name={}", self.base_url_env_var_name),
            format!("base_url_present={}", self.base_url_present),
            format!(
                "websocket_approval_key_env_var_name={}",
                self.websocket_approval_key_env_var_name
            ),
            format!(
                "websocket_approval_key_present={}",
                self.websocket_approval_key_present
            ),
            format!(
                "base_url_preview_redacted={}",
                self.base_url_preview_redacted.clone().unwrap_or_default()
            ),
            format!("readiness_status={:?}", self.readiness_status),
            format!(
                "safe_to_collect_rest_market_data={}",
                self.safe_to_collect_rest_market_data
            ),
            format!(
                "safe_to_collect_realtime_market_data={}",
                self.safe_to_collect_realtime_market_data
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

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        let text_path = output_dir.join("kis_auth_readiness.txt");
        fs::write(&text_path, self.to_text()).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("kis_auth_readiness.json"),
            self.to_json_string()?,
        )
        .map_err(|err| err.to_string())?;
        Ok(text_path)
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
    let has_query = normalized.contains('?');
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
        let mut parts = host.split('.').collect::<Vec<_>>();
        if parts.len() > 2 {
            parts.remove(0);
        }
        format!("***.{}", parts.join("."))
    };
    format!("configured(redacted;base_url={host_preview};https={has_https};query={has_query})")
}
