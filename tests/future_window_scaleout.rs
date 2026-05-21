#[path = "support/sprint47_support.rs"]
mod sprint47_support;

use soma_zero::{FutureWindowScaleOutJobKind, FutureWindowScaleOutPlanner};

#[test]
fn future_window_scaleout_groups_rows_deterministically() {
    let config = sprint47_support::example_future_window("future-window-scaleout");
    let first = FutureWindowScaleOutPlanner::default()
        .plan(&config)
        .expect("first plan");
    let second = FutureWindowScaleOutPlanner::default()
        .plan(&config)
        .expect("second plan");
    assert_eq!(first.grouped_requirements.len(), 2);
    assert!(
        first
            .grouped_requirements
            .iter()
            .all(|group| group.row_count == 1)
    );
    assert!(first.grouped_requirements.iter().all(|group| matches!(
        group.job_kind,
        FutureWindowScaleOutJobKind::SkippedSufficient
            | FutureWindowScaleOutJobKind::LocalExtensionCandidate
            | FutureWindowScaleOutJobKind::LocalReuseOnly
    )));
    assert_eq!(first.to_text(), second.to_text());
    assert_eq!(first.provider_job_groups, 0);
}
