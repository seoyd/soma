use std::path::PathBuf;

use soma_zero::{
    CandleCsvConfig, CandleCsvLoader, ReasonCode, ResampleConfig, Resampler, Timeframe,
};

#[test]
fn one_minute_to_five_minute_ohlcv_aggregation_is_correct() {
    let loaded = load_valid_fixture();
    let result = Resampler
        .resample(
            &loaded.series,
            &ResampleConfig {
                source_timeframe: Timeframe::OneMinute,
                target_timeframe: Timeframe::FiveMinute,
                ..ResampleConfig::default()
            },
        )
        .expect("resampled");
    assert_eq!(result.series.len(), 2);
    let first = &result.series.candles[0];
    assert_eq!(first.open, 100.0);
    assert_eq!(first.close, 101.7);
    assert_eq!(first.high, 101.9);
    assert_eq!(first.low, 99.7);
    assert_eq!(first.volume, 638.0);
}

#[test]
fn partial_windows_are_dropped_by_default() {
    let loaded = load_valid_fixture();
    let result = Resampler
        .resample(&loaded.series, &ResampleConfig::default())
        .expect("resampled");
    assert_eq!(result.series.len(), 2);
    assert!(
        result
            .reason_codes
            .contains(&ReasonCode::PartialWindowDropped)
    );
}

#[test]
fn resampling_is_deterministic() {
    let loaded = load_valid_fixture();
    let first = Resampler
        .resample(&loaded.series, &ResampleConfig::default())
        .expect("resampled");
    let second = Resampler
        .resample(&loaded.series, &ResampleConfig::default())
        .expect("resampled");
    assert_eq!(first, second);
}

#[test]
fn gaps_in_source_can_be_rejected() {
    let loaded = CandleCsvLoader::default()
        .load_from_path(
            &fixture_path("generic_ohlcv_gaps.csv"),
            &CandleCsvConfig::default(),
        )
        .expect("gap fixture loads");
    let error = Resampler
        .resample(&loaded.series, &ResampleConfig::default())
        .expect_err("non contiguous source");
    assert_eq!(error, vec![ReasonCode::NonContiguousSource]);
}

fn load_valid_fixture() -> soma_zero::LoadedCandleData {
    CandleCsvLoader::default()
        .load_from_path(
            &fixture_path("generic_ohlcv_valid.csv"),
            &CandleCsvConfig::default(),
        )
        .expect("valid fixture")
}

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("market_data")
        .join(name)
}
