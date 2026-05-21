use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{
    CoreCheckConfig, CoreCheckRunner, CoreReadinessReport, CoreReadinessStatus, ReasonCode,
    RuntimeMode,
};
use crate::data::{OfficialCollectionPlan, OfficialCollectionReport, OfficialCollectionRunner};
use crate::experiment::external_tabular_stage::ExternalTabularBenchmarkStageBuilder;
use crate::experiment::{
    BenchmarkStorageAudit, CoreCheckGateResult, CoreCheckedBenchmarkRecommendation,
    CoreCheckedBenchmarkReport, CoreCheckedBenchmarkStatus, ExternalTabularBenchmarkStage,
    OfficialAiBenchmarkConfig, OfficialAiBenchmarkReport, OfficialAiBenchmarkRunner,
    OfficialBenchmarkDatasetSelector, OfficialDatasetCoverageStatus,
    OfficialDatasetSelectionPolicy, SelectedOfficialDatasets, synthesize_risk_report,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoreCheckedBenchmarkConfig {
    pub benchmark_id: String,
    #[serde(default)]
    pub core_check_config: Option<CoreCheckConfig>,
    #[serde(default = "default_true")]
    pub require_core_ready: bool,
    #[serde(default = "default_allowed_statuses")]
    pub allowed_core_statuses: Vec<CoreReadinessStatus>,
    #[serde(default)]
    pub official_collection_plan_path: Option<String>,
    #[serde(default)]
    pub official_collection_report_path: Option<String>,
    #[serde(default)]
    pub run_collection: bool,
    #[serde(default = "default_true")]
    pub run_dataset_export: bool,
    #[serde(default = "default_true")]
    pub run_baseline_eval: bool,
    #[serde(default)]
    pub run_python_training: bool,
    #[serde(default)]
    pub run_external_eval: bool,
    #[serde(default)]
    pub existing_prediction_csv: Option<String>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default)]
    pub python_executable: Option<String>,
    #[serde(default)]
    pub train_script_path: Option<String>,
    #[serde(default = "default_true")]
    pub strict_schema_validation: bool,
    #[serde(default = "default_one")]
    pub min_ready_official_datasets: usize,
    #[serde(default = "default_twenty")]
    pub min_outcome_records: usize,
    #[serde(default = "default_twenty")]
    pub min_prediction_rows: usize,
    #[serde(default = "default_twenty")]
    pub min_calibration_count: usize,
    #[serde(default = "default_brier")]
    pub max_allowed_brier_score: f64,
    #[serde(default = "default_ece")]
    pub max_allowed_ece: f64,
    #[serde(default = "default_drawdown")]
    pub max_allowed_drawdown_worsening_pct: f64,
    #[serde(default = "default_storage")]
    pub max_allowed_storage_bytes: usize,
    #[serde(default)]
    pub allow_crypto_only: bool,
    #[serde(default = "default_true")]
    pub allow_missing_equity_auth: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for CoreCheckedBenchmarkConfig {
    fn default() -> Self {
        Self {
            benchmark_id: "core-checked-benchmark".to_string(),
            core_check_config: None,
            require_core_ready: true,
            allowed_core_statuses: default_allowed_statuses(),
            official_collection_plan_path: None,
            official_collection_report_path: None,
            run_collection: false,
            run_dataset_export: true,
            run_baseline_eval: true,
            run_python_training: false,
            run_external_eval: false,
            existing_prediction_csv: None,
            output_root: default_output_root(),
            python_executable: None,
            train_script_path: None,
            strict_schema_validation: true,
            min_ready_official_datasets: 1,
            min_outcome_records: 20,
            min_prediction_rows: 20,
            min_calibration_count: 20,
            max_allowed_brier_score: default_brier(),
            max_allowed_ece: default_ece(),
            max_allowed_drawdown_worsening_pct: default_drawdown(),
            max_allowed_storage_bytes: default_storage(),
            allow_crypto_only: false,
            allow_missing_equity_auth: true,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

impl CoreCheckedBenchmarkConfig {
    pub fn from_toml_str(input: &str) -> Result<Self, String> {
        toml::from_str(input).map_err(|err| err.to_string())
    }

    pub fn from_toml_path(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path).map_err(|err| err.to_string())?;
        Self::from_toml_str(&text)
    }

    pub fn to_toml_string(&self) -> Result<String, String> {
        toml::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn validate_local_paths(&self) -> Vec<ReasonCode> {
        let mut reasons = Vec::new();
        for path in [
            Some(self.output_root.as_str()),
            self.official_collection_plan_path.as_deref(),
            self.official_collection_report_path.as_deref(),
            self.existing_prediction_csv.as_deref(),
            self.python_executable.as_deref(),
            self.train_script_path.as_deref(),
        ]
        .into_iter()
        .flatten()
        {
            if path.contains("://") {
                reasons.push(ReasonCode::RemotePathRejected);
            }
        }
        if reasons.is_empty() {
            reasons.push(ReasonCode::DeterministicPath);
        }
        reasons
    }

    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.benchmark_id)
    }

    pub fn selection_policy(&self) -> OfficialDatasetSelectionPolicy {
        OfficialDatasetSelectionPolicy {
            min_ready_official_datasets: self.min_ready_official_datasets,
            allow_crypto_only: self.allow_crypto_only,
            allow_missing_equity_auth: self.allow_missing_equity_auth,
        }
    }

    pub fn to_official_ai_benchmark_config(
        &self,
        collection_report_path: Option<String>,
    ) -> OfficialAiBenchmarkConfig {
        OfficialAiBenchmarkConfig {
            benchmark_id: self.benchmark_id.clone(),
            official_collection_plan_path: self.official_collection_plan_path.clone(),
            official_collection_report_path: collection_report_path
                .or(self.official_collection_report_path.clone()),
            run_collection: false,
            run_dataset_export: self.run_dataset_export,
            run_python_training: self.run_python_training,
            run_external_prediction_eval: self.run_external_eval
                || self.existing_prediction_csv.is_some(),
            run_baseline_eval: self.run_baseline_eval,
            run_ablation_eval: false,
            output_root: self.output_root.clone(),
            python_executable: self.python_executable.clone(),
            train_script_path: self.train_script_path.clone(),
            existing_prediction_csv: self.existing_prediction_csv.clone(),
            strict_schema_validation: self.strict_schema_validation,
            min_official_ready_datasets: self.min_ready_official_datasets,
            min_outcome_records: self.min_outcome_records,
            min_calibration_count: self.min_calibration_count,
            min_comparable_models: 1,
            max_allowed_drawdown_pct: self.max_allowed_drawdown_worsening_pct,
            max_allowed_ece: self.max_allowed_ece,
            max_allowed_brier_score: self.max_allowed_brier_score,
            min_profit_factor: None,
            min_net_return_pct: None,
            allow_upbit_only: self.allow_crypto_only,
            allow_equity_missing_auth: self.allow_missing_equity_auth,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CoreCheckedBenchmarkRunner;

impl CoreCheckedBenchmarkRunner {
    pub fn run(
        &self,
        config: &CoreCheckedBenchmarkConfig,
    ) -> Result<CoreCheckedBenchmarkReport, String> {
        if config
            .validate_local_paths()
            .iter()
            .any(|reason| *reason == ReasonCode::RemotePathRejected)
        {
            return Err("core-checked benchmark paths must be local".to_string());
        }

        let core_report = config.core_check_config.clone().unwrap_or_default();
        let core_readiness_report = CoreCheckRunner::default().run(&core_report)?;
        let core_check_gate = build_core_check_gate_result(
            Some(&core_readiness_report),
            config.require_core_ready,
            &config.allowed_core_statuses,
        );
        if !core_check_gate.passed && core_report.runtime_mode != RuntimeMode::DiagnosticsOnly {
            return Ok(blocked_report(config, core_check_gate));
        }

        let collection_report = load_or_run_collection_report(config)?;
        let dataset_selection = collection_report.as_ref().map(|report| {
            OfficialBenchmarkDatasetSelector::default()
                .select_ready_entries(report, &config.selection_policy())
        });
        if dataset_selection
            .as_ref()
            .is_some_and(|selection| selection.selected_entries.is_empty())
        {
            return Ok(no_data_report(config, core_check_gate, dataset_selection));
        }

        let collection_report_path =
            write_collection_report_if_needed(config, collection_report.as_ref())?;
        let ai_report = OfficialAiBenchmarkRunner::default()
            .run(&config.to_official_ai_benchmark_config(collection_report_path));
        build_report(config, core_check_gate, dataset_selection, &ai_report)
    }
}

pub fn build_core_check_gate_result(
    core_report: Option<&CoreReadinessReport>,
    require_core_ready: bool,
    allowed_statuses: &[CoreReadinessStatus],
) -> CoreCheckGateResult {
    if let Some(core_report) = core_report {
        let passed = !require_core_ready || allowed_statuses.contains(&core_report.final_status);
        CoreCheckGateResult {
            core_check_ran: true,
            core_status: Some(core_report.final_status),
            passed,
            failed_reasons: if passed {
                Vec::new()
            } else {
                core_report.blockers.clone()
            },
            warnings: core_report.warnings.clone(),
            reason_codes: vec![if passed {
                ReasonCode::CoreReadinessBuilt
            } else {
                ReasonCode::CoreReadinessContractDrift
            }],
        }
    } else {
        CoreCheckGateResult {
            core_check_ran: false,
            core_status: None,
            passed: !require_core_ready,
            failed_reasons: if require_core_ready {
                vec!["core-check report missing".to_string()]
            } else {
                Vec::new()
            },
            warnings: Vec::new(),
            reason_codes: vec![ReasonCode::CoreReadinessAuditGap],
        }
    }
}

fn build_report(
    config: &CoreCheckedBenchmarkConfig,
    core_check_gate: CoreCheckGateResult,
    dataset_selection: Option<SelectedOfficialDatasets>,
    ai_report: &OfficialAiBenchmarkReport,
) -> Result<CoreCheckedBenchmarkReport, String> {
    let dataset_paths = crate::experiment::dataset_export_paths(ai_report);
    let dataset_bundle = if dataset_paths.is_empty() {
        None
    } else {
        Some(crate::experiment::build_dataset_bundle(
            &dataset_paths,
            config.min_outcome_records,
            config.max_allowed_storage_bytes,
        )?)
    };
    let external_stage =
        ExternalTabularBenchmarkStageBuilder::default().build(config, Some(ai_report));
    let mut storage_audit = ai_report.storage_audit.clone();
    if dataset_bundle
        .as_ref()
        .is_some_and(|bundle| bundle.storage_bytes > config.max_allowed_storage_bytes)
    {
        storage_audit.budget_exceeded = true;
        if !storage_audit
            .reason_codes
            .contains(&ReasonCode::BudgetExceeded)
        {
            storage_audit.reason_codes.push(ReasonCode::BudgetExceeded);
        }
    }
    let mut warnings = ai_report.warnings.clone();
    if dataset_selection.as_ref().is_some_and(|selection| {
        selection.coverage_status == OfficialDatasetCoverageStatus::CryptoOnly
    }) {
        warnings.push("official evidence remains crypto-only".to_string());
    }
    let risk_ai_interaction_report = Some(synthesize_risk_report(
        &config.benchmark_id,
        &ai_report.usefulness_report.risk_governor_summary,
    ));
    let (final_status, next_recommendation) =
        map_final_status(config, ai_report, &external_stage, &storage_audit);

    let report = CoreCheckedBenchmarkReport {
        benchmark_id: config.benchmark_id.clone(),
        core_check_gate,
        dataset_selection,
        dataset_bundle,
        baseline_report: Some(ai_report.usefulness_report.baseline_summary.clone()),
        external_report: ai_report.usefulness_report.external_summary.clone(),
        calibration_report: if ai_report.usefulness_report.calibration_summary.total_count > 0 {
            Some(ai_report.usefulness_report.calibration_summary.clone())
        } else {
            None
        },
        model_comparison_report: ai_report.usefulness_report.model_comparison_summary.clone(),
        risk_ai_interaction_report,
        storage_audit,
        usefulness_gate_result: ai_report.usefulness_gate_result.clone(),
        final_status,
        next_recommendation,
        blockers: ai_report.usefulness_report.blockers.clone(),
        warnings,
        reason_codes: vec![ReasonCode::OfficialAiBenchmarkRan],
    };
    report.write_to_dir(&config.output_dir())?;
    Ok(report)
}

fn blocked_report(
    config: &CoreCheckedBenchmarkConfig,
    core_check_gate: CoreCheckGateResult,
) -> CoreCheckedBenchmarkReport {
    CoreCheckedBenchmarkReport {
        benchmark_id: config.benchmark_id.clone(),
        core_check_gate,
        dataset_selection: None,
        dataset_bundle: None,
        baseline_report: None,
        external_report: None,
        calibration_report: None,
        model_comparison_report: None,
        risk_ai_interaction_report: None,
        storage_audit: BenchmarkStorageAudit {
            collection_bytes: 0,
            dataset_export_bytes: 0,
            prediction_bytes: 0,
            report_bytes: 0,
            raw_archive_bytes: 0,
            canonical_bytes: 0,
            budget_exceeded: false,
            largest_files: vec![],
            retention_actions: vec![],
            reason_codes: vec![],
        },
        usefulness_gate_result: crate::experiment::ModelUsefulnessGateResult {
            passed: false,
            failed_gates: vec![],
            warnings: vec![],
            reason_codes: vec![],
        },
        final_status: CoreCheckedBenchmarkStatus::CoreBlocked,
        next_recommendation: CoreCheckedBenchmarkRecommendation::HoldCurrentScope,
        blockers: vec!["core-check gate blocked benchmark execution".to_string()],
        warnings: vec![],
        reason_codes: vec![ReasonCode::CoreReadinessBuilt],
    }
}

fn no_data_report(
    config: &CoreCheckedBenchmarkConfig,
    core_check_gate: CoreCheckGateResult,
    dataset_selection: Option<SelectedOfficialDatasets>,
) -> CoreCheckedBenchmarkReport {
    let final_status = if dataset_selection
        .as_ref()
        .is_some_and(|selection| !selection.missing_auth_entries.is_empty())
    {
        CoreCheckedBenchmarkStatus::MissingAuth
    } else {
        CoreCheckedBenchmarkStatus::MissingOfficialData
    };
    let next_recommendation = if matches!(final_status, CoreCheckedBenchmarkStatus::MissingAuth) {
        CoreCheckedBenchmarkRecommendation::ImproveDataFirst
    } else {
        CoreCheckedBenchmarkRecommendation::MoreOfficialEvidence
    };
    CoreCheckedBenchmarkReport {
        benchmark_id: config.benchmark_id.clone(),
        core_check_gate,
        dataset_selection,
        dataset_bundle: None,
        baseline_report: None,
        external_report: None,
        calibration_report: None,
        model_comparison_report: None,
        risk_ai_interaction_report: None,
        storage_audit: BenchmarkStorageAudit {
            collection_bytes: 0,
            dataset_export_bytes: 0,
            prediction_bytes: 0,
            report_bytes: 0,
            raw_archive_bytes: 0,
            canonical_bytes: 0,
            budget_exceeded: false,
            largest_files: vec![],
            retention_actions: vec![],
            reason_codes: vec![],
        },
        usefulness_gate_result: crate::experiment::ModelUsefulnessGateResult {
            passed: false,
            failed_gates: vec![],
            warnings: vec![],
            reason_codes: vec![],
        },
        final_status,
        next_recommendation,
        blockers: vec![],
        warnings: vec![],
        reason_codes: vec![ReasonCode::AiSignalMissingOfficialData],
    }
}

fn map_final_status(
    config: &CoreCheckedBenchmarkConfig,
    ai_report: &OfficialAiBenchmarkReport,
    external_stage: &ExternalTabularBenchmarkStage,
    storage_audit: &BenchmarkStorageAudit,
) -> (
    CoreCheckedBenchmarkStatus,
    CoreCheckedBenchmarkRecommendation,
) {
    if storage_audit.budget_exceeded {
        return (
            CoreCheckedBenchmarkStatus::NeedMoreExperiments,
            CoreCheckedBenchmarkRecommendation::HoldCurrentScope,
        );
    }
    if ai_report.usefulness_report.total_outcome_records < config.min_outcome_records {
        return (
            CoreCheckedBenchmarkStatus::InsufficientOutcomes,
            CoreCheckedBenchmarkRecommendation::MoreOfficialEvidence,
        );
    }
    match ai_report.usefulness_report.status {
        crate::experiment::AiSignalStatus::PipelineOnly
        | crate::experiment::AiSignalStatus::MissingOfficialData => (
            CoreCheckedBenchmarkStatus::MissingOfficialData,
            CoreCheckedBenchmarkRecommendation::MoreOfficialEvidence,
        ),
        crate::experiment::AiSignalStatus::MissingAuth => (
            CoreCheckedBenchmarkStatus::MissingAuth,
            CoreCheckedBenchmarkRecommendation::ImproveDataFirst,
        ),
        crate::experiment::AiSignalStatus::InsufficientOutcomes => (
            CoreCheckedBenchmarkStatus::InsufficientOutcomes,
            CoreCheckedBenchmarkRecommendation::MoreOfficialEvidence,
        ),
        crate::experiment::AiSignalStatus::PoorCalibration => (
            CoreCheckedBenchmarkStatus::PoorCalibration,
            CoreCheckedBenchmarkRecommendation::ImproveSignalModelFirst,
        ),
        crate::experiment::AiSignalStatus::PoorRiskBehavior
        | crate::experiment::AiSignalStatus::RejectedByRisk => (
            CoreCheckedBenchmarkStatus::PoorRiskBehavior,
            CoreCheckedBenchmarkRecommendation::ImproveRiskGovernorFirst,
        ),
        crate::experiment::AiSignalStatus::WorseThanBaseline => (
            CoreCheckedBenchmarkStatus::WorseThanBaseline,
            CoreCheckedBenchmarkRecommendation::ImproveSignalModelFirst,
        ),
        crate::experiment::AiSignalStatus::UsefulCandidate
            if external_stage.prediction_validation_result.valid =>
        {
            (
                CoreCheckedBenchmarkStatus::ExternalTabularCandidate,
                if matches!(
                    config
                        .core_check_config
                        .as_ref()
                        .map(|cfg| cfg.sequence_dataset_ready)
                        .unwrap_or(false),
                    false
                ) {
                    CoreCheckedBenchmarkRecommendation::BuildSequenceDatasetFirst
                } else {
                    CoreCheckedBenchmarkRecommendation::ExternalModelPrototype
                },
            )
        }
        crate::experiment::AiSignalStatus::ExternalModelEvaluated
            if external_stage.prediction_validation_result.valid =>
        {
            (
                CoreCheckedBenchmarkStatus::ExternalModelEvaluated,
                CoreCheckedBenchmarkRecommendation::HoldCurrentScope,
            )
        }
        crate::experiment::AiSignalStatus::BaselineEvaluated => (
            CoreCheckedBenchmarkStatus::BaselineOnlyEvaluated,
            CoreCheckedBenchmarkRecommendation::HoldCurrentScope,
        ),
        _ => (
            CoreCheckedBenchmarkStatus::NeedMoreExperiments,
            CoreCheckedBenchmarkRecommendation::MoreOfficialEvidence,
        ),
    }
}

fn load_or_run_collection_report(
    config: &CoreCheckedBenchmarkConfig,
) -> Result<Option<OfficialCollectionReport>, String> {
    if config.run_collection {
        let plan_path = config
            .official_collection_plan_path
            .as_ref()
            .ok_or_else(|| {
                "official_collection_plan_path required when run_collection=true".to_string()
            })?;
        let plan = OfficialCollectionPlan::from_toml_path(Path::new(plan_path))?;
        return Ok(Some(OfficialCollectionRunner::default().run_plan(&plan)));
    }
    if let Some(path) = &config.official_collection_report_path {
        return Ok(Some(OfficialCollectionReport::from_json_path(Path::new(
            path,
        ))?));
    }
    Ok(None)
}

fn write_collection_report_if_needed(
    config: &CoreCheckedBenchmarkConfig,
    collection_report: Option<&OfficialCollectionReport>,
) -> Result<Option<String>, String> {
    if let Some(report) = collection_report {
        let output_dir = config.output_dir();
        let path = report.write_to_dir(&output_dir)?;
        Ok(Some(path.display().to_string()))
    } else {
        Ok(None)
    }
}

fn default_true() -> bool {
    true
}

fn default_allowed_statuses() -> Vec<CoreReadinessStatus> {
    vec![
        CoreReadinessStatus::ReadyForMoreOfficialEvidence,
        CoreReadinessStatus::ReadyForExternalModelPrototype,
        CoreReadinessStatus::ReadyForSequenceDatasetBuild,
    ]
}

fn default_output_root() -> String {
    "target/soma_core_checked_benchmark".to_string()
}

fn default_one() -> usize {
    1
}

fn default_twenty() -> usize {
    20
}

fn default_drawdown() -> f64 {
    0.05
}

fn default_ece() -> f64 {
    0.10
}

fn default_brier() -> f64 {
    0.30
}

fn default_storage() -> usize {
    16 * 1024 * 1024
}
