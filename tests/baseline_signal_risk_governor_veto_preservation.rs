mod support;

use soma_zero::BaselineSignalRiskGovernorVetoStatus;
use support::sprint69_support as sprint;

#[test]
fn baseline_signal_risk_governor_veto_stays_absolute() {
    let bundle = sprint::run_sprint96_bundle(
        "soma_sprint96_baseline_signal_recover.toml",
        "baseline-signal-risk-governor-veto-preservation",
    );
    let report = bundle.baseline_signal_risk_governor_veto_preservation_report;
    assert_eq!(
        report.veto_status,
        BaselineSignalRiskGovernorVetoStatus::RiskGovernorVetoPreserved
    );
    assert!(report.risk_governor_hard_veto_preserved);
    assert!(report.risk_denied_overrides_baseline_signal);
    assert!(report.emergency_stop_overrides_baseline_signal);
    assert!(report.cooldown_overrides_baseline_signal);
}
