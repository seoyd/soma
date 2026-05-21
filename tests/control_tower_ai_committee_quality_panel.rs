mod support;

use std::fs;
use std::path::PathBuf;

use soma_zero::{
    ControlTowerAiCommitteeQualityPanel, SafetyCoveragePreservationReportV15Status,
    WorkspaceAcceptanceTruthGateStatus,
};
use support::sprint99_support::run_sprint99;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("sprint99_data")
        .join(name)
}

#[test]
fn control_tower_ai_committee_quality_panel_matches_expected_fixture() {
    let bundle = run_sprint99(
        "soma_control_tower_ai_committee_quality.toml",
        "control-tower-ai-committee-quality",
    );
    let expected: ControlTowerAiCommitteeQualityPanel = serde_json::from_str(
        &fs::read_to_string(fixture_path("control_tower_quality_panel_expected.json"))
            .expect("read fixture"),
    )
    .expect("parse fixture");
    assert_eq!(bundle.control_tower_ai_committee_quality_panel, expected);
    assert_eq!(
        bundle
            .control_tower_ai_committee_quality_panel
            .workspace_acceptance_truth_status,
        WorkspaceAcceptanceTruthGateStatus::FullWorkspaceNotRun
    );
    assert_eq!(
        bundle
            .control_tower_ai_committee_quality_panel
            .safety_coverage_status,
        SafetyCoveragePreservationReportV15Status::SafetyCoveragePreservedWithWarnings
    );
    assert!(
        bundle
            .control_tower_ai_committee_quality_panel
            .runtime_deferred_summary
            .contains("runtime deferred")
    );
}
