use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;

use super::kis_activation_storage::KISActivationStorageReport;
use super::kis_auth_readiness::KISAuthReadinessReport;
use super::kis_candle_sufficiency::KISCandleSufficiencyReport;
use super::kis_canonical_batch_validation::KISCanonicalBatchValidationReport;
use super::kis_collection_batch::KISCollectionBatchPlan;
use super::kis_downstream_rerun::KISDownstreamRerunSummary;
use super::kis_endpoint_policy::KISEndpointPolicyReport;
use super::kis_krx_migration::KISKRXMigrationReport;
use super::kis_market_data_activation::KISOfficialMarketDataActivationReport;
use super::kis_operator_actions::KISOperatorAction;
use super::kis_outcome_link_closure::KISOutcomeLinkClosureReport;
use super::kis_raw_archive::KISRawResponseArchiveSummary;
use super::kis_schema_drift::KISResponseSchemaDriftReport;
use super::kis_symbol_whitelist::KISSymbolWhitelist;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct KISOfficialMarketDataActivationBundle {
    pub auth_readiness_report: KISAuthReadinessReport,
    pub endpoint_policy_report: KISEndpointPolicyReport,
    pub symbol_whitelist: KISSymbolWhitelist,
    pub provider_migration_report: KISKRXMigrationReport,
    pub collection_batch_plan: KISCollectionBatchPlan,
    #[serde(default)]
    pub raw_response_archive_summary: Option<KISRawResponseArchiveSummary>,
    #[serde(default)]
    pub schema_drift_report: Option<KISResponseSchemaDriftReport>,
    pub canonical_batch_validation_report: KISCanonicalBatchValidationReport,
    pub candle_sufficiency_report: KISCandleSufficiencyReport,
    #[serde(default)]
    pub outcome_link_closure_report: Option<KISOutcomeLinkClosureReport>,
    pub downstream_rerun_summary: KISDownstreamRerunSummary,
    pub operator_actions: Vec<KISOperatorAction>,
    pub activation_report: KISOfficialMarketDataActivationReport,
    pub storage_report: KISActivationStorageReport,
    pub final_summary: String,
    pub reason_codes: Vec<ReasonCode>,
}
