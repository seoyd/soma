mod common;
#[path = "support/official_committee_support.rs"]
mod official_committee_support;

use std::fs;
use std::path::PathBuf;

use soma_zero::{
    CoreBottleneckKind, CorePerformanceFinalStatus, CorePerformanceScorecardConfig,
    CorePerformanceScorecardRunner, SignalQualityStatus,
};

fn write_json(name: &str, file_name: &str, contents: &str) -> PathBuf {
    let path = common::output_dir(name).join(file_name);
    fs::write(&path, contents).expect("write json");
    path
}

#[test]
fn controlled_only_runner_stays_diagnostic_only() {
    let pack_config =
        official_committee_support::controlled_pack_config("core-performance-controlled", false);
    let pack_config_path =
        official_committee_support::write_pack_config("core-performance-controlled", &pack_config);
    let linker_config = official_committee_support::controlled_linker_config(
        "core-performance-controlled",
        &common::output_dir("core-performance-controlled-pack-ref")
            .join("official_scenario_pack.json"),
        true,
    );
    let linker_config_path = official_committee_support::write_linker_config(
        "core-performance-controlled",
        &linker_config,
    );
    let benchmark_config = official_committee_support::controlled_benchmark_config(
        "core-performance-controlled",
        &pack_config_path,
        &linker_config_path,
        false,
    );
    let benchmark_config_path = official_committee_support::write_benchmark_config(
        "core-performance-controlled",
        &benchmark_config,
    );
    let coverage_config = official_committee_support::controlled_coverage_config(
        "core-performance-controlled",
        &benchmark_config_path,
        &pack_config_path,
        &official_committee_support::write_candle_series(
            "core-performance-controlled",
            "AAPL",
            1_700_000_000_000,
            1.0,
        ),
    );
    let coverage_path = official_committee_support::write_coverage_config(
        "core-performance-controlled",
        &coverage_config,
    );

    let config = CorePerformanceScorecardConfig {
        scorecard_id: "core-performance-controlled".to_string(),
        committee_outcome_coverage_paths: vec![coverage_path.display().to_string()],
        output_root: common::output_dir("core-performance-controlled-root")
            .display()
            .to_string(),
        ..CorePerformanceScorecardConfig::default()
    };
    let bundle = CorePerformanceScorecardRunner::default()
        .run(&config)
        .expect("controlled bundle");

    assert_eq!(
        bundle.scorecard.final_status,
        CorePerformanceFinalStatus::CoreDiagnosticOnly
    );
    assert!(bundle.scorecard.signal_quality_report.outcome_linked_rows > 0);
    assert!(
        bundle
            .scorecard
            .signal_quality_report
            .warnings
            .iter()
            .any(|warning| warning.contains("controlled evidence"))
    );
}

#[test]
fn runner_keeps_crypto_and_research_inputs_non_official() {
    let crypto_path = write_json(
        "core-performance-runner-crypto",
        "official_replication_crypto_only.json",
        "{\"status\":\"CryptoOnly\"}",
    );
    let crypto_bundle = CorePerformanceScorecardRunner::default()
        .run(&CorePerformanceScorecardConfig {
            scorecard_id: "core-performance-crypto".to_string(),
            official_replication_report_paths: vec![crypto_path.display().to_string()],
            output_root: common::output_dir("core-performance-crypto-root")
                .display()
                .to_string(),
            ..CorePerformanceScorecardConfig::default()
        })
        .expect("crypto bundle");
    assert_eq!(
        crypto_bundle
            .scorecard
            .signal_quality_report
            .signal_quality_status,
        SignalQualityStatus::CryptoOnly
    );

    let research_path = write_json(
        "core-performance-runner-research",
        "yahoo_research_report.json",
        "{}",
    );
    let research_bundle = CorePerformanceScorecardRunner::default()
        .run(&CorePerformanceScorecardConfig {
            scorecard_id: "core-performance-research".to_string(),
            yahoo_research_report_paths: vec![research_path.display().to_string()],
            output_root: common::output_dir("core-performance-research-root")
                .display()
                .to_string(),
            ..CorePerformanceScorecardConfig::default()
        })
        .expect("research bundle");
    assert_eq!(
        research_bundle
            .scorecard
            .signal_quality_report
            .signal_quality_status,
        SignalQualityStatus::ResearchOnly
    );
}

#[test]
fn runner_blocks_when_official_evidence_is_missing() {
    let config = CorePerformanceScorecardConfig {
        scorecard_id: "core-performance-missing-official".to_string(),
        output_root: common::output_dir("core-performance-missing-official-root")
            .display()
            .to_string(),
        ..CorePerformanceScorecardConfig::default()
    };
    let bundle = CorePerformanceScorecardRunner::default()
        .run(&config)
        .expect("missing official bundle");

    assert_eq!(
        bundle.scorecard.final_status,
        CorePerformanceFinalStatus::CoreBlockedByOfficialData
    );
    assert_eq!(
        bundle.scorecard.bottleneck_report.primary_bottleneck,
        CoreBottleneckKind::MissingOfficialData
    );
}

#[test]
fn runner_loads_and_summarizes_official_replication_reports() {
    let config = CorePerformanceScorecardConfig {
        scorecard_id: "core-performance-example-official".to_string(),
        official_replication_report_paths: vec![
            "examples/soma_official_replication_aapl_controlled_official.toml".to_string(),
        ],
        output_root: common::output_dir("core-performance-example-official-root")
            .display()
            .to_string(),
        ..CorePerformanceScorecardConfig::default()
    };

    let bundle = CorePerformanceScorecardRunner::default()
        .run(&config)
        .expect("official bundle");
    assert!(
        bundle
            .scorecard
            .artifact_inventory
            .non_crypto_official_count
            > 0
    );
    assert_ne!(
        bundle.scorecard.final_status,
        CorePerformanceFinalStatus::CoreDiagnosticOnly
    );
    assert!(
        bundle
            .scorecard
            .signal_quality_report
            .official_evaluated_rows
            > 0
    );
    assert!(
        PathBuf::from(&bundle.output_dir)
            .join("core_performance_scorecard.json")
            .exists()
    );
}

#[test]
fn runner_is_deterministic_for_same_input() {
    let config = CorePerformanceScorecardConfig {
        scorecard_id: "core-performance-deterministic".to_string(),
        output_root: common::output_dir("core-performance-deterministic-root")
            .display()
            .to_string(),
        ..CorePerformanceScorecardConfig::default()
    };
    let runner = CorePerformanceScorecardRunner::default();

    let first = runner.run(&config).expect("first");
    let second = runner.run(&config).expect("second");

    assert_eq!(
        first.scorecard.to_json_string().expect("first json"),
        second.scorecard.to_json_string().expect("second json")
    );
}
