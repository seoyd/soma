use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_hash};
use crate::data::{
    DataManifest, DataProvenance, DataQualityReport, DataQualitySeverity, DataSourceKind,
};
use crate::eval::{FeatureSchema, WalkForwardReport};
use crate::model::{
    CalibrationReport, ModelComparisonReport, PredictionValidationResult, ThresholdSearchReport,
};

use super::config::ExperimentConfig;
use super::manifest::ExperimentManifest;
use super::stage::ExperimentStage;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct DatasetExportSummary {
    pub row_count: usize,
    pub feature_count: usize,
    pub feature_names: Vec<String>,
    pub output_path: Option<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct BundleArtifacts {
    pub dataset_csv: Option<String>,
    pub predictions_csv: Option<String>,
    pub model_card_markdown: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExperimentReportBundle {
    pub experiment_manifest: ExperimentManifest,
    pub data_quality_report: DataQualityReport,
    pub baseline_walk_forward_report: Option<WalkForwardReport>,
    pub external_walk_forward_report: Option<WalkForwardReport>,
    pub model_comparison_report: Option<ModelComparisonReport>,
    pub calibration_report: Option<CalibrationReport>,
    pub threshold_search_report: Option<ThresholdSearchReport>,
    pub dataset_export_summary: Option<DatasetExportSummary>,
    pub prediction_validation_result: Option<PredictionValidationResult>,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

impl ExperimentReportBundle {
    pub fn empty(config: &ExperimentConfig) -> Self {
        let input_data_manifest = DataManifest {
            manifest_version: 1,
            dataset_id: format!("unloaded-{:016x}", stable_hash(&config.experiment_id)),
            symbol: config.symbol.clone(),
            normalized_symbol: config.symbol.clone(),
            venue: crate::data::MarketVenue::Generic,
            asset_class: crate::data::AssetClass::Unknown,
            timeframe: config.timeframe,
            source_kind: DataSourceKind::Unknown,
            source_path: Some(config.data_path.clone()),
            provenance: Some(DataProvenance::inferred_from_path(Some(&config.data_path))),
            row_count: 0,
            first_timestamp_ms: 0,
            last_timestamp_ms: 0,
            expected_step_ms: 0,
            data_quality_score: 0.0,
            feature_schema_hash: None,
            label_config_summary: None,
            cost_model_summary: None,
            adjusted_price_policy_summary: None,
            corporate_action_adjusted: None,
            provider_symbol: None,
            collection_size_policy_summary: None,
            truncated: false,
            row_limit_applied: false,
            raw_archive_policy_summary: None,
            auth_requirement_summary: None,
            created_at_ms: config.created_at_ms,
            reason_codes: vec![ReasonCode::DataLoadFailed],
        };
        let feature_schema = FeatureSchema::from_feature_names(&[]);
        Self {
            experiment_manifest: ExperimentManifest::new(
                config,
                input_data_manifest,
                feature_schema,
            ),
            data_quality_report: DataQualityReport {
                symbol: config.symbol.clone(),
                timeframe: config.timeframe,
                row_count: 0,
                valid_row_count: 0,
                invalid_row_count: 0,
                dropped_row_count: 0,
                repaired_row_count: 0,
                duplicate_timestamp_count: 0,
                out_of_order_count: 0,
                gap_count: 0,
                max_gap_ms: 0,
                gap_ratio: 0.0,
                non_positive_price_count: 0,
                negative_volume_count: 0,
                ohlc_invariant_violation_count: 0,
                missing_bid_ask_count: 0,
                extreme_spread_count: 0,
                data_quality_score: 0.0,
                severity: DataQualitySeverity::Unusable,
                reason_codes: vec![ReasonCode::DataLoadFailed],
            },
            baseline_walk_forward_report: None,
            external_walk_forward_report: None,
            model_comparison_report: None,
            calibration_report: None,
            threshold_search_report: None,
            dataset_export_summary: None,
            prediction_validation_result: None,
            errors: Vec::new(),
            warnings: Vec::new(),
            reason_codes: config.reason_codes.clone(),
        }
    }

    pub fn to_deterministic_summary(&self) -> String {
        let mut lines = vec![
            self.experiment_manifest.to_deterministic_string(),
            format!(
                "data_quality=row_count:{};valid_row_count:{};score:{:.6};severity:{:?}",
                self.data_quality_report.row_count,
                self.data_quality_report.valid_row_count,
                self.data_quality_report.data_quality_score,
                self.data_quality_report.severity
            ),
            format!(
                "baseline_present={}",
                self.baseline_walk_forward_report.is_some()
            ),
            format!(
                "external_present={}",
                self.external_walk_forward_report.is_some()
            ),
            format!(
                "comparison_present={}",
                self.model_comparison_report.is_some()
            ),
            format!(
                "dataset_summary_present={}",
                self.dataset_export_summary.is_some()
            ),
            format!(
                "prediction_validation_present={}",
                self.prediction_validation_result.is_some()
            ),
            format!("errors={}", self.errors.join(" | ")),
            format!("warnings={}", self.warnings.join(" | ")),
            format!(
                "reason_codes={}",
                self.reason_codes
                    .iter()
                    .map(|reason| format!("{reason:?}"))
                    .collect::<Vec<_>>()
                    .join("|")
            ),
        ];
        if let Some(dataset) = &self.dataset_export_summary {
            lines.push(format!(
                "dataset=row_count:{};feature_count:{};features:{}",
                dataset.row_count,
                dataset.feature_count,
                dataset.feature_names.join("|")
            ));
        }
        if let Some(validation) = &self.prediction_validation_result {
            lines.push(format!(
                "prediction_validation=valid:{};rows:{};missing:{};extra:{};timestamp_mismatch:{}",
                validation.valid,
                validation.row_count,
                validation.missing_row_count,
                validation.extra_row_count,
                validation.timestamp_mismatch_count
            ));
        }
        lines.join("\n")
    }

    pub fn write_to_dir(&self, dir: &Path, artifacts: &BundleArtifacts) -> Result<(), String> {
        fs::create_dir_all(dir).map_err(|err| err.to_string())?;
        write_file(
            dir.join("manifest.txt"),
            &self.experiment_manifest.to_deterministic_string(),
        )?;
        write_file(
            dir.join("data_quality_report.txt"),
            &data_quality_to_string(&self.data_quality_report),
        )?;
        if let Some(report) = &self.baseline_walk_forward_report {
            write_file(
                dir.join("baseline_report.txt"),
                &walk_forward_to_string(report),
            )?;
        }
        if let Some(report) = &self.external_walk_forward_report {
            write_file(
                dir.join("external_report.txt"),
                &walk_forward_to_string(report),
            )?;
        }
        if let Some(report) = &self.model_comparison_report {
            write_file(
                dir.join("comparison_report.txt"),
                &comparison_to_string(report),
            )?;
        }
        if let Some(dataset) = &artifacts.dataset_csv {
            write_file(dir.join("dataset.csv"), dataset)?;
        }
        if let Some(predictions) = &artifacts.predictions_csv {
            write_file(dir.join("predictions.csv"), predictions)?;
        }
        if let Some(model_card) = &artifacts.model_card_markdown {
            write_file(dir.join("model_card.md"), model_card)?;
        }
        write_file(
            dir.join("experiment_summary.txt"),
            &self.to_deterministic_summary(),
        )?;
        Ok(())
    }

    pub fn stage_status(&self, stage: ExperimentStage) -> super::stage::StageStatus {
        self.experiment_manifest
            .stage_statuses
            .get(&stage)
            .copied()
            .unwrap_or(super::stage::StageStatus::Pending)
    }
}

fn write_file(path: impl AsRef<Path>, contents: &str) -> Result<(), String> {
    fs::write(path, contents).map_err(|err| err.to_string())
}

fn data_quality_to_string(report: &DataQualityReport) -> String {
    [
        format!("symbol={}", report.symbol),
        format!("timeframe={:?}", report.timeframe),
        format!("row_count={}", report.row_count),
        format!("valid_row_count={}", report.valid_row_count),
        format!("invalid_row_count={}", report.invalid_row_count),
        format!("dropped_row_count={}", report.dropped_row_count),
        format!("repaired_row_count={}", report.repaired_row_count),
        format!("gap_count={}", report.gap_count),
        format!("max_gap_ms={}", report.max_gap_ms),
        format!("data_quality_score={:.6}", report.data_quality_score),
        format!("severity={:?}", report.severity),
        format!(
            "reason_codes={}",
            report
                .reason_codes
                .iter()
                .map(|reason| format!("{reason:?}"))
                .collect::<Vec<_>>()
                .join("|")
        ),
    ]
    .join("\n")
}

fn walk_forward_to_string(report: &WalkForwardReport) -> String {
    [
        format!("symbol={}", report.symbol),
        format!("timeframe={:?}", report.timeframe),
        format!("fold_count={}", report.folds.len()),
        format!("feature_schema_hash={}", report.feature_schema.checksum),
        format!(
            "net_return_pct={:.8}",
            report.aggregate_metrics.trade_metrics.net_return_pct
        ),
        format!(
            "max_drawdown_pct={:.8}",
            report.aggregate_metrics.trade_metrics.max_drawdown_pct
        ),
        format!(
            "denied_count={}",
            report.aggregate_metrics.risk_metrics.denied_count
        ),
        format!(
            "no_trade_count={}",
            report.aggregate_metrics.decision_metrics.no_trade
        ),
        format!(
            "reason_codes={}",
            report
                .reason_codes
                .iter()
                .map(|reason| format!("{reason:?}"))
                .collect::<Vec<_>>()
                .join("|")
        ),
    ]
    .join("\n")
}

fn comparison_to_string(report: &ModelComparisonReport) -> String {
    [
        format!("baseline_model_id={}", report.baseline_model_id),
        format!("external_model_id={}", report.external_model_id),
        format!("external_better={}", report.external_better),
        format!("delta_net_return_pct={:.8}", report.delta_net_return_pct),
        format!(
            "delta_max_drawdown_pct={:.8}",
            report.delta_max_drawdown_pct
        ),
        format!(
            "reason_codes={}",
            report
                .reason_codes
                .iter()
                .map(|reason| format!("{reason:?}"))
                .collect::<Vec<_>>()
                .join("|")
        ),
    ]
    .join("\n")
}
