mod common;

use std::env;
use std::path::Path;
use std::sync::{Mutex, OnceLock};

use soma_zero::{
    OperationalRunbookConfig, OperationalRunbookFinalStatus, OperationalRunbookRunner,
};

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn with_kis_envs<F: FnOnce()>(f: F) {
    let _guard = env_lock().lock().expect("env lock");
    let old_key = env::var_os("KIS_APP_KEY");
    let old_secret = env::var_os("KIS_APP_SECRET");
    let old_base_url = env::var_os("KIS_BASE_URL");
    unsafe {
        env::set_var("KIS_APP_KEY", "fixture-key");
        env::set_var("KIS_APP_SECRET", "fixture-secret");
        env::set_var("KIS_BASE_URL", "https://redacted.local");
    }
    f();
    match old_key {
        Some(value) => unsafe { env::set_var("KIS_APP_KEY", value) },
        None => unsafe { env::remove_var("KIS_APP_KEY") },
    }
    match old_secret {
        Some(value) => unsafe { env::set_var("KIS_APP_SECRET", value) },
        None => unsafe { env::remove_var("KIS_APP_SECRET") },
    }
    match old_base_url {
        Some(value) => unsafe { env::set_var("KIS_BASE_URL", value) },
        None => unsafe { env::remove_var("KIS_BASE_URL") },
    }
}

#[test]
fn operational_runbook_builds_ordered_local_steps() {
    with_kis_envs(|| {
        let config = OperationalRunbookConfig::from_toml_path(Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/examples/soma_operational_runbook_kis_loop.toml"
        )))
        .unwrap();
        let report = OperationalRunbookRunner::default().run(&config).unwrap();
        assert_eq!(
            report.final_status,
            OperationalRunbookFinalStatus::ReadyToRun
        );
        assert!(report.required_steps >= 7);
        assert!(report.steps.iter().all(|step| {
            let command = step
                .command_suggestion
                .clone()
                .unwrap_or_default()
                .to_ascii_lowercase();
            !command.contains("order")
                && !command.contains("broker")
                && !command.contains("account")
                && !command.contains("live-trading")
        }));
    });
}

#[test]
fn operational_runbook_missing_evidence_blocks() {
    with_kis_envs(|| {
        let config = OperationalRunbookConfig {
            runbook_id: "missing-evidence".to_string(),
            ..OperationalRunbookConfig::default()
        };
        let report = OperationalRunbookRunner::default().run(&config).unwrap();
        assert_eq!(
            report.final_status,
            OperationalRunbookFinalStatus::BlockedByMissingEvidence
        );
        assert!(report.blocked_steps > 0);
    });
}
