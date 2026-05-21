use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};

use super::kis_auth_readiness::KISAuthReadinessReport;
use super::kis_endpoint_policy::{KISEndpointPolicyReport, KISEndpointPolicyStatus};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderMigrationDecision {
    KeepKRXPrimary,
    SwitchKISToPrimary,
    KISPrimaryKRXReference,
    KISBlockedKeepKRX,
    NeedMoreProviderEvidence,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct KISKRXMigrationReport {
    pub current_korean_equity_primary: String,
    pub proposed_korean_equity_primary: String,
    pub krx_status_summary: String,
    pub kis_status_summary: String,
    pub endpoint_complexity_score: u32,
    pub auth_complexity_score: u32,
    #[serde(default)]
    pub data_quality_score: Option<f64>,
    #[serde(default)]
    pub outcome_link_depth_delta: Option<i64>,
    #[serde(default)]
    pub coverage_delta: Option<i64>,
    pub migration_decision: ProviderMigrationDecision,
    pub retained_fallbacks: Vec<String>,
    pub warnings: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

impl KISKRXMigrationReport {
    pub fn build(
        auth: &KISAuthReadinessReport,
        endpoint_policy: &KISEndpointPolicyReport,
        data_quality_score: Option<f64>,
        outcome_link_depth_delta: Option<i64>,
        coverage_delta: Option<i64>,
    ) -> Self {
        let endpoint_complexity_score =
            endpoint_policy.denied_count as u32 + endpoint_policy.unknown_count as u32;
        let auth_complexity_score = u32::from(!auth.app_key_present)
            + u32::from(!auth.app_secret_present)
            + u32::from(!auth.base_url_present)
            + u32::from(!auth.websocket_approval_key_present);
        let mut warnings = Vec::new();
        let migration_decision = if endpoint_policy.policy_status
            == KISEndpointPolicyStatus::UnsafeBrokerEndpointDetected
        {
            warnings.push(
                "unsafe broker/account endpoint detected; KIS stays blocked until removed"
                    .to_string(),
            );
            ProviderMigrationDecision::KISBlockedKeepKRX
        } else if !auth.safe_to_collect_rest_market_data {
            warnings.push("KIS REST auth/base-url readiness is incomplete".to_string());
            ProviderMigrationDecision::KISBlockedKeepKRX
        } else if outcome_link_depth_delta.unwrap_or(0) > 0 || coverage_delta.unwrap_or(0) >= 0 {
            ProviderMigrationDecision::KISPrimaryKRXReference
        } else if data_quality_score.unwrap_or(0.0) >= 0.80 {
            ProviderMigrationDecision::SwitchKISToPrimary
        } else {
            ProviderMigrationDecision::NeedMoreProviderEvidence
        };
        let mut reason_codes = vec![
            ReasonCode::KISKRXMigrationBuilt,
            ReasonCode::KRXRetainedAsReference,
        ];
        if matches!(
            migration_decision,
            ProviderMigrationDecision::SwitchKISToPrimary
                | ProviderMigrationDecision::KISPrimaryKRXReference
        ) {
            reason_codes.push(ReasonCode::ProviderPriorityUpdated);
        }
        if matches!(
            migration_decision,
            ProviderMigrationDecision::KISBlockedKeepKRX
        ) {
            reason_codes.push(ReasonCode::KISEndpointDenied);
        }
        Self {
            current_korean_equity_primary: "krx-open-api".to_string(),
            proposed_korean_equity_primary: match migration_decision {
                ProviderMigrationDecision::KeepKRXPrimary
                | ProviderMigrationDecision::KISBlockedKeepKRX
                | ProviderMigrationDecision::NeedMoreProviderEvidence => "krx-open-api".to_string(),
                ProviderMigrationDecision::SwitchKISToPrimary
                | ProviderMigrationDecision::KISPrimaryKRXReference => {
                    "kis-market-data-only".to_string()
                }
            },
            krx_status_summary: "KRX retained as exchange-reference and fallback".to_string(),
            kis_status_summary: format!(
                "rest_ready={};realtime_ready={};endpoint_policy={:?}",
                auth.safe_to_collect_rest_market_data,
                auth.safe_to_collect_realtime_market_data,
                endpoint_policy.policy_status
            ),
            endpoint_complexity_score,
            auth_complexity_score,
            data_quality_score,
            outcome_link_depth_delta,
            coverage_delta,
            migration_decision,
            retained_fallbacks: vec![
                "krx-open-api".to_string(),
                "data-go-kr-fsc-stock-price".to_string(),
            ],
            warnings,
            reason_codes: stable_reason_codes(&reason_codes),
        }
    }

    pub fn to_text(&self) -> String {
        [
            "research_only_warning=kis migration is operational and not a profitability claim".to_string(),
            "market_data_only_warning=krx remains reference/fallback and broker endpoints remain denied".to_string(),
            format!("current_korean_equity_primary={}", self.current_korean_equity_primary),
            format!("proposed_korean_equity_primary={}", self.proposed_korean_equity_primary),
            format!("krx_status_summary={}", self.krx_status_summary),
            format!("kis_status_summary={}", self.kis_status_summary),
            format!("endpoint_complexity_score={}", self.endpoint_complexity_score),
            format!("auth_complexity_score={}", self.auth_complexity_score),
            format!("data_quality_score={}", self.data_quality_score.map(|value| format!("{value:.4}")).unwrap_or_default()),
            format!("outcome_link_depth_delta={}", self.outcome_link_depth_delta.map(|value| value.to_string()).unwrap_or_default()),
            format!("coverage_delta={}", self.coverage_delta.map(|value| value.to_string()).unwrap_or_default()),
            format!("migration_decision={:?}", self.migration_decision),
            format!("retained_fallbacks={}", self.retained_fallbacks.join("|")),
            format!("warnings={}", self.warnings.join(" | ")),
            format!("reason_codes={}", self.reason_codes.iter().map(|reason| format!("{reason:?}")).collect::<Vec<_>>().join("|")),
        ]
        .join("\n")
    }

    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        let text_path = output_dir.join("kis_krx_migration_report.txt");
        fs::write(&text_path, self.to_text()).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("kis_krx_migration_report.json"),
            self.to_json_string()?,
        )
        .map_err(|err| err.to_string())?;
        Ok(text_path)
    }
}
