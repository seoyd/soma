mod support;

use soma_zero::WorkspaceTimeoutTrackSplitReportV1;
use support::sprint115_support::{read_fixture, run_sprint115};

#[test]
fn workspace_timeout_track_split_v1_matches_expected() {
    let bundle = run_sprint115(
        "soma_workspace_timeout_track_split_v1.toml",
        "workspace-timeout-track-split-v1",
    );
    let expected: WorkspaceTimeoutTrackSplitReportV1 =
        read_fixture("sprint115_data/workspace_timeout_track_split_expected.json");
    assert_eq!(bundle.workspace_timeout_track_split_report_v1, expected);
    assert!(
        bundle
            .workspace_timeout_track_split_report_v1
            .separation_complete
    );
}
