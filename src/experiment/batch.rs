use std::fs;
use std::path::Path;

use crate::core::ReasonCode;

use super::aggregate::{
    BatchExperimentReport, ExperimentRunStatus, build_aggregate_benchmark,
    build_data_quality_aggregate, build_model_comparison_aggregate, build_regime_aggregate,
    build_risk_governor_aggregate, summarize_run,
};
use super::matrix::{DatasetEntry, ExperimentMatrixConfig};
use super::readiness::{build_expansion_readiness_report, build_persona_readiness_summary};
use super::report_bundle::ExperimentReportBundle;
use super::runner::ExperimentRunner;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct BatchExperimentRunner {
    pub experiment_runner: ExperimentRunner,
}

impl BatchExperimentRunner {
    pub fn run_matrix(&self, config: &ExperimentMatrixConfig) -> BatchExperimentReport {
        let mut run_summaries = Vec::new();
        let mut bundle_refs = Vec::<(String, ExperimentReportBundle)>::new();
        let mut reason_codes = Vec::new();

        let invalid = config.validate_local_paths();
        if !invalid.is_empty() {
            return BatchExperimentReport {
                matrix_id: config.matrix_id.clone(),
                run_summaries,
                aggregate_benchmark: build_aggregate_benchmark(&[]),
                data_quality_summary: build_data_quality_aggregate(&[]),
                regime_summary: build_regime_aggregate(&[]),
                risk_governor_summary: build_risk_governor_aggregate(&[], &[]),
                model_comparison_summary: build_model_comparison_aggregate(&[]),
                persona_readiness_summary: build_persona_readiness_summary(&[]),
                expansion_readiness: build_expansion_readiness_report(
                    &[],
                    &[],
                    &build_data_quality_aggregate(&[]),
                    &build_risk_governor_aggregate(&[], &[]),
                    &build_model_comparison_aggregate(&[]),
                    &build_persona_readiness_summary(&[]),
                ),
                reason_codes: invalid,
            };
        }

        for dataset in &config.dataset_bundle.entries {
            if !dataset.enabled {
                run_summaries.push(skipped_summary(
                    config,
                    dataset,
                    "dataset-disabled",
                    vec![ReasonCode::DatasetDisabled],
                ));
                continue;
            }
            if dataset.data_path.contains("://") {
                run_summaries.push(skipped_summary(
                    config,
                    dataset,
                    "dataset-remote-path",
                    vec![ReasonCode::LocalPathRejected],
                ));
                continue;
            }

            for variant in &config.variants {
                if !variant.enabled {
                    run_summaries.push(skipped_summary(
                        config,
                        dataset,
                        &variant.variant_id,
                        vec![ReasonCode::VariantDisabled],
                    ));
                    continue;
                }

                let experiment_config = config.build_experiment_config(dataset, variant);
                let bundle = self.experiment_runner.run(&experiment_config);
                let summary = summarize_run(&dataset.dataset_id, &variant.variant_id, &bundle);
                let failed = matches!(summary.status, ExperimentRunStatus::Failed);
                run_summaries.push(summary);
                bundle_refs.push((dataset.dataset_id.clone(), bundle));
                if failed && !config.continue_on_failure {
                    reason_codes.push(ReasonCode::BatchRunFailed);
                    break;
                }
            }
            if reason_codes.contains(&ReasonCode::BatchRunFailed) && !config.continue_on_failure {
                break;
            }
        }

        if config.require_all_pass
            && run_summaries
                .iter()
                .any(|summary| matches!(summary.status, ExperimentRunStatus::Failed))
        {
            reason_codes.push(ReasonCode::MatrixRequireAllPassFailed);
        }

        let bundle_dataset_refs = bundle_refs
            .iter()
            .map(|(dataset_id, bundle)| (dataset_id.clone(), bundle))
            .collect::<Vec<_>>();
        let distinct_dataset_bundles = config
            .dataset_bundle
            .entries
            .iter()
            .filter_map(|entry| {
                bundle_refs
                    .iter()
                    .find(|(dataset_id, _)| dataset_id == &entry.dataset_id)
                    .map(|(_, bundle)| (entry.dataset_id.clone(), bundle))
            })
            .collect::<Vec<_>>();
        let bundle_only_refs = bundle_refs
            .iter()
            .map(|(_, bundle)| bundle)
            .collect::<Vec<_>>();
        let aggregate_benchmark = build_aggregate_benchmark(&run_summaries);
        let data_quality_summary = build_data_quality_aggregate(&distinct_dataset_bundles);
        let regime_summary = build_regime_aggregate(&bundle_only_refs);
        let risk_governor_summary =
            build_risk_governor_aggregate(&run_summaries, &bundle_dataset_refs);
        let model_comparison_summary = build_model_comparison_aggregate(&bundle_only_refs);
        let persona_readiness_summary = build_persona_readiness_summary(&bundle_only_refs);
        let expansion_readiness = build_expansion_readiness_report(
            &run_summaries,
            &bundle_only_refs,
            &data_quality_summary,
            &risk_governor_summary,
            &model_comparison_summary,
            &persona_readiness_summary,
        );

        let report = BatchExperimentReport {
            matrix_id: config.matrix_id.clone(),
            run_summaries,
            aggregate_benchmark,
            data_quality_summary,
            regime_summary,
            risk_governor_summary,
            model_comparison_summary,
            persona_readiness_summary,
            expansion_readiness,
            reason_codes,
        };
        let _ = write_batch_outputs(config, &report);
        report
    }
}

fn skipped_summary(
    config: &ExperimentMatrixConfig,
    dataset: &DatasetEntry,
    variant_id: &str,
    reason_codes: Vec<ReasonCode>,
) -> super::aggregate::ExperimentRunSummary {
    super::aggregate::ExperimentRunSummary {
        run_key: super::aggregate::ExperimentRunKey {
            dataset_id: dataset.dataset_id.clone(),
            variant_id: variant_id.to_string(),
            experiment_id: format!("{}-{}-{variant_id}", config.matrix_id, dataset.dataset_id),
        },
        status: ExperimentRunStatus::Skipped,
        manifest_summary: String::new(),
        data_quality_score: 0.0,
        data_quality_severity: crate::data::DataQualitySeverity::Unusable,
        total_decisions: 0,
        executed_trades: 0,
        denied_trades: 0,
        no_trades: 0,
        net_return_pct: 0.0,
        max_drawdown_pct: 0.0,
        profit_factor: None,
        calibration_brier: None,
        risk_defensive_value: None,
        external_better: None,
        reason_codes,
    }
}

fn write_batch_outputs(
    config: &ExperimentMatrixConfig,
    report: &BatchExperimentReport,
) -> Result<(), String> {
    let output_dir = Path::new(&config.dataset_bundle.output_root).join(&config.matrix_id);
    fs::create_dir_all(&output_dir).map_err(|err| err.to_string())?;
    fs::write(
        output_dir.join("batch_summary.txt"),
        batch_summary_to_string(report),
    )
    .map_err(|err| err.to_string())
}

fn batch_summary_to_string(report: &BatchExperimentReport) -> String {
    let mut lines = vec![
        format!("matrix_id={}", report.matrix_id),
        report.aggregate_benchmark.to_markdown_table_string(),
        format!(
            "expansion_decision={:?}",
            report.expansion_readiness.decision
        ),
    ];
    for summary in &report.run_summaries {
        lines.push(format!(
            "run={}/{}/{}:{:?}:{:.6}:{:.6}",
            summary.run_key.dataset_id,
            summary.run_key.variant_id,
            summary.run_key.experiment_id,
            summary.status,
            summary.net_return_pct,
            summary.max_drawdown_pct
        ));
    }
    lines.join("\n")
}
