#[path = "support/sprint69_support.rs"]
mod support;

use serde_json::json;
use soma_zero::{RealEvidenceFollowupRunner, RealEvidenceOutcomeReadinessStatus};

#[test]
fn outcome_ready_example_is_reported() {
    let report = support::run_sprint74_bundle(
        "soma_real_outcome_readiness.toml",
        "real-evidence-outcome-ok",
    )
    .real_evidence_outcome_readiness_report;
    assert_eq!(
        report.readiness_status,
        RealEvidenceOutcomeReadinessStatus::OutcomeReady
    );
    assert!(report.outcome_links_possible);
}

#[test]
fn future_window_gap_is_detected() {
    let manifest_path = support::write_support_json(
        "real-evidence-outcome-future-gap",
        "manifest.json",
        &json!({"records":[{"local_path":"examples/sprint74_data/kis_real_canonical_sample.csv","symbol":"005930","market":"KRX","timeframe":"1d","row_count":8,"source_class":"OfficialKIS","no_lookahead_safe":true,"feature_schema_match":true,"label_manifest_match":true,"barrier_profile_present":true,"outcome_links_present":true,"future_window_gap_count":2,"estimated_windows":4,"storage_within_budget":true}]}),
    );
    let mut config = support::sprint74_config_from_example(
        "soma_real_outcome_readiness.toml",
        "real-evidence-outcome-future-gap-config",
    );
    config.kis_manifest_paths = vec![manifest_path.clone()];
    config.sequence_export_manifest_paths = vec![manifest_path];
    let report = RealEvidenceFollowupRunner::default()
        .run_outcome_readiness(&config)
        .expect("run outcome readiness");
    assert_eq!(
        report.readiness_status,
        RealEvidenceOutcomeReadinessStatus::NeedFutureWindows
    );
}
