mod support;

use soma_zero::BaselineSignalNoTradeDefaultStatus;
use support::sprint69_support as sprint;

#[test]
fn baseline_signal_notrade_default_stays_absolute() {
    let bundle = sprint::run_sprint96_bundle(
        "soma_sprint96_baseline_signal_recover.toml",
        "baseline-signal-notrade-default-preservation",
    );
    let report = bundle.baseline_signal_no_trade_default_preservation_report;
    assert_eq!(
        report.no_trade_status,
        BaselineSignalNoTradeDefaultStatus::NoTradeDefaultPreserved
    );
    assert!(report.default_action_no_trade_preserved);
    assert!(report.uncertain_signal_goes_no_trade);
    assert!(report.missing_feature_goes_no_trade);
    assert!(report.poor_data_goes_no_trade);
}
