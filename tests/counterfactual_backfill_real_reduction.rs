mod support;

use soma_zero::{
    CounterfactualBackfillCompileImpactStatus, CounterfactualBackfillRealReductionPlanStatus,
    CounterfactualBackfillRealReductionStatus,
};
use support::sprint69_support as sprint;

#[test]
fn counterfactual_backfill_real_reduction_is_ready_and_conservative() {
    let bundle = sprint::run_sprint97_bundle(
        "soma_sprint97_counterfactual_backfill_recover.toml",
        "counterfactual-real-reduction",
    );
    assert_eq!(
        bundle
            .counterfactual_backfill_real_reduction_plan
            .plan_status,
        CounterfactualBackfillRealReductionPlanStatus::CounterfactualBackfillReductionPlanReady
    );
    assert_eq!(
        bundle
            .counterfactual_backfill_real_reduction_report
            .reduction_status,
        CounterfactualBackfillRealReductionStatus::CounterfactualBackfillReducedWithWarnings
    );
    assert_eq!(
        bundle
            .counterfactual_backfill_compile_impact_report
            .impact_status,
        CounterfactualBackfillCompileImpactStatus::CompileImpactSampleBacked
    );
}
