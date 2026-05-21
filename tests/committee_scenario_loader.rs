mod common;

use std::fs;

use soma_zero::{
    CommitteeScenarioLoadConfig, CommitteeScenarioLoader, CommitteeScenarioSourceKind, ReasonCode,
};

fn config(source_kind: CommitteeScenarioSourceKind) -> CommitteeScenarioLoadConfig {
    CommitteeScenarioLoadConfig {
        scenario_id: format!("scenario-{source_kind:?}"),
        source_kind,
        output_root: common::output_dir("committee-scenario-loader")
            .display()
            .to_string(),
        reason_codes: vec![ReasonCode::CommitteeScenarioLoaderBuilt],
        ..CommitteeScenarioLoadConfig::default()
    }
}

#[test]
fn fixture_config_loads_into_scenario_set() {
    let report = CommitteeScenarioLoader::default()
        .load(&config(CommitteeScenarioSourceKind::Fixture))
        .expect("load");
    assert_eq!(report.row_count, 2);
    assert_eq!(report.fixture_row_count, 2);
    assert!(report.source_summary.contains("Fixture"));
}

#[test]
fn yfinance_rows_load_as_research_only() {
    let report = CommitteeScenarioLoader::default()
        .load(&config(
            CommitteeScenarioSourceKind::YahooResearchEvidenceReport,
        ))
        .expect("load");
    assert_eq!(report.research_only_row_count, report.row_count);
    assert!(
        report.rows[0]
            .reason_codes
            .contains(&ReasonCode::SummaryDerived)
    );
}

#[test]
fn source_aware_benchmark_rows_preserve_source_kind() {
    let path = common::output_dir("committee-scenario-source-benchmark").join("report.json");
    fs::write(
        &path,
        r#"{"dataset_inventory":{"official_ready_count":1,"yfinance_benchmark_eligible_count":1}}"#,
    )
    .expect("write");
    let mut cfg = config(CommitteeScenarioSourceKind::SourceAwareBenchmarkReport);
    cfg.input_paths = vec![path.display().to_string()];
    let report = CommitteeScenarioLoader::default().load(&cfg).expect("load");
    assert_eq!(report.row_count, 2);
    assert!(
        report.rows.iter().all(|row| {
            row.source_kind == CommitteeScenarioSourceKind::SourceAwareBenchmarkReport
        })
    );
}

#[test]
fn remote_paths_are_rejected() {
    let mut cfg = config(CommitteeScenarioSourceKind::Fixture);
    cfg.output_root = "https://example.com/out".to_string();
    assert!(CommitteeScenarioLoader::default().load(&cfg).is_err());
}

#[test]
fn max_scenarios_are_enforced_and_reason_coded() {
    let mut cfg = config(CommitteeScenarioSourceKind::Fixture);
    cfg.max_scenarios = 1;
    let report = CommitteeScenarioLoader::default().load(&cfg).expect("load");
    assert_eq!(report.row_count, 1);
    assert!(
        report
            .reason_codes
            .contains(&ReasonCode::CommitteeScenarioRowsTruncated)
    );
}

#[test]
fn official_rows_keep_official_provenance_summary() {
    let report = CommitteeScenarioLoader::default()
        .load(&config(
            CommitteeScenarioSourceKind::OfficialBenchmarkReport,
        ))
        .expect("load");
    assert_eq!(report.official_row_count, 1);
    assert!(report.rows[0].provenance_summary.contains("official"));
}

#[test]
fn loader_is_deterministic() {
    let cfg = config(CommitteeScenarioSourceKind::Fixture);
    let first = CommitteeScenarioLoader::default()
        .load(&cfg)
        .expect("first");
    let second = CommitteeScenarioLoader::default()
        .load(&cfg)
        .expect("second");
    assert_eq!(
        first.to_json_string().expect("first json"),
        second.to_json_string().expect("second json")
    );
}
