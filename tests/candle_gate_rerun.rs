mod support;

use soma_zero::{
    CandleExpansionFullGateRerunStatus, CandleExpansionNoRunGateRerunStatus,
    Sprint89CandleRecoveryRunner,
};
use support::sprint69_support as sprint;

#[test]
fn candle_no_run_rerun_stays_not_run_by_default() {
    let config =
        sprint::sprint89_config_from_example("soma_candle_no_run_rerun.toml", "no-run-rerun");
    let report = Sprint89CandleRecoveryRunner::default()
        .run_candle_no_run_rerun(&config)
        .expect("report");
    assert_eq!(report.status, CandleExpansionNoRunGateRerunStatus::NotRun);
    assert!(!report.started);
    assert!(!report.finished);
}

#[test]
fn candle_full_gate_rerun_stays_distinct_from_no_run_rerun() {
    let config =
        sprint::sprint89_config_from_example("soma_candle_full_gate_rerun.toml", "full-gate-rerun");
    let report = Sprint89CandleRecoveryRunner::default()
        .run_candle_full_gate_rerun(&config)
        .expect("report");
    assert_eq!(report.status, CandleExpansionFullGateRerunStatus::NotRun);
    assert_eq!(report.command, "cargo test --workspace --quiet");
}
