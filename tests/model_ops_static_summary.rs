mod common;
#[path = "support/sprint67_support.rs"]
mod sprint67_support;

use soma_zero::ModelOpsStaticSummaryStatus;

#[test]
fn static_summary_is_plain_language_mentions_regressions_and_mamba_deferred() {
    let bundle = sprint67_support::run_rollup("soma_model_ops_rollup.toml", "static-summary");
    let summary = bundle.model_ops_static_summary_report;
    assert_eq!(
        summary.overall_status,
        ModelOpsStaticSummaryStatus::NeedsMorePredictions
    );
    assert!(summary.plain_language_summary.contains("rollup_id="));
    assert!(
        summary
            .plain_language_summary
            .contains("regression_status=")
    );
    assert!(
        summary
            .plain_language_summary
            .contains("mamba_runtime=deferred")
    );
    assert!(
        !summary
            .plain_language_summary
            .to_lowercase()
            .contains("profitability")
    );
}
