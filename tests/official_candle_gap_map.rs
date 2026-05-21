#[path = "support/candle_coverage_support.rs"]
mod candle_coverage_support;
#[path = "support/candle_expansion_support.rs"]
mod candle_expansion_support;
mod common;

use soma_zero::{
    ComparableEvidenceSourceClass, OfficialCandleCoverageGapMap, OfficialCandleCoveragePackConfig,
    OfficialCandleGapConfig, OfficialCandleGapKind, OfficialCandleGapStatus, ProviderMarket,
};

fn build_gap_map(config: &OfficialCandleGapConfig) -> OfficialCandleCoverageGapMap {
    OfficialCandleCoverageGapMap::build(config).expect("gap map")
}

#[test]
fn gap_map_classifies_official_and_non_crypto_missing_series() {
    let bundle = candle_expansion_support::row_bundle_path(
        "gap-map-official-missing",
        "AAPL",
        "1d",
        1_700_000_000_000,
        ComparableEvidenceSourceClass::OfficialNonCrypto,
        false,
    );
    let map = build_gap_map(&OfficialCandleGapConfig {
        gap_id: "gap-map-official-missing".to_string(),
        comparable_evidence_bundle_paths: vec![bundle.display().to_string()],
        output_root: common::output_dir("gap-map-official-missing-out")
            .display()
            .to_string(),
        ..OfficialCandleGapConfig::default()
    });
    let cell = &map.cells[0];
    assert!(
        cell.gap_kinds
            .contains(&OfficialCandleGapKind::MissingOfficialCandleSeries)
    );
    assert!(
        cell.gap_kinds
            .contains(&OfficialCandleGapKind::MissingNonCryptoOfficialCandleSeries)
    );
    assert_eq!(
        map.gap_status,
        OfficialCandleGapStatus::MissingNonCryptoOfficialCandles
    );
}

#[test]
fn gap_map_flags_provenance_preflight_future_window_timeframe_and_timestamp_issues() {
    let timestamps = [1_700_000_000_000_u64, 1_700_086_400_000, 1_700_172_800_000];
    let (csv, _, _, manifest) = candle_expansion_support::official_csv_fixture(
        "gap-map-issues",
        "aapl_1d",
        "AAPL",
        "1d",
        &timestamps,
        false,
        false,
        true,
    );
    let pack_cfg = OfficialCandleCoveragePackConfig {
        pack_id: "gap-map-issues-pack".to_string(),
        canonical_csv_paths: vec![csv.display().to_string()],
        manifest_paths: manifest,
        output_root: common::output_dir("gap-map-issues-pack")
            .display()
            .to_string(),
        ..OfficialCandleCoveragePackConfig::default()
    };
    let pack_path = candle_coverage_support::write_pack_config_file("gap-map-issues", &pack_cfg);

    let provenance_bundle = candle_expansion_support::row_bundle_path(
        "gap-map-provenance",
        "AAPL",
        "1d",
        timestamps[0],
        ComparableEvidenceSourceClass::OfficialNonCrypto,
        false,
    );
    let provenance_map = build_gap_map(&OfficialCandleGapConfig {
        gap_id: "gap-map-provenance".to_string(),
        comparable_evidence_bundle_paths: vec![provenance_bundle.display().to_string()],
        candle_coverage_pack_paths: vec![pack_path.display().to_string()],
        output_root: common::output_dir("gap-map-provenance-out")
            .display()
            .to_string(),
        ..OfficialCandleGapConfig::default()
    });
    assert!(
        provenance_map.cells[0]
            .gap_kinds
            .contains(&OfficialCandleGapKind::MissingProvenance)
    );
    assert!(
        provenance_map.cells[0]
            .gap_kinds
            .contains(&OfficialCandleGapKind::MissingPreflight)
    );

    let future_bundle = candle_expansion_support::row_bundle_path(
        "gap-map-future",
        "AAPL",
        "1d",
        timestamps[2],
        ComparableEvidenceSourceClass::OfficialNonCrypto,
        false,
    );
    let future_map = build_gap_map(&OfficialCandleGapConfig {
        gap_id: "gap-map-future".to_string(),
        comparable_evidence_bundle_paths: vec![future_bundle.display().to_string()],
        candle_coverage_pack_paths: vec![pack_path.display().to_string()],
        output_root: common::output_dir("gap-map-future-out")
            .display()
            .to_string(),
        ..OfficialCandleGapConfig::default()
    });
    assert!(
        future_map.cells[0]
            .gap_kinds
            .contains(&OfficialCandleGapKind::MissingFutureWindow)
    );

    let timeframe_bundle = candle_expansion_support::row_bundle_path(
        "gap-map-timeframe",
        "AAPL",
        "1m",
        timestamps[0],
        ComparableEvidenceSourceClass::OfficialNonCrypto,
        false,
    );
    let timeframe_map = build_gap_map(&OfficialCandleGapConfig {
        gap_id: "gap-map-timeframe".to_string(),
        comparable_evidence_bundle_paths: vec![timeframe_bundle.display().to_string()],
        candle_coverage_pack_paths: vec![pack_path.display().to_string()],
        output_root: common::output_dir("gap-map-timeframe-out")
            .display()
            .to_string(),
        ..OfficialCandleGapConfig::default()
    });
    assert!(
        timeframe_map.cells[0]
            .gap_kinds
            .contains(&OfficialCandleGapKind::TimeframeMismatch)
    );

    let timestamp_bundle = candle_expansion_support::row_bundle_path(
        "gap-map-timestamp",
        "AAPL",
        "1d",
        1_800_000_000_000,
        ComparableEvidenceSourceClass::OfficialNonCrypto,
        false,
    );
    let timestamp_map = build_gap_map(&OfficialCandleGapConfig {
        gap_id: "gap-map-timestamp".to_string(),
        comparable_evidence_bundle_paths: vec![timestamp_bundle.display().to_string()],
        candle_coverage_pack_paths: vec![pack_path.display().to_string()],
        output_root: common::output_dir("gap-map-timestamp-out")
            .display()
            .to_string(),
        ..OfficialCandleGapConfig::default()
    });
    assert!(
        timestamp_map.cells[0]
            .gap_kinds
            .contains(&OfficialCandleGapKind::TimestampMismatch)
    );
}

#[test]
fn gap_map_separates_summary_yfinance_fixture_controlled_crypto_and_is_deterministic() {
    let summary_bundle = candle_expansion_support::row_bundle_path(
        "gap-map-summary",
        "AAPL",
        "1d",
        1_700_000_000_000,
        ComparableEvidenceSourceClass::OfficialNonCrypto,
        true,
    );
    let summary_map = build_gap_map(&OfficialCandleGapConfig {
        gap_id: "gap-map-summary".to_string(),
        comparable_evidence_bundle_paths: vec![summary_bundle.display().to_string()],
        output_root: common::output_dir("gap-map-summary-out")
            .display()
            .to_string(),
        ..OfficialCandleGapConfig::default()
    });
    assert!(
        summary_map.cells[0]
            .gap_kinds
            .contains(&OfficialCandleGapKind::SummaryDerivedOnly)
    );

    for (name, source_class, expected_kind, market) in [
        (
            "gap-map-yfinance",
            ComparableEvidenceSourceClass::YFinanceResearch,
            OfficialCandleGapKind::ResearchOnlySource,
            ProviderMarket::USEquity,
        ),
        (
            "gap-map-fixture",
            ComparableEvidenceSourceClass::FixtureArchitectureTest,
            OfficialCandleGapKind::FixtureOnlySource,
            ProviderMarket::USEquity,
        ),
        (
            "gap-map-controlled",
            ComparableEvidenceSourceClass::ControlledDiagnostic,
            OfficialCandleGapKind::ControlledOnlySource,
            ProviderMarket::USEquity,
        ),
        (
            "gap-map-crypto",
            ComparableEvidenceSourceClass::OfficialCryptoOnly,
            OfficialCandleGapKind::CryptoOnlySource,
            ProviderMarket::Crypto,
        ),
    ] {
        let symbol = if market == ProviderMarket::Crypto {
            "BTCUSDT"
        } else {
            "AAPL"
        };
        let bundle = candle_expansion_support::row_bundle_path(
            name,
            symbol,
            "1d",
            1_700_000_000_000,
            source_class,
            false,
        );
        let first = build_gap_map(&OfficialCandleGapConfig {
            gap_id: format!("{name}-one"),
            comparable_evidence_bundle_paths: vec![bundle.display().to_string()],
            output_root: common::output_dir(&format!("{name}-out-1"))
                .display()
                .to_string(),
            ..OfficialCandleGapConfig::default()
        });
        let second = build_gap_map(&OfficialCandleGapConfig {
            gap_id: format!("{name}-two"),
            comparable_evidence_bundle_paths: vec![bundle.display().to_string()],
            output_root: common::output_dir(&format!("{name}-out-2"))
                .display()
                .to_string(),
            ..OfficialCandleGapConfig::default()
        });
        assert!(first.cells[0].gap_kinds.contains(&expected_kind));
        assert_eq!(
            first.to_text(),
            second
                .to_text()
                .replace("gap_id=", "gap_id=")
                .replace(&format!("{name}-two"), &format!("{name}-one"))
        );
    }
}
