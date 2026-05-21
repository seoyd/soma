use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_hash_string, stable_reason_codes};
use crate::experiment::CoreScorecardRerunSummary;

use super::balanced_outcome_coverage::{
    BalancedOutcomeCoverageConfig, BalancedOutcomeCoverageReport, BalancedOutcomeCoverageRunner,
};
use super::barrier_profile_registry::load_barrier_profile_registry_from_path_or_config;
use super::batch_counterfactual_completion::{
    BatchCounterfactualCompletionConfig, BatchCounterfactualCompletionReport,
    BatchCounterfactualCompletionRunner, load_batch_counterfactual_completion_from_path_or_config,
};
use super::batch_outcome_linkage_v3::{
    BatchOutcomeLinkageV3Config, BatchOutcomeLinkageV3Report, BatchOutcomeLinkageV3Runner,
    load_batch_outcome_linkage_v3_from_path_or_config,
};
use super::committee_official_benchmark::{
    CommitteeOfficialBenchmarkConfig, CommitteeOfficialBenchmarkRunner,
};
use super::committee_outcome_coverage::CommitteeOutcomeCoverageConfig;
use super::committee_outcome_coverage_runner::CommitteeOutcomeCoverageRunner;
use super::counterfactual_depth_closure::{
    CounterfactualDepthClosureConfig, CounterfactualDepthClosureRunner,
};
use super::diversity_aware_sufficiency_v2::{
    DiversityAwareSufficiencyV2Config, DiversityAwareSufficiencyV2Report,
    DiversityAwareSufficiencyV2Runner, DiversityAwareSufficiencyV2Status,
};
use super::future_window_scaleout::{
    FutureWindowScaleOutPlan, load_future_window_scaleout_plan_from_path_or_config,
};
use super::multi_row_official_evidence::{
    MultiRowOfficialEvidenceSet, load_multi_row_official_evidence_set_from_path_or_config,
};
use super::official_diversity_row_selector::{
    OfficialDiversityRowSelector, OfficialDiversityRowSelectorConfig,
    OfficialDiversityRowSelectorReport, OfficialDiversitySweepConfig,
    load_official_diversity_row_selector_report_from_path_or_config,
};
use super::official_evidence_diversity_gap::{
    OfficialEvidenceDiversityGapConfig, OfficialEvidenceDiversityGapMap,
    OfficialEvidenceDiversityGapRunner, OfficialEvidenceDiversityGapStatus,
    load_official_evidence_diversity_gap_map_from_path_or_config,
};
use super::outcome_diversity_audit::{
    OutcomeDiversityAuditConfig, OutcomeDiversityAuditReport, OutcomeDiversityAuditRunner,
    OutcomeDiversityStatus,
};
use crate::experiment::{
    CorePerformanceScorecardConfig, CorePerformanceScorecardRunner, CoreScorecardRerun,
};

use super::official_evidence_diversity_bundle::{
    OfficialEvidenceDiversitySweepBundle, build_official_evidence_diversity_summary,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OfficialEvidenceDiversitySweepConfig {
    pub run_id: String,
    #[serde(default)]
    pub sweep_config_path: Option<String>,
    #[serde(default)]
    pub diversity_gap_config_path: Option<String>,
    #[serde(default)]
    pub diversity_gap_map_path: Option<String>,
    #[serde(default)]
    pub row_selector_config_path: Option<String>,
    #[serde(default)]
    pub barrier_profile_registry_path: Option<String>,
    #[serde(default)]
    pub future_window_scaleout_config_path: Option<String>,
    #[serde(default)]
    pub batch_outcome_linkage_config_path: Option<String>,
    #[serde(default)]
    pub batch_counterfactual_completion_config_path: Option<String>,
    #[serde(default)]
    pub sufficiency_v2_config_path: Option<String>,
    #[serde(default)]
    pub committee_official_benchmark_config_path: Option<String>,
    #[serde(default)]
    pub outcome_coverage_config_path: Option<String>,
    #[serde(default)]
    pub counterfactual_depth_closure_config_path: Option<String>,
    #[serde(default)]
    pub core_performance_config_path: Option<String>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default = "default_true")]
    pub run_gap_map: bool,
    #[serde(default = "default_true")]
    pub run_row_selector: bool,
    #[serde(default = "default_true")]
    pub run_future_window_scaleout: bool,
    #[serde(default = "default_true")]
    pub run_batch_outcome_linkage: bool,
    #[serde(default = "default_true")]
    pub run_batch_counterfactual_completion: bool,
    #[serde(default = "default_true")]
    pub run_balanced_coverage: bool,
    #[serde(default = "default_true")]
    pub run_sufficiency_v2: bool,
    #[serde(default = "default_true")]
    pub run_committee_official_benchmark: bool,
    #[serde(default = "default_true")]
    pub run_outcome_coverage: bool,
    #[serde(default = "default_true")]
    pub run_counterfactual_depth_close: bool,
    #[serde(default = "default_true")]
    pub run_core_performance: bool,
    #[serde(default = "default_max_rows")]
    pub max_rows: usize,
    #[serde(default = "default_max_symbols")]
    pub max_symbols: usize,
    #[serde(default = "default_max_bytes")]
    pub max_bytes: usize,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum OfficialEvidenceDiversitySweepStatus {
    OutcomeDiversityImproved,
    OfficialCompleteRowsExpanded,
    PlumbingValidated,
    CommitteeBenchmarkResearchReady,
    TentativeSignalQualityReviewReady,
    StillNeedMoreOfficialRows,
    StillNeedOutcomeDiversity,
    StillSingleOutcomeDominated,
    StillNeedCounterfactualDepth,
    #[default]
    NoImprovement,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum OfficialEvidenceDiversitySweepRecommendation {
    MoreOfficialRows,
    MoreOfficialSymbols,
    MoreTimeframes,
    MoreHorizons,
    MoreStopLossExamples,
    MoreTimeExpiredExamples,
    MoreCounterfactualDepth,
    RunCommitteeOfficialBenchmark,
    RunCorePerformance,
    ImproveRiskGovernorFirst,
    ImproveChairFirst,
    ImproveSignalModelFirst,
    KeepTrinity,
    #[default]
    NeedMoreEvidence,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfficialEvidenceDiversitySweepReport {
    pub run_id: String,
    #[serde(default)]
    pub previous_official_complete_rows: Option<usize>,
    pub current_official_complete_rows: usize,
    #[serde(default)]
    pub previous_outcome_diversity_status: Option<OutcomeDiversityStatus>,
    pub current_outcome_diversity_status: OutcomeDiversityStatus,
    #[serde(default)]
    pub previous_sufficiency_status: Option<DiversityAwareSufficiencyV2Status>,
    pub current_sufficiency_status: DiversityAwareSufficiencyV2Status,
    #[serde(default)]
    pub previous_core_status: Option<String>,
    #[serde(default)]
    pub current_core_status: Option<String>,
    #[serde(default)]
    pub previous_primary_bottleneck: Option<String>,
    #[serde(default)]
    pub current_primary_bottleneck: Option<String>,
    pub added_official_complete_rows: isize,
    pub added_symbols: isize,
    pub added_timeframes: isize,
    pub added_horizons: isize,
    pub added_take_profit: isize,
    pub added_stop_loss: isize,
    pub added_time_expired: isize,
    pub added_no_trade_counterfactuals: isize,
    pub added_risk_denied_counterfactuals: isize,
    pub bottleneck_changed: bool,
    pub final_status: OfficialEvidenceDiversitySweepStatus,
    pub final_recommendation: OfficialEvidenceDiversitySweepRecommendation,
    #[serde(default)]
    pub blockers: Vec<String>,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfficialEvidenceDiversityStorageReport {
    pub candidate_bytes: usize,
    pub future_window_bytes: usize,
    pub outcome_linkage_bytes: usize,
    pub counterfactual_bytes: usize,
    pub coverage_bytes: usize,
    pub sufficiency_bytes: usize,
    pub benchmark_bytes: usize,
    pub core_scorecard_bytes: usize,
    pub bundle_bytes: usize,
    pub total_bytes: usize,
    pub budget_exceeded: bool,
    pub largest_artifacts: Vec<String>,
    pub compaction_recommendation: String,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OfficialEvidenceDiversitySweepRunner;

impl Default for OfficialEvidenceDiversitySweepConfig {
    fn default() -> Self {
        Self {
            run_id: "official-evidence-diversity-sweep".to_string(),
            sweep_config_path: None,
            diversity_gap_config_path: None,
            diversity_gap_map_path: None,
            row_selector_config_path: None,
            barrier_profile_registry_path: None,
            future_window_scaleout_config_path: None,
            batch_outcome_linkage_config_path: None,
            batch_counterfactual_completion_config_path: None,
            sufficiency_v2_config_path: None,
            committee_official_benchmark_config_path: None,
            outcome_coverage_config_path: None,
            counterfactual_depth_closure_config_path: None,
            core_performance_config_path: None,
            output_root: default_output_root(),
            run_gap_map: true,
            run_row_selector: true,
            run_future_window_scaleout: true,
            run_batch_outcome_linkage: true,
            run_batch_counterfactual_completion: true,
            run_balanced_coverage: true,
            run_sufficiency_v2: true,
            run_committee_official_benchmark: true,
            run_outcome_coverage: true,
            run_counterfactual_depth_close: true,
            run_core_performance: true,
            max_rows: default_max_rows(),
            max_symbols: default_max_symbols(),
            max_bytes: default_max_bytes(),
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

impl OfficialEvidenceDiversitySweepConfig {
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

    pub fn validate(&self) -> Result<(), String> {
        if self.run_id.trim().is_empty() {
            return Err("official evidence diversity sweep run_id must not be empty".to_string());
        }
        if self
            .all_paths()
            .iter()
            .chain(std::iter::once(&self.output_root))
            .any(|path| path.contains("://"))
        {
            return Err("official evidence diversity sweep paths must be local".to_string());
        }
        if self.max_rows == 0 || self.max_rows > 1000 {
            return Err(
                "official evidence diversity sweep max_rows must be between 1 and 1000".to_string(),
            );
        }
        if self.max_symbols == 0 || self.max_symbols > 10 {
            return Err(
                "official evidence diversity sweep max_symbols must be between 1 and 10"
                    .to_string(),
            );
        }
        if self.max_bytes == 0 || self.max_bytes > default_max_bytes() {
            return Err(
                "official evidence diversity sweep max_bytes must be between 1 and 5000000"
                    .to_string(),
            );
        }
        Ok(())
    }

    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.run_id)
    }

    pub fn all_paths(&self) -> Vec<String> {
        self.sweep_config_path
            .iter()
            .chain(self.diversity_gap_config_path.iter())
            .chain(self.diversity_gap_map_path.iter())
            .chain(self.row_selector_config_path.iter())
            .chain(self.barrier_profile_registry_path.iter())
            .chain(self.future_window_scaleout_config_path.iter())
            .chain(self.batch_outcome_linkage_config_path.iter())
            .chain(self.batch_counterfactual_completion_config_path.iter())
            .chain(self.sufficiency_v2_config_path.iter())
            .chain(self.committee_official_benchmark_config_path.iter())
            .chain(self.outcome_coverage_config_path.iter())
            .chain(self.counterfactual_depth_closure_config_path.iter())
            .chain(self.core_performance_config_path.iter())
            .cloned()
            .collect()
    }
}

impl OfficialEvidenceDiversitySweepRunner {
    pub fn run(
        &self,
        config: &OfficialEvidenceDiversitySweepConfig,
    ) -> Result<OfficialEvidenceDiversitySweepBundle, String> {
        config.validate()?;
        let sweep_config = config
            .sweep_config_path
            .as_deref()
            .map(|path| OfficialDiversitySweepConfig::from_toml_path(Path::new(path)))
            .transpose()?;
        let registry_path = config.barrier_profile_registry_path.clone().or_else(|| {
            sweep_config
                .as_ref()
                .and_then(|cfg| cfg.barrier_profile_registry_path.clone())
        });
        let registry = registry_path
            .as_deref()
            .map(load_barrier_profile_registry_from_path_or_config)
            .transpose()?;

        let current_set = sweep_config
            .as_ref()
            .and_then(|cfg| cfg.multi_row_set_config_paths.first().cloned())
            .map(|path| load_multi_row_official_evidence_set_from_path_or_config(&path))
            .transpose()?
            .unwrap_or_else(|| MultiRowOfficialEvidenceSet {
                set_id: config.run_id.clone(),
                items: Vec::new(),
                total_rows: 0,
                official_complete_rows: 0,
                official_partial_rows: 0,
                non_crypto_official_rows: 0,
                crypto_only_rows: 0,
                controlled_rows: 0,
                yfinance_rows: 0,
                fixture_rows: 0,
                outcome_reference_count: 0,
                baseline_reference_count: 0,
                no_trade_counterfactual_count: 0,
                risk_denied_counterfactual_count: 0,
                no_lookahead_safe_count: 0,
                storage_bytes: 0,
                symbol_count: 0,
                timeframe_count: 0,
                horizon_count: 0,
                source_boundaries_preserved: true,
                status: super::multi_row_official_evidence::MultiRowOfficialEvidenceStatus::SourceIneligible,
                warnings: Vec::new(),
                reason_codes: Vec::new(),
            });

        let previous_gap_map = config
            .diversity_gap_map_path
            .as_deref()
            .map(load_official_evidence_diversity_gap_map_from_path_or_config)
            .transpose()?;

        let row_selector_report = if config.run_row_selector {
            if let Some(path) = config.row_selector_config_path.as_deref() {
                Some(load_official_diversity_row_selector_report_from_path_or_config(path)?)
            } else if let Some(sweep_path) = config.sweep_config_path.as_deref() {
                Some(
                    OfficialDiversityRowSelector::default().run(
                        &OfficialDiversityRowSelectorConfig {
                            selector_id: config.run_id.clone(),
                            sweep_config_path: Some(sweep_path.to_string()),
                            candidate_sources: sweep_config
                                .as_ref()
                                .map(|cfg| cfg.multi_row_set_config_paths.clone())
                                .unwrap_or_default(),
                            max_candidates: sweep_config
                                .as_ref()
                                .map(|cfg| cfg.max_new_rows)
                                .unwrap_or(10),
                            require_preregistered_profile: true,
                            ..OfficialDiversityRowSelectorConfig::default()
                        },
                    )?,
                )
            } else {
                None
            }
        } else {
            None
        };

        let future_window_scaleout_plan = if config.run_future_window_scaleout {
            config
                .future_window_scaleout_config_path
                .as_deref()
                .map(load_future_window_scaleout_plan_from_path_or_config)
                .transpose()?
        } else {
            None
        };

        let official_profile = registry
            .as_ref()
            .and_then(|registry| registry.official_profile(None));
        let selected_row_ids = row_selector_report
            .as_ref()
            .map(|report| {
                report
                    .selected_candidates
                    .iter()
                    .map(|candidate| candidate.candidate_id.clone())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        let batch_outcome_linkage_report = if config.run_batch_outcome_linkage {
            config
                .batch_outcome_linkage_config_path
                .as_deref()
                .map(|path| {
                    if path.ends_with(".json") {
                        load_batch_outcome_linkage_v3_from_path_or_config(path)
                    } else {
                        let mut outcome_config =
                            BatchOutcomeLinkageV3Config::from_toml_path(Path::new(path))?;
                        if let Some(profile) = official_profile {
                            outcome_config.default_horizon_bars = profile.horizon_bars;
                            outcome_config.take_profit_pct = profile.take_profit_pct;
                            outcome_config.stop_loss_pct = profile.stop_loss_pct;
                            outcome_config.cost_bps = profile.cost_bps;
                            outcome_config.slippage_bps = profile.slippage_bps;
                            outcome_config.tie_break_policy = profile.tie_break_policy;
                        }
                        if !selected_row_ids.is_empty() {
                            outcome_config.include_row_ids = selected_row_ids.clone();
                        }
                        BatchOutcomeLinkageV3Runner::default().run(&outcome_config)
                    }
                })
                .transpose()?
        } else {
            None
        };

        let batch_counterfactual_completion_report = if config.run_batch_counterfactual_completion {
            config
                .batch_counterfactual_completion_config_path
                .as_deref()
                .map(|path| {
                    if path.ends_with(".json") {
                        load_batch_counterfactual_completion_from_path_or_config(path)
                    } else {
                        let mut counterfactual_config =
                            BatchCounterfactualCompletionConfig::from_toml_path(Path::new(path))?;
                        if !selected_row_ids.is_empty() {
                            counterfactual_config.include_row_ids = selected_row_ids.clone();
                        }
                        BatchCounterfactualCompletionRunner::default().run(&counterfactual_config)
                    }
                })
                .transpose()?
        } else {
            None
        };

        let current_gap_map = if config.run_gap_map {
            if let Some(path) = config.diversity_gap_config_path.as_deref() {
                OfficialEvidenceDiversityGapRunner::default().run(
                    &OfficialEvidenceDiversityGapConfig::from_toml_path(Path::new(path))?,
                )?
            } else {
                OfficialEvidenceDiversityGapRunner::default().run_from_inputs(
                    &OfficialEvidenceDiversityGapConfig {
                        diversity_id: config.run_id.clone(),
                        multi_row_official_set_paths: sweep_config
                            .as_ref()
                            .map(|cfg| cfg.multi_row_set_config_paths.clone())
                            .unwrap_or_default(),
                        batch_outcome_linkage_paths: config
                            .batch_outcome_linkage_config_path
                            .clone()
                            .into_iter()
                            .collect(),
                        batch_counterfactual_completion_paths: config
                            .batch_counterfactual_completion_config_path
                            .clone()
                            .into_iter()
                            .collect(),
                        official_candle_pack_paths: sweep_config
                            .as_ref()
                            .map(|cfg| cfg.official_candle_pack_paths.clone())
                            .unwrap_or_default(),
                        output_root: config.output_root.clone(),
                        ..OfficialEvidenceDiversityGapConfig::default()
                    },
                    Some(&current_set),
                    batch_outcome_linkage_report.as_ref(),
                    batch_counterfactual_completion_report.as_ref(),
                    &[],
                )
            }
        } else {
            previous_gap_map.clone().unwrap_or_else(|| {
                OfficialEvidenceDiversityGapRunner::default().run_from_inputs(
                    &OfficialEvidenceDiversityGapConfig::default(),
                    Some(&current_set),
                    batch_outcome_linkage_report.as_ref(),
                    batch_counterfactual_completion_report.as_ref(),
                    &[],
                )
            })
        };

        let outcome_diversity_audit_report = OutcomeDiversityAuditRunner::default()
            .run_from_inputs(
                &OutcomeDiversityAuditConfig {
                    audit_id: config.run_id.clone(),
                    batch_outcome_linkage_paths: config
                        .batch_outcome_linkage_config_path
                        .clone()
                        .into_iter()
                        .collect(),
                    batch_counterfactual_completion_paths: config
                        .batch_counterfactual_completion_config_path
                        .clone()
                        .into_iter()
                        .collect(),
                    multi_row_set_paths: sweep_config
                        .as_ref()
                        .map(|cfg| cfg.multi_row_set_config_paths.clone())
                        .unwrap_or_default(),
                    output_root: config.output_root.clone(),
                    ..OutcomeDiversityAuditConfig::default()
                },
                batch_outcome_linkage_report.as_ref(),
                batch_counterfactual_completion_report.as_ref(),
                Some(&current_set),
            );

        let balanced_outcome_coverage_report = BalancedOutcomeCoverageRunner::default()
            .run_from_inputs(
                &BalancedOutcomeCoverageConfig {
                    coverage_id: config.run_id.clone(),
                    multi_row_set_paths: sweep_config
                        .as_ref()
                        .map(|cfg| cfg.multi_row_set_config_paths.clone())
                        .unwrap_or_default(),
                    batch_outcome_linkage_paths: config
                        .batch_outcome_linkage_config_path
                        .clone()
                        .into_iter()
                        .collect(),
                    batch_counterfactual_completion_paths: config
                        .batch_counterfactual_completion_config_path
                        .clone()
                        .into_iter()
                        .collect(),
                    barrier_profile_registry_path: registry_path.clone(),
                    output_root: config.output_root.clone(),
                    ..BalancedOutcomeCoverageConfig::default()
                },
                Some(&current_set),
                batch_outcome_linkage_report.as_ref(),
                batch_counterfactual_completion_report.as_ref(),
                registry.as_ref(),
            );

        let diversity_aware_sufficiency_v2_report = if config.run_sufficiency_v2 {
            if let Some(path) = config.sufficiency_v2_config_path.as_deref() {
                if path.ends_with(".json") {
                    super::diversity_aware_sufficiency_v2::load_diversity_aware_sufficiency_v2_from_path_or_config(path)?
                } else {
                    DiversityAwareSufficiencyV2Runner::default().run(
                        &DiversityAwareSufficiencyV2Config::from_toml_path(Path::new(path))?,
                    )?
                }
            } else {
                DiversityAwareSufficiencyV2Runner::default().run_from_inputs(
                    &DiversityAwareSufficiencyV2Config {
                        sufficiency_id: config.run_id.clone(),
                        multi_row_set_paths: sweep_config
                            .as_ref()
                            .map(|cfg| cfg.multi_row_set_config_paths.clone())
                            .unwrap_or_default(),
                        batch_outcome_linkage_paths: config
                            .batch_outcome_linkage_config_path
                            .clone()
                            .into_iter()
                            .collect(),
                        batch_counterfactual_completion_paths: config
                            .batch_counterfactual_completion_config_path
                            .clone()
                            .into_iter()
                            .collect(),
                        barrier_profile_registry_path: registry_path.clone(),
                        output_root: config.output_root.clone(),
                        ..DiversityAwareSufficiencyV2Config::default()
                    },
                    &current_set,
                    batch_outcome_linkage_report.as_ref(),
                    batch_counterfactual_completion_report.as_ref(),
                    Some(&outcome_diversity_audit_report),
                    Some(&balanced_outcome_coverage_report),
                    registry.as_ref(),
                )
            }
        } else {
            DiversityAwareSufficiencyV2Runner::default().run_from_inputs(
                &DiversityAwareSufficiencyV2Config::default(),
                &current_set,
                batch_outcome_linkage_report.as_ref(),
                batch_counterfactual_completion_report.as_ref(),
                Some(&outcome_diversity_audit_report),
                Some(&balanced_outcome_coverage_report),
                registry.as_ref(),
            )
        };

        let committee_benchmark_summary = if config.run_committee_official_benchmark {
            config
                .committee_official_benchmark_config_path
                .as_deref()
                .map(|path| {
                    let benchmark_config =
                        CommitteeOfficialBenchmarkConfig::from_toml_path(Path::new(path))?;
                    let bundle = CommitteeOfficialBenchmarkRunner::default()
                        .run_bundle(&benchmark_config)?;
                    Ok::<_, String>(bundle.final_summary)
                })
                .transpose()?
        } else {
            None
        };
        let outcome_coverage_summary = if config.run_outcome_coverage {
            config
                .outcome_coverage_config_path
                .as_deref()
                .map(|path| {
                    let coverage_config =
                        CommitteeOutcomeCoverageConfig::from_toml_path(Path::new(path))?;
                    let bundle = CommitteeOutcomeCoverageRunner::default().run(&coverage_config)?;
                    Ok::<_, String>(bundle.to_text())
                })
                .transpose()?
        } else {
            None
        };
        let counterfactual_depth_summary = if config.run_counterfactual_depth_close {
            config
                .counterfactual_depth_closure_config_path
                .as_deref()
                .map(|path| {
                    let depth_config =
                        CounterfactualDepthClosureConfig::from_toml_path(Path::new(path))?;
                    let bundle =
                        CounterfactualDepthClosureRunner::default().run_bundle(&depth_config)?;
                    Ok::<_, String>(bundle.final_summary)
                })
                .transpose()?
        } else {
            None
        };
        let core_performance_summary = if config.run_core_performance {
            config
                .core_performance_config_path
                .as_deref()
                .map(|path| {
                    let performance_config =
                        CorePerformanceScorecardConfig::from_toml_path(Path::new(path))?;
                    let bundle =
                        CorePerformanceScorecardRunner::default().run(&performance_config)?;
                    let summary = CoreScorecardRerun::default().summarize(
                        None,
                        Some(&bundle.scorecard),
                        vec![
                            "diversity sweep core rerun has no previous baseline in-config"
                                .to_string(),
                        ],
                        true,
                    );
                    Ok::<_, String>(summary)
                })
                .transpose()?
        } else {
            None
        };

        let diversity_sweep_report = build_diversity_sweep_report(
            config,
            previous_gap_map.as_ref(),
            &current_gap_map,
            &outcome_diversity_audit_report,
            &diversity_aware_sufficiency_v2_report,
            core_performance_summary.as_ref(),
        );
        let storage_report = build_storage_report(
            config.max_bytes,
            row_selector_report.as_ref(),
            future_window_scaleout_plan.as_ref(),
            batch_outcome_linkage_report.as_ref(),
            batch_counterfactual_completion_report.as_ref(),
            &outcome_diversity_audit_report,
            &balanced_outcome_coverage_report,
            &diversity_aware_sufficiency_v2_report,
            committee_benchmark_summary.as_ref(),
            outcome_coverage_summary.as_ref(),
            counterfactual_depth_summary.as_ref(),
            core_performance_summary.as_ref(),
        );

        let mut bundle = OfficialEvidenceDiversitySweepBundle {
            barrier_profile_registry: registry,
            diversity_gap_map: current_gap_map,
            row_selector_report,
            batch_outcome_linkage_report,
            batch_counterfactual_completion_report,
            outcome_diversity_audit_report,
            balanced_outcome_coverage_report,
            diversity_aware_sufficiency_v2_report,
            committee_benchmark_summary,
            outcome_coverage_summary,
            counterfactual_depth_summary,
            core_performance_summary,
            diversity_sweep_report,
            storage_report,
            final_summary: String::new(),
            reason_codes: stable_reason_codes(
                &config
                    .reason_codes
                    .iter()
                    .cloned()
                    .chain([
                        ReasonCode::DeterministicPath,
                        ReasonCode::OfficialEvidenceCounted,
                        ReasonCode::LocalFileOnly,
                    ])
                    .collect::<Vec<_>>(),
            ),
        };
        bundle.final_summary = build_official_evidence_diversity_summary(&bundle);
        bundle.write_to_dir(&config.output_dir())?;
        Ok(bundle)
    }
}

impl OfficialEvidenceDiversitySweepReport {
    pub fn fingerprint(&self) -> String {
        stable_hash_string(&serde_json::to_string(self).unwrap_or_else(|_| self.run_id.clone()))
    }

    pub fn to_text(&self) -> String {
        [
            format!("run_id={}", self.run_id),
            format!(
                "previous_official_complete_rows={}",
                self.previous_official_complete_rows
                    .map(|value| value.to_string())
                    .unwrap_or_default()
            ),
            format!(
                "current_official_complete_rows={}",
                self.current_official_complete_rows
            ),
            format!(
                "previous_outcome_diversity_status={}",
                self.previous_outcome_diversity_status
                    .map(|status| format!("{status:?}"))
                    .unwrap_or_default()
            ),
            format!(
                "current_outcome_diversity_status={:?}",
                self.current_outcome_diversity_status
            ),
            format!(
                "previous_sufficiency_status={}",
                self.previous_sufficiency_status
                    .map(|status| format!("{status:?}"))
                    .unwrap_or_default()
            ),
            format!(
                "current_sufficiency_status={:?}",
                self.current_sufficiency_status
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
            format!(
                "added_official_complete_rows={}",
                self.added_official_complete_rows
            ),
            format!("added_symbols={}", self.added_symbols),
            format!("added_timeframes={}", self.added_timeframes),
            format!("added_horizons={}", self.added_horizons),
            format!("added_take_profit={}", self.added_take_profit),
            format!("added_stop_loss={}", self.added_stop_loss),
            format!("added_time_expired={}", self.added_time_expired),
            format!(
                "added_no_trade_counterfactuals={}",
                self.added_no_trade_counterfactuals
            ),
            format!(
                "added_risk_denied_counterfactuals={}",
                self.added_risk_denied_counterfactuals
            ),
            format!("bottleneck_changed={}", self.bottleneck_changed),
            format!("final_status={:?}", self.final_status),
            format!("final_recommendation={:?}", self.final_recommendation),
            format!("blockers={}", self.blockers.join(" | ")),
            format!("warnings={}", self.warnings.join(" | ")),
            format!("fingerprint={}", self.fingerprint()),
        ]
        .join("\n")
    }
}

impl OfficialEvidenceDiversityStorageReport {
    pub fn to_text(&self) -> String {
        [
            format!("candidate_bytes={}", self.candidate_bytes),
            format!("future_window_bytes={}", self.future_window_bytes),
            format!("outcome_linkage_bytes={}", self.outcome_linkage_bytes),
            format!("counterfactual_bytes={}", self.counterfactual_bytes),
            format!("coverage_bytes={}", self.coverage_bytes),
            format!("sufficiency_bytes={}", self.sufficiency_bytes),
            format!("benchmark_bytes={}", self.benchmark_bytes),
            format!("core_scorecard_bytes={}", self.core_scorecard_bytes),
            format!("bundle_bytes={}", self.bundle_bytes),
            format!("total_bytes={}", self.total_bytes),
            format!("budget_exceeded={}", self.budget_exceeded),
            format!("largest_artifacts={}", self.largest_artifacts.join(" | ")),
            format!(
                "compaction_recommendation={}",
                self.compaction_recommendation
            ),
        ]
        .join("\n")
    }
}

fn build_diversity_sweep_report(
    config: &OfficialEvidenceDiversitySweepConfig,
    previous_gap_map: Option<&OfficialEvidenceDiversityGapMap>,
    current_gap_map: &OfficialEvidenceDiversityGapMap,
    current_outcome_audit: &OutcomeDiversityAuditReport,
    current_sufficiency: &DiversityAwareSufficiencyV2Report,
    core_summary: Option<&CoreScorecardRerunSummary>,
) -> OfficialEvidenceDiversitySweepReport {
    let previous_official_complete_rows =
        previous_gap_map.map(|map| map.current_official_complete_rows);
    let previous_outcome_diversity_status = previous_gap_map.map(|map| match map.gap_status {
        OfficialEvidenceDiversityGapStatus::SingleOutcomeDominated => {
            OutcomeDiversityStatus::SingleOutcomeDominated
        }
        OfficialEvidenceDiversityGapStatus::NeedTimeExpiredOutcomes => {
            OutcomeDiversityStatus::MissingTimeExpired
        }
        OfficialEvidenceDiversityGapStatus::NeedStopLossOutcomes => {
            OutcomeDiversityStatus::MissingStopLoss
        }
        OfficialEvidenceDiversityGapStatus::DiagnosticOnly => {
            OutcomeDiversityStatus::DiagnosticOnly
        }
        _ => OutcomeDiversityStatus::InsufficientDiversity,
    });
    let previous_sufficiency_status = None;
    let previous_core_status = core_summary.and_then(|_| None);
    let previous_primary_bottleneck = None;
    let current_core_status =
        core_summary.and_then(|summary| summary.current_status.map(|status| format!("{status:?}")));
    let current_primary_bottleneck = core_summary.and_then(|summary| {
        summary
            .current_primary_bottleneck
            .map(|status| format!("{status:?}"))
    });
    let added_official_complete_rows = current_gap_map.current_official_complete_rows as isize
        - previous_official_complete_rows.unwrap_or(current_gap_map.current_official_complete_rows)
            as isize;
    let added_symbols = current_gap_map.current_symbols as isize
        - previous_gap_map
            .map(|map| map.current_symbols)
            .unwrap_or(current_gap_map.current_symbols) as isize;
    let added_timeframes = current_gap_map.current_timeframes as isize
        - previous_gap_map
            .map(|map| map.current_timeframes)
            .unwrap_or(current_gap_map.current_timeframes) as isize;
    let added_horizons = current_gap_map.current_horizons as isize
        - previous_gap_map
            .map(|map| map.current_horizons)
            .unwrap_or(current_gap_map.current_horizons) as isize;
    let added_take_profit = current_gap_map.current_take_profit as isize
        - previous_gap_map
            .map(|map| map.current_take_profit)
            .unwrap_or(current_gap_map.current_take_profit) as isize;
    let added_stop_loss = current_gap_map.current_stop_loss as isize
        - previous_gap_map
            .map(|map| map.current_stop_loss)
            .unwrap_or(current_gap_map.current_stop_loss) as isize;
    let added_time_expired = current_gap_map.current_time_expired as isize
        - previous_gap_map
            .map(|map| map.current_time_expired)
            .unwrap_or(current_gap_map.current_time_expired) as isize;
    let added_no_trade_counterfactuals = current_gap_map.current_no_trade_counterfactuals as isize
        - previous_gap_map
            .map(|map| map.current_no_trade_counterfactuals)
            .unwrap_or(current_gap_map.current_no_trade_counterfactuals) as isize;
    let added_risk_denied_counterfactuals = current_gap_map.current_risk_denied_counterfactuals
        as isize
        - previous_gap_map
            .map(|map| map.current_risk_denied_counterfactuals)
            .unwrap_or(current_gap_map.current_risk_denied_counterfactuals) as isize;
    let bottleneck_changed = core_summary
        .map(|summary| summary.bottleneck_changed)
        .unwrap_or(false);
    let final_status = determine_final_status(
        current_sufficiency,
        current_outcome_audit,
        added_official_complete_rows,
    );
    let final_recommendation =
        determine_final_recommendation(current_gap_map, current_sufficiency, config);
    let blockers = if current_sufficiency.failed_gates.is_empty() {
        Vec::new()
    } else {
        current_sufficiency.failed_gates.clone()
    };
    let warnings = vec![
        "official evidence diversity sweep remains research-only, paper-only, local-only, and conservative"
            .to_string(),
    ];
    OfficialEvidenceDiversitySweepReport {
        run_id: config.run_id.clone(),
        previous_official_complete_rows,
        current_official_complete_rows: current_gap_map.current_official_complete_rows,
        previous_outcome_diversity_status,
        current_outcome_diversity_status: current_outcome_audit.outcome_diversity_status,
        previous_sufficiency_status,
        current_sufficiency_status: current_sufficiency.final_status,
        previous_core_status,
        current_core_status,
        previous_primary_bottleneck,
        current_primary_bottleneck,
        added_official_complete_rows,
        added_symbols,
        added_timeframes,
        added_horizons,
        added_take_profit,
        added_stop_loss,
        added_time_expired,
        added_no_trade_counterfactuals,
        added_risk_denied_counterfactuals,
        bottleneck_changed,
        final_status,
        final_recommendation,
        blockers,
        warnings,
        reason_codes: stable_reason_codes(&[
            ReasonCode::DeterministicPath,
            ReasonCode::OfficialEvidenceCounted,
        ]),
    }
}

fn determine_final_status(
    sufficiency: &DiversityAwareSufficiencyV2Report,
    audit: &OutcomeDiversityAuditReport,
    added_official_complete_rows: isize,
) -> OfficialEvidenceDiversitySweepStatus {
    if sufficiency.passed_tentative_signal_quality_review {
        return OfficialEvidenceDiversitySweepStatus::TentativeSignalQualityReviewReady;
    }
    if sufficiency.passed_committee_benchmark_research {
        return OfficialEvidenceDiversitySweepStatus::CommitteeBenchmarkResearchReady;
    }
    if added_official_complete_rows > 0 {
        return OfficialEvidenceDiversitySweepStatus::OfficialCompleteRowsExpanded;
    }
    if matches!(
        audit.outcome_diversity_status,
        OutcomeDiversityStatus::HealthyOutcomeDiversity
    ) {
        return OfficialEvidenceDiversitySweepStatus::OutcomeDiversityImproved;
    }
    if matches!(
        sufficiency.final_status,
        DiversityAwareSufficiencyV2Status::PlumbingValidated
    ) {
        return OfficialEvidenceDiversitySweepStatus::PlumbingValidated;
    }
    if matches!(
        audit.outcome_diversity_status,
        OutcomeDiversityStatus::SingleOutcomeDominated
    ) {
        return OfficialEvidenceDiversitySweepStatus::StillSingleOutcomeDominated;
    }
    if matches!(
        sufficiency.final_status,
        DiversityAwareSufficiencyV2Status::NeedMoreCounterfactualDepth
    ) {
        return OfficialEvidenceDiversitySweepStatus::StillNeedCounterfactualDepth;
    }
    if matches!(
        sufficiency.final_status,
        DiversityAwareSufficiencyV2Status::NeedMoreOfficialRows
    ) {
        return OfficialEvidenceDiversitySweepStatus::StillNeedMoreOfficialRows;
    }
    if matches!(
        sufficiency.final_status,
        DiversityAwareSufficiencyV2Status::NeedMoreOutcomeDiversity
    ) {
        return OfficialEvidenceDiversitySweepStatus::StillNeedOutcomeDiversity;
    }
    OfficialEvidenceDiversitySweepStatus::NoImprovement
}

fn determine_final_recommendation(
    gap_map: &OfficialEvidenceDiversityGapMap,
    sufficiency: &DiversityAwareSufficiencyV2Report,
    config: &OfficialEvidenceDiversitySweepConfig,
) -> OfficialEvidenceDiversitySweepRecommendation {
    if matches!(
        sufficiency.final_status,
        DiversityAwareSufficiencyV2Status::NeedMoreCounterfactualDepth
    ) {
        return OfficialEvidenceDiversitySweepRecommendation::MoreCounterfactualDepth;
    }
    if matches!(
        gap_map.gap_status,
        OfficialEvidenceDiversityGapStatus::NeedTimeExpiredOutcomes
    ) {
        return OfficialEvidenceDiversitySweepRecommendation::MoreTimeExpiredExamples;
    }
    if matches!(
        gap_map.gap_status,
        OfficialEvidenceDiversityGapStatus::NeedStopLossOutcomes
    ) {
        return OfficialEvidenceDiversitySweepRecommendation::MoreStopLossExamples;
    }
    if matches!(
        gap_map.gap_status,
        OfficialEvidenceDiversityGapStatus::NeedMoreSymbols
            | OfficialEvidenceDiversityGapStatus::SingleSymbolDominated
    ) {
        return OfficialEvidenceDiversitySweepRecommendation::MoreOfficialSymbols;
    }
    if matches!(
        gap_map.gap_status,
        OfficialEvidenceDiversityGapStatus::NeedMoreTimeframes
    ) {
        return OfficialEvidenceDiversitySweepRecommendation::MoreTimeframes;
    }
    if matches!(
        gap_map.gap_status,
        OfficialEvidenceDiversityGapStatus::NeedMoreHorizons
    ) {
        return OfficialEvidenceDiversitySweepRecommendation::MoreHorizons;
    }
    if config.committee_official_benchmark_config_path.is_some()
        && !sufficiency.passed_committee_benchmark_research
    {
        return OfficialEvidenceDiversitySweepRecommendation::RunCommitteeOfficialBenchmark;
    }
    if config.core_performance_config_path.is_some() {
        return OfficialEvidenceDiversitySweepRecommendation::RunCorePerformance;
    }
    if matches!(
        sufficiency.final_status,
        DiversityAwareSufficiencyV2Status::NeedMoreOfficialRows
    ) {
        return OfficialEvidenceDiversitySweepRecommendation::MoreOfficialRows;
    }
    OfficialEvidenceDiversitySweepRecommendation::KeepTrinity
}

#[allow(clippy::too_many_arguments)]
fn build_storage_report(
    max_bytes: usize,
    row_selector_report: Option<&OfficialDiversityRowSelectorReport>,
    future_window_plan: Option<&FutureWindowScaleOutPlan>,
    batch_outcome_report: Option<&BatchOutcomeLinkageReportLike>,
    batch_counterfactual_report: Option<&BatchCounterfactualReportLike>,
    outcome_audit: &OutcomeDiversityAuditReport,
    balanced_coverage: &BalancedOutcomeCoverageReport,
    sufficiency: &DiversityAwareSufficiencyV2Report,
    committee_benchmark_summary: Option<&String>,
    outcome_coverage_summary: Option<&String>,
    counterfactual_depth_summary: Option<&String>,
    core_summary: Option<&CoreScorecardRerunSummary>,
) -> OfficialEvidenceDiversityStorageReport {
    let candidate_bytes = row_selector_report
        .and_then(|report| serde_json::to_vec(report).ok())
        .map(|bytes| bytes.len())
        .unwrap_or_default();
    let future_window_bytes = future_window_plan
        .and_then(|plan| serde_json::to_vec(plan).ok())
        .map(|bytes| bytes.len())
        .unwrap_or_default();
    let outcome_linkage_bytes = batch_outcome_report
        .and_then(|report| serde_json::to_vec(report).ok())
        .map(|bytes| bytes.len())
        .unwrap_or_default();
    let counterfactual_bytes = batch_counterfactual_report
        .and_then(|report| serde_json::to_vec(report).ok())
        .map(|bytes| bytes.len())
        .unwrap_or_default();
    let coverage_bytes = serde_json::to_vec(outcome_audit)
        .map(|bytes| bytes.len())
        .unwrap_or_default()
        + serde_json::to_vec(balanced_coverage)
            .map(|bytes| bytes.len())
            .unwrap_or_default();
    let sufficiency_bytes = serde_json::to_vec(sufficiency)
        .map(|bytes| bytes.len())
        .unwrap_or_default();
    let benchmark_bytes = committee_benchmark_summary
        .map(|summary| summary.len())
        .unwrap_or_default()
        + outcome_coverage_summary
            .map(|summary| summary.len())
            .unwrap_or_default()
        + counterfactual_depth_summary
            .map(|summary| summary.len())
            .unwrap_or_default();
    let core_scorecard_bytes = core_summary
        .and_then(|summary| serde_json::to_vec(summary).ok())
        .map(|bytes| bytes.len())
        .unwrap_or_default();
    let mut artifact_sizes = vec![
        ("candidate_bytes".to_string(), candidate_bytes),
        ("future_window_bytes".to_string(), future_window_bytes),
        ("outcome_linkage_bytes".to_string(), outcome_linkage_bytes),
        ("counterfactual_bytes".to_string(), counterfactual_bytes),
        ("coverage_bytes".to_string(), coverage_bytes),
        ("sufficiency_bytes".to_string(), sufficiency_bytes),
        ("benchmark_bytes".to_string(), benchmark_bytes),
        ("core_scorecard_bytes".to_string(), core_scorecard_bytes),
    ];
    artifact_sizes.sort_by(|left, right| right.1.cmp(&left.1).then(left.0.cmp(&right.0)));
    let bundle_bytes = artifact_sizes
        .iter()
        .map(|(_, bytes)| *bytes)
        .sum::<usize>();
    let total_bytes = bundle_bytes;
    let budget_exceeded = total_bytes > max_bytes;
    let compaction_recommendation = if budget_exceeded {
        "reduce retained summaries or disable optional reruns before expanding artifact budget"
            .to_string()
    } else {
        "no compaction needed under current bounded sweep budget".to_string()
    };
    OfficialEvidenceDiversityStorageReport {
        candidate_bytes,
        future_window_bytes,
        outcome_linkage_bytes,
        counterfactual_bytes,
        coverage_bytes,
        sufficiency_bytes,
        benchmark_bytes,
        core_scorecard_bytes,
        bundle_bytes,
        total_bytes,
        budget_exceeded,
        largest_artifacts: artifact_sizes
            .into_iter()
            .take(3)
            .map(|(name, bytes)| format!("{name}={bytes}"))
            .collect(),
        compaction_recommendation,
        reason_codes: stable_reason_codes(&[
            ReasonCode::DeterministicPath,
            ReasonCode::StorageBudgetReportBuilt,
        ]),
    }
}

type BatchOutcomeLinkageReportLike = BatchOutcomeLinkageV3Report;
type BatchCounterfactualReportLike = BatchCounterfactualCompletionReport;

fn default_output_root() -> String {
    "target/soma_official_evidence_diversity".to_string()
}

fn default_true() -> bool {
    true
}

fn default_max_rows() -> usize {
    1000
}

fn default_max_symbols() -> usize {
    10
}

fn default_max_bytes() -> usize {
    5_000_000
}
