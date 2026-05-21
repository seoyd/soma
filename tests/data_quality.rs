use std::path::PathBuf;

use soma_zero::{
    BacktestSimulator, CandleCsvConfig, CandleCsvLoader, DataQualitySeverity, DataValidationConfig,
    ReasonCode,
};

#[test]
fn valid_fixture_produces_good_or_acceptable_quality() {
    let loaded = load_fixture("generic_ohlcv_valid.csv", CandleCsvLoader::default(), false);
    assert!(loaded.quality_report.data_quality_score > 0.80);
    assert!(matches!(
        loaded.quality_report.severity,
        DataQualitySeverity::Good | DataQualitySeverity::Warning
    ));
}

#[test]
fn gap_fixture_reports_gap_count() {
    let loaded = load_fixture("generic_ohlcv_gaps.csv", CandleCsvLoader::default(), false);
    assert!(loaded.quality_report.gap_count > 0);
}

#[test]
fn duplicate_fixture_reports_duplicate_timestamp_count() {
    let loader = CandleCsvLoader {
        validation: DataValidationConfig {
            allow_duplicate_drop: true,
            ..DataValidationConfig::default()
        },
        ..CandleCsvLoader::default()
    };
    let loaded = load_fixture("generic_ohlcv_duplicates.csv", loader, false);
    assert!(loaded.quality_report.duplicate_timestamp_count > 0);
}

#[test]
fn bad_ohlc_fixture_reports_invariant_violations() {
    let loaded = load_fixture(
        "generic_ohlcv_bad_ohlc.csv",
        CandleCsvLoader::default(),
        true,
    );
    assert!(loaded.quality_report.ohlc_invariant_violation_count > 0);
}

#[test]
fn data_quality_score_is_bounded() {
    let loaded = load_fixture("generic_ohlcv_gaps.csv", CandleCsvLoader::default(), false);
    assert!((0.0..=1.0).contains(&loaded.quality_report.data_quality_score));
}

#[test]
fn unusable_severity_occurs_for_too_many_invalid_rows() {
    let loader = CandleCsvLoader::default();
    let config = CandleCsvConfig {
        allow_drop_invalid_rows: true,
        max_invalid_rows: 10,
        ..CandleCsvConfig::default()
    };
    let input = "timestamp_ms,open,high,low,close,volume\n\
1700000000000,1,2,0.5,1.5,10\n\
1700000000000,0,2,0.5,1.5,-1\n\
1700000060000,0,2,0.5,1.5,-1\n\
1700000120000,0,2,0.5,1.5,-1\n";
    let loaded = loader
        .load_from_str(input, &config, None)
        .expect("drops invalid rows");
    assert_eq!(
        loaded.quality_report.severity,
        DataQualitySeverity::Unusable
    );
}

#[test]
fn low_data_quality_propagates_to_deny_or_no_trade_path() {
    let loaded = load_fixture("generic_ohlcv_gaps.csv", CandleCsvLoader::default(), false);
    let simulator = BacktestSimulator::default();
    let result = simulator.run(&loaded.series);
    assert_eq!(
        result.denied_trades + result.no_trades,
        result.total_decisions
    );
}

#[test]
fn data_manifest_is_deterministic_for_same_input() {
    let first = load_fixture("generic_ohlcv_valid.csv", CandleCsvLoader::default(), false);
    let second = load_fixture("generic_ohlcv_valid.csv", CandleCsvLoader::default(), false);
    assert_eq!(
        first.manifest.to_deterministic_string(),
        second.manifest.to_deterministic_string()
    );
}

#[test]
fn manifest_contains_first_last_timestamp_and_row_count() {
    let loaded = load_fixture("generic_ohlcv_valid.csv", CandleCsvLoader::default(), false);
    assert_eq!(loaded.manifest.first_timestamp_ms, 1_700_000_000_000);
    assert_eq!(loaded.manifest.last_timestamp_ms, 1_700_000_660_000);
    assert_eq!(loaded.manifest.row_count, 12);
}

#[test]
fn manifest_does_not_use_wall_clock_unless_passed_explicitly() {
    let loaded = load_fixture("generic_ohlcv_valid.csv", CandleCsvLoader::default(), false);
    assert_eq!(loaded.manifest.created_at_ms, None);
    assert!(
        loaded
            .manifest
            .reason_codes
            .contains(&ReasonCode::DataManifestBuilt)
    );
}

fn load_fixture(
    name: &str,
    loader: CandleCsvLoader,
    allow_drop_invalid_rows: bool,
) -> soma_zero::LoadedCandleData {
    let config = CandleCsvConfig {
        allow_drop_invalid_rows,
        max_invalid_rows: 10,
        ..CandleCsvConfig::default()
    };
    loader
        .load_from_path(&fixture_path(name), &config)
        .expect("fixture load")
}

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("market_data")
        .join(name)
}
