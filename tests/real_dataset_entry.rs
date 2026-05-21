mod common;

use soma_zero::{DataProvenance, EvidenceSourceKind, RealEvidenceClosureRunner, ReasonCode};

#[test]
fn local_path_entry_can_be_constructed() {
    let entry = common::real_local_test_entry("real_fixture", "generic_ohlcv_valid_alt.csv");
    assert!(entry.provenance.is_some());
    assert!(!entry.data_path.contains("://"));
}

#[test]
fn remote_url_like_data_path_is_rejected() {
    let mut config = common::real_evidence_config(
        "real-remote-reject",
        vec![common::real_local_test_entry(
            "real_fixture",
            "generic_ohlcv_valid_alt.csv",
        )],
    );
    config.real_dataset_entries[0].data_path = "https://example.com/data.csv".to_string();
    assert!(
        config
            .validate_local_paths()
            .contains(&ReasonCode::LocalPathRejected)
    );
}

#[test]
fn missing_local_file_produces_missing_real_local_data_reason() {
    let mut entry = common::real_local_test_entry("missing_real", "generic_ohlcv_valid_alt.csv");
    entry.data_path = "data/local/missing.csv".to_string();
    if let Some(provenance) = entry.provenance.as_mut() {
        provenance.local_path = Some(entry.data_path.clone());
    }
    let report = RealEvidenceClosureRunner::default().run(&common::real_evidence_config(
        "missing-real-local",
        vec![entry],
    ));
    assert_eq!(
        report.final_recommendation,
        soma_zero::RealEvidenceRecommendation::MissingRealLocalData
    );
    assert!(!report.blockers.is_empty() || !report.warnings.is_empty());
}

#[test]
fn fixture_needs_explicit_test_override_to_count_as_real_local() {
    let mut entry = common::dataset_entry("fixture_realish", "generic_ohlcv_valid_alt.csv", true);
    entry.provenance = Some(DataProvenance {
        source_kind: EvidenceSourceKind::RealLocal,
        source_label: "fixture-realish".to_string(),
        provider_label: None,
        upstream_label: None,
        local_path: Some(entry.data_path.clone()),
        generated_by: None,
        user_supplied: true,
        downloaded_by_soma: false,
        remote_url_present: false,
        official_provider: Some(false),
        affiliated_or_endorsed: Some(false),
        intended_use: Some("fixture real-local test".to_string()),
        readiness_eligible: Some(true),
        benchmark_eligible: Some(true),
        license_note: None,
        notes: None,
        reason_codes: vec![ReasonCode::DeterministicPath],
    });
    let report = RealEvidenceClosureRunner::default().run(&common::real_evidence_config(
        "no-test-override",
        vec![entry],
    ));
    assert!(report.real_local_dataset_summaries.is_empty());
    assert_ne!(
        report.final_recommendation,
        soma_zero::RealEvidenceRecommendation::HoldCurrentScope
    );
}
