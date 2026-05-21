use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::backtest::Timeframe;
use crate::core::{ReasonCode, stable_reason_codes};
use crate::data::{
    AssetClass, DataProvenance, EvidenceSourceKind, LocalDataOnboardingConfig, MarketVenue,
    PreflightReport, PreflightValidator,
};

use super::krx_candle_sufficiency::{KRXCandleSufficiencyReport, KRXCandleSufficiencyStatus};
use super::krx_canonical_batch_validation::{
    KRXCanonicalBatchValidationReport, infer_sidecar_path,
};
use super::krx_collection_batch::{KRXCollectionBatchJobKind, KRXCollectionBatchPlan};
use super::krx_collection_closure_bundle::{
    KRXCollectionClosureStorageReport, KRXOfficialCollectionClosureBundle,
};
use super::krx_collection_smoke::{
    KRXBoundedCollectionSmokeConfig, KRXCollectionDryRunReport, KRXCollectionDryRunStatus,
};
use super::krx_downstream_rerun_v2::KRXDownstreamRerunV2Summary;
use super::krx_official_activation::KRXOfficialEvidenceActivationConfig;
use super::krx_outcome_link_closure::{
    KRXOutcomeLinkClosureConfig, KRXOutcomeLinkClosureRecommendation, KRXOutcomeLinkClosureReport,
    KRXOutcomeLinkClosureRunner, KRXOutcomeLinkClosureStatus,
};
use super::krx_raw_archive::{
    KRXRawResponseArchiveRecord, KRXRawResponseArchiveSummary, raw_archive_dir,
};
use super::krx_schema_drift::{KRXResponseSchemaDriftReport, KRXResponseSchemaStatus};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KRXOfficialCollectionClosureConfig {
    pub run_id: String,
    #[serde(default)]
    pub bounded_collection_smoke_config_path: Option<String>,
    #[serde(default)]
    pub activation_config_path: Option<String>,
    #[serde(default)]
    pub outcome_link_closure_config_path: Option<String>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default = "default_true")]
    pub run_auth_dry_run: bool,
    #[serde(default = "default_true")]
    pub run_batch_plan: bool,
    #[serde(default = "default_true")]
    pub run_fixture_replay: bool,
    #[serde(default = "default_true")]
    pub run_local_import: bool,
    #[serde(default)]
    pub run_live_collection: bool,
    #[serde(default = "default_true")]
    pub run_raw_archive: bool,
    #[serde(default = "default_true")]
    pub run_schema_drift_check: bool,
    #[serde(default = "default_true")]
    pub run_canonical_validation: bool,
    #[serde(default = "default_true")]
    pub run_candle_sufficiency: bool,
    #[serde(default = "default_true")]
    pub run_outcome_link_closure: bool,
    #[serde(default = "default_true")]
    pub run_downstream_rerun_v2: bool,
    #[serde(default)]
    pub run_core_performance: bool,
    #[serde(default = "default_max_symbols")]
    pub max_symbols: usize,
    #[serde(default = "default_max_rows_per_symbol")]
    pub max_rows_per_symbol: usize,
    #[serde(default = "default_max_requests")]
    pub max_requests: usize,
    #[serde(default = "default_max_days")]
    pub max_days: usize,
    #[serde(default = "default_max_bytes")]
    pub max_bytes: usize,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum KRXOfficialCollectionClosureFinalStatus {
    KRXBoundedCollectionSucceeded,
    KRXOfficialCandlesImproved,
    KRXOutcomeLinksImproved,
    KRXCompleteRowsImproved,
    KRXAuthMissing,
    KRXEndpointMissing,
    KRXSchemaBlocked,
    KRXPreflightBlocked,
    KRXBudgetBlocked,
    StillMissingOfficialCandles,
    StillMissingOutcomeLinks,
    NoImprovement,
    NeedMoreEvidence,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum KRXOfficialCollectionClosureRecommendation {
    SetKRXApiKey,
    SetKRXEndpointTemplate,
    RunKRXDryRun,
    RunKRXBoundedCollection,
    ProvideKRXCanonicalCsv,
    RunKRXPreflight,
    CollectLongerKRXWindow,
    RunOutcomeLinkClosure,
    RunDiversitySweep,
    RunCorePerformance,
    MoreKRXOfficialRows,
    KeepTrinity,
    NeedMoreEvidence,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct KRXOfficialCollectionClosureReport {
    pub run_id: String,
    pub dry_run_status: KRXCollectionDryRunStatus,
    pub batch_plan_summary: String,
    #[serde(default)]
    pub schema_drift_status: Option<KRXResponseSchemaStatus>,
    pub canonical_batch_validation_status:
        super::krx_canonical_batch_validation::KRXCanonicalBatchValidationStatus,
    pub candle_sufficiency_status: KRXCandleSufficiencyStatus,
    #[serde(default)]
    pub outcome_link_closure_status: Option<KRXOutcomeLinkClosureStatus>,
    pub downstream_rerun_summary: KRXDownstreamRerunV2Summary,
    pub added_canonical_csvs: usize,
    pub added_official_rows: usize,
    pub added_official_ready_candles: usize,
    pub added_outcome_links: usize,
    pub added_no_trade_counterfactuals: usize,
    pub added_risk_denied_counterfactuals: usize,
    pub added_complete_krx_rows: usize,
    #[serde(default)]
    pub previous_core_status: Option<String>,
    #[serde(default)]
    pub current_core_status: Option<String>,
    #[serde(default)]
    pub previous_primary_bottleneck: Option<String>,
    #[serde(default)]
    pub current_primary_bottleneck: Option<String>,
    pub bottleneck_changed: bool,
    pub final_status: KRXOfficialCollectionClosureFinalStatus,
    pub final_recommendation: KRXOfficialCollectionClosureRecommendation,
    pub blockers: Vec<String>,
    pub warnings: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct KRXOfficialCollectionClosureRunner;

#[derive(Default)]
struct ImportedArtifact {
    canonical_path: String,
    provenance_path: Option<String>,
    preflight_path: Option<String>,
    manifest_path: Option<String>,
}

impl Default for KRXOfficialCollectionClosureConfig {
    fn default() -> Self {
        Self {
            run_id: "krx_collection_closure".to_string(),
            bounded_collection_smoke_config_path: None,
            activation_config_path: None,
            outcome_link_closure_config_path: None,
            output_root: default_output_root(),
            run_auth_dry_run: true,
            run_batch_plan: true,
            run_fixture_replay: true,
            run_local_import: true,
            run_live_collection: false,
            run_raw_archive: true,
            run_schema_drift_check: true,
            run_canonical_validation: true,
            run_candle_sufficiency: true,
            run_outcome_link_closure: true,
            run_downstream_rerun_v2: true,
            run_core_performance: false,
            max_symbols: default_max_symbols(),
            max_rows_per_symbol: default_max_rows_per_symbol(),
            max_requests: default_max_requests(),
            max_days: default_max_days(),
            max_bytes: default_max_bytes(),
            reason_codes: vec![
                ReasonCode::DeterministicPath,
                ReasonCode::KRXCollectionDisabledByDefault,
            ],
        }
    }
}

impl KRXOfficialCollectionClosureConfig {
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

    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.run_id)
    }

    pub fn validate_local_paths(&self) -> Vec<ReasonCode> {
        let mut reasons = Vec::new();
        for path in [
            Some(self.output_root.as_str()),
            self.bounded_collection_smoke_config_path.as_deref(),
            self.activation_config_path.as_deref(),
            self.outcome_link_closure_config_path.as_deref(),
        ]
        .into_iter()
        .flatten()
        {
            if path.contains("://") {
                reasons.push(ReasonCode::LocalPathRejected);
            }
        }
        stable_reason_codes(&reasons)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.run_id.trim().is_empty() {
            return Err("krx collection closure run_id must not be empty".to_string());
        }
        if !self.validate_local_paths().is_empty() {
            return Err("krx collection closure config paths must be local".to_string());
        }
        if self.max_symbols == 0 || self.max_symbols > default_max_symbols() {
            return Err("krx max_symbols must be between 1 and 5".to_string());
        }
        if self.max_rows_per_symbol == 0 || self.max_rows_per_symbol > default_max_rows_per_symbol()
        {
            return Err("krx max_rows_per_symbol must be between 1 and 300".to_string());
        }
        if self.max_requests == 0 || self.max_requests > default_max_requests() {
            return Err("krx max_requests must be between 1 and 10".to_string());
        }
        if self.max_days == 0 || self.max_days > default_max_days() {
            return Err("krx max_days must be between 1 and 365".to_string());
        }
        if self.max_bytes == 0 {
            return Err("krx max_bytes must be positive".to_string());
        }
        Ok(())
    }
}

impl KRXOfficialCollectionClosureRunner {
    pub fn run(
        &self,
        config: &KRXOfficialCollectionClosureConfig,
    ) -> Result<KRXOfficialCollectionClosureBundle, String> {
        config.validate()?;
        let output_dir = config.output_dir();
        fs::create_dir_all(&output_dir).map_err(|err| err.to_string())?;

        let smoke_config = load_smoke_config(config)?;
        let whitelist = smoke_config.load_whitelist()?;
        let dry_run_report = smoke_config.build_dry_run_report(&whitelist);
        let batch_plan = KRXCollectionBatchPlan::build(&smoke_config, &dry_run_report, &whitelist);

        let mut imported = Vec::new();
        let mut raw_records = Vec::new();
        import_local_jobs(
            config,
            &smoke_config,
            &batch_plan,
            &output_dir,
            &mut imported,
        )?;
        archive_fixture_jobs(config, &batch_plan, &output_dir, &mut raw_records)?;

        let raw_summary = if config.run_raw_archive && !raw_records.is_empty() {
            Some(KRXRawResponseArchiveSummary::new(
                format!("{}-raw-archive", config.run_id),
                raw_records,
            ))
        } else {
            None
        };
        if let Some(summary) = &raw_summary {
            summary.write_to_dir(&output_dir)?;
        }

        let schema_drift_report = if config.run_schema_drift_check {
            let report = KRXResponseSchemaDriftReport::from_archive(
                raw_summary.as_ref(),
                &format!("{}-schema-drift", config.run_id),
            );
            report.write_to_dir(&output_dir)?;
            Some(report)
        } else {
            None
        };

        if matches!(
            schema_drift_report
                .as_ref()
                .map(|report| report.schema_status),
            Some(
                KRXResponseSchemaStatus::SchemaValid | KRXResponseSchemaStatus::UnexpectedFieldSet
            )
        ) {
            canonicalize_fixture_records(
                raw_summary.as_ref().unwrap(),
                &output_dir,
                &mut imported,
            )?;
        }

        let canonical_paths = imported
            .iter()
            .map(|artifact| artifact.canonical_path.clone())
            .collect::<Vec<_>>();
        let canonical_batch_validation_report = KRXCanonicalBatchValidationReport::build(
            &format!("{}-canonical-batch", config.run_id),
            &canonical_paths,
            true,
            true,
        );
        canonical_batch_validation_report.write_to_dir(&output_dir)?;

        let candle_sufficiency_report = if config.run_candle_sufficiency {
            KRXCandleSufficiencyReport::from_batch_validation(
                &canonical_batch_validation_report,
                &format!("{}-candle-sufficiency", config.run_id),
                smoke_config.barrier_profile_registry_path.as_deref(),
            )
        } else {
            KRXCandleSufficiencyReport {
                report_id: format!("{}-candle-sufficiency", config.run_id),
                items: Vec::new(),
                total_series: 0,
                official_ready_series: 0,
                benchmark_ready_series: 0,
                series_with_sufficient_future_window: 0,
                series_missing_future_window: 0,
                sufficiency_status: KRXCandleSufficiencyStatus::DiagnosticOnly,
                reason_codes: vec![ReasonCode::DeterministicPath],
            }
        };
        candle_sufficiency_report.write_to_dir(&output_dir)?;

        let schema_status = schema_drift_report
            .as_ref()
            .map(|report| report.schema_status);
        let schema_blocks_downstream = matches!(
            schema_status,
            Some(
                KRXResponseSchemaStatus::UnsupportedSchema
                    | KRXResponseSchemaStatus::MissingRequiredField
                    | KRXResponseSchemaStatus::BadDateField
                    | KRXResponseSchemaStatus::BadPriceField
                    | KRXResponseSchemaStatus::BadVolumeField
            )
        );
        let outcome_config = load_or_build_outcome_config(config, &smoke_config, &canonical_paths)?;
        let outcome_link_closure_report =
            if config.run_outcome_link_closure && !schema_blocks_downstream {
                let report = KRXOutcomeLinkClosureRunner::default().run(&outcome_config)?;
                Some(report)
            } else {
                None
            };

        let downstream_rerun_v2_summary = KRXDownstreamRerunV2Summary::build(
            &canonical_batch_validation_report,
            &candle_sufficiency_report,
            outcome_link_closure_report.as_ref(),
            config.run_downstream_rerun_v2,
            config.run_core_performance,
        );
        downstream_rerun_v2_summary.write_to_dir(&output_dir)?;
        let final_status = determine_final_status(
            &dry_run_report,
            schema_status,
            &canonical_batch_validation_report,
            &candle_sufficiency_report,
            outcome_link_closure_report.as_ref(),
            config.max_bytes < batch_plan.storage_budget_summary.estimated_bytes,
        );
        let final_recommendation = determine_final_recommendation(
            &dry_run_report,
            &canonical_batch_validation_report,
            &candle_sufficiency_report,
            outcome_link_closure_report.as_ref(),
        );
        let blockers = collect_blockers(
            &dry_run_report,
            &canonical_batch_validation_report,
            schema_status,
            outcome_link_closure_report.as_ref(),
            config.max_bytes < batch_plan.storage_budget_summary.estimated_bytes,
        );
        let warnings = collect_warnings(
            config,
            &dry_run_report,
            &candle_sufficiency_report,
            outcome_link_closure_report.as_ref(),
        );

        let report = KRXOfficialCollectionClosureReport {
            run_id: config.run_id.clone(),
            dry_run_status: dry_run_report.dry_run_status,
            batch_plan_summary: batch_plan.to_text(),
            schema_drift_status: schema_status,
            canonical_batch_validation_status: canonical_batch_validation_report.validation_status,
            candle_sufficiency_status: candle_sufficiency_report.sufficiency_status,
            outcome_link_closure_status: outcome_link_closure_report
                .as_ref()
                .map(|report| report.closure_status),
            downstream_rerun_summary: downstream_rerun_v2_summary.clone(),
            added_canonical_csvs: canonical_paths.len(),
            added_official_rows: canonical_batch_validation_report
                .validation_reports
                .iter()
                .filter(|report| report.official_readiness_eligible)
                .map(|report| report.row_count)
                .sum(),
            added_official_ready_candles: candle_sufficiency_report
                .items
                .iter()
                .filter(|item| item.official_ready)
                .map(|item| item.row_count)
                .sum(),
            added_outcome_links: outcome_link_closure_report
                .as_ref()
                .map(|report| report.generated_outcome_links)
                .unwrap_or(0),
            added_no_trade_counterfactuals: outcome_link_closure_report
                .as_ref()
                .map(|report| report.generated_no_trade_counterfactuals)
                .unwrap_or(0),
            added_risk_denied_counterfactuals: outcome_link_closure_report
                .as_ref()
                .map(|report| report.generated_risk_denied_counterfactuals)
                .unwrap_or(0),
            added_complete_krx_rows: outcome_link_closure_report
                .as_ref()
                .map(|report| report.complete_krx_rows)
                .unwrap_or(0),
            previous_core_status: outcome_link_closure_report
                .as_ref()
                .and_then(|report| report.previous_core_status.clone()),
            current_core_status: outcome_link_closure_report
                .as_ref()
                .and_then(|report| report.current_core_status.clone()),
            previous_primary_bottleneck: outcome_link_closure_report
                .as_ref()
                .and_then(|report| report.previous_primary_bottleneck.clone()),
            current_primary_bottleneck: outcome_link_closure_report
                .as_ref()
                .and_then(|report| report.current_primary_bottleneck.clone()),
            bottleneck_changed: outcome_link_closure_report
                .as_ref()
                .is_some_and(|report| report.bottleneck_changed),
            final_status,
            final_recommendation,
            blockers,
            warnings,
            reason_codes: stable_reason_codes(
                &[
                    config.reason_codes.clone(),
                    dry_run_report.reason_codes.clone(),
                    batch_plan.reason_codes.clone(),
                    canonical_batch_validation_report.reason_codes.clone(),
                    candle_sufficiency_report.reason_codes.clone(),
                    schema_drift_report
                        .as_ref()
                        .map(|report| report.reason_codes.clone())
                        .unwrap_or_default(),
                    outcome_link_closure_report
                        .as_ref()
                        .map(|report| report.reason_codes.clone())
                        .unwrap_or_default(),
                    downstream_rerun_v2_summary.reason_codes.clone(),
                ]
                .concat(),
            ),
        };
        report.write_to_dir(&output_dir)?;

        let raw_paths = raw_summary
            .as_ref()
            .map(|summary| {
                summary
                    .records
                    .iter()
                    .map(|record| record.response_path.clone())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let provenance_paths = imported
            .iter()
            .filter_map(|artifact| artifact.provenance_path.clone())
            .collect::<Vec<_>>();
        let preflight_paths = imported
            .iter()
            .filter_map(|artifact| artifact.preflight_path.clone())
            .collect::<Vec<_>>();
        let manifest_paths = imported
            .iter()
            .filter_map(|artifact| artifact.manifest_path.clone())
            .collect::<Vec<_>>();
        let outcome_paths = if outcome_link_closure_report.is_some() {
            vec![
                output_dir
                    .join("krx_outcome_link_closure.json")
                    .display()
                    .to_string(),
            ]
        } else {
            Vec::new()
        };
        let downstream_paths = vec![
            output_dir
                .join("krx_downstream_rerun_v2.json")
                .display()
                .to_string(),
        ];
        let report_paths = vec![
            output_dir
                .join("krx_auth_dry_run.txt")
                .display()
                .to_string(),
            output_dir
                .join("krx_collection_batch_plan.txt")
                .display()
                .to_string(),
            output_dir
                .join("krx_canonical_batch_validation.txt")
                .display()
                .to_string(),
            output_dir
                .join("krx_candle_sufficiency.txt")
                .display()
                .to_string(),
            output_dir
                .join("krx_collection_closure_report.txt")
                .display()
                .to_string(),
        ];
        let storage_report = KRXCollectionClosureStorageReport::build(
            &raw_paths,
            &canonical_paths,
            &provenance_paths,
            &manifest_paths,
            &preflight_paths,
            &Vec::new(),
            &outcome_paths,
            &Vec::new(),
            &downstream_paths,
            &report_paths,
            config.max_bytes,
        );
        storage_report.write_to_dir(&output_dir)?;

        let mut bundle = KRXOfficialCollectionClosureBundle {
            auth_dry_run_report: dry_run_report,
            collection_batch_plan: batch_plan,
            raw_response_archive_summary: raw_summary,
            schema_drift_report,
            canonical_batch_validation_report,
            candle_sufficiency_report,
            outcome_link_closure_report,
            downstream_rerun_v2_summary,
            collection_closure_report: report.clone(),
            storage_report,
            final_summary: report.to_text(),
            reason_codes: report.reason_codes.clone(),
        };
        bundle.finalize_reason_codes();
        bundle.write_to_dir(&output_dir)?;
        Ok(bundle)
    }
}

impl KRXOfficialCollectionClosureReport {
    pub fn to_text(&self) -> String {
        [
            format!("run_id={}", self.run_id),
            format!("dry_run_status={:?}", self.dry_run_status),
            format!(
                "schema_drift_status={}",
                self.schema_drift_status
                    .map(|status| format!("{status:?}"))
                    .unwrap_or_default()
            ),
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
                    .map(|status| format!("{status:?}"))
                    .unwrap_or_default()
            ),
            format!("added_canonical_csvs={}", self.added_canonical_csvs),
            format!("added_official_rows={}", self.added_official_rows),
            format!(
                "added_official_ready_candles={}",
                self.added_official_ready_candles
            ),
            format!("added_outcome_links={}", self.added_outcome_links),
            format!(
                "added_no_trade_counterfactuals={}",
                self.added_no_trade_counterfactuals
            ),
            format!(
                "added_risk_denied_counterfactuals={}",
                self.added_risk_denied_counterfactuals
            ),
            format!("added_complete_krx_rows={}", self.added_complete_krx_rows),
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
            output_dir.join("krx_collection_closure_report.txt"),
            self.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("krx_collection_closure_report.json"),
            self.to_json_string()?,
        )
        .map_err(|err| err.to_string())?;
        Ok(())
    }
}

fn load_smoke_config(
    config: &KRXOfficialCollectionClosureConfig,
) -> Result<KRXBoundedCollectionSmokeConfig, String> {
    if let Some(path) = config.bounded_collection_smoke_config_path.as_deref() {
        return KRXBoundedCollectionSmokeConfig::from_toml_path(Path::new(path));
    }
    if let Some(path) = config.activation_config_path.as_deref() {
        let activation = KRXOfficialEvidenceActivationConfig::from_toml_path(Path::new(path))?;
        return Ok(KRXBoundedCollectionSmokeConfig {
            smoke_id: format!("{}-smoke", config.run_id),
            activation_config_path: Some(path.to_string()),
            symbol_whitelist_path: activation.symbol_whitelist_path,
            barrier_profile_registry_path: activation.barrier_profile_registry_path,
            local_fixture_response_paths: Vec::new(),
            local_canonical_csv_paths: activation.local_krx_canonical_csv_paths,
            output_root: config.output_root.clone(),
            max_symbols: config.max_symbols,
            max_rows_per_symbol: config.max_rows_per_symbol,
            max_requests: config.max_requests,
            max_days: config.max_days,
            max_raw_bytes: config.max_bytes / 2,
            max_canonical_bytes: config.max_bytes / 2,
            max_total_bytes: config.max_bytes,
            require_krx_api_key: activation.require_krx_api_key,
            require_krx_endpoint_template: activation.require_krx_endpoint_template,
            run_dry_run: true,
            run_live_collection: config.run_live_collection,
            run_fixture_replay: config.run_fixture_replay,
            run_local_import: config.run_local_import,
            run_preflight: activation.run_preflight,
            run_downstream_reruns: config.run_downstream_rerun_v2,
            redact_endpoint_preview: true,
            reason_codes: activation.reason_codes,
        });
    }
    Ok(KRXBoundedCollectionSmokeConfig {
        smoke_id: format!("{}-smoke", config.run_id),
        output_root: config.output_root.clone(),
        max_symbols: config.max_symbols,
        max_rows_per_symbol: config.max_rows_per_symbol,
        max_requests: config.max_requests,
        max_days: config.max_days,
        max_raw_bytes: config.max_bytes / 2,
        max_canonical_bytes: config.max_bytes / 2,
        max_total_bytes: config.max_bytes,
        run_live_collection: config.run_live_collection,
        run_fixture_replay: config.run_fixture_replay,
        run_local_import: config.run_local_import,
        run_downstream_reruns: config.run_downstream_rerun_v2,
        ..KRXBoundedCollectionSmokeConfig::default()
    })
}

fn load_or_build_outcome_config(
    config: &KRXOfficialCollectionClosureConfig,
    smoke_config: &KRXBoundedCollectionSmokeConfig,
    canonical_paths: &[String],
) -> Result<KRXOutcomeLinkClosureConfig, String> {
    if let Some(path) = config.outcome_link_closure_config_path.as_deref() {
        return KRXOutcomeLinkClosureConfig::from_toml_path(Path::new(path));
    }
    Ok(KRXOutcomeLinkClosureConfig {
        closure_id: format!("{}-outcome", config.run_id),
        krx_activation_report_paths: Vec::new(),
        krx_canonical_csv_paths: canonical_paths.to_vec(),
        krx_candle_sufficiency_paths: Vec::new(),
        official_ready_rows_paths: Vec::new(),
        barrier_profile_registry_path: smoke_config.barrier_profile_registry_path.clone(),
        output_root: config.output_root.clone(),
        run_future_window_requirements: true,
        run_outcome_linkage_v3: true,
        run_counterfactual_completion_v2: true,
        run_complete_row_close_v2: true,
        run_official_evidence_scaleout: config.run_downstream_rerun_v2,
        run_official_evidence_diversity_sweep: config.run_downstream_rerun_v2,
        run_core_performance: config.run_core_performance,
        reason_codes: vec![ReasonCode::DeterministicPath],
    })
}

fn import_local_jobs(
    config: &KRXOfficialCollectionClosureConfig,
    smoke_config: &KRXBoundedCollectionSmokeConfig,
    batch_plan: &KRXCollectionBatchPlan,
    output_dir: &Path,
    imported: &mut Vec<ImportedArtifact>,
) -> Result<(), String> {
    if !config.run_local_import {
        return Ok(());
    }
    let import_dir = output_dir.join("artifacts/imported");
    let manifest_dir = output_dir.join("artifacts/manifest");
    fs::create_dir_all(&import_dir).map_err(|err| err.to_string())?;
    fs::create_dir_all(&manifest_dir).map_err(|err| err.to_string())?;
    for job in &batch_plan.jobs {
        if !matches!(
            job.job_kind,
            KRXCollectionBatchJobKind::LocalCanonicalCsvImport
                | KRXCollectionBatchJobKind::ExistingCollectedCsvReuse
        ) {
            continue;
        }
        if let Some(source_path) = job.expected_canonical_csv_path.as_deref() {
            let source = Path::new(source_path);
            if !source.exists() {
                continue;
            }
            let destination = import_dir.join(source.file_name().unwrap_or_default());
            fs::copy(source, &destination).map_err(|err| err.to_string())?;
            let provenance_path = copy_if_exists(
                infer_sidecar_path(source_path, "_provenance.json").as_deref(),
                &import_dir,
            )?;
            let mut preflight_path = copy_if_exists(
                infer_sidecar_path(source_path, "_preflight.json").as_deref(),
                &import_dir,
            )?;
            if preflight_path.is_none() && smoke_config.run_preflight {
                preflight_path = Some(generate_preflight(
                    &destination,
                    &job.normalized_symbol,
                    &import_dir,
                    EvidenceSourceKind::RealLocal,
                    true,
                )?);
            }
            let manifest_path =
                maybe_write_manifest_from_preflight(preflight_path.as_deref(), &manifest_dir)?;
            imported.push(ImportedArtifact {
                canonical_path: destination.display().to_string(),
                provenance_path,
                preflight_path,
                manifest_path,
            });
        }
    }
    Ok(())
}

fn archive_fixture_jobs(
    config: &KRXOfficialCollectionClosureConfig,
    batch_plan: &KRXCollectionBatchPlan,
    output_dir: &Path,
    raw_records: &mut Vec<KRXRawResponseArchiveRecord>,
) -> Result<(), String> {
    if !config.run_fixture_replay {
        return Ok(());
    }
    let archive_dir = raw_archive_dir(output_dir);
    for job in &batch_plan.jobs {
        if job.job_kind != KRXCollectionBatchJobKind::FixtureReplay {
            continue;
        }
        let Some(source_path) = job.expected_raw_archive_path.as_deref() else {
            continue;
        };
        let source = Path::new(source_path);
        if !source.exists() {
            continue;
        }
        raw_records.push(KRXRawResponseArchiveRecord::from_fixture(
            &archive_dir,
            &job.provider_symbol,
            &job.normalized_symbol,
            source,
        )?);
    }
    Ok(())
}

fn canonicalize_fixture_records(
    raw_summary: &KRXRawResponseArchiveSummary,
    output_dir: &Path,
    imported: &mut Vec<ImportedArtifact>,
) -> Result<(), String> {
    let canonical_dir = output_dir.join("artifacts/canonical");
    let manifest_dir = output_dir.join("artifacts/manifest");
    fs::create_dir_all(&canonical_dir).map_err(|err| err.to_string())?;
    fs::create_dir_all(&manifest_dir).map_err(|err| err.to_string())?;
    for record in &raw_summary.records {
        let canonical_path = canonical_dir.join(format!(
            "{}_1d.csv",
            record.normalized_symbol.to_ascii_lowercase()
        ));
        write_canonical_csv_from_raw(record, &canonical_path)?;
        let provenance_path = canonical_dir.join(format!(
            "{}_1d_provenance.json",
            record.normalized_symbol.to_ascii_lowercase()
        ));
        let provenance = DataProvenance {
            source_kind: EvidenceSourceKind::SyntheticFixture,
            source_label: format!("{}-fixture-replay", record.normalized_symbol),
            provider_label: Some("krx-open-api-fixture".to_string()),
            upstream_label: Some("fixture-replay".to_string()),
            local_path: Some(canonical_path.display().to_string()),
            generated_by: Some("sprint50-fixture-replay".to_string()),
            user_supplied: false,
            downloaded_by_soma: false,
            remote_url_present: false,
            official_provider: Some(false),
            affiliated_or_endorsed: Some(false),
            intended_use: Some("fixture replay architecture smoke only".to_string()),
            readiness_eligible: Some(false),
            benchmark_eligible: Some(true),
            license_note: Some("fixture replay; not official evidence".to_string()),
            notes: Some("no secret values".to_string()),
            reason_codes: vec![ReasonCode::SyntheticFixtureEvidence],
        };
        fs::write(
            &provenance_path,
            serde_json::to_string_pretty(&provenance).map_err(|err| err.to_string())?,
        )
        .map_err(|err| err.to_string())?;
        let preflight_path = generate_preflight(
            &canonical_path,
            &record.normalized_symbol,
            &canonical_dir,
            EvidenceSourceKind::SyntheticFixture,
            false,
        )?;
        let manifest_path =
            maybe_write_manifest_from_preflight(Some(&preflight_path), &manifest_dir)?;
        imported.push(ImportedArtifact {
            canonical_path: canonical_path.display().to_string(),
            provenance_path: Some(provenance_path.display().to_string()),
            preflight_path: Some(preflight_path),
            manifest_path,
        });
    }
    Ok(())
}

fn generate_preflight(
    canonical_path: &Path,
    symbol: &str,
    preflight_dir: &Path,
    source_kind: EvidenceSourceKind,
    user_supplied: bool,
) -> Result<String, String> {
    let onboarding = LocalDataOnboardingConfig {
        onboarding_id: format!("{symbol}-preflight"),
        input_path: canonical_path.display().to_string(),
        output_root: preflight_dir.display().to_string(),
        symbol: Some(symbol.to_string()),
        venue: Some(MarketVenue::KRX),
        asset_class: Some(AssetClass::Equity),
        timeframe: Some(Timeframe::OneDay),
        csv_format_hint: None,
        custom_column_map: None,
        source_kind: Some(source_kind),
        user_supplied,
        source_label: Some(format!("krx-{symbol}-sprint50")),
        strict: true,
        allow_format_autodetect: true,
        allow_sort_repair: false,
        allow_duplicate_drop: false,
        min_rows_for_preflight: 3,
        target_min_outcomes: 1,
        target_min_comparable_variants: 1,
        target_min_usable_datasets: 1,
        walk_forward_config: None,
        triple_barrier_config: None,
        cost_model: None,
        reason_codes: vec![ReasonCode::DeterministicPath],
    };
    let report = PreflightValidator::default().run(&onboarding);
    let path = preflight_dir.join(format!("{}_1d_preflight.json", symbol.to_ascii_lowercase()));
    fs::write(&path, report.to_json_string()?).map_err(|err| err.to_string())?;
    Ok(path.display().to_string())
}

fn maybe_write_manifest_from_preflight(
    preflight_path: Option<&str>,
    manifest_dir: &Path,
) -> Result<Option<String>, String> {
    let Some(preflight_path) = preflight_path else {
        return Ok(None);
    };
    let text = fs::read_to_string(preflight_path).map_err(|err| err.to_string())?;
    let report: PreflightReport = serde_json::from_str(&text).map_err(|err| err.to_string())?;
    let Some(manifest) = report.data_manifest_preview else {
        return Ok(None);
    };
    let path = manifest_dir.join(format!(
        "{}.json",
        Path::new(preflight_path)
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("manifest")
            .replace("_preflight", "_manifest")
    ));
    fs::write(
        &path,
        serde_json::to_string_pretty(&manifest).map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    Ok(Some(path.display().to_string()))
}

fn copy_if_exists(source: Option<&str>, destination_dir: &Path) -> Result<Option<String>, String> {
    let Some(source) = source else {
        return Ok(None);
    };
    let source_path = Path::new(source);
    if !source_path.exists() {
        return Ok(None);
    }
    let destination = destination_dir.join(source_path.file_name().unwrap_or_default());
    fs::copy(source_path, &destination).map_err(|err| err.to_string())?;
    Ok(Some(destination.display().to_string()))
}

fn write_canonical_csv_from_raw(
    record: &KRXRawResponseArchiveRecord,
    canonical_path: &Path,
) -> Result<(), String> {
    let text = fs::read_to_string(&record.response_path).map_err(|err| err.to_string())?;
    let value: Value = serde_json::from_str(&text).map_err(|err| err.to_string())?;
    let rows = value
        .get("rows")
        .and_then(Value::as_array)
        .ok_or_else(|| "fixture rows missing".to_string())?;
    let mut lines =
        vec!["timestamp_ms,open,high,low,close,volume,trade_value,bid,ask,spread_bps".to_string()];
    for row in rows {
        let date = row
            .get("date")
            .and_then(Value::as_str)
            .ok_or_else(|| "fixture date missing".to_string())?;
        let timestamp_ms = date_to_timestamp_ms(date)?;
        lines.push(format!(
            "{timestamp_ms},{},{},{},{},{},{},{},{},{}",
            number(row, "open")?,
            number(row, "high")?,
            number(row, "low")?,
            number(row, "close")?,
            number(row, "volume")?,
            number(row, "trade_value")?,
            number(row, "bid")?,
            number(row, "ask")?,
            number(row, "spread_bps")?,
        ));
    }
    fs::write(canonical_path, lines.join("\n")).map_err(|err| err.to_string())
}

fn number(row: &Value, key: &str) -> Result<String, String> {
    row.get(key)
        .and_then(|value| {
            value
                .as_f64()
                .map(|v| trim_float(v))
                .or_else(|| value.as_i64().map(|v| v.to_string()))
        })
        .ok_or_else(|| format!("fixture numeric field '{key}' missing"))
}

fn trim_float(value: f64) -> String {
    let rendered = format!("{value:.4}");
    rendered
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_string()
}

fn date_to_timestamp_ms(value: &str) -> Result<u64, String> {
    let parts = value.split('-').collect::<Vec<_>>();
    if parts.len() != 3 {
        return Err("bad date field".to_string());
    }
    let year: i64 = parts[0].parse().map_err(|_| "bad year".to_string())?;
    let month: i64 = parts[1].parse().map_err(|_| "bad month".to_string())?;
    let day: i64 = parts[2].parse().map_err(|_| "bad day".to_string())?;
    let days = days_from_civil(year, month, day);
    Ok((days * 86_400_000) as u64)
}

fn days_from_civil(mut year: i64, month: i64, day: i64) -> i64 {
    year -= if month <= 2 { 1 } else { 0 };
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let yoe = year - era * 400;
    let mp = month + if month > 2 { -3 } else { 9 };
    let doy = (153 * mp + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

fn determine_final_status(
    dry_run_report: &KRXCollectionDryRunReport,
    schema_status: Option<KRXResponseSchemaStatus>,
    canonical_batch_validation_report: &KRXCanonicalBatchValidationReport,
    candle_sufficiency_report: &KRXCandleSufficiencyReport,
    outcome_report: Option<&KRXOutcomeLinkClosureReport>,
    budget_exceeded: bool,
) -> KRXOfficialCollectionClosureFinalStatus {
    let no_canonical_data = canonical_batch_validation_report.total_rows == 0;
    if budget_exceeded {
        return KRXOfficialCollectionClosureFinalStatus::KRXBudgetBlocked;
    }
    if matches!(
        schema_status,
        Some(
            KRXResponseSchemaStatus::UnsupportedSchema
                | KRXResponseSchemaStatus::MissingRequiredField
                | KRXResponseSchemaStatus::BadDateField
                | KRXResponseSchemaStatus::BadPriceField
                | KRXResponseSchemaStatus::BadVolumeField
        )
    ) && no_canonical_data
    {
        return KRXOfficialCollectionClosureFinalStatus::KRXSchemaBlocked;
    }
    if matches!(
        dry_run_report.dry_run_status,
        KRXCollectionDryRunStatus::MissingApiKey
            | KRXCollectionDryRunStatus::MissingApiKeyAndEndpointTemplate
    ) && no_canonical_data
    {
        return KRXOfficialCollectionClosureFinalStatus::KRXAuthMissing;
    }
    if matches!(
        dry_run_report.dry_run_status,
        KRXCollectionDryRunStatus::MissingEndpointTemplate
    ) && no_canonical_data
    {
        return KRXOfficialCollectionClosureFinalStatus::KRXEndpointMissing;
    }
    if let Some(outcome_report) = outcome_report {
        return match outcome_report.closure_status {
            KRXOutcomeLinkClosureStatus::KRXCompleteRowsImproved => {
                KRXOfficialCollectionClosureFinalStatus::KRXCompleteRowsImproved
            }
            KRXOutcomeLinkClosureStatus::KRXOutcomeLinksImproved
            | KRXOutcomeLinkClosureStatus::KRXCounterfactualsImproved => {
                KRXOfficialCollectionClosureFinalStatus::KRXOutcomeLinksImproved
            }
            KRXOutcomeLinkClosureStatus::StillMissingOfficialCandles => {
                KRXOfficialCollectionClosureFinalStatus::StillMissingOfficialCandles
            }
            KRXOutcomeLinkClosureStatus::StillMissingFutureWindows
            | KRXOutcomeLinkClosureStatus::StillMissingOutcomeLinks
            | KRXOutcomeLinkClosureStatus::StillMissingCounterfactuals => {
                KRXOfficialCollectionClosureFinalStatus::StillMissingOutcomeLinks
            }
            KRXOutcomeLinkClosureStatus::StillNeedMoreKRXRows => {
                KRXOfficialCollectionClosureFinalStatus::NeedMoreEvidence
            }
            KRXOutcomeLinkClosureStatus::NoImprovement => {
                KRXOfficialCollectionClosureFinalStatus::NoImprovement
            }
        };
    }
    if canonical_batch_validation_report.missing_preflight_count > 0
        || canonical_batch_validation_report.missing_provenance_count > 0
    {
        return KRXOfficialCollectionClosureFinalStatus::KRXPreflightBlocked;
    }
    if candle_sufficiency_report.official_ready_series > 0 {
        return KRXOfficialCollectionClosureFinalStatus::KRXOfficialCandlesImproved;
    }
    if canonical_batch_validation_report.valid_csv_count > 0 {
        return KRXOfficialCollectionClosureFinalStatus::KRXBoundedCollectionSucceeded;
    }
    KRXOfficialCollectionClosureFinalStatus::NeedMoreEvidence
}

fn determine_final_recommendation(
    dry_run_report: &KRXCollectionDryRunReport,
    canonical_batch_validation_report: &KRXCanonicalBatchValidationReport,
    candle_sufficiency_report: &KRXCandleSufficiencyReport,
    outcome_report: Option<&KRXOutcomeLinkClosureReport>,
) -> KRXOfficialCollectionClosureRecommendation {
    let no_canonical_data = canonical_batch_validation_report.valid_csv_count == 0;
    if matches!(
        dry_run_report.dry_run_status,
        KRXCollectionDryRunStatus::MissingApiKey
            | KRXCollectionDryRunStatus::MissingApiKeyAndEndpointTemplate
    ) && no_canonical_data
    {
        return KRXOfficialCollectionClosureRecommendation::SetKRXApiKey;
    }
    if matches!(
        dry_run_report.dry_run_status,
        KRXCollectionDryRunStatus::MissingEndpointTemplate
    ) && no_canonical_data
    {
        return KRXOfficialCollectionClosureRecommendation::SetKRXEndpointTemplate;
    }
    if canonical_batch_validation_report.valid_csv_count == 0 {
        return KRXOfficialCollectionClosureRecommendation::ProvideKRXCanonicalCsv;
    }
    if canonical_batch_validation_report.missing_preflight_count > 0
        || canonical_batch_validation_report.missing_provenance_count > 0
    {
        return KRXOfficialCollectionClosureRecommendation::RunKRXPreflight;
    }
    if candle_sufficiency_report.series_missing_future_window > 0 {
        return KRXOfficialCollectionClosureRecommendation::CollectLongerKRXWindow;
    }
    if let Some(outcome_report) = outcome_report {
        return match outcome_report.final_recommendation {
            KRXOutcomeLinkClosureRecommendation::CollectLongerKRXWindow => {
                KRXOfficialCollectionClosureRecommendation::CollectLongerKRXWindow
            }
            KRXOutcomeLinkClosureRecommendation::MoreKRXOfficialRows => {
                KRXOfficialCollectionClosureRecommendation::MoreKRXOfficialRows
            }
            KRXOutcomeLinkClosureRecommendation::RunDiversitySweep => {
                KRXOfficialCollectionClosureRecommendation::RunDiversitySweep
            }
            KRXOutcomeLinkClosureRecommendation::RunCorePerformance => {
                KRXOfficialCollectionClosureRecommendation::RunCorePerformance
            }
            KRXOutcomeLinkClosureRecommendation::KeepTrinity => {
                KRXOfficialCollectionClosureRecommendation::KeepTrinity
            }
            _ => KRXOfficialCollectionClosureRecommendation::RunOutcomeLinkClosure,
        };
    }
    KRXOfficialCollectionClosureRecommendation::RunKRXDryRun
}

fn collect_blockers(
    dry_run_report: &KRXCollectionDryRunReport,
    canonical_batch_validation_report: &KRXCanonicalBatchValidationReport,
    schema_status: Option<KRXResponseSchemaStatus>,
    outcome_report: Option<&KRXOutcomeLinkClosureReport>,
    budget_exceeded: bool,
) -> Vec<String> {
    let mut blockers = Vec::new();
    if matches!(
        dry_run_report.dry_run_status,
        KRXCollectionDryRunStatus::MissingApiKey
            | KRXCollectionDryRunStatus::MissingApiKeyAndEndpointTemplate
    ) {
        blockers.push("KRX_API_KEY env var is absent in this runtime".to_string());
    }
    if matches!(
        dry_run_report.dry_run_status,
        KRXCollectionDryRunStatus::MissingEndpointTemplate
            | KRXCollectionDryRunStatus::MissingApiKeyAndEndpointTemplate
    ) {
        blockers.push("KRX_ENDPOINT_TEMPLATE env var is absent in this runtime".to_string());
    }
    if budget_exceeded {
        blockers.push("bounded storage budget exceeded".to_string());
    }
    if matches!(
        schema_status,
        Some(
            KRXResponseSchemaStatus::UnsupportedSchema
                | KRXResponseSchemaStatus::MissingRequiredField
        )
    ) {
        blockers.push("raw KRX schema drift blocks canonicalization".to_string());
    }
    if canonical_batch_validation_report.missing_provenance_count > 0 {
        blockers.push("missing provenance blocks official readiness".to_string());
    }
    if canonical_batch_validation_report.missing_preflight_count > 0 {
        blockers.push("missing preflight blocks official readiness".to_string());
    }
    if let Some(outcome_report) = outcome_report {
        if outcome_report.generated_outcome_links == 0 {
            blockers
                .push("missing outcome links keeps downstream summaries conservative".to_string());
        }
    }
    blockers
}

fn collect_warnings(
    config: &KRXOfficialCollectionClosureConfig,
    dry_run_report: &KRXCollectionDryRunReport,
    candle_sufficiency_report: &KRXCandleSufficiencyReport,
    outcome_report: Option<&KRXOutcomeLinkClosureReport>,
) -> Vec<String> {
    let mut warnings = Vec::new();
    if config.run_live_collection {
        warnings.push(
            "live KRX collection remains operator-only and is not exercised by repository tests"
                .to_string(),
        );
    }
    if !matches!(
        dry_run_report.dry_run_status,
        KRXCollectionDryRunStatus::ReadyToCollect
    ) {
        warnings.push("dry run remains conservative because auth and/or endpoint are not fully present in this runtime".to_string());
    }
    if candle_sufficiency_report.series_missing_future_window > 0 {
        warnings.push("some KRX series still lack future-window sufficiency; no-lookahead outcome linkage remains partial".to_string());
    }
    if outcome_report.is_some_and(|report| report.generated_outcome_links == 0) {
        warnings.push(
            "outcome_links_after=0 so diversity, committee, and core stay conservative".to_string(),
        );
    }
    warnings
}

fn default_output_root() -> String {
    "target/soma_krx_collection_closure".to_string()
}

fn default_true() -> bool {
    true
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
    800_000
}
