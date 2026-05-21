use soma_zero::{
    ProviderAuthCheckMode, ProviderCredentialProfile, ProviderDataSubject,
    ProviderEntitlementPreflightConfig, ProviderKind, ProviderRealityConfig, ProviderRealityRunner,
    ProviderSecretValuePolicy, StrategyDataCheckRequest, StrategyUseCase,
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
fn provider_reality_report_surfaces_krx_alpha_alpaca_and_yfinance_reality() {
    let krx_key = unique("KRX_KEY");
    let krx_endpoint = unique("KRX_ENDPOINT");
    let alpha_key = unique("ALPHA_KEY");
    unsafe {
        std::env::set_var(&krx_key, "present");
        std::env::set_var(&krx_endpoint, "present");
        std::env::set_var(&alpha_key, "present");
    }
    let report = ProviderRealityRunner::default()
        .run(&ProviderRealityConfig {
            entitlement_preflight: ProviderEntitlementPreflightConfig {
                providers_to_check: vec![
                    ProviderDataSubject::Provider(ProviderKind::KrxOpenApi),
                    ProviderDataSubject::Provider(ProviderKind::AlphaVantage),
                    ProviderDataSubject::Provider(ProviderKind::Alpaca),
                    ProviderDataSubject::Provider(ProviderKind::Upbit),
                    ProviderDataSubject::YFinanceResearch,
                ],
                credential_profile_overrides: vec![
                    override_profile(
                        ProviderKind::KrxOpenApi,
                        std::slice::from_ref(&krx_key),
                        std::slice::from_ref(&krx_endpoint),
                    ),
                    override_profile(
                        ProviderKind::AlphaVantage,
                        std::slice::from_ref(&alpha_key),
                        &[],
                    ),
                ],
                required_use_case: soma_zero::ProviderEntitlementUseCase::EodResearch,
                ..ProviderEntitlementPreflightConfig::default()
            },
            strategy_checks: vec![StrategyDataCheckRequest {
                provider: "yfinance".to_string(),
                use_case: StrategyUseCase::SourceComparison,
            }],
            ..ProviderRealityConfig::default()
        })
        .expect("report");

    assert!(
        report
            .final_summary
            .contains(&soma_zero::ProviderRealitySummary::KRXApprovalPending)
    );
    assert!(
        report
            .final_summary
            .contains(&soma_zero::ProviderRealitySummary::AlphaVantageEodOnly)
    );
    assert!(
        report
            .final_summary
            .contains(&soma_zero::ProviderRealitySummary::AlpacaNeededForRealtime)
    );
    assert!(
        report
            .final_summary
            .contains(&soma_zero::ProviderRealitySummary::YFinanceResearchOnly)
    );
    assert!(!report.operator_actions.is_empty());
    unsafe {
        std::env::remove_var(&krx_key);
        std::env::remove_var(&krx_endpoint);
        std::env::remove_var(&alpha_key);
    }
}

#[test]
fn provider_reality_report_is_deterministic() {
    let first = ProviderRealityRunner::default()
        .run(&ProviderRealityConfig::default())
        .expect("report")
        .to_json_string()
        .expect("json");
    let second = ProviderRealityRunner::default()
        .run(&ProviderRealityConfig::default())
        .expect("report")
        .to_json_string()
        .expect("json");
    assert_eq!(first, second);
}
