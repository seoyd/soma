use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_hash};

pub fn stable_hash_string(input: &str) -> String {
    format!("{:016x}", stable_hash(input))
}

pub fn stable_ordered_strings(values: &[String]) -> Vec<String> {
    let mut ordered = values.to_vec();
    ordered.sort();
    ordered
}

pub fn stable_reason_codes(values: &[ReasonCode]) -> Vec<ReasonCode> {
    let mut ordered = values.to_vec();
    ordered.sort_by_key(|item| format!("{item:?}"));
    ordered.dedup();
    ordered
}

pub fn deterministic_float_format(value: f64) -> String {
    format!("{value:.6}")
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeterminismInputFingerprint {
    pub source_kind: String,
    pub config_fingerprint: String,
    #[serde(default)]
    pub data_fingerprint: Option<String>,
    #[serde(default)]
    pub feature_schema_hash: Option<u64>,
    #[serde(default)]
    pub prediction_schema_hash: Option<u64>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeterminismOutputFingerprint {
    pub report_fingerprint: String,
    pub decision_count: usize,
    pub reason_code_count: usize,
    pub artifact_count: usize,
    pub bytes_summary: usize,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeterminismCheck {
    pub input_fingerprint: DeterminismInputFingerprint,
    pub output_fingerprint: DeterminismOutputFingerprint,
    pub deterministic: bool,
    pub differences: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

impl DeterminismInputFingerprint {
    pub fn new(
        source_kind: impl Into<String>,
        config_material: &str,
        data_material: Option<&str>,
        feature_schema_hash: Option<u64>,
        prediction_schema_hash: Option<u64>,
    ) -> Self {
        Self {
            source_kind: source_kind.into(),
            config_fingerprint: stable_hash_string(config_material),
            data_fingerprint: data_material.map(stable_hash_string),
            feature_schema_hash,
            prediction_schema_hash,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

impl DeterminismOutputFingerprint {
    pub fn new(
        report_material: &str,
        decision_count: usize,
        reason_codes: &[ReasonCode],
        artifact_count: usize,
        bytes_summary: usize,
    ) -> Self {
        Self {
            report_fingerprint: stable_hash_string(report_material),
            decision_count,
            reason_code_count: stable_reason_codes(reason_codes).len(),
            artifact_count,
            bytes_summary,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

impl DeterminismCheck {
    pub fn compare(
        input_fingerprint: DeterminismInputFingerprint,
        left: &DeterminismOutputFingerprint,
        right: &DeterminismOutputFingerprint,
    ) -> Self {
        let mut differences = Vec::new();
        if left.report_fingerprint != right.report_fingerprint {
            differences.push("report_fingerprint".to_string());
        }
        if left.decision_count != right.decision_count {
            differences.push("decision_count".to_string());
        }
        if left.reason_code_count != right.reason_code_count {
            differences.push("reason_code_count".to_string());
        }
        if left.artifact_count != right.artifact_count {
            differences.push("artifact_count".to_string());
        }
        if left.bytes_summary != right.bytes_summary {
            differences.push("bytes_summary".to_string());
        }
        let deterministic = differences.is_empty();
        Self {
            input_fingerprint,
            output_fingerprint: left.clone(),
            deterministic,
            differences,
            reason_codes: vec![if deterministic {
                ReasonCode::DeterminismCheckPassed
            } else {
                ReasonCode::DeterminismCheckFailed
            }],
        }
    }

    pub fn to_text(&self) -> String {
        [
            format!("source_kind={}", self.input_fingerprint.source_kind),
            format!(
                "config_fingerprint={}",
                self.input_fingerprint.config_fingerprint
            ),
            format!(
                "data_fingerprint={}",
                self.input_fingerprint
                    .data_fingerprint
                    .as_deref()
                    .unwrap_or_default()
            ),
            format!(
                "report_fingerprint={}",
                self.output_fingerprint.report_fingerprint
            ),
            format!("deterministic={}", self.deterministic),
            format!("differences={}", self.differences.join("|")),
        ]
        .join("\n")
    }
}
