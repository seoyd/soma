mod common;

use soma_zero::{
    CommitteeScenarioLoadConfig, CommitteeScenarioLoader, CommitteeScenarioSourceKind,
    CommitteeV1FinalStatus, CommitteeV1RunConfig, CommitteeV1Runner, ReasonCode,
};

fn run_config(name: &str, source_kind: CommitteeScenarioSourceKind) -> CommitteeV1RunConfig {
    let load_path = common::output_dir(&format!("{name}-cfg")).join("load.toml");
    std::fs::write(
        &load_path,
        CommitteeScenarioLoadConfig {
            scenario_id: name.to_string(),
            source_kind,
            output_root: common::output_dir(&format!("{name}-scenarios"))
                .display()
                .to_string(),
            reason_codes: vec![ReasonCode::CommitteeScenarioLoaderBuilt],
            ..CommitteeScenarioLoadConfig::default()
        }
        .to_toml_string()
        .expect("toml"),
    )
    .expect("write");
    CommitteeV1RunConfig {
        run_id: name.to_string(),
        scenario_load_config_path: Some(load_path.display().to_string()),
        output_root: common::output_dir(&format!("{name}-out"))
            .display()
            .to_string(),
        reason_codes: vec![ReasonCode::CommitteeV1Built],
        ..CommitteeV1RunConfig::default()
    }
}

#[test]
fn zero_scenarios_need_evidence() {
    let set_path = common::output_dir("committee-v1-zero").join("committee_scenario_set.json");
    let scenario_set = CommitteeScenarioLoader::default()
        .load(&CommitteeScenarioLoadConfig {
            scenario_id: "committee-v1-zero".to_string(),
            source_kind: CommitteeScenarioSourceKind::Unknown,
            output_root: common::output_dir("committee-v1-zero-set")
                .display()
                .to_string(),
            reason_codes: vec![ReasonCode::CommitteeScenarioLoaderBuilt],
            ..CommitteeScenarioLoadConfig::default()
        })
        .expect("load");
    std::fs::write(&set_path, scenario_set.to_json_string().expect("json")).expect("write");
    let load_path = common::output_dir("committee-v1-zero-cfg").join("load.toml");
    std::fs::write(
        &load_path,
        CommitteeScenarioLoadConfig {
            scenario_id: "committee-v1-zero".to_string(),
            source_kind: CommitteeScenarioSourceKind::Unknown,
            output_root: common::output_dir("committee-v1-zero-load")
                .display()
                .to_string(),
            reason_codes: vec![ReasonCode::CommitteeScenarioLoaderBuilt],
            ..CommitteeScenarioLoadConfig::default()
        }
        .to_toml_string()
        .expect("toml"),
    )
    .expect("write");
    let report = CommitteeV1Runner::default()
        .run(&CommitteeV1RunConfig {
            run_id: "committee-v1-zero".to_string(),
            scenario_load_config_path: Some(load_path.display().to_string()),
            output_root: common::output_dir("committee-v1-zero-run")
                .display()
                .to_string(),
            ..CommitteeV1RunConfig::default()
        })
        .expect("run");
    assert_eq!(
        report.final_status,
        CommitteeV1FinalStatus::CommitteeV1NeedsEvidence
    );
}

#[test]
fn fixture_and_yfinance_remain_conservative() {
    let fixture = CommitteeV1Runner::default()
        .run(&run_config(
            "committee-v1-fixture",
            CommitteeScenarioSourceKind::Fixture,
        ))
        .expect("fixture");
    assert_eq!(
        fixture.final_status,
        CommitteeV1FinalStatus::CommitteeV1FixtureOnly
    );

    let yfinance = CommitteeV1Runner::default()
        .run(&run_config(
            "committee-v1-yfinance",
            CommitteeScenarioSourceKind::YahooResearchEvidenceReport,
        ))
        .expect("yfinance");
    assert_eq!(
        yfinance.final_status,
        CommitteeV1FinalStatus::CommitteeV1ResearchOnly
    );
}

#[test]
fn all_risk_denied_paths_stay_conservative_not_live() {
    let report = CommitteeV1Runner::default()
        .run(&run_config(
            "committee-v1-official",
            CommitteeScenarioSourceKind::OfficialBenchmarkReport,
        ))
        .expect("run");
    assert!(matches!(
        report.final_status,
        CommitteeV1FinalStatus::CommitteeV1NeedsEvidence
            | CommitteeV1FinalStatus::CommitteeV1NeedsRiskReview
            | CommitteeV1FinalStatus::CommitteeV1ResearchReady
    ));
    assert!(!report.to_text().contains("live_ready"));
}

#[test]
fn committee_v1_runner_is_deterministic() {
    let cfg = run_config(
        "committee-v1-deterministic",
        CommitteeScenarioSourceKind::Fixture,
    );
    let first = CommitteeV1Runner::default().run(&cfg).expect("first");
    let second = CommitteeV1Runner::default().run(&cfg).expect("second");
    assert_eq!(first.to_text(), second.to_text());
}
