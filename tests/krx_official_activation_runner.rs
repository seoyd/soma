mod common;

use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use soma_zero::{
    KRXOfficialEvidenceActivationConfig, KRXOfficialEvidenceActivationFinalStatus,
    KRXOfficialEvidenceActivationRunner, KRXOperatorActionKind,
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

fn config_from_example(name: &str, output_name: &str) -> KRXOfficialEvidenceActivationConfig {
    let mut config = KRXOfficialEvidenceActivationConfig::from_toml_path(&example_path(name))
        .expect("parse activation config");
    config.output_root = common::output_dir(output_name).display().to_string();
    config
}

#[test]
fn missing_auth_stays_blocked_for_collection_only_runs() {
    with_krx_env(None, None, || {
        let bundle = KRXOfficialEvidenceActivationRunner::default()
            .run(&config_from_example(
                "soma_krx_official_activate_missing_auth.toml",
                "krx-runner-missing-auth",
            ))
            .expect("run activation");
        assert_eq!(
            bundle.activation_report.final_status,
            KRXOfficialEvidenceActivationFinalStatus::KRXAuthMissing
        );
        assert!(
            bundle
                .operator_actions
                .iter()
                .any(|action| action.action_kind == KRXOperatorActionKind::SetKRXApiKey)
        );
    });
}

#[test]
fn local_import_adds_official_rows_even_without_env_auth() {
    let bundle = KRXOfficialEvidenceActivationRunner::default()
        .run(&config_from_example(
            "soma_krx_official_activate_local_import.toml",
            "krx-runner-local-import",
        ))
        .expect("run local import activation");
    assert_eq!(
        bundle.activation_report.final_status,
        KRXOfficialEvidenceActivationFinalStatus::KRXOfficialRowsImported
    );
    assert!(bundle.activation_report.added_krx_official_rows > 0);
    assert!(
        bundle
            .canonical_validation_reports
            .iter()
            .all(|report| report.official_readiness_eligible)
    );
    assert!(
        bundle
            .activation_report
            .blockers
            .iter()
            .all(|blocker| !blocker.contains("KRX_API_KEY"))
    );
    assert!(
        PathBuf::from(&bundle.activation_report.activation_id)
            .components()
            .count()
            > 0
    );
}

#[test]
fn missing_preflight_blocks_local_import_activation() {
    let mut config = config_from_example(
        "soma_krx_official_activate_local_import.toml",
        "krx-runner-missing-preflight",
    );
    config.local_krx_preflight_paths.clear();
    let bundle = KRXOfficialEvidenceActivationRunner::default()
        .run(&config)
        .expect("run blocked activation");
    assert_eq!(
        bundle.activation_report.final_status,
        KRXOfficialEvidenceActivationFinalStatus::KRXCollectionBlockedByPreflight
    );
    assert_eq!(bundle.activation_report.added_krx_official_rows, 0);
    assert!(
        bundle
            .operator_actions
            .iter()
            .any(|action| action.action_kind == KRXOperatorActionKind::RunKRXPreflight)
    );
}

#[test]
fn diversity_and_core_reruns_are_recorded() {
    let bundle = KRXOfficialEvidenceActivationRunner::default()
        .run(&config_from_example(
            "soma_krx_official_activate_diversity_rerun.toml",
            "krx-runner-diversity",
        ))
        .expect("run diversity rerun activation");
    assert!(bundle.downstream_rerun_summary.official_replication_ran);
    assert!(bundle.downstream_rerun_summary.diversity_sweep_ran);
    assert!(bundle.downstream_rerun_summary.core_performance_ran);
    assert!(bundle.activation_report.current_core_status.is_some());
}

#[test]
fn runner_is_deterministic() {
    with_krx_env(None, None, || {
        let config = config_from_example(
            "soma_krx_official_activate_missing_auth.toml",
            "krx-runner-determinism",
        );
        let first = KRXOfficialEvidenceActivationRunner::default()
            .run(&config)
            .expect("first run");
        let second = KRXOfficialEvidenceActivationRunner::default()
            .run(&config)
            .expect("second run");
        assert_eq!(first, second);
    });
}
