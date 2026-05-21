use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;
use crate::experiment::{ExternalTabularBenchmarkStage, OfficialAiBenchmarkReport};
use crate::model::PredictionValidationResult;

use super::core_checked_benchmark::CoreCheckedBenchmarkConfig;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalTabularBenchmarkStageBuilder;

impl ExternalTabularBenchmarkStageBuilder {
    pub fn build(
        &self,
        config: &CoreCheckedBenchmarkConfig,
        benchmark_report: Option<&OfficialAiBenchmarkReport>,
    ) -> ExternalTabularBenchmarkStage {
        let training_requested = config.run_python_training;
        let training_ran = config.run_python_training
            && config.python_executable.is_some()
            && config.train_script_path.is_some();
        let schema_valid = benchmark_report.is_some_and(|report| {
            report
                .dataset_reports
                .iter()
                .all(|dataset| dataset.schema_valid.unwrap_or(false))
        });
        let row_count = benchmark_report
            .map(|report| {
                report
                    .dataset_reports
                    .iter()
                    .map(|dataset| dataset.external_total_trades.unwrap_or(0))
                    .sum()
            })
            .unwrap_or(0usize);
        let missing_row_count = benchmark_report
            .map(|report| {
                report
                    .dataset_reports
                    .iter()
                    .map(|dataset| {
                        dataset
                            .baseline_total_trades
                            .saturating_sub(dataset.external_total_trades.unwrap_or(0))
                    })
                    .sum()
            })
            .unwrap_or(0usize);
        let row_alignment_valid = missing_row_count == 0;
        let mut reason_codes = Vec::new();
        if training_requested && !training_ran && config.existing_prediction_csv.is_none() {
            reason_codes.push(ReasonCode::PythonUnavailable);
        }
        if !schema_valid && (config.run_external_eval || config.existing_prediction_csv.is_some()) {
            reason_codes.push(ReasonCode::SchemaMismatch);
            reason_codes.push(ReasonCode::InvalidPrediction);
        }
        if !row_alignment_valid {
            reason_codes.push(ReasonCode::MissingPredictionRows);
        }
        let prediction_validation_result = PredictionValidationResult {
            valid: schema_valid
                && row_alignment_valid
                && row_count >= config.min_prediction_rows
                && (config.run_external_eval || config.existing_prediction_csv.is_some()),
            row_count,
            missing_row_count,
            extra_row_count: 0,
            schema_match: schema_valid,
            feature_schema_hash_match: schema_valid,
            invalid_probability_count: 0,
            nan_or_inf_count: 0,
            timestamp_mismatch_count: if row_alignment_valid {
                0
            } else {
                missing_row_count
            },
            reason_codes: reason_codes.clone(),
        };

        ExternalTabularBenchmarkStage {
            training_requested,
            training_ran,
            training_backend_used: if training_ran {
                Some("python".to_string())
            } else if config.existing_prediction_csv.is_some() {
                Some("existing-prediction-csv".to_string())
            } else {
                None
            },
            prediction_csv_path: config.existing_prediction_csv.clone(),
            model_card_path: if training_ran {
                Some(
                    config
                        .output_dir()
                        .join("external_tabular_model_card.json")
                        .display()
                        .to_string(),
                )
            } else {
                None
            },
            prediction_validation_result,
            schema_valid,
            row_alignment_valid,
            reason_codes,
        }
    }
}
