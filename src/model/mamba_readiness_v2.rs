use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::core::{ReasonCode, stable_ordered_strings, stable_reason_codes};
use crate::experiment::{
    CoreCompletionAuditReport, CoreCompletionRecommendation, CoreCompletionStatus, CoreSubsystem,
};

use super::sequence_readiness::{SequenceDatasetReadinessReport, SequenceDatasetReadinessStatus};

fn default_output_root() -> String {
    "target/sprint55/mamba_readiness_v2".to_string()
}

fn default_true() -> bool {
    true
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MambaReadinessV2Config {
    pub audit_id: String,
    #[serde(default)]
    pub sequence_readiness_report_paths: Vec<String>,
    #[serde(default)]
    pub core_completion_audit_report_paths: Vec<String>,
    #[serde(default)]
    pub supporting_artifact_paths: Vec<String>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default)]
    pub allow_external_prototype_only: bool,
    #[serde(default = "default_true")]
    pub require_control_tower_visibility: bool,
    #[serde(default = "default_true")]
    pub require_risk_governor_integration: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for MambaReadinessV2Config {
    fn default() -> Self {
        Self {
            audit_id: "sprint55_mamba_readiness_v2".to_string(),
            sequence_readiness_report_paths: Vec::new(),
            core_completion_audit_report_paths: Vec::new(),
            supporting_artifact_paths: Vec::new(),
            output_root: default_output_root(),
            allow_external_prototype_only: false,
            require_control_tower_visibility: true,
            require_risk_governor_integration: true,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

impl MambaReadinessV2Config {
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
        if self.audit_id.trim().is_empty() {
            return Err("mamba readiness v2 audit id must not be empty".to_string());
        }
        if !self.validate_local_paths().is_empty() {
            return Err("mamba-readiness-v2 config paths must be local".to_string());
        }
        Ok(())
    }

    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.audit_id)
    }

    pub fn all_input_paths(&self) -> Vec<String> {
        stable_ordered_strings(
            &self
                .sequence_readiness_report_paths
                .iter()
                .chain(self.core_completion_audit_report_paths.iter())
                .chain(self.supporting_artifact_paths.iter())
                .cloned()
                .collect::<Vec<_>>(),
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mamba3ReadinessDimension {
    EvidenceDepth,
    SequenceDataset,
    FeatureSchema,
    LabelQuality,
    OutcomeDiversity,
    CounterfactualDepth,
    CalibrationBaseline,
    ExternalPrototypeBridge,
    StorageBudget,
    InferenceBudget,
    RiskGovernorIntegration,
    ControlTowerVisibility,
    RustRuntimeSafety,
    TrainingPathSafety,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Mamba3ReadinessDimensionResult {
    pub dimension: Mamba3ReadinessDimension,
    pub ready: bool,
    pub summary: String,
    #[serde(default)]
    pub blockers: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mamba3ReadinessState {
    NotReady,
    ReadyForSequenceDatasetBuild,
    ReadyForExternalPrototype,
    ReadyForExternalMamba3FinLiteOnly,
    RustRuntimeDeferred,
    BlockedByEvidenceDepth,
    BlockedBySequenceDataset,
    BlockedByNoLookahead,
    BlockedByStorage,
    BlockedByRisk,
    Forbidden,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mamba3ReadinessRecommendation {
    HoldMamba3Deferred,
    BuildSequenceDatasetFirst,
    BuildExternalMamba3FinLitePrototype,
    ImproveEvidenceDepthFirst,
    ImproveSignalModelFirst,
    KeepBaselineAndCommittee,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Mamba3ReadinessAuditV2 {
    pub audit_id: String,
    pub dimension_results: Vec<Mamba3ReadinessDimensionResult>,
    pub sequence_readiness_report: SequenceDatasetReadinessReport,
    pub mamba3_runtime_present: bool,
    pub rust_native_training_present: bool,
    pub external_bridge_present: bool,
    pub readiness_state: Mamba3ReadinessState,
    pub final_recommendation: Mamba3ReadinessRecommendation,
    #[serde(default)]
    pub blockers: Vec<String>,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl Mamba3ReadinessAuditV2 {
    pub fn from_value(value: &Value) -> Option<Self> {
        serde_json::from_value(value.clone()).ok().or_else(|| {
            value
                .get("mamba3_readiness_audit_v2")
                .and_then(|item| serde_json::from_value(item.clone()).ok())
        })
    }

    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn to_text(&self) -> String {
        [
            format!("audit_id={}", self.audit_id),
            format!("readiness_state={:?}", self.readiness_state),
            format!("final_recommendation={:?}", self.final_recommendation),
            format!("mamba3_runtime_present={}", self.mamba3_runtime_present),
            format!(
                "rust_native_training_present={}",
                self.rust_native_training_present
            ),
            format!("external_bridge_present={}", self.external_bridge_present),
            format!(
                "dimension_results={}",
                self.dimension_results
                    .iter()
                    .map(|item| format!("{:?}:{}", item.dimension, item.ready))
                    .collect::<Vec<_>>()
                    .join("|")
            ),
            self.sequence_readiness_report.to_text(),
            format!("blockers={}", self.blockers.join(" | ")),
            format!("warnings={}", self.warnings.join(" | ")),
        ]
        .join("\n")
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<(), String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("mamba3_readiness_audit_v2.json"),
            self.to_json_string()?,
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("mamba3_readiness_audit_v2.txt"),
            self.to_text(),
        )
        .map_err(|err| err.to_string())?;
        Ok(())
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MambaReadinessV2Runner;

impl MambaReadinessV2Runner {
    pub fn run(&self, config: &MambaReadinessV2Config) -> Result<Mamba3ReadinessAuditV2, String> {
        config.validate()?;
        let mut warnings = Vec::new();
        let mut reason_codes = config.reason_codes.clone();
        let sequence_values = load_values(
            &config.sequence_readiness_report_paths,
            &mut warnings,
            &mut reason_codes,
            "sequence readiness input",
        );
        let core_values = load_values(
            &config.core_completion_audit_report_paths,
            &mut warnings,
            &mut reason_codes,
            "core completion input",
        );
        let support_values = load_values(
            &config.supporting_artifact_paths,
            &mut warnings,
            &mut reason_codes,
            "mamba readiness support input",
        );

        let sequence_report = sequence_values
            .iter()
            .find_map(SequenceDatasetReadinessReport::from_value)
            .ok_or_else(|| "mamba-readiness-v2 requires a sequence readiness report".to_string())?;
        let core_report = core_values
            .iter()
            .find_map(CoreCompletionAuditReport::from_value);
        let support = SupportSnapshot::from_values(&support_values);

        let evidence_depth_sufficient = support.evidence_depth_sufficient.unwrap_or_else(|| {
            core_report.as_ref().is_some_and(|report| {
                matches!(
                    report.core_completion_status,
                    CoreCompletionStatus::CoreResearchOperatingSystemComplete
                        | CoreCompletionStatus::CorePaperOperatingSystemComplete
                ) && !matches!(
                    report.final_recommendation,
                    CoreCompletionRecommendation::CoreNeedsKISEvidenceDepth
                        | CoreCompletionRecommendation::CoreNeedsOutcomeLinkDepth
                        | CoreCompletionRecommendation::HoldModelComplexity
                )
            })
        });
        let signal_model_weak = support.signal_model_weak.unwrap_or(false);
        let counterfactual_depth_sufficient =
            support.counterfactual_depth_sufficient.unwrap_or_else(|| {
                core_report.as_ref().is_some_and(|report| {
                    !report
                        .blocked_subsystems
                        .contains(&CoreSubsystem::CounterfactualDepth)
                })
            });
        let calibration_baseline_ready = support
            .calibration_baseline_ready
            .unwrap_or(!signal_model_weak);
        let external_bridge_present = support.external_bridge_present.unwrap_or(true);
        let inference_budget_ok = support.inference_budget_ok.unwrap_or(true);
        let risk_governor_integrated = support.risk_governor_integrated.unwrap_or(true);
        let control_tower_visible = support.control_tower_visible.unwrap_or_else(|| {
            core_report.as_ref().is_some_and(|report| {
                report.maturity_matrix.rows.iter().any(|row| {
                    row.subsystem == CoreSubsystem::ControlTowerV1
                        && matches!(
                            row.maturity,
                            crate::experiment::SubsystemMaturity::PaperReady
                                | crate::experiment::SubsystemMaturity::ResearchReady
                        )
                })
            })
        });
        let mamba3_runtime_present = support.mamba3_runtime_present.unwrap_or(false);
        let rust_native_training_present = support.rust_native_training_present.unwrap_or(false);
        let storage_ok = !matches!(
            sequence_report.readiness_status,
            SequenceDatasetReadinessStatus::NeedStorageBudget
        );
        let no_lookahead_ok = sequence_report.no_lookahead_safe
            && !matches!(
                sequence_report.readiness_status,
                SequenceDatasetReadinessStatus::NeedNoLookaheadProof
            );
        let sequence_ready = matches!(
            sequence_report.readiness_status,
            SequenceDatasetReadinessStatus::ReadyForSequenceDatasetExport
        );
        let label_quality_ready = !matches!(
            sequence_report.readiness_status,
            SequenceDatasetReadinessStatus::NeedMoreOutcomeLabels
        );
        let outcome_diversity_ready = sequence_report
            .outcome_label_distribution
            .values()
            .filter(|count| **count > 0)
            .count()
            >= 3;

        let dimension_results = vec![
            dimension(
                Mamba3ReadinessDimension::EvidenceDepth,
                evidence_depth_sufficient,
                if evidence_depth_sufficient {
                    "official KIS evidence depth is sufficient for a research-only gate review"
                } else {
                    "official KIS evidence depth remains too weak for conservative model escalation"
                },
                if evidence_depth_sufficient {
                    Vec::new()
                } else {
                    vec!["NeedMoreKISEvidence".to_string()]
                },
            ),
            dimension(
                Mamba3ReadinessDimension::SequenceDataset,
                sequence_ready,
                &format!("sequence status is {:?}", sequence_report.readiness_status),
                if sequence_ready {
                    Vec::new()
                } else {
                    vec![format!("{:?}", sequence_report.readiness_status)]
                },
            ),
            dimension(
                Mamba3ReadinessDimension::FeatureSchema,
                sequence_report.feature_schema_locked,
                if sequence_report.feature_schema_locked {
                    "feature schema is locked for deterministic exports"
                } else {
                    "feature schema lock is still missing"
                },
                if sequence_report.feature_schema_locked {
                    Vec::new()
                } else {
                    vec!["NeedFeatureSchemaLock".to_string()]
                },
            ),
            dimension(
                Mamba3ReadinessDimension::LabelQuality,
                label_quality_ready,
                if label_quality_ready {
                    "label diversity is adequate for a bounded sequence export"
                } else {
                    "label diversity is still too thin for sequence escalation"
                },
                if label_quality_ready {
                    Vec::new()
                } else {
                    vec!["NeedMoreOutcomeLabels".to_string()]
                },
            ),
            dimension(
                Mamba3ReadinessDimension::OutcomeDiversity,
                outcome_diversity_ready,
                if outcome_diversity_ready {
                    "multiple outcome classes are represented"
                } else {
                    "outcome diversity remains thin"
                },
                if outcome_diversity_ready {
                    Vec::new()
                } else {
                    vec!["NeedMoreOutcomeLabels".to_string()]
                },
            ),
            dimension(
                Mamba3ReadinessDimension::CounterfactualDepth,
                counterfactual_depth_sufficient,
                if counterfactual_depth_sufficient {
                    "counterfactual depth is adequate for a research-only gate review"
                } else {
                    "counterfactual depth remains too weak for stronger claims"
                },
                if counterfactual_depth_sufficient {
                    Vec::new()
                } else {
                    vec!["NeedMoreOutcomeLinkDepth".to_string()]
                },
            ),
            dimension(
                Mamba3ReadinessDimension::CalibrationBaseline,
                calibration_baseline_ready,
                if calibration_baseline_ready {
                    "baseline calibration is acceptable for conservative comparisons"
                } else {
                    "baseline calibration still needs work before model escalation"
                },
                if calibration_baseline_ready {
                    Vec::new()
                } else {
                    vec!["ImproveSignalModelFirst".to_string()]
                },
            ),
            dimension(
                Mamba3ReadinessDimension::ExternalPrototypeBridge,
                external_bridge_present,
                if external_bridge_present {
                    "external prediction CSV bridge is available"
                } else {
                    "external prediction CSV bridge is missing"
                },
                if external_bridge_present {
                    Vec::new()
                } else {
                    vec!["ExternalPredictionBridgeMissing".to_string()]
                },
            ),
            dimension(
                Mamba3ReadinessDimension::StorageBudget,
                storage_ok,
                if storage_ok {
                    "sequence storage remains within configured budget"
                } else {
                    "sequence storage exceeds the configured budget"
                },
                if storage_ok {
                    Vec::new()
                } else {
                    vec!["BlockedByStorage".to_string()]
                },
            ),
            dimension(
                Mamba3ReadinessDimension::InferenceBudget,
                inference_budget_ok,
                if inference_budget_ok {
                    "bounded inference budget remains compatible with external research-only scoring"
                } else {
                    "inference budget is still too weak for even an external prototype"
                },
                if inference_budget_ok {
                    Vec::new()
                } else {
                    vec!["InferenceBudgetExceeded".to_string()]
                },
            ),
            dimension(
                Mamba3ReadinessDimension::RiskGovernorIntegration,
                risk_governor_integrated,
                if risk_governor_integrated {
                    "Risk Governor remains on the path and keeps absolute veto"
                } else {
                    "Risk Governor integration is missing"
                },
                if risk_governor_integrated {
                    Vec::new()
                } else {
                    vec!["BlockedByRisk".to_string()]
                },
            ),
            dimension(
                Mamba3ReadinessDimension::ControlTowerVisibility,
                control_tower_visible,
                if control_tower_visible {
                    "Control Tower can display readiness without implying runtime Mamba support"
                } else {
                    "Control Tower visibility for the gate is missing"
                },
                if control_tower_visible {
                    Vec::new()
                } else {
                    vec!["ControlTowerVisibilityMissing".to_string()]
                },
            ),
            dimension(
                Mamba3ReadinessDimension::RustRuntimeSafety,
                !mamba3_runtime_present,
                if mamba3_runtime_present {
                    "Rust-native Mamba runtime presence would violate Sprint 55"
                } else {
                    "Rust-native Mamba runtime remains deferred and absent"
                },
                if mamba3_runtime_present {
                    vec!["RuntimeMambaForbidden".to_string()]
                } else {
                    Vec::new()
                },
            ),
            dimension(
                Mamba3ReadinessDimension::TrainingPathSafety,
                !rust_native_training_present,
                if rust_native_training_present {
                    "Rust-native neural training presence would violate Sprint 55"
                } else {
                    "Rust-native neural training remains absent and deferred"
                },
                if rust_native_training_present {
                    vec!["RustTrainingForbidden".to_string()]
                } else {
                    Vec::new()
                },
            ),
        ];

        let mut blockers = dimension_results
            .iter()
            .filter(|item| !item.ready)
            .flat_map(|item| item.blockers.clone())
            .collect::<Vec<_>>();

        let readiness_state = if mamba3_runtime_present || rust_native_training_present {
            Mamba3ReadinessState::Forbidden
        } else if !evidence_depth_sufficient || !counterfactual_depth_sufficient {
            Mamba3ReadinessState::BlockedByEvidenceDepth
        } else if !no_lookahead_ok {
            Mamba3ReadinessState::BlockedByNoLookahead
        } else if !storage_ok {
            Mamba3ReadinessState::BlockedByStorage
        } else if config.require_risk_governor_integration && !risk_governor_integrated {
            Mamba3ReadinessState::BlockedByRisk
        } else if !sequence_ready {
            Mamba3ReadinessState::BlockedBySequenceDataset
        } else if external_bridge_present && config.allow_external_prototype_only {
            Mamba3ReadinessState::ReadyForExternalPrototype
        } else if sequence_ready {
            Mamba3ReadinessState::RustRuntimeDeferred
        } else {
            Mamba3ReadinessState::NotReady
        };

        let final_recommendation = if matches!(
            readiness_state,
            Mamba3ReadinessState::ReadyForExternalPrototype
        ) {
            Mamba3ReadinessRecommendation::BuildExternalMamba3FinLitePrototype
        } else if !evidence_depth_sufficient || !counterfactual_depth_sufficient {
            Mamba3ReadinessRecommendation::ImproveEvidenceDepthFirst
        } else if signal_model_weak || !calibration_baseline_ready {
            Mamba3ReadinessRecommendation::ImproveSignalModelFirst
        } else if matches!(
            readiness_state,
            Mamba3ReadinessState::BlockedBySequenceDataset
                | Mamba3ReadinessState::BlockedByNoLookahead
                | Mamba3ReadinessState::BlockedByStorage
        ) {
            Mamba3ReadinessRecommendation::BuildSequenceDatasetFirst
        } else {
            Mamba3ReadinessRecommendation::HoldMamba3Deferred
        };

        warnings.push("Mamba3 readiness does not mean Mamba3 is implemented".to_string());
        warnings.push("Rust-native Mamba runtime remains deferred in Sprint 55".to_string());
        if matches!(
            readiness_state,
            Mamba3ReadinessState::ReadyForExternalPrototype
        ) {
            warnings.push("only an external research-only prototype is allowed; no Rust runtime or live path is permitted".to_string());
        }
        if config.require_control_tower_visibility && !control_tower_visible {
            blockers.push("ControlTowerVisibilityMissing".to_string());
        }

        let report = Mamba3ReadinessAuditV2 {
            audit_id: config.audit_id.clone(),
            dimension_results,
            sequence_readiness_report: sequence_report,
            mamba3_runtime_present,
            rust_native_training_present,
            external_bridge_present,
            readiness_state,
            final_recommendation,
            blockers: stable_ordered_strings(&blockers),
            warnings: stable_ordered_strings(&warnings),
            reason_codes: stable_reason_codes(
                &[
                    reason_codes,
                    vec![
                        ReasonCode::MambaReadinessBuilt,
                        ReasonCode::DeterministicPath,
                    ],
                ]
                .concat(),
            ),
        };
        report.write_to_dir(&config.output_dir())?;
        Ok(report)
    }
}

fn dimension(
    dimension: Mamba3ReadinessDimension,
    ready: bool,
    summary: &str,
    blockers: Vec<String>,
) -> Mamba3ReadinessDimensionResult {
    Mamba3ReadinessDimensionResult {
        dimension,
        ready,
        summary: summary.to_string(),
        blockers: stable_ordered_strings(&blockers),
        reason_codes: stable_reason_codes(&[ReasonCode::DeterministicPath]),
    }
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
    evidence_depth_sufficient: Option<bool>,
    counterfactual_depth_sufficient: Option<bool>,
    calibration_baseline_ready: Option<bool>,
    external_bridge_present: Option<bool>,
    inference_budget_ok: Option<bool>,
    risk_governor_integrated: Option<bool>,
    control_tower_visible: Option<bool>,
    mamba3_runtime_present: Option<bool>,
    rust_native_training_present: Option<bool>,
    signal_model_weak: Option<bool>,
}

impl SupportSnapshot {
    fn from_values(values: &[Value]) -> Self {
        let mut snapshot = SupportSnapshot::default();
        for value in values {
            merge_bool(
                &mut snapshot.evidence_depth_sufficient,
                value,
                &["evidence_depth_sufficient"],
            );
            merge_bool(
                &mut snapshot.counterfactual_depth_sufficient,
                value,
                &["counterfactual_depth_sufficient"],
            );
            merge_bool(
                &mut snapshot.calibration_baseline_ready,
                value,
                &["calibration_baseline_ready"],
            );
            merge_bool(
                &mut snapshot.external_bridge_present,
                value,
                &["external_bridge_present", "prediction_csv_bridge_present"],
            );
            merge_bool(
                &mut snapshot.inference_budget_ok,
                value,
                &["inference_budget_ok"],
            );
            merge_bool(
                &mut snapshot.risk_governor_integrated,
                value,
                &["risk_governor_integrated"],
            );
            merge_bool(
                &mut snapshot.control_tower_visible,
                value,
                &["control_tower_visible"],
            );
            merge_bool(
                &mut snapshot.mamba3_runtime_present,
                value,
                &["mamba3_runtime_present", "mamba_runtime_present"],
            );
            merge_bool(
                &mut snapshot.rust_native_training_present,
                value,
                &["rust_native_training_present", "rust_training_present"],
            );
            merge_bool(
                &mut snapshot.signal_model_weak,
                value,
                &["signal_model_weak"],
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
                "true" | "ready" | "present" | "visible" => Some(true),
                "false" | "missing" | "blocked" => Some(false),
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
