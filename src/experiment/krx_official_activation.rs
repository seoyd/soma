use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};
use crate::data::PreflightReport;
use crate::experiment::{CorePerformanceScorecardConfig, CorePerformanceScorecardRunner};
use crate::league::{
    CommitteeOfficialBenchmarkConfig, CommitteeOfficialBenchmarkRunner,
    OfficialEvidenceDiversitySweepConfig, OfficialEvidenceDiversitySweepRunner,
    OfficialEvidenceReplicationConfig, OfficialEvidenceReplicationRunner,
};

use super::krx_activation_storage::KRXActivationStorageReport;
use super::krx_auth_readiness::KRXAuthReadinessReport;
use super::krx_canonical_validation::{KRXCanonicalValidationReport, KRXCanonicalValidationStatus};
use super::krx_downstream_rerun::KRXDownstreamRerunSummary;
use super::krx_evidence_job::{KRXEvidenceJobKind, KRXEvidenceJobPlan};
use super::krx_official_activation_bundle::KRXOfficialEvidenceActivationBundle;
use super::krx_operator_actions::build_krx_operator_actions;
use super::krx_symbol_whitelist::{
    KRXSymbolEntry, KRXSymbolWhitelist, KRXSymbolWhitelistConfig, normalize_symbol,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KRXOfficialEvidenceActivationConfig {
    pub activation_id: String,
    #[serde(default)]
    pub provider_readiness_config_path: Option<String>,
    #[serde(default)]
    pub provider_readiness_report_paths: Vec<String>,
    #[serde(default)]
    pub provider_reality_report_paths: Vec<String>,
    #[serde(default)]
    pub krx_collection_plan_paths: Vec<String>,
    #[serde(default)]
    pub local_krx_canonical_csv_paths: Vec<String>,
    #[serde(default)]
    pub local_krx_provenance_paths: Vec<String>,
    #[serde(default)]
    pub local_krx_preflight_paths: Vec<String>,
    #[serde(default)]
    pub symbol_whitelist_path: Option<String>,
    #[serde(default)]
    pub barrier_profile_registry_path: Option<String>,
    #[serde(default)]
    pub official_replication_config_path: Option<String>,
    #[serde(default)]
    pub diversity_sweep_config_path: Option<String>,
    #[serde(default)]
    pub committee_official_benchmark_config_path: Option<String>,
    #[serde(default)]
    pub core_performance_config_path: Option<String>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
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
    #[serde(default = "default_true")]
    pub require_krx_api_key: bool,
    #[serde(default = "default_true")]
    pub require_krx_endpoint_template: bool,
    #[serde(default = "default_true")]
    pub require_provenance: bool,
    #[serde(default = "default_true")]
    pub require_preflight: bool,
    #[serde(default)]
    pub require_manifest: bool,
    #[serde(default)]
    pub run_provider_readiness: bool,
    #[serde(default)]
    pub run_krx_collection: bool,
    #[serde(default = "default_true")]
    pub run_local_import: bool,
    #[serde(default)]
    pub run_preflight: bool,
    #[serde(default = "default_true")]
    pub run_official_replication: bool,
    #[serde(default)]
    pub run_candle_pack: bool,
    #[serde(default)]
    pub run_candle_gap_map: bool,
    #[serde(default)]
    pub run_candle_expansion: bool,
    #[serde(default)]
    pub run_join_audit: bool,
    #[serde(default)]
    pub run_ready_match_close: bool,
    #[serde(default)]
    pub run_complete_row_close_v2: bool,
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
pub enum KRXOfficialEvidenceActivationFinalStatus {
    KRXOfficialEvidenceActivated,
    KRXCollectionReady,
    KRXAuthReadyButEndpointMissing,
    KRXAuthMissing,
    KRXCollectionBlockedByPreflight,
    KRXCollectionBlockedByBudget,
    KRXOfficialRowsImported,
    KRXOfficialRowsNeedMoreFutureWindow,
    KRXOutcomeLinksImproved,
    KRXDiversitySweepImproved,
    CommitteeBenchmarkResearchReady,
    CoreStillBlockedByOutcomeLinks,
    NoImprovement,
    NeedMoreEvidence,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum KRXOfficialEvidenceActivationRecommendation {
    SetKRXApiKey,
    SetKRXEndpointTemplate,
    RunKRXProviderReadiness,
    RunKRXOfficialCollection,
    ProvideKRXCanonicalCsv,
    ProvideKRXProvenance,
    RunKRXPreflight,
    RunOfficialReplication,
    RunCandleCoverageClose,
    RunOfficialEvidenceDiversitySweep,
    RunCorePerformance,
    MoreKRXOfficialRows,
    MoreOutcomeDiversity,
    MoreCounterfactualDepth,
    KeepTrinity,
    NeedMoreEvidence,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct KRXOfficialEvidenceActivationReport {
    pub activation_id: String,
    pub auth_readiness: KRXAuthReadinessReport,
    pub symbol_whitelist_summary: String,
    pub job_plan_summary: String,
    pub canonical_validation_reports: Vec<KRXCanonicalValidationReport>,
    #[serde(default)]
    pub official_replication_summary: Option<String>,
    #[serde(default)]
    pub candle_pack_summary: Option<String>,
    #[serde(default)]
    pub candle_gap_summary: Option<String>,
    #[serde(default)]
    pub ready_match_closure_summary: Option<String>,
    #[serde(default)]
    pub complete_row_closure_v2_summary: Option<String>,
    #[serde(default)]
    pub scaleout_summary: Option<String>,
    #[serde(default)]
    pub diversity_sweep_summary: Option<String>,
    #[serde(default)]
    pub committee_benchmark_summary: Option<String>,
    #[serde(default)]
    pub core_performance_summary: Option<String>,
    pub added_krx_canonical_csvs: usize,
    pub added_krx_official_rows: usize,
    pub added_krx_preflight_ready_rows: usize,
    pub added_krx_outcome_links: usize,
    pub added_krx_no_trade_counterfactuals: usize,
    pub added_krx_risk_denied_counterfactuals: usize,
    #[serde(default)]
    pub previous_core_status: Option<String>,
    #[serde(default)]
    pub current_core_status: Option<String>,
    #[serde(default)]
    pub previous_primary_bottleneck: Option<String>,
    #[serde(default)]
    pub current_primary_bottleneck: Option<String>,
    pub bottleneck_changed: bool,
    pub final_status: KRXOfficialEvidenceActivationFinalStatus,
    pub final_recommendation: KRXOfficialEvidenceActivationRecommendation,
    pub blockers: Vec<String>,
    pub warnings: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct KRXOfficialEvidenceActivationRunner;

#[derive(Clone, Debug, Default)]
struct ImportedArtifact {
    canonical_path: String,
    provenance_path: Option<String>,
    preflight_path: Option<String>,
    manifest_path: Option<String>,
}

impl Default for KRXOfficialEvidenceActivationConfig {
    fn default() -> Self {
        Self {
            activation_id: "krx_official_activation".to_string(),
            provider_readiness_config_path: None,
            provider_readiness_report_paths: Vec::new(),
            provider_reality_report_paths: Vec::new(),
            krx_collection_plan_paths: Vec::new(),
            local_krx_canonical_csv_paths: Vec::new(),
            local_krx_provenance_paths: Vec::new(),
            local_krx_preflight_paths: Vec::new(),
            symbol_whitelist_path: None,
            barrier_profile_registry_path: None,
            official_replication_config_path: None,
            diversity_sweep_config_path: None,
            committee_official_benchmark_config_path: None,
            core_performance_config_path: None,
            output_root: default_output_root(),
            max_symbols: default_max_symbols(),
            max_rows_per_symbol: default_max_rows_per_symbol(),
            max_requests: default_max_requests(),
            max_days: default_max_days(),
            max_bytes: default_max_bytes(),
            require_krx_api_key: true,
            require_krx_endpoint_template: true,
            require_provenance: true,
            require_preflight: true,
            require_manifest: false,
            run_provider_readiness: false,
            run_krx_collection: false,
            run_local_import: true,
            run_preflight: false,
            run_official_replication: true,
            run_candle_pack: false,
            run_candle_gap_map: false,
            run_candle_expansion: false,
            run_join_audit: false,
            run_ready_match_close: false,
            run_complete_row_close_v2: false,
            run_official_evidence_scaleout: false,
            run_official_evidence_diversity_sweep: false,
            run_committee_official_benchmark: false,
            run_core_performance: false,
            reason_codes: vec![
                ReasonCode::DeterministicPath,
                ReasonCode::KRXLocalImportPreferred,
                ReasonCode::KRXCollectionDisabledByDefault,
            ],
        }
    }
}

impl KRXOfficialEvidenceActivationConfig {
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
        PathBuf::from(&self.output_root).join(&self.activation_id)
    }

    pub fn validate_local_paths(&self) -> Vec<ReasonCode> {
        let mut reasons = Vec::new();
        for path in [
            Some(self.output_root.as_str()),
            self.provider_readiness_config_path.as_deref(),
            self.symbol_whitelist_path.as_deref(),
            self.barrier_profile_registry_path.as_deref(),
            self.official_replication_config_path.as_deref(),
            self.diversity_sweep_config_path.as_deref(),
            self.committee_official_benchmark_config_path.as_deref(),
            self.core_performance_config_path.as_deref(),
        ]
        .into_iter()
        .flatten()
        .chain(
            self.provider_readiness_report_paths
                .iter()
                .map(String::as_str),
        )
        .chain(
            self.provider_reality_report_paths
                .iter()
                .map(String::as_str),
        )
        .chain(self.krx_collection_plan_paths.iter().map(String::as_str))
        .chain(
            self.local_krx_canonical_csv_paths
                .iter()
                .map(String::as_str),
        )
        .chain(self.local_krx_provenance_paths.iter().map(String::as_str))
        .chain(self.local_krx_preflight_paths.iter().map(String::as_str))
        {
            if path.contains("://") {
                reasons.push(ReasonCode::RemotePathRejected);
            }
        }
        stable_reason_codes(&reasons)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.activation_id.trim().is_empty() {
            return Err("krx activation_id must not be empty".to_string());
        }
        if !self.validate_local_paths().is_empty() {
            return Err("krx-official-activate config path must be local".to_string());
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

impl KRXOfficialEvidenceActivationReport {
    pub fn to_text(&self) -> String {
        [
            format!("activation_id={}", self.activation_id),
            format!("final_status={:?}", self.final_status),
            format!("final_recommendation={:?}", self.final_recommendation),
            format!(
                "symbol_whitelist_summary={}",
                self.symbol_whitelist_summary.replace('\n', " | ")
            ),
            format!(
                "job_plan_summary={}",
                self.job_plan_summary.replace('\n', " | ")
            ),
            format!("added_krx_canonical_csvs={}", self.added_krx_canonical_csvs),
            format!("added_krx_official_rows={}", self.added_krx_official_rows),
            format!(
                "added_krx_preflight_ready_rows={}",
                self.added_krx_preflight_ready_rows
            ),
            format!("added_krx_outcome_links={}", self.added_krx_outcome_links),
            format!(
                "added_krx_no_trade_counterfactuals={}",
                self.added_krx_no_trade_counterfactuals
            ),
            format!(
                "added_krx_risk_denied_counterfactuals={}",
                self.added_krx_risk_denied_counterfactuals
            ),
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
            format!("blockers={}", self.blockers.join(" | ")),
            format!("warnings={}", self.warnings.join(" | ")),
        ]
        .join("\n")
    }

    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }
}

impl KRXOfficialEvidenceActivationRunner {
    pub fn run(
        &self,
        config: &KRXOfficialEvidenceActivationConfig,
    ) -> Result<KRXOfficialEvidenceActivationBundle, String> {
        config.validate()?;
        let output_dir = config.output_dir();
        let _ = fs::remove_dir_all(&output_dir);
        fs::create_dir_all(&output_dir).map_err(|err| err.to_string())?;

        let auth_readiness = KRXAuthReadinessReport::from_config(config);
        let symbol_whitelist = load_or_build_whitelist(config)?;
        let job_plan = KRXEvidenceJobPlan::build(config, &auth_readiness, &symbol_whitelist);
        let imported = self.execute_jobs(config, &job_plan)?;

        let validation_reports = build_validation_reports(config, &job_plan, &imported);
        let eligible_imports = imported
            .iter()
            .filter(|artifact| {
                validation_reports.iter().any(|report| {
                    report.canonical_csv_path == artifact.canonical_path
                        && report.official_readiness_eligible
                })
            })
            .cloned()
            .collect::<Vec<_>>();
        let budget_exceeded = !job_plan.storage_budget_summary.budget_ok;
        let operator_actions = build_krx_operator_actions(
            config,
            &auth_readiness,
            &symbol_whitelist,
            &validation_reports,
            budget_exceeded,
        );

        let mut downstream_summary = KRXDownstreamRerunSummary::default();
        let mut downstream_paths = Vec::new();
        let mut official_replication_summary = None;
        let mut candle_pack_summary = None;
        let mut diversity_sweep_summary = None;
        let mut committee_benchmark_summary = None;
        let mut core_performance_summary = None;
        let mut added_krx_official_rows = 0usize;
        let mut added_krx_outcome_links = 0usize;
        let mut added_krx_no_trade_counterfactuals = 0usize;
        let mut added_krx_risk_denied_counterfactuals = 0usize;
        let mut current_core_status = None;
        let mut current_primary_bottleneck = None;

        if config.run_official_replication && !eligible_imports.is_empty() {
            let replication = run_official_replication(config, &output_dir, &eligible_imports)?;
            downstream_summary.official_replication_ran = true;
            downstream_summary.candle_pack_ran = true;
            downstream_summary.official_rows_after =
                Some(replication.row_injection_result.official_row_count);
            added_krx_official_rows = replication.row_injection_result.official_row_count;
            official_replication_summary = Some(replication.to_text());
            candle_pack_summary = Some(replication.official_candle_coverage_report.to_text());
            downstream_paths.push(
                PathBuf::from(&output_dir)
                    .join("downstream")
                    .join("official_replication")
                    .join("krx_official_replication")
                    .join("official_replication_bundle.json")
                    .display()
                    .to_string(),
            );
        }

        if config.run_official_evidence_diversity_sweep {
            if let Some(path) = config.diversity_sweep_config_path.as_deref() {
                let mut sweep_config =
                    OfficialEvidenceDiversitySweepConfig::from_toml_path(Path::new(path))?;
                sweep_config.output_root = output_dir
                    .join("downstream/diversity")
                    .display()
                    .to_string();
                let report = OfficialEvidenceDiversitySweepRunner::default().run(&sweep_config)?;
                downstream_summary.diversity_sweep_ran = true;
                downstream_summary.diversity_status_after =
                    Some(format!("{:?}", report.diversity_sweep_report.final_status));
                downstream_summary.outcome_links_after =
                    Some(report.diversity_sweep_report.current_official_complete_rows);
                diversity_sweep_summary = Some(report.final_summary.clone());
                added_krx_outcome_links =
                    report.diversity_sweep_report.current_official_complete_rows;
                added_krx_no_trade_counterfactuals = report
                    .diversity_sweep_report
                    .added_no_trade_counterfactuals
                    .max(0) as usize;
                added_krx_risk_denied_counterfactuals = report
                    .diversity_sweep_report
                    .added_risk_denied_counterfactuals
                    .max(0) as usize;
                downstream_paths.push(
                    output_dir
                        .join("downstream/diversity")
                        .join(&sweep_config.run_id)
                        .join("official_evidence_diversity_bundle.json")
                        .display()
                        .to_string(),
                );
            } else {
                downstream_summary
                    .reason_codes
                    .push(ReasonCode::EvidenceStillInsufficient);
            }
        }

        if config.run_committee_official_benchmark {
            if let Some(path) = config.committee_official_benchmark_config_path.as_deref() {
                let mut benchmark_config =
                    CommitteeOfficialBenchmarkConfig::from_toml_path(Path::new(path))?;
                benchmark_config.output_root = output_dir
                    .join("downstream/committee")
                    .display()
                    .to_string();
                let report = CommitteeOfficialBenchmarkRunner::default().run(&benchmark_config)?;
                downstream_summary.committee_benchmark_ran = true;
                downstream_summary.committee_status_after =
                    Some(format!("{:?}", report.final_status));
                committee_benchmark_summary = Some(report.to_text());
                downstream_paths.push(
                    output_dir
                        .join("downstream/committee")
                        .join(&benchmark_config.benchmark_id)
                        .join("committee_official_benchmark_report.json")
                        .display()
                        .to_string(),
                );
            } else {
                downstream_summary
                    .reason_codes
                    .push(ReasonCode::EvidenceStillInsufficient);
            }
        }

        if config.run_core_performance {
            if let Some(path) = config.core_performance_config_path.as_deref() {
                let mut core_config =
                    CorePerformanceScorecardConfig::from_toml_path(Path::new(path))?;
                core_config.output_root = output_dir.join("downstream/core").display().to_string();
                if let Some(replication_summary_path) = downstream_paths
                    .iter()
                    .find(|path| path.contains("official_replication_bundle.json"))
                    .cloned()
                {
                    core_config
                        .official_replication_report_paths
                        .push(replication_summary_path);
                }
                let bundle = CorePerformanceScorecardRunner::default().run(&core_config)?;
                downstream_summary.core_performance_ran = true;
                downstream_summary.core_status_after =
                    Some(format!("{:?}", bundle.scorecard.final_status));
                downstream_summary.primary_bottleneck_after = Some(format!(
                    "{:?}",
                    bundle.scorecard.bottleneck_report.primary_bottleneck
                ));
                current_core_status = downstream_summary.core_status_after.clone();
                current_primary_bottleneck = downstream_summary.primary_bottleneck_after.clone();
                core_performance_summary = Some(bundle.scorecard.to_text());
                downstream_paths.push(
                    output_dir
                        .join("downstream/core")
                        .join(&core_config.scorecard_id)
                        .join("core_performance_scorecard.json")
                        .display()
                        .to_string(),
                );
            } else {
                downstream_summary
                    .reason_codes
                    .push(ReasonCode::EvidenceStillInsufficient);
            }
        }
        downstream_summary.finalize(&[]);

        let storage_report = KRXActivationStorageReport::build(
            &imported
                .iter()
                .map(|artifact| artifact.canonical_path.clone())
                .collect::<Vec<_>>(),
            &Vec::new(),
            &imported
                .iter()
                .filter_map(|artifact| artifact.provenance_path.clone())
                .collect::<Vec<_>>(),
            &imported
                .iter()
                .filter_map(|artifact| artifact.preflight_path.clone())
                .collect::<Vec<_>>(),
            &imported
                .iter()
                .filter_map(|artifact| artifact.manifest_path.clone())
                .collect::<Vec<_>>(),
            &downstream_paths,
            &Vec::new(),
            config.max_bytes,
        );

        let final_status = determine_final_status(
            config,
            &auth_readiness,
            &validation_reports,
            &storage_report,
            added_krx_official_rows,
            downstream_summary.diversity_status_after.as_deref(),
            current_core_status.as_deref(),
        );
        let final_recommendation = determine_final_recommendation(
            config,
            &auth_readiness,
            &validation_reports,
            added_krx_official_rows,
            downstream_summary.diversity_sweep_ran,
        );
        let blockers = collect_blockers(
            config,
            &auth_readiness,
            &validation_reports,
            &storage_report,
        );
        let warnings = collect_warnings(
            config,
            &auth_readiness,
            &job_plan,
            &validation_reports,
            &downstream_summary,
        );
        let reason_codes = stable_reason_codes(
            &[
                config.reason_codes.clone(),
                auth_readiness.reason_codes.clone(),
                symbol_whitelist.reason_codes.clone(),
                job_plan.reason_codes.clone(),
                validation_reports
                    .iter()
                    .flat_map(|report| report.reason_codes.clone())
                    .collect::<Vec<_>>(),
                storage_report.reason_codes.clone(),
                downstream_summary.reason_codes.clone(),
                vec![ReasonCode::KRXOfficialActivationRan],
            ]
            .concat(),
        );

        let report = KRXOfficialEvidenceActivationReport {
            activation_id: config.activation_id.clone(),
            auth_readiness: auth_readiness.clone(),
            symbol_whitelist_summary: symbol_whitelist.to_text(),
            job_plan_summary: job_plan.to_text(),
            canonical_validation_reports: validation_reports.clone(),
            official_replication_summary,
            candle_pack_summary,
            candle_gap_summary: None,
            ready_match_closure_summary: None,
            complete_row_closure_v2_summary: None,
            scaleout_summary: None,
            diversity_sweep_summary,
            committee_benchmark_summary,
            core_performance_summary,
            added_krx_canonical_csvs: imported.len(),
            added_krx_official_rows,
            added_krx_preflight_ready_rows: validation_reports
                .iter()
                .filter(|report| report.preflight_available)
                .count(),
            added_krx_outcome_links,
            added_krx_no_trade_counterfactuals,
            added_krx_risk_denied_counterfactuals,
            previous_core_status: None,
            current_core_status,
            previous_primary_bottleneck: None,
            current_primary_bottleneck,
            bottleneck_changed: false,
            final_status,
            final_recommendation,
            blockers,
            warnings,
            reason_codes,
        };
        let mut bundle = KRXOfficialEvidenceActivationBundle {
            auth_readiness_report: auth_readiness,
            symbol_whitelist,
            job_plan,
            canonical_validation_reports: validation_reports,
            operator_actions,
            downstream_rerun_summary: downstream_summary,
            activation_report: report.clone(),
            storage_report,
            final_summary: report.to_text(),
            reason_codes: report.reason_codes.clone(),
        };
        bundle.finalize_reason_codes();
        bundle.write_to_dir(&output_dir)?;
        Ok(bundle)
    }

    fn execute_jobs(
        &self,
        config: &KRXOfficialEvidenceActivationConfig,
        job_plan: &KRXEvidenceJobPlan,
    ) -> Result<Vec<ImportedArtifact>, String> {
        let mut imported = Vec::new();
        for job in &job_plan.jobs {
            if !matches!(
                job.job_kind,
                KRXEvidenceJobKind::LocalCanonicalCsvImport
                    | KRXEvidenceJobKind::ExistingCollectedCsvReuse
            ) || !matches!(
                job.status,
                super::krx_evidence_job::KRXEvidenceJobStatus::ReadyToRun
                    | super::krx_evidence_job::KRXEvidenceJobStatus::Planned
            ) {
                continue;
            }
            if !config.run_local_import {
                continue;
            }
            let canonical_src = match job.expected_canonical_csv.as_deref() {
                Some(path) => path,
                None => continue,
            };
            let import_dir = config.output_dir().join("krx_imported");
            let manifest_dir = config.output_dir().join("manifests");
            fs::create_dir_all(&import_dir).map_err(|err| err.to_string())?;
            fs::create_dir_all(&manifest_dir).map_err(|err| err.to_string())?;
            let canonical_dest = import_dir.join(file_name_or_default(
                canonical_src,
                &format!("{}_krx_1d.csv", job.normalized_symbol.to_ascii_lowercase()),
            ));
            fs::copy(canonical_src, &canonical_dest).map_err(|err| err.to_string())?;
            let provenance_dest = maybe_copy(job.expected_provenance.as_deref(), &import_dir)?;
            let preflight_dest = maybe_copy(job.expected_preflight.as_deref(), &import_dir)?;
            let manifest_dest = maybe_write_manifest(
                preflight_dest.as_deref(),
                &manifest_dir,
                &job.normalized_symbol,
            )?;
            imported.push(ImportedArtifact {
                canonical_path: canonical_dest.display().to_string(),
                provenance_path: provenance_dest,
                preflight_path: preflight_dest,
                manifest_path: manifest_dest,
            });
        }
        Ok(imported)
    }
}

fn load_or_build_whitelist(
    config: &KRXOfficialEvidenceActivationConfig,
) -> Result<KRXSymbolWhitelist, String> {
    if let Some(path) = config.symbol_whitelist_path.as_deref() {
        let config = KRXSymbolWhitelistConfig::from_toml_path(Path::new(path))?;
        config.validate()?;
        return Ok(config.build());
    }
    let symbols = config
        .local_krx_canonical_csv_paths
        .iter()
        .filter_map(|path| infer_symbol_from_path(path))
        .collect::<Vec<_>>();
    let whitelist_config = KRXSymbolWhitelistConfig {
        whitelist_id: format!("{}-derived", config.activation_id),
        symbols,
        output_root: config.output_root.clone(),
        max_symbols: config.max_symbols,
        require_market: true,
        require_provider_symbol: true,
        reason_codes: vec![ReasonCode::DeterministicPath],
    };
    whitelist_config.validate()?;
    Ok(whitelist_config.build())
}

fn infer_symbol_from_path(path: &str) -> Option<KRXSymbolEntry> {
    let normalized = path
        .chars()
        .collect::<String>()
        .split(|character: char| !character.is_ascii_alphanumeric())
        .find(|part| part.len() == 6 && part.chars().all(|character| character.is_ascii_digit()))?
        .to_string();
    Some(KRXSymbolEntry {
        provider_symbol: normalized.clone(),
        normalized_symbol: normalize_symbol(&normalized),
        market: crate::data::ProviderMarket::KoreanEquity,
        venue: Some("KRX".to_string()),
        display_name: None,
        enabled: true,
        max_rows: None,
        timeframe: "1d".to_string(),
        reason_codes: Vec::new(),
    })
}

fn build_validation_reports(
    config: &KRXOfficialEvidenceActivationConfig,
    job_plan: &KRXEvidenceJobPlan,
    imported: &[ImportedArtifact],
) -> Vec<KRXCanonicalValidationReport> {
    let mut reports = Vec::new();
    for artifact in imported {
        let job = job_plan
            .jobs
            .iter()
            .find(|job| {
                job.expected_canonical_csv.as_deref() == Some(&artifact.canonical_path)
                    || job.normalized_symbol
                        == infer_symbol_from_path(&artifact.canonical_path)
                            .as_ref()
                            .map(|entry| entry.normalized_symbol.clone())
                            .unwrap_or_default()
            })
            .cloned();
        reports.push(KRXCanonicalValidationReport::validate(
            &artifact.canonical_path,
            job.as_ref().map(|job| job.provider_symbol.clone()),
            job.as_ref().map(|job| job.normalized_symbol.clone()),
            artifact.provenance_path.as_deref(),
            artifact.preflight_path.as_deref(),
            config.require_provenance,
            config.require_preflight,
        ));
    }
    reports.sort_by(|left, right| left.canonical_csv_path.cmp(&right.canonical_csv_path));
    reports
}

fn run_official_replication(
    config: &KRXOfficialEvidenceActivationConfig,
    output_dir: &Path,
    imports: &[ImportedArtifact],
) -> Result<crate::league::OfficialEvidenceReplicationReport, String> {
    let mut replication_config =
        if let Some(path) = config.official_replication_config_path.as_deref() {
            OfficialEvidenceReplicationConfig::from_toml_path(Path::new(path))?
        } else {
            OfficialEvidenceReplicationConfig::default()
        };
    replication_config.replication_id = "krx_official_replication".to_string();
    replication_config.output_root = output_dir
        .join("downstream/official_replication")
        .display()
        .to_string();
    replication_config.provider_readiness_report_paths =
        config.provider_readiness_report_paths.clone();
    replication_config.provider_reality_report_paths = config.provider_reality_report_paths.clone();
    replication_config.official_canonical_csv_paths = imports
        .iter()
        .map(|artifact| artifact.canonical_path.clone())
        .collect();
    replication_config.official_preflight_report_paths = imports
        .iter()
        .filter_map(|artifact| artifact.preflight_path.clone())
        .collect();
    replication_config.official_provenance_paths = imports
        .iter()
        .filter_map(|artifact| artifact.provenance_path.clone())
        .collect();
    replication_config.max_rows = config.max_rows_per_symbol;
    replication_config.max_symbols = config.max_symbols;
    replication_config.max_bytes = config.max_bytes;
    replication_config.require_provenance = config.require_provenance;
    replication_config.require_preflight = config.require_preflight;
    replication_config.run_official_committee_benchmark = false;
    OfficialEvidenceReplicationRunner::default().run(&replication_config)
}

fn determine_final_status(
    config: &KRXOfficialEvidenceActivationConfig,
    auth: &KRXAuthReadinessReport,
    validation_reports: &[KRXCanonicalValidationReport],
    storage: &KRXActivationStorageReport,
    added_krx_official_rows: usize,
    diversity_status_after: Option<&str>,
    current_core_status: Option<&str>,
) -> KRXOfficialEvidenceActivationFinalStatus {
    use super::krx_auth_readiness::KRXAuthReadinessStatus::*;

    let has_official_ready_input = validation_reports
        .iter()
        .any(|report| report.official_readiness_eligible);

    if storage.budget_exceeded {
        return KRXOfficialEvidenceActivationFinalStatus::KRXCollectionBlockedByBudget;
    }
    if validation_reports.iter().any(|report| {
        matches!(
            report.validation_status,
            KRXCanonicalValidationStatus::PreflightMissing
                | KRXCanonicalValidationStatus::ProvenanceMissing
        )
    }) {
        return KRXOfficialEvidenceActivationFinalStatus::KRXCollectionBlockedByPreflight;
    }
    if added_krx_official_rows > 0
        && matches!(current_core_status, Some("CoreBlockedByOutcomeLinks"))
    {
        return KRXOfficialEvidenceActivationFinalStatus::CoreStillBlockedByOutcomeLinks;
    }
    if matches!(
        diversity_status_after,
        Some("CommitteeBenchmarkResearchReady")
    ) {
        return KRXOfficialEvidenceActivationFinalStatus::CommitteeBenchmarkResearchReady;
    }
    if matches!(
        diversity_status_after,
        Some(
            "OutcomeDiversityImproved"
                | "OfficialCompleteRowsExpanded"
                | "PlumbingValidated"
                | "TentativeSignalQualityReviewReady"
        )
    ) {
        return KRXOfficialEvidenceActivationFinalStatus::KRXDiversitySweepImproved;
    }
    if added_krx_official_rows > 0 {
        return KRXOfficialEvidenceActivationFinalStatus::KRXOfficialRowsImported;
    }
    if has_official_ready_input {
        return KRXOfficialEvidenceActivationFinalStatus::KRXOfficialEvidenceActivated;
    }
    if config.run_krx_collection {
        match auth.readiness_status {
            MissingApiKey | MissingApiKeyAndEndpointTemplate => {
                return KRXOfficialEvidenceActivationFinalStatus::KRXAuthMissing;
            }
            MissingEndpointTemplate => {
                return KRXOfficialEvidenceActivationFinalStatus::KRXAuthReadyButEndpointMissing;
            }
            _ => {}
        }
    }
    if auth.safe_to_collect_market_data {
        return KRXOfficialEvidenceActivationFinalStatus::KRXCollectionReady;
    }
    KRXOfficialEvidenceActivationFinalStatus::NeedMoreEvidence
}

fn determine_final_recommendation(
    config: &KRXOfficialEvidenceActivationConfig,
    auth: &KRXAuthReadinessReport,
    validation_reports: &[KRXCanonicalValidationReport],
    added_krx_official_rows: usize,
    diversity_ran: bool,
) -> KRXOfficialEvidenceActivationRecommendation {
    use super::krx_auth_readiness::KRXAuthReadinessStatus::*;

    let has_official_ready_input = validation_reports
        .iter()
        .any(|report| report.official_readiness_eligible);

    if validation_reports
        .iter()
        .any(|report| !report.provenance_available)
    {
        return KRXOfficialEvidenceActivationRecommendation::ProvideKRXProvenance;
    }
    if validation_reports
        .iter()
        .any(|report| !report.preflight_available)
    {
        return KRXOfficialEvidenceActivationRecommendation::RunKRXPreflight;
    }
    if added_krx_official_rows == 0 {
        if !has_official_ready_input {
            if config.run_krx_collection {
                match auth.readiness_status {
                    MissingApiKey | MissingApiKeyAndEndpointTemplate => {
                        return KRXOfficialEvidenceActivationRecommendation::SetKRXApiKey;
                    }
                    MissingEndpointTemplate => {
                        return KRXOfficialEvidenceActivationRecommendation::SetKRXEndpointTemplate;
                    }
                    _ => {
                        return KRXOfficialEvidenceActivationRecommendation::RunKRXOfficialCollection;
                    }
                }
            }
            return KRXOfficialEvidenceActivationRecommendation::ProvideKRXCanonicalCsv;
        }
        return KRXOfficialEvidenceActivationRecommendation::RunOfficialReplication;
    }
    if config.run_core_performance && !diversity_ran {
        return KRXOfficialEvidenceActivationRecommendation::RunCorePerformance;
    }
    if config.run_official_evidence_diversity_sweep && !diversity_ran {
        return KRXOfficialEvidenceActivationRecommendation::RunOfficialEvidenceDiversitySweep;
    }
    KRXOfficialEvidenceActivationRecommendation::KeepTrinity
}

fn collect_blockers(
    config: &KRXOfficialEvidenceActivationConfig,
    auth: &KRXAuthReadinessReport,
    validation_reports: &[KRXCanonicalValidationReport],
    storage: &KRXActivationStorageReport,
) -> Vec<String> {
    let mut blockers = Vec::new();
    let requires_collection_auth = config.run_krx_collection && validation_reports.is_empty();
    if requires_collection_auth && !auth.api_key_present {
        blockers.push("KRX_API_KEY is missing".to_string());
    }
    if requires_collection_auth && !auth.endpoint_template_present {
        blockers.push("KRX_ENDPOINT_TEMPLATE is missing".to_string());
    }
    if validation_reports
        .iter()
        .any(|report| !report.provenance_available)
    {
        blockers.push("missing provenance blocks official readiness".to_string());
    }
    if validation_reports
        .iter()
        .any(|report| !report.preflight_available)
    {
        blockers.push("missing preflight blocks official readiness".to_string());
    }
    if storage.budget_exceeded {
        blockers.push("storage budget exceeded".to_string());
    }
    blockers
}

fn collect_warnings(
    config: &KRXOfficialEvidenceActivationConfig,
    auth: &KRXAuthReadinessReport,
    job_plan: &KRXEvidenceJobPlan,
    validation_reports: &[KRXCanonicalValidationReport],
    downstream_summary: &KRXDownstreamRerunSummary,
) -> Vec<String> {
    let mut warnings = vec![
        "research-only warning: KRX activation is market-data-only and never implies live trading"
            .to_string(),
        "secret-safety warning: no secret values are rendered or stored by this activation flow"
            .to_string(),
    ];
    if !config.run_krx_collection {
        warnings.push(
            "provider collection remains disabled by default; local import is preferred"
                .to_string(),
        );
    }
    if !config.run_krx_collection && !auth.api_key_present {
        warnings.push("KRX_API_KEY is not set; local import can still run, but live KRX collection remains unavailable".to_string());
    }
    if !config.run_krx_collection && !auth.endpoint_template_present {
        warnings.push("KRX_ENDPOINT_TEMPLATE is not set; local import can still run, but live KRX collection remains unavailable".to_string());
    }
    if !job_plan.collection_jobs.is_empty() {
        warnings.push(
            "planned KRX collection jobs are not executed in tests and remain conservative"
                .to_string(),
        );
    }
    if validation_reports.iter().any(|report| report.gap_count > 0) {
        warnings.push("detected temporal gaps in at least one canonical KRX series".to_string());
    }
    if !downstream_summary.official_replication_ran {
        warnings.push("official replication was skipped or blocked".to_string());
    }
    warnings
}

fn maybe_copy(source: Option<&str>, dest_dir: &Path) -> Result<Option<String>, String> {
    let Some(source) = source else {
        return Ok(None);
    };
    let destination = dest_dir.join(file_name_or_default(source, "artifact.json"));
    fs::copy(source, &destination).map_err(|err| err.to_string())?;
    Ok(Some(destination.display().to_string()))
}

fn maybe_write_manifest(
    preflight_path: Option<&str>,
    dest_dir: &Path,
    normalized_symbol: &str,
) -> Result<Option<String>, String> {
    fs::create_dir_all(dest_dir).map_err(|err| err.to_string())?;
    let Some(preflight_path) = preflight_path else {
        return Ok(None);
    };
    let text = fs::read_to_string(preflight_path).map_err(|err| err.to_string())?;
    let preflight: PreflightReport = serde_json::from_str(&text).map_err(|err| err.to_string())?;
    let Some(manifest) = preflight.data_manifest_preview else {
        return Ok(None);
    };
    let manifest_path = dest_dir.join(format!(
        "{}_manifest.txt",
        normalized_symbol.to_ascii_lowercase()
    ));
    fs::write(&manifest_path, manifest.to_deterministic_string()).map_err(|err| err.to_string())?;
    Ok(Some(manifest_path.display().to_string()))
}

fn file_name_or_default(source: &str, default_name: &str) -> String {
    Path::new(source)
        .file_name()
        .and_then(|value| value.to_str())
        .map(|value| value.to_string())
        .unwrap_or_else(|| default_name.to_string())
}

fn default_output_root() -> String {
    "target/soma_krx_official_activation".to_string()
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
    2_000_000
}

fn default_true() -> bool {
    true
}
