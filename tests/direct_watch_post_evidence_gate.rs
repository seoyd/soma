#[path = "support/sprint69_support.rs"]
mod support;

use soma_zero::{DirectWatchPostEvidenceGateStatus, RealEvidencePredictionRefreshRunner};

#[test]
fn direct_watch_ready_with_warnings_after_prediction_refresh() {
    let config = support::sprint75_config_from_example(
        "soma_direct_watch_post_evidence_gate.toml",
        "post-evidence-gate",
    );
    let report = RealEvidencePredictionRefreshRunner::default()
        .run_direct_watch_post_evidence_gate(&config)
        .expect("post-evidence gate");
    assert_eq!(
        report.gate_status,
        DirectWatchPostEvidenceGateStatus::DirectWatchReadyWithWarnings
    );
    assert!(report.static_only);
    assert!(report.paper_only);
}

#[test]
fn direct_watch_needs_prediction_refresh_when_stale_remains() {
    let mut config = support::sprint75_config_from_example(
        "soma_direct_watch_post_evidence_gate.toml",
        "post-evidence-gate-missing-predictions",
    );
    config.new_prediction_csv_paths.clear();
    let report = RealEvidencePredictionRefreshRunner::default()
        .run_direct_watch_post_evidence_gate(&config)
        .expect("post-evidence gate");
    assert_eq!(
        report.gate_status,
        DirectWatchPostEvidenceGateStatus::DirectWatchNeedsPredictionRefresh
    );
}
