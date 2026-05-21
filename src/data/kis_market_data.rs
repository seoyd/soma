use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;

use super::ProviderKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum KisMarketEndpoint {
    DailyItemChartPrice,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KisMarketDataRequest {
    pub provider_kind: ProviderKind,
    pub endpoint: KisMarketEndpoint,
    pub path: String,
    pub symbol: String,
    pub market_code: String,
    pub start_date: String,
    pub end_date: String,
    pub timeframe: String,
    pub required_env_vars: Vec<String>,
    pub optional_env_vars: Vec<String>,
    pub supports_order: bool,
    pub supports_account: bool,
    pub reason_codes: Vec<ReasonCode>,
}

pub fn build_kis_daily_chart_request(
    symbol: &str,
    market_code: &str,
    start_date: &str,
    end_date: &str,
) -> KisMarketDataRequest {
    KisMarketDataRequest {
        provider_kind: ProviderKind::KoreaInvestmentMarketData,
        endpoint: KisMarketEndpoint::DailyItemChartPrice,
        path: "/uapi/domestic-stock/v1/quotations/inquire-daily-itemchartprice".to_string(),
        symbol: symbol.to_string(),
        market_code: market_code.to_string(),
        start_date: start_date.to_string(),
        end_date: end_date.to_string(),
        timeframe: "1d".to_string(),
        required_env_vars: vec!["KIS_APP_KEY".to_string(), "KIS_APP_SECRET".to_string()],
        optional_env_vars: vec!["KIS_BASE_URL".to_string()],
        supports_order: false,
        supports_account: false,
        reason_codes: vec![
            ReasonCode::KisMarketDataStubBuilt,
            ReasonCode::DeterministicPath,
        ],
    }
}
