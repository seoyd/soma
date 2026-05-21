use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;
use crate::data::DataQualitySeverity;

use super::ablation::{
    AblationInterpretationFlag, AblationResultStatus, AblationRunner, AblationStudyConfig,
    AblationStudyReport, AblationVariant,
};
use super::aggregate::ExperimentRunStatus;
use super::campaign::{ResearchCampaignConfig, ResearchCampaignReport, ResearchCampaignRunner};
use super::decision_router::Sprint14Track;
use super::matrix::{DatasetEntry, ExperimentMatrixConfig};
use super::render::{
    evidence_closure_report_to_markdown, evidence_closure_report_to_text,
    minimum_evidence_plan_update_to_text,
};
use super::sprint14::{Sprint14Report, Sprint14Runner, Sprint14TrackSpecificReport};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EvidenceClosureConfig {
    pub closure_id: String,
    #[serde(default)]
    pub source_sprint14_report_path: Option<String>,
    #[serde(default)]
    pub source_ablation_report_path: Option<String>,
    #[serde(default)]
    pub source_campaign_config_path: Option<String>,
    #[serde(default)]
    pub embedded_campaign_config: Option<ResearchCampaignConfig>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default = "default_evidence_store_path")]
    pub evidence_store_path: String,
    #[serde(default = "default_one")]
    pub min_additional_usable_datasets: usize,
    #[serde(default = "default_twenty")]
    pub min_additional_outcome_records: usize,
    #[serde(default = "default_two")]
    pub min_additional_comparable_variants: usize,
    #[serde(default = "default_true")]
    pub allow_synthetic_dataset: bool,
    #[serde(default = "default_true")]
    pub synthetic_dataset_must_be_tagged: bool,
    #[serde(default = "default_true")]
    pub continue_on_failure: bool,
    #[serde(default = "default_true")]
    pub strict_data_quality: bool,
    #[serde(default)]
    pub ablation_variants: Vec<AblationVariant>,
    #[serde(default)]
    pub created_at_ms: Option<u64>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for EvidenceClosureConfig {
    fn default() -> Self {
        Self {
            closure_id: "sprint15-evidence-closure".to_string(),
            source_sprint14_report_path: None,
            source_ablation_report_path: None,
            source_campaign_config_path: None,
            embedded_campaign_config: None,
            output_root: default_output_root(),
            evidence_store_path: default_evidence_store_path(),
            min_additional_usable_datasets: default_one(),
            min_additional_outcome_records: default_twenty(),
            min_additional_comparable_variants: default_two(),
            allow_synthetic_dataset: true,
            synthetic_dataset_must_be_tagged: true,
            continue_on_failure: true,
            strict_data_quality: true,
            ablation_variants: default_ablation_variants(),
            created_at_ms: None,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

impl EvidenceClosureConfig {
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
        let mut invalid = Vec::new();
        for path in [
            Some(self.output_root.as_str()),
            Some(self.evidence_store_path.as_str()),
            self.source_sprint14_report_path.as_deref(),
            self.source_ablation_report_path.as_deref(),
            self.source_campaign_config_path.as_deref(),
        ]
        .into_iter()
        .flatten()
        {
            if path.contains("://") {
                invalid.push(ReasonCode::LocalPathRejected);
            }
        }
        if let Some(config) = &self.embedded_campaign_config {
            invalid.extend(config.validate_local_paths());
        }
        dedupe_reasons(invalid)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EvidenceGapTarget {
    pub target_name: String,
    pub required_count: usize,
    pub current_before_count: usize,
    pub current_after_count: usize,
    pub added_count: usize,
    pub closed: bool,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EvidenceClosureMissingCounts {
    pub usable_datasets: usize,
    pub outcome_records: usize,
    pub comparable_variants: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EvidenceClosureStatus {
    pub closure_id: String,
    pub usable_dataset_target: EvidenceGapTarget,
    pub outcome_record_target: EvidenceGapTarget,
    pub comparable_variant_target: EvidenceGapTarget,
    pub all_targets_closed: bool,
    pub partially_closed: bool,
    pub still_missing: EvidenceClosureMissingCounts,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DatasetEvidenceSource {
    RealLocalData,
    SyntheticFixture,
    TestFixture,
    UnknownSource,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AddedDatasetSummary {
    pub dataset_id: String,
    pub data_path: String,
    pub source: DatasetEvidenceSource,
    pub tags: Vec<String>,
    pub data_quality_score: f64,
    pub data_quality_severity: DataQualitySeverity,
    pub counted_as_usable: bool,
    pub warnings: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AddedOutcomeSummary {
    pub campaign_id: String,
    pub additional_outcome_records: usize,
    pub executed_trades: usize,
    pub no_trades: usize,
    pub denied_trades: usize,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AddedVariantSummary {
    pub study_id: String,
    pub additional_comparable_variants: usize,
    pub comparable_variant_ids: Vec<String>,
    pub non_comparable_variant_ids: Vec<String>,
    pub failed_variant_ids: Vec<String>,
    pub warnings: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EvidenceClosureBeforeAfter {
    pub usable_dataset_count_before: usize,
    pub usable_dataset_count_after: usize,
    pub outcome_record_count_before: usize,
    pub outcome_record_count_after: usize,
    pub comparable_variant_count_before: usize,
    pub comparable_variant_count_after: usize,
    pub additional_usable_datasets: usize,
    pub additional_outcome_records: usize,
    pub additional_comparable_variants: usize,
    pub safety_regressions: Vec<String>,
    pub warnings: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SourceGapSummary {
    pub source_report_path: Option<String>,
    pub source_study_id: Option<String>,
    pub previous_recommendation: Sprint14Track,
    pub required_additional_usable_datasets: usize,
    pub required_additional_outcome_records: usize,
    pub required_additional_comparable_variants: usize,
    pub before_usable_dataset_count: usize,
    pub before_outcome_records: usize,
    pub before_comparable_variant_count: usize,
    pub warnings: Vec<String>,
    pub blockers: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MinimumEvidencePlanUpdate {
    pub previous_plan: Vec<String>,
    pub completed_items: Vec<String>,
    pub remaining_items: Vec<String>,
    pub next_required_items: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvidenceClosureRecommendation {
    NeedMoreExperiments,
    ImproveDataFirst,
    ImproveRiskGovernorFirst,
    ImproveSignalModelFirst,
    HoldCurrentScope,
    ReadyForSixPersonaDesignReview,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EvidenceClosureReport {
    pub closure_id: String,
    pub source_gap_summary: SourceGapSummary,
    pub closure_status: EvidenceClosureStatus,
    pub added_dataset_summaries: Vec<AddedDatasetSummary>,
    pub added_outcome_summary: AddedOutcomeSummary,
    pub added_variant_summary: AddedVariantSummary,
    pub before_after_evidence: EvidenceClosureBeforeAfter,
    pub readiness_before: Sprint14Track,
    pub readiness_after: Sprint14Track,
    pub minimum_plan_update: MinimumEvidencePlanUpdate,
    pub final_recommendation: EvidenceClosureRecommendation,
    pub blockers: Vec<String>,
    pub warnings: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

impl EvidenceClosureReport {
    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<(), String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("evidence_closure_report.json"),
            self.to_json_string()?,
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("evidence_closure_report.txt"),
            evidence_closure_report_to_text(self),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("evidence_closure_report.md"),
            evidence_closure_report_to_markdown(self),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("minimum_evidence_plan_update.txt"),
            minimum_evidence_plan_update_to_text(&self.minimum_plan_update),
        )
        .map_err(|err| err.to_string())?;
        Ok(())
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct EvidenceClosureRunner {
    pub campaign_runner: ResearchCampaignRunner,
    pub ablation_runner: AblationRunner,
    pub sprint14_runner: Sprint14Runner,
}

struct SourceGapContext {
    summary: SourceGapSummary,
    source_dataset_ids: BTreeSet<String>,
}

impl EvidenceClosureRunner {
    pub fn run_closure(&self, config: &EvidenceClosureConfig) -> EvidenceClosureReport {
        let invalid = config.validate_local_paths();
        if !invalid.is_empty() {
            return minimal_report(
                config,
                source_gap_defaults(config, vec![ReasonCode::EvidenceClosureDefaultsUsed]),
                AddedOutcomeSummary {
                    campaign_id: config.closure_id.clone(),
                    additional_outcome_records: 0,
                    executed_trades: 0,
                    no_trades: 0,
                    denied_trades: 0,
                    reason_codes: vec![ReasonCode::CampaignRunFailed],
                },
                AddedVariantSummary {
                    study_id: format!("{}-ablation", config.closure_id),
                    additional_comparable_variants: 0,
                    comparable_variant_ids: Vec::new(),
                    non_comparable_variant_ids: Vec::new(),
                    failed_variant_ids: Vec::new(),
                    warnings: Vec::new(),
                    reason_codes: vec![ReasonCode::CampaignRunFailed],
                },
                Vec::new(),
                vec!["closure config contains remote URL-like paths".to_string()],
                invalid,
            );
        }

        let mut blockers = Vec::new();
        let mut warnings = Vec::new();
        let mut reason_codes = config.reason_codes.clone();

        let source_context = self.load_source_gap_context(config);
        warnings.extend(source_context.summary.warnings.clone());
        blockers.extend(source_context.summary.blockers.clone());
        reason_codes.extend(source_context.summary.reason_codes.clone());

        let campaign_config = match load_campaign_config(config) {
            Ok(campaign) => campaign,
            Err(err) => {
                blockers.push(err);
                return minimal_report(
                    config,
                    source_context.summary,
                    AddedOutcomeSummary {
                        campaign_id: config.closure_id.clone(),
                        additional_outcome_records: 0,
                        executed_trades: 0,
                        no_trades: 0,
                        denied_trades: 0,
                        reason_codes: vec![ReasonCode::CampaignRunFailed],
                    },
                    AddedVariantSummary {
                        study_id: format!("{}-ablation", config.closure_id),
                        additional_comparable_variants: 0,
                        comparable_variant_ids: Vec::new(),
                        non_comparable_variant_ids: Vec::new(),
                        failed_variant_ids: Vec::new(),
                        warnings: Vec::new(),
                        reason_codes: vec![ReasonCode::CampaignRunFailed],
                    },
                    Vec::new(),
                    blockers,
                    dedupe_reasons(reason_codes),
                );
            }
        };
        let matrices = match load_campaign_matrices(&campaign_config) {
            Ok(matrices) => matrices,
            Err(err) => {
                blockers.push(err);
                return minimal_report(
                    config,
                    source_context.summary,
                    AddedOutcomeSummary {
                        campaign_id: campaign_config.campaign_id.clone(),
                        additional_outcome_records: 0,
                        executed_trades: 0,
                        no_trades: 0,
                        denied_trades: 0,
                        reason_codes: vec![ReasonCode::CampaignMatrixLoadFailed],
                    },
                    AddedVariantSummary {
                        study_id: format!("{}-ablation", config.closure_id),
                        additional_comparable_variants: 0,
                        comparable_variant_ids: Vec::new(),
                        non_comparable_variant_ids: Vec::new(),
                        failed_variant_ids: Vec::new(),
                        warnings: Vec::new(),
                        reason_codes: vec![ReasonCode::CampaignMatrixLoadFailed],
                    },
                    Vec::new(),
                    blockers,
                    vec![ReasonCode::CampaignMatrixLoadFailed],
                );
            }
        };

        let campaign_report = self.campaign_runner.run_campaign(&campaign_config);
        if !campaign_report.errors.is_empty() {
            warnings.extend(campaign_report.errors.clone());
            reason_codes.push(ReasonCode::CampaignRunFailed);
        }
        reason_codes.extend(campaign_report.reason_codes.clone());
        reason_codes.extend(campaign_report.regression_guard.reason_codes.clone());

        let dataset_entries = collect_dataset_entries(&matrices);
        let added_dataset_summaries = summarize_added_datasets(
            &source_context.source_dataset_ids,
            &dataset_entries,
            &campaign_report,
            config,
        );
        if added_dataset_summaries
            .iter()
            .any(|summary| summary.source == DatasetEvidenceSource::SyntheticFixture)
        {
            warnings.push(
                "synthetic fixture evidence can close pipeline-completeness gaps but does not prove market edge"
                    .to_string(),
            );
            reason_codes.push(ReasonCode::SyntheticFixtureEvidence);
        }

        let added_outcome_summary = build_added_outcome_summary(&campaign_report);
        let (added_variant_summary, readiness_after, ablation_reason_codes, ablation_warnings) =
            self.run_closure_ablation(config, &campaign_config, &source_context.summary);
        reason_codes.extend(ablation_reason_codes);
        warnings.extend(ablation_warnings);

        let closure_status = build_closure_status(
            config,
            &source_context.summary,
            &added_dataset_summaries,
            &added_outcome_summary,
            &added_variant_summary,
        );
        let before_after_evidence =
            build_before_after_evidence(&source_context.summary, &closure_status, &campaign_report);
        let minimum_plan_update =
            build_minimum_plan_update(&source_context.summary, &closure_status);
        let final_recommendation = select_final_recommendation(
            &closure_status,
            &added_dataset_summaries,
            &added_variant_summary,
            &campaign_report,
            readiness_after,
        );

        reason_codes.extend(closure_status.reason_codes.clone());
        reason_codes.extend(before_after_evidence.reason_codes.clone());
        reason_codes.extend(minimum_plan_update.reason_codes.clone());
        reason_codes.push(ReasonCode::EvidenceClosureBuilt);
        if closure_status.all_targets_closed {
            reason_codes.push(ReasonCode::EvidenceClosureTargetClosed);
        } else {
            reason_codes.push(ReasonCode::EvidenceStillInsufficient);
        }
        blockers.extend(
            added_dataset_summaries
                .iter()
                .filter(|summary| !summary.counted_as_usable)
                .map(|summary| format!("dataset {} did not qualify as usable", summary.dataset_id)),
        );
        warnings.extend(
            added_dataset_summaries
                .iter()
                .flat_map(|summary| summary.warnings.iter().cloned()),
        );
        warnings.extend(campaign_report.regression_guard.warnings.clone());
        warnings.extend(added_variant_summary.warnings.clone());

        let mut report = EvidenceClosureReport {
            closure_id: config.closure_id.clone(),
            readiness_before: source_context.summary.previous_recommendation,
            readiness_after,
            source_gap_summary: source_context.summary,
            closure_status,
            added_dataset_summaries,
            added_outcome_summary,
            added_variant_summary,
            before_after_evidence,
            minimum_plan_update,
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
    ) -> Result<EvidenceClosureReport, String> {
        if config_path.to_string_lossy().contains("://") {
            return Err("evidence-close config path must be local".to_string());
        }
        let config = EvidenceClosureConfig::from_toml_path(config_path)?;
        Ok(self.run_closure(&config))
    }

    fn load_source_gap_context(&self, config: &EvidenceClosureConfig) -> SourceGapContext {
        if let Some(path) = config.source_sprint14_report_path.as_deref() {
            let report_path = Path::new(path);
            if let Ok(report) = Sprint14Report::from_json_path(report_path) {
                let dataset_ids = load_source_dataset_ids(
                    config.source_ablation_report_path.clone().or_else(|| {
                        report
                            .decision_record
                            .evidence_inputs
                            .source_report_path
                            .clone()
                    }),
                );
                return SourceGapContext {
                    summary: source_gap_from_sprint14_report(
                        &report,
                        Some(path.to_string()),
                        Vec::new(),
                    ),
                    source_dataset_ids: dataset_ids,
                };
            }
        }

        if let Some(path) = config.source_ablation_report_path.as_deref() {
            let report_path = Path::new(path);
            if let Ok(report) = self
                .sprint14_runner
                .run_from_ablation_report_path(report_path)
            {
                let mut reasons = vec![ReasonCode::EvidenceClosureSourceMissing];
                if config.source_sprint14_report_path.is_none() {
                    reasons.push(ReasonCode::EvidenceClosureDefaultsUsed);
                }
                return SourceGapContext {
                    summary: source_gap_from_sprint14_report(
                        &report,
                        Some(path.to_string()),
                        reasons,
                    ),
                    source_dataset_ids: load_source_dataset_ids(Some(path.to_string())),
                };
            }
        }

        SourceGapContext {
            summary: source_gap_defaults(
                config,
                vec![
                    ReasonCode::EvidenceClosureSourceMissing,
                    ReasonCode::EvidenceClosureDefaultsUsed,
                ],
            ),
            source_dataset_ids: BTreeSet::new(),
        }
    }

    fn run_closure_ablation(
        &self,
        config: &EvidenceClosureConfig,
        campaign_config: &ResearchCampaignConfig,
        source_gap_summary: &SourceGapSummary,
    ) -> (
        AddedVariantSummary,
        Sprint14Track,
        Vec<ReasonCode>,
        Vec<String>,
    ) {
        let Some(ablation_config) = build_ablation_config(config, campaign_config) else {
            return (
                AddedVariantSummary {
                    study_id: format!("{}-ablation", config.closure_id),
                    additional_comparable_variants: 0,
                    comparable_variant_ids: Vec::new(),
                    non_comparable_variant_ids: Vec::new(),
                    failed_variant_ids: Vec::new(),
                    warnings: vec![
                        "closure ablation could not be derived from campaign config".to_string(),
                    ],
                    reason_codes: vec![ReasonCode::CampaignMatrixLoadFailed],
                },
                Sprint14Track::NeedMoreExperiments,
                vec![ReasonCode::CampaignMatrixLoadFailed],
                vec!["closure ablation stage was skipped".to_string()],
            );
        };
        let report = self.ablation_runner.run_study(&ablation_config);
        let sprint14_after = self
            .sprint14_runner
            .run_from_ablation_report(&report, None::<String>.map(|value| value));
        let variant_summary = build_added_variant_summary(
            &report,
            source_gap_summary.required_additional_outcome_records,
        );
        let mut reason_codes = report.reason_codes.clone();
        reason_codes.extend(variant_summary.reason_codes.clone());
        (
            variant_summary,
            sprint14_after.decision_record.selected_track,
            dedupe_reasons(reason_codes),
            report.warnings.clone(),
        )
    }
}

fn build_added_outcome_summary(report: &ResearchCampaignReport) -> AddedOutcomeSummary {
    AddedOutcomeSummary {
        campaign_id: report.campaign_id.clone(),
        additional_outcome_records: report.aggregate.total_outcome_records,
        executed_trades: report.aggregate.total_executed_trades,
        no_trades: report.aggregate.total_no_trades,
        denied_trades: report.aggregate.total_denials,
        reason_codes: vec![ReasonCode::DeterministicPath],
    }
}

fn build_added_variant_summary(
    report: &AblationStudyReport,
    min_records_for_comparison: usize,
) -> AddedVariantSummary {
    let baseline_outcomes = report
        .baseline
        .report
        .expansion_readiness
        .evidence
        .total_outcome_records;
    let baseline_scope = comparable_scope(&report.baseline.report);
    let mut comparable_variant_ids = Vec::new();
    let mut non_comparable_variant_ids = Vec::new();
    let mut failed_variant_ids = Vec::new();

    for variant in &report.variants {
        let is_failed = matches!(variant.status, AblationResultStatus::Failed);
        let flagged_non_comparable = variant
            .flags
            .contains(&AblationInterpretationFlag::NotComparable);
        let has_compatible_report = variant.report.as_ref().is_some_and(|variant_report| {
            variant_report.aggregate_benchmark.failed_runs == 0
                && variant_report.run_summaries.iter().all(|summary| {
                    matches!(
                        summary.status,
                        ExperimentRunStatus::Passed | ExperimentRunStatus::Warning
                    )
                })
                && comparable_scope(variant_report) == baseline_scope
                && baseline_outcomes >= min_records_for_comparison
                && variant_report
                    .expansion_readiness
                    .evidence
                    .total_outcome_records
                    >= min_records_for_comparison
        });

        if is_failed {
            failed_variant_ids.push(variant.variant_id.clone());
        } else if flagged_non_comparable || !has_compatible_report {
            non_comparable_variant_ids.push(variant.variant_id.clone());
        } else {
            comparable_variant_ids.push(variant.variant_id.clone());
        }
    }

    let mut reason_codes = vec![ReasonCode::DeterministicPath];
    if !comparable_variant_ids.is_empty() {
        reason_codes.push(ReasonCode::ComparableVariantCounted);
    }
    if comparable_variant_ids.len() < report.variants.len() {
        reason_codes.push(ReasonCode::EvidenceStillInsufficient);
    }
    AddedVariantSummary {
        study_id: report.study_id.clone(),
        additional_comparable_variants: comparable_variant_ids.len(),
        comparable_variant_ids,
        non_comparable_variant_ids,
        failed_variant_ids,
        warnings: if baseline_outcomes < min_records_for_comparison {
            vec!["ablation baseline still has thin outcome coverage".to_string()]
        } else {
            Vec::new()
        },
        reason_codes: dedupe_reasons(reason_codes),
    }
}

fn comparable_scope(report: &super::aggregate::BatchExperimentReport) -> Vec<(String, String)> {
    let mut scope = report
        .run_summaries
        .iter()
        .filter(|summary| summary.status != ExperimentRunStatus::Skipped)
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

fn build_closure_status(
    config: &EvidenceClosureConfig,
    source_gap_summary: &SourceGapSummary,
    added_dataset_summaries: &[AddedDatasetSummary],
    added_outcome_summary: &AddedOutcomeSummary,
    added_variant_summary: &AddedVariantSummary,
) -> EvidenceClosureStatus {
    let added_datasets = added_dataset_summaries
        .iter()
        .filter(|summary| summary.counted_as_usable)
        .count();
    let usable_dataset_target = build_target(
        "usable_datasets",
        config.min_additional_usable_datasets,
        source_gap_summary.before_usable_dataset_count,
        source_gap_summary.before_usable_dataset_count + added_datasets,
        added_datasets,
    );
    let outcome_record_target = build_target(
        "outcome_records",
        config.min_additional_outcome_records,
        source_gap_summary.before_outcome_records,
        source_gap_summary.before_outcome_records
            + added_outcome_summary.additional_outcome_records,
        added_outcome_summary.additional_outcome_records,
    );
    let comparable_variant_target = build_target(
        "comparable_variants",
        config.min_additional_comparable_variants,
        source_gap_summary.before_comparable_variant_count,
        source_gap_summary.before_comparable_variant_count
            + added_variant_summary.additional_comparable_variants,
        added_variant_summary.additional_comparable_variants,
    );
    let still_missing = EvidenceClosureMissingCounts {
        usable_datasets: config
            .min_additional_usable_datasets
            .saturating_sub(added_datasets),
        outcome_records: config
            .min_additional_outcome_records
            .saturating_sub(added_outcome_summary.additional_outcome_records),
        comparable_variants: config
            .min_additional_comparable_variants
            .saturating_sub(added_variant_summary.additional_comparable_variants),
    };
    let all_targets_closed = usable_dataset_target.closed
        && outcome_record_target.closed
        && comparable_variant_target.closed;
    let partially_closed = usable_dataset_target.added_count > 0
        || outcome_record_target.added_count > 0
        || comparable_variant_target.added_count > 0;
    let mut reason_codes = vec![ReasonCode::DeterministicPath];
    if all_targets_closed {
        reason_codes.push(ReasonCode::EvidenceClosureTargetClosed);
    } else {
        reason_codes.push(ReasonCode::EvidenceStillInsufficient);
    }
    EvidenceClosureStatus {
        closure_id: config.closure_id.clone(),
        usable_dataset_target,
        outcome_record_target,
        comparable_variant_target,
        all_targets_closed,
        partially_closed,
        still_missing,
        reason_codes: dedupe_reasons(reason_codes),
    }
}

fn build_target(
    target_name: &str,
    required_count: usize,
    current_before_count: usize,
    current_after_count: usize,
    added_count: usize,
) -> EvidenceGapTarget {
    EvidenceGapTarget {
        target_name: target_name.to_string(),
        required_count,
        current_before_count,
        current_after_count,
        added_count,
        closed: added_count >= required_count,
        reason_codes: vec![ReasonCode::DeterministicPath],
    }
}

fn build_before_after_evidence(
    source_gap_summary: &SourceGapSummary,
    closure_status: &EvidenceClosureStatus,
    campaign_report: &ResearchCampaignReport,
) -> EvidenceClosureBeforeAfter {
    let mut reason_codes = vec![ReasonCode::DeterministicPath];
    let safety_regressions = campaign_report
        .regression_guard
        .regressions
        .iter()
        .map(|regression| format!("{regression:?}"))
        .collect::<Vec<_>>();
    if !safety_regressions.is_empty() {
        reason_codes.push(ReasonCode::SafetyRegressionDetected);
    }
    EvidenceClosureBeforeAfter {
        usable_dataset_count_before: source_gap_summary.before_usable_dataset_count,
        usable_dataset_count_after: closure_status.usable_dataset_target.current_after_count,
        outcome_record_count_before: source_gap_summary.before_outcome_records,
        outcome_record_count_after: closure_status.outcome_record_target.current_after_count,
        comparable_variant_count_before: source_gap_summary.before_comparable_variant_count,
        comparable_variant_count_after: closure_status
            .comparable_variant_target
            .current_after_count,
        additional_usable_datasets: closure_status.usable_dataset_target.added_count,
        additional_outcome_records: closure_status.outcome_record_target.added_count,
        additional_comparable_variants: closure_status.comparable_variant_target.added_count,
        safety_regressions,
        warnings: campaign_report.regression_guard.warnings.clone(),
        reason_codes: dedupe_reasons(reason_codes),
    }
}

fn build_minimum_plan_update(
    source_gap_summary: &SourceGapSummary,
    closure_status: &EvidenceClosureStatus,
) -> MinimumEvidencePlanUpdate {
    let previous_plan = vec![
        format!(
            "need +{} usable dataset",
            source_gap_summary.required_additional_usable_datasets
        ),
        format!(
            "need +{} outcome records",
            source_gap_summary.required_additional_outcome_records
        ),
        format!(
            "need +{} comparable variants",
            source_gap_summary.required_additional_comparable_variants
        ),
    ];
    let mut completed_items = Vec::new();
    if closure_status.usable_dataset_target.closed {
        completed_items.push(format!(
            "added {} usable dataset(s)",
            closure_status.usable_dataset_target.added_count
        ));
    }
    if closure_status.outcome_record_target.closed {
        completed_items.push(format!(
            "added {} outcome record(s)",
            closure_status.outcome_record_target.added_count
        ));
    }
    if closure_status.comparable_variant_target.closed {
        completed_items.push(format!(
            "added {} comparable ablation variant(s)",
            closure_status.comparable_variant_target.added_count
        ));
    }
    let mut remaining_items = Vec::new();
    if closure_status.still_missing.usable_datasets > 0 {
        remaining_items.push(format!(
            "need {} more usable dataset(s)",
            closure_status.still_missing.usable_datasets
        ));
    }
    if closure_status.still_missing.outcome_records > 0 {
        remaining_items.push(format!(
            "need {} more outcome record(s)",
            closure_status.still_missing.outcome_records
        ));
    }
    if closure_status.still_missing.comparable_variants > 0 {
        remaining_items.push(format!(
            "need {} more comparable variant(s)",
            closure_status.still_missing.comparable_variants
        ));
    }
    let next_required_items = if closure_status.all_targets_closed {
        vec![
            "rerun ablation with the expanded fixture set when new local datasets are available"
                .to_string(),
            "compare the closure campaign against the previous campaign before any wider scope change"
                .to_string(),
            "allow only design-review discussion if later readiness gates remain clean".to_string(),
        ]
    } else {
        remaining_items.clone()
    };
    let mut reason_codes = vec![ReasonCode::MinimumEvidencePlanBuilt];
    if !closure_status.all_targets_closed {
        reason_codes.push(ReasonCode::EvidenceStillInsufficient);
    }
    MinimumEvidencePlanUpdate {
        previous_plan,
        completed_items,
        remaining_items,
        next_required_items,
        reason_codes: dedupe_reasons(reason_codes),
    }
}

fn select_final_recommendation(
    closure_status: &EvidenceClosureStatus,
    added_dataset_summaries: &[AddedDatasetSummary],
    added_variant_summary: &AddedVariantSummary,
    campaign_report: &ResearchCampaignReport,
    readiness_after: Sprint14Track,
) -> EvidenceClosureRecommendation {
    let bad_dataset = added_dataset_summaries.iter().any(|summary| {
        matches!(
            summary.data_quality_severity,
            DataQualitySeverity::Bad | DataQualitySeverity::Unusable
        ) || !summary.counted_as_usable
    });
    if bad_dataset {
        return EvidenceClosureRecommendation::ImproveDataFirst;
    }
    if !campaign_report.regression_guard.regressions.is_empty() {
        return EvidenceClosureRecommendation::ImproveRiskGovernorFirst;
    }
    if !closure_status.all_targets_closed
        || added_variant_summary.additional_comparable_variants
            < closure_status.comparable_variant_target.required_count
    {
        return EvidenceClosureRecommendation::NeedMoreExperiments;
    }
    match readiness_after {
        Sprint14Track::ImproveDataFirst => EvidenceClosureRecommendation::ImproveDataFirst,
        Sprint14Track::ImproveRiskGovernorFirst => {
            EvidenceClosureRecommendation::ImproveRiskGovernorFirst
        }
        Sprint14Track::ImproveSignalModelFirst => {
            EvidenceClosureRecommendation::ImproveSignalModelFirst
        }
        Sprint14Track::HoldCurrentScope => EvidenceClosureRecommendation::HoldCurrentScope,
        Sprint14Track::ReadyForSixPersonaDesignReview => {
            EvidenceClosureRecommendation::ReadyForSixPersonaDesignReview
        }
        _ => EvidenceClosureRecommendation::NeedMoreExperiments,
    }
}

fn summarize_added_datasets(
    source_dataset_ids: &BTreeSet<String>,
    dataset_entries: &BTreeMap<String, DatasetEntry>,
    campaign_report: &ResearchCampaignReport,
    config: &EvidenceClosureConfig,
) -> Vec<AddedDatasetSummary> {
    let quality_by_dataset = campaign_report
        .matrix_results
        .iter()
        .filter_map(|result| result.report.as_ref())
        .flat_map(|report| report.run_summaries.iter())
        .filter(|summary| summary.status != ExperimentRunStatus::Skipped)
        .fold(
            BTreeMap::<String, (f64, DataQualitySeverity)>::new(),
            |mut map, summary| {
                map.entry(summary.run_key.dataset_id.clone())
                    .and_modify(|(score, severity)| {
                        *score = score.min(summary.data_quality_score);
                        if severity_rank(summary.data_quality_severity) > severity_rank(*severity) {
                            *severity = summary.data_quality_severity;
                        }
                    })
                    .or_insert((summary.data_quality_score, summary.data_quality_severity));
                map
            },
        );
    let mut datasets = dataset_entries
        .iter()
        .filter(|(dataset_id, _)| !source_dataset_ids.contains(*dataset_id))
        .map(|(dataset_id, entry)| {
            let (score, severity) = quality_by_dataset
                .get(dataset_id)
                .copied()
                .unwrap_or((0.0, DataQualitySeverity::Unusable));
            let source = classify_dataset_source(entry);
            let quality_ok = if config.strict_data_quality {
                matches!(
                    severity,
                    DataQualitySeverity::Good | DataQualitySeverity::Warning
                )
            } else {
                !matches!(severity, DataQualitySeverity::Unusable)
            };
            let synthetic_ok = if source == DatasetEvidenceSource::SyntheticFixture {
                config.allow_synthetic_dataset
                    && (!config.synthetic_dataset_must_be_tagged
                        || entry.tags.iter().any(|tag| tag == "synthetic"))
            } else {
                true
            };
            let counted_as_usable = quality_ok && synthetic_ok;
            let mut warnings = Vec::new();
            let mut reason_codes = vec![ReasonCode::DeterministicPath];
            if source == DatasetEvidenceSource::SyntheticFixture {
                reason_codes.push(ReasonCode::SyntheticFixtureEvidence);
                warnings.push(
                    "synthetic fixture evidence is pipeline-only and does not prove market edge"
                        .to_string(),
                );
            }
            if counted_as_usable {
                reason_codes.push(ReasonCode::AdditionalDatasetCounted);
            } else if !quality_ok {
                warnings.push("dataset quality did not meet the usable threshold".to_string());
            } else if !synthetic_ok {
                warnings.push("synthetic dataset was not allowed by closure config".to_string());
            }
            AddedDatasetSummary {
                dataset_id: dataset_id.clone(),
                data_path: entry.data_path.clone(),
                source,
                tags: entry.tags.clone(),
                data_quality_score: score,
                data_quality_severity: severity,
                counted_as_usable,
                warnings,
                reason_codes: dedupe_reasons(reason_codes),
            }
        })
        .collect::<Vec<_>>();
    datasets.sort_by(|left, right| left.dataset_id.cmp(&right.dataset_id));
    datasets
}

fn classify_dataset_source(entry: &DatasetEntry) -> DatasetEvidenceSource {
    if entry.tags.iter().any(|tag| tag == "synthetic") {
        DatasetEvidenceSource::SyntheticFixture
    } else if entry.data_path.contains("tests/fixtures")
        || entry.tags.iter().any(|tag| tag == "fixture")
    {
        DatasetEvidenceSource::TestFixture
    } else if Path::new(&entry.data_path).exists() {
        DatasetEvidenceSource::RealLocalData
    } else {
        DatasetEvidenceSource::UnknownSource
    }
}

fn load_campaign_config(config: &EvidenceClosureConfig) -> Result<ResearchCampaignConfig, String> {
    let mut campaign_config = if let Some(embedded) = &config.embedded_campaign_config {
        embedded.clone()
    } else if let Some(path) = &config.source_campaign_config_path {
        ResearchCampaignConfig::from_toml_path(Path::new(path))?
    } else {
        return Err(
            "evidence closure requires embedded_campaign_config or source_campaign_config_path"
                .to_string(),
        );
    };
    campaign_config.output_root = Path::new(&config.output_root)
        .join("campaigns")
        .display()
        .to_string();
    campaign_config.evidence_store_path = config.evidence_store_path.clone();
    campaign_config.continue_on_failure = config.continue_on_failure;
    campaign_config.allow_persona_expansion_recommendation = false;
    if campaign_config.created_at_ms.is_none() {
        campaign_config.created_at_ms = config.created_at_ms;
    }
    Ok(campaign_config)
}

fn load_campaign_matrices(
    config: &ResearchCampaignConfig,
) -> Result<Vec<ExperimentMatrixConfig>, String> {
    let mut matrices = Vec::new();
    for path in &config.matrix_config_paths {
        matrices.push(ExperimentMatrixConfig::from_toml_path(Path::new(path))?);
    }
    matrices.extend(config.embedded_matrices.iter().cloned());
    matrices.sort_by(|left, right| left.matrix_id.cmp(&right.matrix_id));
    if matrices.is_empty() {
        return Err("closure campaign did not provide any matrix config".to_string());
    }
    Ok(matrices)
}

fn collect_dataset_entries(matrices: &[ExperimentMatrixConfig]) -> BTreeMap<String, DatasetEntry> {
    let mut map = BTreeMap::new();
    for matrix in matrices {
        for entry in &matrix.dataset_bundle.entries {
            map.entry(entry.dataset_id.clone())
                .or_insert_with(|| entry.clone());
        }
    }
    map
}

fn build_ablation_config(
    config: &EvidenceClosureConfig,
    campaign_config: &ResearchCampaignConfig,
) -> Option<AblationStudyConfig> {
    let mut matrix_path = campaign_config.matrix_config_paths.clone();
    matrix_path.sort();
    if let Some(path) = matrix_path.first() {
        return Some(AblationStudyConfig {
            study_id: format!("{}-ablation", config.closure_id),
            description: Some("Sprint 15 closure ablation".to_string()),
            base_matrix_config_path: Some(path.clone()),
            embedded_base_matrix: None,
            output_root: Path::new(&config.output_root)
                .join("ablations")
                .display()
                .to_string(),
            require_baseline_pass: true,
            continue_on_variant_failure: config.continue_on_failure,
            variants: if config.ablation_variants.is_empty() {
                default_ablation_variants()
            } else {
                config.ablation_variants.clone()
            },
            created_at_ms: config.created_at_ms,
            reason_codes: vec![ReasonCode::DeterministicPath],
        });
    }
    campaign_config
        .embedded_matrices
        .iter()
        .cloned()
        .min_by(|left, right| left.matrix_id.cmp(&right.matrix_id))
        .map(|matrix| AblationStudyConfig {
            study_id: format!("{}-ablation", config.closure_id),
            description: Some("Sprint 15 closure ablation".to_string()),
            base_matrix_config_path: None,
            embedded_base_matrix: Some(matrix),
            output_root: Path::new(&config.output_root)
                .join("ablations")
                .display()
                .to_string(),
            require_baseline_pass: true,
            continue_on_variant_failure: config.continue_on_failure,
            variants: if config.ablation_variants.is_empty() {
                default_ablation_variants()
            } else {
                config.ablation_variants.clone()
            },
            created_at_ms: config.created_at_ms,
            reason_codes: vec![ReasonCode::DeterministicPath],
        })
}

fn load_source_dataset_ids(path: Option<String>) -> BTreeSet<String> {
    let Some(path) = path else {
        return BTreeSet::new();
    };
    AblationStudyReport::from_json_path(Path::new(&path))
        .ok()
        .map(|report| {
            report
                .baseline
                .report
                .run_summaries
                .iter()
                .map(|summary| summary.run_key.dataset_id.clone())
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default()
}

fn source_gap_from_sprint14_report(
    report: &Sprint14Report,
    source_report_path: Option<String>,
    extra_reason_codes: Vec<ReasonCode>,
) -> SourceGapSummary {
    let (
        required_additional_usable_datasets,
        required_additional_outcome_records,
        required_additional_comparable_variants,
    ) = match &report.track_specific_report {
        Sprint14TrackSpecificReport::NeedMoreExperiments(gap) => (
            gap.minimum_evidence_plan.additional_usable_datasets_needed,
            gap.minimum_evidence_plan.additional_outcome_records_needed,
            gap.minimum_evidence_plan
                .additional_comparable_variants_needed,
        ),
    };
    let mut reason_codes = report.reason_codes.clone();
    reason_codes.extend(extra_reason_codes);
    SourceGapSummary {
        source_report_path,
        source_study_id: report
            .decision_record
            .evidence_inputs
            .source_study_id
            .clone(),
        previous_recommendation: report.decision_record.selected_track,
        required_additional_usable_datasets,
        required_additional_outcome_records,
        required_additional_comparable_variants,
        before_usable_dataset_count: report
            .decision_record
            .evidence_inputs
            .usable_dataset_count
            .unwrap_or(0),
        before_outcome_records: report
            .decision_record
            .evidence_inputs
            .total_outcome_records
            .unwrap_or(0),
        before_comparable_variant_count: report
            .decision_record
            .evidence_inputs
            .comparable_variant_count
            .unwrap_or(0),
        warnings: report.decision_record.warnings.clone(),
        blockers: report.decision_record.blockers.clone(),
        reason_codes: dedupe_reasons(reason_codes),
    }
}

fn source_gap_defaults(
    config: &EvidenceClosureConfig,
    extra_reason_codes: Vec<ReasonCode>,
) -> SourceGapSummary {
    let mut reason_codes = vec![ReasonCode::DeterministicPath];
    reason_codes.extend(extra_reason_codes);
    SourceGapSummary {
        source_report_path: config
            .source_sprint14_report_path
            .clone()
            .or_else(|| config.source_ablation_report_path.clone()),
        source_study_id: None,
        previous_recommendation: Sprint14Track::NeedMoreExperiments,
        required_additional_usable_datasets: config.min_additional_usable_datasets,
        required_additional_outcome_records: config.min_additional_outcome_records,
        required_additional_comparable_variants: config.min_additional_comparable_variants,
        before_usable_dataset_count: 0,
        before_outcome_records: 0,
        before_comparable_variant_count: 0,
        warnings: vec![
            "source Sprint 14 evidence was unavailable; using conservative defaults".to_string(),
        ],
        blockers: Vec::new(),
        reason_codes: dedupe_reasons(reason_codes),
    }
}

fn minimal_report(
    config: &EvidenceClosureConfig,
    source_gap_summary: SourceGapSummary,
    added_outcome_summary: AddedOutcomeSummary,
    added_variant_summary: AddedVariantSummary,
    added_dataset_summaries: Vec<AddedDatasetSummary>,
    blockers: Vec<String>,
    reason_codes: Vec<ReasonCode>,
) -> EvidenceClosureReport {
    let closure_status = build_closure_status(
        config,
        &source_gap_summary,
        &added_dataset_summaries,
        &added_outcome_summary,
        &added_variant_summary,
    );
    let before_after_evidence = EvidenceClosureBeforeAfter {
        usable_dataset_count_before: source_gap_summary.before_usable_dataset_count,
        usable_dataset_count_after: closure_status.usable_dataset_target.current_after_count,
        outcome_record_count_before: source_gap_summary.before_outcome_records,
        outcome_record_count_after: closure_status.outcome_record_target.current_after_count,
        comparable_variant_count_before: source_gap_summary.before_comparable_variant_count,
        comparable_variant_count_after: closure_status
            .comparable_variant_target
            .current_after_count,
        additional_usable_datasets: closure_status.usable_dataset_target.added_count,
        additional_outcome_records: closure_status.outcome_record_target.added_count,
        additional_comparable_variants: closure_status.comparable_variant_target.added_count,
        safety_regressions: Vec::new(),
        warnings: Vec::new(),
        reason_codes: vec![ReasonCode::DeterministicPath],
    };
    EvidenceClosureReport {
        closure_id: config.closure_id.clone(),
        readiness_before: source_gap_summary.previous_recommendation,
        readiness_after: Sprint14Track::NeedMoreExperiments,
        source_gap_summary,
        closure_status,
        added_dataset_summaries,
        added_outcome_summary,
        added_variant_summary,
        before_after_evidence,
        minimum_plan_update: build_minimum_plan_update(
            &source_gap_defaults(config, vec![ReasonCode::EvidenceClosureDefaultsUsed]),
            &build_closure_status(
                config,
                &source_gap_defaults(config, vec![ReasonCode::EvidenceClosureDefaultsUsed]),
                &Vec::new(),
                &AddedOutcomeSummary {
                    campaign_id: config.closure_id.clone(),
                    additional_outcome_records: 0,
                    executed_trades: 0,
                    no_trades: 0,
                    denied_trades: 0,
                    reason_codes: vec![ReasonCode::DeterministicPath],
                },
                &AddedVariantSummary {
                    study_id: format!("{}-ablation", config.closure_id),
                    additional_comparable_variants: 0,
                    comparable_variant_ids: Vec::new(),
                    non_comparable_variant_ids: Vec::new(),
                    failed_variant_ids: Vec::new(),
                    warnings: Vec::new(),
                    reason_codes: vec![ReasonCode::DeterministicPath],
                },
            ),
        ),
        final_recommendation: EvidenceClosureRecommendation::NeedMoreExperiments,
        blockers: dedupe_strings(blockers),
        warnings: Vec::new(),
        reason_codes: dedupe_reasons(reason_codes),
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

fn dedupe_strings(values: Vec<String>) -> Vec<String> {
    let mut seen = BTreeSet::new();
    values
        .into_iter()
        .filter(|value| seen.insert(value.clone()))
        .collect()
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

fn default_output_root() -> String {
    "target/soma_evidence_closure".to_string()
}

fn default_evidence_store_path() -> String {
    "target/soma_evidence_closure_store".to_string()
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

fn default_ablation_variants() -> Vec<AblationVariant> {
    vec![
        AblationVariant {
            variant_id: "volume_off".to_string(),
            dimension: super::ablation::AblationDimension::FeatureGroup,
            overrides: vec![super::ablation::AblationOverride {
                target: "volume".to_string(),
                value: super::ablation::AblationValue::Bool(false),
            }],
            research_only: false,
            enabled: true,
            tags: vec!["closure".to_string()],
            notes: None,
            reason_codes: vec![ReasonCode::DeterministicPath],
        },
        AblationVariant {
            variant_id: "higher_cost".to_string(),
            dimension: super::ablation::AblationDimension::CostModel,
            overrides: vec![super::ablation::AblationOverride {
                target: "spread_bps".to_string(),
                value: super::ablation::AblationValue::Float(4.0),
            }],
            research_only: false,
            enabled: true,
            tags: vec!["closure".to_string()],
            notes: None,
            reason_codes: vec![ReasonCode::DeterministicPath],
        },
    ]
}
