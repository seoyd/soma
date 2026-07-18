pub mod human_confirm_protocol;
pub mod owner_candidate_feedback;
pub mod owner_impact;
pub mod owner_input;
pub mod owner_policy;
pub mod owner_review_queue;
pub mod owner_thesis;

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::core::{ReasonCode, stable_hash_string, stable_reason_codes};
use crate::ui::CandidatePanel;

pub use human_confirm_protocol::{
    HumanConfirmProtocolConfig, HumanConfirmProtocolReport, HumanConfirmState,
    HumanConfirmTransition, build_human_confirm_protocol_report, evaluate_human_confirm_transition,
};
pub use owner_candidate_feedback::{
    OwnerCandidateFeedback, OwnerCandidateFeedbackKind, OwnerFeedbackDecisionEffect,
    build_owner_candidate_feedback,
};
pub use owner_impact::{
    OwnerDecisionImpactFinalStatus, OwnerDecisionImpactKind, OwnerDecisionImpactRecord,
    OwnerDecisionImpactReport, build_owner_decision_impact_report,
};
pub use owner_input::{OwnerInput, OwnerInputKind, OwnerInputStatus, OwnerInputTargetType};
pub use owner_policy::{
    ChairShadowOwnerAdvisoryReviewInputV0, ChairShadowOwnerAdvisoryReviewV0,
    ChairShadowOwnerReviewLedgerV0, OwnerAdvisoryDecisionFirewallProofV0, OwnerPolicyConstraint,
    OwnerPolicyConstraintKind, OwnerPolicyValidationResult, OwnerShadowAdvisoryStatusV0,
    OwnerTradeRequestReview, append_chair_shadow_owner_review_ledger_v0,
    chair_shadow_owner_advisory_fixture_inputs_v0, chair_shadow_owner_advisory_review_input_v0,
    default_owner_policy_constraints, new_chair_shadow_owner_review_ledger_v0,
    owner_advisory_decision_firewall_proof_v0, owner_rejection_explanation,
    read_chair_shadow_owner_review_ledger_v0, review_chair_shadow_owner_advisory_v0,
    review_owner_trade_request, validate_chair_shadow_owner_review_ledger_v0, validate_owner_input,
};
pub use owner_review_queue::{
    AllowedOwnerAction, ForbiddenOwnerAction, OwnerReviewItem, OwnerReviewItemStatus,
    OwnerReviewQueue, build_owner_review_queue,
};
pub use owner_thesis::{OwnerThesisBook, OwnerThesisNote, OwnerThesisType};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OwnerInputValidateConfig {
    pub owner_input: OwnerInput,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OwnerReviewQueueConfig {
    pub queue_id: String,
    pub candidate_queue_path: String,
    pub owner_inputs_path: String,
    #[serde(default)]
    pub protocol: HumanConfirmProtocolConfig,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OwnerApplyInputConfig {
    pub apply_id: String,
    pub candidate_queue_path: String,
    pub owner_input: OwnerInput,
    #[serde(default)]
    pub protocol: HumanConfirmProtocolConfig,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OwnerImpactReportConfig {
    pub report_id: String,
    pub candidate_queue_path: String,
    pub owner_inputs_path: String,
    #[serde(default)]
    pub protocol: HumanConfirmProtocolConfig,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OwnerThesisBookConfig {
    pub thesis_notes_path: String,
    #[serde(default)]
    pub timestamp_ms: Option<u64>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OwnerInputValidationReport {
    pub owner_input: OwnerInput,
    pub validation: OwnerPolicyValidationResult,
    pub fingerprint: String,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OwnerApplyInputReport {
    pub apply_id: String,
    pub owner_input: OwnerInput,
    pub validation: OwnerPolicyValidationResult,
    #[serde(default)]
    pub feedbacks: Vec<OwnerCandidateFeedback>,
    pub impact_report: OwnerDecisionImpactReport,
    pub fingerprint: String,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

macro_rules! impl_toml_loader {
    ($name:ident) => {
        impl $name {
            pub fn from_toml_path(path: &Path) -> Result<Self, String> {
                let contents = fs::read_to_string(path).map_err(|err| err.to_string())?;
                toml::from_str(&contents).map_err(|err| err.to_string())
            }
        }
    };
}

impl_toml_loader!(OwnerInputValidateConfig);
impl_toml_loader!(OwnerReviewQueueConfig);
impl_toml_loader!(OwnerApplyInputConfig);
impl_toml_loader!(OwnerImpactReportConfig);
impl_toml_loader!(OwnerThesisBookConfig);

impl OwnerInputValidateConfig {
    pub fn validate_local_paths(&self) -> Vec<ReasonCode> {
        Vec::new()
    }
}

impl OwnerReviewQueueConfig {
    pub fn validate_local_paths(&self) -> Vec<ReasonCode> {
        remote_path_reason_codes(&[&self.candidate_queue_path, &self.owner_inputs_path])
    }
}

impl OwnerApplyInputConfig {
    pub fn validate_local_paths(&self) -> Vec<ReasonCode> {
        remote_path_reason_codes(&[&self.candidate_queue_path])
    }
}

impl OwnerImpactReportConfig {
    pub fn validate_local_paths(&self) -> Vec<ReasonCode> {
        remote_path_reason_codes(&[&self.candidate_queue_path, &self.owner_inputs_path])
    }
}

impl OwnerThesisBookConfig {
    pub fn validate_local_paths(&self) -> Vec<ReasonCode> {
        remote_path_reason_codes(&[&self.thesis_notes_path])
    }
}

impl OwnerInputValidationReport {
    pub fn to_text(&self) -> String {
        [
            "research_only_warning=owner input validation is structured audited review only"
                .to_string(),
            "paper_only_warning=paper confirm remains paper-only and never creates orders"
                .to_string(),
            self.owner_input.summary_line(),
            self.validation.to_text(),
            format!("fingerprint={}", self.fingerprint),
        ]
        .join("\n")
    }
}

impl OwnerApplyInputReport {
    pub fn to_text(&self) -> String {
        [
            "research_only_warning=owner apply input simulates audited local review only".to_string(),
            "paper_only_warning=paper confirm stays paper-only and never touches broker/account paths".to_string(),
            format!("apply_id={}", self.apply_id),
            format!("input_id={}", self.owner_input.owner_input_id),
            format!("validation_allowed={}", self.validation.allowed),
            format!("feedback_count={}", self.feedbacks.len()),
            format!("impact_status={:?}", self.impact_report.final_status),
            format!("fingerprint={}", self.fingerprint),
        ]
        .join("\n")
    }
}

pub fn run_owner_input_validation(config: &OwnerInputValidateConfig) -> OwnerInputValidationReport {
    let owner_input = config.owner_input.clone().with_fingerprint();
    let validation = validate_owner_input(&owner_input);
    OwnerInputValidationReport {
        fingerprint: stable_hash_string(&format!(
            "{}|{}",
            owner_input.fingerprint(),
            serde_json::to_string(&validation).unwrap_or_default()
        )),
        owner_input,
        validation,
        reason_codes: stable_reason_codes(&[
            ReasonCode::OwnerInputValidated,
            ReasonCode::OwnerPolicyValidated,
        ]),
    }
}

pub fn run_owner_review_queue(config: &OwnerReviewQueueConfig) -> Result<OwnerReviewQueue, String> {
    if !config.validate_local_paths().is_empty() {
        return Err("owner-review-queue config paths must be local".to_string());
    }
    let candidate_panel = load_candidate_panel_from_path(Path::new(&config.candidate_queue_path))?;
    let owner_inputs = load_owner_inputs_from_path(Path::new(&config.owner_inputs_path))?;
    Ok(build_owner_review_queue(
        &config.queue_id,
        &candidate_panel,
        &owner_inputs,
        &config.protocol,
    ))
}

pub fn run_owner_apply_input(
    config: &OwnerApplyInputConfig,
) -> Result<OwnerApplyInputReport, String> {
    if !config.validate_local_paths().is_empty() {
        return Err("owner-apply-input config paths must be local".to_string());
    }
    let candidate_panel = load_candidate_panel_from_path(Path::new(&config.candidate_queue_path))?;
    let owner_input = config.owner_input.clone().with_fingerprint();
    let validation = validate_owner_input(&owner_input);
    let feedbacks = build_owner_candidate_feedback(&owner_input, &validation)
        .into_iter()
        .collect::<Vec<_>>();
    let impact_report = build_owner_decision_impact_report(
        &config.apply_id,
        &candidate_panel,
        &[owner_input.clone()],
        &config.protocol,
    );
    Ok(OwnerApplyInputReport {
        apply_id: config.apply_id.clone(),
        fingerprint: stable_hash_string(&format!(
            "{}|{}",
            owner_input.fingerprint(),
            impact_report.fingerprint()
        )),
        owner_input,
        validation,
        feedbacks,
        impact_report,
        reason_codes: stable_reason_codes(&[
            ReasonCode::OwnerInputApplied,
            ReasonCode::OwnerImpactReportBuilt,
        ]),
    })
}

pub fn run_owner_impact_report(
    config: &OwnerImpactReportConfig,
) -> Result<OwnerDecisionImpactReport, String> {
    if !config.validate_local_paths().is_empty() {
        return Err("owner-impact-report config paths must be local".to_string());
    }
    let candidate_panel = load_candidate_panel_from_path(Path::new(&config.candidate_queue_path))?;
    let owner_inputs = load_owner_inputs_from_path(Path::new(&config.owner_inputs_path))?;
    Ok(build_owner_decision_impact_report(
        &config.report_id,
        &candidate_panel,
        &owner_inputs,
        &config.protocol,
    ))
}

pub fn run_owner_thesis_book(config: &OwnerThesisBookConfig) -> Result<OwnerThesisBook, String> {
    if !config.validate_local_paths().is_empty() {
        return Err("owner-thesis-book config paths must be local".to_string());
    }
    let notes = load_owner_thesis_notes_from_path(Path::new(&config.thesis_notes_path))?;
    Ok(OwnerThesisBook::from_notes(&notes, config.timestamp_ms))
}

pub fn load_candidate_panel_from_path(path: &Path) -> Result<CandidatePanel, String> {
    let value = load_json_value(path)?;
    parse_candidate_panel_value(&value)
}

pub fn load_owner_inputs_from_path(path: &Path) -> Result<Vec<OwnerInput>, String> {
    let value = load_json_value(path)?;
    parse_owner_inputs_value(&value)
}

pub fn load_owner_thesis_notes_from_path(path: &Path) -> Result<Vec<OwnerThesisNote>, String> {
    let value = load_json_value(path)?;
    parse_owner_thesis_notes_value(&value)
}

pub fn parse_owner_inputs_value(value: &Value) -> Result<Vec<OwnerInput>, String> {
    if let Ok(inputs) = serde_json::from_value::<Vec<OwnerInput>>(value.clone()) {
        return Ok(stabilize_owner_inputs(inputs));
    }
    for key in [
        "owner_inputs",
        "recent_owner_inputs",
        "blocked_owner_inputs",
    ] {
        if let Some(entry) = value.get(key) {
            if let Ok(inputs) = serde_json::from_value::<Vec<OwnerInput>>(entry.clone()) {
                return Ok(stabilize_owner_inputs(inputs));
            }
        }
    }
    Err("owner input JSON must contain an array of OwnerInput records".to_string())
}

pub fn parse_owner_thesis_notes_value(value: &Value) -> Result<Vec<OwnerThesisNote>, String> {
    if let Ok(notes) = serde_json::from_value::<Vec<OwnerThesisNote>>(value.clone()) {
        return Ok(stabilize_owner_notes(notes));
    }
    for key in ["owner_thesis_notes", "active_notes", "notes"] {
        if let Some(entry) = value.get(key) {
            if let Ok(notes) = serde_json::from_value::<Vec<OwnerThesisNote>>(entry.clone()) {
                return Ok(stabilize_owner_notes(notes));
            }
        }
    }
    if let Some(entry) = value.get("owner_thesis_book") {
        if let Some(active) = entry.get("active_notes") {
            if let Ok(notes) = serde_json::from_value::<Vec<OwnerThesisNote>>(active.clone()) {
                return Ok(stabilize_owner_notes(notes));
            }
        }
    }
    Err("owner thesis JSON must contain an array of OwnerThesisNote records".to_string())
}

fn parse_candidate_panel_value(value: &Value) -> Result<CandidatePanel, String> {
    serde_json::from_value::<CandidatePanel>(
        value
            .get("candidate_panel")
            .cloned()
            .unwrap_or_else(|| value.clone()),
    )
    .map_err(|err| err.to_string())
}

fn load_json_value(path: &Path) -> Result<Value, String> {
    let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
    serde_json::from_str(&text).map_err(|err| err.to_string())
}

fn stabilize_owner_inputs(mut inputs: Vec<OwnerInput>) -> Vec<OwnerInput> {
    inputs.sort_by(|left, right| left.owner_input_id.cmp(&right.owner_input_id));
    for input in &mut inputs {
        input.stabilize();
    }
    inputs
}

fn stabilize_owner_notes(mut notes: Vec<OwnerThesisNote>) -> Vec<OwnerThesisNote> {
    notes.sort_by(|left, right| left.thesis_id.cmp(&right.thesis_id));
    for note in &mut notes {
        note.stabilize();
    }
    notes
}

fn remote_path_reason_codes(paths: &[&str]) -> Vec<ReasonCode> {
    if paths.iter().any(|path| path.contains("://")) {
        vec![
            ReasonCode::LocalPathRejected,
            ReasonCode::RemotePathRejected,
        ]
    } else {
        Vec::new()
    }
}
