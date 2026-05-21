use soma_zero::{
    ProviderAuthCheckMode, ProviderCredentialProfile, ProviderDataSubject,
    ProviderEntitlementPreflightConfig, ProviderEntitlementPreflightRunner,
    ProviderEntitlementStatusKind, ProviderEntitlementUseCase, ProviderKind,
    ProviderSecretValuePolicy,
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
            ProviderSecretValuePolicy::EnvVarNameOnly,
            ProviderSecretValuePolicy::NeverPersistSecret,
            ProviderSecretValuePolicy::NeverPrintSecret,
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
fn krx_approval_pending_reports_missing_approval() {
    let key = unique("KRX_KEY");
    let endpoint = unique("KRX_ENDPOINT");
    unsafe {
        std::env::set_var(&key, "present");
        std::env::set_var(&endpoint, "present");
    }
    let statuses =
        ProviderEntitlementPreflightRunner::default().run(&ProviderEntitlementPreflightConfig {
            providers_to_check: vec![ProviderDataSubject::Provider(ProviderKind::KrxOpenApi)],
            required_use_case: ProviderEntitlementUseCase::EodResearch,
            credential_profile_overrides: vec![override_profile(
                ProviderKind::KrxOpenApi,
                std::slice::from_ref(&key),
                std::slice::from_ref(&endpoint),
            )],
            ..ProviderEntitlementPreflightConfig::default()
        });
    assert_eq!(
        statuses[0].status,
        ProviderEntitlementStatusKind::MissingApproval
    );
    unsafe {
        std::env::remove_var(&key);
        std::env::remove_var(&endpoint);
    }
}

#[test]
fn krx_missing_endpoint_reports_missing_endpoint_template() {
    let key = unique("KRX_KEY_ONLY");
    let endpoint = unique("KRX_ENDPOINT_MISSING");
    unsafe {
        std::env::set_var(&key, "present");
        std::env::remove_var(&endpoint);
    }
    let statuses =
        ProviderEntitlementPreflightRunner::default().run(&ProviderEntitlementPreflightConfig {
            providers_to_check: vec![ProviderDataSubject::Provider(ProviderKind::KrxOpenApi)],
            required_use_case: ProviderEntitlementUseCase::EodResearch,
            credential_profile_overrides: vec![override_profile(
                ProviderKind::KrxOpenApi,
                std::slice::from_ref(&key),
                std::slice::from_ref(&endpoint),
            )],
            ..ProviderEntitlementPreflightConfig::default()
        });
    assert_eq!(
        statuses[0].status,
        ProviderEntitlementStatusKind::MissingEndpointTemplate
    );
    unsafe { std::env::remove_var(&key) };
}

#[test]
fn alphavantage_free_is_eod_ready_but_not_realtime_ready() {
    let key = unique("ALPHA_KEY");
    unsafe { std::env::set_var(&key, "present") };
    let eod =
        ProviderEntitlementPreflightRunner::default().run(&ProviderEntitlementPreflightConfig {
            providers_to_check: vec![ProviderDataSubject::Provider(ProviderKind::AlphaVantage)],
            required_use_case: ProviderEntitlementUseCase::EodResearch,
            credential_profile_overrides: vec![override_profile(
                ProviderKind::AlphaVantage,
                std::slice::from_ref(&key),
                &[],
            )],
            ..ProviderEntitlementPreflightConfig::default()
        });
    let realtime =
        ProviderEntitlementPreflightRunner::default().run(&ProviderEntitlementPreflightConfig {
            providers_to_check: vec![ProviderDataSubject::Provider(ProviderKind::AlphaVantage)],
            required_use_case: ProviderEntitlementUseCase::RealtimeResearch,
            credential_profile_overrides: vec![override_profile(
                ProviderKind::AlphaVantage,
                std::slice::from_ref(&key),
                &[],
            )],
            ..ProviderEntitlementPreflightConfig::default()
        });
    assert_eq!(
        eod[0].status,
        ProviderEntitlementStatusKind::ReadyForEodResearch
    );
    assert_eq!(
        realtime[0].status,
        ProviderEntitlementStatusKind::MissingPremiumEntitlement
    );
    unsafe { std::env::remove_var(&key) };
}

#[test]
fn alpaca_basic_is_iex_limited_and_not_full_market() {
    let key = unique("ALPACA_KEY");
    let secret = unique("ALPACA_SECRET");
    unsafe {
        std::env::set_var(&key, "present");
        std::env::set_var(&secret, "present");
    }
    let realtime =
        ProviderEntitlementPreflightRunner::default().run(&ProviderEntitlementPreflightConfig {
            providers_to_check: vec![ProviderDataSubject::Provider(ProviderKind::Alpaca)],
            required_use_case: ProviderEntitlementUseCase::RealtimeResearch,
            credential_profile_overrides: vec![override_profile(
                ProviderKind::Alpaca,
                &[key.clone(), secret.clone()],
                &[],
            )],
            ..ProviderEntitlementPreflightConfig::default()
        });
    let full_market =
        ProviderEntitlementPreflightRunner::default().run(&ProviderEntitlementPreflightConfig {
            providers_to_check: vec![ProviderDataSubject::Provider(ProviderKind::Alpaca)],
            required_use_case: ProviderEntitlementUseCase::FullMarketCoverageResearch,
            credential_profile_overrides: vec![override_profile(
                ProviderKind::Alpaca,
                &[key.clone(), secret.clone()],
                &[],
            )],
            ..ProviderEntitlementPreflightConfig::default()
        });
    assert_eq!(
        realtime[0].status,
        ProviderEntitlementStatusKind::ReadyForRealtimeResearchIexOnly
    );
    assert_eq!(
        full_market[0].status,
        ProviderEntitlementStatusKind::MissingPremiumEntitlement
    );
    unsafe {
        std::env::remove_var(&key);
        std::env::remove_var(&secret);
    }
}

#[test]
fn yfinance_is_research_only_fallback() {
    let statuses =
        ProviderEntitlementPreflightRunner::default().run(&ProviderEntitlementPreflightConfig {
            providers_to_check: vec![ProviderDataSubject::YFinanceResearch],
            required_use_case: ProviderEntitlementUseCase::EodResearch,
            ..ProviderEntitlementPreflightConfig::default()
        });
    assert_eq!(
        statuses[0].status,
        ProviderEntitlementStatusKind::ResearchOnlyFallback
    );
}

#[test]
fn mock_fixture_is_not_readiness_eligible() {
    let statuses =
        ProviderEntitlementPreflightRunner::default().run(&ProviderEntitlementPreflightConfig {
            providers_to_check: vec![ProviderDataSubject::Provider(ProviderKind::MockFixture)],
            required_use_case: ProviderEntitlementUseCase::EodResearch,
            ..ProviderEntitlementPreflightConfig::default()
        });
    assert_eq!(statuses[0].status, ProviderEntitlementStatusKind::Deferred);
    assert!(!statuses[0].official_readiness_eligible);
}

#[test]
fn entitlement_preflight_is_deterministic() {
    let first = serde_json::to_string(
        &ProviderEntitlementPreflightRunner::default()
            .run(&ProviderEntitlementPreflightConfig::default()),
    )
    .expect("json");
    let second = serde_json::to_string(
        &ProviderEntitlementPreflightRunner::default()
            .run(&ProviderEntitlementPreflightConfig::default()),
    )
    .expect("json");
    assert_eq!(first, second);
}
