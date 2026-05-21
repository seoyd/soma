mod support;

use soma_zero::{BaselineSignalReadinessPrecheckStatus, Sprint95CommitteeCliSafetyRecoveryRunner};
use support::sprint69_support as sprint;

#[test]
fn baseline_signal_precheck_is_ready() {
    let report = Sprint95CommitteeCliSafetyRecoveryRunner::default()
        .run_baseline_signal_readiness_precheck(&sprint::sprint95_config_from_example(
            "soma_baseline_signal_readiness_precheck.toml",
            "baseline-signal-readiness-precheck",
        ))
        .expect("report");
    assert_eq!(
        report.precheck_status,
        BaselineSignalReadinessPrecheckStatus::BaselineSignalPrecheckReady
    );
}
