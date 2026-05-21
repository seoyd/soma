mod support;

use soma_zero::ChairmanDryRunRecommendation;
use support::sprint102_support::run_sprint102;

#[test]
fn chairman_synthesis_and_audit_remain_conservative() {
    let bundle = run_sprint102("soma_chairman_synthesis_dry_run.toml", "sprint102-chairman");
    let report = &bundle.chairman_synthesis_dry_run_report;
    assert!(matches!(
        report.chairman_recommendation,
        ChairmanDryRunRecommendation::WatchCandidate
            | ChairmanDryRunRecommendation::PaperConditionalCandidate
            | ChairmanDryRunRecommendation::NoTrade
            | ChairmanDryRunRecommendation::RiskDeny
            | ChairmanDryRunRecommendation::NeedMoreEvidence
    ));
    assert!(
        bundle
            .chairman_style_weight_adjustment_audit
            .low_confidence_caps_applied
    );
    assert!(
        !bundle
            .chairman_style_weight_adjustment_audit
            .risk_governor_override_attempted
    );
}
