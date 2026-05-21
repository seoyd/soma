use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::core::{ReasonCode, stable_ordered_strings, stable_reason_codes};

use super::dashboard_panels::{EvidencePanel, ProviderPanel};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum KISMonitorStatus {
    Ready,
    MissingAuth,
    MissingBaseUrl,
    EndpointPolicyBlocked,
    CollectionReady,
    CollectionSkipped,
    CandleSufficiencyReady,
    MissingFutureWindows,
    OutcomeLinksAvailable,
    NeedOutcomeLinks,
    NeedMoreKISEvidence,
    DiagnosticOnly,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct KISMonitorPanel {
    pub auth_ready: bool,
    pub base_url_ready: bool,
    #[serde(default)]
    pub websocket_ready: Option<bool>,
    pub endpoint_policy_status: String,
    pub domestic_whitelist_count: usize,
    pub overseas_whitelist_count: usize,
    pub collection_plan_status: String,
    pub live_collection_enabled: bool,
    #[serde(default)]
    pub last_collection_status: Option<String>,
    pub canonical_csv_count: usize,
    pub official_row_count: usize,
    pub candle_sufficiency_status: String,
    pub outcome_links: usize,
    pub counterfactuals: usize,
    pub complete_rows: usize,
    #[serde(default)]
    pub diversity_status: Option<String>,
    #[serde(default)]
    pub core_bottleneck: Option<String>,
    #[serde(default)]
    pub latest_depth_run_id: Option<String>,
    #[serde(default)]
    pub latest_depth_status: Option<String>,
    #[serde(default)]
    pub latest_outcome_closure: Option<String>,
    #[serde(default)]
    pub latest_next_command: Option<String>,
    #[serde(default)]
    pub next_kis_actions: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl KISMonitorPanel {
    pub fn stabilize(&mut self) {
        self.next_kis_actions = stable_ordered_strings(&self.next_kis_actions);
        if self.endpoint_policy_status.trim().is_empty() {
            self.endpoint_policy_status = "Unknown".to_string();
        }
        if self.collection_plan_status.trim().is_empty() {
            self.collection_plan_status = "Unknown".to_string();
        }
        if self.candle_sufficiency_status.trim().is_empty() {
            self.candle_sufficiency_status = "DiagnosticOnly".to_string();
        }
        if self
            .diversity_status
            .as_deref()
            .is_some_and(|value| value.trim().is_empty())
        {
            self.diversity_status = None;
        }
        if self
            .core_bottleneck
            .as_deref()
            .is_some_and(|value| value.trim().is_empty())
        {
            self.core_bottleneck = None;
        }
        if self
            .latest_depth_run_id
            .as_deref()
            .is_some_and(|value| value.trim().is_empty())
        {
            self.latest_depth_run_id = None;
        }
        if self
            .latest_depth_status
            .as_deref()
            .is_some_and(|value| value.trim().is_empty())
        {
            self.latest_depth_status = None;
        }
        if self
            .latest_outcome_closure
            .as_deref()
            .is_some_and(|value| value.trim().is_empty())
        {
            self.latest_outcome_closure = None;
        }
        if self
            .latest_next_command
            .as_deref()
            .is_some_and(|value| value.trim().is_empty())
        {
            self.latest_next_command = None;
        }
        self.reason_codes = stable_reason_codes(&self.reason_codes);
    }
}

pub fn build_kis_monitor_panel(
    provider_panel: &ProviderPanel,
    evidence_panel: &EvidencePanel,
    kis_activation_values: &[Value],
    kis_collection_closure_values: &[Value],
    kis_market_data_activation_values: &[Value],
) -> KISMonitorPanel {
    let mut panel = KISMonitorPanel {
        auth_ready: provider_panel.kis_status.auth_ready,
        base_url_ready: false,
        websocket_ready: None,
        endpoint_policy_status: provider_panel.kis_status.endpoint_policy_status.clone(),
        domestic_whitelist_count: 0,
        overseas_whitelist_count: 0,
        collection_plan_status: "Unknown".to_string(),
        live_collection_enabled: false,
        last_collection_status: None,
        canonical_csv_count: 0,
        official_row_count: 0,
        candle_sufficiency_status: evidence_panel.candle_sufficiency_status.clone(),
        outcome_links: evidence_panel.outcome_links,
        counterfactuals: evidence_panel.no_trade_counterfactuals
            + evidence_panel.risk_denied_counterfactuals,
        complete_rows: evidence_panel.official_complete_rows,
        diversity_status: Some(evidence_panel.diversity_status.clone())
            .filter(|value| !value.trim().is_empty()),
        core_bottleneck: Some(evidence_panel.current_bottleneck.clone())
            .filter(|value| !value.trim().is_empty()),
        latest_depth_run_id: None,
        latest_depth_status: None,
        latest_outcome_closure: None,
        latest_next_command: None,
        next_kis_actions: Vec::new(),
        reason_codes: vec![ReasonCode::DashboardStateBuilt],
    };

    for value in kis_activation_values
        .iter()
        .chain(kis_collection_closure_values.iter())
        .chain(kis_market_data_activation_values.iter())
    {
        panel.auth_ready |= bool_field(value, &["auth_ready", "app_key_present"]).unwrap_or(false);
        panel.base_url_ready |= bool_field(value, &["base_url_ready", "base_url_present"])
            .unwrap_or(false)
            || value
                .get("base_url_preview_redacted")
                .and_then(|item| item.as_str())
                .is_some();
        if let Some(websocket_ready) = bool_field(
            value,
            &["websocket_ready", "websocket_approval_key_present"],
        ) {
            panel.websocket_ready = Some(panel.websocket_ready.unwrap_or(false) || websocket_ready);
        }
        if let Some(status) = string_field(value, &["endpoint_policy_status", "policy_status"]) {
            panel.endpoint_policy_status = status;
        }
        panel.domestic_whitelist_count = panel.domestic_whitelist_count.max(usize_field(
            value,
            &[
                "domestic_whitelist_count",
                "domestic_series",
                "domestic_jobs",
            ],
        ));
        panel.overseas_whitelist_count = panel.overseas_whitelist_count.max(usize_field(
            value,
            &[
                "overseas_whitelist_count",
                "overseas_series",
                "overseas_jobs",
            ],
        ));
        if let Some(status) = string_field(
            value,
            &[
                "collection_plan_status",
                "collection_batch_plan_summary",
                "symbol_whitelist_summary",
            ],
        ) {
            panel.collection_plan_status = status;
        }
        panel.live_collection_enabled |=
            bool_field(value, &["live_collection_enabled"]).unwrap_or(false);
        if panel.last_collection_status.is_none() {
            panel.last_collection_status = string_field(
                value,
                &["last_collection_status", "final_status", "closure_status"],
            );
        }
        panel.canonical_csv_count = panel.canonical_csv_count.max(usize_field(
            value,
            &[
                "canonical_csv_count",
                "added_kis_canonical_csvs",
                "imported_canonical_csvs",
            ],
        ));
        panel.official_row_count = panel.official_row_count.max(usize_field(
            value,
            &[
                "official_row_count",
                "added_kis_official_rows",
                "official_rows",
                "kis_official_rows",
            ],
        ));
        if let Some(status) =
            string_field(value, &["candle_sufficiency_status", "sufficiency_status"])
        {
            panel.candle_sufficiency_status = status;
        }
        panel.outcome_links = panel.outcome_links.max(usize_field(
            value,
            &[
                "outcome_links",
                "generated_outcome_links",
                "added_kis_outcome_links",
            ],
        ));
        panel.counterfactuals =
            panel
                .counterfactuals
                .max(usize_field(value, &["counterfactuals"]).max(
                    usize_field(
                        value,
                        &[
                            "generated_no_trade_counterfactuals",
                            "added_kis_no_trade_counterfactuals",
                        ],
                    ) + usize_field(
                        value,
                        &[
                            "generated_risk_denied_counterfactuals",
                            "added_kis_risk_denied_counterfactuals",
                        ],
                    ),
                ));
        panel.complete_rows = panel.complete_rows.max(usize_field(
            value,
            &[
                "complete_rows",
                "complete_kis_rows",
                "added_complete_kis_rows",
            ],
        ));
        if panel.diversity_status.is_none() {
            panel.diversity_status = string_field(value, &["diversity_status"]);
        }
        if panel.core_bottleneck.is_none() {
            panel.core_bottleneck = string_field(
                value,
                &[
                    "core_bottleneck",
                    "current_primary_bottleneck",
                    "current_bottleneck",
                ],
            );
        }
        panel.next_kis_actions.extend(array_string_field(
            value,
            &["next_kis_actions", "operator_actions"],
        ));
        panel.reason_codes.extend(reason_code_field(value));
    }

    if panel.collection_plan_status == "Unknown" {
        panel.collection_plan_status =
            if panel.canonical_csv_count > 0 || panel.official_row_count > 0 {
                "CollectionReady".to_string()
            } else {
                "DiagnosticOnly".to_string()
            };
    }

    if !panel.auth_ready {
        panel.reason_codes.push(ReasonCode::MissingAuth);
        panel.next_kis_actions.push(
            "cargo run --quiet --bin soma_experiment -- kis-market-data-activate --config examples/soma_kis_market_data_activate_fixture_replay.toml".to_string(),
        );
    }
    if !panel.base_url_ready {
        panel.reason_codes.push(ReasonCode::MissingEndpointTemplate);
        panel.next_kis_actions.push(
            "Set KIS_BASE_URL in the local environment and rerun the bounded KIS market-data activation smoke.".to_string(),
        );
    }
    if panel
        .endpoint_policy_status
        .to_ascii_lowercase()
        .contains("unsafe")
        || panel
            .endpoint_policy_status
            .to_ascii_lowercase()
            .contains("block")
    {
        panel.reason_codes.push(ReasonCode::KISEndpointDenied);
        panel.next_kis_actions.push(
            "Keep KIS market-data-only: broker/order/account endpoints must remain denied."
                .to_string(),
        );
    }
    if panel.canonical_csv_count == 0 || panel.official_row_count == 0 {
        panel
            .reason_codes
            .push(ReasonCode::EvidenceStillInsufficient);
        panel.next_kis_actions.push(
            "cargo run --quiet --bin soma_experiment -- kis-market-data-activate --config examples/soma_kis_market_data_activate_local_import.toml".to_string(),
        );
    }
    if panel.candle_sufficiency_status.contains("MissingFuture")
        || panel.candle_sufficiency_status.contains("Insufficient")
    {
        panel.reason_codes.push(ReasonCode::InsufficientBars);
        panel.next_kis_actions.push(
            "cargo run --quiet --bin soma_experiment -- kis-candle-sufficiency --config examples/soma_kis_candle_sufficiency.toml".to_string(),
        );
    }
    if panel.outcome_links == 0 || panel.complete_rows == 0 {
        panel
            .reason_codes
            .push(ReasonCode::EvidenceStillInsufficient);
        panel.next_kis_actions.push(
            "cargo run --quiet --bin soma_experiment -- kis-outcome-link-close --config examples/soma_kis_outcome_link_close.toml".to_string(),
        );
    }
    if panel.next_kis_actions.is_empty() {
        panel.next_kis_actions.push(
            "Keep KIS in bounded market-data-only mode and refresh the local Control Tower bundle."
                .to_string(),
        );
    }

    panel.stabilize();
    panel
}

fn bool_field(value: &Value, keys: &[&str]) -> Option<bool> {
    keys.iter()
        .find_map(|key| value.get(key).and_then(|item| item.as_bool()))
}

fn usize_field(value: &Value, keys: &[&str]) -> usize {
    keys.iter()
        .find_map(|key| value.get(key).and_then(|item| item.as_u64()))
        .unwrap_or_default() as usize
}

fn string_field(value: &Value, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| {
        value
            .get(key)
            .and_then(|item| item.as_str())
            .map(ToString::to_string)
    })
}

fn array_string_field(value: &Value, keys: &[&str]) -> Vec<String> {
    keys.iter()
        .find_map(|key| {
            value.get(key).and_then(|item| {
                item.as_array().map(|items| {
                    items
                        .iter()
                        .filter_map(|entry| entry.as_str().map(ToString::to_string))
                        .collect::<Vec<_>>()
                })
            })
        })
        .unwrap_or_default()
}

fn reason_code_field(value: &Value) -> Vec<ReasonCode> {
    value
        .get("reason_codes")
        .cloned()
        .and_then(|item| serde_json::from_value::<Vec<ReasonCode>>(item).ok())
        .unwrap_or_default()
}
