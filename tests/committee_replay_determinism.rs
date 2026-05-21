mod common;

use std::fs;

use soma_zero::{
    CommitteeDebateReplay, CommitteeReplayConfig, CommitteeScenarioLoadConfig,
    CommitteeScenarioSourceKind, ReasonCode,
};

#[test]
fn replay_output_is_deterministic() {
    let load_path = common::output_dir("committee-replay-det").join("load.toml");
    fs::write(
        &load_path,
        CommitteeScenarioLoadConfig {
            scenario_id: "committee-replay-det".to_string(),
            source_kind: CommitteeScenarioSourceKind::Fixture,
            output_root: common::output_dir("committee-replay-det-out")
                .display()
                .to_string(),
            reason_codes: vec![ReasonCode::CommitteeScenarioLoaderBuilt],
            ..CommitteeScenarioLoadConfig::default()
        }
        .to_toml_string()
        .expect("toml"),
    )
    .expect("write");
    let smoke_config_path = common::output_dir("committee-replay-det-smoke").join("smoke.toml");
    fs::write(
        &smoke_config_path,
        soma_zero::CommitteeSmokeTestConfig {
            test_id: "committee-replay-det-smoke".to_string(),
            require_core_check: false,
            reason_codes: vec![ReasonCode::CommitteeSmokeTestBuilt],
            ..soma_zero::CommitteeSmokeTestConfig::default()
        }
        .to_toml_string()
        .expect("toml"),
    )
    .expect("write");
    let cfg = CommitteeReplayConfig {
        replay_id: "committee-replay-det".to_string(),
        committee_smoke_config_path: Some(smoke_config_path.display().to_string()),
        output_root: common::output_dir("committee-replay-det-final")
            .display()
            .to_string(),
        require_core_check: false,
        reason_codes: vec![ReasonCode::CommitteeReplayBuilt],
        ..CommitteeReplayConfig::default()
    };
    let first = CommitteeDebateReplay::default().run(&cfg).expect("first");
    let second = CommitteeDebateReplay::default().run(&cfg).expect("second");
    assert_eq!(first.to_text(), second.to_text());
}
