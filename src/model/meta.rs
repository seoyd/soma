use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelKind {
    BaselineRule,
    ExternalPredictionFile,
    LightGbmExternal,
    XgBoostExternal,
    TinyNetExternal,
    Mamba3FinExternal,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ModelArtifactMeta {
    pub model_id: String,
    pub model_kind: ModelKind,
    pub created_at_ms: Option<u64>,
    pub feature_schema_version: u32,
    pub feature_schema_hash: u64,
    pub training_window: Option<String>,
    pub validation_window: Option<String>,
    pub test_window: Option<String>,
    pub target_label_config: String,
    pub cost_model_summary: String,
    pub notes: Option<String>,
    pub reason_codes: Vec<ReasonCode>,
}
