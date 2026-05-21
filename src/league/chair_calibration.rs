use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;

use super::chair_diagnostics::{ChairDiagnosticStatus, ChairDiagnosticsReport};
use super::committee_decision::ChairCommitteeConfig;
use super::committee_evidence_quality::{
    CommitteeEvidenceQualityReport, CommitteeEvidenceQualityStatus,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CalibrationDirection {
    Increase,
    Decrease,
    Keep,
    NeedsMoreEvidence,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CalibrationSafetyImpact {
    Conservative,
    Neutral,
    PotentiallyRiskier,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChairCalibrationSuggestion {
    pub suggestion_id: String,
    pub parameter_name: String,
    #[serde(default)]
    pub current_value: Option<f64>,
    pub suggested_direction: CalibrationDirection,
    pub rationale: String,
    pub expected_effect: String,
    pub safety_impact: CalibrationSafetyImpact,
    pub apply_automatically: bool,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChairCalibrationRecommendation {
    KeepChairV0,
    TuneThresholds,
    IncreaseContrarianProtection,
    IncreaseNoTradeConservatism,
    ReduceOverFiltering,
    NeedMoreEvidence,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChairCalibrationReport {
    pub suggestions: Vec<ChairCalibrationSuggestion>,
    pub groupthink_suggestions: Vec<ChairCalibrationSuggestion>,
    pub disagreement_suggestions: Vec<ChairCalibrationSuggestion>,
    pub speaker_filter_suggestions: Vec<ChairCalibrationSuggestion>,
    pub cluster_penalty_suggestions: Vec<ChairCalibrationSuggestion>,
    pub contrarian_inclusion_suggestions: Vec<ChairCalibrationSuggestion>,
    pub final_recommendation: ChairCalibrationRecommendation,
    pub reason_codes: Vec<ReasonCode>,
}

pub fn build_chair_calibration_report(
    reports: &[ChairDiagnosticsReport],
    evidence_quality_report: &CommitteeEvidenceQualityReport,
) -> ChairCalibrationReport {
    let config = ChairCommitteeConfig::default();
    let weak_evidence = matches!(
        evidence_quality_report.quality_status,
        CommitteeEvidenceQualityStatus::FixtureOnlyEvidence
            | CommitteeEvidenceQualityStatus::ResearchOnlyEvidence
            | CommitteeEvidenceQualityStatus::InsufficientEvidence
            | CommitteeEvidenceQualityStatus::LowQualityEvidence
    ) || reports.len() < 3;
    if weak_evidence {
        let suggestion = suggestion(
            "chair-need-more-evidence",
            "groupthink_warning_threshold",
            Some(config.groupthink_warning_threshold),
            CalibrationDirection::NeedsMoreEvidence,
            "evidence is too weak to tune Chair thresholds safely",
            "keep ChairV0 defaults and gather more official committee evidence first",
            CalibrationSafetyImpact::Conservative,
        );
        return ChairCalibrationReport {
            suggestions: vec![suggestion.clone()],
            groupthink_suggestions: vec![suggestion.clone()],
            disagreement_suggestions: Vec::new(),
            speaker_filter_suggestions: Vec::new(),
            cluster_penalty_suggestions: Vec::new(),
            contrarian_inclusion_suggestions: Vec::new(),
            final_recommendation: ChairCalibrationRecommendation::NeedMoreEvidence,
            reason_codes: vec![ReasonCode::ChairCalibrationBuilt],
        };
    }

    let groupthink_count = reports
        .iter()
        .filter(|report| report.groupthink_risk >= config.groupthink_warning_threshold)
        .count();
    let disagreement_count = reports
        .iter()
        .filter(|report| report.diagnostic_status == ChairDiagnosticStatus::ExcessiveDisagreement)
        .count();
    let overfiltered_count = reports
        .iter()
        .filter(|report| {
            matches!(
                report.diagnostic_status,
                ChairDiagnosticStatus::OverFiltered | ChairDiagnosticStatus::TooFewSpeakers
            )
        })
        .count();

    let mut groupthink_suggestions = Vec::new();
    let mut disagreement_suggestions = Vec::new();
    let mut speaker_filter_suggestions = Vec::new();
    let mut cluster_penalty_suggestions = Vec::new();
    let mut contrarian_inclusion_suggestions = Vec::new();

    if ratio(groupthink_count, reports.len()) >= 0.50 {
        groupthink_suggestions.push(suggestion(
            "chair-groupthink-threshold",
            "groupthink_warning_threshold",
            Some(config.groupthink_warning_threshold),
            CalibrationDirection::Decrease,
            "groupthink warnings appear frequently across replayed committee decisions",
            "warn earlier and surface clustering risk sooner",
            CalibrationSafetyImpact::Conservative,
        ));
        cluster_penalty_suggestions.push(suggestion(
            "chair-cluster-penalty",
            "cluster_penalty_strength",
            None,
            CalibrationDirection::Increase,
            "aligned speakers dominate too often",
            "reduce cluster dominance without mutating live defaults",
            CalibrationSafetyImpact::Conservative,
        ));
        contrarian_inclusion_suggestions.push(suggestion(
            "chair-contrarian-protection",
            "contrarian_inclusion_requirement",
            Some(if config.require_contrarian { 1.0 } else { 0.0 }),
            CalibrationDirection::Increase,
            "contrarian views need stronger preservation under groupthink conditions",
            "keep dissent visible in research replay",
            CalibrationSafetyImpact::Conservative,
        ));
    }

    if ratio(disagreement_count, reports.len()) >= 0.50 {
        disagreement_suggestions.push(suggestion(
            "chair-no-trade-threshold",
            "no_trade_threshold",
            Some(config.no_trade_threshold),
            CalibrationDirection::Increase,
            "disagreement stays high across too many decisions",
            "push borderline cases toward NoTrade more often",
            CalibrationSafetyImpact::Conservative,
        ));
        disagreement_suggestions.push(suggestion(
            "chair-uncertainty-reduce-threshold",
            "uncertainty_reduce_threshold",
            Some(config.uncertainty_reduce_threshold),
            CalibrationDirection::Decrease,
            "uncertainty is large enough to prefer more ReduceSize outcomes",
            "increase conservative size reductions before approval",
            CalibrationSafetyImpact::Conservative,
        ));
    }

    if ratio(overfiltered_count, reports.len()) >= 0.34 {
        speaker_filter_suggestions.push(suggestion(
            "chair-speaker-filter",
            "speaker_filter_strictness",
            None,
            CalibrationDirection::Decrease,
            "too many candidate speakers are filtered out before selection",
            "allow more diverse committee participation in research review",
            CalibrationSafetyImpact::PotentiallyRiskier,
        ));
    }

    let mut suggestions = Vec::new();
    suggestions.extend(groupthink_suggestions.iter().cloned());
    suggestions.extend(disagreement_suggestions.iter().cloned());
    suggestions.extend(speaker_filter_suggestions.iter().cloned());
    suggestions.extend(cluster_penalty_suggestions.iter().cloned());
    suggestions.extend(contrarian_inclusion_suggestions.iter().cloned());
    suggestions.sort_by(|left, right| left.suggestion_id.cmp(&right.suggestion_id));

    let final_recommendation =
        if !groupthink_suggestions.is_empty() || !contrarian_inclusion_suggestions.is_empty() {
            ChairCalibrationRecommendation::IncreaseContrarianProtection
        } else if !disagreement_suggestions.is_empty() {
            ChairCalibrationRecommendation::IncreaseNoTradeConservatism
        } else if !speaker_filter_suggestions.is_empty() {
            ChairCalibrationRecommendation::ReduceOverFiltering
        } else if !suggestions.is_empty() {
            ChairCalibrationRecommendation::TuneThresholds
        } else {
            ChairCalibrationRecommendation::KeepChairV0
        };

    ChairCalibrationReport {
        suggestions,
        groupthink_suggestions,
        disagreement_suggestions,
        speaker_filter_suggestions,
        cluster_penalty_suggestions,
        contrarian_inclusion_suggestions,
        final_recommendation,
        reason_codes: vec![ReasonCode::ChairCalibrationBuilt],
    }
}

impl ChairCalibrationReport {
    pub fn to_text(&self) -> String {
        let mut lines = vec![format!(
            "final_recommendation={:?}",
            self.final_recommendation
        )];
        for suggestion in &self.suggestions {
            lines.push(format!(
                "suggestion={};parameter={};direction={:?};impact={:?};auto_apply={}",
                suggestion.suggestion_id,
                suggestion.parameter_name,
                suggestion.suggested_direction,
                suggestion.safety_impact,
                suggestion.apply_automatically
            ));
        }
        lines.join("\n")
    }
}

fn suggestion(
    suggestion_id: &str,
    parameter_name: &str,
    current_value: Option<f64>,
    suggested_direction: CalibrationDirection,
    rationale: &str,
    expected_effect: &str,
    safety_impact: CalibrationSafetyImpact,
) -> ChairCalibrationSuggestion {
    ChairCalibrationSuggestion {
        suggestion_id: suggestion_id.to_string(),
        parameter_name: parameter_name.to_string(),
        current_value,
        suggested_direction,
        rationale: rationale.to_string(),
        expected_effect: expected_effect.to_string(),
        safety_impact,
        apply_automatically: false,
        reason_codes: vec![ReasonCode::ChairCalibrationBuilt],
    }
}

fn ratio(count: usize, total: usize) -> f64 {
    count as f64 / total.max(1) as f64
}
