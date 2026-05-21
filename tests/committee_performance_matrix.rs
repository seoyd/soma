mod common;
#[path = "support/official_committee_support.rs"]
mod official_committee_support;

use soma_zero::{
    CommitteeCounterfactualBuildConfig, CommitteeCounterfactualBuilder,
    CommitteeOutcomeCoverageConfig, CommitteePerformanceStatus, CommitteeScenarioRow,
    CommitteeScenarioSourceKind, EvidenceSourceKind, OutcomeLinkedCommitteeScenarioPack,
    OutcomeLinkedCommitteeScenarioRow, PersonaHorizon, ProviderMarket, Regime,
    build_committee_outcome_coverage_report, build_committee_performance_evidence_matrix,
    load_local_candle_series_map,
};

#[test]
fn performance_matrix_builds_comparisons_and_defensive_proxies() {
    let bundle =
        official_committee_support::build_controlled_benchmark_bundle("performance-matrix", true);
    let candle_path = official_committee_support::write_candle_series(
        "performance-matrix",
        "AAPL",
        1_700_000_000_000,
        -1.0,
    );
    let series =
        load_local_candle_series_map(&[candle_path.display().to_string()]).expect("series");
    let records = bundle
        .outcome_linked_pack
        .linked_rows
        .iter()
        .flat_map(|row| {
            CommitteeCounterfactualBuilder::default().build_records(
                row,
                series.get("AAPL"),
                &CommitteeCounterfactualBuildConfig::default(),
            )
        })
        .collect::<Vec<_>>();
    let coverage = build_committee_outcome_coverage_report(
        &CommitteeOutcomeCoverageConfig::default(),
        &[bundle.official_scenario_pack.clone()],
        &[bundle.outcome_linked_pack.clone()],
        &records,
    );
    let matrix = build_committee_performance_evidence_matrix(
        "performance-matrix",
        &coverage,
        &[bundle.outcome_linked_pack.clone()],
        &[bundle.committee_benchmark_report.replay_report.clone()],
        &records,
        false,
    );
    assert_eq!(matrix.total_comparable_rows, 3);
    assert!(matrix.committee_better_than_notrade_count <= matrix.total_comparable_rows);
    assert!(matrix.risk_denied_defensive_value_total >= 0.0);
    assert!(matrix.no_trade_defensive_value_total >= 0.0);
    assert!(
        matrix
            .cells
            .iter()
            .any(|cell| cell.baseline_action.is_some())
    );
    assert!(matches!(
        matrix.performance_status,
        CommitteePerformanceStatus::EvidencePositive
            | CommitteePerformanceStatus::EvidenceMixed
            | CommitteePerformanceStatus::EvidenceNegative
    ));
}

#[test]
fn performance_matrix_marks_research_fixture_and_crypto_only_sources() {
    let scenario_row = CommitteeScenarioRow {
        scenario_row_id: "row-1".to_string(),
        symbol: "BTC-KRW".to_string(),
        timestamp_ms: 1,
        source_kind: CommitteeScenarioSourceKind::Fixture,
        evidence_source_kind: EvidenceSourceKind::YFinanceResearch,
        market: ProviderMarket::Crypto,
        target_horizon: PersonaHorizon::Swing,
        feature_vector: None,
        regime: Regime::Unknown,
        signal_summary: "test".to_string(),
        data_quality_score: 0.9,
        spread_bps: Some(5.0),
        expected_edge_after_cost: 0.01,
        expected_drawdown: 0.02,
        risk_snapshot_summary: None,
        provenance_summary: "fixture".to_string(),
        benchmark_status: None,
        baseline_signal_summary: None,
        external_prediction_summary: None,
        no_trade_counterfactual: None,
        risk_denial_counterfactual: None,
        outcome_reference: Some("outcome-1".to_string()),
        materialization_level: soma_zero::CommitteeScenarioMaterializationLevel::RowLevel,
        materialization_confidence: 1.0,
        reason_codes: vec![],
    };
    let linked_pack = OutcomeLinkedCommitteeScenarioPack {
        pack: soma_zero::OfficialCommitteeScenarioPack {
            pack_id: "synthetic".to_string(),
            rows: vec![scenario_row.clone()],
            source_summary: "YFinanceResearch=1".to_string(),
            official_row_count: 0,
            crypto_only_row_count: 1,
            yfinance_row_count: 1,
            fixture_row_count: 1,
            row_level_count: 1,
            summary_derived_count: 0,
            outcome_linked_count: 1,
            baseline_reference_count: 0,
            external_reference_count: 0,
            no_trade_counterfactual_count: 0,
            risk_denial_counterfactual_count: 0,
            storage_bytes: 0,
            reason_codes: vec![],
        },
        linked_rows: vec![OutcomeLinkedCommitteeScenarioRow {
            scenario_row,
            outcome_reference: Some(soma_zero::CommitteeOutcomeReference {
                outcome_id: "outcome-1".to_string(),
                decision_id: None,
                symbol: "BTC-KRW".to_string(),
                timestamp_ms: 1,
                horizon_bars: 24,
                triple_barrier_label: soma_zero::CommitteeTripleBarrierLabel::TakeProfit,
                net_return_pct: Some(0.02),
                max_favorable_excursion_pct: None,
                max_adverse_excursion_pct: None,
                cost_bps: 5.0,
                slippage_bps: 2.0,
                source_kind: EvidenceSourceKind::YFinanceResearch,
                no_lookahead_safe: true,
                reason_codes: vec![],
            }),
            baseline_reference: None,
            external_reference: None,
            reason_codes: vec![],
        }],
        unmatched_rows: vec![],
        link_summary: soma_zero::CommitteeOutcomeLinkSummary {
            linker_id: "synthetic".to_string(),
            matched_rows: 1,
            unmatched_rows: 0,
            timestamp_tolerance_ms: 0,
            strict_timestamp_match: true,
            no_lookahead_violations: 0,
            warnings: vec![],
            reason_codes: vec![],
        },
        outcome_linked_count: 1,
        baseline_linked_count: 0,
        external_linked_count: 0,
        no_trade_counterfactual_count: 0,
        risk_denial_counterfactual_count: 0,
        no_lookahead_violations: 0,
        reason_codes: vec![],
    };
    let coverage = build_committee_outcome_coverage_report(
        &CommitteeOutcomeCoverageConfig::default(),
        &[linked_pack.pack.clone()],
        &[linked_pack.clone()],
        &[],
    );
    let matrix = build_committee_performance_evidence_matrix(
        "synthetic",
        &coverage,
        &[linked_pack],
        &[],
        &[],
        false,
    );
    assert_eq!(
        matrix.performance_status,
        CommitteePerformanceStatus::ResearchOnly
    );
}
