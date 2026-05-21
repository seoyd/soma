use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_hash_string, stable_ordered_strings, stable_reason_codes};

fn default_output_root() -> String {
    "target/sprint58/secret_redaction_audit".to_string()
}

fn default_secret_env_var_names() -> Vec<String> {
    vec![
        "KIS_APP_KEY".to_string(),
        "KIS_APP_SECRET".to_string(),
        "KIS_WS_APPROVAL_KEY".to_string(),
        "KRX_API_KEY".to_string(),
    ]
}

fn default_true() -> bool {
    true
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SecretRedactionAuditConfig {
    pub audit_id: String,
    #[serde(default)]
    pub artifact_paths: Vec<String>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default = "default_secret_env_var_names")]
    pub secret_env_var_names: Vec<String>,
    #[serde(default = "default_true")]
    pub token_like_patterns_enabled: bool,
    #[serde(default = "default_true")]
    pub reject_account_like_fields: bool,
    #[serde(default = "default_true")]
    pub reject_order_like_fields: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecretRedactionStatus {
    Passed,
    FailedSecretLeak,
    FailedTokenLeak,
    FailedAccountField,
    FailedOrderField,
    #[default]
    DiagnosticOnly,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecretRedactionAuditReport {
    pub audit_id: String,
    pub artifacts_scanned: usize,
    pub secret_leaks_detected: bool,
    pub token_like_values_detected: bool,
    pub account_like_fields_detected: bool,
    pub order_like_fields_detected: bool,
    pub redaction_status: SecretRedactionStatus,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SecretRedactionAuditRunner;

impl Default for SecretRedactionAuditConfig {
    fn default() -> Self {
        Self {
            audit_id: "sprint58-secret-redaction-audit".to_string(),
            artifact_paths: Vec::new(),
            output_root: default_output_root(),
            secret_env_var_names: default_secret_env_var_names(),
            token_like_patterns_enabled: true,
            reject_account_like_fields: true,
            reject_order_like_fields: true,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

impl SecretRedactionAuditConfig {
    pub fn from_toml_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        toml::from_str(&text).map_err(|err| err.to_string())
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.audit_id.trim().is_empty() {
            return Err("secret redaction audit id must not be empty".to_string());
        }
        let mut paths = self.artifact_paths.clone();
        paths.push(self.output_root.clone());
        if paths.iter().any(|path| path.contains("://")) {
            return Err("secret redaction audit paths must be local".to_string());
        }
        Ok(())
    }

    pub fn artifact_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.audit_id)
    }
}

impl SecretRedactionAuditReport {
    pub fn to_text(&self) -> String {
        [
            "secret_safety_warning=secret redaction audit scans local artifacts only".to_string(),
            format!("audit_id={}", self.audit_id),
            format!("artifacts_scanned={}", self.artifacts_scanned),
            format!("secret_leaks_detected={}", self.secret_leaks_detected),
            format!(
                "token_like_values_detected={}",
                self.token_like_values_detected
            ),
            format!(
                "account_like_fields_detected={}",
                self.account_like_fields_detected
            ),
            format!(
                "order_like_fields_detected={}",
                self.order_like_fields_detected
            ),
            format!("redaction_status={:?}", self.redaction_status),
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
        let text_path = output_dir.join("secret_redaction_audit.txt");
        fs::write(&text_path, self.to_text()).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("secret_redaction_audit.json"),
            serde_json::to_string_pretty(self).map_err(|err| err.to_string())?,
        )
        .map_err(|err| err.to_string())?;
        Ok(text_path)
    }
}

impl SecretRedactionAuditRunner {
    pub fn run(
        &self,
        config: &SecretRedactionAuditConfig,
    ) -> Result<SecretRedactionAuditReport, String> {
        config.validate()?;
        let mut artifact_paths = stable_ordered_strings(&config.artifact_paths);
        artifact_paths.retain(|path| Path::new(path).is_file());

        let secret_values = config
            .secret_env_var_names
            .iter()
            .filter_map(|name| env::var(name).ok())
            .filter(|value| !value.trim().is_empty())
            .collect::<Vec<_>>();

        let mut secret_leaks_detected = false;
        let mut token_like_values_detected = false;
        let mut account_like_fields_detected = false;
        let mut order_like_fields_detected = false;

        for path in &artifact_paths {
            let text = fs::read(path)
                .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
                .unwrap_or_default();
            let lower = text.to_ascii_lowercase();
            if !secret_leaks_detected
                && secret_values
                    .iter()
                    .any(|value| !value.is_empty() && text.contains(value))
            {
                secret_leaks_detected = true;
            }
            if config.token_like_patterns_enabled && !token_like_values_detected {
                token_like_values_detected = contains_token_like_value(&text);
            }
            if config.reject_account_like_fields && !account_like_fields_detected {
                account_like_fields_detected = contains_account_like_field(&lower);
            }
            if config.reject_order_like_fields && !order_like_fields_detected {
                order_like_fields_detected = contains_order_like_field(&lower);
            }
        }

        let redaction_status = if artifact_paths.is_empty() {
            SecretRedactionStatus::DiagnosticOnly
        } else if secret_leaks_detected {
            SecretRedactionStatus::FailedSecretLeak
        } else if token_like_values_detected {
            SecretRedactionStatus::FailedTokenLeak
        } else if account_like_fields_detected {
            SecretRedactionStatus::FailedAccountField
        } else if order_like_fields_detected {
            SecretRedactionStatus::FailedOrderField
        } else {
            SecretRedactionStatus::Passed
        };

        let mut reason_codes = config.reason_codes.clone();
        reason_codes.push(ReasonCode::SecretRedactionAuditBuilt);
        if secret_leaks_detected {
            reason_codes.push(ReasonCode::SecretLeakDetected);
        }
        if token_like_values_detected {
            reason_codes.push(ReasonCode::TokenLeakDetected);
        }
        if account_like_fields_detected {
            reason_codes.push(ReasonCode::AccountFieldDetected);
        }
        if order_like_fields_detected {
            reason_codes.push(ReasonCode::OrderFieldDetected);
        }
        let report = SecretRedactionAuditReport {
            audit_id: config.audit_id.clone(),
            artifacts_scanned: artifact_paths.len(),
            secret_leaks_detected,
            token_like_values_detected,
            account_like_fields_detected,
            order_like_fields_detected,
            redaction_status,
            reason_codes: stable_reason_codes(&reason_codes),
        };
        report.write_to_dir(&config.artifact_dir())?;
        Ok(report)
    }
}

fn contains_account_like_field(text: &str) -> bool {
    [
        "\"account_id\"",
        "\"account_no\"",
        "\"account_number\"",
        "\"acct_no\"",
        "account_id=",
        "account_no=",
        "acct_no=",
    ]
    .iter()
    .any(|needle| text.contains(needle))
}

fn contains_order_like_field(text: &str) -> bool {
    [
        "\"order_id\"",
        "\"order_no\"",
        "\"ord_no\"",
        "order_id=",
        "order_no=",
        "ord_no=",
    ]
    .iter()
    .any(|needle| text.contains(needle))
}

fn contains_token_like_value(text: &str) -> bool {
    text.split(|ch: char| {
        ch.is_whitespace() || ['"', '\'', ',', ';', '(', ')', '[', ']', '{', '}'].contains(&ch)
    })
    .filter(|token| token.len() >= 24)
    .any(|token| {
        let stripped = token.trim_matches(|ch: char| ch == ':' || ch == '=');
        let has_alpha = stripped.chars().any(|ch| ch.is_ascii_alphabetic());
        let has_digit = stripped.chars().any(|ch| ch.is_ascii_digit());
        let all_hex = stripped.chars().all(|ch| ch.is_ascii_hexdigit());
        let looks_like_identifier = is_pascal_or_camel_identifier(stripped);
        let separator_count = stripped
            .chars()
            .filter(|ch| matches!(ch, '_' | '-' | '/'))
            .count();
        stripped.len() >= 24
            && has_alpha
            && has_digit
            && !all_hex
            && !looks_like_identifier
            && separator_count <= 1
            && stripped
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '/'))
            && stripped != stable_hash_string(stripped)
    })
}

fn is_pascal_or_camel_identifier(token: &str) -> bool {
    let has_upper = token.chars().any(|ch| ch.is_ascii_uppercase());
    let has_lower = token.chars().any(|ch| ch.is_ascii_lowercase());
    let starts_alpha = token
        .chars()
        .next()
        .is_some_and(|ch| ch.is_ascii_alphabetic());
    let has_identifier_shape = token.chars().all(|ch| ch.is_ascii_alphanumeric());
    let uppercase_runs = token
        .chars()
        .zip(token.chars().skip(1))
        .filter(|(left, right)| left.is_ascii_lowercase() && right.is_ascii_uppercase())
        .count();
    starts_alpha && has_upper && has_lower && has_identifier_shape && uppercase_runs >= 2
}
