use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::backtest::Timeframe;
use crate::core::{ReasonCode, stable_reason_codes};
use crate::data::{
    AssetClass, AuthConfig, CollectionOutputSize, EvidenceSourceKind, LocalDataOnboardingConfig,
    MarketVenue, OfficialCollectionEntry, OfficialCollectionPlan, OfficialCollectionRunner,
    PreflightReport, PreflightValidator, ProviderKind, RawArchivePolicy, RetentionPolicy,
};
use crate::experiment::{CoreBottleneckKind, CoreScorecardRerun, CoreScorecardRerunSummary};

use super::candle_acquisition_job::{
    CandleAcquisitionJob, CandleAcquisitionJobKind, CandleAcquisitionJobStatus,
    CandleAcquisitionPlan,
};
use super::candle_expansion_closure::{
    CandleExpansionClosureReport, build_candle_expansion_closure_report,
};
use super::comparable_committee_evidence::{
    ComparableCommitteeEvidenceBundle, ComparableCommitteeEvidenceConfig,
};
use super::comparable_evidence_backfill::{
    ComparableEvidenceBackfillConfig, ComparableEvidenceBackfillReport,
    ComparableEvidenceBackfillRunner,
};
use super::official_candle_coverage_pack::{
    OfficialCandleCoveragePack, OfficialCandleCoveragePackConfig,
};
use super::official_candle_expansion_bundle::{
    CandleExpansionArtifactSize, CandleExpansionStorageReport, OfficialCandleExpansionBundle,
    build_candle_expansion_storage_report, build_expansion_final_summary,
};
use super::official_candle_expansion_plan::{
    OfficialCandleExpansionPlanConfig, build_official_candle_acquisition_plan, load_gap_map,
};
use super::official_candle_gap_map::{
    OfficialCandleCoverageGapMap, OfficialCandleGapConfig, OfficialCandleGapStatus,
    build_gap_map_from_inputs, load_gap_inputs,
};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CandleExpansionCounts {
    pub gap_count: usize,
    pub official_gap_count: usize,
    pub total_series: usize,
    pub official_series: usize,
    pub non_crypto_official_series: usize,
    pub matches: usize,
    pub official_ready_matches: usize,
    pub complete_comparable_rows: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum OfficialCandleExpansionFinalStatus {
    CandleCoverageExpanded,
    OfficialCandleCoverageExpanded,
    DiagnosticCandleCoverageOnly,
    MissingAuth,
    MissingApproval,
    MissingEndpointTemplate,
    MissingOfficialCsv,
    MissingOfficialProvenance,
    MissingOfficialPreflight,
    StillMissingOfficialCandles,
    StillNeedBetterTimestampAlignment,
    StillNeedBetterTimeframeAlignment,
    StillNeedLongerFutureWindows,
    StillScenarioMaterializationWeak,
    #[default]
    NoImprovement,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum OfficialCandleExpansionRecommendation {
    ProvideOfficialCanonicalCsv,
    SetKrxApiKey,
    SetKrxEndpointTemplate,
    WaitForKrxApproval,
    SetAlphaVantageApiKey,
    RunOfficialAcquisition,
    RunCandleCoverageClose,
    ImproveTimestampAlignmentFirst,
    ImproveTimeframeAlignmentFirst,
    ImproveOutcomeLinkingFirst,
    ImproveCounterfactualDepthFirst,
    RerunCorePerformance,
    CommitteeCoreReadyForDeeperEvidence,
    KeepTrinity,
    #[default]
    NeedMoreEvidence,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfficialCandleExpansionReport {
    pub expansion_id: String,
    pub gap_map: OfficialCandleCoverageGapMap,
    pub acquisition_plan: CandleAcquisitionPlan,
    pub executed_jobs: Vec<CandleAcquisitionJob>,
    #[serde(default)]
    pub new_candle_pack: Option<OfficialCandleCoveragePack>,
    #[serde(default)]
    pub backfill_report: Option<ComparableEvidenceBackfillReport>,
    #[serde(default)]
    pub reference_generation_summary: Option<String>,
    #[serde(default)]
    pub counterfactual_depth_summary: Option<String>,
    #[serde(default)]
    pub core_scorecard_rerun_summary: Option<CoreScorecardRerunSummary>,
    #[serde(default)]
    pub before_counts: Option<CandleExpansionCounts>,
    pub after_counts: CandleExpansionCounts,
    pub added_official_candle_series: usize,
    pub added_non_crypto_official_candle_series: usize,
    pub added_official_ready_matches: usize,
    pub added_backfilled_rows: usize,
    pub added_complete_comparable_rows: usize,
    pub added_outcome_references: usize,
    pub added_no_trade_counterfactuals: usize,
    pub added_risk_denied_counterfactuals: usize,
    #[serde(default)]
    pub previous_primary_bottleneck: Option<CoreBottleneckKind>,
    #[serde(default)]
    pub current_primary_bottleneck: Option<CoreBottleneckKind>,
    pub bottleneck_changed: bool,
    pub final_status: OfficialCandleExpansionFinalStatus,
    pub final_recommendation: OfficialCandleExpansionRecommendation,
    pub blockers: Vec<String>,
    pub warnings: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OfficialCandleExpansionRunner;

impl OfficialCandleExpansionReport {
    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("expansion_id={}", self.expansion_id),
            format!("final_status={:?}", self.final_status),
            format!("final_recommendation={:?}", self.final_recommendation),
            format!(
                "added_official_candle_series={}",
                self.added_official_candle_series
            ),
            format!(
                "added_non_crypto_official_candle_series={}",
                self.added_non_crypto_official_candle_series
            ),
            format!(
                "added_official_ready_matches={}",
                self.added_official_ready_matches
            ),
            format!("added_backfilled_rows={}", self.added_backfilled_rows),
            format!(
                "added_complete_comparable_rows={}",
                self.added_complete_comparable_rows
            ),
            format!(
                "previous_primary_bottleneck={:?}",
                self.previous_primary_bottleneck
            ),
            format!(
                "current_primary_bottleneck={:?}",
                self.current_primary_bottleneck
            ),
            format!("bottleneck_changed={}", self.bottleneck_changed),
            format!("blockers={}", self.blockers.join(" | ")),
            format!("warnings={}", self.warnings.join(" | ")),
        ];
        lines.push("executed_jobs:".to_string());
        lines.extend(self.executed_jobs.iter().map(CandleAcquisitionJob::to_text));
        if let Some(summary) = &self.reference_generation_summary {
            lines.push(summary.clone());
        }
        if let Some(summary) = &self.counterfactual_depth_summary {
            lines.push(summary.clone());
        }
        if let Some(summary) = &self.core_scorecard_rerun_summary {
            lines.push(summary.to_text());
        }
        lines.join("\n")
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("official_candle_expansion_report.json"),
            self.to_json_string()?,
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("official_candle_expansion_report.txt"),
            self.to_text(),
        )
        .map_err(|err| err.to_string())?;
        Ok(output_dir.join("official_candle_expansion_report.json"))
    }
}

impl OfficialCandleExpansionRunner {
    pub fn run(
        &self,
        config: &OfficialCandleExpansionPlanConfig,
    ) -> Result<OfficialCandleExpansionReport, String> {
        self.run_bundle(config)
            .map(|bundle| bundle.expansion_report)
    }

    pub fn run_bundle(
        &self,
        config: &OfficialCandleExpansionPlanConfig,
    ) -> Result<OfficialCandleExpansionBundle, String> {
        config.validate()?;
        let gap_map = load_gap_map(config)?;
        let input_config = load_gap_config(config)?;
        let input_context = input_config.as_ref().map(load_gap_inputs).transpose()?;
        let acquisition_plan = build_official_candle_acquisition_plan(config)?;
        let mut executed_jobs = acquisition_plan.jobs.clone();
        let output_dir = config.output_dir();
        let mut warnings = Vec::new();
        let mut blockers = Vec::new();

        let import_outputs = if config.run_import_jobs {
            execute_import_jobs(&mut executed_jobs, &output_dir)?
        } else {
            Vec::new()
        };
        let collection_outputs = if config.run_collection_jobs {
            execute_collection_jobs(config, &mut executed_jobs, &output_dir)?
        } else {
            Vec::new()
        };

        let before_pack = input_context
            .as_ref()
            .map(|context| context.pack.clone())
            .unwrap_or_else(empty_pack);
        let before_counts = input_context
            .as_ref()
            .map(|context| build_counts(&gap_map, &before_pack, &context.rows, None, None));
        let merged_pack = build_after_pack(
            input_context.as_ref(),
            &import_outputs,
            &collection_outputs,
            &output_dir,
            config,
        )?;
        let rows = input_context
            .as_ref()
            .map(|context| context.rows.clone())
            .unwrap_or_default();
        let after_gap_map = if let Some(gap_cfg) = input_config.as_ref() {
            build_gap_map_from_inputs(gap_cfg, &rows, &merged_pack)
        } else {
            gap_map.clone()
        };

        let backfill_result = if !rows.is_empty() {
            run_backfill(config, &output_dir, &rows, &merged_pack)?
        } else {
            None
        };
        let backfill_report = backfill_result.as_ref().map(|report| report.report.clone());
        let after_counts = build_counts(
            &after_gap_map,
            &merged_pack,
            &rows,
            backfill_result
                .as_ref()
                .map(|result| &result.bundle)
                .or_else(|| None),
            backfill_report.as_ref(),
        );
        let core_scorecard_rerun_summary = maybe_run_core_scorecard(config)?;
        let previous_primary_bottleneck = core_scorecard_rerun_summary
            .as_ref()
            .and_then(|summary| summary.previous_primary_bottleneck)
            .or_else(|| {
                derive_bottleneck(
                    Some(&gap_map),
                    before_counts.as_ref(),
                    backfill_report.as_ref(),
                )
            });
        let current_primary_bottleneck = core_scorecard_rerun_summary
            .as_ref()
            .and_then(|summary| summary.current_primary_bottleneck)
            .or_else(|| {
                derive_bottleneck(
                    Some(&after_gap_map),
                    Some(&after_counts),
                    backfill_report.as_ref(),
                )
            });
        let bottleneck_changed = previous_primary_bottleneck != current_primary_bottleneck;
        let reference_generation_summary = config.run_reference_generation.then(|| {
            if config.reference_pack_config_paths.is_empty() {
                "reference_generation=enabled-without-configs".to_string()
            } else {
                format!(
                    "reference_generation=invoked:{}",
                    config.reference_pack_config_paths.join("|")
                )
            }
        });
        let counterfactual_depth_summary = config.run_counterfactual_depth_close.then(|| {
            config
                .counterfactual_depth_closure_config_path
                .as_deref()
                .map(|path| format!("counterfactual_depth_close=invoked:{path}"))
                .unwrap_or_else(|| "counterfactual_depth_close=enabled-without-config".to_string())
        });
        let (final_status, final_recommendation, mut status_blockers) = classify_final_status(
            &acquisition_plan,
            &executed_jobs,
            &gap_map,
            &after_gap_map,
            before_counts.as_ref(),
            &after_counts,
            backfill_report.as_ref(),
            bottleneck_changed,
        );
        blockers.append(&mut status_blockers);
        if matches!(
            final_status,
            OfficialCandleExpansionFinalStatus::NoImprovement
        ) && merged_pack.descriptors.is_empty()
        {
            warnings.push("no local candle pack was available for expansion".to_string());
        }
        let added_official_candle_series = after_counts.official_series.saturating_sub(
            before_counts
                .as_ref()
                .map(|counts| counts.official_series)
                .unwrap_or(0),
        );
        let added_non_crypto_official_candle_series =
            after_counts.non_crypto_official_series.saturating_sub(
                before_counts
                    .as_ref()
                    .map(|counts| counts.non_crypto_official_series)
                    .unwrap_or(0),
            );
        let added_official_ready_matches = after_counts.official_ready_matches.saturating_sub(
            before_counts
                .as_ref()
                .map(|counts| counts.official_ready_matches)
                .unwrap_or(0),
        );
        let added_backfilled_rows = backfill_report
            .as_ref()
            .map(|report| report.rows_with_new_candle_match)
            .unwrap_or(0);
        let added_complete_comparable_rows = after_counts.complete_comparable_rows.saturating_sub(
            before_counts
                .as_ref()
                .map(|counts| counts.complete_comparable_rows)
                .unwrap_or(0),
        );
        let report = OfficialCandleExpansionReport {
            expansion_id: config.plan_id.clone(),
            gap_map: after_gap_map.clone(),
            acquisition_plan: acquisition_plan.clone(),
            executed_jobs: executed_jobs.clone(),
            new_candle_pack: Some(merged_pack.clone()),
            backfill_report: backfill_report.clone(),
            reference_generation_summary,
            counterfactual_depth_summary,
            core_scorecard_rerun_summary,
            before_counts: before_counts.clone(),
            after_counts: after_counts.clone(),
            added_official_candle_series,
            added_non_crypto_official_candle_series,
            added_official_ready_matches,
            added_backfilled_rows,
            added_complete_comparable_rows,
            added_outcome_references: 0,
            added_no_trade_counterfactuals: 0,
            added_risk_denied_counterfactuals: 0,
            previous_primary_bottleneck,
            current_primary_bottleneck,
            bottleneck_changed,
            final_status,
            final_recommendation,
            blockers,
            warnings,
            reason_codes: stable_reason_codes(
                &config
                    .reason_codes
                    .iter()
                    .cloned()
                    .chain([
                        ReasonCode::OfficialCandleCoverageBuilt,
                        ReasonCode::DeterministicPath,
                    ])
                    .collect::<Vec<_>>(),
            ),
        };
        report.write_to_dir(&output_dir)?;
        let closure_report = build_candle_expansion_closure_report(
            &report,
            &gap_map,
            &after_gap_map,
            before_counts.as_ref(),
            &after_counts,
            backfill_report.as_ref(),
        );
        let operator_actions = acquisition_plan.operator_actions.clone();
        let storage_report = build_bundle_storage_report(
            &report,
            &closure_report,
            output_dir.as_path(),
            config.max_total_bytes,
        );
        let final_summary =
            build_expansion_final_summary(&report, &closure_report, &storage_report);
        let bundle = OfficialCandleExpansionBundle {
            expansion_report: report,
            gap_map: after_gap_map,
            acquisition_plan,
            operator_actions,
            new_candle_pack: Some(merged_pack),
            backfill_report,
            closure_report,
            storage_report,
            final_summary,
            reason_codes: stable_reason_codes(&[
                ReasonCode::OfficialCandleCoverageBuilt,
                ReasonCode::DeterministicPath,
            ]),
        };
        bundle.write_to_dir(&output_dir)?;
        Ok(bundle)
    }
}

fn load_gap_config(
    config: &OfficialCandleExpansionPlanConfig,
) -> Result<Option<OfficialCandleGapConfig>, String> {
    config
        .gap_config_path
        .as_deref()
        .map(|path| OfficialCandleGapConfig::from_toml_path(Path::new(path)))
        .transpose()
}

fn execute_import_jobs(
    jobs: &mut [CandleAcquisitionJob],
    _output_dir: &Path,
) -> Result<Vec<CandleAcquisitionJob>, String> {
    let mut executed = Vec::new();
    for job in jobs.iter_mut().filter(|job| job.is_import_job()) {
        match job.status {
            CandleAcquisitionJobStatus::ReadyToRun | CandleAcquisitionJobStatus::DiagnosticOnly => {
                let Some(input_csv) = job.local_input_csv_path.as_deref() else {
                    job.status = CandleAcquisitionJobStatus::Skipped;
                    continue;
                };
                let Some(expected_csv) = job.expected_canonical_csv_path.as_deref() else {
                    job.status = CandleAcquisitionJobStatus::Failed;
                    continue;
                };
                fs::create_dir_all(
                    Path::new(expected_csv)
                        .parent()
                        .unwrap_or(Path::new(&job.output_root)),
                )
                .map_err(|err| err.to_string())?;
                fs::copy(input_csv, expected_csv).map_err(|err| err.to_string())?;
                if let (Some(source), Some(target)) = (
                    job.local_input_provenance_path.as_deref(),
                    job.expected_provenance_path.as_deref(),
                ) {
                    if Path::new(source).exists() {
                        fs::copy(source, target).map_err(|err| err.to_string())?;
                    }
                }
                if let (Some(source), Some(target)) = (
                    job.local_input_preflight_path.as_deref(),
                    job.expected_preflight_path.as_deref(),
                ) {
                    if Path::new(source).exists() {
                        fs::copy(source, target).map_err(|err| err.to_string())?;
                    }
                } else if let Some(target) = job.expected_preflight_path.as_deref() {
                    let report = run_local_preflight(job, expected_csv)?;
                    fs::write(
                        target,
                        report.to_json_string().map_err(|err| err.to_string())?,
                    )
                    .map_err(|err| err.to_string())?;
                }
                job.status = CandleAcquisitionJobStatus::RanSuccessfully;
                executed.push(job.clone());
            }
            _ => {}
        }
    }
    Ok(executed)
}

fn execute_collection_jobs(
    config: &OfficialCandleExpansionPlanConfig,
    jobs: &mut [CandleAcquisitionJob],
    output_dir: &Path,
) -> Result<Vec<CandleAcquisitionJob>, String> {
    let runnable = jobs
        .iter()
        .filter(|job| {
            job.is_collection_job() && job.status == CandleAcquisitionJobStatus::ReadyToRun
        })
        .cloned()
        .collect::<Vec<_>>();
    if runnable.is_empty() {
        return Ok(Vec::new());
    }
    let plan = OfficialCollectionPlan {
        plan_id: format!("{}-runner-collection", config.plan_id),
        output_root: output_dir.join("collection").display().to_string(),
        max_total_bytes: config.max_total_bytes,
        max_total_rows: config.max_jobs.saturating_mul(config.max_rows_per_job),
        max_total_requests: config.max_jobs.saturating_mul(config.max_requests_per_job),
        default_collection_size_policy: crate::data::CollectionSizePolicy {
            max_symbols_per_run: config.max_symbols_per_job,
            max_rows_per_symbol: config.max_rows_per_job,
            max_total_rows_per_run: config.max_jobs.saturating_mul(config.max_rows_per_job),
            max_raw_bytes_per_run: config.max_total_bytes / 2,
            max_canonical_bytes_per_run: config.max_total_bytes / 2,
            max_requests_per_run: config.max_requests_per_job,
            max_days_per_run: 365,
            default_outputsize: CollectionOutputSize::Compact,
            raw_archive_policy: RawArchivePolicy::CompactJson,
            retention_policy: RetentionPolicy::DeleteRawAfterCanonicalAndManifest,
            allow_full_history: false,
            reason_codes: vec![ReasonCode::DeterministicPath],
        },
        default_compression_policy: crate::data::CompressionPolicy::default(),
        default_retention_policy: RetentionPolicy::DeleteRawAfterCanonicalAndManifest,
        storage_budget: crate::data::StorageBudget::default(),
        entries: runnable.iter().map(job_to_collection_entry).collect(),
        continue_on_missing_auth: true,
        continue_on_provider_failure: true,
        reason_codes: vec![ReasonCode::DeterministicPath],
    };
    let collection_report = OfficialCollectionRunner::default().run_plan(&plan);
    for job in jobs.iter_mut().filter(|job| job.is_collection_job()) {
        if collection_report
            .entry_reports
            .iter()
            .any(|entry| entry.entry_id == job.job_id)
        {
            job.status = CandleAcquisitionJobStatus::RanSuccessfully;
        }
    }
    Ok(jobs
        .iter()
        .filter(|job| {
            job.is_collection_job() && job.status == CandleAcquisitionJobStatus::RanSuccessfully
        })
        .cloned()
        .collect())
}

fn build_after_pack(
    input_context: Option<&super::official_candle_gap_map::OfficialCandleGapInputs>,
    import_outputs: &[CandleAcquisitionJob],
    collection_outputs: &[CandleAcquisitionJob],
    output_dir: &Path,
    config: &OfficialCandleExpansionPlanConfig,
) -> Result<OfficialCandleCoveragePack, String> {
    let mut canonical_csv_paths = input_context
        .map(|context| context.canonical_csv_paths.clone())
        .unwrap_or_else(Vec::new);
    let mut provenance_paths = input_context
        .map(|context| context.provenance_paths.clone())
        .unwrap_or_default();
    let mut preflight_paths = input_context
        .map(|context| context.preflight_paths.clone())
        .unwrap_or_default();
    let mut manifest_paths = input_context
        .map(|context| context.manifest_paths.clone())
        .unwrap_or_default();
    for job in import_outputs.iter().chain(collection_outputs.iter()) {
        if let Some(path) = &job.expected_canonical_csv_path {
            canonical_csv_paths.push(path.clone());
        }
        if let Some(path) = &job.expected_provenance_path {
            if Path::new(path).exists() {
                provenance_paths.push(path.clone());
            }
        }
        if let Some(path) = &job.expected_preflight_path {
            if Path::new(path).exists() {
                preflight_paths.push(path.clone());
            }
        }
        if let Some(path) = &job.local_input_manifest_path {
            if Path::new(path).exists() {
                manifest_paths.push(path.clone());
            }
        }
    }
    canonical_csv_paths.sort();
    canonical_csv_paths.dedup();
    provenance_paths.sort();
    provenance_paths.dedup();
    preflight_paths.sort();
    preflight_paths.dedup();
    manifest_paths.sort();
    manifest_paths.dedup();
    OfficialCandleCoveragePack::build(&OfficialCandleCoveragePackConfig {
        pack_id: format!("{}-expanded-pack", config.plan_id),
        canonical_csv_paths,
        provenance_paths,
        preflight_report_paths: preflight_paths,
        manifest_paths,
        output_root: output_dir.join("pack").display().to_string(),
        ..OfficialCandleCoveragePackConfig::default()
    })
}

fn run_backfill(
    config: &OfficialCandleExpansionPlanConfig,
    output_dir: &Path,
    rows: &[super::ComparableCommitteeEvidenceRow],
    pack: &OfficialCandleCoveragePack,
) -> Result<Option<super::ComparableEvidenceBackfillResult>, String> {
    if rows.is_empty() {
        return Ok(None);
    }
    let comparable_config = ComparableCommitteeEvidenceConfig {
        comparable_id: format!("{}-backfill-input", config.plan_id),
        output_root: output_dir.join("backfill").display().to_string(),
        allow_summary_derived_rows: true,
        require_outcome_reference: false,
        ..ComparableCommitteeEvidenceConfig::default()
    };
    let bundle = ComparableCommitteeEvidenceBundle::from_rows(&comparable_config, rows.to_vec());
    let bundle_path = bundle.write_to_dir(&comparable_config.output_dir())?;
    let pack_output_dir = output_dir.join("pack");
    let pack_path = pack.write_to_dir(&pack_output_dir)?;
    let backfill_config = ComparableEvidenceBackfillConfig {
        backfill_id: format!("{}-backfill", config.plan_id),
        comparable_evidence_bundle_paths: vec![bundle_path.display().to_string()],
        official_candle_coverage_pack_paths: vec![pack_path.display().to_string()],
        output_root: output_dir.join("backfill").display().to_string(),
        allow_diagnostic_backfill: true,
        require_official_for_official_backfill: false,
        ..ComparableEvidenceBackfillConfig::default()
    };
    ComparableEvidenceBackfillRunner::default()
        .run_bundle(&backfill_config)
        .map(Some)
}

fn build_counts(
    gap_map: &OfficialCandleCoverageGapMap,
    pack: &OfficialCandleCoveragePack,
    rows: &[super::ComparableCommitteeEvidenceRow],
    backfilled_bundle: Option<&ComparableCommitteeEvidenceBundle>,
    backfill_report: Option<&ComparableEvidenceBackfillReport>,
) -> CandleExpansionCounts {
    let bundle = backfilled_bundle.cloned().unwrap_or_else(|| {
        ComparableCommitteeEvidenceBundle::from_rows(
            &ComparableCommitteeEvidenceConfig {
                comparable_id: "expansion-counts".to_string(),
                allow_summary_derived_rows: true,
                require_outcome_reference: false,
                ..ComparableCommitteeEvidenceConfig::default()
            },
            rows.to_vec(),
        )
    });
    CandleExpansionCounts {
        gap_count: gap_map.total_gaps,
        official_gap_count: gap_map.official_gap_count,
        total_series: pack.descriptors.len(),
        official_series: pack.official_non_crypto_series.len() + pack.official_crypto_series.len(),
        non_crypto_official_series: pack.official_non_crypto_series.len(),
        matches: bundle
            .rows
            .iter()
            .filter(|row| row.candle_coverage_available)
            .count(),
        official_ready_matches: bundle
            .rows
            .iter()
            .filter(|row| row.candle_official_ready_match)
            .count()
            .max(
                backfill_report
                    .map(|report| report.rows_with_new_official_ready_match)
                    .unwrap_or(0),
            ),
        complete_comparable_rows: bundle.complete_rows,
    }
}

fn classify_final_status(
    plan: &CandleAcquisitionPlan,
    _executed_jobs: &[CandleAcquisitionJob],
    before_gap_map: &OfficialCandleCoverageGapMap,
    after_gap_map: &OfficialCandleCoverageGapMap,
    before_counts: Option<&CandleExpansionCounts>,
    after_counts: &CandleExpansionCounts,
    backfill_report: Option<&ComparableEvidenceBackfillReport>,
    bottleneck_changed: bool,
) -> (
    OfficialCandleExpansionFinalStatus,
    OfficialCandleExpansionRecommendation,
    Vec<String>,
) {
    let mut blockers = Vec::new();
    if plan
        .jobs
        .iter()
        .any(|job| job.job_kind == CandleAcquisitionJobKind::SkippedMissingAuth)
    {
        blockers.push("missing auth blocks bounded collection or import promotion".to_string());
        return (
            OfficialCandleExpansionFinalStatus::MissingAuth,
            if plan
                .jobs
                .iter()
                .any(|job| job.provider_kind == Some(ProviderKind::KrxOpenApi))
            {
                OfficialCandleExpansionRecommendation::SetKrxApiKey
            } else {
                OfficialCandleExpansionRecommendation::SetAlphaVantageApiKey
            },
            blockers,
        );
    }
    if plan
        .jobs
        .iter()
        .any(|job| job.job_kind == CandleAcquisitionJobKind::SkippedMissingApproval)
    {
        blockers.push("krx approval is still missing".to_string());
        return (
            OfficialCandleExpansionFinalStatus::MissingApproval,
            OfficialCandleExpansionRecommendation::WaitForKrxApproval,
            blockers,
        );
    }
    if plan
        .jobs
        .iter()
        .any(|job| job.job_kind == CandleAcquisitionJobKind::SkippedMissingEndpointTemplate)
    {
        blockers.push("krx endpoint template is still missing".to_string());
        return (
            OfficialCandleExpansionFinalStatus::MissingEndpointTemplate,
            OfficialCandleExpansionRecommendation::SetKrxEndpointTemplate,
            blockers,
        );
    }
    if plan.jobs.iter().any(|job| {
        job.job_kind == CandleAcquisitionJobKind::LocalOfficialCsvImport
            && job.status == CandleAcquisitionJobStatus::Skipped
    }) {
        blockers.push("no official canonical csv was available for local import".to_string());
        return (
            OfficialCandleExpansionFinalStatus::MissingOfficialCsv,
            OfficialCandleExpansionRecommendation::ProvideOfficialCanonicalCsv,
            blockers,
        );
    }
    if after_gap_map.cells.iter().any(|cell| {
        cell.gap_kinds
            .contains(&super::official_candle_gap_map::OfficialCandleGapKind::MissingProvenance)
    }) {
        blockers.push("official provenance is still missing".to_string());
        return (
            OfficialCandleExpansionFinalStatus::MissingOfficialProvenance,
            OfficialCandleExpansionRecommendation::ProvideOfficialCanonicalCsv,
            blockers,
        );
    }
    if after_gap_map.cells.iter().any(|cell| {
        cell.gap_kinds
            .contains(&super::official_candle_gap_map::OfficialCandleGapKind::MissingPreflight)
    }) {
        blockers.push("official preflight is still missing".to_string());
        return (
            OfficialCandleExpansionFinalStatus::MissingOfficialPreflight,
            OfficialCandleExpansionRecommendation::RunCandleCoverageClose,
            blockers,
        );
    }
    if matches!(
        after_gap_map.gap_status,
        OfficialCandleGapStatus::TimeframeAlignmentWeak
    ) {
        return (
            OfficialCandleExpansionFinalStatus::StillNeedBetterTimeframeAlignment,
            OfficialCandleExpansionRecommendation::ImproveTimeframeAlignmentFirst,
            blockers,
        );
    }
    if matches!(
        after_gap_map.gap_status,
        OfficialCandleGapStatus::TimestampAlignmentWeak
    ) {
        return (
            OfficialCandleExpansionFinalStatus::StillNeedBetterTimestampAlignment,
            OfficialCandleExpansionRecommendation::ImproveTimestampAlignmentFirst,
            blockers,
        );
    }
    if after_gap_map.cells.iter().any(|cell| {
        cell.gap_kinds
            .contains(&super::official_candle_gap_map::OfficialCandleGapKind::MissingFutureWindow)
    }) {
        return (
            OfficialCandleExpansionFinalStatus::StillNeedLongerFutureWindows,
            OfficialCandleExpansionRecommendation::ImproveCounterfactualDepthFirst,
            blockers,
        );
    }
    if backfill_report.is_some_and(|report| report.rows_still_summary_derived > 0) {
        return (
            OfficialCandleExpansionFinalStatus::StillScenarioMaterializationWeak,
            OfficialCandleExpansionRecommendation::ImproveOutcomeLinkingFirst,
            blockers,
        );
    }
    if after_gap_map.total_gaps == 0
        || after_counts.gap_count
            < before_counts
                .map(|counts| counts.gap_count)
                .unwrap_or(usize::MAX)
    {
        let status = if after_counts.non_crypto_official_series
            > before_counts
                .map(|counts| counts.non_crypto_official_series)
                .unwrap_or(0)
        {
            OfficialCandleExpansionFinalStatus::OfficialCandleCoverageExpanded
        } else if after_counts.official_series
            > before_counts
                .map(|counts| counts.official_series)
                .unwrap_or(0)
        {
            OfficialCandleExpansionFinalStatus::CandleCoverageExpanded
        } else {
            OfficialCandleExpansionFinalStatus::DiagnosticCandleCoverageOnly
        };
        let recommendation = if bottleneck_changed {
            OfficialCandleExpansionRecommendation::RerunCorePerformance
        } else if matches!(
            status,
            OfficialCandleExpansionFinalStatus::OfficialCandleCoverageExpanded
        ) && after_gap_map.total_gaps == 0
        {
            OfficialCandleExpansionRecommendation::CommitteeCoreReadyForDeeperEvidence
        } else {
            OfficialCandleExpansionRecommendation::KeepTrinity
        };
        return (status, recommendation, blockers);
    }
    if matches!(
        after_gap_map.gap_status,
        OfficialCandleGapStatus::DiagnosticOnlyGaps
    ) || after_gap_map.non_crypto_official_gap_count == 0 && after_gap_map.total_gaps > 0
        || after_counts.official_gap_count == 0 && after_counts.gap_count > 0
    {
        return (
            OfficialCandleExpansionFinalStatus::DiagnosticCandleCoverageOnly,
            OfficialCandleExpansionRecommendation::KeepTrinity,
            blockers,
        );
    }
    if before_gap_map.total_gaps == after_gap_map.total_gaps {
        blockers.push("gap count did not improve after bounded expansion".to_string());
    }
    (
        OfficialCandleExpansionFinalStatus::StillMissingOfficialCandles,
        OfficialCandleExpansionRecommendation::NeedMoreEvidence,
        blockers,
    )
}

fn maybe_run_core_scorecard(
    config: &OfficialCandleExpansionPlanConfig,
) -> Result<Option<CoreScorecardRerunSummary>, String> {
    if !config.run_core_scorecard_rerun {
        return Ok(None);
    }
    let previous = config
        .previous_core_scorecard_path
        .as_deref()
        .map(|path| crate::experiment::CorePerformanceScorecard::from_json_path(Path::new(path)))
        .transpose()?;
    let Some(path) = config.core_performance_config_path.as_deref() else {
        return Ok(Some(CoreScorecardRerun::missing(
            "scorecard rerun enabled without core_performance_config_path",
        )));
    };
    let current = CoreScorecardRerun::default().run_bundle(path)?;
    Ok(Some(CoreScorecardRerun::default().summarize(
        previous.as_ref(),
        Some(&current.scorecard),
        Vec::new(),
        true,
    )))
}

fn derive_bottleneck(
    gap_map: Option<&OfficialCandleCoverageGapMap>,
    counts: Option<&CandleExpansionCounts>,
    backfill_report: Option<&ComparableEvidenceBackfillReport>,
) -> Option<CoreBottleneckKind> {
    if gap_map.is_some_and(|map| map.total_gaps > 0) {
        return Some(CoreBottleneckKind::MissingOfficialCandles);
    }
    if backfill_report.is_some_and(|report| report.rows_still_summary_derived > 0) {
        return Some(CoreBottleneckKind::ScenarioMaterializationWeak);
    }
    if counts.is_some_and(|counts| counts.official_ready_matches == 0) {
        return Some(CoreBottleneckKind::MissingOfficialData);
    }
    Some(CoreBottleneckKind::NoBottleneckDetected)
}

fn build_bundle_storage_report(
    report: &OfficialCandleExpansionReport,
    closure: &CandleExpansionClosureReport,
    _output_dir: &Path,
    budget_bytes: usize,
) -> CandleExpansionStorageReport {
    let artifacts = vec![
        CandleExpansionArtifactSize {
            path: "candle_gap_map.txt".to_string(),
            bytes: report.gap_map.to_text().len(),
        },
        CandleExpansionArtifactSize {
            path: "acquisition_plan.txt".to_string(),
            bytes: report.acquisition_plan.to_text().len(),
        },
        CandleExpansionArtifactSize {
            path: "candle_expansion_report.txt".to_string(),
            bytes: report.to_text().len(),
        },
        CandleExpansionArtifactSize {
            path: "candle_expansion_closure.txt".to_string(),
            bytes: closure.to_text().len(),
        },
        CandleExpansionArtifactSize {
            path: "pack/official_candle_coverage_pack.json".to_string(),
            bytes: report
                .new_candle_pack
                .as_ref()
                .map(|pack| pack.storage_bytes)
                .unwrap_or(0),
        },
    ];
    build_candle_expansion_storage_report(budget_bytes, artifacts)
}

fn empty_pack() -> OfficialCandleCoveragePack {
    OfficialCandleCoveragePack::build(&OfficialCandleCoveragePackConfig::default()).unwrap_or_else(
        |_| OfficialCandleCoveragePack {
            pack_id: "empty-pack".to_string(),
            descriptors: Vec::new(),
            official_non_crypto_series: Vec::new(),
            official_crypto_series: Vec::new(),
            controlled_series: Vec::new(),
            yfinance_series: Vec::new(),
            fixture_series: Vec::new(),
            unknown_series: Vec::new(),
            total_rows: 0,
            total_symbols: 0,
            total_timeframes: 0,
            storage_bytes: 0,
            readiness_eligible_series_count: 0,
            benchmark_eligible_series_count: 0,
            warnings: Vec::new(),
            reason_codes: vec![ReasonCode::DeterministicPath],
        },
    )
}

fn run_local_preflight(
    job: &CandleAcquisitionJob,
    input_csv: &str,
) -> Result<PreflightReport, String> {
    let config = LocalDataOnboardingConfig {
        onboarding_id: format!("{}-preflight", job.job_id),
        input_path: input_csv.to_string(),
        output_root: Path::new(&job.output_root)
            .join("preflight")
            .display()
            .to_string(),
        symbol: Some(job.symbol.clone()),
        venue: Some(parse_market_venue(&job.market)),
        asset_class: Some(parse_asset_class(&job.market)),
        timeframe: Some(parse_timeframe(&job.timeframe)),
        source_kind: Some(EvidenceSourceKind::RealLocal),
        strict: true,
        allow_format_autodetect: true,
        allow_sort_repair: false,
        allow_duplicate_drop: false,
        reason_codes: vec![ReasonCode::DeterministicPath],
        ..LocalDataOnboardingConfig::default()
    };
    Ok(PreflightValidator::default().run(&config))
}

fn job_to_collection_entry(job: &CandleAcquisitionJob) -> OfficialCollectionEntry {
    OfficialCollectionEntry {
        entry_id: job.job_id.clone(),
        provider_kind: job.provider_kind.unwrap_or(ProviderKind::Upbit),
        symbol: job.symbol.clone(),
        normalized_symbol: Some(job.symbol.clone()),
        venue: Some(parse_market_venue(&job.market)),
        asset_class: parse_asset_class(&job.market),
        timeframe: parse_timeframe(&job.timeframe),
        start: job.start_timestamp_ms.map(|value| value.to_string()),
        end: job.end_timestamp_ms.map(|value| value.to_string()),
        max_rows: Some(job.max_rows),
        max_requests: Some(job.max_requests),
        outputsize: Some(CollectionOutputSize::Compact),
        auth_config_ref: job.provider_kind.map(auth_config_for_provider),
        endpoint_template: if job.provider_kind == Some(ProviderKind::KrxOpenApi) {
            std::env::var("KRX_ENDPOINT_TEMPLATE").ok()
        } else {
            None
        },
        fixture_path: None,
        enabled: true,
        tags: vec!["sprint43".to_string(), "research-only".to_string()],
        reason_codes: vec![ReasonCode::DeterministicPath],
    }
}

fn auth_config_for_provider(provider_kind: ProviderKind) -> AuthConfig {
    match provider_kind {
        ProviderKind::KrxOpenApi => AuthConfig {
            provider_kind,
            api_key_env_var: Some("KRX_API_KEY".to_string()),
            api_secret_env_var: None,
            auth_header_name: Some("Authorization".to_string()),
            query_param_name: None,
            allow_missing_for_mock: false,
            reason_codes: vec![ReasonCode::DeterministicPath],
        },
        ProviderKind::AlphaVantage => AuthConfig {
            provider_kind,
            api_key_env_var: Some("ALPHAVANTAGE_API_KEY".to_string()),
            api_secret_env_var: None,
            auth_header_name: None,
            query_param_name: Some("apikey".to_string()),
            allow_missing_for_mock: false,
            reason_codes: vec![ReasonCode::DeterministicPath],
        },
        ProviderKind::Alpaca => AuthConfig {
            provider_kind,
            api_key_env_var: Some("ALPACA_API_KEY".to_string()),
            api_secret_env_var: Some("ALPACA_SECRET_KEY".to_string()),
            auth_header_name: Some("APCA-API-KEY-ID".to_string()),
            query_param_name: None,
            allow_missing_for_mock: false,
            reason_codes: vec![ReasonCode::DeterministicPath],
        },
        ProviderKind::DataGoKrFscStockPrice => AuthConfig {
            provider_kind,
            api_key_env_var: Some("DATA_GO_KR_SERVICE_KEY".to_string()),
            api_secret_env_var: None,
            auth_header_name: None,
            query_param_name: Some("serviceKey".to_string()),
            allow_missing_for_mock: false,
            reason_codes: vec![ReasonCode::DeterministicPath],
        },
        _ => AuthConfig {
            provider_kind,
            api_key_env_var: None,
            api_secret_env_var: None,
            auth_header_name: None,
            query_param_name: None,
            allow_missing_for_mock: true,
            reason_codes: vec![ReasonCode::DeterministicPath],
        },
    }
}

fn parse_asset_class(market: &str) -> AssetClass {
    if market.contains("Crypto") {
        AssetClass::Crypto
    } else {
        AssetClass::Equity
    }
}

fn parse_market_venue(market: &str) -> MarketVenue {
    match market {
        value if value.contains("Korean") => MarketVenue::KOSPI,
        value if value.contains("US") => MarketVenue::NASDAQ,
        value if value.contains("Crypto") => MarketVenue::Upbit,
        _ => MarketVenue::Generic,
    }
}

fn parse_timeframe(value: &str) -> Timeframe {
    match value {
        "1m" => Timeframe::OneMinute,
        "5m" => Timeframe::FiveMinute,
        "15m" => Timeframe::FifteenMinute,
        "1h" => Timeframe::OneHour,
        "4h" => Timeframe::OneHour,
        _ => Timeframe::OneDay,
    }
}
