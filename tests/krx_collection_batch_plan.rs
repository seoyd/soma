use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use soma_zero::{
    KRXBoundedCollectionSmokeConfig, KRXCollectionBatchJobKind, KRXCollectionBatchPlan,
    KRXCollectionDryRunStatus,
};

fn example_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join(name)
}

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn with_krx_env(api_key: Option<&str>, endpoint: Option<&str>, run: impl FnOnce()) {
    let _guard = env_lock().lock().expect("env lock");
    let old_key = std::env::var_os("KRX_API_KEY");
    let old_endpoint = std::env::var_os("KRX_ENDPOINT_TEMPLATE");
    match api_key {
        Some(value) => unsafe { std::env::set_var("KRX_API_KEY", value) },
        None => unsafe { std::env::remove_var("KRX_API_KEY") },
    }
    match endpoint {
        Some(value) => unsafe { std::env::set_var("KRX_ENDPOINT_TEMPLATE", value) },
        None => unsafe { std::env::remove_var("KRX_ENDPOINT_TEMPLATE") },
    }
    run();
    match old_key {
        Some(value) => unsafe { std::env::set_var("KRX_API_KEY", value) },
        None => unsafe { std::env::remove_var("KRX_API_KEY") },
    }
    match old_endpoint {
        Some(value) => unsafe { std::env::set_var("KRX_ENDPOINT_TEMPLATE", value) },
        None => unsafe { std::env::remove_var("KRX_ENDPOINT_TEMPLATE") },
    }
}

#[test]
fn fixture_and_local_import_jobs_are_planned() {
    let config = KRXBoundedCollectionSmokeConfig::from_toml_path(&example_path(
        "soma_krx_collection_dry_run.toml",
    ))
    .expect("parse config");
    let whitelist = config.load_whitelist().expect("whitelist");
    let dry_run = config.build_dry_run_report(&whitelist);
    let plan = KRXCollectionBatchPlan::build(&config, &dry_run, &whitelist);
    assert_eq!(plan.fixture_replay_jobs.len(), 2);
    assert_eq!(plan.local_import_jobs.len(), 2);
    assert!(
        plan.jobs
            .iter()
            .any(|job| job.job_kind == KRXCollectionBatchJobKind::SkippedLiveCollectionDisabled)
    );
}

#[test]
fn live_jobs_stay_skipped_when_auth_missing() {
    with_krx_env(None, None, || {
        let config = KRXBoundedCollectionSmokeConfig::from_toml_path(&example_path(
            "soma_krx_collection_plan_missing_auth.toml",
        ))
        .expect("parse config");
        let whitelist = config.load_whitelist().expect("whitelist");
        let dry_run = config.build_dry_run_report(&whitelist);
        assert_eq!(
            dry_run.dry_run_status,
            KRXCollectionDryRunStatus::MissingApiKeyAndEndpointTemplate
        );
        let plan = KRXCollectionBatchPlan::build(&config, &dry_run, &whitelist);
        assert!(plan.live_collection_jobs.is_empty());
        assert!(plan.jobs.iter().any(|job| {
            matches!(
                job.job_kind,
                KRXCollectionBatchJobKind::SkippedMissingApiKey
            )
        }));
    });
}

#[test]
fn budget_exceeded_is_deterministic() {
    let mut config = KRXBoundedCollectionSmokeConfig::from_toml_path(&example_path(
        "soma_krx_collection_dry_run.toml",
    ))
    .expect("parse config");
    config.max_total_bytes = 32;
    let whitelist = config.load_whitelist().expect("whitelist");
    let dry_run = config.build_dry_run_report(&whitelist);
    let first = KRXCollectionBatchPlan::build(&config, &dry_run, &whitelist);
    let second = KRXCollectionBatchPlan::build(&config, &dry_run, &whitelist);
    assert_eq!(first, second);
    assert!(first.jobs.iter().all(|job| matches!(
        job.job_kind,
        KRXCollectionBatchJobKind::SkippedBudgetExceeded
    )));
}
