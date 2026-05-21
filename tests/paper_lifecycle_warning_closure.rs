mod support;

use support::sprint105_support::run_sprint105;

#[test]
fn paper_lifecycle_warning_closure_counts_remaining_warning_backing() {
    let bundle = run_sprint105(
        "soma_paper_lifecycle_warning_closure.toml",
        "paper_lifecycle_warning_closure",
    );
    let report = &bundle.paper_lifecycle_warning_closure_report;
    assert!(
        report.closure_status == "PaperLifecycleWarningsClosed"
            || report.closure_status == "PaperLifecycleStillWarningBacked"
    );
}
