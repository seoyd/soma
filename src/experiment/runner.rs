use std::fs;
use std::path::Path;
use std::process::Command;

use crate::core::ReasonCode;
use crate::data::{
    CandleCsvLoader, DataQualitySeverity, DataValidationConfig, ResampleConfig, Resampler,
};
use crate::eval::{DatasetExportConfig, DatasetOutputFormat, FeatureSchema, WalkForwardEvaluator};
use crate::model::{
    EvaluationMode, ExternalPredictionSignalConfig, ExternalPredictionSignalModel,
    ModelArtifactMeta, ModelKind, PredictionImportConfig, ThresholdSearchConfig,
    build_calibration_report, compare_walk_forward_reports, prediction_frame_from_csv_string,
    search_thresholds,
};

use super::config::{ExperimentConfig, ExperimentMode};
use super::report_bundle::{BundleArtifacts, DatasetExportSummary, ExperimentReportBundle};
use super::stage::{ExperimentStage, StageStatus};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ExperimentRunner {
    pub data_loader: CandleCsvLoader,
}

impl ExperimentRunner {
    pub fn run(&self, config: &ExperimentConfig) -> ExperimentReportBundle {
        let mut bundle = ExperimentReportBundle::empty(config);
        let mut artifacts = BundleArtifacts::default();

        let invalid_config = config.validate_local_paths();
        if !invalid_config.is_empty() {
            bundle.reason_codes.extend(invalid_config.clone());
            bundle
                .experiment_manifest
                .reason_codes
                .extend(invalid_config.clone());
            bundle
                .errors
                .push("experiment config contains remote URL-like path".to_string());
            bundle
                .experiment_manifest
                .set_stage_status(ExperimentStage::LoadData, StageStatus::Failed);
            return self.finish(config, bundle, &artifacts);
        }

        let csv_config = config.build_csv_config();
        let loader = CandleCsvLoader {
            validation: DataValidationConfig {
                expected_step_ms: Some(
                    crate::data::TimeframeSpec::from_timeframe(config.timeframe).expected_ms_step,
                ),
                ..config.data_validation_config.clone()
            },
            ..self.data_loader.clone()
        };

        let loaded = match loader.load_from_path(Path::new(&config.data_path), &csv_config) {
            Ok(loaded) => loaded,
            Err(error) => {
                bundle.reason_codes.push(ReasonCode::DataLoadFailed);
                bundle.reason_codes.extend(error.reason_codes.clone());
                bundle
                    .experiment_manifest
                    .set_stage_status(ExperimentStage::LoadData, StageStatus::Failed);
                bundle
                    .experiment_manifest
                    .set_stage_status(ExperimentStage::ValidateData, StageStatus::Failed);
                if let Some(report) = error.quality_report {
                    bundle.data_quality_report = report;
                }
                if !error.issues.is_empty() {
                    bundle
                        .errors
                        .push(format!("data load failed: {:?}", error.issues[0].error));
                } else {
                    bundle.errors.push("data load failed".to_string());
                }
                return self.finish(config, bundle, &artifacts);
            }
        };

        bundle
            .experiment_manifest
            .set_stage_status(ExperimentStage::LoadData, StageStatus::Passed);
        bundle.data_quality_report = loaded.quality_report.clone();
        bundle.experiment_manifest.input_data_manifest = loaded.manifest.clone();
        let validation_failed = matches!(
            loaded.quality_report.severity,
            DataQualitySeverity::Unusable
        ) || (config.fail_on_bad_data
            && matches!(
                loaded.quality_report.severity,
                DataQualitySeverity::Bad | DataQualitySeverity::Unusable
            ));
        if validation_failed {
            bundle.reason_codes.push(ReasonCode::DataUnusable);
            bundle
                .experiment_manifest
                .set_stage_status(ExperimentStage::ValidateData, StageStatus::Failed);
            if config.mode != ExperimentMode::ValidateDataOnly {
                bundle
                    .errors
                    .push("data quality is unusable for experiment mode".to_string());
                return self.finish(config, bundle, &artifacts);
            }
        } else {
            bundle
                .experiment_manifest
                .set_stage_status(ExperimentStage::ValidateData, StageStatus::Passed);
        }

        if config.mode == ExperimentMode::ValidateDataOnly {
            return self.finish(config, bundle, &artifacts);
        }

        let mut working_series = loaded.series.clone();
        if let Some(target_timeframe) = config.resample_to
            && target_timeframe != config.timeframe
        {
            match Resampler.resample(
                &working_series,
                &ResampleConfig {
                    source_timeframe: config.timeframe,
                    target_timeframe,
                    ..ResampleConfig::default()
                },
            ) {
                Ok(result) => {
                    working_series = result.series;
                    bundle
                        .experiment_manifest
                        .set_stage_status(ExperimentStage::Resample, StageStatus::Passed);
                }
                Err(reasons) => {
                    bundle.reason_codes.extend(reasons);
                    bundle
                        .experiment_manifest
                        .set_stage_status(ExperimentStage::Resample, StageStatus::Failed);
                    bundle.errors.push("resample failed".to_string());
                    return self.finish(config, bundle, &artifacts);
                }
            }
        }

        let evaluator = build_evaluator(config);
        let feature_frame = evaluator.feature_engine.build_frame(&working_series);
        if feature_frame.rows.is_empty() {
            bundle.reason_codes.push(ReasonCode::FeatureBuildFailed);
            bundle
                .experiment_manifest
                .set_stage_status(ExperimentStage::BuildFeatures, StageStatus::Failed);
            bundle.errors.push("feature frame is empty".to_string());
            return self.finish(config, bundle, &artifacts);
        }
        bundle.experiment_manifest.feature_schema =
            FeatureSchema::from_engine(&evaluator.feature_engine);
        bundle
            .experiment_manifest
            .set_stage_status(ExperimentStage::BuildFeatures, StageStatus::Passed);

        let split = evaluator.split(&working_series, config.walk_forward_config);
        let dataset_needed = matches!(
            config.mode,
            ExperimentMode::DatasetExportOnly
                | ExperimentMode::ExternalPredictionOnly
                | ExperimentMode::TrainAndCompare
        );
        let mut dataset_frame = None;
        if dataset_needed {
            let frame = evaluator.export_dataset(
                &working_series,
                &split,
                &DatasetExportConfig {
                    include_labels: true,
                    include_metadata: true,
                    include_reason_codes: true,
                    output_format: DatasetOutputFormat::Csv,
                },
            );
            if frame.rows.is_empty() {
                bundle.reason_codes.push(ReasonCode::DatasetExportFailed);
                bundle
                    .experiment_manifest
                    .set_stage_status(ExperimentStage::BuildDataset, StageStatus::Failed);
                bundle
                    .errors
                    .push("dataset export produced no rows".to_string());
                return self.finish(config, bundle, &artifacts);
            }
            artifacts.dataset_csv = Some(frame.to_csv_string(&DatasetExportConfig {
                include_labels: true,
                include_metadata: true,
                include_reason_codes: true,
                output_format: DatasetOutputFormat::Csv,
            }));
            bundle.dataset_export_summary = Some(DatasetExportSummary {
                row_count: frame.rows.len(),
                feature_count: frame.feature_names.len(),
                feature_names: frame
                    .feature_names
                    .iter()
                    .map(|name| name.as_str().to_string())
                    .collect(),
                output_path: Some(
                    config
                        .output_bundle_dir()
                        .join("dataset.csv")
                        .display()
                        .to_string(),
                ),
                reason_codes: vec![ReasonCode::DatasetExported],
            });
            bundle
                .experiment_manifest
                .set_stage_status(ExperimentStage::BuildDataset, StageStatus::Passed);
            dataset_frame = Some(frame);
        }

        match config.mode {
            ExperimentMode::DatasetExportOnly => return self.finish(config, bundle, &artifacts),
            ExperimentMode::BaselineOnly => {
                let baseline_report =
                    evaluator.evaluate(&working_series, config.walk_forward_config);
                bundle.baseline_walk_forward_report = Some(baseline_report);
                bundle
                    .experiment_manifest
                    .set_stage_status(ExperimentStage::BaselineEvaluate, StageStatus::Passed);
                return self.finish(config, bundle, &artifacts);
            }
            ExperimentMode::ExternalPredictionOnly => {
                self.run_external_flow(
                    config,
                    &working_series,
                    evaluator,
                    dataset_frame.expect("dataset required"),
                    None,
                    &mut bundle,
                    &mut artifacts,
                    false,
                );
                return self.finish(config, bundle, &artifacts);
            }
            ExperimentMode::TrainAndCompare => {
                self.run_train_compare_flow(
                    config,
                    &working_series,
                    evaluator,
                    dataset_frame.expect("dataset required"),
                    &mut bundle,
                    &mut artifacts,
                );
                return self.finish(config, bundle, &artifacts);
            }
            ExperimentMode::ValidateDataOnly => {}
        }

        self.finish(config, bundle, &artifacts)
    }

    #[allow(clippy::too_many_arguments)]
    fn run_external_flow(
        &self,
        config: &ExperimentConfig,
        series: &crate::backtest::CandleSeries,
        mut evaluator: WalkForwardEvaluator,
        dataset_frame: crate::eval::DatasetFrame,
        prediction_csv_override: Option<String>,
        bundle: &mut ExperimentReportBundle,
        artifacts: &mut BundleArtifacts,
        compare_with_baseline: bool,
    ) {
        let prediction_csv = match prediction_csv_override
            .or_else(|| read_optional_file(config.prediction_csv_path.as_deref()))
        {
            Some(csv) => csv,
            None => {
                bundle.reason_codes.push(ReasonCode::PredictionImportFailed);
                bundle
                    .experiment_manifest
                    .set_stage_status(ExperimentStage::ImportPredictions, StageStatus::Failed);
                bundle
                    .errors
                    .push("prediction csv not available".to_string());
                return;
            }
        };
        artifacts.predictions_csv = Some(prediction_csv.clone());
        let model_meta = ModelArtifactMeta {
            model_id: format!("{}_external", config.experiment_id),
            model_kind: ModelKind::ExternalPredictionFile,
            created_at_ms: config.created_at_ms,
            feature_schema_version: bundle.experiment_manifest.feature_schema.schema_version,
            feature_schema_hash: bundle.experiment_manifest.feature_schema.checksum,
            training_window: None,
            validation_window: None,
            test_window: None,
            target_label_config: bundle.experiment_manifest.label_config_summary.clone(),
            cost_model_summary: bundle.experiment_manifest.cost_model_summary.clone(),
            notes: Some("research_only_experiment_import".to_string()),
            reason_codes: vec![ReasonCode::DeterministicPath],
        };
        let prediction_frame = prediction_frame_from_csv_string(
            &prediction_csv,
            model_meta,
            &dataset_frame,
            &bundle.experiment_manifest.feature_schema,
            &PredictionImportConfig {
                require_feature_schema_match: config.strict_schema_validation,
                require_row_alignment: true,
                min_confidence: None,
                max_missing_rows: if config.strict_schema_validation {
                    0
                } else {
                    usize::MAX / 4
                },
                input_format: crate::model::PredictionInputFormat::Csv,
            },
        );
        bundle.prediction_validation_result = Some(prediction_frame.schema_validation.clone());
        if config.strict_schema_validation && !prediction_frame.schema_validation.valid {
            bundle.reason_codes.push(ReasonCode::PredictionImportFailed);
            bundle
                .reason_codes
                .extend(prediction_frame.reason_codes.clone());
            bundle
                .experiment_manifest
                .set_stage_status(ExperimentStage::ImportPredictions, StageStatus::Failed);
            bundle
                .errors
                .push("prediction import failed strict validation".to_string());
            return;
        }
        bundle
            .experiment_manifest
            .set_stage_status(ExperimentStage::ImportPredictions, StageStatus::Passed);

        let calibration = build_calibration_report(&prediction_frame, &dataset_frame, None);
        bundle.calibration_report = Some(calibration.clone());
        let threshold_report = dataset_frame
            .rows
            .iter()
            .filter_map(|row| row.fold_id)
            .min()
            .map(|fold_id| {
                search_thresholds(
                    fold_id,
                    &dataset_frame,
                    &prediction_frame,
                    &ThresholdSearchConfig {
                        p_win_thresholds: vec![0.45, 0.55],
                        p_stop_thresholds: vec![0.35, 0.45],
                        confidence_thresholds: vec![0.0, 0.55],
                        no_trade_thresholds: vec![0.80, 1.0],
                        min_expected_return_thresholds: vec![0.0, 0.002],
                        max_drawdown_constraint: Some(0.50),
                        min_sample_count: 1,
                        optimize_metric: crate::model::OptimizeMetric::NetReturn,
                        validation_only: true,
                    },
                )
            });
        bundle.threshold_search_report = threshold_report;

        evaluator.external_signal_model = Some(ExternalPredictionSignalModel {
            prediction_frame: prediction_frame.clone(),
            config: ExternalPredictionSignalConfig {
                strict_validation: config.strict_schema_validation,
                min_confidence: None,
                fallback_horizon_bars: config.triple_barrier_config.horizon_bars as u32,
            },
        });
        let external_report = evaluator.evaluate_with_mode(
            series,
            config.walk_forward_config,
            EvaluationMode::ExternalPrediction,
        );
        bundle.external_walk_forward_report = Some(external_report.clone());
        bundle
            .experiment_manifest
            .set_stage_status(ExperimentStage::ExternalEvaluate, StageStatus::Passed);

        if compare_with_baseline {
            let baseline_report = evaluator.evaluate(series, config.walk_forward_config);
            bundle.baseline_walk_forward_report = Some(baseline_report.clone());
            bundle
                .experiment_manifest
                .set_stage_status(ExperimentStage::BaselineEvaluate, StageStatus::Passed);
            bundle.model_comparison_report = Some(compare_walk_forward_reports(
                "baseline_rule_v0",
                &prediction_frame.model_meta.model_id,
                &baseline_report,
                &external_report,
                None,
                bundle.calibration_report.as_ref(),
                None,
            ));
            bundle
                .experiment_manifest
                .set_stage_status(ExperimentStage::CompareModels, StageStatus::Passed);
        }
    }

    fn run_train_compare_flow(
        &self,
        config: &ExperimentConfig,
        series: &crate::backtest::CandleSeries,
        evaluator: WalkForwardEvaluator,
        dataset_frame: crate::eval::DatasetFrame,
        bundle: &mut ExperimentReportBundle,
        artifacts: &mut BundleArtifacts,
    ) {
        let existing_prediction_csv = read_optional_file(config.prediction_csv_path.as_deref());
        let prediction_csv = if let Some(csv) = existing_prediction_csv {
            bundle
                .experiment_manifest
                .set_stage_status(ExperimentStage::PythonValidateDataset, StageStatus::Skipped);
            bundle
                .experiment_manifest
                .set_stage_status(ExperimentStage::PythonTrain, StageStatus::Skipped);
            Some(csv)
        } else if !config.run_python_training {
            bundle.reason_codes.push(ReasonCode::PythonUnavailable);
            bundle
                .experiment_manifest
                .set_stage_status(ExperimentStage::PythonValidateDataset, StageStatus::Skipped);
            bundle
                .experiment_manifest
                .set_stage_status(ExperimentStage::PythonTrain, StageStatus::Failed);
            bundle
                .errors
                .push("python training disabled and no prediction csv provided".to_string());
            None
        } else {
            match run_python_pipeline(config, artifacts.dataset_csv.as_deref().unwrap_or_default())
            {
                Ok((prediction_csv, model_card)) => {
                    bundle.experiment_manifest.set_stage_status(
                        ExperimentStage::PythonValidateDataset,
                        StageStatus::Passed,
                    );
                    bundle
                        .experiment_manifest
                        .set_stage_status(ExperimentStage::PythonTrain, StageStatus::Passed);
                    artifacts.model_card_markdown = model_card;
                    Some(prediction_csv)
                }
                Err(reason) => {
                    bundle.reason_codes.push(reason);
                    bundle.experiment_manifest.set_stage_status(
                        ExperimentStage::PythonValidateDataset,
                        StageStatus::Failed,
                    );
                    bundle
                        .experiment_manifest
                        .set_stage_status(ExperimentStage::PythonTrain, StageStatus::Failed);
                    bundle.errors.push("python pipeline failed".to_string());
                    None
                }
            }
        };

        if let Some(prediction_csv) = prediction_csv {
            self.run_external_flow(
                config,
                series,
                evaluator,
                dataset_frame,
                Some(prediction_csv),
                bundle,
                artifacts,
                true,
            );
        }
    }

    fn finish(
        &self,
        config: &ExperimentConfig,
        mut bundle: ExperimentReportBundle,
        artifacts: &BundleArtifacts,
    ) -> ExperimentReportBundle {
        bundle.experiment_manifest.mark_remaining_skipped();
        let output_dir = config.output_bundle_dir();
        bundle
            .experiment_manifest
            .set_stage_status(ExperimentStage::WriteReportBundle, StageStatus::Passed);
        if let Err(err) = bundle.write_to_dir(&output_dir, artifacts) {
            bundle.reason_codes.push(ReasonCode::ReportWriteFailed);
            bundle.errors.push(err);
            bundle
                .experiment_manifest
                .set_stage_status(ExperimentStage::WriteReportBundle, StageStatus::Failed);
        }
        bundle
    }
}

fn build_evaluator(config: &ExperimentConfig) -> WalkForwardEvaluator {
    let mut evaluator = WalkForwardEvaluator::default();
    evaluator.feature_engine.config = config.feature_config.clone();
    evaluator.regime_classifier.config = config.regime_config;
    evaluator.chair.config = config.chair_config;
    evaluator.governor.config = config.risk_config;
    evaluator.triple_barrier_config = config.triple_barrier_config;
    evaluator.cost_model = config.cost_model;
    evaluator.no_trade_score_config = config.no_trade_score_config;
    evaluator.full_auto = config.full_auto;
    evaluator
}

fn read_optional_file(path: Option<&str>) -> Option<String> {
    let path = path?;
    if path.contains("://") {
        return None;
    }
    fs::read_to_string(path).ok()
}

fn run_python_pipeline(
    config: &ExperimentConfig,
    dataset_csv: &str,
) -> Result<(String, Option<String>), ReasonCode> {
    let training_script = config
        .training_script_path
        .as_deref()
        .ok_or(ReasonCode::PythonTrainingFailed)?;
    let Some(python) = resolve_python(config.python_executable.as_deref()) else {
        return Err(ReasonCode::PythonUnavailable);
    };

    let output_dir = config.output_bundle_dir();
    fs::create_dir_all(&output_dir).map_err(|_| ReasonCode::ReportWriteFailed)?;
    let dataset_path = output_dir.join("dataset.csv");
    let prediction_path = output_dir.join("predictions.csv");
    let model_card_path = output_dir.join("model_card.md");
    fs::write(&dataset_path, dataset_csv).map_err(|_| ReasonCode::ReportWriteFailed)?;

    if let Some(validate_script) = sibling_script(training_script, "validate_dataset.py")
        && validate_script.exists()
    {
        let status = Command::new(&python)
            .arg(validate_script)
            .arg("--input")
            .arg(&dataset_path)
            .status()
            .map_err(|_| ReasonCode::PythonUnavailable)?;
        if !status.success() {
            return Err(ReasonCode::PythonValidationFailed);
        }
    }

    let status = Command::new(&python)
        .arg(training_script)
        .arg("--input")
        .arg(&dataset_path)
        .arg("--predictions-out")
        .arg(&prediction_path)
        .arg("--model-card-out")
        .arg(&model_card_path)
        .status()
        .map_err(|_| ReasonCode::PythonUnavailable)?;
    if !status.success() {
        return Err(ReasonCode::PythonTrainingFailed);
    }
    let predictions =
        fs::read_to_string(&prediction_path).map_err(|_| ReasonCode::PythonTrainingFailed)?;
    let model_card = fs::read_to_string(&model_card_path).ok();
    Ok((predictions, model_card))
}

fn resolve_python(configured: Option<&str>) -> Option<String> {
    if let Some(candidate) = configured {
        return Command::new(candidate)
            .arg("--version")
            .status()
            .ok()
            .filter(|status| status.success())
            .map(|_| candidate.to_string());
    }
    for candidate in ["python3", "python"] {
        if Command::new(candidate).arg("--version").status().is_ok() {
            return Some(candidate.to_string());
        }
    }
    None
}

fn sibling_script(path: &str, sibling_name: &str) -> Option<std::path::PathBuf> {
    let path = Path::new(path);
    Some(path.parent()?.join(sibling_name))
}
