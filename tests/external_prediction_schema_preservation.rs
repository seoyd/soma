mod support;

use soma_zero::{
    ExternalPredictionSchemaPreservationStatus, Sprint90ExternalPredictionRecoveryRunner,
};
use support::sprint69_support as sprint;

#[test]
fn external_prediction_schema_preservation_keeps_all_contracts() {
    let config = sprint::sprint90_config_from_example(
        "soma_external_prediction_schema_preservation.toml",
        "external-schema-preservation",
    );
    let report = Sprint90ExternalPredictionRecoveryRunner::default()
        .run_external_prediction_schema_preservation(&config)
        .expect("report");
    assert_eq!(
        report.schema_status,
        ExternalPredictionSchemaPreservationStatus::SchemaPreserved
    );
    assert!(report.schema_v2_preserved);
    assert!(report.sequence_id_match_preserved);
    assert!(report.duplicate_rejection_preserved);
    assert!(report.invalid_probability_rejection_preserved);
    assert!(report.extra_sequence_rejection_preserved);
    assert!(report.missing_sequence_rejection_preserved);
    assert!(report.forbidden_column_rejection_preserved);
}
