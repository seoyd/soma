use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{
    CoreCheckConfig, CoreCheckRunner, CoreReadinessReport, CoreReadinessStatus, ReasonCode,
};
use crate::data::{
    DataProvenance, EvidenceSourceKind, OfficialCollectionReport, YFinanceImportConfig,
};

use super::ai_benchmark::OfficialAiBenchmarkReport;
use super::source_calibration::{SourceCalibrationComparison, build_source_calibration_comparison};
use super::source_inventory::{
    SourceDatasetRecord, SourceKindDatasetInventory, build_source_kind_dataset_inventory,
};
use super::source_mismatch::{SourceMismatchAggregate, build_source_mismatch_aggregate};
use super::source_overlap::{SourceOverlapReport, build_source_overlap_report};
use super::source_risk::{
    SourceRiskInteractionComparison, build_source_risk_interaction_comparison,
};
use super::source_storage::{SourceAwareStorageAudit, build_source_aware_storage_audit};
use super::source_usefulness::{
    SourceModelUsefulnessComparison, build_source_model_usefulness_comparison,
};
use super::yahoo_research::YahooResearchEvidenceReport;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceAwareBenchmarkStatus {
    OfficialOnlyBenchmark,
    YFinanceResearchOnly,
    SourceComparisonAvailable,
    SourceMismatchHigh,
    MissingOfficialData,
    MissingYFinanceData,
    CoreBlocked,
    InsufficientOutcomes,
    PoorCalibration,
    PoorRiskBehavior,
    NeedMoreExperiments,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceAwareBenchmarkRecommendation {
    MoreOfficialEvidence,
    ImproveDataFirst,
    ImproveSignalModelFirst,
    ImproveRiskGovernorFirst,
    UseYFinanceForResearchOnly,
    CompareWithOfficialWhenAuthReady,
    HoldCurrentScope,
    BuildSequenceDatasetFirst,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SourceBenchmarkSummary {
    pub source_label: String,
    pub dataset_count: usize,
    pub total_outcome_records: usize,
    #[serde(default)]
    pub avg_net_return_pct: Option<f64>,
    #[serde(default)]
    pub avg_max_drawdown_pct: Option<f64>,
    #[serde(default)]
    pub avg_brier_score: Option<f64>,
    #[serde(default)]
    pub avg_expected_calibration_error: Option<f64>,
    #[serde(default)]
    pub denial_rate: Option<f64>,
    #[serde(default)]
    pub defensive_value: Option<f64>,
    #[serde(default)]
    pub opportunity_cost: Option<f64>,
    pub useful_candidate: bool,
    #[serde(default)]
    pub status_label: Option<String>,
    pub warnings: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceAwareBenchmarkConfig {
    pub benchmark_id: String,
    #[serde(default)]
    pub core_check_config: Option<CoreCheckConfig>,
    #[serde(default = "default_true")]
    pub require_core_ready: bool,
    #[serde(default)]
    pub official_benchmark_report_paths: Vec<String>,
    #[serde(default)]
    pub official_collection_report_paths: Vec<String>,
    #[serde(default)]
    pub yahoo_research_report_paths: Vec<String>,
    #[serde(default)]
    pub yfinance_import_configs: Vec<YFinanceImportConfig>,
    #[serde(default)]
    pub run_yfinance_imports: bool,
    #[serde(default)]
    pub run_official_benchmarks: bool,
    #[serde(default)]
    pub run_yfinance_benchmarks: bool,
    #[serde(default)]
    pub run_external_eval: bool,
    #[serde(default)]
    pub existing_prediction_csv: Option<String>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default = "default_true")]
    pub strict_schema_validation: bool,
    #[serde(default = "default_one")]
    pub min_official_ready_datasets: usize,
    #[serde(default = "default_one")]
    pub min_yfinance_research_datasets: usize,
    #[serde(default = "default_one")]
    pub min_overlap_symbols: usize,
    #[serde(default = "default_one")]
    pub min_overlap_timeframes: usize,
    #[serde(default = "default_twenty")]
    pub min_outcome_records: usize,
    #[serde(default = "default_price_drift")]
    pub max_allowed_source_price_drift_bps: f64,
    #[serde(default = "default_calibration_delta")]
    pub max_allowed_calibration_delta: f64,
    #[serde(default = "default_risk_delta")]
    pub max_allowed_risk_delta: f64,
    #[serde(default = "default_storage_budget")]
    pub max_storage_bytes: usize,
    #[serde(default = "default_true")]
    pub allow_yfinance_only_research: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SourceAwareBenchmarkReport {
    pub benchmark_id: String,
    #[serde(default)]
    pub core_check_gate: Option<CoreReadinessReport>,
    pub dataset_inventory: SourceKindDatasetInventory,
    pub overlap_report: SourceOverlapReport,
    pub source_mismatch_aggregate: SourceMismatchAggregate,
    #[serde(default)]
    pub official_summary: Option<SourceBenchmarkSummary>,
    #[serde(default)]
    pub yfinance_summary: Option<SourceBenchmarkSummary>,
    pub calibration_comparison: SourceCalibrationComparison,
    pub risk_interaction_comparison: SourceRiskInteractionComparison,
    pub model_usefulness_comparison: SourceModelUsefulnessComparison,
    pub storage_audit: SourceAwareStorageAudit,
    pub final_status: SourceAwareBenchmarkStatus,
    pub final_recommendation: SourceAwareBenchmarkRecommendation,
    pub blockers: Vec<String>,
    pub warnings: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SourceAwareBenchmarkRunner;

impl Default for SourceAwareBenchmarkConfig {
    fn default() -> Self {
        Self {
            benchmark_id: "source-aware-benchmark".to_string(),
            core_check_config: None,
            require_core_ready: true,
            official_benchmark_report_paths: Vec::new(),
            official_collection_report_paths: Vec::new(),
            yahoo_research_report_paths: Vec::new(),
            yfinance_import_configs: Vec::new(),
            run_yfinance_imports: false,
            run_official_benchmarks: false,
            run_yfinance_benchmarks: false,
            run_external_eval: false,
            existing_prediction_csv: None,
            output_root: default_output_root(),
            strict_schema_validation: true,
            min_official_ready_datasets: 1,
            min_yfinance_research_datasets: 1,
            min_overlap_symbols: 1,
            min_overlap_timeframes: 1,
            min_outcome_records: 20,
            max_allowed_source_price_drift_bps: default_price_drift(),
            max_allowed_calibration_delta: default_calibration_delta(),
            max_allowed_risk_delta: default_risk_delta(),
            max_storage_bytes: default_storage_budget(),
            allow_yfinance_only_research: true,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

impl SourceAwareBenchmarkConfig {
    pub fn from_toml_str(input: &str) -> Result<Self, String> {
        toml::from_str(input).map_err(|err| err.to_string())
    }

    pub fn from_toml_path(path: &Path) -> Result<Self, String> {
        let contents = fs::read_to_string(path).map_err(|err| err.to_string())?;
        Self::from_toml_str(&contents)
    }

    pub fn to_toml_string(&self) -> Result<String, String> {
        toml::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn validate_local_paths(&self) -> Vec<ReasonCode> {
        let mut reasons = Vec::new();
        for path in self
            .official_benchmark_report_paths
            .iter()
            .chain(self.official_collection_report_paths.iter())
            .chain(self.yahoo_research_report_paths.iter())
            .chain(self.existing_prediction_csv.iter())
            .chain(std::iter::once(&self.output_root))
        {
            if path.contains("://") {
                reasons.push(ReasonCode::LocalPathRejected);
            }
        }
        if self
            .core_check_config
            .as_ref()
            .is_some_and(|config| !config.validate_local_paths().is_empty())
        {
            reasons.push(ReasonCode::LocalPathRejected);
        }
        if self
            .yfinance_import_configs
            .iter()
            .any(|config| !config.validate_local_paths().is_empty())
        {
            reasons.push(ReasonCode::LocalPathRejected);
        }
        dedupe_reasons(reasons)
    }

    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.benchmark_id)
    }
}

impl SourceAwareBenchmarkReport {
    pub fn to_text(&self) -> String {
        [
            format!("benchmark_id={}", self.benchmark_id),
            format!("final_status={:?}", self.final_status),
            format!("final_recommendation={:?}", self.final_recommendation),
            format!(
                "official_ready_count={}",
                self.dataset_inventory.official_ready_count
            ),
            format!(
                "yfinance_benchmark_eligible_count={}",
                self.dataset_inventory.yfinance_benchmark_eligible_count
            ),
            format!("overlap_count={}", self.overlap_report.overlap_count),
            format!(
                "high_mismatch_count={}",
                self.source_mismatch_aggregate.high_severity_count
            ),
            format!(
                "calibration_consistent={}",
                self.calibration_comparison.calibration_consistent
            ),
            format!(
                "risk_behavior_consistent={}",
                self.risk_interaction_comparison.risk_behavior_consistent
            ),
            format!(
                "can_generalize_from_yfinance_to_official={}",
                self.model_usefulness_comparison
                    .can_generalize_from_yfinance_to_official
            ),
            format!("warnings={}", self.warnings.join(" | ")),
            format!("blockers={}", self.blockers.join(" | ")),
        ]
        .join("\n")
    }

    pub fn to_markdown(&self) -> String {
        [
            format!("# {}", self.benchmark_id),
            format!("- final_status: `{:?}`", self.final_status),
            format!("- final_recommendation: `{:?}`", self.final_recommendation),
            format!(
                "- official_ready_count: `{}`",
                self.dataset_inventory.official_ready_count
            ),
            format!(
                "- yfinance_benchmark_eligible_count: `{}`",
                self.dataset_inventory.yfinance_benchmark_eligible_count
            ),
            format!("- overlap_count: `{}`", self.overlap_report.overlap_count),
            format!("- warnings: {}", self.warnings.join(", ")),
            format!("- blockers: {}", self.blockers.join(", ")),
        ]
        .join("\n")
    }

    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<(), String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("source_aware_benchmark_report.json"),
            self.to_json_string()?,
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("source_aware_benchmark_report.txt"),
            self.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("source_aware_benchmark_report.md"),
            self.to_markdown(),
        )
        .map_err(|err| err.to_string())?;
        Ok(())
    }
}

impl SourceAwareBenchmarkRunner {
    pub fn run(
        &self,
        config: &SourceAwareBenchmarkConfig,
    ) -> Result<SourceAwareBenchmarkReport, String> {
        if !config.validate_local_paths().is_empty() {
            return Err("source-benchmark paths must be local".to_string());
        }

        let core_check_gate = config
            .core_check_config
            .as_ref()
            .map(|core_config| CoreCheckRunner::default().run(core_config))
            .transpose()?;
        let core_blocked = config.require_core_ready
            && core_check_gate
                .as_ref()
                .is_some_and(|report| !core_status_ready(report.final_status));

        let official_benchmark_reports =
            load_official_benchmark_reports(&config.official_benchmark_report_paths)?;
        let official_collection_reports = load_official_collection_reports(
            &config.official_collection_report_paths,
            &official_benchmark_reports,
        )?;
        let yfinance_reports = load_yahoo_research_reports(&config.yahoo_research_report_paths)?;
        let imported_yfinance_records = if config.run_yfinance_imports {
            load_yfinance_records_from_imports(&config.yfinance_import_configs)?
        } else {
            Vec::new()
        };

        let mut records = load_official_records(&official_collection_reports)?;
        let mut yfinance_records = load_yfinance_records_from_reports(&yfinance_reports)?;
        yfinance_records.extend(imported_yfinance_records);
        records.extend(yfinance_records.clone());

        let dataset_inventory = build_source_kind_dataset_inventory(&records);
        let overlap_report = build_source_overlap_report(
            &dataset_inventory.official_datasets,
            &dataset_inventory.yfinance_research_datasets,
        );
        let source_mismatch_aggregate = build_source_mismatch_aggregate(
            &overlap_report,
            &dataset_inventory.official_datasets,
            &dataset_inventory.yfinance_research_datasets,
            config.max_allowed_source_price_drift_bps,
        )?;
        let official_summary = build_official_summary(&official_benchmark_reports);
        let yfinance_summary = build_yfinance_summary(&dataset_inventory);
        let calibration_comparison = build_source_calibration_comparison(
            official_summary.as_ref(),
            yfinance_summary.as_ref(),
            config.max_allowed_calibration_delta,
        );
        let risk_interaction_comparison = build_source_risk_interaction_comparison(
            official_summary.as_ref(),
            yfinance_summary.as_ref(),
            config.max_allowed_risk_delta,
        );
        let low_mismatch = source_mismatch_aggregate.high_severity_count == 0
            && source_mismatch_aggregate.not_comparable_count == 0;
        let model_usefulness_comparison = build_source_model_usefulness_comparison(
            official_summary.as_ref(),
            yfinance_summary.as_ref(),
            low_mismatch,
            calibration_comparison.calibration_consistent,
            risk_interaction_comparison.risk_behavior_consistent,
        );
        let storage_audit = build_storage_audit(
            &dataset_inventory,
            &source_mismatch_aggregate,
            config.max_storage_bytes,
        );

        let (final_status, final_recommendation, blockers, warnings) = classify_source_benchmark(
            config,
            core_blocked,
            &dataset_inventory,
            &overlap_report,
            &source_mismatch_aggregate,
            official_summary.as_ref(),
            yfinance_summary.as_ref(),
            &calibration_comparison,
            &risk_interaction_comparison,
            &model_usefulness_comparison,
        );

        let report = SourceAwareBenchmarkReport {
            benchmark_id: config.benchmark_id.clone(),
            core_check_gate,
            dataset_inventory,
            overlap_report,
            source_mismatch_aggregate,
            official_summary,
            yfinance_summary,
            calibration_comparison,
            risk_interaction_comparison,
            model_usefulness_comparison,
            storage_audit,
            final_status,
            final_recommendation,
            blockers,
            warnings,
            reason_codes: vec![ReasonCode::SourceAwareBenchmarkBuilt],
        };
        report.write_to_dir(&config.output_dir())?;
        Ok(report)
    }
}

pub fn classify_source_benchmark(
    config: &SourceAwareBenchmarkConfig,
    core_blocked: bool,
    dataset_inventory: &SourceKindDatasetInventory,
    overlap_report: &SourceOverlapReport,
    source_mismatch_aggregate: &SourceMismatchAggregate,
    official_summary: Option<&SourceBenchmarkSummary>,
    yfinance_summary: Option<&SourceBenchmarkSummary>,
    calibration_comparison: &SourceCalibrationComparison,
    risk_interaction_comparison: &SourceRiskInteractionComparison,
    model_usefulness_comparison: &SourceModelUsefulnessComparison,
) -> (
    SourceAwareBenchmarkStatus,
    SourceAwareBenchmarkRecommendation,
    Vec<String>,
    Vec<String>,
) {
    let mut blockers = Vec::new();
    let mut warnings = calibration_comparison.warnings.clone();
    warnings.extend(risk_interaction_comparison.warnings.clone());
    warnings.extend(model_usefulness_comparison.warnings.clone());

    if core_blocked {
        blockers.push("core-check gate is not ready".to_string());
        return (
            SourceAwareBenchmarkStatus::CoreBlocked,
            SourceAwareBenchmarkRecommendation::ImproveDataFirst,
            blockers,
            warnings,
        );
    }

    if dataset_inventory.official_ready_count == 0
        && dataset_inventory.yfinance_benchmark_eligible_count
            >= config.min_yfinance_research_datasets
    {
        warnings.push("official data is missing; yfinance remains research-only".to_string());
        return (
            SourceAwareBenchmarkStatus::YFinanceResearchOnly,
            if config.allow_yfinance_only_research {
                SourceAwareBenchmarkRecommendation::UseYFinanceForResearchOnly
            } else {
                SourceAwareBenchmarkRecommendation::CompareWithOfficialWhenAuthReady
            },
            blockers,
            warnings,
        );
    }

    if dataset_inventory.official_ready_count >= config.min_official_ready_datasets
        && dataset_inventory.yfinance_benchmark_eligible_count == 0
    {
        warnings.push("yfinance comparison data is missing".to_string());
        return (
            SourceAwareBenchmarkStatus::OfficialOnlyBenchmark,
            SourceAwareBenchmarkRecommendation::MoreOfficialEvidence,
            blockers,
            warnings,
        );
    }

    if dataset_inventory.official_ready_count == 0 {
        blockers.push("no official ready datasets".to_string());
        return (
            SourceAwareBenchmarkStatus::MissingOfficialData,
            SourceAwareBenchmarkRecommendation::CompareWithOfficialWhenAuthReady,
            blockers,
            warnings,
        );
    }
    if dataset_inventory.yfinance_benchmark_eligible_count == 0 {
        blockers.push("no yfinance research datasets".to_string());
        return (
            SourceAwareBenchmarkStatus::MissingYFinanceData,
            SourceAwareBenchmarkRecommendation::MoreOfficialEvidence,
            blockers,
            warnings,
        );
    }
    if overlap_report.overlap_count < config.min_overlap_symbols {
        blockers.push("overlap symbols are insufficient".to_string());
        return (
            SourceAwareBenchmarkStatus::NeedMoreExperiments,
            SourceAwareBenchmarkRecommendation::MoreOfficialEvidence,
            blockers,
            warnings,
        );
    }
    if source_mismatch_aggregate.high_severity_count > 0 {
        blockers.push("source mismatch severity is high".to_string());
        return (
            SourceAwareBenchmarkStatus::SourceMismatchHigh,
            SourceAwareBenchmarkRecommendation::ImproveDataFirst,
            blockers,
            warnings,
        );
    }
    if !calibration_comparison.calibration_consistent {
        blockers.push("calibration comparison is inconsistent".to_string());
        return (
            SourceAwareBenchmarkStatus::PoorCalibration,
            SourceAwareBenchmarkRecommendation::ImproveSignalModelFirst,
            blockers,
            warnings,
        );
    }
    if !risk_interaction_comparison.risk_behavior_consistent {
        blockers.push("risk interaction comparison is inconsistent".to_string());
        return (
            SourceAwareBenchmarkStatus::PoorRiskBehavior,
            SourceAwareBenchmarkRecommendation::ImproveRiskGovernorFirst,
            blockers,
            warnings,
        );
    }

    let total_outcomes = official_summary
        .map(|summary| summary.total_outcome_records)
        .unwrap_or(0)
        .max(
            yfinance_summary
                .map(|summary| summary.total_outcome_records)
                .unwrap_or(0),
        );
    if total_outcomes < config.min_outcome_records {
        blockers.push("outcome records are insufficient".to_string());
        return (
            SourceAwareBenchmarkStatus::InsufficientOutcomes,
            SourceAwareBenchmarkRecommendation::BuildSequenceDatasetFirst,
            blockers,
            warnings,
        );
    }

    let recommendation = if model_usefulness_comparison.can_generalize_from_yfinance_to_official {
        SourceAwareBenchmarkRecommendation::BuildSequenceDatasetFirst
    } else if official_summary.is_some_and(|summary| summary.useful_candidate) {
        SourceAwareBenchmarkRecommendation::HoldCurrentScope
    } else {
        SourceAwareBenchmarkRecommendation::ImproveSignalModelFirst
    };

    (
        SourceAwareBenchmarkStatus::SourceComparisonAvailable,
        recommendation,
        blockers,
        warnings,
    )
}

fn load_official_benchmark_reports(
    paths: &[String],
) -> Result<Vec<OfficialAiBenchmarkReport>, String> {
    paths
        .iter()
        .map(|path| OfficialAiBenchmarkReport::from_json_path(Path::new(path)))
        .collect()
}

fn load_official_collection_reports(
    explicit_paths: &[String],
    benchmarks: &[OfficialAiBenchmarkReport],
) -> Result<Vec<OfficialCollectionReport>, String> {
    let mut seen = BTreeSet::new();
    let mut paths = explicit_paths.to_vec();
    for benchmark in benchmarks {
        if let Some(path) = &benchmark.collection_report_path {
            paths.push(path.clone());
        }
    }
    let mut reports = Vec::new();
    for path in paths {
        if seen.insert(path.clone()) {
            reports.push(OfficialCollectionReport::from_json_path(Path::new(&path))?);
        }
    }
    Ok(reports)
}

fn load_yahoo_research_reports(
    paths: &[String],
) -> Result<Vec<YahooResearchEvidenceReport>, String> {
    paths
        .iter()
        .map(|path| {
            let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
            serde_json::from_str(&text).map_err(|err| err.to_string())
        })
        .collect()
}

fn load_official_records(
    reports: &[OfficialCollectionReport],
) -> Result<Vec<SourceDatasetRecord>, String> {
    let mut records = Vec::new();
    for report in reports {
        for entry in &report.entry_reports {
            records.push(SourceDatasetRecord {
                dataset_id: entry.entry_id.clone(),
                source_kind: EvidenceSourceKind::OfficialApiCollected,
                symbol: entry.symbol.clone(),
                normalized_symbol: normalize_symbol(&entry.symbol),
                timeframe_label: format!("{:?}", entry.timeframe),
                venue: entry.venue,
                canonical_csv_path: entry.canonical_csv_path.clone(),
                manifest_path: entry.manifest_path.clone(),
                provenance_path: entry.provenance_path.clone(),
                row_count: entry.row_count,
                ready_for_evidence: entry.ready_for_evidence,
                benchmark_eligible: entry.ready_for_evidence,
                adjusted_price_policy: None,
                data_quality_score: None,
                reason_codes: entry.reason_codes.clone(),
            });
        }
    }
    Ok(records)
}

fn load_yfinance_records_from_reports(
    reports: &[YahooResearchEvidenceReport],
) -> Result<Vec<SourceDatasetRecord>, String> {
    let mut records = Vec::new();
    for report in reports {
        for (index, symbol) in report.yfinance_symbols.iter().enumerate() {
            let csv_path = report.canonical_csv_paths.get(index).cloned();
            let provenance_path = report.provenance_paths.get(index).cloned();
            let provenance = provenance_path
                .as_deref()
                .map(load_provenance)
                .transpose()?;
            records.push(SourceDatasetRecord {
                dataset_id: format!("{}-{}", report.research_id, index),
                source_kind: EvidenceSourceKind::YFinanceResearch,
                symbol: symbol.clone(),
                normalized_symbol: normalize_symbol(symbol),
                timeframe_label: csv_path
                    .as_deref()
                    .map(infer_timeframe_label)
                    .unwrap_or_else(|| "OneDay".to_string()),
                venue: None,
                canonical_csv_path: csv_path.clone(),
                manifest_path: None,
                provenance_path,
                row_count: csv_path
                    .as_deref()
                    .map(count_rows)
                    .transpose()?
                    .unwrap_or(0),
                ready_for_evidence: false,
                benchmark_eligible: provenance
                    .as_ref()
                    .and_then(|value| value.benchmark_eligible)
                    .unwrap_or_else(|| {
                        report
                            .preflight_statuses
                            .get(index)
                            .is_some_and(|status| status != "MissingFile")
                    }),
                adjusted_price_policy: None,
                data_quality_score: None,
                reason_codes: vec![ReasonCode::YFinanceResearchReportBuilt],
            });
        }
    }
    Ok(records)
}

fn load_yfinance_records_from_imports(
    imports: &[YFinanceImportConfig],
) -> Result<Vec<SourceDatasetRecord>, String> {
    imports
        .iter()
        .map(|config| {
            let provenance = config
                .provenance_path
                .as_deref()
                .map(load_provenance)
                .transpose()?;
            Ok(SourceDatasetRecord {
                dataset_id: config.import_id.clone(),
                source_kind: EvidenceSourceKind::YFinanceResearch,
                symbol: config.symbol.clone(),
                normalized_symbol: normalize_symbol(&config.symbol),
                timeframe_label: format!("{:?}", config.timeframe),
                venue: Some(config.venue),
                canonical_csv_path: Some(config.canonical_csv_path.clone()),
                manifest_path: config.manifest_path.clone(),
                provenance_path: config.provenance_path.clone(),
                row_count: count_rows(&config.canonical_csv_path)?,
                ready_for_evidence: false,
                benchmark_eligible: provenance
                    .as_ref()
                    .and_then(|value| value.benchmark_eligible)
                    .unwrap_or(true),
                adjusted_price_policy: None,
                data_quality_score: None,
                reason_codes: vec![ReasonCode::YFinanceBridgeBuilt],
            })
        })
        .collect()
}

fn build_official_summary(reports: &[OfficialAiBenchmarkReport]) -> Option<SourceBenchmarkSummary> {
    if reports.is_empty() {
        return None;
    }
    Some(SourceBenchmarkSummary {
        source_label: "OfficialApiCollected".to_string(),
        dataset_count: reports
            .iter()
            .map(|report| report.usefulness_report.official_dataset_count)
            .sum(),
        total_outcome_records: reports
            .iter()
            .map(|report| report.usefulness_report.total_outcome_records)
            .sum(),
        avg_net_return_pct: Some(average(
            &reports
                .iter()
                .map(|report| report.usefulness_report.baseline_summary.avg_net_return_pct)
                .collect::<Vec<_>>(),
        )),
        avg_max_drawdown_pct: Some(average(
            &reports
                .iter()
                .map(|report| {
                    report
                        .usefulness_report
                        .baseline_summary
                        .avg_max_drawdown_pct
                })
                .collect::<Vec<_>>(),
        )),
        avg_brier_score: Some(average(
            &reports
                .iter()
                .map(|report| report.usefulness_report.calibration_summary.avg_brier_score)
                .collect::<Vec<_>>(),
        )),
        avg_expected_calibration_error: Some(average(
            &reports
                .iter()
                .map(|report| {
                    report
                        .usefulness_report
                        .calibration_summary
                        .avg_expected_calibration_error
                })
                .collect::<Vec<_>>(),
        )),
        denial_rate: Some(average(
            &reports
                .iter()
                .map(|report| report.usefulness_report.risk_governor_summary.denial_rate)
                .collect::<Vec<_>>(),
        )),
        defensive_value: Some(average(
            &reports
                .iter()
                .map(|report| {
                    report
                        .usefulness_report
                        .risk_governor_summary
                        .defensive_value
                })
                .collect::<Vec<_>>(),
        )),
        opportunity_cost: Some(average(
            &reports
                .iter()
                .map(|report| {
                    report
                        .usefulness_report
                        .risk_governor_summary
                        .opportunity_cost
                })
                .collect::<Vec<_>>(),
        )),
        useful_candidate: reports.iter().any(|report| {
            matches!(
                report.usefulness_report.status,
                crate::experiment::AiSignalStatus::UsefulCandidate
            )
        }),
        status_label: Some(format!("{:?}", reports[0].usefulness_report.status)),
        warnings: reports
            .iter()
            .flat_map(|report| report.warnings.clone())
            .collect(),
        reason_codes: vec![ReasonCode::SourceAwareBenchmarkBuilt],
    })
}

fn build_yfinance_summary(
    inventory: &SourceKindDatasetInventory,
) -> Option<SourceBenchmarkSummary> {
    if inventory.yfinance_research_datasets.is_empty() {
        return None;
    }
    let dataset_count = inventory.yfinance_research_datasets.len();
    Some(SourceBenchmarkSummary {
        source_label: "YFinanceResearch".to_string(),
        dataset_count,
        total_outcome_records: inventory
            .yfinance_research_datasets
            .iter()
            .map(|record| record.row_count)
            .sum(),
        avg_net_return_pct: None,
        avg_max_drawdown_pct: None,
        avg_brier_score: None,
        avg_expected_calibration_error: None,
        denial_rate: None,
        defensive_value: None,
        opportunity_cost: None,
        useful_candidate: false,
        status_label: Some("ResearchOnlyDataset".to_string()),
        warnings: vec![
            "yfinance summary is research-only dataset inventory; no official-usefulness claim"
                .to_string(),
        ],
        reason_codes: vec![ReasonCode::YFinanceUnofficialEvidence],
    })
}

fn build_storage_audit(
    inventory: &SourceKindDatasetInventory,
    mismatch: &SourceMismatchAggregate,
    max_storage_bytes: usize,
) -> SourceAwareStorageAudit {
    let official_artifact_bytes = inventory
        .official_datasets
        .iter()
        .map(|record| record.row_count * 48)
        .sum();
    let yfinance_artifact_bytes = inventory
        .yfinance_research_datasets
        .iter()
        .filter_map(|record| record.canonical_csv_path.as_deref())
        .filter_map(|path| fs::metadata(path).ok())
        .map(|metadata| metadata.len() as usize)
        .sum();
    let comparison_report_bytes = mismatch.reports.len() * 256;
    let largest_artifacts = inventory
        .official_datasets
        .iter()
        .chain(inventory.yfinance_research_datasets.iter())
        .filter_map(|record| {
            record.canonical_csv_path.as_deref().and_then(|path| {
                fs::metadata(path)
                    .ok()
                    .map(|meta| (path.to_string(), meta.len()))
            })
        })
        .collect::<Vec<_>>();
    let mut largest_artifacts = largest_artifacts
        .into_iter()
        .map(|(path, size)| format!("{path}:{size}"))
        .collect::<Vec<_>>();
    largest_artifacts.sort();
    build_source_aware_storage_audit(
        official_artifact_bytes,
        yfinance_artifact_bytes,
        comparison_report_bytes,
        largest_artifacts,
        max_storage_bytes,
    )
}

fn load_provenance(path: &str) -> Result<DataProvenance, String> {
    let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
    serde_json::from_str(&text).map_err(|err| err.to_string())
}

fn count_rows(path: &str) -> Result<usize, String> {
    let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
    Ok(text
        .lines()
        .skip(1)
        .filter(|line| !line.trim().is_empty())
        .count())
}

fn infer_timeframe_label(path: &str) -> String {
    let lower = path.to_ascii_lowercase();
    if lower.contains("_1m") {
        "OneMinute".to_string()
    } else if lower.contains("_5m") {
        "FiveMinute".to_string()
    } else if lower.contains("_1h") {
        "OneHour".to_string()
    } else {
        "OneDay".to_string()
    }
}

fn normalize_symbol(symbol: &str) -> String {
    symbol
        .chars()
        .filter(|value| value.is_ascii_alphanumeric())
        .collect::<String>()
        .to_ascii_uppercase()
}

fn average(values: &[f64]) -> f64 {
    if values.is_empty() {
        0.0
    } else {
        values.iter().sum::<f64>() / values.len() as f64
    }
}

fn core_status_ready(status: CoreReadinessStatus) -> bool {
    matches!(
        status,
        CoreReadinessStatus::ReadyForMoreOfficialEvidence
            | CoreReadinessStatus::ReadyForExternalModelPrototype
            | CoreReadinessStatus::ReadyForSequenceDatasetBuild
    )
}

fn dedupe_reasons(values: Vec<ReasonCode>) -> Vec<ReasonCode> {
    let mut deduped = Vec::new();
    for value in values {
        if !deduped.contains(&value) {
            deduped.push(value);
        }
    }
    deduped
}

fn default_true() -> bool {
    true
}

fn default_one() -> usize {
    1
}

fn default_twenty() -> usize {
    20
}

fn default_output_root() -> String {
    "target/soma_source_benchmark".to_string()
}

fn default_price_drift() -> f64 {
    50.0
}

fn default_calibration_delta() -> f64 {
    0.05
}

fn default_risk_delta() -> f64 {
    0.10
}

fn default_storage_budget() -> usize {
    10 * 1024 * 1024
}
