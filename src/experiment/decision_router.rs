use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;

use super::ablation::{AblationDimension, AblationStudyReport};
use super::next_step::NextStepRecommendation;
use super::readiness::ExpansionReadinessDecision;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Sprint14Track {
    ImproveDataFirst,
    ImproveFeaturesFirst,
    ImproveRiskGovernorFirst,
    ImproveRegimeClassifierFirst,
    ImproveSignalModelFirst,
    NeedMoreExperiments,
    HoldCurrentScope,
    ReadyForSixPersonaDesignReview,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Sprint14EvidenceInput {
    pub source_study_id: Option<String>,
    pub source_report_path: Option<String>,
    pub sprint13_next_step: Option<NextStepRecommendation>,
    pub dominant_dimension: Option<AblationDimension>,
    pub dataset_count: Option<usize>,
    pub usable_dataset_count: Option<usize>,
    pub total_outcome_records: Option<usize>,
    pub regime_coverage_count: Option<usize>,
    pub comparable_variant_count: Option<usize>,
    pub average_data_quality_score: Option<f64>,
    pub baseline_failed_runs: Option<usize>,
    pub expansion_readiness_decision: Option<ExpansionReadinessDecision>,
    pub warnings: Vec<String>,
    pub blockers: Vec<String>,
}

impl Sprint14EvidenceInput {
    pub fn from_ablation_report(
        report: &AblationStudyReport,
        source_report_path: Option<String>,
    ) -> Self {
        let comparable_variant_count = report
            .variants
            .iter()
            .filter(|variant| {
                !matches!(
                    variant.status,
                    super::ablation::AblationResultStatus::Skipped
                )
            })
            .count();
        let baseline_readiness = &report.baseline.report.expansion_readiness;
        Self {
            source_study_id: Some(report.study_id.clone()),
            source_report_path,
            sprint13_next_step: Some(report.next_step),
            dominant_dimension: report.sensitivity_summary.dominant_dimension,
            dataset_count: Some(baseline_readiness.evidence.dataset_count),
            usable_dataset_count: Some(baseline_readiness.evidence.usable_dataset_count),
            total_outcome_records: Some(baseline_readiness.evidence.total_outcome_records),
            regime_coverage_count: Some(0),
            comparable_variant_count: Some(comparable_variant_count),
            average_data_quality_score: Some(
                report
                    .baseline
                    .report
                    .aggregate_benchmark
                    .avg_data_quality_score,
            ),
            baseline_failed_runs: Some(report.baseline.report.aggregate_benchmark.failed_runs),
            expansion_readiness_decision: Some(baseline_readiness.decision),
            warnings: {
                let mut warnings = report.warnings.clone();
                warnings.extend(baseline_readiness.warnings.iter().cloned());
                warnings
            },
            blockers: baseline_readiness.blockers.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Sprint14RejectedTrack {
    pub track: Sprint14Track,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Sprint14DecisionRecord {
    pub selected_track: Sprint14Track,
    pub reason: String,
    pub evidence_inputs: Sprint14EvidenceInput,
    pub rejected_tracks: Vec<Sprint14RejectedTrack>,
    pub blockers: Vec<String>,
    pub warnings: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Sprint14DecisionRouter;

impl Sprint14DecisionRouter {
    pub fn decide(&self, input: Option<&Sprint14EvidenceInput>) -> Sprint14DecisionRecord {
        let Some(input) = input.cloned() else {
            return Sprint14DecisionRecord {
                selected_track: Sprint14Track::NeedMoreExperiments,
                reason: "missing Sprint 13 ablation evidence".to_string(),
                evidence_inputs: Sprint14EvidenceInput {
                    source_study_id: None,
                    source_report_path: None,
                    sprint13_next_step: None,
                    dominant_dimension: None,
                    dataset_count: None,
                    usable_dataset_count: None,
                    total_outcome_records: None,
                    regime_coverage_count: None,
                    comparable_variant_count: None,
                    average_data_quality_score: None,
                    baseline_failed_runs: None,
                    expansion_readiness_decision: None,
                    warnings: Vec::new(),
                    blockers: Vec::new(),
                },
                rejected_tracks: Vec::new(),
                blockers: vec!["missing ablation report".to_string()],
                warnings: vec!["defaulting to NeedMoreExperiments".to_string()],
                reason_codes: vec![
                    ReasonCode::ComparisonNotConclusive,
                    ReasonCode::Sprint14DecisionBuilt,
                ],
            };
        };

        let candidates = derive_candidates(&input);
        let selected = candidates.first().cloned().unwrap_or((
            Sprint14Track::NeedMoreExperiments,
            "no dominant or sufficient evidence was available".to_string(),
        ));
        let rejected_tracks = candidates
            .iter()
            .skip(1)
            .map(|(track, reason)| Sprint14RejectedTrack {
                track: *track,
                reason: reason.clone(),
            })
            .collect::<Vec<_>>();
        let mut reason_codes = vec![ReasonCode::Sprint14DecisionBuilt];
        if selected.0 == Sprint14Track::NeedMoreExperiments {
            reason_codes.push(ReasonCode::EvidenceGapDetected);
            reason_codes.push(ReasonCode::ComparisonNotConclusive);
        }
        Sprint14DecisionRecord {
            selected_track: selected.0,
            reason: selected.1,
            blockers: input.blockers.clone(),
            warnings: input.warnings.clone(),
            evidence_inputs: input,
            rejected_tracks,
            reason_codes,
        }
    }
}

fn derive_candidates(input: &Sprint14EvidenceInput) -> Vec<(Sprint14Track, String)> {
    let mut candidates = Vec::<(Sprint14Track, String)>::new();
    let missing_critical = input.sprint13_next_step.is_none()
        || input.dataset_count.is_none()
        || input.total_outcome_records.is_none()
        || input.comparable_variant_count.is_none();
    if missing_critical {
        candidates.push((
            Sprint14Track::NeedMoreExperiments,
            "Sprint 13 evidence is incomplete or ambiguous".to_string(),
        ));
    }

    if input
        .average_data_quality_score
        .map(|score| score < 0.80)
        .unwrap_or(false)
        || input
            .warnings
            .iter()
            .any(|warning| warning.contains("data quality"))
    {
        candidates.push((
            Sprint14Track::ImproveDataFirst,
            "data quality evidence is below the conservative gate".to_string(),
        ));
    }

    match input.sprint13_next_step {
        Some(NextStepRecommendation::ImproveDataFirst) => candidates.push((
            Sprint14Track::ImproveDataFirst,
            "Sprint 13 next-step recommendation prioritized data improvement".to_string(),
        )),
        Some(NextStepRecommendation::TightenRiskGates) => candidates.push((
            Sprint14Track::ImproveRiskGovernorFirst,
            "Sprint 13 next-step recommendation prioritized tighter risk controls".to_string(),
        )),
        Some(NextStepRecommendation::RefineFeatureSet) => candidates.push((
            Sprint14Track::ImproveFeaturesFirst,
            "Sprint 13 next-step recommendation prioritized feature refinement".to_string(),
        )),
        Some(NextStepRecommendation::RevisitLabels) => candidates.push((
            Sprint14Track::ImproveSignalModelFirst,
            "Sprint 13 label sensitivity indicates signal evaluation remains under-specified"
                .to_string(),
        )),
        Some(NextStepRecommendation::NeedMoreExperiments) => candidates.push((
            Sprint14Track::NeedMoreExperiments,
            "Sprint 13 next-step recommendation explicitly requires more experiments".to_string(),
        )),
        None => {}
    }

    if input.dominant_dimension == Some(AblationDimension::Regime) {
        candidates.push((
            Sprint14Track::ImproveRegimeClassifierFirst,
            "dominant sensitivity dimension is regime".to_string(),
        ));
    }

    if input.expansion_readiness_decision == Some(ExpansionReadinessDecision::HoldCurrentScope) {
        candidates.push((
            Sprint14Track::HoldCurrentScope,
            "baseline readiness already recommends holding current scope".to_string(),
        ));
    }

    let insufficient_evidence = input.dataset_count.unwrap_or(0) < 3
        || input.usable_dataset_count.unwrap_or(0) < 3
        || input.total_outcome_records.unwrap_or(0) < 20
        || input.comparable_variant_count.unwrap_or(0) == 0;
    if insufficient_evidence {
        candidates.push((
            Sprint14Track::NeedMoreExperiments,
            "dataset, outcome, or comparable ablation evidence is still insufficient".to_string(),
        ));
    }

    if input.blockers.is_empty()
        && !insufficient_evidence
        && input.average_data_quality_score.unwrap_or(0.0) >= 0.90
        && input.expansion_readiness_decision
            != Some(ExpansionReadinessDecision::NeedMoreExperiments)
    {
        candidates.push((
            Sprint14Track::ReadyForSixPersonaDesignReview,
            "conservative design-review-only gates look satisfied".to_string(),
        ));
    }

    candidates.sort_by(|left, right| {
        priority(left.0)
            .cmp(&priority(right.0))
            .then_with(|| left.1.cmp(&right.1))
    });
    candidates.dedup_by(|left, right| left.0 == right.0);
    candidates
}

fn priority(track: Sprint14Track) -> usize {
    match track {
        Sprint14Track::ImproveDataFirst => 0,
        Sprint14Track::ImproveRiskGovernorFirst => 1,
        Sprint14Track::ImproveRegimeClassifierFirst => 2,
        Sprint14Track::ImproveFeaturesFirst => 3,
        Sprint14Track::ImproveSignalModelFirst => 4,
        Sprint14Track::NeedMoreExperiments => 5,
        Sprint14Track::HoldCurrentScope => 6,
        Sprint14Track::ReadyForSixPersonaDesignReview => 7,
    }
}
