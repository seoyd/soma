use soma_zero::{ProviderKind, build_auth_setup_guide};

#[test]
fn krx_setup_guide_includes_required_env_var_names() {
    let guide = build_auth_setup_guide(ProviderKind::KrxOpenApi);
    assert!(guide.required_env_vars.contains(&"KRX_API_KEY".to_string()));
    assert!(
        guide
            .endpoint_template_requirements
            .contains(&"KRX_ENDPOINT_TEMPLATE".to_string())
    );
}

#[test]
fn alphavantage_setup_guide_includes_required_env_var_names() {
    let guide = build_auth_setup_guide(ProviderKind::AlphaVantage);
    assert!(
        guide
            .required_env_vars
            .contains(&"ALPHAVANTAGE_API_KEY".to_string())
    );
}

#[test]
fn auth_setup_guide_contains_no_secret_values() {
    let text = build_auth_setup_guide(ProviderKind::Alpaca).to_text();
    assert!(!text.contains("secret-value"));
    assert!(!text.contains("sk-"));
}

#[test]
fn auth_setup_guide_is_deterministic() {
    let first = build_auth_setup_guide(ProviderKind::KrxOpenApi);
    let second = build_auth_setup_guide(ProviderKind::KrxOpenApi);
    assert_eq!(first, second);
}
