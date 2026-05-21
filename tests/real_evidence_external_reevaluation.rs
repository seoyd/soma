#[path = "support/sprint69_support.rs"]
mod support;

use std::fs;

use soma_zero::{RealEvidenceExternalReevaluationStatus, RealEvidencePredictionRefreshRunner};

#[test]
fn external_reevaluation_computes_offline_metrics() {
    let config =
        support::sprint75_config_from_example("soma_real_external_reevaluate.toml", "reevaluate");
    let report = RealEvidencePredictionRefreshRunner::default()
        .run_external_reevaluation(&config)
        .expect("reevaluate");
    assert_eq!(
        report.evaluation_status,
        RealEvidenceExternalReevaluationStatus::ReEvaluationReady
    );
    assert_eq!(report.evaluated_count, 4);
    assert_eq!(report.coverage_ratio, 1.0);
    assert!(report.brier_score.is_some());
    assert!(report.ece.is_some());
    assert!(report.top_k_precision.is_some());
    assert!(report.rank_correlation.is_some());
    assert!(report.risk_adjusted_score.is_some());
}

#[test]
fn insufficient_coverage_is_reported_conservatively() {
    let mut config = support::sprint75_config_from_example(
        "soma_real_external_reevaluate.toml",
        "reevaluate-partial",
    );
    let dir = support::sprint75_output_dir("reevaluate-partial-inputs");
    let path = dir.join("predictions.csv");
    fs::write(
        &path,
        "model_id,model_version,sequence_id,probability,source_class\next-model-b,1.0.0,real-seq-ext-model-b-0001,0.82,OfficialKIS\next-model-b,1.0.0,real-seq-ext-model-b-0002,0.21,OfficialKIS\next-model-b,1.0.0,real-seq-ext-model-b-0003,0.76,OfficialKIS\n",
    )
    .expect("write csv");
    config.new_prediction_csv_paths = vec![path.display().to_string()];
    let report = RealEvidencePredictionRefreshRunner::default()
        .run_external_reevaluation(&config)
        .expect("reevaluate");
    assert_eq!(
        report.evaluation_status,
        RealEvidenceExternalReevaluationStatus::InsufficientCoverage
    );
    assert!(report.coverage_ratio < 1.0);
}
