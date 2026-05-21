mod support;

use soma_zero::{
    ExternalPredictionFullGateRerunStatus, ExternalPredictionNoRunGateRerunStatus,
    Sprint90ExternalPredictionRecoveryRunner,
};
use support::sprint69_support as sprint;

#[test]
fn external_prediction_no_run_rerun_stays_not_run_by_default() {
    let config = sprint::sprint90_config_from_example(
        "soma_external_prediction_no_run_rerun.toml",
        "external-no-run-rerun",
    );
    let report = Sprint90ExternalPredictionRecoveryRunner::default()
        .run_external_prediction_no_run_rerun(&config)
        .expect("report");
    assert_eq!(
        report.status,
        ExternalPredictionNoRunGateRerunStatus::NotRun
    );
    assert!(!report.started);
    assert!(!report.finished);
}

#[test]
fn external_prediction_full_gate_rerun_stays_distinct_from_no_run() {
    let config = sprint::sprint90_config_from_example(
        "soma_external_prediction_full_gate_rerun.toml",
        "external-full-gate-rerun",
    );
    let report = Sprint90ExternalPredictionRecoveryRunner::default()
        .run_external_prediction_full_gate_rerun(&config)
        .expect("report");
    assert_eq!(report.status, ExternalPredictionFullGateRerunStatus::NotRun);
    assert_eq!(report.command, "cargo test --workspace --quiet");
}
