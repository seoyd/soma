use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};

use super::{
    control_tower_v1::ControlTowerRefreshPlanner,
    dashboard_panels::{CandidatePanel, EvidencePanel, RiskPanel},
    kis_monitor_panel::KISMonitorPanel,
    next_action_panel::NextActionPanel,
    owner_panel::OwnerPanel,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControlTowerHealthStatus {
    #[default]
    HealthyResearchMonitor,
    NeedKISData,
    NeedEvidenceDepth,
    NeedOwnerReview,
    RiskBlockedDominant,
    UIDataStale,
    MissingReports,
    SecretRedactionFailed,
    DiagnosticOnly,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlTowerHealthSummary {
    pub health_status: ControlTowerHealthStatus,
    pub provider_health: String,
    pub evidence_health: String,
    pub committee_health: String,
    pub risk_health: String,
    pub owner_review_health: String,
    pub ui_health: String,
    pub current_primary_bottleneck: String,
    pub current_primary_next_action: String,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl ControlTowerHealthSummary {
    pub fn stabilize(&mut self) {
        self.reason_codes = stable_reason_codes(&self.reason_codes);
    }
}

pub fn summarize_control_tower_health(
    kis_monitor_panel: &KISMonitorPanel,
    evidence_panel: &EvidencePanel,
    owner_panel: &OwnerPanel,
    risk_panel: &RiskPanel,
    candidate_panel: &CandidatePanel,
    next_action_panel: &NextActionPanel,
    refresh_planner: &ControlTowerRefreshPlanner,
    warnings: &[String],
    blockers: &[String],
    redaction_passed: bool,
) -> ControlTowerHealthSummary {
    let mut reason_codes = Vec::new();
    let health_status = if !redaction_passed || contains_secret_like_material(warnings, blockers) {
        reason_codes.push(ReasonCode::DashboardSecretRejected);
        ControlTowerHealthStatus::SecretRedactionFailed
    } else if warnings
        .iter()
        .any(|warning| warning.to_ascii_lowercase().contains("missing"))
        || blockers
            .iter()
            .any(|blocker| blocker.to_ascii_lowercase().contains("missing"))
    {
        reason_codes.push(ReasonCode::DashboardReportMissing);
        ControlTowerHealthStatus::MissingReports
    } else if !kis_monitor_panel.auth_ready
        || !kis_monitor_panel.base_url_ready
        || kis_monitor_panel.official_row_count == 0
    {
        reason_codes.push(ReasonCode::MissingAuth);
        ControlTowerHealthStatus::NeedKISData
    } else if evidence_panel.sufficiency_status.contains("Need")
        || kis_monitor_panel.outcome_links == 0
        || kis_monitor_panel.complete_rows == 0
    {
        reason_codes.push(ReasonCode::EvidenceStillInsufficient);
        ControlTowerHealthStatus::NeedEvidenceDepth
    } else if !owner_panel.pending_review_items.is_empty() {
        reason_codes.push(ReasonCode::OwnerReviewQueueBuilt);
        ControlTowerHealthStatus::NeedOwnerReview
    } else if candidate_panel.blocked_candidates >= candidate_panel.active_candidates
        && candidate_panel.blocked_candidates > 0
    {
        reason_codes.push(ReasonCode::RiskDenied);
        ControlTowerHealthStatus::RiskBlockedDominant
    } else if refresh_planner.missing_outputs > 0 {
        reason_codes.push(ReasonCode::DashboardReportMissing);
        ControlTowerHealthStatus::UIDataStale
    } else if kis_monitor_panel
        .collection_plan_status
        .to_ascii_lowercase()
        .contains("diagnostic")
    {
        reason_codes.push(ReasonCode::ResearchOnlyOverride);
        ControlTowerHealthStatus::DiagnosticOnly
    } else {
        reason_codes.push(ReasonCode::DeterministicPath);
        ControlTowerHealthStatus::HealthyResearchMonitor
    };

    let mut summary = ControlTowerHealthSummary {
        health_status,
        provider_health: format!(
            "auth_ready={} endpoint_policy={}",
            kis_monitor_panel.auth_ready, kis_monitor_panel.endpoint_policy_status
        ),
        evidence_health: format!(
            "official_rows={} outcome_links={} sufficiency={}",
            kis_monitor_panel.official_row_count,
            kis_monitor_panel.outcome_links,
            evidence_panel.sufficiency_status
        ),
        committee_health: format!("active_candidates={}", candidate_panel.active_candidates),
        risk_health: format!(
            "denied={} approved_paper={} no_trade={}",
            risk_panel.denied_count, risk_panel.approved_paper_count, risk_panel.no_trade_count
        ),
        owner_review_health: format!(
            "pending_reviews={} blocked_inputs={}",
            owner_panel.pending_review_items.len(),
            owner_panel.blocked_owner_inputs.len()
        ),
        ui_health: format!(
            "missing_outputs={} refresh_mode={:?}",
            refresh_planner.missing_outputs, refresh_planner.watcher_mode
        ),
        current_primary_bottleneck: kis_monitor_panel
            .core_bottleneck
            .clone()
            .unwrap_or_else(|| evidence_panel.current_bottleneck.clone()),
        current_primary_next_action: format!(
            "{:?}",
            next_action_panel.primary_next_action.action_kind
        ),
        reason_codes,
    };
    summary.stabilize();
    summary
}

fn contains_secret_like_material(warnings: &[String], blockers: &[String]) -> bool {
    warnings.iter().chain(blockers.iter()).any(|text| {
        let normalized = text.to_ascii_lowercase();
        [
            "kis_app_key",
            "kis_app_secret",
            "kis_ws_approval_key",
            "krx_api_key",
            "token=",
            "password=",
            "http://",
            "https://",
        ]
        .iter()
        .any(|needle| normalized.contains(needle))
    })
}
