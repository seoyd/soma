use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;
use crate::data::{DataProvenance, DataQualitySeverity, EvidenceSourceKind};

use super::ablation::{
    AblationDimension, AblationInterpretationFlag, AblationOverride, AblationResultStatus,
    AblationRunner, AblationStudyConfig, AblationStudyReport, AblationValue, AblationVariant,
};
use super::aggregate::{BatchExperimentReport, ExperimentRunStatus};
use super::batch::BatchExperimentRunner;
use super::campaign::{CampaignAggregate, ResearchCampaignReport};
use super::evidence_closure::EvidenceClosureReport;
use super::matrix::{
    DatasetBundleConfig, DatasetEntry, ExperimentMatrixConfig, ExperimentVariant,
    ExperimentVariantOverrides,
};
use super::render::{
    real_evidence_plan_update_to_text, real_evidence_report_to_markdown,
    real_evidence_report_to_text,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EvidenceCountPolicy {
    #[serde(default = "default_true")]
    pub count_synthetic_for_pipeline: bool,
    #[serde(default)]
    pub count_synthetic_for_readiness: bool,
    #[serde(default)]
    pub count_test_fixture_for_readiness: bool,
    #[serde(default = "default_true")]
    pub require_real_local_for_readiness: bool,
    #[serde(default = "default_one")]
    pub min_real_local_datasets: usize,
    #[serde(default = "default_twenty")]
    pub min_real_local_outcomes: usize,
    #[serde(default = "default_two")]
    pub min_real_local_comparable_variants: usize,
    #[serde(default = "default_min_quality")]
    pub min_real_data_quality_score: f64,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for EvidenceCountPolicy {
    fn default() -> Self {
        Self {
            count_synthetic_for_pipeline: true,
            count_synthetic_for_readiness: false,
            count_test_fixture_for_readiness: false,
            require_real_local_for_readiness: true,
            min_real_local_datasets: default_one(),
            min_real_local_outcomes: default_twenty(),
            min_real_local_comparable_variants: default_two(),
            min_real_data_quality_score: default_min_quality(),
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EvidenceSourceSummary {
    pub total_datasets: usize,
    pub real_local_datasets: usize,
    pub synthetic_fixture_datasets: usize,
    pub test_fixture_datasets: usize,
    pub unknown_datasets: usize,
    pub real_local_outcome_records: usize,
    pub synthetic_outcome_records: usize,
    pub test_fixture_outcome_records: usize,
    pub real_local_comparable_variants: usize,
    pub synthetic_comparable_variants: usize,
    pub readiness_eligible_datasets: usize,
    pub readiness_eligible_outcomes: usize,
    pub readiness_eligible_variants: usize,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RealEvidenceClosureConfig {
    pub closure_id: String,
    #[serde(default)]
    pub dataset_bundle_config_path: Option<String>,
    #[serde(default)]
    pub real_dataset_entries: Vec<DatasetEntry>,
    #[serde(default)]
    pub source_sprint15_report_path: Option<String>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default = "default_evidence_store_path")]
    pub evidence_store_path: String,
    #[serde(default = "default_one")]
    pub min_real_local_datasets: usize,
    #[serde(default = "default_twenty")]
    pub min_real_local_outcome_records: usize,
    #[serde(default = "default_two")]
    pub min_real_local_comparable_variants: usize,
    #[serde(default = "default_true")]
    pub allow_synthetic_for_pipeline_smoke: bool,
    #[serde(default)]
    pub allow_synthetic_for_readiness: bool,
    #[serde(default = "default_true")]
    pub continue_on_failure: bool,
    #[serde(default = "default_true")]
    pub strict_data_quality: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for RealEvidenceClosureConfig {
    fn default() -> Self {
        Self {
            closure_id: "sprint16-real-evidence".to_string(),
            dataset_bundle_config_path: None,
            real_dataset_entries: Vec::new(),
            source_sprint15_report_path: None,
            output_root: default_output_root(),
            evidence_store_path: default_evidence_store_path(),
            min_real_local_datasets: default_one(),
            min_real_local_outcome_records: default_twenty(),
            min_real_local_comparable_variants: default_two(),
            allow_synthetic_for_pipeline_smoke: true,
            allow_synthetic_for_readiness: false,
            continue_on_failure: true,
            strict_data_quality: true,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

impl RealEvidenceClosureConfig {
    pub fn from_toml_str(input: &str) -> Result<Self, String> {
        toml::from_str(input).map_err(|err| err.to_string())
    }

    pub fn from_toml_path(path: &Path) -> Result<Self, String> {
        let contents = fs::read_to_string(path).map_err(|err| err.to_string())?;
        Self::from_toml_str(&contents)
    }

    pub fn validate_local_paths(&self) -> Vec<ReasonCode> {
        let mut reason_codes = Vec::new();
        for path in [
            Some(self.output_root.as_str()),
            Some(self.evidence_store_path.as_str()),
            self.dataset_bundle_config_path.as_deref(),
            self.source_sprint15_report_path.as_deref(),
        ]
        .into_iter()
        .flatten()
        {
            if path.contains("://") {
                reason_codes.push(ReasonCode::LocalPathRejected);
            }
        }
        for entry in &self.real_dataset_entries {
            if entry.data_path.contains("://") {
                reason_codes.push(ReasonCode::LocalPathRejected);
            }
            if let Some(provenance) = &entry.provenance {
                reason_codes.extend(
                    provenance
                        .validate_local_only()
                        .into_iter()
                        .filter(|reason| matches!(reason, ReasonCode::LocalPathRejected)),
                );
            }
        }
        dedupe_reasons(reason_codes)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RealEvidenceSourceSummary {
    pub source_sprint15_report_path: Option<String>,
    pub sprint15_targets_closed: Option<bool>,
    pub configured_dataset_count: usize,
    pub classified_real_local_datasets: usize,
    pub classified_synthetic_datasets: usize,
    pub classified_test_datasets: usize,
    pub classified_unknown_datasets: usize,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RealEvidenceDatasetSummary {
    pub dataset_id: String,
    pub source_kind: EvidenceSourceKind,
    pub data_path: String,
    pub provenance: DataProvenance,
    pub exists_locally: bool,
    pub data_quality_score: f64,
    pub data_quality_severity: DataQualitySeverity,
    pub readiness_eligible: bool,
    pub warnings: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SourceEvidenceStatus {
    pub source_label: String,
    pub required_dataset_count: usize,
    pub required_outcome_count: usize,
    pub required_comparable_variant_count: usize,
    pub dataset_count: usize,
    pub outcome_count: usize,
    pub comparable_variant_count: usize,
    pub readiness_eligible: bool,
    pub all_targets_closed: bool,
    pub still_missing_datasets: usize,
    pub still_missing_outcomes: usize,
    pub still_missing_variants: usize,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SyntheticVsRealComparison {
    pub comparable: bool,
    pub synthetic_dataset_count: usize,
    pub real_dataset_count: usize,
    pub synthetic_outcome_count: usize,
    pub real_outcome_count: usize,
    pub delta_net_return_pct: f64,
    pub delta_max_drawdown_pct: f64,
    pub delta_denial_rate: f64,
    pub delta_no_trade_rate: f64,
    pub delta_data_quality_score: f64,
    pub mismatch_warnings: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RealEvidencePlanUpdate {
    pub previous_sprint15_targets: Vec<String>,
    pub sprint15_synthetic_closure_status: String,
    pub real_evidence_targets: Vec<String>,
    pub completed_real_items: Vec<String>,
    pub remaining_real_items: Vec<String>,
    pub next_required_items: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RealEvidenceRecommendation {
    NeedMoreExperiments,
    MissingRealLocalData,
    ImproveDataFirst,
    ImproveRiskGovernorFirst,
    ImproveSignalModelFirst,
    HoldCurrentScope,
    ReadyForSixPersonaDesignReview,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RealEvidenceClosureReport {
    pub closure_id: String,
    pub source_summary: RealEvidenceSourceSummary,
    pub real_local_dataset_summaries: Vec<RealEvidenceDatasetSummary>,
    pub synthetic_dataset_summaries: Vec<RealEvidenceDatasetSummary>,
    pub real_only_evidence_status: SourceEvidenceStatus,
    pub synthetic_evidence_status: SourceEvidenceStatus,
    pub source_evidence_summary: EvidenceSourceSummary,
    pub synthetic_vs_real_comparison: Option<SyntheticVsRealComparison>,
    pub readiness_before: String,
    pub readiness_after: String,
    pub real_evidence_plan_update: RealEvidencePlanUpdate,
    pub final_recommendation: RealEvidenceRecommendation,
    pub blockers: Vec<String>,
    pub warnings: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

impl RealEvidenceClosureReport {
    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<(), String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("real_evidence_report.json"),
            self.to_json_string()?,
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("real_evidence_report.txt"),
            real_evidence_report_to_text(self),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("real_evidence_report.md"),
            real_evidence_report_to_markdown(self),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("real_evidence_plan_update.txt"),
            real_evidence_plan_update_to_text(&self.real_evidence_plan_update),
        )
        .map_err(|err| err.to_string())?;
        Ok(())
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RealEvidenceClosureRunner {
    pub batch_runner: BatchExperimentRunner,
    pub ablation_runner: AblationRunner,
}

impl RealEvidenceClosureRunner {
    pub fn run(&self, config: &RealEvidenceClosureConfig) -> RealEvidenceClosureReport {
        let invalid = config.validate_local_paths();
        if !invalid.is_empty() {
            return minimal_report(
                config,
                Vec::new(),
                None,
                vec!["real-evidence config contains remote URL-like paths".to_string()],
                invalid,
            );
        }

        let source_sprint15_report =
            load_sprint15_report(config.source_sprint15_report_path.as_deref());
        let entries = load_dataset_entries(config).unwrap_or_default();
        let classified = classify_entries(&entries, config);
        let source_summary =
            build_source_summary(config, &classified, source_sprint15_report.as_ref());
        let policy = EvidenceCountPolicy {
            count_synthetic_for_pipeline: config.allow_synthetic_for_pipeline_smoke,
            count_synthetic_for_readiness: config.allow_synthetic_for_readiness,
            count_test_fixture_for_readiness: false,
            require_real_local_for_readiness: true,
            min_real_local_datasets: config.min_real_local_datasets,
            min_real_local_outcomes: config.min_real_local_outcome_records,
            min_real_local_comparable_variants: config.min_real_local_comparable_variants,
            min_real_data_quality_score: default_min_quality(),
            reason_codes: vec![ReasonCode::DeterministicPath],
        };

        let matrix = build_matrix_config(config, &classified);
        let batch_report = self.batch_runner.run_matrix(&matrix);
        let ablation_report = self
            .ablation_runner
            .run_study(&build_ablation_config(config, &matrix));
        let quality_by_dataset = quality_by_dataset(&batch_report);
        let classified_summaries = dataset_summaries(&classified, &quality_by_dataset, &policy);
        let real_local_dataset_summaries = classified_summaries
            .iter()
            .filter(|summary| summary.source_kind == EvidenceSourceKind::RealLocal)
            .cloned()
            .collect::<Vec<_>>();
        let synthetic_dataset_summaries = classified_summaries
            .iter()
            .filter(|summary| summary.source_kind != EvidenceSourceKind::RealLocal)
            .cloned()
            .collect::<Vec<_>>();
        let source_evidence_summary = build_source_evidence_summary(
            &classified_summaries,
            &batch_report,
            &ablation_report,
            source_sprint15_report.as_ref(),
            &policy,
        );
        let real_only_evidence_status =
            build_source_status("real-only", &source_evidence_summary, &policy, true);
        let synthetic_evidence_status =
            build_source_status("synthetic/test", &source_evidence_summary, &policy, false);
        let synthetic_vs_real_comparison = build_synthetic_vs_real_comparison(
            source_sprint15_report.as_ref(),
            &source_evidence_summary,
            &batch_report,
        );
        let readiness_before = source_sprint15_report
            .as_ref()
            .map(|report| format!("{:?}", report.final_recommendation))
            .unwrap_or_else(|| "NeedMoreExperiments".to_string());
        let final_recommendation = select_final_recommendation(
            &real_local_dataset_summaries,
            &real_only_evidence_status,
            synthetic_vs_real_comparison.as_ref(),
        );
        let readiness_after = format!("{:?}", final_recommendation);
        let real_evidence_plan_update =
            build_plan_update(source_sprint15_report.as_ref(), &real_only_evidence_status);
        let mut warnings = batch_report
            .expansion_readiness
            .warnings
            .iter()
            .cloned()
            .chain(
                classified_summaries
                    .iter()
                    .flat_map(|summary| summary.warnings.iter().cloned()),
            )
            .collect::<Vec<_>>();
        if let Some(comparison) = &synthetic_vs_real_comparison {
            warnings.extend(comparison.mismatch_warnings.clone());
        }
        let blockers = classified_summaries
            .iter()
            .filter(|summary| {
                summary.source_kind == EvidenceSourceKind::RealLocal
                    && !summary.exists_locally
                    && summary
                        .reason_codes
                        .contains(&ReasonCode::MissingRealLocalData)
            })
            .map(|summary| format!("missing real local dataset: {}", summary.dataset_id))
            .collect::<Vec<_>>();
        let mut reason_codes = config.reason_codes.clone();
        reason_codes.extend(policy.reason_codes.clone());
        reason_codes.extend(batch_report.reason_codes.clone());
        reason_codes.extend(ablation_report.reason_codes.clone());
        reason_codes.extend(source_summary.reason_codes.clone());
        reason_codes.extend(
            classified_summaries
                .iter()
                .flat_map(|summary| summary.reason_codes.iter().cloned()),
        );
        reason_codes.extend(source_evidence_summary.reason_codes.clone());
        reason_codes.extend(real_only_evidence_status.reason_codes.clone());
        reason_codes.extend(synthetic_evidence_status.reason_codes.clone());
        reason_codes.extend(real_evidence_plan_update.reason_codes.clone());
        if let Some(comparison) = &synthetic_vs_real_comparison {
            reason_codes.extend(comparison.reason_codes.clone());
        }
        reason_codes.push(ReasonCode::RealOnlyEvidenceBuilt);
        if final_recommendation == RealEvidenceRecommendation::MissingRealLocalData {
            reason_codes.push(ReasonCode::MissingRealLocalData);
        }
        let mut report = RealEvidenceClosureReport {
            closure_id: config.closure_id.clone(),
            source_summary,
            real_local_dataset_summaries,
            synthetic_dataset_summaries,
            real_only_evidence_status,
            synthetic_evidence_status,
            source_evidence_summary,
            synthetic_vs_real_comparison,
            readiness_before,
            readiness_after,
            real_evidence_plan_update,
            final_recommendation,
            blockers: dedupe_strings(blockers),
            warnings: dedupe_strings(warnings),
            reason_codes: dedupe_reasons(reason_codes),
        };
        let output_dir = Path::new(&config.output_root).join(&config.closure_id);
        if let Err(err) = report.write_to_dir(&output_dir) {
            report.reason_codes.push(ReasonCode::ReportWriteFailed);
            report.blockers.push(err);
        }
        report.reason_codes = dedupe_reasons(report.reason_codes.clone());
        report.blockers = dedupe_strings(report.blockers.clone());
        report.warnings = dedupe_strings(report.warnings.clone());
        report
    }

    pub fn run_from_config_path(
        &self,
        config_path: &Path,
    ) -> Result<RealEvidenceClosureReport, String> {
        if config_path.to_string_lossy().contains("://") {
            return Err("real-evidence config path must be local".to_string());
        }
        let config = RealEvidenceClosureConfig::from_toml_path(config_path)?;
        Ok(self.run(&config))
    }
}

#[derive(Clone)]
struct ClassifiedEntry {
    entry: DatasetEntry,
    provenance: DataProvenance,
    source_kind: EvidenceSourceKind,
}

#[derive(Deserialize)]
struct EntryListFile {
    entries: Vec<DatasetEntry>,
}

fn load_dataset_entries(config: &RealEvidenceClosureConfig) -> Result<Vec<DatasetEntry>, String> {
    let mut entries = if let Some(path) = &config.dataset_bundle_config_path {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        if let Ok(matrix) = toml::from_str::<ExperimentMatrixConfig>(&text) {
            matrix.dataset_bundle.entries
        } else if let Ok(bundle) = toml::from_str::<DatasetBundleConfig>(&text) {
            bundle.entries
        } else {
            toml::from_str::<EntryListFile>(&text)
                .map(|file| file.entries)
                .map_err(|err| err.to_string())?
        }
    } else {
        Vec::new()
    };
    entries.extend(config.real_dataset_entries.iter().cloned());
    let mut deduped = BTreeMap::<String, DatasetEntry>::new();
    for entry in entries {
        deduped.insert(entry.dataset_id.clone(), entry);
    }
    Ok(deduped.into_values().collect())
}

fn classify_entries(
    entries: &[DatasetEntry],
    config: &RealEvidenceClosureConfig,
) -> Vec<ClassifiedEntry> {
    let mut classified = Vec::new();
    for entry in entries {
        let mut provenance = entry
            .provenance
            .clone()
            .unwrap_or_else(|| DataProvenance::inferred_from_path(Some(&entry.data_path)));
        if provenance.local_path.is_none() {
            provenance.local_path = Some(entry.data_path.clone());
        }
        if provenance.source_label.is_empty() {
            provenance.source_label = entry.dataset_id.clone();
        }
        let mut source_kind = if entry.tags.iter().any(|tag| tag == "synthetic") {
            EvidenceSourceKind::SyntheticFixture
        } else if entry.tags.iter().any(|tag| tag == "generated-synthetic") {
            EvidenceSourceKind::GeneratedSynthetic
        } else if entry.tags.iter().any(|tag| tag == "fixture")
            && provenance.source_kind == EvidenceSourceKind::Unknown
        {
            EvidenceSourceKind::TestFixture
        } else {
            provenance.source_kind
        };
        if matches!(
            source_kind,
            EvidenceSourceKind::RealLocal | EvidenceSourceKind::OfficialApiCollected
        ) && entry.data_path.contains("tests/fixtures")
        {
            if entry.tags.iter().any(|tag| tag == "test-real-local") {
                provenance
                    .reason_codes
                    .push(ReasonCode::RealLocalTestOverride);
            } else {
                provenance
                    .reason_codes
                    .push(ReasonCode::ReadinessEvidenceExcluded);
                source_kind = if entry.tags.iter().any(|tag| tag == "synthetic") {
                    EvidenceSourceKind::SyntheticFixture
                } else {
                    EvidenceSourceKind::TestFixture
                };
            }
        }
        if !matches!(
            source_kind,
            EvidenceSourceKind::RealLocal | EvidenceSourceKind::OfficialApiCollected
        ) && !config.allow_synthetic_for_pipeline_smoke
        {
            provenance
                .reason_codes
                .push(ReasonCode::ReadinessEvidenceExcluded);
        }
        provenance.source_kind = source_kind;
        classified.push(ClassifiedEntry {
            entry: entry.clone(),
            provenance,
            source_kind,
        });
    }
    classified.sort_by(|left, right| left.entry.dataset_id.cmp(&right.entry.dataset_id));
    classified
}

fn build_source_summary(
    config: &RealEvidenceClosureConfig,
    classified: &[ClassifiedEntry],
    source_sprint15_report: Option<&EvidenceClosureReport>,
) -> RealEvidenceSourceSummary {
    RealEvidenceSourceSummary {
        source_sprint15_report_path: config.source_sprint15_report_path.clone(),
        sprint15_targets_closed: source_sprint15_report
            .map(|report| report.closure_status.all_targets_closed),
        configured_dataset_count: classified.len(),
        classified_real_local_datasets: classified
            .iter()
            .filter(|entry| entry.source_kind == EvidenceSourceKind::RealLocal)
            .count()
            + classified
                .iter()
                .filter(|entry| entry.source_kind == EvidenceSourceKind::OfficialApiCollected)
                .count(),
        classified_synthetic_datasets: classified
            .iter()
            .filter(|entry| {
                matches!(
                    entry.source_kind,
                    EvidenceSourceKind::SyntheticFixture | EvidenceSourceKind::GeneratedSynthetic
                )
            })
            .count(),
        classified_test_datasets: classified
            .iter()
            .filter(|entry| entry.source_kind == EvidenceSourceKind::TestFixture)
            .count(),
        classified_unknown_datasets: classified
            .iter()
            .filter(|entry| entry.source_kind == EvidenceSourceKind::Unknown)
            .count(),
        reason_codes: vec![ReasonCode::DeterministicPath],
    }
}

fn build_matrix_config(
    config: &RealEvidenceClosureConfig,
    classified: &[ClassifiedEntry],
) -> ExperimentMatrixConfig {
    ExperimentMatrixConfig {
        matrix_id: format!("{}-real-matrix", config.closure_id),
        dataset_bundle: DatasetBundleConfig {
            bundle_id: format!("{}-bundle", config.closure_id),
            entries: classified.iter().map(|entry| entry.entry.clone()).collect(),
            default_data_validation_config: crate::data::DataValidationConfig {
                strict: true,
                allow_sort_repair: false,
                allow_duplicate_drop: false,
                allow_gap: false,
                max_gap_count: 0,
                max_gap_ratio: 0.0,
                max_invalid_ratio: 0.0,
                expected_step_ms: Some(60_000),
                reason_codes: vec![ReasonCode::DeterministicPath],
            },
            default_feature_config: crate::feature::FeatureConfig {
                min_required_bars: 20,
                include_volume_features: true,
                include_spread_features: true,
                include_data_quality_feature: true,
                include_time_features: true,
                ..crate::feature::FeatureConfig::default()
            },
            default_regime_config: crate::regime::RegimeClassifierConfig {
                min_data_quality: 0.55,
                ..crate::regime::RegimeClassifierConfig::default()
            },
            default_chair_config: crate::chair::ChairConfig {
                strong_threshold: 0.35,
                weak_threshold: 0.18,
                allow_forced_contrarian: true,
                cluster_penalty_enabled: true,
                defensive_bonus_weight: 0.35,
                risk_penalty_weight: 0.50,
                groupthink_penalty_weight: 0.20,
                disagreement_penalty_weight: 0.15,
                cluster_groupthink_penalty: 0.10,
            },
            default_walk_forward_config: crate::eval::WalkForwardConfig {
                train_window_bars: 24,
                validation_window_bars: Some(4),
                test_window_bars: 10,
                step_bars: 8,
                embargo_bars: 0,
                min_train_bars: 20,
                max_folds: Some(4),
                allow_partial_last_fold: false,
            },
            default_triple_barrier_config: crate::backtest::TripleBarrierConfig {
                take_profit_pct: 0.02,
                stop_loss_pct: 0.01,
                horizon_bars: 2,
                fee_bps: 2.0,
                slippage_bps: 2.0,
                side: crate::core::Side::Long,
                use_high_low_intrabar: true,
            },
            default_cost_model: crate::backtest::CostModel {
                fee_bps: 2.0,
                slippage_bps: 2.0,
                spread_bps: Some(2.0),
                min_cost_bps: None,
            },
            default_no_trade_score_config: crate::backtest::NoTradeScoreConfig {
                avoided_loss_weight: 0.7,
                missed_gain_weight: 0.2,
            },
            default_risk_config: crate::risk::GovernorConfig {
                min_trade_value: 10_000.0,
                ..crate::risk::GovernorConfig::default()
            },
            output_root: Path::new(&config.output_root)
                .join("runs")
                .display()
                .to_string(),
            reason_codes: vec![ReasonCode::DeterministicPath],
        },
        variants: vec![ExperimentVariant {
            variant_id: "baseline_1m".to_string(),
            mode: super::config::ExperimentMode::BaselineOnly,
            overrides: ExperimentVariantOverrides {
                timeframe: Some(crate::backtest::Timeframe::OneMinute),
                ..ExperimentVariantOverrides::default()
            },
            enabled: true,
            tags: vec!["baseline".to_string()],
            reason_codes: vec![ReasonCode::DeterministicPath],
        }],
        continue_on_failure: config.continue_on_failure,
        require_all_pass: false,
        deterministic_run_id: Some(format!("{}-seed", config.closure_id)),
        reason_codes: vec![ReasonCode::DeterministicPath],
    }
}

fn build_ablation_config(
    config: &RealEvidenceClosureConfig,
    matrix: &ExperimentMatrixConfig,
) -> AblationStudyConfig {
    AblationStudyConfig {
        study_id: format!("{}-real-ablation", config.closure_id),
        description: Some("Sprint 16 real evidence ablation".to_string()),
        base_matrix_config_path: None,
        embedded_base_matrix: Some(matrix.clone()),
        output_root: Path::new(&config.output_root)
            .join("ablations")
            .display()
            .to_string(),
        require_baseline_pass: true,
        continue_on_variant_failure: config.continue_on_failure,
        variants: default_ablation_variants(),
        created_at_ms: Some(42),
        reason_codes: vec![ReasonCode::DeterministicPath],
    }
}

fn quality_by_dataset(
    report: &BatchExperimentReport,
) -> BTreeMap<String, (f64, DataQualitySeverity)> {
    let mut qualities = BTreeMap::new();
    for summary in &report.run_summaries {
        qualities
            .entry(summary.run_key.dataset_id.clone())
            .and_modify(|(score, severity): &mut (f64, DataQualitySeverity)| {
                *score = score.min(summary.data_quality_score);
                if severity_rank(summary.data_quality_severity) > severity_rank(*severity) {
                    *severity = summary.data_quality_severity;
                }
            })
            .or_insert((summary.data_quality_score, summary.data_quality_severity));
    }
    qualities
}

fn dataset_summaries(
    classified: &[ClassifiedEntry],
    quality_by_dataset: &BTreeMap<String, (f64, DataQualitySeverity)>,
    policy: &EvidenceCountPolicy,
) -> Vec<RealEvidenceDatasetSummary> {
    classified
        .iter()
        .map(|entry| {
            let exists_locally = Path::new(&entry.entry.data_path).exists();
            let (score, severity) = quality_by_dataset
                .get(&entry.entry.dataset_id)
                .copied()
                .unwrap_or((0.0, DataQualitySeverity::Unusable));
            let mut reason_codes = entry.provenance.validate_local_only();
            let mut warnings = Vec::new();
            if matches!(
                entry.source_kind,
                EvidenceSourceKind::RealLocal | EvidenceSourceKind::OfficialApiCollected
            ) && !exists_locally
            {
                reason_codes.push(ReasonCode::MissingRealLocalData);
                warnings.push("configured real local CSV is missing".to_string());
            }
            let readiness_eligible = matches!(
                entry.source_kind,
                EvidenceSourceKind::RealLocal | EvidenceSourceKind::OfficialApiCollected
            ) && exists_locally
                && matches!(
                    severity,
                    DataQualitySeverity::Good | DataQualitySeverity::Warning
                )
                && score >= policy.min_real_data_quality_score
                && ((entry.source_kind == EvidenceSourceKind::OfficialApiCollected
                    && entry.provenance.downloaded_by_soma)
                    || entry.provenance.user_supplied
                    || entry.entry.tags.iter().any(|tag| tag == "test-real-local"));
            if !readiness_eligible
                && !matches!(
                    entry.source_kind,
                    EvidenceSourceKind::RealLocal | EvidenceSourceKind::OfficialApiCollected
                )
            {
                reason_codes.push(ReasonCode::ReadinessEvidenceExcluded);
            }
            RealEvidenceDatasetSummary {
                dataset_id: entry.entry.dataset_id.clone(),
                source_kind: entry.source_kind,
                data_path: entry.entry.data_path.clone(),
                provenance: entry.provenance.clone(),
                exists_locally,
                data_quality_score: score,
                data_quality_severity: severity,
                readiness_eligible,
                warnings,
                reason_codes: dedupe_reasons(reason_codes),
            }
        })
        .collect()
}

fn build_source_evidence_summary(
    dataset_summaries: &[RealEvidenceDatasetSummary],
    batch_report: &BatchExperimentReport,
    ablation_report: &AblationStudyReport,
    source_sprint15_report: Option<&EvidenceClosureReport>,
    policy: &EvidenceCountPolicy,
) -> EvidenceSourceSummary {
    let dataset_source = dataset_summaries
        .iter()
        .map(|summary| (summary.dataset_id.clone(), summary.source_kind))
        .collect::<BTreeMap<_, _>>();
    let synthetic_extra_dataset_count = source_sprint15_report
        .map(|report| {
            report
                .added_dataset_summaries
                .iter()
                .filter(|summary| {
                    matches!(
                        summary.source,
                        super::evidence_closure::DatasetEvidenceSource::SyntheticFixture
                    )
                })
                .count()
        })
        .unwrap_or(0);
    let synthetic_extra_outcomes = source_sprint15_report
        .map(|report| report.added_outcome_summary.additional_outcome_records)
        .unwrap_or(0);
    let real_local_outcome_records = batch_report
        .run_summaries
        .iter()
        .filter(|summary| {
            matches!(
                dataset_source.get(&summary.run_key.dataset_id),
                Some(EvidenceSourceKind::RealLocal | EvidenceSourceKind::OfficialApiCollected)
            )
        })
        .map(|summary| summary.total_decisions)
        .sum::<usize>();
    let synthetic_outcome_records = batch_report
        .run_summaries
        .iter()
        .filter(|summary| {
            matches!(
                dataset_source.get(&summary.run_key.dataset_id),
                Some(EvidenceSourceKind::SyntheticFixture | EvidenceSourceKind::GeneratedSynthetic)
            )
        })
        .map(|summary| summary.total_decisions)
        .sum::<usize>()
        + synthetic_extra_outcomes;
    let test_fixture_outcome_records = batch_report
        .run_summaries
        .iter()
        .filter(|summary| {
            dataset_source.get(&summary.run_key.dataset_id)
                == Some(&EvidenceSourceKind::TestFixture)
        })
        .map(|summary| summary.total_decisions)
        .sum::<usize>();
    let real_dataset_ids = dataset_summaries
        .iter()
        .filter(|summary| {
            matches!(
                summary.source_kind,
                EvidenceSourceKind::RealLocal | EvidenceSourceKind::OfficialApiCollected
            )
        })
        .map(|summary| summary.dataset_id.clone())
        .collect::<BTreeSet<_>>();
    let synthetic_dataset_ids = dataset_summaries
        .iter()
        .filter(|summary| {
            matches!(
                summary.source_kind,
                EvidenceSourceKind::SyntheticFixture | EvidenceSourceKind::GeneratedSynthetic
            )
        })
        .map(|summary| summary.dataset_id.clone())
        .collect::<BTreeSet<_>>();
    let real_local_comparable_variants = comparable_variants_for_ids(
        ablation_report,
        &real_dataset_ids,
        policy.min_real_local_outcomes,
    );
    let synthetic_comparable_variants =
        comparable_variants_for_ids(ablation_report, &synthetic_dataset_ids, 1);
    let readiness_eligible_datasets = dataset_summaries
        .iter()
        .filter(|summary| summary.readiness_eligible)
        .count();
    let readiness_eligible_outcomes = if policy.require_real_local_for_readiness {
        real_local_outcome_records
    } else {
        real_local_outcome_records + synthetic_outcome_records + test_fixture_outcome_records
    };
    let readiness_eligible_variants = if policy.require_real_local_for_readiness {
        real_local_comparable_variants
    } else {
        real_local_comparable_variants + synthetic_comparable_variants
    };
    EvidenceSourceSummary {
        total_datasets: dataset_summaries.len() + synthetic_extra_dataset_count,
        real_local_datasets: dataset_summaries
            .iter()
            .filter(|summary| summary.source_kind == EvidenceSourceKind::RealLocal)
            .count(),
        synthetic_fixture_datasets: dataset_summaries
            .iter()
            .filter(|summary| {
                matches!(
                    summary.source_kind,
                    EvidenceSourceKind::SyntheticFixture | EvidenceSourceKind::GeneratedSynthetic
                )
            })
            .count()
            + synthetic_extra_dataset_count,
        test_fixture_datasets: dataset_summaries
            .iter()
            .filter(|summary| summary.source_kind == EvidenceSourceKind::TestFixture)
            .count(),
        unknown_datasets: dataset_summaries
            .iter()
            .filter(|summary| {
                matches!(
                    summary.source_kind,
                    EvidenceSourceKind::Unknown | EvidenceSourceKind::ExternalPredictionOnly
                )
            })
            .count(),
        real_local_outcome_records,
        synthetic_outcome_records,
        test_fixture_outcome_records,
        real_local_comparable_variants,
        synthetic_comparable_variants,
        readiness_eligible_datasets,
        readiness_eligible_outcomes,
        readiness_eligible_variants,
        reason_codes: vec![ReasonCode::RealOnlyEvidenceBuilt],
    }
}

fn build_source_status(
    label: &str,
    summary: &EvidenceSourceSummary,
    policy: &EvidenceCountPolicy,
    real_only: bool,
) -> SourceEvidenceStatus {
    let (
        dataset_count,
        outcome_count,
        variant_count,
        required_dataset_count,
        required_outcome_count,
        required_variant_count,
    ) = if real_only {
        (
            summary.readiness_eligible_datasets,
            summary.readiness_eligible_outcomes,
            summary.readiness_eligible_variants,
            policy.min_real_local_datasets,
            policy.min_real_local_outcomes,
            policy.min_real_local_comparable_variants,
        )
    } else {
        (
            summary.synthetic_fixture_datasets + summary.test_fixture_datasets,
            summary.synthetic_outcome_records + summary.test_fixture_outcome_records,
            summary.synthetic_comparable_variants,
            0,
            0,
            0,
        )
    };
    let all_targets_closed = if real_only {
        dataset_count >= required_dataset_count
            && outcome_count >= required_outcome_count
            && variant_count >= required_variant_count
    } else {
        false
    };
    SourceEvidenceStatus {
        source_label: label.to_string(),
        required_dataset_count,
        required_outcome_count,
        required_comparable_variant_count: required_variant_count,
        dataset_count,
        outcome_count,
        comparable_variant_count: variant_count,
        readiness_eligible: real_only && all_targets_closed,
        all_targets_closed,
        still_missing_datasets: required_dataset_count.saturating_sub(dataset_count),
        still_missing_outcomes: required_outcome_count.saturating_sub(outcome_count),
        still_missing_variants: required_variant_count.saturating_sub(variant_count),
        reason_codes: if real_only {
            vec![ReasonCode::RealOnlyEvidenceBuilt]
        } else {
            vec![ReasonCode::ReadinessEvidenceExcluded]
        },
    }
}

fn build_synthetic_vs_real_comparison(
    source_sprint15_report: Option<&EvidenceClosureReport>,
    source_summary: &EvidenceSourceSummary,
    batch_report: &BatchExperimentReport,
) -> Option<SyntheticVsRealComparison> {
    let synthetic_aggregate = source_sprint15_report
        .and_then(|report| load_sprint15_campaign_aggregate(report))
        .or_else(|| {
            if source_summary.synthetic_outcome_records > 0
                || source_summary.test_fixture_outcome_records > 0
            {
                Some(CampaignAggregate {
                    campaign_id: "current-synthetic-slice".to_string(),
                    matrix_count: 1,
                    total_runs: batch_report.aggregate_benchmark.total_runs,
                    passed_runs: batch_report.aggregate_benchmark.passed_runs,
                    failed_runs: batch_report.aggregate_benchmark.failed_runs,
                    skipped_runs: batch_report.aggregate_benchmark.skipped_runs,
                    usable_dataset_count: source_summary.synthetic_fixture_datasets
                        + source_summary.test_fixture_datasets,
                    total_dataset_count: source_summary.synthetic_fixture_datasets
                        + source_summary.test_fixture_datasets,
                    total_outcome_records: source_summary.synthetic_outcome_records
                        + source_summary.test_fixture_outcome_records,
                    total_executed_trades: batch_report
                        .run_summaries
                        .iter()
                        .map(|summary| summary.executed_trades)
                        .sum(),
                    total_no_trades: batch_report
                        .run_summaries
                        .iter()
                        .map(|summary| summary.no_trades)
                        .sum(),
                    total_denials: batch_report
                        .run_summaries
                        .iter()
                        .map(|summary| summary.denied_trades)
                        .sum(),
                    average_data_quality_score: batch_report
                        .aggregate_benchmark
                        .avg_data_quality_score,
                    worst_data_quality_score: batch_report
                        .aggregate_benchmark
                        .avg_data_quality_score,
                    average_net_return_pct: batch_report.aggregate_benchmark.avg_net_return_pct,
                    median_net_return_pct: batch_report.aggregate_benchmark.median_net_return_pct,
                    worst_net_return_pct: batch_report.aggregate_benchmark.worst_net_return_pct,
                    average_max_drawdown_pct: batch_report.aggregate_benchmark.avg_max_drawdown_pct,
                    worst_max_drawdown_pct: batch_report.aggregate_benchmark.worst_max_drawdown_pct,
                    average_profit_factor: batch_report.aggregate_benchmark.avg_profit_factor,
                    average_calibration_brier: None,
                    regime_coverage_count: 0,
                    unknown_regime_rate: 0.0,
                    panic_regime_rate: 0.0,
                    risk_defensive_value_total: 0.0,
                    risk_opportunity_cost_total: 0.0,
                    persona_redundancy_warning_count: 0,
                    external_model_validated_count: 0,
                    reason_codes: vec![ReasonCode::DeterministicPath],
                })
            } else {
                None
            }
        });
    let comparable = synthetic_aggregate.is_some() && source_summary.real_local_outcome_records > 0;
    let synthetic_aggregate = synthetic_aggregate?;
    let real_denial_rate = safe_ratio(
        batch_report
            .run_summaries
            .iter()
            .map(|summary| summary.denied_trades)
            .sum::<usize>() as f64,
        source_summary.real_local_outcome_records as f64,
    );
    let real_no_trade_rate = safe_ratio(
        batch_report
            .run_summaries
            .iter()
            .map(|summary| summary.no_trades)
            .sum::<usize>() as f64,
        source_summary.real_local_outcome_records as f64,
    );
    let synthetic_denial_rate = safe_ratio(
        synthetic_aggregate.total_denials as f64,
        synthetic_aggregate.total_outcome_records as f64,
    );
    let synthetic_no_trade_rate = safe_ratio(
        synthetic_aggregate.total_no_trades as f64,
        synthetic_aggregate.total_outcome_records as f64,
    );
    let mut mismatch_warnings = Vec::new();
    if !comparable {
        mismatch_warnings.push(
            "real local data is missing, so synthetic-vs-real comparison is not conclusive"
                .to_string(),
        );
    }
    if comparable
        && (synthetic_aggregate.average_net_return_pct
            - batch_report.aggregate_benchmark.avg_net_return_pct)
            .abs()
            >= 0.05
    {
        mismatch_warnings
            .push("synthetic and real average net return diverge materially".to_string());
    }
    let mut reason_codes = vec![ReasonCode::SyntheticRealComparisonBuilt];
    if mismatch_warnings
        .iter()
        .any(|warning| warning.contains("diverge"))
    {
        reason_codes.push(ReasonCode::SyntheticRealDivergence);
    }
    Some(SyntheticVsRealComparison {
        comparable,
        synthetic_dataset_count: synthetic_aggregate.usable_dataset_count,
        real_dataset_count: source_summary.real_local_datasets,
        synthetic_outcome_count: synthetic_aggregate.total_outcome_records,
        real_outcome_count: source_summary.real_local_outcome_records,
        delta_net_return_pct: batch_report.aggregate_benchmark.avg_net_return_pct
            - synthetic_aggregate.average_net_return_pct,
        delta_max_drawdown_pct: batch_report.aggregate_benchmark.avg_max_drawdown_pct
            - synthetic_aggregate.average_max_drawdown_pct,
        delta_denial_rate: real_denial_rate - synthetic_denial_rate,
        delta_no_trade_rate: real_no_trade_rate - synthetic_no_trade_rate,
        delta_data_quality_score: batch_report.aggregate_benchmark.avg_data_quality_score
            - synthetic_aggregate.average_data_quality_score,
        mismatch_warnings,
        reason_codes: dedupe_reasons(reason_codes),
    })
}

fn load_sprint15_report(path: Option<&str>) -> Option<EvidenceClosureReport> {
    let path = path?;
    let contents = fs::read_to_string(path).ok()?;
    serde_json::from_str(&contents).ok()
}

fn load_sprint15_campaign_aggregate(report: &EvidenceClosureReport) -> Option<CampaignAggregate> {
    let report_path = report.source_gap_summary.source_report_path.as_deref();
    let closure_report_path = report_path
        .and_then(|path| if path.ends_with("ablation_report.json") { None } else { Some(path) })
        .map(PathBuf::from)
        .or_else(|| {
            report
                .closure_id
                .strip_prefix("sprint15")
                .map(|_| PathBuf::from("target/soma_evidence_closure/sprint15_evidence_closure/evidence_closure_report.json"))
        })
        .or_else(|| {
            Some(PathBuf::from(
                "target/soma_evidence_closure/sprint15_evidence_closure/evidence_closure_report.json",
            ))
        })?;
    let campaigns_dir = closure_report_path.parent()?.parent()?.join("campaigns");
    let campaign_report_path = fs::read_dir(campaigns_dir)
        .ok()?
        .filter_map(|entry| {
            entry
                .ok()
                .map(|entry| entry.path().join("campaign_report.json"))
        })
        .find(|path| path.exists())?;
    ResearchCampaignReport::from_json_path(&campaign_report_path)
        .ok()
        .map(|report| report.aggregate)
}

fn build_plan_update(
    source_sprint15_report: Option<&EvidenceClosureReport>,
    real_status: &SourceEvidenceStatus,
) -> RealEvidencePlanUpdate {
    let previous_sprint15_targets = vec![
        "Sprint 15 target: +1 usable dataset".to_string(),
        "Sprint 15 target: +20 outcome records".to_string(),
        "Sprint 15 target: +2 comparable variants".to_string(),
    ];
    let sprint15_synthetic_closure_status = source_sprint15_report
        .map(|report| {
            format!(
                "targets_closed={} final_recommendation={:?}",
                report.closure_status.all_targets_closed, report.final_recommendation
            )
        })
        .unwrap_or_else(|| "source Sprint 15 report unavailable".to_string());
    let real_evidence_targets = vec![
        format!(
            "need {} real local dataset(s)",
            real_status.required_dataset_count
        ),
        format!(
            "need {} real local outcome record(s)",
            real_status.required_outcome_count
        ),
        format!(
            "need {} real local comparable variant(s)",
            real_status.required_comparable_variant_count
        ),
    ];
    let mut completed_real_items = Vec::new();
    if real_status.dataset_count > 0 {
        completed_real_items.push(format!(
            "real local datasets seen: {}",
            real_status.dataset_count
        ));
    }
    if real_status.outcome_count > 0 {
        completed_real_items.push(format!(
            "real local outcome records: {}",
            real_status.outcome_count
        ));
    }
    if real_status.comparable_variant_count > 0 {
        completed_real_items.push(format!(
            "real local comparable variants: {}",
            real_status.comparable_variant_count
        ));
    }
    let mut remaining_real_items = Vec::new();
    if real_status.still_missing_datasets > 0 {
        remaining_real_items.push(format!(
            "need {} more real local dataset(s)",
            real_status.still_missing_datasets
        ));
    }
    if real_status.still_missing_outcomes > 0 {
        remaining_real_items.push(format!(
            "need {} more real local outcome record(s)",
            real_status.still_missing_outcomes
        ));
    }
    if real_status.still_missing_variants > 0 {
        remaining_real_items.push(format!(
            "need {} more real local comparable variant(s)",
            real_status.still_missing_variants
        ));
    }
    let next_required_items = if real_status.dataset_count == 0 {
        vec![
            "provide local CSV for at least one real market symbol".to_string(),
            "ensure the CSV has enough rows for walk-forward evaluation".to_string(),
            "run real-evidence closure again".to_string(),
            "do not expand personas".to_string(),
        ]
    } else if !real_status.all_targets_closed {
        remaining_real_items.clone()
    } else {
        vec![
            "compare real-only runs against additional user-supplied datasets".to_string(),
            "keep persona expansion disabled until real-only design-review gates stay clean"
                .to_string(),
        ]
    };
    RealEvidencePlanUpdate {
        previous_sprint15_targets,
        sprint15_synthetic_closure_status,
        real_evidence_targets,
        completed_real_items,
        remaining_real_items,
        next_required_items,
        reason_codes: vec![ReasonCode::RealOnlyEvidenceBuilt],
    }
}

fn select_final_recommendation(
    real_summaries: &[RealEvidenceDatasetSummary],
    real_status: &SourceEvidenceStatus,
    comparison: Option<&SyntheticVsRealComparison>,
) -> RealEvidenceRecommendation {
    if real_summaries.is_empty() || real_summaries.iter().all(|summary| !summary.exists_locally) {
        return RealEvidenceRecommendation::MissingRealLocalData;
    }
    if real_summaries.iter().any(|summary| {
        summary.source_kind == EvidenceSourceKind::RealLocal
            && matches!(
                summary.data_quality_severity,
                DataQualitySeverity::Bad | DataQualitySeverity::Unusable
            )
    }) {
        return RealEvidenceRecommendation::ImproveDataFirst;
    }
    if !real_status.all_targets_closed {
        return RealEvidenceRecommendation::NeedMoreExperiments;
    }
    if comparison.is_some_and(|comparison| comparison.delta_denial_rate.abs() >= 0.20) {
        return RealEvidenceRecommendation::ImproveRiskGovernorFirst;
    }
    if comparison.is_some_and(|comparison| comparison.delta_net_return_pct <= -0.05) {
        return RealEvidenceRecommendation::ImproveSignalModelFirst;
    }
    RealEvidenceRecommendation::HoldCurrentScope
}

fn comparable_variants_for_ids(
    report: &AblationStudyReport,
    dataset_ids: &BTreeSet<String>,
    min_outcomes: usize,
) -> usize {
    if dataset_ids.is_empty() {
        return 0;
    }
    let baseline_scope = filtered_scope(&report.baseline.report, dataset_ids);
    let baseline_outcomes = filtered_outcomes(&report.baseline.report, dataset_ids);
    report
        .variants
        .iter()
        .filter(|variant| {
            !matches!(
                variant.status,
                AblationResultStatus::Failed | AblationResultStatus::Skipped
            )
        })
        .filter(|variant| {
            !variant
                .flags
                .contains(&AblationInterpretationFlag::NotComparable)
        })
        .filter_map(|variant| variant.report.as_ref())
        .filter(|variant_report| {
            filtered_scope(variant_report, dataset_ids) == baseline_scope
                && baseline_outcomes >= min_outcomes
                && filtered_outcomes(variant_report, dataset_ids) >= min_outcomes
                && variant_report
                    .run_summaries
                    .iter()
                    .filter(|summary| dataset_ids.contains(&summary.run_key.dataset_id))
                    .all(|summary| {
                        matches!(
                            summary.status,
                            ExperimentRunStatus::Passed | ExperimentRunStatus::Warning
                        )
                    })
        })
        .count()
}

fn filtered_scope(
    report: &BatchExperimentReport,
    dataset_ids: &BTreeSet<String>,
) -> Vec<(String, String)> {
    let mut scope = report
        .run_summaries
        .iter()
        .filter(|summary| dataset_ids.contains(&summary.run_key.dataset_id))
        .map(|summary| {
            (
                summary.run_key.dataset_id.clone(),
                summary.run_key.variant_id.clone(),
            )
        })
        .collect::<Vec<_>>();
    scope.sort();
    scope
}

fn filtered_outcomes(report: &BatchExperimentReport, dataset_ids: &BTreeSet<String>) -> usize {
    report
        .run_summaries
        .iter()
        .filter(|summary| dataset_ids.contains(&summary.run_key.dataset_id))
        .map(|summary| summary.total_decisions)
        .sum()
}

fn minimal_report(
    config: &RealEvidenceClosureConfig,
    real_summaries: Vec<RealEvidenceDatasetSummary>,
    source_sprint15_report: Option<&EvidenceClosureReport>,
    blockers: Vec<String>,
    reason_codes: Vec<ReasonCode>,
) -> RealEvidenceClosureReport {
    let source_summary = RealEvidenceSourceSummary {
        source_sprint15_report_path: config.source_sprint15_report_path.clone(),
        sprint15_targets_closed: source_sprint15_report
            .map(|report| report.closure_status.all_targets_closed),
        configured_dataset_count: real_summaries.len(),
        classified_real_local_datasets: 0,
        classified_synthetic_datasets: 0,
        classified_test_datasets: 0,
        classified_unknown_datasets: real_summaries.len(),
        reason_codes: vec![ReasonCode::DeterministicPath],
    };
    let source_evidence_summary = EvidenceSourceSummary {
        total_datasets: real_summaries.len(),
        real_local_datasets: 0,
        synthetic_fixture_datasets: 0,
        test_fixture_datasets: 0,
        unknown_datasets: real_summaries.len(),
        real_local_outcome_records: 0,
        synthetic_outcome_records: 0,
        test_fixture_outcome_records: 0,
        real_local_comparable_variants: 0,
        synthetic_comparable_variants: 0,
        readiness_eligible_datasets: 0,
        readiness_eligible_outcomes: 0,
        readiness_eligible_variants: 0,
        reason_codes: vec![ReasonCode::ReadinessEvidenceExcluded],
    };
    let real_only_evidence_status = build_source_status(
        "real-only",
        &source_evidence_summary,
        &EvidenceCountPolicy::default(),
        true,
    );
    let synthetic_evidence_status = build_source_status(
        "synthetic/test",
        &source_evidence_summary,
        &EvidenceCountPolicy::default(),
        false,
    );
    RealEvidenceClosureReport {
        closure_id: config.closure_id.clone(),
        source_summary,
        real_local_dataset_summaries: real_summaries,
        synthetic_dataset_summaries: Vec::new(),
        real_only_evidence_status,
        synthetic_evidence_status,
        source_evidence_summary,
        synthetic_vs_real_comparison: None,
        readiness_before: "NeedMoreExperiments".to_string(),
        readiness_after: "MissingRealLocalData".to_string(),
        real_evidence_plan_update: build_plan_update(
            source_sprint15_report,
            &build_source_status(
                "real-only",
                &EvidenceSourceSummary {
                    total_datasets: 0,
                    real_local_datasets: 0,
                    synthetic_fixture_datasets: 0,
                    test_fixture_datasets: 0,
                    unknown_datasets: 0,
                    real_local_outcome_records: 0,
                    synthetic_outcome_records: 0,
                    test_fixture_outcome_records: 0,
                    real_local_comparable_variants: 0,
                    synthetic_comparable_variants: 0,
                    readiness_eligible_datasets: 0,
                    readiness_eligible_outcomes: 0,
                    readiness_eligible_variants: 0,
                    reason_codes: vec![ReasonCode::ReadinessEvidenceExcluded],
                },
                &EvidenceCountPolicy::default(),
                true,
            ),
        ),
        final_recommendation: RealEvidenceRecommendation::MissingRealLocalData,
        blockers,
        warnings: Vec::new(),
        reason_codes,
    }
}

fn severity_rank(severity: DataQualitySeverity) -> usize {
    match severity {
        DataQualitySeverity::Good => 0,
        DataQualitySeverity::Warning => 1,
        DataQualitySeverity::Bad => 2,
        DataQualitySeverity::Unusable => 3,
    }
}

fn safe_ratio(numerator: f64, denominator: f64) -> f64 {
    if denominator <= 0.0 {
        0.0
    } else {
        numerator / denominator
    }
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

fn dedupe_strings(values: Vec<String>) -> Vec<String> {
    let mut deduped = Vec::new();
    for value in values {
        if !deduped.contains(&value) {
            deduped.push(value);
        }
    }
    deduped
}

fn default_ablation_variants() -> Vec<AblationVariant> {
    vec![
        AblationVariant {
            variant_id: "volume_off".to_string(),
            dimension: AblationDimension::FeatureGroup,
            overrides: vec![AblationOverride {
                target: "volume".to_string(),
                value: AblationValue::Bool(false),
            }],
            research_only: false,
            enabled: true,
            tags: vec!["real-evidence".to_string()],
            notes: None,
            reason_codes: vec![ReasonCode::DeterministicPath],
        },
        AblationVariant {
            variant_id: "higher_cost".to_string(),
            dimension: AblationDimension::CostModel,
            overrides: vec![AblationOverride {
                target: "spread_bps".to_string(),
                value: AblationValue::Float(4.0),
            }],
            research_only: false,
            enabled: true,
            tags: vec!["real-evidence".to_string()],
            notes: None,
            reason_codes: vec![ReasonCode::DeterministicPath],
        },
    ]
}

fn default_output_root() -> String {
    "target/soma_real_evidence".to_string()
}

fn default_evidence_store_path() -> String {
    "target/soma_real_evidence_store".to_string()
}

fn default_true() -> bool {
    true
}

fn default_one() -> usize {
    1
}

fn default_two() -> usize {
    2
}

fn default_twenty() -> usize {
    20
}

fn default_min_quality() -> f64 {
    0.80
}
