mod support;

use soma_zero::{
    WorkspaceTimeoutObservationBacklogBurnDownReportV1,
    WorkspaceTimeoutObservationBacklogImportReportV1,
};
use support::sprint116_support::{read_fixture, run_sprint116};

#[test]
fn workspace_timeout_observation_backlog_reports_match_expected() {
    let bundle = run_sprint116(
        "soma_workspace_timeout_observation_backlog_burndown_v1.toml",
        "workspace-timeout-observation-backlog-burndown-v1",
    );
    let expected_import: WorkspaceTimeoutObservationBacklogImportReportV1 =
        read_fixture("sprint116_data/observation_backlog_import_expected.json");
    let expected_burndown: WorkspaceTimeoutObservationBacklogBurnDownReportV1 =
        read_fixture("sprint116_data/observation_backlog_burndown_expected.json");
    assert_eq!(
        bundle.workspace_timeout_observation_backlog_import_report_v1,
        expected_import
    );
    assert_eq!(
        bundle.workspace_timeout_observation_backlog_burn_down_report_v1,
        expected_burndown
    );
    assert_eq!(
        bundle
            .workspace_timeout_observation_backlog_import_report_v1
            .no_run_items,
        1
    );
    assert_eq!(
        bundle
            .workspace_timeout_observation_backlog_import_report_v1
            .full_items,
        1
    );
    assert_eq!(
        bundle
            .workspace_timeout_observation_backlog_import_report_v1
            .cargo_json_items,
        1
    );
    assert_eq!(
        bundle
            .workspace_timeout_observation_backlog_import_report_v1
            .cleanup_items,
        1
    );
}
