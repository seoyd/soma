use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};
use crate::experiment::{
    CoreBottleneckKind, CorePerformanceScorecard, CorePerformanceScorecardConfig,
    CoreScorecardRerun,
};

use super::baseline_reference_backfill::{
    BaselineReferenceBackfillPlan, build_baseline_reference_backfill_plan,
};
use super::committee_outcome_coverage::CommitteeOutcomeCoverageConfig;
use super::committee_outcome_coverage_runner::CommitteeOutcomeCoverageRunner;
use super::complete_comparable_row_builder::{
    CompleteComparableRowBuildStatus, CompleteComparableRowBuilder,
    CompleteComparableRowBuilderConfig, CompleteComparableRowBundle,
};
use super::complete_row_closure_bundle::{
    CompleteRowClosureBundle, build_complete_row_closure_final_summary,
    build_complete_row_closure_storage_report,
};
use super::core_bottleneck_movement::build_core_bottleneck_movement_report;
use super::counterfactual_backfill_plan::{
    CounterfactualBackfillPlan, build_counterfactual_backfill_plan,
};
use super::counterfactual_depth_closure::{
    CounterfactualDepthClosureConfig, CounterfactualDepthClosureRunner,
};
use super::official_candle_coverage_pack::{
    OfficialCandleCoveragePack, load_pack_from_path_or_config,
};
use super::official_ready_row_inventory::{
    OfficialReadyRowCompletenessStatus, OfficialReadyRowInventoryConfig,
    OfficialReadyRowInventoryReport, OfficialReadyRowInventoryRunner,
};
use super::outcome_reference_backfill::{
    OutcomeReferenceBackfillPlan, build_outcome_reference_backfill_plan,
};
use super::scenario_materialization_v3::{
    ScenarioMaterializationV3Config, ScenarioMaterializationV3Report,
    ScenarioMaterializationV3Runner, ScenarioMaterializationV3Status,
};
use super::{
    CommitteeReferencePackConfig, CommitteeReferencePackRunner, ComparableCommitteeEvidenceBundle,
    ComparableCommitteeEvidenceConfig, ComparableCommitteeEvidenceRow, ComparableEvidenceBuilder,
    ComparableEvidenceSourceClass,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompleteRowClosureConfig {
    pub closure_id: String,
    #[serde(default)]
    pub official_ready_inventory_config_path: Option<String>,
    #[serde(default)]
    pub scenario_materialization_v3_config_path: Option<String>,
    #[serde(default)]
    pub comparable_evidence_config_path: Option<String>,
    #[serde(default)]
    pub reference_pack_config_paths: Vec<String>,
    #[serde(default)]
    pub outcome_coverage_config_paths: Vec<String>,
    #[serde(default)]
    pub counterfactual_depth_closure_config_path: Option<String>,
    #[serde(default)]
    pub core_performance_config_path: Option<String>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default = "default_true")]
    pub run_inventory: bool,
    #[serde(default = "default_true")]
    pub run_scenario_materialization_v3: bool,
    #[serde(default = "default_true")]
    pub run_outcome_backfill: bool,
    #[serde(default = "default_true")]
    pub run_baseline_backfill: bool,
    #[serde(default = "default_true")]
    pub run_counterfactual_backfill: bool,
    #[serde(default = "default_true")]
    pub run_complete_row_builder: bool,
    #[serde(default)]
    pub run_committee_benchmark: bool,
    #[serde(default)]
    pub run_outcome_coverage: bool,
    #[serde(default)]
    pub run_counterfactual_depth_close: bool,
    #[serde(default)]
    pub run_core_scorecard_rerun: bool,
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
pub enum CompleteRowClosureStatus {
    CompleteRowsImproved,
    OfficialCompleteRowsImproved,
    OutcomeReferencesImproved,
    BaselineReferencesImproved,
    CounterfactualsImproved,
    BottleneckMoved,
    StillMissingScenarioRows,
    StillMissingOutcomeReferences,
    StillMissingBaselineReferences,
    StillMissingCounterfactuals,
    StillScenarioMaterializationWeak,
    #[default]
    NoImprovement,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CompleteRowClosureRecommendation {
    ImproveOutcomeLinkingFirst,
    ImproveBaselineReferenceDepth,
    ImproveCounterfactualDepthFirst,
    ImproveScenarioMaterializationFirst,
    NeedLongerCandleWindow,
    MoreOfficialEvidence,
    RerunCorePerformance,
    CommitteeCoreReadyForDeeperEvidence,
    KeepTrinity,
    #[default]
    NeedMoreEvidence,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CompleteRowClosureReport {
    pub closure_id: String,
    #[serde(default)]
    pub before_complete_rows: Option<usize>,
    pub after_complete_rows: usize,
    #[serde(default)]
    pub before_official_complete_rows: Option<usize>,
    pub after_official_complete_rows: usize,
    pub added_complete_rows: isize,
    pub added_official_complete_rows: isize,
    pub added_outcome_references: isize,
    pub added_baseline_references: isize,
    pub added_no_trade_counterfactuals: isize,
    pub added_risk_denied_counterfactuals: isize,
    #[serde(default)]
    pub previous_bottleneck: Option<CoreBottleneckKind>,
    #[serde(default)]
    pub current_bottleneck: Option<CoreBottleneckKind>,
    pub bottleneck_changed: bool,
    pub closure_status: CompleteRowClosureStatus,
    pub final_recommendation: CompleteRowClosureRecommendation,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CompleteRowClosureRunner;

impl Default for CompleteRowClosureConfig {
    fn default() -> Self {
        Self {
            closure_id: "complete-row-closure".to_string(),
            official_ready_inventory_config_path: None,
            scenario_materialization_v3_config_path: None,
            comparable_evidence_config_path: None,
            reference_pack_config_paths: Vec::new(),
            outcome_coverage_config_paths: Vec::new(),
            counterfactual_depth_closure_config_path: None,
            core_performance_config_path: None,
            output_root: default_output_root(),
            run_inventory: true,
            run_scenario_materialization_v3: true,
            run_outcome_backfill: true,
            run_baseline_backfill: true,
            run_counterfactual_backfill: true,
            run_complete_row_builder: true,
            run_committee_benchmark: false,
            run_outcome_coverage: false,
            run_counterfactual_depth_close: false,
            run_core_scorecard_rerun: false,
            max_rows: default_max_rows(),
            max_symbols: default_max_symbols(),
            max_bytes: default_max_bytes(),
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

impl CompleteRowClosureConfig {
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
        if self.closure_id.trim().is_empty() {
            return Err("complete row closure id must not be empty".to_string());
        }
        if self
            .all_paths()
            .iter()
            .chain(std::iter::once(&self.output_root))
            .any(|path| path.contains("://"))
        {
            return Err("complete row closure paths must be local".to_string());
        }
        if self.max_rows == 0 || self.max_rows > default_max_rows() {
            return Err("complete row closure max_rows must be between 1 and 500".to_string());
        }
        if self.max_symbols == 0 || self.max_symbols > default_max_symbols() {
            return Err("complete row closure max_symbols must be between 1 and 5".to_string());
        }
        if self.max_bytes == 0 || self.max_bytes > default_max_bytes() {
            return Err("complete row closure max_bytes must be between 1 and 5000000".to_string());
        }
        if self.comparable_evidence_config_path.is_none()
            && self.official_ready_inventory_config_path.is_none()
        {
            return Err(
                "complete row closure requires comparable_evidence_config_path or official_ready_inventory_config_path".to_string(),
            );
        }
        Ok(())
    }

    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.closure_id)
    }

    pub fn all_paths(&self) -> Vec<String> {
        self.official_ready_inventory_config_path
            .iter()
            .cloned()
            .chain(self.scenario_materialization_v3_config_path.iter().cloned())
            .chain(self.comparable_evidence_config_path.iter().cloned())
            .chain(self.reference_pack_config_paths.iter().cloned())
            .chain(self.outcome_coverage_config_paths.iter().cloned())
            .chain(
                self.counterfactual_depth_closure_config_path
                    .iter()
                    .cloned(),
            )
            .chain(self.core_performance_config_path.iter().cloned())
            .collect()
    }
}

impl CompleteRowClosureRunner {
    pub fn run(
        &self,
        config: &CompleteRowClosureConfig,
    ) -> Result<CompleteRowClosureBundle, String> {
        config.validate()?;
        let (comparable_config, base_bundle) = load_base_bundle(config)?;
        enforce_limits(config, &base_bundle)?;
        let base_rows = base_bundle.rows.clone();
        let previous_bottleneck_from_rows = infer_bottleneck_from_rows(&base_rows);

        let inventory_report = if config.run_inventory {
            load_or_build_inventory(config, &base_rows)?
        } else {
            load_or_build_inventory(config, &base_rows)?
        };
        let candidate_row_ids = candidate_row_ids(&inventory_report);
        let before_complete_rows = count_complete_rows_for_ids(&base_rows, &candidate_row_ids);
        let before_official_complete_rows =
            count_official_complete_rows_for_ids(&base_rows, &candidate_row_ids);
        let candle_packs = load_candle_packs_from_inventory_config(config)?;
        let scenario_report = if config.run_scenario_materialization_v3 {
            load_or_build_materialization(config, &inventory_report, &candle_packs)?
        } else {
            load_or_build_materialization(config, &inventory_report, &candle_packs)?
        };
        let outcome_backfill_plan = if config.run_outcome_backfill {
            build_outcome_reference_backfill_plan(
                format!("{}-outcome", config.closure_id),
                &base_rows,
                Some(&scenario_report),
                &candle_packs,
            )
        } else {
            build_outcome_reference_backfill_plan(
                format!("{}-outcome", config.closure_id),
                &base_rows,
                Some(&scenario_report),
                &candle_packs,
            )
        };
        let baseline_backfill_plan = if config.run_baseline_backfill {
            build_baseline_reference_backfill_plan(
                format!("{}-baseline", config.closure_id),
                &base_rows,
            )
        } else {
            build_baseline_reference_backfill_plan(
                format!("{}-baseline", config.closure_id),
                &base_rows,
            )
        };
        let counterfactual_plan_rows =
            project_rows_for_counterfactual_plan(&base_rows, &outcome_backfill_plan);
        let counterfactual_backfill_plan = if config.run_counterfactual_backfill {
            build_counterfactual_backfill_plan(
                format!("{}-counterfactual", config.closure_id),
                &counterfactual_plan_rows,
            )
        } else {
            build_counterfactual_backfill_plan(
                format!("{}-counterfactual", config.closure_id),
                &counterfactual_plan_rows,
            )
        };
        let complete_builder_config = CompleteComparableRowBuilderConfig {
            bundle_id: config.closure_id.clone(),
            allow_diagnostic_complete: true,
            allow_controlled_diagnostic: comparable_config.allow_controlled_evidence,
            allow_crypto_only: comparable_config.allow_crypto_only,
            allow_yfinance_research: comparable_config.allow_yfinance_research,
            allow_fixture: comparable_config.allow_fixture,
            reason_codes: config.reason_codes.clone(),
        };
        let complete_bundle = if config.run_complete_row_builder {
            CompleteComparableRowBuilder::default().build(
                &complete_builder_config,
                &base_rows,
                &scenario_report,
                &outcome_backfill_plan,
                &baseline_backfill_plan,
                &counterfactual_backfill_plan,
            )
        } else {
            CompleteComparableRowBuilder::default().build(
                &complete_builder_config,
                &base_rows,
                &scenario_report,
                &outcome_backfill_plan,
                &baseline_backfill_plan,
                &counterfactual_backfill_plan,
            )
        };
        if config.run_committee_benchmark {
            rerun_reference_generation(&config.reference_pack_config_paths)?;
        }
        if config.run_outcome_coverage {
            rerun_outcome_coverage(&config.outcome_coverage_config_paths)?;
        }
        if config.run_counterfactual_depth_close {
            rerun_counterfactual_depth(config.counterfactual_depth_closure_config_path.as_deref())?;
        }
        let (previous_bottleneck, current_bottleneck) = if config.run_core_scorecard_rerun {
            rerun_core_scorecard(
                config.core_performance_config_path.as_deref(),
                previous_bottleneck_from_rows,
                infer_bottleneck_from_complete_bundle(&complete_bundle, &scenario_report),
            )?
        } else {
            (
                previous_bottleneck_from_rows,
                infer_bottleneck_from_complete_bundle(&complete_bundle, &scenario_report),
            )
        };
        let movement_report =
            build_core_bottleneck_movement_report(previous_bottleneck, current_bottleneck);
        let closure_report = build_closure_report(
            config,
            &inventory_report,
            &scenario_report,
            &outcome_backfill_plan,
            &baseline_backfill_plan,
            &counterfactual_backfill_plan,
            &complete_bundle,
            before_complete_rows,
            before_official_complete_rows,
            movement_report.previous_primary_bottleneck,
            movement_report.current_primary_bottleneck,
            movement_report.bottleneck_changed,
        );
        let mut bundle = CompleteRowClosureBundle {
            inventory_report,
            scenario_materialization_v3_report: scenario_report,
            outcome_backfill_plan,
            baseline_backfill_plan,
            counterfactual_backfill_plan,
            complete_comparable_row_bundle: complete_bundle,
            complete_row_closure_report: closure_report,
            core_bottleneck_movement_report: Some(movement_report),
            storage_report: super::complete_row_closure_bundle::CompleteRowClosureStorageReport {
                max_bytes: config.max_bytes,
                estimated_output_bytes: 0,
                within_budget: true,
                guidance: String::new(),
                input_paths: config.all_paths(),
                reason_codes: Vec::new(),
            },
            final_summary: String::new(),
            reason_codes: stable_reason_codes(
                &config
                    .reason_codes
                    .iter()
                    .cloned()
                    .chain([ReasonCode::DeterministicPath, ReasonCode::LocalFileOnly])
                    .collect::<Vec<_>>(),
            ),
        };
        bundle.final_summary = build_complete_row_closure_final_summary(&bundle);
        bundle.storage_report = build_complete_row_closure_storage_report(
            config.max_bytes,
            config.all_paths(),
            &bundle,
        );
        bundle.write_to_dir(&config.output_dir())?;
        Ok(bundle)
    }
}

impl CompleteRowClosureReport {
    pub fn to_text(&self) -> String {
        [
            format!("closure_id={}", self.closure_id),
            format!(
                "before_complete_rows={}",
                self.before_complete_rows.unwrap_or_default()
            ),
            format!("after_complete_rows={}", self.after_complete_rows),
            format!(
                "before_official_complete_rows={}",
                self.before_official_complete_rows.unwrap_or_default()
            ),
            format!(
                "after_official_complete_rows={}",
                self.after_official_complete_rows
            ),
            format!("added_complete_rows={}", self.added_complete_rows),
            format!(
                "added_official_complete_rows={}",
                self.added_official_complete_rows
            ),
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
                "previous_bottleneck={}",
                self.previous_bottleneck
                    .map(|value| format!("{value:?}"))
                    .unwrap_or_default()
            ),
            format!(
                "current_bottleneck={}",
                self.current_bottleneck
                    .map(|value| format!("{value:?}"))
                    .unwrap_or_default()
            ),
            format!("bottleneck_changed={}", self.bottleneck_changed),
            format!("closure_status={:?}", self.closure_status),
            format!("final_recommendation={:?}", self.final_recommendation),
        ]
        .join("\n")
    }
}

fn load_base_bundle(
    config: &CompleteRowClosureConfig,
) -> Result<
    (
        ComparableCommitteeEvidenceConfig,
        ComparableCommitteeEvidenceBundle,
    ),
    String,
> {
    if let Some(path) = config.comparable_evidence_config_path.as_deref() {
        if path.ends_with(".toml") {
            let comparable_config =
                ComparableCommitteeEvidenceConfig::from_toml_path(Path::new(path))?;
            let bundle = ComparableEvidenceBuilder::default().build(&comparable_config)?;
            Ok((comparable_config, bundle))
        } else {
            let bundle = ComparableCommitteeEvidenceBundle::from_json_path(Path::new(path))?;
            Ok((
                ComparableCommitteeEvidenceConfig {
                    comparable_id: bundle.comparable_id.clone(),
                    output_root: config.output_root.clone(),
                    ..ComparableCommitteeEvidenceConfig::default()
                },
                bundle,
            ))
        }
    } else if let Some(path) = config.official_ready_inventory_config_path.as_deref() {
        let inventory_config = OfficialReadyRowInventoryConfig::from_toml_path(Path::new(path))?;
        let comparable_path = inventory_config
            .comparable_evidence_bundle_paths
            .first()
            .ok_or_else(|| {
                "inventory config must include comparable_evidence_bundle_paths".to_string()
            })?;
        let fallback = CompleteRowClosureConfig {
            comparable_evidence_config_path: Some(comparable_path.clone()),
            ..config.clone()
        };
        load_base_bundle(&fallback)
    } else {
        Err("unable to load comparable evidence bundle for complete row closure".to_string())
    }
}

fn enforce_limits(
    config: &CompleteRowClosureConfig,
    bundle: &ComparableCommitteeEvidenceBundle,
) -> Result<(), String> {
    if bundle.rows.len() > config.max_rows {
        return Err(format!(
            "complete row closure loaded {} rows which exceeds max_rows {}",
            bundle.rows.len(),
            config.max_rows
        ));
    }
    let unique_symbols = bundle
        .rows
        .iter()
        .map(|row| row.symbol.clone())
        .collect::<std::collections::BTreeSet<_>>();
    if unique_symbols.len() > config.max_symbols {
        return Err(format!(
            "complete row closure loaded {} symbols which exceeds max_symbols {}",
            unique_symbols.len(),
            config.max_symbols
        ));
    }
    let storage_bytes = bundle
        .storage_bytes
        .max(input_storage_bytes(&config.all_paths()));
    if storage_bytes > config.max_bytes {
        return Err(format!(
            "complete row closure input size {} exceeds max_bytes {}",
            storage_bytes, config.max_bytes
        ));
    }
    Ok(())
}

fn load_or_build_inventory(
    config: &CompleteRowClosureConfig,
    rows: &[ComparableCommitteeEvidenceRow],
) -> Result<OfficialReadyRowInventoryReport, String> {
    if let Some(path) = config.official_ready_inventory_config_path.as_deref() {
        if path.ends_with(".toml") {
            let inventory_config =
                OfficialReadyRowInventoryConfig::from_toml_path(Path::new(path))?;
            OfficialReadyRowInventoryRunner::default().run(&inventory_config)
        } else {
            OfficialReadyRowInventoryReport::from_json_path(Path::new(path))
        }
    } else {
        let inventory_config = OfficialReadyRowInventoryConfig {
            inventory_id: format!("{}-inventory", config.closure_id),
            comparable_evidence_bundle_paths: config
                .comparable_evidence_config_path
                .iter()
                .cloned()
                .collect(),
            output_root: config.output_root.clone(),
            allow_controlled_diagnostic: true,
            allow_crypto_only: true,
            allow_yfinance_research: true,
            allow_fixture: true,
            reason_codes: config.reason_codes.clone(),
            ..OfficialReadyRowInventoryConfig::default()
        };
        OfficialReadyRowInventoryRunner::default().run_from_rows(
            &inventory_config,
            rows,
            &std::collections::BTreeMap::new(),
            &std::collections::BTreeMap::new(),
        )
    }
}

fn load_candle_packs_from_inventory_config(
    config: &CompleteRowClosureConfig,
) -> Result<Vec<OfficialCandleCoveragePack>, String> {
    if let Some(path) = config.official_ready_inventory_config_path.as_deref() {
        if path.ends_with(".toml") {
            let inventory_config =
                OfficialReadyRowInventoryConfig::from_toml_path(Path::new(path))?;
            return inventory_config
                .official_candle_coverage_pack_paths
                .iter()
                .map(|path| load_pack_from_path_or_config(path))
                .collect();
        }
    }
    Ok(Vec::new())
}

fn load_or_build_materialization(
    config: &CompleteRowClosureConfig,
    inventory_report: &OfficialReadyRowInventoryReport,
    candle_packs: &[OfficialCandleCoveragePack],
) -> Result<ScenarioMaterializationV3Report, String> {
    if let Some(path) = config.scenario_materialization_v3_config_path.as_deref() {
        if path.ends_with(".toml") {
            let materialization_config =
                ScenarioMaterializationV3Config::from_toml_path(Path::new(path))?;
            ScenarioMaterializationV3Runner::default().run(&materialization_config)
        } else {
            ScenarioMaterializationV3Report::from_json_path(Path::new(path))
        }
    } else {
        let descriptors = candle_packs
            .iter()
            .flat_map(|pack| pack.descriptors.iter().cloned())
            .map(|descriptor| (descriptor.candle_series_id.clone(), descriptor))
            .collect();
        let materialization_config = ScenarioMaterializationV3Config {
            materialization_id: format!("{}-materialization", config.closure_id),
            output_root: config.output_root.clone(),
            reason_codes: config.reason_codes.clone(),
            ..ScenarioMaterializationV3Config::default()
        };
        ScenarioMaterializationV3Runner::default().run_from_inventory(
            &materialization_config,
            inventory_report,
            &descriptors,
            &std::collections::BTreeMap::new(),
        )
    }
}

fn project_rows_for_counterfactual_plan(
    rows: &[ComparableCommitteeEvidenceRow],
    outcome_plan: &OutcomeReferenceBackfillPlan,
) -> Vec<ComparableCommitteeEvidenceRow> {
    let buildable_rows = outcome_plan
        .items
        .iter()
        .filter(|item| item.can_build_from_candles)
        .map(|item| item.row_id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    rows.iter()
        .cloned()
        .map(|mut row| {
            if buildable_rows.contains(row.row_id.as_str()) {
                row.outcome_reference_available = true;
            }
            row
        })
        .collect()
}

fn rerun_reference_generation(paths: &[String]) -> Result<(), String> {
    for path in paths {
        let config = CommitteeReferencePackConfig::from_toml_path(Path::new(path))?;
        CommitteeReferencePackRunner::default().run(&config)?;
    }
    Ok(())
}

fn rerun_outcome_coverage(paths: &[String]) -> Result<(), String> {
    for path in paths {
        let config = CommitteeOutcomeCoverageConfig::from_toml_path(Path::new(path))?;
        CommitteeOutcomeCoverageRunner::default().run(&config)?;
    }
    Ok(())
}

fn rerun_counterfactual_depth(path: Option<&str>) -> Result<(), String> {
    if let Some(path) = path {
        let config = CounterfactualDepthClosureConfig::from_toml_path(Path::new(path))?;
        CounterfactualDepthClosureRunner::default().run_bundle(&config)?;
    }
    Ok(())
}

fn rerun_core_scorecard(
    path: Option<&str>,
    previous_fallback: Option<CoreBottleneckKind>,
    current_fallback: Option<CoreBottleneckKind>,
) -> Result<(Option<CoreBottleneckKind>, Option<CoreBottleneckKind>), String> {
    let Some(path) = path else {
        return Ok((previous_fallback, current_fallback));
    };
    let config = CorePerformanceScorecardConfig::from_toml_path(Path::new(path))?;
    let previous = config
        .previous_scorecard_paths
        .first()
        .and_then(|path| CorePerformanceScorecard::from_json_path(Path::new(path)).ok());
    let bundle = CoreScorecardRerun::default().run_bundle(path)?;
    let summary = CoreScorecardRerun::default().summarize(
        previous.as_ref(),
        Some(&bundle.scorecard),
        Vec::new(),
        true,
    );
    Ok((
        summary.previous_primary_bottleneck.or(previous_fallback),
        summary.current_primary_bottleneck.or(current_fallback),
    ))
}

fn build_closure_report(
    config: &CompleteRowClosureConfig,
    inventory_report: &OfficialReadyRowInventoryReport,
    scenario_report: &ScenarioMaterializationV3Report,
    outcome_plan: &OutcomeReferenceBackfillPlan,
    baseline_plan: &BaselineReferenceBackfillPlan,
    counterfactual_plan: &CounterfactualBackfillPlan,
    complete_bundle: &CompleteComparableRowBundle,
    before_complete_rows: usize,
    before_official_complete_rows: usize,
    previous_bottleneck: Option<CoreBottleneckKind>,
    current_bottleneck: Option<CoreBottleneckKind>,
    bottleneck_changed: bool,
) -> CompleteRowClosureReport {
    let candidate_row_ids = candidate_row_ids(inventory_report);
    let after_complete_rows =
        count_complete_rows_for_ids(&complete_bundle.rows, &candidate_row_ids);
    let after_official_complete_rows =
        count_official_complete_rows_for_ids(&complete_bundle.rows, &candidate_row_ids);
    let added_complete_rows = after_complete_rows as isize - before_complete_rows as isize;
    let added_official_complete_rows =
        after_official_complete_rows as isize - before_official_complete_rows as isize;
    let added_outcome_references =
        count_outcome_references_for_ids(&complete_bundle.rows, &candidate_row_ids) as isize
            - inventory_report
                .items
                .iter()
                .filter(|item| item.has_outcome_reference)
                .count() as isize;
    let added_baseline_references =
        count_baseline_references_for_ids(&complete_bundle.rows, &candidate_row_ids) as isize
            - inventory_report
                .items
                .iter()
                .filter(|item| item.has_baseline_reference)
                .count() as isize;
    let added_no_trade_counterfactuals =
        count_no_trade_counterfactuals_for_ids(&complete_bundle.rows, &candidate_row_ids) as isize
            - inventory_report
                .items
                .iter()
                .filter(|item| item.has_no_trade_counterfactual)
                .count() as isize;
    let added_risk_denied_counterfactuals =
        count_risk_denied_counterfactuals_for_ids(&complete_bundle.rows, &candidate_row_ids)
            as isize
            - inventory_report
                .items
                .iter()
                .filter(|item| item.has_risk_denied_counterfactual)
                .count() as isize;
    let closure_status = determine_closure_status(
        added_complete_rows,
        added_official_complete_rows,
        added_outcome_references,
        added_baseline_references,
        added_no_trade_counterfactuals,
        added_risk_denied_counterfactuals,
        inventory_report,
        scenario_report,
        bottleneck_changed,
        current_bottleneck,
    );
    let final_recommendation = determine_recommendation(
        closure_status,
        inventory_report,
        scenario_report,
        outcome_plan,
        baseline_plan,
        counterfactual_plan,
        after_complete_rows,
        after_official_complete_rows,
        config.run_core_scorecard_rerun,
    );
    CompleteRowClosureReport {
        closure_id: config.closure_id.clone(),
        before_complete_rows: Some(before_complete_rows),
        after_complete_rows,
        before_official_complete_rows: Some(before_official_complete_rows),
        after_official_complete_rows,
        added_complete_rows,
        added_official_complete_rows,
        added_outcome_references,
        added_baseline_references,
        added_no_trade_counterfactuals,
        added_risk_denied_counterfactuals,
        previous_bottleneck,
        current_bottleneck,
        bottleneck_changed,
        closure_status,
        final_recommendation,
        reason_codes: stable_reason_codes(
            &config
                .reason_codes
                .iter()
                .cloned()
                .chain([ReasonCode::DeterministicPath])
                .collect::<Vec<_>>(),
        ),
    }
}

fn determine_closure_status(
    added_complete_rows: isize,
    added_official_complete_rows: isize,
    added_outcome_references: isize,
    added_baseline_references: isize,
    added_no_trade_counterfactuals: isize,
    added_risk_denied_counterfactuals: isize,
    inventory_report: &OfficialReadyRowInventoryReport,
    scenario_report: &ScenarioMaterializationV3Report,
    bottleneck_changed: bool,
    current_bottleneck: Option<CoreBottleneckKind>,
) -> CompleteRowClosureStatus {
    if added_official_complete_rows > 0 {
        return CompleteRowClosureStatus::OfficialCompleteRowsImproved;
    }
    if added_complete_rows > 0 {
        return CompleteRowClosureStatus::CompleteRowsImproved;
    }
    if added_outcome_references > 0 {
        return CompleteRowClosureStatus::OutcomeReferencesImproved;
    }
    if added_baseline_references > 0 {
        return CompleteRowClosureStatus::BaselineReferencesImproved;
    }
    if added_no_trade_counterfactuals > 0 || added_risk_denied_counterfactuals > 0 {
        return CompleteRowClosureStatus::CounterfactualsImproved;
    }
    if bottleneck_changed {
        return CompleteRowClosureStatus::BottleneckMoved;
    }
    if current_bottleneck == Some(CoreBottleneckKind::ScenarioMaterializationWeak)
        || matches!(
            scenario_report.materialization_status,
            ScenarioMaterializationV3Status::StillMissingScenarioRows
                | ScenarioMaterializationV3Status::StillSummaryDerivedOnly
        )
    {
        return CompleteRowClosureStatus::StillScenarioMaterializationWeak;
    }
    if inventory_report.items.iter().any(|item| {
        item.completeness_statuses
            .contains(&OfficialReadyRowCompletenessStatus::MissingScenarioRow)
    }) {
        return CompleteRowClosureStatus::StillMissingScenarioRows;
    }
    if inventory_report.items.iter().any(|item| {
        item.completeness_statuses
            .contains(&OfficialReadyRowCompletenessStatus::MissingOutcomeReference)
    }) {
        return CompleteRowClosureStatus::StillMissingOutcomeReferences;
    }
    if inventory_report.items.iter().any(|item| {
        item.completeness_statuses
            .contains(&OfficialReadyRowCompletenessStatus::MissingBaselineReference)
    }) {
        return CompleteRowClosureStatus::StillMissingBaselineReferences;
    }
    if inventory_report.items.iter().any(|item| {
        item.completeness_statuses
            .contains(&OfficialReadyRowCompletenessStatus::MissingNoTradeCounterfactual)
            || item
                .completeness_statuses
                .contains(&OfficialReadyRowCompletenessStatus::MissingRiskDeniedCounterfactual)
    }) {
        return CompleteRowClosureStatus::StillMissingCounterfactuals;
    }
    CompleteRowClosureStatus::NoImprovement
}

fn determine_recommendation(
    closure_status: CompleteRowClosureStatus,
    inventory_report: &OfficialReadyRowInventoryReport,
    scenario_report: &ScenarioMaterializationV3Report,
    outcome_plan: &OutcomeReferenceBackfillPlan,
    baseline_plan: &BaselineReferenceBackfillPlan,
    counterfactual_plan: &CounterfactualBackfillPlan,
    after_complete_rows: usize,
    after_official_complete_rows: usize,
    reran_core_scorecard: bool,
) -> CompleteRowClosureRecommendation {
    match closure_status {
        CompleteRowClosureStatus::StillScenarioMaterializationWeak
        | CompleteRowClosureStatus::StillMissingScenarioRows => {
            CompleteRowClosureRecommendation::ImproveScenarioMaterializationFirst
        }
        CompleteRowClosureStatus::StillMissingOutcomeReferences => {
            if outcome_plan.missing_future_window_count > 0 {
                CompleteRowClosureRecommendation::NeedLongerCandleWindow
            } else {
                CompleteRowClosureRecommendation::ImproveOutcomeLinkingFirst
            }
        }
        CompleteRowClosureStatus::StillMissingBaselineReferences => {
            CompleteRowClosureRecommendation::ImproveBaselineReferenceDepth
        }
        CompleteRowClosureStatus::StillMissingCounterfactuals => {
            CompleteRowClosureRecommendation::ImproveCounterfactualDepthFirst
        }
        _ if reran_core_scorecard && after_official_complete_rows > 0 => {
            CompleteRowClosureRecommendation::RerunCorePerformance
        }
        _ if after_official_complete_rows > 0 => {
            CompleteRowClosureRecommendation::CommitteeCoreReadyForDeeperEvidence
        }
        _ if after_complete_rows > 0
            && inventory_report.items.iter().all(|item| {
                !matches!(
                    item.source_class,
                    ComparableEvidenceSourceClass::OfficialNonCrypto
                )
            }) =>
        {
            CompleteRowClosureRecommendation::KeepTrinity
        }
        _ if after_complete_rows > 0 && scenario_report.official_materialized_count > 0 => {
            CompleteRowClosureRecommendation::KeepTrinity
        }
        _ if baseline_plan.unavailable_count == baseline_plan.items.len()
            || counterfactual_plan.unavailable_count == counterfactual_plan.items.len() =>
        {
            CompleteRowClosureRecommendation::MoreOfficialEvidence
        }
        _ => CompleteRowClosureRecommendation::NeedMoreEvidence,
    }
}

fn candidate_row_ids(report: &OfficialReadyRowInventoryReport) -> BTreeSet<&str> {
    report
        .items
        .iter()
        .map(|item| item.row_id.as_str())
        .collect()
}

fn count_complete_rows_for_ids(
    rows: &[ComparableCommitteeEvidenceRow],
    candidate_row_ids: &BTreeSet<&str>,
) -> usize {
    rows.iter()
        .filter(|row| candidate_row_ids.contains(row.row_id.as_str()) && row_complete(row))
        .count()
}

fn count_official_complete_rows_for_ids(
    rows: &[ComparableCommitteeEvidenceRow],
    candidate_row_ids: &BTreeSet<&str>,
) -> usize {
    rows.iter()
        .filter(|row| candidate_row_ids.contains(row.row_id.as_str()) && official_complete(row))
        .count()
}

fn count_outcome_references_for_ids(
    rows: &[ComparableCommitteeEvidenceRow],
    candidate_row_ids: &BTreeSet<&str>,
) -> usize {
    rows.iter()
        .filter(|row| {
            candidate_row_ids.contains(row.row_id.as_str()) && row.outcome_reference_available
        })
        .count()
}

fn count_baseline_references_for_ids(
    rows: &[ComparableCommitteeEvidenceRow],
    candidate_row_ids: &BTreeSet<&str>,
) -> usize {
    rows.iter()
        .filter(|row| {
            candidate_row_ids.contains(row.row_id.as_str()) && row.baseline_reference_available
        })
        .count()
}

fn count_no_trade_counterfactuals_for_ids(
    rows: &[ComparableCommitteeEvidenceRow],
    candidate_row_ids: &BTreeSet<&str>,
) -> usize {
    rows.iter()
        .filter(|row| {
            candidate_row_ids.contains(row.row_id.as_str()) && row.no_trade_counterfactual_available
        })
        .count()
}

fn count_risk_denied_counterfactuals_for_ids(
    rows: &[ComparableCommitteeEvidenceRow],
    candidate_row_ids: &BTreeSet<&str>,
) -> usize {
    rows.iter()
        .filter(|row| {
            candidate_row_ids.contains(row.row_id.as_str())
                && row.risk_denied_counterfactual_available
        })
        .count()
}

fn row_complete(row: &ComparableCommitteeEvidenceRow) -> bool {
    row.no_lookahead_safe
        && (row.row_level || row.scenario_row_id.is_some())
        && row.outcome_reference_available
        && row.baseline_reference_available
        && row.no_trade_counterfactual_available
        && row.risk_denied_counterfactual_available
}

fn official_complete(row: &ComparableCommitteeEvidenceRow) -> bool {
    row_complete(row)
        && row.source_class == ComparableEvidenceSourceClass::OfficialNonCrypto
        && !row.summary_derived
        && !row.diagnostic_only
}

fn infer_bottleneck_from_rows(
    rows: &[ComparableCommitteeEvidenceRow],
) -> Option<CoreBottleneckKind> {
    if rows.iter().any(|row| {
        row.candle_official_ready_match
            && (!row.row_level || row.summary_derived || row.scenario_row_id.is_none())
    }) {
        return Some(CoreBottleneckKind::ScenarioMaterializationWeak);
    }
    if rows
        .iter()
        .any(|row| row.candle_official_ready_match && !row.outcome_reference_available)
    {
        return Some(CoreBottleneckKind::MissingOutcomeLinks);
    }
    if rows
        .iter()
        .any(|row| row.candle_official_ready_match && !row.baseline_reference_available)
    {
        return Some(CoreBottleneckKind::MissingBaselineReferences);
    }
    if rows
        .iter()
        .any(|row| row.candle_official_ready_match && !row.no_trade_counterfactual_available)
    {
        return Some(CoreBottleneckKind::MissingNoTradeCounterfactuals);
    }
    if rows
        .iter()
        .any(|row| row.candle_official_ready_match && !row.risk_denied_counterfactual_available)
    {
        return Some(CoreBottleneckKind::MissingRiskDeniedCounterfactuals);
    }
    if rows
        .iter()
        .all(|row| row.source_class != ComparableEvidenceSourceClass::OfficialNonCrypto)
    {
        return Some(CoreBottleneckKind::MissingOfficialData);
    }
    None
}

fn infer_bottleneck_from_complete_bundle(
    bundle: &CompleteComparableRowBundle,
    scenario_report: &ScenarioMaterializationV3Report,
) -> Option<CoreBottleneckKind> {
    if matches!(
        scenario_report.materialization_status,
        ScenarioMaterializationV3Status::StillMissingScenarioRows
            | ScenarioMaterializationV3Status::StillSummaryDerivedOnly
    ) || bundle
        .build_records
        .iter()
        .any(|record| record.status == CompleteComparableRowBuildStatus::SkippedMissingScenario)
    {
        return Some(CoreBottleneckKind::ScenarioMaterializationWeak);
    }
    if bundle
        .build_records
        .iter()
        .any(|record| record.status == CompleteComparableRowBuildStatus::SkippedMissingOutcome)
    {
        return Some(CoreBottleneckKind::MissingOutcomeLinks);
    }
    if bundle
        .build_records
        .iter()
        .any(|record| record.status == CompleteComparableRowBuildStatus::SkippedMissingBaseline)
    {
        return Some(CoreBottleneckKind::MissingBaselineReferences);
    }
    if bundle.build_records.iter().any(|record| {
        record.status == CompleteComparableRowBuildStatus::SkippedMissingCounterfactuals
    }) {
        let missing_no_trade = bundle
            .rows
            .iter()
            .any(|row| !row.no_trade_counterfactual_available);
        return Some(if missing_no_trade {
            CoreBottleneckKind::MissingNoTradeCounterfactuals
        } else {
            CoreBottleneckKind::MissingRiskDeniedCounterfactuals
        });
    }
    if bundle.official_complete_rows == 0 && bundle.complete_rows == 0 {
        return Some(CoreBottleneckKind::EvidenceTooWeak);
    }
    Some(CoreBottleneckKind::NoBottleneckDetected)
}

fn input_storage_bytes(paths: &[String]) -> usize {
    paths
        .iter()
        .filter_map(|path| fs::metadata(path).ok())
        .map(|metadata| metadata.len() as usize)
        .sum()
}

fn default_output_root() -> String {
    "target/soma_complete_row_closure".to_string()
}

fn default_max_rows() -> usize {
    500
}

fn default_max_symbols() -> usize {
    5
}

fn default_max_bytes() -> usize {
    5_000_000
}

fn default_true() -> bool {
    true
}
