mod common;

use std::path::Path;

use soma_zero::{DashboardOpenStatus, prepare_dashboard_open};

#[test]
fn dashboard_open_prepares_local_file_path_only() {
    let output_root = common::sprint54_output_dir("dashboard-open");
    let html_path = output_root.join("dashboard_v1.html");
    std::fs::write(&html_path, "<html></html>").expect("html");
    let report = prepare_dashboard_open(&output_root, &html_path, false).expect("report");
    assert_eq!(report.status, DashboardOpenStatus::PathPrinted);
    assert!(!report.launched);
    assert!(report.local_open_command.is_some());
}

#[test]
fn dashboard_open_rejects_remote_and_outside_paths() {
    let output_root = common::sprint54_output_dir("dashboard-open-reject");
    let remote = prepare_dashboard_open(
        &output_root,
        Path::new("https://example.com/dashboard.html"),
        false,
    )
    .expect("report");
    assert_eq!(remote.status, DashboardOpenStatus::RejectedRemotePath);

    let outside =
        prepare_dashboard_open(&output_root, Path::new("README.md"), false).expect("report");
    assert_eq!(
        outside.status,
        DashboardOpenStatus::RejectedOutsideOutputRoot
    );
}
