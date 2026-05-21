use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum DashboardServeStatus {
    #[default]
    DeferredForSafety,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DashboardServeReport {
    pub status: DashboardServeStatus,
    pub bind_address: String,
    pub methods_allowed: String,
    pub deferred_reason: String,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl DashboardServeReport {
    pub fn deferred() -> Self {
        Self {
            status: DashboardServeStatus::DeferredForSafety,
            bind_address: "127.0.0.1".to_string(),
            methods_allowed: "GET".to_string(),
            deferred_reason: "local server deferred for safety; static dashboard-open/render path is enough for now".to_string(),
            reason_codes: stable_reason_codes(&[ReasonCode::LocalFileOnly]),
        }
    }

    pub fn to_text(&self) -> String {
        [
            "localhost_warning=dashboard-serve stays deferred unless localhost-only GET static serving is trivial and safe".to_string(),
            format!("status={:?}", self.status),
            format!("bind_address={}", self.bind_address),
            format!("methods_allowed={}", self.methods_allowed),
            format!("deferred_reason={}", self.deferred_reason),
        ]
        .join("\n")
    }
}
