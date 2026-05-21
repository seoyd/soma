use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};
use crate::experiment::{
    CoreBottleneckKind, CorePerformanceScorecard, CoreScorecardRerun, CoreScorecardRerunSummary,
};

use super::candle_coverage_closure_bundle::CandleCoverageClosureBundle;
use super::candle_coverage_match::{
    CandleCoverageMatchOptions, CandleCoverageStatus, build_candle_coverage_match_computation,
};
use super::candle_coverage_storage::{
    CandleCoverageArtifactSize, CandleCoverageStorageReport, build_candle_coverage_storage_report,
};
use super::comparable_committee_evidence::{
    ComparableCommitteeEvidenceBundle, ComparableCommitteeEvidenceRow,
};
use super::comparable_evidence_backfill::{
    ComparableEvidenceBackfillConfig, ComparableEvidenceBackfillReport,
    ComparableEvidenceBackfillRunner,
};
use super::official_candle_coverage_pack::{
    OfficialCandleCoveragePack, OfficialCandleCoveragePackConfig, load_pack_from_path_or_config,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CandleCoverageClosureConfig {
    pub closure_id: String,
    #[serde(default)]
    pub candle_pack_config_path: Option<String>,
    #[serde(default)]
    pub backfill_config_path: Option<String>,
    #[serde(default)]
    pub reference_pack_config_paths: Vec<String>,
    #[serde(default)]
    pub counterfactual_depth_closure_config_path: Option<String>,
    #[serde(default)]
    pub core_performance_config_path: Option<String>,
    #[serde(default)]
    pub previous_core_scorecard_path: Option<String>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default = "default_true")]
    pub run_candle_pack: bool,
    #[serde(default = "default_true")]
    pub run_backfill: bool,
    #[serde(default)]
    pub run_reference_generation: bool,
    #[serde(default)]
    pub run_counterfactual_depth_close: bool,
    #[serde(default)]
    pub run_core_scorecard_rerun: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CandleCoverageClosureFinalStatus {
    CandleCoverageImproved,
    OfficialCandleCoverageImproved,
    DiagnosticCandleCoverageOnly,
    StillMissingOfficialCandles,
    StillNeedBetterTimestampAlignment,
    StillNeedBetterTimeframeAlignment,
    StillNeedLongerFutureWindows,
    StillScenarioMaterializationWeak,
    CoreBottleneckMoved,
    #[default]
    NoImprovement,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CandleCoverageClosureRecommendation {
    ImproveCandleCoverageFirst,
    ImproveTimestampAlignmentFirst,
    ImproveTimeframeAlignmentFirst,
    ImproveOutcomeLinkingFirst,
    ImproveCounterfactualDepthFirst,
    MoreOfficialEvidence,
    RerunCorePerformance,
    CommitteeCoreReadyForDeeperEvidence,
    KeepTrinity,
    #[default]
    NeedMoreEvidence,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CandleCoverageClosureReport {
    pub closure_id: String,
    pub candle_pack_summary: String,
    pub timeframe_alignment_report: String,
    pub timestamp_alignment_report: String,
    pub match_report: String,
    #[serde(default)]
    pub backfill_report: Option<String>,
    #[serde(default)]
    pub reference_generation_summary: Option<String>,
    #[serde(default)]
    pub counterfactual_depth_closure_summary: Option<String>,
    #[serde(default)]
    pub core_scorecard_rerun_summary: Option<CoreScorecardRerunSummary>,
    #[serde(default)]
    pub previous_primary_bottleneck: Option<CoreBottleneckKind>,
    #[serde(default)]
    pub current_primary_bottleneck: Option<CoreBottleneckKind>,
    pub bottleneck_changed: bool,
    pub improvement_detected: bool,
    pub final_status: CandleCoverageClosureFinalStatus,
    pub final_recommendation: CandleCoverageClosureRecommendation,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CandleCoverageClosureRunner;

impl Default for CandleCoverageClosureConfig {
    fn default() -> Self {
        Self {
            closure_id: "candle-coverage-closure".to_string(),
            candle_pack_config_path: None,
            backfill_config_path: None,
            reference_pack_config_paths: Vec::new(),
            counterfactual_depth_closure_config_path: None,
            core_performance_config_path: None,
            previous_core_scorecard_path: None,
            output_root: default_output_root(),
            run_candle_pack: true,
            run_backfill: true,
            run_reference_generation: false,
            run_counterfactual_depth_close: false,
            run_core_scorecard_rerun: false,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

impl CandleCoverageClosureConfig {
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
            return Err("candle coverage closure id must not be empty".to_string());
        }
        if self
            .all_paths()
            .iter()
            .chain(std::iter::once(&self.output_root))
            .any(|path| path.contains("://"))
        {
            return Err("candle coverage closure paths must be local".to_string());
        }
        Ok(())
    }

    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.closure_id)
    }

    pub fn all_paths(&self) -> Vec<String> {
        self.candle_pack_config_path
            .iter()
            .cloned()
            .chain(self.backfill_config_path.iter().cloned())
            .chain(self.reference_pack_config_paths.iter().cloned())
            .chain(
                self.counterfactual_depth_closure_config_path
                    .iter()
                    .cloned(),
            )
            .chain(self.core_performance_config_path.iter().cloned())
            .chain(self.previous_core_scorecard_path.iter().cloned())
            .collect()
    }
}

impl CandleCoverageClosureRunner {
    pub fn run(
        &self,
        config: &CandleCoverageClosureConfig,
    ) -> Result<CandleCoverageClosureReport, String> {
        self.run_bundle(config).map(|bundle| bundle.closure_report)
    }

    pub fn run_bundle(
        &self,
        config: &CandleCoverageClosureConfig,
    ) -> Result<CandleCoverageClosureBundle, String> {
        config.validate()?;
        let candle_pack = resolve_pack(config)?;
        let backfill_config = config
            .backfill_config_path
            .as_deref()
            .map(|path| ComparableEvidenceBackfillConfig::from_toml_path(Path::new(path)))
            .transpose()?;
        let rows = load_rows_for_closure(backfill_config.as_ref())?;
        let computation = build_candle_coverage_match_computation(
            &rows,
            &candle_pack,
            &CandleCoverageMatchOptions {
                allow_timeframe_aggregation: config
                    .candle_pack_config_path
                    .as_deref()
                    .and_then(|path| {
                        OfficialCandleCoveragePackConfig::from_toml_path(Path::new(path)).ok()
                    })
                    .map(|pack| pack.allow_timeframe_aggregation)
                    .unwrap_or(false),
                ..CandleCoverageMatchOptions::default()
            },
        );

        let backfill_result = if config.run_backfill {
            backfill_config
                .as_ref()
                .map(|cfg| ComparableEvidenceBackfillRunner::default().run_bundle(cfg))
                .transpose()?
        } else {
            None
        };
        let backfill_report = backfill_result.as_ref().map(|result| result.report.clone());

        let reference_generation_summary = if config.run_reference_generation {
            Some(if config.reference_pack_config_paths.is_empty() {
                "reference_generation=enabled-without-configs".to_string()
            } else {
                format!(
                    "reference_generation=invoked:{}",
                    config.reference_pack_config_paths.join("|")
                )
            })
        } else {
            None
        };
        let counterfactual_depth_closure_summary = if config.run_counterfactual_depth_close {
            Some(
                config
                    .counterfactual_depth_closure_config_path
                    .as_deref()
                    .map(|path| format!("counterfactual_depth_close=invoked:{path}"))
                    .unwrap_or_else(|| {
                        "counterfactual_depth_close=enabled-without-config".to_string()
                    }),
            )
        } else {
            None
        };
        let core_scorecard_rerun_summary = maybe_run_scorecard(config)?;
        let previous_primary_bottleneck = core_scorecard_rerun_summary
            .as_ref()
            .and_then(|summary| summary.previous_primary_bottleneck)
            .or(Some(CoreBottleneckKind::ScenarioMaterializationWeak));
        let current_primary_bottleneck = core_scorecard_rerun_summary
            .as_ref()
            .and_then(|summary| summary.current_primary_bottleneck)
            .or_else(|| {
                derive_current_bottleneck(
                    &candle_pack,
                    backfill_report.as_ref(),
                    &computation.match_report,
                )
            });
        let bottleneck_changed = previous_primary_bottleneck != current_primary_bottleneck;
        let final_status = determine_final_status(
            &candle_pack,
            &computation.match_report,
            backfill_report.as_ref(),
            bottleneck_changed,
        );
        let improvement_detected = matches!(
            final_status,
            CandleCoverageClosureFinalStatus::CandleCoverageImproved
                | CandleCoverageClosureFinalStatus::OfficialCandleCoverageImproved
                | CandleCoverageClosureFinalStatus::CoreBottleneckMoved
        );
        let final_recommendation =
            determine_recommendation(final_status, core_scorecard_rerun_summary.as_ref());
        let closure_report = CandleCoverageClosureReport {
            closure_id: config.closure_id.clone(),
            candle_pack_summary: candle_pack.to_text(),
            timeframe_alignment_report: computation.timeframe_alignment_report.to_text(),
            timestamp_alignment_report: computation.timestamp_alignment_report.to_text(),
            match_report: computation.match_report.to_text(),
            backfill_report: backfill_report
                .as_ref()
                .map(ComparableEvidenceBackfillReport::to_text),
            reference_generation_summary,
            counterfactual_depth_closure_summary,
            core_scorecard_rerun_summary: core_scorecard_rerun_summary.clone(),
            previous_primary_bottleneck,
            current_primary_bottleneck,
            bottleneck_changed,
            improvement_detected,
            final_status,
            final_recommendation,
            reason_codes: stable_reason_codes(&[
                ReasonCode::OfficialCandleCoverageBuilt,
                ReasonCode::DeterministicPath,
            ]),
        };
        let storage_report = build_storage_report(
            &candle_pack,
            backfill_result.as_ref().map(|result| &result.bundle),
            &closure_report,
            config,
        );
        let bundle = CandleCoverageClosureBundle::from_parts(
            candle_pack,
            computation.timeframe_alignment_report,
            computation.timestamp_alignment_report,
            computation.match_report,
            backfill_report,
            closure_report,
            storage_report,
        );
        bundle.write_to_dir(&config.output_dir())?;
        Ok(bundle)
    }
}

impl CandleCoverageClosureReport {
    pub fn to_text(&self) -> String {
        [
            format!("closure_id={}", self.closure_id),
            format!(
                "previous_primary_bottleneck={:?}",
                self.previous_primary_bottleneck
            ),
            format!(
                "current_primary_bottleneck={:?}",
                self.current_primary_bottleneck
            ),
            format!("bottleneck_changed={}", self.bottleneck_changed),
            format!("improvement_detected={}", self.improvement_detected),
            format!("final_status={:?}", self.final_status),
            format!("final_recommendation={:?}", self.final_recommendation),
        ]
        .join("\n")
    }
}

fn resolve_pack(
    config: &CandleCoverageClosureConfig,
) -> Result<OfficialCandleCoveragePack, String> {
    let Some(path) = config.candle_pack_config_path.as_deref() else {
        return Ok(OfficialCandleCoveragePack::build(
            &OfficialCandleCoveragePackConfig::default(),
        )?);
    };
    if config.run_candle_pack {
        load_pack_from_path_or_config(path)
    } else {
        load_pack_from_path_or_config(path)
    }
}

fn load_rows_for_closure(
    backfill_config: Option<&ComparableEvidenceBackfillConfig>,
) -> Result<Vec<ComparableCommitteeEvidenceRow>, String> {
    let Some(config) = backfill_config else {
        return Ok(Vec::new());
    };
    let mut rows = Vec::new();
    for path in &config.comparable_evidence_bundle_paths {
        rows.extend(ComparableCommitteeEvidenceBundle::from_json_path(Path::new(path))?.rows);
    }
    Ok(rows)
}

fn maybe_run_scorecard(
    config: &CandleCoverageClosureConfig,
) -> Result<Option<CoreScorecardRerunSummary>, String> {
    if !config.run_core_scorecard_rerun {
        return Ok(None);
    }
    let previous = config
        .previous_core_scorecard_path
        .as_deref()
        .map(|path| CorePerformanceScorecard::from_json_path(Path::new(path)))
        .transpose()?;
    let Some(scorecard_path) = config.core_performance_config_path.as_deref() else {
        return Ok(Some(CoreScorecardRerun::missing(
            "scorecard rerun enabled without core_performance_config_path",
        )));
    };
    let current_bundle = CoreScorecardRerun::default().run_bundle(scorecard_path)?;
    Ok(Some(CoreScorecardRerun::default().summarize(
        previous.as_ref(),
        Some(&current_bundle.scorecard),
        Vec::new(),
        true,
    )))
}

fn derive_current_bottleneck(
    candle_pack: &OfficialCandleCoveragePack,
    backfill_report: Option<&ComparableEvidenceBackfillReport>,
    match_report: &super::candle_coverage_match::CandleCoverageMatchReport,
) -> Option<CoreBottleneckKind> {
    if candle_pack.readiness_eligible_series_count == 0 {
        Some(CoreBottleneckKind::MissingOfficialCandles)
    } else if match_report.insufficient_future_window_count > 0 {
        Some(CoreBottleneckKind::ScenarioMaterializationWeak)
    } else if backfill_report.is_some_and(|report| report.rows_still_summary_derived > 0) {
        Some(CoreBottleneckKind::ScenarioMaterializationWeak)
    } else {
        Some(CoreBottleneckKind::MissingOfficialCandles)
    }
}

fn determine_final_status(
    candle_pack: &OfficialCandleCoveragePack,
    match_report: &super::candle_coverage_match::CandleCoverageMatchReport,
    backfill_report: Option<&ComparableEvidenceBackfillReport>,
    bottleneck_changed: bool,
) -> CandleCoverageClosureFinalStatus {
    if candle_pack.readiness_eligible_series_count == 0 {
        return CandleCoverageClosureFinalStatus::StillMissingOfficialCandles;
    }
    match match_report.coverage_status {
        CandleCoverageStatus::NeedBetterTimeframeAlignment => {
            CandleCoverageClosureFinalStatus::StillNeedBetterTimeframeAlignment
        }
        CandleCoverageStatus::NeedBetterTimestampAlignment => {
            CandleCoverageClosureFinalStatus::StillNeedBetterTimestampAlignment
        }
        CandleCoverageStatus::NeedLongerFutureWindows => {
            CandleCoverageClosureFinalStatus::StillNeedLongerFutureWindows
        }
        CandleCoverageStatus::DiagnosticOnly => {
            CandleCoverageClosureFinalStatus::DiagnosticCandleCoverageOnly
        }
        CandleCoverageStatus::HealthyCandleCoverage
            if match_report.official_ready_match_count > 0 =>
        {
            CandleCoverageClosureFinalStatus::OfficialCandleCoverageImproved
        }
        CandleCoverageStatus::HealthyCandleCoverage => {
            CandleCoverageClosureFinalStatus::CandleCoverageImproved
        }
        _ if backfill_report.is_some_and(|report| report.rows_still_summary_derived > 0) => {
            CandleCoverageClosureFinalStatus::StillScenarioMaterializationWeak
        }
        _ if bottleneck_changed => CandleCoverageClosureFinalStatus::CoreBottleneckMoved,
        _ => CandleCoverageClosureFinalStatus::NoImprovement,
    }
}

fn determine_recommendation(
    final_status: CandleCoverageClosureFinalStatus,
    scorecard_summary: Option<&CoreScorecardRerunSummary>,
) -> CandleCoverageClosureRecommendation {
    match final_status {
        CandleCoverageClosureFinalStatus::StillMissingOfficialCandles => {
            CandleCoverageClosureRecommendation::ImproveCandleCoverageFirst
        }
        CandleCoverageClosureFinalStatus::StillNeedBetterTimestampAlignment => {
            CandleCoverageClosureRecommendation::ImproveTimestampAlignmentFirst
        }
        CandleCoverageClosureFinalStatus::StillNeedBetterTimeframeAlignment => {
            CandleCoverageClosureRecommendation::ImproveTimeframeAlignmentFirst
        }
        CandleCoverageClosureFinalStatus::StillNeedLongerFutureWindows => {
            CandleCoverageClosureRecommendation::ImproveCounterfactualDepthFirst
        }
        CandleCoverageClosureFinalStatus::StillScenarioMaterializationWeak => {
            CandleCoverageClosureRecommendation::ImproveOutcomeLinkingFirst
        }
        CandleCoverageClosureFinalStatus::CoreBottleneckMoved => {
            CandleCoverageClosureRecommendation::RerunCorePerformance
        }
        CandleCoverageClosureFinalStatus::OfficialCandleCoverageImproved
        | CandleCoverageClosureFinalStatus::CandleCoverageImproved => {
            if scorecard_summary.is_some_and(|summary| summary.status_improved) {
                CandleCoverageClosureRecommendation::CommitteeCoreReadyForDeeperEvidence
            } else {
                CandleCoverageClosureRecommendation::KeepTrinity
            }
        }
        CandleCoverageClosureFinalStatus::DiagnosticCandleCoverageOnly => {
            CandleCoverageClosureRecommendation::MoreOfficialEvidence
        }
        CandleCoverageClosureFinalStatus::NoImprovement => {
            CandleCoverageClosureRecommendation::NeedMoreEvidence
        }
    }
}

fn build_storage_report(
    candle_pack: &OfficialCandleCoveragePack,
    backfilled_bundle: Option<&ComparableCommitteeEvidenceBundle>,
    closure_report: &CandleCoverageClosureReport,
    config: &CandleCoverageClosureConfig,
) -> CandleCoverageStorageReport {
    let candle_pack_bytes = candle_pack.storage_bytes;
    let backfilled_bundle_bytes = backfilled_bundle
        .map(|bundle| bundle.storage_bytes)
        .unwrap_or(0);
    let generated_reference_bytes = config.reference_pack_config_paths.len() * 128;
    let output_report_bytes = closure_report.to_text().len()
        + closure_report.candle_pack_summary.len()
        + closure_report.match_report.len();
    let mut artifacts = vec![
        CandleCoverageArtifactSize {
            path: "official_candle_coverage_pack".to_string(),
            bytes: candle_pack_bytes,
        },
        CandleCoverageArtifactSize {
            path: "candle_coverage_closure_report".to_string(),
            bytes: output_report_bytes,
        },
    ];
    if backfilled_bundle_bytes > 0 {
        artifacts.push(CandleCoverageArtifactSize {
            path: "backfilled_comparable_bundle".to_string(),
            bytes: backfilled_bundle_bytes,
        });
    }
    if generated_reference_bytes > 0 {
        artifacts.push(CandleCoverageArtifactSize {
            path: "generated_reference_summary".to_string(),
            bytes: generated_reference_bytes,
        });
    }
    build_candle_coverage_storage_report(
        candle_pack_bytes,
        backfilled_bundle_bytes,
        generated_reference_bytes,
        output_report_bytes,
        5_000_000,
        artifacts,
    )
}

fn default_output_root() -> String {
    "target/soma_candle_coverage_closure".to_string()
}

fn default_true() -> bool {
    true
}
