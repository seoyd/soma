use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};
use crate::experiment::{
    CoreBottleneckKind, CorePerformanceFinalStatus, CorePerformanceRerunAfterOutcomeLinkage,
};

use super::baseline_reference_backfill::{
    BaselineReferenceBackfillPlan, build_baseline_reference_backfill_plan,
};
use super::comparable_committee_evidence::{
    ComparableCommitteeEvidenceBundle, ComparableCommitteeEvidenceRow,
    ComparableEvidenceSourceClass,
};
use super::complete_comparable_row_builder::{
    CompleteComparableRowBuildStatus, CompleteComparableRowBuilder,
    CompleteComparableRowBuilderConfig, CompleteComparableRowBundle,
};
use super::complete_row_closure::{CompleteRowClosureConfig, CompleteRowClosureRunner};
use super::complete_row_closure_bundle::{
    CompleteRowClosureBundle, CompleteRowClosureStorageReport,
};
use super::complete_row_closure_v2_bundle::{
    CompleteRowClosureV2Bundle, build_complete_row_closure_v2_storage_report,
    build_complete_row_closure_v2_summary,
};
use super::counterfactual_backfill_plan::{
    CounterfactualBackfillGapKind, CounterfactualBackfillPlan, CounterfactualBackfillPlanItem,
    CounterfactualBackfillSuggestedAction,
};
use super::counterfactual_completion_v2::{
    CounterfactualCompletionV2Config, CounterfactualCompletionV2RecordStatus,
    CounterfactualCompletionV2Report, CounterfactualCompletionV2Runner,
};
use super::future_window_requirements::{
    FutureWindowRequirementConfig, FutureWindowRequirementReport, FutureWindowRequirementRunner,
    load_descriptor_map_from_paths,
};
use super::official_future_window_extension::{
    FutureWindowExtensionPlan, OfficialFutureWindowExtensionConfig,
    build_official_future_window_extension_plan,
};
use super::official_ready_row_inventory::{
    OfficialReadyRowInventoryConfig, OfficialReadyRowInventoryReport,
    OfficialReadyRowInventoryRunner,
};
use super::outcome_linkage_v3::{
    OutcomeLinkageV3Config, OutcomeLinkageV3Report, OutcomeLinkageV3Runner,
};
use super::outcome_reference_backfill::{
    OutcomeBackfillGapKind, OutcomeBackfillSuggestedAction, OutcomeReferenceBackfillPlan,
    OutcomeReferenceBackfillPlanItem,
};
use super::scenario_materialization_v3::{
    ScenarioMaterializationV3Level, ScenarioMaterializationV3Record,
    ScenarioMaterializationV3Report, ScenarioMaterializationV3Status,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompleteRowClosureV2Config {
    pub closure_id: String,
    #[serde(default)]
    pub complete_row_closure_config_path: Option<String>,
    #[serde(default)]
    pub outcome_linkage_v3_config_path: Option<String>,
    #[serde(default)]
    pub counterfactual_completion_v2_config_path: Option<String>,
    #[serde(default)]
    pub core_performance_config_path: Option<String>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default = "default_true")]
    pub run_future_window_requirements: bool,
    #[serde(default = "default_true")]
    pub run_future_window_extension: bool,
    #[serde(default = "default_true")]
    pub run_outcome_linkage_v3: bool,
    #[serde(default = "default_true")]
    pub run_counterfactual_completion_v2: bool,
    #[serde(default = "default_true")]
    pub run_complete_row_close: bool,
    #[serde(default)]
    pub run_committee_official_benchmark: bool,
    #[serde(default)]
    pub run_outcome_coverage: bool,
    #[serde(default)]
    pub run_counterfactual_depth_close: bool,
    #[serde(default)]
    pub run_core_scorecard_rerun: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CompleteRowClosureV2Status {
    CompleteRowsImproved,
    OfficialCompleteRowsImproved,
    OutcomeReferencesImproved,
    CounterfactualsImproved,
    BottleneckMoved,
    StillNeedFutureWindow,
    StillNeedOutcomeReferences,
    StillNeedCounterfactuals,
    StillEvidenceTooWeak,
    StillNeedMoreOfficialRows,
    #[default]
    NoImprovement,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CompleteRowClosureV2Recommendation {
    ProvideLongerCandleWindow,
    ExtendOfficialCandleWindow,
    ImproveOutcomeLinkingFirst,
    ImproveCounterfactualDepthFirst,
    MoreOfficialEvidence,
    RerunCorePerformance,
    ImproveRiskGovernorFirst,
    ImproveChairFirst,
    CommitteeCoreReadyForDeeperEvidence,
    KeepTrinity,
    #[default]
    NeedMoreEvidence,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CompleteRowClosureV2Report {
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
    pub closure_status: CompleteRowClosureV2Status,
    pub final_recommendation: CompleteRowClosureV2Recommendation,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CompleteRowClosureV2Runner;

impl Default for CompleteRowClosureV2Config {
    fn default() -> Self {
        Self {
            closure_id: "complete-row-closure-v2".to_string(),
            complete_row_closure_config_path: None,
            outcome_linkage_v3_config_path: None,
            counterfactual_completion_v2_config_path: None,
            core_performance_config_path: None,
            output_root: default_output_root(),
            run_future_window_requirements: true,
            run_future_window_extension: true,
            run_outcome_linkage_v3: true,
            run_counterfactual_completion_v2: true,
            run_complete_row_close: true,
            run_committee_official_benchmark: false,
            run_outcome_coverage: false,
            run_counterfactual_depth_close: false,
            run_core_scorecard_rerun: false,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

impl CompleteRowClosureV2Config {
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
            return Err("complete row closure v2 id must not be empty".to_string());
        }
        if self
            .all_paths()
            .iter()
            .chain(std::iter::once(&self.output_root))
            .any(|path| path.contains("://"))
        {
            return Err("complete row closure v2 paths must be local".to_string());
        }
        Ok(())
    }

    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.closure_id)
    }

    pub fn all_paths(&self) -> Vec<String> {
        self.complete_row_closure_config_path
            .iter()
            .cloned()
            .chain(self.outcome_linkage_v3_config_path.iter().cloned())
            .chain(
                self.counterfactual_completion_v2_config_path
                    .iter()
                    .cloned(),
            )
            .chain(self.core_performance_config_path.iter().cloned())
            .collect()
    }
}

impl CompleteRowClosureV2Runner {
    pub fn run(
        &self,
        config: &CompleteRowClosureV2Config,
    ) -> Result<CompleteRowClosureV2Bundle, String> {
        config.validate()?;
        let base_rows = load_base_rows(config)?;
        let outcome_config = load_outcome_config(config)?;
        let counterfactual_config = load_counterfactual_config(config)?;
        let inventory_report = load_or_build_inventory(config, &outcome_config, &base_rows)?;
        let before_complete_rows = base_rows.iter().filter(|row| row_complete(row)).count();
        let before_official_complete_rows = base_rows
            .iter()
            .filter(|row| official_complete(row))
            .count();
        let scenario_report = load_or_build_materialization(&base_rows, &inventory_report);

        let future_window_requirement_report = if config.run_future_window_requirements {
            build_requirement_report(config, &outcome_config, &inventory_report)?
        } else {
            build_requirement_report(config, &outcome_config, &inventory_report)?
        };
        let future_window_extension_plan = if config.run_future_window_extension {
            Some(build_extension_plan(
                config,
                &outcome_config,
                &future_window_requirement_report,
            )?)
        } else {
            None
        };
        let descriptors =
            load_descriptor_map_from_paths(&outcome_config.extended_candle_pack_paths)?;
        let row_map = base_rows
            .iter()
            .cloned()
            .map(|row| (row.row_id.clone(), row))
            .collect::<BTreeMap<_, _>>();
        let outcome_linkage_v3_report = OutcomeLinkageV3Runner::default().run_from_inputs(
            &outcome_config,
            &inventory_report,
            &future_window_requirement_report,
            &descriptors,
            &row_map,
        )?;
        let baseline_plan = build_baseline_reference_backfill_plan(
            format!("{}-baseline", config.closure_id),
            &base_rows,
        );
        let counterfactual_completion_v2_report = CounterfactualCompletionV2Runner::default()
            .run_from_inputs(
                &counterfactual_config,
                &outcome_linkage_v3_report,
                &base_rows,
            )?;
        let complete_bundle = build_complete_bundle(
            config,
            &base_rows,
            &scenario_report,
            &outcome_linkage_v3_report,
            &baseline_plan,
            &counterfactual_completion_v2_report,
        );
        let previous_bottleneck = infer_bottleneck_from_rows(&base_rows);
        let current_bottleneck = infer_bottleneck_from_complete_bundle(&complete_bundle);
        let core_performance_rerun_summary = Some(CorePerformanceRerunAfterOutcomeLinkage::build(
            previous_bottleneck,
            current_bottleneck,
            Some(status_from_bottleneck(previous_bottleneck)),
            Some(status_from_bottleneck(current_bottleneck)),
            true,
            Vec::new(),
        ));
        let complete_row_closure_v2_report = build_closure_report(
            config,
            &future_window_requirement_report,
            &outcome_linkage_v3_report,
            &counterfactual_completion_v2_report,
            &complete_bundle,
            &baseline_plan,
            before_complete_rows,
            before_official_complete_rows,
            previous_bottleneck,
            current_bottleneck,
            core_performance_rerun_summary
                .as_ref()
                .is_some_and(|summary| summary.bottleneck_changed),
        );
        let mut bundle = CompleteRowClosureV2Bundle {
            future_window_requirement_report,
            future_window_extension_plan,
            outcome_linkage_v3_report,
            counterfactual_completion_v2_report,
            complete_row_closure_v2_report,
            core_performance_rerun_summary,
            storage_report: CompleteRowClosureStorageReport {
                max_bytes: 5_000_000,
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
        bundle.final_summary = build_complete_row_closure_v2_summary(&bundle);
        bundle.storage_report =
            build_complete_row_closure_v2_storage_report(5_000_000, config.all_paths(), &bundle);
        bundle.write_to_dir(&config.output_dir())?;
        Ok(bundle)
    }
}

impl CompleteRowClosureV2Report {
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

fn build_requirement_report(
    config: &CompleteRowClosureV2Config,
    outcome_config: &OutcomeLinkageV3Config,
    inventory_report: &OfficialReadyRowInventoryReport,
) -> Result<FutureWindowRequirementReport, String> {
    if let Some(path) = outcome_config.future_window_requirement_path.as_deref() {
        if path.ends_with(".json") {
            FutureWindowRequirementReport::from_json_path(Path::new(path))
        } else {
            let requirement_config =
                FutureWindowRequirementConfig::from_toml_path(Path::new(path))?;
            FutureWindowRequirementRunner::default().run(&requirement_config)
        }
    } else {
        let requirement_config = FutureWindowRequirementConfig {
            requirement_id: format!("{}-requirements", config.closure_id),
            official_ready_inventory_paths: outcome_config
                .official_ready_inventory_path
                .iter()
                .cloned()
                .collect(),
            candle_coverage_pack_paths: outcome_config.extended_candle_pack_paths.clone(),
            output_root: config.output_root.clone(),
            default_horizon_bars: outcome_config.default_horizon_bars,
            reason_codes: config.reason_codes.clone(),
            ..FutureWindowRequirementConfig::default()
        };
        let descriptors =
            load_descriptor_map_from_paths(&outcome_config.extended_candle_pack_paths)?;
        FutureWindowRequirementRunner::default().run_from_inventory(
            &requirement_config,
            inventory_report,
            &descriptors,
        )
    }
}

fn build_extension_plan(
    config: &CompleteRowClosureV2Config,
    outcome_config: &OutcomeLinkageV3Config,
    requirement_report: &FutureWindowRequirementReport,
) -> Result<FutureWindowExtensionPlan, String> {
    let temp_path = config
        .output_dir()
        .join("future_window_requirement_report.json");
    fs::create_dir_all(config.output_dir()).map_err(|err| err.to_string())?;
    fs::write(&temp_path, requirement_report.to_json_string()?).map_err(|err| err.to_string())?;
    let extension_config = OfficialFutureWindowExtensionConfig {
        extension_id: format!("{}-extension", config.closure_id),
        future_window_requirement_path: Some(temp_path.display().to_string()),
        official_canonical_csv_paths: outcome_config.extended_candle_pack_paths.clone(),
        output_root: config.output_root.clone(),
        reason_codes: config.reason_codes.clone(),
        ..OfficialFutureWindowExtensionConfig::default()
    };
    build_official_future_window_extension_plan(&extension_config)
}

fn build_complete_bundle(
    config: &CompleteRowClosureV2Config,
    base_rows: &[ComparableCommitteeEvidenceRow],
    scenario_report: &ScenarioMaterializationV3Report,
    outcome_report: &OutcomeLinkageV3Report,
    baseline_plan: &BaselineReferenceBackfillPlan,
    counterfactual_report: &CounterfactualCompletionV2Report,
) -> CompleteComparableRowBundle {
    let outcome_plan = synthesize_outcome_plan(outcome_report);
    let counterfactual_plan = synthesize_counterfactual_plan(counterfactual_report);
    let builder_config = CompleteComparableRowBuilderConfig {
        bundle_id: config.closure_id.clone(),
        allow_diagnostic_complete: true,
        allow_controlled_diagnostic: true,
        allow_crypto_only: true,
        allow_yfinance_research: true,
        allow_fixture: true,
        reason_codes: config.reason_codes.clone(),
    };
    CompleteComparableRowBuilder::default().build(
        &builder_config,
        base_rows,
        scenario_report,
        &outcome_plan,
        baseline_plan,
        &counterfactual_plan,
    )
}

fn synthesize_outcome_plan(report: &OutcomeLinkageV3Report) -> OutcomeReferenceBackfillPlan {
    let items = report
        .records
        .iter()
        .filter(|record| record.outcome_reference.is_some())
        .map(|record| OutcomeReferenceBackfillPlanItem {
            row_id: record.row_id.clone(),
            gap_kind: OutcomeBackfillGapKind::MissingTripleBarrierOutcome,
            can_build_from_candles: true,
            required_horizon_bars: record
                .outcome_reference
                .as_ref()
                .map(|reference| reference.horizon_bars)
                .unwrap_or_default(),
            required_future_window: record
                .outcome_reference
                .as_ref()
                .map(|reference| reference.horizon_bars.saturating_add(1))
                .unwrap_or(0),
            suggested_action: OutcomeBackfillSuggestedAction::BuildTripleBarrierOutcome,
            reason_codes: record.reason_codes.clone(),
        })
        .collect::<Vec<_>>();
    OutcomeReferenceBackfillPlan {
        plan_id: format!("{}-outcome-plan", report.linkage_id),
        buildable_count: items.len(),
        unavailable_count: 0,
        missing_future_window_count: report.skipped_missing_future_bars,
        no_lookahead_blocked_count: report.rejected_no_lookahead,
        items,
        reason_codes: stable_reason_codes(&[
            ReasonCode::CommitteeOutcomeReferenceBuilt,
            ReasonCode::DeterministicPath,
        ]),
    }
}

fn synthesize_counterfactual_plan(
    report: &CounterfactualCompletionV2Report,
) -> CounterfactualBackfillPlan {
    let items = report
        .records
        .iter()
        .map(|record| CounterfactualBackfillPlanItem {
            row_id: record.row_id.clone(),
            gap_kind: if record.risk_denied_counterfactual_built {
                CounterfactualBackfillGapKind::MissingRiskDeniedCounterfactual
            } else {
                CounterfactualBackfillGapKind::MissingNoTradeCounterfactual
            },
            can_build_no_trade: record.no_trade_counterfactual_built,
            can_build_risk_denied: record.risk_denied_counterfactual_built,
            suggested_action: if record.risk_denied_counterfactual_built {
                CounterfactualBackfillSuggestedAction::BuildRiskDeniedCounterfactual
            } else if record.no_trade_counterfactual_built {
                CounterfactualBackfillSuggestedAction::BuildNoTradeCounterfactual
            } else {
                CounterfactualBackfillSuggestedAction::NoSafeAction
            },
            reason_codes: record.reason_codes.clone(),
        })
        .collect::<Vec<_>>();
    CounterfactualBackfillPlan {
        plan_id: format!("{}-counterfactual-plan", report.completion_id),
        no_trade_buildable_count: items.iter().filter(|item| item.can_build_no_trade).count(),
        risk_denied_buildable_count: items
            .iter()
            .filter(|item| item.can_build_risk_denied)
            .count(),
        unavailable_count: items
            .iter()
            .filter(|item| !item.can_build_no_trade && !item.can_build_risk_denied)
            .count(),
        no_lookahead_blocked_count: report
            .records
            .iter()
            .filter(|record| {
                record.status == CounterfactualCompletionV2RecordStatus::RejectedNoLookahead
            })
            .count(),
        items,
        reason_codes: stable_reason_codes(&[
            ReasonCode::CounterfactualEvaluated,
            ReasonCode::DeterministicPath,
        ]),
    }
}

fn build_closure_report(
    config: &CompleteRowClosureV2Config,
    requirement_report: &FutureWindowRequirementReport,
    outcome_report: &OutcomeLinkageV3Report,
    counterfactual_report: &CounterfactualCompletionV2Report,
    complete_bundle: &CompleteComparableRowBundle,
    baseline_plan: &BaselineReferenceBackfillPlan,
    before_complete_rows: usize,
    before_official_complete_rows: usize,
    previous_bottleneck: Option<CoreBottleneckKind>,
    current_bottleneck: Option<CoreBottleneckKind>,
    bottleneck_changed: bool,
) -> CompleteRowClosureV2Report {
    let after_complete_rows = complete_bundle.complete_rows;
    let after_official_complete_rows = complete_bundle.official_complete_rows;
    let added_complete_rows = after_complete_rows as isize - before_complete_rows as isize;
    let added_official_complete_rows =
        after_official_complete_rows as isize - before_official_complete_rows as isize;
    let added_outcome_references = outcome_report.official_outcome_count as isize;
    let added_baseline_references = baseline_plan
        .items
        .iter()
        .filter(|item| item.can_backfill)
        .count() as isize;
    let added_no_trade_counterfactuals = counterfactual_report.no_trade_built_count as isize;
    let added_risk_denied_counterfactuals = counterfactual_report.risk_denied_built_count as isize;
    let closure_status = determine_closure_status(
        added_complete_rows,
        added_official_complete_rows,
        added_outcome_references,
        added_no_trade_counterfactuals,
        added_risk_denied_counterfactuals,
        requirement_report,
        current_bottleneck,
        complete_bundle,
        bottleneck_changed,
    );
    let final_recommendation = determine_recommendation(
        closure_status,
        requirement_report,
        current_bottleneck,
        after_official_complete_rows,
        config.run_core_scorecard_rerun,
    );
    CompleteRowClosureV2Report {
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
    added_no_trade_counterfactuals: isize,
    added_risk_denied_counterfactuals: isize,
    requirement_report: &FutureWindowRequirementReport,
    current_bottleneck: Option<CoreBottleneckKind>,
    complete_bundle: &CompleteComparableRowBundle,
    bottleneck_changed: bool,
) -> CompleteRowClosureV2Status {
    if added_official_complete_rows > 0 {
        return CompleteRowClosureV2Status::OfficialCompleteRowsImproved;
    }
    if added_complete_rows > 0 {
        return CompleteRowClosureV2Status::CompleteRowsImproved;
    }
    if added_outcome_references > 0 {
        return CompleteRowClosureV2Status::OutcomeReferencesImproved;
    }
    if added_no_trade_counterfactuals > 0 || added_risk_denied_counterfactuals > 0 {
        return CompleteRowClosureV2Status::CounterfactualsImproved;
    }
    if bottleneck_changed {
        return CompleteRowClosureV2Status::BottleneckMoved;
    }
    if requirement_report.rows_missing_future_window > 0 {
        return CompleteRowClosureV2Status::StillNeedFutureWindow;
    }
    if complete_bundle
        .build_records
        .iter()
        .any(|record| record.status == CompleteComparableRowBuildStatus::SkippedMissingOutcome)
    {
        return CompleteRowClosureV2Status::StillNeedOutcomeReferences;
    }
    if complete_bundle.build_records.iter().any(|record| {
        record.status == CompleteComparableRowBuildStatus::SkippedMissingCounterfactuals
    }) {
        return CompleteRowClosureV2Status::StillNeedCounterfactuals;
    }
    if current_bottleneck == Some(CoreBottleneckKind::EvidenceTooWeak) {
        return CompleteRowClosureV2Status::StillEvidenceTooWeak;
    }
    if complete_bundle.official_complete_rows == 0 {
        return CompleteRowClosureV2Status::StillNeedMoreOfficialRows;
    }
    CompleteRowClosureV2Status::NoImprovement
}

fn determine_recommendation(
    closure_status: CompleteRowClosureV2Status,
    requirement_report: &FutureWindowRequirementReport,
    current_bottleneck: Option<CoreBottleneckKind>,
    after_official_complete_rows: usize,
    reran_core_scorecard: bool,
) -> CompleteRowClosureV2Recommendation {
    match closure_status {
        CompleteRowClosureV2Status::StillNeedFutureWindow => {
            if requirement_report.rows_extendable_from_local_csv > 0 {
                CompleteRowClosureV2Recommendation::ExtendOfficialCandleWindow
            } else {
                CompleteRowClosureV2Recommendation::ProvideLongerCandleWindow
            }
        }
        CompleteRowClosureV2Status::StillNeedOutcomeReferences => {
            CompleteRowClosureV2Recommendation::ImproveOutcomeLinkingFirst
        }
        CompleteRowClosureV2Status::StillNeedCounterfactuals => {
            CompleteRowClosureV2Recommendation::ImproveCounterfactualDepthFirst
        }
        _ if reran_core_scorecard && after_official_complete_rows > 0 => {
            CompleteRowClosureV2Recommendation::RerunCorePerformance
        }
        _ if current_bottleneck == Some(CoreBottleneckKind::RiskOverBlocking)
            || current_bottleneck == Some(CoreBottleneckKind::RiskUnderBlocking) =>
        {
            CompleteRowClosureV2Recommendation::ImproveRiskGovernorFirst
        }
        _ if current_bottleneck == Some(CoreBottleneckKind::ChairNeedsTuning) => {
            CompleteRowClosureV2Recommendation::ImproveChairFirst
        }
        _ if after_official_complete_rows > 0 => {
            CompleteRowClosureV2Recommendation::CommitteeCoreReadyForDeeperEvidence
        }
        _ if requirement_report.total_items > 0 => CompleteRowClosureV2Recommendation::KeepTrinity,
        _ => CompleteRowClosureV2Recommendation::MoreOfficialEvidence,
    }
}

fn load_base_rows(
    config: &CompleteRowClosureV2Config,
) -> Result<Vec<ComparableCommitteeEvidenceRow>, String> {
    let Some(path) = config.complete_row_closure_config_path.as_deref() else {
        return Err(
            "complete row closure v2 requires complete_row_closure_config_path".to_string(),
        );
    };
    if path.ends_with(".json") {
        if let Ok(bundle) = CompleteRowClosureBundle::from_json_path(Path::new(path)) {
            return Ok(bundle.complete_comparable_row_bundle.rows);
        }
        if let Ok(bundle) = ComparableCommitteeEvidenceBundle::from_json_path(Path::new(path)) {
            return Ok(bundle.rows);
        }
    }
    let closure_config = CompleteRowClosureConfig::from_toml_path(Path::new(path))?;
    let bundle = CompleteRowClosureRunner::default().run(&closure_config)?;
    Ok(bundle.complete_comparable_row_bundle.rows)
}

fn load_outcome_config(
    config: &CompleteRowClosureV2Config,
) -> Result<OutcomeLinkageV3Config, String> {
    if let Some(path) = config.outcome_linkage_v3_config_path.as_deref() {
        OutcomeLinkageV3Config::from_toml_path(Path::new(path))
    } else {
        Ok(OutcomeLinkageV3Config::default())
    }
}

fn load_counterfactual_config(
    config: &CompleteRowClosureV2Config,
) -> Result<CounterfactualCompletionV2Config, String> {
    if let Some(path) = config.counterfactual_completion_v2_config_path.as_deref() {
        CounterfactualCompletionV2Config::from_toml_path(Path::new(path))
    } else {
        Ok(CounterfactualCompletionV2Config::default())
    }
}

fn load_or_build_inventory(
    config: &CompleteRowClosureV2Config,
    outcome_config: &OutcomeLinkageV3Config,
    rows: &[ComparableCommitteeEvidenceRow],
) -> Result<OfficialReadyRowInventoryReport, String> {
    if let Some(path) = outcome_config.official_ready_inventory_path.as_deref() {
        if path.ends_with(".json") {
            return OfficialReadyRowInventoryReport::from_json_path(Path::new(path));
        }
        let inventory_config = OfficialReadyRowInventoryConfig::from_toml_path(Path::new(path))?;
        return OfficialReadyRowInventoryRunner::default().run(&inventory_config);
    }
    let descriptors = load_descriptor_map_from_paths(&outcome_config.extended_candle_pack_paths)?;
    let inventory_config = OfficialReadyRowInventoryConfig {
        inventory_id: format!("{}-inventory", config.closure_id),
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
        &BTreeMap::new(),
        &descriptors,
    )
}

fn load_or_build_materialization(
    rows: &[ComparableCommitteeEvidenceRow],
    inventory: &OfficialReadyRowInventoryReport,
) -> ScenarioMaterializationV3Report {
    let mut records = rows
        .iter()
        .filter(|row| inventory.items.iter().any(|item| item.row_id == row.row_id))
        .map(|row| ScenarioMaterializationV3Record {
            row_id: row.row_id.clone(),
            scenario_row_id: row
                .scenario_row_id
                .clone()
                .unwrap_or_else(|| format!("scenario-{}", row.row_id)),
            materialization_level: if row.scenario_row_id.is_some() || row.row_level {
                ScenarioMaterializationV3Level::ExistingRowLevelScenario
            } else {
                ScenarioMaterializationV3Level::Rejected
            },
            official_ready_match_used: row.candle_official_ready_match,
            candle_series_id: row.matched_candle_series_id.clone(),
            feature_summary_available: true,
            limited_feature_summary: row.summary_derived,
            source_class: row.source_class,
            diagnostic_only: row.diagnostic_only,
            reason_codes: row.reason_codes.clone(),
        })
        .collect::<Vec<_>>();
    records.sort_by(|left, right| left.row_id.cmp(&right.row_id));
    let materialized_count = records
        .iter()
        .filter(|record| record.materialization_level != ScenarioMaterializationV3Level::Rejected)
        .count();
    ScenarioMaterializationV3Report {
        materialized_count,
        row_level_count: records
            .iter()
            .filter(|record| {
                record.materialization_level
                    == ScenarioMaterializationV3Level::ExistingRowLevelScenario
            })
            .count(),
        limited_feature_count: records
            .iter()
            .filter(|record| record.limited_feature_summary)
            .count(),
        rejected_count: records
            .iter()
            .filter(|record| {
                record.materialization_level == ScenarioMaterializationV3Level::Rejected
            })
            .count(),
        official_materialized_count: records
            .iter()
            .filter(|record| {
                record.materialization_level != ScenarioMaterializationV3Level::Rejected
                    && record.source_class == ComparableEvidenceSourceClass::OfficialNonCrypto
            })
            .count(),
        diagnostic_only_count: records
            .iter()
            .filter(|record| record.diagnostic_only)
            .count(),
        materialization_status: if materialized_count > 0 {
            ScenarioMaterializationV3Status::OfficialRowLevelMaterialized
        } else {
            ScenarioMaterializationV3Status::StillMissingScenarioRows
        },
        records,
        reason_codes: stable_reason_codes(&[
            ReasonCode::CommitteeRowLevelMaterialized,
            ReasonCode::DeterministicPath,
        ]),
    }
}

fn infer_bottleneck_from_rows(
    rows: &[ComparableCommitteeEvidenceRow],
) -> Option<CoreBottleneckKind> {
    if rows
        .iter()
        .any(|row| row.candle_official_ready_match && !row.outcome_reference_available)
    {
        return Some(CoreBottleneckKind::MissingOutcomeLinks);
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
    Some(CoreBottleneckKind::EvidenceTooWeak)
}

fn infer_bottleneck_from_complete_bundle(
    bundle: &CompleteComparableRowBundle,
) -> Option<CoreBottleneckKind> {
    if bundle
        .build_records
        .iter()
        .any(|record| record.status == CompleteComparableRowBuildStatus::SkippedMissingOutcome)
    {
        return Some(CoreBottleneckKind::MissingOutcomeLinks);
    }
    if bundle.build_records.iter().any(|record| {
        record.status == CompleteComparableRowBuildStatus::SkippedMissingCounterfactuals
    }) {
        return Some(CoreBottleneckKind::MissingNoTradeCounterfactuals);
    }
    if bundle.official_complete_rows == 0 {
        return Some(CoreBottleneckKind::EvidenceTooWeak);
    }
    Some(CoreBottleneckKind::NoBottleneckDetected)
}

fn status_from_bottleneck(bottleneck: Option<CoreBottleneckKind>) -> CorePerformanceFinalStatus {
    match bottleneck.unwrap_or(CoreBottleneckKind::EvidenceTooWeak) {
        CoreBottleneckKind::MissingOutcomeLinks => {
            CorePerformanceFinalStatus::CoreBlockedByOutcomeLinks
        }
        CoreBottleneckKind::MissingOfficialAuth
        | CoreBottleneckKind::MissingOfficialData
        | CoreBottleneckKind::MissingOfficialCandles => {
            CorePerformanceFinalStatus::CoreBlockedByOfficialData
        }
        CoreBottleneckKind::RiskOverBlocking | CoreBottleneckKind::RiskUnderBlocking => {
            CorePerformanceFinalStatus::CoreBlockedByRiskBehavior
        }
        CoreBottleneckKind::StorageBudgetExceeded | CoreBottleneckKind::LatencyBudgetExceeded => {
            CorePerformanceFinalStatus::CoreBlockedByBudget
        }
        CoreBottleneckKind::NoBottleneckDetected => {
            CorePerformanceFinalStatus::CorePerformanceHealthyForResearch
        }
        _ => CorePerformanceFinalStatus::CoreBlockedByEvidence,
    }
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

fn default_output_root() -> String {
    "target/soma_complete_row_closure_v2".to_string()
}

fn default_true() -> bool {
    true
}
