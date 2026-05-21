mod common;
#[path = "support/sprint61_support.rs"]
mod sprint61_support;

use serde_json::json;
use soma_zero::{BoundedKISOfficialEvidenceClosureRunner, OutcomeLinkDepthClosureStatus};

#[test]
fn outcome_link_depth_example_is_healthy() {
    let config = sprint61_support::outcome_config_from_example(
        "soma_outcome_link_depth_close_v2.toml",
        "outcome-healthy",
    );
    let report = BoundedKISOfficialEvidenceClosureRunner::default()
        .run_outcome_link_depth_closure_v2(&config)
        .expect("run outcome depth");
    assert_eq!(
        report.closure_status,
        OutcomeLinkDepthClosureStatus::OutcomeLinkDepthHealthy
    );
}

#[test]
fn outcome_link_depth_blocks_on_no_lookahead() {
    let mut config = sprint61_support::outcome_config_from_example(
        "soma_outcome_link_depth_close_v2.toml",
        "outcome-blocked",
    );
    let path = sprint61_support::write_support_json(
        "outcome-blocked",
        "outcome_link_depth_blocked.json",
        &json!({
            "outcome_links": 5,
            "eligible_rows": 8,
            "take_profit_count": 2,
            "stop_loss_count": 2,
            "time_expired_count": 1,
            "horizons": [4, 8, 16],
            "missing_future_window_count": 0,
            "no_lookahead_blocked_count": 2
        }),
    );
    config.outcome_link_coverage_paths = vec![path];
    config.kis_canonical_csv_paths = Vec::new();
    config.barrier_profile_registry_paths = Vec::new();
    config.complete_row_paths = Vec::new();
    let report = BoundedKISOfficialEvidenceClosureRunner::default()
        .run_outcome_link_depth_closure_v2(&config)
        .expect("run blocked outcome depth");
    assert_eq!(
        report.closure_status,
        OutcomeLinkDepthClosureStatus::NoLookaheadBlocked
    );
}
