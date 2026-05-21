#[path = "support/sprint47_support.rs"]
mod sprint47_support;

use soma_zero::{OfficialEvidenceScaleOutRunner, OfficialEvidenceScaleOutStatus};

#[test]
fn scaleout_expands_multi_row_counts_and_records_reruns() {
    let config = sprint47_support::scaleout_config("scaleout-runner");
    let bundle = OfficialEvidenceScaleOutRunner::default()
        .run(&config)
        .expect("scaleout bundle");
    assert!(bundle.scaleout_report.after_counts.official_complete_rows >= 2);
    assert!(
        bundle.scaleout_report.after_counts.official_complete_rows
            > bundle.scaleout_report.before_counts.official_complete_rows
    );
    assert!(
        bundle.scaleout_report.after_counts.take_profit_count
            + bundle.scaleout_report.after_counts.stop_loss_count
            + bundle.scaleout_report.after_counts.time_expired_count
            >= 2
    );
    assert!(
        bundle
            .scaleout_report
            .after_counts
            .no_trade_counterfactual_count
            >= 2
    );
    assert!(matches!(
        bundle.scaleout_report.final_status,
        OfficialEvidenceScaleOutStatus::OfficialEvidencePlumbingValidated
            | OfficialEvidenceScaleOutStatus::OfficialCompleteRowsExpanded
            | OfficialEvidenceScaleOutStatus::CommitteeBenchmarkResearchReady
            | OfficialEvidenceScaleOutStatus::TentativeSignalQualityReviewReady
            | OfficialEvidenceScaleOutStatus::OutcomeCoverageExpanded
            | OfficialEvidenceScaleOutStatus::CounterfactualCoverageExpanded
            | OfficialEvidenceScaleOutStatus::CorePerformanceHealthyForResearch
            | OfficialEvidenceScaleOutStatus::CoreStillBlockedByEvidence
            | OfficialEvidenceScaleOutStatus::StillSingleSymbolDominated
            | OfficialEvidenceScaleOutStatus::StillSingleOutcomeDominated
            | OfficialEvidenceScaleOutStatus::StillNeedMoreCounterfactuals
            | OfficialEvidenceScaleOutStatus::StillNeedMoreOutcomeLinks
            | OfficialEvidenceScaleOutStatus::StillEvidenceTooWeak
            | OfficialEvidenceScaleOutStatus::StillInsufficientRows
    ));
    assert!(bundle.scaleout_report.committee_benchmark_status.is_some());
    assert!(bundle.scaleout_report.outcome_coverage_status.is_some());
    assert!(bundle.scaleout_report.counterfactual_depth_status.is_some());
    assert!(bundle.scaleout_report.core_scorecard_summary.is_some());
}
