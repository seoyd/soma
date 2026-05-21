#[path = "support/sprint69_support.rs"]
mod support;

use std::fs;

use serde_json::json;
use soma_zero::{KISRealEvidenceValidationStatus, RealEvidenceFollowupRunner};

#[test]
fn valid_canonical_csv_passes_validation() {
    let report = support::run_sprint74_bundle(
        "soma_kis_real_evidence_validate.toml",
        "kis-real-evidence-validation-ok",
    )
    .kis_real_evidence_validation_report;
    assert_eq!(
        report.validation_status,
        KISRealEvidenceValidationStatus::ValidationReady
    );
    assert_eq!(report.canonical_rows, 8);
    assert_eq!(report.official_ready_rows, 8);
}

#[test]
fn missing_required_column_fails_validation() {
    let dir = support::sprint74_output_dir("kis-real-evidence-validation-missing-column");
    let csv_path = dir.join("bad.csv");
    fs::write(
        &csv_path,
        "symbol,market,timeframe,timestamp,open,high,low,volume\n005930,KRX,1d,2024-01-02,1,2,0,10\n",
    )
    .expect("write csv");
    let provenance_path = dir.join("provenance.json");
    fs::write(
        &provenance_path,
        serde_json::to_string_pretty(&json!({"records":[{"local_path": csv_path.display().to_string(), "provider_label":"KIS", "source_class":"OfficialKIS", "downloaded_by_soma": false, "remote_url": null}]})).expect("serialize provenance"),
    )
    .expect("write provenance");
    let preflight_path = dir.join("preflight.json");
    fs::write(
        &preflight_path,
        serde_json::to_string_pretty(&json!({"records":[{"local_path": csv_path.display().to_string(), "passed": true, "data_quality_score": 0.98}]})).expect("serialize preflight"),
    )
    .expect("write preflight");
    let manifest_path = dir.join("manifest.json");
    fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&json!({"records":[{"local_path": csv_path.display().to_string(), "symbol":"005930", "market":"KRX", "timeframe":"1d", "row_count":1, "source_class":"OfficialKIS", "no_lookahead_safe": true, "feature_schema_match": true, "label_manifest_match": true, "barrier_profile_present": true, "outcome_links_present": true, "future_window_gap_count": 0, "estimated_windows": 1, "storage_within_budget": true}]})).expect("serialize manifest"),
    )
    .expect("write manifest");
    let mut config = support::sprint74_config_from_example(
        "soma_kis_real_evidence_validate.toml",
        "kis-real-evidence-validation-missing-column-config",
    );
    config.kis_canonical_csv_paths = vec![csv_path.display().to_string()];
    config.kis_provenance_paths = vec![provenance_path.display().to_string()];
    config.kis_preflight_paths = vec![preflight_path.display().to_string()];
    config.kis_manifest_paths = vec![manifest_path.display().to_string()];
    config.sequence_export_manifest_paths = vec![manifest_path.display().to_string()];
    let report = RealEvidenceFollowupRunner::default()
        .run_validation(&config)
        .expect("run validation");
    assert_eq!(
        report.validation_status,
        KISRealEvidenceValidationStatus::MissingRequiredColumns
    );
}
