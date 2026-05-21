use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::core::{ReasonCode, stable_ordered_strings, stable_reason_codes};

use super::mamba_readiness_v2::{Mamba3ReadinessAuditV2, Mamba3ReadinessState};
use super::model_escalation_decision::{
    ModelEscalationCandidate, ModelEscalationDecisionStatus, ModelEscalationDecisionV2,
};

fn default_output_root() -> String {
    "target/sprint55/mamba_prototype_plan".to_string()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mamba3FinLitePrototypeBackend {
    ExternalPythonResearch,
    ExistingPredictionCsvOnly,
    Deferred,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Mamba3FinLitePrototypePlan {
    pub plan_id: String,
    pub allowed: bool,
    pub backend: Mamba3FinLitePrototypeBackend,
    pub required_dataset_artifacts: Vec<String>,
    pub required_prediction_schema: Vec<String>,
    pub expected_output_prediction_csv: String,
    pub model_card_requirements: Vec<String>,
    pub evaluation_gate: Vec<String>,
    pub risk_integration_requirements: Vec<String>,
    pub control_tower_visibility_requirements: Vec<String>,
    pub forbidden_actions: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl Mamba3FinLitePrototypePlan {
    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn to_text(&self) -> String {
        [
            format!("plan_id={}", self.plan_id),
            format!("allowed={}", self.allowed),
            format!("backend={:?}", self.backend),
            format!(
                "required_dataset_artifacts={}",
                self.required_dataset_artifacts.join("|")
            ),
            format!(
                "required_prediction_schema={}",
                self.required_prediction_schema.join("|")
            ),
            format!(
                "expected_output_prediction_csv={}",
                self.expected_output_prediction_csv
            ),
            format!(
                "model_card_requirements={}",
                self.model_card_requirements.join("|")
            ),
            format!("evaluation_gate={}", self.evaluation_gate.join("|")),
            format!(
                "risk_integration_requirements={}",
                self.risk_integration_requirements.join("|")
            ),
            format!(
                "control_tower_visibility_requirements={}",
                self.control_tower_visibility_requirements.join("|")
            ),
            format!("forbidden_actions={}", self.forbidden_actions.join("|")),
        ]
        .join("\n")
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<(), String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("mamba3fin_lite_prototype_plan.json"),
            self.to_json_string()?,
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("mamba3fin_lite_prototype_plan.txt"),
            self.to_text(),
        )
        .map_err(|err| err.to_string())?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mamba3FinLitePrototypePlanConfig {
    pub plan_id: String,
    #[serde(default)]
    pub mamba_readiness_report_paths: Vec<String>,
    #[serde(default)]
    pub model_escalation_decision_paths: Vec<String>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for Mamba3FinLitePrototypePlanConfig {
    fn default() -> Self {
        Self {
            plan_id: "sprint55_mamba_prototype_plan".to_string(),
            mamba_readiness_report_paths: Vec::new(),
            model_escalation_decision_paths: Vec::new(),
            output_root: default_output_root(),
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

impl Mamba3FinLitePrototypePlanConfig {
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
            .mamba_readiness_report_paths
            .iter()
            .chain(self.model_escalation_decision_paths.iter())
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
        if self.plan_id.trim().is_empty() {
            return Err("mamba prototype plan id must not be empty".to_string());
        }
        if !self.validate_local_paths().is_empty() {
            return Err("mamba-prototype-plan config paths must be local".to_string());
        }
        Ok(())
    }

    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.plan_id)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Mamba3FinLitePrototypePlanRunner;

impl Mamba3FinLitePrototypePlanRunner {
    pub fn run(
        &self,
        config: &Mamba3FinLitePrototypePlanConfig,
    ) -> Result<Mamba3FinLitePrototypePlan, String> {
        config.validate()?;
        let mut warnings = Vec::new();
        let mut reason_codes = config.reason_codes.clone();
        let mamba_report = load_values(
            &config.mamba_readiness_report_paths,
            &mut warnings,
            &mut reason_codes,
            "mamba readiness report",
        )
        .iter()
        .find_map(Mamba3ReadinessAuditV2::from_value)
        .ok_or_else(|| "mamba-prototype-plan requires a mamba readiness v2 report".to_string())?;
        let decision = load_values(
            &config.model_escalation_decision_paths,
            &mut warnings,
            &mut reason_codes,
            "model escalation decision",
        )
        .iter()
        .find_map(ModelEscalationDecisionV2::from_value)
        .ok_or_else(|| {
            "mamba-prototype-plan requires a model escalation decision report".to_string()
        })?;

        let allowed = matches!(
            mamba_report.readiness_state,
            Mamba3ReadinessState::ReadyForExternalPrototype
        ) && matches!(
            decision.selected_candidate,
            ModelEscalationCandidate::ExternalMamba3FinLite
        ) && matches!(
            decision.decision_status,
            ModelEscalationDecisionStatus::ExternalMambaPrototypeAllowed
        );
        let backend = if allowed {
            Mamba3FinLitePrototypeBackend::ExternalPythonResearch
        } else {
            Mamba3FinLitePrototypeBackend::Deferred
        };
        let plan = Mamba3FinLitePrototypePlan {
            plan_id: config.plan_id.clone(),
            allowed,
            backend,
            required_dataset_artifacts: stable_ordered_strings(&vec![
                "dataset.csv".to_string(),
                "feature_schema.json".to_string(),
                "label_manifest.json".to_string(),
                "sequence_export_manifest.json".to_string(),
            ]),
            required_prediction_schema: stable_ordered_strings(&vec![
                "row_id".to_string(),
                "symbol".to_string(),
                "timestamp_ms".to_string(),
                "p_win".to_string(),
                "p_stop".to_string(),
                "expected_return".to_string(),
                "expected_drawdown".to_string(),
                "confidence".to_string(),
                "no_trade_probability".to_string(),
                "horizon_bars".to_string(),
                "reason_codes".to_string(),
            ]),
            expected_output_prediction_csv:
                "prediction CSV must stay external, deterministic, schema-valid, and importable by the existing external prediction bridge".to_string(),
            model_card_requirements: stable_ordered_strings(&vec![
                "training data scope and source boundaries".to_string(),
                "feature schema and horizon definitions".to_string(),
                "calibration summary and cost-aware metrics".to_string(),
                "known failure modes and deferred live-readiness statement".to_string(),
            ]),
            evaluation_gate: stable_ordered_strings(&vec![
                "prediction CSV passes existing schema validation".to_string(),
                "paper-only evaluation keeps Risk Governor active".to_string(),
                "no live trading, no broker, and no account APIs".to_string(),
                "results remain research-only and non-profitability-claiming".to_string(),
            ]),
            risk_integration_requirements: stable_ordered_strings(&vec![
                "Risk Governor remains absolute veto".to_string(),
                "NoTrade remains the default action".to_string(),
                "external predictions cannot bypass Chair or Risk Governor".to_string(),
            ]),
            control_tower_visibility_requirements: stable_ordered_strings(&vec![
                "Control Tower shows external-prototype-only or deferred status".to_string(),
                "UI stays read-only with no live or train buttons".to_string(),
                "rendered output must not imply Rust runtime Mamba support".to_string(),
            ]),
            forbidden_actions: stable_ordered_strings(&vec![
                "no live trading".to_string(),
                "no real order execution".to_string(),
                "no broker or account APIs".to_string(),
                "no Rust inference".to_string(),
                "no Rust training".to_string(),
                "no automatic online learning".to_string(),
                "no source-boundary overclaiming".to_string(),
            ]),
            reason_codes: stable_reason_codes(&[
                reason_codes,
                vec![ReasonCode::MambaCandidateSpecBuilt, ReasonCode::DeterministicPath],
            ]
            .concat()),
        };
        if !warnings.is_empty() {
            let _ = warnings;
        }
        plan.write_to_dir(&config.output_dir())?;
        Ok(plan)
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
