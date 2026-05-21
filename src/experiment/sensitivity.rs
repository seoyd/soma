use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;

use super::ablation::{AblationDimension, AblationInterpretationFlag, AblationVariantResult};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SensitivityDimensionSummary {
    pub dimension: AblationDimension,
    pub variant_ids: Vec<String>,
    pub comparable_count: usize,
    pub candidate_improvement_count: usize,
    pub fragile_count: usize,
    pub research_only_count: usize,
    pub max_abs_avg_net_return_delta: f64,
    pub max_abs_avg_drawdown_delta: f64,
    pub max_abs_avg_calibration_brier_delta: f64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SensitivitySummary {
    pub dominant_dimension: Option<AblationDimension>,
    pub dimensions: Vec<SensitivityDimensionSummary>,
    pub reason_codes: Vec<ReasonCode>,
}

pub fn build_sensitivity_summary(results: &[AblationVariantResult]) -> SensitivitySummary {
    let mut grouped = BTreeMap::<AblationDimension, Vec<&AblationVariantResult>>::new();
    for result in results {
        grouped.entry(result.dimension).or_default().push(result);
    }

    let mut dimensions = grouped
        .into_iter()
        .map(|(dimension, values)| {
            let mut variant_ids = values
                .iter()
                .map(|value| value.variant_id.clone())
                .collect::<Vec<_>>();
            variant_ids.sort();
            SensitivityDimensionSummary {
                dimension,
                variant_ids,
                comparable_count: values
                    .iter()
                    .filter(|value| {
                        !value
                            .flags
                            .contains(&AblationInterpretationFlag::NotComparable)
                    })
                    .count(),
                candidate_improvement_count: values
                    .iter()
                    .filter(|value| {
                        value
                            .flags
                            .contains(&AblationInterpretationFlag::CandidateImprovement)
                    })
                    .count(),
                fragile_count: values
                    .iter()
                    .filter(|value| {
                        value
                            .flags
                            .contains(&AblationInterpretationFlag::HighFragility)
                            || value
                                .flags
                                .contains(&AblationInterpretationFlag::WorseDrawdown)
                            || value
                                .flags
                                .contains(&AblationInterpretationFlag::WorseCalibration)
                    })
                    .count(),
                research_only_count: values
                    .iter()
                    .filter(|value| {
                        value
                            .flags
                            .contains(&AblationInterpretationFlag::ResearchOnly)
                    })
                    .count(),
                max_abs_avg_net_return_delta: values
                    .iter()
                    .map(|value| value.delta.avg_net_return_pct.abs())
                    .fold(0.0, f64::max),
                max_abs_avg_drawdown_delta: values
                    .iter()
                    .map(|value| value.delta.avg_max_drawdown_pct.abs())
                    .fold(0.0, f64::max),
                max_abs_avg_calibration_brier_delta: values
                    .iter()
                    .map(|value| value.delta.avg_calibration_brier.unwrap_or(0.0).abs())
                    .fold(0.0, f64::max),
            }
        })
        .collect::<Vec<_>>();

    dimensions.sort_by(|left, right| {
        right
            .max_abs_avg_net_return_delta
            .total_cmp(&left.max_abs_avg_net_return_delta)
            .then_with(|| {
                right
                    .max_abs_avg_drawdown_delta
                    .total_cmp(&left.max_abs_avg_drawdown_delta)
            })
            .then_with(|| left.dimension.cmp(&right.dimension))
    });

    SensitivitySummary {
        dominant_dimension: dimensions.first().map(|summary| summary.dimension),
        dimensions,
        reason_codes: vec![ReasonCode::DeterministicPath],
    }
}
