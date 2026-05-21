use soma_zero::{
    ProviderCredentialStatus, ProviderCredentialStatusKind, ProviderKind, ProviderMarket,
    ProviderSelectionResultStatus, build_default_provider_catalog,
    default_provider_selection_policies, select_provider,
};

fn status(
    provider_kind: ProviderKind,
    status: ProviderCredentialStatusKind,
) -> ProviderCredentialStatus {
    ProviderCredentialStatus {
        provider_kind,
        required_env_vars: Vec::new(),
        optional_env_vars: Vec::new(),
        endpoint_template_env_vars: Vec::new(),
        missing_required_env_vars: Vec::new(),
        missing_endpoint_template_env_vars: Vec::new(),
        status,
        reason_codes: Vec::new(),
    }
}

fn policy(market: ProviderMarket) -> soma_zero::ProviderSelectionPolicy {
    default_provider_selection_policies()
        .into_iter()
        .find(|policy| policy.market == market)
        .expect("policy")
}

#[test]
fn korean_equity_selects_krx_when_ready() {
    let result = select_provider(
        &build_default_provider_catalog(),
        &[
            status(
                ProviderKind::KrxOpenApi,
                ProviderCredentialStatusKind::Ready,
            ),
            status(
                ProviderKind::DataGoKrFscStockPrice,
                ProviderCredentialStatusKind::MissingAuth,
            ),
        ],
        &policy(ProviderMarket::KoreanEquity),
    );
    assert_eq!(result.status, ProviderSelectionResultStatus::Selected);
    assert_eq!(result.selected_provider, Some(ProviderKind::KrxOpenApi));
}

#[test]
fn korean_equity_falls_back_to_data_go_kr_when_krx_missing() {
    let result = select_provider(
        &build_default_provider_catalog(),
        &[
            status(
                ProviderKind::KrxOpenApi,
                ProviderCredentialStatusKind::MissingAuth,
            ),
            status(
                ProviderKind::DataGoKrFscStockPrice,
                ProviderCredentialStatusKind::Ready,
            ),
            status(
                ProviderKind::KoreaInvestmentMarketData,
                ProviderCredentialStatusKind::MissingAuth,
            ),
        ],
        &policy(ProviderMarket::KoreanEquity),
    );
    assert_eq!(result.status, ProviderSelectionResultStatus::Selected);
    assert_eq!(
        result.selected_provider,
        Some(ProviderKind::DataGoKrFscStockPrice)
    );
}

#[test]
fn korean_equity_can_select_kis_market_data_only() {
    let result = select_provider(
        &build_default_provider_catalog(),
        &[
            status(
                ProviderKind::KrxOpenApi,
                ProviderCredentialStatusKind::MissingAuth,
            ),
            status(
                ProviderKind::DataGoKrFscStockPrice,
                ProviderCredentialStatusKind::MissingAuth,
            ),
            status(
                ProviderKind::KoreaInvestmentMarketData,
                ProviderCredentialStatusKind::Ready,
            ),
        ],
        &policy(ProviderMarket::KoreanEquity),
    );
    assert_eq!(
        result.selected_provider,
        Some(ProviderKind::KoreaInvestmentMarketData)
    );
}

#[test]
fn us_equity_selects_alphavantage_when_ready() {
    let result = select_provider(
        &build_default_provider_catalog(),
        &[status(
            ProviderKind::AlphaVantage,
            ProviderCredentialStatusKind::Ready,
        )],
        &policy(ProviderMarket::USEquity),
    );
    assert_eq!(result.selected_provider, Some(ProviderKind::AlphaVantage));
}

#[test]
fn us_equity_falls_back_to_alpaca_when_alphavantage_missing() {
    let result = select_provider(
        &build_default_provider_catalog(),
        &[
            status(
                ProviderKind::AlphaVantage,
                ProviderCredentialStatusKind::MissingAuth,
            ),
            status(ProviderKind::Alpaca, ProviderCredentialStatusKind::Ready),
        ],
        &policy(ProviderMarket::USEquity),
    );
    assert_eq!(result.selected_provider, Some(ProviderKind::Alpaca));
}

#[test]
fn us_equity_uses_research_only_fallback_status_when_all_official_auth_missing() {
    let result = select_provider(
        &build_default_provider_catalog(),
        &[
            status(
                ProviderKind::AlphaVantage,
                ProviderCredentialStatusKind::MissingAuth,
            ),
            status(
                ProviderKind::Alpaca,
                ProviderCredentialStatusKind::MissingAuth,
            ),
            status(
                ProviderKind::PolygonProfessional,
                ProviderCredentialStatusKind::MissingAuth,
            ),
            status(
                ProviderKind::NasdaqDataLink,
                ProviderCredentialStatusKind::MissingAuth,
            ),
        ],
        &policy(ProviderMarket::USEquity),
    );
    assert_eq!(
        result.status,
        ProviderSelectionResultStatus::ResearchOnlyFallback
    );
    assert_eq!(result.selected_provider, None);
}

#[test]
fn crypto_selects_upbit() {
    let result = select_provider(
        &build_default_provider_catalog(),
        &[status(
            ProviderKind::Upbit,
            ProviderCredentialStatusKind::NotRequired,
        )],
        &policy(ProviderMarket::Crypto),
    );
    assert_eq!(result.selected_provider, Some(ProviderKind::Upbit));
}

#[test]
fn provider_selection_is_deterministic() {
    let statuses = vec![
        status(
            ProviderKind::KrxOpenApi,
            ProviderCredentialStatusKind::MissingAuth,
        ),
        status(
            ProviderKind::DataGoKrFscStockPrice,
            ProviderCredentialStatusKind::Ready,
        ),
        status(
            ProviderKind::KoreaInvestmentMarketData,
            ProviderCredentialStatusKind::Ready,
        ),
    ];
    let first = select_provider(
        &build_default_provider_catalog(),
        &statuses,
        &policy(ProviderMarket::KoreanEquity),
    );
    let second = select_provider(
        &build_default_provider_catalog(),
        &statuses,
        &policy(ProviderMarket::KoreanEquity),
    );
    assert_eq!(first, second);
}
