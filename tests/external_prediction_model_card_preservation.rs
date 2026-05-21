mod support;

use soma_zero::{
    ExternalPredictionModelCardPreservationStatus, Sprint90ExternalPredictionRecoveryRunner,
};
use support::sprint69_support as sprint;

#[test]
fn external_prediction_model_card_preservation_keeps_runtime_forbidden() {
    let config = sprint::sprint90_config_from_example(
        "soma_external_prediction_model_card_preservation.toml",
        "external-model-card-preservation",
    );
    let report = Sprint90ExternalPredictionRecoveryRunner::default()
        .run_external_prediction_model_card_preservation(&config)
        .expect("report");
    assert_eq!(
        report.model_card_status,
        ExternalPredictionModelCardPreservationStatus::ModelCardPreserved
    );
    assert!(report.model_id_required);
    assert!(report.model_version_required);
    assert!(report.feature_schema_hash_match_required);
    assert!(report.label_manifest_hash_match_required);
    assert!(report.runtime_forbidden_preserved);
    assert!(report.training_forbidden_preserved);
    assert!(report.live_inference_forbidden_preserved);
}
