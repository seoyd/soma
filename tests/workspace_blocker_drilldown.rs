mod support;

use soma_zero::{
    RemainingWorkspaceBlockerDrilldownReportStatus, Sprint85WorkspaceGateRecoveryRunner,
};
use support::sprint69_support as sprint;

#[test]
fn workspace_blocker_drilldown_reports_named_remaining_blockers() {
    let config = sprint::sprint85_config_from_example(
        "soma_workspace_blocker_drilldown.toml",
        "workspace-blocker-drilldown-test",
    );
    let report = Sprint85WorkspaceGateRecoveryRunner::default()
        .run_workspace_blocker_drilldown(&config)
        .expect("drilldown");
    assert_eq!(
        report.report_status,
        RemainingWorkspaceBlockerDrilldownReportStatus::BlockersExplained
    );
    assert!(
        report
            .remaining_blockers
            .iter()
            .any(|name| name.ends_with("artifact_render_cache_plan.rs"))
    );
}
