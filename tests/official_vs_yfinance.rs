use soma_zero::{OfficialVsYFinanceStatus, build_official_vs_yfinance_interpretation};

#[test]
fn yfinance_only_is_research_only_no_official_claim() {
    let report = build_official_vs_yfinance_interpretation(0, 2, None, None);
    assert_eq!(
        report.status,
        OfficialVsYFinanceStatus::ResearchOnlyNoOfficialClaim
    );
    assert!(!report.can_count_as_official);
    assert!(!report.can_count_as_readiness);
}

#[test]
fn comparison_warns_when_metrics_diverge() {
    let report = build_official_vs_yfinance_interpretation(1, 1, Some(100.0), Some(92.0));
    assert_eq!(
        report.status,
        OfficialVsYFinanceStatus::ResearchComparisonOnly
    );
    assert!(report.can_compare_for_research);
    assert!(
        report
            .warnings
            .iter()
            .any(|warning| warning.contains("DataSourceMismatch"))
    );
}
