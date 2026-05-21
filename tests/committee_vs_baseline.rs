use std::collections::BTreeMap;

use soma_zero::{
    CommitteeFinalAction, CommitteeReplayReport, CommitteeScenarioMaterializationLevel,
    CommitteeScenarioRow, CommitteeScenarioSet, CommitteeScenarioSourceKind, EvidenceSourceKind,
    PersonaHorizon, ProviderMarket, ReasonCode, Regime, build_committee_vs_baseline_comparison,
};

fn scenario_row(outcome: bool, baseline: Option<&str>) -> CommitteeScenarioRow {
    CommitteeScenarioRow {
        scenario_row_id: "row".to_string(),
        symbol: "AAPL".to_string(),
        timestamp_ms: 1,
        source_kind: CommitteeScenarioSourceKind::OfficialBenchmarkReport,
        evidence_source_kind: EvidenceSourceKind::OfficialApiCollected,
        market: ProviderMarket::USEquity,
        target_horizon: PersonaHorizon::Swing,
        feature_vector: None,
        regime: Regime::TrendUp,
        signal_summary: "test".to_string(),
        data_quality_score: 0.9,
        spread_bps: Some(5.0),
        expected_edge_after_cost: 0.01,
        expected_drawdown: 0.02,
        risk_snapshot_summary: None,
        provenance_summary: "official".to_string(),
        benchmark_status: Some("row-level".to_string()),
        baseline_signal_summary: baseline.map(str::to_string),
        external_prediction_summary: None,
        no_trade_counterfactual: Some("always-no-trade".to_string()),
        risk_denial_counterfactual: Some("risk-denied".to_string()),
        outcome_reference: outcome.then(|| "outcome".to_string()),
        materialization_level: CommitteeScenarioMaterializationLevel::RowLevel,
        materialization_confidence: 0.9,
        reason_codes: vec![ReasonCode::CommitteeRowLevelMaterialized],
    }
}

#[test]
fn baseline_comparison_handles_missing_references_conservatively() {
    let set = CommitteeScenarioSet {
        scenario_id: "cmp".to_string(),
        rows: vec![scenario_row(false, None); 3],
        source_summary: "Official".to_string(),
        row_count: 3,
        official_row_count: 3,
        research_only_row_count: 0,
        fixture_row_count: 0,
        skipped_row_count: 0,
        reason_codes: vec![ReasonCode::CommitteeMaterializationBuilt],
    };
    let replay = CommitteeReplayReport {
        replay_id: "cmp".to_string(),
        records: vec![],
        record_count: 3,
        source_summary: "Official".to_string(),
        final_action_counts: BTreeMap::from([(
            format!("{:?}", CommitteeFinalAction::FinalDenied),
            3,
        )]),
        risk_denial_counts: BTreeMap::new(),
        chair_decision_counts: BTreeMap::new(),
        deterministic_fingerprint: "fp".to_string(),
        reason_codes: vec![ReasonCode::CommitteeReplayBuilt],
    };
    let report = build_committee_vs_baseline_comparison(&set, &replay);
    assert_eq!(
        report.comparison_status,
        soma_zero::CommitteeVsBaselineStatus::NoBaselineReference
    );
}
