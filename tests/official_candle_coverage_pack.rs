#[path = "support/candle_coverage_support.rs"]
mod candle_coverage_support;
mod common;

use soma_zero::{
    EvidenceSourceKind, OfficialCandleCoveragePack, OfficialCandleSeriesSourceClass,
    PreflightFinalStatus,
};

#[test]
fn official_candle_pack_builds_descriptors_and_tracks_boundaries() {
    let timestamps = (0..8)
        .map(|index| 1_700_000_000_000 + index * 86_400_000)
        .collect::<Vec<_>>();
    let official_csv = candle_coverage_support::write_csv(
        "pack-build",
        "aapl_1d",
        "AAPL",
        "1d",
        &timestamps,
        false,
    );
    let official_provenance = candle_coverage_support::write_provenance(
        "pack-build",
        "aapl_1d",
        EvidenceSourceKind::OfficialApiCollected,
        "official",
        &official_csv,
        true,
    );
    let official_preflight = candle_coverage_support::write_preflight(
        "pack-build",
        "aapl_1d",
        "AAPL",
        soma_zero::Timeframe::OneDay,
        &official_csv,
        EvidenceSourceKind::OfficialApiCollected,
        PreflightFinalStatus::ReadyForRealEvidence,
    );
    let official_manifest = candle_coverage_support::write_manifest(
        "pack-build",
        "aapl_1d",
        "AAPL",
        soma_zero::Timeframe::OneDay,
        EvidenceSourceKind::OfficialApiCollected,
        &official_csv,
        &timestamps,
    );

    let controlled_csv = candle_coverage_support::write_csv(
        "pack-build",
        "controlled_msft_1d",
        "MSFT",
        "1d",
        &timestamps,
        false,
    );
    let controlled_provenance = candle_coverage_support::write_provenance(
        "pack-build",
        "controlled_msft_1d",
        EvidenceSourceKind::RealLocal,
        "controlled",
        &controlled_csv,
        false,
    );
    let controlled_preflight = candle_coverage_support::write_preflight(
        "pack-build",
        "controlled_msft_1d",
        "MSFT",
        soma_zero::Timeframe::OneDay,
        &controlled_csv,
        EvidenceSourceKind::RealLocal,
        PreflightFinalStatus::ReadyForRealEvidence,
    );

    let mut config = candle_coverage_support::pack_config(
        "pack-build",
        vec![
            official_csv.display().to_string(),
            controlled_csv.display().to_string(),
        ],
        vec![
            official_provenance.display().to_string(),
            controlled_provenance.display().to_string(),
        ],
        vec![
            official_preflight.display().to_string(),
            controlled_preflight.display().to_string(),
        ],
        vec![official_manifest.display().to_string()],
    );
    config.require_official_source = false;
    config.allow_controlled_fixture = true;

    let pack = OfficialCandleCoveragePack::build(&config).expect("build pack");
    assert_eq!(pack.descriptors.len(), 2);
    assert_eq!(pack.official_non_crypto_series.len(), 1);
    assert_eq!(pack.controlled_series.len(), 1);
    assert_eq!(pack.readiness_eligible_series_count, 1);
    assert_eq!(pack.benchmark_eligible_series_count, 1);
    assert!(pack.storage_bytes > 0);
    assert_eq!(
        pack.descriptors[0].source_class,
        OfficialCandleSeriesSourceClass::OfficialNonCrypto
    );
    assert_eq!(
        pack.descriptors[1].source_class,
        OfficialCandleSeriesSourceClass::ControlledDiagnostic
    );
}

#[test]
fn official_candle_pack_detects_missing_sidecars_invalid_csv_and_deterministic_ordering() {
    let timestamps = vec![
        1_700_000_000_000,
        1_700_086_400_000,
        1_700_259_200_000,
        1_700_259_200_000,
    ];
    let missing_preflight_csv = candle_coverage_support::write_csv(
        "pack-missing",
        "yfinance_aapl_1d",
        "AAPL",
        "1d",
        &timestamps,
        false,
    );
    let missing_preflight_provenance = candle_coverage_support::write_provenance(
        "pack-missing",
        "yfinance_aapl_1d",
        EvidenceSourceKind::YFinanceResearch,
        "yfinance",
        &missing_preflight_csv,
        false,
    );
    let invalid_csv = candle_coverage_support::write_csv(
        "pack-missing",
        "bad_fixture_1d",
        "BAD",
        "1d",
        &timestamps,
        true,
    );
    let mut config = candle_coverage_support::pack_config(
        "pack-missing",
        vec![
            missing_preflight_csv.display().to_string(),
            invalid_csv.display().to_string(),
        ],
        vec![missing_preflight_provenance.display().to_string()],
        vec![],
        vec![],
    );
    config.require_official_source = false;
    config.allow_yfinance_research = true;
    config.allow_fixture = true;

    let first = OfficialCandleCoveragePack::build(&config).expect("first pack");
    let second = OfficialCandleCoveragePack::build(&config).expect("second pack");
    assert_eq!(first.to_text(), second.to_text());
    assert!(first.descriptors.iter().any(|descriptor| {
        descriptor
            .reason_codes
            .contains(&soma_zero::ReasonCode::MissingOfficialPreflight)
    }));
    assert!(first.descriptors.iter().any(|descriptor| {
        descriptor.path == invalid_csv.display().to_string()
            && descriptor.row_count < timestamps.len()
    }));
    assert!(
        first
            .descriptors
            .iter()
            .any(|descriptor| descriptor.has_duplicates
                || descriptor.has_gaps
                || descriptor.row_count == 0)
    );
}
