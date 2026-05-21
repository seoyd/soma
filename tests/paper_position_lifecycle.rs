use soma_zero::{PaperPositionStatus, PaperPositionView, build_paper_position_lifecycle_report};

#[test]
fn paper_position_lifecycle_simulates_close_reasons_without_order_ids() {
    let report = build_paper_position_lifecycle_report(
        &serde_json::from_str::<Vec<PaperPositionView>>(include_str!(
            "../examples/sprint56_data/paper_positions.json"
        ))
        .unwrap(),
    );
    assert_eq!(report.target_hit_count, 1);
    assert_eq!(report.stop_hit_count, 1);
    assert_eq!(report.expired_count, 1);
    assert_eq!(report.risk_closed_count, 1);
    assert!(
        report
            .closed_positions
            .iter()
            .all(|position| !position.paper_position_id.is_empty())
    );
    assert!(
        report
            .closed_positions
            .iter()
            .any(|position| position.status == PaperPositionStatus::TargetHit)
    );
}
