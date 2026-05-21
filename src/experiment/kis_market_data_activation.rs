use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};

use super::kis_activation_storage::KISActivationStorageReport;
use super::kis_auth_readiness::{KISAuthReadinessReport, KISAuthReadinessStatus};
use super::kis_candle_sufficiency::{KISCandleSufficiencyReport, KISCandleSufficiencyStatus};
use super::kis_canonical_batch_validation::{
    KISCanonicalBatchValidationReport, KISCanonicalBatchValidationStatus,
};
use super::kis_collection_batch::KISCollectionBatchPlan;
use super::kis_downstream_rerun::KISDownstreamRerunSummary;
use super::kis_endpoint_policy::{
    KISEndpointCategory, KISEndpointPolicy, KISEndpointPolicyReport, KISEndpointPolicyStatus,
};
use super::kis_krx_migration::{KISKRXMigrationReport, ProviderMigrationDecision};
use super::kis_market_data_activation_bundle::KISOfficialMarketDataActivationBundle;
use super::kis_operator_actions::{KISOperatorAction, build_kis_operator_actions};
use super::kis_outcome_link_closure::{
    KISOutcomeLinkClosureConfig, KISOutcomeLinkClosureReport, KISOutcomeLinkClosureRunner,
    KISOutcomeLinkClosureStatus,
};
use super::kis_raw_archive::{
    KISRawResponseArchiveRecord, KISRawResponseArchiveSummary, raw_archive_dir,
};
use super::kis_schema_drift::{KISResponseSchemaDriftReport, KISResponseSchemaStatus};
use super::kis_symbol_whitelist::{KISSymbolWhitelist, KISSymbolWhitelistConfig};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KISMarketDataActivationConfig {
    pub activation_id: String,
    #[serde(default)]
    pub provider_readiness_config_path: Option<String>,
    #[serde(default)]
    pub provider_readiness_report_paths: Vec<String>,
    #[serde(default)]
    pub provider_reality_report_paths: Vec<String>,
    #[serde(default)]
    pub kis_collection_plan_paths: Vec<String>,
    #[serde(default)]
    pub local_kis_canonical_csv_paths: Vec<String>,
    #[serde(default)]
    pub local_kis_provenance_paths: Vec<String>,
    #[serde(default)]
    pub local_kis_preflight_paths: Vec<String>,
    #[serde(default)]
    pub domestic_symbol_whitelist_path: Option<String>,
    #[serde(default)]
    pub overseas_symbol_whitelist_path: Option<String>,
    #[serde(default)]
    pub endpoint_policy_path: Option<String>,
    #[serde(default)]
    pub barrier_profile_registry_path: Option<String>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default = "default_max_symbols")]
    pub max_domestic_symbols: usize,
    #[serde(default = "default_max_symbols")]
    pub max_overseas_symbols: usize,
    #[serde(default = "default_max_rows_per_symbol")]
    pub max_rows_per_symbol: usize,
    #[serde(default = "default_max_requests")]
    pub max_requests: usize,
    #[serde(default = "default_max_days")]
    pub max_days: usize,
    #[serde(default = "default_max_bytes")]
    pub max_raw_bytes: usize,
    #[serde(default = "default_max_bytes")]
    pub max_canonical_bytes: usize,
    #[serde(default = "default_max_total_bytes")]
    pub max_total_bytes: usize,
    #[serde(default = "default_true")]
    pub require_kis_app_key: bool,
    #[serde(default = "default_true")]
    pub require_kis_app_secret: bool,
    #[serde(default = "default_true")]
    pub require_kis_base_url: bool,
    #[serde(default = "default_true")]
    pub require_provenance: bool,
    #[serde(default = "default_true")]
    pub require_preflight: bool,
    #[serde(default)]
    pub require_manifest: bool,
    #[serde(default = "default_true")]
    pub run_auth_readiness: bool,
    #[serde(default = "default_true")]
    pub run_endpoint_policy_check: bool,
    #[serde(default = "default_true")]
    pub run_provider_priority_update: bool,
    #[serde(default = "default_true")]
    pub run_collection_dry_run: bool,
    #[serde(default = "default_true")]
    pub run_fixture_replay: bool,
    #[serde(default = "default_true")]
    pub run_local_import: bool,
    #[serde(default)]
    pub run_live_market_data_collection: bool,
    #[serde(default)]
    pub run_preflight: bool,
    #[serde(default = "default_true")]
    pub run_official_replication: bool,
    #[serde(default)]
    pub run_candle_pack: bool,
    #[serde(default = "default_true")]
    pub run_candle_sufficiency: bool,
    #[serde(default = "default_true")]
    pub run_outcome_link_closure: bool,
    #[serde(default)]
    pub run_official_evidence_scaleout: bool,
    #[serde(default)]
    pub run_official_evidence_diversity_sweep: bool,
    #[serde(default)]
    pub run_committee_official_benchmark: bool,
    #[serde(default)]
    pub run_core_performance: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum KISOfficialMarketDataActivationFinalStatus {
    KISMarketDataActivated,
    KISCollectionReady,
    KISAuthMissing,
    KISBaseUrlMissing,
    KISWebSocketApprovalMissing,
    KISEndpointPolicyBlocked,
    KISSchemaBlocked,
    KISPreflightBlocked,
    KISBudgetBlocked,
    KISOfficialRowsImported,
    KISOfficialCandlesImproved,
    KISOutcomeLinksImproved,
    KISCompleteRowsImproved,
    ProviderPriorityUpdated,
    StillMissingOutcomeLinks,
    StillNeedFutureWindows,
    NoImprovement,
    NeedMoreEvidence,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum KISOfficialMarketDataActivationRecommendation {
    SetKISAppKey,
    SetKISAppSecret,
    SetKISBaseUrl,
    SetKISWebSocketApprovalKey,
    RunKISDryRun,
    RunKISBoundedCollection,
    ProvideKISCanonicalCsv,
    RunOutcomeLinkClosure,
    RunKISPreflight,
    CollectLongerKISWindow,
    RunDiversitySweep,
    RunCorePerformance,
    KeepKRXAsReference,
    MoreKISOfficialRows,
    KeepTrinity,
    NeedMoreEvidence,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct KISOfficialMarketDataActivationReport {
    pub activation_id: String,
    #[serde(alias = "auth_readiness_status")]
    pub auth_readiness: KISAuthReadinessStatus,
    pub endpoint_policy_status: KISEndpointPolicyStatus,
    pub provider_migration_decision: ProviderMigrationDecision,
    pub symbol_whitelist_summary: String,
    pub collection_batch_plan_summary: String,
    #[serde(alias = "schema_status")]
    pub schema_drift_status: KISResponseSchemaStatus,
    #[serde(alias = "canonical_validation_status")]
    pub canonical_batch_validation_status: KISCanonicalBatchValidationStatus,
    pub candle_sufficiency_status: KISCandleSufficiencyStatus,
    #[serde(default)]
    pub outcome_link_closure_status: Option<KISOutcomeLinkClosureStatus>,
    pub downstream_rerun_summary: String,
    #[serde(alias = "imported_canonical_csvs")]
    pub added_kis_canonical_csvs: usize,
    #[serde(alias = "official_rows")]
    pub added_kis_official_rows: usize,
    pub added_kis_preflight_ready_rows: usize,
    #[serde(alias = "official_ready_candles")]
    pub added_kis_official_ready_candles: usize,
    #[serde(alias = "outcome_link_rows")]
    pub added_kis_outcome_links: usize,
    pub added_kis_no_trade_counterfactuals: usize,
    pub added_kis_risk_denied_counterfactuals: usize,
    #[serde(alias = "complete_rows")]
    pub added_complete_kis_rows: usize,
    #[serde(default)]
    pub previous_core_status: Option<String>,
    #[serde(default)]
    pub current_core_status: Option<String>,
    #[serde(default)]
    pub previous_primary_bottleneck: Option<String>,
    #[serde(default)]
    pub current_primary_bottleneck: Option<String>,
    pub bottleneck_changed: bool,
    pub operator_action_count: usize,
    pub final_status: KISOfficialMarketDataActivationFinalStatus,
    pub final_recommendation: KISOfficialMarketDataActivationRecommendation,
    pub blockers: Vec<String>,
    pub warnings: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct KISOfficialMarketDataActivationRunner;

impl Default for KISMarketDataActivationConfig {
    fn default() -> Self {
        Self {
            activation_id: "kis_market_data_activation".to_string(),
            provider_readiness_config_path: None,
            provider_readiness_report_paths: Vec::new(),
            provider_reality_report_paths: Vec::new(),
            kis_collection_plan_paths: Vec::new(),
            local_kis_canonical_csv_paths: Vec::new(),
            local_kis_provenance_paths: Vec::new(),
            local_kis_preflight_paths: Vec::new(),
            domestic_symbol_whitelist_path: None,
            overseas_symbol_whitelist_path: None,
            endpoint_policy_path: None,
            barrier_profile_registry_path: None,
            output_root: default_output_root(),
            max_domestic_symbols: default_max_symbols(),
            max_overseas_symbols: default_max_symbols(),
            max_rows_per_symbol: default_max_rows_per_symbol(),
            max_requests: default_max_requests(),
            max_days: default_max_days(),
            max_raw_bytes: default_max_bytes(),
            max_canonical_bytes: default_max_bytes(),
            max_total_bytes: default_max_total_bytes(),
            require_kis_app_key: true,
            require_kis_app_secret: true,
            require_kis_base_url: true,
            require_provenance: true,
            require_preflight: true,
            require_manifest: false,
            run_auth_readiness: true,
            run_endpoint_policy_check: true,
            run_provider_priority_update: true,
            run_collection_dry_run: true,
            run_fixture_replay: true,
            run_local_import: true,
            run_live_market_data_collection: false,
            run_preflight: false,
            run_official_replication: true,
            run_candle_pack: false,
            run_candle_sufficiency: true,
            run_outcome_link_closure: true,
            run_official_evidence_scaleout: false,
            run_official_evidence_diversity_sweep: false,
            run_committee_official_benchmark: false,
            run_core_performance: false,
            reason_codes: vec![
                ReasonCode::DeterministicPath,
                ReasonCode::KISLocalImportPreferred,
                ReasonCode::KISCollectionDisabledByDefault,
            ],
        }
    }
}

impl KISMarketDataActivationConfig {
    pub fn from_toml_str(input: &str) -> Result<Self, String> {
        toml::from_str(input).map_err(|err| err.to_string())
    }

    pub fn from_toml_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        Self::from_toml_str(&text)
    }

    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.activation_id)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.activation_id.trim().is_empty() {
            return Err("kis activation_id must not be empty".to_string());
        }
        for path in [
            Some(self.output_root.as_str()),
            self.domestic_symbol_whitelist_path.as_deref(),
            self.overseas_symbol_whitelist_path.as_deref(),
            self.endpoint_policy_path.as_deref(),
            self.barrier_profile_registry_path.as_deref(),
        ]
        .into_iter()
        .flatten()
        .chain(
            self.local_kis_canonical_csv_paths
                .iter()
                .map(String::as_str),
        )
        .chain(self.local_kis_provenance_paths.iter().map(String::as_str))
        .chain(self.local_kis_preflight_paths.iter().map(String::as_str))
        {
            if path.contains("://") {
                return Err("kis activation paths must be local".to_string());
            }
        }
        Ok(())
    }

    pub fn requested_endpoint_categories(&self) -> Vec<KISEndpointCategory> {
        let mut categories = vec![
            KISEndpointCategory::OAuthToken,
            KISEndpointCategory::DomesticStockPeriodPrice,
            KISEndpointCategory::OverseasStockPeriodPrice,
        ];
        if self.run_live_market_data_collection {
            categories.push(KISEndpointCategory::WebSocketApproval);
            categories.push(KISEndpointCategory::DomesticStockRealtimeQuote);
            categories.push(KISEndpointCategory::OverseasStockRealtimeQuote);
        }
        categories.sort();
        categories.dedup();
        categories
    }
}

impl KISOfficialMarketDataActivationRunner {
    pub fn run(
        &self,
        config: &KISMarketDataActivationConfig,
    ) -> Result<KISOfficialMarketDataActivationBundle, String> {
        config.validate()?;
        let output_dir = config.output_dir();
        fs::create_dir_all(&output_dir).map_err(|err| err.to_string())?;

        let auth = KISAuthReadinessReport::from_config(config);
        let endpoint_policy = load_endpoint_policy(config)?;
        let endpoint_policy_report =
            endpoint_policy.report_for_categories(&config.requested_endpoint_categories());
        let whitelist = load_symbol_whitelist(config)?;
        let collection_plan = KISCollectionBatchPlan::build(
            config,
            &auth,
            &endpoint_policy,
            &endpoint_policy_report,
            &whitelist,
        );
        let raw_archive_summary = build_raw_archive_summary(&output_dir, &collection_plan)?;
        let schema_drift_report = KISResponseSchemaDriftReport::from_archive(
            raw_archive_summary.as_ref(),
            &format!("{}-schema", config.activation_id),
        );
        let canonical_batch_validation_report = KISCanonicalBatchValidationReport::build(
            &format!("{}-canonical", config.activation_id),
            &config.local_kis_canonical_csv_paths,
            config.require_provenance,
            config.require_preflight,
            config.require_manifest,
        );
        let candle_sufficiency_report = KISCandleSufficiencyReport::from_batch_validation(
            &canonical_batch_validation_report,
            &format!("{}-candles", config.activation_id),
            config.barrier_profile_registry_path.as_deref(),
        );

        auth.write_to_dir(&output_dir)?;
        endpoint_policy_report.write_to_dir(&output_dir)?;
        whitelist.write_to_dir(&output_dir)?;
        collection_plan.write_to_dir(&output_dir)?;
        if let Some(summary) = &raw_archive_summary {
            summary.write_to_dir(&output_dir)?;
        }
        schema_drift_report.write_to_dir(&output_dir)?;
        canonical_batch_validation_report.write_to_dir(&output_dir)?;
        candle_sufficiency_report.write_to_dir(&output_dir)?;

        let outcome_link_closure_report = if config.run_outcome_link_closure {
            let outcome_config = KISOutcomeLinkClosureConfig {
                closure_id: format!("{}-outcome", config.activation_id),
                kis_activation_report_paths: Vec::new(),
                kis_canonical_csv_paths: config.local_kis_canonical_csv_paths.clone(),
                kis_candle_sufficiency_paths: vec![
                    output_dir
                        .join("kis_candle_sufficiency.json")
                        .display()
                        .to_string(),
                ],
                official_ready_rows_paths: Vec::new(),
                barrier_profile_registry_path: config.barrier_profile_registry_path.clone(),
                output_root: config.output_root.clone(),
                run_future_window_requirements: true,
                run_outcome_linkage_v3: true,
                run_counterfactual_completion_v2: true,
                run_complete_row_close_v2: true,
                run_official_evidence_scaleout: config.run_official_evidence_scaleout,
                run_official_evidence_diversity_sweep: config.run_official_evidence_diversity_sweep,
                run_core_performance: config.run_core_performance,
                reason_codes: vec![ReasonCode::KISOutcomeLinkClosureBuilt],
            };
            let report = KISOutcomeLinkClosureRunner::default().run(&outcome_config)?;
            report.write_to_dir(&output_dir)?;
            Some(report)
        } else {
            None
        };

        let provider_migration_report = KISKRXMigrationReport::build(
            &auth,
            &endpoint_policy_report,
            average_quality_score(&canonical_batch_validation_report.validation_reports),
            outcome_link_closure_report
                .as_ref()
                .map(|report| report.generated_outcome_links as i64),
            Some(
                official_ready_candle_rows(&candle_sufficiency_report) as i64
                    - candle_sufficiency_report.total_series as i64,
            ),
        );
        provider_migration_report.write_to_dir(&output_dir)?;

        let mut downstream_rerun_summary = KISDownstreamRerunSummary {
            official_replication_ran: config.run_official_replication,
            candle_pack_ran: config.run_candle_pack,
            candle_sufficiency_ran: config.run_candle_sufficiency,
            outcome_link_closure_ran: outcome_link_closure_report.is_some(),
            complete_row_close_v2_ran: outcome_link_closure_report.is_some(),
            official_rows_after: Some(official_row_count(&canonical_batch_validation_report)),
            official_ready_candles_after: Some(official_ready_candle_rows(
                &candle_sufficiency_report,
            )),
            outcome_links_after: outcome_link_closure_report
                .as_ref()
                .map(|report| report.generated_outcome_links),
            counterfactuals_after: outcome_link_closure_report.as_ref().map(|report| {
                report.generated_no_trade_counterfactuals
                    + report.generated_risk_denied_counterfactuals
            }),
            core_status_after: Some(
                if config.run_core_performance {
                    "core-performance-enabled"
                } else {
                    "core-performance-deferred"
                }
                .to_string(),
            ),
            primary_bottleneck_after: Some(primary_bottleneck(
                &auth,
                &endpoint_policy_report,
                &canonical_batch_validation_report,
                &candle_sufficiency_report,
                outcome_link_closure_report.as_ref(),
            )),
            ..KISDownstreamRerunSummary::default()
        };
        downstream_rerun_summary.finalize(&[
            ReasonCode::KISDownstreamRerunBuilt,
            ReasonCode::KRXRetainedAsReference,
        ]);
        downstream_rerun_summary.write_to_dir(&output_dir)?;

        let operator_actions = build_kis_operator_actions(
            config,
            &auth,
            &endpoint_policy_report,
            &whitelist,
            &canonical_batch_validation_report.validation_reports,
            false,
        );
        write_operator_actions(&output_dir, &operator_actions)?;

        let activation_report = build_activation_report(
            config,
            &auth,
            &endpoint_policy_report,
            &whitelist,
            &collection_plan,
            &schema_drift_report,
            &canonical_batch_validation_report,
            &candle_sufficiency_report,
            outcome_link_closure_report.as_ref(),
            &downstream_rerun_summary,
            &provider_migration_report,
            operator_actions.len(),
            config.max_total_bytes,
        );
        activation_report.write_to_dir(&output_dir)?;

        let final_summary = format!(
            "status={:?};recommendation={:?};migration={:?}",
            activation_report.final_status,
            activation_report.final_recommendation,
            provider_migration_report.migration_decision
        );
        fs::write(
            output_dir.join("kis_market_data_activation_summary.txt"),
            &final_summary,
        )
        .map_err(|err| err.to_string())?;

        let storage_report = KISActivationStorageReport::build(
            &raw_paths(raw_archive_summary.as_ref()),
            &config.local_kis_canonical_csv_paths,
            &config.local_kis_provenance_paths,
            &Vec::new(),
            &config.local_kis_preflight_paths,
            &Vec::new(),
            &Vec::new(),
            &Vec::new(),
            &Vec::new(),
            &report_paths(&output_dir),
            config.max_total_bytes,
        );
        storage_report.write_to_dir(&output_dir)?;

        let reason_codes = stable_reason_codes(
            &[
                config.reason_codes.clone(),
                activation_report.reason_codes.clone(),
                storage_report.reason_codes.clone(),
                provider_migration_report.reason_codes.clone(),
            ]
            .concat(),
        );
        Ok(KISOfficialMarketDataActivationBundle {
            auth_readiness_report: auth,
            endpoint_policy_report,
            symbol_whitelist: whitelist,
            provider_migration_report,
            collection_batch_plan: collection_plan,
            raw_response_archive_summary: raw_archive_summary,
            schema_drift_report: Some(schema_drift_report),
            canonical_batch_validation_report,
            candle_sufficiency_report,
            outcome_link_closure_report,
            downstream_rerun_summary,
            operator_actions,
            activation_report,
            storage_report,
            final_summary,
            reason_codes,
        })
    }
}

impl KISOfficialMarketDataActivationReport {
    pub fn to_text(&self) -> String {
        [
            format!("activation_id={}", self.activation_id),
            format!("auth_readiness={:?}", self.auth_readiness),
            format!("endpoint_policy_status={:?}", self.endpoint_policy_status),
            format!(
                "provider_migration_decision={:?}",
                self.provider_migration_decision
            ),
            format!("symbol_whitelist_summary={}", self.symbol_whitelist_summary),
            format!(
                "collection_batch_plan_summary={}",
                self.collection_batch_plan_summary
            ),
            format!("schema_drift_status={:?}", self.schema_drift_status),
            format!(
                "canonical_batch_validation_status={:?}",
                self.canonical_batch_validation_status
            ),
            format!(
                "candle_sufficiency_status={:?}",
                self.candle_sufficiency_status
            ),
            format!(
                "outcome_link_closure_status={}",
                self.outcome_link_closure_status
                    .map(|value| format!("{value:?}"))
                    .unwrap_or_default()
            ),
            format!("downstream_rerun_summary={}", self.downstream_rerun_summary),
            format!("added_kis_canonical_csvs={}", self.added_kis_canonical_csvs),
            format!("added_kis_official_rows={}", self.added_kis_official_rows),
            format!(
                "added_kis_preflight_ready_rows={}",
                self.added_kis_preflight_ready_rows
            ),
            format!(
                "added_kis_official_ready_candles={}",
                self.added_kis_official_ready_candles
            ),
            format!("added_kis_outcome_links={}", self.added_kis_outcome_links),
            format!(
                "added_kis_no_trade_counterfactuals={}",
                self.added_kis_no_trade_counterfactuals
            ),
            format!(
                "added_kis_risk_denied_counterfactuals={}",
                self.added_kis_risk_denied_counterfactuals
            ),
            format!("added_complete_kis_rows={}", self.added_complete_kis_rows),
            format!(
                "previous_core_status={}",
                self.previous_core_status.clone().unwrap_or_default()
            ),
            format!(
                "current_core_status={}",
                self.current_core_status.clone().unwrap_or_default()
            ),
            format!(
                "previous_primary_bottleneck={}",
                self.previous_primary_bottleneck.clone().unwrap_or_default()
            ),
            format!(
                "current_primary_bottleneck={}",
                self.current_primary_bottleneck.clone().unwrap_or_default()
            ),
            format!("bottleneck_changed={}", self.bottleneck_changed),
            format!("operator_action_count={}", self.operator_action_count),
            format!("final_status={:?}", self.final_status),
            format!("final_recommendation={:?}", self.final_recommendation),
            format!("blockers={}", self.blockers.join(" | ")),
            format!("warnings={}", self.warnings.join(" | ")),
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

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<(), String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("kis_market_data_activation_report.txt"),
            self.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("kis_market_data_activation_report.json"),
            self.to_json_string()?,
        )
        .map_err(|err| err.to_string())?;
        Ok(())
    }
}

fn build_activation_report(
    config: &KISMarketDataActivationConfig,
    auth: &KISAuthReadinessReport,
    endpoint: &KISEndpointPolicyReport,
    whitelist: &KISSymbolWhitelist,
    collection_plan: &KISCollectionBatchPlan,
    schema: &KISResponseSchemaDriftReport,
    canonical: &KISCanonicalBatchValidationReport,
    candle: &KISCandleSufficiencyReport,
    outcome: Option<&KISOutcomeLinkClosureReport>,
    downstream: &KISDownstreamRerunSummary,
    migration: &KISKRXMigrationReport,
    operator_action_count: usize,
    max_total_bytes: usize,
) -> KISOfficialMarketDataActivationReport {
    let mut blockers = Vec::new();
    let mut warnings = Vec::new();
    if matches!(
        auth.readiness_status,
        KISAuthReadinessStatus::MissingAppKey
            | KISAuthReadinessStatus::MissingAppSecret
            | KISAuthReadinessStatus::MissingAppKeyAndSecret
    ) {
        blockers.push("KIS REST auth/base-url readiness incomplete".to_string());
    }
    if matches!(
        auth.readiness_status,
        KISAuthReadinessStatus::MissingBaseUrl
    ) {
        blockers.push("KIS base URL readiness incomplete".to_string());
    }
    if matches!(
        auth.readiness_status,
        KISAuthReadinessStatus::MissingWebSocketApprovalKey
    ) {
        blockers.push("KIS websocket approval readiness incomplete".to_string());
    }
    if endpoint.policy_status != KISEndpointPolicyStatus::MarketDataOnly {
        blockers.push("endpoint policy blocked requested scope".to_string());
    }
    if canonical.validation_reports.is_empty() {
        warnings.push("no local canonical CSV inputs were provided".to_string());
    }
    if !matches!(
        schema.schema_status,
        KISResponseSchemaStatus::SchemaValid | KISResponseSchemaStatus::DiagnosticOnly
    ) {
        blockers.push("schema drift report blocked official KIS activation".to_string());
    }
    if matches!(
        canonical.validation_status,
        KISCanonicalBatchValidationStatus::MissingPreflight
            | KISCanonicalBatchValidationStatus::MissingProvenance
    ) {
        blockers.push("preflight or provenance artifacts are missing".to_string());
    }
    let storage_budget_exceeded =
        collection_plan.storage_budget_summary.estimated_total_bytes > max_total_bytes;
    if storage_budget_exceeded {
        blockers.push("storage budget exceeded for configured KIS activation scope".to_string());
    }

    let added_kis_official_rows = official_row_count(canonical);
    let added_kis_preflight_ready_rows = preflight_ready_row_count(canonical);
    let added_kis_official_ready_candles = official_ready_candle_rows(candle);
    let added_kis_outcome_links = outcome
        .map(|value| value.generated_outcome_links)
        .unwrap_or_default();
    let added_kis_no_trade_counterfactuals = outcome
        .map(|value| value.generated_no_trade_counterfactuals)
        .unwrap_or_default();
    let added_kis_risk_denied_counterfactuals = outcome
        .map(|value| value.generated_risk_denied_counterfactuals)
        .unwrap_or_default();
    let added_complete_kis_rows = outcome
        .map(|value| value.complete_kis_rows)
        .unwrap_or_default();

    let (final_status, final_recommendation) = if storage_budget_exceeded {
        (
            KISOfficialMarketDataActivationFinalStatus::KISBudgetBlocked,
            KISOfficialMarketDataActivationRecommendation::RunKISBoundedCollection,
        )
    } else if matches!(
        auth.readiness_status,
        KISAuthReadinessStatus::MissingAppKey
            | KISAuthReadinessStatus::MissingAppSecret
            | KISAuthReadinessStatus::MissingAppKeyAndSecret
    ) {
        (
            KISOfficialMarketDataActivationFinalStatus::KISAuthMissing,
            match auth.readiness_status {
                KISAuthReadinessStatus::MissingAppSecret => {
                    KISOfficialMarketDataActivationRecommendation::SetKISAppSecret
                }
                _ => KISOfficialMarketDataActivationRecommendation::SetKISAppKey,
            },
        )
    } else if matches!(
        auth.readiness_status,
        KISAuthReadinessStatus::MissingBaseUrl
    ) {
        (
            KISOfficialMarketDataActivationFinalStatus::KISBaseUrlMissing,
            KISOfficialMarketDataActivationRecommendation::SetKISBaseUrl,
        )
    } else if matches!(
        auth.readiness_status,
        KISAuthReadinessStatus::MissingWebSocketApprovalKey
    ) {
        (
            KISOfficialMarketDataActivationFinalStatus::KISWebSocketApprovalMissing,
            KISOfficialMarketDataActivationRecommendation::SetKISWebSocketApprovalKey,
        )
    } else if endpoint.policy_status != KISEndpointPolicyStatus::MarketDataOnly {
        (
            KISOfficialMarketDataActivationFinalStatus::KISEndpointPolicyBlocked,
            KISOfficialMarketDataActivationRecommendation::RunKISDryRun,
        )
    } else if !matches!(
        schema.schema_status,
        KISResponseSchemaStatus::SchemaValid | KISResponseSchemaStatus::DiagnosticOnly
    ) {
        (
            KISOfficialMarketDataActivationFinalStatus::KISSchemaBlocked,
            KISOfficialMarketDataActivationRecommendation::RunKISDryRun,
        )
    } else if matches!(
        canonical.validation_status,
        KISCanonicalBatchValidationStatus::MissingPreflight
            | KISCanonicalBatchValidationStatus::MissingProvenance
    ) {
        (
            KISOfficialMarketDataActivationFinalStatus::KISPreflightBlocked,
            KISOfficialMarketDataActivationRecommendation::RunKISPreflight,
        )
    } else if canonical.validation_reports.is_empty() {
        (
            KISOfficialMarketDataActivationFinalStatus::NeedMoreEvidence,
            KISOfficialMarketDataActivationRecommendation::ProvideKISCanonicalCsv,
        )
    } else if matches!(
        candle.sufficiency_status,
        KISCandleSufficiencyStatus::MissingFutureWindows
    ) {
        (
            KISOfficialMarketDataActivationFinalStatus::StillNeedFutureWindows,
            KISOfficialMarketDataActivationRecommendation::CollectLongerKISWindow,
        )
    } else if added_complete_kis_rows > 0 {
        (
            KISOfficialMarketDataActivationFinalStatus::KISCompleteRowsImproved,
            if config.run_core_performance {
                KISOfficialMarketDataActivationRecommendation::RunCorePerformance
            } else if config.run_official_evidence_diversity_sweep {
                KISOfficialMarketDataActivationRecommendation::RunDiversitySweep
            } else {
                KISOfficialMarketDataActivationRecommendation::KeepKRXAsReference
            },
        )
    } else if added_kis_outcome_links > 0 {
        (
            KISOfficialMarketDataActivationFinalStatus::KISOutcomeLinksImproved,
            KISOfficialMarketDataActivationRecommendation::KeepTrinity,
        )
    } else if matches!(
        outcome.map(|value| value.closure_status),
        Some(KISOutcomeLinkClosureStatus::StillMissingOutcomeLinks)
    ) {
        (
            KISOfficialMarketDataActivationFinalStatus::StillMissingOutcomeLinks,
            KISOfficialMarketDataActivationRecommendation::RunOutcomeLinkClosure,
        )
    } else if added_kis_official_ready_candles > 0 {
        (
            KISOfficialMarketDataActivationFinalStatus::KISOfficialCandlesImproved,
            KISOfficialMarketDataActivationRecommendation::RunOutcomeLinkClosure,
        )
    } else if added_kis_official_rows > 0 {
        (
            KISOfficialMarketDataActivationFinalStatus::KISOfficialRowsImported,
            KISOfficialMarketDataActivationRecommendation::MoreKISOfficialRows,
        )
    } else if matches!(
        migration.migration_decision,
        ProviderMigrationDecision::SwitchKISToPrimary
            | ProviderMigrationDecision::KISPrimaryKRXReference
    ) {
        (
            KISOfficialMarketDataActivationFinalStatus::ProviderPriorityUpdated,
            KISOfficialMarketDataActivationRecommendation::KeepKRXAsReference,
        )
    } else if !collection_plan.runnable_jobs.is_empty() {
        (
            KISOfficialMarketDataActivationFinalStatus::KISCollectionReady,
            KISOfficialMarketDataActivationRecommendation::RunKISBoundedCollection,
        )
    } else if config.run_live_market_data_collection {
        (
            KISOfficialMarketDataActivationFinalStatus::KISMarketDataActivated,
            KISOfficialMarketDataActivationRecommendation::KeepKRXAsReference,
        )
    } else if outcome.is_some() {
        (
            KISOfficialMarketDataActivationFinalStatus::NoImprovement,
            KISOfficialMarketDataActivationRecommendation::NeedMoreEvidence,
        )
    } else {
        (
            KISOfficialMarketDataActivationFinalStatus::NeedMoreEvidence,
            KISOfficialMarketDataActivationRecommendation::NeedMoreEvidence,
        )
    };
    let reason_codes = stable_reason_codes(
        &[
            config.reason_codes.clone(),
            auth.reason_codes.clone(),
            endpoint.reason_codes.clone(),
            canonical.reason_codes.clone(),
            candle.reason_codes.clone(),
            outcome
                .map(|value| value.reason_codes.clone())
                .unwrap_or_default(),
            migration.reason_codes.clone(),
            vec![
                ReasonCode::KISMarketDataActivationRan,
                ReasonCode::ProviderPriorityUpdated,
                ReasonCode::KRXRetainedAsReference,
            ],
        ]
        .concat(),
    );
    KISOfficialMarketDataActivationReport {
        activation_id: config.activation_id.clone(),
        auth_readiness: auth.readiness_status,
        endpoint_policy_status: endpoint.policy_status,
        provider_migration_decision: migration.migration_decision,
        symbol_whitelist_summary: whitelist_summary(whitelist),
        collection_batch_plan_summary: collection_batch_summary(collection_plan),
        schema_drift_status: schema.schema_status,
        canonical_batch_validation_status: canonical.validation_status,
        candle_sufficiency_status: candle.sufficiency_status,
        outcome_link_closure_status: outcome.map(|value| value.closure_status),
        downstream_rerun_summary: downstream_summary(downstream),
        added_kis_canonical_csvs: config.local_kis_canonical_csv_paths.len(),
        added_kis_official_rows,
        added_kis_preflight_ready_rows,
        added_kis_official_ready_candles,
        added_kis_outcome_links,
        added_kis_no_trade_counterfactuals,
        added_kis_risk_denied_counterfactuals,
        added_complete_kis_rows,
        previous_core_status: outcome.and_then(|value| value.previous_core_status.clone()),
        current_core_status: outcome.and_then(|value| value.current_core_status.clone()),
        previous_primary_bottleneck: outcome
            .and_then(|value| value.previous_primary_bottleneck.clone()),
        current_primary_bottleneck: outcome
            .and_then(|value| value.current_primary_bottleneck.clone()),
        bottleneck_changed: outcome
            .map(|value| value.bottleneck_changed)
            .unwrap_or(false),
        operator_action_count,
        final_status,
        final_recommendation,
        blockers,
        warnings,
        reason_codes,
    }
}

fn load_endpoint_policy(
    config: &KISMarketDataActivationConfig,
) -> Result<KISEndpointPolicy, String> {
    if let Some(path) = &config.endpoint_policy_path {
        KISEndpointPolicy::from_toml_path(Path::new(path))
    } else {
        Ok(KISEndpointPolicy::default())
    }
}

fn load_symbol_whitelist(
    config: &KISMarketDataActivationConfig,
) -> Result<KISSymbolWhitelist, String> {
    let mut merged = KISSymbolWhitelistConfig {
        whitelist_id: format!("{}-symbols", config.activation_id),
        output_root: config.output_root.clone(),
        max_symbols: config.max_domestic_symbols.max(config.max_overseas_symbols),
        ..KISSymbolWhitelistConfig::default()
    };
    for path in [
        config.domestic_symbol_whitelist_path.as_deref(),
        config.overseas_symbol_whitelist_path.as_deref(),
    ]
    .into_iter()
    .flatten()
    {
        let loaded = KISSymbolWhitelistConfig::from_toml_path(Path::new(path))?;
        merged.symbols.extend(loaded.symbols);
    }
    merged.validate()?;
    Ok(merged.build())
}

fn build_raw_archive_summary(
    output_dir: &Path,
    collection_plan: &KISCollectionBatchPlan,
) -> Result<Option<KISRawResponseArchiveSummary>, String> {
    let mut records = Vec::new();
    let archive_dir = raw_archive_dir(output_dir);
    for job in &collection_plan.jobs {
        let Some(path) = &job.expected_raw_archive_path else {
            continue;
        };
        let source = Path::new(path);
        if !source.exists() {
            continue;
        }
        records.push(KISRawResponseArchiveRecord::from_fixture(
            &archive_dir,
            job.market,
            &job.provider_symbol,
            &job.normalized_symbol,
            &job.timeframe,
            job.endpoint_category,
            source,
        )?);
    }
    if records.is_empty() {
        Ok(None)
    } else {
        Ok(Some(KISRawResponseArchiveSummary::new(
            format!("{}-raw", output_dir.display()),
            records,
        )))
    }
}

fn average_quality_score(
    reports: &[super::kis_canonical_batch_validation::KISCanonicalValidationReport],
) -> Option<f64> {
    let scores = reports
        .iter()
        .filter_map(|report| report.quality_score)
        .collect::<Vec<_>>();
    if scores.is_empty() {
        None
    } else {
        Some(scores.iter().sum::<f64>() / scores.len() as f64)
    }
}

fn raw_paths(summary: Option<&KISRawResponseArchiveSummary>) -> Vec<String> {
    summary
        .map(|value| {
            value
                .records
                .iter()
                .map(|record| record.response_path.clone())
                .collect()
        })
        .unwrap_or_default()
}

fn report_paths(output_dir: &Path) -> Vec<String> {
    [
        "kis_auth_readiness.txt",
        "kis_auth_readiness.json",
        "kis_endpoint_policy.txt",
        "kis_endpoint_policy.json",
        "kis_symbol_whitelist.txt",
        "kis_symbol_whitelist.json",
        "kis_collection_batch_plan.txt",
        "kis_collection_batch_plan.json",
        "kis_raw_archive_summary.txt",
        "kis_raw_archive_summary.json",
        "kis_schema_drift_report.txt",
        "kis_schema_drift_report.json",
        "kis_canonical_batch_validation.txt",
        "kis_canonical_batch_validation.json",
        "kis_candle_sufficiency.txt",
        "kis_candle_sufficiency.json",
        "kis_outcome_link_closure.txt",
        "kis_outcome_link_closure.json",
        "kis_downstream_rerun_summary.txt",
        "kis_downstream_rerun_summary.json",
        "kis_krx_migration_report.txt",
        "kis_krx_migration_report.json",
        "kis_operator_actions.txt",
        "kis_operator_actions.json",
        "kis_market_data_activation_report.txt",
        "kis_market_data_activation_report.json",
        "kis_market_data_activation_summary.txt",
    ]
    .iter()
    .map(|name| output_dir.join(name).display().to_string())
    .collect()
}

fn write_operator_actions(output_dir: &Path, actions: &[KISOperatorAction]) -> Result<(), String> {
    let text = actions
        .iter()
        .map(|action| {
            format!(
                "{:?}\t{}\t{}",
                action.priority, action.action_id, action.description
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(output_dir.join("kis_operator_actions.txt"), text).map_err(|err| err.to_string())?;
    fs::write(
        output_dir.join("kis_operator_actions.json"),
        serde_json::to_string_pretty(actions).map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    Ok(())
}

fn primary_bottleneck(
    auth: &KISAuthReadinessReport,
    endpoint: &KISEndpointPolicyReport,
    canonical: &KISCanonicalBatchValidationReport,
    candle: &KISCandleSufficiencyReport,
    outcome: Option<&KISOutcomeLinkClosureReport>,
) -> String {
    if !auth.safe_to_collect_rest_market_data {
        "kis-auth".to_string()
    } else if endpoint.policy_status != KISEndpointPolicyStatus::MarketDataOnly {
        "endpoint-policy".to_string()
    } else if canonical.validation_reports.is_empty() {
        "local-canonical".to_string()
    } else if matches!(
        candle.sufficiency_status,
        KISCandleSufficiencyStatus::MissingFutureWindows
    ) {
        "future-window".to_string()
    } else if outcome.is_none() {
        "outcome-linkage".to_string()
    } else {
        "krx-reference-retained".to_string()
    }
}

fn default_output_root() -> String {
    "target/soma_kis_market_data_activation".to_string()
}

fn default_max_symbols() -> usize {
    5
}

fn default_max_rows_per_symbol() -> usize {
    300
}

fn default_max_requests() -> usize {
    10
}

fn default_max_days() -> usize {
    365
}

fn default_max_bytes() -> usize {
    20 * 1024 * 1024
}

fn default_max_total_bytes() -> usize {
    40 * 1024 * 1024
}

fn default_true() -> bool {
    true
}

fn official_row_count(report: &KISCanonicalBatchValidationReport) -> usize {
    report
        .validation_reports
        .iter()
        .filter(|item| item.official_readiness_eligible)
        .map(|item| item.row_count)
        .sum()
}

fn preflight_ready_row_count(report: &KISCanonicalBatchValidationReport) -> usize {
    report
        .validation_reports
        .iter()
        .filter(|item| item.preflight_available)
        .map(|item| item.row_count)
        .sum()
}

fn official_ready_candle_rows(report: &KISCandleSufficiencyReport) -> usize {
    report
        .items
        .iter()
        .filter(|item| item.official_ready)
        .map(|item| item.row_count)
        .sum()
}

fn whitelist_summary(whitelist: &KISSymbolWhitelist) -> String {
    format!(
        "symbols={};enabled={};domestic={};overseas={}",
        whitelist.symbol_count,
        whitelist.enabled_entries.len(),
        whitelist.domestic_count,
        whitelist.overseas_count
    )
}

fn collection_batch_summary(plan: &KISCollectionBatchPlan) -> String {
    format!(
        "jobs={};runnable={};skipped={};fixture={};local_import={};dry_run={};live={};budget_ok={}",
        plan.jobs.len(),
        plan.runnable_jobs.len(),
        plan.skipped_jobs.len(),
        plan.fixture_replay_jobs.len(),
        plan.local_import_jobs.len(),
        plan.dry_run_jobs.len(),
        plan.live_collection_jobs.len(),
        plan.storage_budget_summary.budget_ok
    )
}

fn downstream_summary(summary: &KISDownstreamRerunSummary) -> String {
    format!(
        "official_rows_after={};official_ready_candles_after={};outcome_links_after={};counterfactuals_after={};core_status_after={};primary_bottleneck_after={}",
        summary
            .official_rows_after
            .map(|value| value.to_string())
            .unwrap_or_default(),
        summary
            .official_ready_candles_after
            .map(|value| value.to_string())
            .unwrap_or_default(),
        summary
            .outcome_links_after
            .map(|value| value.to_string())
            .unwrap_or_default(),
        summary
            .counterfactuals_after
            .map(|value| value.to_string())
            .unwrap_or_default(),
        summary.core_status_after.clone().unwrap_or_default(),
        summary.primary_bottleneck_after.clone().unwrap_or_default()
    )
}
