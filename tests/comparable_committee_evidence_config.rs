mod common;

use soma_zero::{
    ComparableCommitteeEvidenceConfig, ComparableCommitteeEvidenceRow,
    ComparableEvidenceSourceClass, ProviderMarket,
};

fn config(name: &str) -> ComparableCommitteeEvidenceConfig {
    ComparableCommitteeEvidenceConfig {
        comparable_id: name.to_string(),
        output_root: common::output_dir(name).display().to_string(),
        ..ComparableCommitteeEvidenceConfig::default()
    }
}

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
        committee_decision_id: Some(format!("decision-{id}")),
        committee_final_action: "Approve".to_string(),
        chair_decision: Some("Approve".to_string()),
        risk_governor_decision: Some("Approve".to_string()),
        baseline_action: Some("Approve".to_string()),
        external_action: Some("Approve".to_string()),
        no_trade_baseline_action: "NoTrade".to_string(),
        outcome_label: Some("TakeProfit".to_string()),
        net_return_pct: Some(0.04),
        cost_bps: 5.0,
        slippage_bps: 2.0,
        committee_vs_baseline_delta: Some(0.02),
        committee_vs_notrade_delta: Some(0.04),
        risk_denied_value_proxy: Some(-0.01),
        no_trade_value_proxy: Some(0.0),
        outcome_reference_available: true,
        baseline_reference_available: true,
        no_trade_counterfactual_available: true,
        risk_denied_counterfactual_available: true,
        external_reference_available: true,
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
fn comparable_config_rejects_remote_paths_and_unknown_fields() {
    let mut cfg = config("comparable-config-remote");
    cfg.official_replication_report_paths = vec!["https://example.com/report.json".to_string()];
    assert!(cfg.validate().unwrap_err().contains("local"));

    let text = r#"
comparable_id = "bad"
output_root = "target/test"
broker = "forbidden"
"#;
    assert!(ComparableCommitteeEvidenceConfig::from_toml_str(text).is_err());
}

#[test]
fn official_complete_requires_all_references_and_no_lookahead_safety() {
    let cfg = config("comparable-row-complete");
    let full = row("full");
    assert!(full.complete(&cfg));
    assert!(full.official_complete(&cfg));

    let mut missing = row("missing");
    missing.baseline_reference_available = false;
    assert!(!missing.complete(&cfg));
    assert!(!missing.official_complete(&cfg));

    let mut unsafe_row = row("unsafe");
    unsafe_row.no_lookahead_safe = false;
    assert!(!unsafe_row.official_complete(&cfg));
}
