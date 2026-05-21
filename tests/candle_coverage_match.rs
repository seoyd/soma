#[path = "support/candle_coverage_support.rs"]
mod candle_coverage_support;
mod common;

use soma_zero::{
    ComparableEvidenceSourceClass, EvidenceSourceKind, OfficialCandleCoveragePack,
    PreflightFinalStatus, build_candle_coverage_match_computation,
};

#[test]
fn candle_coverage_match_preserves_official_crypto_and_diagnostic_boundaries() {
    let timestamps = (0..8)
        .map(|index| 1_700_000_000_000 + index * 86_400_000)
        .collect::<Vec<_>>();
    let official_csv =
        candle_coverage_support::write_csv("match", "aapl_1d", "AAPL", "1d", &timestamps, false);
    let official_provenance = candle_coverage_support::write_provenance(
        "match",
        "aapl_1d",
        EvidenceSourceKind::OfficialApiCollected,
        "official",
        &official_csv,
        true,
    );
    let official_preflight = candle_coverage_support::write_preflight(
        "match",
        "aapl_1d",
        "AAPL",
        soma_zero::Timeframe::OneDay,
        &official_csv,
        EvidenceSourceKind::OfficialApiCollected,
        PreflightFinalStatus::ReadyForRealEvidence,
    );
    let official_manifest = candle_coverage_support::write_manifest(
        "match",
        "aapl_1d",
        "AAPL",
        soma_zero::Timeframe::OneDay,
        EvidenceSourceKind::OfficialApiCollected,
        &official_csv,
        &timestamps,
    );

    let crypto_csv =
        candle_coverage_support::write_csv("match", "btc_1d", "BTCUSDT", "1d", &timestamps, false);
    let crypto_provenance = candle_coverage_support::write_provenance(
        "match",
        "btc_1d",
        EvidenceSourceKind::OfficialApiCollected,
        "official-crypto",
        &crypto_csv,
        true,
    );
    let crypto_preflight = candle_coverage_support::write_preflight(
        "match",
        "btc_1d",
        "BTCUSDT",
        soma_zero::Timeframe::OneDay,
        &crypto_csv,
        EvidenceSourceKind::OfficialApiCollected,
        PreflightFinalStatus::ReadyForRealEvidence,
    );

    let yfinance_csv = candle_coverage_support::write_csv(
        "match",
        "yfinance_tsla_1d",
        "TSLA",
        "1d",
        &timestamps,
        false,
    );
    let yfinance_provenance = candle_coverage_support::write_provenance(
        "match",
        "yfinance_tsla_1d",
        EvidenceSourceKind::YFinanceResearch,
        "yfinance",
        &yfinance_csv,
        false,
    );
    let yfinance_preflight = candle_coverage_support::write_preflight(
        "match",
        "yfinance_tsla_1d",
        "TSLA",
        soma_zero::Timeframe::OneDay,
        &yfinance_csv,
        EvidenceSourceKind::YFinanceResearch,
        PreflightFinalStatus::ReadyForRealEvidence,
    );

    let mut config = candle_coverage_support::pack_config(
        "match-pack",
        vec![
            official_csv.display().to_string(),
            crypto_csv.display().to_string(),
            yfinance_csv.display().to_string(),
        ],
        vec![
            official_provenance.display().to_string(),
            crypto_provenance.display().to_string(),
            yfinance_provenance.display().to_string(),
        ],
        vec![
            official_preflight.display().to_string(),
            crypto_preflight.display().to_string(),
            yfinance_preflight.display().to_string(),
        ],
        vec![official_manifest.display().to_string()],
    );
    config.require_official_source = false;
    config.allow_yfinance_research = true;
    let pack = OfficialCandleCoveragePack::build(&config).expect("build pack");

    let rows = vec![
        candle_coverage_support::comparable_row(
            "official",
            "AAPL",
            "1d",
            timestamps[0],
            ComparableEvidenceSourceClass::OfficialNonCrypto,
            true,
            false,
            false,
        ),
        candle_coverage_support::comparable_row(
            "crypto",
            "BTCUSDT",
            "1d",
            timestamps[0],
            ComparableEvidenceSourceClass::OfficialCryptoOnly,
            false,
            false,
            false,
        ),
        candle_coverage_support::comparable_row(
            "research",
            "TSLA",
            "1d",
            timestamps[0],
            ComparableEvidenceSourceClass::YFinanceResearch,
            false,
            true,
            false,
        ),
    ];
    let report =
        build_candle_coverage_match_computation(&rows, &pack, &Default::default()).match_report;
    assert_eq!(report.official_ready_match_count, 1);
    assert_eq!(report.benchmark_ready_match_count, 1);
    assert!(
        report
            .matches
            .iter()
            .any(|entry| entry.scenario_row_id == "official" && entry.official_ready_match)
    );
    assert!(
        report
            .matches
            .iter()
            .any(|entry| entry.scenario_row_id == "crypto" && !entry.official_ready_match)
    );
    assert!(
        report
            .matches
            .iter()
            .any(|entry| entry.scenario_row_id == "research" && entry.diagnostic_only)
    );
}

#[test]
fn candle_coverage_match_counts_missing_provenance_preflight_timeframe_and_timestamp_failures() {
    let timestamps = (0..5)
        .map(|index| 1_700_000_000_000 + index * 86_400_000)
        .collect::<Vec<_>>();
    let csv = candle_coverage_support::write_csv(
        "match-failures",
        "aapl_1d",
        "AAPL",
        "1d",
        &timestamps,
        false,
    );
    let preflight = candle_coverage_support::write_preflight(
        "match-failures",
        "aapl_1d",
        "AAPL",
        soma_zero::Timeframe::OneDay,
        &csv,
        EvidenceSourceKind::OfficialApiCollected,
        PreflightFinalStatus::ReadyForRealEvidence,
    );
    let config = candle_coverage_support::pack_config(
        "match-failures",
        vec![csv.display().to_string()],
        vec![],
        vec![preflight.display().to_string()],
        vec![],
    );
    let mut config = config;
    config.require_official_source = false;
    let pack = OfficialCandleCoveragePack::build(&config).expect("pack");
    let rows = vec![
        candle_coverage_support::comparable_row(
            "missing-prov",
            "AAPL",
            "1d",
            timestamps[0],
            ComparableEvidenceSourceClass::OfficialNonCrypto,
            true,
            false,
            false,
        ),
        candle_coverage_support::comparable_row(
            "timeframe",
            "AAPL",
            "1h",
            timestamps[0],
            ComparableEvidenceSourceClass::OfficialNonCrypto,
            true,
            false,
            false,
        ),
        candle_coverage_support::comparable_row(
            "timestamp",
            "AAPL",
            "1d",
            timestamps[0] + 1_000_000,
            ComparableEvidenceSourceClass::OfficialNonCrypto,
            true,
            false,
            false,
        ),
    ];
    let report =
        build_candle_coverage_match_computation(&rows, &pack, &Default::default()).match_report;
    assert_eq!(report.missing_provenance_count, 3);
    assert_eq!(report.matches.len(), 3);
    assert_eq!(report.to_text(), report.to_text());
}
