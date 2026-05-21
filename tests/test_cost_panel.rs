mod support;

use soma_zero::{
    CliSmokeExecutionPolicyStatus, SharedFixtureHarnessReportStatus,
    TestBinaryConsolidationReportStatus, TestRuntimeImprovementStatus,
    WorkspaceAcceptanceFinalGateV2Status,
};
use support::sprint69_support as sprint;

#[test]
fn sprint84_test_cost_panel_is_read_only_and_runtime_deferred() {
    let bundle = sprint::run_sprint84_bundle(
        "soma_control_tower_test_cost.toml",
        "sprint84-test-cost-panel",
    );
    let panel = bundle.control_tower_test_cost_panel;
    assert_eq!(
        panel.test_binary_consolidation_status,
        TestBinaryConsolidationReportStatus::TestBinariesReduced
    );
    assert_eq!(
        panel.shared_fixture_harness_status,
        SharedFixtureHarnessReportStatus::HarnessReady
    );
    assert_eq!(
        panel.smoke_policy_status,
        CliSmokeExecutionPolicyStatus::SmokePolicyReady
    );
    assert_eq!(
        panel.runtime_before_after_status,
        TestRuntimeImprovementStatus::SampleBackedOnly
    );
    assert_eq!(
        panel.workspace_final_gate_status,
        WorkspaceAcceptanceFinalGateV2Status::FullWorkspaceStillBlocked
    );
    assert!(panel.safety_coverage_status.contains("preserved"));
    assert!(panel.runtime_deferred_status.contains("no train button"));
    assert!(panel.runtime_deferred_status.contains("no runtime button"));
    assert!(panel.runtime_deferred_status.contains("no live button"));
    assert!(
        panel
            .runtime_deferred_status
            .contains("no order/account controls")
    );
    assert!(
        panel
            .runtime_deferred_status
            .contains("no browser execution")
    );
}
