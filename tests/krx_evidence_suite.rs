use std::path::PathBuf;

use soma_zero::{
    KRXAuthReadinessReport, KRXAuthReadinessStatus, KRXEvidenceJobKind,
    KRXOfficialEvidenceActivationConfig, KRXSymbolWhitelistConfig,
};

fn example_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join(name)
}

fn auth(status: KRXAuthReadinessStatus) -> KRXAuthReadinessReport {
    KRXAuthReadinessReport {
        api_key_env_var_name: "KRX_API_KEY".to_string(),
        api_key_present: matches!(
            status,
            KRXAuthReadinessStatus::Ready | KRXAuthReadinessStatus::MissingEndpointTemplate
        ),
        endpoint_template_env_var_name: "KRX_ENDPOINT_TEMPLATE".to_string(),
        endpoint_template_present: matches!(
            status,
            KRXAuthReadinessStatus::Ready | KRXAuthReadinessStatus::MissingApiKey
        ),
        endpoint_template_preview_redacted: None,
        readiness_status: status,
        safe_to_collect_market_data: matches!(status, KRXAuthReadinessStatus::Ready),
        reason_codes: Vec::new(),
    }
}

fn load_whitelist() -> soma_zero::KRXSymbolWhitelist {
    let config = KRXSymbolWhitelistConfig::from_toml_path(&example_path(
        "soma_krx_symbol_whitelist_compact.toml",
    ))
    .expect("parse whitelist config");
    config.build()
}

#[test]
fn krx_evidence_uses_local_import_without_auth() {
    let config = KRXOfficialEvidenceActivationConfig::from_toml_path(&example_path(
        "soma_krx_official_activate_local_import.toml",
    ))
    .expect("parse local import config");
    let plan = soma_zero::KRXEvidenceJobPlan::build(
        &config,
        &auth(KRXAuthReadinessStatus::MissingApiKeyAndEndpointTemplate),
        &load_whitelist(),
    );
    assert_eq!(plan.collection_jobs.len(), 0);
    assert_eq!(plan.local_import_jobs.len(), 2);
    assert_eq!(plan.runnable_jobs.len(), 2);
    assert!(plan.jobs.iter().all(|job| {
        matches!(
            job.job_kind,
            KRXEvidenceJobKind::LocalCanonicalCsvImport
                | KRXEvidenceJobKind::ExistingCollectedCsvReuse
        )
    }));
}

#[test]
fn krx_evidence_missing_auth_and_endpoint_requirements_stay_market_data_only() {
    let config = KRXOfficialEvidenceActivationConfig::from_toml_path(&example_path(
        "soma_krx_official_activate_missing_auth.toml",
    ))
    .expect("parse missing auth config");
    let plan = soma_zero::KRXEvidenceJobPlan::build(
        &config,
        &auth(KRXAuthReadinessStatus::MissingApiKeyAndEndpointTemplate),
        &load_whitelist(),
    );
    assert_eq!(plan.collection_jobs.len(), 0);
    assert_eq!(plan.skipped_jobs.len(), 2);
    assert!(
        plan.jobs
            .iter()
            .all(|job| matches!(job.job_kind, KRXEvidenceJobKind::SkippedMissingApiKey))
    );

    let endpoint_only = auth(KRXAuthReadinessStatus::MissingEndpointTemplate);
    assert!(!endpoint_only.safe_to_collect_market_data);
}

#[test]
fn krx_evidence_collects_only_when_auth_ready_and_is_deterministic() {
    let mut config = KRXOfficialEvidenceActivationConfig::from_toml_path(&example_path(
        "soma_krx_official_activate_missing_auth.toml",
    ))
    .expect("parse config");
    let ready = soma_zero::KRXEvidenceJobPlan::build(
        &config,
        &auth(KRXAuthReadinessStatus::Ready),
        &load_whitelist(),
    );
    assert_eq!(ready.collection_jobs.len(), 2);
    assert!(
        ready
            .jobs
            .iter()
            .all(|job| matches!(job.job_kind, KRXEvidenceJobKind::KrxEodCollect))
    );

    config.max_bytes = 32;
    let first = soma_zero::KRXEvidenceJobPlan::build(
        &config,
        &auth(KRXAuthReadinessStatus::Ready),
        &load_whitelist(),
    );
    let second = soma_zero::KRXEvidenceJobPlan::build(
        &config,
        &auth(KRXAuthReadinessStatus::Ready),
        &load_whitelist(),
    );
    assert_eq!(first, second);
}
