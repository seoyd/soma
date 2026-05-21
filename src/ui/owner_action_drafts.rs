use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::core::{ReasonCode, stable_reason_codes};
use crate::owner::{
    OwnerApplyInputConfig, OwnerInput, OwnerInputKind, OwnerInputStatus, OwnerInputTargetType,
    OwnerThesisBookConfig, OwnerThesisNote, OwnerThesisType,
};

use super::control_tower_v1::{ControlTowerV1Config, ControlTowerV1State};
use super::dashboard_panels::CandidateStatus;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum OwnerActionDraftKind {
    #[default]
    NoteDraft,
    HoldDraft,
    DismissDraft,
    ReanalysisDraft,
    MarkReviewedDraft,
    PaperConfirmDraft,
    ThesisNoteDraft,
    RiskTightenDraft,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OwnerActionDraft {
    pub draft_id: String,
    pub draft_kind: OwnerActionDraftKind,
    #[serde(default)]
    pub target_candidate_id: Option<String>,
    #[serde(default)]
    pub target_symbol: Option<String>,
    #[serde(default)]
    pub target_market: Option<String>,
    pub generated_from_panel: String,
    pub suggested_owner_input_config_path: String,
    pub allowed_by_policy: bool,
    #[serde(default)]
    pub blocked_reason_codes: Vec<ReasonCode>,
    pub requires_owner_review: bool,
    pub paper_only: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OwnerActionDraftBundle {
    #[serde(default)]
    pub drafts: Vec<OwnerActionDraft>,
    pub allowed_drafts: usize,
    pub blocked_drafts: usize,
    pub draft_output_dir: String,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl OwnerActionDraftBundle {
    pub fn to_text(&self) -> String {
        let mut lines = vec![
            "paper_only_warning=owner action drafts are local files and never auto-apply"
                .to_string(),
            format!("draft_output_dir={}", self.draft_output_dir),
            format!("draft_count={}", self.drafts.len()),
            format!("allowed_drafts={}", self.allowed_drafts),
            format!("blocked_drafts={}", self.blocked_drafts),
        ];
        lines.extend(self.drafts.iter().map(|draft| {
            format!(
                "draft_id={};kind={:?};allowed={};path={}",
                draft.draft_id,
                draft.draft_kind,
                draft.allowed_by_policy,
                draft.suggested_owner_input_config_path
            )
        }));
        lines.join("\n")
    }
}

pub fn generate_owner_action_draft_bundle(
    state: &ControlTowerV1State,
    config: &ControlTowerV1Config,
) -> Result<OwnerActionDraftBundle, String> {
    let output_dir = config.artifact_dir().join("owner_action_drafts");
    fs::create_dir_all(&output_dir).map_err(|err| err.to_string())?;
    let candidate_queue_path = config
        .candidate_queue_paths
        .first()
        .cloned()
        .unwrap_or_else(|| {
            "examples/sprint53_data/candidate_queue_with_owner_items.json".to_string()
        });

    let mut drafts = Vec::new();
    for candidate in &state.candidate_panel.candidates {
        drafts.push(write_apply_draft(
            &output_dir,
            config,
            &candidate_queue_path,
            state,
            &candidate.candidate_id,
            OwnerActionDraftKind::NoteDraft,
            OwnerInputKind::CandidateNote,
            Some("owner note draft generated from Control Tower v1".to_string()),
            None,
        )?);
        if !matches!(candidate.status, CandidateStatus::Expired) {
            drafts.push(write_apply_draft(
                &output_dir,
                config,
                &candidate_queue_path,
                state,
                &candidate.candidate_id,
                OwnerActionDraftKind::HoldDraft,
                OwnerInputKind::CandidateHold,
                None,
                Some(vec![(
                    "hold_reason".to_string(),
                    "local_owner_review".to_string(),
                )]),
            )?);
            drafts.push(write_apply_draft(
                &output_dir,
                config,
                &candidate_queue_path,
                state,
                &candidate.candidate_id,
                OwnerActionDraftKind::DismissDraft,
                OwnerInputKind::CandidateDismiss,
                None,
                Some(vec![(
                    "dismiss_reason".to_string(),
                    "owner_local_review".to_string(),
                )]),
            )?);
            drafts.push(write_apply_draft(
                &output_dir,
                config,
                &candidate_queue_path,
                state,
                &candidate.candidate_id,
                OwnerActionDraftKind::ReanalysisDraft,
                OwnerInputKind::CandidateReanalysisRequest,
                None,
                Some(vec![("focus".to_string(), "risk_regime_check".to_string())]),
            )?);
            drafts.push(write_apply_draft(
                &output_dir,
                config,
                &candidate_queue_path,
                state,
                &candidate.candidate_id,
                OwnerActionDraftKind::MarkReviewedDraft,
                OwnerInputKind::MarkReviewed,
                None,
                None,
            )?);
        }
        if paper_confirm_allowed(state, &candidate.candidate_id, candidate.status) {
            drafts.push(write_apply_draft(
                &output_dir,
                config,
                &candidate_queue_path,
                state,
                &candidate.candidate_id,
                OwnerActionDraftKind::PaperConfirmDraft,
                OwnerInputKind::PaperConfirm,
                None,
                Some(vec![(
                    "confirm_reason".to_string(),
                    "paper_only_review_complete".to_string(),
                )]),
            )?);
        }
        drafts.push(write_thesis_draft(&output_dir, candidate, config)?);
    }

    if state.risk_panel.denied_count > 0 {
        drafts.push(write_system_risk_tighten_draft(
            &output_dir,
            config,
            &candidate_queue_path,
        )?);
    }

    drafts.sort_by(|left, right| left.draft_id.cmp(&right.draft_id));
    let allowed_drafts = drafts
        .iter()
        .filter(|draft| draft.allowed_by_policy)
        .count();
    let blocked_drafts = drafts.len().saturating_sub(allowed_drafts);
    Ok(OwnerActionDraftBundle {
        drafts,
        allowed_drafts,
        blocked_drafts,
        draft_output_dir: output_dir.display().to_string(),
        reason_codes: stable_reason_codes(&[ReasonCode::OwnerInputApplied]),
    })
}

fn write_apply_draft(
    output_dir: &Path,
    _config: &ControlTowerV1Config,
    candidate_queue_path: &str,
    state: &ControlTowerV1State,
    candidate_id: &str,
    draft_kind: OwnerActionDraftKind,
    input_kind: OwnerInputKind,
    freeform_note: Option<String>,
    structured_payload: Option<Vec<(String, String)>>,
) -> Result<OwnerActionDraft, String> {
    let candidate = state
        .candidate_panel
        .candidates
        .iter()
        .find(|candidate| candidate.candidate_id == candidate_id)
        .ok_or_else(|| format!("missing candidate for draft {candidate_id}"))?;
    let draft_id = format!(
        "{}-{}",
        kind_label(draft_kind),
        candidate_id.to_ascii_lowercase()
    );
    let path = output_dir.join(format!("{draft_id}.toml"));
    let owner_input = OwnerInput {
        owner_input_id: draft_id.clone(),
        timestamp_ms: Some(1715700000000),
        owner_id: Some("owner-local".to_string()),
        input_kind,
        target_type: OwnerInputTargetType::Candidate,
        target_id: Some(candidate.candidate_id.clone()),
        symbol: Some(candidate.symbol.clone()),
        market: Some(candidate.market.clone()),
        freeform_note,
        structured_payload: structured_payload.map(|items| items.into_iter().collect()),
        requested_action: Some(requested_action(input_kind).to_string()),
        status: OwnerInputStatus::Submitted,
        reason_codes: vec![ReasonCode::DeterministicPath],
    };
    let apply_config = OwnerApplyInputConfig {
        apply_id: format!("apply-{draft_id}"),
        candidate_queue_path: candidate_queue_path.to_string(),
        owner_input,
        protocol: Default::default(),
        reason_codes: vec![ReasonCode::DeterministicPath],
    };
    fs::write(
        &path,
        toml::to_string_pretty(&apply_config).map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    let allowed_by_policy = !matches!(draft_kind, OwnerActionDraftKind::PaperConfirmDraft)
        || paper_confirm_allowed(state, &candidate.candidate_id, candidate.status);
    let blocked_reason_codes = if allowed_by_policy {
        Vec::new()
    } else {
        stable_reason_codes(&[ReasonCode::OwnerPaperConfirmBlocked, ReasonCode::RiskDenied])
    };
    Ok(OwnerActionDraft {
        draft_id,
        draft_kind,
        target_candidate_id: Some(candidate.candidate_id.clone()),
        target_symbol: Some(candidate.symbol.clone()),
        target_market: Some(candidate.market.clone()),
        generated_from_panel: if matches!(draft_kind, OwnerActionDraftKind::PaperConfirmDraft) {
            "human_confirm_panel".to_string()
        } else {
            "owner_panel".to_string()
        },
        suggested_owner_input_config_path: path.display().to_string(),
        allowed_by_policy,
        blocked_reason_codes,
        requires_owner_review: true,
        paper_only: true,
        reason_codes: stable_reason_codes(&[ReasonCode::OwnerInputApplied]),
    })
}

fn write_thesis_draft(
    output_dir: &Path,
    candidate: &super::dashboard_panels::CandidateView,
    _config: &ControlTowerV1Config,
) -> Result<OwnerActionDraft, String> {
    let draft_id = format!(
        "thesis-note-{}",
        candidate.candidate_id.to_ascii_lowercase()
    );
    let json_path = output_dir.join(format!("{draft_id}.json"));
    let config_path = output_dir.join(format!("{draft_id}.toml"));
    let note = OwnerThesisNote {
        thesis_id: draft_id.clone(),
        symbol: Some(candidate.symbol.clone()),
        market: Some(candidate.market.clone()),
        timeframe: Some(candidate.timeframe.clone()),
        thesis_type: if matches!(candidate.status, CandidateStatus::RiskBlocked) {
            OwnerThesisType::RiskWarning
        } else {
            OwnerThesisType::EventNote
        },
        text: format!(
            "Local thesis note draft for {} remains paper-only and requires manual owner review.",
            candidate.symbol
        ),
        structured_tags: vec!["control-tower-v1".to_string(), "paper-only".to_string()],
        evidence_links: Some(vec![
            candidate.created_from_report.clone().unwrap_or_default(),
        ]),
        expires_at_timestamp_ms: candidate.expires_at,
        active: true,
        reason_codes: vec![ReasonCode::DeterministicPath],
    };
    fs::write(
        &json_path,
        serde_json::to_string_pretty(&json!({"owner_thesis_notes": [note]}))
            .map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    let thesis_config = OwnerThesisBookConfig {
        thesis_notes_path: json_path.display().to_string(),
        timestamp_ms: Some(1715700000000),
        reason_codes: vec![ReasonCode::DeterministicPath],
    };
    fs::write(
        &config_path,
        toml::to_string_pretty(&thesis_config).map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    Ok(OwnerActionDraft {
        draft_id,
        draft_kind: OwnerActionDraftKind::ThesisNoteDraft,
        target_candidate_id: Some(candidate.candidate_id.clone()),
        target_symbol: Some(candidate.symbol.clone()),
        target_market: Some(candidate.market.clone()),
        generated_from_panel: "owner_panel".to_string(),
        suggested_owner_input_config_path: config_path.display().to_string(),
        allowed_by_policy: true,
        blocked_reason_codes: Vec::new(),
        requires_owner_review: true,
        paper_only: true,
        reason_codes: stable_reason_codes(&[ReasonCode::OwnerThesisBookBuilt]),
    })
}

fn write_system_risk_tighten_draft(
    output_dir: &Path,
    _config: &ControlTowerV1Config,
    candidate_queue_path: &str,
) -> Result<OwnerActionDraft, String> {
    let draft_id = "risk-tighten-system".to_string();
    let path = output_dir.join("risk-tighten-system.toml");
    let apply_config = OwnerApplyInputConfig {
        apply_id: "apply-risk-tighten-system".to_string(),
        candidate_queue_path: candidate_queue_path.to_string(),
        owner_input: OwnerInput {
            owner_input_id: draft_id.clone(),
            timestamp_ms: Some(1715700000000),
            owner_id: Some("owner-local".to_string()),
            input_kind: OwnerInputKind::RiskTightenRequest,
            target_type: OwnerInputTargetType::System,
            target_id: None,
            symbol: None,
            market: None,
            freeform_note: Some("Request a more conservative paper-only risk review.".to_string()),
            structured_payload: Some(
                vec![("review_mode".to_string(), "more_conservative".to_string())]
                    .into_iter()
                    .collect(),
            ),
            requested_action: Some("tighten_risk".to_string()),
            status: OwnerInputStatus::Submitted,
            reason_codes: vec![ReasonCode::DeterministicPath],
        },
        protocol: Default::default(),
        reason_codes: vec![ReasonCode::DeterministicPath],
    };
    fs::write(
        &path,
        toml::to_string_pretty(&apply_config).map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    Ok(OwnerActionDraft {
        draft_id,
        draft_kind: OwnerActionDraftKind::RiskTightenDraft,
        target_candidate_id: None,
        target_symbol: None,
        target_market: None,
        generated_from_panel: "risk_panel".to_string(),
        suggested_owner_input_config_path: path.display().to_string(),
        allowed_by_policy: true,
        blocked_reason_codes: Vec::new(),
        requires_owner_review: true,
        paper_only: true,
        reason_codes: stable_reason_codes(&[ReasonCode::OwnerRiskTightenRequested]),
    })
}

fn paper_confirm_allowed(
    state: &ControlTowerV1State,
    candidate_id: &str,
    status: CandidateStatus,
) -> bool {
    if matches!(
        status,
        CandidateStatus::RiskBlocked | CandidateStatus::NoTrade
    ) {
        return false;
    }
    state
        .owner_panel
        .pending_review_items
        .iter()
        .chain(state.owner_panel.paper_confirmed_items.iter())
        .find(|item| item.candidate_id.as_deref() == Some(candidate_id))
        .map(|item| {
            item.allowed_owner_actions
                .iter()
                .any(|action| format!("{:?}", action) == "PaperConfirm")
        })
        .unwrap_or(matches!(
            status,
            CandidateStatus::HumanConfirmRequired | CandidateStatus::PaperApproved
        ))
}

fn requested_action(kind: OwnerInputKind) -> &'static str {
    match kind {
        OwnerInputKind::CandidateNote => "add_note",
        OwnerInputKind::CandidateHold => "hold_candidate",
        OwnerInputKind::CandidateDismiss => "dismiss_candidate",
        OwnerInputKind::CandidateReanalysisRequest => "request_reanalysis",
        OwnerInputKind::MarkReviewed => "mark_reviewed",
        OwnerInputKind::PaperConfirm => "paper_confirm_candidate",
        OwnerInputKind::RiskTightenRequest => "tighten_risk",
        _ => "owner_action",
    }
}

fn kind_label(kind: OwnerActionDraftKind) -> &'static str {
    match kind {
        OwnerActionDraftKind::NoteDraft => "note",
        OwnerActionDraftKind::HoldDraft => "hold",
        OwnerActionDraftKind::DismissDraft => "dismiss",
        OwnerActionDraftKind::ReanalysisDraft => "reanalysis",
        OwnerActionDraftKind::MarkReviewedDraft => "mark-reviewed",
        OwnerActionDraftKind::PaperConfirmDraft => "paper-confirm",
        OwnerActionDraftKind::ThesisNoteDraft => "thesis-note",
        OwnerActionDraftKind::RiskTightenDraft => "risk-tighten",
    }
}
