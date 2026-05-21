use std::path::PathBuf;

use soma_zero::{
    CandleCsvConfig, CandleCsvLoader, CandleParseError, DataValidationConfig, Timeframe,
};

#[test]
fn valid_generic_ohlcv_csv_loads_into_candle_series() {
    let loaded = load_fixture("generic_ohlcv_valid.csv", CandleCsvConfig::default());
    assert_eq!(loaded.series.symbol, "UNKNOWN");
    assert_eq!(loaded.series.timeframe, Timeframe::OneMinute);
    assert_eq!(loaded.series.len(), 12);
}

#[test]
fn loader_handles_header_deterministically() {
    let first = load_fixture("generic_ohlcv_valid.csv", CandleCsvConfig::default());
    let second = load_fixture("generic_ohlcv_valid.csv", CandleCsvConfig::default());
    assert_eq!(first.series, second.series);
}

#[test]
fn loader_rejects_missing_required_column() {
    let loader = CandleCsvLoader::default();
    let input = "timestamp_ms,open,high,low,close\n1700000000000,1,2,0.5,1.5\n";
    let error = loader
        .load_from_str(input, &CandleCsvConfig::default(), None)
        .expect_err("missing volume");
    assert_eq!(error.issues[0].error, CandleParseError::MissingColumn);
}

#[test]
fn loader_rejects_invalid_numeric_value() {
    let loader = CandleCsvLoader::default();
    let input = "timestamp_ms,open,high,low,close,volume\n1700000000000,abc,2,0.5,1.5,10\n";
    let error = loader
        .load_from_str(input, &CandleCsvConfig::default(), None)
        .expect_err("invalid number");
    assert_eq!(error.issues[0].error, CandleParseError::InvalidNumber);
}

#[test]
fn loader_rejects_invalid_timestamp() {
    let loader = CandleCsvLoader::default();
    let input = "timestamp_ms,open,high,low,close,volume\nnot-a-ts,1,2,0.5,1.5,10\n";
    let error = loader
        .load_from_str(input, &CandleCsvConfig::default(), None)
        .expect_err("invalid timestamp");
    assert_eq!(error.issues[0].error, CandleParseError::InvalidTimestamp);
}

#[test]
fn loader_rejects_non_positive_price_in_strict_mode() {
    let loader = CandleCsvLoader::default();
    let input = "timestamp_ms,open,high,low,close,volume\n1700000000000,0,2,0.5,1.5,10\n";
    let error = loader
        .load_from_str(input, &CandleCsvConfig::default(), None)
        .expect_err("non positive");
    assert_eq!(error.issues[0].error, CandleParseError::NonPositivePrice);
}

#[test]
fn loader_rejects_negative_volume() {
    let loader = CandleCsvLoader::default();
    let input = "timestamp_ms,open,high,low,close,volume\n1700000000000,1,2,0.5,1.5,-1\n";
    let error = loader
        .load_from_str(input, &CandleCsvConfig::default(), None)
        .expect_err("negative volume");
    assert_eq!(error.issues[0].error, CandleParseError::NegativeVolume);
}

#[test]
fn loader_reports_ohlc_invariant_violation() {
    let loader = CandleCsvLoader::default();
    let path = fixture_path("generic_ohlcv_bad_ohlc.csv");
    let error = loader
        .load_from_path(&path, &CandleCsvConfig::default())
        .expect_err("bad ohlc");
    assert!(
        error
            .issues
            .iter()
            .any(|issue| issue.error == CandleParseError::OhlcInvariantViolation)
    );
}

#[test]
fn loader_detects_duplicate_timestamps() {
    let loader = CandleCsvLoader::default();
    let path = fixture_path("generic_ohlcv_duplicates.csv");
    let error = loader
        .load_from_path(&path, &CandleCsvConfig::default())
        .expect_err("duplicates");
    assert_eq!(error.issues[0].error, CandleParseError::DuplicateTimestamp);
}

#[test]
fn loader_detects_out_of_order_timestamps() {
    let loader = CandleCsvLoader::default();
    let path = fixture_path("generic_ohlcv_out_of_order.csv");
    let error = loader
        .load_from_path(&path, &CandleCsvConfig::default())
        .expect_err("out of order");
    assert_eq!(error.issues[0].error, CandleParseError::OutOfOrderTimestamp);
}

#[test]
fn allow_sort_repair_sorts_out_of_order_rows_and_records_repair_reason() {
    let config = CandleCsvConfig {
        allow_repair_sort: true,
        ..CandleCsvConfig::default()
    };
    let loaded = load_fixture("generic_ohlcv_out_of_order.csv", config);
    let timestamps = loaded
        .series
        .candles
        .iter()
        .map(|candle| candle.timestamp_ms)
        .collect::<Vec<_>>();
    assert!(timestamps.windows(2).all(|pair| pair[0] < pair[1]));
    assert!(
        loaded
            .quality_report
            .reason_codes
            .contains(&soma_zero::ReasonCode::CsvSortedRepairApplied)
    );
}

#[test]
fn allow_duplicate_drop_drops_duplicates_and_records_reason() {
    let loader = CandleCsvLoader {
        validation: DataValidationConfig {
            allow_duplicate_drop: true,
            ..DataValidationConfig::default()
        },
        ..CandleCsvLoader::default()
    };
    let path = fixture_path("generic_ohlcv_duplicates.csv");
    let loaded = loader
        .load_from_path(&path, &CandleCsvConfig::default())
        .expect("dedup load");
    assert_eq!(loaded.series.len(), 4);
    assert!(
        loaded
            .quality_report
            .reason_codes
            .contains(&soma_zero::ReasonCode::DuplicateTimestampDetected)
    );
}

#[test]
fn strict_mode_does_not_silently_drop_invalid_rows() {
    let loader = CandleCsvLoader::default();
    let path = fixture_path("generic_ohlcv_bad_ohlc.csv");
    assert!(
        loader
            .load_from_path(&path, &CandleCsvConfig::default())
            .is_err()
    );
}

fn load_fixture(name: &str, config: CandleCsvConfig) -> soma_zero::LoadedCandleData {
    CandleCsvLoader::default()
        .load_from_path(&fixture_path(name), &config)
        .expect("fixture loads")
}

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("market_data")
        .join(name)
}
