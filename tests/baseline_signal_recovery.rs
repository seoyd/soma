mod support;

use soma_zero::{BaselineSignalRecoveryStatus, Sprint88SevenBlockerRecoveryRunner};
use support::sprint69_support as sprint;

#[test]
fn baseline_signal_recovery_preserves_conservative_flow() {
    let config = sprint::sprint88_config_from_example(
        "soma_baseline_signal_recovery.toml",
        "baseline-recovery",
    );
    let report = Sprint88SevenBlockerRecoveryRunner::default()
        .run_baseline_signal_recovery(&config)
        .expect("report");
    assert!(report.feature_regime_flow_covered);
    assert!(report.conservative_no_trade_default_covered);
    assert!(report.poor_data_quality_denial_covered);
    assert!(report.risk_governor_veto_covered);
    assert!(report.deterministic_summary_covered);
    assert_eq!(
        report.recovery_status,
        BaselineSignalRecoveryStatus::BaselineSignalReduced
    );
}
