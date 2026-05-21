use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;
use crate::security::SecretRedactionAuditReport;
use crate::ui::ControlTowerAutoRefreshReport;

use super::environment_isolation::EnvironmentIsolationReport;
use super::kis_auth_closure::KISAuthClosureReport;
use super::kis_collection_plan_v2::KISCollectionPlanV2;
use super::kis_market_data_dry_run::KISMarketDataDryRunReport;
use super::kis_market_data_smoke::KISMarketDataEvidenceSmokeReport;
use super::operational_runbook_v2::OperationalRunbookV2Report;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct KISMarketDataSmokeControlTowerBundle {
    pub auth_closure_report: KISAuthClosureReport,
    pub dry_run_report: KISMarketDataDryRunReport,
    pub collection_plan_v2: KISCollectionPlanV2,
    pub market_data_smoke_report: KISMarketDataEvidenceSmokeReport,
    pub environment_isolation_report: EnvironmentIsolationReport,
    pub secret_redaction_audit_report: SecretRedactionAuditReport,
    pub control_tower_auto_refresh_report: ControlTowerAutoRefreshReport,
    pub operational_runbook_v2_report: OperationalRunbookV2Report,
    pub storage_report: String,
    pub final_summary: String,
    pub reason_codes: Vec<ReasonCode>,
}
