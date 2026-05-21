mod common;

use soma_zero::{
    CommitteeDiagnosticsConfig, CommitteeDiagnosticsRunner, CommitteeScenarioLoadConfig,
    CommitteeScenarioSourceKind, ReasonCode,
};

fn diagnostics_config(
    name: &str,
    source_kind: CommitteeScenarioSourceKind,
) -> CommitteeDiagnosticsConfig {
    let scenario_load_path = common::output_dir(&format!("{name}-config")).join("load.toml");
    std::fs::write(
        &scenario_load_path,
        CommitteeScenarioLoadConfig {
            scenario_id: format!("{name}-scenarios"),
            source_kind,
            output_root: common::output_dir(&format!("{name}-out"))
                .display()
                .to_string(),
            reason_codes: vec![ReasonCode::CommitteeScenarioLoaderBuilt],
            ..CommitteeScenarioLoadConfig::default()
        }
        .to_toml_string()
        .expect("toml"),
    )
    .expect("write");
    CommitteeDiagnosticsConfig {
        diagnostic_id: name.to_string(),
        scenario_load_config_path: Some(scenario_load_path.display().to_string()),
        output_root: common::output_dir(&format!("{name}-diag"))
            .display()
            .to_string(),
        reason_codes: vec![ReasonCode::CommitteeDiagnosticsBuilt],
        ..CommitteeDiagnosticsConfig::default()
    }
}

#[test]
fn fixture_diagnostics_report_evidence_too_weak() {
    let bundle = CommitteeDiagnosticsRunner::default()
        .run(&diagnostics_config(
            "committee-diagnostics-fixture",
            CommitteeScenarioSourceKind::Fixture,
        ))
        .expect("run");
    assert_eq!(
        bundle.diagnostics.final_status,
        soma_zero::CommitteeDiagnosticsStatus::EvidenceTooWeak
    );
}

#[test]
fn yfinance_diagnostics_remain_research_only() {
    let bundle = CommitteeDiagnosticsRunner::default()
        .run(&diagnostics_config(
            "committee-diagnostics-yfinance",
            CommitteeScenarioSourceKind::YahooResearchEvidenceReport,
        ))
        .expect("run");
    assert_eq!(
        bundle.diagnostics.final_status,
        soma_zero::CommitteeDiagnosticsStatus::ResearchOnly
    );
}

#[test]
fn diagnostics_are_deterministic() {
    let cfg = diagnostics_config(
        "committee-diagnostics-deterministic",
        CommitteeScenarioSourceKind::OfficialBenchmarkReport,
    );
    let first = CommitteeDiagnosticsRunner::default()
        .run(&cfg)
        .expect("first");
    let second = CommitteeDiagnosticsRunner::default()
        .run(&cfg)
        .expect("second");
    assert_eq!(first.to_text(), second.to_text());
}
