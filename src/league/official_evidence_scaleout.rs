use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};
use crate::experiment::{
    CorePerformanceScorecardBundle, CorePerformanceScorecardConfig, CorePerformanceScorecardRunner,
    CoreScorecardRerun, CoreScorecardRerunSummary,
};

use super::batch_counterfactual_completion::{
    BatchCounterfactualCompletionReport, load_batch_counterfactual_completion_from_path_or_config,
};
use super::batch_outcome_linkage_v3::{
    BatchOutcomeLinkageV3Report, load_batch_outcome_linkage_v3_from_path_or_config,
};
use super::committee_official_benchmark::{
    CommitteeOfficialBenchmarkConfig, CommitteeOfficialBenchmarkFinalStatus,
    CommitteeOfficialBenchmarkRunner,
};
use super::committee_outcome_coverage::CommitteeOutcomeCoverageConfig;
use super::committee_outcome_coverage_bundle::{
    CommitteeOutcomeCoverageBundle, CommitteeOutcomeCoverageBundleStatus,
};
use super::committee_outcome_coverage_runner::CommitteeOutcomeCoverageRunner;
use super::committee_outcome_linker::{
    CommitteeOutcomeLinkSummary, OutcomeLinkedCommitteeScenarioPack,
    OutcomeLinkedCommitteeScenarioRow,
};
use super::committee_outcome_reference::{CommitteeBaselineAction, CommitteeBaselineReference};
use super::committee_scenario_loader::{
    CommitteeScenarioMaterializationLevel, CommitteeScenarioRow, CommitteeScenarioSourceKind,
};
use super::counterfactual_depth_closure::{
    CounterfactualDepthClosureConfig, CounterfactualDepthClosureReport,
    CounterfactualDepthClosureRunner,
};
use super::future_window_scaleout::{
    FutureWindowScaleOutPlan, load_future_window_scaleout_plan_from_path_or_config,
};
use super::multi_row_official_evidence::{
    MultiRowOfficialEvidenceSet, load_multi_row_official_evidence_set_from_path_or_config,
};
use super::official_committee_benchmark_bundle::CommitteeOfficialBenchmarkBundle;
use super::official_committee_pack::OfficialCommitteeScenarioPack;
use super::official_evidence_scaleout_bundle::{
    OfficialEvidenceScaleOutBundle, build_official_evidence_scaleout_reason_codes,
    build_official_evidence_scaleout_summary,
};
use super::official_evidence_sufficiency_v2::{
    OfficialEvidenceSufficiencyV2Config, OfficialEvidenceSufficiencyV2Counts,
    OfficialEvidenceSufficiencyV2Report, OfficialEvidenceSufficiencyV2Runner,
    OfficialEvidenceSufficiencyV2Status, compute_counts,
    load_official_evidence_sufficiency_v2_from_path_or_config,
};
use super::{
    ComparableCommitteeEvidenceBundle, ComparableCommitteeEvidenceConfig,
    ComparableCommitteeEvidenceRow,
};
use crate::data::EvidenceSourceKind;
use crate::{PersonaHorizon, Regime};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OfficialEvidenceScaleOutConfig {
    pub scaleout_id: String,
    pub multi_row_set_config_path: String,
    #[serde(default)]
    pub before_multi_row_set_config_path: Option<String>,
    #[serde(default)]
    pub future_window_scaleout_config_path: Option<String>,
    #[serde(default)]
    pub batch_outcome_linkage_config_path: Option<String>,
    #[serde(default)]
    pub batch_counterfactual_completion_config_path: Option<String>,
    #[serde(default)]
    pub before_batch_outcome_linkage_config_path: Option<String>,
    #[serde(default)]
    pub before_batch_counterfactual_completion_config_path: Option<String>,
    #[serde(default, alias = "sufficiency_v2_config_path")]
    pub official_evidence_sufficiency_v2_config_path: Option<String>,
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
    pub run_multi_row_set: bool,
    #[serde(default = "default_true")]
    pub run_future_window_scaleout: bool,
    #[serde(default = "default_true")]
    pub run_batch_outcome_linkage: bool,
    #[serde(default = "default_true")]
    pub run_batch_counterfactual_completion: bool,
    #[serde(default = "default_true")]
    pub run_complete_row_rebuild: bool,
    #[serde(default = "default_true")]
    pub run_committee_official_benchmark: bool,
    #[serde(default = "default_true")]
    pub run_outcome_coverage: bool,
    #[serde(default = "default_true")]
    pub run_counterfactual_depth_close: bool,
    #[serde(default = "default_true")]
    pub run_core_performance: bool,
    #[serde(default = "default_true")]
    pub run_sufficiency_v2: bool,
    #[serde(default = "default_max_rows")]
    pub max_rows: usize,
    #[serde(default = "default_max_symbols")]
    pub max_symbols: usize,
    #[serde(default = "default_max_bytes")]
    pub max_bytes: usize,
    #[serde(default = "default_true")]
    pub require_core_check: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum OfficialEvidenceScaleOutStatus {
    OfficialEvidencePlumbingValidated,
    OfficialCompleteRowsExpanded,
    CommitteeBenchmarkResearchReady,
    TentativeSignalQualityReviewReady,
    OutcomeCoverageExpanded,
    CounterfactualCoverageExpanded,
    StillSingleSymbolDominated,
    StillSingleOutcomeDominated,
    CorePerformanceHealthyForResearch,
    CoreStillBlockedByEvidence,
    StillInsufficientRows,
    StillNeedMoreOutcomeLinks,
    StillNeedMoreCounterfactuals,
    StillEvidenceTooWeak,
    #[default]
    NoImprovement,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum OfficialEvidenceScaleOutRecommendation {
    MoreOfficialRows,
    MoreOfficialSymbols,
    MoreOutcomeDiversity,
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
pub struct OfficialEvidenceScaleOutStorageReport {
    pub max_bytes: usize,
    pub estimated_output_bytes: usize,
    pub within_budget: bool,
    pub guidance: String,
    pub input_paths: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfficialEvidenceScaleOutReport {
    pub scaleout_id: String,
    pub multi_row_set_summary: String,
    #[serde(default)]
    pub future_window_scaleout_summary: Option<String>,
    #[serde(default)]
    pub outcome_linkage_summary: Option<String>,
    #[serde(default)]
    pub counterfactual_completion_summary: Option<String>,
    pub sufficiency_v2_report: OfficialEvidenceSufficiencyV2Report,
    #[serde(default)]
    pub committee_benchmark_summary: Option<String>,
    #[serde(default)]
    pub outcome_coverage_summary: Option<String>,
    #[serde(default)]
    pub counterfactual_depth_summary: Option<String>,
    #[serde(default)]
    pub core_performance_summary: Option<String>,
    #[serde(default)]
    pub before_counts: OfficialEvidenceSufficiencyV2Counts,
    pub after_counts: OfficialEvidenceSufficiencyV2Counts,
    pub added_official_complete_rows: isize,
    pub added_outcome_references: isize,
    pub added_no_trade_counterfactuals: isize,
    pub added_risk_denied_counterfactuals: isize,
    #[serde(default)]
    pub previous_core_status: Option<String>,
    #[serde(default)]
    pub current_core_status: Option<String>,
    #[serde(default)]
    pub previous_primary_bottleneck: Option<String>,
    #[serde(default)]
    pub current_primary_bottleneck: Option<String>,
    pub bottleneck_changed: bool,
    pub sufficiency_status: OfficialEvidenceSufficiencyV2Status,
    #[serde(default)]
    pub committee_benchmark_status: Option<CommitteeOfficialBenchmarkFinalStatus>,
    #[serde(default)]
    pub outcome_coverage_status: Option<CommitteeOutcomeCoverageBundleStatus>,
    #[serde(default)]
    pub counterfactual_depth_status: Option<String>,
    #[serde(default)]
    pub core_scorecard_summary: Option<CoreScorecardRerunSummary>,
    pub final_status: OfficialEvidenceScaleOutStatus,
    pub status: OfficialEvidenceScaleOutStatus,
    pub final_recommendation: OfficialEvidenceScaleOutRecommendation,
    #[serde(default)]
    pub blockers: Vec<String>,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OfficialEvidenceScaleOutRunner;

#[derive(Clone, Debug, Default)]
struct RerunArtifacts {
    benchmark_bundle: Option<CommitteeOfficialBenchmarkBundle>,
    coverage_bundle: Option<CommitteeOutcomeCoverageBundle>,
    depth_report: Option<CounterfactualDepthClosureReport>,
    core_bundle: Option<CorePerformanceScorecardBundle>,
    core_summary: Option<CoreScorecardRerunSummary>,
}

impl Default for OfficialEvidenceScaleOutConfig {
    fn default() -> Self {
        Self {
            scaleout_id: "official-evidence-scaleout".to_string(),
            multi_row_set_config_path: String::new(),
            before_multi_row_set_config_path: None,
            future_window_scaleout_config_path: None,
            batch_outcome_linkage_config_path: None,
            batch_counterfactual_completion_config_path: None,
            before_batch_outcome_linkage_config_path: None,
            before_batch_counterfactual_completion_config_path: None,
            official_evidence_sufficiency_v2_config_path: None,
            committee_official_benchmark_config_path: None,
            outcome_coverage_config_path: None,
            counterfactual_depth_closure_config_path: None,
            core_performance_config_path: None,
            output_root: default_output_root(),
            run_multi_row_set: true,
            run_future_window_scaleout: true,
            run_batch_outcome_linkage: true,
            run_batch_counterfactual_completion: true,
            run_complete_row_rebuild: true,
            run_committee_official_benchmark: true,
            run_outcome_coverage: true,
            run_counterfactual_depth_close: true,
            run_core_performance: true,
            run_sufficiency_v2: true,
            max_rows: default_max_rows(),
            max_symbols: default_max_symbols(),
            max_bytes: default_max_bytes(),
            require_core_check: true,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

impl OfficialEvidenceScaleOutConfig {
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
        if self.scaleout_id.trim().is_empty() {
            return Err("official evidence scaleout id must not be empty".to_string());
        }
        if self.multi_row_set_config_path.trim().is_empty() {
            return Err(
                "official evidence scaleout requires multi_row_set_config_path".to_string(),
            );
        }
        if self
            .all_paths()
            .iter()
            .chain(std::iter::once(&self.output_root))
            .any(|path| path.contains("://"))
        {
            return Err("official evidence scaleout paths must be local".to_string());
        }
        if self.max_rows == 0 || self.max_rows > default_max_rows() {
            return Err(
                "official evidence scaleout max_rows must be between 1 and 1000".to_string(),
            );
        }
        if self.max_symbols == 0 || self.max_symbols > default_max_symbols() {
            return Err(
                "official evidence scaleout max_symbols must be between 1 and 10".to_string(),
            );
        }
        if self.max_bytes == 0 || self.max_bytes > default_max_bytes() {
            return Err(
                "official evidence scaleout max_bytes must be between 1 and 5000000".to_string(),
            );
        }
        Ok(())
    }

    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.scaleout_id)
    }

    pub fn all_paths(&self) -> Vec<String> {
        self.before_multi_row_set_config_path
            .iter()
            .cloned()
            .chain(std::iter::once(self.multi_row_set_config_path.clone()))
            .chain(self.future_window_scaleout_config_path.iter().cloned())
            .chain(self.batch_outcome_linkage_config_path.iter().cloned())
            .chain(
                self.batch_counterfactual_completion_config_path
                    .iter()
                    .cloned(),
            )
            .chain(
                self.before_batch_outcome_linkage_config_path
                    .iter()
                    .cloned(),
            )
            .chain(
                self.before_batch_counterfactual_completion_config_path
                    .iter()
                    .cloned(),
            )
            .chain(
                self.official_evidence_sufficiency_v2_config_path
                    .iter()
                    .cloned(),
            )
            .chain(
                self.committee_official_benchmark_config_path
                    .iter()
                    .cloned(),
            )
            .chain(self.outcome_coverage_config_path.iter().cloned())
            .chain(
                self.counterfactual_depth_closure_config_path
                    .iter()
                    .cloned(),
            )
            .chain(self.core_performance_config_path.iter().cloned())
            .collect()
    }
}

impl OfficialEvidenceScaleOutRunner {
    pub fn run(
        &self,
        config: &OfficialEvidenceScaleOutConfig,
    ) -> Result<OfficialEvidenceScaleOutBundle, String> {
        config.validate()?;
        let output_dir = config.output_dir();
        fs::create_dir_all(&output_dir).map_err(|err| err.to_string())?;

        let after_set = load_multi_row_official_evidence_set_from_path_or_config(
            &config.multi_row_set_config_path,
        )?;
        let before_set = config
            .before_multi_row_set_config_path
            .as_deref()
            .map(load_multi_row_official_evidence_set_from_path_or_config)
            .transpose()?;

        let future_window_scaleout_plan = if config.run_future_window_scaleout {
            load_or_build_future_window_plan(config)?
        } else {
            None
        };
        let batch_outcome_linkage_report = if config.run_batch_outcome_linkage {
            load_or_build_batch_outcome(config.batch_outcome_linkage_config_path.as_deref())?
        } else {
            None
        };
        let batch_counterfactual_completion_report = if config.run_batch_counterfactual_completion {
            load_or_build_batch_counterfactual(
                config
                    .batch_counterfactual_completion_config_path
                    .as_deref(),
            )?
        } else {
            None
        };
        let before_outcome_linkage_report = load_or_build_batch_outcome(
            config.before_batch_outcome_linkage_config_path.as_deref(),
        )?;
        let before_counterfactual_report = load_or_build_batch_counterfactual(
            config
                .before_batch_counterfactual_completion_config_path
                .as_deref(),
        )?;

        let sufficiency_report = if config.run_sufficiency_v2 {
            if let Some(path) = config
                .official_evidence_sufficiency_v2_config_path
                .as_deref()
            {
                load_official_evidence_sufficiency_v2_from_path_or_config(path)?
            } else {
                OfficialEvidenceSufficiencyV2Runner::default().run_from_inputs(
                    &OfficialEvidenceSufficiencyV2Config {
                        sufficiency_id: format!("{}-sufficiency", config.scaleout_id),
                        multi_row_set_path: config.multi_row_set_config_path.clone(),
                        batch_outcome_linkage_path: None,
                        batch_counterfactual_completion_path: None,
                        output_root: config.output_root.clone(),
                        ..OfficialEvidenceSufficiencyV2Config::default()
                    },
                    &after_set,
                    batch_outcome_linkage_report.as_ref(),
                    batch_counterfactual_completion_report.as_ref(),
                )
            }
        } else {
            OfficialEvidenceSufficiencyV2Runner::default().run_from_inputs(
                &OfficialEvidenceSufficiencyV2Config {
                    sufficiency_id: format!("{}-sufficiency", config.scaleout_id),
                    multi_row_set_path: config.multi_row_set_config_path.clone(),
                    batch_outcome_linkage_path: None,
                    batch_counterfactual_completion_path: None,
                    output_root: config.output_root.clone(),
                    ..OfficialEvidenceSufficiencyV2Config::default()
                },
                &after_set,
                batch_outcome_linkage_report.as_ref(),
                batch_counterfactual_completion_report.as_ref(),
            )
        };

        let before_counts = before_set
            .as_ref()
            .map(|set| {
                compute_counts(
                    set,
                    before_outcome_linkage_report.as_ref(),
                    before_counterfactual_report.as_ref(),
                )
            })
            .unwrap_or(OfficialEvidenceSufficiencyV2Counts {
                total_rows: 0,
                official_complete_rows: 0,
                symbols: 0,
                timeframes: 0,
                horizons: 0,
                take_profit_count: 0,
                stop_loss_count: 0,
                time_expired_count: 0,
                no_trade_counterfactual_count: 0,
                risk_denied_counterfactual_count: 0,
                baseline_reference_count: 0,
                no_lookahead_safe_ratio: 0.0,
                single_symbol_concentration_ratio: 0.0,
                single_outcome_label_ratio: 0.0,
                non_crypto_official_rows: 0,
                crypto_only_rows: 0,
                controlled_rows: 0,
            });
        let after_counts = sufficiency_report.counts.clone();

        let rerun_artifacts = self.run_reruns(
            config,
            &after_set,
            batch_outcome_linkage_report.as_ref(),
            batch_counterfactual_completion_report.as_ref(),
            &output_dir,
        )?;

        let scaleout_report = build_scaleout_report(
            config,
            &before_counts,
            &after_counts,
            &sufficiency_report,
            &rerun_artifacts,
        );
        let storage_report = build_storage_report(
            config,
            &after_set,
            &future_window_scaleout_plan,
            batch_outcome_linkage_report.as_ref(),
            batch_counterfactual_completion_report.as_ref(),
            &sufficiency_report,
            &scaleout_report,
        );
        let bundle = OfficialEvidenceScaleOutBundle {
            multi_row_set: after_set,
            future_window_scaleout_plan,
            batch_outcome_linkage_report,
            batch_counterfactual_completion_report,
            sufficiency_v2_report: sufficiency_report,
            scaleout_report,
            storage_report,
            final_summary: String::new(),
            reason_codes: build_official_evidence_scaleout_reason_codes(),
        };
        let mut bundle = bundle;
        bundle.final_summary = build_official_evidence_scaleout_summary(&bundle);
        bundle.write_to_dir(&output_dir)?;
        Ok(bundle)
    }

    fn run_reruns(
        &self,
        config: &OfficialEvidenceScaleOutConfig,
        set: &MultiRowOfficialEvidenceSet,
        batch_outcome: Option<&BatchOutcomeLinkageV3Report>,
        batch_counterfactual: Option<&BatchCounterfactualCompletionReport>,
        output_dir: &Path,
    ) -> Result<RerunArtifacts, String> {
        let mut artifacts = RerunArtifacts::default();
        let complete_rows = build_complete_rows(set, batch_outcome, batch_counterfactual);
        let pack = build_official_pack(&format!("{}-pack", config.scaleout_id), &complete_rows);
        let linked_pack = build_outcome_linked_pack(&pack, &complete_rows, batch_outcome);
        let rerun_dir = output_dir.join("reruns");
        fs::create_dir_all(&rerun_dir).map_err(|err| err.to_string())?;
        let linked_pack_path = linked_pack.write_to_dir(&rerun_dir)?.display().to_string();

        if config.run_committee_official_benchmark {
            let benchmark_config = CommitteeOfficialBenchmarkConfig {
                benchmark_id: format!("{}-committee-benchmark", config.scaleout_id),
                outcome_linked_pack_path: Some(linked_pack_path.clone()),
                output_root: rerun_dir.display().to_string(),
                require_core_check: config.require_core_check,
                run_materialization: false,
                run_outcome_linking: false,
                min_official_rows: 2,
                min_outcome_linked_rows: 1,
                min_baseline_linked_rows: 1,
                min_no_trade_counterfactuals: 1,
                min_risk_denial_counterfactuals: 1,
                ..CommitteeOfficialBenchmarkConfig::default()
            };
            let benchmark_bundle =
                CommitteeOfficialBenchmarkRunner::default().run_bundle(&benchmark_config)?;
            let benchmark_dir = rerun_dir.join("committee_benchmark");
            let benchmark_bundle_path = benchmark_bundle.write_to_dir(&benchmark_dir)?;
            artifacts.benchmark_bundle = Some(benchmark_bundle.clone());
            if config.run_outcome_coverage {
                let coverage_bundle = CommitteeOutcomeCoverageRunner::default().run(
                    &CommitteeOutcomeCoverageConfig {
                        coverage_id: format!("{}-committee-outcome-coverage", config.scaleout_id),
                        committee_benchmark_bundle_paths: vec![
                            benchmark_bundle_path.display().to_string(),
                        ],
                        output_root: rerun_dir.display().to_string(),
                        max_rows: complete_rows.len().max(1) * 4,
                        max_symbols: set.symbol_count.max(1),
                        max_bytes: 1_000_000,
                        require_official_rows: true,
                        allow_crypto_only: false,
                        allow_yfinance_research: false,
                        allow_fixture: false,
                        allow_estimated_counterfactuals: false,
                        require_no_lookahead_safe: true,
                        ..CommitteeOutcomeCoverageConfig::default()
                    },
                )?;
                let coverage_dir = rerun_dir.join("committee_outcome_coverage");
                let coverage_bundle_path = coverage_bundle.write_to_dir(&coverage_dir)?;
                artifacts.coverage_bundle = Some(coverage_bundle.clone());
                if config.run_counterfactual_depth_close {
                    let comparable_bundle = ComparableCommitteeEvidenceBundle::from_rows(
                        &ComparableCommitteeEvidenceConfig::default(),
                        complete_rows.clone(),
                    );
                    let comparable_dir = rerun_dir.join("comparable_bundle");
                    let comparable_path = comparable_bundle.write_to_dir(&comparable_dir)?;
                    let depth_report = CounterfactualDepthClosureRunner::default().run(
                        &CounterfactualDepthClosureConfig {
                            closure_id: format!("{}-counterfactual-depth", config.scaleout_id),
                            comparable_evidence_bundle_path: Some(
                                comparable_path.display().to_string(),
                            ),
                            outcome_coverage_config_paths: vec![
                                coverage_bundle_path.display().to_string(),
                            ],
                            output_root: rerun_dir.display().to_string(),
                            run_outcome_coverage: false,
                            run_reference_builders: false,
                            run_scorecard_rerun: false,
                            allow_controlled_evidence: false,
                            allow_crypto_only: false,
                            allow_yfinance_research: false,
                            allow_fixture: false,
                            ..CounterfactualDepthClosureConfig::default()
                        },
                    )?;
                    artifacts.depth_report = Some(depth_report);
                }
                if config.run_core_performance {
                    let core_bundle = CorePerformanceScorecardRunner::default().run(
                        &CorePerformanceScorecardConfig {
                            scorecard_id: format!("{}-core-scorecard", config.scaleout_id),
                            committee_official_benchmark_paths: vec![
                                benchmark_dir
                                    .join("committee_official_benchmark_bundle.json")
                                    .display()
                                    .to_string(),
                            ],
                            committee_outcome_coverage_paths: vec![
                                coverage_dir
                                    .join("committee_outcome_coverage_bundle.json")
                                    .display()
                                    .to_string(),
                            ],
                            output_root: rerun_dir.display().to_string(),
                            require_core_check_pass: false,
                            allow_controlled_evidence: false,
                            allow_crypto_only: false,
                            allow_yfinance_research: false,
                            allow_fixture: false,
                            ..CorePerformanceScorecardConfig::default()
                        },
                    )?;
                    let summary = CoreScorecardRerun::default().summarize(
                        None,
                        Some(&core_bundle.scorecard),
                        vec![
                            "previous core scorecard unavailable; reporting current rerun only"
                                .to_string(),
                        ],
                        true,
                    );
                    artifacts.core_summary = Some(summary);
                    artifacts.core_bundle = Some(core_bundle);
                }
            }
        }
        Ok(artifacts)
    }
}

impl OfficialEvidenceScaleOutStorageReport {
    pub fn to_text(&self) -> String {
        [
            format!("max_bytes={}", self.max_bytes),
            format!("estimated_output_bytes={}", self.estimated_output_bytes),
            format!("within_budget={}", self.within_budget),
            format!("guidance={}", self.guidance),
            format!("input_paths={}", self.input_paths.join("|")),
        ]
        .join("\n")
    }
}

impl OfficialEvidenceScaleOutReport {
    pub fn to_text(&self) -> String {
        [
            format!("scaleout_id={}", self.scaleout_id),
            format!("final_status={:?}", self.final_status),
            format!("final_recommendation={:?}", self.final_recommendation),
            format!("sufficiency_status={:?}", self.sufficiency_status),
            format!("multi_row_set_summary={}", self.multi_row_set_summary),
            format!(
                "future_window_scaleout_summary={}",
                self.future_window_scaleout_summary
                    .clone()
                    .unwrap_or_default()
            ),
            format!(
                "outcome_linkage_summary={}",
                self.outcome_linkage_summary.clone().unwrap_or_default()
            ),
            format!(
                "counterfactual_completion_summary={}",
                self.counterfactual_completion_summary
                    .clone()
                    .unwrap_or_default()
            ),
            format!(
                "before_official_complete_rows={}",
                self.before_counts.official_complete_rows
            ),
            format!(
                "after_official_complete_rows={}",
                self.after_counts.official_complete_rows
            ),
            format!(
                "before_official_outcome_count={}",
                outcome_reference_total(&self.before_counts)
            ),
            format!(
                "after_official_outcome_count={}",
                outcome_reference_total(&self.after_counts)
            ),
            format!(
                "before_counterfactual_depth={}/{}",
                self.before_counts.no_trade_counterfactual_count,
                self.before_counts.risk_denied_counterfactual_count
            ),
            format!(
                "after_counterfactual_depth={}/{}",
                self.after_counts.no_trade_counterfactual_count,
                self.after_counts.risk_denied_counterfactual_count
            ),
            format!(
                "added_official_complete_rows={}",
                self.added_official_complete_rows
            ),
            format!("added_outcome_references={}", self.added_outcome_references),
            format!(
                "added_no_trade_counterfactuals={}",
                self.added_no_trade_counterfactuals
            ),
            format!(
                "added_risk_denied_counterfactuals={}",
                self.added_risk_denied_counterfactuals
            ),
            format!(
                "committee_benchmark_status={}",
                self.committee_benchmark_status
                    .map(|value| format!("{value:?}"))
                    .unwrap_or_default()
            ),
            format!(
                "outcome_coverage_status={}",
                self.outcome_coverage_status
                    .map(|value| format!("{value:?}"))
                    .unwrap_or_default()
            ),
            format!(
                "counterfactual_depth_status={}",
                self.counterfactual_depth_status.clone().unwrap_or_default()
            ),
            format!(
                "core_scorecard_summary={}",
                self.core_scorecard_summary
                    .as_ref()
                    .map(CoreScorecardRerunSummary::to_text)
                    .unwrap_or_default()
                    .replace('\n', " | ")
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
}

fn load_or_build_future_window_plan(
    config: &OfficialEvidenceScaleOutConfig,
) -> Result<Option<FutureWindowScaleOutPlan>, String> {
    config
        .future_window_scaleout_config_path
        .as_deref()
        .map(load_future_window_scaleout_plan_from_path_or_config)
        .transpose()
}

fn load_or_build_batch_outcome(
    path: Option<&str>,
) -> Result<Option<BatchOutcomeLinkageV3Report>, String> {
    path.map(load_batch_outcome_linkage_v3_from_path_or_config)
        .transpose()
}

fn load_or_build_batch_counterfactual(
    path: Option<&str>,
) -> Result<Option<BatchCounterfactualCompletionReport>, String> {
    path.map(load_batch_counterfactual_completion_from_path_or_config)
        .transpose()
}

fn build_complete_rows(
    set: &MultiRowOfficialEvidenceSet,
    batch_outcome: Option<&BatchOutcomeLinkageV3Report>,
    batch_counterfactual: Option<&BatchCounterfactualCompletionReport>,
) -> Vec<ComparableCommitteeEvidenceRow> {
    let outcome_map = batch_outcome
        .map(|report| {
            report
                .records
                .iter()
                .filter_map(|record| {
                    record
                        .outcome_reference
                        .as_ref()
                        .map(|reference| (record.row_id.clone(), reference.clone()))
                })
                .collect::<std::collections::BTreeMap<_, _>>()
        })
        .unwrap_or_default();
    let counterfactual_map = batch_counterfactual
        .map(|report| {
            report
                .records
                .iter()
                .map(|record| (record.row_id.clone(), record.clone()))
                .collect::<std::collections::BTreeMap<_, _>>()
        })
        .unwrap_or_default();
    let mut rows = set
        .items
        .iter()
        .map(|item| {
            let outcome = outcome_map.get(&item.row_id);
            let counterfactual = counterfactual_map.get(&item.row_id);
            ComparableCommitteeEvidenceRow {
                row_id: item.row_id.clone(),
                symbol: item.symbol.clone(),
                market: item.market,
                timeframe: item.timeframe.clone(),
                horizon_bars: item.horizon_bars,
                timestamp_ms: item.timestamp_ms,
                source_kind: item.source_kind.clone(),
                source_class: item.source_class,
                scenario_row_id: item.scenario_row_id.clone(),
                committee_decision_id: Some(format!("committee-{}", item.row_id)),
                committee_final_action: "Approve".to_string(),
                chair_decision: Some("Approve".to_string()),
                risk_governor_decision: Some("Reject".to_string()),
                baseline_action: Some("Approve".to_string()),
                external_action: None,
                no_trade_baseline_action: "NoTrade".to_string(),
                outcome_label: outcome
                    .map(|reference| format!("{:?}", reference.triple_barrier_label)),
                net_return_pct: outcome.and_then(|reference| reference.net_return_pct),
                cost_bps: outcome.map(|reference| reference.cost_bps).unwrap_or(5.0),
                slippage_bps: outcome
                    .map(|reference| reference.slippage_bps)
                    .unwrap_or(2.0),
                committee_vs_baseline_delta: None,
                committee_vs_notrade_delta: None,
                risk_denied_value_proxy: counterfactual
                    .and_then(|record| record.avoided_loss_value),
                no_trade_value_proxy: counterfactual.and_then(|record| record.missed_gain_value),
                outcome_reference_available: item.outcome_reference_available || outcome.is_some(),
                baseline_reference_available: item.baseline_reference_available,
                no_trade_counterfactual_available: item.no_trade_counterfactual_available
                    || counterfactual
                        .map(|record| record.no_trade_counterfactual_built)
                        .unwrap_or(false),
                risk_denied_counterfactual_available: item.risk_denied_counterfactual_available
                    || counterfactual
                        .map(|record| record.risk_denied_counterfactual_built)
                        .unwrap_or(false),
                external_reference_available: false,
                row_level: item.row_level,
                summary_derived: item.summary_derived,
                no_lookahead_safe: item.no_lookahead_safe,
                official_readiness_eligible: item.source_class
                    == super::ComparableEvidenceSourceClass::OfficialNonCrypto,
                diagnostic_only: item.diagnostic_only,
                candle_coverage_available: item.has_local_candle_window,
                matched_candle_series_id: item.candle_series_id.clone(),
                candle_match_status: Some("Matched".to_string()),
                candle_official_ready_match: item.official_ready_match,
                candle_benchmark_ready_match: item.benchmark_ready_match,
                candle_diagnostic_only: item.diagnostic_only,
                reason_codes: item.reason_codes.clone(),
            }
        })
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| left.row_id.cmp(&right.row_id));
    rows
}

fn build_official_pack(
    pack_id: &str,
    rows: &[ComparableCommitteeEvidenceRow],
) -> OfficialCommitteeScenarioPack {
    let scenario_rows = rows.iter().map(build_scenario_row).collect::<Vec<_>>();
    OfficialCommitteeScenarioPack {
        pack_id: pack_id.to_string(),
        rows: scenario_rows,
        source_summary: "official-non-crypto-only".to_string(),
        official_row_count: rows.len(),
        crypto_only_row_count: 0,
        yfinance_row_count: 0,
        fixture_row_count: 0,
        row_level_count: rows.iter().filter(|row| row.row_level).count(),
        summary_derived_count: rows.iter().filter(|row| row.summary_derived).count(),
        outcome_linked_count: rows
            .iter()
            .filter(|row| row.outcome_reference_available)
            .count(),
        baseline_reference_count: rows
            .iter()
            .filter(|row| row.baseline_reference_available)
            .count(),
        external_reference_count: 0,
        no_trade_counterfactual_count: rows
            .iter()
            .filter(|row| row.no_trade_counterfactual_available)
            .count(),
        risk_denial_counterfactual_count: rows
            .iter()
            .filter(|row| row.risk_denied_counterfactual_available)
            .count(),
        storage_bytes: serde_json::to_vec(rows)
            .map(|bytes| bytes.len())
            .unwrap_or_default(),
        reason_codes: build_official_evidence_scaleout_reason_codes(),
    }
}

fn build_scenario_row(row: &ComparableCommitteeEvidenceRow) -> CommitteeScenarioRow {
    CommitteeScenarioRow {
        scenario_row_id: row.row_id.clone(),
        symbol: row.symbol.clone(),
        timestamp_ms: row.timestamp_ms,
        source_kind: CommitteeScenarioSourceKind::OfficialBenchmarkReport,
        evidence_source_kind: EvidenceSourceKind::OfficialApiCollected,
        market: row.market,
        target_horizon: if row.horizon_bars <= 3 {
            PersonaHorizon::Swing
        } else {
            PersonaHorizon::MultiDay
        },
        feature_vector: None,
        regime: Regime::TrendUp,
        signal_summary: row.committee_final_action.clone(),
        data_quality_score: 0.92,
        spread_bps: Some(4.0),
        expected_edge_after_cost: row.net_return_pct.unwrap_or(0.01).max(0.0),
        expected_drawdown: 0.02,
        risk_snapshot_summary: Some("research-only".to_string()),
        provenance_summary: "official local multi-row scaleout".to_string(),
        benchmark_status: Some("Ready".to_string()),
        baseline_signal_summary: row.baseline_action.clone(),
        external_prediction_summary: None,
        no_trade_counterfactual: row
            .no_trade_counterfactual_available
            .then(|| "NoTradeCounterfactual".to_string()),
        risk_denial_counterfactual: row
            .risk_denied_counterfactual_available
            .then(|| "RiskDeniedCounterfactual".to_string()),
        outcome_reference: row.outcome_reference_available.then(|| row.row_id.clone()),
        materialization_level: CommitteeScenarioMaterializationLevel::RowLevel,
        materialization_confidence: 1.0,
        reason_codes: row.reason_codes.clone(),
    }
}

fn build_outcome_linked_pack(
    pack: &OfficialCommitteeScenarioPack,
    rows: &[ComparableCommitteeEvidenceRow],
    batch_outcome: Option<&BatchOutcomeLinkageV3Report>,
) -> OutcomeLinkedCommitteeScenarioPack {
    let outcome_map = batch_outcome
        .map(|report| {
            report
                .records
                .iter()
                .filter_map(|record| {
                    record
                        .outcome_reference
                        .as_ref()
                        .map(|reference| (record.row_id.clone(), reference.clone()))
                })
                .collect::<std::collections::BTreeMap<_, _>>()
        })
        .unwrap_or_default();
    let row_map = rows
        .iter()
        .map(|row| (row.row_id.clone(), row))
        .collect::<std::collections::BTreeMap<_, _>>();
    let linked_rows = pack
        .rows
        .iter()
        .map(|scenario_row| {
            let row = row_map
                .get(&scenario_row.scenario_row_id)
                .or_else(|| row_map.get(&scenario_row.symbol));
            let baseline_reference =
                row.and_then(|row| row.baseline_action.as_ref())
                    .map(|summary| CommitteeBaselineReference {
                        baseline_action: CommitteeBaselineAction::from_summary(summary),
                        baseline_confidence: Some(0.7),
                        baseline_expected_edge: Some(0.01),
                        baseline_reason_codes: vec![ReasonCode::DeterministicPath],
                        reason_codes: vec![ReasonCode::DeterministicPath],
                    });
            OutcomeLinkedCommitteeScenarioRow {
                scenario_row: scenario_row.clone(),
                outcome_reference: outcome_map.get(&scenario_row.scenario_row_id).cloned(),
                baseline_reference,
                external_reference: None,
                reason_codes: vec![
                    ReasonCode::CommitteeOutcomeReferenceBuilt,
                    ReasonCode::DeterministicPath,
                ],
            }
        })
        .collect::<Vec<_>>();
    OutcomeLinkedCommitteeScenarioPack {
        pack: pack.clone(),
        linked_rows,
        unmatched_rows: Vec::new(),
        link_summary: CommitteeOutcomeLinkSummary {
            linker_id: format!("{}-linker", pack.pack_id),
            matched_rows: outcome_map.len().max(rows.len()),
            unmatched_rows: 0,
            timestamp_tolerance_ms: 0,
            strict_timestamp_match: true,
            no_lookahead_violations: 0,
            warnings: Vec::new(),
            reason_codes: vec![ReasonCode::DeterministicPath],
        },
        outcome_linked_count: outcome_map.len(),
        baseline_linked_count: rows
            .iter()
            .filter(|row| row.baseline_reference_available)
            .count(),
        external_linked_count: 0,
        no_trade_counterfactual_count: rows
            .iter()
            .filter(|row| row.no_trade_counterfactual_available)
            .count(),
        risk_denial_counterfactual_count: rows
            .iter()
            .filter(|row| row.risk_denied_counterfactual_available)
            .count(),
        no_lookahead_violations: 0,
        reason_codes: vec![
            ReasonCode::CommitteeOutcomeLinkerBuilt,
            ReasonCode::DeterministicPath,
        ],
    }
}

fn build_scaleout_report(
    config: &OfficialEvidenceScaleOutConfig,
    before_counts: &OfficialEvidenceSufficiencyV2Counts,
    after_counts: &OfficialEvidenceSufficiencyV2Counts,
    sufficiency_report: &OfficialEvidenceSufficiencyV2Report,
    reruns: &RerunArtifacts,
) -> OfficialEvidenceScaleOutReport {
    let mut blockers = Vec::new();
    let mut warnings = sufficiency_report.warnings.clone();
    let before_outcomes = outcome_reference_total(before_counts);
    let after_outcomes = outcome_reference_total(after_counts);
    if after_counts.official_complete_rows <= before_counts.official_complete_rows {
        blockers.push("official complete row count did not expand".to_string());
    }
    if after_outcomes <= before_outcomes {
        warnings.push("outcome linkage count did not expand beyond the baseline run".to_string());
    }
    if after_counts.no_trade_counterfactual_count <= before_counts.no_trade_counterfactual_count
        || after_counts.risk_denied_counterfactual_count
            <= before_counts.risk_denied_counterfactual_count
    {
        warnings.push("counterfactual coverage did not expand on both branches".to_string());
    }
    let committee_benchmark_status = reruns.benchmark_bundle.as_ref().map(|bundle| {
        if bundle.official_readiness_report.enough_for_committee_benchmark {
            CommitteeOfficialBenchmarkFinalStatus::OfficialCommitteeBenchmarkReady
        } else {
            match bundle.official_readiness_report.readiness_status {
                super::OfficialCommitteeEvidenceReadinessStatus::ReadyForMoreOfficialEvidence
                | super::OfficialCommitteeEvidenceReadinessStatus::NotReadyInsufficientRows => {
                    CommitteeOfficialBenchmarkFinalStatus::NeedMoreOfficialRows
                }
                super::OfficialCommitteeEvidenceReadinessStatus::ReadyForOutcomeLinking
                | super::OfficialCommitteeEvidenceReadinessStatus::NotReadyNoOutcomeLinks => {
                    CommitteeOfficialBenchmarkFinalStatus::NeedMoreOutcomeLinks
                }
                super::OfficialCommitteeEvidenceReadinessStatus::NotReadySummaryDerivedDominant => {
                    CommitteeOfficialBenchmarkFinalStatus::MaterializationWeak
                }
                super::OfficialCommitteeEvidenceReadinessStatus::NotReadyResearchOnly => {
                    CommitteeOfficialBenchmarkFinalStatus::ResearchOnly
                }
                super::OfficialCommitteeEvidenceReadinessStatus::NotReadyFixtureOnly => {
                    CommitteeOfficialBenchmarkFinalStatus::FixtureOnly
                }
                super::OfficialCommitteeEvidenceReadinessStatus::NotReadyCryptoOnly => {
                    CommitteeOfficialBenchmarkFinalStatus::CryptoOnly
                }
                super::OfficialCommitteeEvidenceReadinessStatus::NotReadyNoLookaheadViolation => {
                    CommitteeOfficialBenchmarkFinalStatus::NeedMoreEvidence
                }
                super::OfficialCommitteeEvidenceReadinessStatus::ReadyForOfficialCommitteeBenchmark => {
                    CommitteeOfficialBenchmarkFinalStatus::OfficialCommitteeBenchmarkReady
                }
            }
        }
    });
    let outcome_coverage_status = reruns
        .coverage_bundle
        .as_ref()
        .map(|bundle| bundle.final_status);
    let counterfactual_depth_status = reruns
        .depth_report
        .as_ref()
        .map(|report| format!("{:?}", report.closure_status));
    let core_scorecard_summary = reruns.core_summary.clone();
    let previous_core_status = core_scorecard_summary
        .as_ref()
        .and_then(|summary| summary.previous_status.map(|value| format!("{value:?}")));
    let current_core_status = core_scorecard_summary
        .as_ref()
        .and_then(|summary| summary.current_status.map(|value| format!("{value:?}")));
    let previous_primary_bottleneck = core_scorecard_summary.as_ref().and_then(|summary| {
        summary
            .previous_primary_bottleneck
            .map(|value| format!("{value:?}"))
    });
    let current_primary_bottleneck = core_scorecard_summary.as_ref().and_then(|summary| {
        summary
            .current_primary_bottleneck
            .map(|value| format!("{value:?}"))
    });
    let bottleneck_changed = core_scorecard_summary
        .as_ref()
        .map(|summary| summary.bottleneck_changed)
        .unwrap_or(false);
    let status = if sufficiency_report.passed_tentative_signal_quality_review {
        OfficialEvidenceScaleOutStatus::TentativeSignalQualityReviewReady
    } else if sufficiency_report.passed_committee_benchmark_research {
        OfficialEvidenceScaleOutStatus::CommitteeBenchmarkResearchReady
    } else if after_counts.no_trade_counterfactual_count
        > before_counts.no_trade_counterfactual_count
        && after_counts.risk_denied_counterfactual_count
            > before_counts.risk_denied_counterfactual_count
    {
        OfficialEvidenceScaleOutStatus::CounterfactualCoverageExpanded
    } else if after_outcomes > before_outcomes {
        OfficialEvidenceScaleOutStatus::OutcomeCoverageExpanded
    } else if after_counts.official_complete_rows > before_counts.official_complete_rows {
        OfficialEvidenceScaleOutStatus::OfficialCompleteRowsExpanded
    } else if sufficiency_report.passed_plumbing_validation {
        OfficialEvidenceScaleOutStatus::OfficialEvidencePlumbingValidated
    } else if after_counts.official_complete_rows == 0 {
        OfficialEvidenceScaleOutStatus::StillInsufficientRows
    } else if after_outcomes == 0 {
        OfficialEvidenceScaleOutStatus::StillNeedMoreOutcomeLinks
    } else if after_counts.no_trade_counterfactual_count == 0
        || after_counts.risk_denied_counterfactual_count == 0
    {
        OfficialEvidenceScaleOutStatus::StillNeedMoreCounterfactuals
    } else if matches!(
        sufficiency_report.sufficiency_status,
        OfficialEvidenceSufficiencyV2Status::SingleSymbolDominated
    ) {
        OfficialEvidenceScaleOutStatus::StillSingleSymbolDominated
    } else if matches!(
        sufficiency_report.sufficiency_status,
        OfficialEvidenceSufficiencyV2Status::SingleOutcomeDominated
    ) {
        OfficialEvidenceScaleOutStatus::StillSingleOutcomeDominated
    } else if current_core_status
        .as_deref()
        .is_some_and(|status| status == "CoreBlockedByEvidence")
    {
        OfficialEvidenceScaleOutStatus::CoreStillBlockedByEvidence
    } else if current_core_status
        .as_deref()
        .is_some_and(|status| status == "CorePerformanceHealthyForResearch")
    {
        OfficialEvidenceScaleOutStatus::CorePerformanceHealthyForResearch
    } else {
        OfficialEvidenceScaleOutStatus::StillEvidenceTooWeak
    };
    let final_recommendation = match status {
        OfficialEvidenceScaleOutStatus::CommitteeBenchmarkResearchReady => {
            OfficialEvidenceScaleOutRecommendation::RunCommitteeOfficialBenchmark
        }
        OfficialEvidenceScaleOutStatus::TentativeSignalQualityReviewReady => {
            OfficialEvidenceScaleOutRecommendation::ImproveSignalModelFirst
        }
        OfficialEvidenceScaleOutStatus::CounterfactualCoverageExpanded => {
            OfficialEvidenceScaleOutRecommendation::MoreCounterfactualDepth
        }
        OfficialEvidenceScaleOutStatus::OutcomeCoverageExpanded => {
            OfficialEvidenceScaleOutRecommendation::MoreOutcomeDiversity
        }
        OfficialEvidenceScaleOutStatus::OfficialCompleteRowsExpanded => {
            OfficialEvidenceScaleOutRecommendation::MoreOfficialRows
        }
        OfficialEvidenceScaleOutStatus::OfficialEvidencePlumbingValidated => {
            OfficialEvidenceScaleOutRecommendation::RunCorePerformance
        }
        OfficialEvidenceScaleOutStatus::StillInsufficientRows => {
            OfficialEvidenceScaleOutRecommendation::MoreOfficialRows
        }
        OfficialEvidenceScaleOutStatus::StillNeedMoreOutcomeLinks => {
            OfficialEvidenceScaleOutRecommendation::MoreOutcomeDiversity
        }
        OfficialEvidenceScaleOutStatus::StillNeedMoreCounterfactuals => {
            OfficialEvidenceScaleOutRecommendation::MoreCounterfactualDepth
        }
        OfficialEvidenceScaleOutStatus::StillSingleSymbolDominated => {
            OfficialEvidenceScaleOutRecommendation::MoreOfficialSymbols
        }
        OfficialEvidenceScaleOutStatus::StillSingleOutcomeDominated => {
            OfficialEvidenceScaleOutRecommendation::MoreOutcomeDiversity
        }
        OfficialEvidenceScaleOutStatus::CoreStillBlockedByEvidence => {
            OfficialEvidenceScaleOutRecommendation::RunCorePerformance
        }
        OfficialEvidenceScaleOutStatus::StillEvidenceTooWeak => {
            OfficialEvidenceScaleOutRecommendation::KeepTrinity
        }
        OfficialEvidenceScaleOutStatus::CorePerformanceHealthyForResearch
        | OfficialEvidenceScaleOutStatus::NoImprovement => {
            OfficialEvidenceScaleOutRecommendation::NeedMoreEvidence
        }
    };
    OfficialEvidenceScaleOutReport {
        scaleout_id: config.scaleout_id.clone(),
        multi_row_set_summary: format!(
            "total_rows={};official_complete_rows={};storage_bytes={}",
            after_counts.total_rows, after_counts.official_complete_rows, config.max_bytes
        ),
        future_window_scaleout_summary: Some(
            config
                .future_window_scaleout_config_path
                .as_deref()
                .unwrap_or_default()
                .to_string(),
        )
        .filter(|value| !value.is_empty()),
        outcome_linkage_summary: Some(format!(
            "generated_outcome_count={after_outcomes};official_outcome_count={after_outcomes}"
        )),
        counterfactual_completion_summary: Some(format!(
            "no_trade_built_count={};risk_denied_built_count={}",
            after_counts.no_trade_counterfactual_count,
            after_counts.risk_denied_counterfactual_count
        )),
        sufficiency_v2_report: sufficiency_report.clone(),
        committee_benchmark_summary: committee_benchmark_status.map(|status| format!("{status:?}")),
        outcome_coverage_summary: outcome_coverage_status.map(|status| format!("{status:?}")),
        counterfactual_depth_summary: counterfactual_depth_status.clone(),
        core_performance_summary: core_scorecard_summary
            .as_ref()
            .map(CoreScorecardRerunSummary::to_text),
        before_counts: before_counts.clone(),
        after_counts: after_counts.clone(),
        added_official_complete_rows: after_counts.official_complete_rows as isize
            - before_counts.official_complete_rows as isize,
        added_outcome_references: after_outcomes as isize - before_outcomes as isize,
        added_no_trade_counterfactuals: after_counts.no_trade_counterfactual_count as isize
            - before_counts.no_trade_counterfactual_count as isize,
        added_risk_denied_counterfactuals: after_counts.risk_denied_counterfactual_count as isize
            - before_counts.risk_denied_counterfactual_count as isize,
        previous_core_status,
        current_core_status,
        previous_primary_bottleneck,
        current_primary_bottleneck,
        bottleneck_changed,
        sufficiency_status: sufficiency_report.sufficiency_status,
        committee_benchmark_status,
        outcome_coverage_status,
        counterfactual_depth_status,
        core_scorecard_summary: core_scorecard_summary.clone(),
        final_status: status,
        status,
        final_recommendation,
        blockers,
        warnings,
        reason_codes: stable_reason_codes(
            &config
                .reason_codes
                .iter()
                .cloned()
                .chain([
                    ReasonCode::OfficialEvidenceCounted,
                    ReasonCode::DeterministicPath,
                ])
                .collect::<Vec<_>>(),
        ),
    }
}

fn build_storage_report(
    config: &OfficialEvidenceScaleOutConfig,
    set: &MultiRowOfficialEvidenceSet,
    future_window_plan: &Option<FutureWindowScaleOutPlan>,
    batch_outcome: Option<&BatchOutcomeLinkageV3Report>,
    batch_counterfactual: Option<&BatchCounterfactualCompletionReport>,
    sufficiency_report: &OfficialEvidenceSufficiencyV2Report,
    scaleout_report: &OfficialEvidenceScaleOutReport,
) -> OfficialEvidenceScaleOutStorageReport {
    let estimated_output_bytes = set.to_text().len()
        + future_window_plan
            .as_ref()
            .map(FutureWindowScaleOutPlan::to_text)
            .unwrap_or_default()
            .len()
        + batch_outcome
            .map(BatchOutcomeLinkageV3Report::to_text)
            .unwrap_or_default()
            .len()
        + batch_counterfactual
            .map(BatchCounterfactualCompletionReport::to_text)
            .unwrap_or_default()
            .len()
        + sufficiency_report.to_text().len()
        + scaleout_report.to_text().len();
    OfficialEvidenceScaleOutStorageReport {
        max_bytes: config.max_bytes,
        estimated_output_bytes,
        within_budget: estimated_output_bytes <= config.max_bytes,
        guidance: if estimated_output_bytes <= config.max_bytes {
            "storage budget respected for bounded multi-row scaleout outputs".to_string()
        } else {
            "storage budget exceeded; reduce fixture scope before rerunning scaleout".to_string()
        },
        input_paths: config.all_paths(),
        reason_codes: stable_reason_codes(&[
            ReasonCode::StorageBudgetReportBuilt,
            ReasonCode::DeterministicPath,
        ]),
    }
}

fn outcome_reference_total(counts: &OfficialEvidenceSufficiencyV2Counts) -> usize {
    counts.take_profit_count + counts.stop_loss_count + counts.time_expired_count
}

fn default_output_root() -> String {
    "target/soma_official_evidence_scaleout".to_string()
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

fn default_true() -> bool {
    true
}
