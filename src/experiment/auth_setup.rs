use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;
use crate::data::ProviderKind;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthSetupGuide {
    pub provider_kind: ProviderKind,
    pub required_env_vars: Vec<String>,
    pub endpoint_template_requirements: Vec<String>,
    pub setup_steps: Vec<String>,
    pub docs_links_label_only: Vec<String>,
    pub warnings: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

impl AuthSetupGuide {
    pub fn to_text(&self) -> String {
        [
            format!("provider={:?}", self.provider_kind),
            format!("required_env_vars={}", self.required_env_vars.join("|")),
            format!(
                "endpoint_template_requirements={}",
                self.endpoint_template_requirements.join("|")
            ),
            format!("setup_steps={}", self.setup_steps.join(" | ")),
            format!(
                "docs_links_label_only={}",
                self.docs_links_label_only.join("|")
            ),
            format!("warnings={}", self.warnings.join(" | ")),
        ]
        .join("\n")
    }
}

pub fn build_auth_setup_guide(provider_kind: ProviderKind) -> AuthSetupGuide {
    let (
        required_env_vars,
        endpoint_template_requirements,
        setup_steps,
        docs_links_label_only,
        warnings,
    ) = match provider_kind {
        ProviderKind::KrxOpenApi => (
            vec!["KRX_API_KEY".to_string()],
            vec!["KRX_ENDPOINT_TEMPLATE".to_string()],
            vec![
                "Create a research-only KRX market-data app registration.".to_string(),
                "Set KRX_API_KEY in the local environment.".to_string(),
                "Set KRX_ENDPOINT_TEMPLATE in the local environment.".to_string(),
                "Re-run provider-auth-check before collection.".to_string(),
            ],
            vec!["KRX Open API developer console".to_string()],
            vec!["Do not print or store raw key material.".to_string()],
        ),
        ProviderKind::AlphaVantage => (
            vec!["ALPHAVANTAGE_API_KEY".to_string()],
            vec![],
            vec![
                "Create a compact research-only AlphaVantage key.".to_string(),
                "Set ALPHAVANTAGE_API_KEY in the local environment.".to_string(),
                "Keep collection bounded to compact daily or small intraday windows.".to_string(),
            ],
            vec!["AlphaVantage account settings".to_string()],
            vec!["No secret value should appear in reports.".to_string()],
        ),
        ProviderKind::Alpaca => (
            vec![
                "ALPACA_API_KEY_ID".to_string(),
                "ALPACA_API_SECRET_KEY".to_string(),
            ],
            vec![],
            vec![
                "Provision a market-data-only Alpaca paper key if this path is enabled later."
                    .to_string(),
                "Set ALPACA_API_KEY_ID and ALPACA_API_SECRET_KEY locally.".to_string(),
                "Keep Alpaca deferred unless the provider path is explicitly enabled.".to_string(),
            ],
            vec!["Alpaca developer dashboard".to_string()],
            vec!["Sprint 25 keeps Alpaca optional/deferred.".to_string()],
        ),
        ProviderKind::DataGoKrFscStockPrice => (
            vec!["DATA_GO_KR_SERVICE_KEY".to_string()],
            vec![],
            vec![
                "Provision a research-only data.go.kr service key.".to_string(),
                "Set DATA_GO_KR_SERVICE_KEY in the local environment.".to_string(),
                "Confirm the approved FSC stock-price endpoint profile before collection."
                    .to_string(),
            ],
            vec!["data.go.kr API portal".to_string()],
            vec!["Do not print or persist service-key values.".to_string()],
        ),
        ProviderKind::KoreaInvestmentMarketData => (
            vec!["KIS_APP_KEY".to_string(), "KIS_APP_SECRET".to_string()],
            vec![],
            vec![
                "Provision a market-data-only KIS app.".to_string(),
                "Set KIS_APP_KEY and KIS_APP_SECRET in the local environment.".to_string(),
                "Optionally set KIS_BASE_URL when a non-default base URL is required.".to_string(),
                "Do not use order or account endpoints.".to_string(),
            ],
            vec!["KIS developer portal".to_string()],
            vec!["Sprint 29 allows market-data-only request stubs, not trading flows.".to_string()],
        ),
        ProviderKind::Upbit => (
            vec![],
            vec![],
            vec!["Upbit public candles do not require auth for Sprint 25.".to_string()],
            vec!["Upbit market-data docs".to_string()],
            vec![],
        ),
        _ => (
            vec![],
            vec![],
            vec!["No auth guide defined for this provider in Sprint 25.".to_string()],
            vec![],
            vec![],
        ),
    };

    AuthSetupGuide {
        provider_kind,
        required_env_vars,
        endpoint_template_requirements,
        setup_steps,
        docs_links_label_only,
        warnings,
        reason_codes: vec![ReasonCode::AuthSetupGuideBuilt],
    }
}
