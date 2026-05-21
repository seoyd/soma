mod common;
#[path = "support/sprint46_support.rs"]
mod sprint46_support;

use soma_zero::{
    FutureWindowExtensionJobKind, FutureWindowExtensionJobStatus,
    build_official_future_window_extension_plan,
};

#[test]
fn future_window_extension_plan_prefers_local_extension_jobs() {
    let plan = build_official_future_window_extension_plan(&sprint46_support::extension_config(
        "future-window-extension",
    ))
    .expect("extension plan");
    assert_eq!(plan.jobs.len(), 1);
    assert_eq!(
        plan.jobs[0].job_kind,
        FutureWindowExtensionJobKind::LocalCsvWindowExtension
    );
    assert_eq!(
        plan.jobs[0].status,
        FutureWindowExtensionJobStatus::ReadyToRun
    );
    assert_eq!(plan.runnable_jobs, 1);
}
