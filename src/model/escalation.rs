use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;
use crate::experiment::{AiSignalStatus, OfficialConsistencyStatus};

use super::mamba_readiness::{Mamba3FinCandidateReadiness, Mamba3FinCandidateReport};
use super::sequence_spec::SequenceDatasetSpec;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelEscalationDecision {
    KeepBaselineAndExternalBridge,
    ImproveOfficialDataFirst,
    ImproveFeatureSetFirst,
    ImproveSignalModelFirst,
    ImproveRiskGovernorFirst,
    BuildSequenceDatasetFirst,
    BuildMamba3FinExternalPrototype,
    Blocked,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelEscalationGate {
    OfficialConsistency,
    MinimumOutcomes,
    Calibration,
    RiskStability,
    StorageBudget,
    SequenceSpec,
    CryptoOnlyPrototypeAllowed,
    RustNativeDeferred,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ModelEscalationGateConfig {
    #[serde(default = "default_true")]
    pub require_official_consistency: bool,
    pub require_min_outcomes: usize,
    pub require_calibration_threshold: f64,
    #[serde(default = "default_true")]
    pub require_risk_stability: bool,
    #[serde(default = "default_true")]
    pub require_storage_budget_ok: bool,
    #[serde(default = "default_true")]
    pub require_sequence_spec: bool,
    #[serde(default)]
    pub allow_mamba3_prototype_without_equity_data: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for ModelEscalationGateConfig {
    fn default() -> Self {
        Self {
            require_official_consistency: true,
            require_min_outcomes: 20,
            require_calibration_threshold: 0.10,
            require_risk_stability: true,
            require_storage_budget_ok: true,
            require_sequence_spec: true,
            allow_mamba3_prototype_without_equity_data: false,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ModelEscalationGateResult {
    pub decision: ModelEscalationDecision,
    pub passed_gates: Vec<ModelEscalationGate>,
    pub failed_gates: Vec<ModelEscalationGate>,
    pub warnings: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

impl ModelEscalationGateResult {
    #[allow(clippy::too_many_arguments)]
    pub fn evaluate(
        config: &ModelEscalationGateConfig,
        consistency_status: OfficialConsistencyStatus,
        usefulness_status: Option<AiSignalStatus>,
        total_outcomes: usize,
        calibration_error: f64,
        risk_stable: bool,
        storage_budget_ok: bool,
        sequence_spec: Option<&SequenceDatasetSpec>,
        candidate_report: &Mamba3FinCandidateReport,
    ) -> Self {
        let mut passed_gates = vec![ModelEscalationGate::RustNativeDeferred];
        let mut failed_gates = Vec::new();
        let mut warnings = Vec::new();

        let official_consistency_ok =
            matches!(
                consistency_status,
                OfficialConsistencyStatus::ConsistentEnough
            ) || (matches!(consistency_status, OfficialConsistencyStatus::CryptoOnly)
                && config.allow_mamba3_prototype_without_equity_data);
        if config.require_official_consistency && !official_consistency_ok {
            failed_gates.push(ModelEscalationGate::OfficialConsistency);
        } else {
            passed_gates.push(ModelEscalationGate::OfficialConsistency);
        }
        if total_outcomes < config.require_min_outcomes {
            failed_gates.push(ModelEscalationGate::MinimumOutcomes);
        } else {
            passed_gates.push(ModelEscalationGate::MinimumOutcomes);
        }
        if calibration_error > config.require_calibration_threshold {
            failed_gates.push(ModelEscalationGate::Calibration);
        } else {
            passed_gates.push(ModelEscalationGate::Calibration);
        }
        if config.require_risk_stability && !risk_stable {
            failed_gates.push(ModelEscalationGate::RiskStability);
        } else {
            passed_gates.push(ModelEscalationGate::RiskStability);
        }
        if config.require_storage_budget_ok && !storage_budget_ok {
            failed_gates.push(ModelEscalationGate::StorageBudget);
        } else {
            passed_gates.push(ModelEscalationGate::StorageBudget);
        }
        if config.require_sequence_spec
            && !sequence_spec
                .as_ref()
                .is_some_and(|spec| spec.no_lookahead_safe && spec.storage_budget_ok)
        {
            failed_gates.push(ModelEscalationGate::SequenceSpec);
        } else {
            passed_gates.push(ModelEscalationGate::SequenceSpec);
        }
        if matches!(consistency_status, OfficialConsistencyStatus::CryptoOnly)
            && !config.allow_mamba3_prototype_without_equity_data
        {
            failed_gates.push(ModelEscalationGate::CryptoOnlyPrototypeAllowed);
        } else {
            passed_gates.push(ModelEscalationGate::CryptoOnlyPrototypeAllowed);
        }
        if matches!(consistency_status, OfficialConsistencyStatus::CryptoOnly)
            && config.allow_mamba3_prototype_without_equity_data
        {
            warnings
                .push("prototype remains crypto-only until equity evidence is added".to_string());
        }

        let decision = if matches!(
            consistency_status,
            OfficialConsistencyStatus::MissingAuth
                | OfficialConsistencyStatus::MissingEquityData
                | OfficialConsistencyStatus::NeedMoreExperiments
        ) {
            ModelEscalationDecision::ImproveOfficialDataFirst
        } else if matches!(
            usefulness_status.unwrap_or(AiSignalStatus::PipelineOnly),
            AiSignalStatus::PoorCalibration
                | AiSignalStatus::WorseThanBaseline
                | AiSignalStatus::InsufficientOutcomes
        ) {
            ModelEscalationDecision::ImproveSignalModelFirst
        } else if matches!(
            usefulness_status.unwrap_or(AiSignalStatus::PipelineOnly),
            AiSignalStatus::PoorRiskBehavior | AiSignalStatus::RejectedByRisk
        ) {
            ModelEscalationDecision::ImproveRiskGovernorFirst
        } else if failed_gates.contains(&ModelEscalationGate::SequenceSpec) {
            ModelEscalationDecision::BuildSequenceDatasetFirst
        } else if failed_gates.is_empty()
            && matches!(
                candidate_report.readiness,
                Mamba3FinCandidateReadiness::BuildExternalPrototype
            )
        {
            ModelEscalationDecision::BuildMamba3FinExternalPrototype
        } else if failed_gates.is_empty() {
            ModelEscalationDecision::KeepBaselineAndExternalBridge
        } else {
            ModelEscalationDecision::Blocked
        };
        if matches!(
            decision,
            ModelEscalationDecision::BuildMamba3FinExternalPrototype
        ) && !matches!(
            candidate_report.candidate_spec.inference_mode,
            super::mamba_readiness::InferenceMode::ExternalPredictionFile
        ) {
            warnings.push("Rust-native inference is not allowed in this sprint".to_string());
        }

        Self {
            decision,
            passed_gates,
            failed_gates,
            warnings,
            reason_codes: vec![ReasonCode::ModelEscalationEvaluated],
        }
    }

    pub fn to_text(&self) -> String {
        [
            format!("decision={:?}", self.decision),
            format!(
                "passed_gates={}",
                self.passed_gates
                    .iter()
                    .map(|gate| format!("{gate:?}"))
                    .collect::<Vec<_>>()
                    .join("|")
            ),
            format!(
                "failed_gates={}",
                self.failed_gates
                    .iter()
                    .map(|gate| format!("{gate:?}"))
                    .collect::<Vec<_>>()
                    .join("|")
            ),
            format!("warnings={}", self.warnings.join(" | ")),
        ]
        .join("\n")
    }
}

fn default_true() -> bool {
    true
}
