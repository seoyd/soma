use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;
use crate::risk::GovernorConfig;

use super::committee_decision_quality::CommitteeDecisionQualityReport;
use super::committee_evidence_quality::{
    CommitteeEvidenceQualityReport, CommitteeEvidenceQualityStatus,
};
use super::risk_bridge_diagnostics::{RiskBridgeDiagnosticStatus, RiskBridgeDiagnosticsReport};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskCalibrationDirection {
    Tighten,
    LoosenResearchOnly,
    Keep,
    NeedsMoreEvidence,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskCalibrationSafetyImpact {
    Conservative,
    Neutral,
    PotentiallyRiskier,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskCalibrationArea {
    DataQuality,
    NegativeEdge,
    MissingStop,
    Cooldown,
    EmergencyStop,
    SpreadLiquidity,
    ConfidenceFloor,
    ExpectedDrawdown,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RiskCalibrationSuggestion {
    pub suggestion_id: String,
    pub risk_area: RiskCalibrationArea,
    pub suggested_direction: RiskCalibrationDirection,
    pub rationale: String,
    pub safety_impact: RiskCalibrationSafetyImpact,
    pub hard_veto_affected: bool,
    pub apply_automatically: bool,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskCalibrationRecommendation {
    KeepRiskGovernor,
    ImproveRiskDiagnostics,
    TightenRiskRules,
    ResearchOnlyReviewForOverblocking,
    NeedMoreEvidence,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RiskCalibrationReport {
    pub suggestions: Vec<RiskCalibrationSuggestion>,
    pub overblocking_suspected: bool,
    pub underblocking_suspected: bool,
    pub final_recommendation: RiskCalibrationRecommendation,
    pub reason_codes: Vec<ReasonCode>,
}

pub fn build_risk_calibration_report(
    reports: &[RiskBridgeDiagnosticsReport],
    evidence_quality_report: &CommitteeEvidenceQualityReport,
    decision_quality_report: &CommitteeDecisionQualityReport,
) -> RiskCalibrationReport {
    let weak_evidence = matches!(
        evidence_quality_report.quality_status,
        CommitteeEvidenceQualityStatus::FixtureOnlyEvidence
            | CommitteeEvidenceQualityStatus::ResearchOnlyEvidence
            | CommitteeEvidenceQualityStatus::InsufficientEvidence
            | CommitteeEvidenceQualityStatus::LowQualityEvidence
    ) || reports.len() < 3;
    if weak_evidence {
        let suggestion = suggestion(
            "risk-need-more-evidence",
            RiskCalibrationArea::ConfidenceFloor,
            RiskCalibrationDirection::NeedsMoreEvidence,
            "weak evidence cannot support safe risk-threshold tuning",
            RiskCalibrationSafetyImpact::Conservative,
            false,
        );
        return RiskCalibrationReport {
            suggestions: vec![suggestion],
            overblocking_suspected: false,
            underblocking_suspected: false,
            final_recommendation: RiskCalibrationRecommendation::NeedMoreEvidence,
            reason_codes: vec![ReasonCode::RiskCalibrationBuilt],
        };
    }

    let governor = GovernorConfig::default();
    let deny_count = reports.iter().filter(|report| report.veto_applied).count();
    let all_denied = deny_count == reports.len() && !reports.is_empty();
    let soft_threshold_denials = reports
        .iter()
        .filter(|report| {
            report.veto_applied
                && !report.emergency_stop_triggered
                && !report.cooldown_triggered
                && !report.data_quality_block
                && !report.negative_edge_block
                && !report.schema_mismatch_block
                && !report.invalid_prediction_block
        })
        .count();
    let overblocking_suspected = all_denied
        && soft_threshold_denials == reports.len()
        && decision_quality_report.approve_candidate_ratio > 0.0;
    let underblocking_suspected = reports.iter().any(|report| {
        report.diagnostic_status == RiskBridgeDiagnosticStatus::RiskPassed
            && (report.invalid_prediction_block || report.schema_mismatch_block)
    });

    let mut suggestions = Vec::new();
    if reports.iter().any(|report| report.data_quality_block) {
        suggestions.push(suggestion(
            "risk-data-quality-keep",
            RiskCalibrationArea::DataQuality,
            RiskCalibrationDirection::Keep,
            "data-quality denials are still valid hard safety boundaries",
            RiskCalibrationSafetyImpact::Conservative,
            true,
        ));
    }
    if reports.iter().any(|report| report.negative_edge_block) {
        suggestions.push(suggestion(
            "risk-negative-edge-keep",
            RiskCalibrationArea::NegativeEdge,
            RiskCalibrationDirection::Keep,
            "negative-edge denials remain a valid veto and should not be loosened",
            RiskCalibrationSafetyImpact::Conservative,
            true,
        ));
    }
    if overblocking_suspected {
        suggestions.push(suggestion(
            "risk-confidence-floor-review",
            RiskCalibrationArea::ConfidenceFloor,
            RiskCalibrationDirection::LoosenResearchOnly,
            &format!(
                "all committee candidates were denied by soft thresholds; review min_confidence {:.2} in sandbox only",
                governor.min_confidence
            ),
            RiskCalibrationSafetyImpact::PotentiallyRiskier,
            false,
        ));
        suggestions.push(suggestion(
            "risk-spread-liquidity-review",
            RiskCalibrationArea::SpreadLiquidity,
            RiskCalibrationDirection::LoosenResearchOnly,
            "spread/liquidity gating may be overly strict for research-only replay cases",
            RiskCalibrationSafetyImpact::PotentiallyRiskier,
            false,
        ));
    }
    if underblocking_suspected {
        suggestions.push(suggestion(
            "risk-tighten-schema",
            RiskCalibrationArea::MissingStop,
            RiskCalibrationDirection::Tighten,
            "approval with schema or invalid-prediction warnings suggests tighter validation review",
            RiskCalibrationSafetyImpact::Conservative,
            true,
        ));
    }
    if reports
        .iter()
        .filter(|report| {
            matches!(
                report.diagnostic_status,
                RiskBridgeDiagnosticStatus::EmergencyStop | RiskBridgeDiagnosticStatus::Cooldown
            )
        })
        .count()
        * 2
        >= reports.len().max(1)
    {
        suggestions.push(suggestion(
            "risk-drawdown-tighten",
            RiskCalibrationArea::ExpectedDrawdown,
            RiskCalibrationDirection::Tighten,
            "frequent cooldown or emergency states suggest unstable risk conditions",
            RiskCalibrationSafetyImpact::Conservative,
            false,
        ));
    }
    suggestions.sort_by(|left, right| left.suggestion_id.cmp(&right.suggestion_id));

    let final_recommendation = if overblocking_suspected {
        RiskCalibrationRecommendation::ResearchOnlyReviewForOverblocking
    } else if underblocking_suspected {
        RiskCalibrationRecommendation::TightenRiskRules
    } else if suggestions.is_empty() {
        RiskCalibrationRecommendation::KeepRiskGovernor
    } else {
        RiskCalibrationRecommendation::ImproveRiskDiagnostics
    };

    RiskCalibrationReport {
        suggestions,
        overblocking_suspected,
        underblocking_suspected,
        final_recommendation,
        reason_codes: vec![ReasonCode::RiskCalibrationBuilt],
    }
}

impl RiskCalibrationReport {
    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("overblocking_suspected={}", self.overblocking_suspected),
            format!("underblocking_suspected={}", self.underblocking_suspected),
            format!("final_recommendation={:?}", self.final_recommendation),
        ];
        for suggestion in &self.suggestions {
            lines.push(format!(
                "suggestion={};risk_area={:?};direction={:?};impact={:?};hard_veto_affected={};auto_apply={}",
                suggestion.suggestion_id,
                suggestion.risk_area,
                suggestion.suggested_direction,
                suggestion.safety_impact,
                suggestion.hard_veto_affected,
                suggestion.apply_automatically
            ));
        }
        lines.join("\n")
    }
}

fn suggestion(
    suggestion_id: &str,
    risk_area: RiskCalibrationArea,
    suggested_direction: RiskCalibrationDirection,
    rationale: &str,
    safety_impact: RiskCalibrationSafetyImpact,
    hard_veto_affected: bool,
) -> RiskCalibrationSuggestion {
    RiskCalibrationSuggestion {
        suggestion_id: suggestion_id.to_string(),
        risk_area,
        suggested_direction,
        rationale: rationale.to_string(),
        safety_impact,
        hard_veto_affected,
        apply_automatically: false,
        reason_codes: vec![ReasonCode::RiskCalibrationBuilt],
    }
}
