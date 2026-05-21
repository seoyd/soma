mod common;
#[path = "support/sprint60_support.rs"]
mod sprint60_support;

use serde_json::json;
use soma_zero::{EvidenceHardeningRunner, OutcomeLinkCoverageStatus};

#[test]
fn healthy_outcome_link_coverage_works() {
    let mut config = sprint60_support::config_from_example(
        "soma_outcome_link_coverage.toml",
        "outcome-link-healthy",
    );
    config.min_outcome_links = 2;
    let report = EvidenceHardeningRunner::default()
        .run(&config)
        .expect("run outcome link healthy")
        .outcome_link_coverage_report;
    assert_eq!(report.coverage_status, OutcomeLinkCoverageStatus::Healthy);
}

#[test]
fn missing_future_windows_and_no_lookahead_block_are_detected() {
    let mut config = sprint60_support::config_from_example(
        "soma_outcome_link_coverage.toml",
        "outcome-link-blocked",
    );
    config.supporting_artifact_paths = vec![sprint60_support::write_support_json(
        "outcome-link-blocked-support",
        "support.json",
        &json!({
            "future_window_count": 0,
            "no_lookahead_safe": false,
            "outcome_links": 8
        }),
    )];
    let report = EvidenceHardeningRunner::default()
        .run(&config)
        .expect("run outcome link blocked")
        .outcome_link_coverage_report;
    assert_eq!(
        report.coverage_status,
        OutcomeLinkCoverageStatus::BlockedByNoLookahead
    );
}
