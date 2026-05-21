use soma_zero::{DataProvenance, EvidenceSourceKind, ReasonCode};

#[test]
fn local_user_supplied_provenance_is_accepted() {
    let provenance = DataProvenance {
        source_kind: EvidenceSourceKind::RealLocal,
        source_label: "user-local".to_string(),
        provider_label: None,
        upstream_label: None,
        local_path: Some("data/local/BTCUSDT_1m.csv".to_string()),
        generated_by: None,
        user_supplied: true,
        downloaded_by_soma: false,
        remote_url_present: false,
        official_provider: Some(false),
        affiliated_or_endorsed: Some(false),
        intended_use: Some("test".to_string()),
        readiness_eligible: Some(true),
        benchmark_eligible: Some(true),
        license_note: None,
        notes: None,
        reason_codes: vec![ReasonCode::DeterministicPath],
    };
    assert!(
        !provenance
            .validate_local_only()
            .contains(&ReasonCode::LocalPathRejected)
    );
}

#[test]
fn remote_url_provenance_is_rejected() {
    let provenance = DataProvenance {
        source_kind: EvidenceSourceKind::RealLocal,
        source_label: "remote".to_string(),
        provider_label: None,
        upstream_label: None,
        local_path: Some("https://example.com/data.csv".to_string()),
        generated_by: None,
        user_supplied: true,
        downloaded_by_soma: false,
        remote_url_present: true,
        official_provider: Some(false),
        affiliated_or_endorsed: Some(false),
        intended_use: Some("test".to_string()),
        readiness_eligible: Some(true),
        benchmark_eligible: Some(true),
        license_note: None,
        notes: None,
        reason_codes: vec![],
    };
    assert!(
        provenance
            .validate_local_only()
            .contains(&ReasonCode::LocalPathRejected)
    );
}

#[test]
fn provenance_rendering_is_deterministic_and_download_disabled() {
    let provenance = DataProvenance::inferred_from_path(Some("data/local/BTCUSDT_1m.csv"));
    assert!(!provenance.downloaded_by_soma);
    assert_eq!(
        provenance.to_deterministic_string(),
        provenance.to_deterministic_string()
    );
}

#[test]
fn official_api_collected_provenance_allows_soma_download_flag() {
    let provenance = DataProvenance {
        source_kind: EvidenceSourceKind::OfficialApiCollected,
        source_label: "upbit/KRW-BTC/1m".to_string(),
        provider_label: Some("upbit".to_string()),
        upstream_label: Some("Upbit".to_string()),
        local_path: Some("data/collected/upbit/KRWBTC/1m/canonical.csv".to_string()),
        generated_by: Some("soma_experiment.collect-candles".to_string()),
        user_supplied: false,
        downloaded_by_soma: true,
        remote_url_present: false,
        official_provider: Some(true),
        affiliated_or_endorsed: Some(true),
        intended_use: Some("official research".to_string()),
        readiness_eligible: Some(true),
        benchmark_eligible: Some(true),
        license_note: None,
        notes: None,
        reason_codes: vec![ReasonCode::DeterministicPath],
    };
    assert!(
        !provenance
            .validate_local_only()
            .contains(&ReasonCode::DoctrineViolation)
    );
}
