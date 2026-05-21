mod support;

use soma_zero::{DualAgentWorkflowConfig, Sprint104DualAgentPaperLifecycleRunner};

#[test]
fn safety_invariants_are_verified_by_default() {
    let bundle = Sprint104DualAgentPaperLifecycleRunner::default()
        .run(&DualAgentWorkflowConfig::default())
        .expect("run");
    let report = bundle.safety_invariant_verification_report;
    assert_eq!(report.safety_status, "SafetyInvariantsVerified");
    assert!(report.no_live_trading);
    assert!(report.no_broker_order_account);
    assert!(report.no_runtime_llm_live_decision);
    assert_eq!(
        bundle.safety_coverage_preservation_report_v20.safety_status,
        "SafetyCoveragePreservedV20"
    );
}

#[test]
fn safety_invariant_violation_is_detected_when_guards_are_disabled() {
    let mut config = DualAgentWorkflowConfig::default();
    config.preserve_safety_guards = false;
    let bundle = Sprint104DualAgentPaperLifecycleRunner::default()
        .run(&config)
        .expect("run");
    assert_eq!(
        bundle.safety_invariant_verification_report.safety_status,
        "SafetyInvariantViolation"
    );
    assert_eq!(
        bundle.safety_coverage_preservation_report_v20.safety_status,
        "SafetyCoverageRegressionV20"
    );
    assert!(
        !bundle
            .safety_coverage_preservation_report_v20
            .live_trading_guard_present
    );
}
