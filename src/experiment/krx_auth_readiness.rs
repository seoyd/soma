use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};

use super::krx_official_activation::KRXOfficialEvidenceActivationConfig;

pub const KRX_API_KEY_ENV_VAR: &str = "KRX_API_KEY";
pub const KRX_ENDPOINT_TEMPLATE_ENV_VAR: &str = "KRX_ENDPOINT_TEMPLATE";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum KRXAuthReadinessStatus {
    Ready,
    MissingApiKey,
    MissingEndpointTemplate,
    MissingApiKeyAndEndpointTemplate,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KRXAuthReadinessReport {
    pub api_key_env_var_name: String,
    pub api_key_present: bool,
    pub endpoint_template_env_var_name: String,
    pub endpoint_template_present: bool,
    #[serde(default)]
    pub endpoint_template_preview_redacted: Option<String>,
    pub readiness_status: KRXAuthReadinessStatus,
    pub safe_to_collect_market_data: bool,
    pub reason_codes: Vec<ReasonCode>,
}

impl KRXAuthReadinessReport {
    pub fn from_config(config: &KRXOfficialEvidenceActivationConfig) -> Self {
        let api_key_present = env_var_present(KRX_API_KEY_ENV_VAR);
        let endpoint_value = env::var(KRX_ENDPOINT_TEMPLATE_ENV_VAR).ok();
        let endpoint_template_present = endpoint_value
            .as_deref()
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false);
        let readiness_status = match (
            config.require_krx_api_key,
            config.require_krx_endpoint_template,
            api_key_present,
            endpoint_template_present,
        ) {
            (false, false, _, _) => KRXAuthReadinessStatus::DiagnosticOnly,
            (_, _, false, false)
                if config.require_krx_api_key && config.require_krx_endpoint_template =>
            {
                KRXAuthReadinessStatus::MissingApiKeyAndEndpointTemplate
            }
            (_, _, false, _) if config.require_krx_api_key => KRXAuthReadinessStatus::MissingApiKey,
            (_, _, _, false) if config.require_krx_endpoint_template => {
                KRXAuthReadinessStatus::MissingEndpointTemplate
            }
            _ => KRXAuthReadinessStatus::Ready,
        };
        let mut reason_codes = vec![ReasonCode::KRXAuthReadinessBuilt];
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
        Self {
            api_key_env_var_name: KRX_API_KEY_ENV_VAR.to_string(),
            api_key_present,
            endpoint_template_env_var_name: KRX_ENDPOINT_TEMPLATE_ENV_VAR.to_string(),
            endpoint_template_present,
            endpoint_template_preview_redacted: endpoint_value
                .as_deref()
                .filter(|value| !value.trim().is_empty())
                .map(redacted_endpoint_preview),
            readiness_status,
            safe_to_collect_market_data: matches!(readiness_status, KRXAuthReadinessStatus::Ready),
            reason_codes: stable_reason_codes(&reason_codes),
        }
    }

    pub fn to_text(&self) -> String {
        [
            "research_only_warning=krx auth readiness is market-data-only and never implies live trading".to_string(),
            "secret_safety_warning=reports expose env-var names and redacted endpoint previews only".to_string(),
            format!("api_key_env_var_name={}", self.api_key_env_var_name),
            format!("api_key_present={}", self.api_key_present),
            format!(
                "endpoint_template_env_var_name={}",
                self.endpoint_template_env_var_name
            ),
            format!("endpoint_template_present={}", self.endpoint_template_present),
            format!(
                "endpoint_template_preview_redacted={}",
                self.endpoint_template_preview_redacted
                    .clone()
                    .unwrap_or_default()
            ),
            format!("readiness_status={:?}", self.readiness_status),
            format!(
                "safe_to_collect_market_data={}",
                self.safe_to_collect_market_data
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
        let text_path = output_dir.join("krx_auth_readiness.txt");
        fs::write(&text_path, self.to_text()).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("krx_auth_readiness.json"),
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
