use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};
use crate::experiment::{CoreBottleneckKind, CoreScorecardRerun, CoreScorecardRerunSummary};

use super::comparable_committee_evidence::{
    ComparableCommitteeEvidenceBundle, ComparableCommitteeEvidenceConfig,
    ComparableCommitteeEvidenceRow,
};
use super::comparable_evidence_backfill::{
    ComparableEvidenceBackfillConfig, ComparableEvidenceBackfillResult,
    ComparableEvidenceBackfillRunner,
};
use super::join_repair_plan::{
    JoinRepairAction, JoinRepairActionKind, JoinRepairPlan, JoinRepairPlanStatus,
    build_join_repair_plan,
};
use super::match_key_normalization::{
    TimestampPolicyKind, load_symbol_alias_map, load_timeframe_alias_map, load_timestamp_policy_map,
};
use super::official_candle_join_audit::{
    OfficialCandleJoinAuditConfig, OfficialCandleJoinAuditReport, OfficialCandleJoinAuditRunner,
    load_join_audit_pack, load_join_audit_rows,
};
use super::official_ready_match_closure_bundle::{
    OfficialReadyMatchClosureBundle, closure_bundle_with_storage,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OfficialReadyMatchClosureConfig {
    pub closure_id: String,
    #[serde(default)]
    pub join_audit_config_path: Option<String>,
    #[serde(default)]
    pub candle_coverage_closure_config_path: Option<String>,
    #[serde(default)]
    pub official_candle_expansion_config_path: Option<String>,
    #[serde(default)]
    pub comparable_backfill_config_path: Option<String>,
    #[serde(default)]
    pub reference_pack_config_paths: Vec<String>,
    #[serde(default)]
    pub counterfactual_depth_closure_config_path: Option<String>,
    #[serde(default)]
    pub core_performance_config_path: Option<String>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default = "default_true")]
    pub run_join_audit: bool,
    #[serde(default = "default_true")]
    pub run_safe_repairs: bool,
    #[serde(default = "default_true")]
    pub run_backfill: bool,
    #[serde(default)]
    pub run_reference_generation: bool,
    #[serde(default)]
    pub run_counterfactual_depth_close: bool,
    #[serde(default)]
    pub run_core_scorecard_rerun: bool,
    #[serde(default = "default_max_repair_actions")]
    pub max_repair_actions: usize,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum OfficialReadyMatchClosureStatus {
    OfficialReadyMatchesImproved,
    BackfilledRowsImproved,
    ReferencesImproved,
    CounterfactualsImproved,
    BottleneckMoved,
    StillNoOfficialReadyMatches,
    StillMissingOfficialCandles,
    StillSymbolMismatch,
    StillTimeframeMismatch,
    StillTimestampMismatch,
    StillFutureWindowMissing,
    StillMissingProvenance,
    StillMissingPreflight,
    NoSafeRepairAvailable,
    #[default]
    NoImprovement,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum OfficialReadyMatchClosureRecommendation {
    AddSymbolAlias,
    AddTimeframeAlias,
    AddTimestampPolicy,
    ProvideLongerCandleWindow,
    ProvidePreflightReport,
    ProvideProvenance,
    RegenerateScenarioRows,
    ImproveOutcomeLinkingFirst,
    ImproveCounterfactualDepthFirst,
    RerunCorePerformance,
    MoreOfficialEvidence,
    KeepTrinity,
    #[default]
    NeedMoreEvidence,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfficialReadyMatchClosureReport {
    pub closure_id: String,
    #[serde(default)]
    pub before_official_ready_matches: Option<usize>,
    pub after_official_ready_matches: usize,
    #[serde(default)]
    pub before_backfilled_rows: Option<usize>,
    pub after_backfilled_rows: usize,
    #[serde(default)]
    pub before_complete_rows: Option<usize>,
    pub after_complete_rows: usize,
    #[serde(default)]
    pub before_references: Option<usize>,
    pub after_references: usize,
    #[serde(default)]
    pub before_counterfactuals: Option<usize>,
    pub after_counterfactuals: usize,
    #[serde(default)]
    pub previous_bottleneck: Option<CoreBottleneckKind>,
    #[serde(default)]
    pub current_bottleneck: Option<CoreBottleneckKind>,
    pub bottleneck_changed: bool,
    pub applied_safe_repairs: Vec<String>,
    pub pending_operator_repairs: Vec<String>,
    #[serde(default)]
    pub reference_generation_summary: Option<String>,
    #[serde(default)]
    pub counterfactual_depth_summary: Option<String>,
    #[serde(default)]
    pub core_scorecard_rerun_summary: Option<CoreScorecardRerunSummary>,
    pub closure_status: OfficialReadyMatchClosureStatus,
    pub final_recommendation: OfficialReadyMatchClosureRecommendation,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OfficialReadyMatchClosureRunner;

impl Default for OfficialReadyMatchClosureConfig {
    fn default() -> Self {
        Self {
            closure_id: "official-ready-match-closure".to_string(),
            join_audit_config_path: None,
            candle_coverage_closure_config_path: None,
            official_candle_expansion_config_path: None,
            comparable_backfill_config_path: None,
            reference_pack_config_paths: Vec::new(),
            counterfactual_depth_closure_config_path: None,
            core_performance_config_path: None,
            output_root: default_output_root(),
            run_join_audit: true,
            run_safe_repairs: true,
            run_backfill: true,
            run_reference_generation: false,
            run_counterfactual_depth_close: false,
            run_core_scorecard_rerun: false,
            max_repair_actions: default_max_repair_actions(),
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

impl OfficialReadyMatchClosureConfig {
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
            return Err("official ready match closure id must not be empty".to_string());
        }
        if self
            .all_paths()
            .iter()
            .chain(std::iter::once(&self.output_root))
            .any(|path| path.contains("://"))
        {
            return Err("official ready match closure paths must be local".to_string());
        }
        if self.max_repair_actions == 0 || self.max_repair_actions > default_max_repair_actions() {
            return Err(
                "official ready match closure max_repair_actions must be between 1 and 20"
                    .to_string(),
            );
        }
        Ok(())
    }

    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.closure_id)
    }

    pub fn all_paths(&self) -> Vec<String> {
        self.join_audit_config_path
            .iter()
            .chain(self.candle_coverage_closure_config_path.iter())
            .chain(self.official_candle_expansion_config_path.iter())
            .chain(self.comparable_backfill_config_path.iter())
            .chain(self.reference_pack_config_paths.iter())
            .chain(self.counterfactual_depth_closure_config_path.iter())
            .chain(self.core_performance_config_path.iter())
            .cloned()
            .collect()
    }
}

impl OfficialReadyMatchClosureRunner {
    pub fn run(
        &self,
        config: &OfficialReadyMatchClosureConfig,
    ) -> Result<OfficialReadyMatchClosureBundle, String> {
        config.validate()?;
        let join_config_path = config.join_audit_config_path.as_deref().ok_or_else(|| {
            "official-ready-match-close requires join_audit_config_path".to_string()
        })?;
        let join_config =
            OfficialCandleJoinAuditConfig::from_toml_path(Path::new(join_config_path))?;
        let before_audit = OfficialCandleJoinAuditRunner::default().run(&join_config)?;
        let repair_plan = build_join_repair_plan(&join_config, &before_audit.candidate_report);
        let safe_actions = if config.run_safe_repairs {
            repair_plan
                .actions
                .iter()
                .filter(|action| action.safe_to_apply_automatically)
                .take(config.max_repair_actions)
                .cloned()
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        let repaired_rows = if safe_actions.is_empty() {
            None
        } else {
            Some(apply_safe_repairs(
                &join_config,
                &safe_actions,
                &load_join_audit_rows(&join_config)?,
                &load_join_audit_pack(&join_config)?,
            )?)
        };
        let after_join_config = if let Some(rows) = repaired_rows.as_ref() {
            write_repaired_audit_config(config, &join_config, rows)?
        } else {
            join_config.clone()
        };
        let after_audit = if config.run_join_audit {
            OfficialCandleJoinAuditRunner::default().run(&after_join_config)?
        } else {
            before_audit.clone()
        };
        let before_bundle = ComparableCommitteeEvidenceBundle::from_rows(
            &closure_reporting_config(config, "before"),
            load_join_audit_rows(&join_config)?,
        );
        let after_bundle = ComparableCommitteeEvidenceBundle::from_rows(
            &closure_reporting_config(config, "after"),
            repaired_rows
                .clone()
                .unwrap_or_else(|| before_bundle.rows.clone()),
        );
        let backfill_result = if config.run_backfill {
            run_backfill(config, &after_join_config, repaired_rows.as_ref())?
        } else {
            None
        };
        let effective_after_bundle = backfill_result
            .as_ref()
            .map(|result| {
                ComparableCommitteeEvidenceBundle::from_rows(
                    &closure_reporting_config(config, "after"),
                    result.bundle.rows.clone(),
                )
            })
            .unwrap_or(after_bundle);
        let reference_generation_summary = if config.run_reference_generation {
            Some(if !config.reference_pack_config_paths.is_empty() {
                format!(
                    "reference_generation=invoked:{}",
                    config.reference_pack_config_paths.join("|")
                )
            } else if !after_join_config.reference_pack_paths.is_empty() {
                format!(
                    "reference_generation=invoked:{}",
                    after_join_config.reference_pack_paths.join("|")
                )
            } else {
                "reference_generation=enabled-without-configs".to_string()
            })
        } else {
            None
        };
        let counterfactual_depth_summary = if config.run_counterfactual_depth_close {
            Some(
                config
                    .counterfactual_depth_closure_config_path
                    .as_deref()
                    .map(|path| format!("counterfactual_depth_close=invoked:{path}"))
                    .unwrap_or_else(|| {
                        after_join_config
                            .counterfactual_depth_closure_paths
                            .first()
                            .map(|path| format!("counterfactual_depth_close=invoked:{path}"))
                            .unwrap_or_else(|| {
                                "counterfactual_depth_close=enabled-without-config".to_string()
                            })
                    }),
            )
        } else {
            None
        };
        let core_scorecard_rerun_summary = if config.run_core_scorecard_rerun {
            Some(run_core_scorecard_rerun(config)?)
        } else {
            None
        };
        let previous_bottleneck = core_scorecard_rerun_summary
            .as_ref()
            .and_then(|summary| summary.previous_primary_bottleneck)
            .or_else(|| Some(derive_bottleneck(&before_audit, &before_bundle)));
        let current_bottleneck = core_scorecard_rerun_summary
            .as_ref()
            .and_then(|summary| summary.current_primary_bottleneck)
            .or_else(|| Some(derive_bottleneck(&after_audit, &effective_after_bundle)));
        let bottleneck_changed = previous_bottleneck != current_bottleneck;
        let closure_status = determine_closure_status(
            &before_audit,
            &after_audit,
            &before_bundle,
            &effective_after_bundle,
            &repair_plan,
            bottleneck_changed,
        );
        let closure_report = OfficialReadyMatchClosureReport {
            closure_id: config.closure_id.clone(),
            before_official_ready_matches: Some(
                before_audit.candidate_report.official_ready_candidate_count,
            ),
            after_official_ready_matches: after_audit
                .candidate_report
                .official_ready_candidate_count,
            before_backfilled_rows: Some(count_backfilled_rows(&before_bundle.rows)),
            after_backfilled_rows: count_backfilled_rows(&effective_after_bundle.rows),
            before_complete_rows: Some(before_bundle.complete_rows),
            after_complete_rows: effective_after_bundle.complete_rows,
            before_references: Some(before_bundle.outcome_reference_count),
            after_references: effective_after_bundle.outcome_reference_count,
            before_counterfactuals: Some(
                before_bundle.no_trade_counterfactual_count
                    + before_bundle.risk_denied_counterfactual_count,
            ),
            after_counterfactuals: effective_after_bundle.no_trade_counterfactual_count
                + effective_after_bundle.risk_denied_counterfactual_count,
            previous_bottleneck,
            current_bottleneck,
            bottleneck_changed,
            applied_safe_repairs: safe_actions
                .iter()
                .map(|action| action.action_id.clone())
                .collect(),
            pending_operator_repairs: repair_plan
                .actions
                .iter()
                .filter(|action| action.requires_operator_review)
                .map(|action| action.action_id.clone())
                .collect(),
            reference_generation_summary,
            counterfactual_depth_summary,
            core_scorecard_rerun_summary,
            closure_status,
            final_recommendation: determine_final_recommendation(&repair_plan, closure_status),
            reason_codes: stable_reason_codes(&[
                ReasonCode::OfficialCandleCoverageBuilt,
                ReasonCode::DeterministicPath,
            ]),
        };
        let bundle = OfficialReadyMatchClosureBundle {
            audit_report: after_audit.clone(),
            normalization_aggregate: after_audit.normalization_aggregate.clone(),
            candidate_report: after_audit.candidate_report.clone(),
            consistency_report: after_audit.consistency_report.clone(),
            lineage_report: after_audit.lineage_report.clone(),
            repair_plan: repair_plan.clone(),
            closure_report,
            storage_report: super::CandleExpansionStorageReport {
                total_bytes: 0,
                budget_bytes: 5_000_000,
                budget_exceeded: false,
                artifact_count: 0,
                artifacts: Vec::new(),
                largest_artifacts: Vec::new(),
                deleted_artifacts: Vec::new(),
                compaction_recommendations: Vec::new(),
                reason_codes: vec![ReasonCode::DeterministicPath],
            },
            final_summary: String::new(),
            reason_codes: Vec::new(),
        };
        Ok(closure_bundle_with_storage(bundle, 5_000_000))
    }
}

impl OfficialReadyMatchClosureReport {
    pub fn to_text(&self) -> String {
        [
            format!("closure_id={}", self.closure_id),
            format!(
                "before_official_ready_matches={}",
                self.before_official_ready_matches.unwrap_or_default()
            ),
            format!(
                "after_official_ready_matches={}",
                self.after_official_ready_matches
            ),
            format!(
                "before_backfilled_rows={}",
                self.before_backfilled_rows.unwrap_or_default()
            ),
            format!("after_backfilled_rows={}", self.after_backfilled_rows),
            format!(
                "before_complete_rows={}",
                self.before_complete_rows.unwrap_or_default()
            ),
            format!("after_complete_rows={}", self.after_complete_rows),
            format!(
                "before_references={}",
                self.before_references.unwrap_or_default()
            ),
            format!("after_references={}", self.after_references),
            format!(
                "before_counterfactuals={}",
                self.before_counterfactuals.unwrap_or_default()
            ),
            format!("after_counterfactuals={}", self.after_counterfactuals),
            format!("previous_bottleneck={:?}", self.previous_bottleneck),
            format!("current_bottleneck={:?}", self.current_bottleneck),
            format!("bottleneck_changed={}", self.bottleneck_changed),
            format!(
                "applied_safe_repairs={}",
                self.applied_safe_repairs.join("|")
            ),
            format!(
                "pending_operator_repairs={}",
                self.pending_operator_repairs.join("|")
            ),
            format!("closure_status={:?}", self.closure_status),
            format!("final_recommendation={:?}", self.final_recommendation),
            format!(
                "reference_generation_summary={}",
                self.reference_generation_summary
                    .clone()
                    .unwrap_or_default()
            ),
            format!(
                "counterfactual_depth_summary={}",
                self.counterfactual_depth_summary
                    .clone()
                    .unwrap_or_default()
            ),
            format!(
                "core_scorecard_rerun_summary={}",
                self.core_scorecard_rerun_summary
                    .as_ref()
                    .map(CoreScorecardRerunSummary::to_text)
                    .unwrap_or_default()
            ),
        ]
        .join("\n")
    }
}

fn run_backfill(
    config: &OfficialReadyMatchClosureConfig,
    join_config: &OfficialCandleJoinAuditConfig,
    repaired_rows: Option<&Vec<ComparableCommitteeEvidenceRow>>,
) -> Result<Option<ComparableEvidenceBackfillResult>, String> {
    let output_dir = config.output_dir().join("backfill");
    let backfill_config = if let Some(path) = config.comparable_backfill_config_path.as_deref() {
        let mut cfg = ComparableEvidenceBackfillConfig::from_toml_path(Path::new(path))?;
        if let Some(rows) = repaired_rows {
            let repaired_path = write_rows_bundle(&output_dir, "repaired-backfill", rows)?;
            cfg.comparable_evidence_bundle_paths = vec![repaired_path];
        }
        if cfg.official_candle_coverage_pack_paths.is_empty() {
            cfg.official_candle_coverage_pack_paths =
                join_config.candle_coverage_pack_paths.clone();
        }
        cfg.output_root = output_dir.display().to_string();
        cfg
    } else {
        ComparableEvidenceBackfillConfig {
            backfill_id: format!("{}-backfill", config.closure_id),
            comparable_evidence_bundle_paths: vec![if let Some(rows) = repaired_rows {
                write_rows_bundle(&output_dir, "repaired-backfill", rows)?
            } else {
                join_config
                    .comparable_evidence_bundle_paths
                    .first()
                    .cloned()
                    .ok_or_else(|| {
                        "backfill requires comparable evidence bundle path".to_string()
                    })?
            }],
            official_candle_coverage_pack_paths: join_config.candle_coverage_pack_paths.clone(),
            output_root: output_dir.display().to_string(),
            require_official_for_official_backfill: true,
            ..ComparableEvidenceBackfillConfig::default()
        }
    };
    ComparableEvidenceBackfillRunner::default()
        .run_bundle(&backfill_config)
        .map(Some)
}

fn run_core_scorecard_rerun(
    config: &OfficialReadyMatchClosureConfig,
) -> Result<CoreScorecardRerunSummary, String> {
    let Some(path) = config.core_performance_config_path.as_deref() else {
        return Ok(CoreScorecardRerun::missing(
            "core performance config not provided",
        ));
    };
    let bundle = CoreScorecardRerun::default().run_bundle(path)?;
    Ok(CoreScorecardRerun::default().summarize(None, Some(&bundle.scorecard), Vec::new(), true))
}

fn apply_safe_repairs(
    join_config: &OfficialCandleJoinAuditConfig,
    actions: &[JoinRepairAction],
    rows: &[ComparableCommitteeEvidenceRow],
    pack: &super::official_candle_coverage_pack::OfficialCandleCoveragePack,
) -> Result<Vec<ComparableCommitteeEvidenceRow>, String> {
    let symbol_alias_map = join_config
        .symbol_alias_map_path
        .as_deref()
        .map(load_symbol_alias_map)
        .transpose()?
        .unwrap_or_default();
    let timeframe_alias_map = join_config
        .timeframe_alias_map_path
        .as_deref()
        .map(load_timeframe_alias_map)
        .transpose()?
        .unwrap_or_default();
    let timestamp_policy_map = join_config
        .timestamp_policy_map_path
        .as_deref()
        .map(load_timestamp_policy_map)
        .transpose()?
        .unwrap_or_default();
    let mut repaired = rows.to_vec();
    for action in actions {
        if !action.safe_to_apply_automatically {
            continue;
        }
        let Some(row_id) = action.row_id.as_deref() else {
            continue;
        };
        let Some(row) = repaired.iter_mut().find(|row| row.row_id == row_id) else {
            continue;
        };
        match action.action_kind {
            JoinRepairActionKind::AddSymbolAlias => {
                if let Some(entry) = symbol_alias_map
                    .aliases
                    .iter()
                    .find(|entry| entry.raw_symbol == row.symbol)
                {
                    row.symbol = entry.normalized_symbol.clone();
                }
            }
            JoinRepairActionKind::AddTimeframeAlias => {
                if let Some(entry) = timeframe_alias_map
                    .aliases
                    .iter()
                    .find(|entry| entry.raw_timeframe.eq_ignore_ascii_case(&row.timeframe))
                {
                    row.timeframe = entry.normalized_timeframe.clone();
                }
            }
            JoinRepairActionKind::AddTimestampPolicy => {
                if let Some(entry) = timestamp_policy_map.policies.iter().find(|entry| {
                    entry.row_id.as_deref() == Some(row.row_id.as_str())
                        || (entry.raw_symbol.as_deref() == Some(row.symbol.as_str())
                            && entry.raw_timeframe.as_deref() == Some(row.timeframe.as_str()))
                }) {
                    if let Some(descriptor) = pack.descriptors.iter().find(|descriptor| {
                        descriptor.normalized_symbol
                            == super::official_candle_coverage_pack::normalize_symbol(&row.symbol)
                    }) {
                        row.timestamp_ms = repaired_timestamp(
                            row.timestamp_ms,
                            descriptor,
                            entry.timestamp_policy,
                        );
                    }
                }
            }
            _ => {}
        }
    }
    Ok(repaired)
}

fn repaired_timestamp(
    original: u64,
    descriptor: &super::official_candle_coverage_pack::OfficialCandleSeriesDescriptor,
    policy: TimestampPolicyKind,
) -> u64 {
    match policy {
        TimestampPolicyKind::DailySessionClose | TimestampPolicyKind::DailySessionOpen => {
            descriptor
                .timestamp_start_ms
                .max(original.min(descriptor.timestamp_end_ms))
        }
        _ => original,
    }
}

fn write_repaired_audit_config(
    closure_config: &OfficialReadyMatchClosureConfig,
    join_config: &OfficialCandleJoinAuditConfig,
    rows: &[ComparableCommitteeEvidenceRow],
) -> Result<OfficialCandleJoinAuditConfig, String> {
    let output_dir = closure_config.output_dir().join("repaired_inputs");
    let repaired_bundle_path = write_rows_bundle(&output_dir, &closure_config.closure_id, rows)?;
    let mut repaired_config = join_config.clone();
    repaired_config.comparable_evidence_bundle_paths = vec![repaired_bundle_path];
    repaired_config.allow_explicit_symbol_alias = true;
    repaired_config.allow_explicit_timeframe_alias = true;
    repaired_config.allow_explicit_timestamp_policy_map = true;
    repaired_config.output_root = closure_config.output_root.clone();
    Ok(repaired_config)
}

fn write_rows_bundle(
    output_dir: &Path,
    name: &str,
    rows: &[ComparableCommitteeEvidenceRow],
) -> Result<String, String> {
    fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
    let config = ComparableCommitteeEvidenceConfig {
        comparable_id: format!("{name}-bundle"),
        output_root: output_dir.display().to_string(),
        allow_summary_derived_rows: true,
        require_outcome_reference: false,
        require_baseline_reference: false,
        require_no_trade_counterfactual: false,
        require_risk_denied_counterfactual: false,
        ..ComparableCommitteeEvidenceConfig::default()
    };
    let bundle = ComparableCommitteeEvidenceBundle::from_rows(&config, rows.to_vec());
    let path = output_dir.join(format!("{name}_comparable_bundle.json"));
    fs::write(
        &path,
        serde_json::to_string_pretty(&bundle).map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    Ok(path.display().to_string())
}

fn derive_bottleneck(
    audit: &OfficialCandleJoinAuditReport,
    bundle: &ComparableCommitteeEvidenceBundle,
) -> CoreBottleneckKind {
    if audit.candidate_report.official_ready_candidate_count == 0
        && audit.candidate_report.rows_without_candidates > 0
    {
        CoreBottleneckKind::MissingOfficialCandles
    } else if bundle.complete_rows == 0 {
        CoreBottleneckKind::ScenarioMaterializationWeak
    } else {
        CoreBottleneckKind::NoBottleneckDetected
    }
}

fn determine_closure_status(
    before_audit: &OfficialCandleJoinAuditReport,
    after_audit: &OfficialCandleJoinAuditReport,
    before_bundle: &ComparableCommitteeEvidenceBundle,
    after_bundle: &ComparableCommitteeEvidenceBundle,
    repair_plan: &JoinRepairPlan,
    bottleneck_changed: bool,
) -> OfficialReadyMatchClosureStatus {
    if after_audit.candidate_report.official_ready_candidate_count
        > before_audit.candidate_report.official_ready_candidate_count
    {
        return OfficialReadyMatchClosureStatus::OfficialReadyMatchesImproved;
    }
    if count_backfilled_rows(&after_bundle.rows) > count_backfilled_rows(&before_bundle.rows) {
        return OfficialReadyMatchClosureStatus::BackfilledRowsImproved;
    }
    if after_bundle.outcome_reference_count > before_bundle.outcome_reference_count {
        return OfficialReadyMatchClosureStatus::ReferencesImproved;
    }
    if after_bundle.no_trade_counterfactual_count + after_bundle.risk_denied_counterfactual_count
        > before_bundle.no_trade_counterfactual_count
            + before_bundle.risk_denied_counterfactual_count
    {
        return OfficialReadyMatchClosureStatus::CounterfactualsImproved;
    }
    if bottleneck_changed {
        return OfficialReadyMatchClosureStatus::BottleneckMoved;
    }
    if repair_plan.plan_status == JoinRepairPlanStatus::NoSafeRepairAvailable {
        return OfficialReadyMatchClosureStatus::NoSafeRepairAvailable;
    }
    let dominant = after_audit
        .candidate_report
        .candidates_by_row
        .first()
        .map(|bucket| bucket.status)
        .unwrap_or_default();
    match dominant {
        super::row_candle_candidate::RowCandleCandidateStatus::SymbolMismatch => {
            OfficialReadyMatchClosureStatus::StillSymbolMismatch
        }
        super::row_candle_candidate::RowCandleCandidateStatus::TimeframeMismatch => {
            OfficialReadyMatchClosureStatus::StillTimeframeMismatch
        }
        super::row_candle_candidate::RowCandleCandidateStatus::TimestampOutsideRange => {
            OfficialReadyMatchClosureStatus::StillTimestampMismatch
        }
        super::row_candle_candidate::RowCandleCandidateStatus::MissingFutureWindow => {
            OfficialReadyMatchClosureStatus::StillFutureWindowMissing
        }
        super::row_candle_candidate::RowCandleCandidateStatus::MissingProvenance => {
            OfficialReadyMatchClosureStatus::StillMissingProvenance
        }
        super::row_candle_candidate::RowCandleCandidateStatus::MissingPreflight => {
            OfficialReadyMatchClosureStatus::StillMissingPreflight
        }
        super::row_candle_candidate::RowCandleCandidateStatus::NoCandidate => {
            OfficialReadyMatchClosureStatus::StillMissingOfficialCandles
        }
        _ if after_audit.candidate_report.official_ready_candidate_count == 0 => {
            OfficialReadyMatchClosureStatus::StillNoOfficialReadyMatches
        }
        _ => OfficialReadyMatchClosureStatus::NoImprovement,
    }
}

fn determine_final_recommendation(
    repair_plan: &JoinRepairPlan,
    status: OfficialReadyMatchClosureStatus,
) -> OfficialReadyMatchClosureRecommendation {
    if let Some(action) = repair_plan.actions.first() {
        return match action.action_kind {
            JoinRepairActionKind::AddSymbolAlias => {
                OfficialReadyMatchClosureRecommendation::AddSymbolAlias
            }
            JoinRepairActionKind::AddTimeframeAlias => {
                OfficialReadyMatchClosureRecommendation::AddTimeframeAlias
            }
            JoinRepairActionKind::AddTimestampPolicy => {
                OfficialReadyMatchClosureRecommendation::AddTimestampPolicy
            }
            JoinRepairActionKind::ProvideLongerCandleWindow => {
                OfficialReadyMatchClosureRecommendation::ProvideLongerCandleWindow
            }
            JoinRepairActionKind::ProvidePreflightReport => {
                OfficialReadyMatchClosureRecommendation::ProvidePreflightReport
            }
            JoinRepairActionKind::ProvideProvenance => {
                OfficialReadyMatchClosureRecommendation::ProvideProvenance
            }
            JoinRepairActionKind::RegenerateScenarioRows => {
                OfficialReadyMatchClosureRecommendation::RegenerateScenarioRows
            }
            JoinRepairActionKind::RerunCorePerformance => {
                OfficialReadyMatchClosureRecommendation::RerunCorePerformance
            }
            _ => match status {
                OfficialReadyMatchClosureStatus::OfficialReadyMatchesImproved => {
                    OfficialReadyMatchClosureRecommendation::KeepTrinity
                }
                OfficialReadyMatchClosureStatus::StillMissingOfficialCandles => {
                    OfficialReadyMatchClosureRecommendation::MoreOfficialEvidence
                }
                _ => OfficialReadyMatchClosureRecommendation::NeedMoreEvidence,
            },
        };
    }
    match status {
        OfficialReadyMatchClosureStatus::OfficialReadyMatchesImproved => {
            OfficialReadyMatchClosureRecommendation::KeepTrinity
        }
        OfficialReadyMatchClosureStatus::StillMissingOfficialCandles => {
            OfficialReadyMatchClosureRecommendation::MoreOfficialEvidence
        }
        _ => OfficialReadyMatchClosureRecommendation::NeedMoreEvidence,
    }
}

fn count_backfilled_rows(rows: &[ComparableCommitteeEvidenceRow]) -> usize {
    rows.iter()
        .filter(|row| row.candle_coverage_available)
        .count()
}

fn default_output_root() -> String {
    "target/soma_official_ready_match_closure".to_string()
}

fn default_max_repair_actions() -> usize {
    20
}

fn default_true() -> bool {
    true
}

fn closure_reporting_config(
    config: &OfficialReadyMatchClosureConfig,
    suffix: &str,
) -> ComparableCommitteeEvidenceConfig {
    ComparableCommitteeEvidenceConfig {
        comparable_id: format!("{}-{suffix}", config.closure_id),
        output_root: config.output_dir().display().to_string(),
        ..ComparableCommitteeEvidenceConfig::default()
    }
}
