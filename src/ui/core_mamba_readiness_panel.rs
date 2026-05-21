use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::core::{ReasonCode, stable_ordered_strings, stable_reason_codes};
use crate::experiment::CoreCompletionAuditReport;
use crate::model::{
    Mamba3ReadinessAuditV2, ModelEscalationDecisionV2, SequenceDatasetReadinessReport,
};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CoreMambaReadinessPanel {
    pub core_completion_status: String,
    pub mamba3_readiness_state: String,
    pub sequence_dataset_status: String,
    pub selected_model_escalation_decision: String,
    #[serde(default)]
    pub blocked_reasons: Vec<String>,
    #[serde(default)]
    pub next_actions: Vec<String>,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl CoreMambaReadinessPanel {
    pub fn stabilize(&mut self) {
        self.blocked_reasons = stable_ordered_strings(&self.blocked_reasons);
        self.next_actions = stable_ordered_strings(&self.next_actions);
        self.warnings = stable_ordered_strings(&self.warnings);
        self.reason_codes = stable_reason_codes(&self.reason_codes);
    }

    pub fn to_text(&self) -> String {
        [
            format!("core_completion_status={}", self.core_completion_status),
            format!("mamba3_readiness_state={}", self.mamba3_readiness_state),
            format!("sequence_dataset_status={}", self.sequence_dataset_status),
            format!(
                "selected_model_escalation_decision={}",
                self.selected_model_escalation_decision
            ),
            format!("blocked_reasons={}", self.blocked_reasons.join(" | ")),
            format!("next_actions={}", self.next_actions.join(" | ")),
            format!("warnings={}", self.warnings.join(" | ")),
            "mamba_runtime_implemented=false".to_string(),
            "train_button=false".to_string(),
            "live_button=false".to_string(),
        ]
        .join("\n")
    }
}

pub fn build_core_mamba_readiness_panel(
    core_report: Option<&CoreCompletionAuditReport>,
    mamba_report: Option<&Mamba3ReadinessAuditV2>,
    sequence_report: Option<&SequenceDatasetReadinessReport>,
    model_decision: Option<&ModelEscalationDecisionV2>,
) -> Option<CoreMambaReadinessPanel> {
    if core_report.is_none()
        && mamba_report.is_none()
        && sequence_report.is_none()
        && model_decision.is_none()
    {
        return None;
    }

    let mut panel = CoreMambaReadinessPanel {
        core_completion_status: core_report
            .map(|report| format!("{:?}", report.core_completion_status))
            .unwrap_or_else(|| "Unavailable".to_string()),
        mamba3_readiness_state: mamba_report
            .map(|report| format!("{:?}", report.readiness_state))
            .unwrap_or_else(|| "Deferred".to_string()),
        sequence_dataset_status: sequence_report
            .map(|report| format!("{:?}", report.readiness_status))
            .unwrap_or_else(|| "Unavailable".to_string()),
        selected_model_escalation_decision: model_decision
            .map(|report| format!("{:?}", report.selected_candidate))
            .unwrap_or_else(|| "NoEscalation".to_string()),
        blocked_reasons: core_report
            .map(|report| report.failed_core_requirements.clone())
            .unwrap_or_default(),
        next_actions: model_decision
            .map(|report| report.next_actions.clone())
            .unwrap_or_default(),
        warnings: core_report
            .map(|report| report.warnings.clone())
            .unwrap_or_default(),
        reason_codes: vec![
            ReasonCode::DashboardStateBuilt,
            ReasonCode::DeterministicPath,
        ],
    };
    if let Some(report) = sequence_report {
        panel.blocked_reasons.extend(report.blockers.clone());
        panel.warnings.extend(report.warnings.clone());
    }
    if let Some(report) = mamba_report {
        panel.blocked_reasons.extend(report.blockers.clone());
        panel.warnings.extend(report.warnings.clone());
        if !matches!(
            report.readiness_state,
            crate::model::Mamba3ReadinessState::ReadyForExternalPrototype
        ) {
            panel.warnings.push(
                "Mamba remains deferred in the UI until an external-prototype-only gate passes."
                    .to_string(),
            );
        }
    }
    if let Some(report) = model_decision {
        panel.blocked_reasons.extend(report.prerequisites.clone());
    }
    panel.stabilize();
    Some(panel)
}

pub fn build_core_mamba_readiness_panel_from_values(
    core_values: &[Value],
    mamba_values: &[Value],
    sequence_values: &[Value],
    model_values: &[Value],
) -> Option<CoreMambaReadinessPanel> {
    let core_report = core_values
        .iter()
        .find_map(CoreCompletionAuditReport::from_value);
    let mamba_report = mamba_values
        .iter()
        .find_map(Mamba3ReadinessAuditV2::from_value);
    let sequence_report = sequence_values
        .iter()
        .find_map(SequenceDatasetReadinessReport::from_value);
    let model_decision = model_values
        .iter()
        .find_map(ModelEscalationDecisionV2::from_value);
    build_core_mamba_readiness_panel(
        core_report.as_ref(),
        mamba_report.as_ref(),
        sequence_report.as_ref(),
        model_decision.as_ref(),
    )
}
