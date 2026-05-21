mod support;

use soma_zero::WorkspaceTimeoutReductionHypothesisReportV1;
use support::sprint118_support::{read_fixture, run_sprint118};

#[test]
fn workspace_timeout_reduction_hypothesis_matches_expected() {
    let bundle = run_sprint118(
        "soma_workspace_timeout_reduction_hypothesis_v1.toml",
        "workspace-timeout-reduction-hypothesis-v1",
    );
    let expected: WorkspaceTimeoutReductionHypothesisReportV1 =
        read_fixture("sprint118_data/timeout_reduction_hypothesis_expected.json");
    assert_eq!(
        bundle.workspace_timeout_reduction_hypothesis_report_v1,
        expected
    );
    assert!(
        bundle
            .workspace_timeout_reduction_hypothesis_report_v1
            .hypotheses
            .iter()
            .any(|value| value == "IntegrationTestBinaryFanout")
    );
}
