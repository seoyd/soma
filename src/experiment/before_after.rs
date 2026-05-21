use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;

use super::ablation::AblationStudyReport;
use super::decision_router::{Sprint14DecisionRecord, Sprint14Track};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Sprint14ComparableSummary {
    pub study_id: String,
    pub selected_track: Option<Sprint14Track>,
    pub dataset_count: usize,
    pub usable_dataset_count: usize,
    pub total_outcome_records: usize,
    pub comparable_variant_count: usize,
    pub average_data_quality_score: f64,
    pub no_runtime_llm: bool,
    pub no_live_api: bool,
    pub no_real_broker: bool,
    pub no_real_order_execution: bool,
    pub no_new_personas: bool,
    pub reason_codes: Vec<ReasonCode>,
}

impl Sprint14ComparableSummary {
    pub fn from_ablation_report(report: &AblationStudyReport) -> Self {
        let baseline = &report.baseline.report.expansion_readiness.evidence;
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
        Self {
            study_id: report.study_id.clone(),
            selected_track: None,
            dataset_count: baseline.dataset_count,
            usable_dataset_count: baseline.usable_dataset_count,
            total_outcome_records: baseline.total_outcome_records,
            comparable_variant_count,
            average_data_quality_score: report
                .baseline
                .report
                .aggregate_benchmark
                .avg_data_quality_score,
            no_runtime_llm: true,
            no_live_api: true,
            no_real_broker: true,
            no_real_order_execution: true,
            no_new_personas: true,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Sprint14BeforeAfterReport {
    pub before_summary: Sprint14ComparableSummary,
    pub after_summary: Sprint14ComparableSummary,
    pub comparable: bool,
    pub safety_regressions: Vec<String>,
    pub improvements: Vec<String>,
    pub unchanged_metrics: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

pub fn build_before_after_report(
    before_summary: Sprint14ComparableSummary,
    after_summary: Sprint14ComparableSummary,
) -> Sprint14BeforeAfterReport {
    let comparable = before_summary.study_id == after_summary.study_id;
    let mut safety_regressions = Vec::new();
    let mut improvements = Vec::new();
    let mut unchanged_metrics = Vec::new();

    compare_bool(
        "no_runtime_llm",
        before_summary.no_runtime_llm,
        after_summary.no_runtime_llm,
        &mut safety_regressions,
        &mut unchanged_metrics,
    );
    compare_bool(
        "no_live_api",
        before_summary.no_live_api,
        after_summary.no_live_api,
        &mut safety_regressions,
        &mut unchanged_metrics,
    );
    compare_bool(
        "no_real_broker",
        before_summary.no_real_broker,
        after_summary.no_real_broker,
        &mut safety_regressions,
        &mut unchanged_metrics,
    );
    compare_bool(
        "no_real_order_execution",
        before_summary.no_real_order_execution,
        after_summary.no_real_order_execution,
        &mut safety_regressions,
        &mut unchanged_metrics,
    );
    compare_bool(
        "no_new_personas",
        before_summary.no_new_personas,
        after_summary.no_new_personas,
        &mut safety_regressions,
        &mut unchanged_metrics,
    );

    compare_numeric(
        "dataset_count",
        before_summary.dataset_count,
        after_summary.dataset_count,
        &mut improvements,
        &mut unchanged_metrics,
    );
    compare_numeric(
        "usable_dataset_count",
        before_summary.usable_dataset_count,
        after_summary.usable_dataset_count,
        &mut improvements,
        &mut unchanged_metrics,
    );
    compare_numeric(
        "total_outcome_records",
        before_summary.total_outcome_records,
        after_summary.total_outcome_records,
        &mut improvements,
        &mut unchanged_metrics,
    );
    compare_numeric(
        "comparable_variant_count",
        before_summary.comparable_variant_count,
        after_summary.comparable_variant_count,
        &mut improvements,
        &mut unchanged_metrics,
    );

    if after_summary.average_data_quality_score > before_summary.average_data_quality_score {
        improvements.push("average_data_quality_score_improved".to_string());
    } else if (after_summary.average_data_quality_score - before_summary.average_data_quality_score)
        .abs()
        <= f64::EPSILON
    {
        unchanged_metrics.push("average_data_quality_score".to_string());
    } else {
        safety_regressions.push("average_data_quality_score_regressed".to_string());
    }

    let mut reason_codes = vec![ReasonCode::Sprint14BeforeAfterBuilt];
    if !safety_regressions.is_empty() {
        reason_codes.push(ReasonCode::SafetyRegressionDetected);
    }
    Sprint14BeforeAfterReport {
        before_summary,
        after_summary,
        comparable,
        safety_regressions,
        improvements,
        unchanged_metrics,
        reason_codes,
    }
}

pub fn after_summary_from_decision(
    before_summary: &Sprint14ComparableSummary,
    decision: &Sprint14DecisionRecord,
) -> Sprint14ComparableSummary {
    Sprint14ComparableSummary {
        selected_track: Some(decision.selected_track),
        reason_codes: vec![ReasonCode::DeterministicPath],
        ..before_summary.clone()
    }
}

fn compare_bool(
    label: &str,
    before: bool,
    after: bool,
    safety_regressions: &mut Vec<String>,
    unchanged_metrics: &mut Vec<String>,
) {
    if before == after {
        unchanged_metrics.push(label.to_string());
    } else if before && !after {
        safety_regressions.push(format!("{label}_regressed"));
    }
}

fn compare_numeric(
    label: &str,
    before: usize,
    after: usize,
    improvements: &mut Vec<String>,
    unchanged_metrics: &mut Vec<String>,
) {
    if after > before {
        improvements.push(format!("{label}_improved"));
    } else if after == before {
        unchanged_metrics.push(label.to_string());
    }
}
