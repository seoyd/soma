#[path = "support/sprint45_support.rs"]
mod sprint45_support;
#[path = "support/sprint46_support.rs"]
mod sprint46_support;
mod support;

use soma_zero::{
    CompleteRowClosureV2Runner, RuntimeMode, RuntimeStage, RuntimeState, RuntimeStateReport,
};
use support::{shared_fixture_harness as harness, sprint69_support as sprint};

#[test]
fn complete_row_closure_v2_is_deterministic_for_same_fixture() {
    let config = sprint46_support::closure_v2_config("workspace-determinism-complete-row");
    let first = CompleteRowClosureV2Runner::default()
        .run(&config)
        .expect("first");
    let second = CompleteRowClosureV2Runner::default()
        .run(&config)
        .expect("second");
    assert_eq!(first.final_summary, second.final_summary);
    assert_eq!(
        first.complete_row_closure_v2_report,
        second.complete_row_closure_v2_report
    );
    assert_eq!(
        first.outcome_linkage_v3_report,
        second.outcome_linkage_v3_report
    );
}

#[test]
fn runtime_transition_history_is_deterministic() {
    let mut left = RuntimeState::new(RuntimeMode::Research);
    let mut right = RuntimeState::new(RuntimeMode::Research);
    for stage in [
        RuntimeStage::LoadConfig,
        RuntimeStage::ValidateConfig,
        RuntimeStage::ChairDecision,
        RuntimeStage::RiskEvaluation,
        RuntimeStage::PaperExecution,
        RuntimeStage::OutcomeEvaluation,
    ] {
        left.transition_to(stage, true).expect("left transition");
        right.transition_to(stage, true).expect("right transition");
    }

    assert_eq!(
        RuntimeStateReport::from_state(left).to_text(),
        RuntimeStateReport::from_state(right).to_text()
    );
}

#[test]
fn sprint84_and_sprint85_bundles_are_deterministic() {
    let sprint84_left = sprint::run_sprint84_bundle(
        "soma_sprint84_test_cost_reduce.toml",
        "workspace-determinism-sprint84-left",
    );
    let sprint84_right = sprint::run_sprint84_bundle(
        "soma_sprint84_test_cost_reduce.toml",
        "workspace-determinism-sprint84-right",
    );
    assert_eq!(
        sprint84_left.test_binary_consolidation_report,
        sprint84_right.test_binary_consolidation_report
    );
    assert_eq!(
        sprint84_left.control_tower_test_cost_panel,
        sprint84_right.control_tower_test_cost_panel
    );

    let sprint85_left = sprint::run_sprint85_bundle(
        "soma_sprint85_workspace_gate_recovery.toml",
        "workspace-determinism-sprint85-left",
    );
    let sprint85_right = sprint::run_sprint85_bundle(
        "soma_sprint85_workspace_gate_recovery.toml",
        "workspace-determinism-sprint85-right",
    );
    assert_eq!(
        sprint85_left.workspace_wide_test_surface_audit_report,
        sprint85_right.workspace_wide_test_surface_audit_report
    );
    assert_eq!(
        sprint85_left.control_tower_workspace_gate_panel_v2,
        sprint85_right.control_tower_workspace_gate_panel_v2
    );
    harness::assert_deterministic_text(&sprint85_left.final_summary, &sprint85_right.final_summary);
}
