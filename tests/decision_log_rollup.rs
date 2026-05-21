mod common;
#[path = "support/sprint67_support.rs"]
mod sprint67_support;

use soma_zero::{
    ModelOpsDecisionKind, ModelOpsDecisionRecord, ModelOpsDecisionSource, ModelOpsRollupRunner,
};

#[test]
fn decision_log_rollup_computes_counts_by_kind_and_source() {
    let config = sprint67_support::rollup_config_from_example(
        "soma_decision_log_rollup.toml",
        "decision-log-rollup",
    );
    let report = ModelOpsRollupRunner::default()
        .run_decision_log_rollup(&config)
        .expect("run decision log rollup");
    assert_eq!(report.total_decisions, 21);
    assert!(
        report
            .by_kind_counts
            .get("RequestMorePredictions")
            .copied()
            .unwrap_or(0)
            > 0
    );
    assert!(
        report
            .by_source_counts
            .get("OwnerAction")
            .copied()
            .unwrap_or(0)
            > 0
    );
    assert!(report.repeated_decision_count > 0);
}

#[test]
fn decision_log_rollup_detects_conflicts_and_selects_latest_deterministically() {
    let mut config = sprint67_support::rollup_config_from_example(
        "soma_decision_log_rollup.toml",
        "decision-log-rollup-conflict",
    );
    let mut log: soma_zero::ModelOpsDecisionLog =
        sprint67_support::read_json(&config.model_ops_decision_log_paths[0]);
    log.records.push(ModelOpsDecisionRecord {
        decision_id: "zzz-conflict-retire".to_string(),
        model_id: "ext-model-a".to_string(),
        model_version: "1.1.0".to_string(),
        decision_kind: ModelOpsDecisionKind::RetireModelVersion,
        source: ModelOpsDecisionSource::PolicyRule,
        before_status: None,
        after_status: Some("RetireModelVersion".to_string()),
        reason_codes: Vec::new(),
    });
    config.model_ops_decision_log_paths[0] = sprint67_support::write_support_json(
        "decision-log-rollup-conflict",
        "decision_log.json",
        &log,
    );
    let report = ModelOpsRollupRunner::default()
        .run_decision_log_rollup(&config)
        .expect("run conflict rollup");
    assert!(report.conflict_count > 0);
    let summary = report
        .model_decision_summaries
        .iter()
        .find(|item| item.model_id == "ext-model-a" && item.model_version == "1.1.0")
        .expect("ext-model-a 1.1.0 decision summary");
    assert_eq!(
        summary.latest_decision,
        Some(ModelOpsDecisionKind::RetireModelVersion)
    );
}
