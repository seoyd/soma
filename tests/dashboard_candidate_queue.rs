mod common;

use soma_zero::{CandidateStatus, DashboardSnapshotBuilder, DashboardSourceConfig};

#[test]
fn candidate_queue_shows_conservative_lifecycle_without_orders() {
    let mut config = DashboardSourceConfig::from_toml_path(&common::example_path(
        "soma_dashboard_source_kis_control_tower.toml",
    ))
    .expect("config");
    config.output_root = common::sprint52_output_dir("dashboard-candidates")
        .display()
        .to_string();
    let first = DashboardSnapshotBuilder::default()
        .build(&config)
        .expect("build");
    let second = DashboardSnapshotBuilder::default()
        .build(&config)
        .expect("build");
    let statuses = first
        .candidate_panel
        .candidates
        .iter()
        .map(|candidate| candidate.status)
        .collect::<Vec<_>>();
    assert!(statuses.contains(&CandidateStatus::Candidate));
    assert!(statuses.contains(&CandidateStatus::HumanConfirmRequired));
    assert!(statuses.contains(&CandidateStatus::PaperApproved));
    assert!(statuses.contains(&CandidateStatus::PaperPositionOpen));
    assert!(statuses.contains(&CandidateStatus::RiskBlocked));
    assert!(statuses.contains(&CandidateStatus::Expired));
    let json = serde_json::to_string(&first.candidate_panel).expect("json");
    assert!(!json.contains("order_id"));
    assert_eq!(
        serde_json::to_string(&first.candidate_panel).expect("json"),
        serde_json::to_string(&second.candidate_panel).expect("json")
    );
}
