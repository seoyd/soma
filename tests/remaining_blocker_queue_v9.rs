mod support;

use soma_zero::{
    RealWorkspaceTimeoutAttributionConfig, RemainingBlockerQueueV9Status,
    Sprint93TimeoutAttributionRunner,
};
use support::sprint69_support as sprint;

fn config(name: &str) -> RealWorkspaceTimeoutAttributionConfig {
    sprint::sprint93_config_from_example("soma_remaining_blocker_queue_v9.toml", name)
}

#[test]
fn remaining_queue_advances_to_dashboard_renderer_only_after_proof() {
    let report = Sprint93TimeoutAttributionRunner::default()
        .run_remaining_blocker_queue_v9(&config("remaining-blocker-queue-v9"))
        .expect("report");
    assert_eq!(
        report.queue_status,
        RemainingBlockerQueueV9Status::QueueAdvancedToDashboardRenderer
    );
    assert_eq!(report.primary_next_family, "DashboardRenderer");
    assert!(report.dashboard_renderer_entry_allowed);
}
