mod support;

use serde_json::to_value;
use soma_zero::{DualAgentWorkflowConfig, Sprint104DualAgentPaperLifecycleRunner};
use support::sprint104_support::{read_fixture, run_sprint104};

#[test]
fn dual_agent_workflow_config_defaults_and_policy_are_ready() {
    let config = DualAgentWorkflowConfig::default();
    assert_eq!(config.implementation_agent_name, "codex-5.4");
    assert_eq!(config.verification_agent_name, "gpt-5.5");
    assert!(config.require_implementation_summary);
    assert!(config.require_verification_summary);
    assert!(config.preserve_runtime_deferred);
    assert!(config.preserve_safety_guards);
    assert!(config.validate().is_ok());

    let bundle = run_sprint104(
        "soma_sprint104_dual_agent_paper_lifecycle.toml",
        "dual_agent_workflow_policy",
    );
    let actual = to_value(&bundle.dual_agent_workflow_policy).expect("actual");
    let expected: serde_json::Value =
        read_fixture("sprint104_data/dual_agent_workflow_expected.json");
    assert_eq!(actual, expected);
    assert!(bundle.dual_agent_workflow_policy.handoff_required);
    assert!(bundle.dual_agent_workflow_policy.findings_required);
    assert!(
        bundle
            .dual_agent_workflow_policy
            .final_verification_required
    );
    assert!(
        !bundle
            .workspace_truth_verification_report
            .can_claim_full_acceptance
    );
    assert!(!bundle.workspace_acceptance_attempt_v20.no_run_started);
    assert!(!bundle.workspace_acceptance_attempt_v20.no_run_finished);
    assert!(!bundle.workspace_acceptance_attempt_v20.full_started);
    assert!(!bundle.workspace_acceptance_attempt_v20.full_finished);
}

#[test]
fn dual_agent_workflow_rejects_remote_paths() {
    let mut config = DualAgentWorkflowConfig::default();
    config.output_root = "https://example.com/out".to_string();
    assert!(config.validate().is_err());
}

#[test]
fn dual_agent_workflow_disabled_requirements_are_unsafe() {
    let mut config = DualAgentWorkflowConfig::default();
    config.require_verification_summary = false;
    let bundle = Sprint104DualAgentPaperLifecycleRunner::default()
        .run(&config)
        .expect("run");
    assert_eq!(
        bundle.dual_agent_workflow_policy.policy_status,
        "DualAgentWorkflowUnsafe"
    );
}
