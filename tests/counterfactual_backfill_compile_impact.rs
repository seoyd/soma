mod support;

use soma_zero::CounterfactualBackfillCompileImpactStatus;
use support::sprint69_support as sprint;

#[test]
fn counterfactual_backfill_compile_impact_stays_sample_backed() {
    let report = sprint::run_sprint97_bundle(
        "soma_sprint97_counterfactual_backfill_recover.toml",
        "counterfactual-compile-impact",
    )
    .counterfactual_backfill_compile_impact_report;
    assert_eq!(
        report.impact_status,
        CounterfactualBackfillCompileImpactStatus::CompileImpactSampleBacked
    );
    assert_eq!(report.counterfactual_backfill_delta, Some(0));
}
