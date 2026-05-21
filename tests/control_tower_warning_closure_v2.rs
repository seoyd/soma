#[path = "support/sprint69_support.rs"]
mod support;

use soma_zero::{
    ControlTowerWarningClosureV2Status, ControlTowerWarningKind,
    RealEvidencePredictionRefreshRunner,
};

#[test]
fn warning_closure_removes_model_predictions_stale_only() {
    let config = support::sprint75_config_from_example(
        "soma_control_tower_warning_close_v2.toml",
        "warning-close-v2",
    );
    let report = RealEvidencePredictionRefreshRunner::default()
        .run_warning_closure_v2(&config)
        .expect("warning closure");
    assert_eq!(
        report.closure_status,
        ControlTowerWarningClosureV2Status::WarningsReduced
    );
    assert!(
        report
            .closed_warnings
            .contains(&ControlTowerWarningKind::ModelPredictionsStale)
    );
    assert_eq!(
        report.remaining_warnings,
        vec![
            ControlTowerWarningKind::DirectWatchMonitoringOnly,
            ControlTowerWarningKind::RuntimeMambaDeferred,
            ControlTowerWarningKind::LiveTradingForbidden,
            ControlTowerWarningKind::BrokerForbidden,
        ]
    );
}
