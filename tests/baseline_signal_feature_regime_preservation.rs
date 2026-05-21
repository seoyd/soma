mod support;

use soma_zero::BaselineSignalFeatureRegimeFlowStatus;
use support::sprint69_support as sprint;

#[test]
fn baseline_signal_feature_regime_flow_remains_preserved() {
    let bundle = sprint::run_sprint96_bundle(
        "soma_sprint96_baseline_signal_recover.toml",
        "baseline-signal-feature-regime-preservation",
    );
    let report = bundle.baseline_signal_feature_regime_flow_preservation_report;
    assert_eq!(
        report.flow_status,
        BaselineSignalFeatureRegimeFlowStatus::FeatureRegimeFlowPreserved
    );
    assert!(report.feature_input_validation_preserved);
    assert!(report.feature_order_preserved);
    assert!(report.regime_classification_preserved);
    assert!(report.signal_score_calculation_preserved);
}
