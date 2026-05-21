use std::path::PathBuf;

use soma_zero::{KRXCandleSufficiencyReport, KRXCandleSufficiencyStatus};

fn example_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join(name)
}

#[test]
fn candle_sufficiency_counts_future_window_gaps() {
    let barrier = example_path("soma_barrier_profiles_primary.toml");
    let report = KRXCandleSufficiencyReport::build(
        "sprint50-sufficiency",
        &vec![
            example_path("sprint50_data/krx_005930_extended_1d.csv")
                .display()
                .to_string(),
            example_path("sprint50_data/krx_000660_extended_1d.csv")
                .display()
                .to_string(),
        ],
        Some(barrier.to_str().expect("barrier path")),
    );
    assert_eq!(report.official_ready_series, 2);
    assert_eq!(report.benchmark_ready_series, 1);
    assert_eq!(report.series_missing_future_window, 1);
    assert_eq!(
        report.sufficiency_status,
        KRXCandleSufficiencyStatus::MissingFutureWindows
    );
}
