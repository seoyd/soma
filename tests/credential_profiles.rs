use soma_zero::{
    ProviderAuthCheckMode, ProviderKind, default_provider_credential_profiles,
    evaluate_provider_credential_profile,
};

fn profile(provider_kind: ProviderKind) -> soma_zero::ProviderCredentialProfile {
    default_provider_credential_profiles()
        .into_iter()
        .find(|profile| profile.provider_kind == provider_kind)
        .expect("profile")
}

#[test]
fn sprint29_profiles_expose_expected_env_var_names() {
    assert_eq!(
        profile(ProviderKind::KrxOpenApi).required_env_vars,
        vec!["KRX_API_KEY".to_string()]
    );
    assert_eq!(
        profile(ProviderKind::KrxOpenApi).endpoint_template_env_vars,
        vec!["KRX_ENDPOINT_TEMPLATE".to_string()]
    );
    assert_eq!(
        profile(ProviderKind::DataGoKrFscStockPrice).required_env_vars,
        vec!["DATA_GO_KR_SERVICE_KEY".to_string()]
    );
    assert_eq!(
        profile(ProviderKind::KoreaInvestmentMarketData).required_env_vars,
        vec!["KIS_APP_KEY".to_string(), "KIS_APP_SECRET".to_string()]
    );
    assert_eq!(
        profile(ProviderKind::KoreaInvestmentMarketData).optional_env_vars,
        vec!["KIS_BASE_URL".to_string()]
    );
    assert_eq!(
        profile(ProviderKind::AlphaVantage).required_env_vars,
        vec!["ALPHAVANTAGE_API_KEY".to_string()]
    );
    assert_eq!(
        profile(ProviderKind::Alpaca).required_env_vars,
        vec![
            "ALPACA_API_KEY_ID".to_string(),
            "ALPACA_API_SECRET_KEY".to_string()
        ]
    );
    assert_eq!(
        profile(ProviderKind::PolygonProfessional).required_env_vars,
        vec!["POLYGON_API_KEY".to_string()]
    );
    assert_eq!(
        profile(ProviderKind::NasdaqDataLink).required_env_vars,
        vec!["NASDAQ_DATA_LINK_API_KEY".to_string()]
    );
}

#[test]
fn credential_profiles_use_env_var_name_only_policy() {
    let profile = profile(ProviderKind::Alpaca);
    let status = evaluate_provider_credential_profile(&profile);
    let json = serde_json::to_string(&status).expect("json");
    assert!(!json.contains("super-secret-value"));
    assert!(json.contains("ALPACA_API_KEY_ID"));
    assert!(json.contains("ALPACA_API_SECRET_KEY"));
    assert_eq!(profile.auth_check_mode, ProviderAuthCheckMode::PresenceOnly);
}
