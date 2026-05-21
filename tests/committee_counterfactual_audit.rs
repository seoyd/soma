mod common;
#[path = "support/official_committee_support.rs"]
mod official_committee_support;

use soma_zero::{
    CommitteeCounterfactualAuditConfig, CommitteeCounterfactualAuditRunner,
    CommitteeCounterfactualAuditStatus,
};

#[test]
fn counterfactual_audit_computes_counts_and_is_deterministic() {
    let pack_config =
        official_committee_support::controlled_pack_config("counterfactual-audit", false);
    let pack_config_path =
        official_committee_support::write_pack_config("counterfactual-audit", &pack_config);
    let candle_path = official_committee_support::write_candle_series(
        "counterfactual-audit",
        "AAPL",
        1_700_000_000_000,
        -1.0,
    );
    let config = CommitteeCounterfactualAuditConfig {
        audit_id: "counterfactual-audit".to_string(),
        scenario_pack_paths: vec![pack_config_path.display().to_string()],
        candle_series_paths: vec![candle_path.display().to_string()],
        output_root: common::output_dir("counterfactual-audit-out")
            .display()
            .to_string(),
        ..CommitteeCounterfactualAuditConfig::default()
    };
    let first = CommitteeCounterfactualAuditRunner::default()
        .run(&config)
        .expect("first");
    let second = CommitteeCounterfactualAuditRunner::default()
        .run(&config)
        .expect("second");
    assert_eq!(first, second);
    assert_eq!(
        first.audit_status,
        CommitteeCounterfactualAuditStatus::HealthyCounterfactuals
    );
    assert!(first.built_count >= 2);
    assert!(first.no_trade_count >= 1);
    assert!(first.risk_denied_count >= 1);
    assert!(first.avoided_loss_total >= 0.0);
}

#[test]
fn counterfactual_audit_reports_need_more_candle_data() {
    let pack_config =
        official_committee_support::controlled_pack_config("counterfactual-audit-missing", false);
    let pack_config_path =
        official_committee_support::write_pack_config("counterfactual-audit-missing", &pack_config);
    let report = CommitteeCounterfactualAuditRunner::default()
        .run(&CommitteeCounterfactualAuditConfig {
            audit_id: "counterfactual-audit-missing".to_string(),
            scenario_pack_paths: vec![pack_config_path.display().to_string()],
            output_root: common::output_dir("counterfactual-audit-missing-out")
                .display()
                .to_string(),
            ..CommitteeCounterfactualAuditConfig::default()
        })
        .expect("report");
    assert_eq!(
        report.audit_status,
        CommitteeCounterfactualAuditStatus::NeedMoreCandleData
    );
    assert_eq!(report.built_count, 0);
}
