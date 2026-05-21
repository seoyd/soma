use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};

use super::comparable_committee_evidence::{
    ComparableCommitteeEvidenceBundle, ComparableCommitteeEvidenceConfig,
    ComparableCommitteeEvidenceRow,
};
use super::comparable_evidence_builder::ComparableEvidenceBuilder;
use super::gap_expansion_consistency::{
    GapExpansionConsistencyReport, build_gap_expansion_consistency_report,
};
use super::match_key_normalization::{
    MatchKeyNormalizationAggregate, MatchKeyNormalizationOptions,
    build_match_key_normalization_aggregate, load_symbol_alias_map, load_timeframe_alias_map,
    load_timestamp_policy_map,
};
use super::official_candle_coverage_pack::{
    OfficialCandleCoveragePack, OfficialCandleSeriesDescriptor, OfficialCandleSeriesSourceClass,
    load_pack_from_path_or_config,
};
use super::official_candle_expansion_plan::OfficialCandleExpansionPlanConfig;
use super::official_candle_expansion_runner::{
    OfficialCandleExpansionReport, OfficialCandleExpansionRunner,
};
use super::official_candle_gap_map::{
    OfficialCandleCoverageGapMap, load_gap_map_from_path_or_config,
};
use super::official_candle_lineage::{
    OfficialCandleLineageReport, build_official_candle_lineage_report,
};
use super::row_candle_candidate::{
    RowCandleCandidateOptions, RowCandleCandidateReport, build_row_candle_candidate_report,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OfficialCandleJoinAuditConfig {
    pub audit_id: String,
    #[serde(default)]
    pub comparable_evidence_bundle_paths: Vec<String>,
    #[serde(default)]
    pub scenario_pack_paths: Vec<String>,
    #[serde(default)]
    pub official_candle_gap_map_paths: Vec<String>,
    #[serde(default)]
    pub official_candle_expansion_report_paths: Vec<String>,
    #[serde(default)]
    pub candle_coverage_pack_paths: Vec<String>,
    #[serde(default)]
    pub candle_coverage_match_report_paths: Vec<String>,
    #[serde(default)]
    pub comparable_backfill_report_paths: Vec<String>,
    #[serde(default)]
    pub reference_pack_paths: Vec<String>,
    #[serde(default)]
    pub counterfactual_depth_closure_paths: Vec<String>,
    #[serde(default)]
    pub core_scorecard_paths: Vec<String>,
    #[serde(default)]
    pub symbol_alias_map_path: Option<String>,
    #[serde(default)]
    pub timeframe_alias_map_path: Option<String>,
    #[serde(default)]
    pub timestamp_policy_map_path: Option<String>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default = "default_max_rows")]
    pub max_rows: usize,
    #[serde(default = "default_max_symbols")]
    pub max_symbols: usize,
    #[serde(default = "default_max_bytes")]
    pub max_bytes: usize,
    #[serde(default = "default_true")]
    pub allow_explicit_symbol_alias: bool,
    #[serde(default = "default_true")]
    pub allow_explicit_timeframe_alias: bool,
    #[serde(default = "default_true")]
    pub allow_explicit_timestamp_policy_map: bool,
    #[serde(default = "default_true")]
    pub allow_session_daily_alignment: bool,
    #[serde(default = "default_true")]
    pub allow_timestamp_tolerance: bool,
    #[serde(default = "default_timestamp_tolerance_ms")]
    pub timestamp_tolerance_ms: u64,
    #[serde(default = "default_true")]
    pub require_exact_horizon_match: bool,
    #[serde(default = "default_true")]
    pub require_no_lookahead_safe: bool,
    #[serde(default = "default_true")]
    pub require_official_source_for_official_ready: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum OfficialCandleJoinAuditStatus {
    Healthy,
    Repairable,
    DiagnosticOnly,
    #[default]
    IncompleteArtifacts,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfficialCandleJoinAuditReport {
    pub audit_id: String,
    pub rows_scanned: usize,
    pub normalization_aggregate: MatchKeyNormalizationAggregate,
    pub candidate_report: RowCandleCandidateReport,
    pub consistency_report: GapExpansionConsistencyReport,
    pub lineage_report: OfficialCandleLineageReport,
    pub join_status: OfficialCandleJoinAuditStatus,
    pub unmatched_reason_taxonomy: Vec<String>,
    pub warnings: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OfficialCandleJoinAuditRunner;

impl Default for OfficialCandleJoinAuditConfig {
    fn default() -> Self {
        Self {
            audit_id: "official-candle-join-audit".to_string(),
            comparable_evidence_bundle_paths: Vec::new(),
            scenario_pack_paths: Vec::new(),
            official_candle_gap_map_paths: Vec::new(),
            official_candle_expansion_report_paths: Vec::new(),
            candle_coverage_pack_paths: Vec::new(),
            candle_coverage_match_report_paths: Vec::new(),
            comparable_backfill_report_paths: Vec::new(),
            reference_pack_paths: Vec::new(),
            counterfactual_depth_closure_paths: Vec::new(),
            core_scorecard_paths: Vec::new(),
            symbol_alias_map_path: None,
            timeframe_alias_map_path: None,
            timestamp_policy_map_path: None,
            output_root: default_output_root(),
            max_rows: default_max_rows(),
            max_symbols: default_max_symbols(),
            max_bytes: default_max_bytes(),
            allow_explicit_symbol_alias: true,
            allow_explicit_timeframe_alias: true,
            allow_explicit_timestamp_policy_map: true,
            allow_session_daily_alignment: true,
            allow_timestamp_tolerance: true,
            timestamp_tolerance_ms: default_timestamp_tolerance_ms(),
            require_exact_horizon_match: true,
            require_no_lookahead_safe: true,
            require_official_source_for_official_ready: true,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

impl OfficialCandleJoinAuditConfig {
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
        if self.audit_id.trim().is_empty() {
            return Err("official candle join audit id must not be empty".to_string());
        }
        if self
            .all_paths()
            .iter()
            .chain(std::iter::once(&self.output_root))
            .any(|path| path.contains("://"))
        {
            return Err("official candle join audit paths must be local".to_string());
        }
        if self.max_rows == 0 || self.max_rows > default_max_rows() {
            return Err(
                "official candle join audit max_rows must be between 1 and 500".to_string(),
            );
        }
        if self.max_symbols == 0 || self.max_symbols > default_max_symbols() {
            return Err(
                "official candle join audit max_symbols must be between 1 and 5".to_string(),
            );
        }
        if self.max_bytes == 0 || self.max_bytes > default_max_bytes() {
            return Err(
                "official candle join audit max_bytes must be between 1 and 5000000".to_string(),
            );
        }
        if self.timestamp_tolerance_ms > 86_400_000 {
            return Err(
                "official candle join audit timestamp_tolerance_ms must be bounded to one day"
                    .to_string(),
            );
        }
        Ok(())
    }

    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.audit_id)
    }

    pub fn all_paths(&self) -> Vec<String> {
        self.comparable_evidence_bundle_paths
            .iter()
            .chain(self.scenario_pack_paths.iter())
            .chain(self.official_candle_gap_map_paths.iter())
            .chain(self.official_candle_expansion_report_paths.iter())
            .chain(self.candle_coverage_pack_paths.iter())
            .chain(self.candle_coverage_match_report_paths.iter())
            .chain(self.comparable_backfill_report_paths.iter())
            .chain(self.reference_pack_paths.iter())
            .chain(self.counterfactual_depth_closure_paths.iter())
            .chain(self.core_scorecard_paths.iter())
            .chain(self.symbol_alias_map_path.iter())
            .chain(self.timeframe_alias_map_path.iter())
            .chain(self.timestamp_policy_map_path.iter())
            .cloned()
            .collect()
    }
}

impl OfficialCandleJoinAuditRunner {
    pub fn run(
        &self,
        config: &OfficialCandleJoinAuditConfig,
    ) -> Result<OfficialCandleJoinAuditReport, String> {
        config.validate()?;
        let rows = load_join_audit_rows(config)?;
        let pack = load_join_audit_pack(config)?;
        let gap_maps = load_join_audit_gap_maps(config)?;
        let expansion_reports = load_join_audit_expansion_reports(config)?;
        let symbol_alias_map = config
            .symbol_alias_map_path
            .as_deref()
            .map(load_symbol_alias_map)
            .transpose()?;
        let timeframe_alias_map = config
            .timeframe_alias_map_path
            .as_deref()
            .map(load_timeframe_alias_map)
            .transpose()?;
        let timestamp_policy_map = config
            .timestamp_policy_map_path
            .as_deref()
            .map(load_timestamp_policy_map)
            .transpose()?;
        let normalization_aggregate = build_match_key_normalization_aggregate(
            &rows,
            &MatchKeyNormalizationOptions {
                allow_explicit_symbol_alias: config.allow_explicit_symbol_alias,
                allow_explicit_timeframe_alias: config.allow_explicit_timeframe_alias,
                allow_explicit_timestamp_policy_map: config.allow_explicit_timestamp_policy_map,
            },
            symbol_alias_map.as_ref(),
            timeframe_alias_map.as_ref(),
            timestamp_policy_map.as_ref(),
        );
        let candidate_report = build_row_candle_candidate_report(
            &rows,
            &pack,
            &normalization_aggregate,
            &RowCandleCandidateOptions {
                allow_session_daily_alignment: config.allow_session_daily_alignment,
                allow_timestamp_tolerance: config.allow_timestamp_tolerance,
                timestamp_tolerance_ms: config.timestamp_tolerance_ms,
                require_no_lookahead_safe: config.require_no_lookahead_safe,
                require_exact_horizon_match: config.require_exact_horizon_match,
                require_official_source_for_official_ready: config
                    .require_official_source_for_official_ready,
            },
        );
        let consistency_report = build_gap_expansion_consistency_report(
            &gap_maps,
            &expansion_reports,
            &candidate_report,
        );
        let lineage_report = build_official_candle_lineage_report(
            &rows,
            &candidate_report,
            &gap_maps,
            &expansion_reports,
            &config.reference_pack_paths,
            &config.counterfactual_depth_closure_paths,
            &config.core_scorecard_paths,
        );
        let join_status = determine_join_status(&candidate_report, &rows, &pack);
        let unmatched_reason_taxonomy = build_unmatched_reason_taxonomy(&candidate_report);
        Ok(OfficialCandleJoinAuditReport {
            audit_id: config.audit_id.clone(),
            rows_scanned: rows.len(),
            normalization_aggregate,
            candidate_report,
            consistency_report,
            lineage_report,
            join_status,
            unmatched_reason_taxonomy,
            warnings: if pack.descriptors.is_empty() {
                vec!["official candle pack was empty; candidate matching is incomplete".to_string()]
            } else {
                Vec::new()
            },
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
        })
    }
}

impl OfficialCandleJoinAuditReport {
    pub fn to_text(&self) -> String {
        [
            format!("audit_id={}", self.audit_id),
            format!("rows_scanned={}", self.rows_scanned),
            format!("join_status={:?}", self.join_status),
            format!(
                "unmatched_reason_taxonomy={}",
                self.unmatched_reason_taxonomy.join(" | ")
            ),
            format!("warnings={}", self.warnings.join(" | ")),
            self.normalization_aggregate.to_text(),
            self.candidate_report.to_text(),
            self.consistency_report.to_text(),
            self.lineage_report.to_text(),
        ]
        .join("\n\n")
    }

    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("match_key_normalization.txt"),
            self.normalization_aggregate.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("row_candle_candidates.txt"),
            self.candidate_report.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("gap_expansion_consistency.txt"),
            self.consistency_report.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("official_candle_lineage.txt"),
            self.lineage_report.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("official_candle_join_audit.txt"),
            self.to_text(),
        )
        .map_err(|err| err.to_string())?;
        let json_path = output_dir.join("official_candle_join_audit.json");
        fs::write(&json_path, self.to_json_string()?).map_err(|err| err.to_string())?;
        Ok(json_path)
    }
}

pub fn load_join_audit_rows(
    config: &OfficialCandleJoinAuditConfig,
) -> Result<Vec<ComparableCommitteeEvidenceRow>, String> {
    let mut rows = Vec::new();
    for path in &config.comparable_evidence_bundle_paths {
        if path.ends_with(".toml") {
            let comparable_config =
                ComparableCommitteeEvidenceConfig::from_toml_path(Path::new(path))?;
            rows.extend(
                ComparableEvidenceBuilder::default()
                    .build(&comparable_config)?
                    .rows,
            );
        } else {
            rows.extend(ComparableCommitteeEvidenceBundle::from_json_path(Path::new(path))?.rows);
        }
    }
    rows.sort_by(|left, right| {
        left.row_id
            .cmp(&right.row_id)
            .then(left.symbol.cmp(&right.symbol))
            .then(left.timeframe.cmp(&right.timeframe))
            .then(left.timestamp_ms.cmp(&right.timestamp_ms))
    });
    let mut bounded_rows = Vec::new();
    let mut total_bytes = 0usize;
    let mut symbols = BTreeSet::new();
    for row in rows {
        let row_bytes = serde_json::to_string(&row)
            .map(|text| text.len())
            .unwrap_or_default();
        if bounded_rows.len() >= config.max_rows
            || total_bytes.saturating_add(row_bytes) > config.max_bytes
            || symbols.insert(row.symbol.clone()) && symbols.len() > config.max_symbols
        {
            break;
        }
        total_bytes = total_bytes.saturating_add(row_bytes);
        bounded_rows.push(row);
    }
    Ok(bounded_rows)
}

pub fn load_join_audit_pack(
    config: &OfficialCandleJoinAuditConfig,
) -> Result<OfficialCandleCoveragePack, String> {
    let packs = config
        .candle_coverage_pack_paths
        .iter()
        .map(|path| load_pack_from_path_or_config(path))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(merge_official_packs(&config.audit_id, packs))
}

pub fn load_join_audit_gap_maps(
    config: &OfficialCandleJoinAuditConfig,
) -> Result<Vec<OfficialCandleCoverageGapMap>, String> {
    config
        .official_candle_gap_map_paths
        .iter()
        .map(|path| load_gap_map_from_path_or_config(path))
        .collect()
}

pub fn load_join_audit_expansion_reports(
    config: &OfficialCandleJoinAuditConfig,
) -> Result<Vec<OfficialCandleExpansionReport>, String> {
    config
        .official_candle_expansion_report_paths
        .iter()
        .map(|path| load_expansion_report(path))
        .collect()
}

pub fn merge_official_packs(
    pack_id: &str,
    packs: Vec<OfficialCandleCoveragePack>,
) -> OfficialCandleCoveragePack {
    let mut descriptors = packs
        .into_iter()
        .flat_map(|pack| pack.descriptors)
        .collect::<Vec<_>>();
    descriptors.sort_by(|left, right| {
        left.candle_series_id
            .cmp(&right.candle_series_id)
            .then(left.path.cmp(&right.path))
    });
    descriptors.dedup_by(|left, right| {
        left.candle_series_id == right.candle_series_id && left.path == right.path
    });
    build_pack_from_descriptors(pack_id, descriptors)
}

fn build_pack_from_descriptors(
    pack_id: &str,
    descriptors: Vec<OfficialCandleSeriesDescriptor>,
) -> OfficialCandleCoveragePack {
    let total_rows = descriptors.iter().map(|entry| entry.row_count).sum();
    let storage_bytes = descriptors.iter().map(|entry| entry.storage_bytes).sum();
    let total_symbols = descriptors
        .iter()
        .map(|entry| entry.normalized_symbol.clone())
        .collect::<BTreeSet<_>>()
        .len();
    let total_timeframes = descriptors
        .iter()
        .map(|entry| entry.timeframe.clone())
        .collect::<BTreeSet<_>>()
        .len();
    let readiness_eligible_series_count = descriptors
        .iter()
        .filter(|entry| entry.official_readiness_eligible)
        .count();
    let benchmark_eligible_series_count = descriptors
        .iter()
        .filter(|entry| entry.benchmark_eligible)
        .count();
    let select = |class: OfficialCandleSeriesSourceClass| {
        descriptors
            .iter()
            .filter(|entry| entry.source_class == class)
            .cloned()
            .collect::<Vec<_>>()
    };
    OfficialCandleCoveragePack {
        pack_id: format!("{}-merged-pack", pack_id),
        descriptors: descriptors.clone(),
        official_non_crypto_series: select(OfficialCandleSeriesSourceClass::OfficialNonCrypto),
        official_crypto_series: select(OfficialCandleSeriesSourceClass::OfficialCryptoOnly),
        controlled_series: select(OfficialCandleSeriesSourceClass::ControlledDiagnostic),
        yfinance_series: select(OfficialCandleSeriesSourceClass::YFinanceResearch),
        fixture_series: select(OfficialCandleSeriesSourceClass::FixtureArchitectureTest),
        unknown_series: select(OfficialCandleSeriesSourceClass::Unknown),
        total_rows,
        total_symbols,
        total_timeframes,
        storage_bytes,
        readiness_eligible_series_count,
        benchmark_eligible_series_count,
        warnings: Vec::new(),
        reason_codes: stable_reason_codes(&[
            ReasonCode::OfficialCandleCoverageBuilt,
            ReasonCode::DeterministicPath,
        ]),
    }
}

fn load_expansion_report(path: &str) -> Result<OfficialCandleExpansionReport, String> {
    if path.ends_with(".toml") {
        let config = OfficialCandleExpansionPlanConfig::from_toml_path(Path::new(path))?;
        OfficialCandleExpansionRunner::default().run(&config)
    } else {
        let text = fs::read_to_string(Path::new(path)).map_err(|err| err.to_string())?;
        serde_json::from_str(&text).map_err(|err| err.to_string())
    }
}

fn build_unmatched_reason_taxonomy(report: &RowCandleCandidateReport) -> Vec<String> {
    let mut counts = BTreeMap::<String, usize>::new();
    for bucket in &report.candidates_by_row {
        *counts.entry(format!("{:?}", bucket.status)).or_default() += 1;
    }
    let mut items = counts.into_iter().collect::<Vec<_>>();
    items.sort_by(|left, right| right.1.cmp(&left.1).then(left.0.cmp(&right.0)));
    items
        .into_iter()
        .map(|(status, count)| format!("{status}:{count}"))
        .collect()
}

fn determine_join_status(
    candidate_report: &RowCandleCandidateReport,
    rows: &[ComparableCommitteeEvidenceRow],
    pack: &OfficialCandleCoveragePack,
) -> OfficialCandleJoinAuditStatus {
    if rows.is_empty() || pack.descriptors.is_empty() {
        return OfficialCandleJoinAuditStatus::IncompleteArtifacts;
    }
    if candidate_report.official_ready_candidate_count > 0 {
        OfficialCandleJoinAuditStatus::Healthy
    } else if rows.iter().all(|row| row.diagnostic_only) {
        OfficialCandleJoinAuditStatus::DiagnosticOnly
    } else {
        OfficialCandleJoinAuditStatus::Repairable
    }
}

fn default_output_root() -> String {
    "target/soma_candle_join_audit".to_string()
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

fn default_timestamp_tolerance_ms() -> u64 {
    60_000
}

fn default_true() -> bool {
    true
}
