use soma_zero::{ProviderKind, build_kis_daily_chart_request};

#[test]
fn kis_market_data_request_has_no_order_or_account_capabilities() {
    let request = build_kis_daily_chart_request("005930", "J", "20240101", "20240131");
    assert_eq!(
        request.provider_kind,
        ProviderKind::KoreaInvestmentMarketData
    );
    assert!(!request.supports_order);
    assert!(!request.supports_account);
    assert!(
        request
            .required_env_vars
            .contains(&"KIS_APP_KEY".to_string())
    );
    assert!(
        request
            .required_env_vars
            .contains(&"KIS_APP_SECRET".to_string())
    );
    assert!(
        request
            .optional_env_vars
            .contains(&"KIS_BASE_URL".to_string())
    );
    assert!(!request.path.contains("order"));
    assert!(!request.path.contains("account"));
}

#[test]
fn kis_market_data_request_builder_is_deterministic() {
    let first = build_kis_daily_chart_request("005930", "J", "20240101", "20240131");
    let second = build_kis_daily_chart_request("005930", "J", "20240101", "20240131");
    assert_eq!(first, second);
}
