mod support;

use soma_zero::WorkspaceTimeoutTrackActivationReportV1;
use support::sprint116_support::{read_fixture, run_sprint116};

#[test]
fn workspace_timeout_track_activation_v1_matches_expected() {
    let bundle = run_sprint116(
        "soma_workspace_timeout_track_activation_v1.toml",
        "workspace-timeout-track-activation-v1",
    );
    let expected: WorkspaceTimeoutTrackActivationReportV1 =
        read_fixture("sprint116_data/timeout_track_activation_expected.json");
    assert_eq!(
        bundle.workspace_timeout_track_activation_report_v1,
        expected
    );
    assert!(
        bundle
            .workspace_timeout_track_activation_report_v1
            .diagnostic_track_active
    );
}
