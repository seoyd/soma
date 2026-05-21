mod support;

use soma_zero::{DualAgentWorkflowConfig, Sprint104DualAgentPaperLifecycleRunner};

#[test]
fn prompt_compliance_verification_succeeds_by_default() {
    let bundle = Sprint104DualAgentPaperLifecycleRunner::default()
        .run(&DualAgentWorkflowConfig::default())
        .expect("run");
    assert_eq!(
        bundle
            .prompt_compliance_verification_report
            .requirements_missing
            .len(),
        0
    );
    assert!(
        bundle
            .prompt_compliance_verification_report
            .compliance_status
            .starts_with("PromptComplianceVerified")
    );
}

#[test]
fn prompt_compliance_verification_detects_missing_requirements() {
    let mut config = DualAgentWorkflowConfig::default();
    config.require_verification_summary = false;
    let bundle = Sprint104DualAgentPaperLifecycleRunner::default()
        .run(&config)
        .expect("run");
    assert_eq!(
        bundle
            .prompt_compliance_verification_report
            .compliance_status,
        "PromptComplianceFailed"
    );
    assert!(
        bundle
            .prompt_compliance_verification_report
            .requirements_missing
            .contains(&"verification-summary-required".to_string())
    );
}
