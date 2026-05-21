use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};
use crate::experiment::ProviderPriorityMode;
use crate::owner::{
    AllowedOwnerAction, ForbiddenOwnerAction, OwnerCandidateFeedback, OwnerThesisNote,
};

use super::dashboard_state::DashboardEntityStatus;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DashboardNamedProviderStatus {
    pub provider_label: String,
    pub mode: ProviderPriorityMode,
    pub enabled: bool,
    #[serde(default)]
    pub notes: Vec<String>,
    #[serde(default)]
    pub blocked_reasons: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DashboardKisStatus {
    pub auth_ready: bool,
    pub endpoint_policy_status: String,
    pub domestic_market_data_ready: bool,
    pub overseas_market_data_ready: bool,
    pub realtime_ready: bool,
    #[serde(default)]
    pub blocked_reasons: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DashboardKrxStatus {
    pub reference_enabled: bool,
    pub fallback_enabled: bool,
    #[serde(default)]
    pub blocked_reasons: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderPanel {
    #[serde(default)]
    pub active_primary_provider_by_market: BTreeMap<String, String>,
    pub kis_status: DashboardKisStatus,
    pub krx_status: DashboardKrxStatus,
    pub alpha_vantage_status: DashboardNamedProviderStatus,
    pub yfinance_status: DashboardNamedProviderStatus,
    pub upbit_status: DashboardNamedProviderStatus,
    #[serde(default)]
    pub operator_actions: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl ProviderPanel {
    pub fn stabilize(&mut self) {
        self.operator_actions.sort();
        self.operator_actions.dedup();
        self.kis_status.blocked_reasons.sort();
        self.kis_status.blocked_reasons.dedup();
        self.krx_status.blocked_reasons.sort();
        self.krx_status.blocked_reasons.dedup();
        self.alpha_vantage_status.notes.sort();
        self.alpha_vantage_status.notes.dedup();
        self.alpha_vantage_status.blocked_reasons.sort();
        self.alpha_vantage_status.blocked_reasons.dedup();
        self.yfinance_status.notes.sort();
        self.yfinance_status.notes.dedup();
        self.upbit_status.notes.sort();
        self.upbit_status.notes.dedup();
        self.reason_codes = stable_reason_codes(&self.reason_codes);
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceSourceBreakdown {
    pub official_non_crypto: usize,
    pub crypto_only: usize,
    pub research_only: usize,
    pub fixture_only: usize,
    pub controlled_diagnostic: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidencePanel {
    #[serde(default)]
    pub official_rows_before: Option<usize>,
    pub official_rows: usize,
    #[serde(default)]
    pub official_rows_after: Option<usize>,
    #[serde(default)]
    pub complete_rows_before: Option<usize>,
    pub official_complete_rows: usize,
    #[serde(default)]
    pub complete_rows_after: Option<usize>,
    #[serde(default)]
    pub outcome_links_before: Option<usize>,
    pub outcome_links: usize,
    #[serde(default)]
    pub outcome_links_after: Option<usize>,
    #[serde(default)]
    pub counterfactuals_before: Option<usize>,
    #[serde(default)]
    pub counterfactuals_after: Option<usize>,
    pub no_trade_counterfactuals: usize,
    pub risk_denied_counterfactuals: usize,
    pub candle_sufficiency_status: String,
    pub diversity_status: String,
    pub sufficiency_status: String,
    pub source_breakdown: EvidenceSourceBreakdown,
    pub current_bottleneck: String,
    pub next_recommended_action: String,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl EvidencePanel {
    pub fn stabilize(&mut self) {
        self.reason_codes = stable_reason_codes(&self.reason_codes);
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CommitteeMemberView {
    pub persona_id: String,
    pub archetype_label: String,
    pub status: DashboardEntityStatus,
    #[serde(default)]
    pub current_symbol: Option<String>,
    #[serde(default)]
    pub current_candidate_id: Option<String>,
    #[serde(default)]
    pub last_stance: Option<String>,
    #[serde(default)]
    pub conviction: Option<f64>,
    #[serde(default)]
    pub voice_power: Option<f64>,
    pub selected_by_chair: bool,
    #[serde(default)]
    pub filtered_reason: Option<String>,
    #[serde(default)]
    pub last_reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CommitteePanel {
    pub active_personas: usize,
    #[serde(default)]
    pub member_views: Vec<CommitteeMemberView>,
    #[serde(default)]
    pub disagreement_score: Option<f64>,
    #[serde(default)]
    pub groupthink_risk: Option<f64>,
    #[serde(default)]
    pub conflict_status: Option<String>,
    #[serde(default)]
    pub evidence_quality_status: Option<String>,
    pub recommendation: String,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl CommitteePanel {
    pub fn stabilize(&mut self) {
        self.member_views
            .sort_by(|left, right| left.persona_id.cmp(&right.persona_id));
        for member in &mut self.member_views {
            member.last_reason_codes = stable_reason_codes(&member.last_reason_codes);
            member.conviction = member.conviction.map(|value| value.clamp(0.0, 1.0));
            member.voice_power = member.voice_power.map(|value| value.clamp(0.0, 1.0));
        }
        self.active_personas = self.member_views.len();
        self.reason_codes = stable_reason_codes(&self.reason_codes);
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ChairPanel {
    #[serde(default)]
    pub last_decision_id: Option<String>,
    #[serde(default)]
    pub selected_speakers: Vec<String>,
    #[serde(default)]
    pub filtered_speakers: Vec<String>,
    #[serde(default)]
    pub weighted_score: Option<f64>,
    #[serde(default)]
    pub uncertainty: Option<f64>,
    pub final_decision: String,
    pub groupthink_warning: bool,
    pub human_confirm_required: bool,
    #[serde(default)]
    pub chair_reason_codes: Vec<ReasonCode>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl ChairPanel {
    pub fn stabilize(&mut self) {
        self.selected_speakers.sort();
        self.selected_speakers.dedup();
        self.filtered_speakers.sort();
        self.filtered_speakers.dedup();
        self.chair_reason_codes = stable_reason_codes(&self.chair_reason_codes);
        self.reason_codes = stable_reason_codes(&self.reason_codes);
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RiskPanel {
    pub risk_governor_mode: String,
    pub default_deny_active: bool,
    pub emergency_stop_active: bool,
    pub cooldown_active: bool,
    #[serde(default)]
    pub last_risk_decision: Option<String>,
    #[serde(default)]
    pub last_denial_reason_codes: Vec<ReasonCode>,
    pub denied_count: usize,
    pub approved_paper_count: usize,
    pub no_trade_count: usize,
    pub human_confirm_count: usize,
    #[serde(default)]
    pub risk_value_summary: Option<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl RiskPanel {
    pub fn stabilize(&mut self) {
        self.last_denial_reason_codes = stable_reason_codes(&self.last_denial_reason_codes);
        self.reason_codes = stable_reason_codes(&self.reason_codes);
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum CandidateStatus {
    #[default]
    Detected,
    UnderAnalysis,
    CommitteeVoting,
    ChairReviewed,
    RiskReview,
    Candidate,
    HumanConfirmRequired,
    PaperApproved,
    PaperPositionOpen,
    PaperClosed,
    NoTrade,
    RiskBlocked,
    Expired,
    DiagnosticOnly,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CandidateView {
    pub candidate_id: String,
    pub symbol: String,
    pub market: String,
    pub source_kind: String,
    pub provider_kind: String,
    pub timeframe: String,
    pub horizon_bars: u32,
    pub status: CandidateStatus,
    pub signal_summary: String,
    pub committee_summary: String,
    pub chair_summary: String,
    pub risk_summary: String,
    #[serde(default)]
    pub expected_edge: Option<f64>,
    #[serde(default)]
    pub expected_drawdown: Option<f64>,
    #[serde(default)]
    pub data_quality_score: Option<f64>,
    #[serde(default)]
    pub created_from_report: Option<String>,
    #[serde(default)]
    pub expires_at: Option<u64>,
    #[serde(default)]
    pub owner_feedback_history: Vec<OwnerCandidateFeedback>,
    #[serde(default)]
    pub owner_hold_active: bool,
    #[serde(default)]
    pub owner_dismissed: bool,
    #[serde(default)]
    pub owner_reanalysis_requested: bool,
    #[serde(default)]
    pub owner_paper_confirmed: bool,
    #[serde(default)]
    pub linked_thesis_notes: Vec<OwnerThesisNote>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CandidatePanel {
    #[serde(default)]
    pub candidates: Vec<CandidateView>,
    pub active_candidates: usize,
    pub blocked_candidates: usize,
    pub human_confirm_candidates: usize,
    pub paper_approved_candidates: usize,
    pub expired_candidates: usize,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl CandidatePanel {
    pub fn stabilize(&mut self) {
        self.candidates
            .sort_by(|left, right| left.candidate_id.cmp(&right.candidate_id));
        for candidate in &mut self.candidates {
            candidate
                .owner_feedback_history
                .sort_by(|left, right| left.feedback_id.cmp(&right.feedback_id));
            for feedback in &mut candidate.owner_feedback_history {
                feedback.stabilize();
            }
            candidate
                .linked_thesis_notes
                .sort_by(|left, right| left.thesis_id.cmp(&right.thesis_id));
            for note in &mut candidate.linked_thesis_notes {
                note.stabilize();
            }
            candidate.reason_codes = stable_reason_codes(&candidate.reason_codes);
        }
        self.active_candidates = self
            .candidates
            .iter()
            .filter(|candidate| {
                matches!(
                    candidate.status,
                    CandidateStatus::Detected
                        | CandidateStatus::UnderAnalysis
                        | CandidateStatus::CommitteeVoting
                        | CandidateStatus::ChairReviewed
                        | CandidateStatus::RiskReview
                        | CandidateStatus::Candidate
                        | CandidateStatus::PaperPositionOpen
                )
            })
            .count();
        self.blocked_candidates = self
            .candidates
            .iter()
            .filter(|candidate| matches!(candidate.status, CandidateStatus::RiskBlocked))
            .count();
        self.human_confirm_candidates = self
            .candidates
            .iter()
            .filter(|candidate| matches!(candidate.status, CandidateStatus::HumanConfirmRequired))
            .count();
        self.paper_approved_candidates = self
            .candidates
            .iter()
            .filter(|candidate| matches!(candidate.status, CandidateStatus::PaperApproved))
            .count();
        self.expired_candidates = self
            .candidates
            .iter()
            .filter(|candidate| matches!(candidate.status, CandidateStatus::Expired))
            .count();
        self.reason_codes = stable_reason_codes(&self.reason_codes);
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaperPositionSide {
    #[default]
    Long,
    Short,
    Flat,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaperPositionStatus {
    #[default]
    Open,
    Closed,
    Stopped,
    TargetHit,
    Expired,
    RiskClosed,
    DiagnosticOnly,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PaperPositionView {
    pub paper_position_id: String,
    pub candidate_id: String,
    pub symbol: String,
    pub market: String,
    pub side: PaperPositionSide,
    #[serde(default)]
    pub entry_timestamp_ms: Option<u64>,
    #[serde(default)]
    pub entry_price: Option<f64>,
    #[serde(default)]
    pub stop_price: Option<f64>,
    #[serde(default)]
    pub target_price: Option<f64>,
    #[serde(default)]
    pub current_price: Option<f64>,
    #[serde(default)]
    pub unrealized_return_pct: Option<f64>,
    #[serde(default)]
    pub realized_return_pct: Option<f64>,
    pub status: PaperPositionStatus,
    pub source_kind: String,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PaperPositionPanel {
    #[serde(default)]
    pub open_positions: Vec<PaperPositionView>,
    #[serde(default)]
    pub closed_positions: Vec<PaperPositionView>,
    #[serde(default)]
    pub risk_closed_positions: Vec<PaperPositionView>,
    #[serde(default)]
    pub diagnostic_positions: Vec<PaperPositionView>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl PaperPositionPanel {
    pub fn stabilize(&mut self) {
        for positions in [
            &mut self.open_positions,
            &mut self.closed_positions,
            &mut self.risk_closed_positions,
            &mut self.diagnostic_positions,
        ] {
            positions.sort_by(|left, right| left.paper_position_id.cmp(&right.paper_position_id));
            for position in positions.iter_mut() {
                position.reason_codes = stable_reason_codes(&position.reason_codes);
            }
        }
        self.reason_codes = stable_reason_codes(&self.reason_codes);
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum HumanConfirmRequiredBy {
    #[default]
    Chair,
    RiskGovernor,
    LowConfidence,
    HighDisagreement,
    ResearchOnly,
    DiagnosticOnly,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum HumanConfirmSafeAction {
    #[default]
    ViewOnly,
    MarkReviewed,
    Defer,
    DismissCandidate,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum HumanConfirmForbiddenAction {
    #[default]
    ExecuteOrder,
    PlaceTrade,
    ModifyAccount,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HumanConfirmItem {
    pub confirm_id: String,
    pub candidate_id: String,
    pub reason: String,
    pub required_by: HumanConfirmRequiredBy,
    #[serde(default)]
    pub paper_confirm_allowed: bool,
    #[serde(default)]
    pub paper_confirm_explanation: String,
    #[serde(default)]
    pub safe_actions: Vec<HumanConfirmSafeAction>,
    #[serde(default)]
    pub allowed_owner_actions: Vec<AllowedOwnerAction>,
    #[serde(default)]
    pub forbidden_actions: Vec<HumanConfirmForbiddenAction>,
    #[serde(default)]
    pub forbidden_owner_actions: Vec<ForbiddenOwnerAction>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HumanConfirmPanel {
    #[serde(default)]
    pub pending_items: Vec<HumanConfirmItem>,
    #[serde(default)]
    pub reviewed_items: Vec<HumanConfirmItem>,
    #[serde(default)]
    pub deferred_items: Vec<HumanConfirmItem>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl HumanConfirmPanel {
    pub fn stabilize(&mut self) {
        for items in [
            &mut self.pending_items,
            &mut self.reviewed_items,
            &mut self.deferred_items,
        ] {
            items.sort_by(|left, right| left.confirm_id.cmp(&right.confirm_id));
            for item in items.iter_mut() {
                item.safe_actions.sort_by_key(|value| format!("{value:?}"));
                item.safe_actions.dedup();
                item.allowed_owner_actions
                    .sort_by_key(|value| format!("{value:?}"));
                item.allowed_owner_actions.dedup();
                item.forbidden_actions
                    .sort_by_key(|value| format!("{value:?}"));
                item.forbidden_actions.dedup();
                item.forbidden_owner_actions
                    .sort_by_key(|value| format!("{value:?}"));
                item.forbidden_owner_actions.dedup();
                item.reason_codes = stable_reason_codes(&item.reason_codes);
            }
        }
        self.reason_codes = stable_reason_codes(&self.reason_codes);
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BottleneckPanel {
    pub primary_bottleneck: String,
    #[serde(default)]
    pub secondary_bottlenecks: Vec<String>,
    pub core_status: String,
    pub evidence_status: String,
    pub risk_status: String,
    pub committee_status: String,
    pub next_action: String,
    #[serde(default)]
    pub operator_actions: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl BottleneckPanel {
    pub fn stabilize(&mut self) {
        self.secondary_bottlenecks.sort();
        self.secondary_bottlenecks.dedup();
        self.operator_actions.sort();
        self.operator_actions.dedup();
        self.reason_codes = stable_reason_codes(&self.reason_codes);
    }
}
