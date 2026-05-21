use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum KISEndpointCategory {
    DomesticStockBasicQuote,
    DomesticStockPeriodPrice,
    DomesticStockMinutePrice,
    DomesticStockOrderbookQuote,
    DomesticStockRealtimeQuote,
    DomesticStockSymbolInfo,
    OverseasStockBasicQuote,
    OverseasStockPeriodPrice,
    OverseasStockRealtimeQuote,
    OverseasStockSymbolInfo,
    OAuthToken,
    WebSocketApproval,
    DomesticOrder,
    OverseasOrder,
    DomesticAccount,
    OverseasAccount,
    Balance,
    Position,
    Holdings,
    BuyingPower,
    OrderableQuantity,
    OrderCorrectionCancel,
    ExecutionNotification,
    RealizedPnl,
    AccountAssetInquiry,
    Unknown,
}

impl KISEndpointCategory {
    pub fn is_broker_surface(self) -> bool {
        matches!(
            self,
            Self::DomesticOrder
                | Self::OverseasOrder
                | Self::DomesticAccount
                | Self::OverseasAccount
                | Self::Balance
                | Self::Position
                | Self::Holdings
                | Self::BuyingPower
                | Self::OrderableQuantity
                | Self::OrderCorrectionCancel
                | Self::ExecutionNotification
                | Self::RealizedPnl
                | Self::AccountAssetInquiry
        )
    }

    pub fn requires_websocket_approval(self) -> bool {
        matches!(
            self,
            Self::DomesticStockRealtimeQuote | Self::OverseasStockRealtimeQuote
        )
    }

    pub fn is_market_data_allowed_by_default(self) -> bool {
        matches!(
            self,
            Self::DomesticStockBasicQuote
                | Self::DomesticStockPeriodPrice
                | Self::DomesticStockMinutePrice
                | Self::DomesticStockOrderbookQuote
                | Self::DomesticStockRealtimeQuote
                | Self::DomesticStockSymbolInfo
                | Self::OverseasStockBasicQuote
                | Self::OverseasStockPeriodPrice
                | Self::OverseasStockRealtimeQuote
                | Self::OverseasStockSymbolInfo
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KISEndpointPolicy {
    pub policy_id: String,
    pub allowed_market_data_categories: Vec<KISEndpointCategory>,
    pub denied_broker_categories: Vec<KISEndpointCategory>,
    pub oauth_allowed: bool,
    pub websocket_approval_allowed: bool,
    pub strict_unknown_deny: bool,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum KISEndpointPolicyStatus {
    MarketDataOnly,
    UnsafeBrokerEndpointDetected,
    UnknownEndpointBlocked,
    MissingPolicy,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KISEndpointPolicyReport {
    pub policy_id: String,
    pub allowed_count: usize,
    pub denied_count: usize,
    pub unknown_count: usize,
    pub broker_surface_detected: bool,
    pub unsafe_endpoint_detected: bool,
    pub policy_status: KISEndpointPolicyStatus,
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for KISEndpointPolicy {
    fn default() -> Self {
        Self {
            policy_id: "kis_market_data_only_policy".to_string(),
            allowed_market_data_categories: vec![
                KISEndpointCategory::DomesticStockBasicQuote,
                KISEndpointCategory::DomesticStockMinutePrice,
                KISEndpointCategory::DomesticStockOrderbookQuote,
                KISEndpointCategory::DomesticStockPeriodPrice,
                KISEndpointCategory::DomesticStockRealtimeQuote,
                KISEndpointCategory::DomesticStockSymbolInfo,
                KISEndpointCategory::OverseasStockBasicQuote,
                KISEndpointCategory::OverseasStockPeriodPrice,
                KISEndpointCategory::OverseasStockRealtimeQuote,
                KISEndpointCategory::OverseasStockSymbolInfo,
            ],
            denied_broker_categories: denied_categories(),
            oauth_allowed: true,
            websocket_approval_allowed: true,
            strict_unknown_deny: true,
            reason_codes: vec![
                ReasonCode::KISEndpointPolicyBuilt,
                ReasonCode::DeniedByDefault,
            ],
        }
    }
}

impl KISEndpointPolicy {
    pub fn from_toml_str(input: &str) -> Result<Self, String> {
        toml::from_str(input).map_err(|err| err.to_string())
    }

    pub fn from_toml_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        Self::from_toml_str(&text)
    }

    pub fn to_toml_string(&self) -> Result<String, String> {
        toml::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.policy_id.trim().is_empty() {
            return Err("kis endpoint policy_id must not be empty".to_string());
        }
        Ok(())
    }

    pub fn is_allowed(&self, category: KISEndpointCategory) -> bool {
        if category == KISEndpointCategory::OAuthToken {
            return self.oauth_allowed;
        }
        if category == KISEndpointCategory::WebSocketApproval {
            return self.websocket_approval_allowed;
        }
        if category == KISEndpointCategory::Unknown {
            return !self.strict_unknown_deny;
        }
        self.allowed_market_data_categories.contains(&category) && !category.is_broker_surface()
    }

    pub fn is_denied(&self, category: KISEndpointCategory) -> bool {
        if category == KISEndpointCategory::Unknown {
            return self.strict_unknown_deny;
        }
        if category.is_broker_surface() {
            return true;
        }
        !self.is_allowed(category)
    }

    pub fn report_for_categories(
        &self,
        categories: &[KISEndpointCategory],
    ) -> KISEndpointPolicyReport {
        if categories.is_empty() {
            return KISEndpointPolicyReport {
                policy_id: self.policy_id.clone(),
                allowed_count: 0,
                denied_count: 0,
                unknown_count: 0,
                broker_surface_detected: false,
                unsafe_endpoint_detected: false,
                policy_status: KISEndpointPolicyStatus::DiagnosticOnly,
                reason_codes: vec![ReasonCode::KISEndpointPolicyBuilt],
            };
        }
        let allowed_count = categories
            .iter()
            .filter(|category| self.is_allowed(**category))
            .count();
        let denied_count = categories
            .iter()
            .filter(|category| self.is_denied(**category))
            .count();
        let unknown_count = categories
            .iter()
            .filter(|category| **category == KISEndpointCategory::Unknown)
            .count();
        let broker_surface_detected = categories
            .iter()
            .any(|category| category.is_broker_surface());
        let unsafe_endpoint_detected = broker_surface_detected || unknown_count > 0;
        let mut reason_codes = vec![ReasonCode::KISEndpointPolicyBuilt];
        if broker_surface_detected {
            reason_codes.push(ReasonCode::KISBrokerEndpointDetected);
            reason_codes.push(ReasonCode::KISEndpointDenied);
        }
        if unknown_count > 0 {
            reason_codes.push(ReasonCode::DeniedByDefault);
        }
        let policy_status = if broker_surface_detected {
            KISEndpointPolicyStatus::UnsafeBrokerEndpointDetected
        } else if unknown_count > 0 {
            KISEndpointPolicyStatus::UnknownEndpointBlocked
        } else {
            KISEndpointPolicyStatus::MarketDataOnly
        };
        KISEndpointPolicyReport {
            policy_id: self.policy_id.clone(),
            allowed_count,
            denied_count,
            unknown_count,
            broker_surface_detected,
            unsafe_endpoint_detected,
            policy_status,
            reason_codes: stable_reason_codes(&reason_codes),
        }
    }

    pub fn default_report(&self) -> KISEndpointPolicyReport {
        let categories = self
            .allowed_market_data_categories
            .iter()
            .copied()
            .chain([
                KISEndpointCategory::OAuthToken,
                KISEndpointCategory::WebSocketApproval,
            ])
            .chain(self.denied_broker_categories.iter().copied())
            .collect::<Vec<_>>();
        self.report_for_categories(&categories)
    }
}

impl KISEndpointPolicyReport {
    pub fn to_text(&self) -> String {
        [
            "research_only_warning=kis endpoint policy remains market-data-only".to_string(),
            "broker_order_account_warning=all broker order account balance and execution surfaces are denied".to_string(),
            format!("policy_id={}", self.policy_id),
            format!("allowed_count={}", self.allowed_count),
            format!("denied_count={}", self.denied_count),
            format!("unknown_count={}", self.unknown_count),
            format!("broker_surface_detected={}", self.broker_surface_detected),
            format!("unsafe_endpoint_detected={}", self.unsafe_endpoint_detected),
            format!("policy_status={:?}", self.policy_status),
            format!(
                "reason_codes={}",
                self.reason_codes
                    .iter()
                    .map(|reason| format!("{reason:?}"))
                    .collect::<Vec<_>>()
                    .join("|")
            ),
        ]
        .join("\n")
    }

    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        let text_path = output_dir.join("kis_endpoint_policy.txt");
        fs::write(&text_path, self.to_text()).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("kis_endpoint_policy.json"),
            self.to_json_string()?,
        )
        .map_err(|err| err.to_string())?;
        Ok(text_path)
    }
}

fn denied_categories() -> Vec<KISEndpointCategory> {
    vec![
        KISEndpointCategory::DomesticOrder,
        KISEndpointCategory::OverseasOrder,
        KISEndpointCategory::DomesticAccount,
        KISEndpointCategory::OverseasAccount,
        KISEndpointCategory::Balance,
        KISEndpointCategory::Position,
        KISEndpointCategory::Holdings,
        KISEndpointCategory::BuyingPower,
        KISEndpointCategory::OrderableQuantity,
        KISEndpointCategory::OrderCorrectionCancel,
        KISEndpointCategory::ExecutionNotification,
        KISEndpointCategory::RealizedPnl,
        KISEndpointCategory::AccountAssetInquiry,
    ]
}
