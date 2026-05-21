use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::core::{ReasonCode, stable_ordered_strings, stable_reason_codes};
use crate::experiment::{CoreCompletionAuditReport, CoreCompletionRecommendation};

use super::mamba_readiness_v2::{
    Mamba3ReadinessAuditV2, Mamba3ReadinessRecommendation, Mamba3ReadinessState,
};
use super::sequence_readiness::{SequenceDatasetReadinessReport, SequenceDatasetReadinessStatus};

fn default_output_root() -> String {
    "target/sprint55/model_escalation_decision".to_string()
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelEscalationDecisionV2Config {
    pub decision_id: String,
    #[serde(default)]
    pub core_completion_audit_report_paths: Vec<String>,
    #[serde(default)]
    pub sequence_readiness_report_paths: Vec<String>,
    #[serde(default)]
    pub mamba_readiness_report_paths: Vec<String>,
    #[serde(default)]
    pub supporting_artifact_paths: Vec<String>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default)]
    pub prefer_external_prototype: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for ModelEscalationDecisionV2Config {
    fn default() -> Self {
        Self {
            decision_id: "sprint55_model_escalation".to_string(),
            core_completion_audit_report_paths: Vec::new(),
            sequence_readiness_report_paths: Vec::new(),
            mamba_readiness_report_paths: Vec::new(),
            supporting_artifact_paths: Vec::new(),
            output_root: default_output_root(),
            prefer_external_prototype: false,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

impl ModelEscalationDecisionV2Config {
    pub fn from_toml_str(input: &str) -> Result<Self, String> {
        toml::from_str(input).map_err(|err| err.to_string())
    }

    pub fn from_toml_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        Self::from_toml_str(&text)
    }

    pub fn to_toml_string(&self) -> Result<String, String> {
        toml::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn validate_local_paths(&self) -> Vec<ReasonCode> {
        if self
            .all_input_paths()
            .iter()
            .chain([self.output_root.clone()].iter())
            .any(|path| path.contains("://"))
        {
            vec![
                ReasonCode::LocalPathRejected,
                ReasonCode::RemotePathRejected,
            ]
        } else {
            Vec::new()
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.decision_id.trim().is_empty() {
            return Err("model escalation decision id must not be empty".to_string());
        }
        if !self.validate_local_paths().is_empty() {
            return Err("model-escalation-decision config paths must be local".to_string());
        }
        Ok(())
    }

    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.decision_id)
    }

    pub fn all_input_paths(&self) -> Vec<String> {
        stable_ordered_strings(
            &self
                .core_completion_audit_report_paths
                .iter()
                .chain(self.sequence_readiness_report_paths.iter())
                .chain(self.mamba_readiness_report_paths.iter())
                .chain(self.supporting_artifact_paths.iter())
                .cloned()
                .collect::<Vec<_>>(),
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ModelEscalationCandidate {
    BaselineSignalModel,
    CommitteeTrinity,
    ExternalTabularModel,
    SequenceDatasetBuild,
    ExternalMamba3FinLite,
    RustNativeMamba3Runtime,
    NoEscalation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelEscalationDecisionStatus {
    KeepBaselineAndEvidence,
    ImproveBaselineSignal,
    BuildSequenceDatasetFirst,
    ExternalMambaPrototypeAllowed,
    MambaDeferred,
    RuntimeMambaForbidden,
    NeedMoreEvidence,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ModelEscalationDecisionV2 {
    pub decision_id: String,
    pub selected_candidate: ModelEscalationCandidate,
    pub rejected_candidates: Vec<ModelEscalationCandidate>,
    pub rationale: String,
    pub prerequisites: Vec<String>,
    pub next_actions: Vec<String>,
    pub decision_status: ModelEscalationDecisionStatus,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl ModelEscalationDecisionV2 {
    pub fn from_value(value: &Value) -> Option<Self> {
        serde_json::from_value(value.clone()).ok().or_else(|| {
            value
                .get("model_escalation_decision_v2")
                .and_then(|item| serde_json::from_value(item.clone()).ok())
        })
    }

    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn to_text(&self) -> String {
        [
            format!("decision_id={}", self.decision_id),
            format!("selected_candidate={:?}", self.selected_candidate),
            format!("decision_status={:?}", self.decision_status),
            format!(
                "rejected_candidates={}",
                self.rejected_candidates
                    .iter()
                    .map(|candidate| format!("{candidate:?}"))
                    .collect::<Vec<_>>()
                    .join("|")
            ),
            format!("rationale={}", self.rationale),
            format!("prerequisites={}", self.prerequisites.join(" | ")),
            format!("next_actions={}", self.next_actions.join(" | ")),
        ]
        .join("\n")
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<(), String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("model_escalation_decision_v2.json"),
            self.to_json_string()?,
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("model_escalation_decision_v2.txt"),
            self.to_text(),
        )
        .map_err(|err| err.to_string())?;
        Ok(())
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ModelEscalationDecisionRunner;

impl ModelEscalationDecisionRunner {
    pub fn run(
        &self,
        config: &ModelEscalationDecisionV2Config,
    ) -> Result<ModelEscalationDecisionV2, String> {
        config.validate()?;
        let mut warnings = Vec::new();
        let mut reason_codes = config.reason_codes.clone();

        let core_report = load_values(
            &config.core_completion_audit_report_paths,
            &mut warnings,
            &mut reason_codes,
            "core completion report",
        )
        .iter()
        .find_map(CoreCompletionAuditReport::from_value)
        .ok_or_else(|| {
            "model-escalation-decision requires a core completion audit report".to_string()
        })?;
        let sequence_report = load_values(
            &config.sequence_readiness_report_paths,
            &mut warnings,
            &mut reason_codes,
            "sequence readiness report",
        )
        .iter()
        .find_map(SequenceDatasetReadinessReport::from_value)
        .ok_or_else(|| {
            "model-escalation-decision requires a sequence readiness report".to_string()
        })?;
        let mamba_report = load_values(
            &config.mamba_readiness_report_paths,
            &mut warnings,
            &mut reason_codes,
            "mamba readiness report",
        )
        .iter()
        .find_map(Mamba3ReadinessAuditV2::from_value)
        .ok_or_else(|| {
            "model-escalation-decision requires a mamba readiness v2 report".to_string()
        })?;
        let support = SupportSnapshot::from_values(&load_values(
            &config.supporting_artifact_paths,
            &mut warnings,
            &mut reason_codes,
            "model escalation support input",
        ));

        let mut rejected_candidates = vec![ModelEscalationCandidate::RustNativeMamba3Runtime];
        let mut prerequisites = Vec::new();
        let (selected_candidate, decision_status, rationale, mut next_actions) = if mamba_report
            .mamba3_runtime_present
            || mamba_report.rust_native_training_present
        {
            (
                ModelEscalationCandidate::NoEscalation,
                ModelEscalationDecisionStatus::RuntimeMambaForbidden,
                "Rust-native Mamba runtime or training is forbidden in Sprint 55.".to_string(),
                vec!["remove runtime/training scope and keep research-only gating".to_string()],
            )
        } else if support.kis_outcome_depth_bottleneck.unwrap_or(false)
            || matches!(
                core_report.final_recommendation,
                CoreCompletionRecommendation::CoreNeedsKISEvidenceDepth
                    | CoreCompletionRecommendation::CoreNeedsOutcomeLinkDepth
            )
            || matches!(
                mamba_report.final_recommendation,
                Mamba3ReadinessRecommendation::ImproveEvidenceDepthFirst
            )
        {
            prerequisites.push("stronger KIS evidence depth".to_string());
            prerequisites.push("deeper official outcome links".to_string());
            (
                ModelEscalationCandidate::NoEscalation,
                ModelEscalationDecisionStatus::NeedMoreEvidence,
                "Current KIS outcome-depth bottleneck keeps Mamba deferred and the recommendation conservative.".to_string(),
                vec![
                    "expand official KIS rows with outcome linkage".to_string(),
                    "keep baseline, committee, and risk governor as the active stack".to_string(),
                ],
            )
        } else if !matches!(
            sequence_report.readiness_status,
            SequenceDatasetReadinessStatus::ReadyForSequenceDatasetExport
        ) {
            prerequisites.push("sequence dataset export readiness".to_string());
            (
                ModelEscalationCandidate::SequenceDatasetBuild,
                ModelEscalationDecisionStatus::BuildSequenceDatasetFirst,
                "Sequence dataset readiness is still the blocking step before any sequence-model escalation.".to_string(),
                vec![
                    format!(
                        "resolve sequence readiness status {:?}",
                        sequence_report.readiness_status
                    ),
                    "keep Mamba deferred until the sequence gate passes".to_string(),
                ],
            )
        } else if support.signal_model_weak.unwrap_or(false)
            || matches!(
                mamba_report.final_recommendation,
                Mamba3ReadinessRecommendation::ImproveSignalModelFirst
            )
        {
            prerequisites.push("stronger baseline calibration and risk behavior".to_string());
            rejected_candidates.push(ModelEscalationCandidate::ExternalMamba3FinLite);
            (
                ModelEscalationCandidate::BaselineSignalModel,
                ModelEscalationDecisionStatus::ImproveBaselineSignal,
                "Baseline signal quality still needs work before external sequence-model escalation.".to_string(),
                vec![
                    "improve baseline signal calibration first".to_string(),
                    "re-run core and Mamba readiness gates after signal improvements".to_string(),
                ],
            )
        } else if matches!(
            mamba_report.readiness_state,
            Mamba3ReadinessState::ReadyForExternalPrototype
        ) && (config.prefer_external_prototype
            || !support.keep_committee_only.unwrap_or(false))
        {
            prerequisites.push("prediction CSV schema validation".to_string());
            prerequisites.push("research-only external backend".to_string());
            (
                ModelEscalationCandidate::ExternalMamba3FinLite,
                ModelEscalationDecisionStatus::ExternalMambaPrototypeAllowed,
                "Only an external research-only Mamba3Fin-lite prototype is allowed; Rust runtime remains rejected.".to_string(),
                vec![
                    "build the external prototype behind prediction CSV import only".to_string(),
                    "keep Risk Governor and Control Tower in the loop".to_string(),
                ],
            )
        } else {
            rejected_candidates.push(ModelEscalationCandidate::ExternalMamba3FinLite);
            (
                ModelEscalationCandidate::CommitteeTrinity,
                ModelEscalationDecisionStatus::MambaDeferred,
                "Keep the baseline and trinity committee active while Mamba remains deferred."
                    .to_string(),
                vec![
                    "keep baseline plus committee evidence accumulation active".to_string(),
                    "do not start Rust-native Mamba runtime work".to_string(),
                ],
            )
        };

        if matches!(
            selected_candidate,
            ModelEscalationCandidate::CommitteeTrinity | ModelEscalationCandidate::NoEscalation
        ) {
            rejected_candidates.push(ModelEscalationCandidate::ExternalTabularModel);
        }
        if !warnings.is_empty() {
            next_actions.extend(warnings.iter().cloned());
        }

        let decision = ModelEscalationDecisionV2 {
            decision_id: config.decision_id.clone(),
            selected_candidate,
            rejected_candidates: stable_candidates(rejected_candidates),
            rationale,
            prerequisites: stable_ordered_strings(&prerequisites),
            next_actions: stable_ordered_strings(&next_actions),
            decision_status,
            reason_codes: stable_reason_codes(
                &[
                    reason_codes,
                    vec![
                        ReasonCode::ModelEscalationEvaluated,
                        ReasonCode::DeterministicPath,
                    ],
                ]
                .concat(),
            ),
        };
        decision.write_to_dir(&config.output_dir())?;
        Ok(decision)
    }
}

fn stable_candidates(
    mut candidates: Vec<ModelEscalationCandidate>,
) -> Vec<ModelEscalationCandidate> {
    candidates.sort();
    candidates.dedup();
    candidates
}

fn load_values(
    paths: &[String],
    warnings: &mut Vec<String>,
    reason_codes: &mut Vec<ReasonCode>,
    label: &str,
) -> Vec<Value> {
    let mut values = Vec::new();
    for path in stable_ordered_strings(paths) {
        match fs::read_to_string(&path) {
            Ok(text) => match serde_json::from_str::<Value>(&text) {
                Ok(value) => values.push(value),
                Err(err) => {
                    warnings.push(format!("failed to parse {label}: {err}"));
                    reason_codes.push(ReasonCode::DataLoadFailed);
                }
            },
            Err(_) => {
                warnings.push(format!("missing {label}: {path}"));
                reason_codes.push(ReasonCode::MissingFile);
            }
        }
    }
    values
}

#[derive(Clone, Debug, Default)]
struct SupportSnapshot {
    signal_model_weak: Option<bool>,
    kis_outcome_depth_bottleneck: Option<bool>,
    keep_committee_only: Option<bool>,
}

impl SupportSnapshot {
    fn from_values(values: &[Value]) -> Self {
        let mut snapshot = SupportSnapshot::default();
        for value in values {
            merge_bool(
                &mut snapshot.signal_model_weak,
                value,
                &["signal_model_weak"],
            );
            merge_bool(
                &mut snapshot.kis_outcome_depth_bottleneck,
                value,
                &["kis_outcome_depth_bottleneck", "outcome_depth_bottleneck"],
            );
            merge_bool(
                &mut snapshot.keep_committee_only,
                value,
                &["keep_committee_only"],
            );
        }
        snapshot
    }
}

fn merge_bool(slot: &mut Option<bool>, value: &Value, keys: &[&str]) {
    if let Some(flag) = bool_field(value, keys) {
        *slot = Some(slot.unwrap_or(false) || flag);
    }
}

fn bool_field(value: &Value, keys: &[&str]) -> Option<bool> {
    keys.iter().find_map(|key| {
        let mut matches = Vec::new();
        collect_matches(value, key, &mut matches);
        matches.into_iter().find_map(|item| match item {
            Value::Bool(flag) => Some(*flag),
            Value::Number(number) => number.as_u64().map(|value| value > 0),
            Value::String(text) => match text.to_ascii_lowercase().as_str() {
                "true" | "ready" | "allowed" => Some(true),
                "false" | "blocked" | "missing" => Some(false),
                _ => None,
            },
            _ => None,
        })
    })
}

fn collect_matches<'a>(value: &'a Value, key: &str, output: &mut Vec<&'a Value>) {
    match value {
        Value::Object(map) => {
            if let Some(item) = map.get(key) {
                output.push(item);
            }
            for child in map.values() {
                collect_matches(child, key, output);
            }
        }
        Value::Array(items) => {
            for child in items {
                collect_matches(child, key, output);
            }
        }
        _ => {}
    }
}
