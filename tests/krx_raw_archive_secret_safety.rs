mod common;

use std::fs;
use std::path::PathBuf;

use soma_zero::{
    KRXRawResponseArchiveRecord, KRXResponseSchemaDriftReport, KRXResponseSchemaStatus,
};

fn example_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join(name)
}

#[test]
fn archive_redaction_assertions() {
    let output_dir = common::output_dir("krx-raw-archive");
    let record = KRXRawResponseArchiveRecord::from_fixture(
        &output_dir,
        "005930",
        "005930",
        &example_path("sprint50_data/krx_005930_raw_response_fixture.json"),
    )
    .expect("archive fixture");
    let rendered = record.to_text();
    assert!(rendered.contains("auth=redacted"));
    assert!(!rendered.contains("KRX_API_KEY"));
    let drift = KRXResponseSchemaDriftReport::check_records(&[record], "schema-ok");
    assert_eq!(drift.schema_status, KRXResponseSchemaStatus::SchemaValid);
}

#[test]
fn schema_drift_detects_missing_required_field() {
    let output_dir = common::output_dir("krx-schema-missing-field");
    let bad_path = output_dir.join("bad_raw.json");
    fs::write(
        &bad_path,
        r#"{"symbol":"005930","timeframe":"1d","rows":[{"date":"2024-01-02","open":1.0}]}"#,
    )
    .expect("write bad fixture");
    let record =
        KRXRawResponseArchiveRecord::from_fixture(&output_dir, "005930", "005930", &bad_path)
            .expect("archive bad fixture");
    let drift = KRXResponseSchemaDriftReport::check_records(&[record], "schema-bad");
    assert_eq!(
        drift.schema_status,
        KRXResponseSchemaStatus::MissingRequiredField
    );
    assert!(drift.missing_fields.iter().any(|field| field == "high"));
}
