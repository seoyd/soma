use soma_zero::{
    OfficialProviderReadinessConfig, OfficialProviderReadinessRunner, ProviderAuthCheckMode,
    ProviderCredentialProfile, ProviderKind, ProviderMarket,
};

fn unique(name: &str) -> String {
    format!("SOMA_TEST_{name}")
}

fn override_profile(
    provider_kind: ProviderKind,
    required_env_vars: &[String],
    endpoint_template_env_vars: &[String],
) -> ProviderCredentialProfile {
    ProviderCredentialProfile {
        provider_kind,
        required_env_vars: required_env_vars.to_vec(),
        optional_env_vars: Vec::new(),
        endpoint_template_env_vars: endpoint_template_env_vars.to_vec(),
        secret_value_policy: vec![
            soma_zero::ProviderSecretValuePolicy::EnvVarNameOnly,
            soma_zero::ProviderSecretValuePolicy::NeverPersistSecret,
            soma_zero::ProviderSecretValuePolicy::NeverPrintSecret,
        ],
        auth_check_mode: if endpoint_template_env_vars.is_empty() {
            ProviderAuthCheckMode::PresenceOnly
        } else {
            ProviderAuthCheckMode::EndpointTemplateRequired
        },
        reason_codes: Vec::new(),
    }
}

#[test]
fn readiness_report_marks_multi_market_ready_when_crypto_krx_and_us_are_ready() {
    let krx_key = unique("KRX_KEY_READY");
    let krx_endpoint = unique("KRX_ENDPOINT_READY");
    let alpha = unique("ALPHA_READY");
    unsafe {
        std::env::set_var(&krx_key, "present");
        std::env::set_var(&krx_endpoint, "present");
        std::env::set_var(&alpha, "present");
    }
    let report = OfficialProviderReadinessRunner::default().run(&OfficialProviderReadinessConfig {
        credential_profile_overrides: vec![
            override_profile(
                ProviderKind::KrxOpenApi,
                std::slice::from_ref(&krx_key),
                std::slice::from_ref(&krx_endpoint),
            ),
            override_profile(
                ProviderKind::AlphaVantage,
                std::slice::from_ref(&alpha),
                &[],
            ),
        ],
        ..OfficialProviderReadinessConfig::default()
    });

    assert_eq!(
        report.final_status,
        soma_zero::OfficialProviderReadinessStatus::ReadyForMultiVenueEvidence
    );
    assert!(
        report
            .official_ready_markets
            .contains(&ProviderMarket::Crypto)
    );
    assert!(
        report
            .official_ready_markets
            .contains(&ProviderMarket::KoreanEquity)
    );
    assert!(
        report
            .official_ready_markets
            .contains(&ProviderMarket::USEquity)
    );
    unsafe {
        std::env::remove_var(&krx_key);
        std::env::remove_var(&krx_endpoint);
        std::env::remove_var(&alpha);
    }
}

#[test]
fn missing_korean_auth_produces_missing_korean_auth_status() {
    let alpha = unique("ALPHA_ONLY");
    unsafe { std::env::set_var(&alpha, "present") };
    let report = OfficialProviderReadinessRunner::default().run(&OfficialProviderReadinessConfig {
        credential_profile_overrides: vec![
            override_profile(
                ProviderKind::KrxOpenApi,
                &[unique("KRX_MISSING")],
                &[unique("KRX_ENDPOINT_MISSING")],
            ),
            override_profile(
                ProviderKind::DataGoKrFscStockPrice,
                &[unique("DATA_GO_MISSING")],
                &[],
            ),
            override_profile(
                ProviderKind::KoreaInvestmentMarketData,
                &[unique("KIS_KEY_MISSING"), unique("KIS_SECRET_MISSING")],
                &[],
            ),
            override_profile(
                ProviderKind::AlphaVantage,
                std::slice::from_ref(&alpha),
                &[],
            ),
        ],
        ..OfficialProviderReadinessConfig::default()
    });

    assert_eq!(
        report.final_status,
        soma_zero::OfficialProviderReadinessStatus::MissingKoreanAuth
    );
    unsafe { std::env::remove_var(&alpha) };
}

#[test]
fn missing_us_auth_produces_missing_us_auth_status() {
    let krx_key = unique("KRX_KEY_ONLY");
    let krx_endpoint = unique("KRX_ENDPOINT_ONLY");
    unsafe {
        std::env::set_var(&krx_key, "present");
        std::env::set_var(&krx_endpoint, "present");
    }
    let report = OfficialProviderReadinessRunner::default().run(&OfficialProviderReadinessConfig {
        credential_profile_overrides: vec![
            override_profile(
                ProviderKind::KrxOpenApi,
                std::slice::from_ref(&krx_key),
                std::slice::from_ref(&krx_endpoint),
            ),
            override_profile(ProviderKind::AlphaVantage, &[unique("ALPHA_MISSING")], &[]),
            override_profile(
                ProviderKind::Alpaca,
                &[
                    unique("ALPACA_KEY_MISSING"),
                    unique("ALPACA_SECRET_MISSING"),
                ],
                &[],
            ),
        ],
        ..OfficialProviderReadinessConfig::default()
    });

    assert_eq!(
        report.final_status,
        soma_zero::OfficialProviderReadinessStatus::MissingUSAuth
    );
    unsafe {
        std::env::remove_var(&krx_key);
        std::env::remove_var(&krx_endpoint);
    }
}

#[test]
fn provider_readiness_report_is_deterministic() {
    let first = OfficialProviderReadinessRunner::default()
        .run(&OfficialProviderReadinessConfig::default())
        .to_json_string()
        .expect("json");
    let second = OfficialProviderReadinessRunner::default()
        .run(&OfficialProviderReadinessConfig::default())
        .to_json_string()
        .expect("json");
    assert_eq!(first, second);
}
