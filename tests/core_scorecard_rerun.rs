use soma_zero::CoreScorecardRerun;

#[test]
fn scorecard_rerun_missing_reason_codes_are_reported() {
    let summary = CoreScorecardRerun::missing("missing config");
    assert!(!summary.ran);
    assert!(
        summary
            .warnings
            .iter()
            .any(|warning| warning.contains("missing config"))
    );
}

#[test]
fn scorecard_rerun_summarize_reports_no_run_warning() {
    let summary = CoreScorecardRerun::default().summarize(None, None, Vec::new(), false);
    assert!(!summary.ran);
    assert!(
        summary
            .warnings
            .iter()
            .any(|warning| warning.contains("not attempted"))
    );
}
