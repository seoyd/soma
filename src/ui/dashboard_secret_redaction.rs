use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::core::{ReasonCode, stable_hash_string, stable_ordered_strings, stable_reason_codes};

use super::dashboard_state::DashboardState;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DashboardSecretRedactionReport {
    #[serde(default)]
    pub redacted_field_paths: Vec<String>,
    #[serde(default)]
    pub rejected_field_paths: Vec<String>,
    pub passed: bool,
    pub fingerprint: String,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

pub fn redact_dashboard_state(
    state: &DashboardState,
) -> Result<(DashboardState, DashboardSecretRedactionReport), String> {
    let mut value = serde_json::to_value(state).map_err(|err| err.to_string())?;
    let mut redacted = Vec::new();
    let mut rejected = Vec::new();
    redact_value("$", &mut value, &mut redacted, &mut rejected);
    redacted = stable_ordered_strings(&redacted);
    rejected = stable_ordered_strings(&rejected);
    let mut reason_codes = vec![ReasonCode::DashboardSecretRedacted];
    let serialized = serde_json::to_string(&value).map_err(|err| err.to_string())?;
    let passed = !contains_secret_like_material(&serialized);
    if !passed {
        rejected.push("$".to_string());
        reason_codes.push(ReasonCode::DashboardSecretRejected);
        reason_codes.push(ReasonCode::UnsafeSecretExposure);
    }
    let mut state: DashboardState = serde_json::from_value(value).map_err(|err| err.to_string())?;
    state = state.with_fingerprint();
    let report = DashboardSecretRedactionReport {
        redacted_field_paths: redacted,
        rejected_field_paths: rejected,
        passed,
        fingerprint: stable_hash_string(&serialized),
        reason_codes: stable_reason_codes(&reason_codes),
    };
    Ok((state, report))
}

fn redact_value(
    path: &str,
    value: &mut Value,
    redacted: &mut Vec<String>,
    rejected: &mut Vec<String>,
) {
    match value {
        Value::Object(map) => {
            let mut keys = map.keys().cloned().collect::<Vec<_>>();
            keys.sort();
            for key in keys {
                if let Some(child) = map.get_mut(&key) {
                    let child_path = format!("{path}.{key}");
                    if is_sensitive_key(&key) {
                        if !matches!(child, Value::Null) {
                            *child = Value::String(redacted_placeholder_for_key(&key));
                            redacted.push(child_path);
                        }
                    } else {
                        redact_value(&child_path, child, redacted, rejected);
                    }
                }
            }
        }
        Value::Array(items) => {
            for (index, item) in items.iter_mut().enumerate() {
                redact_value(&format!("{path}[{index}]"), item, redacted, rejected);
            }
        }
        Value::String(text) => {
            if looks_sensitive_string(text) {
                let redacted_value = redact_string_value(text);
                if redacted_value == *text {
                    rejected.push(path.to_string());
                } else {
                    redacted.push(path.to_string());
                    *text = redacted_value;
                }
            }
        }
        _ => {}
    }
}

fn is_sensitive_key(key: &str) -> bool {
    let key = key.to_ascii_lowercase();
    ["key", "secret", "token", "password", "approval", "base_url"]
        .iter()
        .any(|needle| key.contains(needle))
}

fn looks_sensitive_string(text: &str) -> bool {
    let normalized = text.to_ascii_lowercase();
    normalized.contains("kis_app_key")
        || normalized.contains("kis_app_secret")
        || normalized.contains("kis_ws_approval_key")
        || normalized.contains("krx_api_key")
        || normalized.contains("token=")
        || normalized.contains("secret=")
        || normalized.contains("password=")
        || normalized.contains("appkey=")
        || normalized.contains("app_key=")
        || normalized.contains("appsecret=")
        || normalized.contains("app_secret=")
        || normalized.contains("kis_base_url")
        || ((normalized.starts_with("http://") || normalized.starts_with("https://"))
            && (normalized.contains('?')
                || normalized.contains("token")
                || normalized.contains("key")
                || normalized.contains("secret")))
}

fn redact_string_value(text: &str) -> String {
    let normalized = text.trim();
    if normalized.starts_with("http://") || normalized.starts_with("https://") {
        let https = normalized.starts_with("https://");
        let host = normalized
            .split("//")
            .nth(1)
            .unwrap_or(normalized)
            .split('/')
            .next()
            .unwrap_or("unknown-host");
        let host_tail = host
            .split('.')
            .rev()
            .take(2)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join(".");
        return format!(
            "configured(redacted;base_url=***.{host_tail};https={https};query={})",
            normalized.contains('?')
        );
    }
    if normalized.contains('=') {
        return "[REDACTED]".to_string();
    }
    "[REDACTED]".to_string()
}

fn redacted_placeholder_for_key(key: &str) -> String {
    if key.eq_ignore_ascii_case("base_url") || key.eq_ignore_ascii_case("kis_base_url") {
        "configured(redacted;base_url=***.redacted;https=true;query=false)".to_string()
    } else {
        "[REDACTED]".to_string()
    }
}

fn contains_secret_like_material(serialized: &str) -> bool {
    let normalized = serialized.to_ascii_lowercase();
    normalized.contains("top-secret")
        || normalized.contains("secret-value")
        || normalized.contains("token=")
        || normalized.contains("password=")
        || normalized.contains("app_key=")
        || normalized.contains("app_secret=")
}
