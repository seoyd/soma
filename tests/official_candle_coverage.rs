mod common;
#[path = "support/official_committee_support.rs"]
mod official_committee_support;

use std::fs;
use std::path::PathBuf;

use serde_json::json;
use soma_zero::{
    CommitteeScenarioRow, OfficialCandleCoverageRunner, OfficialCandleCoverageStatus,
    OfficialEvidenceReplicationConfig, OfficialReplicationArtifactInventory, PersonaHorizon,
    ProviderMarket, materialize_official_candle_series,
};

fn official_row(name: &str, timestamp_ms: u64) -> CommitteeScenarioRow {
    let mut row = official_committee_support::scenario_row(name, 0, "AAPL", timestamp_ms);
    row.provenance_summary = "row-level-provenance: official-api-collected".to_string();
    row
}

fn write_series(name: &str, symbol: &str, timestamps: &[u64], bad_quality: bool) -> PathBuf {
    let path = common::output_dir(&format!("{name}-series")).join(format!("{symbol}.json"));
    let candles = timestamps
        .iter()
        .enumerate()
        .map(|(index, timestamp)| {
            let close = if bad_quality && index == 0 {
                -1.0
            } else {
                100.0 + index as f64
            };
            json!({
                "timestamp_ms": timestamp,
                "open": close,
                "high": close + 1.0,
                "low": close - 1.0,
                "close": close,
                "volume": 1000.0 + index as f64,
                "spread_bps": 4.0
            })
        })
        .collect::<Vec<_>>();
    fs::write(
        &path,
        serde_json::to_string_pretty(&json!({
            "symbol": symbol,
            "timeframe": "OneDay",
            "candles": candles
        }))
        .expect("series json"),
    )
    .expect("write series");
    path
}

#[test]
fn candle_coverage_reports_healthy_and_materializes_from_csv() {
    let row = official_row("healthy-coverage", 1_700_000_000_000);
    let candle_path = official_committee_support::write_candle_series(
        "healthy-coverage",
        "AAPL",
        1_700_000_000_000,
        1.0,
    );
    let report = OfficialCandleCoverageRunner::default()
        .run(&[row], &[candle_path.display().to_string()])
        .expect("coverage");
    assert_eq!(
        report.coverage_status,
        OfficialCandleCoverageStatus::HealthyOfficialCandleCoverage
    );
    assert_eq!(report.rows_with_candles, 1);
    assert_eq!(report.rows_with_future_window, 1);
    assert_eq!(report.rows_no_lookahead_safe, 1);

    let csv_path = official_committee_support::write_official_csv_bundle(
        "coverage-materialize",
        "AAPL",
        3,
        true,
        true,
        true,
    );
    let config = OfficialEvidenceReplicationConfig {
        replication_id: "coverage-materialize".to_string(),
        official_canonical_csv_paths: vec![csv_path.display().to_string()],
        output_root: common::output_dir("coverage-materialize-root")
            .display()
            .to_string(),
        ..OfficialEvidenceReplicationConfig::default()
    };
    let inventory = OfficialReplicationArtifactInventory::from_paths(&config.all_artifact_paths());
    let materialized =
        materialize_official_candle_series(&config, &inventory).expect("materialize");
    assert_eq!(materialized.len(), 1);
    assert!(materialized[0].ends_with("aapl_candles.json"));
}

#[test]
fn candle_coverage_reports_missing_candles_and_future_windows() {
    let row = official_row("missing-candles", 1_700_000_000_000);
    let missing = OfficialCandleCoverageRunner::default()
        .run(&[row.clone()], &[])
        .expect("missing candles");
    assert_eq!(
        missing.coverage_status,
        OfficialCandleCoverageStatus::MissingOfficialCandles
    );
    assert_eq!(missing.missing_candle_rows, 1);

    let short_path = write_series(
        "missing-future-window",
        "AAPL",
        &[1_700_000_000_000, 1_700_000_000_001, 1_700_000_000_002],
        false,
    );
    let short = OfficialCandleCoverageRunner::default()
        .run(&[row], &[short_path.display().to_string()])
        .expect("short");
    assert_eq!(
        short.coverage_status,
        OfficialCandleCoverageStatus::MissingFutureWindow
    );
    assert_eq!(short.missing_future_window_rows, 1);
}

#[test]
fn candle_coverage_reports_symbol_timestamp_and_timeframe_mismatches() {
    let row = official_row("coverage-mismatch", 1_700_000_000_000);
    let timestamp_path = write_series(
        "timestamp-mismatch",
        "AAPL",
        &[
            1_700_000_000_001,
            1_700_000_000_002,
            1_700_000_000_003,
            1_700_000_000_004,
            1_700_000_000_005,
            1_700_000_000_006,
            1_700_000_000_007,
            1_700_000_000_008,
            1_700_000_000_009,
            1_700_000_000_010,
            1_700_000_000_011,
            1_700_000_000_012,
            1_700_000_000_013,
            1_700_000_000_014,
            1_700_000_000_015,
            1_700_000_000_016,
            1_700_000_000_017,
            1_700_000_000_018,
            1_700_000_000_019,
            1_700_000_000_020,
            1_700_000_000_021,
            1_700_000_000_022,
            1_700_000_000_023,
            1_700_000_000_024,
            1_700_000_000_025,
        ],
        false,
    );
    let timestamp = OfficialCandleCoverageRunner::default()
        .run(&[row.clone()], &[timestamp_path.display().to_string()])
        .expect("timestamp");
    assert_eq!(
        timestamp.coverage_status,
        OfficialCandleCoverageStatus::MissingOfficialCandles
    );
    assert_eq!(timestamp.timestamp_mismatch_rows, 1);

    let symbol_path = write_series(
        "symbol-mismatch",
        "MSFT",
        &[
            1_700_000_000_000,
            1_700_000_000_001,
            1_700_000_000_002,
            1_700_000_000_003,
            1_700_000_000_004,
            1_700_000_000_005,
            1_700_000_000_006,
            1_700_000_000_007,
            1_700_000_000_008,
            1_700_000_000_009,
            1_700_000_000_010,
            1_700_000_000_011,
            1_700_000_000_012,
            1_700_000_000_013,
            1_700_000_000_014,
            1_700_000_000_015,
            1_700_000_000_016,
            1_700_000_000_017,
            1_700_000_000_018,
            1_700_000_000_019,
            1_700_000_000_020,
            1_700_000_000_021,
            1_700_000_000_022,
            1_700_000_000_023,
            1_700_000_000_024,
        ],
        false,
    );
    let symbol = OfficialCandleCoverageRunner::default()
        .run(&[row.clone()], &[symbol_path.display().to_string()])
        .expect("symbol");
    assert_eq!(
        symbol.coverage_status,
        OfficialCandleCoverageStatus::MissingOfficialCandles
    );
    assert_eq!(symbol.symbol_mismatch_rows, 1);

    let mut intraday = row;
    intraday.target_horizon = PersonaHorizon::Intraday;
    let exact_path = official_committee_support::write_candle_series(
        "timeframe-mismatch",
        "AAPL",
        1_700_000_000_000,
        1.0,
    );
    let timeframe = OfficialCandleCoverageRunner::default()
        .run(&[intraday], &[exact_path.display().to_string()])
        .expect("timeframe");
    assert_eq!(
        timeframe.coverage_status,
        OfficialCandleCoverageStatus::TimeframeMismatch
    );
    assert_eq!(timeframe.timeframe_mismatch_rows, 1);
}

#[test]
fn candle_coverage_reports_bad_quality_and_boundary_only_modes() {
    let row = official_row("bad-quality", 1_700_000_000_000);
    let bad_path = write_series(
        "bad-quality",
        "AAPL",
        &[
            1_700_000_000_000,
            1_700_000_000_001,
            1_700_000_000_002,
            1_700_000_000_003,
            1_700_000_000_004,
            1_700_000_000_005,
            1_700_000_000_006,
            1_700_000_000_007,
            1_700_000_000_008,
            1_700_000_000_009,
            1_700_000_000_010,
            1_700_000_000_011,
            1_700_000_000_012,
            1_700_000_000_013,
            1_700_000_000_014,
            1_700_000_000_015,
            1_700_000_000_016,
            1_700_000_000_017,
            1_700_000_000_018,
            1_700_000_000_019,
            1_700_000_000_020,
            1_700_000_000_021,
            1_700_000_000_022,
            1_700_000_000_023,
            1_700_000_000_024,
        ],
        true,
    );
    let bad = OfficialCandleCoverageRunner::default()
        .run(&[row], &[bad_path.display().to_string()])
        .expect("bad quality");
    assert_eq!(
        bad.coverage_status,
        OfficialCandleCoverageStatus::BadDataQuality
    );

    let mut controlled = official_row("controlled-only", 1_700_000_000_000);
    controlled.provenance_summary = "controlled-local".to_string();
    let controlled_report = OfficialCandleCoverageRunner::default()
        .run(&[controlled], &[])
        .expect("controlled coverage");
    assert_eq!(
        controlled_report.coverage_status,
        OfficialCandleCoverageStatus::ControlledOnlyCoverage
    );

    let mut crypto = official_row("crypto-only", 1_700_000_000_000);
    crypto.market = ProviderMarket::Crypto;
    let crypto_report = OfficialCandleCoverageRunner::default()
        .run(&[crypto], &[])
        .expect("crypto coverage");
    assert_eq!(
        crypto_report.coverage_status,
        OfficialCandleCoverageStatus::CryptoOnlyCoverage
    );
}
