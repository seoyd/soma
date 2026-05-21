mod common;

use soma_zero::{
    ComparableCommitteeEvidenceBundle, ComparableCommitteeEvidenceConfig,
    ComparableCommitteeEvidenceRow, ComparableEvidenceSourceClass,
    CounterfactualDepthClosureConfig, CounterfactualDepthClosureRunner, ProviderMarket,
};

fn row(id: &str) -> ComparableCommitteeEvidenceRow {
    ComparableCommitteeEvidenceRow {
        row_id: id.to_string(),
        symbol: "AAPL".to_string(),
        market: ProviderMarket::USEquity,
        timeframe: "1d".to_string(),
        horizon_bars: 24,
        timestamp_ms: 1_700_000_000_000,
        source_kind: "OfficialApiCollected".to_string(),
        source_class: ComparableEvidenceSourceClass::OfficialNonCrypto,
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
        official_readiness_eligible: true,
        diagnostic_only: false,
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
fn closure_runner_writes_expected_bundle_files() {
    let comparable_config = ComparableCommitteeEvidenceConfig {
        comparable_id: "closure-runner-base".to_string(),
        output_root: common::output_dir("closure-runner-config")
            .display()
            .to_string(),
        ..ComparableCommitteeEvidenceConfig::default()
    };
    let bundle =
        ComparableCommitteeEvidenceBundle::from_rows(&comparable_config, vec![row("runner")]);
    let bundle_dir = common::output_dir("closure-runner-bundle");
    let bundle_path = bundle.write_to_dir(&bundle_dir).expect("write bundle");

    let config = CounterfactualDepthClosureConfig {
        closure_id: "closure-runner".to_string(),
        comparable_evidence_bundle_path: Some(bundle_path.display().to_string()),
        output_root: common::output_dir("closure-runner-out")
            .display()
            .to_string(),
        max_build_attempts: 2,
        ..CounterfactualDepthClosureConfig::default()
    };

    let bundle = CounterfactualDepthClosureRunner::default()
        .run_bundle(&config)
        .expect("run bundle");
    let out = config.output_dir();
    assert_eq!(bundle.closure_id, "closure-runner");
    for name in [
        "comparable_evidence_bundle.txt",
        "comparable_evidence_quality.txt",
        "counterfactual_depth_plan.txt",
        "counterfactual_depth_closure.txt",
        "scenario_materialization_weak_closure.txt",
        "counterfactual_depth_summary.txt",
        "counterfactual_depth_closure_bundle.json",
    ] {
        assert!(out.join(name).exists(), "missing {name}");
    }
}
