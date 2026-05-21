mod support;

use soma_zero::{BaselineSignalFullGateRerunStatus, BaselineSignalNoRunGateRerunStatus};
use support::sprint69_support as sprint;

#[test]
fn baseline_signal_gate_reruns_remain_not_run_when_flags_are_disabled() {
    let bundle = sprint::run_sprint96_bundle(
        "soma_sprint96_baseline_signal_recover.toml",
        "baseline-signal-gate-rerun",
    );
    assert_eq!(
        bundle.baseline_signal_no_run_gate_rerun_report.status,
        BaselineSignalNoRunGateRerunStatus::NotRun
    );
    assert_eq!(
        bundle.baseline_signal_full_gate_rerun_report.status,
        BaselineSignalFullGateRerunStatus::NotRun
    );
}
