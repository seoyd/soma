use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::core::{ReasonCode, stable_hash_string, stable_ordered_strings, stable_reason_codes};
use crate::league::{TrinityCommitteeOperationalLoopConfig, TrinityOperationalLoopRunner};
use crate::ui::{ControlTowerRefreshConfig, ControlTowerRefreshReport, ControlTowerRefreshRunner};

use super::operational_runbook::{
    OperationalRunbookConfig, OperationalRunbookReport, OperationalRunbookRunner,
};

fn default_output_root() -> String {
    "target/soma_kis_evidence_depth".to_string()
}

fn default_max_rows() -> usize {
    100_000
}

fn default_max_symbols() -> usize {
    64
}

fn default_max_timeframes() -> usize {
    16
}

fn default_max_horizons() -> usize {
    16
}

fn default_max_artifacts() -> usize {
    64
}

fn default_max_bytes() -> usize {
    20_000_000
}

fn default_true() -> bool {
    true
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KISEvidenceDepthRunConfig {
    pub run_id: String,
    #[serde(default)]
    pub kis_activation_report_paths: Vec<String>,
    #[serde(default)]
    pub kis_collection_closure_paths: Vec<String>,
    #[serde(default)]
    pub kis_candle_sufficiency_paths: Vec<String>,
    #[serde(default)]
    pub kis_outcome_link_closure_paths: Vec<String>,
    #[serde(default)]
    pub official_evidence_scaleout_paths: Vec<String>,
    #[serde(default)]
    pub official_evidence_diversity_paths: Vec<String>,
    #[serde(default)]
    pub complete_row_closure_paths: Vec<String>,
    #[serde(default)]
    pub core_scorecard_paths: Vec<String>,
    #[serde(default)]
    pub trinity_loop_config_paths: Vec<String>,
    #[serde(default)]
    pub control_tower_config_paths: Vec<String>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default = "default_max_rows")]
    pub max_rows: usize,
    #[serde(default = "default_max_symbols")]
    pub max_symbols: usize,
    #[serde(default = "default_max_timeframes")]
    pub max_timeframes: usize,
    #[serde(default = "default_max_horizons")]
    pub max_horizons: usize,
    #[serde(default = "default_max_artifacts")]
    pub max_artifacts: usize,
    #[serde(default = "default_max_bytes")]
    pub max_bytes: usize,
    #[serde(default = "default_true")]
    pub run_kis_candle_sufficiency: bool,
    #[serde(default = "default_true")]
    pub run_outcome_linkage: bool,
    #[serde(default = "default_true")]
    pub run_counterfactual_completion: bool,
    #[serde(default = "default_true")]
    pub run_complete_row_closure: bool,
    #[serde(default = "default_true")]
    pub run_diversity_sweep: bool,
    #[serde(default = "default_true")]
    pub run_core_performance: bool,
    #[serde(default = "default_true")]
    pub run_trinity_operational_loop: bool,
    #[serde(default = "default_true")]
    pub run_control_tower_refresh: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum KISEvidenceDepthStatus {
    Improved,
    NeedMoreKISEvidence,
    NeedOutcomeLinkDepth,
    NeedCounterfactualDepth,
    NeedFutureWindows,
    NeedDiversity,
    NoImprovement,
    #[default]
    DiagnosticOnly,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum KISEvidenceDepthFinalRecommendation {
    RunKISCandleSufficiency,
    ImproveOutcomeLinkDepth,
    ImproveCounterfactualDepth,
    RunDiversitySweep,
    RunCorePerformance,
    RunTrinityLoop,
    RefreshControlTower,
    #[default]
    KeepTrinity,
    NeedMoreEvidence,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct KISEvidenceDepthReport {
    pub run_id: String,
    #[serde(default)]
    pub official_rows_before: Option<usize>,
    pub official_rows_after: usize,
    #[serde(default)]
    pub complete_rows_before: Option<usize>,
    pub complete_rows_after: usize,
    #[serde(default)]
    pub outcome_links_before: Option<usize>,
    pub outcome_links_after: usize,
    #[serde(default)]
    pub no_trade_counterfactuals_before: Option<usize>,
    pub no_trade_counterfactuals_after: usize,
    #[serde(default)]
    pub risk_denied_counterfactuals_before: Option<usize>,
    pub risk_denied_counterfactuals_after: usize,
    #[serde(default)]
    pub diversity_status_before: Option<String>,
    #[serde(default)]
    pub diversity_status_after: Option<String>,
    #[serde(default)]
    pub core_status_before: Option<String>,
    #[serde(default)]
    pub core_status_after: Option<String>,
    #[serde(default)]
    pub primary_bottleneck_before: Option<String>,
    pub primary_bottleneck_after: String,
    pub bottleneck_changed: bool,
    pub depth_status: KISEvidenceDepthStatus,
    pub final_recommendation: KISEvidenceDepthFinalRecommendation,
    #[serde(default)]
    pub blockers: Vec<String>,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
    pub fingerprint: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrinityLoopRefreshSummary {
    pub loop_ran: bool,
    pub generated_candidates: usize,
    pub paper_approved: usize,
    pub paper_open: usize,
    pub risk_blocked: usize,
    pub no_trade: usize,
    pub owner_review_pending: usize,
    pub final_status: String,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct KISEvidenceDepthStorageReport {
    pub max_bytes: usize,
    pub estimated_output_bytes: usize,
    pub within_budget: bool,
    pub file_count: usize,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct KISEvidenceDepthControlTowerBundle {
    pub kis_evidence_depth_report: KISEvidenceDepthReport,
    #[serde(default)]
    pub trinity_loop_refresh_summary: Option<TrinityLoopRefreshSummary>,
    pub control_tower_refresh_report: ControlTowerRefreshReport,
    pub operational_runbook_report: OperationalRunbookReport,
    pub storage_report: KISEvidenceDepthStorageReport,
    pub final_summary: String,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct KISEvidenceDepthRunRunner;

#[derive(Clone, Debug, Default)]
struct DepthSnapshot {
    official_rows: Option<usize>,
    complete_rows: Option<usize>,
    outcome_links: Option<usize>,
    no_trade_counterfactuals: Option<usize>,
    risk_denied_counterfactuals: Option<usize>,
    diversity_status: Option<String>,
    core_status: Option<String>,
    bottleneck: Option<String>,
    symbol_count: Option<usize>,
    timeframe_count: Option<usize>,
    horizon_count: Option<usize>,
}

impl Default for KISEvidenceDepthRunConfig {
    fn default() -> Self {
        Self {
            run_id: "sprint57-kis-evidence-depth".to_string(),
            kis_activation_report_paths: Vec::new(),
            kis_collection_closure_paths: Vec::new(),
            kis_candle_sufficiency_paths: Vec::new(),
            kis_outcome_link_closure_paths: Vec::new(),
            official_evidence_scaleout_paths: Vec::new(),
            official_evidence_diversity_paths: Vec::new(),
            complete_row_closure_paths: Vec::new(),
            core_scorecard_paths: Vec::new(),
            trinity_loop_config_paths: Vec::new(),
            control_tower_config_paths: Vec::new(),
            output_root: default_output_root(),
            max_rows: default_max_rows(),
            max_symbols: default_max_symbols(),
            max_timeframes: default_max_timeframes(),
            max_horizons: default_max_horizons(),
            max_artifacts: default_max_artifacts(),
            max_bytes: default_max_bytes(),
            run_kis_candle_sufficiency: true,
            run_outcome_linkage: true,
            run_counterfactual_completion: true,
            run_complete_row_closure: true,
            run_diversity_sweep: true,
            run_core_performance: true,
            run_trinity_operational_loop: true,
            run_control_tower_refresh: true,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

impl KISEvidenceDepthRunConfig {
    pub fn from_toml_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        toml::from_str(&text).map_err(|err| err.to_string())
    }

    pub fn to_toml_string(&self) -> Result<String, String> {
        toml::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.run_id.trim().is_empty() {
            return Err("kis evidence depth run id must not be empty".to_string());
        }
        if self
            .all_input_paths()
            .iter()
            .chain(std::iter::once(&self.output_root))
            .any(|path| path.contains("://"))
        {
            return Err("kis evidence depth paths must be local".to_string());
        }
        if self.max_rows == 0 || self.max_rows > 1_000_000 {
            return Err("kis evidence depth max_rows must be between 1 and 1000000".to_string());
        }
        if self.max_symbols == 0 || self.max_symbols > 1024 {
            return Err("kis evidence depth max_symbols must be between 1 and 1024".to_string());
        }
        if self.max_timeframes == 0 || self.max_timeframes > 256 {
            return Err("kis evidence depth max_timeframes must be between 1 and 256".to_string());
        }
        if self.max_horizons == 0 || self.max_horizons > 256 {
            return Err("kis evidence depth max_horizons must be between 1 and 256".to_string());
        }
        if self.max_artifacts == 0 || self.max_artifacts > 512 {
            return Err("kis evidence depth max_artifacts must be between 1 and 512".to_string());
        }
        if self.max_bytes == 0 || self.max_bytes > 20_000_000 {
            return Err("kis evidence depth max_bytes must be between 1 and 20000000".to_string());
        }
        Ok(())
    }

    pub fn artifact_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.run_id)
    }

    pub fn all_input_paths(&self) -> Vec<String> {
        stable_ordered_strings(
            &self
                .kis_activation_report_paths
                .iter()
                .chain(self.kis_collection_closure_paths.iter())
                .chain(self.kis_candle_sufficiency_paths.iter())
                .chain(self.kis_outcome_link_closure_paths.iter())
                .chain(self.official_evidence_scaleout_paths.iter())
                .chain(self.official_evidence_diversity_paths.iter())
                .chain(self.complete_row_closure_paths.iter())
                .chain(self.core_scorecard_paths.iter())
                .chain(self.trinity_loop_config_paths.iter())
                .chain(self.control_tower_config_paths.iter())
                .cloned()
                .collect::<Vec<_>>(),
        )
    }
}

impl KISEvidenceDepthReport {
    pub fn stabilize(&mut self) {
        self.blockers = stable_ordered_strings(&self.blockers);
        self.warnings = stable_ordered_strings(&self.warnings);
        self.reason_codes = stable_reason_codes(&self.reason_codes);
        self.fingerprint = stable_hash_string(&serde_json::to_string(self).unwrap_or_default());
    }

    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn from_json_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        serde_json::from_str(&text).map_err(|err| err.to_string())
    }

    pub fn to_text(&self) -> String {
        [
            "research_only_warning=kis evidence depth improvement is local research-only and does not imply profitability or live readiness"
                .to_string(),
            format!("run_id={}", self.run_id),
            format!("official_rows_before={}", self.official_rows_before.unwrap_or_default()),
            format!("official_rows_after={}", self.official_rows_after),
            format!("complete_rows_before={}", self.complete_rows_before.unwrap_or_default()),
            format!("complete_rows_after={}", self.complete_rows_after),
            format!("outcome_links_before={}", self.outcome_links_before.unwrap_or_default()),
            format!("outcome_links_after={}", self.outcome_links_after),
            format!(
                "no_trade_counterfactuals_before={}",
                self.no_trade_counterfactuals_before.unwrap_or_default()
            ),
            format!(
                "no_trade_counterfactuals_after={}",
                self.no_trade_counterfactuals_after
            ),
            format!(
                "risk_denied_counterfactuals_before={}",
                self.risk_denied_counterfactuals_before.unwrap_or_default()
            ),
            format!(
                "risk_denied_counterfactuals_after={}",
                self.risk_denied_counterfactuals_after
            ),
            format!(
                "diversity_status_before={}",
                self.diversity_status_before.clone().unwrap_or_default()
            ),
            format!(
                "diversity_status_after={}",
                self.diversity_status_after.clone().unwrap_or_default()
            ),
            format!("core_status_before={}", self.core_status_before.clone().unwrap_or_default()),
            format!("core_status_after={}", self.core_status_after.clone().unwrap_or_default()),
            format!(
                "primary_bottleneck_before={}",
                self.primary_bottleneck_before.clone().unwrap_or_default()
            ),
            format!("primary_bottleneck_after={}", self.primary_bottleneck_after),
            format!("bottleneck_changed={}", self.bottleneck_changed),
            format!("depth_status={:?}", self.depth_status),
            format!("final_recommendation={:?}", self.final_recommendation),
            format!("fingerprint={}", self.fingerprint),
        ]
        .join("\n")
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("kis_evidence_depth_report.txt"),
            self.to_text(),
        )
        .map_err(|err| err.to_string())?;
        let json_path = output_dir.join("kis_evidence_depth_report.json");
        fs::write(&json_path, self.to_json_string()?).map_err(|err| err.to_string())?;
        Ok(json_path)
    }
}

impl TrinityLoopRefreshSummary {
    pub fn to_text(&self) -> String {
        [
            format!("loop_ran={}", self.loop_ran),
            format!("generated_candidates={}", self.generated_candidates),
            format!("paper_approved={}", self.paper_approved),
            format!("paper_open={}", self.paper_open),
            format!("risk_blocked={}", self.risk_blocked),
            format!("no_trade={}", self.no_trade),
            format!("owner_review_pending={}", self.owner_review_pending),
            format!("final_status={}", self.final_status),
        ]
        .join("\n")
    }
}

impl KISEvidenceDepthStorageReport {
    pub fn to_text(&self) -> String {
        [
            format!("max_bytes={}", self.max_bytes),
            format!("estimated_output_bytes={}", self.estimated_output_bytes),
            format!("within_budget={}", self.within_budget),
            format!("file_count={}", self.file_count),
        ]
        .join("\n")
    }
}

impl KISEvidenceDepthControlTowerBundle {
    pub fn write_to_dir(&self, output_dir: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        self.kis_evidence_depth_report.write_to_dir(output_dir)?;
        fs::write(
            output_dir.join("trinity_loop_refresh_summary.txt"),
            self.trinity_loop_refresh_summary
                .as_ref()
                .map(TrinityLoopRefreshSummary::to_text)
                .unwrap_or_else(|| "loop_ran=false".to_string()),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("control_tower_refresh_report.txt"),
            self.control_tower_refresh_report.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("operational_runbook.txt"),
            self.operational_runbook_report.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("storage_report.txt"),
            self.storage_report.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(output_dir.join("summary.txt"), &self.final_summary)
            .map_err(|err| err.to_string())?;
        let json_path = output_dir.join("kis_evidence_depth_control_tower_bundle.json");
        fs::write(
            &json_path,
            serde_json::to_string_pretty(self).map_err(|err| err.to_string())?,
        )
        .map_err(|err| err.to_string())?;
        Ok(json_path)
    }
}

impl KISEvidenceDepthRunRunner {
    pub fn run(
        &self,
        config: &KISEvidenceDepthRunConfig,
        config_path: Option<&Path>,
    ) -> Result<KISEvidenceDepthControlTowerBundle, String> {
        config.validate()?;

        let mut warnings = Vec::new();
        let mut blockers = Vec::new();
        let mut reason_codes = config.reason_codes.clone();
        reason_codes.push(ReasonCode::OfficialEvidenceCounted);

        let snapshots = load_snapshots(config, &mut warnings, &mut blockers, &mut reason_codes)?;
        let report = build_depth_report(config, snapshots, warnings, blockers, reason_codes);

        let trinity_summary = if config.run_trinity_operational_loop {
            config
                .trinity_loop_config_paths
                .first()
                .map(|path| {
                    let loop_config =
                        TrinityCommitteeOperationalLoopConfig::from_toml_path(Path::new(path))?;
                    let loop_bundle = TrinityOperationalLoopRunner::default().run(&loop_config)?;
                    Ok::<_, String>(TrinityLoopRefreshSummary {
                        loop_ran: true,
                        generated_candidates: loop_bundle.report.generated_candidate_count,
                        paper_approved: loop_bundle.report.paper_approved_count,
                        paper_open: loop_bundle.report.paper_position_open_count,
                        risk_blocked: loop_bundle.report.risk_blocked_count,
                        no_trade: loop_bundle.report.no_trade_count,
                        owner_review_pending: loop_bundle.report.human_confirm_required_count,
                        final_status: format!("{:?}", loop_bundle.report.final_status),
                        reason_codes: stable_reason_codes(&[
                            ReasonCode::DeterministicPath,
                            ReasonCode::PaperExecutionOnly,
                        ]),
                    })
                })
                .transpose()?
        } else {
            None
        };

        let runbook_config = OperationalRunbookConfig {
            runbook_id: format!("{}-runbook", config.run_id),
            kis_evidence_depth_config_path: config_path.map(|path| path.display().to_string()),
            control_tower_refresh_config_path: config.control_tower_config_paths.first().cloned(),
            trinity_loop_config_path: config.trinity_loop_config_paths.first().cloned(),
            output_root: config.output_root.clone(),
            include_commands: true,
            include_expected_artifacts: true,
            include_blockers: true,
            include_risk_notes: true,
            reason_codes: vec![ReasonCode::DeterministicPath],
        };
        let operational_runbook_report =
            OperationalRunbookRunner::default().run(&runbook_config)?;

        let control_tower_refresh_report = if config.run_control_tower_refresh {
            if let Some(path) = config.control_tower_config_paths.first() {
                let refresh_config = ControlTowerRefreshConfig::from_toml_path(Path::new(path))?;
                ControlTowerRefreshRunner::default()
                    .run(
                        &refresh_config,
                        Some(Path::new(path)),
                        Some(&report),
                        Some(&operational_runbook_report),
                    )?
                    .report
            } else {
                ControlTowerRefreshReport::diagnostic_only(
                    format!("{}-refresh", config.run_id),
                    config.output_root.clone(),
                )
            }
        } else {
            ControlTowerRefreshReport::diagnostic_only(
                format!("{}-refresh", config.run_id),
                config.output_root.clone(),
            )
        };

        let storage_report = build_storage_report(
            config.max_bytes,
            &report,
            trinity_summary.as_ref(),
            &control_tower_refresh_report,
            &operational_runbook_report,
        );
        let final_summary = build_final_summary(
            &report,
            trinity_summary.as_ref(),
            &control_tower_refresh_report,
            &operational_runbook_report,
        );

        let bundle = KISEvidenceDepthControlTowerBundle {
            kis_evidence_depth_report: report,
            trinity_loop_refresh_summary: trinity_summary,
            control_tower_refresh_report,
            operational_runbook_report,
            storage_report,
            final_summary,
            reason_codes: stable_reason_codes(&[
                ReasonCode::OfficialEvidenceCounted,
                ReasonCode::DeterministicPath,
                ReasonCode::LocalFileOnly,
            ]),
        };
        bundle.write_to_dir(&config.artifact_dir())?;
        Ok(bundle)
    }
}

fn load_snapshots(
    config: &KISEvidenceDepthRunConfig,
    warnings: &mut Vec<String>,
    blockers: &mut Vec<String>,
    reason_codes: &mut Vec<ReasonCode>,
) -> Result<Vec<DepthSnapshot>, String> {
    let ordered_groups = [
        &config.kis_activation_report_paths,
        &config.kis_collection_closure_paths,
        &config.kis_candle_sufficiency_paths,
        &config.kis_outcome_link_closure_paths,
        &config.official_evidence_scaleout_paths,
        &config.official_evidence_diversity_paths,
        &config.complete_row_closure_paths,
        &config.core_scorecard_paths,
    ];
    let ordered_paths = ordered_groups
        .iter()
        .flat_map(|paths| paths.iter().cloned())
        .collect::<Vec<_>>();
    if ordered_paths.len() > config.max_artifacts {
        warnings.push(format!(
            "artifact limit applied: {} > {}",
            ordered_paths.len(),
            config.max_artifacts
        ));
        reason_codes.push(ReasonCode::CollectionBudgetExceeded);
    }
    let mut total_bytes = 0usize;
    let mut snapshots = Vec::new();
    for path in ordered_paths.into_iter().take(config.max_artifacts) {
        match fs::read_to_string(&path) {
            Ok(text) => {
                total_bytes += text.len();
                match serde_json::from_str::<Value>(&text) {
                    Ok(value) => snapshots.push(snapshot_from_value(&value)),
                    Err(err) => {
                        warnings.push(format!("failed to parse {}: {}", path, err));
                        reason_codes.push(ReasonCode::DataLoadFailed);
                    }
                }
            }
            Err(_) => {
                warnings.push(format!("missing {}", path));
                reason_codes.push(ReasonCode::MissingFile);
            }
        }
    }
    if total_bytes > config.max_bytes {
        blockers.push(format!(
            "input bytes exceed budget: {} > {}",
            total_bytes, config.max_bytes
        ));
        reason_codes.push(ReasonCode::BudgetExceeded);
    }
    Ok(snapshots)
}

fn build_depth_report(
    config: &KISEvidenceDepthRunConfig,
    snapshots: Vec<DepthSnapshot>,
    mut warnings: Vec<String>,
    mut blockers: Vec<String>,
    mut reason_codes: Vec<ReasonCode>,
) -> KISEvidenceDepthReport {
    let official_rows_before = first_metric(&snapshots, |item| item.official_rows);
    let mut official_rows_after =
        last_metric(&snapshots, |item| item.official_rows).unwrap_or_default();
    let complete_rows_before = first_metric(&snapshots, |item| item.complete_rows);
    let mut complete_rows_after =
        last_metric(&snapshots, |item| item.complete_rows).unwrap_or_default();
    let outcome_links_before = first_metric(&snapshots, |item| item.outcome_links);
    let outcome_links_after =
        last_metric(&snapshots, |item| item.outcome_links).unwrap_or_default();
    let no_trade_counterfactuals_before =
        first_metric(&snapshots, |item| item.no_trade_counterfactuals);
    let no_trade_counterfactuals_after =
        last_metric(&snapshots, |item| item.no_trade_counterfactuals).unwrap_or_default();
    let risk_denied_counterfactuals_before =
        first_metric(&snapshots, |item| item.risk_denied_counterfactuals);
    let risk_denied_counterfactuals_after =
        last_metric(&snapshots, |item| item.risk_denied_counterfactuals).unwrap_or_default();
    let diversity_status_before = first_text(&snapshots, |item| item.diversity_status.clone());
    let diversity_status_after = last_text(&snapshots, |item| item.diversity_status.clone());
    let core_status_before = first_text(&snapshots, |item| item.core_status.clone());
    let core_status_after = last_text(&snapshots, |item| item.core_status.clone());
    let primary_bottleneck_before = first_text(&snapshots, |item| item.bottleneck.clone());
    let primary_bottleneck_after = last_text(&snapshots, |item| item.bottleneck.clone())
        .unwrap_or_else(|| "NeedMoreEvidence".to_string());
    let bottleneck_changed =
        primary_bottleneck_before.as_deref().unwrap_or_default() != primary_bottleneck_after;

    if let Some(symbol_count) = last_metric(&snapshots, |item| item.symbol_count) {
        if symbol_count > config.max_symbols {
            blockers.push(format!(
                "symbol count {} exceeds max_symbols {}",
                symbol_count, config.max_symbols
            ));
            reason_codes.push(ReasonCode::CollectionBudgetExceeded);
        }
    }
    if let Some(timeframe_count) = last_metric(&snapshots, |item| item.timeframe_count) {
        if timeframe_count > config.max_timeframes {
            blockers.push(format!(
                "timeframe count {} exceeds max_timeframes {}",
                timeframe_count, config.max_timeframes
            ));
            reason_codes.push(ReasonCode::CollectionBudgetExceeded);
        }
    }
    if let Some(horizon_count) = last_metric(&snapshots, |item| item.horizon_count) {
        if horizon_count > config.max_horizons {
            blockers.push(format!(
                "horizon count {} exceeds max_horizons {}",
                horizon_count, config.max_horizons
            ));
            reason_codes.push(ReasonCode::CollectionBudgetExceeded);
        }
    }

    official_rows_after = official_rows_after.min(config.max_rows);
    complete_rows_after = complete_rows_after.min(config.max_rows);

    let depth_status = determine_depth_status(
        official_rows_after,
        complete_rows_after,
        outcome_links_after,
        no_trade_counterfactuals_after,
        risk_denied_counterfactuals_after,
        diversity_status_after.as_deref(),
        official_rows_before,
        complete_rows_before,
        outcome_links_before,
        no_trade_counterfactuals_before,
        risk_denied_counterfactuals_before,
    );
    let final_recommendation = determine_final_recommendation(
        depth_status,
        config,
        core_status_after.as_deref(),
        &blockers,
    );

    if matches!(depth_status, KISEvidenceDepthStatus::DiagnosticOnly) {
        warnings.push("no reliable official evidence depth input was loaded".to_string());
        reason_codes.push(ReasonCode::ResearchOnlyOverride);
    }
    if matches!(
        depth_status,
        KISEvidenceDepthStatus::NeedMoreKISEvidence
            | KISEvidenceDepthStatus::NeedOutcomeLinkDepth
            | KISEvidenceDepthStatus::NeedCounterfactualDepth
            | KISEvidenceDepthStatus::NeedFutureWindows
            | KISEvidenceDepthStatus::NeedDiversity
    ) {
        reason_codes.push(ReasonCode::EvidenceStillInsufficient);
    }

    let mut report = KISEvidenceDepthReport {
        run_id: config.run_id.clone(),
        official_rows_before,
        official_rows_after,
        complete_rows_before,
        complete_rows_after,
        outcome_links_before,
        outcome_links_after,
        no_trade_counterfactuals_before,
        no_trade_counterfactuals_after,
        risk_denied_counterfactuals_before,
        risk_denied_counterfactuals_after,
        diversity_status_before,
        diversity_status_after,
        core_status_before,
        core_status_after,
        primary_bottleneck_before,
        primary_bottleneck_after,
        bottleneck_changed,
        depth_status,
        final_recommendation,
        blockers,
        warnings,
        reason_codes: stable_reason_codes(&reason_codes),
        fingerprint: String::new(),
    };
    report.stabilize();
    report
}

fn determine_depth_status(
    official_rows_after: usize,
    complete_rows_after: usize,
    outcome_links_after: usize,
    no_trade_after: usize,
    risk_denied_after: usize,
    diversity_status_after: Option<&str>,
    official_rows_before: Option<usize>,
    complete_rows_before: Option<usize>,
    outcome_links_before: Option<usize>,
    no_trade_before: Option<usize>,
    risk_denied_before: Option<usize>,
) -> KISEvidenceDepthStatus {
    if official_rows_after == 0
        && complete_rows_after == 0
        && outcome_links_after == 0
        && no_trade_after == 0
        && risk_denied_after == 0
    {
        KISEvidenceDepthStatus::DiagnosticOnly
    } else if official_rows_after == 0 || complete_rows_after == 0 {
        KISEvidenceDepthStatus::NeedMoreKISEvidence
    } else if complete_rows_after < official_rows_after {
        KISEvidenceDepthStatus::NeedFutureWindows
    } else if outcome_links_after < 8 {
        KISEvidenceDepthStatus::NeedOutcomeLinkDepth
    } else if no_trade_after + risk_denied_after < 4 {
        KISEvidenceDepthStatus::NeedCounterfactualDepth
    } else if diversity_status_after
        .map(|value| {
            let value = value.to_ascii_lowercase();
            value.contains("need") || value.contains("weak") || value.contains("missing")
        })
        .unwrap_or(false)
    {
        KISEvidenceDepthStatus::NeedDiversity
    } else if official_rows_after > official_rows_before.unwrap_or_default()
        || complete_rows_after > complete_rows_before.unwrap_or_default()
        || outcome_links_after > outcome_links_before.unwrap_or_default()
        || no_trade_after > no_trade_before.unwrap_or_default()
        || risk_denied_after > risk_denied_before.unwrap_or_default()
    {
        KISEvidenceDepthStatus::Improved
    } else {
        KISEvidenceDepthStatus::NoImprovement
    }
}

fn determine_final_recommendation(
    depth_status: KISEvidenceDepthStatus,
    config: &KISEvidenceDepthRunConfig,
    core_status_after: Option<&str>,
    blockers: &[String],
) -> KISEvidenceDepthFinalRecommendation {
    if !blockers.is_empty() {
        return KISEvidenceDepthFinalRecommendation::NeedMoreEvidence;
    }
    match depth_status {
        KISEvidenceDepthStatus::NeedMoreKISEvidence | KISEvidenceDepthStatus::NeedFutureWindows => {
            KISEvidenceDepthFinalRecommendation::RunKISCandleSufficiency
        }
        KISEvidenceDepthStatus::NeedOutcomeLinkDepth => {
            KISEvidenceDepthFinalRecommendation::ImproveOutcomeLinkDepth
        }
        KISEvidenceDepthStatus::NeedCounterfactualDepth => {
            KISEvidenceDepthFinalRecommendation::ImproveCounterfactualDepth
        }
        KISEvidenceDepthStatus::NeedDiversity => {
            KISEvidenceDepthFinalRecommendation::RunDiversitySweep
        }
        KISEvidenceDepthStatus::NoImprovement | KISEvidenceDepthStatus::DiagnosticOnly => {
            KISEvidenceDepthFinalRecommendation::NeedMoreEvidence
        }
        KISEvidenceDepthStatus::Improved => {
            if config.run_control_tower_refresh {
                KISEvidenceDepthFinalRecommendation::RefreshControlTower
            } else if config.run_trinity_operational_loop {
                KISEvidenceDepthFinalRecommendation::RunTrinityLoop
            } else if core_status_after
                .map(|value| {
                    let value = value.to_ascii_lowercase();
                    value.contains("blocked") || value.contains("need")
                })
                .unwrap_or(false)
                && config.run_core_performance
            {
                KISEvidenceDepthFinalRecommendation::RunCorePerformance
            } else {
                KISEvidenceDepthFinalRecommendation::KeepTrinity
            }
        }
    }
}

fn build_storage_report(
    max_bytes: usize,
    report: &KISEvidenceDepthReport,
    trinity_summary: Option<&TrinityLoopRefreshSummary>,
    refresh_report: &ControlTowerRefreshReport,
    runbook_report: &OperationalRunbookReport,
) -> KISEvidenceDepthStorageReport {
    let estimated_output_bytes = report.to_text().len()
        + trinity_summary
            .map(TrinityLoopRefreshSummary::to_text)
            .unwrap_or_default()
            .len()
        + refresh_report.to_text().len()
        + runbook_report.to_text().len();
    KISEvidenceDepthStorageReport {
        max_bytes,
        estimated_output_bytes,
        within_budget: estimated_output_bytes <= max_bytes,
        file_count: 6,
        reason_codes: stable_reason_codes(&[
            ReasonCode::StorageBudgetReportBuilt,
            ReasonCode::DeterministicPath,
        ]),
    }
}

fn build_final_summary(
    report: &KISEvidenceDepthReport,
    trinity_summary: Option<&TrinityLoopRefreshSummary>,
    refresh_report: &ControlTowerRefreshReport,
    runbook_report: &OperationalRunbookReport,
) -> String {
    [
        format!("run_id={}", report.run_id),
        format!("depth_status={:?}", report.depth_status),
        format!("final_recommendation={:?}", report.final_recommendation),
        format!(
            "trinity_final_status={}",
            trinity_summary
                .map(|summary| summary.final_status.clone())
                .unwrap_or_else(|| "NotRun".to_string())
        ),
        format!("refresh_status={:?}", refresh_report.refresh_status),
        format!("runbook_status={:?}", runbook_report.final_status),
        "research_only_warning=evidence depth improvements do not imply profitability, production readiness, or live trading readiness"
            .to_string(),
    ]
    .join("\n")
}

fn snapshot_from_value(value: &Value) -> DepthSnapshot {
    DepthSnapshot {
        official_rows: usize_field(
            value,
            &[
                "official_rows",
                "official_row_count",
                "official_rows_after",
                "after_official_rows",
            ],
        ),
        complete_rows: usize_field(
            value,
            &[
                "complete_rows",
                "official_complete_rows",
                "complete_rows_after",
                "after_complete_rows",
                "after_official_complete_rows",
            ],
        ),
        outcome_links: usize_field(
            value,
            &[
                "outcome_links",
                "outcome_links_after",
                "generated_outcome_links",
            ],
        ),
        no_trade_counterfactuals: usize_field(
            value,
            &[
                "no_trade_counterfactuals",
                "no_trade_counterfactuals_after",
                "generated_no_trade_counterfactuals",
            ],
        ),
        risk_denied_counterfactuals: usize_field(
            value,
            &[
                "risk_denied_counterfactuals",
                "risk_denied_counterfactuals_after",
                "generated_risk_denied_counterfactuals",
            ],
        ),
        diversity_status: string_field(
            value,
            &["diversity_status", "current_outcome_diversity_status"],
        ),
        core_status: string_field(
            value,
            &[
                "core_status",
                "core_completion_status",
                "core_status_after",
                "final_status",
            ],
        ),
        bottleneck: string_field(
            value,
            &[
                "primary_bottleneck",
                "primary_bottleneck_after",
                "current_primary_bottleneck",
                "current_bottleneck",
            ],
        ),
        symbol_count: usize_field(value, &["symbol_count"]),
        timeframe_count: usize_field(value, &["timeframe_count"]),
        horizon_count: usize_field(value, &["horizon_count"]),
    }
}

fn usize_field(value: &Value, keys: &[&str]) -> Option<usize> {
    keys.iter().find_map(|key| {
        value
            .get(*key)
            .and_then(|item| item.as_u64())
            .map(|item| item as usize)
    })
}

fn string_field(value: &Value, keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|key| value.get(*key).and_then(|item| item.as_str()))
        .map(ToOwned::to_owned)
}

fn first_metric<F>(snapshots: &[DepthSnapshot], mut f: F) -> Option<usize>
where
    F: FnMut(&DepthSnapshot) -> Option<usize>,
{
    snapshots.iter().find_map(&mut f)
}

fn last_metric<F>(snapshots: &[DepthSnapshot], mut f: F) -> Option<usize>
where
    F: FnMut(&DepthSnapshot) -> Option<usize>,
{
    snapshots.iter().rev().find_map(&mut f)
}

fn first_text<F>(snapshots: &[DepthSnapshot], mut f: F) -> Option<String>
where
    F: FnMut(&DepthSnapshot) -> Option<String>,
{
    snapshots.iter().find_map(&mut f)
}

fn last_text<F>(snapshots: &[DepthSnapshot], mut f: F) -> Option<String>
where
    F: FnMut(&DepthSnapshot) -> Option<String>,
{
    snapshots.iter().rev().find_map(&mut f)
}
