use soma_zero::{
    CommitteeAttributionStatus, CommitteeDebateReplay, CommitteeScenarioLoadConfig,
    CommitteeScenarioLoader, CommitteeScenarioSourceKind, ReasonCode,
    build_committee_attribution_report,
};

#[test]
fn attribution_tracks_votes_and_is_deterministic() {
    let set = CommitteeScenarioLoader::default()
        .load(&CommitteeScenarioLoadConfig {
            scenario_id: "attr".to_string(),
            source_kind: CommitteeScenarioSourceKind::Fixture,
            output_root: "target/committee_attribution".to_string(),
            reason_codes: vec![ReasonCode::CommitteeScenarioLoaderBuilt],
            ..CommitteeScenarioLoadConfig::default()
        })
        .expect("load");
    let replay = CommitteeDebateReplay::default()
        .run_for_scenario_set("attr", &set, 10)
        .expect("replay");
    let first = build_committee_attribution_report(&replay);
    let second = build_committee_attribution_report(&replay);
    assert!(matches!(
        first.attribution_status,
        CommitteeAttributionStatus::InsufficientSamples
            | CommitteeAttributionStatus::Balanced
            | CommitteeAttributionStatus::PersonaDominated
            | CommitteeAttributionStatus::RiskDominated
            | CommitteeAttributionStatus::ChairDominated
            | CommitteeAttributionStatus::SourceLimited
    ));
    assert_eq!(first.to_text(), second.to_text());
}
