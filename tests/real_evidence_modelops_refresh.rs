#[path = "support/sprint69_support.rs"]
mod support;

use soma_zero::{
    RealEvidenceExternalReevaluationStatus, RealEvidenceModelOpsRefreshStatus,
    RealEvidencePredictionImportStatus, RealEvidencePredictionRefreshRunner,
};

#[test]
fn modelops_refresh_threads_statuses() {
    let config = support::sprint75_config_from_example(
        "soma_real_modelops_refresh.toml",
        "modelops-refresh",
    );
    let report = RealEvidencePredictionRefreshRunner::default()
        .run_model_ops_refresh(&config)
        .expect("modelops refresh");
    assert_eq!(
        report.prediction_import_status,
        RealEvidencePredictionImportStatus::PredictionImportReady
    );
    assert_eq!(
        report.reevaluation_status,
        RealEvidenceExternalReevaluationStatus::ReEvaluationReady
    );
    assert_eq!(
        report.model_ops_status,
        RealEvidenceModelOpsRefreshStatus::ModelOpsRefreshed
    );
    assert!(!report.registry_refresh_needed);
}
