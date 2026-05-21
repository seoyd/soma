#[path = "support/sprint69_support.rs"]
mod support;

use soma_zero::{
    RealEvidenceModelOpsRunbookStatus, RealEvidenceModelOpsRunbookStepKind,
    RealEvidencePredictionRefreshRunner,
};

#[test]
fn runbook_includes_copy_only_refresh_steps() {
    let config =
        support::sprint75_config_from_example("soma_real_modelops_runbook.toml", "runbook");
    let runbook = RealEvidencePredictionRefreshRunner::default()
        .run_model_ops_runbook(&config)
        .expect("runbook");
    let kinds = runbook
        .steps
        .iter()
        .map(|step| step.step_kind)
        .collect::<Vec<_>>();
    assert!(kinds.contains(&RealEvidenceModelOpsRunbookStepKind::GeneratePredictionCsvOffline));
    assert!(kinds.contains(&RealEvidenceModelOpsRunbookStepKind::ImportPredictionCsv));
    assert!(kinds.contains(&RealEvidenceModelOpsRunbookStepKind::RunExternalReevaluation));
    assert!(kinds.contains(&RealEvidenceModelOpsRunbookStepKind::RefreshLeaderboard));
    assert!(kinds.contains(&RealEvidenceModelOpsRunbookStepKind::RefreshModelOps));
    assert!(kinds.contains(&RealEvidenceModelOpsRunbookStepKind::RefreshControlTower));
    assert_eq!(
        runbook.runbook_status,
        RealEvidenceModelOpsRunbookStatus::RunbookReady
    );
    let text = serde_json::to_string(&runbook).expect("serialize");
    assert!(!text.to_lowercase().contains("training"));
    assert!(!text.to_lowercase().contains("live inference"));
}

#[test]
fn runbook_blocks_when_prediction_csv_is_missing() {
    let mut config = support::sprint75_config_from_example(
        "soma_real_modelops_runbook.toml",
        "runbook-missing-csv",
    );
    config.new_prediction_csv_paths.clear();
    let runbook = RealEvidencePredictionRefreshRunner::default()
        .run_model_ops_runbook(&config)
        .expect("runbook");
    assert_eq!(
        runbook.runbook_status,
        RealEvidenceModelOpsRunbookStatus::MissingPredictionCsv
    );
    assert!(
        runbook
            .blocked_steps
            .contains(&"ImportPredictionCsv".to_string())
    );
}
