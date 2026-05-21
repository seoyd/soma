mod common;

use std::path::Path;

use soma_zero::{ControlTowerRefreshConfig, ControlTowerRefreshRunner, ControlTowerRefreshStatus};

#[test]
fn control_tower_refresh_overlays_depth_and_loop_reports() {
    let config_path = Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/examples/soma_control_tower_refresh_after_kis_depth.toml"
    ));
    let config = ControlTowerRefreshConfig::from_toml_path(config_path).unwrap();
    let output = ControlTowerRefreshRunner::default()
        .run(&config, Some(config_path), None, None)
        .unwrap();
    assert!(matches!(
        output.report.refresh_status,
        ControlTowerRefreshStatus::Refreshed | ControlTowerRefreshStatus::RefreshedWithWarnings
    ));
    assert!(output.report.secret_redaction_passed);
    assert!(!output.report.unsafe_control_detected);
    assert_eq!(
        output
            .state
            .kis_monitor_panel
            .latest_depth_run_id
            .as_deref(),
        Some("sprint57-depth-after")
    );
    assert_eq!(
        output
            .state
            .operational_loop_panel
            .last_loop_run_id
            .as_deref(),
        Some("sprint57-trinity-loop-refresh")
    );
    assert_eq!(output.state.evidence_panel.official_rows_before, Some(12));
    assert_eq!(output.state.evidence_panel.official_rows_after, Some(28));
    assert_eq!(
        output.state.next_action_panel.primary_next_action.action_id,
        "step-01-kis-auth-check"
    );
    assert_eq!(output.state.next_action_panel.recommended_actions.len(), 7);
    assert_eq!(output.state.next_action_panel.blocked_actions.len(), 4);
}
