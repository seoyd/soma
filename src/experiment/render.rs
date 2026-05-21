use super::ExternalTabularBenchmarkStage;
use super::ablation::{AblationStudyReport, AblationVariantResult};
use super::ai_benchmark::OfficialAiBenchmarkReport;
use super::ai_usefulness::AiSignalUsefulnessReport;
use super::auth_setup::AuthSetupGuide;
use super::before_after::Sprint14BeforeAfterReport;
use super::campaign::{CampaignAggregate, ResearchCampaignReport};
use super::core_benchmark::{CoreCheckedBenchmarkReport, SelectedOfficialDatasets};
use super::decision_router::Sprint14DecisionRecord;
use super::diff::CampaignDiffReport;
use super::evidence_closure::{
    DatasetEvidenceSource, EvidenceClosureReport, MinimumEvidencePlanUpdate,
};
use super::evidence_delta::OfficialEvidenceDelta;
use super::evidence_gap::EvidenceGapReport;
use super::executable_evidence_plan::ExecutableEvidencePlan;
use super::kis_auth_readiness::KISAuthReadinessReport;
use super::kis_candle_sufficiency::KISCandleSufficiencyReport;
use super::kis_canonical_batch_validation::KISCanonicalBatchValidationReport;
use super::kis_collection_batch::KISCollectionBatchPlan;
use super::kis_endpoint_policy::KISEndpointPolicyReport;
use super::kis_krx_migration::KISKRXMigrationReport;
use super::kis_market_data_activation::KISOfficialMarketDataActivationReport;
use super::kis_outcome_link_closure::KISOutcomeLinkClosureReport;
use super::kis_symbol_whitelist::KISSymbolWhitelist;
use super::krx_auth_readiness::KRXAuthReadinessReport;
use super::krx_candle_sufficiency::KRXCandleSufficiencyReport;
use super::krx_canonical_batch_validation::KRXCanonicalBatchValidationReport;
use super::krx_canonical_validation::KRXCanonicalValidationReport;
use super::krx_collection_batch::KRXCollectionBatchPlan;
use super::krx_collection_closure::KRXOfficialCollectionClosureReport;
use super::krx_collection_smoke::KRXCollectionDryRunReport;
use super::krx_downstream_rerun::KRXDownstreamRerunSummary;
use super::krx_downstream_rerun_v2::KRXDownstreamRerunV2Summary;
use super::krx_evidence_job::KRXEvidenceJobPlan;
use super::krx_official_activation::KRXOfficialEvidenceActivationReport;
use super::krx_outcome_link_closure::KRXOutcomeLinkClosureReport;
use super::krx_raw_archive::KRXRawResponseArchiveSummary;
use super::krx_schema_drift::KRXResponseSchemaDriftReport;
use super::krx_symbol_whitelist::KRXSymbolWhitelist;
use super::model_gates::ModelUsefulnessGateResult;
use super::official_acquisition::OfficialEvidenceAcquisitionReport;
use super::official_coverage::OfficialDatasetCoverageReport;
use super::official_expansion::OfficialEvidenceExpansionReport;
use super::official_vs_yfinance::OfficialVsYFinanceInterpretation;
use super::operator_action::OperatorActionPlan;
use super::previous_collection::PreviousCollectionComparison;
use super::provider_readiness::OfficialProviderReadinessReport;
use super::provider_reality::ProviderRealityReport;
use super::provider_reality_executor::ProviderRealityEvidenceReport;
use super::readiness::CampaignExpansionReadinessReport;
use super::readiness_matrix::EvidenceReadinessMatrix;
use super::real_evidence::{RealEvidenceClosureReport, RealEvidencePlanUpdate};
use super::risk_ai_interaction::RiskAiInteractionReport;
use super::sensitivity::SensitivitySummary;
use super::source_benchmark::SourceAwareBenchmarkReport;
use super::sprint14::{Sprint14Report, Sprint14TrackSpecificReport};
use super::storage_delta::OfficialStorageDelta;
use super::venue_coverage::VenueCoverageExpansionReport;
use super::yahoo_research::YahooResearchEvidenceReport;
use crate::data::ProviderAuthPreflightReport;

pub fn official_ai_benchmark_report_to_text(report: &OfficialAiBenchmarkReport) -> String {
    report.to_text()
}

pub fn core_checked_benchmark_report_to_text(report: &CoreCheckedBenchmarkReport) -> String {
    report.to_text()
}

pub fn core_checked_benchmark_report_to_markdown(report: &CoreCheckedBenchmarkReport) -> String {
    report.to_markdown()
}

pub fn dataset_selection_to_text(report: &SelectedOfficialDatasets) -> String {
    super::core_benchmark::dataset_selection_to_text(report)
}

pub fn external_tabular_stage_to_text(stage: &ExternalTabularBenchmarkStage) -> String {
    super::core_benchmark::external_tabular_stage_to_text(stage)
}

pub fn ai_signal_usefulness_report_to_markdown(report: &AiSignalUsefulnessReport) -> String {
    report.to_markdown()
}

pub fn model_usefulness_gate_report_to_text(report: &ModelUsefulnessGateResult) -> String {
    report.to_text()
}

pub fn official_dataset_coverage_to_text(report: &OfficialDatasetCoverageReport) -> String {
    report.to_text()
}

pub fn provider_auth_preflight_report_to_text(report: &ProviderAuthPreflightReport) -> String {
    report.to_text()
}

pub fn provider_readiness_report_to_text(report: &OfficialProviderReadinessReport) -> String {
    report.to_text()
}

pub fn provider_reality_report_to_text(report: &ProviderRealityReport) -> String {
    report.to_text()
}

pub fn executable_evidence_plan_to_text(report: &ExecutableEvidencePlan) -> String {
    report.to_text()
}

pub fn provider_reality_evidence_report_to_text(report: &ProviderRealityEvidenceReport) -> String {
    report.to_text()
}

pub fn krx_auth_readiness_report_to_text(report: &KRXAuthReadinessReport) -> String {
    report.to_text()
}

pub fn kis_auth_readiness_report_to_text(report: &KISAuthReadinessReport) -> String {
    report.to_text()
}

pub fn krx_collection_dry_run_to_text(report: &KRXCollectionDryRunReport) -> String {
    report.to_text()
}

pub fn kis_endpoint_policy_to_text(report: &KISEndpointPolicyReport) -> String {
    report.to_text()
}

pub fn krx_symbol_whitelist_to_text(report: &KRXSymbolWhitelist) -> String {
    report.to_text()
}

pub fn kis_symbol_whitelist_to_text(report: &KISSymbolWhitelist) -> String {
    report.to_text()
}

pub fn krx_evidence_job_plan_to_text(report: &KRXEvidenceJobPlan) -> String {
    report.to_text()
}

pub fn krx_collection_batch_plan_to_text(report: &KRXCollectionBatchPlan) -> String {
    report.to_text()
}

pub fn kis_collection_batch_plan_to_text(report: &KISCollectionBatchPlan) -> String {
    report.to_text()
}

pub fn krx_canonical_validation_to_text(report: &KRXCanonicalValidationReport) -> String {
    report.to_text()
}

pub fn krx_canonical_batch_validation_to_text(
    report: &KRXCanonicalBatchValidationReport,
) -> String {
    report.to_text()
}

pub fn kis_canonical_batch_validation_to_text(
    report: &KISCanonicalBatchValidationReport,
) -> String {
    report.to_text()
}

pub fn krx_candle_sufficiency_to_text(report: &KRXCandleSufficiencyReport) -> String {
    report.to_text()
}

pub fn kis_candle_sufficiency_to_text(report: &KISCandleSufficiencyReport) -> String {
    report.to_text()
}

pub fn krx_downstream_rerun_to_text(report: &KRXDownstreamRerunSummary) -> String {
    report.to_text()
}

pub fn krx_downstream_rerun_v2_to_text(report: &KRXDownstreamRerunV2Summary) -> String {
    report.to_text()
}

pub fn krx_official_activation_report_to_text(
    report: &KRXOfficialEvidenceActivationReport,
) -> String {
    report.to_text()
}

pub fn kis_official_activation_report_to_text(
    report: &KISOfficialMarketDataActivationReport,
) -> String {
    report.to_text()
}

pub fn krx_raw_archive_summary_to_text(report: &KRXRawResponseArchiveSummary) -> String {
    report.to_text()
}

pub fn krx_schema_drift_report_to_text(report: &KRXResponseSchemaDriftReport) -> String {
    report.to_text()
}

pub fn kis_krx_migration_to_text(report: &KISKRXMigrationReport) -> String {
    report.to_text()
}

pub fn krx_outcome_link_closure_to_text(report: &KRXOutcomeLinkClosureReport) -> String {
    report.to_text()
}

pub fn kis_outcome_link_closure_to_text(report: &KISOutcomeLinkClosureReport) -> String {
    report.to_text()
}

pub fn krx_collection_closure_report_to_text(
    report: &KRXOfficialCollectionClosureReport,
) -> String {
    report.to_text()
}

pub fn readiness_matrix_to_text(report: &EvidenceReadinessMatrix) -> String {
    report.to_text()
}

pub fn venue_coverage_report_to_text(report: &VenueCoverageExpansionReport) -> String {
    report.to_text()
}

pub fn official_evidence_delta_to_text(report: &OfficialEvidenceDelta) -> String {
    report.to_text()
}

pub fn official_storage_delta_to_text(report: &OfficialStorageDelta) -> String {
    report.to_text()
}

pub fn auth_setup_guide_to_text(report: &AuthSetupGuide) -> String {
    report.to_text()
}

pub fn official_evidence_expansion_report_to_text(
    report: &OfficialEvidenceExpansionReport,
) -> String {
    report.to_text()
}

pub fn official_evidence_expansion_report_to_markdown(
    report: &OfficialEvidenceExpansionReport,
) -> String {
    report.to_markdown()
}

pub fn previous_collection_comparison_to_text(report: &PreviousCollectionComparison) -> String {
    report.to_text()
}

pub fn operator_action_plan_to_text(report: &OperatorActionPlan) -> String {
    report.to_text()
}

pub fn official_evidence_acquisition_report_to_text(
    report: &OfficialEvidenceAcquisitionReport,
) -> String {
    report.to_text()
}

pub fn official_evidence_acquisition_report_to_markdown(
    report: &OfficialEvidenceAcquisitionReport,
) -> String {
    report.to_markdown()
}

pub fn yahoo_research_report_to_text(report: &YahooResearchEvidenceReport) -> String {
    report.to_text()
}

pub fn official_vs_yfinance_to_text(report: &OfficialVsYFinanceInterpretation) -> String {
    report.to_text()
}

pub fn source_aware_benchmark_report_to_text(report: &SourceAwareBenchmarkReport) -> String {
    report.to_text()
}

pub fn risk_ai_interaction_report_to_text(report: &RiskAiInteractionReport) -> String {
    report.to_text()
}

pub fn campaign_summary_to_text(report: &ResearchCampaignReport) -> String {
    [
        format!("campaign_id={}", report.campaign_id),
        report.aggregate.to_markdown_table_string(),
        format!("readiness_decision={:?}", report.readiness_report.decision),
        format!("diff_comparable={}", report.diff_report.comparable),
    ]
    .join("\n")
}

pub fn campaign_summary_to_markdown_table(aggregate: &CampaignAggregate) -> String {
    aggregate.to_markdown_table_string()
}

pub fn diff_report_to_text(diff: &CampaignDiffReport) -> String {
    let mut lines = vec![
        format!("current_campaign_id={}", diff.current_campaign_id),
        format!(
            "previous_campaign_id={}",
            diff.previous_campaign_id.as_deref().unwrap_or("")
        ),
        format!("comparable={}", diff.comparable),
        format!("delta_passed_runs={}", diff.metric_deltas.delta_passed_runs),
        format!(
            "delta_usable_dataset_count={}",
            diff.metric_deltas.delta_usable_dataset_count
        ),
        format!(
            "delta_outcome_records={}",
            diff.metric_deltas.delta_outcome_records
        ),
        format!(
            "delta_avg_net_return_pct={:.8}",
            diff.metric_deltas.delta_avg_net_return_pct
        ),
        format!(
            "delta_worst_drawdown_pct={:.8}",
            diff.metric_deltas.delta_worst_drawdown_pct
        ),
        format!(
            "delta_avg_calibration_brier={:.8}",
            diff.metric_deltas.delta_avg_calibration_brier
        ),
        format!(
            "delta_data_quality_score={:.8}",
            diff.metric_deltas.delta_data_quality_score
        ),
    ];
    for regression in &diff.regressions {
        lines.push(format!("regression={regression:?}"));
    }
    for improvement in &diff.improvements {
        lines.push(format!("improvement={improvement:?}"));
    }
    lines.join("\n")
}

pub fn readiness_report_to_text(report: &CampaignExpansionReadinessReport) -> String {
    let mut lines = vec![
        format!("decision={:?}", report.decision),
        format!("confidence={:.8}", report.confidence),
    ];
    for blocker in &report.blockers {
        lines.push(format!("blocker={blocker}"));
    }
    for warning in &report.warnings {
        lines.push(format!("warning={warning}"));
    }
    lines.join("\n")
}

pub fn sensitivity_summary_to_text(summary: &SensitivitySummary) -> String {
    let mut lines = vec![format!(
        "dominant_dimension={}",
        summary
            .dominant_dimension
            .map(|dimension| format!("{dimension:?}"))
            .unwrap_or_default()
    )];
    for dimension in &summary.dimensions {
        lines.push(format!(
            "dimension={:?};comparable={};candidate_improvements={};fragile={};research_only={};max_abs_avg_net_return_delta={:.8};max_abs_avg_drawdown_delta={:.8};max_abs_avg_calibration_brier_delta={:.8}",
            dimension.dimension,
            dimension.comparable_count,
            dimension.candidate_improvement_count,
            dimension.fragile_count,
            dimension.research_only_count,
            dimension.max_abs_avg_net_return_delta,
            dimension.max_abs_avg_drawdown_delta,
            dimension.max_abs_avg_calibration_brier_delta,
        ));
    }
    lines.join("\n")
}

pub fn ablation_report_to_text(report: &AblationStudyReport) -> String {
    let mut lines = vec![
        format!("study_id={}", report.study_id),
        format!("baseline_matrix_id={}", report.baseline.matrix_id),
        format!(
            "baseline_avg_net_return_pct={:.8}",
            report
                .baseline
                .report
                .aggregate_benchmark
                .avg_net_return_pct
        ),
        format!("next_step={:?}", report.next_step),
    ];
    lines.push(sensitivity_summary_to_text(&report.sensitivity_summary));
    for variant in &report.variants {
        lines.push(variant_summary_line(variant));
    }
    for warning in &report.warnings {
        lines.push(format!("warning={warning}"));
    }
    lines.join("\n")
}

pub fn ablation_summary_to_markdown_table(report: &AblationStudyReport) -> String {
    let mut lines = vec![
        "| variant | dimension | status | delta_avg_net_return_pct | delta_avg_max_drawdown_pct | delta_avg_calibration_brier | flags |".to_string(),
        "| --- | --- | --- | ---: | ---: | ---: | --- |".to_string(),
        format!(
            "| baseline | baseline | Applied | {:.8} | {:.8} | {} | - |",
            0.0,
            0.0,
            report
                .baseline
                .avg_calibration_brier
                .map(|_| "0.00000000".to_string())
                .unwrap_or_default()
        ),
    ];
    for variant in &report.variants {
        lines.push(format!(
            "| {} | {:?} | {:?} | {:.8} | {:.8} | {} | {} |",
            variant.variant_id,
            variant.dimension,
            variant.status,
            variant.delta.avg_net_return_pct,
            variant.delta.avg_max_drawdown_pct,
            variant
                .delta
                .avg_calibration_brier
                .map(|value| format!("{value:.8}"))
                .unwrap_or_default(),
            variant
                .flags
                .iter()
                .map(|flag| format!("{flag:?}"))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    lines.join("\n")
}

fn variant_summary_line(variant: &AblationVariantResult) -> String {
    format!(
        "variant={};dimension={:?};status={:?};delta_avg_net_return_pct={:.8};delta_avg_max_drawdown_pct={:.8};delta_avg_calibration_brier={};flags={}",
        variant.variant_id,
        variant.dimension,
        variant.status,
        variant.delta.avg_net_return_pct,
        variant.delta.avg_max_drawdown_pct,
        variant
            .delta
            .avg_calibration_brier
            .map(|value| format!("{value:.8}"))
            .unwrap_or_default(),
        variant
            .flags
            .iter()
            .map(|flag| format!("{flag:?}"))
            .collect::<Vec<_>>()
            .join("|")
    )
}

pub fn sprint14_report_to_text(report: &Sprint14Report) -> String {
    let mut lines = vec![
        format!("selected_track={:?}", report.decision_record.selected_track),
        sprint14_decision_to_text(&report.decision_record),
        sprint14_before_after_to_text(&report.before_after_report),
    ];
    match &report.track_specific_report {
        Sprint14TrackSpecificReport::NeedMoreExperiments(gap) => {
            lines.push(evidence_gap_report_to_text(gap));
        }
    }
    lines.push(format!(
        "next_recommendation={}",
        report.next_recommendation
    ));
    lines.join("\n")
}

pub fn sprint14_report_to_markdown(report: &Sprint14Report) -> String {
    let mut lines = vec![
        "| section | value |".to_string(),
        "| --- | --- |".to_string(),
        format!(
            "| selected_track | {:?} |",
            report.decision_record.selected_track
        ),
        format!("| decision_reason | {} |", report.decision_record.reason),
        format!("| comparable | {} |", report.before_after_report.comparable),
        format!(
            "| safety_regressions | {} |",
            report.before_after_report.safety_regressions.join(", ")
        ),
        format!("| next_recommendation | {} |", report.next_recommendation),
    ];
    let Sprint14TrackSpecificReport::NeedMoreExperiments(gap) = &report.track_specific_report;
    lines.push(format!(
        "| evidence_gap_blocked_expansion | {} |",
        gap.minimum_evidence_plan.blocked_expansion
    ));
    lines.join("\n")
}

pub fn sprint14_decision_to_text(record: &Sprint14DecisionRecord) -> String {
    let mut lines = vec![
        format!("decision_reason={}", record.reason),
        format!(
            "source_study_id={}",
            record
                .evidence_inputs
                .source_study_id
                .as_deref()
                .unwrap_or("")
        ),
    ];
    for rejected in &record.rejected_tracks {
        lines.push(format!(
            "rejected_track={:?}:{}",
            rejected.track, rejected.reason
        ));
    }
    for blocker in &record.blockers {
        lines.push(format!("blocker={blocker}"));
    }
    for warning in &record.warnings {
        lines.push(format!("warning={warning}"));
    }
    lines.join("\n")
}

pub fn sprint14_before_after_to_text(report: &Sprint14BeforeAfterReport) -> String {
    let mut lines = vec![
        format!("before.study_id={}", report.before_summary.study_id),
        format!("after.study_id={}", report.after_summary.study_id),
        format!("comparable={}", report.comparable),
    ];
    for regression in &report.safety_regressions {
        lines.push(format!("safety_regression={regression}"));
    }
    for improvement in &report.improvements {
        lines.push(format!("improvement={improvement}"));
    }
    for unchanged in &report.unchanged_metrics {
        lines.push(format!("unchanged={unchanged}"));
    }
    lines.join("\n")
}

pub fn evidence_gap_report_to_text(report: &EvidenceGapReport) -> String {
    let mut lines = vec![
        format!("evidence_gap.study_id={}", report.study_id),
        format!("evidence_gap.insufficient={}", report.insufficient_evidence),
    ];
    for item in &report.checklist {
        lines.push(format!(
            "checklist={}:current={}:required={}:satisfied={}",
            item.label, item.current, item.required, item.satisfied
        ));
    }
    lines.push(format!(
        "minimum_plan.additional_usable_datasets_needed={}",
        report
            .minimum_evidence_plan
            .additional_usable_datasets_needed
    ));
    lines.push(format!(
        "minimum_plan.additional_outcome_records_needed={}",
        report
            .minimum_evidence_plan
            .additional_outcome_records_needed
    ));
    lines.push(format!(
        "minimum_plan.additional_comparable_variants_needed={}",
        report
            .minimum_evidence_plan
            .additional_comparable_variants_needed
    ));
    lines.extend(
        report
            .warnings
            .iter()
            .map(|warning| format!("warning={warning}")),
    );
    lines.join("\n")
}

pub fn minimum_evidence_plan_update_to_text(plan: &MinimumEvidencePlanUpdate) -> String {
    let mut lines = Vec::new();
    for item in &plan.previous_plan {
        lines.push(format!("previous_plan={item}"));
    }
    for item in &plan.completed_items {
        lines.push(format!("completed={item}"));
    }
    for item in &plan.remaining_items {
        lines.push(format!("remaining={item}"));
    }
    for item in &plan.next_required_items {
        lines.push(format!("next_required={item}"));
    }
    lines.join("\n")
}

pub fn evidence_closure_report_to_text(report: &EvidenceClosureReport) -> String {
    let mut lines = vec![
        format!("closure_id={}", report.closure_id),
        format!("readiness_before={:?}", report.readiness_before),
        format!("readiness_after={:?}", report.readiness_after),
        format!("final_recommendation={:?}", report.final_recommendation),
        format!(
            "closure_status.all_targets_closed={}",
            report.closure_status.all_targets_closed
        ),
        format!(
            "closure_status.partially_closed={}",
            report.closure_status.partially_closed
        ),
        format!(
            "target=usable_datasets:before={}:after={}:added={}:required={}:closed={}",
            report
                .closure_status
                .usable_dataset_target
                .current_before_count,
            report
                .closure_status
                .usable_dataset_target
                .current_after_count,
            report.closure_status.usable_dataset_target.added_count,
            report.closure_status.usable_dataset_target.required_count,
            report.closure_status.usable_dataset_target.closed
        ),
        format!(
            "target=outcome_records:before={}:after={}:added={}:required={}:closed={}",
            report
                .closure_status
                .outcome_record_target
                .current_before_count,
            report
                .closure_status
                .outcome_record_target
                .current_after_count,
            report.closure_status.outcome_record_target.added_count,
            report.closure_status.outcome_record_target.required_count,
            report.closure_status.outcome_record_target.closed
        ),
        format!(
            "target=comparable_variants:before={}:after={}:added={}:required={}:closed={}",
            report
                .closure_status
                .comparable_variant_target
                .current_before_count,
            report
                .closure_status
                .comparable_variant_target
                .current_after_count,
            report.closure_status.comparable_variant_target.added_count,
            report
                .closure_status
                .comparable_variant_target
                .required_count,
            report.closure_status.comparable_variant_target.closed
        ),
    ];
    for dataset in &report.added_dataset_summaries {
        lines.push(format!(
            "dataset={};source={};quality={:?};score={:.8};usable={}",
            dataset.dataset_id,
            dataset_source_label(dataset.source),
            dataset.data_quality_severity,
            dataset.data_quality_score,
            dataset.counted_as_usable
        ));
    }
    lines.push(format!(
        "added_outcomes={};executed={};no_trade={};denied={}",
        report.added_outcome_summary.additional_outcome_records,
        report.added_outcome_summary.executed_trades,
        report.added_outcome_summary.no_trades,
        report.added_outcome_summary.denied_trades
    ));
    lines.push(format!(
        "added_comparable_variants={};variant_ids={}",
        report.added_variant_summary.additional_comparable_variants,
        report
            .added_variant_summary
            .comparable_variant_ids
            .join("|")
    ));
    for blocker in &report.blockers {
        lines.push(format!("blocker={blocker}"));
    }
    for warning in &report.warnings {
        lines.push(format!("warning={warning}"));
    }
    lines.push(minimum_evidence_plan_update_to_text(
        &report.minimum_plan_update,
    ));
    lines.join("\n")
}

pub fn evidence_closure_report_to_markdown(report: &EvidenceClosureReport) -> String {
    let mut lines = vec![
        "| section | value |".to_string(),
        "| --- | --- |".to_string(),
        format!("| readiness_before | {:?} |", report.readiness_before),
        format!("| readiness_after | {:?} |", report.readiness_after),
        format!(
            "| final_recommendation | {:?} |",
            report.final_recommendation
        ),
        format!(
            "| usable_datasets_added | {} / {} |",
            report.closure_status.usable_dataset_target.added_count,
            report.closure_status.usable_dataset_target.required_count
        ),
        format!(
            "| outcome_records_added | {} / {} |",
            report.closure_status.outcome_record_target.added_count,
            report.closure_status.outcome_record_target.required_count
        ),
        format!(
            "| comparable_variants_added | {} / {} |",
            report.closure_status.comparable_variant_target.added_count,
            report
                .closure_status
                .comparable_variant_target
                .required_count
        ),
        format!(
            "| all_targets_closed | {} |",
            report.closure_status.all_targets_closed
        ),
    ];
    for dataset in &report.added_dataset_summaries {
        lines.push(format!(
            "| dataset:{} | {} / {:?} / usable={} |",
            dataset.dataset_id,
            dataset_source_label(dataset.source),
            dataset.data_quality_severity,
            dataset.counted_as_usable
        ));
    }
    lines.push(format!(
        "| synthetic_fixture_note | {} |",
        if report
            .added_dataset_summaries
            .iter()
            .any(|dataset| dataset.source == DatasetEvidenceSource::SyntheticFixture)
        {
            "pipeline-only evidence"
        } else {
            "not used"
        }
    ));
    lines.join("\n")
}

fn dataset_source_label(source: DatasetEvidenceSource) -> &'static str {
    match source {
        DatasetEvidenceSource::RealLocalData => "real-local-data",
        DatasetEvidenceSource::SyntheticFixture => "synthetic-fixture",
        DatasetEvidenceSource::TestFixture => "test-fixture",
        DatasetEvidenceSource::UnknownSource => "unknown-source",
    }
}

pub fn real_evidence_plan_update_to_text(plan: &RealEvidencePlanUpdate) -> String {
    let mut lines = Vec::new();
    for item in &plan.previous_sprint15_targets {
        lines.push(format!("previous_sprint15_target={item}"));
    }
    lines.push(format!(
        "sprint15_status={}",
        plan.sprint15_synthetic_closure_status
    ));
    for item in &plan.real_evidence_targets {
        lines.push(format!("real_target={item}"));
    }
    for item in &plan.completed_real_items {
        lines.push(format!("completed_real={item}"));
    }
    for item in &plan.remaining_real_items {
        lines.push(format!("remaining_real={item}"));
    }
    for item in &plan.next_required_items {
        lines.push(format!("next_required={item}"));
    }
    lines.join("\n")
}

pub fn real_evidence_report_to_text(report: &RealEvidenceClosureReport) -> String {
    let mut lines = vec![
        format!("closure_id={}", report.closure_id),
        format!("readiness_before={}", report.readiness_before),
        format!("readiness_after={}", report.readiness_after),
        format!("final_recommendation={:?}", report.final_recommendation),
        format!(
            "real_status.datasets={}",
            report.real_only_evidence_status.dataset_count
        ),
        format!(
            "real_status.outcomes={}",
            report.real_only_evidence_status.outcome_count
        ),
        format!(
            "real_status.variants={}",
            report.real_only_evidence_status.comparable_variant_count
        ),
        format!(
            "real_status.all_targets_closed={}",
            report.real_only_evidence_status.all_targets_closed
        ),
    ];
    for dataset in &report.real_local_dataset_summaries {
        lines.push(format!(
            "real_dataset={};quality={:?};score={:.8};eligible={}",
            dataset.dataset_id,
            dataset.data_quality_severity,
            dataset.data_quality_score,
            dataset.readiness_eligible
        ));
    }
    for dataset in &report.synthetic_dataset_summaries {
        lines.push(format!(
            "non_real_dataset={};source={:?};quality={:?};score={:.8}",
            dataset.dataset_id,
            dataset.source_kind,
            dataset.data_quality_severity,
            dataset.data_quality_score
        ));
    }
    if let Some(comparison) = &report.synthetic_vs_real_comparison {
        lines.push(format!(
            "synthetic_vs_real.comparable={}",
            comparison.comparable
        ));
        lines.push(format!(
            "synthetic_vs_real.delta_net_return_pct={:.8}",
            comparison.delta_net_return_pct
        ));
        lines.push(format!(
            "synthetic_vs_real.delta_max_drawdown_pct={:.8}",
            comparison.delta_max_drawdown_pct
        ));
    }
    for blocker in &report.blockers {
        lines.push(format!("blocker={blocker}"));
    }
    for warning in &report.warnings {
        lines.push(format!("warning={warning}"));
    }
    lines.push(real_evidence_plan_update_to_text(
        &report.real_evidence_plan_update,
    ));
    lines.join("\n")
}

pub fn real_evidence_report_to_markdown(report: &RealEvidenceClosureReport) -> String {
    let mut lines = vec![
        "| section | value |".to_string(),
        "| --- | --- |".to_string(),
        format!("| readiness_before | {} |", report.readiness_before),
        format!("| readiness_after | {} |", report.readiness_after),
        format!(
            "| final_recommendation | {:?} |",
            report.final_recommendation
        ),
        format!(
            "| real_dataset_count | {} / {} |",
            report.real_only_evidence_status.dataset_count,
            report.real_only_evidence_status.required_dataset_count
        ),
        format!(
            "| real_outcome_count | {} / {} |",
            report.real_only_evidence_status.outcome_count,
            report.real_only_evidence_status.required_outcome_count
        ),
        format!(
            "| real_variant_count | {} / {} |",
            report.real_only_evidence_status.comparable_variant_count,
            report
                .real_only_evidence_status
                .required_comparable_variant_count
        ),
        format!(
            "| real_targets_closed | {} |",
            report.real_only_evidence_status.all_targets_closed
        ),
    ];
    if let Some(comparison) = &report.synthetic_vs_real_comparison {
        lines.push(format!(
            "| synthetic_vs_real_comparable | {} |",
            comparison.comparable
        ));
    }
    lines.join("\n")
}
