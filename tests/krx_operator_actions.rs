use std::path::PathBuf;

use soma_zero::{
    KRXAuthReadinessReport, KRXAuthReadinessStatus, KRXCanonicalValidationReport,
    KRXOfficialEvidenceActivationConfig, KRXOperatorActionKind, KRXSymbolWhitelistConfig,
    build_krx_operator_actions,
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

fn whitelist() -> soma_zero::KRXSymbolWhitelist {
    KRXSymbolWhitelistConfig::from_toml_path(&example_path(
        "soma_krx_symbol_whitelist_compact.toml",
    ))
    .expect("parse whitelist")
    .build()
}

fn validation_without_official_readiness() -> KRXCanonicalValidationReport {
    KRXCanonicalValidationReport::validate(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("examples")
            .join("sprint49_data")
            .join("krx_005930_1d_compact.csv")
            .display()
            .to_string(),
        Some("005930".to_string()),
        Some("005930".to_string()),
        None,
        None,
        true,
        true,
    )
}

#[test]
fn missing_auth_creates_secret_safe_actions() {
    let config = KRXOfficialEvidenceActivationConfig::from_toml_path(&example_path(
        "soma_krx_official_activate_missing_auth.toml",
    ))
    .expect("parse config");
    let actions = build_krx_operator_actions(
        &config,
        &auth(KRXAuthReadinessStatus::MissingApiKeyAndEndpointTemplate),
        &whitelist(),
        &[],
        false,
    );
    let kinds = actions
        .iter()
        .map(|action| action.action_kind)
        .collect::<Vec<_>>();
    assert!(kinds.contains(&KRXOperatorActionKind::SetKRXApiKey));
    assert!(kinds.contains(&KRXOperatorActionKind::SetKRXEndpointTemplate));
    let rendered = actions
        .iter()
        .map(|action| action.to_text())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(!rendered.contains("krx_test_redaction_value"));
    assert!(rendered.contains("KRX_API_KEY"));
}

#[test]
fn missing_provenance_and_preflight_create_required_actions() {
    let config = KRXOfficialEvidenceActivationConfig::from_toml_path(&example_path(
        "soma_krx_official_activate_local_import.toml",
    ))
    .expect("parse config");
    let actions = build_krx_operator_actions(
        &config,
        &auth(KRXAuthReadinessStatus::Ready),
        &whitelist(),
        &[validation_without_official_readiness()],
        false,
    );
    let kinds = actions
        .iter()
        .map(|action| action.action_kind)
        .collect::<Vec<_>>();
    assert!(kinds.contains(&KRXOperatorActionKind::ProvideKRXProvenance));
    assert!(kinds.contains(&KRXOperatorActionKind::RunKRXPreflight));
}

#[test]
fn broad_scope_creates_reduce_scope_action_deterministically() {
    let mut config = KRXOfficialEvidenceActivationConfig::from_toml_path(&example_path(
        "soma_krx_official_activate_local_import.toml",
    ))
    .expect("parse config");
    config.max_symbols = 1;
    let first = build_krx_operator_actions(
        &config,
        &auth(KRXAuthReadinessStatus::Ready),
        &whitelist(),
        &[],
        false,
    );
    let second = build_krx_operator_actions(
        &config,
        &auth(KRXAuthReadinessStatus::Ready),
        &whitelist(),
        &[],
        false,
    );
    assert_eq!(first, second);
    assert!(
        first
            .iter()
            .any(|action| action.action_kind == KRXOperatorActionKind::ReduceScope)
    );
}
