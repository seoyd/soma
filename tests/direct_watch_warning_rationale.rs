#[path = "support/sprint69_support.rs"]
mod support;

use soma_zero::DirectWatchWarningRationaleStatus;

#[test]
fn direct_watch_warning_rationale_is_explicit() {
    let report = support::run_sprint74_bundle(
        "soma_direct_watch_warning_rationale.toml",
        "direct-watch-warning-rationale",
    )
    .direct_watch_warning_rationale_report;
    assert_eq!(
        report.rationale_status,
        DirectWatchWarningRationaleStatus::RationaleReady
    );
    assert!(
        report
            .monitoring_only_explanation
            .contains("monitoring-only")
    );
    assert!(
        report
            .execution_forbidden_explanation
            .contains("execution stays forbidden")
    );
    assert!(!report.warning_rationales.is_empty());
}
