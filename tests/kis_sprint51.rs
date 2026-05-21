use std::path::Path;

use soma_zero::{
    KISMarketDataActivationConfig, KISOfficialMarketDataActivationRunner,
    KISOutcomeLinkClosureConfig, KISOutcomeLinkClosureRunner, ProviderCredentialStatus,
    ProviderCredentialStatusKind, ProviderKind, ProviderMarket, build_default_provider_catalog,
    default_provider_selection_policies, select_provider,
};

#[test]
fn provider_selection_prefers_kis_for_korean_equity() {
    let catalog = build_default_provider_catalog();
    let policy = default_provider_selection_policies()
        .into_iter()
        .find(|policy| policy.market == ProviderMarket::KoreanEquity)
        .expect("korean equity policy");
    let statuses = vec![
        ProviderCredentialStatus {
            provider_kind: ProviderKind::KoreaInvestmentMarketData,
            required_env_vars: vec!["KIS_APP_KEY".to_string(), "KIS_APP_SECRET".to_string()],
            optional_env_vars: vec!["KIS_BASE_URL".to_string()],
            endpoint_template_env_vars: Vec::new(),
            missing_required_env_vars: Vec::new(),
            missing_endpoint_template_env_vars: Vec::new(),
            status: ProviderCredentialStatusKind::Ready,
            reason_codes: Vec::new(),
        },
        ProviderCredentialStatus {
            provider_kind: ProviderKind::KrxOpenApi,
            required_env_vars: vec!["KRX_API_KEY".to_string()],
            optional_env_vars: Vec::new(),
            endpoint_template_env_vars: vec!["KRX_ENDPOINT_TEMPLATE".to_string()],
            missing_required_env_vars: vec!["KRX_API_KEY".to_string()],
            missing_endpoint_template_env_vars: vec!["KRX_ENDPOINT_TEMPLATE".to_string()],
            status: ProviderCredentialStatusKind::MissingAuth,
            reason_codes: Vec::new(),
        },
    ];
    let result = select_provider(&catalog, &statuses, &policy);
    assert_eq!(
        result.selected_provider,
        Some(ProviderKind::KoreaInvestmentMarketData)
    );
}

#[test]
fn kis_outcome_link_closure_can_derive_sufficiency_from_canonical_paths() {
    let config = KISOutcomeLinkClosureConfig::from_toml_path(Path::new(
        "examples/soma_kis_outcome_link_close.toml",
    ))
    .expect("load outcome config");
    let report = KISOutcomeLinkClosureRunner::default()
        .run(&config)
        .expect("run outcome closure");
    assert!(report.generated_outcome_links > 0);
    assert!(report.complete_kis_rows > 0);
}

#[test]
fn kis_activation_local_import_example_runs() {
    let config = KISMarketDataActivationConfig::from_toml_path(Path::new(
        "examples/soma_kis_market_data_activate_local_import.toml",
    ))
    .expect("load activation config");
    let bundle = KISOfficialMarketDataActivationRunner::default()
        .run(&config)
        .expect("run activation");
    assert_eq!(bundle.activation_report.added_kis_canonical_csvs, 2);
    assert_eq!(bundle.activation_report.added_kis_official_rows, 12);
    assert_eq!(
        bundle.activation_report.added_kis_official_ready_candles,
        12
    );
    assert!(bundle.activation_report.added_complete_kis_rows > 0);
    assert_eq!(
        format!("{:?}", bundle.activation_report.final_status),
        "KISCompleteRowsImproved"
    );
    assert!(
        Path::new(
            "state/experiments/kis_examples/kis-activate-local-import/kis_outcome_link_closure.txt"
        )
        .exists()
    );
    assert!(Path::new(
        "state/experiments/kis_examples/kis-activate-local-import/kis_downstream_rerun_summary.txt"
    )
    .exists());
    assert!(Path::new(
        "state/experiments/kis_examples/kis-activate-local-import/kis_market_data_activation_summary.txt"
    )
    .exists());
}
