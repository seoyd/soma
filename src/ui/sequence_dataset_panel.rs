use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_ordered_strings, stable_reason_codes};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SequenceDatasetPanel {
    pub export_status: String,
    #[serde(default)]
    pub dataset_csv_path: Option<String>,
    pub feature_schema_hash: String,
    pub label_manifest_hash: String,
    pub sequence_count: usize,
    pub row_count: usize,
    pub symbol_count: usize,
    #[serde(default)]
    pub label_distribution: BTreeMap<String, usize>,
    pub split_policy: String,
    pub no_lookahead_status: String,
    pub storage_status: String,
    #[serde(default)]
    pub drift_status: Option<String>,
    pub external_bridge_status: String,
    pub mamba_gate_status: String,
    #[serde(default)]
    pub next_actions: Vec<String>,
    pub no_train_button: bool,
    pub no_live_button: bool,
    pub no_order_account_controls: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl SequenceDatasetPanel {
    pub fn stabilize(&mut self) {
        self.next_actions = stable_ordered_strings(&self.next_actions);
        self.reason_codes = stable_reason_codes(&self.reason_codes);
    }

    pub fn to_text(&self) -> String {
        [
            format!("export_status={}", self.export_status),
            format!(
                "dataset_csv_path={}",
                self.dataset_csv_path
                    .clone()
                    .unwrap_or_else(|| "Unavailable".to_string())
            ),
            format!("feature_schema_hash={}", self.feature_schema_hash),
            format!("label_manifest_hash={}", self.label_manifest_hash),
            format!("sequence_count={}", self.sequence_count),
            format!("row_count={}", self.row_count),
            format!("symbol_count={}", self.symbol_count),
            format!("split_policy={}", self.split_policy),
            format!("no_lookahead_status={}", self.no_lookahead_status),
            format!("storage_status={}", self.storage_status),
            format!(
                "drift_status={}",
                self.drift_status
                    .clone()
                    .unwrap_or_else(|| "Unavailable".to_string())
            ),
            format!("external_bridge_status={}", self.external_bridge_status),
            format!("mamba_gate_status={}", self.mamba_gate_status),
            format!("next_actions={}", self.next_actions.join(" || ")),
            format!("no_train_button={}", self.no_train_button),
            format!("no_live_button={}", self.no_live_button),
            format!(
                "no_order_account_controls={}",
                self.no_order_account_controls
            ),
        ]
        .join("\n")
    }
}
