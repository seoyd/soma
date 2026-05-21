use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LiveSafetyStatus {
    SafeResearchOnly,
    UnsafePathDetected,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiveSafetyReport {
    pub live_mode_constructible: bool,
    pub broker_path_present: bool,
    pub order_execution_path_present: bool,
    pub account_api_path_present: bool,
    pub credential_storage_present: bool,
    pub runtime_llm_path_present: bool,
    pub unsafe_cli_commands: Vec<String>,
    pub status: LiveSafetyStatus,
    pub reason_codes: Vec<ReasonCode>,
}

pub fn build_live_safety_report(
    command_names: &[String],
    runtime_llm_path_present: bool,
) -> LiveSafetyReport {
    let unsafe_cli_commands = command_names
        .iter()
        .filter(|name| {
            let name = name.as_str();
            name.contains("live")
                || name.contains("broker")
                || name.contains("order")
                || name.contains("account")
        })
        .cloned()
        .collect::<Vec<_>>();
    let unsafe_detected = runtime_llm_path_present || !unsafe_cli_commands.is_empty();
    LiveSafetyReport {
        live_mode_constructible: false,
        broker_path_present: false,
        order_execution_path_present: false,
        account_api_path_present: false,
        credential_storage_present: false,
        runtime_llm_path_present,
        unsafe_cli_commands,
        status: if unsafe_detected {
            LiveSafetyStatus::UnsafePathDetected
        } else {
            LiveSafetyStatus::SafeResearchOnly
        },
        reason_codes: vec![if unsafe_detected {
            ReasonCode::LiveSafetyUnsafePath
        } else {
            ReasonCode::LiveSafetyReportBuilt
        }],
    }
}

impl LiveSafetyReport {
    pub fn to_text(&self) -> String {
        [
            format!("live_mode_constructible={}", self.live_mode_constructible),
            format!("broker_path_present={}", self.broker_path_present),
            format!(
                "order_execution_path_present={}",
                self.order_execution_path_present
            ),
            format!("account_api_path_present={}", self.account_api_path_present),
            format!(
                "credential_storage_present={}",
                self.credential_storage_present
            ),
            format!("runtime_llm_path_present={}", self.runtime_llm_path_present),
            format!("unsafe_cli_commands={}", self.unsafe_cli_commands.join("|")),
            format!("status={:?}", self.status),
        ]
        .join("\n")
    }
}
