use serde::{Deserialize, Serialize};

use super::ablation::{AblationDimension, AblationInterpretationFlag, AblationVariantResult};
use super::sensitivity::SensitivitySummary;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NextStepRecommendation {
    ImproveDataFirst,
    TightenRiskGates,
    RefineFeatureSet,
    RevisitLabels,
    NeedMoreExperiments,
}

pub fn select_next_step(
    results: &[AblationVariantResult],
    summary: &SensitivitySummary,
) -> NextStepRecommendation {
    let data_quality_sensitive = results.iter().any(|result| {
        !result
            .flags
            .contains(&AblationInterpretationFlag::NotComparable)
            && matches!(
                result.dimension,
                AblationDimension::FeatureGroup | AblationDimension::RiskGovernor
            )
            && (result
                .overrides
                .iter()
                .any(|override_item| override_item.target == "data_quality")
                || result
                    .overrides
                    .iter()
                    .any(|override_item| override_item.target == "min_data_quality"))
            && result.delta.avg_net_return_pct.abs() >= dominant_threshold(summary)
    });
    if data_quality_sensitive {
        return NextStepRecommendation::ImproveDataFirst;
    }

    let cost_fragile = results.iter().any(|result| {
        matches!(
            result.dimension,
            AblationDimension::CostModel | AblationDimension::NoTradeScoring
        ) && result
            .flags
            .contains(&AblationInterpretationFlag::HighFragility)
    });
    if cost_fragile {
        return NextStepRecommendation::TightenRiskGates;
    }

    let stable_feature_gain = results.iter().any(|result| {
        result.dimension == AblationDimension::FeatureGroup
            && result
                .flags
                .contains(&AblationInterpretationFlag::CandidateImprovement)
            && !result
                .flags
                .contains(&AblationInterpretationFlag::ResearchOnly)
    });
    if stable_feature_gain {
        return NextStepRecommendation::RefineFeatureSet;
    }

    let robust_label_change = results.iter().any(|result| {
        result.dimension == AblationDimension::TripleBarrier
            && !result
                .flags
                .contains(&AblationInterpretationFlag::NotComparable)
            && !result
                .flags
                .contains(&AblationInterpretationFlag::WorseDrawdown)
            && !result
                .flags
                .contains(&AblationInterpretationFlag::WorseCalibration)
    });
    if robust_label_change {
        return NextStepRecommendation::RevisitLabels;
    }

    NextStepRecommendation::NeedMoreExperiments
}

fn dominant_threshold(summary: &SensitivitySummary) -> f64 {
    summary
        .dimensions
        .first()
        .map(|dimension| (dimension.max_abs_avg_net_return_delta * 0.5).max(0.001))
        .unwrap_or(0.001)
}
