use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};

use super::comparable_committee_evidence::{
    ComparableCommitteeEvidenceBundle, ComparableCommitteeEvidenceConfig,
};
use super::comparable_evidence_builder::ComparableEvidenceBuilder;
use super::comparable_evidence_quality::{
    ComparableEvidenceQualityReport, ComparableEvidenceQualityStatus,
    build_comparable_evidence_quality_report,
};
use super::counterfactual_depth_closure_bundle::CounterfactualDepthClosureBundle;
use super::counterfactual_depth_plan::CounterfactualDepthPlan;
use super::scenario_materialization_closure::{
    ScenarioMaterializationWeakClosureReport, ScenarioMaterializationWeakClosureStatus,
    build_scenario_materialization_weak_closure_report,
};
use crate::experiment::{CoreScorecardRerun, CoreScorecardRerunSummary};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CounterfactualDepthClosureConfig {
    pub closure_id: String,
    #[serde(default)]
    pub comparable_evidence_config_path: Option<String>,
    #[serde(default)]
    pub comparable_evidence_bundle_path: Option<String>,
    #[serde(default)]
    pub reference_pack_config_paths: Vec<String>,
    #[serde(default)]
    pub outcome_coverage_config_paths: Vec<String>,
    #[serde(default)]
    pub official_replication_config_paths: Vec<String>,
    #[serde(default)]
    pub core_performance_config_path: Option<String>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default)]
    pub run_reference_builders: bool,
    #[serde(default)]
    pub run_outcome_coverage: bool,
    #[serde(default)]
    pub run_scorecard_rerun: bool,
    #[serde(default = "default_true")]
    pub allow_controlled_evidence: bool,
    #[serde(default = "default_true")]
    pub allow_crypto_only: bool,
    #[serde(default = "default_true")]
    pub allow_yfinance_research: bool,
    #[serde(default = "default_true")]
    pub allow_fixture: bool,
    #[serde(default = "default_max_build_attempts")]
    pub max_build_attempts: usize,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ComparableEvidenceCountSummary {
    pub total_rows: usize,
    pub complete_rows: usize,
    pub official_comparable_rows: usize,
    pub row_level_rows: usize,
    pub summary_derived_rows: usize,
    pub outcome_references: usize,
    pub baseline_references: usize,
    pub no_trade_counterfactuals: usize,
    pub risk_denied_counterfactuals: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CounterfactualDepthClosureStatus {
    CounterfactualDepthImproved,
    ScenarioMaterializationImproved,
    OfficialComparableRowsImproved,
    ImprovedButStillEvidenceBlocked,
    #[default]
    NoImprovement,
    MissingLocalCandles,
    MissingOutcomeReferences,
    MissingBaselineReferences,
    MissingCounterfactuals,
    ControlledOnlyImprovement,
    CryptoOnlyImprovement,
    ResearchOnlyImprovement,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CounterfactualDepthFinalRecommendation {
    #[default]
    ImproveCounterfactualDepthFirst,
    ImproveScenarioMaterializationFirst,
    MoreOfficialEvidence,
    ImproveCandleCoverageFirst,
    ImproveBaselineReferenceDepth,
    RerunCorePerformance,
    CommitteeCoreReadyForDeeperEvidence,
    KeepTrinity,
    NeedMoreEvidence,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CounterfactualDepthClosureReport {
    pub closure_id: String,
    pub comparable_bundle_summary: ComparableEvidenceCountSummary,
    pub depth_plan: CounterfactualDepthPlan,
    pub build_attempts: Vec<String>,
    #[serde(default)]
    pub previous_counts: Option<ComparableEvidenceCountSummary>,
    pub current_counts: ComparableEvidenceCountSummary,
    pub added_complete_rows: isize,
    pub added_outcome_references: isize,
    pub added_baseline_references: isize,
    pub added_no_trade_counterfactuals: isize,
    pub added_risk_denied_counterfactuals: isize,
    pub added_official_comparable_rows: isize,
    #[serde(default)]
    pub bottleneck_before: Option<String>,
    #[serde(default)]
    pub bottleneck_after: Option<String>,
    pub improvement_detected: bool,
    pub closure_status: CounterfactualDepthClosureStatus,
    pub final_recommendation: CounterfactualDepthFinalRecommendation,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CounterfactualDepthClosureRunner;

impl Default for CounterfactualDepthClosureConfig {
    fn default() -> Self {
        Self {
            closure_id: "counterfactual-depth-closure".to_string(),
            comparable_evidence_config_path: None,
            comparable_evidence_bundle_path: None,
            reference_pack_config_paths: Vec::new(),
            outcome_coverage_config_paths: Vec::new(),
            official_replication_config_paths: Vec::new(),
            core_performance_config_path: None,
            output_root: default_output_root(),
            run_reference_builders: false,
            run_outcome_coverage: false,
            run_scorecard_rerun: false,
            allow_controlled_evidence: true,
            allow_crypto_only: true,
            allow_yfinance_research: true,
            allow_fixture: true,
            max_build_attempts: default_max_build_attempts(),
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

impl CounterfactualDepthClosureConfig {
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

    pub fn validate(&self) -> Result<(), String> {
        if self.closure_id.trim().is_empty() {
            return Err("counterfactual depth closure id must not be empty".to_string());
        }
        if self.max_build_attempts == 0 || self.max_build_attempts > 16 {
            return Err(
                "counterfactual depth max_build_attempts must be between 1 and 16".to_string(),
            );
        }
        if self
            .all_paths()
            .iter()
            .chain(std::iter::once(&self.output_root))
            .any(|path| path.contains("://"))
        {
            return Err("counterfactual depth paths must be local".to_string());
        }
        if self.comparable_evidence_config_path.is_none()
            && self.comparable_evidence_bundle_path.is_none()
        {
            return Err(
                "counterfactual depth closure needs a comparable config or bundle path".to_string(),
            );
        }
        Ok(())
    }

    pub fn all_paths(&self) -> Vec<String> {
        self.comparable_evidence_config_path
            .iter()
            .cloned()
            .chain(self.comparable_evidence_bundle_path.iter().cloned())
            .chain(self.reference_pack_config_paths.iter().cloned())
            .chain(self.outcome_coverage_config_paths.iter().cloned())
            .chain(self.official_replication_config_paths.iter().cloned())
            .chain(self.core_performance_config_path.iter().cloned())
            .collect()
    }

    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.closure_id)
    }

    pub fn to_comparable_config(&self) -> ComparableCommitteeEvidenceConfig {
        ComparableCommitteeEvidenceConfig {
            comparable_id: self.closure_id.clone(),
            output_root: self.output_root.clone(),
            allow_controlled_evidence: self.allow_controlled_evidence,
            allow_crypto_only: self.allow_crypto_only,
            allow_yfinance_research: self.allow_yfinance_research,
            allow_fixture: self.allow_fixture,
            ..ComparableCommitteeEvidenceConfig::default()
        }
    }
}

impl ComparableEvidenceCountSummary {
    pub fn from_bundle(
        config: &ComparableCommitteeEvidenceConfig,
        bundle: &ComparableCommitteeEvidenceBundle,
    ) -> Self {
        Self {
            total_rows: bundle.rows.len(),
            complete_rows: bundle.complete_rows,
            official_comparable_rows: bundle
                .rows
                .iter()
                .filter(|row| row.official_complete(config))
                .count(),
            row_level_rows: bundle.row_level_count,
            summary_derived_rows: bundle.summary_derived_count,
            outcome_references: bundle.outcome_reference_count,
            baseline_references: bundle.baseline_reference_count,
            no_trade_counterfactuals: bundle.no_trade_counterfactual_count,
            risk_denied_counterfactuals: bundle.risk_denied_counterfactual_count,
        }
    }
}

impl CounterfactualDepthClosureRunner {
    pub fn run(
        &self,
        config: &CounterfactualDepthClosureConfig,
    ) -> Result<CounterfactualDepthClosureReport, String> {
        self.run_bundle(config).map(|bundle| bundle.closure_report)
    }

    pub fn run_bundle(
        &self,
        config: &CounterfactualDepthClosureConfig,
    ) -> Result<CounterfactualDepthClosureBundle, String> {
        config.validate()?;
        let builder = ComparableEvidenceBuilder::default();
        let comparable_config = if let Some(path) = &config.comparable_evidence_config_path {
            ComparableCommitteeEvidenceConfig::from_toml_path(Path::new(path))?
        } else {
            config.to_comparable_config()
        };
        let before_bundle = if let Some(path) = &config.comparable_evidence_bundle_path {
            ComparableCommitteeEvidenceBundle::from_json_path(Path::new(path))?
        } else {
            builder.build(&comparable_config)?
        };
        let _before_quality =
            build_comparable_evidence_quality_report(&comparable_config, &before_bundle);
        let _before_plan = CounterfactualDepthPlan::from_bundle(&comparable_config, &before_bundle);

        let rerun = CoreScorecardRerun::default();
        let scorecard_before = if config.run_scorecard_rerun {
            if let Some(path) = &config.core_performance_config_path {
                Some(rerun.run_bundle(path)?)
            } else {
                None
            }
        } else {
            None
        };

        let mut build_attempts = Vec::new();
        let mut extra_config = config.to_comparable_config();
        let mut attempts = 0usize;

        if config.run_reference_builders {
            for path in &config.official_replication_config_paths {
                if attempts >= config.max_build_attempts {
                    break;
                }
                let runner_config = super::official_evidence_replication::OfficialEvidenceReplicationConfig::from_toml_path(Path::new(path))?;
                let _bundle = super::official_evidence_replication::OfficialEvidenceReplicationRunner::default()
                    .run_bundle(&runner_config)?;
                let output = runner_config
                    .output_dir()
                    .join("official_replication_bundle.json");
                extra_config
                    .official_replication_report_paths
                    .push(output.display().to_string());
                build_attempts.push(format!("official_replication={}", output.display()));
                attempts += 1;
            }
            for path in &config.reference_pack_config_paths {
                if attempts >= config.max_build_attempts {
                    break;
                }
                let runner_config =
                    super::committee_reference_pack::CommitteeReferencePackConfig::from_toml_path(
                        Path::new(path),
                    )?;
                let _bundle =
                    super::committee_reference_pack_runner::CommitteeReferencePackRunner::default()
                        .run(&runner_config)?;
                let output = runner_config
                    .output_dir()
                    .join("committee_reference_pack_bundle.json");
                extra_config
                    .reference_pack_bundle_paths
                    .push(output.display().to_string());
                build_attempts.push(format!("reference_pack={}", output.display()));
                attempts += 1;
            }
        }

        if config.run_outcome_coverage {
            for path in &config.outcome_coverage_config_paths {
                if attempts >= config.max_build_attempts {
                    break;
                }
                let runner_config = super::committee_outcome_coverage::CommitteeOutcomeCoverageConfig::from_toml_path(Path::new(path))?;
                let _bundle = super::committee_outcome_coverage_runner::CommitteeOutcomeCoverageRunner::default()
                    .run(&runner_config)?;
                let output = runner_config
                    .output_dir()
                    .join("committee_outcome_coverage_bundle.json");
                extra_config
                    .outcome_coverage_bundle_paths
                    .push(output.display().to_string());
                build_attempts.push(format!("outcome_coverage={}", output.display()));
                attempts += 1;
            }
        }

        let current_bundle = if build_attempts.is_empty() {
            before_bundle.clone()
        } else {
            let extra_rows = builder.load_rows(&extra_config)?;
            builder.merge_bundle(&comparable_config, &before_bundle, extra_rows)
        };
        let current_quality =
            build_comparable_evidence_quality_report(&comparable_config, &current_bundle);
        let current_plan =
            CounterfactualDepthPlan::from_bundle(&comparable_config, &current_bundle);
        let materialization_report = build_scenario_materialization_weak_closure_report(
            Some(&before_bundle),
            &current_bundle,
        );

        let scorecard_after = if config.run_scorecard_rerun {
            if let Some(path) = &config.core_performance_config_path {
                Some(rerun.run_bundle(path)?)
            } else {
                None
            }
        } else {
            None
        };
        let scorecard_summary = if config.run_scorecard_rerun {
            if config.core_performance_config_path.is_some() {
                rerun.summarize(
                    scorecard_before.as_ref().map(|bundle| &bundle.scorecard),
                    scorecard_after.as_ref().map(|bundle| &bundle.scorecard),
                    Vec::new(),
                    true,
                )
            } else {
                CoreScorecardRerun::missing("scorecard rerun config is missing")
            }
        } else {
            CoreScorecardRerun::missing("scorecard rerun disabled")
        };

        let previous_counts =
            ComparableEvidenceCountSummary::from_bundle(&comparable_config, &before_bundle);
        let current_counts =
            ComparableEvidenceCountSummary::from_bundle(&comparable_config, &current_bundle);
        let added_complete_rows =
            current_counts.complete_rows as isize - previous_counts.complete_rows as isize;
        let added_outcome_references = current_counts.outcome_references as isize
            - previous_counts.outcome_references as isize;
        let added_baseline_references = current_counts.baseline_references as isize
            - previous_counts.baseline_references as isize;
        let added_no_trade_counterfactuals = current_counts.no_trade_counterfactuals as isize
            - previous_counts.no_trade_counterfactuals as isize;
        let added_risk_denied_counterfactuals = current_counts.risk_denied_counterfactuals as isize
            - previous_counts.risk_denied_counterfactuals as isize;
        let added_official_comparable_rows = current_counts.official_comparable_rows as isize
            - previous_counts.official_comparable_rows as isize;
        let improvement_detected = added_complete_rows > 0
            || added_outcome_references > 0
            || added_baseline_references > 0
            || added_no_trade_counterfactuals > 0
            || added_risk_denied_counterfactuals > 0
            || added_official_comparable_rows > 0
            || materialization_report.status
                == ScenarioMaterializationWeakClosureStatus::MaterializationImproved
            || scorecard_summary.status_improved;

        let closure_status = determine_closure_status(
            &current_plan,
            &current_quality,
            &materialization_report,
            &current_bundle,
            improvement_detected,
        );
        let final_recommendation = determine_final_recommendation(
            &current_plan,
            &current_quality,
            &scorecard_summary,
            closure_status,
        );
        let closure_report = CounterfactualDepthClosureReport {
            closure_id: config.closure_id.clone(),
            comparable_bundle_summary: current_counts.clone(),
            depth_plan: current_plan.clone(),
            build_attempts,
            previous_counts: Some(previous_counts.clone()),
            current_counts: current_counts.clone(),
            added_complete_rows,
            added_outcome_references,
            added_baseline_references,
            added_no_trade_counterfactuals,
            added_risk_denied_counterfactuals,
            added_official_comparable_rows,
            bottleneck_before: scorecard_before.as_ref().map(|bundle| {
                format!(
                    "{:?}",
                    bundle.scorecard.bottleneck_report.primary_bottleneck
                )
            }),
            bottleneck_after: scorecard_after.as_ref().map(|bundle| {
                format!(
                    "{:?}",
                    bundle.scorecard.bottleneck_report.primary_bottleneck
                )
            }),
            improvement_detected,
            closure_status,
            final_recommendation,
            reason_codes: stable_reason_codes(
                &config
                    .reason_codes
                    .iter()
                    .cloned()
                    .chain([
                        ReasonCode::EvidenceClosureBuilt,
                        ReasonCode::DeterministicPath,
                    ])
                    .collect::<Vec<_>>(),
            ),
        };
        let bundle = CounterfactualDepthClosureBundle::new(
            config.closure_id.clone(),
            current_bundle,
            current_quality,
            current_plan,
            closure_report,
            materialization_report,
            Some(scorecard_summary),
        );
        bundle.write_to_dir(&config.output_dir())?;
        Ok(bundle)
    }
}

impl CounterfactualDepthClosureReport {
    pub fn to_text(&self) -> String {
        [
            format!("closure_id={}", self.closure_id),
            format!("added_complete_rows={}", self.added_complete_rows),
            format!("added_outcome_references={}", self.added_outcome_references),
            format!(
                "added_baseline_references={}",
                self.added_baseline_references
            ),
            format!(
                "added_no_trade_counterfactuals={}",
                self.added_no_trade_counterfactuals
            ),
            format!(
                "added_risk_denied_counterfactuals={}",
                self.added_risk_denied_counterfactuals
            ),
            format!(
                "added_official_comparable_rows={}",
                self.added_official_comparable_rows
            ),
            format!(
                "bottleneck_before={}",
                self.bottleneck_before.clone().unwrap_or_default()
            ),
            format!(
                "bottleneck_after={}",
                self.bottleneck_after.clone().unwrap_or_default()
            ),
            format!("improvement_detected={}", self.improvement_detected),
            format!("closure_status={:?}", self.closure_status),
            format!("final_recommendation={:?}", self.final_recommendation),
            format!("build_attempts={}", self.build_attempts.join(" | ")),
        ]
        .join("\n")
    }
}

fn determine_closure_status(
    plan: &CounterfactualDepthPlan,
    quality: &ComparableEvidenceQualityReport,
    materialization: &ScenarioMaterializationWeakClosureReport,
    bundle: &ComparableCommitteeEvidenceBundle,
    improvement_detected: bool,
) -> CounterfactualDepthClosureStatus {
    if plan.rows_missing_candles > 0 {
        CounterfactualDepthClosureStatus::MissingLocalCandles
    } else if plan.rows_missing_outcome > 0 {
        CounterfactualDepthClosureStatus::MissingOutcomeReferences
    } else if plan.rows_missing_baseline > 0 {
        CounterfactualDepthClosureStatus::MissingBaselineReferences
    } else if plan.rows_missing_no_trade > 0 || plan.rows_missing_risk_denied > 0 {
        CounterfactualDepthClosureStatus::MissingCounterfactuals
    } else if !improvement_detected {
        CounterfactualDepthClosureStatus::NoImprovement
    } else if bundle.non_crypto_official_rows == 0 && bundle.controlled_rows > 0 {
        CounterfactualDepthClosureStatus::ControlledOnlyImprovement
    } else if bundle.non_crypto_official_rows == 0 && bundle.crypto_only_rows > 0 {
        CounterfactualDepthClosureStatus::CryptoOnlyImprovement
    } else if bundle.non_crypto_official_rows == 0 && bundle.yfinance_rows > 0 {
        CounterfactualDepthClosureStatus::ResearchOnlyImprovement
    } else if materialization.status
        == ScenarioMaterializationWeakClosureStatus::MaterializationImproved
    {
        CounterfactualDepthClosureStatus::ScenarioMaterializationImproved
    } else if quality.official_complete_rows > 0 {
        CounterfactualDepthClosureStatus::OfficialComparableRowsImproved
    } else if improvement_detected {
        CounterfactualDepthClosureStatus::CounterfactualDepthImproved
    } else {
        CounterfactualDepthClosureStatus::ImprovedButStillEvidenceBlocked
    }
}

fn determine_final_recommendation(
    plan: &CounterfactualDepthPlan,
    quality: &ComparableEvidenceQualityReport,
    scorecard_summary: &CoreScorecardRerunSummary,
    closure_status: CounterfactualDepthClosureStatus,
) -> CounterfactualDepthFinalRecommendation {
    if plan.rows_missing_candles > 0 {
        CounterfactualDepthFinalRecommendation::ImproveCandleCoverageFirst
    } else if quality.quality_status == ComparableEvidenceQualityStatus::NeedMoreBaselineReferences
        || plan.rows_missing_baseline > 0
    {
        CounterfactualDepthFinalRecommendation::ImproveBaselineReferenceDepth
    } else if !scorecard_summary.ran {
        CounterfactualDepthFinalRecommendation::RerunCorePerformance
    } else if matches!(
        closure_status,
        CounterfactualDepthClosureStatus::OfficialComparableRowsImproved
            | CounterfactualDepthClosureStatus::CounterfactualDepthImproved
            | CounterfactualDepthClosureStatus::ScenarioMaterializationImproved
    ) {
        CounterfactualDepthFinalRecommendation::CommitteeCoreReadyForDeeperEvidence
    } else if matches!(
        closure_status,
        CounterfactualDepthClosureStatus::ControlledOnlyImprovement
            | CounterfactualDepthClosureStatus::CryptoOnlyImprovement
            | CounterfactualDepthClosureStatus::ResearchOnlyImprovement
    ) {
        CounterfactualDepthFinalRecommendation::KeepTrinity
    } else if quality.quality_status
        == ComparableEvidenceQualityStatus::NeedMoreOfficialComparableRows
    {
        CounterfactualDepthFinalRecommendation::MoreOfficialEvidence
    } else if quality.summary_derived_ratio > 0.50 {
        CounterfactualDepthFinalRecommendation::ImproveScenarioMaterializationFirst
    } else {
        CounterfactualDepthFinalRecommendation::NeedMoreEvidence
    }
}

fn default_output_root() -> String {
    "target/soma_counterfactual_depth_closure".to_string()
}

fn default_true() -> bool {
    true
}

fn default_max_build_attempts() -> usize {
    4
}
