mod common;

use soma_zero::{
    CommitteeArtifactKind, CommitteeBenchmarkConfig, CommitteeBenchmarkFinalStatus,
    CommitteeBenchmarkRunner, CommitteeMaterializationConfig, ReasonCode,
};

fn materialization_config(name: &str, artifact_kind: CommitteeArtifactKind) -> std::path::PathBuf {
    let path = common::output_dir(&format!("{name}-mat")).join("materialize.toml");
    std::fs::write(
        &path,
        CommitteeMaterializationConfig {
            materialization_id: name.to_string(),
            allowed_artifact_kinds: vec![artifact_kind],
            output_root: common::output_dir(&format!("{name}-out"))
                .display()
                .to_string(),
            reason_codes: vec![ReasonCode::CommitteeMaterializationBuilt],
            ..CommitteeMaterializationConfig::default()
        }
        .to_toml_string()
        .expect("toml"),
    )
    .expect("write");
    path
}

#[test]
fn benchmark_runner_stays_conservative_for_fixture_and_yfinance() {
    let fixture = CommitteeBenchmarkRunner::default()
        .run(&CommitteeBenchmarkConfig {
            benchmark_id: "benchmark-fixture".to_string(),
            materialization_config_path: Some(
                materialization_config("benchmark-fixture", CommitteeArtifactKind::FixtureScenario)
                    .display()
                    .to_string(),
            ),
            output_root: common::output_dir("benchmark-fixture-run")
                .display()
                .to_string(),
            require_core_check: false,
            ..CommitteeBenchmarkConfig::default()
        })
        .expect("fixture");
    assert_eq!(
        fixture.benchmark_report.final_status,
        CommitteeBenchmarkFinalStatus::FixtureOnlyBenchmark
    );

    let yfinance = CommitteeBenchmarkRunner::default()
        .run(&CommitteeBenchmarkConfig {
            benchmark_id: "benchmark-yfinance".to_string(),
            materialization_config_path: Some(
                materialization_config(
                    "benchmark-yfinance",
                    CommitteeArtifactKind::YahooResearchEvidenceReport,
                )
                .display()
                .to_string(),
            ),
            output_root: common::output_dir("benchmark-yfinance-run")
                .display()
                .to_string(),
            require_core_check: false,
            ..CommitteeBenchmarkConfig::default()
        })
        .expect("yfinance");
    assert_eq!(
        yfinance.benchmark_report.final_status,
        CommitteeBenchmarkFinalStatus::ResearchOnlyBenchmark
    );
}

#[test]
fn benchmark_runner_is_deterministic_and_includes_reports() {
    let cfg = CommitteeBenchmarkConfig {
        benchmark_id: "benchmark-det".to_string(),
        materialization_config_path: Some(
            materialization_config("benchmark-det", CommitteeArtifactKind::FixtureScenario)
                .display()
                .to_string(),
        ),
        output_root: common::output_dir("benchmark-det-run")
            .display()
            .to_string(),
        require_core_check: false,
        ..CommitteeBenchmarkConfig::default()
    };
    let first = CommitteeBenchmarkRunner::default()
        .run(&cfg)
        .expect("first");
    let second = CommitteeBenchmarkRunner::default()
        .run(&cfg)
        .expect("second");
    assert_eq!(first.to_text(), second.to_text());
    assert!(!first.diagnostics_summary.chair.reports.is_empty());
    assert!(!first.diagnostics_summary.risk.reports.is_empty());
}
