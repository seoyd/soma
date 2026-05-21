mod common;
#[path = "support/sprint62_support.rs"]
mod sprint62_support;

use soma_zero::{ExternalModelBridgeReadinessStatus, SequenceDatasetExportRunner};

#[test]
fn external_bridge_is_ready_for_prediction_csv_import() {
    let config = sprint62_support::export_config_from_example(
        "soma_external_bridge_readiness.toml",
        "external-bridge",
    );
    let report = SequenceDatasetExportRunner::default()
        .run_external_bridge_readiness(&config)
        .expect("run bridge");
    assert_eq!(
        report.readiness_status,
        ExternalModelBridgeReadinessStatus::ReadyForPredictionCsvImport
    );
    assert!(!report.example_prediction_schema.is_empty());
}
