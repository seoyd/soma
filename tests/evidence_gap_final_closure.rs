#[path = "support/sprint69_support.rs"]
mod support;

use soma_zero::EvidenceGapFinalClosureStatus;

#[test]
fn evidence_gap_final_closure_closes_remaining_ext_model_b_gap() {
    let bundle = support::run_sprint73_bundle(
        "soma_evidence_gap_final_close.toml",
        "evidence-gap-final-closure",
    );
    let report = bundle.evidence_gap_final_closure_report;

    assert_eq!(report.gaps_before, 1);
    assert_eq!(report.gaps_after, 0);
    assert_eq!(report.closed_gaps, vec!["ext-model-b:1.0.0".to_string()]);
    assert!(report.prediction_history_gap_closed);
    assert!(report.retirement_gap_stable);
    assert!(report.owner_gap_closed);
    assert_eq!(
        report.evidence_status,
        EvidenceGapFinalClosureStatus::EvidenceGapClosed
    );
}

#[test]
fn evidence_gap_final_closure_reports_prediction_history_blocker_when_threshold_fails() {
    let mut config = support::sprint73_config_from_example(
        "soma_evidence_gap_final_close.toml",
        "evidence-gap-final-closure-still-open",
    );
    config.min_coverage_ratio = 1.1;
    let report = soma_zero::ExtModelBPredictionClosureRunner::default()
        .run_evidence_gap_final_closure(&config)
        .expect("run evidence final closure");
    assert_eq!(report.gaps_after, 1);
    assert_eq!(
        report.evidence_status,
        EvidenceGapFinalClosureStatus::StillNeedsPredictionHistory
    );
}
