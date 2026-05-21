mod common;
#[path = "support/sprint66_support.rs"]
mod sprint66_support;

use soma_zero::{ModelOpsDecisionKind, ModelOpsDecisionSource, ModelReviewClosureRunner};

#[test]
fn decision_log_records_owner_policy_risk_leaderboard_and_coverage_sources() {
    let config = sprint66_support::closure_config_from_example(
        "soma_model_ops_decision_log.toml",
        "model-ops-decision-log",
    );
    let log = ModelReviewClosureRunner::default()
        .run_decision_log(&config)
        .expect("run decision log");
    assert!(
        log.records
            .iter()
            .any(|record| record.source == ModelOpsDecisionSource::OwnerAction)
    );
    assert!(
        log.records
            .iter()
            .any(|record| record.source == ModelOpsDecisionSource::PolicyRule)
    );
    assert!(
        log.records
            .iter()
            .any(|record| record.source == ModelOpsDecisionSource::RiskProfile)
    );
    assert!(
        log.records
            .iter()
            .any(|record| record.source == ModelOpsDecisionSource::LeaderboardChange)
    );
    assert!(
        log.records
            .iter()
            .any(|record| record.source == ModelOpsDecisionSource::CoverageGap)
    );
    assert!(
        log.records
            .iter()
            .any(|record| record.decision_kind == ModelOpsDecisionKind::RetireModelVersion)
    );
    assert!(
        log.records
            .iter()
            .any(|record| record.decision_kind == ModelOpsDecisionKind::RequestMorePredictions)
    );
}

#[test]
fn decision_log_counts_by_kind_are_computed() {
    let config = sprint66_support::closure_config_from_example(
        "soma_model_ops_decision_log.toml",
        "model-ops-decision-log-counts",
    );
    let log = ModelReviewClosureRunner::default()
        .run_decision_log(&config)
        .expect("run decision log counts");
    assert_eq!(log.decision_count, log.records.len());
    assert!(
        log.by_kind_counts
            .get("RetireModelVersion")
            .copied()
            .unwrap_or(0)
            > 0
    );
    assert!(
        log.by_kind_counts
            .get("RequestMorePredictions")
            .copied()
            .unwrap_or(0)
            > 0
    );
}
