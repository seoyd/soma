mod support;

use std::fs;
use std::path::PathBuf;

use soma_zero::{
    CommitteePaperReadinessGateStatus, ControlTowerAiCommitteeClosurePanel,
    SafetyCoveragePreservationReportV16Status, WorkspaceAcceptanceTruthGateStatus,
};
use support::sprint100_support::run_sprint100;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("sprint100_data")
        .join(name)
}

#[test]
fn control_tower_ai_committee_closure_panel_matches_expected_fixture() {
    let bundle = run_sprint100(
        "soma_control_tower_ai_committee_closure.toml",
        "control-tower-ai-committee-closure",
    );
    let expected: ControlTowerAiCommitteeClosurePanel = serde_json::from_str(
        &fs::read_to_string(fixture_path("control_tower_closure_panel_expected.json"))
            .expect("read fixture"),
    )
    .expect("parse fixture");
    assert_eq!(bundle.control_tower_ai_committee_closure_panel, expected);
    assert_eq!(
        bundle
            .control_tower_ai_committee_closure_panel
            .workspace_acceptance_truth_status,
        WorkspaceAcceptanceTruthGateStatus::FullWorkspaceNotRun
    );
    assert_eq!(
        bundle
            .control_tower_ai_committee_closure_panel
            .safety_coverage_status,
        SafetyCoveragePreservationReportV16Status::SafetyCoveragePreservedWithWarnings
    );
    assert_eq!(
        bundle
            .control_tower_ai_committee_closure_panel
            .paper_readiness_gate_status,
        CommitteePaperReadinessGateStatus::PaperCommitteeReadyWithWarnings
    );
}
