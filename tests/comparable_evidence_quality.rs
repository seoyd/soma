mod common;

use soma_zero::{
    ComparableCommitteeEvidenceBundle, ComparableCommitteeEvidenceConfig,
    ComparableCommitteeEvidenceRow, ComparableEvidenceQualityStatus, ComparableEvidenceSourceClass,
    ProviderMarket, build_comparable_evidence_quality_report,
};

fn row(
    id: &str,
    source_class: ComparableEvidenceSourceClass,
    diagnostic_only: bool,
) -> ComparableCommitteeEvidenceRow {
    ComparableCommitteeEvidenceRow {
        row_id: id.to_string(),
        symbol: "AAPL".to_string(),
        market: ProviderMarket::USEquity,
        timeframe: "1d".to_string(),
        horizon_bars: 24,
        timestamp_ms: 1_700_000_000_000,
        source_kind: format!("{source_class:?}"),
        source_class,
        scenario_row_id: Some(id.to_string()),
        committee_decision_id: None,
        committee_final_action: "Approve".to_string(),
        chair_decision: None,
        risk_governor_decision: None,
        baseline_action: Some("Approve".to_string()),
        external_action: None,
        no_trade_baseline_action: "NoTrade".to_string(),
        outcome_label: Some("TakeProfit".to_string()),
        net_return_pct: Some(0.03),
        cost_bps: 5.0,
        slippage_bps: 2.0,
        committee_vs_baseline_delta: Some(0.01),
        committee_vs_notrade_delta: Some(0.03),
        risk_denied_value_proxy: Some(-0.01),
        no_trade_value_proxy: Some(0.0),
        outcome_reference_available: true,
        baseline_reference_available: true,
        no_trade_counterfactual_available: true,
        risk_denied_counterfactual_available: true,
        external_reference_available: false,
        row_level: true,
        summary_derived: false,
        no_lookahead_safe: true,
        official_readiness_eligible: source_class
            == ComparableEvidenceSourceClass::OfficialNonCrypto,
        diagnostic_only,
        candle_coverage_available: false,
        matched_candle_series_id: None,
        candle_match_status: None,
        candle_official_ready_match: false,
        candle_benchmark_ready_match: false,
        candle_diagnostic_only: false,
        reason_codes: Vec::new(),
    }
}

#[test]
fn quality_reports_controlled_only_and_healthy_official_cases() {
    let mut config = ComparableCommitteeEvidenceConfig::default();
    config.comparable_id = "quality".to_string();
    config.output_root = common::output_dir("comparable-quality")
        .display()
        .to_string();

    let controlled_bundle = ComparableCommitteeEvidenceBundle::from_rows(
        &config,
        vec![row(
            "controlled",
            ComparableEvidenceSourceClass::ControlledDiagnostic,
            true,
        )],
    );
    let controlled = build_comparable_evidence_quality_report(&config, &controlled_bundle);
    assert_eq!(
        controlled.quality_status,
        ComparableEvidenceQualityStatus::ControlledOnly
    );

    let official_bundle = ComparableCommitteeEvidenceBundle::from_rows(
        &config,
        vec![row(
            "official",
            ComparableEvidenceSourceClass::OfficialNonCrypto,
            false,
        )],
    );
    let official = build_comparable_evidence_quality_report(&config, &official_bundle);
    assert_eq!(
        official.quality_status,
        ComparableEvidenceQualityStatus::HealthyComparableEvidence
    );
}
