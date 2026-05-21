use serde::{Deserialize, Serialize};

use crate::backtest::TripleBarrierOutcome;
use crate::core::ReasonCode;
use crate::eval::{CalibrationBin, DatasetFrame, DatasetSplitKind};

use super::prediction::PredictionFrame;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CalibrationReport {
    pub model_id: String,
    pub fold_id: Option<usize>,
    pub total_count: usize,
    pub brier_score: f64,
    pub expected_calibration_error: f64,
    pub bins: Vec<CalibrationBin>,
    pub reason_codes: Vec<ReasonCode>,
}

pub fn build_calibration_report(
    prediction_frame: &PredictionFrame,
    dataset_frame: &DatasetFrame,
    fold_id: Option<usize>,
) -> CalibrationReport {
    let dataset_rows = dataset_frame
        .rows
        .iter()
        .filter(|row| {
            matches!(
                row.split_kind,
                DatasetSplitKind::Validation | DatasetSplitKind::Test
            ) && fold_id
                .map(|expected| row.fold_id == Some(expected))
                .unwrap_or(true)
        })
        .collect::<Vec<_>>();
    let observations = dataset_rows
        .iter()
        .filter_map(|row| {
            let prediction = prediction_frame.find_by_row_id(&row.row_id)?;
            let actual = match row.label_outcome? {
                TripleBarrierOutcome::Win => 1.0,
                _ => 0.0,
            };
            Some((prediction.p_win, actual))
        })
        .collect::<Vec<_>>();

    if observations.is_empty() {
        return CalibrationReport {
            model_id: prediction_frame.model_meta.model_id.clone(),
            fold_id,
            total_count: 0,
            brier_score: 0.0,
            expected_calibration_error: 0.0,
            bins: deterministic_bins(&observations),
            reason_codes: vec![ReasonCode::CalibrationEmpty],
        };
    }

    let brier_score = average(
        &observations
            .iter()
            .map(|(predicted, actual)| {
                let diff = predicted - actual;
                diff * diff
            })
            .collect::<Vec<_>>(),
    );
    let bins = deterministic_bins(&observations);
    let expected_calibration_error = if observations.is_empty() {
        0.0
    } else {
        bins.iter()
            .map(|bin| {
                (bin.count as f64 / observations.len() as f64)
                    * (bin.predicted_avg - bin.actual_win_rate).abs()
            })
            .sum()
    };

    CalibrationReport {
        model_id: prediction_frame.model_meta.model_id.clone(),
        fold_id,
        total_count: observations.len(),
        brier_score,
        expected_calibration_error,
        bins,
        reason_codes: Vec::new(),
    }
}

fn deterministic_bins(observations: &[(f64, f64)]) -> Vec<CalibrationBin> {
    let intervals = [
        (0.0, 0.2),
        (0.2, 0.4),
        (0.4, 0.6),
        (0.6, 0.8),
        (0.8, 1.000_000_1),
    ];
    intervals
        .into_iter()
        .map(|(lower, upper)| {
            let bucket = observations
                .iter()
                .filter(|(predicted, _)| *predicted >= lower && *predicted < upper)
                .collect::<Vec<_>>();
            CalibrationBin {
                bin_lower: lower,
                bin_upper: upper.min(1.0),
                count: bucket.len(),
                predicted_avg: average(
                    &bucket
                        .iter()
                        .map(|(predicted, _)| *predicted)
                        .collect::<Vec<_>>(),
                ),
                actual_win_rate: average(
                    &bucket.iter().map(|(_, actual)| *actual).collect::<Vec<_>>(),
                ),
                brier_score: average(
                    &bucket
                        .iter()
                        .map(|(predicted, actual)| {
                            let diff = predicted - actual;
                            diff * diff
                        })
                        .collect::<Vec<_>>(),
                ),
            }
        })
        .collect()
}

fn average(values: &[f64]) -> f64 {
    if values.is_empty() {
        0.0
    } else {
        values.iter().sum::<f64>() / values.len() as f64
    }
}
