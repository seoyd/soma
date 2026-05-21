use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_hash_string};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RuntimeMode {
    Research,
    Backtest,
    Paper,
    CollectOnly,
    EvaluateOnly,
    DiagnosticsOnly,
    LiveDisabled,
}

impl RuntimeMode {
    pub fn from_active_label(label: &str) -> Result<Self, ReasonCode> {
        match label {
            "research" => Ok(Self::Research),
            "backtest" => Ok(Self::Backtest),
            "paper" => Ok(Self::Paper),
            "collect-only" => Ok(Self::CollectOnly),
            "evaluate-only" => Ok(Self::EvaluateOnly),
            "diagnostics-only" => Ok(Self::DiagnosticsOnly),
            "live" => Err(ReasonCode::LiveModeDisabled),
            _ => Err(ReasonCode::ExperimentConfigInvalid),
        }
    }

    pub fn paper_execution_allowed(self) -> bool {
        matches!(self, Self::Research | Self::Backtest | Self::Paper)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RuntimeStage {
    Init,
    LoadConfig,
    ValidateConfig,
    LoadData,
    ValidateData,
    BuildFeatures,
    GenerateSignals,
    ChairDecision,
    RiskEvaluation,
    PaperExecution,
    OutcomeEvaluation,
    ReportGeneration,
    Completed,
    Failed,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeTransition {
    pub from_stage: RuntimeStage,
    pub to_stage: RuntimeStage,
    pub allowed: bool,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeState {
    pub mode: RuntimeMode,
    pub stage: RuntimeStage,
    pub previous_stage: Option<RuntimeStage>,
    pub transition_history: Vec<RuntimeTransition>,
    pub failed_reason: Option<ReasonCode>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeStateReport {
    pub state: RuntimeState,
    pub fingerprint: String,
    pub reason_codes: Vec<ReasonCode>,
}

impl RuntimeState {
    pub fn new(mode: RuntimeMode) -> Self {
        Self {
            mode,
            stage: RuntimeStage::Init,
            previous_stage: None,
            transition_history: Vec::new(),
            failed_reason: None,
            reason_codes: vec![ReasonCode::RuntimeStateInitialized],
        }
    }

    pub fn transition_to(
        &mut self,
        to_stage: RuntimeStage,
        decision_record_exists: bool,
    ) -> Result<(), ReasonCode> {
        let transition = self.evaluate_transition(to_stage, decision_record_exists);
        self.transition_history.push(transition.clone());
        if transition.allowed {
            self.previous_stage = Some(self.stage);
            self.stage = to_stage;
            if !self
                .reason_codes
                .contains(&ReasonCode::RuntimeTransitionRecorded)
            {
                self.reason_codes
                    .push(ReasonCode::RuntimeTransitionRecorded);
            }
            Ok(())
        } else {
            self.failed_reason = transition.reason_codes.first().cloned();
            self.stage = RuntimeStage::Failed;
            if !self
                .reason_codes
                .contains(&ReasonCode::RuntimeTransitionBlocked)
            {
                self.reason_codes.push(ReasonCode::RuntimeTransitionBlocked);
            }
            Err(self
                .failed_reason
                .clone()
                .unwrap_or(ReasonCode::RuntimeTransitionBlocked))
        }
    }

    pub fn evaluate_transition(
        &self,
        to_stage: RuntimeStage,
        decision_record_exists: bool,
    ) -> RuntimeTransition {
        let mut reason_codes = Vec::new();
        let allowed = if self.stage == RuntimeStage::Failed
            && self.mode != RuntimeMode::DiagnosticsOnly
        {
            reason_codes.push(ReasonCode::RuntimeTransitionBlocked);
            false
        } else if to_stage == RuntimeStage::PaperExecution && !self.mode.paper_execution_allowed() {
            reason_codes.push(ReasonCode::LiveModeDisabled);
            false
        } else if to_stage == RuntimeStage::PaperExecution
            && self.stage != RuntimeStage::RiskEvaluation
            && !self
                .transition_history
                .iter()
                .any(|item| item.allowed && item.to_stage == RuntimeStage::RiskEvaluation)
        {
            reason_codes.push(ReasonCode::RiskDenied);
            false
        } else if to_stage == RuntimeStage::OutcomeEvaluation && !decision_record_exists {
            reason_codes.push(ReasonCode::NoTradeDefault);
            false
        } else if self.stage == RuntimeStage::Completed {
            reason_codes.push(ReasonCode::RuntimeTransitionBlocked);
            false
        } else {
            reason_codes.push(ReasonCode::RuntimeTransitionRecorded);
            true
        };
        RuntimeTransition {
            from_stage: self.stage,
            to_stage,
            allowed,
            reason_codes,
        }
    }
}

impl RuntimeStateReport {
    pub fn from_state(state: RuntimeState) -> Self {
        let fingerprint = stable_hash_string(&format!(
            "{:?}|{:?}|{:?}|{}",
            state.mode,
            state.stage,
            state.previous_stage,
            state
                .transition_history
                .iter()
                .map(|item| format!(
                    "{:?}->{:?}:{}",
                    item.from_stage, item.to_stage, item.allowed
                ))
                .collect::<Vec<_>>()
                .join("|")
        ));
        Self {
            state,
            fingerprint,
            reason_codes: vec![ReasonCode::RuntimeTransitionRecorded],
        }
    }

    pub fn to_text(&self) -> String {
        [
            format!("mode={:?}", self.state.mode),
            format!("stage={:?}", self.state.stage),
            format!("previous_stage={:?}", self.state.previous_stage),
            format!(
                "transition_history={}",
                self.state
                    .transition_history
                    .iter()
                    .map(|item| format!(
                        "{:?}->{:?}:{}",
                        item.from_stage, item.to_stage, item.allowed
                    ))
                    .collect::<Vec<_>>()
                    .join("|")
            ),
            format!("failed_reason={:?}", self.state.failed_reason),
            format!("fingerprint={}", self.fingerprint),
        ]
        .join("\n")
    }
}
