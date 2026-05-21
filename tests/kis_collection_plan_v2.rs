#[path = "support/sprint58_support.rs"]
mod sprint58_support;

use soma_zero::{
    KISCollectionPlanV2JobKind, KISCollectionPlanV2Runner, KISCollectionPlanV2Status,
    KISMarketDataDryRunRunner,
};

#[test]
fn collection_plan_v2_keeps_live_collection_disabled() {
    sprint58_support::with_kis_env(
        Some("fixture-key"),
        Some("fixture-secret"),
        Some("https://redacted.local"),
        None,
        || {
            let out = sprint58_support::output_dir("kis-collection-plan-v2");
            let dry_run = KISMarketDataDryRunRunner::default()
                .run(&sprint58_support::dry_run_config(&out))
                .expect("dry run");
            let plan = KISCollectionPlanV2Runner::default()
                .run_with_dry_run(
                    &sprint58_support::collection_plan_config(&out),
                    Some(&dry_run),
                )
                .expect("plan");
            assert_eq!(
                plan.plan_status,
                KISCollectionPlanV2Status::LiveCollectionDisabled
            );
            assert_eq!(plan.fixture_jobs.len(), 2);
            assert_eq!(plan.local_import_jobs.len(), 2);
            assert!(plan.jobs.iter().any(|job| {
                matches!(
                    job.job_kind,
                    KISCollectionPlanV2JobKind::SkippedLiveCollectionDisabled
                )
            }));
        },
    );
}
