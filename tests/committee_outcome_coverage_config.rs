mod common;
#[path = "support/official_committee_support.rs"]
mod official_committee_support;

use soma_zero::CommitteeOutcomeCoverageConfig;

#[test]
fn outcome_coverage_config_defaults_are_conservative() {
    let config = CommitteeOutcomeCoverageConfig::default();
    assert!(config.validate().is_ok());
    assert_eq!(config.max_rows, 100);
    assert_eq!(config.max_symbols, 3);
    assert!(!config.allow_yfinance_research);
    assert!(!config.allow_fixture);
    assert!(!config.allow_estimated_counterfactuals);
    assert!(config.require_no_lookahead_safe);
}

#[test]
fn outcome_coverage_config_rejects_remote_paths() {
    let config = CommitteeOutcomeCoverageConfig {
        official_benchmark_report_paths: vec!["https://example.com/report.json".to_string()],
        ..CommitteeOutcomeCoverageConfig::default()
    };
    assert!(config.validate().is_err());
}

#[test]
fn outcome_coverage_config_enforces_bounds() {
    assert!(
        CommitteeOutcomeCoverageConfig {
            max_rows: 101,
            ..CommitteeOutcomeCoverageConfig::default()
        }
        .validate()
        .is_err()
    );
    assert!(
        CommitteeOutcomeCoverageConfig {
            max_symbols: 11,
            ..CommitteeOutcomeCoverageConfig::default()
        }
        .validate()
        .is_err()
    );
    assert!(
        CommitteeOutcomeCoverageConfig {
            max_bytes: 5_000_001,
            ..CommitteeOutcomeCoverageConfig::default()
        }
        .validate()
        .is_err()
    );
}
