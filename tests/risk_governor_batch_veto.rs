mod support;

use serde_json::to_value;
use soma_zero::{DualAgentWorkflowConfig, Sprint104DualAgentPaperLifecycleRunner};
use support::sprint104_support::{
    read_fixture, run_default_sprint103_fixture, write_sprint103_bundle,
};

#[test]
fn risk_governor_batch_veto_preserves_final_veto_and_zero_live_execution() {
    let bundle = Sprint104DualAgentPaperLifecycleRunner::default()
        .run(&DualAgentWorkflowConfig::default())
        .expect("run");
    let actual = to_value(&bundle.risk_governor_batch_veto_report).expect("actual");
    let expected: serde_json::Value =
        read_fixture("sprint104_data/risk_governor_batch_veto_expected.json");
    assert_eq!(actual, expected);
    assert_eq!(
        bundle
            .risk_governor_batch_veto_report
            .broker_execution_allowed_count,
        0
    );
    assert_eq!(
        bundle
            .risk_governor_batch_veto_report
            .live_execution_allowed_count,
        0
    );
}

#[test]
fn risk_governor_batch_veto_detects_bypass_attempt() {
    let mut sprint103 = run_default_sprint103_fixture();
    sprint103
        .risk_governor_notrade_reason_audit
        .bypass_attempt_count = 1;
    let bundle_path = write_sprint103_bundle(
        "risk_governor_batch_veto",
        "sprint103_bundle.json",
        &sprint103,
    );
    let mut config = DualAgentWorkflowConfig::default();
    config.sprint103_bundle_paths = Some(vec![bundle_path]);
    let bundle = Sprint104DualAgentPaperLifecycleRunner::default()
        .run(&config)
        .expect("run");
    assert_eq!(
        bundle.risk_governor_batch_veto_report.veto_status,
        "RiskBypassDetected"
    );
}
