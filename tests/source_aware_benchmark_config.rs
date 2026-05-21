use std::path::PathBuf;

use soma_zero::SourceAwareBenchmarkConfig;

fn example_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join(name)
}

#[test]
fn source_aware_benchmark_config_can_be_constructed() {
    let config = SourceAwareBenchmarkConfig::default();
    assert!(!config.benchmark_id.is_empty());
}

#[test]
fn source_aware_benchmark_remote_paths_are_rejected() {
    let config = SourceAwareBenchmarkConfig {
        official_collection_report_paths: vec!["https://example.com/report.json".to_string()],
        ..SourceAwareBenchmarkConfig::default()
    };
    assert!(
        config
            .validate_local_paths()
            .contains(&soma_zero::ReasonCode::LocalPathRejected)
    );
}

#[test]
fn yfinance_only_example_allows_research_only() {
    let config = SourceAwareBenchmarkConfig::from_toml_path(&example_path(
        "soma_source_benchmark_yfinance_only.toml",
    ))
    .expect("parse");
    assert!(config.allow_yfinance_only_research);
}

#[test]
fn official_vs_yfinance_example_requires_official_source_paths() {
    let config = SourceAwareBenchmarkConfig::from_toml_path(&example_path(
        "soma_source_benchmark_official_vs_yfinance.toml",
    ))
    .expect("parse");
    assert!(!config.official_benchmark_report_paths.is_empty());
    assert!(!config.official_collection_report_paths.is_empty());
}
