use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_hash_string, stable_reason_codes};
use crate::data::ProviderMarket;

use super::candle_coverage_match::{
    CandleCoverageMatch, CandleCoverageMatchOptions, CandleCoverageMatchStatus,
    build_candle_coverage_match_computation,
};
use super::comparable_committee_evidence::{
    ComparableCommitteeEvidenceBundle, ComparableCommitteeEvidenceConfig,
    ComparableCommitteeEvidenceRow, ComparableEvidenceSourceClass,
};
use super::comparable_evidence_builder::ComparableEvidenceBuilder;
use super::official_candle_coverage_pack::{
    OfficialCandleCoveragePack, OfficialCandleCoveragePackConfig, OfficialCandleSeriesDescriptor,
    OfficialCandleSeriesSourceClass, load_pack_from_path_or_config, normalize_symbol,
    timeframe_seconds,
};
use super::timestamp_alignment_v2::TimestampAlignmentV2Status;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OfficialCandleGapConfig {
    pub gap_id: String,
    #[serde(default)]
    pub core_scorecard_paths: Vec<String>,
    #[serde(default)]
    pub comparable_evidence_bundle_paths: Vec<String>,
    #[serde(default)]
    pub candle_coverage_closure_paths: Vec<String>,
    #[serde(default)]
    pub candle_coverage_pack_paths: Vec<String>,
    #[serde(default)]
    pub official_replication_report_paths: Vec<String>,
    #[serde(default)]
    pub official_committee_benchmark_paths: Vec<String>,
    #[serde(default)]
    pub outcome_coverage_bundle_paths: Vec<String>,
    #[serde(default)]
    pub reference_pack_bundle_paths: Vec<String>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default)]
    pub target_markets: Vec<ProviderMarket>,
    #[serde(default)]
    pub target_symbols: Vec<String>,
    #[serde(default)]
    pub target_timeframes: Vec<String>,
    #[serde(default)]
    pub target_horizons_bars: Vec<usize>,
    #[serde(default = "default_max_gaps")]
    pub max_gaps: usize,
    #[serde(default = "default_max_symbols")]
    pub max_symbols: usize,
    #[serde(default = "default_max_timeframes")]
    pub max_timeframes: usize,
    #[serde(default = "default_max_bytes")]
    pub max_bytes: usize,
    #[serde(default = "default_true")]
    pub require_non_crypto_official: bool,
    #[serde(default = "default_true")]
    pub allow_crypto_only: bool,
    #[serde(default = "default_true")]
    pub allow_controlled_diagnostic: bool,
    #[serde(default)]
    pub allow_yfinance_research: bool,
    #[serde(default)]
    pub allow_fixture: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum OfficialCandleGapKind {
    MissingCandleSeries,
    MissingOfficialCandleSeries,
    MissingNonCryptoOfficialCandleSeries,
    MissingProvenance,
    MissingPreflight,
    MissingManifest,
    MissingFutureWindow,
    MissingTimeframe,
    TimeframeMismatch,
    TimestampMismatch,
    GapHeavy,
    DuplicateTimestamp,
    NoLookaheadViolation,
    SourceNotEligible,
    SummaryDerivedOnly,
    ResearchOnlySource,
    FixtureOnlySource,
    ControlledOnlySource,
    CryptoOnlySource,
    #[default]
    Unknown,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum OfficialCandleGapStatus {
    NoGapsDetected,
    MissingOfficialCandles,
    MissingNonCryptoOfficialCandles,
    MissingFutureWindows,
    TimestampAlignmentWeak,
    TimeframeAlignmentWeak,
    SourceIneligible,
    DiagnosticOnlyGaps,
    #[default]
    NeedMoreEvidence,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfficialCandleGapCell {
    pub market: ProviderMarket,
    pub symbol: String,
    pub normalized_symbol: String,
    #[serde(default)]
    pub venue: Option<String>,
    pub timeframe: String,
    pub horizon_bars: usize,
    #[serde(default)]
    pub source_kind: Option<String>,
    pub source_class: ComparableEvidenceSourceClass,
    pub row_count_impacted: usize,
    pub comparable_rows_impacted: usize,
    pub missing_future_bars: usize,
    #[serde(default)]
    pub required_start_timestamp_ms: Option<u64>,
    #[serde(default)]
    pub required_end_timestamp_ms: Option<u64>,
    pub required_min_rows: usize,
    pub gap_kinds: Vec<OfficialCandleGapKind>,
    pub buildable_from_existing_local_csv: bool,
    pub buildable_from_provider_collection: bool,
    pub requires_operator_action: bool,
    #[serde(default)]
    pub related_artifact_paths: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfficialCandleCoverageGapMap {
    pub gap_id: String,
    pub cells: Vec<OfficialCandleGapCell>,
    pub total_gaps: usize,
    pub official_gap_count: usize,
    pub non_crypto_official_gap_count: usize,
    pub crypto_gap_count: usize,
    pub diagnostic_gap_count: usize,
    pub research_only_gap_count: usize,
    pub fixture_gap_count: usize,
    pub buildable_gap_count: usize,
    pub operator_action_gap_count: usize,
    pub gap_status: OfficialCandleGapStatus,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct OfficialCandleGapInputs {
    pub rows: Vec<ComparableCommitteeEvidenceRow>,
    pub pack: OfficialCandleCoveragePack,
    pub canonical_csv_paths: Vec<String>,
    pub provenance_paths: Vec<String>,
    pub preflight_paths: Vec<String>,
    pub manifest_paths: Vec<String>,
}

#[derive(Default)]
struct GapAccumulator {
    row_count_impacted: usize,
    comparable_rows_impacted: usize,
    missing_future_bars: usize,
    required_start_timestamp_ms: Option<u64>,
    required_end_timestamp_ms: Option<u64>,
    required_min_rows: usize,
    gap_kinds: BTreeSet<OfficialCandleGapKind>,
    buildable_from_existing_local_csv: bool,
    buildable_from_provider_collection: bool,
    requires_operator_action: bool,
    related_artifact_paths: BTreeSet<String>,
    reason_codes: Vec<ReasonCode>,
}

impl Default for OfficialCandleGapConfig {
    fn default() -> Self {
        Self {
            gap_id: "official-candle-gap-map".to_string(),
            core_scorecard_paths: Vec::new(),
            comparable_evidence_bundle_paths: Vec::new(),
            candle_coverage_closure_paths: Vec::new(),
            candle_coverage_pack_paths: Vec::new(),
            official_replication_report_paths: Vec::new(),
            official_committee_benchmark_paths: Vec::new(),
            outcome_coverage_bundle_paths: Vec::new(),
            reference_pack_bundle_paths: Vec::new(),
            output_root: default_output_root(),
            target_markets: Vec::new(),
            target_symbols: Vec::new(),
            target_timeframes: Vec::new(),
            target_horizons_bars: Vec::new(),
            max_gaps: default_max_gaps(),
            max_symbols: default_max_symbols(),
            max_timeframes: default_max_timeframes(),
            max_bytes: default_max_bytes(),
            require_non_crypto_official: true,
            allow_crypto_only: true,
            allow_controlled_diagnostic: true,
            allow_yfinance_research: false,
            allow_fixture: false,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

impl OfficialCandleGapConfig {
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
        if self.gap_id.trim().is_empty() {
            return Err("official candle gap id must not be empty".to_string());
        }
        if self
            .all_paths()
            .iter()
            .chain(std::iter::once(&self.output_root))
            .any(|path| is_remote_path(path))
        {
            return Err("official candle gap paths must be local".to_string());
        }
        if self.max_gaps == 0 || self.max_gaps > default_max_gaps() {
            return Err("official candle gap max_gaps must be between 1 and 100".to_string());
        }
        if self.max_symbols == 0 || self.max_symbols > default_max_symbols() {
            return Err("official candle gap max_symbols must be between 1 and 5".to_string());
        }
        if self.max_timeframes == 0 || self.max_timeframes > default_max_timeframes() {
            return Err("official candle gap max_timeframes must be between 1 and 5".to_string());
        }
        if self.max_bytes == 0 || self.max_bytes > default_max_bytes() {
            return Err("official candle gap max_bytes must be between 1 and 5000000".to_string());
        }
        Ok(())
    }

    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.gap_id)
    }

    pub fn all_paths(&self) -> Vec<String> {
        self.core_scorecard_paths
            .iter()
            .chain(self.comparable_evidence_bundle_paths.iter())
            .chain(self.candle_coverage_closure_paths.iter())
            .chain(self.candle_coverage_pack_paths.iter())
            .chain(self.official_replication_report_paths.iter())
            .chain(self.official_committee_benchmark_paths.iter())
            .chain(self.outcome_coverage_bundle_paths.iter())
            .chain(self.reference_pack_bundle_paths.iter())
            .cloned()
            .collect()
    }
}

impl OfficialCandleCoverageGapMap {
    pub fn build(config: &OfficialCandleGapConfig) -> Result<Self, String> {
        let inputs = load_gap_inputs(config)?;
        Ok(build_gap_map_from_inputs(
            config,
            &inputs.rows,
            &inputs.pack,
        ))
    }

    pub fn from_json_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        serde_json::from_str(&text).map_err(|err| err.to_string())
    }

    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn fingerprint(&self) -> String {
        let fingerprint_payload = serde_json::json!({
            "cells": self.cells,
            "total_gaps": self.total_gaps,
            "official_gap_count": self.official_gap_count,
            "non_crypto_official_gap_count": self.non_crypto_official_gap_count,
            "crypto_gap_count": self.crypto_gap_count,
            "diagnostic_gap_count": self.diagnostic_gap_count,
            "research_only_gap_count": self.research_only_gap_count,
            "fixture_gap_count": self.fixture_gap_count,
            "buildable_gap_count": self.buildable_gap_count,
            "operator_action_gap_count": self.operator_action_gap_count,
            "gap_status": self.gap_status,
            "warnings": self.warnings,
            "reason_codes": self.reason_codes,
        });
        stable_hash_string(
            &serde_json::to_string(&fingerprint_payload)
                .unwrap_or_else(|_| self.cells.len().to_string()),
        )
    }

    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("gap_id={}", self.gap_id),
            format!("total_gaps={}", self.total_gaps),
            format!("official_gap_count={}", self.official_gap_count),
            format!(
                "non_crypto_official_gap_count={}",
                self.non_crypto_official_gap_count
            ),
            format!("crypto_gap_count={}", self.crypto_gap_count),
            format!("diagnostic_gap_count={}", self.diagnostic_gap_count),
            format!("research_only_gap_count={}", self.research_only_gap_count),
            format!("fixture_gap_count={}", self.fixture_gap_count),
            format!("buildable_gap_count={}", self.buildable_gap_count),
            format!(
                "operator_action_gap_count={}",
                self.operator_action_gap_count
            ),
            format!("gap_status={:?}", self.gap_status),
            format!("warnings={}", self.warnings.join(" | ")),
            format!("fingerprint={}", self.fingerprint()),
        ];
        lines.extend(self.cells.iter().map(|cell| {
            format!(
                "market={:?};symbol={};timeframe={};horizon_bars={};source_class={:?};row_count_impacted={};missing_future_bars={};gap_kinds={};buildable_from_existing_local_csv={};buildable_from_provider_collection={};requires_operator_action={};related_artifact_paths={}",
                cell.market,
                cell.symbol,
                cell.timeframe,
                cell.horizon_bars,
                cell.source_class,
                cell.row_count_impacted,
                cell.missing_future_bars,
                cell.gap_kinds.iter().map(|kind| format!("{kind:?}")).collect::<Vec<_>>().join("|"),
                cell.buildable_from_existing_local_csv,
                cell.buildable_from_provider_collection,
                cell.requires_operator_action,
                cell.related_artifact_paths.join("|"),
            )
        }));
        lines.join("\n")
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        fs::write(output_dir.join("candle_gap_map.txt"), self.to_text())
            .map_err(|err| err.to_string())?;
        let json_path = output_dir.join("official_candle_gap_map.json");
        fs::write(&json_path, self.to_json_string()?).map_err(|err| err.to_string())?;
        Ok(json_path)
    }
}

pub fn load_gap_map_from_path_or_config(
    path: &str,
) -> Result<OfficialCandleCoverageGapMap, String> {
    if path.ends_with(".json") {
        OfficialCandleCoverageGapMap::from_json_path(Path::new(path))
    } else {
        OfficialCandleGapConfig::from_toml_path(Path::new(path))
            .and_then(|config| OfficialCandleCoverageGapMap::build(&config))
    }
}

pub fn load_gap_inputs(
    config: &OfficialCandleGapConfig,
) -> Result<OfficialCandleGapInputs, String> {
    config.validate()?;
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
            .then(left.timestamp_ms.cmp(&right.timestamp_ms))
            .then(left.timeframe.cmp(&right.timeframe))
    });
    let mut canonical_csv_paths = Vec::new();
    let mut provenance_paths = Vec::new();
    let mut preflight_paths = Vec::new();
    let mut manifest_paths = Vec::new();
    let mut packs = Vec::new();
    for path in &config.candle_coverage_pack_paths {
        if path.ends_with(".json") {
            let pack = load_pack_from_path_or_config(path)?;
            collect_pack_sidecars(
                &pack,
                &mut canonical_csv_paths,
                &mut provenance_paths,
                &mut preflight_paths,
                &mut manifest_paths,
            );
            packs.push(pack);
            continue;
        }
        let pack_config = OfficialCandleCoveragePackConfig::from_toml_path(Path::new(path))?;
        canonical_csv_paths.extend(pack_config.canonical_csv_paths.iter().cloned());
        provenance_paths.extend(pack_config.provenance_paths.iter().cloned());
        preflight_paths.extend(pack_config.preflight_report_paths.iter().cloned());
        manifest_paths.extend(pack_config.manifest_paths.iter().cloned());
        packs.push(build_gap_input_pack(&pack_config)?);
    }
    canonical_csv_paths.sort();
    canonical_csv_paths.dedup();
    provenance_paths.sort();
    provenance_paths.dedup();
    preflight_paths.sort();
    preflight_paths.dedup();
    manifest_paths.sort();
    manifest_paths.dedup();
    let pack = merge_official_candle_packs(&config.gap_id, packs)?;
    Ok(OfficialCandleGapInputs {
        rows,
        pack,
        canonical_csv_paths,
        provenance_paths,
        preflight_paths,
        manifest_paths,
    })
}

fn build_gap_input_pack(
    config: &OfficialCandleCoveragePackConfig,
) -> Result<OfficialCandleCoveragePack, String> {
    let mut relaxed = config.clone();
    relaxed.require_official_source = false;
    relaxed.require_provenance = false;
    relaxed.require_preflight = false;
    relaxed.require_manifest = false;
    relaxed.allow_crypto_only = true;
    relaxed.allow_controlled_fixture = true;
    relaxed.allow_yfinance_research = true;
    relaxed.allow_fixture = true;
    relaxed.allow_synthetic_test = true;
    OfficialCandleCoveragePack::build(&relaxed)
}

fn collect_pack_sidecars(
    pack: &OfficialCandleCoveragePack,
    canonical_csv_paths: &mut Vec<String>,
    provenance_paths: &mut Vec<String>,
    preflight_paths: &mut Vec<String>,
    manifest_paths: &mut Vec<String>,
) {
    for descriptor in &pack.descriptors {
        canonical_csv_paths.push(descriptor.path.clone());
        if descriptor.provenance_available {
            if let Some(path) = discover_sibling_sidecar(&descriptor.path, "_provenance.json") {
                provenance_paths.push(path);
            }
        }
        if descriptor.preflight_ready {
            if let Some(path) = discover_sibling_sidecar(&descriptor.path, "_preflight.json") {
                preflight_paths.push(path);
            }
        }
        if descriptor.manifest_available {
            if let Some(path) = discover_sibling_sidecar(&descriptor.path, "_manifest.json") {
                manifest_paths.push(path);
            }
        }
    }
}

fn discover_sibling_sidecar(csv_path: &str, suffix: &str) -> Option<String> {
    let path = Path::new(csv_path);
    let stem = path.file_stem()?.to_str()?;
    let candidate = path.parent()?.join(format!("{stem}{suffix}"));
    candidate.exists().then(|| candidate.display().to_string())
}

pub fn build_gap_map_from_inputs(
    config: &OfficialCandleGapConfig,
    rows: &[ComparableCommitteeEvidenceRow],
    pack: &OfficialCandleCoveragePack,
) -> OfficialCandleCoverageGapMap {
    let filtered_rows = rows
        .iter()
        .filter(|row| row_allowed(row, config))
        .cloned()
        .collect::<Vec<_>>();
    let computation = build_candle_coverage_match_computation(
        &filtered_rows,
        pack,
        &CandleCoverageMatchOptions::default(),
    );
    let mut warnings = Vec::new();
    let mut grouped = BTreeMap::new();

    for row in &filtered_rows {
        let matched = computation
            .match_report
            .matches
            .iter()
            .find(|entry| entry.comparable_row_id.as_deref() == Some(row.row_id.as_str()));
        let candidate = select_symbol_candidate(row, pack);
        let gap_kinds = classify_gap_kinds(row, matched, candidate);
        if gap_kinds.is_empty() {
            continue;
        }
        let key = GapKey {
            market: row.market,
            symbol: row.symbol.clone(),
            normalized_symbol: normalize_symbol(&row.symbol),
            timeframe: row.timeframe.clone(),
            horizon_bars: row.horizon_bars,
            source_class: row.source_class,
            venue: candidate.and_then(|descriptor| descriptor.venue.clone()),
        };
        let accumulator = grouped.entry(key).or_insert_with(GapAccumulator::default);
        accumulator.row_count_impacted += 1;
        accumulator.comparable_rows_impacted += 1;
        accumulator.required_min_rows = accumulator.required_min_rows.max(row.horizon_bars + 1);
        accumulator.required_start_timestamp_ms = Some(
            accumulator
                .required_start_timestamp_ms
                .map(|current| current.min(row.timestamp_ms))
                .unwrap_or(row.timestamp_ms),
        );
        let required_end = estimate_required_end(row);
        accumulator.required_end_timestamp_ms = Some(
            accumulator
                .required_end_timestamp_ms
                .map(|current| current.max(required_end))
                .unwrap_or(required_end),
        );
        accumulator.missing_future_bars = accumulator
            .missing_future_bars
            .max(estimate_missing_future_bars(row, candidate, matched));
        accumulator.buildable_from_existing_local_csv |= candidate.is_some();
        accumulator.buildable_from_provider_collection |= provider_collection_possible(row, config);
        accumulator.requires_operator_action |=
            requires_operator_action(&gap_kinds, row, candidate);
        for kind in gap_kinds {
            accumulator.gap_kinds.insert(kind);
        }
        if let Some(descriptor) = candidate {
            accumulator
                .related_artifact_paths
                .insert(descriptor.path.clone());
        }
        for path in matching_related_paths(config, row, candidate) {
            accumulator.related_artifact_paths.insert(path);
        }
        accumulator.reason_codes = stable_reason_codes(
            &accumulator
                .reason_codes
                .iter()
                .cloned()
                .chain(reason_codes_for_row(row, matched, candidate))
                .collect::<Vec<_>>(),
        );
    }

    let mut cells = grouped
        .into_iter()
        .map(|(key, accumulator)| OfficialCandleGapCell {
            market: key.market,
            symbol: key.symbol,
            normalized_symbol: key.normalized_symbol,
            venue: key.venue,
            timeframe: key.timeframe,
            horizon_bars: key.horizon_bars,
            source_kind: Some(format!("{:?}", key.source_class)),
            source_class: key.source_class,
            row_count_impacted: accumulator.row_count_impacted,
            comparable_rows_impacted: accumulator.comparable_rows_impacted,
            missing_future_bars: accumulator.missing_future_bars,
            required_start_timestamp_ms: accumulator.required_start_timestamp_ms,
            required_end_timestamp_ms: accumulator.required_end_timestamp_ms,
            required_min_rows: accumulator.required_min_rows,
            gap_kinds: accumulator.gap_kinds.into_iter().collect(),
            buildable_from_existing_local_csv: accumulator.buildable_from_existing_local_csv,
            buildable_from_provider_collection: accumulator.buildable_from_provider_collection,
            requires_operator_action: accumulator.requires_operator_action,
            related_artifact_paths: accumulator.related_artifact_paths.into_iter().collect(),
            reason_codes: accumulator.reason_codes,
        })
        .collect::<Vec<_>>();
    cells.sort_by_key(cell_sort_key);
    if cells.len() > config.max_gaps {
        warnings.push(format!(
            "truncated_gap_cells={};max_gaps={}",
            cells.len() - config.max_gaps,
            config.max_gaps
        ));
        cells.truncate(config.max_gaps);
    }

    let total_gaps = cells.len();
    let official_gap_count = cells.iter().filter(|cell| official_gap(cell)).count();
    let non_crypto_official_gap_count = cells
        .iter()
        .filter(|cell| cell.source_class == ComparableEvidenceSourceClass::OfficialNonCrypto)
        .count();
    let crypto_gap_count = cells
        .iter()
        .filter(|cell| {
            cell.source_class == ComparableEvidenceSourceClass::OfficialCryptoOnly
                || cell
                    .gap_kinds
                    .contains(&OfficialCandleGapKind::CryptoOnlySource)
        })
        .count();
    let diagnostic_gap_count = cells
        .iter()
        .filter(|cell| {
            cell.source_class == ComparableEvidenceSourceClass::ControlledDiagnostic
                || cell
                    .gap_kinds
                    .contains(&OfficialCandleGapKind::ControlledOnlySource)
        })
        .count();
    let research_only_gap_count = cells
        .iter()
        .filter(|cell| {
            cell.source_class == ComparableEvidenceSourceClass::YFinanceResearch
                || cell
                    .gap_kinds
                    .contains(&OfficialCandleGapKind::ResearchOnlySource)
        })
        .count();
    let fixture_gap_count = cells
        .iter()
        .filter(|cell| {
            matches!(
                cell.source_class,
                ComparableEvidenceSourceClass::FixtureArchitectureTest
                    | ComparableEvidenceSourceClass::SyntheticTest
            ) || cell
                .gap_kinds
                .contains(&OfficialCandleGapKind::FixtureOnlySource)
        })
        .count();
    let buildable_gap_count = cells
        .iter()
        .filter(|cell| {
            cell.buildable_from_existing_local_csv || cell.buildable_from_provider_collection
        })
        .count();
    let operator_action_gap_count = cells
        .iter()
        .filter(|cell| cell.requires_operator_action)
        .count();
    let gap_status = determine_gap_status(&cells);

    OfficialCandleCoverageGapMap {
        gap_id: config.gap_id.clone(),
        cells,
        total_gaps,
        official_gap_count,
        non_crypto_official_gap_count,
        crypto_gap_count,
        diagnostic_gap_count,
        research_only_gap_count,
        fixture_gap_count,
        buildable_gap_count,
        operator_action_gap_count,
        gap_status,
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
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct GapKey {
    market: ProviderMarket,
    symbol: String,
    normalized_symbol: String,
    venue: Option<String>,
    timeframe: String,
    horizon_bars: usize,
    source_class: ComparableEvidenceSourceClass,
}

fn row_allowed(row: &ComparableCommitteeEvidenceRow, config: &OfficialCandleGapConfig) -> bool {
    (config.target_markets.is_empty() || config.target_markets.contains(&row.market))
        && (config.target_symbols.is_empty()
            || config
                .target_symbols
                .iter()
                .any(|symbol| normalize_symbol(symbol) == normalize_symbol(&row.symbol)))
        && (config.target_timeframes.is_empty()
            || config.target_timeframes.contains(&row.timeframe))
        && (config.target_horizons_bars.is_empty()
            || config.target_horizons_bars.contains(&row.horizon_bars))
}

fn select_symbol_candidate<'a>(
    row: &ComparableCommitteeEvidenceRow,
    pack: &'a OfficialCandleCoveragePack,
) -> Option<&'a OfficialCandleSeriesDescriptor> {
    let normalized_symbol = normalize_symbol(&row.symbol);
    pack.descriptors
        .iter()
        .filter(|descriptor| descriptor.normalized_symbol == normalized_symbol)
        .min_by(|left, right| descriptor_sort_key(row, left).cmp(&descriptor_sort_key(row, right)))
}

fn descriptor_sort_key(
    row: &ComparableCommitteeEvidenceRow,
    descriptor: &OfficialCandleSeriesDescriptor,
) -> (usize, usize, String) {
    let class_rank = match descriptor.source_class {
        OfficialCandleSeriesSourceClass::OfficialNonCrypto => 0,
        OfficialCandleSeriesSourceClass::OfficialCryptoOnly => 1,
        OfficialCandleSeriesSourceClass::ControlledDiagnostic => 2,
        OfficialCandleSeriesSourceClass::YFinanceResearch => 3,
        OfficialCandleSeriesSourceClass::FixtureArchitectureTest => 4,
        OfficialCandleSeriesSourceClass::SyntheticTest => 5,
        OfficialCandleSeriesSourceClass::Unknown => 6,
    };
    let timeframe_penalty = usize::from(descriptor.timeframe != row.timeframe);
    (class_rank, timeframe_penalty, descriptor.path.clone())
}

fn classify_gap_kinds(
    row: &ComparableCommitteeEvidenceRow,
    matched: Option<&CandleCoverageMatch>,
    candidate: Option<&OfficialCandleSeriesDescriptor>,
) -> Vec<OfficialCandleGapKind> {
    let mut kinds = BTreeSet::new();
    if row.summary_derived {
        kinds.insert(OfficialCandleGapKind::SummaryDerivedOnly);
    }
    match row.source_class {
        ComparableEvidenceSourceClass::YFinanceResearch => {
            kinds.insert(OfficialCandleGapKind::ResearchOnlySource);
        }
        ComparableEvidenceSourceClass::ControlledDiagnostic => {
            kinds.insert(OfficialCandleGapKind::ControlledOnlySource);
        }
        ComparableEvidenceSourceClass::FixtureArchitectureTest
        | ComparableEvidenceSourceClass::SyntheticTest => {
            kinds.insert(OfficialCandleGapKind::FixtureOnlySource);
        }
        ComparableEvidenceSourceClass::OfficialCryptoOnly => {
            kinds.insert(OfficialCandleGapKind::CryptoOnlySource);
        }
        _ => {}
    }
    if let Some(descriptor) = candidate {
        if !descriptor.provenance_available {
            kinds.insert(OfficialCandleGapKind::MissingProvenance);
        }
        if !descriptor.preflight_ready {
            kinds.insert(OfficialCandleGapKind::MissingPreflight);
        }
        if descriptor.has_duplicates {
            kinds.insert(OfficialCandleGapKind::DuplicateTimestamp);
        }
        if descriptor.has_gaps {
            kinds.insert(OfficialCandleGapKind::GapHeavy);
        }
    }
    let Some(matched) = matched else {
        kinds.insert(OfficialCandleGapKind::MissingCandleSeries);
        match row.source_class {
            ComparableEvidenceSourceClass::OfficialNonCrypto => {
                kinds.insert(OfficialCandleGapKind::MissingOfficialCandleSeries);
                kinds.insert(OfficialCandleGapKind::MissingNonCryptoOfficialCandleSeries);
            }
            ComparableEvidenceSourceClass::OfficialCryptoOnly => {
                kinds.insert(OfficialCandleGapKind::MissingOfficialCandleSeries);
            }
            _ => {}
        }
        return kinds.into_iter().collect();
    };
    if matches!(
        matched.timeframe_alignment_status,
        super::timeframe_alignment::TimeframeAlignmentStatus::CompatibleDownsampleDiagnosticOnly
            | super::timeframe_alignment::TimeframeAlignmentStatus::IncompatibleUpsample
            | super::timeframe_alignment::TimeframeAlignmentStatus::IncompatibleMixedGranularity
            | super::timeframe_alignment::TimeframeAlignmentStatus::MissingScenarioTimeframe
            | super::timeframe_alignment::TimeframeAlignmentStatus::MissingCandleTimeframe
            | super::timeframe_alignment::TimeframeAlignmentStatus::Unknown
    ) {
        kinds.insert(OfficialCandleGapKind::TimeframeMismatch);
        kinds.insert(OfficialCandleGapKind::MissingTimeframe);
    }
    match matched.timestamp_alignment_status {
        TimestampAlignmentV2Status::RejectedNoLookahead => {
            kinds.insert(OfficialCandleGapKind::NoLookaheadViolation);
        }
        TimestampAlignmentV2Status::InsufficientFutureWindow => {
            kinds.insert(OfficialCandleGapKind::MissingFutureWindow);
        }
        TimestampAlignmentV2Status::DuplicateTimestamp => {
            kinds.insert(OfficialCandleGapKind::TimestampMismatch);
            kinds.insert(OfficialCandleGapKind::DuplicateTimestamp);
        }
        TimestampAlignmentV2Status::MissingTimestamp
        | TimestampAlignmentV2Status::OutsideCandleRange
        | TimestampAlignmentV2Status::GapBeforeTimestamp
        | TimestampAlignmentV2Status::GapAfterTimestamp
        | TimestampAlignmentV2Status::BadDataQuality
        | TimestampAlignmentV2Status::Unknown => {
            kinds.insert(OfficialCandleGapKind::TimestampMismatch);
        }
        _ => {}
    }
    match matched.match_status {
        CandleCoverageMatchStatus::Matched => {
            if row.summary_derived {
                kinds.insert(OfficialCandleGapKind::SummaryDerivedOnly);
            }
        }
        CandleCoverageMatchStatus::MatchedDiagnosticOnly => {
            if row.source_class == ComparableEvidenceSourceClass::OfficialCryptoOnly {
                kinds.insert(OfficialCandleGapKind::CryptoOnlySource);
            } else if row.source_class == ComparableEvidenceSourceClass::ControlledDiagnostic {
                kinds.insert(OfficialCandleGapKind::ControlledOnlySource);
            } else {
                kinds.insert(OfficialCandleGapKind::SourceNotEligible);
            }
        }
        CandleCoverageMatchStatus::NoMatchingSeries => {
            kinds.insert(OfficialCandleGapKind::MissingCandleSeries);
            match row.source_class {
                ComparableEvidenceSourceClass::OfficialNonCrypto => {
                    kinds.insert(OfficialCandleGapKind::MissingOfficialCandleSeries);
                    kinds.insert(OfficialCandleGapKind::MissingNonCryptoOfficialCandleSeries);
                }
                ComparableEvidenceSourceClass::OfficialCryptoOnly => {
                    kinds.insert(OfficialCandleGapKind::MissingOfficialCandleSeries);
                }
                _ => {}
            }
        }
        CandleCoverageMatchStatus::TimeframeMismatch => {
            kinds.insert(OfficialCandleGapKind::TimeframeMismatch);
            kinds.insert(OfficialCandleGapKind::MissingTimeframe);
        }
        CandleCoverageMatchStatus::TimestampMismatch => {
            kinds.insert(OfficialCandleGapKind::TimestampMismatch);
            if matched.timestamp_alignment_status == TimestampAlignmentV2Status::DuplicateTimestamp
            {
                kinds.insert(OfficialCandleGapKind::DuplicateTimestamp);
            }
        }
        CandleCoverageMatchStatus::InsufficientFutureWindow => {
            kinds.insert(OfficialCandleGapKind::MissingFutureWindow);
        }
        CandleCoverageMatchStatus::SourceNotEligible => {
            kinds.insert(OfficialCandleGapKind::SourceNotEligible);
        }
        CandleCoverageMatchStatus::PreflightMissing => {
            kinds.insert(OfficialCandleGapKind::MissingPreflight);
        }
        CandleCoverageMatchStatus::ProvenanceMissing => {
            kinds.insert(OfficialCandleGapKind::MissingProvenance);
        }
        CandleCoverageMatchStatus::NoLookaheadRejected => {
            kinds.insert(OfficialCandleGapKind::NoLookaheadViolation);
        }
        CandleCoverageMatchStatus::Unknown => {
            kinds.insert(OfficialCandleGapKind::MissingCandleSeries);
        }
    }
    kinds.into_iter().collect()
}

fn estimate_required_end(row: &ComparableCommitteeEvidenceRow) -> u64 {
    timeframe_seconds(&row.timeframe)
        .map(|seconds| {
            row.timestamp_ms
                .saturating_add(seconds.saturating_mul(1000) * row.horizon_bars as u64)
        })
        .unwrap_or(row.timestamp_ms)
}

fn estimate_missing_future_bars(
    row: &ComparableCommitteeEvidenceRow,
    candidate: Option<&OfficialCandleSeriesDescriptor>,
    matched: Option<&CandleCoverageMatch>,
) -> usize {
    if matched.is_some_and(|entry| {
        entry.match_status != CandleCoverageMatchStatus::InsufficientFutureWindow
    }) {
        return 0;
    }
    let Some(descriptor) = candidate else {
        return row.horizon_bars;
    };
    let Some(step_ms) =
        timeframe_seconds(&row.timeframe).map(|seconds| seconds.saturating_mul(1000))
    else {
        return row.horizon_bars;
    };
    let required_end = estimate_required_end(row);
    if descriptor.timestamp_end_ms >= required_end {
        return 0;
    }
    let deficit = required_end.saturating_sub(descriptor.timestamp_end_ms);
    ((deficit + step_ms - 1) / step_ms) as usize
}

fn provider_collection_possible(
    row: &ComparableCommitteeEvidenceRow,
    config: &OfficialCandleGapConfig,
) -> bool {
    match row.source_class {
        ComparableEvidenceSourceClass::OfficialNonCrypto => true,
        ComparableEvidenceSourceClass::OfficialCryptoOnly => config.allow_crypto_only,
        ComparableEvidenceSourceClass::ControlledDiagnostic => config.allow_controlled_diagnostic,
        ComparableEvidenceSourceClass::YFinanceResearch => config.allow_yfinance_research,
        ComparableEvidenceSourceClass::FixtureArchitectureTest
        | ComparableEvidenceSourceClass::SyntheticTest => config.allow_fixture,
        ComparableEvidenceSourceClass::Unknown => false,
    }
}

fn requires_operator_action(
    kinds: &[OfficialCandleGapKind],
    row: &ComparableCommitteeEvidenceRow,
    candidate: Option<&OfficialCandleSeriesDescriptor>,
) -> bool {
    kinds.contains(&OfficialCandleGapKind::MissingProvenance)
        || kinds.contains(&OfficialCandleGapKind::MissingPreflight)
        || kinds.contains(&OfficialCandleGapKind::MissingCandleSeries)
        || kinds.contains(&OfficialCandleGapKind::MissingOfficialCandleSeries)
        || kinds.contains(&OfficialCandleGapKind::MissingNonCryptoOfficialCandleSeries)
        || candidate.is_none()
        || row.summary_derived
}

fn matching_related_paths(
    config: &OfficialCandleGapConfig,
    row: &ComparableCommitteeEvidenceRow,
    candidate: Option<&OfficialCandleSeriesDescriptor>,
) -> Vec<String> {
    let normalized_symbol = normalize_symbol(&row.symbol);
    let mut paths = config
        .official_replication_report_paths
        .iter()
        .chain(config.candle_coverage_closure_paths.iter())
        .chain(config.core_scorecard_paths.iter())
        .chain(config.official_committee_benchmark_paths.iter())
        .chain(config.outcome_coverage_bundle_paths.iter())
        .chain(config.reference_pack_bundle_paths.iter())
        .filter(|path| {
            let lowercase = path.to_ascii_lowercase();
            lowercase.contains(&normalized_symbol.to_ascii_lowercase())
                || lowercase.contains(&row.timeframe.to_ascii_lowercase())
        })
        .cloned()
        .collect::<Vec<_>>();
    if let Some(descriptor) = candidate {
        paths.push(descriptor.path.clone());
    }
    paths.sort();
    paths.dedup();
    paths.into_iter().take(8).collect()
}

fn reason_codes_for_row(
    row: &ComparableCommitteeEvidenceRow,
    matched: Option<&CandleCoverageMatch>,
    candidate: Option<&OfficialCandleSeriesDescriptor>,
) -> Vec<ReasonCode> {
    let mut reasons = row.reason_codes.clone();
    if row.summary_derived {}
    if matches!(
        row.source_class,
        ComparableEvidenceSourceClass::YFinanceResearch
    ) {
        reasons.push(ReasonCode::YFinanceResearchOnly);
    }
    if matches!(
        row.source_class,
        ComparableEvidenceSourceClass::ControlledDiagnostic
    ) {
        reasons.push(ReasonCode::ControlledOnlyEvidence);
    }
    if matches!(
        row.source_class,
        ComparableEvidenceSourceClass::OfficialCryptoOnly
    ) {
        reasons.push(ReasonCode::CryptoOnlyEvidence);
    }
    if candidate.is_none() {
        reasons.push(ReasonCode::MissingOfficialCandles);
    }
    if let Some(matched) = matched {
        reasons.extend(matched.reason_codes.clone());
        if matched.match_status == CandleCoverageMatchStatus::TimeframeMismatch {
            reasons.push(ReasonCode::UnsupportedTimeframe);
        }
        if matched.match_status == CandleCoverageMatchStatus::TimestampMismatch {
            reasons.push(ReasonCode::UnsupportedTimestampFormat);
        }
        if matched.match_status == CandleCoverageMatchStatus::NoLookaheadRejected {
            reasons.push(ReasonCode::RejectedNoLookaheadReference);
        }
    }
    if candidate.is_some_and(|descriptor| !descriptor.provenance_available) {
        reasons.push(ReasonCode::MissingOfficialProvenance);
    }
    if candidate.is_some_and(|descriptor| !descriptor.preflight_ready) {
        reasons.push(ReasonCode::MissingOfficialPreflight);
    }
    if candidate.is_some_and(|descriptor| descriptor.has_duplicates) {
        reasons.push(ReasonCode::DuplicateTimestampDetected);
    }
    stable_reason_codes(&reasons)
}

fn official_gap(cell: &OfficialCandleGapCell) -> bool {
    matches!(
        cell.source_class,
        ComparableEvidenceSourceClass::OfficialNonCrypto
            | ComparableEvidenceSourceClass::OfficialCryptoOnly
    ) && !cell
        .gap_kinds
        .contains(&OfficialCandleGapKind::ResearchOnlySource)
        && !cell
            .gap_kinds
            .contains(&OfficialCandleGapKind::FixtureOnlySource)
        && !cell
            .gap_kinds
            .contains(&OfficialCandleGapKind::ControlledOnlySource)
}

fn determine_gap_status(cells: &[OfficialCandleGapCell]) -> OfficialCandleGapStatus {
    if cells.is_empty() {
        return OfficialCandleGapStatus::NoGapsDetected;
    }
    if cells.iter().any(|cell| {
        cell.gap_kinds
            .contains(&OfficialCandleGapKind::MissingNonCryptoOfficialCandleSeries)
    }) {
        return OfficialCandleGapStatus::MissingNonCryptoOfficialCandles;
    }
    if cells.iter().any(|cell| {
        cell.gap_kinds
            .contains(&OfficialCandleGapKind::MissingOfficialCandleSeries)
            || cell
                .gap_kinds
                .contains(&OfficialCandleGapKind::MissingCandleSeries)
    }) {
        return OfficialCandleGapStatus::MissingOfficialCandles;
    }
    if cells.iter().any(|cell| {
        cell.gap_kinds
            .contains(&OfficialCandleGapKind::MissingFutureWindow)
    }) {
        return OfficialCandleGapStatus::MissingFutureWindows;
    }
    if cells.iter().any(|cell| {
        cell.gap_kinds
            .contains(&OfficialCandleGapKind::TimestampMismatch)
            || cell
                .gap_kinds
                .contains(&OfficialCandleGapKind::DuplicateTimestamp)
            || cell
                .gap_kinds
                .contains(&OfficialCandleGapKind::NoLookaheadViolation)
    }) {
        return OfficialCandleGapStatus::TimestampAlignmentWeak;
    }
    if cells.iter().any(|cell| {
        cell.gap_kinds
            .contains(&OfficialCandleGapKind::TimeframeMismatch)
            || cell
                .gap_kinds
                .contains(&OfficialCandleGapKind::MissingTimeframe)
    }) {
        return OfficialCandleGapStatus::TimeframeAlignmentWeak;
    }
    if cells.iter().all(|cell| {
        cell.gap_kinds
            .contains(&OfficialCandleGapKind::ResearchOnlySource)
            || cell
                .gap_kinds
                .contains(&OfficialCandleGapKind::FixtureOnlySource)
            || cell
                .gap_kinds
                .contains(&OfficialCandleGapKind::ControlledOnlySource)
            || cell
                .gap_kinds
                .contains(&OfficialCandleGapKind::CryptoOnlySource)
    }) {
        return OfficialCandleGapStatus::DiagnosticOnlyGaps;
    }
    if cells.iter().any(|cell| {
        cell.gap_kinds
            .contains(&OfficialCandleGapKind::SourceNotEligible)
    }) {
        return OfficialCandleGapStatus::SourceIneligible;
    }
    OfficialCandleGapStatus::NeedMoreEvidence
}

fn merge_official_candle_packs(
    pack_id: &str,
    packs: Vec<OfficialCandleCoveragePack>,
) -> Result<OfficialCandleCoveragePack, String> {
    if packs.is_empty() {
        return OfficialCandleCoveragePack::build(&OfficialCandleCoveragePackConfig {
            pack_id: format!("{pack_id}-empty-pack"),
            ..OfficialCandleCoveragePackConfig::default()
        });
    }
    let mut descriptors = packs
        .into_iter()
        .flat_map(|pack| pack.descriptors)
        .collect::<Vec<_>>();
    descriptors.sort_by(|left, right| {
        left.normalized_symbol
            .cmp(&right.normalized_symbol)
            .then(left.timeframe.cmp(&right.timeframe))
            .then(left.timestamp_start_ms.cmp(&right.timestamp_start_ms))
            .then(left.path.cmp(&right.path))
    });
    descriptors.dedup_by(|left, right| {
        left.normalized_symbol == right.normalized_symbol
            && left.timeframe == right.timeframe
            && left.path == right.path
    });
    let official_non_crypto_series = descriptors
        .iter()
        .filter(|descriptor| {
            descriptor.source_class == OfficialCandleSeriesSourceClass::OfficialNonCrypto
        })
        .cloned()
        .collect::<Vec<_>>();
    let official_crypto_series = descriptors
        .iter()
        .filter(|descriptor| {
            descriptor.source_class == OfficialCandleSeriesSourceClass::OfficialCryptoOnly
        })
        .cloned()
        .collect::<Vec<_>>();
    let controlled_series = descriptors
        .iter()
        .filter(|descriptor| {
            descriptor.source_class == OfficialCandleSeriesSourceClass::ControlledDiagnostic
        })
        .cloned()
        .collect::<Vec<_>>();
    let yfinance_series = descriptors
        .iter()
        .filter(|descriptor| {
            descriptor.source_class == OfficialCandleSeriesSourceClass::YFinanceResearch
        })
        .cloned()
        .collect::<Vec<_>>();
    let fixture_series = descriptors
        .iter()
        .filter(|descriptor| {
            matches!(
                descriptor.source_class,
                OfficialCandleSeriesSourceClass::FixtureArchitectureTest
                    | OfficialCandleSeriesSourceClass::SyntheticTest
            )
        })
        .cloned()
        .collect::<Vec<_>>();
    let unknown_series = descriptors
        .iter()
        .filter(|descriptor| descriptor.source_class == OfficialCandleSeriesSourceClass::Unknown)
        .cloned()
        .collect::<Vec<_>>();
    let total_rows = descriptors
        .iter()
        .map(|descriptor| descriptor.row_count)
        .sum();
    let storage_bytes = descriptors
        .iter()
        .map(|descriptor| descriptor.storage_bytes)
        .sum();
    let total_symbols = descriptors
        .iter()
        .map(|descriptor| descriptor.normalized_symbol.clone())
        .collect::<BTreeSet<_>>()
        .len();
    let total_timeframes = descriptors
        .iter()
        .map(|descriptor| descriptor.timeframe.clone())
        .collect::<BTreeSet<_>>()
        .len();
    let readiness_eligible_series_count = descriptors
        .iter()
        .filter(|descriptor| descriptor.official_readiness_eligible)
        .count();
    let benchmark_eligible_series_count = descriptors
        .iter()
        .filter(|descriptor| descriptor.benchmark_eligible)
        .count();
    Ok(OfficialCandleCoveragePack {
        pack_id: pack_id.to_string(),
        descriptors,
        official_non_crypto_series,
        official_crypto_series,
        controlled_series,
        yfinance_series,
        fixture_series,
        unknown_series,
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
    })
}

fn cell_sort_key(
    cell: &OfficialCandleGapCell,
) -> (
    ProviderMarket,
    String,
    String,
    usize,
    ComparableEvidenceSourceClass,
) {
    (
        cell.market,
        cell.normalized_symbol.clone(),
        cell.timeframe.clone(),
        cell.horizon_bars,
        cell.source_class,
    )
}

fn is_remote_path(value: &str) -> bool {
    value.contains("://")
}

fn default_output_root() -> String {
    "target/soma_official_candle_gap_map".to_string()
}

fn default_max_gaps() -> usize {
    100
}

fn default_max_symbols() -> usize {
    5
}

fn default_max_timeframes() -> usize {
    5
}

fn default_max_bytes() -> usize {
    5_000_000
}

fn default_true() -> bool {
    true
}
