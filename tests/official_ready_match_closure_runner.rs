#[path = "support/sprint44_support.rs"]
mod sprint44_support;

use soma_zero::{OfficialReadyMatchClosureRunner, OfficialReadyMatchClosureStatus};

#[test]
fn official_ready_match_closure_runner_applies_safe_repairs_and_records_summaries() {
    let config = sprint44_support::load_closure_config(
        "examples/soma_official_ready_match_close_official_replication.toml",
    );
    let bundle = OfficialReadyMatchClosureRunner::default()
        .run(&config)
        .expect("closure bundle");
    assert!(
        bundle.closure_report.after_official_ready_matches
            >= bundle
                .closure_report
                .before_official_ready_matches
                .unwrap_or_default()
    );
    assert!(
        bundle.closure_report.after_backfilled_rows
            >= bundle
                .closure_report
                .before_backfilled_rows
                .unwrap_or_default()
    );
    assert!(bundle.closure_report.reference_generation_summary.is_some());
    assert!(bundle.closure_report.counterfactual_depth_summary.is_some());
    assert!(matches!(
        bundle.closure_report.closure_status,
        OfficialReadyMatchClosureStatus::OfficialReadyMatchesImproved
            | OfficialReadyMatchClosureStatus::BackfilledRowsImproved
            | OfficialReadyMatchClosureStatus::BottleneckMoved
            | OfficialReadyMatchClosureStatus::NoImprovement
    ));
}

#[test]
fn official_ready_match_closure_runner_preserves_controlled_and_diagnostics_boundaries() {
    let controlled = OfficialReadyMatchClosureRunner::default()
        .run(&sprint44_support::load_closure_config(
            "examples/soma_official_ready_match_close_controlled.toml",
        ))
        .expect("controlled closure");
    assert_eq!(controlled.closure_report.after_official_ready_matches, 0);

    let diagnostics = OfficialReadyMatchClosureRunner::default()
        .run(&sprint44_support::load_closure_config(
            "examples/soma_official_ready_match_close_diagnostics_only.toml",
        ))
        .expect("diagnostics closure");
    assert_eq!(diagnostics.closure_report.after_official_ready_matches, 0);
}
