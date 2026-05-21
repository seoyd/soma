mod support;

use std::fs;

use soma_zero::{SafetyCoveragePreservationReportV14, WorkspaceAcceptanceTruthGateStatus};
use std::path::PathBuf;
use support::shared_fixture_harness::{assert_deterministic_text, load_json_fixture};
use support::sprint69_support::example_path;

use support::sprint98_support::run_sprint98;

#[test]
fn sprint98_bundle_and_outputs_are_deterministic_and_conservative() {
    let first = run_sprint98(
        "soma_sprint98_committee_owned_core.toml",
        "sprint98-determinism",
    );
    let second = run_sprint98(
        "soma_sprint98_committee_owned_core.toml",
        "sprint98-determinism",
    );
    assert_eq!(first, second);
    assert_deterministic_text(&first.final_summary, &second.final_summary);
    assert!(
        first
            .final_summary
            .contains("## 29. Safety coverage status")
    );
    assert_eq!(
        first.workspace_acceptance_truth_import.truth_status,
        WorkspaceAcceptanceTruthGateStatus::FullWorkspaceNotRun
    );
    assert!(
        !first
            .workspace_acceptance_truth_import
            .can_claim_full_acceptance
    );
    let expected: SafetyCoveragePreservationReportV14 = load_json_fixture(example_path(
        "sprint98_data/safety_coverage_v14_expected.json",
    ));
    assert_eq!(first.safety_coverage_preservation_report_v14, expected);
    let summary =
        fs::read_to_string(PathBuf::from(&first.storage_report.output_dir).join("summary.txt"))
            .expect("summary");
    assert_deterministic_text(&summary, &first.final_summary);
}
