mod support;

use support::sprint102_support::run_sprint102;

#[test]
fn control_tower_panel_and_workspace_truth_are_read_only() {
    let bundle = run_sprint102("soma_control_tower_paper_rotation.toml", "sprint102-panel");
    let panel = &bundle.control_tower_paper_rotation_panel;
    for summary in [
        &panel.scenario_summary,
        &panel.member_selection_summary,
        &panel.lower_confidence_evidence_summary,
        &panel.proposal_run_summary,
        &panel.debate_summary,
        &panel.chairman_synthesis_summary,
        &panel.risk_governor_handoff_summary,
        &panel.paper_decision_trace_summary,
        &panel.roster_usage_summary,
        &panel.workspace_truth_summary,
    ] {
        assert!(!summary.is_empty());
    }
    let text = serde_json::to_string(panel).expect("panel json");
    for forbidden in [
        "train_button",
        "runtime_button",
        "live_button",
        "order_control",
        "account_control",
        "activate_all_18_live",
    ] {
        assert!(
            !text.contains(forbidden),
            "unexpected panel control {forbidden}"
        );
    }
    assert!(
        !bundle
            .workspace_acceptance_truth_closure_plan_v3
            .can_claim_full_acceptance
    );
    assert!(
        !bundle
            .workspace_acceptance_attempt_v18
            .can_claim_full_acceptance
    );
    assert!(
        bundle
            .safety_coverage_preservation_report_v18
            .mamba_runtime_guard_present
    );
    assert!(
        bundle
            .safety_coverage_preservation_report_v18
            .gated_runtime_guard_present
    );
    assert!(
        bundle
            .safety_coverage_preservation_report_v18
            .model_training_guard_present
    );
    assert!(
        bundle
            .safety_coverage_preservation_report_v18
            .python_training_dependency_guard_present
    );
    assert!(
        bundle
            .safety_coverage_preservation_report_v18
            .eighteen_live_activation_forbidden
    );
}
