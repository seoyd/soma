mod common;

use soma_zero::{
    CommitteeArtifactKind, CommitteeDebateReplay, CommitteeMaterializationConfig,
    CommitteeReplayConfig, CommitteeScenarioLoadConfig, CommitteeScenarioLoader,
    CommitteeScenarioMaterializationLevel, CommitteeScenarioMaterializerV2,
    CommitteeScenarioSourceKind, EvidenceSourceKind, PersonaHorizon, PersonaStance, PersonaVote,
    ReasonCode, build_status_report_from_votes, idle_trinity_operational_status_report,
};

fn replay_config(name: &str) -> CommitteeReplayConfig {
    let scenario_set = CommitteeScenarioLoader::default()
        .load(&CommitteeScenarioLoadConfig {
            scenario_id: name.to_string(),
            source_kind: CommitteeScenarioSourceKind::Fixture,
            output_root: common::output_dir(&format!("{name}-scenarios"))
                .display()
                .to_string(),
            reason_codes: vec![ReasonCode::CommitteeScenarioLoaderBuilt],
            ..CommitteeScenarioLoadConfig::default()
        })
        .expect("load scenarios");
    let scenario_set_path = scenario_set
        .write_to_dir(&common::output_dir(&format!("{name}-set")))
        .expect("write set");
    CommitteeReplayConfig {
        replay_id: name.to_string(),
        scenario_set_path: Some(scenario_set_path.display().to_string()),
        output_root: common::output_dir(&format!("{name}-replay"))
            .display()
            .to_string(),
        reason_codes: vec![ReasonCode::CommitteeReplayBuilt],
        ..CommitteeReplayConfig::default()
    }
}

#[test]
fn committee_materializer_preserves_fixture_research_and_provenance_rules() {
    let artifact = common::output_dir("committee-materializer-fixture").join("fixture_rows.json");
    std::fs::write(
        &artifact,
        r#"{"rows":[{"symbol":"BTC-KRW"},{"symbol":"ETH-KRW"},{"symbol":"XRP-KRW"}]}"#,
    )
    .expect("write");
    let cfg = CommitteeMaterializationConfig {
        materialization_id: "committee-materializer-fixture".to_string(),
        input_artifact_paths: vec![artifact.display().to_string()],
        allowed_artifact_kinds: vec![CommitteeArtifactKind::FixtureScenario],
        output_root: common::output_dir("committee-materializer-fixture-out")
            .display()
            .to_string(),
        max_rows: 2,
        ..CommitteeMaterializationConfig::default()
    };
    let first = CommitteeScenarioMaterializerV2::default()
        .materialize(&cfg)
        .expect("first");
    let second = CommitteeScenarioMaterializerV2::default()
        .materialize(&cfg)
        .expect("second");
    assert_eq!(first.row_count, 2);
    assert!(first.rows.iter().all(|row| {
        row.materialization_level == CommitteeScenarioMaterializationLevel::RowLevel
    }));
    assert_eq!(first.to_text(), second.to_text());

    let artifact = common::output_dir("committee-materializer-yfinance").join("yfinance_rows.json");
    std::fs::write(&artifact, r#"{"yfinance_symbols":["AAPL","MSFT"]}"#).expect("write");
    let cfg = CommitteeMaterializationConfig {
        materialization_id: "committee-materializer-yfinance".to_string(),
        input_artifact_paths: vec![artifact.display().to_string()],
        allowed_artifact_kinds: vec![CommitteeArtifactKind::YahooResearchEvidenceReport],
        output_root: common::output_dir("committee-materializer-yfinance-out")
            .display()
            .to_string(),
        ..CommitteeMaterializationConfig::default()
    };
    let set = CommitteeScenarioMaterializerV2::default()
        .materialize(&cfg)
        .expect("materialize");
    assert!(
        set.rows
            .iter()
            .all(|row| row.evidence_source_kind == EvidenceSourceKind::YFinanceResearch)
    );
    assert!(
        set.rows
            .iter()
            .all(|row| (0.0..=1.0).contains(&row.materialization_confidence))
    );
}

#[test]
fn committee_materializer_requires_provenance_or_marks_summary_fallback() {
    let artifact = common::output_dir("committee-materializer-official").join("source_rows.json");
    std::fs::write(&artifact, r#"{"rows":[{"symbol":"AAPL"}]}"#).expect("write");
    let cfg = CommitteeMaterializationConfig {
        materialization_id: "committee-materializer-official".to_string(),
        input_artifact_paths: vec![artifact.display().to_string()],
        allowed_artifact_kinds: vec![CommitteeArtifactKind::SourceAwareBenchmarkReport],
        output_root: common::output_dir("committee-materializer-official-out")
            .display()
            .to_string(),
        allow_summary_derived_rows: false,
        require_provenance: true,
        ..CommitteeMaterializationConfig::default()
    };
    let set = CommitteeScenarioMaterializerV2::default()
        .materialize(&cfg)
        .expect("materialize");
    assert!(set.rows.is_empty());

    let cfg = CommitteeMaterializationConfig {
        materialization_id: "committee-materializer-summary".to_string(),
        input_artifact_paths: vec!["virtual-yfinance".to_string()],
        allowed_artifact_kinds: vec![CommitteeArtifactKind::YahooResearchEvidenceReport],
        output_root: common::output_dir("committee-materializer-summary-out")
            .display()
            .to_string(),
        allow_summary_derived_rows: true,
        require_provenance: false,
        ..CommitteeMaterializationConfig::default()
    };
    let set = CommitteeScenarioMaterializerV2::default()
        .materialize(&cfg)
        .expect("materialize");
    assert!(set.rows.iter().any(|row| {
        row.reason_codes
            .contains(&ReasonCode::CommitteeSummaryFallbackUsed)
    }));
}

#[test]
fn committee_replay_is_stable_and_has_no_wall_clock() {
    let cfg = replay_config("committee-replay-stable");
    let report = CommitteeDebateReplay::default().run(&cfg).expect("replay");
    assert_eq!(report.record_count, report.records.len());
    assert!(!report.records.is_empty());
    assert_eq!(
        report.records[0].replay_fingerprint,
        CommitteeDebateReplay::default()
            .run(&cfg)
            .expect("replay again")
            .records[0]
            .replay_fingerprint
    );

    let mut cfg = replay_config("committee-replay-max");
    cfg.max_decisions = 1;
    let report = CommitteeDebateReplay::default().run(&cfg).expect("replay");
    let text = report.to_text();
    assert_eq!(report.record_count, 1);
    assert!(!text.contains("2026-"));
    assert!(!text.contains("timestamp_now"));
}

#[test]
fn committee_replay_and_persona_status_stay_trinity_only_and_paper_only() {
    let cfg = replay_config("committee-replay-risk");
    let first = CommitteeDebateReplay::default().run(&cfg).expect("first");
    let second = CommitteeDebateReplay::default().run(&cfg).expect("second");
    assert_eq!(
        first.records[0].risk_bridge_outcome.risk_decision.kind,
        second.records[0].risk_bridge_outcome.risk_decision.kind
    );

    assert_eq!(idle_trinity_operational_status_report().active_count, 3);
    let report = build_status_report_from_votes(
        "candidate",
        "AAPL",
        &[
            PersonaVote {
                persona_id: "trend_breakout_fast".to_string(),
                stance: PersonaStance::Approve,
                conviction: 0.7,
                voice_power: 0.8,
                horizon: PersonaHorizon::Intraday,
                source_kind: EvidenceSourceKind::OfficialApiCollected,
                regime_fit: 1.0,
                data_quality_fit: 1.0,
                risk_fit: 1.0,
                expected_edge_fit: 1.0,
                doctrine_violations: vec![],
                reason_codes: vec![],
            },
            PersonaVote {
                persona_id: "defensive_value_risk".to_string(),
                stance: PersonaStance::Abstain,
                conviction: 0.4,
                voice_power: 0.5,
                horizon: PersonaHorizon::Swing,
                source_kind: EvidenceSourceKind::OfficialApiCollected,
                regime_fit: 0.8,
                data_quality_fit: 0.9,
                risk_fit: 0.8,
                expected_edge_fit: 0.6,
                doctrine_violations: vec![],
                reason_codes: vec![],
            },
            PersonaVote {
                persona_id: "cycle_regime_guard".to_string(),
                stance: PersonaStance::Veto,
                conviction: 0.9,
                voice_power: 0.6,
                horizon: PersonaHorizon::Swing,
                source_kind: EvidenceSourceKind::OfficialApiCollected,
                regime_fit: 0.7,
                data_quality_fit: 0.7,
                risk_fit: 0.7,
                expected_edge_fit: 0.4,
                doctrine_violations: vec!["hard-stop".to_string()],
                reason_codes: vec![],
            },
        ],
    );
    let json = serde_json::to_string(&report).expect("json");
    assert_eq!(report.active_count, 3);
    assert!(!json.to_ascii_lowercase().contains("runtime_llm"));
    assert!(!json.to_ascii_lowercase().contains("order_id"));
    assert!(!json.to_ascii_lowercase().contains("account_id"));
}
